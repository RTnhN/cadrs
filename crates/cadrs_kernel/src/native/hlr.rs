//! Hidden-line removal for drawing views ([`crate::Kernel::project`], P3C.2), pure Rust.
//!
//! See [`crate::projection`] for the output types and the 2D frame. The lines of a view are:
//! - every edge of each body, except seams (an edge used twice by one face: not a drawing line)
//!   and degenerate edges. An edge between two faces whose outward normals agree along it
//!   (tangent faces, a fillet's boundary) is Smooth, any other edge Sharp;
//! - the outlines (silhouettes) of curved faces, where the surface normal is square to the
//!   direction of sight, clipped to the face's trimmed domain: on cylinders and cones the
//!   generators at the (one or two) angles u where n·dir = 0 (along u, n·dir is
//!   `A cos u + B sin u + C`, fitted from three normals); on spheres the great circle square to
//!   the direction of sight; on anything else (tori, surfaces of revolution and extrusion,
//!   NURBS) the zero set of n·dir traced on a uv grid (marching squares, each grid crossing
//!   bisected onto the exact zero) as a polyline.
//!
//! Each line is projected to 2D as a polyline within the tolerance and keeps its exact 2D shape:
//! a line stays a line, a circle seen face-on becomes an arc, a circle or ellipse seen edge-on a
//! line, split where its projection turns back (those splits are never merged); anything else is
//! a polyline. The lines are split wherever they cross or touch each other in 2D (pairs found on
//! a 2D grid; crossings of exact curves refined by Newton; collinear overlaps split at each
//! other's ends), and each piece is visible when a ray from a point of it towards the eye meets
//! no face of any body (so bodies hide each other): planes, cylinders, cones and spheres exactly
//! (`classify::ray_surface` against the trimmed domain, a hit on a face's boundary not hiding),
//! other faces through their triangles. Hits within a skin of the point (its own faces) don't
//! count, and faces seen edge-on hide nothing (the lines bounding them do). A long piece is
//! tested at three points; where they disagree (an edge passing through another body's face)
//! it is split by bisection.
//!
//! Then, as OCCT's post-processing does: a hidden piece lying on a visible one (a box's back
//! edges behind its front ones) is dropped, and of two pieces of the same visibility lying on
//! each other the one nearer the eye stays (an edge before an outline on a tie). Consecutive
//! pieces of one line with the same visibility are merged back, round the closing point of a
//! whole circle too (so an unbroken circle is one full arc). Sources: the body's edge index for
//! edges, its face index for outlines.

use std::collections::HashMap;

use rayon::prelude::*;

use super::classify::{in_face, ray_surface, InFace};
use super::curve::Curve;
use super::math::*;
use super::surface::Surface;
use super::tess;
use super::topo::Body;
use crate::projection::{ProjClass, ProjCurve, ProjEdge, ProjSource, ProjVisibility, Projection, ProjectOptions, ViewFrame};
use crate::{EdgeId, FaceId};

/// A body to project and its faces' uv polygons (`classify::face_polygons`, cached by the
/// kernel's entry).
pub struct HlrBody<'a> {
    pub body: &'a Body,
    pub polys: &'a [Vec<Vec<P2>>],
}

/// Largest turn (radians) of the tangent between two polyline points.
const ANG: f64 = 0.1;
/// Two faces meet smoothly when their normals are within 0.5° along the edge.
const SMOOTH_COS: f64 = 0.999_96;
/// Two 2D points closer than this (mm) are the same point.
const EPS2: f64 = 1e-7;
/// Lines whose directions differ by less than this sine lie along each other.
const PARALLEL: f64 = 0.15;

/// A face's trimmed domain: its uv polygons and their box.
struct Domain<'a> {
    surf: &'a Surface,
    polys: &'a [Vec<P2>],
    ub: (f64, f64),
    vb: (f64, f64),
}

impl<'a> Domain<'a> {
    fn new(surf: &'a Surface, polys: &'a [Vec<P2>]) -> Self {
        let mut ub = (f64::INFINITY, f64::NEG_INFINITY);
        let mut vb = (f64::INFINITY, f64::NEG_INFINITY);
        for q in polys.iter().flatten() {
            ub = (ub.0.min(q.x), ub.1.max(q.x));
            vb = (vb.0.min(q.y), vb.1.max(q.y));
        }
        Self { surf, polys, ub, vb }
    }

    fn is_empty(&self) -> bool {
        !(self.ub.0 < self.ub.1 && self.vb.0 <= self.vb.1)
    }

    /// Whether (u, v), shifted by whole periods, lies in the face (on its boundary too unless
    /// `strict`).
    fn contains(&self, u: f64, v: f64, strict: bool) -> bool {
        if self.is_empty() {
            return false;
        }
        let shifts = |x: f64, (lo, hi): (f64, f64), periodic: bool| -> Vec<f64> {
            let ks: &[f64] = if periodic { &[0.0, -1.0, 1.0, -2.0, 2.0] } else { &[0.0] };
            ks.iter().map(|k| x + TAU * k).filter(|y| *y >= lo - 1e-9 && *y <= hi + 1e-9).collect()
        };
        for u in shifts(u, self.ub, self.surf.u_periodic()) {
            for v in shifts(v, self.vb, self.surf.v_periodic()) {
                match in_face(self.polys, self.surf, P2::new(u, v), 1e-6) {
                    InFace::Inside => return true,
                    InFace::Boundary if !strict => return true,
                    _ => {}
                }
            }
        }
        false
    }
}

/// A line's 3D geometry: an exact curve, or a polyline parametrized by vertex index.
enum Geo {
    Exact(Curve),
    Poly(Vec<P3>),
}

/// The exact 2D shape of a projected line.
#[derive(Clone, Copy)]
enum Shape {
    Line,
    Arc { center: P2, radius: f64 },
    Other,
}

/// One line of the view before splitting: an edge or an outline.
struct Src {
    body: usize,
    class: ProjClass,
    edge: Option<usize>,
    face: Option<usize>,
    /// The faces (of `body`) the line lies on: their hits near it are within the skin.
    own: Vec<usize>,
    geo: Geo,
    t: (f64, f64),
    /// A whole periodic curve (circle, ellipse): pieces may merge round `t.0`.
    closed: bool,
    shape: Shape,
    /// Parameters where the projection turns back, and whether `t.0` is one (closed curves).
    turns: Vec<f64>,
    turn_at_start: bool,
    /// The 2D polyline: parameters, points and depths.
    ts: Vec<f64>,
    p2: Vec<P2>,
}

impl Src {
    fn point(&self, t: f64) -> P3 {
        match &self.geo {
            Geo::Exact(c) => c.point(t),
            Geo::Poly(p) => {
                let n = p.len() - 1;
                let t = t.clamp(0.0, n as f64);
                let i = (t.floor() as usize).min(n - 1);
                p[i] + (p[i + 1] - p[i]) * (t - i as f64)
            }
        }
    }

    fn deriv(&self, t: f64) -> V3 {
        match &self.geo {
            Geo::Exact(c) => c.tangent(t),
            Geo::Poly(p) => {
                let n = p.len() - 1;
                let i = (t.clamp(0.0, n as f64).floor() as usize).min(n - 1);
                p[i + 1] - p[i]
            }
        }
    }

    /// Polyline parameters over [a, b] within `tol`, both ends included.
    fn params(&self, a: f64, b: f64, tol: f64) -> Vec<f64> {
        match &self.geo {
            Geo::Exact(c) => {
                let mut v = c.sample_params(a, b, tol, ANG);
                if let Some(f) = v.first_mut() {
                    *f = a;
                }
                if let Some(l) = v.last_mut() {
                    *l = b;
                }
                if v.len() < 2 {
                    v = vec![a, b];
                }
                v
            }
            Geo::Poly(_) => {
                let mut v = vec![a];
                let mut k = a.floor() + 1.0;
                while k < b - 1e-9 {
                    if k > a + 1e-9 {
                        v.push(k);
                    }
                    k += 1.0;
                }
                v.push(b);
                v
            }
        }
    }

    fn is_exact(&self) -> bool {
        matches!(self.geo, Geo::Exact(_))
    }
}

/// The 2D shape of a curve seen along `dir`, the parameters in (t0, t1) where its projection
/// turns back (a circle or ellipse seen edge-on), and whether t0 is such a parameter.
fn shape_of(frame: &ViewFrame, c: &Curve, t0: f64, t1: f64) -> (Shape, Vec<f64>, bool) {
    let dir = frame.dir;
    let edge_on = |n: V3, xa: V3, ya: V3| -> (Shape, Vec<f64>, bool) {
        let w = n.cross(&dir).normalize();
        let ts = ya.dot(&w).atan2(xa.dot(&w));
        let mut turns = Vec::new();
        let mut at_start = false;
        for k in -6..=6 {
            let t = ts + PI * k as f64;
            if t > t0 + 1e-9 && t < t1 - 1e-9 {
                turns.push(t);
            }
            if (wrap(t - t0 + PI, 0.0, TAU) - PI).abs() < 1e-9 {
                at_start = true;
            }
        }
        turns.sort_by(f64::total_cmp);
        (Shape::Line, turns, at_start)
    };
    match c {
        Curve::Line { .. } => (Shape::Line, vec![], false),
        Curve::Circle { c, x, y, r } => {
            let n = x.cross(y).normalize();
            let nd = n.dot(&dir);
            if nd.abs() > 1.0 - 1e-9 {
                (Shape::Arc { center: frame.to_2d(c), radius: *r }, vec![], false)
            } else if nd.abs() < 1e-9 {
                edge_on(n, x * *r, y * *r)
            } else {
                (Shape::Other, vec![], false)
            }
        }
        Curve::Ellipse { x, y, a, b, .. } => {
            let n = x.cross(y).normalize();
            if n.dot(&dir).abs() < 1e-9 {
                edge_on(n, x * *a, y * *b)
            } else {
                (Shape::Other, vec![], false)
            }
        }
        Curve::Nurbs(_) => (Shape::Other, vec![], false),
    }
}

#[allow(clippy::too_many_arguments)]
fn make_src(frame: &ViewFrame, body: usize, class: ProjClass, edge: Option<usize>, face: Option<usize>, own: Vec<usize>, geo: Geo, t: (f64, f64), closed: bool, tol: f64) -> Option<Src> {
    let (shape, turns, turn_at_start) = match &geo {
        Geo::Exact(c) => shape_of(frame, c, t.0, t.1),
        Geo::Poly(_) => (Shape::Other, vec![], false),
    };
    let mut s = Src { body, class, edge, face, own, geo, t, closed, shape, turns, turn_at_start, ts: vec![], p2: vec![] };
    let mut bps = vec![t.0];
    bps.extend(s.turns.iter().copied());
    bps.push(t.1);
    let mut ts: Vec<f64> = Vec::new();
    for w in bps.windows(2) {
        let p = s.params(w[0], w[1], tol);
        let skip = usize::from(!ts.is_empty());
        ts.extend(p.into_iter().skip(skip));
    }
    s.p2 = ts.iter().map(|&t| frame.to_2d(&s.point(t))).collect();
    s.ts = ts;
    // A line seen end-on projects to a point: nothing to draw.
    let (mut lo, mut hi) = (s.p2[0], s.p2[0]);
    for p in &s.p2 {
        lo = lo.inf(p);
        hi = hi.sup(p);
    }
    if (hi - lo).norm() < 1e-6 {
        return None;
    }
    Some(s)
}

/// The body's edges as lines of the view.
fn edge_srcs(frame: &ViewFrame, b: &Body, bi: usize, tol: f64) -> Vec<Src> {
    let ef = b.edge_faces();
    (0..b.edges.len())
        .into_par_iter()
        .filter_map(|ei| {
            let e = &b.edges[ei];
            if e.degenerate {
                return None;
            }
            let faces = &ef[ei];
            if faces.len() == 2 && faces[0] == faces[1] {
                return None;
            }
            let smooth = faces.len() == 2 && {
                [0.2, 0.5, 0.8].iter().all(|s| {
                    let p = e.point(e.t[0] + (e.t[1] - e.t[0]) * s);
                    let n = |fi: usize| {
                        let f = &b.faces[fi];
                        let (u, v) = f.surf.project(&p, None);
                        f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 }
                    };
                    n(faces[0]).dot(&n(faces[1])) > SMOOTH_COS
                })
            };
            let class = if smooth { ProjClass::Smooth } else { ProjClass::Sharp };
            let closed = e.closed() && matches!(e.curve, Curve::Circle { .. } | Curve::Ellipse { .. }) && (e.t[1] - e.t[0] - TAU).abs() < 1e-9;
            make_src(frame, bi, class, Some(ei), None, faces.clone(), Geo::Exact(e.curve.clone()), (e.t[0], e.t[1]), closed, tol)
        })
        .collect()
}

/// The runs of [a, b] where `inside` holds, found on `n` samples and refined by bisection.
fn clip(a: f64, b: f64, n: usize, inside: impl Fn(f64) -> bool) -> Vec<(f64, f64)> {
    let ts: Vec<f64> = (0..=n).map(|i| a + (b - a) * i as f64 / n as f64).collect();
    let st: Vec<bool> = ts.iter().map(|&t| inside(t)).collect();
    let cut = |mut lo: f64, mut hi: f64, s: bool| {
        for _ in 0..50 {
            let m = 0.5 * (lo + hi);
            if inside(m) == s {
                lo = m;
            } else {
                hi = m;
            }
        }
        0.5 * (lo + hi)
    };
    let mut runs = Vec::new();
    let mut start = if st[0] { Some(a) } else { None };
    for i in 1..=n {
        if st[i] != st[i - 1] {
            let c = cut(ts[i - 1], ts[i], st[i - 1]);
            match start.take() {
                Some(s) => runs.push((s, c)),
                None => start = Some(c),
            }
        }
    }
    if let Some(s) = start {
        runs.push((s, b));
    }
    runs.retain(|r| r.1 - r.0 > 1e-12);
    runs
}

/// The zero set of n·dir on the face, traced on a uv grid and clipped to the face, as 3D
/// polylines. In a periodic direction the grid is shifted off the box by a fraction of a cell
/// and runs a cell past it, so a contour lying on the seam (a torus's meridian seen side-on)
/// falls inside cells rather than on grid lines (the clipping and the overlap test drop the
/// copy a period away).
fn trace_outline(dom: &Domain, dir: &V3) -> Vec<Vec<P3>> {
    let n = 64usize;
    let s = dom.surf;
    let (ub, vb) = (dom.ub, dom.vb);
    let (du, dv) = ((ub.1 - ub.0) / n as f64, (vb.1 - vb.0) / n as f64);
    if du <= 0.0 || dv <= 0.0 {
        return vec![];
    }
    let axis = |lo: f64, d: f64, periodic: bool| if periodic { (lo - 0.381_966 * d, n + 1) } else { (lo, n) };
    let ((u0, nu), (v0, nv)) = (axis(ub.0, du, s.u_periodic()), axis(vb.0, dv, s.v_periodic()));
    let g = |q: P2| s.normal(q.x, q.y).dot(dir);
    let uv = |i: usize, j: usize| P2::new(u0 + du * i as f64, v0 + dv * j as f64);
    let vals: Vec<f64> = (0..(nu + 1) * (nv + 1)).map(|k| g(uv(k % (nu + 1), k / (nu + 1)))).collect();
    if vals.iter().all(|x| x.abs() < 1e-9) {
        return vec![];
    }
    let val = |i: usize, j: usize| vals[j * (nu + 1) + i];
    let pos = |x: f64| x >= 0.0;
    // Grid edge keys: 2·node for the edge to (i + 1, j), 2·node + 1 for the edge to (i, j + 1).
    let node = |i: usize, j: usize| j * (nu + 1) + i;
    let mut segs: Vec<(usize, usize)> = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            let c = [val(i, j), val(i + 1, j), val(i + 1, j + 1), val(i, j + 1)];
            let sg = c.map(pos);
            let keys = [2 * node(i, j), 2 * node(i + 1, j) + 1, 2 * node(i, j + 1), 2 * node(i, j) + 1];
            let cross = [sg[0] != sg[1], sg[1] != sg[2], sg[3] != sg[2], sg[0] != sg[3]];
            let list: Vec<usize> = (0..4).filter(|&k| cross[k]).collect();
            match list.len() {
                2 => segs.push((keys[list[0]], keys[list[1]])),
                4 => {
                    let centre = g(P2::new(u0 + du * (i as f64 + 0.5), v0 + dv * (j as f64 + 0.5)));
                    if pos(centre) == sg[0] {
                        segs.push((keys[0], keys[1]));
                        segs.push((keys[2], keys[3]));
                    } else {
                        segs.push((keys[3], keys[0]));
                        segs.push((keys[1], keys[2]));
                    }
                }
                _ => {}
            }
        }
    }
    if segs.is_empty() {
        return vec![];
    }
    // The exact zero on a grid edge.
    let point = |key: usize| -> P2 {
        let nd = key / 2;
        let (i, j) = (nd % (nu + 1), nd / (nu + 1));
        let (a, b) = if key.is_multiple_of(2) { (uv(i, j), uv(i + 1, j)) } else { (uv(i, j), uv(i, j + 1)) };
        let (mut lo, mut hi) = (a, b);
        let slo = pos(g(lo));
        for _ in 0..40 {
            let m = nalgebra::center(&lo, &hi);
            if pos(g(m)) == slo {
                lo = m;
            } else {
                hi = m;
            }
        }
        nalgebra::center(&lo, &hi)
    };
    // Chain the segments, open chains from their ends first.
    let mut at: HashMap<usize, Vec<usize>> = HashMap::new();
    for (k, &(a, b)) in segs.iter().enumerate() {
        at.entry(a).or_default().push(k);
        at.entry(b).or_default().push(k);
    }
    let mut keys: Vec<usize> = at.keys().copied().collect();
    keys.sort_by_key(|k| (at[k].len() != 1, *k));
    let mut used = vec![false; segs.len()];
    let mut chains: Vec<Vec<usize>> = Vec::new();
    for start in keys {
        while let Some(&first) = at[&start].iter().find(|&&k| !used[k]) {
            let mut chain = vec![start];
            let mut cur = start;
            let mut next = Some(first);
            while let Some(k) = next {
                used[k] = true;
                let (a, b) = segs[k];
                cur = if a == cur { b } else { a };
                chain.push(cur);
                next = at[&cur].iter().copied().find(|&k| !used[k]);
            }
            chains.push(chain);
        }
    }
    // Clip each chain to the face.
    let mut out = Vec::new();
    for chain in chains {
        let pts: Vec<P2> = chain.iter().map(|&k| point(k)).collect();
        let inside: Vec<bool> = pts.iter().map(|q| dom.contains(q.x, q.y, false)).collect();
        let cut = |a: P2, b: P2| -> P2 {
            let (mut lo, mut hi) = (a, b);
            let s = dom.contains(a.x, a.y, false);
            for _ in 0..30 {
                let m = nalgebra::center(&lo, &hi);
                if dom.contains(m.x, m.y, false) == s {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            nalgebra::center(&lo, &hi)
        };
        let mut run: Vec<P2> = Vec::new();
        for k in 0..pts.len() {
            if inside[k] {
                if run.is_empty() && k > 0 {
                    run.push(cut(pts[k], pts[k - 1]));
                }
                run.push(pts[k]);
            } else if !run.is_empty() {
                run.push(cut(pts[k - 1], pts[k]));
                out.push(std::mem::take(&mut run));
            }
        }
        if !run.is_empty() {
            out.push(run);
        }
    }
    out.into_iter().filter(|r| r.len() >= 2).map(|r| r.iter().map(|q| s.eval(q.x, q.y)).collect()).collect()
}

/// The outlines of one face as lines of the view.
fn face_outlines(frame: &ViewFrame, bi: usize, fi: usize, dom: &Domain, tol: f64) -> Vec<Src> {
    let dir = frame.dir;
    let s = dom.surf;
    if dom.is_empty() {
        return vec![];
    }
    let mk = |geo: Geo, t: (f64, f64), closed: bool| make_src(frame, bi, ProjClass::Outline, None, Some(fi), vec![fi], geo, t, closed, tol);
    match s {
        Surface::Plane { .. } => vec![],
        Surface::Cylinder { .. } | Surface::Cone { .. } => {
            // n·dir = A cos u + B sin u + C along u (the normal is constant along a generator).
            let vm = 0.5 * (dom.vb.0 + dom.vb.1);
            let g = |u: f64| s.normal(u, vm).dot(&dir);
            let (g0, g1, g2) = (g(0.0), g(PI / 2.0), g(PI));
            let c = 0.5 * (g0 + g2);
            let (a, b) = (0.5 * (g0 - g2), g1 - c);
            let r = a.hypot(b);
            if r < 1e-9 || c.abs() > r {
                return vec![];
            }
            let phi = b.atan2(a);
            let alpha = (-c / r).clamp(-1.0, 1.0).acos();
            let mut us = vec![phi + alpha];
            if alpha > 1e-9 {
                us.push(phi - alpha);
            }
            let mut out = Vec::new();
            for u0 in us {
                let Some(line) = s.iso_u(u0) else { continue };
                for run in clip(dom.vb.0, dom.vb.1, 32, |v| dom.contains(u0, v, false)) {
                    out.extend(mk(Geo::Exact(line.clone()), run, false));
                }
            }
            out
        }
        Surface::Sphere { f, r } => {
            let a = any_perpendicular(&dir);
            let circle = Curve::Circle { c: f.o, x: a, y: dir.cross(&a).normalize(), r: *r };
            let inside = |t: f64| {
                let (u, v) = s.project(&circle.point(t), None);
                dom.contains(u, v, false)
            };
            let mut runs = clip(0.0, TAU, 64, inside);
            if runs.len() >= 2 && runs[0].0 == 0.0 && runs[runs.len() - 1].1 == TAU {
                let last = runs.pop().unwrap();
                runs[0] = (last.0, runs[0].1 + TAU);
            }
            let full = runs.len() == 1 && runs[0] == (0.0, TAU);
            runs.into_iter().filter_map(|run| mk(Geo::Exact(circle.clone()), run, full)).collect()
        }
        _ => trace_outline(dom, &dir)
            .into_iter()
            .filter_map(|p| {
                let n = (p.len() - 1) as f64;
                mk(Geo::Poly(p), (0.0, n), false)
            })
            .collect(),
    }
}

/// A uniform 2D grid of items by their boxes.
struct Grid {
    lo: P2,
    cell: f64,
    nx: usize,
    ny: usize,
    cells: Vec<Vec<u32>>,
}

impl Grid {
    fn new(lo: P2, hi: P2, items: usize) -> Self {
        let (w, h) = ((hi.x - lo.x).max(1e-9), (hi.y - lo.y).max(1e-9));
        let k = ((items as f64).sqrt().ceil() as usize).clamp(1, 256);
        let cell = (w.max(h) / k as f64).max(1e-9);
        let nx = ((w / cell).ceil() as usize).clamp(1, 512);
        let ny = ((h / cell).ceil() as usize).clamp(1, 512);
        let cell = (w / nx as f64).max(h / ny as f64);
        Self { lo, cell, nx, ny, cells: vec![Vec::new(); nx * ny] }
    }

    fn ij(&self, p: P2) -> (usize, usize) {
        let f = |x: f64, n: usize| ((x / self.cell).floor().max(0.0) as usize).min(n - 1);
        (f(p.x - self.lo.x, self.nx), f(p.y - self.lo.y, self.ny))
    }

    fn insert(&mut self, id: u32, min: P2, max: P2) {
        let (i0, j0) = self.ij(min);
        let (i1, j1) = self.ij(max);
        for j in j0..=j1 {
            for i in i0..=i1 {
                self.cells[j * self.nx + i].push(id);
            }
        }
    }

    fn at(&self, p: P2) -> &[u32] {
        let (i, j) = self.ij(p);
        &self.cells[j * self.nx + i]
    }
}

fn bounds(pts: impl IntoIterator<Item = P2>) -> (P2, P2) {
    let mut lo = P2::new(f64::INFINITY, f64::INFINITY);
    let mut hi = P2::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in pts {
        lo = lo.inf(&p);
        hi = hi.sup(&p);
    }
    (lo, hi)
}

fn inside_box(q: &P2, lo: &P2, hi: &P2) -> bool {
    q.x >= lo.x && q.y >= lo.y && q.x <= hi.x && q.y <= hi.y
}

/// Something that can hide a line: a face hit exactly, or a triangle of a face's mesh.
enum Occ {
    Face,
    Tri([P3; 3]),
}

/// An occluder of a face (body, face) with its 2D box.
type OccItem = (usize, usize, Occ, P2, P2);

/// Every face of every body, for rays towards the eye.
struct Occluders<'a> {
    items: Vec<OccItem>,
    grid: Grid,
    doms: &'a [Vec<Domain<'a>>],
    dir: V3,
    frame: ViewFrame,
    /// Triangle hits nearer than this don't hide (mesh deviation); `skin_own` for the line's
    /// own faces.
    skin_tri: f64,
    skin_own: f64,
}

impl<'a> Occluders<'a> {
    fn new(bodies: &[HlrBody], doms: &'a [Vec<Domain<'a>>], frame: &ViewFrame, defl: f64, tol: f64) -> Self {
        let dir = frame.dir;
        let per_body: Vec<Vec<OccItem>> = bodies
            .par_iter()
            .enumerate()
            .map(|(bi, hb)| {
                let b = hb.body;
                let (meshes, _) = tess::mesh_body(b, defl, 0.2);
                let mut out = Vec::new();
                for (fi, (f, m)) in b.faces.iter().zip(&meshes).enumerate() {
                    if m.positions.is_empty() {
                        continue;
                    }
                    let exact = match &f.surf {
                        Surface::Plane { f: fr } => {
                            if fr.x.cross(&fr.y).normalize().dot(&dir).abs() < 1e-9 {
                                continue;
                            }
                            true
                        }
                        Surface::Cylinder { f: fr, .. } => {
                            if fr.z.cross(&dir).norm() < 1e-9 {
                                continue;
                            }
                            true
                        }
                        Surface::Cone { .. } | Surface::Sphere { .. } => true,
                        _ => false,
                    };
                    if exact {
                        let (lo, hi) = bounds(m.positions.iter().map(|p| frame.to_2d(p)));
                        let g = V2::repeat(defl + tol);
                        out.push((bi, fi, Occ::Face, lo - g, hi + g));
                    } else {
                        for t in &m.indices {
                            let tri = t.map(|i| m.positions[i as usize]);
                            let n = (tri[1] - tri[0]).cross(&(tri[2] - tri[0]));
                            let l = n.norm();
                            if l < 1e-14 || n.dot(&dir).abs() < 1e-6 * l {
                                continue;
                            }
                            let (lo, hi) = bounds(tri.iter().map(|p| frame.to_2d(p)));
                            out.push((bi, fi, Occ::Tri(tri), lo, hi));
                        }
                    }
                }
                out
            })
            .collect();
        let items: Vec<_> = per_body.into_iter().flatten().collect();
        let (lo, hi) = bounds(items.iter().flat_map(|it| [it.3, it.4]));
        let mut grid = Grid::new(if items.is_empty() { P2::origin() } else { lo }, if items.is_empty() { P2::origin() } else { hi }, items.len());
        for (k, it) in items.iter().enumerate() {
            grid.insert(k as u32, it.3, it.4);
        }
        Self { items, grid, doms, dir, frame: *frame, skin_tri: 2.0 * defl + tol, skin_own: 20.0 * defl + tol }
    }

    /// Whether a ray from `p` (on body `body`'s faces `own`) towards the eye leaves without
    /// meeting a face.
    fn visible(&self, p: &P3, body: usize, own: &[usize]) -> bool {
        let q = self.frame.to_2d(p);
        let rd = -self.dir;
        // Hits on face boundaries: two faces met at one point there means the ray runs through
        // their common edge (out of the material, as along a tangent edge): hidden.
        let mut rim: Vec<(f64, usize, usize)> = Vec::new();
        for &k in self.grid.at(q) {
            let (bi, fi, occ, lo, hi) = &self.items[k as usize];
            if !inside_box(&q, lo, hi) {
                continue;
            }
            match occ {
                Occ::Face => {
                    let dom = &self.doms[*bi][*fi];
                    for (t, u, v) in ray_surface(dom.surf, p, &rd, -1e9, 1e9) {
                        if t > 1e-5 && dom.contains(u, v, true) {
                            return false;
                        }
                        if t > 1e-5 && dom.contains(u, v, false) {
                            if rim.iter().any(|&(t2, b2, f2)| (t2 - t).abs() < 1e-6 && (b2, f2) != (*bi, *fi)) {
                                return false;
                            }
                            rim.push((t, *bi, *fi));
                        }
                    }
                }
                Occ::Tri(t) => {
                    let skin = if *bi == body && own.contains(fi) { self.skin_own } else { self.skin_tri };
                    let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
                    let h = rd.cross(&e2);
                    let det = e1.dot(&h);
                    if det.abs() < 1e-15 {
                        continue;
                    }
                    let f = 1.0 / det;
                    let sv = p - t[0];
                    let u = f * sv.dot(&h);
                    if !(-1e-9..=1.0 + 1e-9).contains(&u) {
                        continue;
                    }
                    let qv = sv.cross(&e1);
                    let v = f * rd.dot(&qv);
                    if v < -1e-9 || u + v > 1.0 + 1e-9 {
                        continue;
                    }
                    if f * e2.dot(&qv) > skin {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Where two 2D segments meet: fractions along each (`None` where only the other is split, at
/// a collinear overlap's end).
fn seg_meet(a0: P2, a1: P2, b0: P2, b1: P2) -> Vec<(Option<f64>, Option<f64>, bool)> {
    let (r, s) = (a1 - a0, b1 - b0);
    let (rl, sl) = (r.norm(), s.norm());
    if rl < 1e-15 || sl < 1e-15 {
        return vec![];
    }
    let den = cross2(&r, &s);
    let w = b0 - a0;
    if den.abs() > 1e-9 * rl * sl {
        let fa = cross2(&w, &s) / den;
        let fb = cross2(&w, &r) / den;
        let (ea, eb) = (EPS2 / rl, EPS2 / sl);
        if fa >= -ea && fa <= 1.0 + ea && fb >= -eb && fb <= 1.0 + eb {
            return vec![(Some(fa.clamp(0.0, 1.0)), Some(fb.clamp(0.0, 1.0)), true)];
        }
        return vec![];
    }
    // Parallel: collinear overlaps split each segment at the other's ends.
    if cross2(&r, &w).abs() / rl > EPS2 {
        return vec![];
    }
    let mut out = Vec::new();
    for p in [b0, b1] {
        let f = (p - a0).dot(&r) / (rl * rl);
        if f > 1e-9 && f < 1.0 - 1e-9 {
            out.push((Some(f), None, false));
        }
    }
    for p in [a0, a1] {
        let f = (p - b0).dot(&s) / (sl * sl);
        if f > 1e-9 && f < 1.0 - 1e-9 {
            out.push((None, Some(f), false));
        }
    }
    out
}

/// Newton on P_a(ta) = P_b(tb) in 2D for a crossing of two exact curves.
fn refine(frame: &ViewFrame, a: &Src, ta: f64, b: &Src, tb: f64) -> Option<(f64, f64)> {
    let (mut ta, mut tb) = (ta, tb);
    for _ in 0..8 {
        let f = frame.to_2d(&a.point(ta)) - frame.to_2d(&b.point(tb));
        if f.norm() < 1e-12 {
            break;
        }
        let (ja, jb) = (frame.dir_2d(&a.deriv(ta)), frame.dir_2d(&b.deriv(tb)));
        let (da, db) = solve2(ja.x, -jb.x, ja.y, -jb.y, -f.x, -f.y)?;
        ta += da;
        tb += db;
    }
    let f = frame.to_2d(&a.point(ta)) - frame.to_2d(&b.point(tb));
    (f.norm() < 1e-9 && ta >= a.t.0 && ta <= a.t.1 && tb >= b.t.0 && tb <= b.t.1).then_some((ta, tb))
}

/// The split parameters of every line where it meets another in 2D.
fn crossings(frame: &ViewFrame, srcs: &[Src]) -> Vec<Vec<f64>> {
    let segs: Vec<(u32, u32)> = srcs.iter().enumerate().flat_map(|(s, src)| (0..src.p2.len() - 1).map(move |k| (s as u32, k as u32))).collect();
    let mut out = vec![Vec::new(); srcs.len()];
    if segs.is_empty() {
        return out;
    }
    let seg = |&(s, k): &(u32, u32)| (srcs[s as usize].p2[k as usize], srcs[s as usize].p2[k as usize + 1]);
    let boxes: Vec<(P2, P2)> = segs.iter().map(|sk| {
        let (a, b) = seg(sk);
        let g = V2::repeat(EPS2);
        (a.inf(&b) - g, a.sup(&b) + g)
    }).collect();
    let (lo, hi) = bounds(boxes.iter().flat_map(|b| [b.0, b.1]));
    let mut grid = Grid::new(lo, hi, segs.len());
    for (k, b) in boxes.iter().enumerate() {
        grid.insert(k as u32, b.0, b.1);
    }
    let param = |s: usize, k: usize, f: f64| {
        let ts = &srcs[s].ts;
        ts[k] + (ts[k + 1] - ts[k]) * f
    };
    let found: Vec<(usize, f64)> = (0..grid.cells.len())
        .into_par_iter()
        .flat_map_iter(|c| {
            let (ci, cj) = (c % grid.nx, c / grid.nx);
            let cell = &grid.cells[c];
            let mut found = Vec::new();
            for x in 0..cell.len() {
                for y in x + 1..cell.len() {
                    let (i, j) = (cell[x] as usize, cell[y] as usize);
                    let ((si, ki), (sj, kj)) = (segs[i], segs[j]);
                    let (si, ki, sj, kj) = (si as usize, ki as usize, sj as usize, kj as usize);
                    if si == sj {
                        let n = srcs[si].p2.len() - 1;
                        let d = ki.abs_diff(kj);
                        if d <= 1 || (srcs[si].closed && d == n - 1) {
                            continue;
                        }
                    }
                    let (bi, bj) = (boxes[i], boxes[j]);
                    if bi.0.x > bj.1.x || bj.0.x > bi.1.x || bi.0.y > bj.1.y || bj.0.y > bi.1.y {
                        continue;
                    }
                    // Each pair once: in the cell holding the low corner of the boxes' overlap.
                    if grid.ij(bi.0.sup(&bj.0)) != (ci, cj) {
                        continue;
                    }
                    let (a0, a1) = seg(&segs[i]);
                    let (b0, b1) = seg(&segs[j]);
                    for (fa, fb, proper) in seg_meet(a0, a1, b0, b1) {
                        let (a, b) = (&srcs[si], &srcs[sj]);
                        let mut ta = fa.map(|f| param(si, ki, f));
                        let mut tb = fb.map(|f| param(sj, kj, f));
                        if let (true, Some(x), Some(y)) = (proper, ta, tb) {
                            let both_lines = matches!((a.shape, b.shape), (Shape::Line, Shape::Line));
                            if a.is_exact() && b.is_exact() && !both_lines
                                && let Some((x, y)) = refine(frame, a, x, b, y)
                            {
                                ta = Some(x);
                                tb = Some(y);
                            }
                        }
                        if let Some(t) = ta {
                            found.push((si, t));
                        }
                        if let Some(t) = tb {
                            found.push((sj, t));
                        }
                    }
                }
            }
            found
        })
        .collect();
    for (s, t) in found {
        out[s].push(t);
    }
    out
}

/// A piece of a line between two parameters, and its visibility.
struct Piece {
    src: usize,
    a: f64,
    b: f64,
    visible: bool,
    /// The line turns back at `b`: never merged with the next piece.
    turn_end: bool,
}

/// Splits a piece where its visibility changes (tested at three points, then bisected).
fn piece_visibility(frame: &ViewFrame, occ: &Occluders, src: &Src, a: f64, b: f64, tol: f64) -> Vec<(f64, f64, bool)> {
    let vis = |t: f64| occ.visible(&src.point(t), src.body, &src.own);
    let at = |f: f64| a + (b - a) * f;
    let len = (frame.to_2d(&src.point(a)) - frame.to_2d(&src.point(b))).norm();
    let mid = vis(at(0.5));
    if len < 20.0 * tol || (vis(at(0.15)) == mid && vis(at(0.85)) == mid) {
        return vec![(a, b, mid)];
    }
    let n = 16;
    let fs: Vec<f64> = (0..n).map(|k| (k as f64 + 0.5) / n as f64).collect();
    let st: Vec<bool> = fs.iter().map(|&f| vis(at(f))).collect();
    let mut out = Vec::new();
    let mut start = a;
    for k in 1..n {
        if st[k] != st[k - 1] {
            let (mut lo, mut hi) = (at(fs[k - 1]), at(fs[k]));
            for _ in 0..30 {
                let m = 0.5 * (lo + hi);
                if vis(m) == st[k - 1] {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let c = 0.5 * (lo + hi);
            out.push((start, c, st[k - 1]));
            start = c;
        }
    }
    out.push((start, b, st[n - 1]));
    out
}

/// Projects the bodies along `frame` with hidden-line removal (see the module docs).
pub fn project(bodies: &[HlrBody], frame: &ViewFrame, opts: &ProjectOptions) -> Projection {
    let tol = opts.tolerance.max(1e-4);
    let defl = (2.0 * tol).clamp(0.005, 0.05);
    let doms: Vec<Vec<Domain>> = bodies.iter().map(|hb| hb.body.faces.iter().zip(hb.polys).map(|(f, p)| Domain::new(&f.surf, p)).collect()).collect();
    // The lines: edges and outlines.
    let mut srcs: Vec<Src> = Vec::new();
    for (bi, hb) in bodies.iter().enumerate() {
        srcs.extend(edge_srcs(frame, hb.body, bi, tol));
        let outl: Vec<Src> = (0..hb.body.faces.len()).into_par_iter().flat_map_iter(|fi| face_outlines(frame, bi, fi, &doms[bi][fi], tol)).collect();
        srcs.extend(outl);
    }
    if srcs.is_empty() {
        return Projection::default();
    }
    let occ = Occluders::new(bodies, &doms, frame, defl, tol);
    // Split at crossings and turning points.
    let splits = crossings(frame, &srcs);
    let p2 = |s: &Src, t: f64| frame.to_2d(&s.point(t));
    let pieces: Vec<Piece> = srcs
        .par_iter()
        .zip(splits)
        .enumerate()
        .flat_map_iter(|(si, (s, mut sp))| {
            let mut cuts: Vec<(f64, bool)> = sp.drain(..).map(|t| (t, false)).chain(s.turns.iter().map(|&t| (t, true))).collect();
            cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut bps: Vec<(f64, bool)> = vec![(s.t.0, false)];
            let end = p2(s, s.t.1);
            for (t, turn) in cuts {
                let first = bps.len() == 1;
                let last = bps.last_mut().unwrap();
                let q = p2(s, t);
                if t <= last.0 || (q - p2(s, last.0)).norm() < EPS2 {
                    last.1 |= turn && !first;
                    continue;
                }
                if t >= s.t.1 || (q - end).norm() < EPS2 {
                    continue;
                }
                bps.push((t, turn));
            }
            bps.push((s.t.1, false));
            let mut out = Vec::new();
            for w in bps.windows(2) {
                let parts = piece_visibility(frame, &occ, s, w[0].0, w[1].0, tol);
                let n = parts.len();
                for (k, (a, b, visible)) in parts.into_iter().enumerate() {
                    out.push(Piece { src: si, a, b, visible, turn_end: k + 1 == n && w[1].1 });
                }
            }
            out
        })
        .collect();
    // Polylines of the pieces, for the overlap test.
    let polys: Vec<(Vec<P2>, Vec<f64>)> = pieces
        .par_iter()
        .map(|pc| {
            let s = &srcs[pc.src];
            let ts = s.params(pc.a, pc.b, tol);
            let pts: Vec<P3> = ts.iter().map(|&t| s.point(t)).collect();
            (pts.iter().map(|p| frame.to_2d(p)).collect(), pts.iter().map(|p| frame.depth(p)).collect())
        })
        .collect();
    let cov = 2.0 * tol + 1e-6;
    let segs: Vec<(u32, u32)> = polys.iter().enumerate().flat_map(|(i, (p, _))| (0..p.len() - 1).map(move |k| (i as u32, k as u32))).collect();
    let (lo, hi) = bounds(polys.iter().flat_map(|(p, _)| p.iter().copied()));
    let mut grid = Grid::new(lo - V2::repeat(cov), hi + V2::repeat(cov), segs.len());
    for (k, &(i, j)) in segs.iter().enumerate() {
        let p = &polys[i as usize].0;
        let (a, b) = (p[j as usize], p[j as usize + 1]);
        grid.insert(k as u32, a.inf(&b) - V2::repeat(cov), a.sup(&b) + V2::repeat(cov));
    }
    let rank = |c: ProjClass| u8::from(c == ProjClass::Outline);
    // A piece lying along a visible one (hidden), or along a nearer one of its own visibility.
    let dropped: Vec<bool> = (0..pieces.len())
        .into_par_iter()
        .map(|pi| {
            let pc = &pieces[pi];
            let s = &srcs[pc.src];
            [0.25, 0.5, 0.75].iter().all(|f| {
                let t = pc.a + (pc.b - pc.a) * f;
                let p = s.point(t);
                let (q, d) = (frame.to_2d(&p), frame.depth(&p));
                let tg = frame.dir_2d(&s.deriv(t));
                if tg.norm() < 1e-12 {
                    return false;
                }
                let tg = tg.normalize();
                grid.at(q).iter().any(|&k| {
                    let (oi, j) = segs[k as usize];
                    let (oi, j) = (oi as usize, j as usize);
                    if oi == pi {
                        return false;
                    }
                    let o = &pieces[oi];
                    if !o.visible && pc.visible {
                        return false;
                    }
                    let (pts, dep) = &polys[oi];
                    let (a, b) = (pts[j], pts[j + 1]);
                    let ab = b - a;
                    let l = ab.norm();
                    if l < 1e-12 || cross2(&ab, &tg).abs() > PARALLEL * l {
                        return false;
                    }
                    // Beside the segment, not past its end (a line ending short of `q` leaves
                    // it bare).
                    let f = (q - a).dot(&ab) / (l * l);
                    if !(-1e-9..=1.0 + 1e-9).contains(&f) {
                        return false;
                    }
                    let f = f.clamp(0.0, 1.0);
                    if (a + ab * f - q).norm() > cov {
                        return false;
                    }
                    // A hidden silhouette stays even under a visible line (as the OCCT backend's
                    // post-processing kept them); hidden edges there are dropped.
                    if o.visible && !pc.visible {
                        return s.class != ProjClass::Outline;
                    }
                    let od = dep[j] + (dep[j + 1] - dep[j]) * f;
                    if (od - d).abs() > 1e-6 {
                        return od < d;
                    }
                    (rank(srcs[o.src].class), oi) < (rank(s.class), pi)
                })
            })
        })
        .collect();
    // Merge consecutive kept pieces of a line with the same visibility.
    let mut runs: Vec<(usize, f64, f64, bool, bool)> = Vec::new(); // src, a, b, visible, full
    let mut k = 0;
    while k < pieces.len() {
        let si = pieces[k].src;
        let s = &srcs[si];
        let first = runs.len();
        let mut prev: Option<usize> = None;
        while k < pieces.len() && pieces[k].src == si {
            let pc = &pieces[k];
            if !dropped[k] {
                let joins = prev.is_some_and(|p| !dropped[p] && pieces[p].visible == pc.visible && !pieces[p].turn_end && k == p + 1);
                if joins {
                    runs.last_mut().unwrap().2 = pc.b;
                } else {
                    runs.push((si, pc.a, pc.b, pc.visible, false));
                }
            }
            prev = Some(k);
            k += 1;
        }
        if s.closed && runs.len() > first {
            let n = runs.len();
            let (r0, rl) = (runs[first], runs[n - 1]);
            let period = s.t.1 - s.t.0;
            if n - first == 1 {
                if r0.1 == s.t.0 && r0.2 == s.t.1 {
                    runs[first].4 = true;
                }
            } else if r0.1 == s.t.0 && rl.2 == s.t.1 && r0.3 == rl.3 && !s.turn_at_start {
                runs[first] = (si, rl.1, r0.2 + period, r0.3, false);
                runs.pop();
            }
        }
    }
    let edges: Vec<ProjEdge> = runs
        .par_iter()
        .filter(|r| r.3 || opts.hidden)
        .map(|&(si, a, b, visible, full)| {
            let s = &srcs[si];
            let points: Vec<P2> = s.params(a, b, tol).iter().map(|&t| p2(s, t)).collect();
            let (start, end) = (points[0], points[points.len() - 1]);
            let (curve, points) = match s.shape {
                Shape::Line => (ProjCurve::Line { start, end }, vec![start, end]),
                Shape::Arc { center, radius } => (ProjCurve::Arc { center, radius, start, mid: p2(s, 0.5 * (a + b)), end, full }, points),
                Shape::Other => (ProjCurve::Polyline, points),
            };
            ProjEdge {
                visibility: if visible { ProjVisibility::Visible } else { ProjVisibility::Hidden },
                class: s.class,
                curve,
                points,
                source: Some(ProjSource {
                    body: s.body,
                    edge: s.edge.map(|e| EdgeId(e as u64)),
                    face: s.face.map(|f| FaceId(f as u64)),
                    ..Default::default()
                }),
            }
        })
        .collect();
    Projection { edges }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Point2, Point3, Vector3};

    use crate::*;
    use std::f64::consts::{PI, TAU};

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
        let p = [Point2::new(x0, y0), Point2::new(x1, y0), Point2::new(x1, y1), Point2::new(x0, y1)];
        Loop { curves: (0..4).map(|i| Curve2::Line { a: p[i], b: p[(i + 1) % 4], source: None }).collect() }
    }

    fn extrude(k: &mut dyn Kernel, outer: Loop, z0: f64, h: f64) -> BodyId {
        let profile = Profile::new(Plane { origin: Point3::new(0.0, 0.0, z0), ..Plane::top() }, vec![Region { outer, holes: vec![], source: None }]);
        k.extrude(&profile, Extent::Blind(h)).unwrap().bodies[0]
    }

    fn front() -> ViewFrame {
        ViewFrame::new(Vector3::y(), Vector3::x())
    }

    fn count(p: &Projection, v: ProjVisibility, c: ProjClass) -> usize {
        p.of(v, c).count()
    }

    #[test]
    fn box_from_the_front() {
        let mut k = native::NativeKernel::new();
        let b = extrude(&mut k, rect(0.0, 0.0, 10.0, 20.0), 0.0, 30.0);
        let p = k.project(&[b], &front(), &ProjectOptions::default()).unwrap();
        assert_eq!(count(&p, ProjVisibility::Visible, ProjClass::Sharp), 4, "{p:#?}");
        assert!(p.edges.iter().all(|e| e.visibility == ProjVisibility::Visible && matches!(e.curve, ProjCurve::Line { .. })), "{p:#?}");
        let len: f64 = p.edges.iter().map(|e| e.length()).sum();
        assert!((len - 80.0).abs() < 1e-9, "{len}");
        // Each visible edge is one of the front face's (y = 0).
        let infos = k.edges(b).unwrap();
        for e in &p.edges {
            let id = e.source.unwrap().edge.unwrap();
            assert!(infos[id.0 as usize].mid.y.abs() < 1e-9);
        }
    }

    #[test]
    fn cylinder_from_the_side() {
        let mut k = native::NativeKernel::new();
        let disc = Loop { curves: vec![Curve2::Circle { center: Point2::new(0.0, 0.0), radius: 5.0, source: None }] };
        let c = extrude(&mut k, disc, 0.0, 20.0);
        let p = k.project(&[c], &front(), &ProjectOptions::default()).unwrap();
        let outlines: Vec<&ProjEdge> = p.of(ProjVisibility::Visible, ProjClass::Outline).collect();
        assert_eq!(outlines.len(), 2, "{p:#?}");
        let mut xs: Vec<f64> = outlines.iter().map(|e| e.points[0].x).collect();
        xs.sort_by(f64::total_cmp);
        assert!((xs[0] + 5.0).abs() < 1e-9 && (xs[1] - 5.0).abs() < 1e-9, "{xs:?}");
        for e in &outlines {
            assert!((e.length() - 20.0).abs() < 1e-9);
            assert!(e.source.unwrap().face.is_some());
        }
        // The two circles seen edge-on: lines across, the far halves dropped behind the near.
        let sharp: Vec<&ProjEdge> = p.of(ProjVisibility::Visible, ProjClass::Sharp).collect();
        assert_eq!(sharp.len(), 2, "{p:#?}");
        for e in &sharp {
            assert!(matches!(e.curve, ProjCurve::Line { .. }));
            assert!((e.length() - 10.0).abs() < 1e-6, "{}", e.length());
        }
        assert!(!p.edges.iter().any(|e| e.visibility == ProjVisibility::Hidden), "{p:#?}");
        // From above: one full circle, no outline.
        let top = k.project(&[c], &ViewFrame::new(-Vector3::z(), Vector3::x()), &ProjectOptions::default()).unwrap();
        assert_eq!(top.edges.len(), 1, "{top:#?}");
        assert!(matches!(top.edges[0].curve, ProjCurve::Arc { full: true, .. }));
    }

    #[test]
    fn a_box_behind_another_is_hidden() {
        let mut k = native::NativeKernel::new();
        let near = extrude(&mut k, rect(0.0, 0.0, 20.0, 20.0), 0.0, 20.0);
        let far = extrude(&mut k, rect(5.0, 30.0, 15.0, 40.0), 5.0, 10.0);
        let p = k.project(&[near, far], &front(), &ProjectOptions::default()).unwrap();
        let of = |b: usize, v: ProjVisibility| p.edges.iter().filter(|e| e.visibility == v && e.source.unwrap().body == b).count();
        assert_eq!(of(1, ProjVisibility::Visible), 0, "{p:#?}");
        // Its front square (the back one lies right behind it).
        assert_eq!(of(1, ProjVisibility::Hidden), 4, "{p:#?}");
        assert_eq!(of(0, ProjVisibility::Visible), 4, "{p:#?}");
        // Seen from behind, the small box is in front and nothing of the big one is hidden.
        let back = k.project(&[near, far], &ViewFrame::new(-Vector3::y(), -Vector3::x()), &ProjectOptions::default()).unwrap();
        assert_eq!(back.edges.iter().filter(|e| e.visibility == ProjVisibility::Visible && e.source.unwrap().body == 1).count(), 4, "{back:#?}");
        assert!(!back.edges.iter().any(|e| e.visibility == ProjVisibility::Hidden && e.source.unwrap().body == 0), "{back:#?}");
    }

    /// The XZ plane (in-plane y along +Z).
    fn xz() -> Plane {
        Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() }
    }

    fn z_axis() -> Axis {
        Axis { origin: Point3::origin(), dir: Vector3::z_axis() }
    }

    #[test]
    fn sphere_and_torus_outlines() {
        let mut k = native::NativeKernel::new();
        // A ball of radius 6: a half disc turned about Z. Its outline from anywhere is a circle.
        let half = Loop {
            curves: vec![
                Curve2::Arc { center: Point2::new(0.0, 0.0), radius: 6.0, start_angle: -PI / 2.0, sweep: PI, source: None },
                Curve2::Line { a: Point2::new(0.0, 6.0), b: Point2::new(0.0, -6.0), source: None },
            ],
        };
        let ball = k.revolve(&Profile::new(xz(), vec![Region { outer: half, holes: vec![], source: None }]), z_axis(), TAU).unwrap().bodies[0];
        let view = ViewFrame::new(Vector3::new(1.0, 2.0, -0.5), Vector3::z());
        let p = k.project(&[ball], &view, &ProjectOptions::default()).unwrap();
        let arcs: Vec<&ProjEdge> = p.of(ProjVisibility::Visible, ProjClass::Outline).collect();
        let len: f64 = arcs.iter().map(|e| e.length()).sum();
        assert!((len - TAU * 6.0).abs() < 0.05, "{len} {p:#?}");
        for e in &arcs {
            let ProjCurve::Arc { radius, center, .. } = e.curve else { panic!("{e:?}") };
            assert!((radius - 6.0).abs() < 1e-9 && center.coords.norm() < 1e-9);
        }
        // A torus R10 r3 from the front: its outline spans x −13..13, z −3..3.
        let ring = Loop { curves: vec![Curve2::Circle { center: Point2::new(10.0, 0.0), radius: 3.0, source: None }] };
        let torus = k.revolve(&Profile::new(xz(), vec![Region { outer: ring, holes: vec![], source: None }]), z_axis(), TAU).unwrap().bodies[0];
        let p = k.project(&[torus], &front(), &ProjectOptions::default()).unwrap();
        assert!(p.of(ProjVisibility::Visible, ProjClass::Outline).count() > 0, "{p:#?}");
        let (lo, hi) = p.bounds().unwrap();
        assert!((lo.x + 13.0).abs() < 0.02 && (hi.x - 13.0).abs() < 0.02, "{lo} {hi}");
        assert!((lo.y + 3.0).abs() < 0.02 && (hi.y - 3.0).abs() < 0.02, "{lo} {hi}");
        // From above: the outer and inner circles (r 13 and 7) are outlines.
        let p = k.project(&[torus], &ViewFrame::new(-Vector3::z(), Vector3::x()), &ProjectOptions::default()).unwrap();
        let (lo, hi) = p.bounds().unwrap();
        assert!((hi.x - 13.0).abs() < 0.02 && (lo.y + 13.0).abs() < 0.02, "{lo} {hi}");
        assert!(!p.edges.iter().any(|e| e.visibility == ProjVisibility::Hidden), "{p:#?}");
    }

    /// A 10 × 10 grid of holes in a plate, seen from the front and from a corner.
    #[test]
    fn many_holes_quickly() {
        let mut k = native::NativeKernel::new();
        let holes: Vec<Loop> = (0..100)
            .map(|i| Loop { curves: vec![Curve2::Circle { center: Point2::new(5.0 + 10.0 * (i % 10) as f64, 5.0 + 10.0 * (i / 10) as f64), radius: 3.0, source: None }] })
            .collect();
        let profile = Profile::new(Plane::top(), vec![Region { outer: rect(0.0, 0.0, 100.0, 100.0), holes, source: None }]);
        let b = k.extrude(&profile, Extent::Blind(10.0)).unwrap().bodies[0];
        let t = std::time::Instant::now();
        let p = k.project(&[b], &front(), &ProjectOptions::default()).unwrap();
        let iso = k.project(&[b], &ViewFrame::new(Vector3::new(-1.0, 1.0, -1.0), Vector3::new(1.0, 1.0, 0.0)), &ProjectOptions::default()).unwrap();
        let dt = t.elapsed();
        eprintln!("100 holes: {} + {} edges in {dt:?}", p.edges.len(), iso.edges.len());
        // Front: 10 columns of hidden hole sides (two each), the plate's outline visible.
        assert_eq!(p.of(ProjVisibility::Hidden, ProjClass::Outline).count(), 20, "{p:#?}");
        let len: f64 = p.edges.iter().filter(|e| e.visibility == ProjVisibility::Visible).map(|e| e.length()).sum();
        assert!((len - 220.0).abs() < 1e-6, "{len}");
        assert!(dt.as_secs_f64() < 5.0, "{dt:?}");
    }
}
