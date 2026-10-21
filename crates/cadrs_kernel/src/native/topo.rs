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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Body {
    pub verts: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
    pub shells: Vec<Shell>,
    pub solids: Vec<Solid>,
}

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
