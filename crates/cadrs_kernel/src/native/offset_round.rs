//! Rounded offsets (the body grown by a ball: a Minkowski sum) and shells.
//!
//! The grown body is the body, each face swept out by the distance (a slab), a tube round each
//! convex edge and a ball at each vertex, united. Shells: inward, the body less its inward
//! sharp offset (faces moved in, removed faces moved out past the body); when that offset
//! can't be made (faces vanish, as round a counterbore), the walls are built from their
//! definition, the body's points within the thickness of its kept faces: the cavity is the body
//! less each kept face swept in, a tube round each concave edge and a ball at each concave
//! vertex. Outward, the grown body less the body, cut back to the removed faces' planes.

use super::blend::{ball_body, Tracked};
use super::boolean::boolean_bodies;
use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use super::surface::Surface;
use super::topo::Body;
use crate::*;

/// A planar face swept along its outward normal by `d` (negative: inward).
fn face_slab(b: &Body, fi: usize, d: f64) -> Result<Body> {
    let f = &b.faces[fi];
    let Surface::Plane { f: fr } = &f.surf else {
        return super::slab::face_slab(b, fi, d);
    };
    let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
    let plane = Plane { origin: fr.o, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(n) };
    let tmp = super::kernel::subset(b, &[fi], false);
    let mut k = NativeKernel::new();
    let id = k.insert(tmp);
    let prof = Profile::new(plane, vec![]);
    let mf = super::features::face_as_profile(&k, &prof, &FaceInput { body: id, face: FaceId(0), source: 0 })?;
    let dir = if d >= 0.0 { n } else { -n };
    let kind = super::sweep::Prism { plane, dir };
    Ok(super::sweep::sweep_full(&[mf], &[], &kind, 0.0, d.abs(), false, true)?.body)
}

/// A tube of radius r round an edge (a cylinder along a line, a torus section round an arc),
/// with flat ends.
fn edge_tube(b: &Body, e: usize, r: f64) -> Result<Body> {
    let ed = &b.edges[e];
    match &ed.curve {
        Curve::Line { d, .. } => {
            let p0 = ed.point(ed.t[0]);
            let t = d.normalize();
            let fr = Frame::from_z(p0, t);
            let plane = Plane { origin: p0, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(t) };
            let prof = Profile::new(plane, vec![Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: r, source: None }] }, holes: vec![], source: None }]);
            let regions = super::profile::regions(&prof)?;
            let mf = super::profile::merge_regions(&regions)?;
            let len = (ed.t[1] - ed.t[0]) * d.norm();
            let kind = super::sweep::Prism { plane, dir: t };
            Ok(super::sweep::sweep_full(&mf, &[], &kind, 0.0, len, false, true)?.body)
        }
        Curve::Circle { c, x, y, r: rc } => {
            if r >= *rc {
                return Err(KernelError::Unsupported("a tube wider than its arc's radius"));
            }
            let q = ed.point(ed.t[0]);
            let axis = x.cross(y).normalize();
            let radial = (q - c).normalize();
            let tangent = axis.cross(&radial);
            let plane = Plane { origin: q, x_dir: nalgebra::Unit::new_normalize(radial), normal: nalgebra::Unit::new_normalize(tangent) };
            let prof = Profile::new(plane, vec![Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: r, source: None }] }, holes: vec![], source: None }]);
            let regions = super::profile::regions(&prof)?;
            let mf = super::profile::merge_regions(&regions)?;
            let f = Frame { o: *c, x: radial, y: axis.cross(&radial), z: axis };
            let span = ed.t[1] - ed.t[0];
            let whole = span >= TAU - 1e-9;
            let kind = super::sweep::Revolve { plane, f };
            Ok(super::sweep::sweep_full(&mf, &[], &kind, 0.0, if whole { TAU } else { span }, whole, true)?.body)
        }
        _ => Err(KernelError::Unsupported("tubes round this kind of edge")),
    }
}

/// Whether an edge is convex (its faces' material inside the wedge).
fn convex(b: &Body, e: usize) -> Option<bool> {
    let ed = &b.edges[e];
    let tm = ed.mid_t();
    let mut sides = Vec::new();
    for (fi, f) in b.faces.iter().enumerate() {
        for c in f.loops.iter().flatten() {
            if c.edge == e {
                let uv = c.pc.point(tm);
                let n = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
                let d = ed.curve.tangent(tm).normalize() * if c.fwd { 1.0 } else { -1.0 };
                sides.push((fi, n, n.cross(&d)));
            }
        }
    }
    if sides.len() != 2 || sides[0].0 == sides[1].0 {
        return None;
    }
    let (n1, _w1) = (sides[0].1, sides[0].2);
    let w2 = sides[1].2;
    if n1.cross(&sides[1].1).norm() < 1e-9 {
        return None; // tangent faces: smooth
    }
    Some(w2.dot(&n1) < 0.0)
}

/// The body grown by a ball of radius d (all faces moved out by d, edges and corners rounded).
pub fn offset_rounded(b: &Body, dist: &[f64]) -> Result<Body> {
    let d = dist[0];
    if dist.iter().any(|x| (x - d).abs() > 1e-12) || d <= 0.0 {
        return Err(KernelError::Unsupported("rounded offsets with different distances per face"));
    }
    let mut acc = b.clone();
    for fi in 0..b.faces.len() {
        let slab = face_slab(b, fi, d)?;
        acc = boolean_bodies(&acc, &slab, BoolOp::Union)?.body;
    }
    for e in 0..b.edges.len() {
        if b.edges[e].degenerate || convex(b, e) != Some(true) {
            continue;
        }
        let tube = edge_tube(b, e, d)?;
        acc = boolean_bodies(&acc, &tube, BoolOp::Union)?.body;
    }
    let mut done: Vec<P3> = Vec::new();
    for e in &b.edges {
        for v in e.v {
            let p = b.verts[v].p;
            if done.iter().any(|q| (q - p).norm() < 1e-9) {
                continue;
            }
            done.push(p);
            let ball = ball_body(p, d)?;
            acc = boolean_bodies(&acc, &ball, BoolOp::Union)?.body;
        }
    }
    Ok(acc)
}

pub fn shell(k: &mut NativeKernel, body: BodyId, spec: &ShellSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let t = spec.thickness;
    if !(t > 0.0) {
        return Err(KernelError::InvalidParameter("The shell's thickness must be positive".into()));
    }
    let remove: Vec<usize> = if spec.hollow { vec![] } else { spec.remove.iter().map(|f| f.0 as usize).collect() };
    if remove.is_empty() && !spec.hollow {
        return Err(KernelError::InvalidParameter("Pick the faces to remove, or make the shell hollow".into()));
    }
    if remove.iter().any(|&f| f >= b.faces.len()) {
        return Err(KernelError::InvalidParameter("unknown face".into()));
    }
    let size = b.bbox().diagonal();
    if !spec.outward && walls_cross(&b, &remove, t) {
        return Err(KernelError::InvalidParameter("The shell's walls would intersect themselves; reduce the thickness".into()));
    }
    let cur = Tracked::of(b.clone());
    let result = if !spec.outward {
        // The cavity: kept faces moved in by t, removed faces moved out past the body.
        let dist: Vec<f64> = (0..b.faces.len()).map(|f| if remove.contains(&f) { size * 0.1 + 1.0 } else { -t }).collect();
        // Concave edges between kept faces round the cavity (the walls are t thick
        // everywhere): build it from its definition then.
        let concave = (0..b.edges.len()).any(|e| !b.edges[e].degenerate && convex(&b, e) == Some(false));
        let cavity = if concave {
            cavity_by_regions(&b, &remove, t)?
        } else {
            match super::local::offset_sharp(&b, &dist) {
                Ok(c) if super::mass::body_mass(&c).volume > 0.0 => c,
                _ => cavity_by_regions(&b, &remove, t)?,
            }
        };
        if spec.hollow {
            // A closed void: the body with the cavity as an inner shell.
            let mut out = cur.apply(&cavity, &vec![Vec::new(); cavity.faces.len()], BoolOp::Subtract)?;
            if out.body.solids.len() != 1 {
                return Err(KernelError::OperationFailed("the hollow shell isn't one solid".into()));
            }
            out.body.solids[0].shells.sort();
            out
        } else {
            cur.apply(&cavity, &vec![Vec::new(); cavity.faces.len()], BoolOp::Subtract)?
        }
    } else {
        // Outward: the body grown by a ball, less the body, cut back at the removed faces.
        let grown = offset_rounded(&b, &vec![t; b.faces.len()])?;
        let mut walls = Tracked { src: vec![Vec::new(); grown.faces.len()], body: grown };
        for &f in &remove {
            let face = &b.faces[f];
            let Surface::Plane { f: fr } = &face.surf else {
                return Err(KernelError::Unsupported("removing a curved face from an outward shell"));
            };
            let n = fr.x.cross(&fr.y) * if face.sense { 1.0 } else { -1.0 };
            let hs = half_space(fr.o, n, size * 4.0 + t * 4.0 + 10.0)?;
            walls = walls.apply(&hs, &vec![Vec::new(); hs.faces.len()], BoolOp::Subtract)?;
        }
        let inner = Tracked::of(b.clone());
        // walls − body: the body's faces bound the inside.
        let r = boolean_bodies(&walls.body, &inner.body, BoolOp::Subtract)?;
        let src = r.from.iter().map(|s| s.iter().filter(|(o, _)| *o == 1).map(|(_, f)| super::blend::FaceSrc::Input(*f)).collect()).collect();
        Tracked { body: r.body, src }
    };
    let v0 = super::mass::body_mass(&b).volume;
    let v1 = super::mass::body_mass(&result.body).volume;
    if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
        eprintln!("shell: body {v0} result {v1} solids {} faces {}", result.body.solids.len(), result.body.faces.len());
    }
    if v1 <= 1e-9 || (!spec.outward && v1 > v0 + 1e-9) {
        return Err(KernelError::OperationFailed("the shell came out wrong".into()));
    }
    Ok(result.into_result(k, body))
}

/// A large box beyond the plane through `o` along the normal `n`.
fn half_space(o: P3, n: V3, size: f64) -> Result<Body> {
    let f = Frame::from_z(o, n);
    let c = |x: f64, y: f64, z: f64| f.o + f.x * x + f.y * y + f.z * z;
    super::blend::hexahedron(&[
        c(-size, -size, 0.0),
        c(size, -size, 0.0),
        c(size, size, 0.0),
        c(-size, size, 0.0),
        c(-size, -size, size),
        c(size, -size, size),
        c(size, size, size),
        c(-size, size, size),
    ])
}

/// Walls facing each other across the body closer than twice the thickness (kept, planar,
/// anti-parallel, overlapping).
fn walls_cross(b: &Body, remove: &[usize], t: f64) -> bool {
    let planes: Vec<(usize, P3, V3, Bbox)> = b
        .faces
        .iter()
        .enumerate()
        .filter(|(i, _)| !remove.contains(i))
        .filter_map(|(i, f)| match &f.surf {
            Surface::Plane { f: fr } => {
                let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
                let sub = super::kernel::subset(b, &[i], false);
                Some((i, fr.o, n, sub.bbox()))
            }
            _ => None,
        })
        .collect();
    for (i, oa, na, ba) in &planes {
        for (j, ob, nb, bb) in &planes {
            if i >= j || na.dot(nb) > -1.0 + 1e-9 {
                continue;
            }
            let gap = (ob - oa).dot(na).abs();
            // Overlap of the faces' boxes across the gap direction.
            let pa = ba.grow(gap);
            if gap < 2.0 * t + 1e-9 && pa.overlaps(bb) {
                return true;
            }
        }
    }
    false
}

/// The cavity built from its definition: the body less each kept face swept inward by t, a
/// tube round each concave edge between kept faces and a ball at each vertex of a concave edge.
pub fn cavity_by_regions(b: &Body, remove: &[usize], t: f64) -> Result<Body> {
    let mut acc = b.clone();
    for fi in 0..b.faces.len() {
        if remove.contains(&fi) {
            continue;
        }
        let slab = match &b.faces[fi].surf {
            Surface::Plane { .. } => face_slab(b, fi, -t)?,
            _ => face_band(b, fi, t)?,
        };
        acc = boolean_bodies(&acc, &slab, BoolOp::Subtract)?.body;
        if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
            eprintln!("after slab of face {fi} ({:?}): cavity {:.6} solids {}", b.faces[fi].surf.kind(), super::mass::body_mass(&acc).volume, acc.solids.len());
        }
    }
    let mut balls: Vec<P3> = Vec::new();
    for e in 0..b.edges.len() {
        if b.edges[e].degenerate || convex(b, e) != Some(false) {
            continue;
        }
        let tube = edge_tube(b, e, t)?;
        acc = boolean_bodies(&acc, &tube, BoolOp::Subtract)?.body;
        if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
            eprintln!("after tube of edge {e}: cavity {:.6} solids {}", super::mass::body_mass(&acc).volume, acc.solids.len());
        }
        // A closed edge (a whole circle) has no corner: its seam vertex needs no ball.
        if b.edges[e].v[0] == b.edges[e].v[1] {
            continue;
        }
        for v in b.edges[e].v {
            let p = b.verts[v].p;
            if !balls.iter().any(|q| (q - p).norm() < 1e-9) {
                balls.push(p);
            }
        }
    }
    for p in balls {
        let ball = ball_body(p, t)?;
        acc = boolean_bodies(&acc, &ball, BoolOp::Subtract)?.body;
    }
    Ok(acc)
}

/// The region within t inside a curved face (a cylinder's band).
fn face_band(b: &Body, fi: usize, t: f64) -> Result<Body> {
    let f = &b.faces[fi];
    match &f.surf {
        Surface::Cylinder { f: fr, r } => {
            // The ring between r and r ∓ t over the face's height range (whole turn).
            let outward = f.sense == fr.right_handed();
            let (r0, r1) = if outward { (r - t, *r) } else { (*r, r + t) };
            if r0 <= 0.0 {
                return Err(KernelError::Unsupported("a shell thicker than a cylinder's radius"));
            }
            let sub = super::kernel::subset(b, &[fi], false);
            let (lo, hi) = super::kernel::extent_along(&sub, &fr.o, &fr.z);
            let plane = Plane { origin: fr.o + fr.z * lo, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(fr.z) };
            let circ = |rr: f64| Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: rr, source: None }] };
            // The face's angular range (its p-curves' u).
            let (mut u0, mut u1) = (f64::INFINITY, f64::NEG_INFINITY);
            for c in f.loops.iter().flatten() {
                let e = &b.edges[c.edge];
                for i in 0..=8 {
                    let q = c.pc.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0);
                    u0 = u0.min(q.x);
                    u1 = u1.max(q.x);
                }
            }
            // A ring for a whole turn, else the annular sector behind the face.
            let region = if !(u1 - u0 < TAU - 1e-9) {
                Region { outer: circ(r1), holes: vec![circ(r0)], source: None }
            } else {
                let o = nalgebra::Point2::origin();
                let at = |rr: f64, u: f64| nalgebra::Point2::new(rr * u.cos(), rr * u.sin());
                Region {
                    outer: Loop {
                        curves: vec![
                            Curve2::Arc { center: o, radius: r1, start_angle: u0, sweep: u1 - u0, source: None },
                            Curve2::Line { a: at(r1, u1), b: at(r0, u1), source: None },
                            Curve2::Arc { center: o, radius: r0, start_angle: u1, sweep: u0 - u1, source: None },
                            Curve2::Line { a: at(r0, u0), b: at(r1, u0), source: None },
                        ],
                    },
                    holes: vec![],
                    source: None,
                }
            };
            let prof = Profile::new(plane, vec![region]);
            let regions = super::profile::regions(&prof)?;
            let mf = super::profile::merge_regions(&regions)?;
            let kind = super::sweep::Prism { plane, dir: fr.z };
            Ok(super::sweep::sweep_full(&mf, &[], &kind, 0.0, hi - lo, false, true)?.body)
        }
        _ => super::slab::face_slab(b, fi, -t),
    }
}
