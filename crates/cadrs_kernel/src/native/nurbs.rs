//! Rational B-spline curves and surfaces (Piegl & Tiller, "The NURBS Book").
//!
//! Control points are stored homogeneous: `[w·x, w·y, w·z, w]`. 2D curves (p-curves) use z = 0.

use super::math::*;
use nalgebra::{DMatrix, DVector, Vector4};

pub type H = Vector4<f64>;

pub fn hom(p: &P3, w: f64) -> H {
    H::new(p.x * w, p.y * w, p.z * w, w)
}

pub fn dehom(h: &H) -> P3 {
    P3::new(h.x / h.w, h.y / h.w, h.z / h.w)
}

/// The knot span index of `u` (A2.1): `knots[span] <= u < knots[span + 1]`, clamped to the last
/// non-empty span at the end.
pub fn find_span(n: usize, p: usize, u: f64, knots: &[f64]) -> usize {
    // n: number of control points − 1
    if u >= knots[n + 1] {
        // last non-degenerate span
        let mut k = n;
        while k > p && knots[k] >= knots[n + 1] {
            k -= 1;
        }
        return k;
    }
    if u <= knots[p] {
        let mut k = p;
        while k < n && knots[k + 1] <= knots[p] {
            k += 1;
        }
        return k;
    }
    let (mut lo, mut hi) = (p, n + 1);
    let mut mid = (lo + hi) / 2;
    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            hi = mid;
        } else {
            lo = mid;
        }
        mid = (lo + hi) / 2;
    }
    mid
}

/// The non-zero basis functions and their derivatives up to `d` at `u` (A2.3):
/// `ders[k][j]` is the k-th derivative of N_{span−p+j, p}.
pub fn basis_ders(span: usize, u: f64, p: usize, d: usize, knots: &[f64]) -> Vec<Vec<f64>> {
    let mut ndu = vec![vec![0.0; p + 1]; p + 1];
    let mut left = vec![0.0; p + 1];
    let mut right = vec![0.0; p + 1];
    ndu[0][0] = 1.0;
    for j in 1..=p {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;
        for r in 0..j {
            ndu[j][r] = right[r + 1] + left[j - r];
            let temp = if ndu[j][r] == 0.0 { 0.0 } else { ndu[r][j - 1] / ndu[j][r] };
            ndu[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        ndu[j][j] = saved;
    }
    let d = d.min(p);
    let mut ders = vec![vec![0.0; p + 1]; d + 1];
    for j in 0..=p {
        ders[0][j] = ndu[j][p];
    }
    let mut a = vec![vec![0.0; p + 1]; 2];
    for r in 0..=p {
        let (mut s1, mut s2) = (0usize, 1usize);
        a[0][0] = 1.0;
        for k in 1..=d {
            let mut dd = 0.0;
            let rk = r as isize - k as isize;
            let pk = p as isize - k as isize;
            if r >= k {
                a[s2][0] = a[s1][0] / ndu[(pk + 1) as usize][rk as usize];
                dd = a[s2][0] * ndu[rk as usize][pk as usize];
            }
            let j1 = if rk >= -1 { 1 } else { (-rk) as usize };
            let j2 = if (r as isize - 1) <= pk { k - 1 } else { p - r };
            for j in j1..=j2 {
                let idx = (rk + j as isize) as usize;
                a[s2][j] = (a[s1][j] - a[s1][j - 1]) / ndu[(pk + 1) as usize][idx];
                dd += a[s2][j] * ndu[idx][pk as usize];
            }
            if r as isize <= pk {
                a[s2][k] = -a[s1][k - 1] / ndu[(pk + 1) as usize][r];
                dd += a[s2][k] * ndu[r][pk as usize];
            }
            ders[k][r] = dd;
            std::mem::swap(&mut s1, &mut s2);
        }
    }
    let mut rr = p as f64;
    for k in 1..=d {
        for j in 0..=p {
            ders[k][j] *= rr;
        }
        rr *= (p - k) as f64;
    }
    // Pad the derivatives above the degree with zeros.
    ders
}

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    pub degree: usize,
    pub knots: Vec<f64>,
    pub cps: Vec<H>,
}

fn binom(n: usize, k: usize) -> f64 {
    let mut r = 1.0;
    for i in 0..k {
        r = r * (n - i) as f64 / (i + 1) as f64;
    }
    r
}

impl NurbsCurve {
    /// A (rational) Bézier curve from its poles and weights, on [0, 1].
    pub fn bezier(poles: &[P3], weights: &[f64]) -> Self {
        let p = poles.len() - 1;
        let mut knots = vec![0.0; p + 1];
        knots.extend(vec![1.0; p + 1]);
        Self { degree: p, knots, cps: poles.iter().zip(weights).map(|(q, w)| hom(q, *w)).collect() }
    }

    pub fn range(&self) -> (f64, f64) {
        (self.knots[self.degree], self.knots[self.cps.len()])
    }

    pub fn is_rational(&self) -> bool {
        let w0 = self.cps[0].w;
        self.cps.iter().any(|c| (c.w - w0).abs() > 1e-14)
    }

    /// Homogeneous derivatives up to `d` (A3.2).
    fn hders(&self, u: f64, d: usize) -> Vec<H> {
        let n = self.cps.len() - 1;
        let p = self.degree;
        let span = find_span(n, p, u, &self.knots);
        let nd = basis_ders(span, u, p, d, &self.knots);
        let mut out = vec![H::zeros(); d + 1];
        for k in 0..=d.min(p) {
            for j in 0..=p {
                out[k] += self.cps[span - p + j] * nd[k][j];
            }
        }
        out
    }

    /// Point and derivatives up to `d` of the rational curve (A4.2).
    pub fn ders(&self, u: f64, d: usize) -> Vec<V3> {
        let h = self.hders(u, d);
        let mut ck: Vec<V3> = Vec::with_capacity(d + 1);
        for k in 0..=d {
            let mut v = h[k].xyz();
            for i in 1..=k {
                v -= ck[k - i] * (binom(k, i) * h[i].w);
            }
            ck.push(v / h[0].w);
        }
        ck
    }

    pub fn point(&self, u: f64) -> P3 {
        let h = self.hders(u, 0);
        dehom(&h[0])
    }

    pub fn transformed(&self, m: &Affine) -> Self {
        Self {
            degree: self.degree,
            knots: self.knots.clone(),
            cps: self.cps.iter().map(|c| hom(&m.point(&dehom(c)), c.w)).collect(),
        }
    }

    /// The same curve run backwards, on the mirrored parameter range.
    pub fn reversed(&self) -> Self {
        let (a, b) = self.range();
        let _ = (a, b);
        let k0 = self.knots[0];
        let k1 = *self.knots.last().unwrap();
        Self {
            degree: self.degree,
            knots: self.knots.iter().rev().map(|k| k0 + k1 - k).collect(),
            cps: self.cps.iter().rev().copied().collect(),
        }
    }

    /// Interior knot values (distinct), for splitting quadrature.
    pub fn breaks(&self) -> Vec<f64> {
        let (a, b) = self.range();
        let mut v: Vec<f64> = vec![a];
        for &k in &self.knots {
            if k > a + 1e-14 && k < b - 1e-14 && (k - *v.last().unwrap()).abs() > 1e-14 {
                v.push(k);
            }
        }
        v.push(b);
        v
    }

    /// Re-parametrizes the knots linearly to [a, b].
    pub fn reparam(&self, a: f64, b: f64) -> Self {
        let (a0, b0) = self.range();
        let s = (b - a) / (b0 - a0);
        Self { degree: self.degree, knots: self.knots.iter().map(|k| a + (k - a0) * s).collect(), cps: self.cps.clone() }
    }

    /// Global interpolation (A9.1) of degree `p` through `pts` at parameters `params`, with
    /// optional end derivatives. Non-rational.
    pub fn interpolate(pts: &[P3], params: &[f64], p: usize, d0: Option<V3>, d1: Option<V3>) -> Self {
        // Parameters must increase strictly: drop points whose parameter repeats.
        let mut keep: Vec<usize> = Vec::with_capacity(pts.len());
        for i in 0..pts.len() {
            if keep.last().map_or(true, |&j| params[i] > params[j] + 1e-14 * (1.0 + params[j].abs())) {
                keep.push(i);
            }
        }
        if keep.len() < pts.len() {
            let pts2: Vec<P3> = keep.iter().map(|&i| pts[i]).collect();
            let par2: Vec<f64> = keep.iter().map(|&i| params[i]).collect();
            return Self::interpolate(&pts2, &par2, p, d0, d1);
        }
        if pts.len() == 1 {
            return Self { degree: 1, knots: vec![params[0], params[0], params[0] + 1.0, params[0] + 1.0], cps: vec![hom(&pts[0], 1.0), hom(&pts[0], 1.0)] };
        }
        match Self::try_interpolate(pts, params, p, d0, d1) {
            Some(c) => c,
            None if p > 1 || d0.is_some() || d1.is_some() => Self::interpolate(pts, params, 1, None, None),
            None => {
                // Degree one through the points always works.
                let mut knots = vec![params[0]];
                knots.extend_from_slice(params);
                knots.push(*params.last().unwrap());
                Self { degree: 1, knots, cps: pts.iter().map(|q| hom(q, 1.0)).collect() }
            }
        }
    }

    fn try_interpolate(pts: &[P3], params: &[f64], p: usize, d0: Option<V3>, d1: Option<V3>) -> Option<Self> {
        let n_pts = pts.len();
        let extra = d0.is_some() as usize + d1.is_some() as usize;
        let n = n_pts + extra; // number of control points
        let p = p.min(n - 1);
        // Knots by averaging (with derivative ends: Piegl & Tiller 9.2.2).
        let mut knots = vec![params[0]; p + 1];
        let m = n + p + 1;
        let inner = m - 2 * (p + 1);
        // Parameter list extended for derivative ends.
        let mut ext: Vec<f64> = Vec::new();
        if d0.is_some() {
            ext.push(params[0]);
        }
        ext.extend_from_slice(params);
        if d1.is_some() {
            ext.push(params[n_pts - 1]);
        }
        for j in 1..=inner {
            let s: f64 = (j..j + p).map(|i| ext[i.min(ext.len() - 1)]).sum();
            knots.push(s / p as f64);
        }
        knots.extend(vec![params[n_pts - 1]; p + 1]);
        let mut a = DMatrix::<f64>::zeros(n, n);
        let mut rhs = DMatrix::<f64>::zeros(n, 3);
        let mut row = 0;
        let put = |a: &mut DMatrix<f64>, row: usize, u: f64, d: usize| {
            let span = find_span(n - 1, p, u, &knots);
            let nd = basis_ders(span, u, p, d, &knots);
            for j in 0..=p {
                a[(row, span - p + j)] = nd[d][j];
            }
        };
        for (i, q) in pts.iter().enumerate() {
            put(&mut a, row, params[i], 0);
            rhs[(row, 0)] = q.x;
            rhs[(row, 1)] = q.y;
            rhs[(row, 2)] = q.z;
            row += 1;
            if i == 0 {
                if let Some(d) = d0 {
                    put(&mut a, row, params[0], 1);
                    rhs[(row, 0)] = d.x;
                    rhs[(row, 1)] = d.y;
                    rhs[(row, 2)] = d.z;
                    row += 1;
                }
            }
            if i == n_pts - 1 {
                if let Some(d) = d1 {
                    put(&mut a, row, params[n_pts - 1], 1);
                    rhs[(row, 0)] = d.x;
                    rhs[(row, 1)] = d.y;
                    rhs[(row, 2)] = d.z;
                    row += 1;
                }
            }
        }
        let sol = a.lu().solve(&rhs)?;
        if sol.iter().any(|x| !x.is_finite()) {
            return None;
        }
        let cps = (0..n).map(|i| H::new(sol[(i, 0)], sol[(i, 1)], sol[(i, 2)], 1.0)).collect();
        Some(Self { degree: p, knots, cps })
    }

    /// Interpolation (degree `p`) through points at parameters, with optional first and second
    /// derivatives at both ends. Non-rational. Falls back to fewer conditions when singular.
    pub fn interpolate_ext(pts: &[P3], params: &[f64], p: usize, d1: [Option<V3>; 2], d2: [Option<V3>; 2]) -> Self {
        if d2[0].is_none() && d2[1].is_none() {
            return Self::interpolate(pts, params, p, d1[0], d1[1]);
        }
        let n_pts = pts.len();
        let s0 = d1[0].is_some() as usize + d2[0].is_some() as usize;
        let s1 = d1[1].is_some() as usize + d2[1].is_some() as usize;
        let n = n_pts + s0 + s1;
        let p = p.min(n - 1).max(1);
        let mut ext: Vec<f64> = vec![params[0]; s0];
        ext.extend_from_slice(params);
        ext.extend(vec![params[n_pts - 1]; s1]);
        let mut knots = vec![params[0]; p + 1];
        let inner = n - p - 1;
        for j in 1..=inner {
            let s: f64 = (j..j + p).map(|i| ext[i.min(ext.len() - 1)]).sum();
            knots.push(s / p as f64);
        }
        knots.extend(vec![params[n_pts - 1]; p + 1]);
        let mut a = DMatrix::<f64>::zeros(n, n);
        let mut rhs = DMatrix::<f64>::zeros(n, 3);
        let mut row = 0;
        let mut put = |a: &mut DMatrix<f64>, rhs: &mut DMatrix<f64>, u: f64, d: usize, v: V3| {
            let span = find_span(n - 1, p, u, &knots);
            let nd = basis_ders(span, u, p, d, &knots);
            if d < nd.len() {
                for j in 0..=p {
                    a[(row, span - p + j)] = nd[d][j];
                }
            }
            rhs[(row, 0)] = v.x;
            rhs[(row, 1)] = v.y;
            rhs[(row, 2)] = v.z;
            row += 1;
        };
        for (i, q) in pts.iter().enumerate() {
            put(&mut a, &mut rhs, params[i], 0, q.coords);
            if i == 0 {
                if let Some(d) = d1[0] {
                    put(&mut a, &mut rhs, params[0], 1, d);
                }
                if let Some(d) = d2[0] {
                    put(&mut a, &mut rhs, params[0], 2, d);
                }
            }
            if i == n_pts - 1 {
                if let Some(d) = d1[1] {
                    put(&mut a, &mut rhs, params[n_pts - 1], 1, d);
                }
                if let Some(d) = d2[1] {
                    put(&mut a, &mut rhs, params[n_pts - 1], 2, d);
                }
            }
        }
        match a.lu().solve(&rhs) {
            Some(sol) if sol.iter().all(|x| x.is_finite()) => {
                let cps = (0..n).map(|i| H::new(sol[(i, 0)], sol[(i, 1)], sol[(i, 2)], 1.0)).collect();
                Self { degree: p, knots, cps }
            }
            _ => Self::interpolate(pts, params, p, d1[0], d1[1]),
        }
    }

    /// Chord-length parameters in [0, 1].
    pub fn chord_params(pts: &[P3]) -> Vec<f64> {
        let mut t = vec![0.0];
        let mut s = 0.0;
        for w in pts.windows(2) {
            s += (w[1] - w[0]).norm().sqrt().max(1e-12);
            t.push(s);
        }
        t.iter().map(|x| x / s).collect()
    }

    /// Splits at `u` into two curves (knot insertion to full multiplicity).
    pub fn split(&self, u: f64) -> (Self, Self) {
        let p = self.degree;
        let mut c = self.clone();
        let mult = c.knots.iter().filter(|k| (**k - u).abs() < 1e-14).count();
        for _ in mult..p + 1 {
            c = c.insert_knot(u);
        }
        let idx = c.knots.iter().position(|k| (*k - u).abs() < 1e-14).unwrap();
        // knots[idx..idx+p+1] == u
        let left_knots: Vec<f64> = c.knots[..idx + p + 1].to_vec();
        let mut lk = left_knots;
        lk.push(u);
        let left = Self { degree: p, knots: lk, cps: c.cps[..idx].to_vec() };
        let mut rk = vec![u];
        rk.extend_from_slice(&c.knots[idx..]);
        let right = Self { degree: p, knots: rk, cps: c.cps[idx - 1..].to_vec() };
        (left, right)
    }

    /// Inserts the knot `u` once (A5.1).
    pub fn insert_knot(&self, u: f64) -> Self {
        let p = self.degree;
        let n = self.cps.len() - 1;
        let k = find_span(n, p, u, &self.knots);
        let mut knots = self.knots[..=k].to_vec();
        knots.push(u);
        knots.extend_from_slice(&self.knots[k + 1..]);
        let mut cps = Vec::with_capacity(self.cps.len() + 1);
        for i in 0..=n + 1 {
            if i + p <= k {
                cps.push(self.cps[i]);
            } else if i > k {
                cps.push(self.cps[i - 1]);
            } else {
                let a = (u - self.knots[i]) / (self.knots[i + p] - self.knots[i]);
                cps.push(self.cps[i] * a + self.cps[i - 1] * (1.0 - a));
            }
        }
        Self { degree: p, knots, cps }
    }

    /// Degree elevation by repeated knot refinement is not needed yet; this raises the degree of
    /// a Bézier segment-wise curve by converting each span (only used for loft compatibility).
    pub fn elevate_bezier(&self) -> Self {
        // Valid for single-span (Bézier) curves only.
        let p = self.degree;
        let q = &self.cps;
        let mut out = Vec::with_capacity(p + 2);
        out.push(q[0]);
        for i in 1..=p {
            let a = i as f64 / (p + 1) as f64;
            out.push(q[i - 1] * a + q[i] * (1.0 - a));
        }
        out.push(q[p]);
        let (a, b) = self.range();
        let mut knots = vec![a; p + 2];
        knots.extend(vec![b; p + 2]);
        Self { degree: p + 1, knots, cps: out }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsSurface {
    pub du: usize,
    pub dv: usize,
    pub ku: Vec<f64>,
    pub kv: Vec<f64>,
    /// `cps[i][j]`: i along u, j along v.
    pub cps: Vec<Vec<H>>,
}

impl NurbsSurface {
    pub fn range(&self) -> ((f64, f64), (f64, f64)) {
        let nu = self.cps.len();
        let nv = self.cps[0].len();
        ((self.ku[self.du], self.ku[nu]), (self.kv[self.dv], self.kv[nv]))
    }

    /// Point and first and second derivatives: [S, Su, Sv, Suu, Suv, Svv] (A4.4).
    pub fn ders2(&self, u: f64, v: f64) -> [V3; 6] {
        let nu = self.cps.len() - 1;
        let nv = self.cps[0].len() - 1;
        let su = find_span(nu, self.du, u, &self.ku);
        let sv = find_span(nv, self.dv, v, &self.kv);
        let bu = basis_ders(su, u, self.du, 2, &self.ku);
        let bv = basis_ders(sv, v, self.dv, 2, &self.kv);
        // Homogeneous derivatives A[k][l].
        let mut a = [[H::zeros(); 3]; 3];
        for k in 0..=2usize.min(self.du) {
            let mut temp = vec![H::zeros(); self.dv + 1];
            for s in 0..=self.dv {
                for r in 0..=self.du {
                    temp[s] += self.cps[su - self.du + r][sv - self.dv + s] * bu[k][r];
                }
            }
            for l in 0..=(2 - k).min(self.dv) {
                for s in 0..=self.dv {
                    a[k][l] += temp[s] * bv[l][s];
                }
            }
        }
        // Rational: S_kl.
        let mut s = [[V3::zeros(); 3]; 3];
        for k in 0..=2 {
            for l in 0..=(2 - k) {
                let mut v = a[k][l].xyz();
                for j in 1..=l {
                    v -= s[k][l - j] * (binom(l, j) * a[0][j].w);
                }
                for i in 1..=k {
                    v -= s[k - i][l] * (binom(k, i) * a[i][0].w);
                    let mut v2 = V3::zeros();
                    for j in 1..=l {
                        v2 += s[k - i][l - j] * (binom(l, j) * a[i][j].w);
                    }
                    v -= v2 * binom(k, i);
                }
                s[k][l] = v / a[0][0].w;
            }
        }
        [s[0][0], s[1][0], s[0][1], s[2][0], s[1][1], s[0][2]]
    }

    pub fn point(&self, u: f64, v: f64) -> P3 {
        P3::from(self.ders2(u, v)[0])
    }

    pub fn transformed(&self, m: &Affine) -> Self {
        Self {
            du: self.du,
            dv: self.dv,
            ku: self.ku.clone(),
            kv: self.kv.clone(),
            cps: self.cps.iter().map(|row| row.iter().map(|c| hom(&m.point(&dehom(c)), c.w)).collect()).collect(),
        }
    }

    /// Distinct knot values in u and v inside the range.
    pub fn breaks(&self) -> (Vec<f64>, Vec<f64>) {
        let b = |k: &[f64], lo: f64, hi: f64| {
            let mut v = vec![lo];
            for &x in k {
                if x > lo + 1e-14 && x < hi - 1e-14 && (x - *v.last().unwrap()).abs() > 1e-14 {
                    v.push(x);
                }
            }
            v.push(hi);
            v
        };
        let ((u0, u1), (v0, v1)) = self.range();
        (b(&self.ku, u0, u1), b(&self.kv, v0, v1))
    }

    /// A ruled or interpolated surface through compatible curves (same degree and knots), the
    /// curves along u, interpolated along v at `params` with degree `q` (lofts).
    pub fn through_curves(curves: &[NurbsCurve], params: &[f64], q: usize, d0: Option<Vec<V3>>, d1: Option<Vec<V3>>) -> Self {
        let ncp = curves[0].cps.len();
        let mut cols: Vec<NurbsCurve> = Vec::new();
        for i in 0..ncp {
            let pts: Vec<P3> = curves.iter().map(|c| dehom(&c.cps[i])).collect();
            let a = d0.as_ref().map(|d| d[i]);
            let b = d1.as_ref().map(|d| d[i]);
            cols.push(NurbsCurve::interpolate(&pts, params, q, a, b));
        }
        let cps: Vec<Vec<H>> = (0..ncp).map(|i| cols[i].cps.clone()).collect();
        Self { du: curves[0].degree, dv: cols[0].degree, ku: curves[0].knots.clone(), kv: cols[0].knots.clone(), cps }
    }
}

/// Solves a dense square system (used by fits).
pub fn solve(a: DMatrix<f64>, b: DVector<f64>) -> Option<DVector<f64>> {
    a.lu().solve(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_circle_is_exact() {
        let w = std::f64::consts::FRAC_1_SQRT_2;
        let c = NurbsCurve::bezier(&[P3::new(1.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(0.0, 1.0, 0.0)], &[1.0, w, 1.0]);
        for i in 0..=10 {
            let p = c.point(i as f64 / 10.0);
            assert!((p.coords.norm() - 1.0).abs() < 1e-14);
        }
        let d = c.ders(0.0, 2);
        assert!(d[1].x.abs() < 1e-14 && d[1].y > 0.0);
    }

    #[test]
    fn interpolation_passes_through_points() {
        let pts: Vec<P3> = (0..7).map(|i| P3::new(i as f64, (i as f64).sin(), 0.0)).collect();
        let t = NurbsCurve::chord_params(&pts);
        let c = NurbsCurve::interpolate(&pts, &t, 3, None, None);
        for (p, u) in pts.iter().zip(&t) {
            assert!((c.point(*u) - p).norm() < 1e-12);
        }
        let c = NurbsCurve::interpolate(&pts, &t, 3, Some(V3::x()), Some(V3::y()));
        for (p, u) in pts.iter().zip(&t) {
            assert!((c.point(*u) - p).norm() < 1e-12);
        }
        assert!((c.ders(0.0, 1)[1] - V3::x()).norm() < 1e-10);
    }

    #[test]
    fn split_keeps_shape() {
        let c = NurbsCurve::bezier(&[P3::new(0.0, 0.0, 0.0), P3::new(0.0, 10.0, 0.0), P3::new(20.0, 10.0, 0.0), P3::new(20.0, 0.0, 0.0)], &[1.0; 4]);
        let (a, b) = c.split(0.3);
        assert!((a.point(0.15) - c.point(0.15)).norm() < 1e-12);
        assert!((b.point(0.8) - c.point(0.8)).norm() < 1e-12);
    }

    #[test]
    fn bilinear_surface_derivatives() {
        let s = NurbsSurface {
            du: 1,
            dv: 1,
            ku: vec![0.0, 0.0, 1.0, 1.0],
            kv: vec![0.0, 0.0, 1.0, 1.0],
            cps: vec![
                vec![hom(&P3::new(0.0, 0.0, 0.0), 1.0), hom(&P3::new(0.0, 1.0, 0.0), 1.0)],
                vec![hom(&P3::new(1.0, 0.0, 0.0), 1.0), hom(&P3::new(1.0, 1.0, 1.0), 1.0)],
            ],
        };
        let d = s.ders2(0.5, 0.5);
        assert!((d[0] - V3::new(0.5, 0.5, 0.25)).norm() < 1e-14);
        assert!((d[1] - V3::new(1.0, 0.0, 0.5)).norm() < 1e-14);
        assert!((d[4] - V3::new(0.0, 0.0, 1.0)).norm() < 1e-14);
    }
}
