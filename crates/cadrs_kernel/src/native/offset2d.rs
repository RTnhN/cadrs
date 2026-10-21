//! Offsets of planar wires (DRAW's `mkoffset`, OCCT's `BRepOffsetAPI_MakeOffset` with arc
//! joins), in the wire's plane (z = 0 here).
//!
//! 0. The edges are cut where their curvature has an extremum or changes sign, as OCCT cuts the
//!    spine (so the result has OCCT's edges); a nearly planar wire is first flattened.
//! 1. Every edge is offset to one side: lines and arcs exactly (an arc whose radius would fall
//!    to nothing is dropped), other curves as cubics through offset samples.
//! 2. Consecutive offsets are joined by arcs about the vertex between them, turning as the
//!    wire turns (at convex corners however slight). An open wire is run out and back, so its ends are half turns (round caps).
//! 3. The raw loop is split where it crosses itself, and only the pieces that stay the full
//!    distance from the wire are kept (the rest lie in overlaps: concave corners, arcs inside
//!    the distance, loops of the offset curves).
//! 4. Neighbouring pieces of one curve are joined again and the pieces chained into loops.

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::tol;
use crate::{KernelError, Result};
use std::f64::consts::PI;

/// A curve between two parameters, run from `t0` to `t1` (`t0 < t1`).
#[derive(Debug, Clone)]
pub struct Piece2 {
    pub c: Curve,
    pub t0: f64,
    pub t1: f64,
}

impl Piece2 {
    fn start(&self) -> P3 {
        self.c.point(self.t0)
    }
    fn end(&self) -> P3 {
        self.c.point(self.t1)
    }
    fn dir(&self, t: f64) -> V2 {
        let d = self.c.tangent(t);
        V2::new(d.x, d.y).try_normalize(1e-300).unwrap_or(V2::new(1.0, 0.0))
    }
}

/// One element of the raw offset loop and where it came from.
#[derive(Debug, Clone)]
struct Raw {
    p: Piece2,
    /// How far it may lie from the exact offset (an approximated curve's).
    err: f64,
}

fn left(v: V2) -> V2 {
    V2::new(-v.y, v.x)
}

fn p2(p: &P3) -> V2 {
    V2::new(p.x, p.y)
}

/// The edge run forwards (`t0 < t1`).
fn forward(c: &Curve, t0: f64, t1: f64) -> Piece2 {
    if t1 >= t0 {
        Piece2 { c: c.clone(), t0, t1 }
    } else {
        // `reversed` maps an increasing range: the edge's [t1, t0] run the other way.
        let (c, a, b) = c.reversed(t1, t0);
        Piece2 { c, t0: a, t1: b }
    }
}

/// The edge offset `l` to its left and how far that lies from the exact offset at most (lines and
/// arcs exactly; other curves as cubics through offset points, within about 1e-7 of the wire's
/// size), or None when it shrinks to nothing.
fn offset_edge(e: &Piece2, l: f64, size: f64) -> Option<(Piece2, f64)> {
    match &e.c {
        Curve::Line { .. } => {
            let n = left(e.dir(e.t0));
            let sh = V3::new(n.x, n.y, 0.0) * l;
            let (c, len) = Curve::line(e.start() + sh, e.end() + sh);
            Some((Piece2 { c, t0: 0.0, t1: len }, 0.0))
        }
        Curve::Circle { c, x, y, r } => {
            // Run counter-clockwise the left is the centre's side.
            let ccw = x.cross(y).z > 0.0;
            let r2 = if ccw { r - l } else { r + l };
            if r2 <= 1e-9 * size {
                return None;
            }
            Some((Piece2 { c: Curve::Circle { c: *c, x: *x, y: *y, r: r2 }, t0: e.t0, t1: e.t1 }, 0.0))
        }
        _ => {
            let mut ts = e.c.sample_params(e.t0, e.t1, size * 1e-7, 0.05);
            if ts.len() < 65 {
                ts = (0..=64).map(|i| e.t0 + (e.t1 - e.t0) * i as f64 / 64.0).collect();
            }
            let at = |t: f64| {
                let n = left(e.dir(t));
                e.c.point(t) + V3::new(n.x, n.y, 0.0) * l
            };
            let pts: Vec<P3> = ts.iter().map(|&t| at(t)).collect();
            let params = NurbsCurve::chord_params(&pts);
            let c = NurbsCurve::interpolate(&pts, &params, 3, None, None);
            // The error: across the cubic, halfway between the points it passes through.
            let err = ts
                .windows(2)
                .zip(params.windows(2))
                .map(|(t, s)| {
                    let d = c.ders(0.5 * (s[0] + s[1]), 1);
                    let (q, d1) = (P3::from(d[0]), d[1]);
                    let n = left(V2::new(d1.x, d1.y).try_normalize(1e-300).unwrap_or(V2::new(1.0, 0.0)));
                    let w = at(0.5 * (t[0] + t[1])) - q;
                    (w.x * n.x + w.y * n.y).abs()
                })
                .fold(0.0, f64::max);
            let (a, b) = c.range();
            Some((Piece2 { c: Curve::Nurbs(std::sync::Arc::new(c)), t0: a, t1: b }, err))
        }
    }
}

/// The arc about `v` from `v + l·n1` turning by `phi`.
fn join_arc(v: P3, n1: V2, phi: f64, l: f64) -> Piece2 {
    let u0 = n1 * l.signum();
    let y = left(u0) * phi.signum();
    Piece2 { c: Curve::Circle { c: v, x: V3::new(u0.x, u0.y, 0.0), y: V3::new(y.x, y.y, 0.0), r: l.abs() }, t0: 0.0, t1: phi.abs() }
}

/// The plane through `pts` fitting them best (least squares): its centroid and normal, when
/// every point lies within `rel` of the points' extent from it.
pub fn best_plane(pts: &[P3], rel: f64) -> Option<(P3, V3)> {
    if pts.len() < 3 {
        return None;
    }
    let o = P3::from(pts.iter().fold(V3::zeros(), |s, p| s + p.coords) / pts.len() as f64);
    let cov = pts.iter().fold(nalgebra::Matrix3::zeros(), |m, p| {
        let d = p - o;
        m + d * d.transpose()
    });
    let eig = nalgebra::SymmetricEigen::new(cov);
    let k = eig.eigenvalues.imin();
    let n: V3 = eig.eigenvectors.column(k).into();
    let n = n.try_normalize(1e-300)?;
    let size = pts.iter().map(|p| (p - o).norm()).fold(0.0, f64::max);
    pts.iter().all(|p| (p - o).dot(&n).abs() <= rel * size).then_some((o, n))
}

/// The curve projected onto the plane z = 0 (a wire only nearly planar, fitted to that plane),
/// with its range: unchanged when it lies in the plane already.
fn flattened(c: &Curve, t0: f64, t1: f64) -> (Curve, f64, f64) {
    let flat = |n: &NurbsCurve| {
        let mut n = n.clone();
        for h in &mut n.cps {
            h.z = 0.0;
        }
        n
    };
    match c {
        Curve::Line { o, d } if d.z != 0.0 || o.z != 0.0 => {
            let (a, b) = (c.point(t0), c.point(t1));
            let (l, len) = Curve::line(P3::new(a.x, a.y, 0.0), P3::new(b.x, b.y, 0.0));
            if t1 >= t0 { (l, 0.0, len) } else { (l, len, 0.0) }
        }
        Curve::Circle { c: o, x, y, r } if x.z.abs() < 1e-12 && y.z.abs() < 1e-12 && o.z != 0.0 => {
            (Curve::Circle { c: P3::new(o.x, o.y, 0.0), x: *x, y: *y, r: *r }, t0, t1)
        }
        Curve::Ellipse { c: o, x, y, a, b } if x.z.abs() < 1e-12 && y.z.abs() < 1e-12 && o.z != 0.0 => {
            (Curve::Ellipse { c: P3::new(o.x, o.y, 0.0), x: *x, y: *y, a: *a, b: *b }, t0, t1)
        }
        Curve::Circle { x, y, .. } | Curve::Ellipse { x, y, .. } if x.z.abs() >= 1e-12 || y.z.abs() >= 1e-12 => {
            let (lo, hi) = (t0.min(t1), t0.max(t1));
            let n = flat(&c.to_nurbs(lo, hi));
            if t1 >= t0 { (Curve::Nurbs(std::sync::Arc::new(n)), lo, hi) } else { (Curve::Nurbs(std::sync::Arc::new(n)), hi, lo) }
        }
        Curve::Nurbs(n) if n.cps.iter().any(|h| h.z != 0.0) => (Curve::Nurbs(std::sync::Arc::new(flat(n))), t0, t1),
        _ => (c.clone(), t0, t1),
    }
}

/// The parameters of `c` on [t0, t1] where its signed curvature has an extremum or changes sign
/// (OCCT's `Geom2dLProp_CurAndInf2d`), in order: none on lines and circles, the vertices of an
/// ellipse, and on a B-spline the sign changes of κ' and κ sampled 100 times over each of its
/// C3 pieces.
fn curvature_points(c: &Curve, t0: f64, t1: f64) -> Vec<f64> {
    let mut out = Vec::new();
    match c {
        Curve::Line { .. } | Curve::Circle { .. } => {}
        Curve::Ellipse { .. } => {
            let k0 = (t0 / (PI / 2.0)).floor() as i64;
            for k in k0..=k0 + 5 {
                let t = k as f64 * PI / 2.0;
                if t >= t0 && t <= t1 {
                    out.push(t);
                }
            }
        }
        Curve::Nurbs(n) => {
            // The C3 pieces: interior knots of multiplicity above degree − 3.
            let p = n.degree;
            let mut breaks = vec![t0];
            let mut i = 0;
            while i < n.knots.len() {
                let k = n.knots[i];
                let m = n.knots[i..].iter().take_while(|&&x| x == k).count();
                if k > t0 && k < t1 && p < m + 3 {
                    breaks.push(k);
                }
                i += m;
            }
            breaks.push(t1);
            // κ' · |D1|⁵ and κ · |D1|³ have the signs of κ' and κ.
            let dk = |u: f64| {
                let d = n.ders(u, 3);
                let (d1, d2, d3) = (V2::new(d[1].x, d[1].y), V2::new(d[2].x, d[2].y), V2::new(d[3].x, d[3].y));
                let cr = |a: V2, b: V2| a.x * b.y - a.y * b.x;
                let s = d1.norm_squared();
                let f = cr(d1, d3) * s - 3.0 * cr(d1, d2) * d1.dot(&d2);
                (f / (s * s * s.sqrt()).max(1e-300), cr(d1, d2) / (d1.norm() * d2.norm()).max(1e-300))
            };
            for w in breaks.windows(2) {
                let (a, b) = (w[0], w[1]);
                if b - a < 1e-12 {
                    continue;
                }
                let ns = 100;
                // (The last sample just inside: at the knot itself the next piece would count.)
                let us: Vec<f64> = (0..=ns).map(|i| if i == ns { b - (b - a) * 1e-9 } else { a + (b - a) * i as f64 / ns as f64 }).collect();
                let fs: Vec<(f64, f64)> = us.iter().map(|&u| dk(u)).collect();
                for which in 0..2 {
                    let val = |u: f64| if which == 0 { dk(u).0 } else { dk(u).1 };
                    let get = |f: &(f64, f64)| if which == 0 { f.0 } else { f.1 };
                    let scale = fs.iter().map(|f| get(f).abs()).fold(0.0, f64::max);
                    if scale == 0.0 {
                        continue;
                    }
                    // Values this near zero count as zero (OCCT's tolerances: 1e-4 of the
                    // piece's span for κ', 1e-6 for the sine of κ).
                    let null = if which == 0 { 1e-4 * (b - a) } else { tol::POINT };
                    // Sign changes between samples (null samples skipped), refined by bisection.
                    let mut last: Option<(f64, f64)> = None;
                    for (u, f) in us.iter().zip(&fs) {
                        let f = get(f);
                        if f.abs() <= null {
                            continue;
                        }
                        if let Some((u0, f0)) = last
                            && f0 * f < 0.0
                        {
                            let (mut lo, mut hi) = (u0, *u);
                            for _ in 0..60 {
                                let m = 0.5 * (lo + hi);
                                if val(m) * f0 > 0.0 {
                                    lo = m;
                                } else {
                                    hi = m;
                                }
                            }
                            out.push(0.5 * (lo + hi));
                        }
                        last = Some((*u, f));
                    }
                }
            }
            out.sort_by(f64::total_cmp);
        }
    }
    out
}

/// `e` cut at `ts` as OCCT's `MAT2d_CutCurve` does: in order, skipping cuts within 1e-8 in
/// parameter or 1e-6 in space of the last cut or of the end. Empty when nothing is cut.
fn cut_at(e: &Piece2, ts: &[f64]) -> Vec<Piece2> {
    let (ptol, tol) = (1e-8, 1e-6);
    let (mut uf, ul) = (e.t0, e.t1);
    let (mut pf, pl) = (e.start(), e.end());
    let mut out = Vec::new();
    for &uc in ts {
        let pc = e.c.point(uc);
        if uc - uf > ptol && (pc - pf).norm() > tol {
            if ul - uc < ptol || (pl - pc).norm() < tol {
                continue;
            }
            out.push(Piece2 { c: e.c.clone(), t0: uf, t1: uc });
            uf = uc;
            pf = pc;
        }
    }
    if !out.is_empty() {
        out.push(Piece2 { c: e.c.clone(), t0: uf, t1: ul });
    }
    out
}

/// `e` cut into `n` pieces of equal parameter span (OCCT's `CutCurve`).
fn cut_even(e: &Piece2, n: usize) -> Vec<Piece2> {
    let ts: Vec<f64> = (1..n).map(|i| e.t0 + (e.t1 - e.t0) * i as f64 / n as f64).collect();
    cut_at(e, &ts)
}

/// The wire's edges cut where their curvature has extrema or changes sign, as OCCT prepares the
/// spine of a planar offset (`BRepFill_OffsetWire::PrepareSpine`): every piece offsets to an edge
/// of its own. A wire of one edge has it cut in three at least (a closed edge otherwise has no
/// vertex to start from), and of two edges the second in two when the first stays whole.
fn prepare_spine(wire: Vec<Piece2>) -> Vec<Piece2> {
    let n = wire.len();
    let mut out = Vec::new();
    let mut first_cut = 0;
    for (i, e) in wire.into_iter().enumerate() {
        let cuts = cut_at(&e, &curvature_points(&e.c, e.t0, e.t1));
        let force = if n == 1 {
            2
        } else if n == 2 && i == 1 && first_cut == 0 {
            1
        } else {
            0
        };
        let pieces = match force {
            1 if cuts.is_empty() => cut_even(&e, 2),
            2 if cuts.is_empty() => cut_even(&e, 3),
            2 if cuts.len() == 2 => {
                let mid = 0.5 * (e.t0 + e.t1);
                let mut v = cuts.clone();
                let k = if v[0].t1 > mid { 0 } else { 1 };
                let half = cut_even(&v[k], 2);
                if !half.is_empty() {
                    v.splice(k..=k, half);
                }
                v
            }
            _ => cuts,
        };
        if i == 0 {
            first_cut = pieces.len();
        }
        if pieces.is_empty() {
            out.push(e);
        } else {
            out.extend(pieces);
        }
    }
    out
}

/// The loops of the wire `edges` (in the plane z = 0, chained in order) offset by `d`: a closed
/// wire outwards for d > 0, an open one on both sides with round ends.
pub fn offset_wire(edges: &[(Curve, f64, f64)], d: f64) -> Result<Vec<Vec<Piece2>>> {
    if edges.is_empty() || d == 0.0 {
        return Err(KernelError::InvalidParameter("an offset needs edges and a distance".into()));
    }
    let clock = std::time::Instant::now();
    let wire = chained(edges.iter().map(|(c, a, b)| flattened(c, *a, *b)).map(|(c, a, b)| forward(&c, a, b)).collect());
    let mut bb = Bbox::empty();
    // Each edge's length in the plane (its chords): an edge square to the plane (a wire not in
    // z = 0) flattens to a point, which has no direction to offset along.
    let mut lens = Vec::with_capacity(wire.len());
    for e in &wire {
        let pts: Vec<P3> = (0..=8).map(|k| e.c.point(e.t0 + (e.t1 - e.t0) * k as f64 / 8.0)).collect();
        if !(e.t0.is_finite() && e.t1.is_finite()) || pts.iter().any(|p| !(p.x.is_finite() && p.y.is_finite())) {
            return Err(KernelError::InvalidParameter("an offset of a wire with an edge of no length in its plane".into()));
        }
        lens.push(pts.windows(2).map(|w| (w[1] - w[0]).norm()).sum::<f64>());
        pts.iter().for_each(|p| bb.add(p));
    }
    if lens.iter().any(|l| *l <= bb.diagonal() * 1e-12) {
        return Err(KernelError::InvalidParameter("an offset of a wire with an edge of no length in its plane".into()));
    }
    let size = bb.diagonal().max(d.abs());
    let tol = size * 1e-7;
    // (Edges however short are kept: OCCT offsets them to edges of their own.)
    // The wire's own gaps between edges (vertex tolerances) set the closure and chaining.
    let gap = wire.windows(2).map(|w| (w[0].end() - w[1].start()).norm()).fold(0.0, f64::max);
    let closing = (wire[0].start() - wire.last().unwrap().end()).norm();
    let closed = (wire[0].start() - wire.last().unwrap().end()).norm() < (tol * 10.0).max(size * 1e-4).max(gap * 3.0);
    let wire = prepare_spine(wire);
    if std::env::var_os("CADRS_OFFSET_DEBUG").is_some() {
        eprintln!("spine: {} pieces", wire.len());
    }
    // The loop to offset to its left by l: a closed wire (counter-clockwise: outwards is the
    // right), or an open one out and back.
    let (seq, l): (Vec<Piece2>, f64) = if closed {
        // Outwards is the right of a counter-clockwise loop.
        let mut area = 0.0;
        for e in &wire {
            for k in 0..32 {
                let (a, b) = (e.c.point(e.t0 + (e.t1 - e.t0) * k as f64 / 32.0), e.c.point(e.t0 + (e.t1 - e.t0) * (k + 1) as f64 / 32.0));
                area += a.x * b.y - b.x * a.y;
            }
        }
        (wire.clone(), if area >= 0.0 { -d } else { d })
    } else {
        let mut s = wire.clone();
        s.extend(wire.iter().rev().map(|e| {
            let (c, a, b) = e.c.reversed(e.t0, e.t1);
            Piece2 { c, t0: a, t1: b }
        }));
        (s, d.abs())
    };
    let chain_tol = (size * 1e-6).max(l.abs() * 1e-6).max(gap * 3.0).max(if closed { closing * 3.0 } else { 0.0 });
    let n = seq.len();
    let tick = |what: &str| {
        if std::env::var_os("CADRS_OFFSET_DEBUG").is_some() {
            eprintln!("  {what}: {:?}", clock.elapsed());
        }
    };
    let mut raw: Vec<Raw> = Vec::new();
    for i in 0..n {
        let e = &seq[i];
        if let Some((o, err)) = offset_edge(e, l, size) {
            raw.push(Raw { p: o, err });
        }
        // The join to the next edge.
        let next = &seq[(i + 1) % n];
        let (t1, t2) = (e.dir(e.t1), next.dir(next.t0));
        let mut phi = (t1.x * t2.y - t1.y * t2.x).atan2(t1.dot(&t2));
        // Tangent: the offsets meet without an arc. A convex corner gets one however small its
        // offset (as OCCT's, from an angle of 1e-7 up); a concave one within the wire's gaps
        // closes up in the chaining.
        let convex = phi * l < 0.0;
        if phi.abs() < 1e-7 || (!convex && (l * phi).abs() <= chain_tol) {
            continue;
        }
        if phi.abs() > PI - 1e-9 {
            phi = -PI * l.signum();
        }
        raw.push(Raw { p: join_arc(e.end(), left(t1), phi, l), err: 0.0 });
    }
    tick("raw");
    // Where the raw loop crosses itself.
    let polys: Vec<Vec<f64>> = raw
        .iter()
        .map(|r| {
            let k = match &r.p.c {
                Curve::Line { .. } => 1,
                Curve::Circle { .. } | Curve::Ellipse { .. } => ((r.p.t1 - r.p.t0).abs() / 0.05).ceil() as usize,
                Curve::Nurbs(n) => (8 * n.cps.len()).max(64),
            }
            .clamp(1, 4096);
            let mut ts: Vec<f64> = (0..=k).map(|i| r.p.t0 + (r.p.t1 - r.p.t0) * i as f64 / k as f64).collect();
            if ts.len() < 2 {
                ts = vec![r.p.t0, r.p.t1];
            }
            ts
        })
        .collect();
    struct Seg {
        el: usize,
        k: usize,
        a: V2,
        b: V2,
        lo: f64,
        hi: f64,
    }
    let mut segs: Vec<Seg> = Vec::new();
    for (el, ts) in polys.iter().enumerate() {
        for k in 0..ts.len() - 1 {
            let (a, b) = (p2(&raw[el].p.c.point(ts[k])), p2(&raw[el].p.c.point(ts[k + 1])));
            segs.push(Seg { el, k, a, b, lo: a.x.min(b.x), hi: a.x.max(b.x) });
        }
    }
    tick(&format!("{} segments", segs.len()));
    segs.sort_by(|a, b| a.lo.total_cmp(&b.lo));
    let mut cuts: Vec<Vec<f64>> = vec![Vec::new(); raw.len()];
    let near_end = |el: usize, t: f64| {
        let p = &raw[el].p;
        let span = (p.t1 - p.t0).abs().max(1e-300);
        (t - p.t0).abs() < 1e-9 * span || (t - p.t1).abs() < 1e-9 * span
    };
    let slack = tol * 10.0;
    for i in 0..segs.len() {
        let s = &segs[i];
        for o in &segs[i + 1..] {
            if o.lo > s.hi + slack {
                break;
            }
            if s.a.y.max(s.b.y) + slack < o.a.y.min(o.b.y) || o.a.y.max(o.b.y) + slack < s.a.y.min(s.b.y) {
                continue;
            }
            // Neighbouring segments of one curve meet at their shared point.
            if s.el == o.el && (s.k as i64 - o.k as i64).abs() <= 1 {
                continue;
            }
            let Some((u, v)) = seg_cross(&s.a, &s.b, &o.a, &o.b, slack) else { continue };
            let (ts, to) = (&polys[s.el], &polys[o.el]);
            let ta = ts[s.k] + (ts[s.k + 1] - ts[s.k]) * u;
            let tb = to[o.k] + (to[o.k + 1] - to[o.k]) * v;
            let Some((ta, tb)) = refine(&raw[s.el].p, &raw[o.el].p, ta, tb, tol) else { continue };
            // A curve meets itself only at two different parameters.
            if s.el == o.el {
                let p = &raw[s.el].p;
                let (a, b) = (p.c.point(ta.min(tb)), p.c.point(0.5 * (ta + tb)));
                if (ta - tb).abs() < 1e-9 * (p.t1 - p.t0).abs() || (a - b).norm() < tol * 10.0 {
                    continue;
                }
            }
            let (ea, eb) = (near_end(s.el, ta), near_end(o.el, tb));
            if ea && eb {
                continue;
            }
            if !ea {
                cuts[s.el].push(ta);
            }
            if !eb {
                cuts[o.el].push(tb);
            }
        }
    }
    tick("crossings");
    // The pieces between cuts, kept where they stay the distance from the wire.
    // (Less the wire's gaps: an arc about one edge's end lies that much nearer the next.)
    // (Within 1e-9 of the distance: a swallowtail of an offset curve lies very little nearer.)
    let keep_at = l.abs() * (1.0 - 1e-9) - (tol * 0.01).max(gap * 2.0).max(if closed { closing * 2.0 } else { 0.0 });
    // Boxes round the wire's edges (a B-spline's control points, else samples with a margin)
    // rule most edges out without a closest-point search.
    let boxes: Vec<Bbox> = wire.iter().map(edge_box).collect();
    let far = |p: &P3, err: f64| -> bool {
        let keep_at = keep_at - 2.0 * err;
        wire.iter().zip(&boxes).all(|(e, b)| {
            let dx = (b.min.x - p.x).max(p.x - b.max.x).max(0.0);
            let dy = (b.min.y - p.y).max(p.y - b.max.y).max(0.0);
            if dx * dx + dy * dy >= keep_at * keep_at {
                return true;
            }
            let q = e.c.point(e.c.closest(p, e.t0, e.t1));
            ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt() >= keep_at
        })
    };
    let mut kept: Vec<(usize, Piece2)> = Vec::new();
    for (el, r) in raw.iter().enumerate() {
        let mut ts = cuts[el].clone();
        ts.push(r.p.t0);
        ts.push(r.p.t1);
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-12 * (1.0 + b.abs()));
        // Cuts closer together than the tolerance are one (the ends stay). Not the two cuts of
        // a self-crossing, which meet at one point with a loop between (a swallowtail).
        let mut merged: Vec<f64> = Vec::new();
        let near = |a: f64, b: f64| (r.p.c.point(a) - r.p.c.point(b)).norm() < tol * 10.0;
        for (i, &t) in ts.iter().enumerate() {
            let last = i + 1 == ts.len();
            match merged.last() {
                Some(&m) if near(t, m) && [0.25, 0.5, 0.75].iter().all(|f| near(m + (t - m) * f, m)) => {
                    if last {
                        *merged.last_mut().unwrap() = t;
                    }
                }
                _ => merged.push(t),
            }
        }
        if merged.len() < 2 {
            merged = vec![r.p.t0, r.p.t1];
        }
        let ts = merged;
        let mut run: Option<Piece2> = None;
        for w in ts.windows(2) {
            let (a, b) = (w[0], w[1]);
            if b - a < 1e-14 * (1.0 + a.abs()) {
                continue;
            }
            let good = [0.25, 0.5, 0.75].iter().all(|f| far(&r.p.c.point(a + (b - a) * f), r.err));
            match (&mut run, good) {
                (Some(p), true) => p.t1 = b,
                (None, true) => run = Some(Piece2 { c: r.p.c.clone(), t0: a, t1: b }),
                (Some(_), false) => kept.push((el, run.take().unwrap())),
                (None, false) => {}
            }
        }
        if let Some(p) = run {
            kept.push((el, p));
        }
    }
    // Slivers (bits of an element between near cuts) close up in the chaining; a whole element
    // stays however short (the offset of a short edge, or of a corner).
    let sliver = chain_tol.min(tol::POINT);
    kept.retain(|(el, p)| (p.t0 == raw[*el].p.t0 && p.t1 == raw[*el].p.t1) || (p.end() - p.start()).norm() > sliver || polyline_len(p, 16) > sliver * 2.0);
    tick("kept");
    if std::env::var_os("CADRS_OFFSET_DEBUG").is_some() {
        eprintln!("offset l {l} closed {closed} size {size}: {} raw, {} cuts, {} kept", raw.len(), cuts.iter().map(Vec::len).sum::<usize>(), kept.len());
        for (i, r) in raw.iter().enumerate() {
            eprintln!("  raw {i}: {:?} -> {:?} cuts {:?}", p2(&r.p.start()), p2(&r.p.end()), cuts[i]);
        }
        for (el, p) in &kept {
            eprintln!("  kept {el}: {:?} -> {:?}", p2(&p.start()), p2(&p.end()));
        }
    }
    // Chained into loops (pieces that don't close up are dropped).
    let mut used = vec![false; kept.len()];
    let mut loops: Vec<Vec<Piece2>> = Vec::new();
    for s0 in 0..kept.len() {
        if used[s0] {
            continue;
        }
        let mut lp = vec![s0];
        used[s0] = true;
        let first = kept[s0].1.start();
        let ok;
        loop {
            let end = kept[*lp.last().unwrap()].1.end();
            if lp.len() > 1 && (end - first).norm() < chain_tol {
                ok = true;
                break;
            }
            let next = (0..kept.len())
                .filter(|&j| !used[j])
                .map(|j| (j, (kept[j].1.start() - end).norm()))
                .filter(|(_, dd)| *dd < chain_tol)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            match next {
                Some((j, _)) => {
                    used[j] = true;
                    lp.push(j);
                }
                None => {
                    ok = lp.len() == 1 && (end - first).norm() < chain_tol;
                    break;
                }
            }
        }
        // (A sliver closing on itself is a leftover of a corner, not a loop.)
        if ok && lp.iter().map(|&j| polyline_len(&kept[j].1, 16)).sum::<f64>() > chain_tol * 10.0 {
            // A piece split where its own curve's loop closes is one edge again.
            let mut pieces: Vec<Piece2> = lp.iter().map(|&j| kept[j].1.clone()).collect();
            let els: Vec<usize> = lp.iter().map(|&j| kept[j].0).collect();
            if pieces.len() > 1 && els[0] == *els.last().unwrap() && (pieces.last().unwrap().t1 - pieces[0].t0).abs() > 0.0 {
                let (a, b) = (&pieces[pieces.len() - 1], &pieces[0]);
                if (a.t1 - b.t0).abs() < 1e-12 * (1.0 + a.t1.abs()) {
                    let merged = Piece2 { c: a.c.clone(), t0: a.t0, t1: b.t1 };
                    pieces.pop();
                    pieces[0] = merged;
                }
            }
            loops.push(pieces);
        }
    }
    tick("loops");
    Ok(loops)
}

/// The length of the polyline through `n + 1` points of the piece (a lower bound).
fn polyline_len(p: &Piece2, n: usize) -> f64 {
    (0..n).map(|i| (p.c.point(p.t0 + (p.t1 - p.t0) * (i + 1) as f64 / n as f64) - p.c.point(p.t0 + (p.t1 - p.t0) * i as f64 / n as f64)).norm()).sum()
}

/// A box holding the piece: a B-spline's control points (a valid hull with positive weights),
/// else samples grown by the sag between them.
fn edge_box(e: &Piece2) -> Bbox {
    let mut b = Bbox::empty();
    if let Curve::Nurbs(n) = &e.c
        && n.cps.iter().all(|h| h.w > 0.0)
    {
        for h in &n.cps {
            b.add(&super::nurbs::dehom(h));
        }
        return b;
    }
    let k = 32;
    for i in 0..=k {
        b.add(&e.c.point(e.t0 + (e.t1 - e.t0) * i as f64 / k as f64));
    }
    b.grow(b.diagonal() * 0.01 + 1e-9)
}

/// The edges in the order and direction they chain in (each next the one whose end is nearest
/// the last end), from the first edge; an open wire from one of its free ends.
fn chained(mut rest: Vec<Piece2>) -> Vec<Piece2> {
    let rev = |e: &Piece2| {
        let (c, a, b) = e.c.reversed(e.t0, e.t1);
        Piece2 { c, t0: a, t1: b }
    };
    // Start at an end no other edge reaches (an open wire's), else at the first edge.
    let free = |p: P3, skip: usize, all: &[Piece2]| {
        all.iter().enumerate().filter(|(j, _)| *j != skip).map(|(_, e)| (e.start() - p).norm().min((e.end() - p).norm())).fold(f64::INFINITY, f64::min)
    };
    let mut start = (0, false, 0.0);
    for (i, e) in rest.iter().enumerate() {
        for (flip, p) in [(false, e.start()), (true, e.end())] {
            let d = free(p, i, &rest);
            if d > start.2 * 10.0 + 1e-12 && d > 1e-9 {
                start = (i, flip, d);
            }
        }
    }
    let lens: f64 = rest.iter().map(|e| (e.end() - e.start()).norm()).sum();
    let mut out = Vec::new();
    let first = rest.remove(start.0);
    out.push(if start.1 && start.2 > lens * 1e-4 { rev(&first) } else { first });
    while !rest.is_empty() {
        let end = out.last().unwrap().end();
        let (j, flip, _) = rest
            .iter()
            .enumerate()
            .flat_map(|(j, e)| [(j, false, (e.start() - end).norm()), (j, true, (e.end() - end).norm())])
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .unwrap();
        let e = rest.remove(j);
        out.push(if flip { rev(&e) } else { e });
    }
    out
}

/// Where segments ab and cd cross: their parameters, with `slack` at the ends.
fn seg_cross(a: &V2, b: &V2, c: &V2, d: &V2, slack: f64) -> Option<(f64, f64)> {
    let (r, s) = (b - a, d - c);
    let den = r.x * s.y - r.y * s.x;
    let (lr, ls) = (r.norm(), s.norm());
    if den.abs() <= 1e-14 * lr * ls {
        return None;
    }
    let q = c - a;
    let u = (q.x * s.y - q.y * s.x) / den;
    let v = (q.x * r.y - q.y * r.x) / den;
    let (eu, ev) = (slack / lr.max(1e-300), slack / ls.max(1e-300));
    if u < -eu || u > 1.0 + eu || v < -ev || v > 1.0 + ev {
        return None;
    }
    Some((u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)))
}

/// Newton on a(t) = b(u) from a guess, kept inside both ranges.
fn refine(a: &Piece2, b: &Piece2, mut t: f64, mut u: f64, tol: f64) -> Option<(f64, f64)> {
    for _ in 0..30 {
        let (pa, da, _) = a.c.d2(t);
        let (pb, db, _) = b.c.d2(u);
        let f = V2::new(pa.x - pb.x, pa.y - pb.y);
        if f.norm() < tol * 1e-3 {
            break;
        }
        let (j00, j01, j10, j11) = (da.x, -db.x, da.y, -db.y);
        let Some((dt, du)) = solve2(j00, j01, j10, j11, -f.x, -f.y) else { break };
        t = (t + dt).clamp(a.t0, a.t1);
        u = (u + du).clamp(b.t0, b.t1);
    }
    let (pa, pb) = (a.c.point(t), b.c.point(u));
    if ((pa.x - pb.x).powi(2) + (pa.y - pb.y).powi(2)).sqrt() < tol * 10.0 {
        Some((t, u))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn total_len(loops: &[Vec<Piece2>]) -> f64 {
        loops.iter().flatten().map(|p| p.c.length(p.t0, p.t1)).sum()
    }

    #[test]
    fn ellipse_cut_at_its_vertices() {
        let c = Curve::Ellipse { c: P3::origin(), x: V3::x(), y: V3::y(), a: 3.0, b: 1.0 };
        let ts = curvature_points(&c, 0.0, 2.0 * PI);
        assert_eq!(ts.len(), 5);
        let spine = prepare_spine(vec![Piece2 { c, t0: 0.0, t1: 2.0 * PI }]);
        assert_eq!(spine.len(), 4);
    }

    #[test]
    fn single_closed_circle_cut_in_three() {
        // OCCT's ForcedCut: a closed wire of one uncut edge is cut in three.
        let c = Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::y(), r: 2.0 };
        let loops = offset_wire(&[(c, 0.0, 2.0 * PI)], 0.5).unwrap();
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 3);
        assert!((total_len(&loops) - 2.0 * PI * 2.5).abs() < 1e-9);
    }

    #[test]
    fn b_spline_cut_at_curvature_extremum() {
        // A symmetric cubic arch: curvature peaks at its middle.
        let n = NurbsCurve::bezier(&[P3::new(0.0, 0.0, 0.0), P3::new(1.0, 2.0, 0.0), P3::new(3.0, 2.0, 0.0), P3::new(4.0, 0.0, 0.0)], &[1.0; 4]);
        let ts = curvature_points(&Curve::nurbs(n), 0.0, 1.0);
        assert!(ts.iter().any(|t| (t - 0.5).abs() < 1e-6), "{ts:?}");
    }

    #[test]
    fn slight_convex_corner_gets_an_arc() {
        // A square whose top edge is tilted by 1e-6 rad: outwards the corner is convex and gets
        // a (tiny) arc as OCCT's, inwards none.
        let pts = [P3::new(0.0, 0.0, 0.0), P3::new(10.0, 0.0, 0.0), P3::new(10.0, 10.0, 0.0), P3::new(5.0, 10.0, 0.0), P3::new(0.0, 10.0 - 5e-6, 0.0)];
        let edges: Vec<(Curve, f64, f64)> = (0..5)
            .map(|i| {
                let (c, l) = Curve::line(pts[i], pts[(i + 1) % 5]);
                (c, 0.0, l)
            })
            .collect();
        let out = offset_wire(&edges, 1.0).unwrap();
        let inn = offset_wire(&edges, -1.0).unwrap();
        // Outwards: 5 edges, 4 corner arcs and the slight one; inwards the 5 edges.
        assert_eq!(out[0].len(), 10);
        assert_eq!(inn[0].len(), 5);
    }

    #[test]
    fn nearly_planar_points_fit_a_plane() {
        let pts: Vec<P3> = (0..20).map(|i| P3::new(i as f64, (i * i) as f64 * 0.1, 1e-4 * (i % 3) as f64)).collect();
        let (_, n) = best_plane(&pts, 1e-2).unwrap();
        assert!(n.z.abs() > 0.999);
        assert!(best_plane(&pts, 1e-9).is_none());
    }
}
