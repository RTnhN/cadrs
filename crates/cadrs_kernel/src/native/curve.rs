//! Exact curves, used both as 3D edge curves and (with z = 0) as p-curves in a face's (u, v)
//! parameter plane.

use std::sync::Arc;

use super::math::*;
use super::nurbs::NurbsCurve;

#[derive(Debug, Clone, PartialEq)]
pub enum Curve {
    /// `o + t·d`.
    Line { o: P3, d: V3 },
    /// `c + r(cos t·x + sin t·y)`, x and y orthonormal.
    Circle { c: P3, x: V3, y: V3, r: f64 },
    /// `c + a cos t·x + b sin t·y`.
    Ellipse { c: P3, x: V3, y: V3, a: f64, b: f64 },
    Nurbs(Arc<NurbsCurve>),
}

impl Curve {
    pub fn line(a: P3, b: P3) -> (Curve, f64) {
        let d = b - a;
        let l = d.norm();
        (Curve::Line { o: a, d: d / l }, l)
    }

    pub fn nurbs(c: NurbsCurve) -> Curve {
        Curve::Nurbs(Arc::new(c))
    }

    pub fn point(&self, t: f64) -> P3 {
        match self {
            Curve::Line { o, d } => o + d * t,
            Curve::Circle { c, x, y, r } => c + (x * t.cos() + y * t.sin()) * *r,
            Curve::Ellipse { c, x, y, a, b } => c + x * (a * t.cos()) + y * (b * t.sin()),
            Curve::Nurbs(n) => n.point(t),
        }
    }

    /// Point, first and second derivative.
    pub fn d2(&self, t: f64) -> (P3, V3, V3) {
        match self {
            Curve::Line { o, d } => (o + d * t, *d, V3::zeros()),
            Curve::Circle { c, x, y, r } => {
                let (s, co) = t.sin_cos();
                (c + (x * co + y * s) * *r, (y * co - x * s) * *r, -(x * co + y * s) * *r)
            }
            Curve::Ellipse { c, x, y, a, b } => {
                let (s, co) = t.sin_cos();
                (c + x * (a * co) + y * (b * s), y * (b * co) - x * (a * s), -(x * (a * co) + y * (b * s)))
            }
            Curve::Nurbs(n) => {
                let d = n.ders(t, 2);
                (P3::from(d[0]), d[1], d[2])
            }
        }
    }

    pub fn tangent(&self, t: f64) -> V3 {
        self.d2(t).1
    }

    pub fn is_periodic(&self) -> bool {
        matches!(self, Curve::Circle { .. } | Curve::Ellipse { .. })
    }

    /// The natural parameter range, or None when unbounded/periodic.
    pub fn nurbs_range(&self) -> Option<(f64, f64)> {
        match self {
            Curve::Nurbs(n) => Some(n.range()),
            _ => None,
        }
    }

    pub fn transformed(&self, m: &Affine) -> Curve {
        match self {
            Curve::Line { o, d } => {
                let dd = m.vector(d);
                Curve::Line { o: m.point(o), d: dd }
            }
            Curve::Circle { c, x, y, r } => {
                let s = m.scale();
                Curve::Circle { c: m.point(c), x: m.vector(x) / s, y: m.vector(y) / s, r: r * s }
            }
            Curve::Ellipse { c, x, y, a, b } => {
                let s = m.scale();
                Curve::Ellipse { c: m.point(c), x: m.vector(x) / s, y: m.vector(y) / s, a: a * s, b: b * s }
            }
            Curve::Nurbs(n) => Curve::Nurbs(Arc::new(n.transformed(m))),
        }
    }

    /// Breakpoints for quadrature and sampling over [t0, t1]: pieces where the curve is smooth
    /// and turns at most a quarter turn.
    pub fn pieces(&self, t0: f64, t1: f64) -> Vec<f64> {
        let mut out = vec![t0];
        match self {
            Curve::Line { .. } => {}
            Curve::Circle { .. } | Curve::Ellipse { .. } => {
                let n = ((t1 - t0).abs() / (PI / 4.0)).ceil().max(1.0) as usize;
                for i in 1..n {
                    out.push(t0 + (t1 - t0) * i as f64 / n as f64);
                }
            }
            Curve::Nurbs(c) => {
                for b in c.breaks() {
                    if b > t0 + 1e-12 && b < t1 - 1e-12 {
                        out.push(b);
                    }
                }
            }
        }
        out.push(t1);
        out
    }

    /// Exact (to quadrature precision) length over [t0, t1].
    pub fn length(&self, t0: f64, t1: f64) -> f64 {
        match self {
            Curve::Line { d, .. } => d.norm() * (t1 - t0).abs(),
            Curve::Circle { r, .. } => r * (t1 - t0).abs(),
            _ => {
                let p = self.pieces(t0, t1);
                let mut s = 0.0;
                for w in p.windows(2) {
                    // Subdivide each piece adaptively by comparing two orders.
                    s += adaptive_len(self, w[0], w[1], 0);
                }
                s
            }
        }
    }

    /// The parameter of the point of the curve nearest `p`, in [t0, t1] (periodic curves:
    /// unwrapped into the range when possible).
    pub fn closest(&self, p: &P3, t0: f64, t1: f64) -> f64 {
        match self {
            Curve::Line { o, d } => ((p - o).dot(d) / d.norm_squared()).clamp(t0, t1),
            Curve::Circle { c, x, y, .. } => {
                let v = p - c;
                let a = v.dot(y).atan2(v.dot(x));
                clamp_periodic(a, t0, t1)
            }
            _ => {
                // Sample then Newton.
                let n = 64;
                let mut best = (f64::INFINITY, t0);
                for i in 0..=n {
                    let t = t0 + (t1 - t0) * i as f64 / n as f64;
                    let d = (self.point(t) - p).norm_squared();
                    if d < best.0 {
                        best = (d, t);
                    }
                }
                let mut t = best.1;
                for _ in 0..30 {
                    let (q, d1, d2) = self.d2(t);
                    let r = q - p;
                    let f = r.dot(&d1);
                    let df = d1.norm_squared() + r.dot(&d2);
                    if df.abs() < 1e-300 {
                        break;
                    }
                    let nt = (t - f / df).clamp(t0, t1);
                    if (nt - t).abs() < 1e-15 * (1.0 + t.abs()) {
                        t = nt;
                        break;
                    }
                    t = nt;
                }
                t
            }
        }
    }

    /// Parameters along [t0, t1] dividing the curve so the chord deviates at most `defl` and
    /// the tangent turns at most `ang` per step. Includes both ends.
    pub fn sample_params(&self, t0: f64, t1: f64, defl: f64, ang: f64) -> Vec<f64> {
        match self {
            Curve::Line { .. } => vec![t0, t1],
            Curve::Circle { r, .. } => {
                let step = arc_step(*r, defl, ang);
                let n = ((t1 - t0).abs() / step).ceil().max(1.0) as usize;
                (0..=n).map(|i| t0 + (t1 - t0) * i as f64 / n as f64).collect()
            }
            _ => {
                // Bound the curvature on each piece by sampling.
                let pieces = self.pieces(t0, t1);
                let mut out = vec![t0];
                for w in pieces.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    let m = 16;
                    let mut kmax: f64 = 0.0;
                    let mut speed: f64 = 0.0;
                    for i in 0..=m {
                        let t = a + (b - a) * i as f64 / m as f64;
                        let (_, d1, d2) = self.d2(t);
                        let s = d1.norm();
                        speed = speed.max(s);
                        if s > 1e-300 {
                            kmax = kmax.max(d1.cross(&d2).norm() / (s * s * s));
                        }
                    }
                    let len = self.length(a, b);
                    let n = if kmax < 1e-12 {
                        1
                    } else {
                        let step = arc_step(1.0 / kmax, defl, ang);
                        ((len * kmax / step).ceil() as usize).max(1)
                    };
                    let n = n.max(if speed > 0.0 { 2 } else { 1 }).min(4096);
                    for i in 1..=n {
                        out.push(a + (b - a) * i as f64 / n as f64);
                    }
                }
                out
            }
        }
    }

    /// The same geometry with t ↦ −t (line, circle, ellipse) or a reversed NURBS; returns the new
    /// curve and the map of the old range [t0, t1] onto the new one.
    pub fn reversed(&self, t0: f64, t1: f64) -> (Curve, f64, f64) {
        match self {
            Curve::Line { o, d } => (Curve::Line { o: *o, d: -d }, -t1, -t0),
            Curve::Circle { c, x, y, r } => (Curve::Circle { c: *c, x: *x, y: -y, r: *r }, -t1, -t0),
            Curve::Ellipse { c, x, y, a, b } => (Curve::Ellipse { c: *c, x: *x, y: -y, a: *a, b: *b }, -t1, -t0),
            Curve::Nurbs(n) => {
                let k0 = n.knots[0];
                let k1 = *n.knots.last().unwrap();
                (Curve::Nurbs(Arc::new(n.reversed())), k0 + k1 - t1, k0 + k1 - t0)
            }
        }
    }

    /// A NURBS curve exactly representing this curve on [t0, t1] (circles and ellipses as
    /// rational quadratics), parametrized on [t0, t1] (not arc-length equivalent for conics).
    pub fn to_nurbs(&self, t0: f64, t1: f64) -> NurbsCurve {
        match self {
            Curve::Line { .. } => NurbsCurve::bezier(&[self.point(t0), self.point(t1)], &[1.0, 1.0]).reparam(t0, t1),
            Curve::Circle { c, x, y, r } => conic_nurbs(*c, x * *r, y * *r, t0, t1),
            Curve::Ellipse { c, x, y, a, b } => conic_nurbs(*c, x * *a, y * *b, t0, t1),
            Curve::Nurbs(n) => {
                let (a, b) = n.range();
                let mut c = (**n).clone();
                if t0 > a + 1e-12 {
                    c = c.split(t0).1;
                }
                if t1 < b - 1e-12 {
                    c = c.split(t1).0;
                }
                c
            }
        }
    }
}

fn adaptive_len(c: &Curve, a: f64, b: f64, depth: usize) -> f64 {
    let f = |t: f64| c.tangent(t).norm();
    let g8 = integrate(a, b, 1, &GAUSS8, f);
    let g16 = integrate(a, b, 1, &GAUSS16, f);
    if (g8 - g16).abs() < 1e-13 * g16.abs().max(1e-9) || depth > 12 {
        g16
    } else {
        let m = (a + b) * 0.5;
        adaptive_len(c, a, m, depth + 1) + adaptive_len(c, m, b, depth + 1)
    }
}

/// The angle step for an arc of radius r to stay within `defl` and `ang`.
/// The angle step along an arc of radius r: within `defl` of it, and the chords turning by at
/// most half of `ang` (the mesh's area then stays within ~3e-4 of the exact one at 5°).
pub fn arc_step(r: f64, defl: f64, ang: f64) -> f64 {
    let d = if defl < r { 2.0 * (1.0 - defl / r).acos() } else { PI / 2.0 };
    d.min(ang * 0.5).max(1e-3)
}

/// Puts an angle into the range [t0, t1] modulo 2π when it fits, else the nearer end.
pub fn clamp_periodic(a: f64, t0: f64, t1: f64) -> f64 {
    let a = wrap(a, t0, TAU);
    if a <= t1 + 1e-12 {
        return a.min(t1);
    }
    // Outside: nearest end (angularly).
    let d0 = (a - (t0 + TAU)).abs();
    let d1 = (a - t1).abs();
    if d0 < d1 { t0 } else { t1 }
}

/// A rational quadratic B-spline for `c + cos t·x + sin t·y` on [t0, t1], parametrized on
/// [t0, t1] by angle at the arc joints.
pub(crate) fn conic_nurbs(c: P3, x: V3, y: V3, t0: f64, t1: f64) -> NurbsCurve {
    let sweep = t1 - t0;
    let n = ((sweep.abs() / (PI / 2.0)).ceil() as usize).max(1);
    let dt = sweep / n as f64;
    let w = (dt / 2.0).cos();
    let pt = |t: f64| c + x * t.cos() + y * t.sin();
    let mut cps = Vec::new();
    let mut knots = vec![t0; 3];
    for i in 0..n {
        let a = t0 + dt * i as f64;
        let b = a + dt;
        let m = (a + b) / 2.0;
        if i == 0 {
            cps.push(super::nurbs::hom(&pt(a), 1.0));
        }
        let mid = c + (x * m.cos() + y * m.sin()) / w;
        cps.push(super::nurbs::hom(&mid, w));
        cps.push(super::nurbs::hom(&pt(b), 1.0));
        if i + 1 < n {
            knots.push(b);
            knots.push(b);
        }
    }
    knots.extend([t1; 3]);
    NurbsCurve { degree: 2, knots, cps }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_length_and_closest() {
        let c = Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::y(), r: 10.0 };
        assert!((c.length(0.0, TAU) - TAU * 10.0).abs() < 1e-12);
        let t = c.closest(&P3::new(0.0, -5.0, 1.0), 0.0, TAU);
        assert!((t - 1.5 * PI).abs() < 1e-12);
    }

    #[test]
    fn ellipse_length() {
        let e = Curve::Ellipse { c: P3::origin(), x: V3::x(), y: V3::y(), a: 30.0, b: 20.0 };
        // Ramanujan II approximation is accurate to ~1e-9 relative here.
        let (a, b) = (30.0f64, 20.0f64);
        let h = ((a - b) / (a + b)).powi(2);
        let ram = PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()));
        assert!((e.length(0.0, TAU) - ram).abs() < 1e-6);
    }

    #[test]
    fn conic_nurbs_matches() {
        let c = Curve::Circle { c: P3::new(1.0, 2.0, 3.0), x: V3::x(), y: V3::z(), r: 4.0 };
        let n = c.to_nurbs(0.3, 5.0);
        for i in 0..=20 {
            let t = 0.3 + 4.7 * i as f64 / 20.0;
            let p = n.point(t);
            assert!(((p - P3::new(1.0, 2.0, 3.0)).norm() - 4.0).abs() < 1e-12);
        }
        assert!((n.point(0.3) - c.point(0.3)).norm() < 1e-12);
        assert!((n.point(5.0) - c.point(5.0)).norm() < 1e-12);
    }
}
