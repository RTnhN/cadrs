//! Local operations that keep a body's topology and change its faces' surfaces: sharp
//! offsets (faces moved along their normals), drafts (faces turned about a neutral plane) and
//! the simple inward shell (the body less its inward offset). Vertices are recomputed as the
//! points where their faces' new surfaces meet, edges as the new surfaces' intersection curves
//! between those points (a seam as the new surface's iso-line), p-curves from the new edges.

use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::surface::Surface;
use super::topo::{Body, Coedge};
use crate::*;

/// The faces around each vertex (distinct).
fn vertex_faces(b: &Body) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new(); b.verts.len()];
    for (fi, f) in b.faces.iter().enumerate() {
        for c in f.loops.iter().flatten() {
            for v in b.edges[c.edge].v {
                if !out[v].contains(&fi) {
                    out[v].push(fi);
                }
            }
        }
    }
    out
}

/// A point on all the surfaces near `p` (Newton on their signed distances, least squares when
/// more than three meet).
fn meet(surfs: &[&Surface], p: P3) -> Option<P3> {
    let mut x = p;
    for _ in 0..60 {
        let mut jt = nalgebra::DMatrix::<f64>::zeros(surfs.len(), 3);
        let mut r = nalgebra::DVector::<f64>::zeros(surfs.len());
        for (i, s) in surfs.iter().enumerate() {
            let d = super::ssi::sdist(s, &x);
            let e = 1e-6;
            let g = V3::new(
                (super::ssi::sdist(s, &(x + V3::x() * e)) - super::ssi::sdist(s, &(x - V3::x() * e))) / (2.0 * e),
                (super::ssi::sdist(s, &(x + V3::y() * e)) - super::ssi::sdist(s, &(x - V3::y() * e))) / (2.0 * e),
                (super::ssi::sdist(s, &(x + V3::z() * e)) - super::ssi::sdist(s, &(x - V3::z() * e))) / (2.0 * e),
            );
            jt[(i, 0)] = g.x;
            jt[(i, 1)] = g.y;
            jt[(i, 2)] = g.z;
            r[i] = -d;
        }
        // Minimum-norm least-squares step.
        let svd = jt.clone().svd(true, true);
        let dx = svd.solve(&r, 1e-12).ok()?;
        let step = V3::new(dx[0], dx[1], dx[2]);
        x += step;
        if step.norm() < 1e-14 * (1.0 + x.coords.norm()) {
            break;
        }
    }
    let worst = surfs.iter().map(|s| s.distance(&x)).fold(0.0, f64::max);
    (worst < 1e-7).then_some(x)
}

/// Rebuilds the body's geometry with new surfaces for its faces (same topology).
pub fn replace_surfaces(b: &Body, surfs: &[Surface]) -> Result<Body> {
    let mut out = b.clone();
    let vf = vertex_faces(b);
    let ef = b.edge_faces();
    // Vertices.
    for v in 0..b.verts.len() {
        let fs: Vec<&Surface> = vf[v].iter().map(|&f| &surfs[f]).collect();
        if fs.is_empty() {
            continue;
        }
        let moved = vf[v].iter().any(|&f| surfs[f] != b.faces[f].surf);
        if !moved {
            continue;
        }
        let p = meet(&fs, b.verts[v].p).ok_or_else(|| KernelError::OperationFailed("the moved faces no longer meet at a corner".into()))?;
        out.verts[v].p = p;
    }
    // Edges.
    for e in 0..b.edges.len() {
        let ed = &b.edges[e];
        let fs = &ef[e];
        let moved = fs.iter().any(|&f| surfs[f] != b.faces[f].surf);
        let (p0, p1) = (out.verts[ed.v[0]].p, out.verts[ed.v[1]].p);
        if !moved {
            // Same curve; the ends may have moved along it.
            if !ed.degenerate && ((p0 - b.verts[ed.v[0]].p).norm() > 1e-12 || (p1 - b.verts[ed.v[1]].p).norm() > 1e-12) {
                let q = ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * 0.25);
                let (t0, t1) = param_range(&ed.curve, &p0, &p1, ed.v[0] == ed.v[1], &q);
                if let Curve::Line { .. } = ed.curve {
                    let (a, c) = (ed.curve.closest(&p0, -1e12, 1e12), ed.curve.closest(&p1, -1e12, 1e12));
                    out.edges[e].t = if a <= c { [a, c] } else { [c, a] };
                } else {
                    out.edges[e].t = if t0 <= t1 { [t0, t1] } else { [t1, t0] };
                }
            }
            continue;
        }
        if ed.degenerate {
            out.edges[e].curve = Curve::Line { o: p0, d: V3::zeros() };
            continue;
        }
        let old_mid = ed.point(ed.mid_t());
        let curve = if fs.len() == 2 && fs[0] == fs[1] {
            // A seam: the new surface's iso-u line through the vertex.
            let s = &surfs[fs[0]];
            let (u, _) = s.project(&p0, None);
            s.iso_u(u).ok_or_else(|| KernelError::Unsupported("moving a seam of this surface"))?
        } else if fs.len() == 2 {
            let (sa, sb) = (&surfs[fs[0]], &surfs[fs[1]]);
            let mut bb = Bbox::empty();
            bb.add(&p0);
            bb.add(&p1);
            bb.add(&old_mid);
            for i in 0..=8 {
                bb.add(&ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0));
            }
            let bb = bb.grow(bb.diagonal() * 0.5 + 1.0);
            let box_of = |s: &Surface| {
                let ((u0, u1), (v0, v1)) = s.param_box();
                ((u0, u1), (v0.max(-1e4), v1.min(1e4)))
            };
            let cs = super::ssi::surface_surface(sa, box_of(sa), sb, box_of(sb), &bb);
            // The component through both new end points (nearest the old middle).
            let mut best: Option<(f64, Curve)> = None;
            for ic in cs {
                let t0 = ic.c.closest(&p0, ic.t0, ic.t1);
                let t1 = ic.c.closest(&p1, ic.t0, ic.t1);
                let d = (ic.c.point(t0) - p0).norm() + (ic.c.point(t1) - p1).norm();
                if best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                    best = Some((d, ic.c));
                }
            }
            let (d, c) = best.ok_or_else(|| KernelError::OperationFailed("moved faces no longer meet along an edge".into()))?;
            if d > 1e-6 {
                return Err(KernelError::OperationFailed("moved faces no longer meet along an edge".into()));
            }
            c
        } else {
            return Err(KernelError::Unsupported("moving faces of a non-manifold edge"));
        };
        // Parameters at the new ends, in the old edge's direction.
        let (t0, t1) = param_range(&curve, &p0, &p1, ed.v[0] == ed.v[1], &(ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * 0.25)));
        let (curve, t0, t1) = if t1 < t0 { let (c, a, b2) = curve.reversed(t0, t1); (c, b2.min(a), a.max(b2)) } else { (curve, t0, t1) };
        let (t0, t1) = if t1 < t0 { (t1, t0) } else { (t0, t1) };
        out.edges[e].curve = curve;
        out.edges[e].t = [t0, t1];
    }
    for (fi, s) in surfs.iter().enumerate() {
        out.faces[fi].surf = s.clone();
    }
    // p-curves, continuous along each loop.
    for fi in 0..out.faces.len() {
        let surf = out.faces[fi].surf.clone();
        for li in 0..out.faces[fi].loops.len() {
            let lp = out.faces[fi].loops[li].clone();
            let mut at: Option<P2> = None;
            let mut nl: Vec<Coedge> = Vec::new();
            // Seams on periodic faces: the second use of an edge goes a period on.
            for c in &lp {
                let e = &out.edges[c.edge];
                let mut pc = super::pcurve::pcurve(&surf, &e.curve, e.t[0], e.t[1], None);
                let (t0, _) = super::topo::co_range(e, c.fwd);
                if let Some(a) = at {
                    let q = pc.point(t0);
                    let du = if surf.u_periodic() { ((a.x - q.x) / TAU).round() * TAU } else { 0.0 };
                    let dv = if surf.v_periodic() { ((a.y - q.y) / TAU).round() * TAU } else { 0.0 };
                    if du != 0.0 || dv != 0.0 {
                        pc = pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                    }
                }
                let (_, t1) = super::topo::co_range(e, c.fwd);
                let q = pc.point(t1);
                at = Some(P2::new(q.x, q.y));
                nl.push(Coedge { edge: c.edge, fwd: c.fwd, pc });
            }
            out.faces[fi].loops[li] = nl;
        }
    }
    // Seams: both coedges of a seam got the same p-curve; shift the one that should be a
    // period away so the loop closes.
    for fi in 0..out.faces.len() {
        let surf = out.faces[fi].surf.clone();
        if !surf.u_periodic() {
            continue;
        }
        for li in 0..out.faces[fi].loops.len() {
            let n = out.faces[fi].loops[li].len();
            for k in 0..n {
                let prev = out.faces[fi].loops[li][(k + n - 1) % n].clone();
                let cur = out.faces[fi].loops[li][k].clone();
                let pe = &out.edges[prev.edge];
                let ce = &out.edges[cur.edge];
                let (_, pt1) = super::topo::co_range(pe, prev.fwd);
                let (ct0, _) = super::topo::co_range(ce, cur.fwd);
                let a = prev.pc.point(pt1);
                let q = cur.pc.point(ct0);
                let du = ((a.x - q.x) / TAU).round() * TAU;
                if du != 0.0 {
                    out.faces[fi].loops[li][k].pc = cur.pc.transformed(&Affine::translation(V3::new(du, 0.0, 0.0)));
                }
            }
        }
    }
    super::build::fix_senses(&mut out, 0..b.faces.len());
    // The senses must not flip (an inside-out face means the move turned it over).
    for (fi, f) in out.faces.iter().enumerate() {
        let _ = fi;
        let _ = f;
    }
    let errs = out.check();
    if !errs.is_empty() {
        return Err(KernelError::OperationFailed(format!("the moved faces don't close: {}", errs[0])));
    }
    Ok(out)
}

/// Parameters of two points on a curve; a closed edge spans a whole turn from p0.
fn param_range(c: &Curve, p0: &P3, p1: &P3, closed: bool, quarter: &P3) -> (f64, f64) {
    match c {
        Curve::Circle { .. } | Curve::Ellipse { .. } => {
            let t0 = c.closest(p0, -PI, PI);
            if closed {
                // The direction from the old edge's quarter point.
                let tq = c.closest(quarter, t0 - PI, t0 + PI);
                if tq >= t0 { (t0, t0 + TAU) } else { (t0, t0 - TAU) }
            } else {
                let t1 = c.closest(p1, t0 - PI, t0 + PI);
                // Choose the way round through the old quarter point.
                let tq = c.closest(quarter, t0 - TAU, t0 + TAU);
                let between = |a: f64, b: f64, x: f64| (x - a) * (x - b) <= 0.0;
                if between(t0, t1, unwrap_near(tq, (t0 + t1) / 2.0, TAU)) {
                    (t0, t1)
                } else if t1 > t0 {
                    (t0, t1 - TAU)
                } else {
                    (t0, t1 + TAU)
                }
            }
        }
        _ => {
            let (a, b) = c.nurbs_range().unwrap_or((-1e7, 1e7));
            (c.closest(p0, a, b), c.closest(p1, a, b))
        }
    }
}

/// A surface moved `d` along its outward normal (sense applied).
pub fn offset_surface(s: &Surface, sense: bool, d: f64) -> Option<Surface> {
    let d = if sense { d } else { -d };
    Some(match s {
        Surface::Plane { f } => {
            let n = f.x.cross(&f.y);
            Surface::Plane { f: Frame { o: f.o + n * d, ..*f } }
        }
        Surface::Cylinder { f, r } => {
            let r2 = r + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Cylinder { f: *f, r: r2 }
        }
        Surface::Sphere { f, r } => {
            let r2 = r + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Sphere { f: *f, r: r2 }
        }
        Surface::Torus { f, rmaj, rmin } => {
            let r2 = rmin + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Torus { f: *f, rmaj: *rmaj, rmin: r2 }
        }
        Surface::Cone { f, r, a } => {
            // Moving along the normal (cos a, −sin a) in the (ρ, h) meridian.
            let k = d * if f.right_handed() { 1.0 } else { -1.0 };
            Surface::Cone { f: Frame { o: f.o - f.z * (k * a.sin()), ..*f }, r: r + k * a.cos(), a: *a }
        }
        // An extrusion: its curve moved square to the direction (the normal is), extruded the
        // same way.
        Surface::Extrusion { c, d: dir } => {
            let off = offset_curve_along(c, &|t| s.normal(t, 0.0), d)?;
            Surface::Extrusion { c: off, d: *dir }
        }
        // A revolution: its meridian (at u = 0) moved along the normal there, revolved.
        Surface::Revolution { f, m } => {
            let off = offset_curve_along(m, &|t| s.normal(0.0, t), d)?;
            Surface::Revolution { f: *f, m: off }
        }
        // A B-spline: a surface through a grid of offset points (exact offsets aren't
        // B-splines).
        Surface::Nurbs(n) => {
            let ((u0, u1), (v0, v1)) = n.range();
            let k = 32;
            let us: Vec<f64> = (0..=k).map(|i| u0 + (u1 - u0) * i as f64 / k as f64).collect();
            let vs: Vec<f64> = (0..=k).map(|j| v0 + (v1 - v0) * j as f64 / k as f64).collect();
            let rows: Vec<NurbsCurve> = us
                .iter()
                .map(|&u| {
                    let pts: Vec<P3> = vs.iter().map(|&v| s.eval(u, v) + s.normal(u, v) * d).collect();
                    NurbsCurve::interpolate(&pts, &vs, 3, None, None)
                })
                .collect();
            let surf = super::nurbs::NurbsSurface::through_curves(&rows, &us, 3, None, None);
            // through_curves runs its rows along v and interpolates them along u: transpose to
            // keep the parameters' roles.
            let cps: Vec<Vec<super::nurbs::H>> = (0..surf.cps[0].len()).map(|j| surf.cps.iter().map(|r| r[j]).collect()).collect();
            Surface::Nurbs(std::sync::Arc::new(super::nurbs::NurbsSurface { du: surf.dv, dv: surf.du, ku: surf.kv.clone(), kv: surf.ku.clone(), cps }))
        }
        _ => return None,
    })
}

/// A curve moved by `d` along the unit field `n(t)`: exact for a line moved by a constant
/// vector and a circle moved square to itself in its plane (concentric), else a cubic through
/// 65 offset points on its parameters. None when a circle would shrink to nothing.
fn offset_curve_along(c: &Curve, n: &dyn Fn(f64) -> V3, d: f64) -> Option<Curve> {
    let (t0, t1) = c.nurbs_range().unwrap_or(match c {
        Curve::Circle { .. } | Curve::Ellipse { .. } => (0.0, TAU),
        _ => (-1.0, 1.0),
    });
    match c {
        Curve::Line { o, d: dir } => {
            let (a, b) = (n(t0), n(t1));
            if (a - b).norm() < 1e-9 {
                return Some(Curve::Line { o: o + a * d, d: *dir });
            }
        }
        Curve::Circle { c: cc, x, y, r } => {
            // Normal along the radius (in the plane): a concentric circle.
            let p = c.point(0.0);
            let radial = (p - cc) / *r;
            let nn = n(0.0);
            let along = nn.dot(&radial);
            if (along.abs() - 1.0).abs() < 1e-9 {
                let r2 = r + d * along;
                return (r2 > 1e-9).then_some(Curve::Circle { c: *cc, x: *x, y: *y, r: r2 });
            }
            // Normal across the plane: the circle moved along it.
            let z = x.cross(y);
            if (nn.dot(&z).abs() - 1.0).abs() < 1e-9 {
                return Some(Curve::Circle { c: cc + nn * d, x: *x, y: *y, r: *r });
            }
        }
        _ => {}
    }
    let k = 64;
    let ts: Vec<f64> = (0..=k).map(|i| t0 + (t1 - t0) * i as f64 / k as f64).collect();
    let pts: Vec<P3> = ts.iter().map(|&t| c.point(t) + n(t) * d).collect();
    Some(Curve::Nurbs(std::sync::Arc::new(NurbsCurve::interpolate(&pts, &ts, 3, None, None))))
}

/// Sharp offset: every face moved by its distance, the edges where the moved faces meet.
pub fn offset_sharp(b: &Body, dist: &[f64]) -> Result<Body> {
    let surfs: Vec<Surface> = b
        .faces
        .iter()
        .zip(dist)
        .map(|(f, d)| if *d == 0.0 { Some(f.surf.clone()) } else { offset_surface(&f.surf, f.sense, *d) })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| KernelError::Unsupported("offsetting this kind of face"))?;
    replace_surfaces(b, &surfs)
}

pub fn offset(k: &mut NativeKernel, body: BodyId, spec: &OffsetSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let mut dist = vec![spec.distance; b.faces.len()];
    for (f, d) in &spec.faces {
        if let Some(x) = dist.get_mut(f.0 as usize) {
            *x = *d;
        }
    }
    let out = if spec.sharp || dist.iter().all(|d| *d <= 0.0) {
        offset_sharp(&b, &dist)?
    } else {
        super::offset_round::offset_rounded(&b, &dist)?
    };
    let v0 = super::mass::body_mass(&b).volume;
    let v1 = super::mass::body_mass(&out).volume;
    let grow = dist.iter().any(|d| *d > 0.0);
    let shrink = dist.iter().any(|d| *d < 0.0);
    if (grow && !shrink && v1 < v0 - 1e-9) || (shrink && !grow && v1 > v0 + 1e-9) || v1 <= 0.0 {
        return Err(KernelError::OperationFailed("the offset turned the body inside out".into()));
    }
    let nf = out.faces.len();
    let id = k.insert(out);
    let history = History {
        modified: (0..nf.min(b.faces.len())).map(|i| (FaceId(i as u64), InputFace { body, face: FaceId(i as u64) })).collect(),
        ..Default::default()
    };
    Ok(OpResult { bodies: vec![id], history })
}

/// Draft: the faces turned about the neutral plane by the angle, leaning in as they rise along
/// the pull direction.
pub fn draft(k: &mut NativeKernel, body: BodyId, spec: &DraftSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let pull = spec.pull.into_inner();
    let mut surfs: Vec<Surface> = b.faces.iter().map(|f| f.surf.clone()).collect();
    for fid in &spec.faces {
        let fi = fid.0 as usize;
        let f = b.faces.get(fi).ok_or_else(|| KernelError::InvalidParameter(format!("unknown {fid:?}")))?;
        surfs[fi] = match &f.surf {
            Surface::Plane { f: fr } => {
                let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
                if n.cross(&pull).norm() < 1e-9 {
                    return Err(KernelError::InvalidParameter("A face square to the pull direction can't be drafted".into()));
                }
                if n.dot(&pull).abs() > 1e-9 {
                    return Err(KernelError::Unsupported("drafting a face that is already inclined"));
                }
                // The line where the face meets the neutral plane, and the face turned about it
                // so its outward normal tips towards the pull (the face leans in as it rises).
                let axis = pull.cross(&n).normalize();
                let on = fr.o - pull * (fr.o - spec.neutral).dot(&pull);
                let r = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(axis), spec.angle).into_inner();
                let rot = Affine { m: r, t: on.coords - r * on.coords };
                // Which way: the new normal must gain a component along the pull.
                let n2 = r * n;
                let rot = if n2.dot(&pull) * spec.angle.signum() >= 0.0 {
                    rot
                } else {
                    let r = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(axis), -spec.angle).into_inner();
                    Affine { m: r, t: on.coords - r * on.coords }
                };
                Surface::Plane { f: fr.transformed(&rot) }
            }
            Surface::Cylinder { f: fr, r } => {
                if fr.z.cross(&pull).norm() > 1e-9 {
                    return Err(KernelError::Unsupported("drafting a cylinder not along the pull"));
                }
                // A cone through the cylinder's circle at the neutral plane, narrowing along
                // the pull for a positive angle on an outward-facing cylinder.
                let h0 = (spec.neutral - fr.o).dot(&fr.z);
                let outward = f.sense == fr.right_handed();
                let s_pull = fr.z.dot(&pull).signum();
                // v along the generator; radius r + v sin a with a negative to narrow as z rises.
                let a = -spec.angle * s_pull * if outward { 1.0 } else { -1.0 };
                Surface::Cone { f: Frame { o: fr.o + fr.z * h0, ..*fr }, r: *r, a }
            }
            _ => return Err(KernelError::Unsupported("drafting this kind of face")),
        };
    }
    let out = replace_surfaces(&b, &surfs)?;
    let nf = out.faces.len();
    let id = k.insert(out);
    let history = History { modified: (0..nf).map(|i| (FaceId(i as u64), InputFace { body, face: FaceId(i as u64) })).collect(), ..Default::default() };
    Ok(OpResult { bodies: vec![id], history })
}
