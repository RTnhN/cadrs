//! Silhouette curves of faces (SMLib's `SmSurface::CreateSilhouetteCurves` followed by
//! `SmFace::CreateCurvesByTrimming`): the curves on a face where its surface turns from facing
//! the eye to facing away, for a parallel view along a direction `d` (n·d = 0) or a perspective
//! view from an eye point `e` (n·(p − e) = 0). Every face's whole silhouette is found, seen or
//! hidden; hidden-line removal ([`super::hlr`]) decides what is drawn.
//!
//! - Planes have none, unless seen edge-on (the eye in the plane). A face seen edge-on lies
//!   wholly on its silhouette; as SMLib (whose silhouette of such a surface is its uv
//!   rectangle's sides, then trimmed by the face), the curves given are the sides of the face's
//!   uv box where they lie in the face: a rectangle's four sides, nothing of a disc.
//! - Cylinders and cones: n·d (and n·(p − e), the tangent plane holding the axis direction or
//!   the apex) is constant along a generator and `A cos u + B sin u + C` round the axis, fitted
//!   from three normals; the silhouettes are the (zero, one or two) generators where it
//!   vanishes, exact lines. A cylinder seen along its axis lies wholly on its silhouette.
//! - Spheres: the great circle square to the direction of sight; from an eye point at distance
//!   δ > r from the centre, the circle where the tangent cone touches (radius r·√(δ² − r²)/δ,
//!   its centre r²/δ from the sphere's towards the eye). Exact circles.
//! - Tori seen along their axis: the outer and inner equators, exact circles.
//! - Anything else (tori, revolutions, extrusions, B-splines): the zero set of
//!   g = n̂·ŵ (ŵ the unit direction of sight at the point) traced on a uv grid by marching
//!   squares. Each crossing of a grid edge is bisected onto the exact zero; where two branches
//!   cross (a torus seen side-on: its top circle meets its meridians) the cell's saddle point
//!   is found by Newton on ∇g = 0 and the branches meet there; then each polyline is refined,
//!   midpoints pulled onto g = 0 by Newton along ∇g, until its chords are within the
//!   tolerance. A direction in which the face spans the whole period of a closed surface
//!   (periodic, or a B-spline whose ends meet) wraps round, shifted off the seam by a fraction
//!   of a cell, so a silhouette lying on the seam (a torus's meridian seen side-on) is traced
//!   once, inside cells.
//!
//! Every curve is clipped to the face's trimmed domain ([`FaceDomain`]).

use std::collections::HashMap;

use rayon::prelude::*;

use super::curve::Curve;
use super::math::*;
use super::surface::Surface;
use super::topo::Body;
use super::uvdomain::FaceDomain;
use super::tol;

/// How a face is seen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    /// Along a direction (a drawing view).
    Parallel(V3),
    /// From an eye point.
    Perspective(P3),
}

impl View {
    /// The unit direction of sight at `p` (from the eye towards `p`); zero at the eye.
    pub fn sight(&self, p: &P3) -> V3 {
        match self {
            View::Parallel(d) => d.normalize(),
            View::Perspective(e) => {
                let w = p - e;
                let l = w.norm();
                if l > tol::TINY { w / l } else { V3::zeros() }
            }
        }
    }
}

/// A silhouette curve's geometry.
#[derive(Debug, Clone)]
pub enum SilGeom {
    /// An exact curve over a parameter range (generators, circles, edges).
    Exact { curve: Curve, t: (f64, f64) },
    /// A polyline of points on the silhouette, its chords within the tolerance, and their
    /// (u, v).
    Poly { pts: Vec<P3>, uv: Vec<P2> },
}

/// One silhouette curve of a face.
#[derive(Debug, Clone)]
pub struct Silhouette {
    /// The face's index in its body.
    pub face: usize,
    pub geom: SilGeom,
    /// A whole closed curve (a full circle, or a polyline ending where it starts).
    pub closed: bool,
}

impl Silhouette {
    /// The curve's length.
    pub fn length(&self) -> f64 {
        match &self.geom {
            SilGeom::Exact { curve, t } => curve.length(t.0, t.1),
            SilGeom::Poly { pts, .. } => pts.windows(2).map(|w| (w[1] - w[0]).norm()).sum(),
        }
    }

    /// Points along the curve, its chords within `tol`, with their (u, v) on `surf` (the
    /// face's surface).
    pub fn samples(&self, surf: &Surface, tol: f64) -> (Vec<P3>, Vec<P2>) {
        match &self.geom {
            SilGeom::Exact { curve, t } => {
                let pts: Vec<P3> = curve.sample_params(t.0, t.1, tol, 0.1).into_iter().map(|s| curve.point(s)).collect();
                let mut hint = None;
                let uv = pts
                    .iter()
                    .map(|p| {
                        let (u, v) = surf.project(p, hint.or_else(|| Some(surf.project_coarse(p, 16))));
                        hint = Some((u, v));
                        P2::new(u, v)
                    })
                    .collect();
                (pts, uv)
            }
            SilGeom::Poly { pts, uv } => (pts.clone(), uv.clone()),
        }
    }
}

/// How finely to trace.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SilOptions {
    /// Largest distance (mm) of a traced polyline's chords from the silhouette.
    pub tol: f64,
    /// Cells of the tracing grid along the face's longer direction.
    pub grid: usize,
}

impl Default for SilOptions {
    fn default() -> Self {
        Self { tol: 1e-4, grid: 160 }
    }
}

/// What a face's surface gives over its domain.
pub enum Found {
    /// Curves (with whether each is closed).
    Curves(Vec<(SilGeom, bool)>),
    /// The face lies wholly on its silhouette (seen edge-on).
    Whole,
}

/// The silhouettes of every face of `body` (`polys`: its faces' uv polygons).
pub fn body_silhouettes(body: &Body, polys: &[Vec<Vec<P2>>], view: &View, opts: &SilOptions) -> Vec<Silhouette> {
    (0..body.faces.len()).into_par_iter().flat_map_iter(|fi| face_silhouettes(body, fi, &polys[fi], view, opts)).collect()
}

/// The silhouettes of face `fi` of `body` (`polys`: its uv polygons).
pub fn face_silhouettes(body: &Body, fi: usize, polys: &[Vec<P2>], view: &View, opts: &SilOptions) -> Vec<Silhouette> {
    let dom = FaceDomain::new(&body.faces[fi].surf, polys);
    let curves = match domain_silhouettes(&dom, view, opts) {
        Found::Curves(cs) => cs,
        Found::Whole => box_sides(&dom),
    };
    curves.into_iter().map(|(geom, closed)| Silhouette { face: fi, geom, closed }).collect()
}

/// The sides of the face's uv box, as iso-curves, where they lie in the face (a seen edge-on
/// face's silhouette, as SMLib gives it). Of two sides a period apart, one.
fn box_sides(dom: &FaceDomain) -> Vec<(SilGeom, bool)> {
    let s = dom.surf;
    let ((u0, u1), (v0, v1)) = (dom.ub, dom.vb);
    let full = |a: f64, b: f64, periodic: bool| periodic && ((b - a) - TAU).abs() < tol::ANGULAR;
    let mut out = Vec::new();
    let mut side = |fixed_u: bool, at: f64| {
        let Some(c) = super::face_build::iso_curve(s, fixed_u, at) else { return };
        let (t0, t1) = if fixed_u { (v0, v1) } else { (u0, u1) };
        let inside = |t: f64| if fixed_u { dom.contains(at, t, false) } else { dom.contains(t, at, false) };
        // Runs where a side only touches the face (a disc's box) are the boundary band's width.
        for run in clip(t0, t1, 64, inside).into_iter().filter(|r| r.1 - r.0 > 1e-2 * (t1 - t0)) {
            out.push((SilGeom::Exact { curve: c.clone(), t: run }, false));
        }
    };
    side(false, v0);
    if !full(v0, v1, s.v_periodic()) {
        side(false, v1);
    }
    side(true, u0);
    if !full(u0, u1, s.u_periodic()) {
        side(true, u1);
    }
    out
}

/// The silhouettes of a surface over a face's domain.
pub fn domain_silhouettes(dom: &FaceDomain, view: &View, opts: &SilOptions) -> Found {
    if dom.is_empty() {
        return Found::Curves(vec![]);
    }
    match dom.surf {
        Surface::Plane { f } => {
            let n = f.x.cross(&f.y).normalize();
            let edge_on = match view {
                View::Parallel(d) => n.dot(&d.normalize()).abs() < tol::ANGULAR,
                View::Perspective(e) => n.dot(&(f.o - e)).abs() < tol::REL * (1.0 + (f.o - e).norm()),
            };
            if edge_on { Found::Whole } else { Found::Curves(vec![]) }
        }
        Surface::Cylinder { .. } | Surface::Cone { .. } => generators(dom, view),
        Surface::Sphere { f, r } => {
            let (c, x, y, rad) = match view {
                View::Parallel(d) => {
                    let d = d.normalize();
                    let a = any_perpendicular(&d);
                    (f.o, a, d.cross(&a).normalize(), *r)
                }
                View::Perspective(e) => {
                    let w = e - f.o;
                    let dd = w.norm();
                    if dd <= r * (1.0 + 1e-12) {
                        return Found::Curves(vec![]);
                    }
                    let wn = w / dd;
                    let a = any_perpendicular(&wn);
                    (f.o + wn * (r * r / dd), a, wn.cross(&a).normalize(), r * (dd * dd - r * r).sqrt() / dd)
                }
            };
            Found::Curves(exact_runs(dom, Curve::Circle { c, x, y, r: rad }, (0.0, TAU), true))
        }
        Surface::Torus { f, .. } if matches!(view, View::Parallel(d) if d.normalize().dot(&f.z).abs() > 1.0 - 1e-12) => {
            let mut out = Vec::new();
            for v in [0.0, PI] {
                if let Some(c) = dom.surf.iso_v(v) {
                    out.extend(exact_runs(dom, c, (0.0, TAU), true));
                }
            }
            Found::Curves(out)
        }
        _ => Tracer::new(dom, view, opts).trace(),
    }
}

/// Generators of a cylinder or cone where n·d (n·(p − e)) vanishes.
fn generators(dom: &FaceDomain, view: &View) -> Found {
    let s = dom.surf;
    let vm = 0.5 * (dom.vb.0 + dom.vb.1);
    // Constant along a generator, A cos u + B sin u + C round the axis (the raw n·(p − e), not
    // divided by the distance to the eye, keeps that form).
    let g = |u: f64| {
        let n = s.normal(u, vm);
        match view {
            View::Parallel(d) => n.dot(&d.normalize()),
            View::Perspective(e) => n.dot(&(s.eval(u, vm) - e)),
        }
    };
    let scale = match view {
        View::Parallel(_) => 1.0,
        View::Perspective(e) => 1.0 + (s.eval(0.0, vm) - e).norm(),
    };
    let (g0, g1, g2) = (g(0.0), g(PI / 2.0), g(PI));
    let c = 0.5 * (g0 + g2);
    let (a, b) = (0.5 * (g0 - g2), g1 - c);
    let r = a.hypot(b);
    if r < tol::REL * scale {
        return if c.abs() < tol::REL * scale { Found::Whole } else { Found::Curves(vec![]) };
    }
    if c.abs() > r {
        return Found::Curves(vec![]);
    }
    let phi = b.atan2(a);
    let alpha = (-c / r).clamp(-1.0, 1.0).acos();
    let mut us = vec![phi + alpha];
    if alpha > tol::ANGULAR && alpha < PI - tol::ANGULAR {
        us.push(phi - alpha);
    }
    let mut out = Vec::new();
    for u0 in us {
        let Some(line) = s.iso_u(u0) else { continue };
        for run in clip(dom.vb.0, dom.vb.1, 32, |v| dom.contains(u0, v, false)) {
            out.push((SilGeom::Exact { curve: line.clone(), t: run }, false));
        }
    }
    Found::Curves(out)
}

/// The runs of an exact curve over `t` lying in the face (found through the surface's
/// projection); a periodic curve's runs joined round its start.
fn exact_runs(dom: &FaceDomain, c: Curve, t: (f64, f64), periodic: bool) -> Vec<(SilGeom, bool)> {
    let inside = |s: f64| {
        let (u, v) = dom.surf.project(&c.point(s), None);
        dom.contains(u, v, false)
    };
    let mut runs = clip(t.0, t.1, 64, inside);
    if periodic && runs.len() >= 2 && runs[0].0 == t.0 && runs[runs.len() - 1].1 == t.1 {
        let last = runs.remove(runs.len() - 1);
        runs[0] = (last.0, runs[0].1 + (t.1 - t.0));
    }
    let full = periodic && runs.len() == 1 && runs[0] == t;
    runs.into_iter().map(|run| (SilGeom::Exact { curve: c.clone(), t: run }, full)).collect()
}

/// The runs of [a, b] where `inside` holds, found on `n` samples and refined by bisection.
pub(crate) fn clip(a: f64, b: f64, n: usize, inside: impl Fn(f64) -> bool) -> Vec<(f64, f64)> {
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

/// One direction of the tracing grid.
#[derive(Debug, Clone, Copy)]
struct Axis {
    /// The first node and the step between nodes.
    lo: f64,
    step: f64,
    nodes: usize,
    /// Wraps round a whole period starting at `base` (the last cell joins the last node to the
    /// first).
    wrap: bool,
    base: f64,
    period: f64,
}

impl Axis {
    /// `cells` cells over [a, b]: wrapping round when `wrap`, else a cell past each end when the
    /// surface goes on beyond (`extend`), else exactly over it.
    fn new(a: f64, b: f64, cells: usize, wrap: bool, extend: bool) -> Self {
        let step = (b - a) / cells as f64;
        let (lo, nodes) = if wrap {
            (a + 0.381_966 * step, cells)
        } else if extend {
            (a - 0.618_034 * step, cells + 2)
        } else {
            (a, cells + 1)
        };
        Self { lo, step, nodes, wrap, base: a, period: b - a }
    }

    fn at(&self, i: usize) -> f64 {
        self.lo + self.step * i as f64
    }

    fn cells(&self) -> usize {
        if self.wrap { self.nodes } else { self.nodes - 1 }
    }

    fn next(&self, i: usize) -> usize {
        if self.wrap { (i + 1) % self.nodes } else { i + 1 }
    }

    /// `x` brought into the period (wrapping axes only).
    fn fold(&self, x: f64) -> f64 {
        if self.wrap { self.base + (x - self.base).rem_euclid(self.period) } else { x }
    }

    /// `b − a` the short way round.
    fn delta(&self, a: f64, b: f64) -> f64 {
        let d = b - a;
        if self.wrap { d - self.period * (d / self.period).round() } else { d }
    }
}

/// Traces the zero set of n̂·ŵ on a face (see the module docs).
struct Tracer<'a> {
    dom: &'a FaceDomain<'a>,
    view: View,
    tol: f64,
    au: Axis,
    av: Axis,
}

impl<'a> Tracer<'a> {
    fn new(dom: &'a FaceDomain<'a>, view: &View, opts: &SilOptions) -> Self {
        let s = dom.surf;
        let (ub, vb) = (dom.ub, dom.vb);
        // Lengths across the face along u and v, for cells of about equal size.
        let along = |f: &dyn Fn(f64, f64) -> P3| -> f64 {
            (0..3)
                .map(|k| {
                    let c = 0.2 + 0.3 * k as f64;
                    (0..16).map(|i| (f(i as f64 / 16.0, c) - f((i + 1) as f64 / 16.0, c)).norm()).sum::<f64>()
                })
                .fold(0.0, f64::max)
        };
        let lu = along(&|a, c| s.eval(ub.0 + (ub.1 - ub.0) * a, vb.0 + (vb.1 - vb.0) * c));
        let lv = along(&|a, c| s.eval(ub.0 + (ub.1 - ub.0) * c, vb.0 + (vb.1 - vb.0) * a));
        let lmax = lu.max(lv).max(tol::TINY);
        let grid = opts.grid.max(16) as f64;
        let cells = |l: f64| ((grid * l / lmax).ceil() as usize).clamp(16, opts.grid.max(16));
        let size = s.patch_bbox(ub, vb).diagonal().max(tol::ROUNDOFF);
        let (wu, wv) = closed_dirs(s, ub, vb, size);
        let au = Axis::new(ub.0, ub.1, cells(lu), wu, !wu && s.u_periodic());
        let av = Axis::new(vb.0, vb.1, cells(lv), wv, !wv && s.v_periodic());
        Self { dom, view: *view, tol: opts.tol.max(tol::ROUNDOFF), au, av }
    }

    /// The point at (u, v), folded into the period.
    fn eval(&self, q: P2) -> P3 {
        self.dom.surf.eval(self.au.fold(q.x), self.av.fold(q.y))
    }

    /// n̂·ŵ at (u, v).
    fn g(&self, q: P2) -> f64 {
        let s = self.dom.surf;
        let (u, v) = (self.au.fold(q.x), self.av.fold(q.y));
        let d = s.d2(u, v);
        let n = d[1].cross(&d[2]);
        let l = n.norm();
        let scale = (d[1].norm() * d[2].norm()).max(d[1].norm_squared()).max(d[2].norm_squared());
        let n = if l > tol::REL * scale && l > tol::TINY {
            n / l
        } else {
            // A pole or apex (a collapsed row of a B-spline too): the normal just inside the face,
            // a step towards the middle of its box.
            let (cu, cv) = (0.5 * (self.dom.ub.0 + self.dom.ub.1), 0.5 * (self.dom.vb.0 + self.dom.vb.1));
            let (qu, qv) = (u + (cu - u) * 1e-6, v + (cv - v) * 1e-6);
            let e = s.d2(qu, qv);
            let m = e[1].cross(&e[2]);
            if m.norm() > tol::TINY { m.normalize() } else { s.normal(u, v) }
        };
        n.dot(&self.view.sight(&P3::from(d[0])))
    }

    /// The midpoint of a and b (the short way round on wrapping axes).
    fn mid(&self, a: P2, b: P2) -> P2 {
        P2::new(self.au.fold(a.x + 0.5 * self.au.delta(a.x, b.x)), self.av.fold(a.y + 0.5 * self.av.delta(a.y, b.y)))
    }

    /// b − a the short way round.
    fn diff(&self, a: P2, b: P2) -> V2 {
        V2::new(self.au.delta(a.x, b.x), self.av.delta(a.y, b.y))
    }

    fn trace(&self) -> Found {
        let (au, av) = (self.au, self.av);
        let (nu, nv) = (au.nodes, av.nodes);
        let node = |i: usize, j: usize| j * nu + i;
        let uv = |i: usize, j: usize| P2::new(au.at(i), av.at(j));
        let vals: Vec<f64> = (0..nu * nv).into_par_iter().map(|k| self.g(uv(k % nu, k / nu))).collect();
        if vals.iter().all(|x| x.abs() < tol::ROUNDOFF) {
            return Found::Whole;
        }
        let pos = |x: f64| x >= 0.0;
        // Keys: 2·node for the grid edge to (i + 1, j), 2·node + 1 for the edge to (i, j + 1);
        // a saddle point where two branches cross, 2·nu·nv + cell.
        let mut segs: Vec<(usize, usize)> = Vec::new();
        let mut saddles: HashMap<usize, P2> = HashMap::new();
        for j in 0..av.cells() {
            for i in 0..au.cells() {
                let (i1, j1) = (au.next(i), av.next(j));
                let c = [vals[node(i, j)], vals[node(i1, j)], vals[node(i1, j1)], vals[node(i, j1)]];
                let sg = c.map(pos);
                let keys = [2 * node(i, j), 2 * node(i1, j) + 1, 2 * node(i, j1), 2 * node(i, j) + 1];
                let cross = [sg[0] != sg[1], sg[1] != sg[2], sg[3] != sg[2], sg[0] != sg[3]];
                let list: Vec<usize> = (0..4).filter(|&k| cross[k]).collect();
                match list.len() {
                    2 => segs.push((keys[list[0]], keys[list[1]])),
                    4 => {
                        let lo = uv(i, j);
                        let centre = P2::new(lo.x + 0.5 * au.step, lo.y + 0.5 * av.step);
                        let sad = self.saddle(centre);
                        match sad {
                            Some((p, gs)) if gs.abs() < 1e-8 => {
                                let key = 2 * nu * nv + j * nu + i;
                                saddles.insert(key, p);
                                for k in keys {
                                    segs.push((k, key));
                                }
                            }
                            _ => {
                                let gc = sad.map_or_else(|| self.g(centre), |s| s.1);
                                if pos(gc) == sg[0] {
                                    segs.push((keys[0], keys[1]));
                                    segs.push((keys[2], keys[3]));
                                } else {
                                    segs.push((keys[3], keys[0]));
                                    segs.push((keys[1], keys[2]));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if segs.is_empty() {
            return Found::Curves(vec![]);
        }
        // The exact zero on a grid edge (unwrapped ends, folded result).
        let point = |key: usize| -> P2 {
            if let Some(p) = saddles.get(&key) {
                return *p;
            }
            let nd = key / 2;
            let a = uv(nd % nu, nd / nu);
            let b = if key.is_multiple_of(2) { P2::new(a.x + au.step, a.y) } else { P2::new(a.x, a.y + av.step) };
            let (mut lo, mut hi) = (a, b);
            let slo = pos(self.g(lo));
            for _ in 0..50 {
                let m = nalgebra::center(&lo, &hi);
                if pos(self.g(m)) == slo {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let m = nalgebra::center(&lo, &hi);
            P2::new(au.fold(m.x), av.fold(m.y))
        };
        let mut out = Vec::new();
        for chain in chains(&segs) {
            let closed = chain.len() > 2 && chain[0] == chain[chain.len() - 1];
            let pts: Vec<P2> = chain.iter().map(|&k| point(k)).collect();
            let fine = self.refine(&pts);
            out.extend(self.clip_chain(&fine, closed));
        }
        Found::Curves(out)
    }

    /// The saddle point of g near `c` (Newton on ∇g = 0, finite differences), within the cell
    /// round `c` grown by half a cell, and g there.
    fn saddle(&self, c: P2) -> Option<(P2, f64)> {
        let (hu, hv) = (1e-3 * self.au.step, 1e-3 * self.av.step);
        let mut q = c;
        for _ in 0..12 {
            let g = |du: f64, dv: f64| self.g(P2::new(q.x + du, q.y + dv));
            let g0 = g(0.0, 0.0);
            let (gpu, gmu, gpv, gmv) = (g(hu, 0.0), g(-hu, 0.0), g(0.0, hv), g(0.0, -hv));
            let gx = (gpu - gmu) / (2.0 * hu);
            let gy = (gpv - gmv) / (2.0 * hv);
            let gxx = (gpu - 2.0 * g0 + gmu) / (hu * hu);
            let gyy = (gpv - 2.0 * g0 + gmv) / (hv * hv);
            let gxy = (g(hu, hv) - g(hu, -hv) - g(-hu, hv) + g(-hu, -hv)) / (4.0 * hu * hv);
            let (du, dv) = solve2(gxx, gxy, gxy, gyy, -gx, -gy)?;
            q = P2::new(q.x + du, q.y + dv);
            if (q.x - c.x).abs() > self.au.step || (q.y - c.y).abs() > self.av.step {
                return None;
            }
            if du.abs() < 1e-12 * self.au.step && dv.abs() < 1e-12 * self.av.step {
                break;
            }
        }
        Some((P2::new(self.au.fold(q.x), self.av.fold(q.y)), self.g(q)))
    }

    /// `m` pulled onto g = 0 by Newton along ∇g, moving at most `reach` (uv).
    fn to_zero(&self, m: P2, reach: f64) -> Option<P2> {
        let (hu, hv) = (1e-6 * self.au.step, 1e-6 * self.av.step);
        let mut q = m;
        for _ in 0..8 {
            let g0 = self.g(q);
            if g0.abs() < 1e-13 {
                break;
            }
            let gx = (self.g(P2::new(q.x + hu, q.y)) - self.g(P2::new(q.x - hu, q.y))) / (2.0 * hu);
            let gy = (self.g(P2::new(q.x, q.y + hv)) - self.g(P2::new(q.x, q.y - hv))) / (2.0 * hv);
            let gg = gx * gx + gy * gy;
            if gg < tol::TINY {
                return None;
            }
            q = P2::new(q.x - g0 * gx / gg, q.y - g0 * gy / gg);
            if self.diff(m, q).norm() > reach {
                return None;
            }
        }
        (self.g(q).abs() < tol::ROUNDOFF).then(|| P2::new(self.au.fold(q.x), self.av.fold(q.y)))
    }

    /// The polyline with points added on the silhouette until its chords are within the
    /// tolerance.
    fn refine(&self, pts: &[P2]) -> Vec<P2> {
        let mut out = Vec::with_capacity(pts.len());
        if let Some(&p) = pts.first() {
            out.push(p);
        }
        for w in pts.windows(2) {
            self.refine_seg(w[0], w[1], self.eval(w[0]), self.eval(w[1]), 0, &mut out);
            out.push(w[1]);
        }
        out
    }

    fn refine_seg(&self, a: P2, b: P2, pa: P3, pb: P3, depth: usize, out: &mut Vec<P2>) {
        if depth >= 14 {
            return;
        }
        let reach = self.diff(a, b).norm();
        let Some(m) = self.to_zero(self.mid(a, b), reach) else { return };
        let pm = self.eval(m);
        if (pm - nalgebra::center(&pa, &pb)).norm() <= self.tol {
            return;
        }
        self.refine_seg(a, m, pa, pm, depth + 1, out);
        out.push(m);
        self.refine_seg(m, b, pm, pb, depth + 1, out);
    }

    /// The runs of a polyline in the face, as 3D polylines (a closed polyline lying wholly in
    /// the face stays closed; runs through its closing point are joined).
    fn clip_chain(&self, pts: &[P2], closed: bool) -> Vec<(SilGeom, bool)> {
        let dom = self.dom;
        let inside: Vec<bool> = pts.iter().map(|q| dom.contains(q.x, q.y, false)).collect();
        let cut = |a: P2, b: P2| -> P2 {
            let (mut lo, mut hi) = (a, b);
            let s = dom.contains(a.x, a.y, false);
            for _ in 0..40 {
                let m = self.mid(lo, hi);
                if dom.contains(m.x, m.y, false) == s {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            self.mid(lo, hi)
        };
        let mut runs: Vec<Vec<P2>> = Vec::new();
        let mut run: Vec<P2> = Vec::new();
        for k in 0..pts.len() {
            if inside[k] {
                if run.is_empty() && k > 0 {
                    run.push(cut(pts[k], pts[k - 1]));
                }
                run.push(pts[k]);
            } else if !run.is_empty() {
                run.push(cut(pts[k - 1], pts[k]));
                runs.push(std::mem::take(&mut run));
            }
        }
        let whole = runs.is_empty() && run.len() == pts.len();
        if !run.is_empty() {
            runs.push(run);
        }
        if closed && !whole && runs.len() >= 2 && inside[0] && inside[pts.len() - 1] {
            // The last run goes on into the first through the closing point.
            let first = runs.remove(0);
            if let Some(last) = runs.last_mut() {
                last.extend(first.into_iter().skip(1));
            }
        }
        runs.into_iter()
            .filter(|r| r.len() >= 2)
            .map(|r| (SilGeom::Poly { pts: r.iter().map(|q| self.eval(*q)).collect(), uv: r }, closed && whole))
            .collect()
    }
}

/// Whether the face's box spans the whole of a closed direction of its surface: a periodic one's
/// period, or a B-spline's range whose ends meet (within 1e-7 of the patch's size).
fn closed_dirs(s: &Surface, ub: (f64, f64), vb: (f64, f64), size: f64) -> (bool, bool) {
    let full = |(a, b): (f64, f64), (r0, r1): (f64, f64)| (a - r0).abs() <= tol::REL * (1.0 + r0.abs()) && (b - r1).abs() <= tol::REL * (1.0 + r1.abs());
    let ((u0, u1), (v0, v1)) = s.param_box();
    let meets = |along_u: bool| {
        (0..5).all(|k| {
            let f = 0.1 + 0.2 * k as f64;
            let (p, q) = if along_u {
                let v = vb.0 + (vb.1 - vb.0) * f;
                (s.eval(ub.0, v), s.eval(ub.1, v))
            } else {
                let u = ub.0 + (ub.1 - ub.0) * f;
                (s.eval(u, vb.0), s.eval(u, vb.1))
            };
            (p - q).norm() <= 1e-7 * size
        })
    };
    let wu = if s.u_periodic() { ((ub.1 - ub.0) - TAU).abs() < tol::ANGULAR } else { full(ub, (u0, u1)) && meets(true) };
    let wv = if s.v_periodic() { ((vb.1 - vb.0) - TAU).abs() < tol::ANGULAR } else { full(vb, (v0, v1)) && meets(false) };
    (wu, wv)
}

/// Chains of segments (pairs of node keys), open chains from their ends first; a closed chain
/// ends on its first key.
fn chains(segs: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut at: HashMap<usize, Vec<usize>> = HashMap::new();
    for (k, &(a, b)) in segs.iter().enumerate() {
        at.entry(a).or_default().push(k);
        at.entry(b).or_default().push(k);
    }
    let mut keys: Vec<usize> = at.keys().copied().collect();
    keys.sort_by_key(|k| (at[k].len().is_multiple_of(2), *k));
    let mut used = vec![false; segs.len()];
    let mut out: Vec<Vec<usize>> = Vec::new();
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
            out.push(chain);
        }
    }
    out
}

/// [`crate::Kernel::silhouettes`] for the native kernel.
pub fn kernel_silhouettes(k: &super::NativeKernel, body: crate::BodyId, view: &crate::SilhouetteView, tolerance: f64) -> crate::Result<Vec<crate::SilhouetteCurve>> {
    use crate::{KernelError, SilhouetteView};
    if !(tolerance > 0.0 && tolerance.is_finite()) {
        return Err(KernelError::InvalidParameter(format!("a silhouette tolerance of {tolerance} mm")));
    }
    let v = match view {
        SilhouetteView::Parallel(d) if d.iter().all(|x| x.is_finite()) && d.norm() > 1e-12 => View::Parallel(*d),
        SilhouetteView::Perspective(e) if e.iter().all(|x| x.is_finite()) => View::Perspective(*e),
        _ => return Err(KernelError::InvalidParameter("a view direction must be a finite non-zero vector, an eye point finite".into())),
    };
    let e = k.entry(body)?;
    let opts = SilOptions { tol: tolerance, ..SilOptions::default() };
    Ok(body_silhouettes(&e.body, e.polys(), &v, &opts)
        .into_iter()
        .map(|s| {
            let (points, uv) = s.samples(&e.body.faces[s.face].surf, tolerance);
            crate::SilhouetteCurve { face: crate::FaceId(s.face as u64), points, uv, closed: s.closed }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::math::Frame;

    fn whole_face(ub: (f64, f64), vb: (f64, f64)) -> Vec<Vec<P2>> {
        vec![vec![P2::new(ub.0, vb.0), P2::new(ub.1, vb.0), P2::new(ub.1, vb.1), P2::new(ub.0, vb.1)]]
    }

    fn total(found: Found) -> (usize, f64) {
        match found {
            Found::Curves(cs) => (cs.len(), cs.iter().map(|(g, _)| match g {
                SilGeom::Exact { curve, t } => curve.length(t.0, t.1),
                SilGeom::Poly { pts, .. } => pts.windows(2).map(|w| (w[1] - w[0]).norm()).sum(),
            }).sum()),
            Found::Whole => (usize::MAX, 0.0),
        }
    }

    /// A torus R 10 r 5 seen side-on: its two meridians square to the view and its top and
    /// bottom circles, 4π(R + r) long, crossing at four saddle points; one meridian lies on
    /// the seam. Seen from above: the equators, 4πR.
    #[test]
    fn torus_side_on_and_from_above() {
        let s = Surface::Torus { f: Frame::world(), rmaj: 10.0, rmin: 5.0 };
        let polys = whole_face((0.0, TAU), (-PI, PI));
        let dom = FaceDomain::new(&s, &polys);
        let opts = SilOptions::default();
        for d in [V3::y(), V3::x(), V3::new(1.0, 1.0, 0.0)] {
            let (_, len) = total(domain_silhouettes(&dom, &View::Parallel(d), &opts));
            assert!((len - 4.0 * PI * 15.0).abs() < 1e-3 * len, "{d:?}: {len}");
        }
        let (n, len) = total(domain_silhouettes(&dom, &View::Parallel(V3::z()), &opts));
        assert_eq!(n, 2);
        assert!((len - 2.0 * TAU * 10.0).abs() < 1e-9, "{len}");
        // A NURBS-free general direction: every point on the silhouette.
        let d = V3::new(0.3, 0.5, 1.0).normalize();
        let Found::Curves(cs) = domain_silhouettes(&dom, &View::Parallel(d), &opts) else { panic!() };
        assert!(!cs.is_empty());
        for (g, _) in &cs {
            let SilGeom::Poly { pts, .. } = g else { panic!() };
            for q in pts {
                let (u, v) = s.project(q, None);
                assert!(s.normal(u, v).dot(&d).abs() < 1e-6 && (s.eval(u, v) - q).norm() < 1e-9);
            }
        }
    }

    /// A sphere from an eye point 13 from its centre (r 5): the circle of radius 5·12/13.
    #[test]
    fn sphere_in_perspective() {
        let s = Surface::Sphere { f: Frame::world(), r: 5.0 };
        let polys = whole_face((0.0, TAU), (-PI / 2.0, PI / 2.0));
        let dom = FaceDomain::new(&s, &polys);
        let (_, len) = total(domain_silhouettes(&dom, &View::Perspective(P3::new(0.0, 13.0, 0.0)), &SilOptions::default()));
        assert!((len - TAU * 60.0 / 13.0).abs() < 1e-6, "{len}");
        // The same traced (the sphere as a revolution) agrees.
        let m = Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::z(), r: 5.0 };
        let rev = Surface::Revolution { f: Frame::world(), m };
        let polys = whole_face((0.0, TAU), (-PI / 2.0, PI / 2.0));
        let dom = FaceDomain::new(&rev, &polys);
        let (_, len2) = total(domain_silhouettes(&dom, &View::Perspective(P3::new(0.0, 13.0, 0.0)), &SilOptions::default()));
        assert!((len2 - len).abs() < 1e-3, "{len2} vs {len}");
    }

    /// A cylinder from an eye point: the two generators whose tangent planes pass through it.
    #[test]
    fn cylinder_in_perspective() {
        let s = Surface::Cylinder { f: Frame::world(), r: 3.0 };
        let polys = whole_face((0.0, TAU), (0.0, 10.0));
        let dom = FaceDomain::new(&s, &polys);
        let eye = P3::new(6.0, 0.0, 5.0);
        let Found::Curves(cs) = domain_silhouettes(&dom, &View::Perspective(eye), &SilOptions::default()) else { panic!() };
        assert_eq!(cs.len(), 2);
        for (g, _) in &cs {
            let SilGeom::Exact { curve, t } = g else { panic!() };
            let p = curve.point(0.5 * (t.0 + t.1));
            // Tangent from (6, 0) to the circle r 3: the touching point is at x = 9/6.
            assert!((p.x - 1.5).abs() < 1e-9, "{p:?}");
            assert!((t.1 - t.0 - 10.0).abs() < 1e-9);
        }
        // Seen along its axis: wholly on its silhouette.
        assert!(matches!(domain_silhouettes(&dom, &View::Parallel(V3::z()), &SilOptions::default()), Found::Whole));
    }
}
