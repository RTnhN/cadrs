//! Booleans (union, subtract, intersect) of B-rep bodies.
//!
//! 1. Both bodies go into one pool of vertices and edges (coincident vertices and edges shared).
//! 2. Vertices lying on edges of the other body, edge–edge and edge–face intersections split
//!    edges; every such point becomes a vertex.
//! 3. Each pair of faces whose boxes overlap is intersected; the intersection curves are cut at
//!    the vertices lying on them and the pieces inside both faces become cut edges of both.
//!    Faces on the same surface imprint each other's boundary edges instead.
//! 4. Every face is split along its cuts in its (u, v) domain.
//! 5. Each piece is classified against the other body (inside, outside, on with the same or the
//!    opposite orientation) and kept or dropped by the operation's rule; Subtract flips the
//!    tool's kept pieces.
//! 6. The kept faces are assembled into shells and solids; a union merges faces of the two
//!    inputs that lie on the same surface and meet along an edge.

use std::collections::HashMap;

use super::classify::{self, InFace};
use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use super::pcurve;
use super::split::{self, Arc};
use super::ssi;
use super::surface::Surface;
use super::topo::{Body, Coedge, Edge, Face, Shell, Solid};
use crate::*;

/// Vertices closer than this are merged (mm).
pub const VTOL: f64 = 1e-6;

#[derive(Debug, Clone)]
struct WEdge {
    curve: Curve,
    t: [f64; 2],
    v: [usize; 2],
    degenerate: bool,
    /// Split points (t, vertex).
    splits: Vec<(f64, usize)>,
}

#[derive(Debug, Clone)]
struct WFace {
    surf: Surface,
    sense: bool,
    loops: Vec<Vec<(usize, bool, Curve)>>,
    owner: usize,
    orig: usize,
    polys: Vec<Vec<P2>>,
    bbox: Bbox,
    uv_center: P2,
}

#[derive(Default)]
struct Pool {
    verts: Vec<P3>,
    grid: HashMap<(i64, i64, i64), Vec<usize>>,
    edges: Vec<WEdge>,
}

impl Pool {
    fn cell(p: &P3) -> (i64, i64, i64) {
        let s = 1.0 / (VTOL * 16.0);
        ((p.x * s).floor() as i64, (p.y * s).floor() as i64, (p.z * s).floor() as i64)
    }
    fn find(&self, p: &P3) -> Option<usize> {
        let (cx, cy, cz) = Self::cell(p);
        let mut best: Option<(f64, usize)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(v) = self.grid.get(&(cx + dx, cy + dy, cz + dz)) {
                        for &i in v {
                            let d = (self.verts[i] - p).norm();
                            if d < VTOL && best.map_or(true, |(bd, _)| d < bd) {
                                best = Some((d, i));
                            }
                        }
                    }
                }
            }
        }
        best.map(|(_, i)| i)
    }
    fn vertex(&mut self, p: P3) -> usize {
        if let Some(i) = self.find(&p) {
            return i;
        }
        self.verts.push(p);
        let i = self.verts.len() - 1;
        self.grid.entry(Self::cell(&p)).or_default().push(i);
        i
    }
    fn new_vertex(&mut self, p: P3) -> usize {
        self.verts.push(p);
        let i = self.verts.len() - 1;
        self.grid.entry(Self::cell(&p)).or_default().push(i);
        i
    }
    fn split(&mut self, e: usize, t: f64, v: usize) {
        let ed = &mut self.edges[e];
        if ed.degenerate {
            return;
        }
        let span = (ed.t[1] - ed.t[0]).abs();
        if t <= ed.t[0] + span * 1e-12 || t >= ed.t[1] - span * 1e-12 {
            return;
        }
        if v == ed.v[0] || v == ed.v[1] {
            // A closed edge can be split at its own vertex only at its ends.
            if !(ed.v[0] == ed.v[1]) {
                return;
            }
            return;
        }
        if ed.splits.iter().any(|(_, sv)| *sv == v) {
            return;
        }
        ed.splits.push((t, v));
    }
}

fn edge_bbox(e: &WEdge, verts: &[P3]) -> Bbox {
    let mut bb = Bbox::empty();
    bb.add(&verts[e.v[0]]);
    bb.add(&verts[e.v[1]]);
    if !e.degenerate {
        let n = match e.curve {
            Curve::Line { .. } => 1,
            _ => 32,
        };
        for i in 0..=n {
            bb.add(&e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / n as f64));
        }
        if let Curve::Circle { r, .. } | Curve::Ellipse { a: r, .. } = e.curve {
            bb = bb.grow(r * 0.02);
        }
    }
    bb.grow(VTOL * 10.0)
}

/// Imports a body's vertices, edges and faces into the pool.
fn import(pool: &mut Pool, faces: &mut Vec<WFace>, b: &Body, owner: usize, merge: bool) {
    let vmap: Vec<usize> = b.verts.iter().map(|v| if merge { pool.vertex(v.p) } else { pool.new_vertex(v.p) }).collect();
    let e0 = pool.edges.len();
    for e in &b.edges {
        pool.edges.push(WEdge { curve: e.curve.clone(), t: e.t, v: [vmap[e.v[0]], vmap[e.v[1]]], degenerate: e.degenerate, splits: Vec::new() });
    }
    for (fi, f) in b.faces.iter().enumerate() {
        let loops: Vec<Vec<(usize, bool, Curve)>> = f.loops.iter().map(|l| l.iter().map(|c| (c.edge + e0, c.fwd, c.pc.clone())).collect()).collect();
        let polys = classify::face_polygons(b, f);
        let mut bb = Bbox::empty();
        let mut uvb = Bbox::empty();
        for p in &polys {
            for q in p {
                uvb.add(&P3::new(q.x, q.y, 0.0));
            }
        }
        for l in &f.loops {
            for c in l {
                let e = &b.edges[c.edge];
                bb.add(&b.verts[e.v[0]].p);
                if !e.degenerate {
                    for i in 0..=16 {
                        bb.add(&e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 16.0));
                    }
                }
            }
        }
        // Curved faces may bulge past their edges: add interior extremes.
        if !matches!(f.surf, Surface::Plane { .. }) {
            let (u0, u1, v0, v1) = (uvb.min.x, uvb.max.x, uvb.min.y, uvb.max.y);
            let n = 8;
            for i in 0..=n {
                for j in 0..=n {
                    let (u, v) = (u0 + (u1 - u0) * i as f64 / n as f64, v0 + (v1 - v0) * j as f64 / n as f64);
                    bb.add(&f.surf.eval(u, v));
                }
            }
            let d = bb.diagonal();
            bb = bb.grow(d * 0.02);
        }
        let center = uvb.center();
        faces.push(WFace {
            surf: f.surf.clone(),
            sense: f.sense,
            loops,
            owner,
            orig: fi,
            polys,
            bbox: bb.grow(VTOL * 10.0),
            uv_center: P2::new(center.x, center.y),
        });
    }
}

fn on_curve(c: &Curve, t0: f64, t1: f64, p: &P3, tol: f64) -> Option<f64> {
    let t = match c {
        Curve::Circle { .. } | Curve::Ellipse { .. } => {
            let t = c.closest(p, t0, t1);
            // The closest point for a periodic curve: retry unwrapped.
            t
        }
        _ => c.closest(p, t0, t1),
    };
    if (c.point(t) - p).norm() < tol { Some(t) } else { None }
}

/// The parameter of a point on a closed curve, unwrapped into [t0, t1].
fn param_on(c: &Curve, t0: f64, t1: f64, p: &P3) -> f64 {
    c.closest(p, t0, t1)
}

/// Whether an edge lies on a surface (its sample points are on it).
fn edge_on_surface(e: &WEdge, s: &Surface) -> bool {
    if e.degenerate {
        return true;
    }
    [0.0, 0.3, 0.5, 0.7, 1.0].iter().all(|f| {
        let p = e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * f);
        s.distance(&p) < VTOL
    })
}

/// In-face test of a model point (projected onto the face's surface).
fn point_in_face(f: &WFace, p: &P3, tol: f64) -> InFace {
    let (u, v) = f.surf.project(p, Some((f.uv_center.x, f.uv_center.y)));
    if (f.surf.eval(u, v) - p).norm() > tol * 10.0 + 1e-9 {
        return InFace::Outside;
    }
    let mut res = InFace::Outside;
    for uv in classify::periodic_candidates(&f.surf, u, v, &f.polys) {
        match classify::in_face(&f.polys, &f.surf, uv, tol) {
            InFace::Inside => return InFace::Inside,
            InFace::Boundary => res = InFace::Boundary,
            InFace::Outside => {}
        }
    }
    res
}

/// A point inside a face's domain (uv) given its loops as arcs.
fn interior_point(loops: &[Vec<Arc>]) -> Option<P2> {
    let mut polys: Vec<Vec<P2>> = Vec::new();
    for l in loops {
        let mut pts = Vec::new();
        for a in l {
            let n = match a.pc {
                Curve::Line { .. } => 1,
                _ => 64,
            };
            for i in 0..n {
                pts.push(a.uv_at(i as f64 / n as f64));
            }
        }
        polys.push(pts);
    }
    interior_of_polys(&polys)
}

pub fn interior_of_polys(polys: &[Vec<P2>]) -> Option<P2> {
    let mut vb = (f64::INFINITY, f64::NEG_INFINITY);
    for p in polys {
        for q in p {
            vb = (vb.0.min(q.y), vb.1.max(q.y));
        }
    }
    if !vb.0.is_finite() || vb.1 - vb.0 < 1e-14 {
        return None;
    }
    let mut best: Option<(f64, P2)> = None;
    for frac in [0.5, 0.3137, 0.6863, 0.1571, 0.8429, 0.0713, 0.9287] {
        let y = vb.0 + (vb.1 - vb.0) * frac;
        let mut xs: Vec<f64> = Vec::new();
        for p in polys {
            let n = p.len();
            for i in 0..n {
                let (a, b) = (p[i], p[(i + 1) % n]);
                if (a.y > y) != (b.y > y) {
                    xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
                }
            }
        }
        xs.sort_by(f64::total_cmp);
        let mut k = 0;
        while k + 1 < xs.len() {
            let w = xs[k + 1] - xs[k];
            if best.map_or(true, |(bw, _)| w > bw) {
                best = Some((w, P2::new((xs[k] + xs[k + 1]) * 0.5, y)));
            }
            k += 2;
        }
    }
    best.map(|(_, p)| p)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    In,
    Out,
    OnSame,
    OnOpp,
}

pub struct BoolOut {
    pub body: Body,
    /// For each output face: (owner 0 = target / 1 = tool, its face in that input).
    pub from: Vec<Vec<(usize, usize)>>,
}

/// The boolean of two bodies.
pub fn boolean_bodies(a: &Body, b: &Body, op: BoolOp) -> Result<BoolOut> {
    // Disjoint boxes: nothing to intersect.
    let (ba, bb) = (a.bbox(), b.bbox());
    if !ba.grow(VTOL).overlaps(&bb.grow(VTOL)) {
        return match op {
            BoolOp::Union => {
                let mut out = a.clone();
                let n = a.faces.len();
                out.append(b);
                let mut from: Vec<Vec<(usize, usize)>> = (0..n).map(|i| vec![(0, i)]).collect();
                from.extend((0..b.faces.len()).map(|i| vec![(1, i)]));
                Ok(BoolOut { body: out, from })
            }
            BoolOp::Subtract => Ok(BoolOut { body: a.clone(), from: (0..a.faces.len()).map(|i| vec![(0, i)]).collect() }),
            BoolOp::Intersect => Err(KernelError::OperationFailed("the result is empty".into())),
        };
    }
    let mut pool = Pool::default();
    let mut faces: Vec<WFace> = Vec::new();
    import(&mut pool, &mut faces, a, 0, false);
    let na_edges = pool.edges.len();
    let na_faces = faces.len();
    import(&mut pool, &mut faces, b, 1, true);
    let ne = pool.edges.len();
    let owner_of_edge = |e: usize| if e < na_edges { 0 } else { 1 };
    let ebox: Vec<Bbox> = pool.edges.iter().map(|e| edge_bbox(e, &pool.verts)).collect();
    // Edges of each face, and faces of each edge.
    let mut face_edges: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    for (fi, f) in faces.iter().enumerate() {
        for l in &f.loops {
            for (e, _, _) in l {
                if !face_edges[fi].contains(e) {
                    face_edges[fi].push(*e);
                }
            }
        }
    }
    // 2a. Vertices on edges of the other body.
    for e in 0..ne {
        if pool.edges[e].degenerate {
            continue;
        }
        let other_verts: Vec<usize> = {
            // Vertices of the other body's edges.
            let mut vs = Vec::new();
            let range = if owner_of_edge(e) == 0 { na_edges..ne } else { 0..na_edges };
            for o in range {
                for v in pool.edges[o].v {
                    if !vs.contains(&v) {
                        vs.push(v);
                    }
                }
            }
            vs
        };
        for v in other_verts {
            let p = pool.verts[v];
            if !ebox[e].contains(&p) {
                continue;
            }
            let ed = &pool.edges[e];
            if v == ed.v[0] || v == ed.v[1] {
                continue;
            }
            if let Some(t) = on_curve(&ed.curve, ed.t[0], ed.t[1], &p, VTOL) {
                pool.split(e, t, v);
            }
        }
    }
    // 2b. Edge–edge intersections (A × B).
    for ea in 0..na_edges {
        if pool.edges[ea].degenerate {
            continue;
        }
        for eb in na_edges..ne {
            if pool.edges[eb].degenerate || !ebox[ea].overlaps(&ebox[eb]) {
                continue;
            }
            let (ca, cb) = (pool.edges[ea].clone(), pool.edges[eb].clone());
            for (ta, tb) in ssi::curve_curve(&ca.curve, ca.t[0], ca.t[1], &cb.curve, cb.t[0], cb.t[1], VTOL * 0.5) {
                let p = ca.curve.point(ta);
                let v = pool.vertex(p);
                pool.split(ea, ta, v);
                pool.split(eb, tb, v);
            }
        }
    }
    // 2c. Edge–face intersections.
    let mut face_pts: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    for e in 0..ne {
        if pool.edges[e].degenerate {
            continue;
        }
        let oe = owner_of_edge(e);
        for fi in 0..faces.len() {
            if faces[fi].owner == oe || !ebox[e].overlaps(&faces[fi].bbox) {
                continue;
            }
            let ed = pool.edges[e].clone();
            if edge_on_surface(&ed, &faces[fi].surf) {
                continue;
            }
            for t in ssi::curve_surface(&ed.curve, ed.t[0], ed.t[1], &faces[fi].surf) {
                let p = ed.curve.point(t);
                match point_in_face(&faces[fi], &p, VTOL) {
                    InFace::Outside => continue,
                    _ => {}
                }
                let v = pool.vertex(p);
                pool.split(e, t, v);
                if !face_pts[fi].contains(&v) {
                    face_pts[fi].push(v);
                }
            }
        }
    }
    // Vertices on each face's boundary (with the split points).
    let boundary_verts = |pool: &Pool, fi: usize| -> Vec<usize> {
        let mut vs = Vec::new();
        for &e in &face_edges[fi] {
            let ed = &pool.edges[e];
            for v in ed.v.iter().copied().chain(ed.splits.iter().map(|s| s.1)) {
                if !vs.contains(&v) {
                    vs.push(v);
                }
            }
        }
        vs
    };
    // 3. Face–face.
    let mut cuts: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    let mut coincident: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    let mut new_edges: Vec<usize> = Vec::new();
    for fa in 0..na_faces {
        for fb in na_faces..faces.len() {
            if !faces[fa].bbox.overlaps(&faces[fb].bbox) {
                continue;
            }
            if faces[fa].surf.same_domain(&faces[fb].surf, VTOL).is_some() {
                coincident[fa].push(fb);
                coincident[fb].push(fa);
                continue;
            }
            let mut ob = faces[fa].bbox;
            ob.min = ob.min.sup(&faces[fb].bbox.min);
            ob.max = ob.max.inf(&faces[fb].bbox.max);
            let uvbox = |f: &WFace| {
                let mut ub = (f64::INFINITY, f64::NEG_INFINITY);
                let mut vb = (f64::INFINITY, f64::NEG_INFINITY);
                for p in &f.polys {
                    for q in p {
                        ub = (ub.0.min(q.x), ub.1.max(q.x));
                        vb = (vb.0.min(q.y), vb.1.max(q.y));
                    }
                }
                (ub, vb)
            };
            let curves = ssi::surface_surface(&faces[fa].surf, uvbox(&faces[fa]), &faces[fb].surf, uvbox(&faces[fb]), &ob);
            if curves.is_empty() {
                continue;
            }
            let mut cand: Vec<usize> = boundary_verts(&pool, fa);
            for v in boundary_verts(&pool, fb).into_iter().chain(face_pts[fa].iter().copied()).chain(face_pts[fb].iter().copied()) {
                if !cand.contains(&v) {
                    cand.push(v);
                }
            }
            for ic in curves {
                let mut ps: Vec<(f64, usize)> = Vec::new();
                for &v in &cand {
                    let p = pool.verts[v];
                    if !ob.grow(VTOL * 10.0).contains(&p) {
                        continue;
                    }
                    let t = param_on(&ic.c, ic.t0, ic.t1, &p);
                    if (ic.c.point(t) - p).norm() < VTOL * 2.0 {
                        ps.push((t, v));
                        // A closed curve: the vertex at its start is also at its end.
                        if ic.closed && ((t - ic.t0).abs() < 1e-12 || (t - ic.t1).abs() < 1e-12) {
                            ps.push((if (t - ic.t0).abs() < 1e-12 { ic.t1 } else { ic.t0 }, v));
                        }
                    }
                }
                ps.sort_by(|a, b| a.0.total_cmp(&b.0));
                ps.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-12 && a.1 == b.1);
                // Intervals.
                let mut ivs: Vec<(f64, f64, Option<usize>, Option<usize>)> = Vec::new();
                if ic.closed {
                    if ps.is_empty() {
                        ivs.push((ic.t0, ic.t1, None, None));
                    } else {
                        for k in 0..ps.len() {
                            let (ta, va) = ps[k];
                            let (tb, vb) = if k + 1 < ps.len() { ps[k + 1] } else { (ps[0].0 + (ic.t1 - ic.t0), ps[0].1) };
                            if tb - ta > 1e-10 {
                                ivs.push((ta, tb, Some(va), Some(vb)));
                            }
                        }
                    }
                } else {
                    for w in ps.windows(2) {
                        if w[1].0 - w[0].0 > 1e-10 {
                            ivs.push((w[0].0, w[1].0, Some(w[0].1), Some(w[1].1)));
                        }
                    }
                }
                for (ta, tb, va, vb) in ivs {
                    // Wrap periodic parameters past the end.
                    let tm = (ta + tb) * 0.5;
                    let pm = ic.c.point(tm);
                    let ia = point_in_face(&faces[fa], &pm, VTOL);
                    let ib = point_in_face(&faces[fb], &pm, VTOL);
                    if ia == InFace::Outside || ib == InFace::Outside {
                        continue;
                    }
                    let va = va.unwrap_or_else(|| pool.vertex(ic.c.point(ta)));
                    let vb = vb.unwrap_or(va);
                    // Reuse an existing edge running there.
                    let existing = (0..pool.edges.len()).find(|&e| {
                        let ed = &pool.edges[e];
                        if ed.degenerate || !ebox.get(e).map_or(true, |b| b.contains(&pm)) {
                            return false;
                        }
                        let ends_ok = (ed.v == [va, vb] || ed.v == [vb, va]) && ed.splits.is_empty() || (ed.splits.iter().any(|s| s.1 == va) || ed.v.contains(&va)) && (ed.splits.iter().any(|s| s.1 == vb) || ed.v.contains(&vb));
                        ends_ok && on_curve(&ed.curve, ed.t[0], ed.t[1], &pm, VTOL).is_some()
                    });
                    let e = match existing {
                        Some(e) => e,
                        None => {
                            // Unwrap the parameter range for the curve (closed curves may run past
                            // t1).
                            let (ta, tb) = (ta, tb);
                            pool.edges.push(WEdge { curve: ic.c.clone(), t: [ta, tb], v: [va, vb], degenerate: false, splits: Vec::new() });
                            new_edges.push(pool.edges.len() - 1);
                            pool.edges.len() - 1
                        }
                    };
                    for f in [fa, fb] {
                        if !face_edges[f].contains(&e) && !cuts[f].contains(&e) {
                            cuts[f].push(e);
                        }
                    }
                }
            }
        }
    }
    // Pieces of the split edges.
    let mut pieces: Vec<Vec<usize>> = Vec::with_capacity(pool.edges.len());
    let n_all = pool.edges.len();
    for e in 0..n_all {
        let mut sp = pool.edges[e].splits.clone();
        if sp.is_empty() {
            pieces.push(vec![e]);
            continue;
        }
        sp.sort_by(|a, b| a.0.total_cmp(&b.0));
        let ed = pool.edges[e].clone();
        let mut bounds = vec![(ed.t[0], ed.v[0])];
        bounds.extend(sp);
        bounds.push((ed.t[1], ed.v[1]));
        let mut ps = Vec::new();
        for w in bounds.windows(2) {
            pool.edges.push(WEdge { curve: ed.curve.clone(), t: [w[0].0, w[1].0], v: [w[0].1, w[1].1], degenerate: false, splits: Vec::new() });
            ps.push(pool.edges.len() - 1);
        }
        pieces.push(ps);
    }
    // Coincident pieces (the same curve between the same vertices) become one edge.
    let np = pool.edges.len();
    let mut canon: Vec<usize> = (0..np).collect();
    {
        let mut groups: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        for ps in &pieces {
            for &p in ps {
                let ed = &pool.edges[p];
                if ed.degenerate {
                    continue;
                }
                let key = (ed.v[0].min(ed.v[1]), ed.v[0].max(ed.v[1]));
                groups.entry(key).or_default().push(p);
            }
        }
        for (_, g) in groups {
            for i in 0..g.len() {
                if canon[g[i]] != g[i] {
                    continue;
                }
                for j in i + 1..g.len() {
                    if canon[g[j]] != g[j] {
                        continue;
                    }
                    let (p, q) = (&pool.edges[g[i]], &pool.edges[g[j]]);
                    let ok = [0.25, 0.5, 0.75].iter().all(|f| {
                        let m = q.curve.point(q.t[0] + (q.t[1] - q.t[0]) * f);
                        on_curve(&p.curve, p.t[0], p.t[1], &m, VTOL * 10.0).is_some()
                    });
                    if ok {
                        canon[g[j]] = g[i];
                    }
                }
            }
        }
    }
    // Coincident faces imprint each other's boundary pieces.
    if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
        for (i, c) in coincident.iter().enumerate() {
            if !c.is_empty() {
                eprintln!("coincident {i}: {c:?}");
            }
        }
    }
    for fa in 0..faces.len() {
        for &fb in &coincident[fa].clone() {
            for &e in &face_edges[fb].clone() {
                for &pe in &pieces[e] {
                    let ed = &pool.edges[pe];
                    if ed.degenerate {
                        continue;
                    }
                    let pm = ed.curve.point((ed.t[0] + ed.t[1]) * 0.5);
                    if point_in_face(&faces[fa], &pm, VTOL) == InFace::Inside && !cuts[fa].contains(&pe) {
                        cuts[fa].push(pe);
                    }
                }
            }
        }
    }
    // 4. Split faces.
    // An arc on its canonical edge (re-parametrized onto that edge's curve if aliased).
    let canon_arc = |pool: &Pool, f: &WFace, a: Arc| -> Arc {
        let c = canon[a.edge];
        if c == a.edge {
            return a;
        }
        let ce = &pool.edges[c];
        let near = a.uv_at(0.5);
        let pc = pcurve::pcurve(&f.surf, &ce.curve, ce.t[0], ce.t[1], Some(near));
        // Same direction when the canonical edge runs from the arc's start vertex (or, closed,
        // when the tangents agree).
        let ae = &pool.edges[a.edge];
        let same = if ce.v[0] != ce.v[1] {
            ce.v[0] == ae.v[0]
        } else {
            let tm = (ae.t[0] + ae.t[1]) * 0.5;
            let p = ae.curve.point(tm);
            let tc = ce.curve.closest(&p, ce.t[0], ce.t[1]);
            ce.curve.tangent(tc).dot(&ae.curve.tangent(tm)) > 0.0
        };
        let fwd = if same { a.fwd } else { !a.fwd };
        let v = if fwd { [ce.v[0], ce.v[1]] } else { [ce.v[1], ce.v[0]] };
        Arc { edge: c, fwd, pc, t: ce.t, v, cut: a.cut }
    };
    let arcs_of = |pool: &Pool, f: &WFace, e: usize, fwd: bool, pc: &Curve| -> Vec<Arc> {
        let ps: Vec<usize> = if e < pieces.len() { pieces[e].clone() } else { vec![e] };
        let mut out: Vec<Arc> = ps
            .iter()
            .map(|&p| {
                let ed = &pool.edges[p];
                let v = if fwd { [ed.v[0], ed.v[1]] } else { [ed.v[1], ed.v[0]] };
                canon_arc(pool, f, Arc { edge: p, fwd, pc: pc.clone(), t: ed.t, v, cut: false })
            })
            .collect();
        if !fwd {
            out.reverse();
        }
        out
    };
    let mut subs: Vec<(usize, Vec<Vec<Arc>>)> = Vec::new(); // (face, loops)
    for (fi, f) in faces.iter().enumerate() {
        let boundary: Vec<Vec<Arc>> = f.loops.iter().map(|l| l.iter().flat_map(|(e, fwd, pc)| arcs_of(&pool, f, *e, *fwd, pc)).collect()).collect();
        if cuts[fi].is_empty() {
            subs.push((fi, boundary));
            continue;
        }
        let own: Vec<usize> = boundary.iter().flatten().map(|a| a.edge).collect();
        let mut cut_arcs: Vec<Arc> = Vec::new();
        for &ce in &cuts[fi] {
            let cps: Vec<usize> = if ce < pieces.len() { pieces[ce].clone() } else { vec![ce] };
            for p in cps {
                let c = canon[p];
                if own.contains(&c) || cut_arcs.iter().any(|a| a.edge == c) {
                    continue;
                }
                let ed = &pool.edges[c];
                let pc = pcurve::pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], Some(f.uv_center));
                cut_arcs.push(Arc { edge: c, fwd: true, pc, t: ed.t, v: ed.v, cut: true });
            }
        }
        if cut_arcs.is_empty() {
            subs.push((fi, boundary));
            continue;
        }
        let res = split::split_face(&f.surf, !f.sense, &boundary, &cut_arcs);
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            eprintln!("face {fi} (owner {} orig {}): {} cuts -> {} faces {:?}", f.owner, f.orig, cut_arcs.len(), res.faces.len(), res.faces.iter().map(|l| l.len()).collect::<Vec<_>>());
        }
        for lps in res.faces {
            subs.push((fi, lps));
        }
    }
    // 5. Classify.
    let polys_a: Vec<Vec<Vec<P2>>> = a.faces.iter().map(|f| classify::face_polygons(a, f)).collect();
    let polys_b: Vec<Vec<Vec<P2>>> = b.faces.iter().map(|f| classify::face_polygons(b, f)).collect();
    let mut keep: Vec<(usize, Vec<Vec<Arc>>, bool)> = Vec::new(); // (face, loops, flip)
    for (fi, loops) in subs {
        let f = &faces[fi];
        let Some(uv) = interior_point(&loops) else { continue };
        let p = f.surf.eval(uv.x, uv.y);
        let n_out = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
        let mut class = None;
        for &g in &coincident[fi] {
            let gf = &faces[g];
            if point_in_face(gf, &p, VTOL * 10.0) != InFace::Outside {
                let (gu, gv) = gf.surf.project(&p, Some((gf.uv_center.x, gf.uv_center.y)));
                let gn = gf.surf.normal(gu, gv) * if gf.sense { 1.0 } else { -1.0 };
                class = Some(if gn.dot(&n_out) > 0.0 { Class::OnSame } else { Class::OnOpp });
                break;
            }
        }
        let class = class.unwrap_or_else(|| {
            let (other, polys) = if f.owner == 0 { (b, &polys_b) } else { (a, &polys_a) };
            match classify::classify_point(other, polys, &p, VTOL) {
                PointClass::Inside => Class::In,
                PointClass::Outside => Class::Out,
                PointClass::OnBoundary => {
                    // On the other body's boundary but not on a coincident face: decide by a
                    // point nudged inwards along the face.
                    let q = p - n_out * 1e-5;
                    if classify::parity_inside(other, polys, &q) { Class::In } else { Class::Out }
                }
            }
        });
        let a_side = f.owner == 0;
        let (k, flip) = match (op, a_side, class) {
            (BoolOp::Union, _, Class::Out) => (true, false),
            (BoolOp::Union, true, Class::OnSame) => (true, false),
            (BoolOp::Intersect, _, Class::In) => (true, false),
            (BoolOp::Intersect, true, Class::OnSame) => (true, false),
            (BoolOp::Subtract, true, Class::Out) => (true, false),
            (BoolOp::Subtract, true, Class::OnOpp) => (true, false),
            (BoolOp::Subtract, false, Class::In) => (true, true),
            _ => (false, false),
        };
        if k {
            keep.push((fi, loops, flip));
        }
    }
    if keep.is_empty() {
        return Err(KernelError::OperationFailed("the result is empty".into()));
    }
    // 6. Assemble.
    let mut out = Body::default();
    let mut vmap: HashMap<usize, usize> = HashMap::new();
    let mut emap: HashMap<usize, usize> = HashMap::new();
    let mut from = Vec::new();
    for (fi, loops, flip) in keep {
        let f = &faces[fi];
        let mut nloops: Vec<Vec<Coedge>> = Vec::new();
        for l in loops {
            let mut nl = Vec::new();
            for a in l {
                let ne = *emap.entry(a.edge).or_insert_with(|| {
                    let ed = &pool.edges[a.edge];
                    let mut vv = [0usize; 2];
                    for k in 0..2 {
                        vv[k] = *vmap.entry(ed.v[k]).or_insert_with(|| {
                            out.verts.push(super::topo::Vertex { p: pool.verts[ed.v[k]] });
                            out.verts.len() - 1
                        });
                    }
                    out.edges.push(Edge { curve: ed.curve.clone(), t: ed.t, v: vv, degenerate: ed.degenerate });
                    out.edges.len() - 1
                });
                nl.push(Coedge { edge: ne, fwd: a.fwd, pc: a.pc.clone() });
            }
            nloops.push(nl);
        }
        let mut face = Face { surf: f.surf.clone(), sense: f.sense, loops: nloops };
        if flip {
            Body::reverse_face(&mut face);
        }
        out.faces.push(face);
        from.push((f.owner, f.orig));
    }
    if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
        for (i, f) in out.faces.iter().enumerate() {
            eprintln!("out face {i} {:?} from {:?} loops {:?}", f.surf.kind(), from[i], f.loops.iter().map(|l| l.iter().map(|c| (c.edge, c.fwd)).collect::<Vec<_>>()).collect::<Vec<_>>());
        }
        eprintln!("check: {:?}", out.check());
    }
    let mut from: Vec<Vec<(usize, usize)>> = from.into_iter().map(|x| vec![x]).collect();
    if op == BoolOp::Union {
        let owners: Vec<usize> = from.iter().map(|f| f[0].0).collect();
        from = super::unify::unify_faces(&mut out, from, &|f, g| owners[f] != owners[g]);
    }
    super::unify::merge_edges(&mut out);
    assemble_shells(&mut out)?;
    Ok(BoolOut { body: out, from })
}

/// Groups faces into shells by shared edges and shells into solids (outer shells and voids).
pub fn assemble_shells(b: &mut Body) -> Result<()> {
    let ef = b.edge_faces();
    let nf = b.faces.len();
    let mut comp = vec![usize::MAX; nf];
    let mut shells: Vec<Vec<usize>> = Vec::new();
    for s in 0..nf {
        if comp[s] != usize::MAX {
            continue;
        }
        let id = shells.len();
        let mut stack = vec![s];
        comp[s] = id;
        let mut fs = Vec::new();
        while let Some(f) = stack.pop() {
            fs.push(f);
            for l in &b.faces[f].loops {
                for c in l {
                    for &g in &ef[c.edge] {
                        if comp[g] == usize::MAX {
                            comp[g] = id;
                            stack.push(g);
                        }
                    }
                }
            }
        }
        fs.sort();
        shells.push(fs);
    }
    // Closed shells: every non-degenerate edge used twice.
    b.shells.clear();
    b.solids.clear();
    let mut outer: Vec<(usize, f64)> = Vec::new();
    let mut voids: Vec<(usize, f64)> = Vec::new();
    for fs in shells {
        let mut closed = true;
        let mut uses: HashMap<usize, i32> = HashMap::new();
        for &f in &fs {
            for l in &b.faces[f].loops {
                for c in l {
                    *uses.entry(c.edge).or_default() += if c.fwd { 1 } else { -1 };
                }
            }
        }
        for (e, u) in &uses {
            if *u != 0 && !b.edges[*e].degenerate {
                closed = false;
                if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
                    eprintln!("open edge {e}: uses {u}, {:?}", b.edges[*e]);
                }
            }
        }
        let si = b.shells.len();
        b.shells.push(Shell { faces: fs.clone(), closed });
        if closed {
            let shift = b.bbox().center().coords;
            let vol: f64 = fs.iter().map(|&f| super::mass::face_integrals(b, &b.faces[f], &shift)[4]).sum();
            if vol >= 0.0 {
                outer.push((si, vol));
            } else {
                voids.push((si, vol));
            }
        }
    }
    let open = b.shells.iter().any(|s| !s.closed);
    if open && !outer.is_empty() {
        return Err(KernelError::OperationFailed("the boolean left an open shell".into()));
    }
    let mut solids: Vec<Vec<usize>> = outer.iter().map(|(s, _)| vec![*s]).collect();
    for (v, _) in voids {
        // The smallest outer shell containing a point of the void.
        let f = b.shells[v].faces[0];
        let p = {
            let fc = &b.faces[f];
            let polys = classify::face_polygons(b, fc);
            let uv = interior_of_polys(&polys).unwrap_or(P2::origin());
            fc.surf.eval(uv.x, uv.y)
        };
        let mut best: Option<(f64, usize)> = None;
        for (k, (s, vol)) in outer.iter().enumerate() {
            let sub = super::kernel::subset(b, &b.shells[*s].faces, true);
            let polys: Vec<Vec<Vec<P2>>> = sub.faces.iter().map(|f| classify::face_polygons(&sub, f)).collect();
            if classify::parity_inside(&sub, &polys, &p) && best.map_or(true, |(bv, _)| *vol < bv) {
                best = Some((*vol, k));
            }
        }
        if let Some((_, k)) = best {
            solids[k].push(v);
        }
    }
    for s in solids {
        b.solids.push(Solid { shells: s });
    }
    Ok(())
}

/// The kernel operation: the target combined with each tool in turn.
pub fn boolean(k: &mut NativeKernel, op: BoolOp, target: BodyId, tools: &[BodyId]) -> Result<OpResult> {
    let mut cur = (**k.get(target)?).clone();
    // For each face of `cur`: where it came from (body, face).
    let mut origin: Vec<Vec<(BodyId, usize)>> = (0..cur.faces.len()).map(|i| vec![(target, i)]).collect();
    for &t in tools {
        let tb = (**k.get(t)?).clone();
        let r = boolean_bodies(&cur, &tb, op)?;
        origin = r
            .from
            .iter()
            .map(|srcs| srcs.iter().flat_map(|&(o, f)| if o == 0 { origin[f].clone() } else { vec![(t, f)] }).collect())
            .collect();
        cur = r.body;
    }
    let mut history = History::default();
    for (i, srcs) in origin.iter().enumerate() {
        for (bd, f) in srcs {
            history.modified.push((FaceId(i as u64), InputFace { body: *bd, face: FaceId(*f as u64) }));
        }
    }
    let mut inputs = vec![target];
    inputs.extend_from_slice(tools);
    for &ib in &inputs {
        let n = k.get(ib)?.faces.len();
        for f in 0..n {
            if !origin.iter().flatten().any(|(b, g)| *b == ib && *g == f) {
                history.deleted.push(InputFace { body: ib, face: FaceId(f as u64) });
            }
        }
    }
    let id = k.insert(cur);
    Ok(OpResult { bodies: vec![id], history })
}
