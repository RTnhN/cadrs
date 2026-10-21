//! p-curves: a 3D curve lying on a surface as a curve in the surface's (u, v), parametrized by
//! the 3D curve's own parameter. Exact for curves in planes and for rims and rulings of
//! surfaces of revolution; otherwise an interpolating cubic through projected points, refined
//! until it is within 1e-10 of the projection.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::surface::Surface;

fn line2(o: P2, d: V2) -> Curve {
    Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(d.x, d.y, 0.0) }
}

/// The p-curve of `c` on [t0, t1] on `s`, its periodic coordinates shifted so its middle lies
/// near `near` (in the face's chart) when given.
pub fn pcurve(s: &Surface, c: &Curve, t0: f64, t1: f64, near: Option<P2>) -> Curve {
    let pc = exact(s, c, t0, t1).or_else(|| iso_line(s, c, t0, t1)).unwrap_or_else(|| fitted(s, c, t0, t1));
    shift_into(s, pc, (t0 + t1) * 0.5, near)
}

/// Shifts a p-curve by whole periods so its point at `tm` lies nearest `near`.
pub fn shift_into(s: &Surface, pc: Curve, tm: f64, near: Option<P2>) -> Curve {
    let m = pc.point(tm);
    // A sphere's meridian p-curves repeat every turn in v: bring them into [−π/2, π/2].
    if let (Surface::Sphere { .. }, None) = (s, near) {
        let dv = -((m.y + PI / 2.0) / TAU).floor() * TAU;
        if dv != 0.0 {
            return pc.transformed(&Affine::translation(V3::new(0.0, dv, 0.0)));
        }
        return pc;
    }
    let Some(near) = near else { return pc };
    let du = if s.u_periodic() { ((near.x - m.x) / TAU).round() * TAU } else { 0.0 };
    let dv = if s.v_periodic() {
        ((near.y - m.y) / TAU).round() * TAU
    } else if matches!(s, Surface::Sphere { .. }) {
        -((m.y + PI / 2.0) / TAU).floor() * TAU
    } else {
        0.0
    };
    if du == 0.0 && dv == 0.0 {
        return pc;
    }
    pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)))
}

/// The angle of the direction `v` in the frame's xy plane.
fn ang(f: &Frame, v: &V3) -> f64 {
    v.dot(&f.y).atan2(v.dot(&f.x))
}

fn exact(s: &Surface, c: &Curve, t0: f64, t1: f64) -> Option<Curve> {
    match s {
        Surface::Plane { f } => {
            // Model → plane coordinates is affine (rows x, y, z).
            let m = nalgebra::Matrix3::from_rows(&[f.x.transpose(), f.y.transpose(), f.x.cross(&f.y).transpose()]);
            let a = Affine { m, t: -(m * f.o.coords) };
            let pc = match c {
                Curve::Circle { c: cc, x, y, r } => {
                    let (xl, yl) = (m * x, m * y);
                    Curve::Circle { c: a.point(cc), x: V3::new(xl.x, xl.y, 0.0), y: V3::new(yl.x, yl.y, 0.0), r: *r }
                }
                Curve::Ellipse { c: cc, x, y, a: ra, b: rb } => {
                    let (xl, yl) = (m * x, m * y);
                    Curve::Ellipse { c: a.point(cc), x: V3::new(xl.x, xl.y, 0.0), y: V3::new(yl.x, yl.y, 0.0), a: *ra, b: *rb }
                }
                _ => c.transformed(&a),
            };
            Some(flatten(pc))
        }
        Surface::Cylinder { f, r } => match c {
            Curve::Line { o, d } if d.cross(&f.z).norm() < 1e-12 * d.norm() => {
                let l = f.local(o);
                Some(line2(P2::new(l.y.atan2(l.x), l.z), V2::new(0.0, d.dot(&f.z))))
            }
            Curve::Circle { c: cc, x, y, r: rc } if coaxial_circle(f, cc, x, y) && (rc - r).abs() < 1e-7 => {
                let sgn = x.cross(y).dot(&f.x.cross(&f.y)).signum();
                Some(line2(P2::new(ang(f, x), f.local(cc).z), V2::new(sgn, 0.0)))
            }
            _ => None,
        },
        Surface::Cone { f, r, a } => match c {
            Curve::Circle { c: cc, x, y, .. } if coaxial_circle(f, cc, x, y) => {
                let sgn = x.cross(y).dot(&f.x.cross(&f.y)).signum();
                let h = f.local(cc).z;
                Some(line2(P2::new(ang(f, x), h / a.cos()), V2::new(sgn, 0.0)))
            }
            Curve::Line { o, d } => {
                // A ruling: in a plane through the axis, along the cone's slope.
                let l = f.local(o);
                let rho = (l.x * l.x + l.y * l.y).sqrt();
                let u = if rho > 1e-9 { l.y.atan2(l.x) } else {
                    let ld = V3::new(d.dot(&f.x), d.dot(&f.y), 0.0);
                    ld.y.atan2(ld.x)
                };
                let radial = f.x * u.cos() + f.y * u.sin();
                let ruling = radial * a.sin() + f.z * a.cos();
                if d.cross(&ruling).norm() > 1e-9 * d.norm() {
                    return None;
                }
                let _ = r;
                let v0 = (rho - r) * a.sin() + l.z * a.cos();
                Some(line2(P2::new(u, v0), V2::new(0.0, d.dot(&ruling))))
            }
            _ => None,
        },
        Surface::Sphere { f, .. } => match c {
            Curve::Circle { c: cc, x, y, .. } if coaxial_circle(f, cc, x, y) => {
                let sgn = x.cross(y).dot(&f.x.cross(&f.y)).signum();
                let l = f.local(&c.point(0.0));
                let v = l.z.atan2((l.x * l.x + l.y * l.y).sqrt());
                Some(line2(P2::new(ang(f, x), v), V2::new(sgn, 0.0)))
            }
            Curve::Circle { c: cc, x, y, .. } if (cc - f.o).norm() < 1e-9 && x.cross(y).dot(&f.z).abs() < 1e-12 => {
                // A meridian: u const, v = ±t + v0, in the half plane holding the arc.
                let m = c.point((t0 + t1) * 0.5) - f.o;
                let side = m - f.z * m.dot(&f.z);
                if side.norm() < 1e-9 {
                    return None;
                }
                // The arc must stay in that half plane (not run over a pole).
                for s in [0.0, 1.0] {
                    let q = c.point(t0 + (t1 - t0) * s) - f.o;
                    if (q - f.z * q.dot(&f.z)).dot(&side) < -1e-9 {
                        return None;
                    }
                }
                meridian_side(f, x, y, &side.normalize())
            }
            _ => None,
        },
        Surface::Torus { f, rmaj, .. } => match c {
            Curve::Circle { c: cc, x, y, .. } if coaxial_circle(f, cc, x, y) => {
                let sgn = x.cross(y).dot(&f.x.cross(&f.y)).signum();
                let l = f.local(&c.point(0.0));
                let v = l.z.atan2((l.x * l.x + l.y * l.y).sqrt() - rmaj);
                Some(line2(P2::new(ang(f, x), v), V2::new(sgn, 0.0)))
            }
            Curve::Circle { c: cc, x, y, .. } if x.cross(y).dot(&f.z).abs() < 1e-12 => {
                let l = f.local(cc);
                if l.z.abs() > 1e-9 || ((l.x * l.x + l.y * l.y).sqrt() - rmaj).abs() > 1e-9 {
                    return None;
                }
                // The half plane of the circle's centre.
                let w = cc - f.o;
                let radial = (w - f.z * w.dot(&f.z)).normalize();
                meridian_side(f, x, y, &radial)
            }
            _ => None,
        },
        Surface::Revolution { f, m } => match c {
            Curve::Circle { c: cc, x, y, .. } if coaxial_circle(f, cc, x, y) => {
                let sgn = x.cross(y).dot(&f.x.cross(&f.y)).signum();
                let (_, v) = s.project(&c.point(0.0), None);
                // A meridian off the frame's xz plane sits at its own azimuth.
                let az = if super::surface::meridian_off_plane(f, m) { super::surface::meridian_azimuth(f, m, v) } else { 0.0 };
                Some(line2(P2::new(ang(f, x) - az, v), V2::new(sgn, 0.0)))
            }
            _ => None,
        },
        Surface::Extrusion { c: base, d } => match c {
            Curve::Line { o, d: dl } if dl.cross(d).norm() < 1e-12 * dl.norm() => {
                let (u, v) = s.project(o, None);
                let _ = base;
                Some(line2(P2::new(u, v), V2::new(0.0, dl.dot(d) / d.norm_squared())))
            }
            _ => None,
        },
        Surface::Nurbs(_) => None,
    }
}

/// A sphere's meridian in the half plane of `radial`: u const, v = σt + v0.
fn meridian_side(f: &Frame, x: &V3, y: &V3, radial: &V3) -> Option<Curve> {
    let (xr, xz) = (x.dot(radial), x.dot(&f.z));
    let (yr, yz) = (y.dot(radial), y.dot(&f.z));
    let sig = (xr * yz - xz * yr).signum();
    let v0 = xz.atan2(xr);
    let u = ang(f, radial);
    Some(line2(P2::new(u, v0), V2::new(0.0, sig)))
}

fn coaxial_circle(f: &Frame, cc: &P3, x: &V3, y: &V3) -> bool {
    let n = x.cross(y);
    if n.cross(&f.z).norm() > 1e-10 {
        return false;
    }
    let w = cc - f.o;
    (w - f.z * w.dot(&f.z)).norm() < 1e-9
}

/// Drops the z coordinate of a 2D curve (numerical noise from projecting into a plane).
fn flatten(c: Curve) -> Curve {
    match c {
        Curve::Line { o, d } => Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(d.x, d.y, 0.0) },
        Curve::Nurbs(n) => {
            let mut n = (*n).clone();
            for h in &mut n.cps {
                h.z = 0.0;
            }
            Curve::Nurbs(Arc::new(n))
        }
        other => other,
    }
}

/// A curve that runs along an iso-line of a B-spline or general surface with its parameter an
/// affine function of the iso-line's (a sweep's rails, a loft's rails): a straight p-curve.
fn iso_line(s: &Surface, c: &Curve, t0: f64, t1: f64) -> Option<Curve> {
    if !matches!(s, Surface::Nurbs(_) | Surface::Extrusion { .. } | Surface::Revolution { .. }) {
        return None;
    }
    let n = 8;
    let mut uv: Vec<P2> = Vec::with_capacity(n + 1);
    let mut hint: Option<(f64, f64)> = None;
    for i in 0..=n {
        let t = t0 + (t1 - t0) * i as f64 / n as f64;
        let p = c.point(t);
        let (u, v) = s.project(&p, hint);
        if (s.eval(u, v) - p).norm() > 1e-9 {
            return None;
        }
        hint = Some((u, v));
        uv.push(P2::new(u, v));
    }
    let (a, b) = (uv[0], uv[n]);
    let d = (b - a) / (t1 - t0);
    let line = Curve::Line { o: P3::new(a.x - d.x * t0, a.y - d.y * t0, 0.0), d: V3::new(d.x, d.y, 0.0) };
    // Straight in uv and affine in t, and an iso-line (u or v constant).
    if d.x.abs() > 1e-12 * d.y.abs().max(1e-300) && d.y.abs() > 1e-12 * d.x.abs().max(1e-300) {
        return None;
    }
    let tol = 1e-10 * (1.0 + a.coords.norm() + b.coords.norm());
    for (i, q) in uv.iter().enumerate() {
        let t = t0 + (t1 - t0) * i as f64 / n as f64;
        let l = line.point(t);
        if (l.x - q.x).abs() > tol || (l.y - q.y).abs() > tol {
            return None;
        }
    }
    // And between the samples, on the surface.
    for i in 0..n {
        let t = t0 + (t1 - t0) * (i as f64 + 0.5) / n as f64;
        let l = line.point(t);
        if (s.eval(l.x, l.y) - c.point(t)).norm() > 1e-9 {
            return None;
        }
    }
    Some(line)
}

/// Interpolates projected points with the curve's own parameters.
fn fitted(s: &Surface, c: &Curve, t0: f64, t1: f64) -> Curve {
    let pieces = c.pieces(t0, t1);
    let mut ts: Vec<f64> = Vec::new();
    // A few samples a piece to start (refined below where the fit is off).
    let n = (512 / pieces.len().max(1)).clamp(2, 16);
    for w in pieces.windows(2) {
        for i in 0..n {
            ts.push(w[0] + (w[1] - w[0]) * i as f64 / n as f64);
        }
    }
    ts.push(t1);
    let mut uvs: Vec<P2> = Vec::new();
    let project = |t: f64, hint: Option<(f64, f64)>| -> P2 {
        let (u, v) = s.project(&c.point(t), hint);
        P2::new(u, v)
    };
    let fit_at = |ts: &[f64]| -> (NurbsCurve, Vec<P2>) {
        let mut uvs = Vec::with_capacity(ts.len());
        let mut hint: Option<(f64, f64)> = None;
        for &t in ts {
            let q = project(t, hint);
            hint = Some((q.x, q.y));
            uvs.push(q);
        }
        let pts: Vec<P3> = uvs.iter().map(|q| P3::new(q.x, q.y, 0.0)).collect();
        (NurbsCurve::interpolate(&pts, ts, 3.min(pts.len() - 1), None, None), uvs)
    };
    let _ = &mut uvs;
    for _round in 0..12 {
        let (fit, uvs) = fit_at(&ts);
        // Check the midpoints.
        let mut ins = Vec::new();
        for k in 0..ts.len() - 1 {
            let tm = (ts[k] + ts[k + 1]) * 0.5;
            let f = fit.point(tm);
            // Measured in model space between the fit's point and the projection's (uv noise of
            // projections on B-spline surfaces would never settle; an intersection curve may lie
            // a little off the surface, which no p-curve can make up).
            let q = project(tm, Some((uvs[k].x, uvs[k].y)));
            let err = (s.eval(f.x, f.y) - s.eval(q.x, q.y)).norm();
            if err > 1e-9 {
                ins.push(tm);
            }
        }
        if ins.is_empty() || ts.len() > 4096 {
            if !ins.is_empty() && std::env::var("CADRS_PCURVE_DEBUG").is_ok() {
                eprintln!("pcurve: no fit after {} points ({} off) of {c:?} on {s:?}", ts.len(), ins.len());
            }
            return Curve::Nurbs(Arc::new(fit));
        }
        ts.extend(ins);
        ts.sort_by(f64::total_cmp);
    }
    Curve::Nurbs(Arc::new(fit_at(&ts).0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(s: &Surface, c: &Curve, t0: f64, t1: f64) {
        let pc = pcurve(s, c, t0, t1, None);
        for i in 0..=20 {
            let t = t0 + (t1 - t0) * i as f64 / 20.0;
            let q = pc.point(t);
            let p = s.eval(q.x, q.y);
            assert!((p - c.point(t)).norm() < 1e-8, "{s:?} {c:?} at {t}: {}", (p - c.point(t)).norm());
        }
    }

    #[test]
    fn pcurves_lie_on_their_curves() {
        let f = Frame::from_xz(P3::new(1.0, 2.0, 3.0), V3::new(1.0, 0.2, 0.0), V3::new(0.1, 0.0, 1.0));
        let cyl = Surface::Cylinder { f, r: 5.0 };
        let rim = Curve::Circle { c: f.o + f.z * 2.0, x: f.y, y: -f.x, r: 5.0 };
        check(&cyl, &rim, 0.3, 4.0);
        let ruling = Curve::Line { o: f.o + f.x * 5.0, d: -f.z };
        check(&cyl, &ruling, -3.0, 2.0);
        // An oblique ellipse: fitted.
        let pl = Surface::Plane { f: Frame::from_z(f.o, V3::new(0.3, 0.1, 1.0)) };
        let e = super::super::ssi::surface_surface(&pl, ((0.0, 1.0), (0.0, 1.0)), &cyl, ((0.0, TAU), (-10.0, 10.0)), &Bbox { min: P3::new(-20.0, -20.0, -20.0), max: P3::new(20.0, 20.0, 20.0) });
        assert_eq!(e.len(), 1);
        check(&cyl, &e[0].c, 0.0, TAU);
        check(&pl, &e[0].c, 0.0, TAU);
        let sph = Surface::Sphere { f, r: 5.0 };
        let lat = Curve::Circle { c: f.o + f.z * 3.0, x: f.x, y: f.y, r: 4.0 };
        check(&sph, &lat, 0.0, TAU);
        let mer = Curve::Circle { c: f.o, x: f.x, y: f.z, r: 5.0 };
        check(&sph, &mer, -1.0, 1.0);
        let tor = Surface::Torus { f, rmaj: 10.0, rmin: 2.0 };
        let m2 = Curve::Circle { c: f.o + f.y * 10.0, x: f.y, y: f.z, r: 2.0 };
        check(&tor, &m2, 0.0, TAU);
        let cone = Surface::Cone { f, r: 3.0, a: 0.3 };
        let ruling = Curve::Line { o: cone.eval(1.0, 0.0), d: cone.d1(1.0, 0.0).2 };
        check(&cone, &ruling, -2.0, 2.0);
    }
}
