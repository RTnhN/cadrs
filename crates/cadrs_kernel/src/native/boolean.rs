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

use rayon::prelude::*;

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

/// Accumulated time per boolean phase (CADRS_PROFILE=1 prints it after each boolean).
struct Prof {
    t: std::time::Instant,
    on: bool,
    phases: Vec<(&'static str, std::time::Duration)>,
}

impl Prof {
    fn new() -> Self {
        Self { t: std::time::Instant::now(), on: std::env::var("CADRS_PROFILE").is_ok(), phases: Vec::new() }
    }
    fn mark(&mut self, name: &'static str) {
        if self.on {
            let now = std::time::Instant::now();
            self.phases.push((name, now - self.t));
            self.t = now;
        }
    }
    fn report(&self, what: &str) {
        if self.on {
            let total: std::time::Duration = self.phases.iter().map(|p| p.1).sum();
            let parts: Vec<String> = self.phases.iter().map(|(n, d)| format!("{n} {:.1}ms", d.as_secs_f64() * 1e3)).collect();
            eprintln!("[profile] {what}: {:.1}ms = {}", total.as_secs_f64() * 1e3, parts.join(", "));
        }
    }
}

/// The default distance under which two points of a boolean are one (mm; [`tol::POINT`]; a
/// run's is [`BoolOptions::tol`]'s).
pub const VTOL: f64 = super::tol::POINT;

/// How a boolean runs: its tolerances, and which faces on one surface are merged afterwards.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoolOptions {
    /// The merge distance of the first run and of the retries after one that fails.
    pub tol: super::tol::Tolerances,
    /// Faces of the two inputs on one surface are merged whatever the operation (a blend's
    /// tool face flush with a face it ends against becomes part of it); otherwise only a
    /// union merges them.
    pub flush: bool,
    /// Faces on one surface are merged after the run (true by default); OCCT's booleans (the
    /// DRAW harness's) keep them apart.
    pub merge_coplanar: bool,
    /// The most work a run may do before it fails ([`KernelError::BudgetExhausted`]): the
    /// surface intersections' cell pairs and traced points, each split face's arcs squared,
    /// and the classified and assembled faces (counts of the inputs, so the same run always
    /// spends the same). [`DEFAULT_WORK_BUDGET`] by default; a caller may pass less.
    pub work_budget: u64,
}

/// The default work budget of a boolean run ([`BoolOptions::work_budget`]): about three times
/// the most a passing case of the test suites spends (7.1 million units, an OCCT boolean case);
/// the runaway runs it stops spend 30 million and more over minutes.
pub const DEFAULT_WORK_BUDGET: u64 = 20_000_000;

impl Default for BoolOptions {
    fn default() -> Self {
        BoolOptions { tol: super::tol::Tolerances::default(), flush: false, merge_coplanar: true, work_budget: DEFAULT_WORK_BUDGET }
    }
}

/// What one run of the boolean takes from its [`BoolOptions`]: the merge distance (the first
/// run's or a retry's) and the rest as given.
#[derive(Debug, Clone, Copy)]
struct RunOpts {
    tol: f64,
    flush: bool,
    merge_coplanar: bool,
    work_budget: u64,
}

/// A run's work so far against its budget ([`BoolOptions::work_budget`]).
struct Work {
    done: u64,
    budget: u64,
}

impl Work {
    /// Counts `w` more; fails once the budget is spent.
    fn spend(&mut self, w: u64, stage: &'static str) -> Result<()> {
        self.done = self.done.saturating_add(w);
        if self.done > self.budget {
            return Err(KernelError::BudgetExhausted { stage, why: format!("the boolean ran out of its work budget ({} units) in its {stage} stage", self.budget) });
        }
        Ok(())
    }
}

impl RunOpts {
    /// Section and imprint runs: a boolean's defaults.
    const PLAIN: RunOpts = RunOpts { tol: VTOL, flush: false, merge_coplanar: true, work_budget: DEFAULT_WORK_BUDGET };
}

#[derive(Debug, Clone)]
struct WEdge {
    curve: Curve,
    t: [f64; 2],
    v: [usize; 2],
    degenerate: bool,
    /// The edge's tolerance (the input's, or the run's for new edges).
    tol: f64,
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
    /// The face's own tolerance (0: exact, the run's applies).
    tol: f64,
}

struct Pool {
    verts: Vec<P3>,
    /// Each vertex's tolerance: points within the larger of two tolerances (and the run's) are
    /// one vertex.
    vtols: Vec<f64>,
    /// The grid's cell: big enough that a merge never reaches past the neighbouring cells.
    cell: f64,
    /// The run's merge distance.
    tol: f64,
    grid: HashMap<(i64, i64, i64), Vec<usize>>,
    edges: Vec<WEdge>,
}

impl Pool {
    /// A pool for a run merging within `tol`, of entities of tolerances up to `max_tol`.
    fn with_tolerance(tol: f64, max_tol: f64) -> Self {
        Pool { verts: Vec::new(), vtols: Vec::new(), cell: (tol * 16.0).max(max_tol * 2.0), tol, grid: HashMap::new(), edges: Vec::new() }
    }
    fn cell_of(&self, p: &P3) -> (i64, i64, i64) {
        let s = 1.0 / self.cell;
        ((p.x * s).floor() as i64, (p.y * s).floor() as i64, (p.z * s).floor() as i64)
    }
    /// The vertex `p` (of tolerance `tol`) falls on.
    fn find_tol(&self, p: &P3, tol: f64) -> Option<usize> {
        let (cx, cy, cz) = self.cell_of(p);
        let mut best: Option<(f64, usize)> = None;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(v) = self.grid.get(&(cx + dx, cy + dy, cz + dz)) {
                        for &i in v {
                            let d = (self.verts[i] - p).norm();
                            if d < self.tol.max(tol).max(self.vtols[i]) && best.is_none_or(|(bd, _)| d < bd) {
                                best = Some((d, i));
                            }
                        }
                    }
                }
            }
        }
        best.map(|(_, i)| i)
    }
    /// The vertex at `p` with tolerance `tol`: an existing one within reach (its tolerance grown
    /// to hold this one's ball), else a new one.
    fn vertex_tol(&mut self, p: P3, tol: f64) -> usize {
        if let Some(i) = self.find_tol(&p, tol) {
            if tol > 0.0 {
                let reach = (self.verts[i] - p).norm() + tol;
                if reach > self.vtols[i] {
                    self.vtols[i] = reach;
                }
            }
            return i;
        }
        if std::env::var("CADRS_VERTEX_DEBUG").is_ok() {
            let near_round = |x: f64| {
                let r = (x * 2.0).round() / 2.0;
                (x - r).abs() > 1e-11 && (x - r).abs() < 1e-5
            };
            if near_round(p.x) || near_round(p.y) || near_round(p.z) {
                eprintln!("inexact vertex {p:?}\n{}", std::backtrace::Backtrace::force_capture());
            }
        }
        self.new_vertex_tol(p, tol)
    }
    fn new_vertex_tol(&mut self, p: P3, tol: f64) -> usize {
        self.verts.push(p);
        self.vtols.push(tol);
        let i = self.verts.len() - 1;
        let c = self.cell_of(&p);
        self.grid.entry(c).or_default().push(i);
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

fn edge_bbox(e: &WEdge, verts: &[P3], tol: f64) -> Bbox {
    let mut bb = Bbox::empty();
    bb.add(&verts[e.v[0]]);
    bb.add(&verts[e.v[1]]);
    if !e.degenerate {
        bb.union(&curve_bbox(&e.curve, e.t[0], e.t[1]));
    }
    bb.grow(tol * 10.0 + e.tol)
}

/// A box that holds the curve between t0 and t1: exact for lines, circles and ellipses (their
/// extremes in each axis), the poles' box for B-splines (a rational B-spline with positive
/// weights lies in its poles' convex hull).
pub fn curve_bbox(c: &Curve, t0: f64, t1: f64) -> Bbox {
    let (lo, hi) = (t0.min(t1), t0.max(t1));
    let mut bb = Bbox::empty();
    bb.add(&c.point(lo));
    bb.add(&c.point(hi));
    match c {
        Curve::Line { .. } => {}
        Curve::Circle { c: o, x, y, r } | Curve::Ellipse { c: o, x, y, a: r, .. } => {
            let (rx, ry) = match c {
                Curve::Ellipse { a, b, .. } => (*a, *b),
                _ => (*r, *r),
            };
            let _ = o;
            // Coordinate k is o_k + rx·x_k·cos t + ry·y_k·sin t: extreme where tan t = ry·y_k / (rx·x_k).
            for k in 0..3 {
                let t = (ry * y[k]).atan2(rx * x[k]);
                for base in [t, t + std::f64::consts::PI] {
                    // Every copy of the extreme inside [lo, hi].
                    let mut tt = base + ((lo - base) / std::f64::consts::TAU).ceil() * std::f64::consts::TAU;
                    while tt <= hi {
                        bb.add(&c.point(tt));
                        tt += std::f64::consts::TAU;
                    }
                }
            }
        }
        Curve::Nurbs(_) => {
            let n = c.to_nurbs(lo, hi);
            if n.cps.iter().all(|h| h.w > 0.0) {
                for h in &n.cps {
                    bb.add(&super::nurbs::dehom(h));
                }
            } else {
                for i in 0..=64 {
                    bb.add(&c.point(lo + (hi - lo) * i as f64 / 64.0));
                }
                bb = bb.grow(bb.diagonal() * 0.02);
            }
        }
    }
    bb
}

/// Imports a body's vertices, edges and faces into the pool.
fn import(pool: &mut Pool, faces: &mut Vec<WFace>, b: &Body, owner: usize, merge: bool) {
    // Exact inputs carry no tolerance of their own (the run's applies).
    let own = |t: f64| if t > super::topo::TOL_MIN { t } else { 0.0 };
    let vmap: Vec<usize> = b.verts.iter().map(|v| if merge { pool.vertex_tol(v.p, own(v.tol)) } else { pool.new_vertex_tol(v.p, own(v.tol)) }).collect();
    let e0 = pool.edges.len();
    for e in &b.edges {
        pool.edges.push(WEdge { curve: e.curve.clone(), t: e.t, v: [vmap[e.v[0]], vmap[e.v[1]]], degenerate: e.degenerate, tol: own(e.tol), splits: Vec::new() });
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
        // Curved faces may bulge past their edges: a box sure to hold the face.
        if !matches!(f.surf, Surface::Plane { .. }) {
            bb.union(&b.face_bbox(fi));
        }
        let center = uvb.center();
        // Faces don't carry a tolerance of their own yet (OCCT's are usually exact even where
        // their edges' are not): the run's applies, and the edges' and vertices' where they are
        // involved.
        let ftol = 0.0;
        faces.push(WFace {
            surf: f.surf.clone(),
            sense: f.sense,
            loops,
            owner,
            orig: fi,
            polys,
            bbox: bb.grow(pool.tol * 10.0 + ftol),
            uv_center: P2::new(center.x, center.y),
            tol: ftol,
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

/// The parameter of the point of the curve nearest `p`, starting from `t` (exact for lines and
/// circles; Newton otherwise).
fn refine_param(c: &Curve, t0: f64, t1: f64, p: &P3, t: f64) -> f64 {
    match c {
        Curve::Line { .. } | Curve::Circle { .. } => {
            let tc = c.closest(p, t0 - (t1 - t0), t1 + (t1 - t0));
            if (tc - t).abs() < 1e-4 * (1.0 + t.abs()) { tc.clamp(t0, t1) } else { t }
        }
        _ => {
            let mut x = t;
            for _ in 0..20 {
                let (q, d1, d2) = c.d2(x);
                let r = q - p;
                let f = r.dot(&d1);
                let df = d1.norm_squared() + r.dot(&d2);
                if df.abs() < 1e-300 {
                    break;
                }
                let nx = (x - f / df).clamp(t0, t1);
                if (nx - x).abs() < 1e-16 * (1.0 + x.abs()) {
                    x = nx;
                    break;
                }
                x = nx;
            }
            x
        }
    }
}

/// Two faces sharing an edge (the same vertices and curve) whose surfaces are tangent along
/// it, and which aren't both analytic (analytic pairs are intersected exactly).
fn tangent_along_shared_edge(pool: &Pool, fa: &WFace, fb: &WFace, ea: &[usize], eb: &[usize]) -> bool {
    let analytic = |s: &Surface| !matches!(s, Surface::Nurbs(_) | Surface::Extrusion { .. } | Surface::Revolution { .. });
    if analytic(&fa.surf) && analytic(&fb.surf) {
        return false;
    }
    // A boundary edge of one face lying on the other's surface with the two tangent along it
    // (a blend's contact curve on the face it blends into).
    // (Also a boundary edge lying on the other surface where the face leaves it: a blend
    // tool's strip meeting the face it blends along its contact curve. The edge is imprinted
    // exactly; contouring would only find it again, inexactly.)
    // (Tangent there: an edge of a face lying on a surface the face crosses, as a tube's top
    // line on a wall through its axis, is no contact; the face–face stage drops the curve the
    // intersection finds along such an edge again.)
    let tangent_at = |f: &WFace, g: &WFace, p: &P3| {
        let ((fu, fv), (gu, gv)) = (f.surf.project(p, None), g.surf.project(p, None));
        f.surf.normal(fu, fv).cross(&g.surf.normal(gu, gv)).norm() < 1e-3
    };
    let tangent_on = |edges: &[usize], f: &WFace, g: &WFace| -> bool {
        edges.iter().any(|&x| {
            let e = &pool.edges[x];
            !e.degenerate
                && (e.t[1] - e.t[0]).abs() > 0.0
                && edge_on_surface(e, &g.surf, pool.tol)
                && e.curve.length(e.t[0], e.t[1]) > 1e-3
                && [0.25, 0.5, 0.75].iter().all(|k| tangent_at(f, g, &e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * k)))
        })
    };
    if tangent_on(eb, fb, fa) || tangent_on(ea, fa, fb) {
        return true;
    }
    for &x in ea {
        let e = &pool.edges[x];
        if e.degenerate {
            continue;
        }
        for &y in eb {
            let o = &pool.edges[y];
            if o.degenerate {
                continue;
            }
            // The same curve, at least where they overlap (whole circles may have their seams in
            // different places).
            let same_ends = x == y || {
                let on = |a: &WEdge, b: &WEdge| [0.25, 0.5, 0.75].iter().all(|f| {
                    let m = b.curve.point(b.t[0] + (b.t[1] - b.t[0]) * f);
                    on_curve(&a.curve, a.t[0], a.t[1], &m, pool.tol * 10.0).is_some()
                });
                on(e, o) && on(o, e)
            };
            if std::env::var("CADRS_BOOL_DEBUG").is_ok() && same_ends {
                let p = e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * 0.5);
                let (ua, va) = fa.surf.project(&p, Some((fa.uv_center.x, fa.uv_center.y)));
                let (ub, vb) = fb.surf.project(&p, Some((fb.uv_center.x, fb.uv_center.y)));
                eprintln!("shared edge {x}/{y}: normals dot {:.12}, proj err {:.3e} {:.3e}", fa.surf.normal(ua, va).dot(&fb.surf.normal(ub, vb)), (fa.surf.eval(ua, va) - p).norm(), (fb.surf.eval(ub, vb) - p).norm());
            }
            if !same_ends {
                continue;
            }
            // Tangent along it: the normals agree (up to sign) at a few points.
            let tangent = [0.2, 0.5, 0.8].iter().all(|f| {
                let p = e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * f);
                let (ua, va) = fa.surf.project(&p, None);
                let (ub, vb) = fb.surf.project(&p, None);
                fa.surf.normal(ua, va).dot(&fb.surf.normal(ub, vb)).abs() > 1.0 - 1e-6
            });
            if tangent {
                return true;
            }
        }
    }
    false
}

/// Whether an edge lies on a surface (its sample points are on it).
/// (Within the run's `tol` and the edge's own tolerance.)
fn edge_on_surface(e: &WEdge, s: &Surface, tol: f64) -> bool {
    edge_on_surface_tol(e, s, 0.0, tol)
}

/// [`edge_on_surface`] within also `ftol` (the face's).
fn edge_on_surface_tol(e: &WEdge, s: &Surface, ftol: f64, tol: f64) -> bool {
    if e.degenerate {
        return true;
    }
    let tol = tol.max(e.tol).max(ftol);
    [0.0, 0.3, 0.5, 0.7, 1.0].iter().all(|f| {
        let p = e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * f);
        s.distance(&p) < tol
    })
}

/// In-face test of a model point (projected onto the face's surface).
fn point_in_face(f: &WFace, p: &P3, tol: f64) -> InFace {
    let (mut u, mut v) = f.surf.project(p, Some((f.uv_center.x, f.uv_center.y)));
    // A projection from the face's middle can stop short on a B-spline or an extrusion (a
    // local minimum, the end of the range): from the whole surface then.
    if (f.surf.eval(u, v) - p).norm() > tol * 10.0 + 1e-9 && matches!(f.surf, Surface::Nurbs(_) | Surface::Extrusion { .. }) {
        let (u2, v2) = f.surf.project(p, None);
        if (f.surf.eval(u2, v2) - p).norm() < (f.surf.eval(u, v) - p).norm() {
            (u, v) = (u2, v2);
        }
    }
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
    interior_points(loops).into_iter().next()
}

fn interior_points(loops: &[Vec<Arc>]) -> Vec<P2> {
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
    interior_points_of_polys(&polys)
}

/// Points inside the polygons (on a few scanlines, each midway across an inside interval),
/// farthest from the boundary first.
pub fn interior_points_of_polys(polys: &[Vec<P2>]) -> Vec<P2> {
    let mut vb = (f64::INFINITY, f64::NEG_INFINITY);
    let mut ub = (f64::INFINITY, f64::NEG_INFINITY);
    for p in polys {
        for q in p {
            vb = (vb.0.min(q.y), vb.1.max(q.y));
            ub = (ub.0.min(q.x), ub.1.max(q.x));
        }
    }
    if !vb.0.is_finite() || vb.1 - vb.0 < 1e-14 {
        return Vec::new();
    }
    // Distance from a point to the boundary, scaled to the box.
    let (su, sv) = (1.0 / (ub.1 - ub.0).max(1e-300), 1.0 / (vb.1 - vb.0).max(1e-300));
    let clearance = |q: &P2| -> f64 {
        let qs = P2::new(q.x * su, q.y * sv);
        let mut d = f64::INFINITY;
        for p in polys {
            let n = p.len();
            for i in 0..n {
                let (a, b) = (p[i], p[(i + 1) % n]);
                d = d.min(seg_dist2(&qs, &P2::new(a.x * su, a.y * sv), &P2::new(b.x * su, b.y * sv)));
            }
        }
        d
    };
    let mut cands: Vec<(f64, P2)> = Vec::new();
    for frac in [0.5123, 0.3137, 0.6863, 0.1571, 0.8429, 0.0713, 0.9287, 0.4231, 0.5869, 0.2377, 0.7623] {
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
            let q = P2::new((xs[k] + xs[k + 1]) * 0.5, y);
            cands.push((clearance(&q), q));
            k += 2;
        }
    }
    cands.sort_by(|a, b| b.0.total_cmp(&a.0));
    cands.into_iter().map(|(_, p)| p).collect()
}

pub fn interior_of_polys(polys: &[Vec<P2>]) -> Option<P2> {
    interior_points_of_polys(polys).into_iter().next()
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

/// The boolean of two bodies, with the default [`BoolOptions`].
pub fn boolean_bodies(a: &Body, b: &Body, op: BoolOp) -> Result<BoolOut> {
    boolean_bodies_with(a, b, op, BoolOptions::default())
}

/// The boolean of two bodies with explicit options.
pub fn boolean_bodies_with(a: &Body, b: &Body, op: BoolOp, opts: BoolOptions) -> Result<BoolOut> {
    // At the default tolerance first; when that leaves the result broken (near-coincident
    // geometry the tolerance splits into slivers or gaps: a surface through a corner within
    // 1e-5), again at growing tolerances, as OCCT grows its entities' tolerances.
    // Faces of the two bodies on one surface but for float noise (made apart: radii 25 and
    // 25.0000015) get the gap as their edges' and vertices' tolerance, so they meet.
    let raised = near_coincident_tolerances(a, b, opts.tol.point);
    let (a, b) = match &raised {
        Some((x, y)) => (x, y),
        None => (a, b),
    };
    let at = |tol: f64| RunOpts { tol, flush: opts.flush, merge_coplanar: opts.merge_coplanar, work_budget: opts.work_budget };
    let first = run(a, b, Mode::Bool(op), at(opts.tol.point));
    // (Retried: an open shell, a face that doesn't close, an invalid result: `BooleanFailed`, and
    // the failures of the steps a run calls that say so.)
    let broken = |m: &str| ["open shell", "left", "doesn't close", "is invalid"].iter().any(|w| m.contains(w));
    if !matches!(&first, Err(KernelError::OpenResult(_))) && !matches!(&first, Err(e) if e.is_failure() && broken(&e.message())) {
        return first;
    }
    for t in opts.tol.retries.into_iter().filter(|_| std::env::var_os("CADRS_DEBUG_NO_RETRY").is_none()) {
        if let Ok(r) = run(a, b, Mode::Bool(op), at(t)) {
            return Ok(r);
        }
    }
    first
}

/// The faces of `a` and `b` that lie on one surface only within 1e-7 of their size (beyond the
/// run's tolerance): the two bodies with those faces' edges and vertices given the gap (with
/// margin) as tolerance; None when there are none.
fn near_coincident_tolerances(a: &Body, b: &Body, tol: f64) -> Option<(Body, Body)> {
    let face_pts = |body: &Body, f: usize| -> Vec<P3> {
        let mut pts = Vec::new();
        for c in body.faces[f].loops.iter().flatten() {
            let e = &body.edges[c.edge];
            for i in 0..=8 {
                pts.push(e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0));
            }
        }
        pts
    };
    let boxes = |body: &Body| -> Vec<(Vec<P3>, Bbox)> {
        (0..body.faces.len())
            .map(|f| {
                let pts = face_pts(body, f);
                let mut bb = Bbox::empty();
                for p in &pts {
                    bb.add(p);
                }
                (pts, bb)
            })
            .collect()
    };
    let (ba, bb) = (boxes(a), boxes(b));
    let mut raise: Vec<(usize, usize, f64)> = Vec::new();
    for (fa, (pa, xa)) in ba.iter().enumerate() {
        for (fb, (pb, xb)) in bb.iter().enumerate() {
            let size = xa.diagonal().max(xb.diagonal());
            let ctol = size * 1e-7;
            if ctol <= tol || !xa.grow(ctol).overlaps(&xb.grow(ctol)) {
                continue;
            }
            let (sa, sb) = (&a.faces[fa].surf, &b.faces[fb].surf);
            if sa.same_domain(sb, tol).is_some() || sa.same_domain(sb, ctol).is_none() {
                continue;
            }
            // The gap: the farthest boundary point of either face from the other's surface.
            let gap = pb.iter().map(|p| sa.distance(p)).chain(pa.iter().map(|p| sb.distance(p))).fold(0.0, f64::max);
            if gap > tol && gap <= ctol * 2.0 {
                raise.push((fa, fb, gap * 1.5));
            }
        }
    }
    if std::env::var_os("CADRS_BOOL_DEBUG").is_some() {
        eprintln!("near-coincident faces (a, b, tolerance): {raise:?}");
    }
    if raise.is_empty() {
        return None;
    }
    let (mut a2, mut b2) = (a.clone(), b.clone());
    let bump = |body: &mut Body, f: usize, t: f64| {
        let es: Vec<usize> = body.faces[f].loops.iter().flatten().map(|c| c.edge).collect();
        for e in es {
            body.edges[e].tol = body.edges[e].tol.max(t);
            for v in body.edges[e].v {
                body.verts[v].tol = body.verts[v].tol.max(t);
            }
        }
    };
    for (fa, fb, t) in raise {
        bump(&mut a2, fa, t);
        bump(&mut b2, fb, t);
    }
    Some((a2, b2))
}

/// [`boolean_bodies`], with faces of the two inputs on one surface merged whatever the
/// operation (a blend's tool face flush with a face it ends against becomes part of it).
pub fn boolean_bodies_flush(a: &Body, b: &Body, op: BoolOp) -> Result<BoolOut> {
    boolean_bodies_with(a, b, op, BoolOptions { flush: true, ..BoolOptions::default() })
}

/// The first loop of `b` whose p-curves don't meet in (u, v) at a vertex (beyond the vertex's
/// tolerance, as [`Body::check`] has it; whole turns apart across a periodic seam is meeting).
fn uv_gap(b: &Body) -> Option<String> {
    for (fi, f) in b.faces.iter().enumerate() {
        let (mu, mv) = f.surf.metric();
        let off = |d: f64, periodic: bool| if periodic { (d - (d / TAU).round() * TAU).abs() } else { d.abs() };
        for (li, l) in f.loops.iter().enumerate() {
            for k in 0..l.len() {
                let (x, y) = (&l[k], &l[(k + 1) % l.len()]);
                let (ex, ey) = (b.edges.get(x.edge)?, b.edges.get(y.edge)?);
                let (_, t1) = super::topo::co_range(ex, x.fwd);
                let (t0, _) = super::topo::co_range(ey, y.fwd);
                let (pa, pb) = (x.pc.point(t1), y.pc.point(t0));
                let v = if x.fwd { ex.v[1] } else { ex.v[0] };
                let tol = b.verts.get(v).map_or(0.0, |v| v.tol).max(super::tol::LOOP_GAP);
                if off(pa.x - pb.x, f.surf.u_periodic()) * mu > tol || off(pa.y - pb.y, f.surf.v_periodic()) * mv > tol {
                    return Some(format!("face {fi} loop {li}: p-curve gap after coedge {k} ({:.3e}, {:.3e})", pa.x - pb.x, pa.y - pb.y));
                }
            }
        }
    }
    None
}

/// Gives B-spline faces of `a` that lie (within 2 × [`VTOL`]) on a plane or cylinder face of
/// `b` that surface, with their p-curves and sense recomputed. True when any changed.
fn snap_to(a: &mut Body, b: &Body, tol: f64) -> bool {
    if std::env::var("CADRS_DEBUG_NO_SNAP").is_ok() {
        return false;
    }
    let mut changed = false;
    let targets: Vec<(usize, Bbox)> = b
        .faces
        .iter()
        .enumerate()
        .filter(|(_, f)| matches!(f.surf, Surface::Plane { .. } | Surface::Cylinder { .. }))
        .map(|(i, _)| (i, super::kernel::subset(b, &[i], false).bbox().grow(1e-3)))
        .collect();
    if targets.is_empty() {
        return false;
    }
    for fi in 0..a.faces.len() {
        let Surface::Nurbs(ns) = &a.faces[fi].surf else { continue };
        let ((u0, u1), (v0, v1)) = ns.range();
        let s = a.faces[fi].surf.clone();
        let n = 16;
        let pts: Vec<P3> = (0..=n).flat_map(|i| (0..=n).map(move |j| (i, j))).map(|(i, j)| s.eval(u0 + (u1 - u0) * i as f64 / n as f64, v0 + (v1 - v0) * j as f64 / n as f64)).collect();
        let mut fb = Bbox::empty();
        for p in &pts {
            fb.add(p);
        }
        for &(ti, ref tb) in &targets {
            if !tb.overlaps(&fb) {
                continue;
            }
            let t = &b.faces[ti].surf;
            if std::env::var("CADRS_SNAP_DEBUG").is_ok() {
                let dev = pts.iter().filter_map(|p| t.signed_distance(p)).map(f64::abs).fold(0.0, f64::max);
                eprintln!("snap? face {fi} onto {ti} ({:?}): deviation {dev:e}", t.kind());
            }
            if pts.iter().any(|p| t.signed_distance(p).is_none_or(|d| d.abs() > 2.0 * tol)) {
                continue;
            }
            // Between the samples too.
            let fine = 64;
            let ok = (0..fine).all(|i| (0..fine).all(|j| {
                let p = s.eval(u0 + (u1 - u0) * (i as f64 + 0.5) / fine as f64, v0 + (v1 - v0) * (j as f64 + 0.5) / fine as f64);
                t.signed_distance(&p).is_some_and(|d| d.abs() <= 2.0 * tol)
            }));
            if !ok {
                continue;
            }
            let mid = s.eval(0.5 * (u0 + u1), 0.5 * (v0 + v1));
            let near = t.project(&mid, None);
            let edges = a.edges.clone();
            for l in &mut a.faces[fi].loops {
                for co in l.iter_mut() {
                    let e = &edges[co.edge];
                    co.pc = super::pcurve::pcurve(t, &e.curve, e.t[0], e.t[1], Some(P2::new(near.0, near.1)));
                }
            }
            a.faces[fi].surf = t.clone();
            super::build::fix_senses(a, [fi]);
            if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
                eprintln!("snapped face {fi} onto the other body's face {ti} ({:?})", t.kind());
            }
            changed = true;
            break;
        }
    }
    changed
}

/// What to do with the split faces.
#[derive(Debug, Clone)]
pub enum Mode {
    Bool(BoolOp),
    /// Split the listed faces of the target (all when None) by the tool's faces and keep every
    /// piece of the target: the target's faces imprinted by the tool.
    Imprint(Option<Vec<usize>>),
    /// Only where the faces of the two meet: the intersection edges (and edges of one lying
    /// on faces of the other), as a body of edges.
    Section,
}

/// The section of two bodies: the edges where their faces meet (a body of edges, no faces).
pub fn section(a: &Body, b: &Body) -> Result<Body> {
    run(a, b, Mode::Section, RunOpts::PLAIN).map(|o| o.body)
}

pub fn imprint(a: &Body, b: &Body, faces: Option<Vec<usize>>) -> Result<BoolOut> {
    run(a, b, Mode::Imprint(faces), RunOpts::PLAIN)
}

/// One run of the boolean.
fn run(a: &Body, b: &Body, mode: Mode, opts: RunOpts) -> Result<BoolOut> {
    let RunOpts { tol: run_tol, flush, merge_coplanar, work_budget } = opts;
    // The work done so far, against the budget.
    let mut work = Work { done: 0, budget: work_budget };
    // B-spline faces lying on an analytic face of the other body (within the tolerance:
    // approximated sweeps along helices, lofts of circles) take its exact surface, so the two
    // are imprinted as coincident instead of intersected nearly tangentially.
    let (sa, sb);
    let (a, b) = {
        let mut a2 = a.clone();
        let mut b2 = b.clone();
        let ca = snap_to(&mut a2, b, run_tol);
        let cb = snap_to(&mut b2, a, run_tol);
        sa = a2;
        sb = b2;
        (if ca { &sa } else { a }, if cb { &sb } else { b })
    };
    let op = match &mode {
        Mode::Bool(op) => *op,
        Mode::Imprint(_) | Mode::Section => BoolOp::Subtract,
    };
    let selected = |f: usize| -> bool {
        match &mode {
            Mode::Imprint(Some(list)) => list.contains(&f),
            _ => true,
        }
    };
    let imprinting = matches!(mode, Mode::Imprint(_));
    // Disjoint boxes: nothing to intersect.
    let (ba, bb) = (a.bbox(), b.bbox());
    if !ba.grow(run_tol).overlaps(&bb.grow(run_tol)) {
        if imprinting {
            return Ok(BoolOut { body: a.clone(), from: (0..a.faces.len()).map(|i| vec![(0, i)]).collect() });
        }
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
            BoolOp::Intersect => Err(KernelError::EmptyResult("the result is empty".into())),
        };
    }
    let mut prof = Prof::new();
    let max_tol = a.verts.iter().chain(&b.verts).map(|v| v.tol).chain(a.edges.iter().chain(&b.edges).map(|e| e.tol)).fold(0.0, f64::max);
    let mut pool = Pool::with_tolerance(run_tol, max_tol);
    let mut faces: Vec<WFace> = Vec::new();
    import(&mut pool, &mut faces, a, 0, false);
    let na_edges = pool.edges.len();
    let na_faces = faces.len();
    import(&mut pool, &mut faces, b, 1, true);
    let ne = pool.edges.len();
    let owner_of_edge = |e: usize| if e < na_edges { 0 } else { 1 };
    let ebox: Vec<Bbox> = pool.edges.iter().map(|e| edge_bbox(e, &pool.verts, run_tol)).collect();
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
    prof.mark("import");
    // 2a. Vertices on edges of the other body (within the larger of the vertex's, the edge's
    // and the run's tolerances).
    let verts_of = |range: std::ops::Range<usize>, pool: &Pool| -> Vec<usize> {
        let mut seen = std::collections::HashSet::new();
        let mut vs = Vec::new();
        for o in range {
            for v in pool.edges[o].v {
                if seen.insert(v) {
                    vs.push(v);
                }
            }
        }
        vs
    };
    let body_verts = [verts_of(0..na_edges, &pool), verts_of(na_edges..ne, &pool)];
    // Spatial indexes (candidates come back in the order the full scans used).
    let point_box = |p: &P3| {
        let mut b = Bbox::empty();
        b.add(p);
        b
    };
    let vtrees = [0, 1].map(|k| super::bvh::Bvh::new(body_verts[k].iter().map(|&v| point_box(&pool.verts[v])).collect()));
    let reach = pool.vtols.iter().copied().fold(run_tol, f64::max).max(pool.edges.iter().map(|e| e.tol).fold(0.0, f64::max));
    for e in 0..ne {
        if pool.edges[e].degenerate {
            continue;
        }
        let ob = 1 - owner_of_edge(e);
        let mut near = vtrees[ob].query(&ebox[e].grow(reach));
        near.sort_unstable();
        let other: Vec<usize> = near.into_iter().map(|i| body_verts[ob][i]).collect();
        for &v in &other {
            let p = pool.verts[v];
            let ed = &pool.edges[e];
            let tol = run_tol.max(pool.vtols[v]).max(ed.tol);
            if !ebox[e].grow(tol).contains(&p) {
                continue;
            }
            if v == ed.v[0] || v == ed.v[1] {
                continue;
            }
            if let Some(t) = on_curve(&ed.curve, ed.t[0], ed.t[1], &p, tol) {
                pool.split(e, t, v);
            }
        }
    }
    // 2a'. Closed edges (whole circles) are split at every vertex lying on them, of either body:
    // coincident circles with their seams at different places then share their pieces.
    let all_verts: Vec<usize> = verts_of(0..ne, &pool);
    for e in 0..ne {
        let ed = pool.edges[e].clone();
        if ed.degenerate || ed.v[0] != ed.v[1] {
            continue;
        }
        for &v in &all_verts {
            if v == ed.v[0] {
                continue;
            }
            let p = pool.verts[v];
            let tol = run_tol.max(pool.vtols[v]).max(ed.tol);
            if !ebox[e].grow(tol).contains(&p) {
                continue;
            }
            if let Some(t) = on_curve(&ed.curve, ed.t[0], ed.t[1], &p, tol) {
                pool.split(e, t, v);
            }
        }
    }
    // 2b. Edge–edge intersections (A × B).
    let btree = super::bvh::Bvh::new(ebox[na_edges..ne].to_vec());
    for ea in 0..na_edges {
        if pool.edges[ea].degenerate {
            continue;
        }
        let mut near = btree.query(&ebox[ea]);
        near.sort_unstable();
        for eb in near.into_iter().map(|j| j + na_edges) {
            if pool.edges[eb].degenerate || !ebox[ea].overlaps(&ebox[eb]) {
                continue;
            }
            let (ca, cb) = (pool.edges[ea].clone(), pool.edges[eb].clone());
            let et = ca.tol.max(cb.tol);
            for (ta, tb) in ssi::curve_curve(&ca.curve, ca.t[0], ca.t[1], &cb.curve, cb.t[0], cb.t[1], run_tol.max(et) * 0.5) {
                let p = ca.curve.point(ta);
                let v = pool.vertex_tol(p, et);
                // The parameters of the (possibly merged) vertex itself.
                let q = pool.verts[v];
                let ta = refine_param(&ca.curve, ca.t[0], ca.t[1], &q, ta);
                let tb = refine_param(&cb.curve, cb.t[0], cb.t[1], &q, tb);
                pool.split(ea, ta, v);
                pool.split(eb, tb, v);
            }
        }
    }
    prof.mark("edge-edge");
    // 2c. Edge–face intersections.
    let mut face_pts: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    // Edges of one body lying on a face of the other (imprinted as cuts once split).
    let mut on_face: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    let ftree = super::bvh::Bvh::new(faces.iter().map(|f| f.bbox).collect());
    for e in 0..ne {
        if pool.edges[e].degenerate {
            continue;
        }
        let oe = owner_of_edge(e);
        let mut near = ftree.query(&ebox[e]);
        near.sort_unstable();
        for fi in near {
            if faces[fi].owner == oe || !ebox[e].overlaps(&faces[fi].bbox) {
                continue;
            }
            let ed = pool.edges[e].clone();
            if edge_on_surface_tol(&ed, &faces[fi].surf, faces[fi].tol, run_tol) {
                if !on_face[fi].contains(&e) {
                    on_face[fi].push(e);
                }
                continue;
            }
            for t in ssi::curve_surface(&ed.curve, ed.t[0], ed.t[1], &faces[fi].surf) {
                let p = ed.curve.point(t);
                let et = ed.tol.max(faces[fi].tol);
                if point_in_face(&faces[fi], &p, run_tol.max(et)) == InFace::Outside {
                    continue;
                }
                let v = pool.vertex_tol(p, et);
                let t = refine_param(&ed.curve, ed.t[0], ed.t[1], &pool.verts[v], t);
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
    prof.mark("edge-face");
    // 3. Face–face.
    let mut cuts: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    let mut coincident: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    let mut new_edges: Vec<usize> = Vec::new();
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
    let overlap = |fa: usize, fb: usize| {
        let mut ob = faces[fa].bbox;
        ob.min = ob.min.sup(&faces[fb].bbox.min);
        ob.max = ob.max.inf(&faces[fb].bbox.max);
        ob
    };
    // The pairs of faces to intersect, in a fixed order.
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for fa in 0..na_faces {
        if !selected(faces[fa].orig) {
            continue;
        }
        let mut near = ftree.query(&faces[fa].bbox);
        near.sort_unstable();
        for fb in near.into_iter().filter(|&f| f >= na_faces) {
            if !faces[fa].bbox.overlaps(&faces[fb].bbox) {
                continue;
            }
            // Coincident within the faces' tolerances (near-coincident faces have had theirs
            // raised: `near_coincident_tolerances`).
            let ctol = run_tol.max(faces[fa].tol).max(faces[fb].tol);
            if faces[fa].surf.same_domain(&faces[fb].surf, ctol).is_some() {
                coincident[fa].push(fb);
                coincident[fb].push(fa);
                continue;
            }
            // Faces that share an edge and are tangent along it (one continues the other): the
            // shared edge is their contact; contouring tangent surfaces finds nothing more but
            // noise.
            if tangent_along_shared_edge(&pool, &faces[fa], &faces[fb], &face_edges[fa], &face_edges[fb]) {
                continue;
            }
            if std::env::var("CADRS_TANGENT_DEBUG").is_ok() && (matches!(faces[fa].surf, Surface::Nurbs(_)) || matches!(faces[fb].surf, Surface::Nurbs(_))) {
                let dev = |edges: &[usize], s: &Surface| -> f64 {
                    edges.iter().map(|&x| {
                        let e = &pool.edges[x];
                        [0.0, 0.3, 0.5, 0.7, 1.0].iter().map(|f| s.distance(&e.curve.point(e.t[0] + (e.t[1] - e.t[0]) * f))).fold(0.0, f64::max)
                    }).fold(f64::INFINITY, f64::min)
                };
                eprintln!("not tangent {fa}×{fb}: closest edge of b to a {:e}, of a to b {:e}", dev(&face_edges[fb], &faces[fa].surf), dev(&face_edges[fa], &faces[fb].surf));
            }
            pairs.push((fa, fb));
        }
    }
    // Their intersections, each a function of the two faces alone: computed in parallel and
    // used below in the pairs' order, so the result doesn't depend on the scheduling.
    // (In chunks of pairs, the budget checked after each: a run past it stops within a chunk.)
    // (With the boundary of a region where the two surfaces coincide, when they do over part.)
    type Found = (Vec<ssi::IntCurve>, Vec<P3>, std::time::Duration, Vec<ssi::IntCurve>);
    let mut ssis: Vec<Found> = Vec::with_capacity(pairs.len());
    for chunk in pairs.chunks(64) {
        let done: Vec<(Found, u64)> = chunk
            .par_iter()
            .map(|&(fa, fb)| {
                let t = std::time::Instant::now();
                // (The curves the surfaces only touch along are left out: the faces' tangent
                // contact is found from their boundaries, as for faces tangent along an edge.)
                let r = ssi::surface_surface_tol(&faces[fa].surf, uvbox(&faces[fa]), &faces[fb].surf, uvbox(&faces[fb]), &overlap(fa, fb), run_tol);
                ((r.curves, r.singular, t.elapsed(), r.coincident), r.work)
            })
            .collect();
        work.spend(done.iter().map(|d| d.1).sum(), "surface intersection")?;
        ssis.extend(done.into_iter().map(|(f, _)| f));
    }
    // Pairs of faces whose surfaces coincide over part of the faces (from the intersections): their
    // sub-faces on the other face's surface classify as on it, as a coincident pair's do.
    let mut partial: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for (&(fa, fb), (mut curves, singular, el, shared)) in pairs.iter().zip(ssis) {
        // The shared region's boundary cuts both faces, as a crossing does.
        if !shared.is_empty() {
            partial.insert((fa, fb));
            partial.insert((fb, fa));
            coincident[fa].push(fb);
            coincident[fb].push(fa);
            curves.extend(shared);
        }
        let ob = overlap(fa, fb);
        // A boundary edge of one face lying on the other's surface, the faces crossing there (a
        // blend tool's strip ending on the face it blends, a wall through a tube's axis), is
        // imprinted exactly by the edge stages: the curve the intersection finds along it
        // again, inexactly, goes.
        let near = (run_tol * 10.0).max(1e-7 * ob.diagonal());
        let along = |ic: &ssi::IntCurve| {
            let pts: Vec<P3> = (0..=4).map(|k| ic.c.point(ic.t0 + (ic.t1 - ic.t0) * k as f64 / 4.0)).collect();
            let runs_on = |x: usize| {
                let e = &pool.edges[x];
                !e.degenerate && ebox[x].grow(near).contains(&pts[2]) && pts.iter().all(|p| on_curve(&e.curve, e.t[0], e.t[1], p, near).is_some())
            };
            face_edges[fa].iter().any(|&x| runs_on(x) && edge_on_surface(&pool.edges[x], &faces[fb].surf, run_tol))
                || face_edges[fb].iter().any(|&x| runs_on(x) && edge_on_surface(&pool.edges[x], &faces[fa].surf, run_tol))
        };
        let curves: Vec<ssi::IntCurve> = curves.into_iter().filter(|ic| !along(ic)).collect();
        // Whether a boundary edge of either face lies on the other's surface.
        let edge_on_other = {
            let lies = |edges: &[usize], g: &WFace| {
                edges.iter().any(|&x| {
                    let e = &pool.edges[x];
                    !e.degenerate && e.curve.length(e.t[0].min(e.t[1]), e.t[0].max(e.t[1])) > 1e-3 && edge_on_surface(e, &g.surf, pool.tol)
                })
            };
            lies(&face_edges[fa], &faces[fb]) || lies(&face_edges[fb], &faces[fa])
        };
        // A curve running within the tolerance of another all along (branches of a near
        // tangent contact traced twice) is that curve again: the copy goes, else every point
        // where the two come within the tolerance would cut both.
        let curves: Vec<ssi::IntCurve> = {
            let near_c = (run_tol * 10.0).max(faces[fa].tol).max(faces[fb].tol);
            let mut kept: Vec<ssi::IntCurve> = Vec::new();
            for ic in curves {
                let bb = curve_bbox(&ic.c, ic.t0, ic.t1);
                let copy = kept.iter().any(|k| {
                    curve_bbox(&k.c, k.t0, k.t1).grow(near_c).overlaps(&bb)
                        && (0..=8).all(|q| on_curve(&k.c, k.t0.min(k.t1), k.t0.max(k.t1), &ic.c.point(ic.t0 + (ic.t1 - ic.t0) * q as f64 / 8.0), near_c).is_some())
                });
                if !copy {
                    kept.push(ic);
                }
            }
            kept
        };
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            eprintln!("ssi {fa}×{fb} {:?}×{:?}: {} curves in {el:?}", faces[fa].surf.kind(), faces[fb].surf.kind(), curves.len());
            for ic in &curves {
                eprintln!("    curve closed {} from {:?} to {:?}", ic.closed, ic.c.point(ic.t0), ic.c.point(ic.t1));
            }
        }
        if curves.is_empty() {
            continue;
        }
        let mut cand: Vec<usize> = boundary_verts(&pool, fa);
        for v in boundary_verts(&pool, fb).into_iter().chain(face_pts[fa].iter().copied()).chain(face_pts[fb].iter().copied()) {
            if !cand.contains(&v) {
                cand.push(v);
            }
        }
        // The pair's tolerance: the larger of the faces' (new vertices and edges carry it).
        let ft = faces[fa].tol.max(faces[fb].tol);
        let tf = run_tol.max(ft);
        // Where two curves of the pair cross or touch (a branch point: the surfaces are
        // tangent there) the faces' new edges meet, so both need a vertex: the SSI's own
        // such points (exact, each curve passing one runs through it), and crossings found
        // between the curves.
        for p in &singular {
            if ob.grow(tf * 10.0).contains(p) {
                let v = pool.vertex_tol(*p, ft);
                if !cand.contains(&v) {
                    cand.push(v);
                }
            }
        }
        // (Curves whose boxes are apart can't meet.)
        let cboxes: Vec<Bbox> = curves.iter().map(|ic| curve_bbox(&ic.c, ic.t0, ic.t1).grow(tf)).collect();
        for i in 0..curves.len() {
            for j in i + 1..curves.len() {
                if !cboxes[i].overlaps(&cboxes[j]) {
                    continue;
                }
                let (ci, cj) = (&curves[i], &curves[j]);
                for t in overlap_runs(&ci.c, &cj.c, ssi::curve_curve(&ci.c, ci.t0, ci.t1, &cj.c, cj.t0, cj.t1, tf), tf) {
                    let p = ci.c.point(t);
                    if ob.grow(tf * 10.0).contains(&p) {
                        let v = pool.vertex_tol(p, ft);
                        if !cand.contains(&v) {
                            cand.push(v);
                        }
                    }
                }
            }
        }
        for ic in curves {
            let mut ps: Vec<(f64, usize)> = Vec::new();
            for &v in &cand {
                let p = pool.verts[v];
                if !ob.grow(tf * 10.0).contains(&p) {
                    continue;
                }
                let mut t = param_on(&ic.c, ic.t0, ic.t1, &p);
                let on = (run_tol * 2.0).max(ft);
                if (ic.c.point(t) - p).norm() < on {
                    // A closed curve: the vertex at its start (within the tolerance, though
                    // its parameter may land a hair off where the curve runs along it) is
                    // also at its end.
                    if ic.closed && (ic.c.point(ic.t0) - p).norm() < on {
                        t = if (t - ic.t0).abs() <= (t - ic.t1).abs() { ic.t0 } else { ic.t1 };
                    }
                    ps.push((t, v));
                    if ic.closed && ((t - ic.t0).abs() < 1e-12 || (t - ic.t1).abs() < 1e-12) {
                        ps.push((if (t - ic.t0).abs() < 1e-12 { ic.t1 } else { ic.t0 }, v));
                    }
                }
            }
            // Sections only (booleans rely on the edge–face stage's crossings): where the curve
            // leaves either face (a crossing that stage missed, as on a B-spline face),
            // found by sampling the in-face status along it and bisecting each change, and
            // the open curve's own ends, cut it too.
            let sectioning = matches!(mode, Mode::Section);
            if sectioning {
                let inside = |t: f64| {
                    let p = ic.c.point(t);
                    point_in_face(&faces[fa], &p, tf) != InFace::Outside && point_in_face(&faces[fb], &p, tf) != InFace::Outside
                };
                let n = 64;
                let span = ic.t1 - ic.t0;
                let mut prev = inside(ic.t0);
                for k in 1..=n {
                    let t = ic.t0 + span * k as f64 / n as f64;
                    let cur = inside(t);
                    if cur != prev {
                        let (mut lo, mut hi) = (t - span / n as f64, t);
                        for _ in 0..50 {
                            let m = 0.5 * (lo + hi);
                            if inside(m) == prev {
                                lo = m;
                            } else {
                                hi = m;
                            }
                        }
                        let tc = 0.5 * (lo + hi);
                        let pc = ic.c.point(tc);
                        if !ps.iter().any(|(pt, v)| (pt - tc).abs() < 1e-9 * (1.0 + span.abs()) || (pool.verts[*v] - pc).norm() < tf * 10.0) {
                            let v = pool.vertex_tol(pc, ft);
                            ps.push((tc, v));
                        }
                    }
                    prev = cur;
                }
            }
            if sectioning && !ic.closed {
                for t in [ic.t0, ic.t1] {
                    if !ps.iter().any(|(pt, _)| (pt - t).abs() < 1e-12 * (1.0 + t.abs())) {
                        let v = pool.vertex_tol(ic.c.point(t), ft);
                        ps.push((t, v));
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
            // A closed curve that isn't periodic (a fitted B-spline) can't run past its end:
            // the interval that wraps becomes two pieces meeting at the curve's start.
            if ic.closed && !ic.c.is_periodic() {
                let period = ic.t1 - ic.t0;
                let mut split: Vec<(f64, f64, Option<usize>, Option<usize>)> = Vec::new();
                for (ta, tb, va, vb) in ivs {
                    if tb > ic.t1 + 1e-12 && ta < ic.t1 - 1e-12 {
                        let vend = pool.vertex_tol(ic.c.point(ic.t1), ft);
                        split.push((ta, ic.t1, va, Some(vend)));
                        if tb - period > ic.t0 + 1e-12 {
                            split.push((ic.t0, tb - period, Some(vend), vb));
                        }
                    } else if ta >= ic.t1 - 1e-12 {
                        split.push((ta - period, tb - period, va, vb));
                    } else {
                        split.push((ta, tb, va, vb));
                    }
                }
                ivs = split;
            }
            for (ta, tb, va, vb) in ivs {
                // Wrap periodic parameters past the end.
                let tm = (ta + tb) * 0.5;
                let pm = ic.c.point(tm);
                let ia = point_in_face(&faces[fa], &pm, tf);
                let ib = point_in_face(&faces[fb], &pm, tf);
                if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
                    eprintln!("  interval {ta}..{tb} at {pm:?}: in a {ia:?}, in b {ib:?}");
                }
                if ia == InFace::Outside || ib == InFace::Outside {
                    continue;
                }
                // A piece no longer than the tolerance is a point (two vertices of the inputs
                // closer than it, both on the curve): no edge.
                if (ic.c.point(ta) - ic.c.point(tb)).norm() <= tf && ic.c.length(ta.min(tb), ta.max(tb)) <= tf {
                    continue;
                }
                // Where the faces touch along an edge of one (a blend's spring edge on a wall it
                // is tangent to), the curve is that edge found again: the edge is already there,
                // and the curve would cut the faces at every vertex along it.
                let along_edge = |f: usize| {
                    [0.25, 0.75].iter().all(|k| point_in_face(&faces[f], &ic.c.point(ta + (tb - ta) * k), tf) == InFace::Boundary)
                };
                let tangent_at = |p: &P3| {
                    let (sa, sb) = (&faces[fa].surf, &faces[fb].surf);
                    let ((ua, va_), (ub, vb_)) = (sa.project(p, None), sb.project(p, None));
                    sa.normal(ua, va_).cross(&sb.normal(ub, vb_)).norm() < 1e-3
                };
                // (Also where they cross along it, the edge lying on the other surface: the edge
                // stages imprint it.)
                if ((ia == InFace::Boundary && along_edge(fa)) || (ib == InFace::Boundary && along_edge(fb))) && (edge_on_other || tangent_at(&pm)) {
                    continue;
                }
                let va = va.unwrap_or_else(|| pool.vertex_tol(ic.c.point(ta), ft));
                let vb = vb.unwrap_or(va);
                // Reuse an existing edge running there.
                let existing = (0..pool.edges.len()).find(|&e| {
                    let ed = &pool.edges[e];
                    if ed.degenerate || !ebox.get(e).is_none_or(|b| b.contains(&pm)) {
                        return false;
                    }
                    // Only an unsplit edge between exactly these vertices (pieces of split
                    // edges that coincide with the new edge are unified later).
                    let ends_ok = (ed.v == [va, vb] || ed.v == [vb, va]) && ed.splits.is_empty();
                    ends_ok && on_curve(&ed.curve, ed.t[0], ed.t[1], &pm, tf.max(ed.tol)).is_some()
                });
                let e = match existing {
                    Some(e) => e,
                    None => {
                        // Unwrap the parameter range for the curve (closed curves may run past
                        // t1).
                        let (ta, tb) = (ta, tb);
                        pool.edges.push(WEdge { curve: ic.c.clone(), t: [ta, tb], v: [va, vb], degenerate: false, tol: ft, splits: Vec::new() });
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
    prof.mark("face-face");
    if matches!(mode, Mode::Section) {
        // The new intersection edges, and edges of one body lying on a face of the other
        // (their parts inside that face).
        let mut out = Body::default();
        let mut vmap: HashMap<usize, usize> = HashMap::new();
        let mut add = |out: &mut Body, curve: &Curve, t: [f64; 2], v: [usize; 2]| {
            let vs = v.map(|x| *vmap.entry(x).or_insert_with(|| out.add_vertex(pool.verts[x])));
            out.add_edge(curve.clone(), t[0], t[1], vs[0], vs[1]);
        };
        for (fi, list) in on_face.iter().enumerate() {
            for &e in list {
                let ed = pool.edges[e].clone();
                // The edge's pieces between its split points, kept where inside the face.
                let mut ts: Vec<(f64, usize)> = vec![(ed.t[0], ed.v[0])];
                let mut sp = ed.splits.clone();
                sp.sort_by(|a, b| a.0.total_cmp(&b.0));
                ts.extend(sp);
                ts.push((ed.t[1], ed.v[1]));
                for w in ts.windows(2) {
                    let pm = ed.curve.point(0.5 * (w[0].0 + w[1].0));
                    if point_in_face(&faces[fi], &pm, run_tol) != InFace::Outside {
                        add(&mut out, &ed.curve, [w[0].0, w[1].0], [w[0].1, w[1].1]);
                    }
                }
            }
        }
        // An intersection curve running along one of those edges (a plane touching a torus
        // along the circle bounding it) is that edge, found again with ends that the tangency
        // leaves ill-defined: kept once, as the edge.
        let kept = out.edges.len();
        for &e in &new_edges {
            let ed = &pool.edges[e];
            let along = (0..kept).any(|o| {
                let oe = &out.edges[o];
                [0.25, 0.5, 0.75].iter().all(|&f| {
                    let p = ed.curve.point(ed.t[0] + (ed.t[1] - ed.t[0]) * f);
                    (oe.point(oe.curve.closest(&p, oe.t[0], oe.t[1])) - p).norm() < run_tol * 10.0
                })
            });
            if !along {
                add(&mut out, &ed.curve, ed.t, ed.v);
            }
        }
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            for (i, e) in out.edges.iter().enumerate() {
                eprintln!("section edge {i}: v {:?} t {:?} {:?} -> {:?} mid {:?}", e.v, e.t, e.point(e.t[0]), e.point(e.t[1]), e.point(0.5 * (e.t[0] + e.t[1])));
            }
        }
        return Ok(BoolOut { body: out, from: Vec::new() });
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
            pool.edges.push(WEdge { curve: ed.curve.clone(), t: [w[0].0, w[1].0], v: [w[0].1, w[1].1], degenerate: false, tol: ed.tol, splits: Vec::new() });
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
                        on_curve(&p.curve, p.t[0], p.t[1], &m, run_tol * 10.0).is_some()
                    });
                    if ok {
                        canon[g[j]] = g[i];
                    }
                }
            }
        }
    }
    // Edges lying on a face (not on its boundary) imprint it.
    for fi in 0..faces.len() {
        for &e in &on_face[fi].clone() {
            for &pe in &pieces[e] {
                let ed = &pool.edges[pe];
                if ed.degenerate {
                    continue;
                }
                let pm = ed.curve.point((ed.t[0] + ed.t[1]) * 0.5);
                if point_in_face(&faces[fi], &pm, run_tol) == InFace::Inside && !cuts[fi].contains(&pe) {
                    cuts[fi].push(pe);
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
                    // (Of a pair coinciding over part, only the pieces lying on fa's surface.)
                    if ed.degenerate || (partial.contains(&(fa, fb)) && !edge_on_surface(ed, &faces[fa].surf, run_tol * 10.0)) {
                        continue;
                    }
                    let pm = ed.curve.point((ed.t[0] + ed.t[1]) * 0.5);
                    if point_in_face(&faces[fa], &pm, run_tol) == InFace::Inside && !cuts[fa].contains(&pe) {
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
        Arc { edge: c, fwd, pc, t: ce.t, v, cut: a.cut, deg: ce.degenerate }
    };
    let arcs_of = |pool: &Pool, f: &WFace, e: usize, fwd: bool, pc: &Curve| -> Vec<Arc> {
        let ps: Vec<usize> = if e < pieces.len() { pieces[e].clone() } else { vec![e] };
        let mut out: Vec<Arc> = ps
            .iter()
            .map(|&p| {
                let ed = &pool.edges[p];
                let v = if fwd { [ed.v[0], ed.v[1]] } else { [ed.v[1], ed.v[0]] };
                canon_arc(pool, f, Arc { edge: p, fwd, pc: pc.clone(), t: ed.t, v, cut: false, deg: ed.degenerate })
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
                if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
                    for (k, t) in ed.t.iter().enumerate() {
                        let q = pc.point(*t);
                        let d = (f.surf.eval(q.x, q.y) - pool.verts[ed.v[k]]).norm();
                        let dc = (ed.curve.point(*t) - pool.verts[ed.v[k]]).norm();
                        if d > 1e-6 || dc > 1e-6 {
                            eprintln!("cut {c} end {k}: p-curve off by {d:e}, curve off by {dc:e}; {:?} t {:?} surf {:?}", ed.curve, ed.t, f.surf);
                        }
                    }
                }
                cut_arcs.push(Arc { edge: c, fwd: true, pc, t: ed.t, v: ed.v, cut: true, deg: false });
            }
        }
        if cut_arcs.is_empty() {
            subs.push((fi, boundary));
            continue;
        }
        let arcs = (boundary.iter().map(|l| l.len()).sum::<usize>() + cut_arcs.len()) as u64;
        work.spend(arcs * arcs, "face splitting")?;
        let res = split::split_face(&f.surf, !f.sense, &boundary, &cut_arcs);
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            eprintln!("face {fi} (owner {} orig {}): {} cuts -> {} faces {:?}", f.owner, f.orig, cut_arcs.len(), res.faces.len(), res.faces.iter().map(|l| l.len()).collect::<Vec<_>>());
        }
        for lps in res.faces {
            subs.push((fi, lps));
        }
    }
    prof.mark("split");
    // 5. Classify.
    let polys_a: Vec<Vec<Vec<P2>>> = a.faces.iter().map(|f| classify::face_polygons(a, f)).collect();
    let polys_b: Vec<Vec<Vec<P2>>> = b.faces.iter().map(|f| classify::face_polygons(b, f)).collect();
    // (Each body prepared once for the many points classified against it.)
    let (cls_a, cls_b) = (classify::Classifier::new(a, &polys_a), classify::Classifier::new(b, &polys_b));
    let mut keep: Vec<(usize, Vec<Vec<Arc>>, bool)> = Vec::new(); // (face, loops, flip)
    // Sub-faces of one body joined across an edge piece no cut runs along lie on one side of
    // the other body: grouped (union-find over shared uncut pieces), each group is classified
    // once and its answer shared (faces on a coincident face keep their own test).
    let mut parent: Vec<usize> = (0..subs.len()).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    {
        // Pieces touching the other body (cuts of any face, edges lying on its faces, edges
        // merged with its edges) can separate sides: they don't join.
        let mut touching: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for list in cuts.iter().chain(on_face.iter()) {
            for &e in list {
                touching.insert(e);
                touching.insert(canon[e.min(canon.len() - 1)]);
                if e < pieces.len() {
                    for &p in &pieces[e] {
                        touching.insert(p);
                        touching.insert(canon[p]);
                    }
                }
            }
        }
        for (e, &c) in canon.iter().enumerate() {
            if c != e {
                touching.insert(c);
                touching.insert(e);
            }
        }
        // Nor pieces ending where a cut ends (the other body meets this one there; within a
        // tolerance a crossing seen on one face can be missed on its neighbour).
        let cut_ends: std::collections::HashSet<usize> = subs.iter().flat_map(|(_, l)| l.iter().flatten()).filter(|a| a.cut).flat_map(|a| a.v).collect();
        let mut by_piece: HashMap<(usize, i64, i64, usize), usize> = HashMap::new();
        for (k, (fi, loops)) in subs.iter().enumerate() {
            for a in loops.iter().flatten().filter(|a| !a.cut && !a.deg && !touching.contains(&a.edge) && !a.v.iter().any(|v| cut_ends.contains(v))) {
                let (lo, hi) = (a.t[0].min(a.t[1]), a.t[0].max(a.t[1]));
                let key = (a.edge, (lo * 1e9).round() as i64, (hi * 1e9).round() as i64, faces[*fi].owner);
                match by_piece.get(&key) {
                    Some(&other) => {
                        let (x, y) = (root(&mut parent, k), root(&mut parent, other));
                        if x != y {
                            parent[x] = y;
                        }
                    }
                    None => {
                        by_piece.insert(key, k);
                    }
                }
            }
        }
    }
    let groups: Vec<usize> = (0..subs.len()).map(|k| root(&mut parent, k)).collect();
    // A group's class: (class, confirmed by a second member). A disagreement makes the group
    // classify every member itself.
    let mut group_class: HashMap<usize, (Class, bool)> = HashMap::new();
    let mut unreliable: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let propagate = std::env::var_os("CADRS_DEBUG_NO_PROPAGATE").is_none();
    work.spend(subs.len() as u64 * (a.faces.len() + b.faces.len()) as u64, "classification")?;
    for (sk, (fi, loops)) in subs.into_iter().enumerate() {
        let f = &faces[fi];
        if imprinting {
            if f.owner == 0 {
                keep.push((fi, loops, false));
            }
            continue;
        }
        let Some(uv) = interior_point(&loops) else { continue };
        let p = f.surf.eval(uv.x, uv.y);
        let n_out = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
        let mut class = None;
        for &g in &coincident[fi] {
            let gf = &faces[g];
            // (A face coinciding with g over part: where this sub-face lies on g's surface.)
            let on_g = !partial.contains(&(fi, g)) || gf.surf.distance(&p) <= (run_tol * 10.0).max(f.tol).max(gf.tol);
            if on_g && point_in_face(gf, &p, run_tol * 10.0) != InFace::Outside {
                let (gu, gv) = gf.surf.project(&p, Some((gf.uv_center.x, gf.uv_center.y)));
                let gn = gf.surf.normal(gu, gv) * if gf.sense { 1.0 } else { -1.0 };
                // Against a sheet an overlap is an overlap, whichever way the faces face.
                let other_sheet = if f.owner == 0 { b.solids.is_empty() } else { a.solids.is_empty() };
                class = Some(if gn.dot(&n_out) > 0.0 || other_sheet { Class::OnSame } else { Class::OnOpp });
                break;
            }
        }
        let g = groups[sk];
        let shared = match group_class.get(&g) {
            Some(&(c, true)) if class.is_none() && propagate && !unreliable.contains(&g) => Some(c),
            _ => None,
        };
        let fresh = class.is_none() && shared.is_none();
        let class = class.or(shared).unwrap_or_else(|| {
            let other = if f.owner == 0 { &cls_b } else { &cls_a };
            // A point on the other body's boundary (a sub-face whose sample landed on an edge of
            // the other body lying in it) decides nothing: try the face's other inside points.
            let mut pc = other.classify(&p, run_tol);
            if pc == PointClass::OnBoundary {
                for q in interior_points(&loops).into_iter().skip(1).take(12) {
                    let c = other.classify(&f.surf.eval(q.x, q.y), run_tol);
                    if c != PointClass::OnBoundary {
                        pc = c;
                        break;
                    }
                }
            }
            match pc {
                PointClass::Inside => Class::In,
                PointClass::Outside => Class::Out,
                PointClass::OnBoundary => {
                    // On the other body's boundary but not on a coincident face: decide by a
                    // point nudged inwards along the face.
                    let q = p - n_out * 1e-5;
                    if other.parity_inside(&q) { Class::In } else { Class::Out }
                }
            }
        });
        if fresh && matches!(class, Class::In | Class::Out) {
            match group_class.get(&g).copied() {
                None => {
                    group_class.insert(g, (class, false));
                }
                Some((c, false)) => {
                    if c == class {
                        group_class.insert(g, (c, true));
                    } else {
                        unreliable.insert(g);
                    }
                }
                Some((c, true)) => {
                    if c != class {
                        unreliable.insert(g);
                    }
                }
            }
        }
        let a_side = f.owner == 0;
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            eprintln!("sub of face {fi} (owner {} orig {}): {} loops, class {class:?} at {p:?}{}", f.owner, f.orig, loops.len(), if shared.is_some() { " (shared)" } else { "" });
        }
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
    prof.mark("classify");
    if keep.is_empty() {
        return Err(KernelError::EmptyResult("the result is empty".into()));
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
                if a.deg {
                    // A pole piece: its own degenerate edge over the arc's range.
                    let ed = &pool.edges[a.edge];
                    let v = *vmap.entry(ed.v[0]).or_insert_with(|| {
                        out.verts.push(super::topo::Vertex { p: pool.verts[ed.v[0]], tol: pool.vtols[ed.v[0]].max(super::topo::TOL_MIN) });
                        out.verts.len() - 1
                    });
                    out.edges.push(Edge { curve: ed.curve.clone(), t: a.t, v: [v, v], degenerate: true, tol: ed.tol.max(super::topo::TOL_MIN) });
                    nl.push(Coedge { edge: out.edges.len() - 1, fwd: a.fwd, pc: a.pc.clone() });
                    continue;
                }
                let ne = *emap.entry(a.edge).or_insert_with(|| {
                    let ed = &pool.edges[a.edge];
                    let mut vv = [0usize; 2];
                    for k in 0..2 {
                        vv[k] = *vmap.entry(ed.v[k]).or_insert_with(|| {
                            out.verts.push(super::topo::Vertex { p: pool.verts[ed.v[k]], tol: pool.vtols[ed.v[k]].max(super::topo::TOL_MIN) });
                            out.verts.len() - 1
                        });
                    }
                    out.edges.push(Edge { curve: ed.curve.clone(), t: ed.t, v: vv, degenerate: ed.degenerate, tol: ed.tol.max(super::topo::TOL_MIN) });
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
    if !imprinting && merge_coplanar {
        from = super::unify::unify_boolean(&mut out, from, flush);
    }
    prof.mark("assemble");
    super::unify::merge_edges(&mut out);
    // The run merged within its tolerance: the result's entities say so, the p-curves' gaps
    // searched for as the Full check measures them. An edge lying off a face by more (an
    // input's edge off its own face, which the cut faces keep) has that recorded, up to a part
    // in 10⁵ of the result's size.
    out.fit_tolerances_searched(run_tol * 10.0);
    let wide = 1e-5 * out.bbox().diagonal();
    for fi in 0..out.faces.len() {
        for c in out.faces[fi].loops.iter().flatten() {
            let e = &out.edges[c.edge];
            if e.degenerate {
                continue;
            }
            let g = super::topo::pcurve_gap(e, &c.pc, &out.faces[fi].surf);
            if g <= wide && g * 1.01 > e.tol {
                out.edges[c.edge].tol = g * 1.01;
            }
        }
    }
    prof.mark("unify");
    work.spend((out.faces.len() as u64).pow(2), "shell assembly")?;
    if std::env::var_os("CADRS_BOOL_WORK").is_some() {
        eprintln!("boolean work: {}", work.done);
    }
    assemble_shells(&mut out)?;
    touch_voids(&mut out, run_tol * 10.0);
    // Faces of the two inputs on one surface (by input face): only a solid holding both of a
    // pair can enclose nothing.
    let pairs: std::collections::HashSet<(usize, usize)> = (0..na_faces).flat_map(|fa| coincident[fa].iter().map(move |&fb| (fa, fb))).map(|(fa, fb)| (faces[fa].orig, faces[fb].orig)).collect();
    let (dropped, volumes) = drop_empty_solids(&mut out, &mut from, &pairs);
    if dropped && out.faces.is_empty() {
        return Err(KernelError::EmptyResult("the result is empty".into()));
    }
    super::unify::split_shared_vertices(&mut out);
    // A face whose loop's p-curves don't meet in (u, v) can't be closed: an error, never an
    // invalid body.
    if let Some(why) = uv_gap(&out) {
        return Err(KernelError::BooleanFailed { stage: "face closure", why: format!("the boolean built a face that doesn't close: {why}") });
    }
    // Nor any other invalid body ([`Body::check`]; the volumes of the solids that hold faces on
    // one surface known from above, the others' shells oriented by ray parity).
    if matches!(mode, Mode::Bool(_)) {
        let why = out.check_structure().into_iter().next().or_else(|| {
            let (si, v) = volumes.iter().enumerate().find_map(|(si, v)| v.filter(|v| !(*v > 0.0)).map(|v| (si, v)))?;
            Some(format!("solid {si} encloses a volume of {v:.3e} (its faces point in)"))
        });
        if let Some(why) = why {
            return Err(KernelError::BooleanFailed { stage: "validity", why: format!("the boolean's result is invalid: {why}") });
        }
    }
    // Solids in, faces out but no solid: some piece went missing.
    if !a.solids.is_empty() && !b.solids.is_empty() && !out.faces.is_empty() && out.solids.is_empty() {
        return Err(KernelError::OpenResult("the boolean left an open shell".into()));
    }
    prof.mark("shells");
    prof.report(&format!("{op:?} {}×{} faces", a.faces.len(), b.faces.len()));
    Ok(BoolOut { body: out, from })
}

/// A void's vertices lying outside its solid's outer shell by no more than `tol` (a void
/// touching the outer shell, as a cube's corners a sphere round it, within the run's
/// tolerance) record the gap in their tolerance: they lie on the shell.
fn touch_voids(b: &mut Body, tol: f64) {
    let mut raise: Vec<(usize, f64)> = Vec::new();
    for s in &b.solids {
        let [outer, voids @ ..] = s.shells.as_slice() else { continue };
        if voids.is_empty() {
            continue;
        }
        let out = super::kernel::subset(b, &b.shells[*outer].faces, true);
        let polys: Vec<Vec<Vec<P2>>> = out.faces.iter().map(|f| classify::face_polygons(&out, f)).collect();
        for &sh in voids {
            let mut vs: Vec<usize> = b.shells[sh].faces.iter().flat_map(|&f| b.faces[f].loops.iter().flatten().flat_map(|c| b.edges[c.edge].v)).collect();
            vs.sort_unstable();
            vs.dedup();
            for v in vs {
                let vx = &b.verts[v];
                if classify::classify_point(&out, &polys, &vx.p, vx.tol.max(super::tol::POINT)) != PointClass::Outside {
                    continue;
                }
                let d = out.faces.iter().map(|f| f.surf.distance(&vx.p)).fold(f64::INFINITY, f64::min);
                if d <= tol {
                    raise.push((v, d * 1.01));
                }
            }
        }
    }
    for (v, t) in raise {
        b.verts[v].tol = b.verts[v].tol.max(t);
    }
}

/// The points where two curves meet (`hits`: (t on `a`, t on `b`), as [`ssi::curve_curve`]
/// gives them) with each stretch where the curves run within `tol` of each other reduced to its
/// two ends: two intersection curves along one contact (pieces of a near tangent or partly
/// coincident contact) touch at every wiggle within the tolerance, and each touch would cut
/// both. Two hits are on one stretch when `a` stays within `tol` of `b` at three points between
/// them. The parameters on `a`, in order.
fn overlap_runs(a: &Curve, b: &Curve, mut hits: Vec<(f64, f64)>, tol: f64) -> Vec<f64> {
    hits.sort_by(|x, y| x.0.total_cmp(&y.0));
    let close = |s: f64, t: f64, s2: f64, t2: f64| {
        [0.25, 0.5, 0.75].iter().all(|k| {
            let p = a.point(s + (s2 - s) * k);
            let tb = b.closest(&p, t.min(t2), t.max(t2));
            (b.point(tb) - p).norm() <= 2.0 * tol
        })
    };
    let mut out: Vec<f64> = Vec::new();
    let mut k = 0;
    while k < hits.len() {
        let mut e = k;
        while e + 1 < hits.len() && close(hits[e].0, hits[e].1, hits[e + 1].0, hits[e + 1].1) {
            e += 1;
        }
        out.push(hits[k].0);
        if e > k {
            out.push(hits[e].0);
        }
        k = e + 1;
    }
    out
}

/// Drops the solids of `b` that enclose no volume (to 1e-9 of the body's box, cubed): a closed
/// shell of faces of the two inputs that met face to face, both kept, is a sheet-thin sliver,
/// not a solid, as are two faces closing on each other. Only solids of a few faces, or holding
/// faces from both faces of a pair in `pairs` (input faces of the two bodies on one surface:
/// (face of a, face of b)), are measured. Their faces go too (and
/// their entries in `from`). Returns whether any went, and the volumes of the solids left
/// (None: not measured).
fn drop_empty_solids(b: &mut Body, from: &mut Vec<Vec<(usize, usize)>>, pairs: &std::collections::HashSet<(usize, usize)>) -> (bool, Vec<Option<f64>>) {
    let size = b.bbox().diagonal();
    let suspect = |si: usize| {
        let faces = b.solid_faces(si);
        // (A few faces are cheap to measure: two faces closing on each other are a sliver too.)
        if faces.len() <= 8 {
            return true;
        }
        let origins: Vec<(usize, usize)> = faces.iter().flat_map(|&f| from.get(f).into_iter().flatten().copied()).collect();
        origins.iter().filter(|o| o.0 == 0).any(|&(_, x)| origins.iter().any(|&(w, y)| w == 1 && pairs.contains(&(x, y))))
    };
    let volumes: Vec<Option<f64>> = (0..b.solids.len())
        .map(|si| suspect(si).then(|| super::mass::body_mass(&super::kernel::subset(b, &b.solid_faces(si), true)).volume))
        .collect();
    let empty: Vec<usize> = (0..b.solids.len()).filter(|&si| volumes[si].is_some_and(|v| v.abs() <= 1e-9 * size * size * size)).collect();
    if empty.is_empty() {
        return (false, volumes);
    }
    let left: Vec<Option<f64>> = (0..b.solids.len()).filter(|si| !empty.contains(si)).map(|si| volumes[si]).collect();
    let mut dead = vec![false; b.faces.len()];
    for &si in &empty {
        for f in b.solid_faces(si) {
            dead[f] = true;
        }
    }
    let mut fmap = vec![usize::MAX; b.faces.len()];
    let mut faces = Vec::new();
    let mut kept_from = Vec::new();
    for (fi, f) in std::mem::take(&mut b.faces).into_iter().enumerate() {
        if !dead[fi] {
            fmap[fi] = faces.len();
            faces.push(f);
            kept_from.push(from.get(fi).cloned().unwrap_or_default());
        }
    }
    b.faces = faces;
    *from = kept_from;
    let dead_shells: std::collections::HashSet<usize> = empty.iter().flat_map(|&si| b.solids[si].shells.clone()).collect();
    let mut smap = vec![usize::MAX; b.shells.len()];
    let mut shells = Vec::new();
    for (si, mut sh) in std::mem::take(&mut b.shells).into_iter().enumerate() {
        if dead_shells.contains(&si) {
            continue;
        }
        sh.faces = sh.faces.iter().filter(|&&f| !dead[f]).map(|&f| fmap[f]).collect();
        smap[si] = shells.len();
        shells.push(sh);
    }
    b.shells = shells;
    let solids = std::mem::take(&mut b.solids);
    b.solids = solids
        .into_iter()
        .enumerate()
        .filter(|(si, _)| !empty.contains(si))
        .map(|(_, mut so)| {
            so.shells = so.shells.iter().map(|&s| smap[s]).collect();
            so
        })
        .collect();
    super::unify::compact(b);
    (true, left)
}

/// Splits edges used by more than two faces (solids touching along an edge) into one edge per
/// pair of faces bounding the same material wedge, pairing the faces in their order around
/// the edge.
pub fn split_nonmanifold(b: &mut Body) {
    let ne = b.edges.len();
    let mut uses: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); ne]; // (face, loop, index)
    for (fi, f) in b.faces.iter().enumerate() {
        for (li, l) in f.loops.iter().enumerate() {
            for (k, c) in l.iter().enumerate() {
                uses[c.edge].push((fi, li, k));
            }
        }
    }
    for e in 0..ne {
        let u = uses[e].clone();
        if u.len() <= 2 || b.edges[e].degenerate {
            continue;
        }
        let ed = b.edges[e].clone();
        let tm = ed.mid_t();
        let p = ed.point(tm);
        let t = ed.curve.tangent(tm).normalize();
        let a = any_perpendicular(&t);
        let bb = t.cross(&a);
        // Each use: its angle about t and whether the material lies counter-clockwise of it.
        let mut items: Vec<(f64, bool, (usize, usize, usize))> = Vec::new();
        for &(fi, li, k) in &u {
            let f = &b.faces[fi];
            let c = &f.loops[li][k];
            let uv = c.pc.point(tm);
            let n = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
            let d = t * if c.fwd { 1.0 } else { -1.0 };
            let w0 = n.cross(&d); // into the face (first order)
            // A point a little way into the face, so faces tangent at the edge separate.
            let (_, duv, _) = c.pc.d2(tm);
            let perp = V2::new(-duv.y, duv.x);
            let len = (ed.t[1] - ed.t[0]).abs() * ed.curve.tangent(tm).norm();
            // (Not `clamp`: a NaN length takes 0.05 here.)
            #[allow(clippy::manual_clamp)]
            let step = (len * 1e-2).min(0.05).max(1e-4);
            let (pu, su, sv) = f.surf.d1(uv.x, uv.y);
            let g = su * perp.x + sv * perp.y;
            let sgn = if g.dot(&w0) >= 0.0 { 1.0 } else { -1.0 };
            let scale = step / g.norm().max(1e-300);
            let q = f.surf.eval(uv.x + perp.x * scale * sgn, uv.y + perp.y * scale * sgn);
            let w = {
                let r = q - pu;
                let r = r - t * r.dot(&t);
                if r.norm() > 1e-12 { r.normalize() } else { w0 }
            };
            let ang = w.dot(&bb).atan2(w.dot(&a));
            let ccw = t.cross(&w0);
            let mat_ccw = (-n).dot(&ccw) > 0.0;
            items.push((ang, mat_ccw, (fi, li, k)));
        }
        let _ = p;
        items.sort_by(|x, y| x.0.total_cmp(&y.0));
        let m = items.len();
        // Pair each face whose material is counter-clockwise with the next one.
        type Use = (usize, usize, usize);
        let mut pairs: Vec<(Use, Use)> = Vec::new();
        for i in 0..m {
            if items[i].1 {
                let j = (i + 1) % m;
                pairs.push((items[i].2, items[j].2));
            }
        }
        if pairs.len() < 2 {
            continue;
        }
        for (pi, (x, y)) in pairs.into_iter().enumerate() {
            if pi == 0 {
                continue;
            }
            b.edges.push(ed.clone());
            let nid = b.edges.len() - 1;
            for (fi, li, k) in [x, y] {
                b.faces[fi].loops[li][k].edge = nid;
            }
        }
    }
}

/// Groups faces into shells by shared edges and shells into solids (outer shells and voids).
pub fn assemble_shells(b: &mut Body) -> Result<()> {
    split_nonmanifold(b);
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
            outer.push((si, f64::NAN));
        }
    }
    // With several closed shells, tell outer shells from voids: whether a shell's faces point
    // out of what it encloses (rays leaving it along a face normal cross it an even number of
    // times). Each is sized by its box, for picking the smallest one around a void. (Their
    // volumes tell the same, but integrating faces cut by long intersection curves is slow.)
    if outer.len() > 1 {
        let polys: Vec<Vec<Vec<P2>>> = b.faces.iter().map(|f| classify::face_polygons(b, f)).collect();
        let all: Vec<(usize, f64, bool)> = outer
            .iter()
            .map(|(si, _)| {
                let fs = &b.shells[*si].faces;
                let mut bb = Bbox::empty();
                for &f in fs {
                    for l in &b.faces[f].loops {
                        for c in l {
                            let e = &b.edges[c.edge];
                            bb.add(&e.point(e.t[0]));
                            bb.add(&e.point(0.5 * (e.t[0] + e.t[1])));
                        }
                    }
                }
                let out = shell_points_out(b, fs, &polys).unwrap_or_else(|| {
                    let shift = b.bbox().center().coords;
                    fs.iter().map(|&f| super::mass::face_integrals(b, &b.faces[f], &shift)[4]).sum::<f64>() >= 0.0
                });
                (*si, bb.diagonal(), out)
            })
            .collect();
        outer.clear();
        for (si, size, out) in all {
            if out {
                outer.push((si, size));
            } else {
                voids.push((si, size));
            }
        }
    }
    let open = b.shells.iter().any(|s| !s.closed);
    if open && !outer.is_empty() {
        return Err(KernelError::OpenResult("the boolean left an open shell".into()));
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
            if classify::parity_inside(&sub, &polys, &p) && best.is_none_or(|(bv, _)| *vol < bv) {
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

/// Whether the faces of a closed shell point out of the region it encloses, by ray parity:
/// from points inside a few of its faces, rays along the outward normal (and two tilts of it)
/// cross the shell an even number of times when they do. None when the votes don't settle it.
fn shell_points_out(b: &Body, faces: &[usize], polys: &[Vec<Vec<P2>>]) -> Option<bool> {
    let size = b.bbox().diagonal().max(1e-9);
    let (mut out, mut inward) = (0, 0);
    for &f in faces.iter().take(3) {
        let fc = &b.faces[f];
        let Some(uv) = interior_of_polys(&polys[f]) else { continue };
        let p = fc.surf.eval(uv.x, uv.y);
        let n = fc.surf.normal(uv.x, uv.y) * if fc.sense { 1.0 } else { -1.0 };
        let (t1, t2) = (any_perpendicular(&n), n.cross(&any_perpendicular(&n)));
        for d in [n, (n + t1 * 0.3).normalize(), (n + t2 * 0.3).normalize()] {
            let hits = classify::ray_hits(b, polys, &p, &d);
            let k = hits.iter().filter(|(t, hf, _)| *t > 1e-7 * size && faces.contains(hf)).count();
            if k % 2 == 0 {
                out += 1;
            } else {
                inward += 1;
            }
        }
    }
    match out.cmp(&inward) {
        std::cmp::Ordering::Greater => Some(true),
        std::cmp::Ordering::Less => Some(false),
        std::cmp::Ordering::Equal => None,
    }
}

/// The kernel operation: the target combined with each tool in turn.
pub fn boolean(k: &mut NativeKernel, op: BoolOp, target: BodyId, tools: &[BodyId]) -> Result<OpResult> {
    boolean_with(k, op, target, tools, true)
}

/// [`boolean`], merging faces on one surface afterwards or not (`merge_coplanar`, as in
/// [`BoolOptions`]).
pub fn boolean_with(k: &mut NativeKernel, op: BoolOp, target: BodyId, tools: &[BodyId], merge_coplanar: bool) -> Result<OpResult> {
    let opts = BoolOptions { tol: k.tolerances(), merge_coplanar, ..BoolOptions::default() };
    // Sheets, wires and points: their own rules (`boolean_kinds`).
    if let Some(r) = super::boolean_kinds::boolean(k, op, target, tools, opts) {
        return r;
    }
    let mut cur = (**k.get(target)?).clone();
    // For each face of `cur`: where it came from (body, face).
    let mut origin: Vec<Vec<(BodyId, usize)>> = (0..cur.faces.len()).map(|i| vec![(target, i)]).collect();
    for &t in tools {
        let tb = (**k.get(t)?).clone();
        let r = boolean_bodies_with(&cur, &tb, op, opts)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A boolean past its work budget fails (deterministically, at the same stage every
    /// time), and the default budget holds a plain one easily.
    #[test]
    fn work_budget() {
        let cube = |x: f64| {
            let mut k = NativeKernel::new();
            let super::super::draw::Shape::Body(id) = k.box_at(x, 0.0, 0.0, 1.0, 1.0, 1.0).unwrap() else { unreachable!() };
            (**k.get(id).unwrap()).clone()
        };
        let (a, b) = (cube(0.0), cube(0.5));
        assert!(boolean_bodies(&a, &b, BoolOp::Union).is_ok());
        let tight = BoolOptions { work_budget: 10, ..BoolOptions::default() };
        let fail = || boolean_bodies_with(&a, &b, BoolOp::Union, tight).err().map(|e| e.to_string());
        let err = fail().unwrap_or_default();
        assert!(err.contains("work budget"), "{err}");
        assert_eq!(fail(), Some(err));
    }

    /// Curves running along each other within the tolerance meet in one stretch, its two ends,
    /// however many touches the intersection reports along it; separate crossings stay.
    #[test]
    fn overlap_runs_keep_the_ends() {
        let a = Curve::Line { o: P3::origin(), d: V3::x() };
        let b = Curve::Line { o: P3::new(0.0, 4e-7, 0.0), d: V3::x() };
        assert_eq!(overlap_runs(&a, &b, vec![(3.0, 3.0), (1.0, 1.0), (2.0, 2.0)], 1e-6), vec![1.0, 3.0]);
        let c = Curve::Line { o: P3::new(0.0, -1.0, 0.0), d: V3::new(1.0, 1.0, 0.0) };
        assert_eq!(overlap_runs(&a, &c, vec![(1.0, 1.0), (5.0, 5.0)], 1e-6), vec![1.0, 5.0]);
        assert_eq!(overlap_runs(&a, &c, vec![(1.0, 1.0)], 1e-6), vec![1.0]);
    }

    /// Booleans merge faces on one surface by default, and keep them apart with
    /// `merge_coplanar: false` (the DRAW harness's, as OCCT's).
    #[test]
    fn merge_coplanar_option() {
        let cube = |x: f64| {
            let mut k = NativeKernel::new();
            let super::super::draw::Shape::Body(id) = k.box_at(x, 0.0, 0.0, 1.0, 1.0, 1.0).unwrap() else { unreachable!() };
            (**k.get(id).unwrap()).clone()
        };
        let (a, b) = (cube(0.0), cube(1.0));
        let merged = boolean_bodies(&a, &b, BoolOp::Union).unwrap().body;
        assert_eq!(merged.faces.len(), 6);
        let apart = boolean_bodies_with(&a, &b, BoolOp::Union, BoolOptions { merge_coplanar: false, ..BoolOptions::default() }).unwrap().body;
        assert_eq!(apart.faces.len(), 10);
    }
}
