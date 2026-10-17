//! Exact surfaces with their (u, v) parametrizations.
//!
//! - Plane: `o + u·x + v·y`.
//! - Cylinder: `o + r(cos u·x + sin u·y) + v·z`.
//! - Cone: `o + (r + v sin a)(cos u·x + sin u·y) + v cos a·z` (a: the semi-angle).
//! - Sphere: `o + r cos v(cos u·x + sin u·y) + r sin v·z`, v in [−π/2, π/2].
//! - Torus: `o + (R + r cos v)(cos u·x + sin u·y) + r sin v·z`.
//! - Revolution: the meridian curve (in the frame's xz half plane) turned by u about z.
//! - Extrusion: `C(u) + v·d`.
//!
//! The natural normal is Su × Sv; for the analytic surfaces it points away from the axis
//! (outward from the material of a solid cylinder etc.).

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsSurface;
use crate::SurfaceKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Surface {
    Plane { f: Frame },
    Cylinder { f: Frame, r: f64 },
    Cone { f: Frame, r: f64, a: f64 },
    Sphere { f: Frame, r: f64 },
    Torus { f: Frame, rmaj: f64, rmin: f64 },
    Revolution { f: Frame, m: Curve },
    Extrusion { c: Curve, d: V3 },
    Nurbs(Arc<NurbsSurface>),
}

/// Point and derivatives: S, Su, Sv, Suu, Suv, Svv.
pub type D2 = [V3; 6];

impl Surface {
    pub fn kind(&self) -> SurfaceKind {
        match self {
            Surface::Plane { .. } => SurfaceKind::Plane,
            Surface::Cylinder { .. } => SurfaceKind::Cylinder,
            Surface::Cone { .. } => SurfaceKind::Cone,
            Surface::Sphere { .. } => SurfaceKind::Sphere,
            Surface::Torus { .. } => SurfaceKind::Torus,
            _ => SurfaceKind::Other,
        }
    }

    pub fn frame(&self) -> Option<&Frame> {
        match self {
            Surface::Plane { f } | Surface::Cylinder { f, .. } | Surface::Cone { f, .. } | Surface::Sphere { f, .. } | Surface::Torus { f, .. } | Surface::Revolution { f, .. } => Some(f),
            _ => None,
        }
    }

    pub fn u_periodic(&self) -> bool {
        match self {
            Surface::Plane { .. } | Surface::Nurbs(_) => false,
            Surface::Extrusion { c, .. } => c.is_periodic(),
            _ => true,
        }
    }

    pub fn v_periodic(&self) -> bool {
        matches!(self, Surface::Torus { .. })
    }

    pub fn eval(&self, u: f64, v: f64) -> P3 {
        match self {
            Surface::Plane { f } => f.o + f.x * u + f.y * v,
            Surface::Cylinder { f, r } => f.o + (f.x * u.cos() + f.y * u.sin()) * *r + f.z * v,
            Surface::Cone { f, r, a } => f.o + (f.x * u.cos() + f.y * u.sin()) * (r + v * a.sin()) + f.z * (v * a.cos()),
            Surface::Sphere { f, r } => f.o + (f.x * u.cos() + f.y * u.sin()) * (r * v.cos()) + f.z * (r * v.sin()),
            Surface::Torus { f, rmaj, rmin } => f.o + (f.x * u.cos() + f.y * u.sin()) * (rmaj + rmin * v.cos()) + f.z * (rmin * v.sin()),
            _ => P3::from(self.d2(u, v)[0]),
        }
    }

    pub fn d1(&self, u: f64, v: f64) -> (P3, V3, V3) {
        let d = self.d2(u, v);
        (P3::from(d[0]), d[1], d[2])
    }

    pub fn d2(&self, u: f64, v: f64) -> D2 {
        match self {
            Surface::Plane { f } => [(f.o + f.x * u + f.y * v).coords, f.x, f.y, V3::zeros(), V3::zeros(), V3::zeros()],
            Surface::Cylinder { f, r } => {
                let (s, c) = u.sin_cos();
                let rho = f.x * c + f.y * s;
                let phi = f.y * c - f.x * s;
                [(f.o + rho * *r + f.z * v).coords, phi * *r, f.z, -rho * *r, V3::zeros(), V3::zeros()]
            }
            Surface::Cone { f, r, a } => {
                let (s, c) = u.sin_cos();
                let rho = f.x * c + f.y * s;
                let phi = f.y * c - f.x * s;
                let (sa, ca) = a.sin_cos();
                let rr = r + v * sa;
                [(f.o + rho * rr + f.z * (v * ca)).coords, phi * rr, rho * sa + f.z * ca, -rho * rr, phi * sa, V3::zeros()]
            }
            Surface::Sphere { f, r } => {
                let (s, c) = u.sin_cos();
                let (sv, cv) = v.sin_cos();
                let rho = f.x * c + f.y * s;
                let phi = f.y * c - f.x * s;
                [
                    (f.o + rho * (r * cv) + f.z * (r * sv)).coords,
                    phi * (r * cv),
                    (-rho * sv + f.z * cv) * *r,
                    -rho * (r * cv),
                    -phi * (r * sv),
                    (-rho * cv - f.z * sv) * *r,
                ]
            }
            Surface::Torus { f, rmaj, rmin } => {
                let (s, c) = u.sin_cos();
                let (sv, cv) = v.sin_cos();
                let rho = f.x * c + f.y * s;
                let phi = f.y * c - f.x * s;
                let rr = rmaj + rmin * cv;
                [
                    (f.o + rho * rr + f.z * (rmin * sv)).coords,
                    phi * rr,
                    (-rho * sv + f.z * cv) * *rmin,
                    -rho * rr,
                    -phi * (rmin * sv),
                    (-rho * cv - f.z * sv) * *rmin,
                ]
            }
            Surface::Revolution { f, m } => {
                let (p, d1, d2) = m.d2(v);
                let l = f.local(&p);
                let ld1 = V3::new(d1.dot(&f.x), d1.dot(&f.y), d1.dot(&f.z));
                let ld2 = V3::new(d2.dot(&f.x), d2.dot(&f.y), d2.dot(&f.z));
                let (s, c) = u.sin_cos();
                let rot = |q: &V3| V3::new(q.x * c - q.y * s, q.x * s + q.y * c, q.z);
                let drot = |q: &V3| V3::new(-q.x * s - q.y * c, q.x * c - q.y * s, 0.0);
                let ddrot = |q: &V3| V3::new(-q.x * c + q.y * s, -q.x * s - q.y * c, 0.0);
                let w = |q: V3| f.world_vec(&q);
                [
                    f.world_point(&rot(&l)).coords,
                    w(drot(&l)),
                    w(rot(&ld1)),
                    w(ddrot(&l)),
                    w(drot(&ld1)),
                    w(rot(&ld2)),
                ]
            }
            Surface::Extrusion { c, d } => {
                let (p, d1, d2) = c.d2(u);
                [(p + d * v).coords, d1, *d, d2, V3::zeros(), V3::zeros()]
            }
            Surface::Nurbs(n) => n.ders2(u, v),
        }
    }

    /// The natural (Su × Sv) normal, unit length; falls back to neighbouring points at poles.
    pub fn normal(&self, u: f64, v: f64) -> V3 {
        let d = self.d2(u, v);
        let n = d[1].cross(&d[2]);
        let l = n.norm();
        if l > 1e-14 {
            return n / l;
        }
        // Degenerate point (a pole or apex): look a little inside.
        match self {
            Surface::Sphere { f, .. } => {
                let p = P3::from(d[0]);
                let l = f.local(&p);
                if l.z >= 0.0 { f.z } else { -f.z }
            }
            Surface::Cone { f, a, .. } => {
                // Apex: the average normal is along −z·sin a (outward-ish); use a nearby point.
                let _ = (f, a);
                let e = 1e-6;
                let d = self.d2(u, v + e);
                d[1].cross(&d[2]).normalize()
            }
            _ => {
                let e = 1e-7;
                for (du, dv) in [(e, 0.0), (0.0, e), (-e, 0.0), (0.0, -e), (e, e)] {
                    let d = self.d2(u + du, v + dv);
                    let n = d[1].cross(&d[2]);
                    if n.norm() > 1e-14 {
                        return n.normalize();
                    }
                }
                V3::z()
            }
        }
    }

    /// The (u, v) of the point of the surface nearest `p`; periodic coordinates near `hint` when
    /// given, else in [0, 2π).
    pub fn project(&self, p: &P3, hint: Option<(f64, f64)>) -> (f64, f64) {
        let ang = |f: &Frame| {
            let l = f.local(p);
            let a = l.y.atan2(l.x);
            match hint {
                Some((hu, _)) => unwrap_near(a, hu, TAU),
                None => wrap(a, 0.0, TAU),
            }
        };
        match self {
            Surface::Plane { f } => {
                let l = f.local(p);
                (l.x, l.y)
            }
            Surface::Cylinder { f, .. } => (ang(f), f.local(p).z),
            Surface::Cone { f, r, a } => {
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let (sa, ca) = a.sin_cos();
                let v = (rho - r) * sa + l.z * ca;
                // At the apex side (v below the apex) atan2 is meaningless: keep the hint.
                (ang(f), v)
            }
            Surface::Sphere { f, .. } => {
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let v = l.z.atan2(rho);
                let u = if rho < 1e-12 { hint.map(|h| h.0).unwrap_or(0.0) } else { ang(f) };
                (u, v)
            }
            Surface::Torus { f, rmaj, .. } => {
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let v = l.z.atan2(rho - rmaj);
                let v = match hint {
                    Some((_, hv)) => unwrap_near(v, hv, TAU),
                    None => wrap(v, -PI, TAU),
                };
                (ang(f), v)
            }
            Surface::Revolution { f, m } => {
                let u = ang(f);
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let q = f.world_point(&V3::new(rho, 0.0, l.z));
                let (a, b) = match m.nurbs_range() {
                    Some(r) => r,
                    None if m.is_periodic() => {
                        let c = hint.map(|h| h.1).unwrap_or(0.0);
                        (c - PI, c + PI)
                    }
                    None => (-1e12, 1e12),
                };
                let v = m.closest(&q, a, b);
                (u, v)
            }
            Surface::Extrusion { .. } | Surface::Nurbs(_) => self.project_newton(p, hint),
        }
    }

    /// Generic closest point by sampling and Newton (extrusions and NURBS).
    fn project_newton(&self, p: &P3, hint: Option<(f64, f64)>) -> (f64, f64) {
        let ((u0, u1), (v0, v1)) = self.param_box();
        let (mut u, mut v) = match hint {
            Some(h) => h,
            None => {
                let n = 24;
                let mut best = (f64::INFINITY, (u0, v0));
                for i in 0..=n {
                    for j in 0..=n {
                        let (a, b) = (u0 + (u1 - u0) * i as f64 / n as f64, v0 + (v1 - v0) * j as f64 / n as f64);
                        let d = (self.eval(a, b) - p).norm_squared();
                        if d < best.0 {
                            best = (d, (a, b));
                        }
                    }
                }
                best.1
            }
        };
        for _ in 0..50 {
            let d = self.d2(u, v);
            let r = d[0] - p.coords;
            let fu = r.dot(&d[1]);
            let fv = r.dot(&d[2]);
            let a = d[1].norm_squared() + r.dot(&d[3]);
            let b = d[1].dot(&d[2]) + r.dot(&d[4]);
            let c = d[2].norm_squared() + r.dot(&d[5]);
            let Some((du, dv)) = solve2(a, b, b, c, -fu, -fv) else { break };
            let (nu, nv) = (self.clamp_u(u + du, u0, u1), (v + dv).clamp(v0.min(v1), v1.max(v0)));
            let step = (nu - u).abs() + (nv - v).abs();
            u = nu;
            v = nv;
            if step < 1e-15 {
                break;
            }
        }
        (u, v)
    }

    fn clamp_u(&self, u: f64, u0: f64, u1: f64) -> f64 {
        if self.u_periodic() { u } else { u.clamp(u0, u1) }
    }

    /// The parameter box used for sampling (NURBS: its range; extrusions: curve range × ±1e4).
    pub fn param_box(&self) -> ((f64, f64), (f64, f64)) {
        match self {
            Surface::Nurbs(n) => n.range(),
            Surface::Extrusion { c, .. } => (c.nurbs_range().unwrap_or((0.0, TAU)), (-1e4, 1e4)),
            Surface::Sphere { .. } => ((0.0, TAU), (-PI / 2.0, PI / 2.0)),
            Surface::Torus { .. } => ((0.0, TAU), (-PI, PI)),
            _ => ((0.0, TAU), (-1e4, 1e4)),
        }
    }

    /// Signed distance from `p` to an analytic surface (positive on the natural normal's side).
    pub fn signed_distance(&self, p: &P3) -> Option<f64> {
        Some(match self {
            Surface::Plane { f } => (p - f.o).dot(&f.x.cross(&f.y)),
            Surface::Cylinder { f, r } => {
                let l = f.local(p);
                (l.x * l.x + l.y * l.y).sqrt() - r
            }
            Surface::Cone { f, r, a } => {
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let (sa, ca) = a.sin_cos();
                // Distance to the line (r + v sa, v ca) in the meridian plane, normal (ca, −sa).
                (rho - r) * ca - l.z * sa
            }
            Surface::Sphere { f, r } => (p - f.o).norm() - r,
            Surface::Torus { f, rmaj, rmin } => {
                let l = f.local(p);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                ((rho - rmaj).powi(2) + l.z * l.z).sqrt() - rmin
            }
            _ => return None,
        })
    }

    /// Distance from `p` to the surface (exact for analytic surfaces, by projection otherwise).
    pub fn distance(&self, p: &P3) -> f64 {
        if let Some(d) = self.signed_distance(p) {
            return d.abs();
        }
        let (u, v) = self.project(p, None);
        (self.eval(u, v) - p).norm()
    }

    pub fn transformed(&self, m: &Affine) -> Surface {
        let s = m.scale();
        let tf = |f: &Frame| {
            let g = f.transformed(m);
            Frame { o: g.o, x: g.x / s, y: g.y / s, z: g.z / s }
        };
        match self {
            Surface::Plane { f } => Surface::Plane { f: tf(f) },
            Surface::Cylinder { f, r } => Surface::Cylinder { f: tf(f), r: r * s },
            Surface::Cone { f, r, a } => Surface::Cone { f: tf(f), r: r * s, a: *a },
            Surface::Sphere { f, r } => Surface::Sphere { f: tf(f), r: r * s },
            Surface::Torus { f, rmaj, rmin } => Surface::Torus { f: tf(f), rmaj: rmaj * s, rmin: rmin * s },
            Surface::Revolution { f, m: c } => Surface::Revolution { f: tf(f), m: c.transformed(m) },
            Surface::Extrusion { c, d } => Surface::Extrusion { c: c.transformed(m), d: m.vector(d) },
            Surface::Nurbs(n) => Surface::Nurbs(Arc::new(n.transformed(m))),
        }
    }

    /// Typical lengths per unit of u and v (for isotropic meshing and step sizes).
    pub fn metric(&self) -> (f64, f64) {
        match self {
            Surface::Plane { .. } => (1.0, 1.0),
            Surface::Cylinder { r, .. } => (*r, 1.0),
            Surface::Cone { r, .. } => (r.abs().max(1e-3), 1.0),
            Surface::Sphere { r, .. } => (*r, *r),
            Surface::Torus { rmaj, rmin, .. } => (rmaj + rmin, *rmin),
            _ => (1.0, 1.0),
        }
    }

    /// The axis of a surface of revolution.
    pub fn axis(&self) -> Option<(P3, V3)> {
        match self {
            Surface::Cylinder { f, .. } | Surface::Cone { f, .. } | Surface::Sphere { f, .. } | Surface::Torus { f, .. } | Surface::Revolution { f, .. } => Some((f.o, f.z)),
            _ => None,
        }
    }

    /// The u = const iso curve, parametrized by v.
    pub fn iso_u(&self, u: f64) -> Option<Curve> {
        Some(match self {
            Surface::Plane { f } => Curve::Line { o: f.o + f.x * u, d: f.y },
            Surface::Cylinder { f, r } => Curve::Line { o: f.o + (f.x * u.cos() + f.y * u.sin()) * *r, d: f.z },
            Surface::Cone { f, r, a } => {
                let rho = f.x * u.cos() + f.y * u.sin();
                Curve::Line { o: f.o + rho * *r, d: rho * a.sin() + f.z * a.cos() }
            }
            Surface::Sphere { f, r } => Curve::Circle { c: f.o, x: f.x * u.cos() + f.y * u.sin(), y: f.z, r: *r },
            Surface::Torus { f, rmaj, rmin } => {
                let rho = f.x * u.cos() + f.y * u.sin();
                Curve::Circle { c: f.o + rho * *rmaj, x: rho, y: f.z, r: *rmin }
            }
            _ => return None,
        })
    }

    /// The v = const iso curve, parametrized by u.
    pub fn iso_v(&self, v: f64) -> Option<Curve> {
        Some(match self {
            Surface::Plane { f } => Curve::Line { o: f.o + f.y * v, d: f.x },
            Surface::Cylinder { f, r } => Curve::Circle { c: f.o + f.z * v, x: f.x, y: f.y, r: *r },
            Surface::Cone { f, r, a } => Curve::Circle { c: f.o + f.z * (v * a.cos()), x: f.x, y: f.y, r: r + v * a.sin() },
            Surface::Sphere { f, r } => Curve::Circle { c: f.o + f.z * (r * v.sin()), x: f.x, y: f.y, r: r * v.cos() },
            Surface::Torus { f, rmaj, rmin } => Curve::Circle { c: f.o + f.z * (rmin * v.sin()), x: f.x, y: f.y, r: rmaj + rmin * v.cos() },
            Surface::Extrusion { c, d } => c.transformed(&Affine::translation(d * v)),
            _ => return None,
        })
    }

    /// Whether two surfaces are the same point set (same plane, cylinder, ...). `Some(true)`
    /// when their natural normals agree, `Some(false)` when opposite.
    pub fn same_domain(&self, o: &Surface, tol: f64) -> Option<bool> {
        let par = |a: &V3, b: &V3| -> Option<bool> {
            let d = a.dot(b);
            if d > 1.0 - 1e-9 {
                Some(true)
            } else if d < -1.0 + 1e-9 {
                Some(false)
            } else {
                None
            }
        };
        let on_axis = |fa: &Frame, fb: &Frame| -> bool {
            let d = fb.o - fa.o;
            (d - fa.z * d.dot(&fa.z)).norm() < tol
        };
        match (self, o) {
            (Surface::Plane { f: a }, Surface::Plane { f: b }) => {
                let (na, nb) = (a.x.cross(&a.y), b.x.cross(&b.y));
                let same = par(&na, &nb)?;
                if (b.o - a.o).dot(&na).abs() < tol { Some(same) } else { None }
            }
            (Surface::Cylinder { f: a, r: ra }, Surface::Cylinder { f: b, r: rb }) => {
                par(&a.z, &b.z)?;
                if (ra - rb).abs() < tol && on_axis(a, b) {
                    // The natural normal points away from the axis exactly when the frame is
                    // right-handed.
                    Some(a.right_handed() == b.right_handed())
                } else {
                    None
                }
            }
            (Surface::Sphere { f: a, r: ra }, Surface::Sphere { f: b, r: rb }) => {
                if (ra - rb).abs() < tol && (a.o - b.o).norm() < tol {
                    Some(a.right_handed() == b.right_handed())
                } else {
                    None
                }
            }
            (Surface::Torus { f: a, rmaj: ra, rmin: sa }, Surface::Torus { f: b, rmaj: rb, rmin: sb }) => {
                par(&a.z, &b.z)?;
                if (ra - rb).abs() < tol && (sa - sb).abs() < tol && (a.o - b.o).norm() < tol {
                    Some(a.right_handed() == b.right_handed())
                } else {
                    None
                }
            }
            (Surface::Cone { f: a, r: ra, a: aa }, Surface::Cone { f: b, r: rb, a: ab }) => {
                // Compare the apex and the opening direction.
                let apex = |f: &Frame, r: f64, ang: f64| f.o - f.z * (r / ang.tan());
                let (pa, pb) = (apex(a, *ra, *aa), apex(b, *rb, *ab));
                if (pa - pb).norm() > tol * 10.0 {
                    return None;
                }
                let da = a.z * aa.cos() + V3::zeros();
                let _ = da;
                let open_a = if aa.sin() > 0.0 { a.z } else { -a.z };
                let open_b = if ab.sin() > 0.0 { b.z } else { -b.z };
                if open_a.dot(&open_b) < 1.0 - 1e-9 || (aa.sin().abs() - ab.sin().abs()).abs() > 1e-9 {
                    return None;
                }
                Some(a.right_handed() == b.right_handed())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_derivs(s: &Surface, u: f64, v: f64) {
        let h = 1e-6;
        let d = s.d2(u, v);
        let num_u = (s.eval(u + h, v) - s.eval(u - h, v)) / (2.0 * h);
        let num_v = (s.eval(u, v + h) - s.eval(u, v - h)) / (2.0 * h);
        assert!((num_u - d[1]).norm() < 1e-6, "{s:?} Su {num_u} {}", d[1]);
        assert!((num_v - d[2]).norm() < 1e-6, "{s:?} Sv");
        let duu = (s.d2(u + h, v)[1] - s.d2(u - h, v)[1]) / (2.0 * h);
        let duv = (s.d2(u, v + h)[1] - s.d2(u, v - h)[1]) / (2.0 * h);
        let dvv = (s.d2(u, v + h)[2] - s.d2(u, v - h)[2]) / (2.0 * h);
        assert!((duu - d[3]).norm() < 1e-5, "{s:?} Suu");
        assert!((duv - d[4]).norm() < 1e-5, "{s:?} Suv");
        assert!((dvv - d[5]).norm() < 1e-5, "{s:?} Svv");
        let (pu, pv) = s.project(&s.eval(u, v), Some((u, v)));
        assert!((s.eval(pu, pv) - s.eval(u, v)).norm() < 1e-9, "{s:?} project");
    }

    #[test]
    fn analytic_derivatives_and_projection() {
        let f = Frame::from_xz(P3::new(1.0, 2.0, 3.0), V3::new(1.0, 1.0, 0.0), V3::new(0.0, 0.3, 1.0));
        let m = Curve::Circle { c: f.o + f.x * 5.0, x: f.x, y: f.z, r: 2.0 };
        for s in [
            Surface::Plane { f },
            Surface::Cylinder { f, r: 3.0 },
            Surface::Cone { f, r: 3.0, a: 0.4 },
            Surface::Sphere { f, r: 3.0 },
            Surface::Torus { f, rmaj: 5.0, rmin: 2.0 },
            Surface::Revolution { f, m },
            Surface::Extrusion { c: Curve::Ellipse { c: P3::origin(), x: V3::x(), y: V3::y(), a: 3.0, b: 2.0 }, d: V3::new(0.1, 0.0, 1.0) },
        ] {
            check_derivs(&s, 0.7, 0.4);
            if let Some(d) = s.signed_distance(&(s.eval(0.7, 0.4) + s.normal(0.7, 0.4) * 0.5)) {
                assert!((d - 0.5).abs() < 1e-12, "{s:?} distance {d}");
            }
        }
    }
}
