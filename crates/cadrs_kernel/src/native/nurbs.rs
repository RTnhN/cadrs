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
    for (k, row) in ders.iter_mut().enumerate().skip(1) {
        for x in row.iter_mut() {
            *x *= rr;
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

/// Why a knot vector, degree and control points don't make a B-spline: `None` when they do.
/// `n` control points of degree `p` need `n + p + 1` finite non-decreasing knots, `n > p`, a
/// non-empty range `knots[p] < knots[n]`, and finite control points with positive weights.
/// (Knots repeated more than `p + 1` times are allowed: some files have them, and the
/// evaluators take them.)
fn knots_issue(p: usize, knots: &[f64], cps: &[H]) -> Option<String> {
    let n = cps.len();
    if p == 0 {
        return Some("a B-spline needs degree 1 or more".into());
    }
    if n <= p {
        return Some(format!("a degree {p} B-spline needs at least {} control points, not {n}", p + 1));
    }
    if knots.len() != n + p + 1 {
        return Some(format!("{n} control points of degree {p} need {} knots, not {}", n + p + 1, knots.len()));
    }
    if knots.iter().any(|k| !k.is_finite()) {
        return Some("a knot isn't a finite number".into());
    }
    if knots.windows(2).any(|w| w[1] < w[0]) {
        return Some("the knots decrease".into());
    }
    if knots[n] <= knots[p] {
        return Some("the knots leave an empty parameter range".into());
    }
    if cps.iter().any(|c| !c.iter().all(|x| x.is_finite())) {
        return Some("a control point isn't finite".into());
    }
    if let Some(c) = cps.iter().find(|c| c.w <= 0.0) {
        return Some(format!("a weight {} isn't positive", c.w));
    }
    None
}

impl NurbsCurve {
    /// A B-spline curve checked to be well formed (see [`NurbsCurve::validate`]): the constructor
    /// for data from outside the kernel (files, scripts), where the fields could disagree.
    pub fn new(degree: usize, knots: Vec<f64>, cps: Vec<H>) -> crate::Result<Self> {
        let c = Self { degree, knots, cps };
        c.validate()?;
        Ok(c)
    }

    /// Checks that the knots, degree and control points make a B-spline (the evaluators index
    /// by them unchecked): `n + p + 1` finite non-decreasing knots for `n > p` control points, a
    /// non-empty range, finite control points and positive weights.
    pub fn validate(&self) -> crate::Result<()> {
        match knots_issue(self.degree, &self.knots, &self.cps) {
            Some(why) => Err(crate::KernelError::InvalidInput(why)),
            None => Ok(()),
        }
    }

    /// A (rational) Bézier curve from its poles and weights, on [0, 1], checked: at least two
    /// poles, as many weights, all finite and the weights positive.
    pub fn try_bezier(poles: &[P3], weights: &[f64]) -> crate::Result<Self> {
        if poles.len() < 2 || weights.len() != poles.len() {
            return Err(crate::KernelError::InvalidInput(format!("a Bézier curve needs two or more poles and a weight for each ({} poles, {} weights)", poles.len(), weights.len())));
        }
        let c = Self::bezier(poles, weights);
        c.validate()?;
        Ok(c)
    }

    /// A (rational) Bézier curve from its poles and weights, on [0, 1]: for poles the kernel
    /// made itself; [`NurbsCurve::try_bezier`] checks poles from outside.
    ///
    /// # Panics
    /// With no poles, or fewer weights than poles.
    pub fn bezier(poles: &[P3], weights: &[f64]) -> Self {
        assert!(!poles.is_empty() && weights.len() >= poles.len(), "a Bézier curve needs poles and a weight for each");
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
    /// The closed curve of degree `p` through `pts` (the first not repeated at the end) at
    /// `params`, with `params[m]` (one more than the points) the parameter where it closes: a
    /// periodic B-spline (C^(p-1) at the closure too), returned clamped on
    /// [params[0], params[m]]. None when the system is singular.
    pub fn interpolate_periodic(pts: &[P3], params: &[f64], p: usize) -> Option<Self> {
        let m = pts.len();
        if m < 2 || params.len() != m + 1 || p == 0 {
            return None;
        }
        let period = params[m] - params[0];
        // The parameters repeated a period either way, as many as the degree needs.
        let t = |k: isize| -> f64 {
            let q = k.div_euclid(m as isize);
            let r = k.rem_euclid(m as isize) as usize;
            params[r] + q as f64 * period
        };
        let ncp = m + 2 * p;
        let knots: Vec<f64> = (0..ncp + p + 1).map(|i| t(i as isize - p as isize)).collect();
        let mut a = DMatrix::<f64>::zeros(m, m);
        for k in 0..m {
            let span = find_span(ncp - 1, p, params[k], &knots);
            let nb = basis_ders(span, params[k], p, 0, &knots);
            for j in 0..=p {
                a[(k, (span - p + j) % m)] += nb[0][j];
            }
        }
        let lu = a.lu();
        let mut sol = [DVector::<f64>::zeros(m), DVector::<f64>::zeros(m), DVector::<f64>::zeros(m)];
        for (c, out) in sol.iter_mut().enumerate() {
            let b = DVector::from_iterator(m, pts.iter().map(|q| q[c]));
            *out = lu.solve(&b)?;
        }
        let cps: Vec<H> = (0..ncp).map(|i| H::new(sol[0][i % m], sol[1][i % m], sol[2][i % m], 1.0)).collect();
        let mut c = Self { degree: p, knots, cps };
        // Clamp: knots of multiplicity p at both ends of the period, then cut.
        let (lo, hi) = (params[0], params[m]);
        let eps = 1e-12 * (1.0 + period.abs());
        let mult = |c: &Self, u: f64| c.knots.iter().filter(|k| (*k - u).abs() <= eps).count();
        while mult(&c, lo) < p {
            c = c.insert_knot(lo);
        }
        while mult(&c, hi) < p {
            c = c.insert_knot(hi);
        }
        let a = c.knots.iter().position(|k| (k - lo).abs() <= eps)?;
        let b = c.knots.iter().rposition(|k| (k - hi).abs() <= eps)?;
        let mut knots = vec![lo];
        knots.extend(c.knots[a..=b].iter().map(|k| k.clamp(lo, hi)));
        knots.push(hi);
        let cps = c.cps[a - 1..=b - p].to_vec();
        Some(Self { degree: p, knots, cps })
    }

    pub fn interpolate(pts: &[P3], params: &[f64], p: usize, d0: Option<V3>, d1: Option<V3>) -> Self {
        // Parameters must increase strictly: drop points whose parameter repeats.
        let mut keep: Vec<usize> = Vec::with_capacity(pts.len());
        for i in 0..pts.len() {
            if keep.last().is_none_or(|&j| params[i] > params[j] + 1e-14 * (1.0 + params[j].abs())) {
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
        // Large systems straight into band storage (a dense n × n matrix would not fit).
        let banded = n > 24;
        let bw = p + 2;
        let mut a = DMatrix::<f64>::zeros(if banded { 0 } else { n }, if banded { 0 } else { n });
        let mut band = Band::new(if banded { n } else { 0 }, bw);
        let mut outside = false;
        let mut rhs = DMatrix::<f64>::zeros(n, 3);
        let mut row = 0;
        let mut put = |a: &mut DMatrix<f64>, row: usize, u: f64, d: usize| {
            let span = find_span(n - 1, p, u, &knots);
            let nd = basis_ders(span, u, p, d, &knots);
            for j in 0..=p {
                if banded {
                    outside |= !band.set(row, span - p + j, nd[d][j]);
                } else {
                    a[(row, span - p + j)] = nd[d][j];
                }
            }
        };
        for (i, q) in pts.iter().enumerate() {
            put(&mut a, row, params[i], 0);
            rhs[(row, 0)] = q.x;
            rhs[(row, 1)] = q.y;
            rhs[(row, 2)] = q.z;
            row += 1;
            if i == 0
                && let Some(d) = d0 {
                    put(&mut a, row, params[0], 1);
                    rhs[(row, 0)] = d.x;
                    rhs[(row, 1)] = d.y;
                    rhs[(row, 2)] = d.z;
                    row += 1;
                }
            if i == n_pts - 1
                && let Some(d) = d1 {
                    put(&mut a, row, params[n_pts - 1], 1);
                    rhs[(row, 0)] = d.x;
                    rhs[(row, 1)] = d.y;
                    rhs[(row, 2)] = d.z;
                    row += 1;
                }
        }
        drop(put);
        let sol = if banded {
            if outside {
                return None;
            }
            band.solve(rhs)?
        } else {
            a.lu().solve(&rhs)?
        };
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
        // Large systems straight into band storage (a dense n × n matrix would not fit).
        let banded = n > 24;
        let bw = p + 2;
        let mut a = DMatrix::<f64>::zeros(if banded { 0 } else { n }, if banded { 0 } else { n });
        let mut band = Band::new(if banded { n } else { 0 }, bw);
        let mut outside = false;
        let mut rhs = DMatrix::<f64>::zeros(n, 3);
        let mut row = 0;
        let mut put = |a: &mut DMatrix<f64>, rhs: &mut DMatrix<f64>, u: f64, d: usize, v: V3| {
            let span = find_span(n - 1, p, u, &knots);
            let nd = basis_ders(span, u, p, d, &knots);
            if d < nd.len() {
                for j in 0..=p {
                    if banded {
                        outside |= !band.set(row, span - p + j, nd[d][j]);
                    } else {
                        a[(row, span - p + j)] = nd[d][j];
                    }
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
        drop(put);
        let sol = if banded {
            if outside { None } else { band.solve(rhs) }
        } else {
            a.lu().solve(&rhs)
        };
        match sol {
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
        // knots[idx..idx+p+1] == u: the left curve's poles end at idx − 1, the right's start at
        // idx (both the point at u).
        let left = Self { degree: p, knots: c.knots[..idx + p + 1].to_vec(), cps: c.cps[..idx].to_vec() };
        let right = Self { degree: p, knots: c.knots[idx..].to_vec(), cps: c.cps[idx..].to_vec() };
        (left, right)
    }

    /// This curve followed by `next` (which starts where this one ends, same degree): a C0
    /// joint, `next` reparametrised to carry on from this curve's end and its weights scaled
    /// to match at the joint.
    pub fn joined(&self, next: &Self) -> Self {
        let p = self.degree;
        let (_, end) = self.range();
        let (start, _) = next.range();
        let s = self.cps[self.cps.len() - 1].w / next.cps[0].w;
        let mut knots = self.knots[..self.knots.len() - 1].to_vec();
        knots.extend(next.knots[p + 1..].iter().map(|k| k - start + end));
        let mut cps = self.cps.clone();
        cps.extend(next.cps[1..].iter().map(|c| c * s));
        Self { degree: p, knots, cps }
    }

    /// A closed curve restarted at `u`: from `u` to its end, then from its start back to `u`.
    pub fn restarted(&self, u: f64) -> Self {
        let (a, b) = self.range();
        if u <= a || u >= b {
            return self.clone();
        }
        let (l, r) = self.split(u);
        r.joined(&l)
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

    /// The same curve one degree higher (exact): each span raised as a Bézier segment, the
    /// spans joined again at knots of full multiplicity.
    pub fn elevated(&self) -> Self {
        let p = self.degree;
        let (a, b) = self.range();
        let eps = 1e-12 * (1.0 + a.abs().max(b.abs()));
        let mut c = self.clone();
        let inner: Vec<f64> = {
            let mut v: Vec<f64> = Vec::new();
            for &k in &self.knots[p + 1..self.knots.len() - p - 1] {
                if v.last().is_none_or(|l| (k - l).abs() > eps) {
                    v.push(k);
                }
            }
            v
        };
        for &u in &inner {
            while c.knots.iter().filter(|k| (**k - u).abs() <= eps).count() < p {
                c = c.insert_knot(u);
            }
        }
        // Each span's poles end at its first knot's last occurrence.
        let starts: Vec<f64> = std::iter::once(a).chain(inner.iter().copied()).collect();
        let mut cps: Vec<H> = Vec::new();
        for (sgi, &u) in starts.iter().enumerate() {
            let r = c.knots.iter().rposition(|k| (*k - u).abs() <= eps).unwrap_or(p).min(c.cps.len() - 1).max(p);
            let seg = &c.cps[r - p..=r];
            let up: Vec<H> = (0..=p + 1)
                .map(|i| {
                    let t = i as f64 / (p + 1) as f64;
                    let prev = if i > 0 { seg[i - 1] * t } else { H::zeros() };
                    let cur = if i <= p { seg[i] * (1.0 - t) } else { H::zeros() };
                    prev + cur
                })
                .collect();
            cps.extend(if sgi == 0 { &up[..] } else { &up[1..] });
        }
        let mut knots = vec![a; p + 2];
        for &u in &inner {
            knots.extend(std::iter::repeat_n(u, p + 1));
        }
        knots.extend(std::iter::repeat_n(b, p + 2));
        Self { degree: p + 1, knots, cps }
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

impl NurbsCurve {
    /// The part of the curve over [a, b] (clamped to its range).
    pub fn restricted(&self, a: f64, b: f64) -> Self {
        let (lo, hi) = self.range();
        let (a, b) = (a.max(lo), b.min(hi));
        let mut c = self.clone();
        if a > lo && a < hi {
            c = c.split(a).1;
        }
        if b < hi && b > a {
            c = c.split(b).0;
        }
        c
    }
}

impl NurbsSurface {
    /// A B-spline surface checked to be well formed (see [`NurbsSurface::validate`]).
    pub fn new(du: usize, dv: usize, ku: Vec<f64>, kv: Vec<f64>, cps: Vec<Vec<H>>) -> crate::Result<Self> {
        let s = Self { du, dv, ku, kv, cps };
        s.validate()?;
        Ok(s)
    }

    /// Checks that the net is rectangular and each direction's knots, degree and control points
    /// make a B-spline (as [`NurbsCurve::validate`]).
    pub fn validate(&self) -> crate::Result<()> {
        let bad = |why: String| Err(crate::KernelError::InvalidInput(why));
        let Some(first) = self.cps.first().filter(|r| !r.is_empty()) else { return bad("a B-spline surface needs control points".into()) };
        if self.cps.iter().any(|r| r.len() != first.len()) {
            return bad("the B-spline surface's rows of control points differ in length".into());
        }
        let col: Vec<H> = self.cps.iter().map(|r| r[0]).collect();
        if let Some(why) = knots_issue(self.du, &self.ku, &col) {
            return bad(format!("along u: {why}"));
        }
        for r in &self.cps {
            if let Some(why) = knots_issue(self.dv, &self.kv, r) {
                return bad(format!("along v: {why}"));
            }
        }
        Ok(())
    }

    /// The control net of the patch over u × v (each row's and then each column's curve cut
    /// there): its points' hull holds that part of the surface.
    pub fn sub_net(&self, u: (f64, f64), v: (f64, f64)) -> Vec<Vec<H>> {
        let rows: Vec<Vec<H>> = self.cps.iter().map(|r| NurbsCurve { degree: self.dv, knots: self.kv.clone(), cps: r.clone() }.restricted(v.0, v.1).cps).collect();
        let nv = rows[0].len();
        let cols: Vec<Vec<H>> = (0..nv).map(|j| NurbsCurve { degree: self.du, knots: self.ku.clone(), cps: rows.iter().map(|r| r[j]).collect() }.restricted(u.0, u.1).cps).collect();
        cols
    }

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

/// Solves a banded system A·X = B (|i − j| ≤ `bw` outside the band is zero) by Gaussian
/// elimination with partial pivoting inside the band: O(n·bw²) instead of O(n³). B-spline
/// collocation matrices are banded (and totally positive).
pub fn banded_solve(a: &DMatrix<f64>, b: &DMatrix<f64>, bw: usize) -> Option<DMatrix<f64>> {
    let n = a.nrows();
    let mut band = Band::new(n, bw);
    for i in 0..n {
        for j in i.saturating_sub(bw)..(i + 2 * bw + 1).min(n) {
            band.set(i, j, a[(i, j)]);
        }
        // Entries just outside the stored band must be zero (B-spline collocation rows are
        // local; checking every entry would make the solve O(n²)).
        for j in [i.wrapping_sub(bw + 1), i + 2 * bw + 1] {
            if j < n && a[(i, j)] != 0.0 {
                return a.clone().lu().solve(b);
            }
        }
    }
    band.solve(b.clone())
}

/// A square matrix in band storage: row i keeps columns [i − bw, i + 2bw] (room for the
/// fill-in of partial pivoting).
pub struct Band {
    n: usize,
    bw: usize,
    band: Vec<f64>,
}

impl Band {
    pub fn new(n: usize, bw: usize) -> Self {
        Self { n, bw, band: vec![0.0; n * (3 * bw + 1)] }
    }
    fn at(&self, i: usize, j: usize) -> Option<usize> {
        let w = 3 * self.bw + 1;
        let k = j as isize + self.bw as isize - i as isize;
        (k >= 0 && (k as usize) < w).then(|| i * w + k as usize)
    }
    /// Sets an entry; false when it lies outside the band.
    pub fn set(&mut self, i: usize, j: usize, v: f64) -> bool {
        match self.at(i, j) {
            Some(k) => {
                self.band[k] = v;
                true
            }
            None => v == 0.0,
        }
    }
    /// Solves A·X = B by Gaussian elimination with partial pivoting inside the band.
    pub fn solve(self, b: DMatrix<f64>) -> Option<DMatrix<f64>> {
        let (n, bw) = (self.n, self.bw);
        let m = b.ncols();
        let Band { band: mut band_v, .. } = self;
        let me = Band { n, bw, band: Vec::new() };
        let at = |i: usize, j: usize| me.at(i, j);
        let band = &mut band_v;
    let mut x = b.clone();
    for c in 0..n {
        // Pivot among rows c..c+bw.
        let last = (c + bw + 1).min(n);
        let mut piv = c;
        let mut best = 0.0;
        for r in c..last {
            let v = at(r, c).map(|k| band[k].abs()).unwrap_or(0.0);
            if v > best {
                best = v;
                piv = r;
            }
        }
        if best < 1e-300 {
            return None;
        }
        if piv != c {
            for j in c.saturating_sub(bw)..(c + 2 * bw + 1).min(n) {
                let (ka, kb) = (at(c, j), at(piv, j));
                match (ka, kb) {
                    (Some(ka), Some(kb)) => band.swap(ka, kb),
                    (Some(ka), None) => band[ka] = 0.0,
                    (None, Some(kb)) => {
                        if band[kb] != 0.0 {
                            return None;
                        }
                    }
                    (None, None) => {}
                }
            }
            x.swap_rows(c, piv);
        }
        let d = band[at(c, c).unwrap()];
        for r in c + 1..last {
            let Some(krc) = at(r, c) else { continue };
            let f = band[krc] / d;
            if f == 0.0 {
                continue;
            }
            for j in c..(c + 2 * bw + 1).min(n) {
                if let (Some(kr), Some(kc)) = (at(r, j), at(c, j)) {
                    band[kr] -= f * band[kc];
                }
            }
            for col in 0..m {
                let v = x[(c, col)];
                x[(r, col)] -= f * v;
            }
        }
    }
    // Back substitution.
    for c in (0..n).rev() {
        let d = band[at(c, c).unwrap()];
        for col in 0..m {
            let mut s = x[(c, col)];
            for j in c + 1..(c + 2 * bw + 1).min(n) {
                if let Some(k) = at(c, j) {
                    s -= band[k] * x[(j, col)];
                }
            }
            x[(c, col)] = s / d;
        }
    }
    Some(x)
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
    fn split_halves_are_valid_curves() {
        let c = NurbsCurve::interpolate(&[P3::new(0.0, 0.0, 0.0), P3::new(1.0, 2.0, 0.0), P3::new(3.0, 1.0, 1.0), P3::new(4.0, 3.0, 0.0), P3::new(6.0, 0.0, 2.0)], &[0.0, 0.2, 0.5, 0.7, 1.0], 3, None, None);
        for u in [0.3, 0.5] {
            let (l, r) = c.split(u);
            for h in [&l, &r] {
                assert_eq!(h.knots.len(), h.cps.len() + h.degree + 1);
            }
            assert_eq!((l.range(), r.range()), ((0.0, u), (u, 1.0)));
            for i in 0..=20 {
                let t = u * i as f64 / 20.0;
                assert!((l.point(t) - c.point(t)).norm() < 1e-12);
                let t = u + (1.0 - u) * i as f64 / 20.0;
                assert!((r.point(t) - c.point(t)).norm() < 1e-12);
            }
        }
    }

    #[test]
    fn elevation_keeps_the_curve() {
        let c = NurbsCurve::interpolate(&[P3::new(0.0, 0.0, 0.0), P3::new(1.0, 2.0, 0.0), P3::new(3.0, 1.0, 1.0), P3::new(4.0, 3.0, 0.0), P3::new(6.0, 0.0, 2.0)], &[0.0, 0.2, 0.5, 0.7, 1.0], 2, None, None);
        let w = std::f64::consts::FRAC_1_SQRT_2;
        let arc = NurbsCurve::bezier(&[P3::new(1.0, 0.0, 0.0), P3::new(1.0, 1.0, 0.0), P3::new(0.0, 1.0, 0.0)], &[1.0, w, 1.0]);
        for c in [c, arc] {
            let e = c.elevated();
            assert_eq!(e.degree, c.degree + 1);
            assert_eq!(e.knots.len(), e.cps.len() + e.degree + 1);
            for i in 0..=40 {
                let u = i as f64 / 40.0;
                assert!((e.point(u) - c.point(u)).norm() < 1e-12, "{u}");
            }
        }
    }

    #[test]
    fn periodic_interpolation_closes_smoothly() {
        let pts = [P3::new(1.0, 0.0, 0.0), P3::new(0.0, 2.0, 0.5), P3::new(-1.5, 0.0, 0.0), P3::new(0.0, -1.0, -0.5)];
        let params = [0.0, 0.2, 0.55, 0.8, 1.0];
        let c = NurbsCurve::interpolate_periodic(&pts, &params, 3).unwrap();
        assert_eq!(c.range(), (0.0, 1.0));
        for (q, u) in pts.iter().zip(params) {
            assert!((c.point(u) - q).norm() < 1e-9, "{u}");
        }
        assert!((c.point(1.0) - pts[0]).norm() < 1e-9);
        let (d0, d1) = (c.ders(0.0, 2), c.ders(1.0, 2));
        assert!((d0[1] - d1[1]).norm() < 1e-8 && (d0[2] - d1[2]).norm() < 1e-7, "{d0:?} {d1:?}");
    }

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
    fn a_sub_net_holds_its_patch() {
        let pts: Vec<Vec<P3>> = (0..5).map(|i| (0..4).map(|j| P3::new(i as f64, j as f64, ((i * 7 + j * 3) % 5) as f64)).collect()).collect();
        let cps: Vec<Vec<H>> = pts.iter().map(|r| r.iter().map(|p| H::new(p.x, p.y, p.z, 1.0)).collect()).collect();
        let s = NurbsSurface { du: 3, dv: 2, ku: vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0], kv: vec![0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0], cps };
        let (u, v) = ((0.2, 0.7), (0.1, 0.55));
        let net = s.sub_net(u, v);
        let mut lo = V3::repeat(f64::MAX);
        let mut hi = V3::repeat(f64::MIN);
        for h in net.iter().flatten() {
            let p = V3::new(h.x / h.w, h.y / h.w, h.z / h.w);
            lo = lo.inf(&p);
            hi = hi.sup(&p);
        }
        let whole: f64 = (0..5).map(|i| pts[i].iter().map(|p| p.z).fold(f64::MIN, f64::max)).fold(f64::MIN, f64::max);
        for i in 0..=20 {
            for j in 0..=20 {
                let q = s.ders2(u.0 + (u.1 - u.0) * i as f64 / 20.0, v.0 + (v.1 - v.0) * j as f64 / 20.0)[0];
                assert!(q.x >= lo.x - 1e-12 && q.x <= hi.x + 1e-12 && q.y >= lo.y - 1e-12 && q.y <= hi.y + 1e-12 && q.z >= lo.z - 1e-12 && q.z <= hi.z + 1e-12);
            }
        }
        // Tighter than the whole net.
        assert!((hi.x - lo.x) < 4.0 && (hi.y - lo.y) < 3.0 && hi.z <= whole);
    }

    #[test]
    fn restarting_a_closed_curve_keeps_its_points() {
        // A closed rational quadratic: the unit circle as four quarter arcs.
        let r = std::f64::consts::FRAC_1_SQRT_2;
        let pts = [(1.0, 0.0, 1.0), (1.0, 1.0, r), (0.0, 1.0, 1.0), (-1.0, 1.0, r), (-1.0, 0.0, 1.0), (-1.0, -1.0, r), (0.0, -1.0, 1.0), (1.0, -1.0, r), (1.0, 0.0, 1.0)];
        let c = NurbsCurve { degree: 2, knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0], cps: pts.iter().map(|&(x, y, w)| H::new(x * w, y * w, 0.0, w)).collect() };
        let d = c.restarted(1.3);
        assert_eq!(d.knots.len(), d.cps.len() + d.degree + 1);
        assert_eq!(d.range(), (1.3, 5.3));
        assert!((d.point(1.3) - c.point(1.3)).norm() < 1e-12);
        for i in 0..=40 {
            let t = 1.3 + 4.0 * i as f64 / 40.0;
            let q = d.point(t);
            assert!((q.coords.norm() - 1.0).abs() < 1e-12);
            let want = if t <= 4.0 { c.point(t) } else { c.point(t - 4.0) };
            assert!((q - want).norm() < 1e-12, "{t}");
        }
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

    /// Malformed knots, degrees, weights and coordinates are errors from the checked
    /// constructors (regression: `range` indexed `knots[cps.len()]` unchecked, and `bezier`
    /// underflowed on no poles).
    #[test]
    fn checked_constructors_refuse_malformed_splines() {
        let cps = |n: usize| (0..n).map(|i| H::new(i as f64, 0.0, 0.0, 1.0)).collect::<Vec<_>>();
        assert!(NurbsCurve::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], cps(3)).is_ok());
        // OCCT blend complex_e7: as many knots as poles.
        assert!(NurbsCurve::new(2, vec![0.0, 0.0, 1.0, 1.0], cps(4)).is_err());
        assert!(NurbsCurve::new(0, vec![0.0, 1.0], cps(1)).is_err());
        assert!(NurbsCurve::new(3, vec![0.0; 7], cps(3)).is_err());
        assert!(NurbsCurve::new(1, vec![0.0, 0.0, 1.0, 0.5, 1.0], cps(3)).is_err());
        assert!(NurbsCurve::new(1, vec![0.0, 0.0, 0.0, 0.0], cps(2)).is_err());
        assert!(NurbsCurve::new(1, vec![0.0, f64::NAN, 1.0, 1.0], cps(2)).is_err());
        let mut w = cps(2);
        w[1].w = 0.0;
        assert!(NurbsCurve::new(1, vec![0.0, 0.0, 1.0, 1.0], w.clone()).is_err());
        w[1] = H::new(f64::INFINITY, 0.0, 0.0, 1.0);
        assert!(NurbsCurve::new(1, vec![0.0, 0.0, 1.0, 1.0], w).is_err());
        assert!(NurbsCurve::try_bezier(&[], &[]).is_err());
        assert!(NurbsCurve::try_bezier(&[P3::origin()], &[1.0]).is_err());
        assert!(NurbsCurve::try_bezier(&[P3::origin(), P3::new(1.0, 0.0, 0.0)], &[1.0]).is_err());
        assert!(NurbsCurve::try_bezier(&[P3::origin(), P3::new(1.0, 0.0, 0.0)], &[1.0, -1.0]).is_err());
        assert_eq!(NurbsCurve::try_bezier(&[P3::origin(), P3::new(1.0, 0.0, 0.0)], &[1.0, 1.0]).map(|c| c.range()), Ok((0.0, 1.0)));
        let k = vec![0.0, 0.0, 1.0, 1.0];
        assert!(NurbsSurface::new(1, 1, k.clone(), k.clone(), vec![cps(2), cps(2)]).is_ok());
        assert!(NurbsSurface::new(1, 1, k.clone(), k.clone(), vec![cps(2), cps(3)]).is_err());
        assert!(NurbsSurface::new(1, 1, k.clone(), k.clone(), vec![]).is_err());
        assert!(NurbsSurface::new(0, 0, vec![], vec![], vec![vec![]]).is_err());
        assert!(NurbsSurface::new(1, 2, k.clone(), k, vec![cps(2), cps(2)]).is_err());
    }
}
