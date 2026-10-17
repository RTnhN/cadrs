//! Intersections: surface–surface (analytic cases exactly, the rest by contouring the signed
//! distance over one surface's parameter box and refining onto both surfaces), curve–surface
//! and curve–curve.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
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
    if let Some(r) = analytic(a, b, bb) {
        return r;
    }
    if let Some(r) = analytic(b, a, bb) {
        return r;
    }
    // Numeric: contour over the surface whose box is smaller in model space, against the
    // other's distance.
    let _ = ub;
    numeric(a, ua, b, bb)
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
            None
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
            // Coaxial surfaces of revolution: circles where the meridians cross.
            let (Some((oa, za)), Some((ob, zb))) = (a.axis(), b.axis()) else { return None };
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

/// The meridian function of a surface of revolution about (o, z): its signed distance at the
/// point (ρ, h) of a half plane through the axis.
fn meridian_fn<'a>(s: &'a Surface, o: &P3, z: &V3) -> impl Fn(f64, f64) -> f64 + 'a {
    let x = any_perpendicular(z);
    let (o, z) = (*o, *z);
    move |rho: f64, h: f64| s.signed_distance(&(o + x * rho + z * h)).unwrap_or(1.0)
}

fn coaxial(a: &Surface, b: &Surface, o: &P3, z: &V3, bb: &Bbox) -> Vec<IntCurve> {
    // Sample the meridian of `a` in (ρ, h) and find where b's meridian function changes sign.
    let fa = meridian_fn(a, o, z);
    let fb = meridian_fn(b, o, z);
    // Range of ρ and h from the box.
    let mut hmin = f64::INFINITY;
    let mut hmax = f64::NEG_INFINITY;
    let mut rmax: f64 = 0.0;
    for i in 0..8 {
        let p = P3::new(
            if i & 1 == 0 { bb.min.x } else { bb.max.x },
            if i & 2 == 0 { bb.min.y } else { bb.max.y },
            if i & 4 == 0 { bb.min.z } else { bb.max.z },
        );
        let w = p - o;
        let h = w.dot(z);
        hmin = hmin.min(h);
        hmax = hmax.max(h);
        rmax = rmax.max((w - z * h).norm());
    }
    let pad = (hmax - hmin).max(rmax) * 0.05 + 1e-6;
    let (hmin, hmax, rmax) = (hmin - pad, hmax + pad, rmax + pad);
    // Contour fa = 0 and fb = 0 on a grid; find common points by Newton from grid cells.
    let n = 96;
    let mut pts: Vec<(f64, f64)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let r0 = rmax * i as f64 / n as f64;
            let r1 = rmax * (i + 1) as f64 / n as f64;
            let h0 = hmin + (hmax - hmin) * j as f64 / n as f64;
            let h1 = hmin + (hmax - hmin) * (j + 1) as f64 / n as f64;
            let ca = [fa(r0, h0), fa(r1, h0), fa(r0, h1), fa(r1, h1)];
            let cb = [fb(r0, h0), fb(r1, h0), fb(r0, h1), fb(r1, h1)];
            let sa = ca.iter().any(|x| *x <= 0.0) && ca.iter().any(|x| *x >= 0.0);
            let sb = cb.iter().any(|x| *x <= 0.0) && cb.iter().any(|x| *x >= 0.0);
            if !(sa && sb) {
                continue;
            }
            // Newton on (fa, fb) = 0 from the cell centre.
            let (mut r, mut h) = ((r0 + r1) / 2.0, (h0 + h1) / 2.0);
            let e = 1e-7 * (1.0 + rmax);
            let mut ok = false;
            for _ in 0..40 {
                let (a0, b0) = (fa(r, h), fb(r, h));
                let (ar, br) = ((fa(r + e, h) - fa(r - e, h)) / (2.0 * e), (fb(r + e, h) - fb(r - e, h)) / (2.0 * e));
                let (ah, bh) = ((fa(r, h + e) - fa(r, h - e)) / (2.0 * e), (fb(r, h + e) - fb(r, h - e)) / (2.0 * e));
                let Some((dr, dh)) = solve2(ar, ah, br, bh, -a0, -b0) else { break };
                r += dr;
                h += dh;
                if dr.abs() + dh.abs() < 1e-14 * (1.0 + rmax) {
                    ok = true;
                    break;
                }
            }
            if ok && fa(r, h).abs() < 1e-9 && fb(r, h).abs() < 1e-9 && r > 1e-9 {
                if !pts.iter().any(|(pr, ph)| (pr - r).abs() < 1e-7 && (ph - h).abs() < 1e-7) {
                    pts.push((r, h));
                }
            }
        }
    }
    let f = Frame::from_z(*o, *z);
    pts.into_iter()
        .map(|(r, h)| IntCurve { c: Curve::Circle { c: o + z * h, x: f.x, y: f.y, r }, t0: 0.0, t1: TAU, closed: true })
        .collect()
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
    let g = |u: f64, v: f64| sdist(b, &a.eval(u, v));
    let mut grid = vec![vec![0.0; nv + 1]; nu + 1];
    for i in 0..=nu {
        for j in 0..=nv {
            grid[i][j] = g(u0 + (u1 - u0) * i as f64 / nu as f64, v0 + (v1 - v0) * j as f64 / nv as f64);
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
    let refine = |uv: P2| -> P3 { refine_point(a, b, uv) };
    let mut polys: Vec<(Vec<P3>, bool)> = lines.into_iter().map(|(pl, c)| (pl.into_iter().map(refine).collect(), c)).collect();
    // Join polylines that meet in 3D (across a periodic seam).
    join_polylines(&mut polys, h * 1e-3);
    polys.into_iter().filter(|(p, _)| p.len() >= 2).map(|(p, closed)| fit_curve(a, b, p, closed)).collect()
}

/// A point on both surfaces near a's (u, v): Newton on a's parameters for sd_b = 0, then a
/// few alternating projections to tighten.
pub fn refine_point(a: &Surface, b: &Surface, uv: P2) -> P3 {
    let (mut u, mut v) = (uv.x, uv.y);
    let e = 1e-7;
    for _ in 0..30 {
        let p = a.eval(u, v);
        let g0 = sdist(b, &p);
        if g0.abs() < 1e-13 {
            break;
        }
        let (_, su, sv) = a.d1(u, v);
        // ∇g ≈ n_b·(Su, Sv) with n_b the distance gradient.
        let gu = (sdist(b, &(p + su * e)) - sdist(b, &(p - su * e))) / (2.0 * e);
        let gv = (sdist(b, &(p + sv * e)) - sdist(b, &(p - sv * e))) / (2.0 * e);
        let l = gu * gu + gv * gv;
        if l < 1e-300 {
            break;
        }
        u -= g0 * gu / l;
        v -= g0 * gv / l;
    }
    a.eval(u, v)
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
    for _round in 0..6 {
        let t = chord(&pts);
        let c = NurbsCurve::interpolate(&pts, &t, 3.min(pts.len() - 1), None, None);
        let mut ins: Vec<(usize, P3)> = Vec::new();
        for k in 0..pts.len() - 1 {
            let tm = (t[k] + t[k + 1]) / 2.0;
            let q = c.point(tm);
            let (u, v) = a.project(&q, None);
            let exact = refine_point(a, b, P2::new(u, v));
            if (exact - q).norm() > 1e-9 && (pts[k + 1] - pts[k]).norm() > 1e-6 {
                ins.push((k + 1, exact));
            }
        }
        if ins.is_empty() || pts.len() > 20000 {
            let n = c.range();
            return IntCurve { c: Curve::Nurbs(Arc::new(c)), t0: n.0, t1: n.1, closed };
        }
        for (k, p) in ins.into_iter().rev() {
            pts.insert(k, p);
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
    let f = |t: f64| sdist(s, &c.point(t));
    let mut out: Vec<f64> = Vec::new();
    let mut prev = (t0, f(t0));
    let mut vals = vec![prev];
    for i in 1..=n {
        let t = t0 + (t1 - t0) * i as f64 / n as f64;
        vals.push((t, f(t)));
    }
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
