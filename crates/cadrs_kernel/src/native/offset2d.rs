//! Offsets of planar wires (DRAW's `mkoffset`, OCCT's `BRepOffsetAPI_MakeOffset` with arc
//! joins), in the wire's plane (z = 0 here).
//!
//! 1. Every edge is offset to one side: lines and arcs exactly (an arc whose radius would fall
//!    to nothing is dropped), other curves as cubics through offset samples.
//! 2. Consecutive offsets are joined by arcs about the vertex between them, turning as the
//!    wire turns. An open wire is run out and back, so its ends are half turns (round caps).
//! 3. The raw loop is split where it crosses itself, and only the pieces that stay the full
//!    distance from the wire are kept (the rest lie in overlaps: concave corners, arcs inside
//!    the distance, loops of the offset curves).
//! 4. Neighbouring pieces of one curve are joined again and the pieces chained into loops.

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
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
        let (c, a, b) = c.reversed(t0, t1);
        Piece2 { c, t0: a, t1: b }
    }
}

/// The edge offset `l` to its left, or None when it shrinks to nothing.
fn offset_edge(e: &Piece2, l: f64, size: f64) -> Option<Piece2> {
    match &e.c {
        Curve::Line { .. } => {
            let n = left(e.dir(e.t0));
            let sh = V3::new(n.x, n.y, 0.0) * l;
            let (c, len) = Curve::line(e.start() + sh, e.end() + sh);
            Some(Piece2 { c, t0: 0.0, t1: len })
        }
        Curve::Circle { c, x, y, r } => {
            // Run counter-clockwise the left is the centre's side.
            let ccw = x.cross(y).z > 0.0;
            let r2 = if ccw { r - l } else { r + l };
            if r2 <= 1e-9 * size {
                return None;
            }
            Some(Piece2 { c: Curve::Circle { c: *c, x: *x, y: *y, r: r2 }, t0: e.t0, t1: e.t1 })
        }
        _ => {
            let mut ts = e.c.sample_params(e.t0, e.t1, size * 1e-5, 0.05);
            if ts.len() < 65 {
                ts = (0..=64).map(|i| e.t0 + (e.t1 - e.t0) * i as f64 / 64.0).collect();
            }
            let pts: Vec<P3> = ts
                .iter()
                .map(|&t| {
                    let n = left(e.dir(t));
                    e.c.point(t) + V3::new(n.x, n.y, 0.0) * l
                })
                .collect();
            let params = NurbsCurve::chord_params(&pts);
            let c = NurbsCurve::interpolate(&pts, &params, 3, None, None);
            let (a, b) = c.range();
            Some(Piece2 { c: Curve::Nurbs(std::sync::Arc::new(c)), t0: a, t1: b })
        }
    }
}

/// The arc about `v` from `v + l·n1` turning by `phi`.
fn join_arc(v: P3, n1: V2, phi: f64, l: f64) -> Piece2 {
    let u0 = n1 * l.signum();
    let y = left(u0) * phi.signum();
    Piece2 { c: Curve::Circle { c: v, x: V3::new(u0.x, u0.y, 0.0), y: V3::new(y.x, y.y, 0.0), r: l.abs() }, t0: 0.0, t1: phi.abs() }
}

/// The loops of the wire `edges` (in the plane z = 0, chained in order) offset by `d`: a closed
/// wire outwards for d > 0, an open one on both sides with round ends.
pub fn offset_wire(edges: &[(Curve, f64, f64)], d: f64) -> Result<Vec<Vec<Piece2>>> {
    if edges.is_empty() || d == 0.0 {
        return Err(KernelError::InvalidParameter("an offset needs edges and a distance".into()));
    }
    let wire = chained(edges.iter().map(|(c, a, b)| forward(c, *a, *b)).collect());
    let mut bb = Bbox::empty();
    for e in &wire {
        for k in 0..=8 {
            bb.add(&e.c.point(e.t0 + (e.t1 - e.t0) * k as f64 / 8.0));
        }
    }
    let size = bb.diagonal().max(d.abs());
    let tol = size * 1e-7;
    // Edges shorter than the wire's tolerance (slivers between real edges) are left out: their
    // ends join the neighbours within the chaining tolerance.
    let wire: Vec<Piece2> = if wire.len() > 1 {
        let w: Vec<Piece2> = wire.iter().filter(|e| e.c.length(e.t0, e.t1) > size * 1e-5).cloned().collect();
        if w.is_empty() { wire } else { w }
    } else {
        wire
    };
    // The wire's own gaps between edges (vertex tolerances) set the closure and chaining.
    let gap = wire.windows(2).map(|w| (w[0].end() - w[1].start()).norm()).fold(0.0, f64::max);
    let closing = (wire[0].start() - wire.last().unwrap().end()).norm();
    let closed = (wire[0].start() - wire.last().unwrap().end()).norm() < (tol * 10.0).max(size * 1e-4).max(gap * 3.0);
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
    let clock = std::time::Instant::now();
    let tick = |what: &str| {
        if std::env::var_os("CADRS_OFFSET_DEBUG").is_some() {
            eprintln!("  {what}: {:?}", clock.elapsed());
        }
    };
    let mut raw: Vec<Raw> = Vec::new();
    for i in 0..n {
        let e = &seq[i];
        if let Some(o) = offset_edge(e, l, size) {
            raw.push(Raw { p: o });
        }
        // The join to the next edge.
        let next = &seq[(i + 1) % n];
        let (t1, t2) = (e.dir(e.t1), next.dir(next.t0));
        let mut phi = (t1.x * t2.y - t1.y * t2.x).atan2(t1.dot(&t2));
        // Tangent (within the wire's gaps): the offsets meet without an arc.
        if phi.abs() < 1e-9 || (l * phi).abs() <= chain_tol {
            continue;
        }
        if phi.abs() > PI - 1e-9 {
            phi = -PI * l.signum();
        }
        raw.push(Raw { p: join_arc(e.end(), left(t1), phi, l) });
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
    let dist = |p: &P3| -> f64 {
        wire.iter()
            .map(|e| {
                let t = e.c.closest(p, e.t0, e.t1);
                let q = e.c.point(t);
                ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt()
            })
            .fold(f64::INFINITY, f64::min)
    };
    // (Less the wire's gaps: an arc about one edge's end lies that much nearer the next.)
    let keep_at = l.abs() * (1.0 - 1e-5) - (tol * 10.0).max(gap * 2.0).max(if closed { closing * 2.0 } else { 0.0 });
    let mut kept: Vec<(usize, Piece2)> = Vec::new();
    for (el, r) in raw.iter().enumerate() {
        let mut ts = cuts[el].clone();
        ts.push(r.p.t0);
        ts.push(r.p.t1);
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-12 * (1.0 + b.abs()));
        // Cuts closer together than the tolerance are one (the ends stay).
        let mut merged: Vec<f64> = Vec::new();
        for (i, &t) in ts.iter().enumerate() {
            let last = i + 1 == ts.len();
            match merged.last() {
                Some(&m) if (r.p.c.point(t) - r.p.c.point(m)).norm() < tol * 10.0 => {
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
            let good = [0.25, 0.5, 0.75].iter().all(|f| dist(&r.p.c.point(a + (b - a) * f)) >= keep_at);
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
    // Slivers shorter than the chaining tolerance close up in the chaining.
    kept.retain(|(_, p)| (p.end() - p.start()).norm() > chain_tol || p.c.length(p.t0, p.t1) > chain_tol * 2.0);
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
        if ok {
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
    Ok(loops)
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
