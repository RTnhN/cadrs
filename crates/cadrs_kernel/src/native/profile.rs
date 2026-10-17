//! Sketch profiles as exact 2D pieces: validation, orientation (outer loops counter-clockwise,
//! holes clockwise) and the merging of regions that share boundary curves (so touching
//! regions extrude as one body with one cap).

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
use crate::{unsourced_curve, Curve2, KernelError, Profile, Result};

/// A piece of a loop: the curve (z = 0) between `t0 < t1`, run backwards when `rev`.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub c: Curve,
    pub t0: f64,
    pub t1: f64,
    pub rev: bool,
    /// (region source, curve source) for naming the side face.
    pub src: (u64, u64),
    /// Further origins when pieces of several regions were fused into this one.
    pub more: Vec<(u64, u64)>,
}

impl Piece {
    pub fn ts(&self) -> f64 {
        if self.rev { self.t1 } else { self.t0 }
    }
    pub fn te(&self) -> f64 {
        if self.rev { self.t0 } else { self.t1 }
    }
    pub fn start(&self) -> P2 {
        let p = self.c.point(self.ts());
        P2::new(p.x, p.y)
    }
    pub fn end(&self) -> P2 {
        let p = self.c.point(self.te());
        P2::new(p.x, p.y)
    }
    /// The point at fraction s along the direction of travel.
    pub fn at(&self, s: f64) -> P2 {
        let t = self.ts() + (self.te() - self.ts()) * s;
        let p = self.c.point(t);
        P2::new(p.x, p.y)
    }
    /// Unit tangent in the direction of travel at fraction s.
    pub fn dir(&self, s: f64) -> V2 {
        let t = self.ts() + (self.te() - self.ts()) * s;
        let d = self.c.tangent(t);
        let d = V2::new(d.x, d.y) * if self.rev { -1.0 } else { 1.0 };
        let l = d.norm();
        if l > 0.0 { d / l } else { d }
    }
    pub fn length(&self) -> f64 {
        self.c.length(self.t0, self.t1)
    }
    pub fn reversed(&self) -> Piece {
        Piece { rev: !self.rev, ..self.clone() }
    }
    /// ∮ x dy along the direction of travel (Green: the signed area of a closed loop).
    pub fn x_dy(&self) -> f64 {
        let ts = self.c.pieces(self.t0, self.t1);
        let mut s = 0.0;
        for w in ts.windows(2) {
            s += integrate(w[0], w[1], 2, &GAUSS16, |t| {
                let (p, d, _) = self.c.d2(t);
                p.x * d.y
            });
        }
        if self.rev { -s } else { s }
    }
    /// Points along the piece (both ends included), at most `defl` from it.
    pub fn sample(&self, defl: f64) -> Vec<P2> {
        let ts = self.c.sample_params(self.t0, self.t1, defl, 0.2);
        let mut pts: Vec<P2> = ts.iter().map(|&t| {
            let p = self.c.point(t);
            P2::new(p.x, p.y)
        }).collect();
        if self.rev {
            pts.reverse();
        }
        pts
    }
    /// The same carrier curve (so two pieces can overlap).
    pub fn same_carrier(&self, o: &Piece, tol: f64) -> bool {
        match (&self.c, &o.c) {
            (Curve::Line { o: a, d: da }, Curve::Line { o: b, d: db }) => {
                let cr = da.x * db.y - da.y * db.x;
                if cr.abs() > 1e-9 {
                    return false;
                }
                let w = b - a;
                (w.x * da.y - w.y * da.x).abs() < tol
            }
            (Curve::Circle { c: a, r: ra, .. }, Curve::Circle { c: b, r: rb, .. }) => (a - b).norm() < tol && (ra - rb).abs() < tol,
            (a, b) => a == b,
        }
    }
    /// The parameter on the carrier of a point on it.
    pub fn param_of(&self, p: &P2) -> f64 {
        let q = P3::new(p.x, p.y, 0.0);
        match &self.c {
            Curve::Circle { c, x, y, .. } => {
                let v = q - c;
                let a = v.dot(y).atan2(v.dot(x));
                unwrap_into(a, self.t0, self.t1)
            }
            Curve::Ellipse { .. } => {
                let t = self.c.closest(&q, self.t0 - TAU, self.t1 + TAU);
                unwrap_into(t, self.t0, self.t1)
            }
            _ => self.c.closest(&q, self.t0 - 1e9, self.t1 + 1e9),
        }
    }
    /// Splits at carrier parameters strictly inside, keeping the direction of travel.
    pub fn split_at(&self, mut ts: Vec<f64>) -> Vec<Piece> {
        ts.retain(|t| *t > self.t0 && *t < self.t1);
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        let mut bounds = vec![self.t0];
        bounds.extend(ts);
        bounds.push(self.t1);
        let mut out: Vec<Piece> = bounds.windows(2).map(|w| Piece { t0: w[0], t1: w[1], ..self.clone() }).collect();
        if self.rev {
            out.reverse();
        }
        out
    }
}

/// `a` shifted by whole turns towards [t0, t1].
fn unwrap_into(a: f64, t0: f64, t1: f64) -> f64 {
    let m = (t0 + t1) * 0.5;
    unwrap_near(a, m, TAU)
}

pub type Loop2 = Vec<Piece>;

#[derive(Debug, Clone)]
pub struct Region2 {
    pub outer: Loop2,
    pub holes: Vec<Loop2>,
    pub source: u64,
}

/// A planar face to build: an outer loop, holes and the regions it covers.
#[derive(Debug, Clone)]
pub struct MergedFace {
    pub outer: Loop2,
    pub holes: Vec<Loop2>,
    pub regions: Vec<u64>,
}

pub fn loop_area(l: &[Piece]) -> f64 {
    l.iter().map(|p| p.x_dy()).sum()
}

pub fn reverse_loop(l: &[Piece]) -> Loop2 {
    l.iter().rev().map(|p| p.reversed()).collect()
}

/// A sampled polygon of the loop.
pub fn loop_polygon(l: &[Piece], defl: f64) -> Vec<P2> {
    let mut out = Vec::new();
    for p in l {
        let s = p.sample(defl);
        out.extend_from_slice(&s[..s.len() - 1]);
    }
    out
}

/// A 2D curve from a sketch curve, with the piece's range and direction.
pub fn piece_of(c: &Curve2, src: (u64, u64)) -> Result<Piece> {
    let e2 = |v: V2| V3::new(v.x, v.y, 0.0);
    let p3 = |p: P2| P3::new(p.x, p.y, 0.0);
    let arc = |c: Curve, start: f64, sweep: f64| {
        if sweep >= 0.0 {
            Piece { c, t0: start, t1: start + sweep, rev: false, src, more: vec![] }
        } else {
            Piece { c, t0: start + sweep, t1: start, rev: true, src, more: vec![] }
        }
    };
    let ellipse = |center: P2, major: f64, minor: f64, rotation: f64| {
        let (s, co) = rotation.sin_cos();
        let x = V2::new(co, s) * major.signum();
        let y = V2::new(-s, co) * minor.signum();
        Curve::Ellipse { c: p3(center), x: e2(x), y: e2(y), a: major.abs(), b: minor.abs() }
    };
    let piece = match *c {
        Curve2::Line { a, b, .. } => {
            let d = b - a;
            let l = d.norm();
            if l < TOL {
                return Err(KernelError::InvalidProfile("a line shorter than the modelling tolerance".into()));
            }
            Piece { c: Curve::Line { o: p3(a), d: e2(d / l) }, t0: 0.0, t1: l, rev: false, src, more: vec![] }
        }
        Curve2::Arc { center, radius, start_angle, sweep, .. } => {
            if radius < TOL || (radius * sweep).abs() < TOL {
                return Err(KernelError::InvalidProfile("a degenerate arc".into()));
            }
            arc(Curve::Circle { c: p3(center), x: V3::x(), y: V3::y(), r: radius }, start_angle, sweep)
        }
        Curve2::Circle { center, radius, .. } => {
            if radius < TOL {
                return Err(KernelError::InvalidProfile("a degenerate circle".into()));
            }
            Piece { c: Curve::Circle { c: p3(center), x: V3::x(), y: V3::y(), r: radius }, t0: 0.0, t1: TAU, rev: false, src, more: vec![] }
        }
        Curve2::Ellipse { center, major_radius, minor_radius, rotation, .. } => {
            if major_radius.abs() < TOL || minor_radius.abs() < TOL {
                return Err(KernelError::InvalidProfile("a degenerate ellipse".into()));
            }
            Piece { c: ellipse(center, major_radius, minor_radius, rotation), t0: 0.0, t1: TAU, rev: false, src, more: vec![] }
        }
        Curve2::EllipseArc { center, major_radius, minor_radius, rotation, start, sweep, .. } => {
            if major_radius.abs() < TOL || minor_radius.abs() < TOL || sweep.abs() < 1e-12 {
                return Err(KernelError::InvalidProfile("a degenerate ellipse arc".into()));
            }
            arc(ellipse(center, major_radius, minor_radius, rotation), start, sweep)
        }
        Curve2::OffsetEllipseArc { center, major_radius, minor_radius, rotation, start, sweep, offset, .. } => {
            let (a, b) = (major_radius.abs(), minor_radius.abs());
            let rmin = a.min(b).powi(2) / a.max(b);
            if offset <= -rmin + 1e-9 {
                return Err(KernelError::InvalidParameter(format!(
                    "The offset {:.3} reaches past the ellipse's smallest radius of curvature ({:.3})",
                    -offset, rmin
                )));
            }
            let n = offset_ellipse_nurbs(center, major_radius, minor_radius, rotation, start, sweep, offset);
            if sweep >= 0.0 {
                Piece { c: Curve::Nurbs(Arc::new(n)), t0: start, t1: start + sweep, rev: false, src, more: vec![] }
            } else {
                Piece { c: Curve::Nurbs(Arc::new(n)), t0: start + sweep, t1: start, rev: true, src, more: vec![] }
            }
        }
        Curve2::Bezier { poles, .. } => {
            let n = NurbsCurve::bezier(&poles.map(p3), &[1.0; 4]);
            Piece { c: Curve::Nurbs(Arc::new(n)), t0: 0.0, t1: 1.0, rev: false, src, more: vec![] }
        }
    };
    Ok(piece)
}

/// The offset ellipse as a C2 cubic B-spline interpolating the exact curve at the ellipse
/// parameter (the knots are the ellipse angles, so the B-spline's t is the ellipse's).
fn offset_ellipse_nurbs(center: P2, major: f64, minor: f64, rotation: f64, start: f64, sweep: f64, offset: f64) -> NurbsCurve {
    let (lo, hi) = if sweep >= 0.0 { (start, start + sweep) } else { (start + sweep, start) };
    let n = ((hi - lo).abs() / TAU * 1024.0).ceil().max(16.0) as usize;
    let ts: Vec<f64> = (0..=n).map(|i| lo + (hi - lo) * i as f64 / n as f64).collect();
    let pts: Vec<P3> = ts
        .iter()
        .map(|&t| {
            let q = crate::offset_ellipse_point(center, major, minor, rotation, offset, t);
            P3::new(q.x, q.y, 0.0)
        })
        .collect();
    // End derivatives by central differences of the exact curve.
    let h = 1e-6;
    let d = |t: f64| {
        let a = crate::offset_ellipse_point(center, major, minor, rotation, offset, t - h);
        let b = crate::offset_ellipse_point(center, major, minor, rotation, offset, t + h);
        V3::new(b.x - a.x, b.y - a.y, 0.0) / (2.0 * h)
    };
    NurbsCurve::interpolate(&pts, &ts, 3, Some(d(lo)), Some(d(hi)))
}

/// Converts and validates the profile's regions: closed loops, outer counter-clockwise, holes
/// clockwise.
pub fn regions(profile: &Profile) -> Result<Vec<Region2>> {
    let mut out = Vec::new();
    for (ri, r) in profile.regions.iter().enumerate() {
        let source = r.source.unwrap_or(ri as u64);
        let mut lp = |l: &crate::Loop, li: usize| -> Result<Loop2> {
            let mut pieces = Vec::new();
            for (ci, c) in l.curves.iter().enumerate() {
                let cs = c.source().unwrap_or(unsourced_curve(li, ci));
                pieces.push(piece_of(c, (source, cs))?);
            }
            let pieces = chain_loop(pieces)?;
            check_closed(&pieces)?;
            Ok(pieces)
        };
        let mut outer = lp(&r.outer, 0)?;
        let a = loop_area(&outer);
        if a.abs() < 1e-10 {
            return Err(KernelError::InvalidProfile("a region with no area".into()));
        }
        if a < 0.0 {
            outer = reverse_loop(&outer);
        }
        let mut holes = Vec::new();
        for (hi, h) in r.holes.iter().enumerate() {
            let mut h = lp(h, hi + 1)?;
            let a = loop_area(&h);
            if a.abs() < 1e-10 {
                return Err(KernelError::InvalidProfile("a hole with no area".into()));
            }
            if a > 0.0 {
                h = reverse_loop(&h);
            }
            holes.push(h);
        }
        out.push(Region2 { outer, holes, source });
    }
    Ok(out)
}

/// Joints within this distance count as joined.
pub const JOIN: f64 = 1e-5;

pub fn check_closed(l: &[Piece]) -> Result<()> {
    if l.is_empty() {
        return Err(KernelError::InvalidProfile("an empty loop".into()));
    }
    // Reorder so each piece starts where the previous ends (sketch loops may list curves in
    // any direction): flip pieces as needed, in order.
    for k in 0..l.len() {
        let (a, b) = (&l[k], &l[(k + 1) % l.len()]);
        if (a.end() - b.start()).norm() > JOIN {
            return Err(KernelError::InvalidProfile(format!(
                "the loop is not closed: a gap of {:.3e} mm",
                (a.end() - b.start()).norm()
            )));
        }
    }
    Ok(())
}

/// Puts a loop's pieces in order and direction so each starts where the previous ends (sketch
/// loops list their curves in their own directions).
pub fn chain_loop(mut pieces: Vec<Piece>) -> Result<Loop2> {
    if pieces.len() <= 1 {
        return Ok(pieces);
    }
    let mut out = vec![pieces.remove(0)];
    // The first piece's direction: keep it unless its end meets nothing but its start does.
    while !pieces.is_empty() {
        let end = out.last().unwrap().end();
        let mut found = None;
        for (i, p) in pieces.iter().enumerate() {
            if (p.start() - end).norm() < JOIN {
                found = Some((i, false));
                break;
            }
            if (p.end() - end).norm() < JOIN {
                found = Some((i, true));
                break;
            }
        }
        match found {
            Some((i, flip)) => {
                let p = pieces.remove(i);
                out.push(if flip { p.reversed() } else { p });
            }
            None => {
                if out.len() == 1 {
                    // Try the first piece the other way round.
                    let p = out.pop().unwrap().reversed();
                    out.push(p);
                    let end = out[0].end();
                    if !pieces.iter().any(|p| (p.start() - end).norm() < JOIN || (p.end() - end).norm() < JOIN) {
                        return Err(KernelError::InvalidProfile("the loop is not closed".into()));
                    }
                    continue;
                }
                return Err(KernelError::InvalidProfile("the loop is not closed".into()));
            }
        }
    }
    Ok(out)
}

/// Merges regions that share boundary pieces into faces (one cap per connected group).
pub fn merge_regions(regions: &[Region2]) -> Result<Vec<MergedFace>> {
    let separate = || -> Vec<MergedFace> {
        regions.iter().map(|r| MergedFace { outer: r.outer.clone(), holes: r.holes.clone(), regions: vec![r.source] }).collect()
    };
    if regions.len() < 2 {
        return Ok(separate());
    }
    // All loops with their owner.
    let mut loops: Vec<Loop2> = Vec::new();
    for r in regions {
        loops.push(r.outer.clone());
        loops.extend(r.holes.iter().cloned());
    }
    let size = {
        let mut bb = Bbox::empty();
        for l in &loops {
            for p in l {
                let q = p.start();
                bb.add(&P3::new(q.x, q.y, 0.0));
            }
        }
        bb.diagonal().max(1.0)
    };
    let tol = JOIN.max(size * 1e-10);
    // Split every piece at the vertices of other loops lying on it.
    let verts: Vec<(usize, P2)> = loops.iter().enumerate().flat_map(|(li, l)| l.iter().map(move |p| (li, p.start()))).collect();
    let mut pieces: Vec<(usize, Piece)> = Vec::new();
    for (li, l) in loops.iter().enumerate() {
        for p in l {
            let mut ts = Vec::new();
            for (lj, v) in &verts {
                if *lj == li {
                    continue;
                }
                let t = p.param_of(v);
                if t > p.t0 + 1e-9 && t < p.t1 - 1e-9 {
                    let q = p.c.point(t);
                    if (P2::new(q.x, q.y) - v).norm() < tol {
                        ts.push(t);
                    }
                }
            }
            for s in p.split_at(ts) {
                pieces.push((li, s));
            }
        }
    }
    // Cancel coincident pieces run in opposite directions.
    let mut dead = vec![false; pieces.len()];
    let mut any = false;
    for i in 0..pieces.len() {
        if dead[i] {
            continue;
        }
        for j in i + 1..pieces.len() {
            if dead[j] || pieces[i].0 == pieces[j].0 {
                continue;
            }
            let (a, b) = (&pieces[i].1, &pieces[j].1);
            if (a.start() - b.end()).norm() < tol && (a.end() - b.start()).norm() < tol && (a.at(0.5) - b.at(0.5)).norm() < tol && a.same_carrier(b, tol) {
                dead[i] = true;
                dead[j] = true;
                any = true;
                break;
            }
        }
    }
    if !any {
        return Ok(separate());
    }
    let live: Vec<Piece> = pieces.into_iter().zip(dead).filter(|(_, d)| !d).map(|((_, p), _)| p).collect();
    // Trace loops, turning as far left as possible at junctions.
    let mut used = vec![false; live.len()];
    let mut traced: Vec<Loop2> = Vec::new();
    for s in 0..live.len() {
        if used[s] {
            continue;
        }
        let mut lp = vec![live[s].clone()];
        used[s] = true;
        let start = live[s].start();
        loop {
            let cur = lp.last().unwrap();
            let end = cur.end();
            if (end - start).norm() < tol {
                break;
            }
            let din = cur.dir(1.0);
            let mut best: Option<(f64, usize)> = None;
            for (k, q) in live.iter().enumerate() {
                if used[k] || (q.start() - end).norm() >= tol {
                    continue;
                }
                let dout = q.dir(0.0);
                let ang = cross2(&din, &dout).atan2(din.dot(&dout));
                if best.map_or(true, |(b, _)| ang > b) {
                    best = Some((ang, k));
                }
            }
            match best {
                Some((_, k)) => {
                    used[k] = true;
                    lp.push(live[k].clone());
                }
                None => {
                    return Err(KernelError::InvalidProfile("the regions' boundaries don't close after merging".into()));
                }
            }
            if lp.len() > live.len() {
                return Err(KernelError::InvalidProfile("the regions' boundaries don't close after merging".into()));
            }
        }
        traced.push(fuse_pieces(lp, tol));
    }
    // Outer loops and holes.
    let mut outers: Vec<(Loop2, Vec<P2>, f64)> = Vec::new();
    let mut holes: Vec<Loop2> = Vec::new();
    for l in traced {
        let a = loop_area(&l);
        if a > 0.0 {
            let poly = loop_polygon(&l, size * 1e-4);
            outers.push((l, poly, a));
        } else {
            holes.push(l);
        }
    }
    let mut faces: Vec<MergedFace> = outers.iter().map(|(l, _, _)| MergedFace { outer: l.clone(), holes: vec![], regions: vec![] }).collect();
    for h in holes {
        let q = h[0].at(0.5);
        let best = outers
            .iter()
            .enumerate()
            .filter(|(_, (_, poly, _))| point_in_polygon(&q, poly))
            .min_by(|a, b| a.1 .2.total_cmp(&b.1 .2))
            .map(|(i, _)| i);
        match best {
            Some(i) => faces[i].holes.push(h),
            None => return Err(KernelError::InvalidProfile("a hole outside every region".into())),
        }
    }
    // Which face each region went into: a point just inside its outer loop.
    for r in regions {
        let p = &r.outer[0];
        let d = p.dir(0.5);
        let len = p.length();
        let q = p.at(0.5) + V2::new(-d.y, d.x) * (len * 1e-3).min(size * 1e-5).max(1e-7);
        let mut found = None;
        for (i, (_, poly, _)) in outers.iter().enumerate() {
            if point_in_polygon(&q, poly) {
                let in_hole = faces[i].holes.iter().any(|h| point_in_polygon(&q, &loop_polygon(h, size * 1e-4)));
                if !in_hole {
                    found = Some(i);
                    break;
                }
            }
        }
        let i = found.unwrap_or(0);
        faces[i].regions.push(r.source);
    }
    Ok(faces)
}

/// Fuses consecutive pieces of different regions that continue each other on one line or
/// circle (Parasolid merges the side faces of touching regions).
fn fuse_pieces(mut lp: Loop2, tol: f64) -> Loop2 {
    let mut changed = true;
    while changed && lp.len() > 1 {
        changed = false;
        let n = lp.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let (a, b) = (&lp[i], &lp[j]);
            if a.src.0 == b.src.0 || !a.same_carrier(b, tol) || a.rev != b.rev {
                continue;
            }
            if a.dir(1.0).dot(&b.dir(0.0)) < 1.0 - 1e-9 {
                continue;
            }
            let mut more = a.more.clone();
            more.push(b.src);
            more.extend(b.more.iter().copied());
            let fused = match &a.c {
                Curve::Line { .. } => {
                    let (s, e) = (a.start(), b.end());
                    let d = e - s;
                    let l = d.norm();
                    Piece { c: Curve::Line { o: P3::new(s.x, s.y, 0.0), d: V3::new(d.x / l, d.y / l, 0.0) }, t0: 0.0, t1: l, rev: false, src: a.src, more }
                }
                Curve::Circle { .. } => {
                    let span = b.t1 - b.t0;
                    let (t0, t1) = if a.rev { (a.t0 - span, a.t1) } else { (a.t0, a.t1 + span) };
                    if t1 - t0 > TAU + 1e-9 {
                        continue;
                    }
                    Piece { c: a.c.clone(), t0, t1, rev: a.rev, src: a.src, more }
                }
                _ => continue,
            };
            lp[i] = fused;
            lp.remove(j);
            if j < i {
                lp.rotate_left(0);
            }
            changed = true;
            break;
        }
    }
    lp
}
