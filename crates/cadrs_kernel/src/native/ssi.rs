//! Intersections: surface–surface (analytic cases exactly, the rest by contouring the signed
//! distance over one surface's parameter box and refining onto both surfaces), curve–surface
//! and curve–curve.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use rayon::prelude::*;
use super::nurbs::NurbsCurve;
use super::surface::Surface;

/// A piece of an intersection curve.
#[derive(Debug, Clone)]
pub struct IntCurve {
    pub c: Curve,
    pub t0: f64,
    pub t1: f64,
    pub closed: bool,
}

/// The intersection of two surfaces near the box `bb` (the overlap of the two faces' boxes),
/// given the uv boxes of the faces for the numeric fallback.
pub fn surface_surface(a: &Surface, ua: ((f64, f64), (f64, f64)), b: &Surface, ub: ((f64, f64), (f64, f64)), bb: &Bbox) -> Vec<IntCurve> {
    if let Some(r) = plane_extrusion(a, b, ub, bb) {
        return r;
    }
    if let Some(r) = plane_extrusion(b, a, ua, bb) {
        return r;
    }
    if let Some(r) = analytic(a, b, bb) {
        return r;
    }
    if let Some(r) = analytic(b, a, bb) {
        return r;
    }
    // Numeric: contour over a surface without an implicit form against the other's exact
    // signed distance when it has one.
    let implicit = |s: &Surface| s.signed_distance(&P3::origin()).is_some();
    if implicit(a) && !implicit(b) {
        return numeric(b, ub, a, bb);
    }
    numeric(a, ua, b, bb)
}

/// A plane and an extrusion: rulings where the plane contains the direction, otherwise the
/// base curve projected along the direction onto the plane (an affine image: exact).
fn plane_extrusion(p: &Surface, e: &Surface, ue: ((f64, f64), (f64, f64)), bb: &Bbox) -> Option<Vec<IntCurve>> {
    let (Surface::Plane { f }, Surface::Extrusion { c, d }) = (p, e) else { return None };
    let n = f.x.cross(&f.y);
    let (u0, u1) = ue.0;
    let pad = (u1 - u0).abs() * 1e-6;
    let (u0, u1) = (u0 - pad, u1 + pad);
    let dn = n.dot(d);
    if dn.abs() < 1e-12 * d.norm() {
        // Rulings through the points where the base curve meets the plane.
        let pl = Surface::Plane { f: *f };
        let bbig = bb.grow(bb.diagonal() * 0.01 + 1e-6);
        let mut out = Vec::new();
        let (lo, hi) = match c.nurbs_range() {
            Some((a, b)) => (u0.max(a), u1.min(b)),
            None => (u0, u1),
        };
        for t in curve_surface(c, lo, hi, &pl) {
            out.extend(bound_line(c.point(t), d.normalize(), &bbig));
        }
        return Some(out);
    }
    // x ↦ x − d (n·(x − o)) / (n·d): the projection along d onto the plane.
    let m = nalgebra::Matrix3::identity() - d * n.transpose() / dn;
    let aff = Affine { m, t: d * (n.dot(&f.o.coords) / dn) };
    let (lo, hi) = match c.nurbs_range() {
        Some((a, b)) => (u0.max(a), u1.min(b)),
        None => (u0, u1),
    };
    if hi <= lo {
        return Some(vec![]);
    }
    let curve = match c {
        Curve::Line { o, d: dl } => {
            let o2 = aff.point(o);
            let d2 = m * dl;
            Curve::Line { o: o2, d: d2 }
        }
        Curve::Circle { .. } | Curve::Ellipse { .. } => Curve::Nurbs(Arc::new(c.to_nurbs(lo, hi).transformed(&aff))),
        Curve::Nurbs(nc) => Curve::Nurbs(Arc::new(nc.transformed(&aff))),
    };
    let closed = c.is_periodic() && (hi - lo) >= TAU - 1e-9;
    Some(vec![IntCurve { c: curve, t0: lo, t1: hi, closed }])
}

/// A plane and a cone meet in a conic: in the cone's frame, with g(u) = n_x cos u + n_y sin u,
/// the plane n·X = c gives v = (c − r g)/(s g + n_z k) (s, k the sine and cosine of the half
/// angle), so X·D = (A cos u, A sin u, k(c − r g)) with D = s g + n_z k and A = r n_z k + s c:
/// with the half-angle substitution a rational quadratic in t, exact as rational Bézier pieces.
/// The pieces cover the parts of the curve inside `bb`. None when the plane passes through the
/// apex (A = 0: lines, handled elsewhere or numerically).
fn plane_cone(fp: &Frame, fc: &Frame, r: f64, ang: f64, bb: &Bbox) -> Option<Vec<IntCurve>> {
    use super::nurbs::{NurbsCurve, H};
    let n = plane_n(fp);
    let nl = V3::new(n.dot(&fc.x), n.dot(&fc.y), n.dot(&fc.z));
    let c = n.dot(&(fp.o - fc.o));
    let (s, k) = ang.sin_cos();
    let q = nl.z * k;
    let a = r * q + s * c;
    if a.abs() < 1e-9 * (1.0 + r.abs() + c.abs()) {
        return None;
    }
    let local_point = |u: f64| -> Option<P3> {
        let g = nl.x * u.cos() + nl.y * u.sin();
        let d = s * g + q;
        if d.abs() < 1e-14 {
            return None;
        }
        let v = (c - r * g) / d;
        let rho = r + s * v;
        Some(fc.o + (fc.x * u.cos() + fc.y * u.sin()) * rho + fc.z * (k * v))
    };
    let inside = |u: f64| local_point(u).is_some_and(|p| bb.contains(&p));
    // Runs of u inside the box.
    let m = 360;
    let us: Vec<f64> = (0..m).map(|i| TAU * i as f64 / m as f64).collect();
    let ins: Vec<bool> = us.iter().map(|&u| inside(u)).collect();
    if !ins.iter().any(|x| *x) {
        return Some(vec![]);
    }
    let edge_of = |a: f64, b: f64| -> f64 {
        // The boundary between an inside `a` and an outside `b`.
        let (mut lo, mut hi) = (a, b);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if inside(mid) { lo = mid } else { hi = mid }
        }
        lo
    };
    let mut runs: Vec<(f64, f64, bool)> = Vec::new();
    if ins.iter().all(|x| *x) {
        runs.push((0.0, TAU, true));
    } else {
        let start = (0..m).find(|&i| !ins[i]).unwrap();
        let mut i = 0;
        while i < m {
            let j = (start + i) % m;
            if ins[j] {
                let mut e = i;
                while e + 1 < m && ins[(start + e + 1) % m] {
                    e += 1;
                }
                let u_in0 = us[j] + if (start + i) >= m { TAU } else { 0.0 };
                let first = (start + i) as f64 * TAU / m as f64;
                let last = (start + e) as f64 * TAU / m as f64;
                let _ = u_in0;
                let ua = edge_of(first, first - TAU / m as f64);
                let ub = edge_of(last, last + TAU / m as f64);
                runs.push((ua, ub, false));
                i = e + 1;
            } else {
                i += 1;
            }
        }
    }
    let to_world = |h: &nalgebra::Vector4<f64>| -> H {
        let p = fc.o.coords * h.w + fc.x * h.x + fc.y * h.y + fc.z * h.z;
        H::new(p.x, p.y, p.z, h.w)
    };
    // One rational quadratic Bézier piece on [ua, ub] (|ub − ua| < π), homogeneous, local.
    let piece = |ua: f64, ub: f64| -> [nalgebra::Vector4<f64>; 3] {
        let um = 0.5 * (ua + ub);
        let tt = (0.25 * (ub - ua)).tan();
        let (cm, sm) = (um.cos(), um.sin());
        let at = |tau: f64| -> nalgebra::Vector4<f64> {
            let t = -tt + 2.0 * tt * tau;
            let (c1, s1, one) = (1.0 - t * t, 2.0 * t, 1.0 + t * t);
            let cu = cm * c1 - sm * s1;
            let su = sm * c1 + cm * s1;
            let g = nl.x * cu + nl.y * su;
            nalgebra::Vector4::new(a * cu, a * su, k * (c * one - r * g), s * g + q * one)
        };
        let (p0, pm, p1) = (at(0.0), at(0.5), at(1.0));
        [p0, pm * 2.0 - (p0 + p1) * 0.5, p1]
    };
    let mut out = Vec::new();
    for (ua, ub, closed) in runs {
        if ub - ua < 1e-9 {
            continue;
        }
        // Pieces of at most π/4 whose weights share one sign.
        let mut spans: Vec<(f64, f64)> = Vec::new();
        let n0 = ((ub - ua) / (PI / 4.0)).ceil().max(1.0) as usize;
        let mut stack: Vec<(f64, f64)> = (0..n0).rev().map(|i| (ua + (ub - ua) * i as f64 / n0 as f64, ua + (ub - ua) * (i + 1) as f64 / n0 as f64)).collect();
        while let Some((x, y)) = stack.pop() {
            let b = piece(x, y);
            let pos = b.iter().all(|h| h.w > 1e-300);
            let neg = b.iter().all(|h| h.w < -1e-300);
            if pos || neg || y - x < 1e-6 {
                spans.push((x, y));
            } else {
                let mid = 0.5 * (x + y);
                stack.push((mid, y));
                stack.push((x, mid));
            }
        }
        let mut cps: Vec<H> = Vec::new();
        let mut knots = vec![0.0; 3];
        for (i, &(x, y)) in spans.iter().enumerate() {
            let mut b = piece(x, y);
            if b[0].w < 0.0 {
                for h in b.iter_mut() {
                    *h = -*h;
                }
            }
            if let Some(last) = cps.last() {
                let f = last.w / b[0].w;
                for h in b.iter_mut() {
                    *h *= f;
                }
                cps.push(to_world(&b[1]));
                cps.push(to_world(&b[2]));
            } else {
                cps.extend(b.iter().map(|h| to_world(h)));
            }
            let kv = (i + 1) as f64;
            if i + 1 < spans.len() {
                knots.extend([kv, kv]);
            } else {
                knots.extend([kv; 3]);
            }
        }
        let nc = NurbsCurve { degree: 2, knots, cps };
        let (t0, t1) = nc.range();
        out.push(IntCurve { c: Curve::Nurbs(std::sync::Arc::new(nc)), t0, t1, closed });
    }
    Some(out)
}

fn bound_line(o: P3, d: V3, bb: &Bbox) -> Option<IntCurve> {
    // Clip the line to the box (slabs).
    let (mut t0, mut t1) = (f64::NEG_INFINITY, f64::INFINITY);
    for k in 0..3 {
        let (lo, hi) = (bb.min[k], bb.max[k]);
        if d[k].abs() < 1e-14 {
            if o[k] < lo - 1e-9 || o[k] > hi + 1e-9 {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo - o[k]) / d[k], (hi - o[k]) / d[k]);
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        t0 = t0.max(a);
        t1 = t1.min(b);
    }
    if t1 - t0 < 1e-9 {
        return None;
    }
    Some(IntCurve { c: Curve::Line { o, d }, t0, t1, closed: false })
}

fn circle(c: P3, n: V3, r: f64) -> IntCurve {
    let f = Frame::from_z(c, n);
    IntCurve { c: Curve::Circle { c, x: f.x, y: f.y, r }, t0: 0.0, t1: TAU, closed: true }
}

fn plane_n(f: &Frame) -> V3 {
    f.x.cross(&f.y)
}

/// Closed forms. `None` when the pair isn't handled here (try the other order, then numeric).
fn analytic(a: &Surface, b: &Surface, bb: &Bbox) -> Option<Vec<IntCurve>> {
    let bbig = bb.grow(bb.diagonal() * 0.01 + 1e-6);
    match (a, b) {
        (Surface::Plane { f: fa }, Surface::Plane { f: fb }) => {
            let (na, nb) = (plane_n(fa), plane_n(fb));
            let d = na.cross(&nb);
            if d.norm() < 1e-12 {
                return Some(vec![]);
            }
            let d = d.normalize();
            // A point on both planes.
            let (ha, hb) = (na.dot(&fa.o.coords), nb.dot(&fb.o.coords));
            let n12 = na.dot(&nb);
            let det = 1.0 - n12 * n12;
            let c1 = (ha - hb * n12) / det;
            let c2 = (hb - ha * n12) / det;
            let p = P3::from(na * c1 + nb * c2);
            // Start the line near the box centre for good conditioning.
            let p = p + d * (bb.center() - p).dot(&d);
            Some(bound_line(p, d, &bbig).into_iter().collect())
        }
        (Surface::Plane { f: fp }, Surface::Cylinder { f: fc, r }) => {
            let n = plane_n(fp);
            let z = fc.z;
            let cz = n.dot(&z);
            if (1.0 - cz.abs()) < 1e-12 {
                // Square to the axis: a circle.
                let t = (fp.o - fc.o).dot(&n) / cz;
                let c = fc.o + z * t;
                return Some(vec![IntCurve { c: Curve::Circle { c, x: fc.x, y: fc.y, r: *r }, t0: 0.0, t1: TAU, closed: true }]);
            }
            if cz.abs() < 1e-12 {
                // Parallel to the axis: lines where the plane cuts the circle.
                let d = (fc.o - fp.o).dot(&n); // signed distance of the axis from the plane
                if d.abs() > r + 1e-12 {
                    return Some(vec![]);
                }
                let foot = fc.o - n * d;
                let w = z.cross(&n).normalize();
                let h = (r * r - d * d).max(0.0).sqrt();
                let mut out = Vec::new();
                if h < 1e-9 {
                    out.extend(bound_line(foot, z, &bbig));
                } else {
                    out.extend(bound_line(foot + w * h, z, &bbig));
                    out.extend(bound_line(foot - w * h, z, &bbig));
                }
                return Some(out);
            }
            // Oblique: an ellipse centred where the axis meets the plane.
            let t = (fp.o - fc.o).dot(&n) / cz;
            let c = fc.o + z * t;
            let minor = z.cross(&n).normalize(); // in the plane, square to the axis
            let major = n.cross(&minor).normalize(); // in the plane, along the tilt
            let a_len = r / cz.abs();
            // A plane almost along the axis: the ellipse is far bigger than the region the faces
            // span (parameters on it are ill-conditioned there); its part in the box as exact
            // rational pieces instead (a cylinder is a cone of half angle 0).
            if a_len > 4.0 * bbig.diagonal() {
                return plane_cone(fp, fc, *r, 0.0, &bbig);
            }
            Some(vec![IntCurve { c: Curve::Ellipse { c, x: major, y: minor, a: a_len, b: *r }, t0: 0.0, t1: TAU, closed: true }])
        }
        (Surface::Plane { f: fp }, Surface::Sphere { f: fs, r }) => {
            let n = plane_n(fp);
            let d = (fs.o - fp.o).dot(&n);
            if d.abs() >= *r - 1e-12 {
                return Some(vec![]);
            }
            Some(vec![circle(fs.o - n * d, n, (r * r - d * d).sqrt())])
        }
        (Surface::Plane { f: fp }, Surface::Cone { f: fc, r, a: ang }) => {
            let n = plane_n(fp);
            let cz = n.dot(&fc.z);
            if (1.0 - cz.abs()) < 1e-12 {
                // Square to the axis: v = height / cos a; radius r + v sin a.
                let h = (fp.o - fc.o).dot(&fc.z);
                let v = h / ang.cos();
                let rr = r + v * ang.sin();
                if rr <= 1e-12 {
                    return Some(vec![]);
                }
                let c = fc.o + fc.z * h;
                return Some(vec![IntCurve { c: Curve::Circle { c, x: fc.x, y: fc.y, r: rr }, t0: 0.0, t1: TAU, closed: true }]);
            }
            // Through the apex and containing the axis: two rulings.
            let apex = fc.o - fc.z * (r / ang.tan());
            if cz.abs() < 1e-12 && (apex - fp.o).dot(&n).abs() < 1e-9 {
                let w = fc.z.cross(&n).normalize();
                let open = if ang.tan() * ang.cos().signum() > 0.0 { fc.z } else { -fc.z };
                let k = ang.tan().abs();
                let mut out = Vec::new();
                for s in [1.0, -1.0] {
                    let d = (open + w * (k * s)).normalize();
                    if let Some(mut l) = bound_line(apex, d, &bbig) {
                        l.t0 = l.t0.max(0.0);
                        if l.t1 > l.t0 + 1e-9 {
                            out.push(l);
                        }
                    }
                }
                return Some(out);
            }
            plane_cone(fp, fc, *r, *ang, &bbig)
        }
        (Surface::Plane { f: fp }, Surface::Torus { f: ft, rmaj, rmin }) => {
            let n = plane_n(fp);
            let cz = n.dot(&ft.z);
            if (1.0 - cz.abs()) < 1e-12 {
                let h = (fp.o - ft.o).dot(&ft.z);
                if h.abs() >= *rmin - 1e-12 {
                    return Some(vec![]);
                }
                let w = (rmin * rmin - h * h).sqrt();
                let c = ft.o + ft.z * h;
                return Some(vec![
                    IntCurve { c: Curve::Circle { c, x: ft.x, y: ft.y, r: rmaj + w }, t0: 0.0, t1: TAU, closed: true },
                    IntCurve { c: Curve::Circle { c, x: ft.x, y: ft.y, r: rmaj - w }, t0: 0.0, t1: TAU, closed: true },
                ]);
            }
            if cz.abs() < 1e-12 && (ft.o - fp.o).dot(&n).abs() < 1e-9 {
                // A plane through the axis: two meridian circles.
                let w = ft.z.cross(&n).normalize();
                return Some(vec![
                    IntCurve { c: Curve::Circle { c: ft.o + w * *rmaj, x: w, y: ft.z, r: *rmin }, t0: 0.0, t1: TAU, closed: true },
                    IntCurve { c: Curve::Circle { c: ft.o - w * *rmaj, x: -w, y: ft.z, r: *rmin }, t0: 0.0, t1: TAU, closed: true },
                ]);
            }
            None
        }
        (Surface::Cylinder { f: f1, r: r1 }, Surface::Cylinder { f: f2, r: r2 }) => {
            if f1.z.cross(&f2.z).norm() > 1e-12 {
                return None;
            }
            // Parallel axes: lines.
            let d = f2.o - f1.o;
            let d = d - f1.z * d.dot(&f1.z);
            let l = d.norm();
            if l < 1e-12 {
                return Some(vec![]); // coaxial: no curve (or the same surface)
            }
            if l > r1 + r2 + 1e-12 || l < (r1 - r2).abs() - 1e-12 {
                return Some(vec![]);
            }
            let x = (l * l + r1 * r1 - r2 * r2) / (2.0 * l);
            let h = (r1 * r1 - x * x).max(0.0).sqrt();
            let ex = d / l;
            let ey = f1.z.cross(&ex);
            let base = f1.o + ex * x;
            let mut out = Vec::new();
            if h < 1e-9 {
                out.extend(bound_line(base, f1.z, &bbig));
            } else {
                out.extend(bound_line(base + ey * h, f1.z, &bbig));
                out.extend(bound_line(base - ey * h, f1.z, &bbig));
            }
            Some(out)
        }
        (Surface::Sphere { f: f1, r: r1 }, Surface::Sphere { f: f2, r: r2 }) => {
            let d = f2.o - f1.o;
            let l = d.norm();
            if l < 1e-12 || l > r1 + r2 || l < (r1 - r2).abs() {
                return Some(vec![]);
            }
            let x = (l * l + r1 * r1 - r2 * r2) / (2.0 * l);
            Some(vec![circle(f1.o + d / l * x, d / l, (r1 * r1 - x * x).max(0.0).sqrt())])
        }
        _ => {
            // Coaxial surfaces of revolution: circles where the meridians cross. A sphere is
            // symmetric about any axis through its centre.
            let (Some((mut oa, mut za)), Some((ob, zb))) = (a.axis(), b.axis()) else { return None };
            if let Surface::Sphere { f, .. } = a {
                let w = f.o - ob;
                if (w - zb * w.dot(&zb)).norm() < 1e-9 {
                    oa = ob;
                    za = zb;
                }
            }
            let (ob, zb) = if let Surface::Sphere { f, .. } = b {
                let w = f.o - oa;
                if (w - za * w.dot(&za)).norm() < 1e-9 { (oa, za) } else { (ob, zb) }
            } else {
                (ob, zb)
            };
            if !matches!(a, Surface::Cylinder { .. } | Surface::Cone { .. } | Surface::Sphere { .. } | Surface::Torus { .. }) {
                return None;
            }
            if !matches!(b, Surface::Cylinder { .. } | Surface::Cone { .. } | Surface::Sphere { .. } | Surface::Torus { .. }) {
                return None;
            }
            if za.cross(&zb).norm() > 1e-12 {
                return None;
            }
            let w = ob - oa;
            if (w - za * w.dot(&za)).norm() > 1e-9 {
                return None;
            }
            Some(coaxial(a, b, &oa, &za, bb))
        }
    }
}

/// The meridian of an analytic surface of revolution about (o, z) in the (ρ, h) half plane:
/// a line (cylinder, cone, plane square to the axis) or a circle (sphere, torus).
#[derive(Debug, Clone, Copy)]
enum Meridian {
    /// Points p + s·d.
    Line { p: (f64, f64), d: (f64, f64) },
    Circle { c: (f64, f64), r: f64 },
}

fn meridian(s: &Surface, o: &P3, z: &V3) -> Option<Meridian> {
    let h_of = |p: &P3| (p - o).dot(z);
    Some(match s {
        Surface::Cylinder { r, .. } => Meridian::Line { p: (*r, 0.0), d: (0.0, 1.0) },
        Surface::Cone { f, r, a } => {
            let s = f.z.dot(z).signum();
            Meridian::Line { p: (*r, h_of(&f.o)), d: (a.sin(), a.cos() * s) }
        }
        Surface::Sphere { f, r } => Meridian::Circle { c: (0.0, h_of(&f.o)), r: *r },
        Surface::Torus { f, rmaj, rmin } => Meridian::Circle { c: (*rmaj, h_of(&f.o)), r: *rmin },
        Surface::Plane { f } => {
            let n = f.x.cross(&f.y);
            if n.cross(z).norm() > 1e-12 {
                return None;
            }
            Meridian::Line { p: (0.0, h_of(&f.o)), d: (1.0, 0.0) }
        }
        _ => return None,
    })
}

/// Intersections of two meridians (ρ, h), tangencies included (within `tol`).
fn meridian_points(a: Meridian, b: Meridian, tol: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    match (a, b) {
        (Meridian::Line { p: p1, d: d1 }, Meridian::Line { p: p2, d: d2 }) => {
            let den = d1.0 * d2.1 - d1.1 * d2.0;
            if den.abs() < 1e-14 {
                return out;
            }
            let (wx, wy) = (p2.0 - p1.0, p2.1 - p1.1);
            let s = (wx * d2.1 - wy * d2.0) / den;
            out.push((p1.0 + d1.0 * s, p1.1 + d1.1 * s));
        }
        (Meridian::Line { p, d }, Meridian::Circle { c, r }) | (Meridian::Circle { c, r }, Meridian::Line { p, d }) => {
            let l = (d.0 * d.0 + d.1 * d.1).sqrt();
            let (dx, dy) = (d.0 / l, d.1 / l);
            let (wx, wy) = (p.0 - c.0, p.1 - c.1);
            let b = wx * dx + wy * dy;
            let cc = wx * wx + wy * wy - r * r;
            let disc = b * b - cc;
            if disc < -2.0 * r * tol {
                return out;
            }
            if disc.abs() <= 2.0 * r * tol {
                let s = -b;
                out.push((p.0 + dx * s, p.1 + dy * s));
            } else {
                let q = disc.sqrt();
                for s in [-b - q, -b + q] {
                    out.push((p.0 + dx * s, p.1 + dy * s));
                }
            }
        }
        (Meridian::Circle { c: c1, r: r1 }, Meridian::Circle { c: c2, r: r2 }) => {
            let (dx, dy) = (c2.0 - c1.0, c2.1 - c1.1);
            let d = (dx * dx + dy * dy).sqrt();
            if d < 1e-14 {
                return out;
            }
            if d > r1 + r2 + tol || d < (r1 - r2).abs() - tol {
                return out;
            }
            let x = (d * d + r1 * r1 - r2 * r2) / (2.0 * d);
            let h2 = r1 * r1 - x * x;
            let (ex, ey) = (dx / d, dy / d);
            let base = (c1.0 + ex * x, c1.1 + ey * x);
            if h2 <= (r1 * tol * 2.0).max(1e-18) {
                out.push(base);
            } else {
                let h = h2.sqrt();
                out.push((base.0 - ey * h, base.1 + ex * h));
                out.push((base.0 + ey * h, base.1 - ex * h));
            }
        }
    }
    out
}

fn coaxial(a: &Surface, b: &Surface, o: &P3, z: &V3, bb: &Bbox) -> Vec<IntCurve> {
    let (Some(ma), Some(mb)) = (meridian(a, o, z), meridian(b, o, z)) else { return vec![] };
    let f = Frame::from_z(*o, *z);
    let _ = bb;
    meridian_points(ma, mb, 1e-7)
        .into_iter()
        .filter(|(r, _)| *r > 1e-9)
        .map(|(r, h)| IntCurve { c: Curve::Circle { c: o + z * h, x: f.x, y: f.y, r }, t0: 0.0, t1: TAU, closed: true })
        .collect()
}

/// The signed distance to `s` by projection from a hint (`hint` is updated with the foot's uv).
pub fn sdist_hint(s: &Surface, p: &P3, hint: &mut Option<(f64, f64)>) -> f64 {
    if let Some(d) = s.signed_distance(p) {
        return d;
    }
    let (mut u, mut v) = s.project(p, *hint);
    let (mut q, mut su, mut sv) = s.d1(u, v);
    // A hinted Newton may stall on the wrong side of a fold or on the domain's edge: when the
    // foot isn't square to the surface, check against a projection from a coarse grid.
    let r = p - q;
    let tang = (r.dot(&su) / su.norm().max(1e-300)).abs() + (r.dot(&sv) / sv.norm().max(1e-300)).abs();
    if hint.is_some() && tang > 1e-7 * (1.0 + r.norm()) {
        let (u2, v2) = s.project_coarse(p, 8);
        let (q2, su2, sv2) = s.d1(u2, v2);
        if (p - q2).norm() < r.norm() - 1e-12 {
            (u, v, q, su, sv) = (u2, v2, q2, su2, sv2);
        }
    }
    let n = su.cross(&sv);
    let l = n.norm();
    *hint = Some((u, v));
    if l < 1e-300 {
        return (p - q).norm();
    }
    (p - q).dot(&(n / l))
}

/// The signed distance to `s` (by projection when it has no implicit form).
pub fn sdist(s: &Surface, p: &P3) -> f64 {
    if let Some(d) = s.signed_distance(p) {
        return d;
    }
    let (u, v) = s.project(p, None);
    let (q, su, sv) = s.d1(u, v);
    let n = su.cross(&sv);
    let l = n.norm();
    if l < 1e-300 {
        return (p - q).norm();
    }
    (p - q).dot(&(n / l))
}

/// Contours sd_b(S_a(u, v)) = 0 over a's uv box, refines and fits B-splines.
fn numeric(a: &Surface, ua: ((f64, f64), (f64, f64)), b: &Surface, bb: &Bbox) -> Vec<IntCurve> {
    let ((u0, u1), (v0, v1)) = ua;
    let (mu, mv) = a.metric();
    let ext = ((u1 - u0) * mu).max((v1 - v0) * mv).max(1e-9);
    let size = bb.diagonal().max(ext);
    let h = size / 96.0;
    let nu = (((u1 - u0) * mu / h).ceil() as usize).clamp(8, 400);
    let nv = (((v1 - v0) * mv / h).ceil() as usize).clamp(8, 400);
    let mut grid = vec![vec![0.0; nv + 1]; nu + 1];
    let mut row_hint: Option<(f64, f64)> = None;
    for i in 0..=nu {
        let mut hint = row_hint;
        for j in 0..=nv {
            let p = a.eval(u0 + (u1 - u0) * i as f64 / nu as f64, v0 + (v1 - v0) * j as f64 / nv as f64);
            grid[i][j] = sdist_hint(b, &p, &mut hint);
            if j == 0 {
                row_hint = hint;
            }
        }
    }
    let uvat = |i: f64, j: f64| P2::new(u0 + (u1 - u0) * i / nu as f64, v0 + (v1 - v0) * j / nv as f64);
    // Marching squares: segments between edge crossings.
    let mut segs: Vec<(P2, P2)> = Vec::new();
    let cross = |ga: f64, gb: f64| -> f64 { ga / (ga - gb) };
    for i in 0..nu {
        for j in 0..nv {
            let c = [grid[i][j], grid[i + 1][j], grid[i + 1][j + 1], grid[i][j + 1]];
            let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
            let mut pts: Vec<P2> = Vec::new();
            for k in 0..4 {
                let (ga, gb) = (c[k], c[(k + 1) % 4]);
                if (ga < 0.0) != (gb < 0.0) {
                    let t = cross(ga, gb);
                    let (a0, a1) = (corners[k], corners[(k + 1) % 4]);
                    pts.push(uvat(i as f64 + a0.0 + (a1.0 - a0.0) * t, j as f64 + a0.1 + (a1.1 - a0.1) * t));
                }
            }
            if pts.len() == 2 {
                segs.push((pts[0], pts[1]));
            } else if pts.len() == 4 {
                // Saddle: pair by the centre's sign.
                let cen = (c[0] + c[1] + c[2] + c[3]) / 4.0;
                if (cen < 0.0) == (c[0] < 0.0) {
                    segs.push((pts[0], pts[3]));
                    segs.push((pts[1], pts[2]));
                } else {
                    segs.push((pts[0], pts[1]));
                    segs.push((pts[2], pts[3]));
                }
            }
        }
    }
    if segs.is_empty() {
        return vec![];
    }
    // Link segments into polylines.
    let key = |p: &P2| ((p.x * 1e9).round() as i64, (p.y * 1e9).round() as i64);
    let mut by_end: std::collections::HashMap<(i64, i64), Vec<usize>> = std::collections::HashMap::new();
    for (k, (p, q)) in segs.iter().enumerate() {
        by_end.entry(key(p)).or_default().push(k);
        by_end.entry(key(q)).or_default().push(k);
    }
    let mut used = vec![false; segs.len()];
    let mut lines: Vec<(Vec<P2>, bool)> = Vec::new();
    for s in 0..segs.len() {
        if used[s] {
            continue;
        }
        used[s] = true;
        let mut pl = vec![segs[s].0, segs[s].1];
        // Extend forward then backward.
        for dir in 0..2 {
            loop {
                let end = if dir == 0 { *pl.last().unwrap() } else { pl[0] };
                let next = by_end.get(&key(&end)).and_then(|v| v.iter().find(|k| !used[**k]).copied());
                let Some(k) = next else { break };
                used[k] = true;
                let (p, q) = segs[k];
                let other = if key(&p) == key(&end) { q } else { p };
                if dir == 0 {
                    pl.push(other);
                } else {
                    pl.insert(0, other);
                }
            }
        }
        let closed = pl.len() > 3 && key(&pl[0]) == key(pl.last().unwrap());
        if closed {
            pl.pop();
        }
        lines.push((pl, closed));
    }
    // Refine onto both surfaces, in 3D.
    // Ends on the box's sides stay on them (Newton along the side), so the curve reaches the
    // face's boundary there.
    let (tu, tv) = ((u1 - u0) * 1e-9, (v1 - v0) * 1e-9);
    let refine = |uv: P2| -> P3 {
        let side_u = (uv.x - u0).abs() < tu || (uv.x - u1).abs() < tu;
        let side_v = (uv.y - v0).abs() < tv || (uv.y - v1).abs() < tv;
        if side_u != side_v {
            if let Some(p) = refine_on_side(a, b, uv, side_u) {
                return p;
            }
        }
        refine_point(a, b, uv)
    };
    let mut polys: Vec<(Vec<P3>, bool)> = lines.into_iter().map(|(pl, c)| (pl.into_iter().map(refine).collect(), c)).collect();
    // Join polylines that meet in 3D (across a periodic seam).
    join_polylines(&mut polys, h * 1e-3);
    polys.into_iter().filter(|(p, _)| p.len() >= 2).map(|(p, closed)| fit_curve(a, b, p, closed)).collect()
}

/// A point on both surfaces on a's iso line through `uv` (u fixed when `fix_u`): Newton on the
/// other parameter for sd_b = 0. `None` when it doesn't converge.
fn refine_on_side(a: &Surface, b: &Surface, uv: P2, fix_u: bool) -> Option<P3> {
    let (u, mut v) = if fix_u { (uv.x, uv.y) } else { (uv.y, uv.x) };
    let at = |w: f64| if fix_u { a.eval(u, w) } else { a.eval(w, u) };
    let mut hint: Option<(f64, f64)> = if b.signed_distance(&P3::origin()).is_some() { None } else { Some(b.project(&at(v), None)) };
    let e = 1e-7;
    for _ in 0..30 {
        let g0 = sdist_hint(b, &at(v), &mut hint);
        if g0.abs() < 1e-13 {
            return Some(at(v));
        }
        let (mut h1, mut h2) = (hint, hint);
        let d = (sdist_hint(b, &at(v + e), &mut h1) - sdist_hint(b, &at(v - e), &mut h2)) / (2.0 * e);
        if d.abs() < 1e-300 {
            return None;
        }
        v -= g0 / d;
    }
    let p = at(v);
    (sdist(b, &p).abs() < 1e-9).then_some(p)
}

/// A point on both surfaces near a's (u, v): Newton on a's parameters for sd_b = 0, then a
/// few alternating projections to tighten.
pub fn refine_point(a: &Surface, b: &Surface, uv: P2) -> P3 {
    refine_point_uv(a, b, uv).0
}

/// [`refine_point`] with the point's (u, v) on `a`.
pub fn refine_point_uv(a: &Surface, b: &Surface, uv: P2) -> (P3, P2) {
    let (mut u, mut v) = (uv.x, uv.y);
    let e = 1e-7;
    let mut hint: Option<(f64, f64)> = if b.signed_distance(&P3::origin()).is_some() { None } else { Some(b.project(&a.eval(u, v), None)) };
    for _ in 0..30 {
        let p = a.eval(u, v);
        let g0 = sdist_hint(b, &p, &mut hint);
        if g0.abs() < 1e-13 {
            break;
        }
        let (_, su, sv) = a.d1(u, v);
        // ∇g ≈ n_b·(Su, Sv) with n_b the distance gradient.
        let mut h1 = hint;
        let mut h2 = hint;
        let mut h3 = hint;
        let mut h4 = hint;
        let gu = (sdist_hint(b, &(p + su * e), &mut h1) - sdist_hint(b, &(p - su * e), &mut h2)) / (2.0 * e);
        let gv = (sdist_hint(b, &(p + sv * e), &mut h3) - sdist_hint(b, &(p - sv * e), &mut h4)) / (2.0 * e);
        let l = gu * gu + gv * gv;
        if l < 1e-300 {
            break;
        }
        u -= g0 * gu / l;
        v -= g0 * gv / l;
    }
    (a.eval(u, v), P2::new(u, v))
}

fn join_polylines(polys: &mut Vec<(Vec<P3>, bool)>, tol: f64) {
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for i in 0..polys.len() {
            if polys[i].1 {
                continue;
            }
            for j in 0..polys.len() {
                if i == j || polys[j].1 {
                    continue;
                }
                let (ai, bi) = (polys[i].0[0], *polys[i].0.last().unwrap());
                let (aj, bj) = (polys[j].0[0], *polys[j].0.last().unwrap());
                let mut pj = polys[j].0.clone();
                let mut pi = polys[i].0.clone();
                let joined = if (bi - aj).norm() < tol {
                    pi.extend(pj.drain(1..));
                    true
                } else if (bi - bj).norm() < tol {
                    pj.reverse();
                    pi.extend(pj.drain(1..));
                    true
                } else if (ai - bj).norm() < tol {
                    pj.extend(pi.drain(1..));
                    pi = pj.clone();
                    true
                } else if (ai - aj).norm() < tol {
                    pj.reverse();
                    pj.extend(pi.drain(1..));
                    pi = pj.clone();
                    true
                } else {
                    false
                };
                if joined {
                    let closed = (pi[0] - *pi.last().unwrap()).norm() < tol;
                    if closed {
                        pi.pop();
                    }
                    polys[i] = (pi, closed);
                    polys.remove(j);
                    changed = true;
                    break 'outer;
                }
            }
        }
        // Close single polylines whose ends meet.
        for p in polys.iter_mut() {
            if !p.1 && p.0.len() > 2 && (p.0[0] - *p.0.last().unwrap()).norm() < tol {
                p.0.pop();
                p.1 = true;
            }
        }
    }
}

/// Fits a cubic B-spline through exact intersection points, adding refined points where the
/// fit strays (> 1e-9) from the true intersection.
fn fit_curve(a: &Surface, b: &Surface, mut pts: Vec<P3>, closed: bool) -> IntCurve {
    if closed {
        pts.push(pts[0]);
    }
    // Remove near-duplicates.
    pts.dedup_by(|x, y| (*x - *y).norm() < 1e-9);
    // The points' (u, v) on a, each projection hinted by the previous one's.
    let mut uvs: Vec<P2> = Vec::with_capacity(pts.len());
    let mut hint: Option<(f64, f64)> = None;
    for p in &pts {
        let (u, v) = a.project(p, hint);
        hint = Some((u, v));
        uvs.push(P2::new(u, v));
    }
    for _round in 0..6 {
        let t = chord(&pts);
        let c = NurbsCurve::interpolate(&pts, &t, 3.min(pts.len() - 1), None, None);
        let found: Vec<Option<(usize, P3, P2)>> = (0..pts.len() - 1)
            .into_par_iter()
            .map(|k| {
                let tm = (t[k] + t[k + 1]) / 2.0;
                let q = c.point(tm);
                let (u, v) = a.project(&q, Some((uvs[k].x, uvs[k].y)));
                let (exact, uv) = refine_point_uv(a, b, P2::new(u, v));
                ((exact - q).norm() > 1e-9 && (pts[k + 1] - pts[k]).norm() > 1e-6).then_some((k + 1, exact, uv))
            })
            .collect();
        let ins: Vec<(usize, P3, P2)> = found.into_iter().flatten().collect();
        if ins.is_empty() || pts.len() > 20000 {
            let n = c.range();
            return IntCurve { c: Curve::Nurbs(Arc::new(c)), t0: n.0, t1: n.1, closed };
        }
        for (k, p, uv) in ins.into_iter().rev() {
            pts.insert(k, p);
            uvs.insert(k, uv);
        }
    }
    let t = chord(&pts);
    let c = NurbsCurve::interpolate(&pts, &t, 3.min(pts.len() - 1), None, None);
    let n = c.range();
    IntCurve { c: Curve::Nurbs(Arc::new(c)), t0: n.0, t1: n.1, closed }
}

fn chord(pts: &[P3]) -> Vec<f64> {
    let mut t = vec![0.0];
    for w in pts.windows(2) {
        t.push(t.last().unwrap() + (w[1] - w[0]).norm());
    }
    t
}

/// Parameters in [t0, t1] where the curve meets the surface (sign changes of the distance,
/// and touching points), refined.
pub fn curve_surface(c: &Curve, t0: f64, t1: f64, s: &Surface) -> Vec<f64> {
    // A circle or an ellipse against a plane: A cos t + B sin t + C = 0, tangency included.
    if let (Curve::Circle { .. } | Curve::Ellipse { .. }, Surface::Plane { f }) = (c, s) {
        let n = f.x.cross(&f.y);
        let (cc, ax, ay) = match c {
            Curve::Circle { c, x, y, r } => (*c, x * *r, y * *r),
            Curve::Ellipse { c, x, y, a, b } => (*c, x * *a, y * *b),
            _ => unreachable!(),
        };
        let (a, b, k) = (n.dot(&ax), n.dot(&ay), n.dot(&(cc - f.o)));
        let r = (a * a + b * b).sqrt();
        if r < 1e-14 {
            return vec![]; // in a plane parallel to it
        }
        let phi = b.atan2(a);
        let q = -k / r;
        let mut ts = Vec::new();
        if q.abs() > 1.0 + 1e-12 {
            return ts;
        }
        if (q.abs() - 1.0).abs() <= 1e-12 * 4.0 || (1.0 - q * q) * r * r < 1e-20 {
            ts.push(if q > 0.0 { phi } else { phi + PI });
        } else {
            let d = q.clamp(-1.0, 1.0).acos();
            ts.push(phi + d);
            ts.push(phi - d);
        }
        let mut out: Vec<f64> = Vec::new();
        for t in ts {
            for k in -2..=2 {
                let tt = t + TAU * k as f64;
                if tt >= t0 - 1e-12 && tt <= t1 + 1e-12 && !out.iter().any(|x| (x - tt).abs() < 1e-12) {
                    out.push(tt.clamp(t0, t1));
                }
            }
        }
        out.sort_by(f64::total_cmp);
        return out;
    }
    // Exact cases.
    if let (Curve::Line { o, d }, true) = (c, s.signed_distance(&P3::origin()).is_some()) {
        let ts: Vec<f64> = super::classify::ray_surface(s, o, d, t0 - 1.0, t1 + 1.0).into_iter().map(|(t, _, _)| t).collect();
        if !matches!(s, Surface::Torus { .. } | Surface::Revolution { .. } | Surface::Extrusion { .. } | Surface::Nurbs(_)) {
            return ts.into_iter().filter(|t| *t >= t0 - 1e-9 && *t <= t1 + 1e-9).map(|t| t.clamp(t0, t1)).collect();
        }
    }
    let n = match c {
        Curve::Line { .. } => 8,
        Curve::Circle { .. } | Curve::Ellipse { .. } => (((t1 - t0) / (PI / 32.0)).ceil() as usize).max(8),
        Curve::Nurbs(nc) => (nc.cps.len() * 8).clamp(16, 2000),
    };
    // The signed distance, its projection hinted (on B-spline and other non-implicit surfaces
    // a fresh projection is a global search): along the samples from the previous one, inside
    // a bracket from the bracket's own sample.
    let implicit = s.signed_distance(&P3::origin()).is_some();
    let fh = |t: f64, hint: &mut Option<(f64, f64)>| -> f64 { if implicit { sdist(s, &c.point(t)) } else { sdist_hint(s, &c.point(t), hint) } };
    let mut out: Vec<f64> = Vec::new();
    let mut hints: Vec<Option<(f64, f64)>> = Vec::with_capacity(n + 1);
    let mut h: Option<(f64, f64)> = None;
    let mut vals = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = t0 + (t1 - t0) * i as f64 / n as f64;
        let v = fh(t, &mut h);
        vals.push((t, v));
        hints.push(h);
    }
    let mut prev: (f64, f64);
    let f = |t: f64| -> f64 {
        // Hinted from the nearest sample.
        let i = (((t - t0) / (t1 - t0)) * n as f64).round().clamp(0.0, n as f64) as usize;
        let mut hh = hints[i];
        fh(t, &mut hh)
    };
    for i in 0..vals.len() {
        let (t, ft) = vals[i];
        if ft.abs() < 1e-10 {
            out.push(t);
        }
        if i > 0 {
            prev = vals[i - 1];
            if prev.1.abs() >= 1e-10 && ft.abs() >= 1e-10 && prev.1.signum() != ft.signum() {
                let (mut a, mut b, mut fa) = (prev.0, t, prev.1);
                for _ in 0..100 {
                    let m = 0.5 * (a + b);
                    let fm = f(m);
                    if fm == 0.0 {
                        a = m;
                        b = m;
                        break;
                    }
                    if fm.signum() == fa.signum() {
                        a = m;
                        fa = fm;
                    } else {
                        b = m;
                    }
                    if (b - a).abs() < 1e-15 * (1.0 + a.abs()) {
                        break;
                    }
                }
                out.push(0.5 * (a + b));
            }
        }
        // Touching: a local minimum of |f| near zero between samples.
        if i > 0 && i + 1 < vals.len() {
            let (fa, fb, fc) = (vals[i - 1].1.abs(), ft.abs(), vals[i + 1].1.abs());
            if fb < fa && fb < fc && fb < 1e-3 && vals[i - 1].1.signum() == vals[i + 1].1.signum() {
                // Golden-section minimize |f|.
                let (mut a, mut b) = (vals[i - 1].0, vals[i + 1].0);
                for _ in 0..80 {
                    let m1 = a + (b - a) * 0.382;
                    let m2 = a + (b - a) * 0.618;
                    if f(m1).abs() < f(m2).abs() {
                        b = m2;
                    } else {
                        a = m1;
                    }
                }
                let m = 0.5 * (a + b);
                if f(m).abs() < 1e-9 {
                    out.push(m);
                }
            }
        }
    }
    out.sort_by(f64::total_cmp);
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    out
}

/// Points where two bounded curves meet (within `tol`): (t on a, t on b).
pub fn curve_curve(a: &Curve, a0: f64, a1: f64, b: &Curve, b0: f64, b1: f64, tol: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    // Line–line in closed form.
    if let (Curve::Line { o: oa, d: da }, Curve::Line { o: ob, d: db }) = (a, b) {
        let w = oa - ob;
        let (aa, bb, cc) = (da.dot(da), da.dot(db), db.dot(db));
        let (dd, ee) = (da.dot(&w), db.dot(&w));
        let den = aa * cc - bb * bb;
        if den.abs() < 1e-14 * aa * cc {
            return out; // parallel: overlaps are found from the end points
        }
        let s = (bb * ee - cc * dd) / den;
        let t = (aa * ee - bb * dd) / den;
        if s >= a0 - 1e-9 && s <= a1 + 1e-9 && t >= b0 - 1e-9 && t <= b1 + 1e-9 {
            let (p, q) = (a.point(s), b.point(t));
            if (p - q).norm() < tol {
                out.push((s.clamp(a0, a1), t.clamp(b0, b1)));
            }
        }
        return out;
    }
    // A line and a circle in closed form (either order).
    let line_circle = |l: &Curve, l0: f64, l1: f64, c: &Curve, c0: f64, c1: f64| -> Option<Vec<(f64, f64)>> {
        let (Curve::Line { o, d }, Curve::Circle { c: cc, x, y, r }) = (l, c) else { return None };
        let n = x.cross(y);
        let dd = d.norm();
        let du = d / dd;
        let mut hits: Vec<P3> = Vec::new();
        if du.dot(&n).abs() < 1e-12 {
            // In a plane parallel to the circle's: only meets it in the circle's plane.
            if (o - cc).dot(&n).abs() > tol {
                return Some(vec![]);
            }
            let w = o - cc;
            let b = w.dot(&du);
            let disc = b * b - (w.norm_squared() - r * r);
            if disc < -2.0 * r * tol {
                return Some(vec![]);
            }
            if disc.abs() <= 2.0 * r * tol * 1e-3 || disc <= 0.0 {
                hits.push(o + du * (-b));
            } else {
                let q = disc.sqrt();
                hits.push(o + du * (-b - q));
                hits.push(o + du * (-b + q));
            }
        } else {
            let s = (cc - o).dot(&n) / du.dot(&n);
            hits.push(o + du * s);
        }
        let mut out = Vec::new();
        for p in hits {
            let w = p - cc;
            if (w.norm() - r).abs() > tol {
                continue;
            }
            let tl = (p - o).dot(d) / (dd * dd);
            let ang = w.dot(y).atan2(w.dot(x));
            let tc = super::curve::clamp_periodic(ang, c0, c1);
            if tl >= l0 - 1e-9 && tl <= l1 + 1e-9 && (c.point(tc) - p).norm() < tol {
                out.push((tl.clamp(l0, l1), tc));
            }
        }
        Some(out)
    };
    if let Some(r) = line_circle(a, a0, a1, b, b0, b1) {
        return r;
    }
    if let Some(r) = line_circle(b, b0, b1, a, a0, a1) {
        return r.into_iter().map(|(x, y)| (y, x)).collect();
    }
    // Two circles in closed form.
    if let (&Curve::Circle { c: ca, x: xa, y: ya, r: ra }, &Curve::Circle { c: cb, x: xb, y: yb, r: rb }) = (a, b) {
        let (na, nb) = (xa.cross(&ya), xb.cross(&yb));
        let mut hits: Vec<P3> = Vec::new();
        if na.cross(&nb).norm() < 1e-12 {
            // Parallel planes: they meet only in one plane.
            if (cb - ca).dot(&na).abs() > tol {
                return out;
            }
            let d = cb - ca;
            let d = d - na * d.dot(&na);
            let dl = d.norm();
            if dl < tol && (ra - rb).abs() < tol {
                return out; // the same circle: overlaps are found from the end points
            }
            if dl < 1e-300 || dl > ra + rb + tol || dl < (ra - rb).abs() - tol {
                return out;
            }
            // Radical line: distance along d from ca.
            let u = d / dl;
            let x = ((dl * dl + ra * ra - rb * rb) / (2.0 * dl)).clamp(-ra, ra);
            let h2 = ra * ra - x * x;
            let w = na.cross(&u);
            if h2 <= (ra * tol * 1e-3).max(0.0) {
                hits.push(ca + u * x);
            } else {
                let h = h2.sqrt();
                hits.push(ca + u * x + w * h);
                hits.push(ca + u * x - w * h);
            }
        } else {
            // A's points on B's plane: the line where the planes meet, through circle A.
            let dir = na.cross(&nb).normalize();
            // A point on both planes.
            let (da, db) = (na.dot(&ca.coords), nb.dot(&cb.coords));
            let m = nalgebra::Matrix3::from_rows(&[na.transpose(), nb.transpose(), dir.transpose()]);
            let Some(inv) = m.try_inverse() else { return out };
            let o = P3::from(inv * V3::new(da, db, dir.dot(&ca.coords)));
            let w = o - ca;
            let bq = w.dot(&dir);
            let disc = bq * bq - (w.norm_squared() - ra * ra);
            if disc < -2.0 * ra * tol {
                return out;
            }
            if disc <= 0.0 {
                hits.push(o - dir * bq);
            } else {
                let q = disc.sqrt();
                hits.push(o + dir * (-bq - q));
                hits.push(o + dir * (-bq + q));
            }
        }
        for p in hits {
            let (wa, wb) = (p - ca, p - cb);
            if (wa.norm() - ra).abs() > tol || (wb.norm() - rb).abs() > tol {
                continue;
            }
            let ta = super::curve::clamp_periodic(wa.dot(&ya).atan2(wa.dot(&xa)), a0, a1);
            let tb = super::curve::clamp_periodic(wb.dot(&yb).atan2(wb.dot(&xb)), b0, b1);
            if (a.point(ta) - p).norm() < tol && (b.point(tb) - p).norm() < tol {
                out.push((ta, tb));
            }
        }
        return out;
    }
    // Sample a, find local minima of the distance to b, refine by Newton on both parameters.
    let n = match a {
        Curve::Line { .. } => 32,
        Curve::Circle { .. } | Curve::Ellipse { .. } => (((a1 - a0) / (PI / 32.0)).ceil() as usize).max(16),
        Curve::Nurbs(nc) => (nc.cps.len() * 8).clamp(32, 2000),
    };
    let dist = |t: f64| -> (f64, f64) {
        let p = a.point(t);
        let s = b.closest(&p, b0, b1);
        ((b.point(s) - p).norm(), s)
    };
    let samples: Vec<(f64, f64, f64)> = (0..=n)
        .map(|i| {
            let t = a0 + (a1 - a0) * i as f64 / n as f64;
            let (d, s) = dist(t);
            (t, d, s)
        })
        .collect();
    let spacing = a.length(a0, a1) / n as f64;
    for i in 0..samples.len() {
        let (t, d, s) = samples[i];
        let left = if i > 0 { samples[i - 1].1 } else { f64::INFINITY };
        let right = if i + 1 < samples.len() { samples[i + 1].1 } else { f64::INFINITY };
        // A run of coincident samples is an overlap, not crossings (found from the ends).
        if d < tol && (left < tol || right < tol) {
            continue;
        }
        if d <= left && d <= right && d < spacing * 2.0 + tol {
            // Newton on min |a(t) − b(s)|².
            let (mut t, mut s) = (t, s);
            for _ in 0..50 {
                let (p, pa, paa) = a.d2(t);
                let (q, qb, qbb) = b.d2(s);
                let r = p - q;
                let ft = r.dot(&pa);
                let fs = -r.dot(&qb);
                let (j11, j12, j22) = (pa.dot(&pa) + r.dot(&paa), -pa.dot(&qb), qb.dot(&qb) - r.dot(&qbb));
                let Some((dt, ds)) = solve2(j11, j12, j12, j22, -ft, -fs) else { break };
                t = (t + dt).clamp(a0, a1);
                s = (s + ds).clamp(b0, b1);
                if dt.abs() + ds.abs() < 1e-15 {
                    break;
                }
            }
            if (a.point(t) - b.point(s)).norm() < tol && !out.iter().any(|(x, _): &(f64, f64)| (x - t).abs() < 1e-9) {
                out.push((t, s));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plane sections of a cone (ellipse, parabola, hyperbola): every point of the exact conic
    /// lies on both surfaces.
    #[test]
    fn plane_cone_conics_lie_on_both() {
        let fc = Frame { o: P3::new(1.0, -2.0, 0.5), x: V3::x(), y: V3::y(), z: V3::z() };
        let (r, a) = (3.0, -0.4);
        let bb = Bbox { min: P3::new(-40.0, -40.0, -40.0), max: P3::new(40.0, 40.0, 40.0) };
        for (n, o) in [
            (V3::new(0.2, 0.1, 1.0), P3::new(0.0, 0.0, 1.0)),
            (V3::new(0.0, 1.0, 0.0), P3::new(0.0, -1.5, 0.0)),
            (V3::new((PI / 2.0 - 0.4).cos(), 0.0, (PI / 2.0 - 0.4).sin()), P3::new(2.0, 0.0, 0.0)),
            (V3::new(1.0, 0.3, 0.6), P3::new(1.5, -2.0, 1.0)),
        ] {
            let fp = Frame::from_z(o, n.normalize());
            let curves = plane_cone(&fp, &fc, r, a, &bb).unwrap();
            assert!(!curves.is_empty(), "{n:?}");
            for ic in &curves {
                for i in 0..=200 {
                    let t = ic.t0 + (ic.t1 - ic.t0) * i as f64 / 200.0;
                    let p = ic.c.point(t);
                    assert!((p - fp.o).dot(&n.normalize()).abs() < 1e-9, "off the plane {n:?}");
                    // Either nappe: (±ρ − r)·cos a = z·sin a.
                    let l = fc.local(&p);
                    let rho = l.x.hypot(l.y);
                    let off = [1.0, -1.0].map(|sg: f64| ((sg * rho - r) * a.cos() - l.z * a.sin()).abs()).into_iter().fold(f64::INFINITY, f64::min);
                    assert!(off < 1e-9, "off the cone {n:?}: {off}");
                }
            }
        }
    }
}
