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
        // A concave cylinder whose axis the offset reaches: the wedge out to its axis.
        if let Surface::Cylinder { f: fr, r } = &f.surf
            && d > 0.0
            && f.sense != fr.right_handed()
            && d >= r * (1.0 - 1e-9)
            && let Some(w) = cylinder_wedge(b, fi)?
        {
            return Ok(w);
        }
        return super::slab::face_slab(b, fi, d);
    };
    let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
    let plane = Plane { origin: fr.o, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(n) };
    let tmp = super::kernel::subset(b, &[fi], false);
    let mut k = NativeKernel::new();
    let id = k.insert(tmp);
    let prof = Profile::new(plane, vec![]);
    let mut mf = super::features::face_as_profile(&k, &prof, &FaceInput { body: id, face: FaceId(0), source: 0 })?;
    // A closed spline edge would sweep a band round a closed, non-periodic surface (one seam
    // edge used twice), which the boolean doesn't take: in two halves, two plain sides.
    let halve = |l: &mut super::profile::Loop2| {
        *l = l
            .drain(..)
            .flat_map(|p| {
                let closed = matches!(p.c, Curve::Nurbs(_)) && (p.c.point(p.t0) - p.c.point(p.t1)).norm() < 1e-9 * (1.0 + p.c.point(p.t0).coords.norm());
                if closed {
                    let m = 0.5 * (p.t0 + p.t1);
                    let (mut a, mut b) = (p.clone(), p);
                    // The halves in the loop's order (a reversed piece runs from t1 to t0).
                    if a.rev {
                        a.t0 = m;
                        b.t1 = m;
                    } else {
                        a.t1 = m;
                        b.t0 = m;
                    }
                    vec![a, b]
                } else {
                    vec![p]
                }
            })
            .collect();
    };
    halve(&mut mf.outer);
    mf.holes.iter_mut().for_each(halve);
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
            let q = ed.point(ed.t[0]);
            let axis = x.cross(y).normalize();
            let radial = (q - c).normalize();
            let tangent = axis.cross(&radial);
            // The profile plane's x runs out from the axis, its y down it (−axis); the axis is
            // the line x = −rc.
            let plane = Plane { origin: q, x_dir: nalgebra::Unit::new_normalize(radial), normal: nalgebra::Unit::new_normalize(tangent) };
            let outer = if r < rc * (1.0 - 1e-9) {
                Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: r, source: None }] }
            } else {
                // A tube wider than its arc's radius (a spindle torus): the balls along the
                // arc reach across the axis, but a point on the arc's side of the axis is
                // nearest the arc there, so the tube is the disc's part on that side revolved
                // (past the arc's ends the vertices' balls cover the rest).
                // (As wide as the arc's radius, the disc only touches the axis: its chord there
                // is kept 1e-4 of the radius long, a sliver 5e-9 of it off the axis.)
                let hmin = 1e-4 * r;
                let h = (r * r - rc * rc).max(0.0).sqrt();
                let (x, h) = if h < hmin { (-(r * r - hmin * hmin).sqrt(), hmin) } else { (-rc, h) };
                let (a, b) = (nalgebra::Point2::new(x, h), nalgebra::Point2::new(x, -h));
                let (ta, tb) = (h.atan2(x), (-h).atan2(x));
                Loop {
                    curves: vec![
                        Curve2::Arc { center: nalgebra::Point2::origin(), radius: r, start_angle: tb, sweep: ta - tb, source: None },
                        Curve2::Line { a, b, source: None },
                    ],
                }
            };
            let prof = Profile::new(plane, vec![Region { outer, holes: vec![], source: None }]);
            let regions = super::profile::regions(&prof)?;
            let mf = super::profile::merge_regions(&regions)?;
            let f = Frame { o: *c, x: radial, y: axis.cross(&radial), z: axis };
            let span = ed.t[1] - ed.t[0];
            let whole = span >= TAU - 1e-9;
            let kind = super::sweep::Revolve { plane, f };
            Ok(super::sweep::sweep_full(&mf, &[], &kind, 0.0, if whole { TAU } else { span }, whole, true)?.body)
        }
        // Any other curve: a disc square to the edge at its start swept along it (a pipe).
        _ => {
            // Where the edge curves tighter than the radius the pipe folds over itself: not a
            // valid solid (and its booleans fail only after every retry).
            let n = 256;
            let tight = (0..=n).any(|i| {
                let (_, d1, d2) = ed.curve.d2(ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / n as f64);
                d1.cross(&d2).norm() * r >= d1.norm().powi(3)
            });
            if tight {
                return Err(KernelError::Unsupported("a tube round an edge curved tighter than its radius"));
            }
            let p0 = ed.point(ed.t[0]);
            let t = ed.curve.tangent(ed.t[0]).try_normalize(1e-12).ok_or(KernelError::Unsupported("a tube round an edge with no direction"))?;
            let fr = Frame::from_z(p0, t);
            let plane = Plane { origin: p0, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(t) };
            let prof = Profile::new(plane, vec![Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: r, source: None }] }, holes: vec![], source: None }]);
            let mut k = NativeKernel::new();
            let body = k.insert(b.clone());
            let spec = SweepSpec { body: BodyKind::Solid, path: vec![PathCurve::Edge { body, edge: EdgeId(e as u64) }], control: SweepControl::None, faces: vec![] };
            let out = k.sweep_with(&prof, &spec)?;
            let [id] = out.bodies[..] else { return Err(KernelError::InvalidResult { op: "offset", why: "a tube round an edge isn't one body".into() }) };
            Ok((**k.get(id)?).clone())
        }
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

/// `acc` (op) `piece`; a failure says which piece (`what`) with `CADRS_OFFSET_DEBUG`, and
/// leaves both bodies in `CADRS_DUMP_DIR`.
fn combine(acc: &Body, piece: &Body, op: BoolOp, what: &dyn Fn() -> String) -> Result<Body> {
    let debug = std::env::var_os("CADRS_OFFSET_DEBUG").is_some();
    let dump = || {
        if std::env::var_os("CADRS_DUMP_DIR").is_some() {
            super::kernel::dump("offset-target", acc, &format!("{op:?} {}", what()));
            super::kernel::dump("offset-tool", piece, &format!("{op:?} {}", what()));
        }
    };
    // (The boolean returns only valid bodies, so a piece is never built on a broken result:
    // one that fails is tried again later.)
    Ok(boolean_bodies(acc, piece, op)
        .inspect_err(|e| {
            if debug {
                eprintln!("offset: {op:?} with {}: {e:?}", what());
            }
            dump();
        })?
        .body)
}

/// The body with its adjacent faces on one surface merged (a face imprinted with a region of
/// its own plane): the offset doesn't depend on how a surface is split, but the pieces' slabs
/// would meet along bands on one surface, which the boolean can't settle. Unchanged when the
/// merge doesn't give a valid solid.
fn merged(b: &Body) -> Body {
    let mut k = NativeKernel::new();
    let id = k.insert(b.clone());
    match crate::Kernel::simplify(&mut k, id).ok().and_then(|r| r.bodies.first().and_then(|x| k.get(*x).ok()).map(|x| (**x).clone())) {
        Some(m) if m.check().is_empty() && m.faces.len() < b.faces.len() => m,
        _ => b.clone(),
    }
}

/// The body grown by a ball of radius d (all faces moved out by d, edges and corners rounded).
pub fn offset_rounded(b: &Body, dist: &[f64]) -> Result<Body> {
    let d = dist[0];
    if dist.iter().any(|x| (x - d).abs() > 1e-12) || d <= 0.0 {
        return Err(KernelError::Unsupported("rounded offsets with different distances per face"));
    }
    grown_except(b, d, &[])
}

/// A face's outward normal at a point on it.
fn outward_normal(b: &Body, fi: usize, p: &P3) -> V3 {
    let f = &b.faces[fi];
    let (u, v) = f.surf.project(p, None);
    f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 }
}

/// The body grown by a ball of radius d, but for the parts beyond the planes `cut` (a point
/// and an outward normal: an outward shell's removed faces, beyond which the caller cuts the
/// grown body away): the slab of a face on a cut plane, the tube round an edge of one whose
/// other face doesn't point below the plane, and the ball at a vertex of one whose faces don't
/// (the points within d of those lie beyond the plane or within d of a face's middle, in its
/// slab). Rounds there are the costliest pieces to unite (a tube round a spline touches its
/// faces' slabs along their offsets) and are cut away whole.
fn grown_except(b: &Body, d: f64, cut: &[(P3, V3)]) -> Result<Body> {
    let merged = merged(b);
    let b = &merged;
    let size = 1.0 + b.bbox().diagonal();
    let ef = b.edge_faces();
    // The cut plane each face lies on.
    let on_cut: Vec<Option<V3>> = (0..b.faces.len())
        .map(|fi| {
            let Surface::Plane { f: fr } = &b.faces[fi].surf else { return None };
            let n = fr.x.cross(&fr.y).normalize() * if b.faces[fi].sense { 1.0 } else { -1.0 };
            cut.iter().find(|(o, m)| n.dot(m) > 1.0 - 1e-9 && (fr.o - o).dot(m).abs() < 1e-9 * size).map(|(_, m)| *m)
        })
        .collect();
    // A face that doesn't point below the cut plane `m` at `p`.
    let above = |fi: usize, m: &V3, p: &P3| outward_normal(b, fi, p).dot(m) >= -1e-9;
    let mut pieces: Vec<(Body, String)> = Vec::new();
    for fi in 0..b.faces.len() {
        if on_cut[fi].is_none() {
            pieces.push((face_slab(b, fi, d)?, format!("the slab of face {fi} ({:?})", b.faces[fi].surf.kind())));
        }
    }
    for e in 0..b.edges.len() {
        let ed = &b.edges[e];
        if ed.degenerate || convex(b, e) != Some(true) {
            continue;
        }
        let beyond = ef[e].iter().any(|&f| {
            on_cut[f].is_some_and(|m| {
                ef[e].iter().all(|&g| (0..=8).all(|i| above(g, &m, &ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0))))
            })
        });
        if !beyond {
            pieces.push((edge_tube(b, e, d)?, format!("the tube round edge {e}")));
        }
    }
    // Each vertex's faces.
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); b.verts.len()];
    for (e, fs) in ef.iter().enumerate() {
        for &v in &b.edges[e].v {
            for &f in fs {
                if !vf[v].contains(&f) {
                    vf[v].push(f);
                }
            }
        }
    }
    let mut done: Vec<P3> = Vec::new();
    for e in &b.edges {
        for v in e.v {
            let p = b.verts[v].p;
            if done.iter().any(|q| (q - p).norm() < 1e-9) {
                continue;
            }
            done.push(p);
            let beyond = vf[v].iter().any(|&f| on_cut[f].is_some_and(|m| vf[v].iter().all(|&g| above(g, &m, &p))));
            if !beyond {
                pieces.push((ball_body(p, d)?, format!("the ball at {p:?}")));
            }
        }
    }
    combine_all(b.clone(), pieces, BoolOp::Union)
}

/// `acc` combined (`op`) with each piece in turn. A piece whose boolean fails is tried again
/// after the others: a union or difference of pieces doesn't depend on their order, but the
/// touching configurations a boolean has to settle do (a slab's side against a neighbour's
/// slab, not yet merged into a face). Fails when a round makes no progress, or after three
/// rounds (each failed boolean costs its retries).
fn combine_all(mut acc: Body, mut todo: Vec<(Body, String)>, op: BoolOp) -> Result<Body> {
    for round in 0..3 {
        let n = todo.len();
        let mut left = Vec::new();
        let mut first: Option<KernelError> = None;
        for (piece, what) in todo {
            match combine(&acc, &piece, op, &|| what.clone()) {
                Ok(r) => {
                    acc = r;
                    if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
                        eprintln!("after {what}: {:.6} solids {}", super::mass::body_mass(&acc).volume, acc.solids.len());
                    }
                }
                Err(e) => {
                    first.get_or_insert(e);
                    left.push((piece, what));
                }
            }
        }
        match first {
            None => return Ok(acc),
            Some(e) if left.len() == n || round == 2 => return Err(e),
            Some(_) => todo = left,
        }
    }
    Err(KernelError::InvalidResult { op: "offset", why: "the offset's pieces don't combine".into() })
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
    // Walls closer than twice the thickness: the faces moved in cross over each other, so no
    // sharp inset exists. Rounded, the cavity from its definition leaves the body solid where
    // they meet (OCCT's thick solids do the same); with no cavity left, or sharp, an error.
    let crossing = !spec.outward && walls_cross(&b, &remove, t);
    let too_thick = || KernelError::InvalidParameter("The shell's walls would intersect themselves; reduce the thickness".into());
    if crossing && spec.sharp {
        return Err(too_thick());
    }
    let cur = Tracked::of(b.clone());
    let v0 = super::mass::body_mass(&b).volume;
    let result = if !spec.outward {
        // The cavity: kept faces moved in by t, removed faces moved out past the body, or,
        // failing that (a spline wall's offset doesn't reach that far), where they are (the
        // cavity opens flush with them).
        let dist: Vec<f64> = (0..b.faces.len()).map(|f| if remove.contains(&f) { size * 0.1 + 1.0 } else { -t }).collect();
        let flush: Vec<f64> = dist.iter().enumerate().map(|(f, d)| if remove.contains(&f) { 0.0 } else { *d }).collect();
        // Concave edges between kept faces round the cavity (the walls are t thick
        // everywhere): build it from its definition then. Each way of making the cavity is
        // tried in turn until the body less it comes out (a cavity flush with a removed face
        // shares that face with the body, which the boolean may not settle).
        let concave = (0..b.edges.len()).any(|e| !b.edges[e].degenerate && convex(&b, e) == Some(false));
        let sharp = |x: &Vec<f64>| super::local::offset_sharp(&b, x).and_then(|c| {
            if super::mass::body_mass(&c).volume > 0.0 { Ok(c) } else { Err(KernelError::TooLarge { op: "shell", why: "the inset is empty".into() }) }
        });
        let regions = || cavity_by_regions(&b, &remove, t);
        let ways: Vec<Box<dyn Fn() -> Result<Body>>> = if (concave || crossing) && !spec.sharp {
            vec![Box::new(regions)]
        } else {
            vec![Box::new(|| sharp(&dist)), Box::new(|| sharp(&flush)), Box::new(regions)]
        };
        let mut first: Option<KernelError> = None;
        let mut done = None;
        for way in &ways {
            let out = way().and_then(|cavity| {
                let mut out = cur.apply(&cavity, &vec![Vec::new(); cavity.faces.len()], BoolOp::Subtract)?;
                if spec.hollow {
                    // A closed void: the body with the cavity as an inner shell.
                    if out.body.solids.len() != 1 {
                        return Err(KernelError::InvalidResult { op: "shell", why: "the hollow shell isn't one solid".into() });
                    }
                    out.body.solids[0].shells.sort();
                }
                let v1 = super::mass::body_mass(&out.body).volume;
                if v1 <= 1e-9 || v1 > v0 + 1e-9 {
                    return Err(KernelError::InvalidResult { op: "shell", why: "the shell came out wrong".into() });
                }
                // Walls that meet everywhere leave no cavity.
                if crossing && v1 >= v0 * (1.0 - 1e-9) {
                    return Err(too_thick());
                }
                Ok(out)
            });
            match out {
                Ok(o) => {
                    done = Some(o);
                    break;
                }
                Err(e) => {
                    first.get_or_insert(e);
                }
            }
        }
        match (done, first) {
            (Some(o), _) => o,
            (None, Some(_)) if crossing => return Err(too_thick()),
            (None, Some(e)) => return Err(e),
            (None, None) => return Err(KernelError::TooLarge { op: "shell", why: "no cavity".into() }),
        }
    } else {
        // Outward: the body grown by a ball, less the body, cut back at the removed faces.
        let rounded = || -> Result<Tracked> {
            // (Not the rounds beyond the removed faces, which are cut away.)
            let cut: Vec<(P3, V3)> = remove
                .iter()
                .filter_map(|&f| match &b.faces[f].surf {
                    Surface::Plane { f: fr } => Some((fr.o, fr.x.cross(&fr.y).normalize() * if b.faces[f].sense { 1.0 } else { -1.0 })),
                    _ => None,
                })
                .collect();
            let grown = grown_except(&b, t, &cut)?;
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
            // walls − body: the body's faces bound the inside.
            let r = boolean_bodies(&walls.body, &b, BoolOp::Subtract)?;
            let src = r.from.iter().map(|s| s.iter().filter(|(o, _)| *o == 1).map(|(_, f)| super::blend::FaceSrc::Input(*f)).collect()).collect();
            Ok(Tracked { body: r.body, src })
        };
        // With sharp corners: the kept faces moved out by t (the removed ones staying where
        // they are, so the walls end flush with them), less the body. They differ from rounded
        // ones only at a convex edge between kept faces; where the sharp walls can't be made
        // (an offset face that doesn't reach its neighbour, a boolean that fails), rounded.
        let ef = b.edge_faces();
        let corners = (0..b.edges.len()).any(|e| !b.edges[e].degenerate && convex(&b, e) == Some(true) && ef[e].iter().all(|f| !remove.contains(f)));
        let sharp = || -> Result<Tracked> {
            let dist: Vec<f64> = (0..b.faces.len()).map(|f| if remove.contains(&f) { 0.0 } else { t }).collect();
            let grown = super::local::offset_sharp(&b, &dist)?;
            let r = boolean_bodies(&grown, &b, BoolOp::Subtract)?;
            let src = r.from.iter().map(|s| s.iter().filter(|(o, _)| *o == 1).map(|(_, f)| super::blend::FaceSrc::Input(*f)).collect()).collect();
            Ok(Tracked { body: r.body, src })
        };
        if spec.sharp && corners {
            match sharp() {
                Ok(r) => r,
                Err(e) => {
                    if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
                        eprintln!("shell: sharp outward walls failed ({e}); rounded");
                    }
                    rounded()?
                }
            }
        } else {
            rounded()?
        }
    };
    // The walls' faces the booleans couldn't trace (the cavity's, the grown body's): the kept
    // faces' offsets (modified) and the rounds (generated), from where they lie.
    let mut result = result;
    let dist: Vec<f64> = (0..b.faces.len()).map(|f| if remove.contains(&f) { f64::NAN } else if spec.outward { t } else { -t }).collect();
    let found = offset_sources(&b, body, &result.body, &dist, !spec.sharp);
    for (s, f) in result.src.iter_mut().zip(found) {
        if s.is_empty() {
            *s = f;
        }
    }
    let v1 = super::mass::body_mass(&result.body).volume;
    if std::env::var("CADRS_SHELL_DEBUG").is_ok() {
        eprintln!("shell: body {v0} result {v1} solids {} faces {}", result.body.solids.len(), result.body.faces.len());
    }
    if v1 <= 1e-9 || (!spec.outward && v1 > v0 + 1e-9) {
        return Err(KernelError::InvalidResult { op: "shell", why: "the shell came out wrong".into() });
    }
    Ok(result.into_result(k, body))
}

/// What each face of `out` came from, when `out` is bounded by offsets of `b`'s faces by `dist`
/// (a face not offset at `f64::NAN`) and their rounds: the faces whose offset surface it lies on
/// ([`FaceSrc::Input`], the face moved: modified), else the edge or vertex its round is about
/// ([`FaceSrc::New`], generated: it lies the distance from that edge or vertex). Names survive
/// an offset however its pieces were united. Empty for a face that is neither.
pub fn offset_sources(b: &Body, input: BodyId, out: &Body, dist: &[f64], rounded: bool) -> Vec<Vec<super::blend::FaceSrc>> {
    use super::blend::FaceSrc;
    let size = 1.0 + b.bbox().diagonal();
    let moved: Vec<Option<Surface>> = b
        .faces
        .iter()
        .zip(dist)
        .map(|(f, d)| if d.is_nan() { None } else if *d == 0.0 { Some(f.surf.clone()) } else { super::local::offset_surface(&f.surf, f.sense, *d) })
        .collect();
    let reach = dist.iter().filter(|d| !d.is_nan()).fold(0.0f64, |m, d| m.max(d.abs()));
    // A sharp offset's step between neighbours moved apart (or a face that closes over a slot
    // their moves filled): from the input edges it holds, on its plane and within it.
    let steps = |f: &super::topo::Face| -> Vec<FaceSrc> {
        if !matches!(f.surf, Surface::Plane { .. }) {
            return Vec::new();
        }
        let tol = 1e-7 * size;
        let polys = super::classify::face_polygons(out, f);
        (0..b.edges.len())
            .filter(|&e| !b.edges[e].degenerate)
            .filter(|&e| {
                let ed = &b.edges[e];
                let m = ed.point(ed.mid_t());
                if f.surf.distance(&m) > tol {
                    return false;
                }
                let (u, v) = f.surf.project(&m, None);
                !matches!(super::classify::in_face(&polys, &f.surf, P2::new(u, v), tol), super::classify::InFace::Outside)
            })
            .map(|e| FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(e as u64) }))
            .collect()
    };
    out.faces
        .iter()
        .map(|f| {
            // A point of the face: the middle of its outer loop in (u, v), on its surface.
            let mut sum = P2::origin();
            let mut n = 0.0;
            for c in f.loops.first().into_iter().flatten() {
                let e = &out.edges[c.edge];
                for i in 0..8 {
                    let q = c.pc.point(e.t[0] + (e.t[1] - e.t[0]) * (i as f64 + 0.5) / 8.0);
                    sum += q.coords.xy();
                    n += 1.0;
                }
            }
            if n == 0.0 {
                return Vec::new();
            }
            let p = f.surf.eval(sum.x / n, sum.y / n);
            // (Offsets of splines are fits.)
            let near = |s: &Surface| s.distance(&p) < if matches!(s, Surface::Nurbs(_)) { 1e-5 } else { 1e-7 } * size;
            let faces: Vec<FaceSrc> = (0..b.faces.len()).filter(|&fi| moved[fi].as_ref().is_some_and(&near)).map(FaceSrc::Input).collect();
            if !faces.is_empty() || reach == 0.0 {
                return faces;
            }
            if !rounded {
                return steps(f);
            }
            // A round: the edge it lies the distance from (nearest within the edge, not at an
            // end: a corner's ball is that far from its edges' ends too), else the vertex.
            let tol = 1e-6 * size;
            let off = |d: f64| (d - reach).abs();
            let edge = (0..b.edges.len())
                .filter(|&e| !b.edges[e].degenerate)
                .filter_map(|e| {
                    let ed = &b.edges[e];
                    let t = ed.curve.closest(&p, ed.t[0], ed.t[1]);
                    let inside = (t - ed.t[0]).abs().min((ed.t[1] - t).abs()) > 1e-9 * (ed.t[1] - ed.t[0]).abs();
                    inside.then(|| (e, off((ed.curve.point(t) - p).norm())))
                })
                .filter(|x| x.1 < tol)
                .min_by(|a, c| a.1.total_cmp(&c.1));
            if let Some((e, _)) = edge {
                return vec![FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(e as u64) })];
            }
            if let Some(v) = (0..b.verts.len()).find(|&v| off((b.verts[v].p - p).norm()) < tol) {
                return vec![FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(v as u64) })];
            }
            steps(f)
        })
        .collect()
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

/// The body shrunk by a ball of radius `d` (every face moved in by d, the concave edges and
/// corners rounded: the points farther than d inside the body), the faces at distance 0 left
/// where they are (SMLib's inset with its convex edges, the body's concave ones, rounded:
/// `SmOffsetExecutive` with `SetExtendConvexEdges(FALSE)`). A body with no concave edge
/// shrinks to its sharp inset.
pub fn inset_rounded(b: &Body, dist: &[f64]) -> Result<Body> {
    let t = -dist.iter().copied().fold(0.0, f64::min);
    if dist.iter().any(|x| *x != 0.0 && (x + t).abs() > 1e-12) || !(t > 0.0) {
        return Err(KernelError::Unsupported("rounded insets with different distances per face"));
    }
    let concave = (0..b.edges.len()).any(|e| !b.edges[e].degenerate && convex(b, e) == Some(false));
    if !concave
        && let Ok(c) = super::local::offset_sharp(b, dist)
    {
        return Ok(c);
    }
    let still: Vec<usize> = (0..b.faces.len()).filter(|&f| dist[f] == 0.0).collect();
    inner_core(b, &still, t, false)
}

/// A tube round each concave edge and a ball at each of their vertices (not round the `still`
/// faces' edges and vertices unless `round_still`).
fn rounds(b: &Body, still: &[usize], t: f64, round_still: bool) -> Result<Vec<(Body, String)>> {
    let ef = b.edge_faces();
    let at_still = |e: usize| !round_still && ef[e].iter().any(|f| still.contains(f));
    let still_verts: Vec<usize> = (0..b.edges.len()).filter(|&e| at_still(e)).flat_map(|e| b.edges[e].v).collect();
    let mut pieces: Vec<(Body, String)> = Vec::new();
    let mut balls: Vec<P3> = Vec::new();
    for e in 0..b.edges.len() {
        if b.edges[e].degenerate || convex(b, e) != Some(false) || at_still(e) {
            continue;
        }
        pieces.push((edge_tube(b, e, t)?, format!("the tube round edge {e}")));
        // A closed edge (a whole circle) has no corner: its seam vertex needs no ball.
        if b.edges[e].v[0] == b.edges[e].v[1] {
            continue;
        }
        for v in b.edges[e].v {
            let p = b.verts[v].p;
            if !still_verts.contains(&v) && !balls.iter().any(|q| (q - p).norm() < 1e-9) {
                balls.push(p);
            }
        }
    }
    for p in balls {
        pieces.push((ball_body(p, t)?, format!("the ball at {p:?}")));
    }
    Ok(pieces)
}

/// The cavity built from its definition: the body less each kept face swept inward by t, a
/// tube round each concave edge between kept faces and a ball at each vertex of a concave edge.
pub fn cavity_by_regions(b: &Body, remove: &[usize], t: f64) -> Result<Body> {
    inner_core(b, remove, t, true)
}

/// The body less each face but `still` swept inward by t, a tube round each concave edge and a
/// ball at each vertex of one; with `round_still` false, not round the edges and vertices of
/// the `still` faces (which stay where they are).
fn inner_core(b: &Body, remove: &[usize], t: f64, round_still: bool) -> Result<Body> {
    let mut pieces: Vec<(Body, String)> = Vec::new();
    for fi in 0..b.faces.len() {
        if remove.contains(&fi) {
            continue;
        }
        let slab = match &b.faces[fi].surf {
            Surface::Plane { .. } => face_slab(b, fi, -t)?,
            _ => face_band(b, fi, t)?,
        };
        pieces.push((slab, format!("the inward slab of face {fi} ({:?})", b.faces[fi].surf.kind())));
    }
    pieces.extend(rounds(b, remove, t, round_still)?);
    combine_all(b.clone(), pieces, BoolOp::Subtract)
}

/// The region within t inside a curved face (a cylinder's band). A convex round (a cylinder
/// or sphere bulging out) at least t thick inside collapses onto its axis or centre: the cavity
/// is sharp there, its walls the planes the round is tangent to moved in by t, so the round's
/// part is the block between those planes and their offsets ([`collapsed_round`]).
fn face_band(b: &Body, fi: usize, t: f64) -> Result<Body> {
    let f = &b.faces[fi];
    let convex = f.surf.frame().is_some_and(|fr| f.sense == fr.right_handed());
    match &f.surf {
        Surface::Cylinder { r, .. } | Surface::Sphere { r, .. } if convex && t >= r * (1.0 - 1e-9) => {
            return collapsed_round(b, fi, t)?.ok_or(KernelError::Unsupported("a shell thicker than a round's radius"));
        }
        _ => {}
    }
    if let Surface::Cylinder { f: fr, r } = &f.surf {
        // The ring between r and r ∓ t behind the face.
        let outward = f.sense == fr.right_handed();
        let (r0, r1) = if outward { (r - t, *r) } else { (*r, r + t) };
        if r0 <= 0.0 {
            return Err(KernelError::Unsupported("a shell thicker than a cylinder's radius"));
        }
        if let Some(band) = cylinder_band(b, fi, r0, r1)? {
            return Ok(band);
        }
    }
    super::slab::face_slab(b, fi, -t)
}

/// A convex round's part of a shell's walls when the shell is at least as thick as the round's
/// radius: the block between the planes the round is tangent to along its edges and those
/// planes moved in by t (a box edge's round: the t × t square at the edge, along it; a box
/// corner's ball: the t cube at the corner). Holds the round and what lies between it and its
/// neighbours' moved planes. Only for a cylinder over a rectangle of angle (under a half turn)
/// and height, and a sphere's octant (bounded by three great circles square to each other);
/// None for another face.
fn collapsed_round(b: &Body, fi: usize, t: f64) -> Result<Option<Body>> {
    let f = &b.faces[fi];
    // Planes' normals e_i (at distance r from the centre point o): the block's corners are
    // o + Σ a_i·(dual of e_i) with a_i ∈ {r − t, r}.
    let corner = |e: &[V3], a: &[f64]| -> Option<V3> {
        match e {
            [e0, e1] => {
                // α e0 + β e1 with α + β g = a0, α g + β = a1.
                let g = e0.dot(e1);
                let det = 1.0 - g * g;
                (det > 1e-9).then(|| e0 * ((a[0] - g * a[1]) / det) + e1 * ((a[1] - g * a[0]) / det))
            }
            [e0, e1, e2] => Some(e0 * a[0] + e1 * a[1] + e2 * a[2]),
            _ => None,
        }
    };
    let cycle = |lo: f64, hi: f64| [[lo, lo], [hi, lo], [hi, hi], [lo, hi]];
    match &f.surf {
        Surface::Cylinder { f: fr, r } => {
            let Some((lo, hi, u0, u1)) = cylinder_domain(b, fi) else { return Ok(None) };
            if u1 - u0 >= PI - 1e-9 {
                return Ok(None);
            }
            let radial = |u: f64| fr.x * u.cos() + fr.y * u.sin();
            let e = [radial(u0), radial(u1)];
            let mut c = [P3::origin(); 8];
            for (k, a) in cycle(r - t, *r).iter().enumerate() {
                let Some(q) = corner(&e, a) else { return Ok(None) };
                c[k] = fr.o + fr.z * lo + q;
                c[k + 4] = fr.o + fr.z * hi + q;
            }
            Ok(Some(super::blend::hexahedron(&c)?))
        }
        Surface::Sphere { f: fr, r } => {
            // The planes of its edges, which must hold the centre.
            let o = fr.o;
            let mut normals: Vec<V3> = Vec::new();
            let mut mid = V3::zeros();
            for co in f.loops.iter().flatten() {
                let e = &b.edges[co.edge];
                if e.degenerate {
                    continue;
                }
                let Curve::Circle { c: cc, x, y, r: rc } = &e.curve else { return Ok(None) };
                if (cc - o).norm() > 1e-7 * (1.0 + r) || (rc - r).abs() > 1e-7 * (1.0 + r) {
                    return Ok(None);
                }
                let n = x.cross(y).normalize();
                if !normals.iter().any(|m| m.cross(&n).norm() < 1e-9) {
                    normals.push(n);
                }
                mid += (e.point(e.mid_t()) - o).normalize();
            }
            let [n1, n2, n3] = normals[..] else { return Ok(None) };
            if n1.dot(&n2).abs() > 1e-9 || n2.dot(&n3).abs() > 1e-9 || n3.dot(&n1).abs() > 1e-9 {
                return Ok(None);
            }
            // The octant's axes: each plane's normal towards the face.
            let side = |n: V3| if n.dot(&mid) >= 0.0 { n } else { -n };
            let e = [side(n1), side(n2), side(n3)];
            let mut c = [P3::origin(); 8];
            for (k, a) in cycle(r - t, *r).iter().enumerate() {
                for (z, off) in [(r - t, 0), (*r, 4)] {
                    let Some(q) = corner(&e, &[a[0], a[1], z]) else { return Ok(None) };
                    c[k + off] = o + q;
                }
            }
            Ok(Some(super::blend::hexahedron(&c)?))
        }
        _ => Ok(None),
    }
}

/// The points of a concave cylindrical face's slab nearer the face than its axis: the wedge
/// between the face and the axis. A concave face's normals meet at its axis, so no point past
/// the axis is nearer the face than the rest of the body: when the offset reaches the axis
/// this is the face's part of the grown body (its offset surface would have no radius).
fn cylinder_wedge(b: &Body, fi: usize) -> Result<Option<Body>> {
    let Surface::Cylinder { r, .. } = &b.faces[fi].surf else { return Ok(None) };
    cylinder_band(b, fi, 0.0, *r)
}

/// The solid between radii r0 and r1 (r0 = 0: out from the axis) over a cylindrical face's
/// angles and heights: an annular sector (a ring for a whole turn; a pie slice or a disc from
/// the axis) swept along the axis. None unless the face spans whole axial lines between two
/// circles square to the axis (its domain a rectangle in angle and height).
fn cylinder_band(b: &Body, fi: usize, r0: f64, r1: f64) -> Result<Option<Body>> {
    let Surface::Cylinder { f: fr, .. } = &b.faces[fi].surf else { return Ok(None) };
    let Some((lo, hi, u0, u1)) = cylinder_domain(b, fi) else { return Ok(None) };
    let plane = Plane { origin: fr.o + fr.z * lo, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(fr.z) };
    let circ = |rr: f64| Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: rr, source: None }] };
    // The angles in the plane (whose y is z × x: the frame's y for a right-handed frame, else
    // the other way).
    let whole = !(u1 - u0 < TAU - 1e-9);
    let (u0, u1) = if fr.right_handed() { (u0, u1) } else { (-u1, -u0) };
    let o = nalgebra::Point2::origin();
    let at = |rr: f64, u: f64| nalgebra::Point2::new(rr * u.cos(), rr * u.sin());
    let region = match (whole, r0 > 0.0) {
        (true, true) => Region { outer: circ(r1), holes: vec![circ(r0)], source: None },
        (true, false) => Region { outer: circ(r1), holes: vec![], source: None },
        (false, true) => Region {
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
        },
        (false, false) => Region {
            outer: Loop {
                curves: vec![
                    Curve2::Arc { center: o, radius: r1, start_angle: u0, sweep: u1 - u0, source: None },
                    Curve2::Line { a: at(r1, u1), b: o, source: None },
                    Curve2::Line { a: o, b: at(r1, u0), source: None },
                ],
            },
            holes: vec![],
            source: None,
        },
    };
    let prof = Profile::new(plane, vec![region]);
    let regions = super::profile::regions(&prof)?;
    let mf = super::profile::merge_regions(&regions)?;
    let kind = super::sweep::Prism { plane, dir: fr.z };
    Ok(Some(super::sweep::sweep_full(&mf, &[], &kind, 0.0, hi - lo, false, true)?.body))
}

/// A cylindrical face's heights along its axis and angles (its p-curves' u), when it spans
/// whole axial lines between two circles square to the axis (its domain a rectangle in angle
/// and height): (lo, hi, u0, u1).
fn cylinder_domain(b: &Body, fi: usize) -> Option<(f64, f64, f64, f64)> {
    let f = &b.faces[fi];
    let Surface::Cylinder { f: fr, r } = &f.surf else { return None };
    let size = 1.0 + r + fr.o.coords.norm();
    let rect = f.loops.iter().flatten().all(|c| {
        let e = &b.edges[c.edge];
        e.degenerate
            || match &e.curve {
                Curve::Line { d, .. } => d.normalize().cross(&fr.z).norm() < 1e-9,
                Curve::Circle { c: cc, x, y, r: rc } => {
                    let w = cc - fr.o;
                    x.cross(y).normalize().cross(&fr.z).norm() < 1e-9 && (w - fr.z * w.dot(&fr.z)).norm() < 1e-7 * size && (rc - r).abs() < 1e-7 * size
                }
                _ => false,
            }
    });
    if !rect {
        return None;
    }
    let sub = super::kernel::subset(b, &[fi], false);
    let (lo, hi) = super::kernel::extent_along(&sub, &fr.o, &fr.z);
    let (mut u0, mut u1) = (f64::INFINITY, f64::NEG_INFINITY);
    for c in f.loops.iter().flatten() {
        let e = &b.edges[c.edge];
        for i in 0..=8 {
            let q = c.pc.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0);
            u0 = u0.min(q.x);
            u1 = u1.max(q.x);
        }
    }
    Some((lo, hi, u0, u1))
}
