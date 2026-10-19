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

/// The smallest tolerance an entity has (OCCT's `Precision::Confusion`): exact geometry.
pub const TOL_MIN: f64 = 1e-7;

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

    /// The bounding box of the vertices and sampled edges and faces (loose but covering).
    pub fn bbox(&self) -> Bbox {
        let mut bb = Bbox::empty();
        for v in &self.verts {
            bb.add(&v.p);
        }
        for e in &self.edges {
            if e.degenerate {
                continue;
            }
            let n = 16;
            for i in 0..=n {
                bb.add(&e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / n as f64));
            }
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

    /// Checks the structural invariants; returns a list of problems (empty when valid).
    pub fn check(&self) -> Vec<String> {
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
                    if du * mu > 1e-5 || dv * mv > 1e-5 {
                        errs.push(format!("face {fi} loop {li}: p-curve gap after coedge {k}: ({:.3e}, {:.3e})", pa.x - pb.x, pa.y - pb.y));
                    }
                    // p-curve on the edge in 3D.
                    if !ea.degenerate {
                        for s in [0.25, 0.5, 0.75] {
                            let t = ea.t[0] + (ea.t[1] - ea.t[0]) * s;
                            let uv = a.pc.point(t);
                            let q = f.surf.eval(uv.x, uv.y);
                            let p = ea.point(t);
                            if (q - p).norm() > 1e-6 {
                                errs.push(format!("face {fi} loop {li} coedge {k} (edge {}): p-curve off the edge by {:.3e}", a.edge, (q - p).norm()));
                                break;
                            }
                        }
                    }
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
        errs
    }
}
