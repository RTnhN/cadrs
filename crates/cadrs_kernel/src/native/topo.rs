//! Boundary representation: vertices, edges, faces with loops of coedges, shells and solids.
//!
//! Conventions (as OCCT's):
//! - An edge runs along its curve for increasing t from `v[0]` (at `t[0]`) to `v[1]`; a closed
//!   edge (a whole circle) has `v[0] == v[1]`.
//! - A coedge uses an edge forward or reversed and carries the edge's p-curve on its face,
//!   parametrized by the edge's t.
//! - Loops run counter-clockwise about the face's outward normal (material on the left seen
//!   from outside); the first loop is the outer one.
//! - `sense`: the outward normal is the surface's natural normal Su × Sv when true.
//! - A seam (on a periodic surface) is one edge used twice by the same face, once each way, with
//!   two p-curves a period apart.
//!
//! Body kinds (Onshape's set, [`BodyClass`]): a body holds solids (closed shells), sheets (open
//! shells), wires (chains of free edges: edges no face uses, listed in [`Body::wires`]) and
//! acorn points (free vertices, listed in [`Body::points`]). Wires and points sit beside the
//! face lists, so code that walks `faces` and `edges` by index is unchanged: a wire body is a
//! body with edges and no faces, a point body one with vertices only.

use super::curve::Curve;
use super::math::*;
use super::surface::Surface;

/// The smallest tolerance an entity has (OCCT's `Precision::Confusion`): exact geometry
/// ([`super::tol::CONFUSION`]).
pub const TOL_MIN: f64 = super::tol::CONFUSION;

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub p: P3,
    /// The ball about `p` the vertex stands for: ends of its edges and its position on faces
    /// lie within it ([`TOL_MIN`] for exact geometry; larger for read or healed shapes).
    pub tol: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub curve: Curve,
    pub t: [f64; 2],
    pub v: [usize; 2],
    /// A point edge (a cone's apex, a sphere's pole): only its p-curves mean anything.
    pub degenerate: bool,
    /// How far the edge's 3D curve, its p-curves on its faces and the faces may stray from one
    /// another along it ([`TOL_MIN`] for exact geometry).
    pub tol: f64,
}

impl Edge {
    pub fn point(&self, t: f64) -> P3 {
        self.curve.point(t)
    }
    pub fn mid_t(&self) -> f64 {
        (self.t[0] + self.t[1]) * 0.5
    }
    pub fn closed(&self) -> bool {
        self.v[0] == self.v[1] && !self.degenerate
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coedge {
    pub edge: usize,
    pub fwd: bool,
    /// The p-curve (z = 0) in the face's (u, v), parametrized by the edge's t.
    pub pc: Curve,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub surf: Surface,
    pub sense: bool,
    pub loops: Vec<Vec<Coedge>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Shell {
    pub faces: Vec<usize>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Solid {
    /// Outer shell first, then voids.
    pub shells: Vec<usize>,
}

/// A chain of free edges (used by no face), each with the direction the chain runs along it:
/// edge k ends where edge k + 1 starts. It is closed when the last edge ends where the first
/// starts.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Wire {
    pub edges: Vec<(usize, bool)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Body {
    pub verts: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
    pub shells: Vec<Shell>,
    pub solids: Vec<Solid>,
    /// Wire bodies' edge chains (Phase 1b). Edges of faces are never in a wire.
    pub wires: Vec<Wire>,
    /// Acorn points: vertices no edge uses (a point body's points).
    pub points: Vec<usize>,
}

pub use crate::BodyClass;

/// A coedge's start and end parameters in its direction of travel.
pub fn co_range(e: &Edge, fwd: bool) -> (f64, f64) {
    if fwd { (e.t[0], e.t[1]) } else { (e.t[1], e.t[0]) }
}

impl Body {
    pub fn add_vertex(&mut self, p: P3) -> usize {
        self.verts.push(Vertex { p, tol: TOL_MIN });
        self.verts.len() - 1
    }

    pub fn add_edge(&mut self, curve: Curve, t0: f64, t1: f64, v0: usize, v1: usize) -> usize {
        self.edges.push(Edge { curve, t: [t0, t1], v: [v0, v1], degenerate: false, tol: TOL_MIN });
        self.edges.len() - 1
    }

    pub fn add_degenerate(&mut self, at: P3, t0: f64, t1: f64, v: usize) -> usize {
        self.edges.push(Edge { curve: Curve::Line { o: at, d: V3::zeros() }, t: [t0, t1], v: [v, v], degenerate: true, tol: TOL_MIN });
        self.edges.len() - 1
    }

    /// The start vertex of a coedge.
    pub fn co_start(&self, c: &Coedge) -> usize {
        let e = &self.edges[c.edge];
        if c.fwd { e.v[0] } else { e.v[1] }
    }

    pub fn co_end(&self, c: &Coedge) -> usize {
        let e = &self.edges[c.edge];
        if c.fwd { e.v[1] } else { e.v[0] }
    }

    /// For each edge, the faces using it (a seam lists its face twice).
    pub fn edge_faces(&self) -> Vec<Vec<usize>> {
        let mut out = vec![Vec::new(); self.edges.len()];
        for (fi, f) in self.faces.iter().enumerate() {
            for l in &f.loops {
                for c in l {
                    out[c.edge].push(fi);
                }
            }
        }
        out
    }

    /// Faces of each solid (and the free faces of sheets as a last group).
    pub fn solid_faces(&self, s: usize) -> Vec<usize> {
        self.solids[s].shells.iter().flat_map(|&sh| self.shells[sh].faces.iter().copied()).collect()
    }

    /// Appends another body (indices shifted). Returns the offsets (verts, edges, faces).
    pub fn append(&mut self, o: &Body) -> (usize, usize, usize) {
        let (vo, eo, fo, so) = (self.verts.len(), self.edges.len(), self.faces.len(), self.shells.len());
        self.verts.extend(o.verts.iter().cloned());
        for e in &o.edges {
            let mut e = e.clone();
            e.v = [e.v[0] + vo, e.v[1] + vo];
            self.edges.push(e);
        }
        for f in &o.faces {
            let mut f = f.clone();
            for l in &mut f.loops {
                for c in l {
                    c.edge += eo;
                }
            }
            self.faces.push(f);
        }
        for s in &o.shells {
            self.shells.push(Shell { faces: s.faces.iter().map(|x| x + fo).collect(), closed: s.closed });
        }
        for s in &o.solids {
            self.solids.push(Solid { shells: s.shells.iter().map(|x| x + so).collect() });
        }
        for w in &o.wires {
            self.wires.push(Wire { edges: w.edges.iter().map(|&(e, f)| (e + eo, f)).collect() });
        }
        self.points.extend(o.points.iter().map(|v| v + vo));
        (vo, eo, fo)
    }

    /// The body moved by an affine map (a reflection flips every face's sense and loops).
    pub fn transformed(&self, m: &Affine) -> Body {
        let mut b = self.clone();
        for v in &mut b.verts {
            v.p = m.point(&v.p);
        }
        for e in &mut b.edges {
            e.curve = e.curve.transformed(m);
        }
        let mirror = m.det() < 0.0;
        let k = m.scale();
        for f in &mut b.faces {
            // Lengths in the surface's parameters scale with the body.
            if (k - 1.0).abs() > 1e-15 {
                let uv = match f.surf {
                    Surface::Plane { .. } => Some(V3::new(k, k, 1.0)),
                    Surface::Cylinder { .. } | Surface::Cone { .. } => Some(V3::new(1.0, k, 1.0)),
                    _ => None,
                };
                if let Some(d) = uv {
                    let a = Affine { m: nalgebra::Matrix3::from_diagonal(&d), t: V3::zeros() };
                    for l in &mut f.loops {
                        for c in l.iter_mut() {
                            c.pc = c.pc.transformed(&a);
                        }
                    }
                }
            }
            f.surf = f.surf.transformed(m);
            if mirror {
                f.sense = !f.sense;
                for l in &mut f.loops {
                    l.reverse();
                    for c in l.iter_mut() {
                        c.fwd = !c.fwd;
                    }
                }
            }
        }
        b
    }

    /// A box sure to hold the body: its vertices, its edges' exact boxes and its faces' boxes
    /// (curved faces bulge past their edges).
    pub fn bbox(&self) -> Bbox {
        let mut bb = Bbox::empty();
        for v in &self.verts {
            bb.add(&v.p);
        }
        for e in &self.edges {
            if !e.degenerate {
                bb.union(&super::boolean::curve_bbox(&e.curve, e.t[0], e.t[1]));
            }
        }
        for fi in 0..self.faces.len() {
            if !matches!(self.faces[fi].surf, Surface::Plane { .. }) {
                bb.union(&self.face_bbox(fi));
            }
        }
        bb
    }

    /// A box sure to hold face `fi`: its edges' exact boxes and its surface's box over the
    /// rectangle holding its p-curves.
    pub fn face_bbox(&self, fi: usize) -> Bbox {
        let f = &self.faces[fi];
        let mut bb = Bbox::empty();
        let mut uv = Bbox::empty();
        for c in f.loops.iter().flatten() {
            let e = &self.edges[c.edge];
            if !e.degenerate {
                bb.union(&super::boolean::curve_bbox(&e.curve, e.t[0], e.t[1]));
            }
            uv.union(&super::boolean::curve_bbox(&c.pc, e.t[0], e.t[1]));
        }
        if !matches!(f.surf, Surface::Plane { .. }) && !uv.is_empty() {
            bb.union(&f.surf.patch_bbox((uv.min.x, uv.max.x), (uv.min.y, uv.max.y)));
        }
        bb
    }

    pub fn is_solid(&self) -> bool {
        !self.solids.is_empty()
    }

    /// Reverses a face's orientation (the outward normal flips): sense and loops.
    pub fn reverse_face(f: &mut Face) {
        f.sense = !f.sense;
        for l in &mut f.loops {
            l.reverse();
            for c in l.iter_mut() {
                c.fwd = !c.fwd;
            }
        }
    }

    /// Checks that every index the body holds refers to an entity it has (coedges to edges,
    /// edges to vertices, shells to faces, solids to shells) and that its geometry is well
    /// formed (finite points, valid B-splines): what every other method relies on without
    /// checking. Bodies from outside (files, the body format) are checked with it before use.
    /// Returns the problems (empty when sound).
    pub fn check_refs(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let finite = |p: &P3| p.iter().all(|x| x.is_finite());
        for (vi, v) in self.verts.iter().enumerate() {
            if !finite(&v.p) || !v.tol.is_finite() {
                errs.push(format!("vertex {vi} isn't finite"));
            }
        }
        for (ei, e) in self.edges.iter().enumerate() {
            if let Some(&v) = e.v.iter().find(|&&v| v >= self.verts.len()) {
                errs.push(format!("edge {ei}: vertex {v} doesn't exist"));
            }
            if !e.t.iter().all(|t| t.is_finite()) || !e.tol.is_finite() {
                errs.push(format!("edge {ei}: its range isn't finite"));
            }
            if let Some(why) = curve_issue(&e.curve) {
                errs.push(format!("edge {ei}: {why}"));
            }
        }
        for (fi, f) in self.faces.iter().enumerate() {
            if let Some(why) = surface_issue(&f.surf) {
                errs.push(format!("face {fi}: {why}"));
            }
            for (li, l) in f.loops.iter().enumerate() {
                if l.is_empty() {
                    errs.push(format!("face {fi} loop {li} is empty"));
                }
                for c in l {
                    if c.edge >= self.edges.len() {
                        errs.push(format!("face {fi} loop {li}: edge {} doesn't exist", c.edge));
                    }
                    if let Some(why) = curve_issue(&c.pc) {
                        errs.push(format!("face {fi} loop {li}: p-curve: {why}"));
                    }
                }
            }
        }
        for (si, s) in self.shells.iter().enumerate() {
            if let Some(f) = s.faces.iter().find(|&&f| f >= self.faces.len()) {
                errs.push(format!("shell {si}: face {f} doesn't exist"));
            }
        }
        for (si, s) in self.solids.iter().enumerate() {
            if let Some(sh) = s.shells.iter().find(|&&sh| sh >= self.shells.len()) {
                errs.push(format!("solid {si}: shell {sh} doesn't exist"));
            }
        }
        for (wi, w) in self.wires.iter().enumerate() {
            if w.edges.is_empty() {
                errs.push(format!("wire {wi} is empty"));
            }
            if let Some((e, _)) = w.edges.iter().find(|&&(e, _)| e >= self.edges.len()) {
                errs.push(format!("wire {wi}: edge {e} doesn't exist"));
            }
        }
        if let Some(v) = self.points.iter().find(|&&v| v >= self.verts.len()) {
            errs.push(format!("point vertex {v} doesn't exist"));
        }
        errs
    }

    /// Checks the structural invariants; returns a list of problems (empty when valid). A body
    /// whose indices or geometry are unsound ([`Body::check_refs`]) gets only those problems.
    pub fn check(&self) -> Vec<String> {
        let refs = self.check_refs();
        if !refs.is_empty() {
            return refs;
        }
        let mut errs = Vec::new();
        for (fi, f) in self.faces.iter().enumerate() {
            for (li, l) in f.loops.iter().enumerate() {
                for k in 0..l.len() {
                    let (a, b) = (&l[k], &l[(k + 1) % l.len()]);
                    if self.co_end(a) != self.co_start(b) {
                        errs.push(format!("face {fi} loop {li}: coedge {k} doesn't meet the next"));
                    }
                    // p-curve ends meet in uv (modulo periods).
                    let ea = &self.edges[a.edge];
                    let eb = &self.edges[b.edge];
                    let (_, ta1) = co_range(ea, a.fwd);
                    let (tb0, _) = co_range(eb, b.fwd);
                    let pa = a.pc.point(ta1);
                    let pb = b.pc.point(tb0);
                    let du = (pa.x - pb.x).abs();
                    let dv = (pa.y - pb.y).abs();
                    let (mu, mv) = f.surf.metric();
                    // Within the shared vertex's tolerance (LOOP_GAP for exact geometry).
                    let gap_tol = self.verts.get(self.co_end(a)).map_or(0.0, |v| v.tol).max(super::tol::LOOP_GAP);
                    if du * mu > gap_tol || dv * mv > gap_tol {
                        errs.push(format!("face {fi} loop {li}: p-curve gap after coedge {k}: ({:.3e}, {:.3e})", pa.x - pb.x, pa.y - pb.y));
                    }
                    // p-curve on the edge in 3D.
                    if !ea.degenerate {
                        for s in [0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9] {
                            let t = ea.t[0] + (ea.t[1] - ea.t[0]) * s;
                            let uv = a.pc.point(t);
                            let q = f.surf.eval(uv.x, uv.y);
                            let p = ea.point(t);
                            // Within the edge's tolerance (a read model's p-curves stray by it).
                            if (q - p).norm() > self.edges[a.edge].tol.max(super::tol::POINT) {
                                errs.push(format!("face {fi} loop {li} coedge {k} (edge {}): p-curve off the edge by {:.3e}", a.edge, (q - p).norm()));
                                break;
                            }
                        }
                    }
                }
            }
        }
        // Each edge ends at its vertices (within their tolerances).
        for (ei, e) in self.edges.iter().enumerate() {
            if e.degenerate {
                continue;
            }
            for k in 0..2 {
                let Some(v) = self.verts.get(e.v[k]) else {
                    errs.push(format!("edge {ei}: vertex {} doesn't exist", e.v[k]));
                    continue;
                };
                let d = (e.point(e.t[k]) - v.p).norm();
                if d > v.tol.max(e.tol).max(super::tol::POINT) {
                    errs.push(format!("edge {ei}: end {k} is {d:.3e} from its vertex"));
                }
            }
        }
        // Loops don't cross themselves in (u, v).
        for (fi, f) in self.faces.iter().enumerate() {
            for (li, l) in f.loops.iter().enumerate() {
                let mut poly: Vec<P2> = Vec::new();
                for c in l {
                    let e = &self.edges[c.edge];
                    let (t0, t1) = co_range(e, c.fwd);
                    for i in 0..8 {
                        let q = c.pc.point(t0 + (t1 - t0) * i as f64 / 8.0);
                        poly.push(P2::new(q.x, q.y));
                    }
                }
                if let Some((i, j)) = self_crossing(&poly) {
                    errs.push(format!("face {fi} loop {li} crosses itself in (u, v) (segments {i} and {j})"));
                }
            }
        }
        // Each edge of a closed shell is used twice, once each way.
        let mut uses = vec![(0i32, 0i32); self.edges.len()];
        for f in &self.faces {
            for l in &f.loops {
                for c in l {
                    if c.fwd {
                        uses[c.edge].0 += 1;
                    } else {
                        uses[c.edge].1 += 1;
                    }
                }
            }
        }
        for s in &self.shells {
            if !s.closed {
                continue;
            }
            let mut used = vec![false; self.edges.len()];
            for &fi in &s.faces {
                for l in &self.faces[fi].loops {
                    for c in l {
                        used[c.edge] = true;
                    }
                }
            }
            for (ei, u) in uses.iter().enumerate() {
                if used[ei] && *u != (1, 1) && !self.edges[ei].degenerate {
                    errs.push(format!("edge {ei} used {u:?} times in a closed shell"));
                }
            }
        }
        errs.extend(self.check_kinds(&uses));
        // Solids enclose a positive volume (their faces point out).
        if errs.is_empty() {
            for si in 0..self.solids.len() {
                let faces = self.solid_faces(si);
                let sub = super::kernel::subset(self, &faces, true);
                let v = super::mass::body_mass(&sub).volume;
                if !(v > 0.0) {
                    errs.push(format!("solid {si} encloses a volume of {v:.3e} (its faces point in)"));
                }
            }
        }
        errs
    }
}

impl Body {
    /// What the body is: one kind of piece (solids, sheets, wires or points), several
    /// (`Mixed`, a compound), or nothing. Free edges left in no wire don't count (they are
    /// construction leftovers, as before Phase 1b).
    pub fn class(&self) -> BodyClass {
        let in_solid: std::collections::HashSet<usize> = self.solids.iter().flat_map(|s| s.shells.iter().copied()).collect();
        let in_shell: std::collections::HashSet<usize> = self.shells.iter().flat_map(|s| s.faces.iter().copied()).collect();
        let solid = !self.solids.is_empty();
        // Faces of a shell of no solid, or of no shell at all.
        let sheet = (0..self.shells.len()).any(|s| !in_solid.contains(&s) && !self.shells[s].faces.is_empty())
            || (0..self.faces.len()).any(|f| !in_shell.contains(&f));
        let wire = !self.wires.is_empty();
        let point = !self.points.is_empty();
        match (solid, sheet, wire, point) {
            (false, false, false, false) => BodyClass::Empty,
            (true, false, false, false) => BodyClass::Solid,
            (false, true, false, false) => BodyClass::Sheet,
            (false, false, true, false) => BodyClass::Wire,
            (false, false, false, true) => BodyClass::Point,
            _ => BodyClass::Mixed,
        }
    }

    /// Whether wire `w` ends where it starts.
    pub fn wire_closed(&self, w: &Wire) -> bool {
        let (Some(&(e0, f0)), Some(&(e1, f1))) = (w.edges.first(), w.edges.last()) else { return false };
        let start = self.edges[e0].v[if f0 { 0 } else { 1 }];
        let end = self.edges[e1].v[if f1 { 1 } else { 0 }];
        start == end
    }

    /// Adds an acorn point.
    pub fn add_point(&mut self, p: P3) -> usize {
        let v = self.add_vertex(p);
        self.points.push(v);
        v
    }

    /// The edges no face uses and no degenerate one (wire edges and construction leftovers).
    pub fn free_edges(&self) -> Vec<usize> {
        let mut used = vec![false; self.edges.len()];
        for c in self.faces.iter().flat_map(|f| f.loops.iter().flatten()) {
            used[c.edge] = true;
        }
        (0..self.edges.len()).filter(|&e| !used[e] && !self.edges[e].degenerate).collect()
    }

    /// Gathers the free edges in no wire into wires: each chain of edges meeting at vertices
    /// that only two of them share (a vertex where three or more meet ends the chains there).
    /// Chains start at an end (a vertex of one edge) where there is one, so open chains come
    /// out whole; what is left is closed loops.
    pub fn chain_wires(&mut self) {
        let mut taken = vec![false; self.edges.len()];
        for w in &self.wires {
            for &(e, _) in &w.edges {
                taken[e] = true;
            }
        }
        let free: Vec<usize> = self.free_edges().into_iter().filter(|&e| !taken[e]).collect();
        let mut at: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
        for &e in &free {
            at.entry(self.edges[e].v[0]).or_default().push(e);
            if !self.edges[e].closed() {
                at.entry(self.edges[e].v[1]).or_default().push(e);
            }
        }
        let degree = |v: usize| at.get(&v).map_or(0, |l| l.len());
        // Ends of chains first, then anything left (closed loops).
        let mut starts: Vec<(usize, bool)> = Vec::new();
        for &e in &free {
            let ed = &self.edges[e];
            if degree(ed.v[0]) != 2 {
                starts.push((e, true));
            }
            if !ed.closed() && degree(ed.v[1]) != 2 {
                starts.push((e, false));
            }
        }
        starts.extend(free.iter().map(|&e| (e, true)));
        for (e0, fwd0) in starts {
            if taken[e0] {
                continue;
            }
            taken[e0] = true;
            let mut w = Wire { edges: vec![(e0, fwd0)] };
            let first = self.edges[e0].v[if fwd0 { 0 } else { 1 }];
            loop {
                let &(e, f) = w.edges.last().unwrap_or(&(e0, fwd0));
                let end = self.edges[e].v[if f { 1 } else { 0 }];
                if end == first || degree(end) != 2 {
                    break;
                }
                let Some(&next) = at[&end].iter().find(|&&n| !taken[n]) else { break };
                taken[next] = true;
                w.edges.push((next, self.edges[next].v[0] == end));
            }
            self.wires.push(w);
        }
    }

    /// The rules of each kind of piece ([`Body::check`]): `uses` counts each edge's forward and
    /// reversed uses by faces.
    /// - solids: their shells are closed (and, as checked before, each edge of a closed shell is
    ///   used once each way and the faces point out);
    /// - sheets: manifold: an edge of open shells only has at most two uses (boundary edges
    ///   have one). Not required oriented: read and 2D-boolean sheets may use an edge twice the
    ///   same way;
    /// - wires: chains of free, non-degenerate edges, each edge starting where the one before
    ///   ends, each in one wire once; a body with wires has every free edge in one;
    /// - points: vertices no edge uses, each listed once.
    fn check_kinds(&self, uses: &[(i32, i32)]) -> Vec<String> {
        let mut errs = Vec::new();
        for (si, s) in self.solids.iter().enumerate() {
            for &sh in &s.shells {
                if !self.shells[sh].closed {
                    errs.push(format!("solid {si}: shell {sh} is open"));
                }
            }
        }
        let mut in_closed = vec![false; self.edges.len()];
        for s in self.shells.iter().filter(|s| s.closed) {
            for &fi in &s.faces {
                for c in self.faces[fi].loops.iter().flatten() {
                    in_closed[c.edge] = true;
                }
            }
        }
        for (ei, &(f, r)) in uses.iter().enumerate() {
            if in_closed[ei] || self.edges[ei].degenerate {
                continue;
            }
            if f + r > 2 {
                errs.push(format!("edge {ei} is used by {} faces (a sheet must be manifold)", f + r));
            }
        }
        let mut in_wire = vec![0usize; self.edges.len()];
        for (wi, w) in self.wires.iter().enumerate() {
            for (k, &(e, fwd)) in w.edges.iter().enumerate() {
                in_wire[e] += 1;
                if self.edges[e].degenerate {
                    errs.push(format!("wire {wi}: edge {e} is degenerate"));
                }
                if uses[e] != (0, 0) {
                    errs.push(format!("wire {wi}: edge {e} is an edge of a face"));
                }
                if let Some(&(n, nf)) = w.edges.get(k + 1) {
                    let end = self.edges[e].v[if fwd { 1 } else { 0 }];
                    let start = self.edges[n].v[if nf { 0 } else { 1 }];
                    if end != start {
                        errs.push(format!("wire {wi}: edge {k} doesn't meet the next"));
                    }
                }
            }
        }
        for (e, &n) in in_wire.iter().enumerate() {
            if n > 1 {
                errs.push(format!("edge {e} is in wires {n} times"));
            }
        }
        if !self.wires.is_empty()
            && let Some(e) = self.free_edges().into_iter().find(|&e| in_wire[e] == 0)
        {
            errs.push(format!("edge {e} is free but in no wire"));
        }
        let mut on_edge = vec![false; self.verts.len()];
        for e in &self.edges {
            on_edge[e.v[0]] = true;
            on_edge[e.v[1]] = true;
        }
        let mut seen = std::collections::HashSet::new();
        for &v in &self.points {
            if on_edge[v] {
                errs.push(format!("point vertex {v} is the end of an edge"));
            }
            if !seen.insert(v) {
                errs.push(format!("point vertex {v} is listed twice"));
            }
        }
        errs
    }

    /// The first of the body's points, curves and surfaces that reaches outside the size box
    /// ±`max` (vertices; centres, origins and radii of curves and surfaces; B-spline control
    /// points; p-curves, in parameters, are left alone): a file's model the kernel can't hold
    /// to its tolerances ([`super::tol::SIZE`]).
    pub fn size_issue(&self, max: f64) -> Option<String> {
        let out = |x: f64| !(x.abs() <= max);
        let pt = |p: &P3| p.iter().any(|x| out(*x));
        let curve = |c: &Curve| match c {
            Curve::Line { o, .. } => pt(o),
            Curve::Circle { c, r, .. } => pt(c) || out(*r),
            Curve::Ellipse { c, a, b, .. } => pt(c) || out(*a) || out(*b),
            Curve::Nurbs(n) => n.cps.iter().any(|h| pt(&super::nurbs::dehom(h))),
        };
        if let Some(i) = self.verts.iter().position(|v| pt(&v.p)) {
            return Some(format!("vertex {i} lies outside the size box (±{max:e} mm)"));
        }
        if let Some(i) = self.edges.iter().position(|e| curve(&e.curve)) {
            return Some(format!("edge {i}'s curve reaches outside the size box (±{max:e} mm)"));
        }
        let surf = |s: &Surface| match s {
            Surface::Plane { f } => pt(&f.o),
            Surface::Cylinder { f, r } | Surface::Sphere { f, r } | Surface::Cone { f, r, .. } => pt(&f.o) || out(*r),
            Surface::Torus { f, rmaj, rmin } => pt(&f.o) || out(*rmaj) || out(*rmin),
            Surface::Revolution { f, m } => pt(&f.o) || curve(m),
            Surface::Extrusion { c, .. } => curve(c),
            Surface::Nurbs(n) => n.cps.iter().flatten().any(|h| pt(&super::nurbs::dehom(h))),
        };
        let i = self.faces.iter().position(|f| surf(&f.surf))?;
        Some(format!("face {i}'s surface reaches outside the size box (±{max:e} mm)"))
    }
}

/// Why a curve can't be evaluated (non-finite numbers, a malformed B-spline), if it can't.
fn curve_issue(c: &Curve) -> Option<String> {
    let finite = |v: &[f64]| v.iter().all(|x| x.is_finite());
    let ok = match c {
        Curve::Line { o, d } => finite(o.coords.as_slice()) && finite(d.as_slice()),
        Curve::Circle { c, x, y, r } => finite(c.coords.as_slice()) && finite(x.as_slice()) && finite(y.as_slice()) && r.is_finite(),
        Curve::Ellipse { c, x, y, a, b } => finite(c.coords.as_slice()) && finite(x.as_slice()) && finite(y.as_slice()) && a.is_finite() && b.is_finite(),
        Curve::Nurbs(n) => return n.validate().err().map(|e| e.message()),
    };
    (!ok).then(|| "a curve isn't finite".to_string())
}

/// Why a surface can't be evaluated, if it can't.
fn surface_issue(s: &Surface) -> Option<String> {
    let finite = |v: &[f64]| v.iter().all(|x| x.is_finite());
    let frame = |f: &Frame| finite(f.o.coords.as_slice()) && finite(f.x.as_slice()) && finite(f.y.as_slice()) && finite(f.z.as_slice());
    let ok = match s {
        Surface::Plane { f } => frame(f),
        Surface::Cylinder { f, r } | Surface::Sphere { f, r } => frame(f) && r.is_finite(),
        Surface::Cone { f, r, a } => frame(f) && r.is_finite() && a.is_finite(),
        Surface::Torus { f, rmaj, rmin } => frame(f) && rmaj.is_finite() && rmin.is_finite(),
        Surface::Revolution { f, m } => return if frame(f) { curve_issue(m) } else { Some("a surface isn't finite".into()) },
        Surface::Extrusion { c, d } => return if finite(d.as_slice()) { curve_issue(c) } else { Some("a surface isn't finite".into()) },
        Surface::Nurbs(n) => return n.validate().err().map(|e| e.message()),
    };
    (!ok).then(|| "a surface isn't finite".to_string())
}

impl Body {
    /// Raises vertex and edge tolerances to the gaps actually there (an edge's ends from its
    /// vertices, its p-curves from its curve), up to `cap`: an operation that merged within a
    /// tolerance records it, as OCCT does. Gaps beyond `cap` are left for [`Body::check`].
    pub fn fit_tolerances(&mut self, cap: f64) {
        for ei in 0..self.edges.len() {
            let e = &self.edges[ei];
            if e.degenerate {
                continue;
            }
            for k in 0..2 {
                let d = (e.point(e.t[k]) - self.verts[e.v[k]].p).norm();
                if d <= cap && d * 1.01 > self.verts[e.v[k]].tol {
                    self.verts[e.v[k]].tol = (d * 1.01).max(TOL_MIN);
                }
            }
        }
        let mut etol = vec![0.0f64; self.edges.len()];
        for f in &self.faces {
            for c in f.loops.iter().flatten() {
                let e = &self.edges[c.edge];
                if e.degenerate {
                    continue;
                }
                for s in [0.1, 0.25, 0.4, 0.5, 0.6, 0.75, 0.9] {
                    let t = e.t[0] + (e.t[1] - e.t[0]) * s;
                    let uv = c.pc.point(t);
                    etol[c.edge] = etol[c.edge].max((f.surf.eval(uv.x, uv.y) - e.point(t)).norm());
                }
            }
        }
        for (e, d) in self.edges.iter_mut().zip(etol) {
            if d <= cap && d * 1.01 > e.tol {
                e.tol = (d * 1.01).max(TOL_MIN);
            }
        }
    }
}

/// Two non-neighbouring segments of a closed polygon that cross (proper crossings only:
/// touching at a point is allowed, as a loop through a pole or along a seam does).
fn self_crossing(p: &[P2]) -> Option<(usize, usize)> {
    let n = p.len();
    if n < 4 {
        return None;
    }
    let mut size: f64 = 0.0;
    for q in p {
        size = size.max(q.x.abs()).max(q.y.abs());
    }
    let eps = 1e-9 * (1.0 + size);
    let cross = |a: &P2, b: &P2, c: &P2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    for i in 0..n {
        let (a, b) = (p[i], p[(i + 1) % n]);
        for j in i + 2..n {
            if (j + 1) % n == i {
                continue;
            }
            let (c, d) = (p[j], p[(j + 1) % n]);
            if a.x.max(b.x) < c.x.min(d.x) - eps || c.x.max(d.x) < a.x.min(b.x) - eps || a.y.max(b.y) < c.y.min(d.y) - eps || c.y.max(d.y) < a.y.min(b.y) - eps {
                continue;
            }
            let (d1, d2, d3, d4) = (cross(&a, &b, &c), cross(&a, &b, &d), cross(&c, &d, &a), cross(&c, &d, &b));
            let l1 = (b - a).norm() * eps.max(1e-300);
            let l2 = (d - c).norm() * eps.max(1e-300);
            if ((d1 > l1 && d2 < -l1) || (d1 < -l1 && d2 > l1)) && ((d3 > l2 && d4 < -l2) || (d3 < -l2 && d4 > l2)) {
                return Some((i, j));
            }
        }
    }
    None
}

/// Body kinds (Phase 1b): each kind's `check`, mass properties, tessellation, the body format
/// and STEP/IGES round trips, sweeps one kind higher, the cylindrical box and the revolve about
/// an axis off the sketch plane.
#[cfg(test)]
mod kind_tests {
    use std::f64::consts::{PI, TAU};

    use nalgebra::{Point2, Point3, Unit, Vector3};

    use super::Body;
    use crate::exchange::ExchangeFormat;
    use crate::native::kernel::NativeKernel;
    use crate::*;

    fn near(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * b.abs().max(1.0)
    }

    fn line(a: [f64; 2], b: [f64; 2]) -> PathCurve {
        PathCurve::Sketch { plane: Plane::top(), curve: Curve2::Line { a: Point2::new(a[0], a[1]), b: Point2::new(b[0], b[1]), source: None } }
    }

    /// A closed 10 × 10 square wire in the top plane.
    fn square(k: &mut NativeKernel) -> BodyId {
        let c = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let curves: Vec<PathCurve> = (0..4).map(|i| line(c[i], c[(i + 1) % 4])).collect();
        k.create_wire(&curves).unwrap().bodies[0]
    }

    fn body(k: &NativeKernel, id: BodyId) -> &Body {
        k.get(id).unwrap()
    }

    fn valid(k: &NativeKernel, id: BodyId) {
        let issues = body(k, id).check();
        assert!(issues.is_empty(), "{issues:?}");
    }

    /// Round trips through STEP, IGES and the body format; returns the bodies read.
    fn round_trips(k: &mut NativeKernel, id: BodyId) -> Vec<BodyId> {
        let bytes = k.write_body(id).unwrap();
        let back = k.read_body(&bytes).unwrap();
        assert_eq!(body(k, back), body(k, id), "the body format changed the body");
        let mut out = vec![back];
        for format in [ExchangeFormat::Step, ExchangeFormat::Iges] {
            let file = k.export_model(format, "kinds", &[(id, "part".into())], &[]).unwrap();
            let m = k.import_model(format, &file).unwrap_or_else(|e| panic!("{format:?}: {e}"));
            assert_eq!(m.parts.len(), 1, "{format:?}");
            out.push(m.parts[0].body);
        }
        out
    }

    #[test]
    fn point_bodies() {
        let mut k = NativeKernel::new();
        let id = k.create_points(&[Point3::new(1.0, 2.0, 3.0), Point3::new(3.0, 4.0, 5.0)]).unwrap().bodies[0];
        assert_eq!(k.body_class(id).unwrap(), BodyClass::Point);
        valid(&k, id);
        let m = k.mass_properties(id).unwrap();
        assert_eq!((m.volume, m.surface_area, m.length), (0.0, 0.0, 0.0));
        assert!((m.center_of_mass - Point3::new(2.0, 3.0, 4.0)).norm() < 1e-12);
        let t = k.tessellate(id, Tessellation::default()).unwrap();
        assert_eq!(t.points.len(), 2);
        assert!(t.indices.is_empty() && t.edges.is_empty());
        assert_eq!(k.vertices(id).unwrap().len(), 2);
        assert!(k.faces(id).unwrap().is_empty() && k.edges(id).unwrap().is_empty());
        for b in round_trips(&mut k, id) {
            assert_eq!(k.body_class(b).unwrap(), BodyClass::Point);
            valid(&k, b);
            let pts: Vec<Point3<f64>> = body(&k, b).points.iter().map(|&v| body(&k, b).verts[v].p).collect();
            assert_eq!(pts.len(), 2);
            assert!((pts[1] - Point3::new(3.0, 4.0, 5.0)).norm() < 1e-9);
        }
        assert!(matches!(k.create_points(&[]), Err(KernelError::InvalidParameter(_))));
        assert!(matches!(k.create_points(&[Point3::new(f64::NAN, 0.0, 0.0)]), Err(KernelError::InvalidInput(_))));
    }

    #[test]
    fn wire_bodies() {
        let mut k = NativeKernel::new();
        let id = square(&mut k);
        assert_eq!(k.body_class(id).unwrap(), BodyClass::Wire);
        valid(&k, id);
        let b = body(&k, id);
        assert_eq!(b.wires.len(), 1);
        assert!(b.wire_closed(&b.wires[0]));
        let m = k.mass_properties(id).unwrap();
        assert!(near(m.length, 40.0, 1e-12));
        assert!((m.center_of_mass - Point3::new(5.0, 5.0, 0.0)).norm() < 1e-9);
        let t = k.tessellate(id, Tessellation::default()).unwrap();
        assert_eq!(t.edges.len(), 4);
        assert!(t.indices.is_empty());
        let es = k.edges(id).unwrap();
        assert_eq!(es.len(), 4);
        assert!(es.iter().all(|e| e.faces == [None, None]));
        assert_eq!(k.vertices(id).unwrap().len(), 4);
        // An open wire with an arc: a quarter circle of radius 5, then a line of 3.
        let arc = PathCurve::Sketch { plane: Plane::top(), curve: Curve2::Arc { center: Point2::origin(), radius: 5.0, start_angle: 0.0, sweep: PI / 2.0, source: None } };
        let open = k.create_wire(&[line([0.0, 5.0], [-3.0, 5.0]), arc]).unwrap().bodies[0];
        valid(&k, open);
        let ob = body(&k, open);
        assert!(!ob.wire_closed(&ob.wires[0]));
        let want = 5.0 * PI / 2.0 + 3.0;
        assert!(near(k.mass_properties(open).unwrap().length, want, 1e-9));
        for b in round_trips(&mut k, id) {
            assert_eq!(k.body_class(b).unwrap(), BodyClass::Wire, "{b:?}");
            valid(&k, b);
            let bb = body(&k, b);
            assert!(bb.wires.len() == 1 && bb.wire_closed(&bb.wires[0]));
            assert!(near(k.mass_properties(b).unwrap().length, 40.0, 1e-9));
        }
        for b in round_trips(&mut k, open) {
            valid(&k, b);
            assert!(near(k.mass_properties(b).unwrap().length, want, 1e-9), "{}", k.mass_properties(b).unwrap().length);
        }
        // Curves that don't join are refused.
        assert!(k.create_wire(&[line([0.0, 0.0], [1.0, 0.0]), line([5.0, 5.0], [6.0, 5.0])]).is_err());
    }

    #[test]
    fn check_finds_broken_kinds() {
        let mut k = NativeKernel::new();
        let id = square(&mut k);
        let good = body(&k, id).clone();
        // A wire whose edges don't meet.
        let mut b = good.clone();
        b.wires[0].edges.swap(1, 2);
        assert!(b.check().iter().any(|i| i.contains("doesn't meet")), "{:?}", b.check());
        // A free edge left out of the wires.
        let mut b = good.clone();
        b.wires[0].edges.pop();
        assert!(b.check().iter().any(|i| i.contains("in no wire")), "{:?}", b.check());
        // An edge in two wires; a wire edge out of range.
        let mut b = good.clone();
        b.wires.push(super::Wire { edges: vec![b.wires[0].edges[0]] });
        assert!(b.check().iter().any(|i| i.contains("in wires 2 times")));
        let mut b = good.clone();
        b.wires[0].edges.push((99, true));
        assert!(b.check_refs().iter().any(|i| i.contains("edge 99")));
        // A point on an edge, and listed twice.
        let mut b = good.clone();
        b.points = vec![0, 0];
        let issues = b.check();
        assert!(issues.iter().any(|i| i.contains("end of an edge")) && issues.iter().any(|i| i.contains("twice")), "{issues:?}");
        // A sheet whose edge three faces use (non-manifold).
        let sheet = k.sweep_body(id, &BodySweep::Translate(Vector3::new(0.0, 0.0, 5.0))).unwrap().bodies[0];
        let mut b = body(&k, sheet).clone();
        valid(&k, sheet);
        b.faces.push(b.faces[0].clone());
        b.shells[0].faces.push(b.faces.len() - 1);
        assert!(b.check().iter().any(|i| i.contains("manifold")), "{:?}", b.check());
        // A solid whose shell isn't closed.
        let mut c = k.extrude(&Profile::new(Plane::top(), vec![Region { outer: Loop { curves: vec![Curve2::Circle { center: Point2::origin(), radius: 1.0, source: None }] }, holes: vec![], source: None }]), Extent::Blind(1.0)).map(|r| body(&k, r.bodies[0]).clone()).unwrap();
        c.shells[0].closed = false;
        assert!(c.check().iter().any(|i| i.contains("is open")));
    }

    #[test]
    fn sweeps_one_kind_higher() {
        let mut k = NativeKernel::new();
        // Points into a wire, straight and turned (part and whole).
        let p = k.create_points(&[Point3::new(5.0, 0.0, 0.0)]).unwrap().bodies[0];
        let w = k.sweep_body(p, &BodySweep::Translate(Vector3::new(0.0, 3.0, 4.0))).unwrap().bodies[0];
        assert_eq!(k.body_class(w).unwrap(), BodyClass::Wire);
        assert!(near(k.mass_properties(w).unwrap().length, 5.0, 1e-12));
        let z = Axis { origin: Point3::origin(), dir: Vector3::z_axis() };
        let arc = k.sweep_body(p, &BodySweep::Rotate { axis: z, angle: PI / 2.0 }).unwrap().bodies[0];
        valid(&k, arc);
        assert!(near(k.mass_properties(arc).unwrap().length, 2.5 * PI, 1e-12));
        let ring = k.sweep_body(p, &BodySweep::Rotate { axis: z, angle: -TAU }).unwrap().bodies[0];
        valid(&k, ring);
        assert!(body(&k, ring).edges[0].closed());
        assert!(near(k.mass_properties(ring).unwrap().length, 10.0 * PI, 1e-12));
        // A point on the axis sweeps nothing.
        let o = k.create_points(&[Point3::new(0.0, 0.0, 2.0)]).unwrap().bodies[0];
        assert!(k.sweep_body(o, &BodySweep::Rotate { axis: z, angle: 1.0 }).is_err());
        // A wire into a sheet: the square's walls.
        let sq = square(&mut k);
        let walls = k.sweep_body(sq, &BodySweep::Translate(Vector3::new(0.0, 0.0, 5.0))).unwrap();
        let wb = walls.bodies[0];
        valid(&k, wb);
        assert_eq!(k.body_class(wb).unwrap(), BodyClass::Sheet);
        assert!(near(k.mass_properties(wb).unwrap().surface_area, 200.0, 1e-12));
        assert_eq!(walls.history.generated.len(), 4);
        assert!(walls.history.generated.iter().all(|(_, o)| matches!(o, Origin::FromEdge { body, .. } if *body == sq)));
        // A segment into a flat sheet, then that into a box.
        let seg = k.create_wire(&[line([0.0, 0.0], [10.0, 0.0])]).unwrap().bodies[0];
        let flat = k.sweep_body(seg, &BodySweep::Translate(Vector3::new(0.0, 10.0, 0.0))).unwrap().bodies[0];
        valid(&k, flat);
        assert!(near(k.mass_properties(flat).unwrap().surface_area, 100.0, 1e-12));
        let boxed = k.sweep_body(flat, &BodySweep::Translate(Vector3::new(0.0, 0.0, -5.0))).unwrap().bodies[0];
        valid(&k, boxed);
        assert_eq!(k.body_class(boxed).unwrap(), BodyClass::Solid);
        let m = k.mass_properties(boxed).unwrap();
        assert!(near(m.volume, 500.0, 1e-12) && near(m.surface_area, 400.0, 1e-12), "{m:?}");
        // A sheet swept along itself makes no solid.
        assert!(matches!(k.sweep_body(flat, &BodySweep::Translate(Vector3::new(1.0, 0.0, 0.0))), Err(KernelError::InvalidParameter(_))));
        // Turned: a segment parallel to the axis into a cylinder, its rectangle into a tube.
        let up = k.create_wire(&[PathCurve::Bezier3 { poles: vec![Point3::new(5.0, 0.0, 0.0), Point3::new(5.0, 0.0, 4.0)], weights: vec![] }]).unwrap().bodies[0];
        let cyl = k.sweep_body(up, &BodySweep::Rotate { axis: z, angle: TAU }).unwrap().bodies[0];
        valid(&k, cyl);
        assert!(near(k.mass_properties(cyl).unwrap().surface_area, 40.0 * PI, 1e-9));
        let rect = k.sweep_body(up, &BodySweep::Translate(Vector3::new(2.0, 0.0, 0.0))).unwrap().bodies[0];
        let tube = k.sweep_body(rect, &BodySweep::Rotate { axis: z, angle: TAU }).unwrap().bodies[0];
        valid(&k, tube);
        assert!(near(k.mass_properties(tube).unwrap().volume, PI * 24.0 * 4.0, 1e-9));
        let quarter = k.sweep_body(rect, &BodySweep::Rotate { axis: z, angle: PI / 2.0 }).unwrap().bodies[0];
        valid(&k, quarter);
        assert!(near(k.mass_properties(quarter).unwrap().volume, PI * 24.0, 1e-9));
        // A segment skew to the axis turned: a hyperboloid; area 2π ∫ ρ √(ρ'² + h'²) dt.
        let skew = k.create_wire(&[PathCurve::Bezier3 { poles: vec![Point3::new(5.0, -1.0, 0.0), Point3::new(5.0, 1.0, 3.0)], weights: vec![] }]).unwrap().bodies[0];
        let hyp = k.sweep_body(skew, &BodySweep::Rotate { axis: z, angle: TAU }).unwrap().bodies[0];
        valid(&k, hyp);
        let n = 20000;
        let want: f64 = (0..n)
            .map(|i| {
                let t = (i as f64 + 0.5) / n as f64;
                let (y, dy) = (-1.0 + 2.0 * t, 2.0);
                let rho = (25.0 + y * y).sqrt();
                TAU * rho * ((y * dy / rho).powi(2) + 9.0).sqrt() / n as f64
            })
            .sum();
        assert!(near(k.mass_properties(hyp).unwrap().surface_area, want, 1e-6), "{} vs {want}", k.mass_properties(hyp).unwrap().surface_area);
        for b in round_trips(&mut k, hyp) {
            assert!(near(k.mass_properties(b).unwrap().surface_area, want, 1e-6));
        }
    }

    #[test]
    fn cylindrical_box() {
        let mut k = NativeKernel::new();
        let spec = |len: f64, ri: f64, ro: f64, a0: f64, a1: f64| CylindricalBox { frame: Plane::top(), length: len, inner_radius: ri, outer_radius: ro, start_angle: a0.to_radians(), end_angle: a1.to_radians() };
        let id = k.cylindrical_box(&spec(10.0, 2.0, 5.0, 30.0, 120.0)).unwrap().bodies[0];
        valid(&k, id);
        let m = k.mass_properties(id).unwrap();
        let th = PI / 2.0;
        assert!(near(m.volume, th / 2.0 * 21.0 * 10.0, 1e-9));
        assert!(near(m.surface_area, th * 21.0 + th * 7.0 * 10.0 + 2.0 * 3.0 * 10.0, 1e-9));
        // It starts at 30°: its centroid lies at 75°.
        let c = m.center_of_mass;
        assert!((c.y.atan2(c.x) - 75f64.to_radians()).abs() < 1e-9);
        let tube = k.cylindrical_box(&spec(1.0, 1.0, 2.0, 0.0, 360.0)).unwrap().bodies[0];
        valid(&k, tube);
        assert!(near(k.mass_properties(tube).unwrap().volume, 3.0 * PI, 1e-9));
        for bad in [spec(0.0, 2.0, 5.0, 0.0, 90.0), spec(-1.0, 2.0, 5.0, 0.0, 90.0), spec(10.0, 0.0, 5.0, 0.0, 90.0), spec(10.0, 5.0, 2.0, 0.0, 90.0), spec(10.0, 5.0, 5.0, 0.0, 90.0), spec(10.0, 2.0, 5.0, 400.0, 450.0), spec(10.0, 2.0, 5.0, -10.0, 50.0), spec(10.0, 2.0, 5.0, 90.0, 10.0), spec(10.0, 2.0, 5.0, 0.0, 400.0), spec(f64::NAN, 2.0, 5.0, 0.0, 90.0), spec(10.0, 2.0, f64::INFINITY, 0.0, 90.0)] {
            assert!(matches!(k.cylindrical_box(&bad), Err(KernelError::InvalidParameter(_))), "{bad:?}");
        }
    }

    #[test]
    fn revolve_about_an_axis_off_the_plane() {
        let mut k = NativeKernel::new();
        // The square [1, 2] × [0, 1] of the top plane about the y-parallel axis one below it:
        // swept volume θ ∫ x dA = 1.5 θ (each point leaves the plane at speed x).
        let sq = Region { outer: Loop { curves: [[1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [1.0, 1.0]].iter().zip([[2.0, 0.0], [2.0, 1.0], [1.0, 1.0], [1.0, 0.0]]).map(|(a, b)| Curve2::Line { a: Point2::new(a[0], a[1]), b: Point2::new(b[0], b[1]), source: None }).collect() }, holes: vec![], source: None };
        let p = Profile::new(Plane::top(), vec![sq]);
        let axis = Axis { origin: Point3::new(0.0, 0.0, -1.0), dir: Vector3::y_axis() };
        for (angle, vol) in [(PI / 2.0, 0.75 * PI), (TAU, 3.0 * PI)] {
            let id = k.revolve(&p, axis, angle).unwrap().bodies[0];
            valid(&k, id);
            assert!(near(k.mass_properties(id).unwrap().volume, vol, 1e-9), "{angle}: {}", k.mass_properties(id).unwrap().volume);
        }
        // A skew axis (SMLib sweep 31's kind): valid, and volume θ ∫ (z × r)·n dA.
        let skew = Axis { origin: Point3::new(4.0, 2.0, -3.0), dir: Unit::new_normalize(Vector3::new(-2.0, 5.0, 1.0)) };
        let id = k.revolve(&p, skew, 1.0).unwrap().bodies[0];
        valid(&k, id);
        let (z, o) = (skew.dir.into_inner(), skew.origin);
        let speed = |x: f64, y: f64| z.cross(&(Point3::new(x, y, 0.0) - o)).z.abs();
        let n = 200;
        let want: f64 = (0..n * n).map(|i| speed(1.0 + ((i % n) as f64 + 0.5) / n as f64, ((i / n) as f64 + 0.5) / n as f64) / (n * n) as f64).sum();
        assert!(near(k.mass_properties(id).unwrap().volume, want, 1e-6), "{} vs {want}", k.mass_properties(id).unwrap().volume);
        // An axis square to the plane, or one the profile straddles, is refused.
        let normal = Axis { origin: Point3::new(0.0, 0.0, -1.0), dir: Vector3::z_axis() };
        assert!(k.revolve(&p, normal, 1.0).is_err());
        let across = Axis { origin: Point3::new(1.5, 0.0, -1.0), dir: Vector3::y_axis() };
        assert!(k.revolve(&p, across, 1.0).is_err());
    }
}
