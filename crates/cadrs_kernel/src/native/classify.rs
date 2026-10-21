//! Ray–face intersection, point-in-face and point-in-solid classification.

use super::math::*;
use super::surface::Surface;
use super::topo::{co_range, Body, Face};

/// Where a (u, v) point lies relative to a face's trimmed domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InFace {
    Inside,
    Outside,
    Boundary,
}

/// A face's loops sampled as uv polygons (for in-face tests).
pub fn face_polygons(body: &Body, f: &Face) -> Vec<Vec<P2>> {
    f.loops
        .iter()
        .map(|l| {
            let mut pts = Vec::new();
            for c in l {
                let e = &body.edges[c.edge];
                let (a, b) = co_range(e, c.fwd);
                let n = match &c.pc {
                    super::curve::Curve::Line { .. } => 1,
                    _ => {
                        // Enough points that the polygon is within ~1e-6 of the p-curve.
                        let span = (b - a).abs();
                        ((span * 256.0).ceil() as usize).clamp(16, 4096)
                    }
                };
                for i in 0..n {
                    let t = a + (b - a) * i as f64 / n as f64;
                    let p = c.pc.point(t);
                    pts.push(P2::new(p.x, p.y));
                }
            }
            pts
        })
        .collect()
}

/// Point in a face's domain given its sampled loops; `tol` in model units (scaled by the
/// surface metric).
pub fn in_face(polys: &[Vec<P2>], surf: &Surface, uv: P2, tol: f64) -> InFace {
    let (mu, mv) = surf.metric();
    let q = P2::new(uv.x * mu, uv.y * mv);
    let mut inside = false;
    for p in polys {
        let n = p.len();
        for i in 0..n {
            let (a, b) = (p[i], p[(i + 1) % n]);
            let (a, b) = (P2::new(a.x * mu, a.y * mv), P2::new(b.x * mu, b.y * mv));
            if seg_dist2(&q, &a, &b) < tol {
                return InFace::Boundary;
            }
        }
        if point_in_polygon(&uv, p) {
            inside = !inside;
        }
    }
    if inside { InFace::Inside } else { InFace::Outside }
}

/// Parameters t where the line o + t·d meets the surface (all of them, any sign), with the
/// (u, v) there.
pub fn ray_surface(s: &Surface, o: &P3, d: &V3, tmin: f64, tmax: f64) -> Vec<(f64, f64, f64)> {
    let mut ts: Vec<f64> = Vec::new();
    match s {
        Surface::Plane { f } => {
            let n = f.x.cross(&f.y);
            let den = n.dot(d);
            if den.abs() > 1e-15 {
                ts.push((f.o - o).dot(&n) / den);
            }
        }
        Surface::Cylinder { f, r } => {
            let l = f.local(o);
            let ld = V3::new(d.dot(&f.x), d.dot(&f.y), d.dot(&f.z));
            ts = quadratic(ld.x * ld.x + ld.y * ld.y, 2.0 * (l.x * ld.x + l.y * ld.y), l.x * l.x + l.y * l.y - r * r);
            if ld.x * ld.x + ld.y * ld.y < 1e-24 {
                ts.clear();
            }
        }
        Surface::Sphere { f, r } => {
            let l = o - f.o;
            ts = quadratic(d.norm_squared(), 2.0 * l.dot(d), l.norm_squared() - r * r);
        }
        Surface::Cone { f, r, a } => {
            // ρ cos a − z sin a − r cos a = 0 ⇔ ρ = r + z tan a (on the cone's sheet).
            let l = f.local(o);
            let ld = V3::new(d.dot(&f.x), d.dot(&f.y), d.dot(&f.z));
            let ta = a.tan();
            // x² + y² = (r + z tan a)²
            let (c0, c1) = (*r + l.z * ta, ld.z * ta);
            let qa = ld.x * ld.x + ld.y * ld.y - c1 * c1;
            let qb = 2.0 * (l.x * ld.x + l.y * ld.y - c0 * c1);
            let qc = l.x * l.x + l.y * l.y - c0 * c0;
            for t in quadratic(qa, qb, qc) {
                // Keep the sheet where r + z tan a >= 0.
                if c0 + c1 * t >= -1e-9 {
                    ts.push(t);
                }
            }
        }
        Surface::Extrusion { c, d: dir } => {
            // The plane through the ray parallel to the extrusion cuts it in rulings where the
            // base curve crosses that plane; each ruling meets the ray at one point.
            let m = d.cross(dir);
            if m.norm() < 1e-12 {
                return Vec::new();
            }
            let fr = Frame::from_z(*o, m.normalize());
            let pl = Surface::Plane { f: fr };
            let (a, b) = c.nurbs_range().unwrap_or((0.0, TAU));
            for u in super::ssi::curve_surface(c, a, b, &pl) {
                let q = c.point(u);
                // o + t d = q + v dir: solve in least squares.
                let w = q - o;
                let (dd, dr, rr) = (d.dot(d), d.dot(dir), dir.dot(dir));
                let (wd, wr) = (w.dot(d), w.dot(dir));
                if let Some((t, _v)) = solve2(dd, -dr, dr, -rr, wd, wr) {
                    ts.push(t);
                }
            }
        }
        Surface::Nurbs(_) | Surface::Revolution { .. } => {
            return ray_parametric(s, o, d);
        }
        _ => {
            // Implicit along the ray where available, else sampled distance with projection.
            let n = 400;
            let f = |t: f64| -> f64 {
                let p = o + d * t;
                match s.signed_distance(&p) {
                    Some(x) => x,
                    None => {
                        let (u, v) = s.project(&p, None);
                        let (q, su, sv) = s.d1(u, v);
                        (p - q).dot(&su.cross(&sv).normalize())
                    }
                }
            };
            let (lo, hi) = (tmin, tmax);
            let mut prev = (lo, f(lo));
            for i in 1..=n {
                let t = lo + (hi - lo) * i as f64 / n as f64;
                let ft = f(t);
                if prev.1 == 0.0 {
                    ts.push(prev.0);
                } else if prev.1.signum() != ft.signum() {
                    // Bisect.
                    let (mut a, mut b, mut fa) = (prev.0, t, prev.1);
                    for _ in 0..80 {
                        let m = 0.5 * (a + b);
                        let fm = f(m);
                        if fm.signum() == fa.signum() {
                            a = m;
                            fa = fm;
                        } else {
                            b = m;
                        }
                    }
                    let m = 0.5 * (a + b);
                    // Reject sign flips across the far side of a projection jump.
                    let p = o + d * m;
                    if s.distance(&p) < 1e-6 {
                        ts.push(m);
                    }
                }
                prev = (t, ft);
            }
        }
    }
    let mut out = Vec::new();
    for t in ts {
        let p = o + d * t;
        let (u, v) = s.project(&p, None);
        out.push((t, u, v));
    }
    // Torus: the quadric cases above miss it; handled by the sampled branch.
    out
}

/// Line–surface hits on a parametric surface: the ray against a triangulated grid of the
/// surface's parameter box, each hit refined by Newton on S(u, v) = o + t·d.
fn ray_parametric(s: &Surface, o: &P3, d: &V3) -> Vec<(f64, f64, f64)> {
    let ((u0, u1), (v0, v1)) = match s {
        Surface::Revolution { m, .. } => ((0.0, TAU), m.nurbs_range().unwrap_or((-1e3, 1e3))),
        _ => s.param_box(),
    };
    let n = 32;
    let at = |i: usize, j: usize| (u0 + (u1 - u0) * i as f64 / n as f64, v0 + (v1 - v0) * j as f64 / n as f64);
    let mut grid = vec![P3::origin(); (n + 1) * (n + 1)];
    for i in 0..=n {
        for j in 0..=n {
            let (u, v) = at(i, j);
            grid[i * (n + 1) + j] = s.eval(u, v);
        }
    }
    let mut out: Vec<(f64, f64, f64)> = Vec::new();
    let tri = |a: &P3, b: &P3, c: &P3| -> Option<(f64, f64, f64)> {
        // Möller–Trumbore with a small margin; returns (t, β, γ).
        let (e1, e2) = (b - a, c - a);
        let h = d.cross(&e2);
        let det = e1.dot(&h);
        if det.abs() < 1e-300 {
            return None;
        }
        let f = 1.0 / det;
        let sv = o - a;
        let bu = f * sv.dot(&h);
        let q = sv.cross(&e1);
        let bv = f * d.dot(&q);
        let m = 0.05;
        if bu < -m || bv < -m || bu + bv > 1.0 + m {
            return None;
        }
        Some((f * e2.dot(&q), bu, bv))
    };
    for i in 0..n {
        for j in 0..n {
            let (p00, p10, p01, p11) = (grid[i * (n + 1) + j], grid[(i + 1) * (n + 1) + j], grid[i * (n + 1) + j + 1], grid[(i + 1) * (n + 1) + j + 1]);
            let (ua, va) = at(i, j);
            let (ub, vb) = at(i + 1, j + 1);
            for (k, (a, b, c)) in [(p00, p10, p11), (p00, p11, p01)].into_iter().enumerate() {
                let Some((t, bu, bv)) = tri(&a, &b, &c) else { continue };
                // Seed (u, v) inside the cell.
                let (mut u, mut v) = if k == 0 { (ua + (ub - ua) * (bu + bv), va + (vb - va) * bv) } else { (ua + (ub - ua) * bu, va + (vb - va) * (bu + bv)) };
                let mut tt = t;
                let mut ok = false;
                for _ in 0..30 {
                    let (q, su, sv) = s.d1(u, v);
                    let r = q - (o + d * tt);
                    if r.norm() < 1e-12 {
                        ok = true;
                        break;
                    }
                    let m = nalgebra::Matrix3::from_columns(&[su, sv, -d]);
                    let Some(inv) = m.try_inverse() else { break };
                    let dx = inv * (-r);
                    u += dx.x;
                    v += dx.y;
                    tt += dx.z;
                    if dx.norm() < 1e-14 {
                        ok = (s.eval(u, v) - (o + d * tt)).norm() < 1e-9;
                        break;
                    }
                }
                if ok && !out.iter().any(|(x, ..)| (x - tt).abs() < 1e-9) {
                    out.push((tt, u, v));
                }
            }
        }
    }
    out
}

/// Every crossing of the line with the faces of the body: (t, face, point), sorted by t.
pub fn ray_hits(body: &Body, polys: &[Vec<Vec<P2>>], o: &P3, d: &V3) -> Vec<(f64, usize, P3)> {
    let bb = body.bbox().grow(1.0);
    let span = bb.diagonal() + (bb.center() - o).norm();
    let mut out = Vec::new();
    for (fi, f) in body.faces.iter().enumerate() {
        for (t, u, v) in ray_surface(&f.surf, o, d, -span, span) {
            let uvs = periodic_candidates(&f.surf, u, v, &polys[fi]);
            for uv in uvs {
                if in_face(&polys[fi], &f.surf, uv, 1e-9) != InFace::Outside {
                    out.push((t, fi, o + d * t));
                    break;
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Hits on a shared edge are reported once per face; keep them (OCCT reports both too) but
    // drop exact duplicates on the same face.
    out.dedup_by(|a, b| a.1 == b.1 && (a.0 - b.0).abs() < 1e-9);
    out
}

/// The (u, v) shifted by whole periods into the face's uv box.
pub fn periodic_candidates(s: &Surface, u: f64, v: f64, polys: &[Vec<P2>]) -> Vec<P2> {
    let mut ub = (f64::INFINITY, f64::NEG_INFINITY);
    let mut vb = (f64::INFINITY, f64::NEG_INFINITY);
    for p in polys {
        for q in p {
            ub = (ub.0.min(q.x), ub.1.max(q.x));
            vb = (vb.0.min(q.y), vb.1.max(q.y));
        }
    }
    let us: Vec<f64> = if s.u_periodic() {
        let mut v = Vec::new();
        for k in -2..=2 {
            let x = u + TAU * k as f64;
            if x >= ub.0 - 1e-9 && x <= ub.1 + 1e-9 {
                v.push(x);
            }
        }
        if v.is_empty() {
            v.push(u);
        }
        v
    } else {
        vec![u]
    };
    let vs: Vec<f64> = if s.v_periodic() {
        let mut w = Vec::new();
        for k in -2..=2 {
            let x = v + TAU * k as f64;
            if x >= vb.0 - 1e-9 && x <= vb.1 + 1e-9 {
                w.push(x);
            }
        }
        if w.is_empty() {
            w.push(v);
        }
        w
    } else {
        vec![v]
    };
    let mut out = Vec::new();
    for &a in &us {
        for &b in &vs {
            out.push(P2::new(a, b));
        }
    }
    out
}

/// Inside / outside / on the boundary of the body's solids.
pub fn classify_point(body: &Body, polys: &[Vec<Vec<P2>>], p: &P3, tol: f64) -> crate::PointClass {
    // On a face?
    for (fi, f) in body.faces.iter().enumerate() {
        let (u, v) = f.surf.project(p, None);
        let q = f.surf.eval(u, v);
        if (q - p).norm() <= tol.max(1e-9) {
            for uv in periodic_candidates(&f.surf, u, v, &polys[fi]) {
                if in_face(&polys[fi], &f.surf, uv, tol.max(1e-9)) != InFace::Outside {
                    return crate::PointClass::OnBoundary;
                }
            }
        }
    }
    if parity_inside(body, polys, p) { crate::PointClass::Inside } else { crate::PointClass::Outside }
}

/// Ray parity with retries in other directions when a ray grazes an edge or a face.
#[allow(clippy::approx_constant, reason = "ray directions picked off the axes and diagonals, not constants")]
pub fn parity_inside(body: &Body, polys: &[Vec<Vec<P2>>], p: &P3) -> bool {
    // Five fixed directions, then 32 more spread over the sphere (a golden-angle spiral) while
    // every ray so far grazed a face or ran through an edge.
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    let dirs: Vec<V3> = [
        V3::new(0.5773, 0.5774, 0.5772),
        V3::new(-0.3014, 0.8123, 0.4993),
        V3::new(0.7071, -0.1213, -0.6966),
        V3::new(0.1234, 0.2345, -0.9641),
        V3::new(-0.8, -0.35, 0.487),
    ]
    .into_iter()
    .chain((0..32).map(|i| {
        let z = 1.0 - (i as f64 + 0.5) / 16.0;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let a = golden * i as f64 + 0.1;
        V3::new(r * a.cos(), r * a.sin(), z)
    }))
    .collect();
    let bb = body.bbox().grow(1.0);
    let span = bb.diagonal() + (bb.center() - p).norm();
    let mut votes = (0, 0);
    for d in dirs {
        let d = d.normalize();
        let mut count = 0;
        let mut bad = false;
        'faces: for (fi, f) in body.faces.iter().enumerate() {
            for (t, u, v) in ray_surface(&f.surf, p, &d, -span, span) {
                if t <= 1e-12 {
                    continue;
                }
                // Grazing: the ray nearly tangent to the surface.
                let n = f.surf.normal(u, v);
                if n.dot(&d).abs() < 1e-6 {
                    bad = true;
                    break 'faces;
                }
                for uv in periodic_candidates(&f.surf, u, v, &polys[fi]) {
                    match in_face(&polys[fi], &f.surf, uv, 1e-7) {
                        InFace::Inside => {
                            count += 1;
                            break;
                        }
                        InFace::Boundary => {
                            bad = true;
                            break 'faces;
                        }
                        InFace::Outside => {}
                    }
                }
            }
        }
        if bad {
            continue;
        }
        if count % 2 == 1 {
            votes.0 += 1;
        } else {
            votes.1 += 1;
        }
        if votes.0 + votes.1 >= 1 {
            break;
        }
    }
    if votes.0 + votes.1 == 0 {
        // Every ray grazed: by the side of the nearest face.
        return nearest_face_inside(body, p);
    }
    votes.0 > votes.1
}

/// Whether `p` lies behind the nearest face of `body` (against its outward normal there).
fn nearest_face_inside(body: &Body, p: &P3) -> bool {
    let mut best: Option<(f64, V3, P3)> = None;
    for f in &body.faces {
        let (u, v) = f.surf.project(p, None);
        let q = f.surf.eval(u, v);
        let d = (q - p).norm();
        if best.is_none_or(|b| d < b.0) {
            let n = f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 };
            best = Some((d, n, q));
        }
    }
    best.is_some_and(|(_, n, q)| (p - q).dot(&n) < 0.0)
}
