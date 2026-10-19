//! The slab of a face: the solid swept by the face moving along its normals by a distance (out
//! of the body for d > 0, into it for d < 0). Bounded by the face, its offset (the same loops
//! on the offset surface), and a side face between each boundary edge and its offset curve:
//! a plane between parallel lines or coplanar circles, else the ruled surface. The edges at the
//! vertices are the normal lines. Built through [`XBuilder`], which makes the p-curves, seams
//! and senses.

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::surface::Surface;
use super::topo::Body;
use super::xbuild::{XBuilder, XFace};
use crate::{KernelError, Result};
use std::collections::HashMap;
use std::sync::Arc;

fn unsupported(m: &'static str) -> KernelError {
    KernelError::Unsupported(m)
}

/// The face's outward normal at a point near it.
fn normal_at(s: &Surface, sense: bool, p: &P3) -> V3 {
    let (u, v) = s.project(p, None);
    s.normal(u, v).normalize() * if sense { 1.0 } else { -1.0 }
}

/// The curve `c` moved by `d` along the face's normal, with the same parameters where that is
/// exact (lines, circles), else a cubic through samples on [t0, t1].
fn offset_curve(s: &Surface, sense: bool, c: &Curve, t0: f64, t1: f64, d: f64) -> Curve {
    let at = |t: f64| {
        let p = c.point(t);
        p + normal_at(s, sense, &p) * d
    };
    let tol = 1e-9 * (1.0 + d.abs() + (c.point(t0) - c.point(t1)).norm());
    match c {
        Curve::Line { o, d: dir } => {
            let (a, b) = (at(t0), at(t1));
            let mid = at(0.5 * (t0 + t1));
            let shift = a - c.point(t0);
            if (b - c.point(t1) - shift).norm() < tol && (mid - c.point(0.5 * (t0 + t1)) - shift).norm() < tol {
                return Curve::Line { o: o + shift, d: *dir };
            }
        }
        Curve::Circle { .. } => {
            // A circle again when three offset points' circle holds the others.
            let (p0, p1, p2) = (at(t0), at(t0 + (t1 - t0) / 3.0), at(t0 + 2.0 * (t1 - t0) / 3.0));
            if let Some((cc, nn, r)) = circle3(&p0, &p1, &p2) {
                let x = (at(0.0) - cc) / r;
                let y = (at(std::f64::consts::FRAC_PI_2) - cc) / r;
                let cand = Curve::Circle { c: cc, x, y, r };
                let ok = (0..=8).all(|i| {
                    let t = t0 + (t1 - t0) * i as f64 / 8.0;
                    (cand.point(t) - at(t)).norm() < tol * 1e3
                }) && x.dot(&nn).abs() < 1e-9;
                if ok {
                    return cand;
                }
            }
        }
        _ => {}
    }
    let n = 48;
    let pts: Vec<P3> = (0..=n).map(|i| at(t0 + (t1 - t0) * i as f64 / n as f64)).collect();
    let params: Vec<f64> = (0..=n).map(|i| t0 + (t1 - t0) * i as f64 / n as f64).collect();
    Curve::Nurbs(Arc::new(NurbsCurve::interpolate(&pts, &params, 3, None, None)))
}

/// The circle through three points: centre, normal, radius.
fn circle3(a: &P3, b: &P3, c: &P3) -> Option<(P3, V3, f64)> {
    let (u, v) = (b - a, c - a);
    let w = u.cross(&v);
    let ww = w.norm_squared();
    if ww < 1e-24 {
        return None;
    }
    let cc = a + (v * u.norm_squared() - u * v.norm_squared()).cross(&w) * (-0.5 / ww);
    Some((cc, w.normalize(), (a - cc).norm()))
}

/// The side surface between an edge and its offset: a plane when they lie in one, else ruled.
fn side_surface(c: &Curve, t0: f64, t1: f64, o: &Curve) -> Surface {
    let pts: Vec<P3> = (0..=8).flat_map(|i| {
        let t = t0 + (t1 - t0) * i as f64 / 8.0;
        [c.point(t), o.point(t)]
    }).collect();
    let p0 = pts[0];
    let far = pts.iter().max_by(|a, b| (*a - p0).norm().total_cmp(&(*b - p0).norm())).copied().unwrap_or(p0);
    if let Some(x) = (far - p0).try_normalize(1e-12) {
        if let Some(n) = pts.iter().map(|p| x.cross(&(p - p0))).max_by(|a, b| a.norm().total_cmp(&b.norm())).and_then(|n| n.try_normalize(1e-12)) {
            let size = (far - p0).norm();
            if pts.iter().all(|p| (p - p0).dot(&n).abs() < 1e-9 * (1.0 + size)) {
                return Surface::Plane { f: Frame { o: p0, x, y: n.cross(&x), z: n } };
            }
        }
    }
    let (mut a, mut b) = (c.to_nurbs(t0, t1), o.to_nurbs(t0, t1));
    while a.degree < b.degree {
        a = a.elevated();
    }
    while b.degree < a.degree {
        b = b.elevated();
    }
    let (a, b) = (a.reparam(0.0, 1.0), b.reparam(0.0, 1.0));
    let mut curves = vec![a, b];
    if !super::loft_build::make_compatible(&mut curves) {
        // Different knots that won't merge: sample both.
        let n = 32;
        let pa: Vec<P3> = (0..=n).map(|i| c.point(t0 + (t1 - t0) * i as f64 / n as f64)).collect();
        let pb: Vec<P3> = (0..=n).map(|i| o.point(t0 + (t1 - t0) * i as f64 / n as f64)).collect();
        let prm: Vec<f64> = (0..=n).map(|i| i as f64 / n as f64).collect();
        curves = vec![NurbsCurve::interpolate(&pa, &prm, 3, None, None), NurbsCurve::interpolate(&pb, &prm, 3, None, None)];
    }
    Surface::Nurbs(Arc::new(super::loft_build::skin(&curves, &[0.0, 1.0])))
}

/// The slab of face `fi` of `b` swept by `d` along its outward normals.
pub fn face_slab(b: &Body, fi: usize, d: f64) -> Result<Body> {
    let f = &b.faces[fi];
    let top_surf = super::local::offset_surface(&f.surf, f.sense, d).ok_or(unsupported("slabs of this kind of face"))?;
    let mut xb = XBuilder::new();
    let mut vb: HashMap<usize, usize> = HashMap::new();
    let mut vt: HashMap<usize, usize> = HashMap::new();
    let mut rails: HashMap<usize, usize> = HashMap::new();
    let mut eb: HashMap<usize, usize> = HashMap::new();
    let mut et: HashMap<usize, usize> = HashMap::new();
    // How often each edge is used by the face (a seam twice).
    let mut uses: HashMap<usize, usize> = HashMap::new();
    for c in f.loops.iter().flatten() {
        *uses.entry(c.edge).or_default() += 1;
    }
    for c in f.loops.iter().flatten() {
        let e = &b.edges[c.edge];
        if e.degenerate {
            continue;
        }
        for v in e.v {
            if !vb.contains_key(&v) {
                let p = b.verts[v].p;
                let q = p + normal_at(&f.surf, f.sense, &p) * d;
                vb.insert(v, xb.vertex(p));
                vt.insert(v, xb.vertex(q));
            }
        }
    }
    let mut sides: Vec<XFace> = Vec::new();
    for c in f.loops.iter().flatten() {
        let ei = c.edge;
        let e = &b.edges[ei];
        if e.degenerate || eb.contains_key(&ei) {
            continue;
        }
        let (v0, v1) = (e.v[0], e.v[1]);
        let oc = offset_curve(&f.surf, f.sense, &e.curve, e.t[0], e.t[1], d);
        let a = xb.edge(e.curve.clone(), vb[&v0], vb[&v1]).ok_or(unsupported("an empty edge in a slab"))?;
        let t = xb.edge(oc.clone(), vt[&v0], vt[&v1]).ok_or(unsupported("an empty offset edge in a slab"))?;
        eb.insert(ei, a);
        et.insert(ei, t);
        // A seam (used twice) bounds no side.
        if uses[&ei] > 1 {
            continue;
        }
        let surf = side_surface(&e.curve, e.t[0], e.t[1], &oc);
        if v0 == v1 {
            sides.push(XFace { surf, sense: true, loops: vec![vec![(a, true)], vec![(t, false)]] });
            continue;
        }
        let mut rail = |v: usize, xb: &mut XBuilder| -> Result<usize> {
            if let Some(r) = rails.get(&v) {
                return Ok(*r);
            }
            let (p, q) = (xb.body.verts[vb[&v]].p, xb.body.verts[vt[&v]].p);
            let (line, _) = Curve::line(p, q);
            let r = xb.edge(line, vb[&v], vt[&v]).ok_or(unsupported("a slab of no thickness"))?;
            rails.insert(v, r);
            Ok(r)
        };
        let (r0, r1) = (rail(v0, &mut xb)?, rail(v1, &mut xb)?);
        sides.push(XFace { surf, sense: true, loops: vec![vec![(a, true), (r1, true), (t, false), (r0, false)]] });
    }
    let map_loops = |m: &HashMap<usize, usize>| -> Vec<Vec<(usize, bool)>> {
        f.loops.iter().map(|l| l.iter().filter(|c| !b.edges[c.edge].degenerate).map(|c| (m[&c.edge], c.fwd)).collect()).filter(|l: &Vec<(usize, bool)>| !l.is_empty()).collect()
    };
    let mut faces = vec![
        xb.face(XFace { surf: f.surf.clone(), sense: if d > 0.0 { !f.sense } else { f.sense }, loops: map_loops(&eb) }),
        xb.face(XFace { surf: top_surf, sense: if d > 0.0 { f.sense } else { !f.sense }, loops: map_loops(&et) }),
    ];
    for s in sides {
        faces.push(xb.face(s));
    }
    let sh = xb.shell(faces, true);
    xb.solid(vec![sh]);
    let (body, _warnings) = xb.finish();
    let issues = body.check();
    if !issues.is_empty() || body.solids.is_empty() {
        return Err(KernelError::OperationFailed(format!("the face's slab isn't valid: {}", issues.first().map_or("no solid", |s| s.as_str()))));
    }
    if super::mass::body_mass(&body).volume <= 0.0 {
        return Err(KernelError::OperationFailed("the face's slab has no volume".into()));
    }
    Ok(body)
}
