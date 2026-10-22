//! Constant-radius fillets by local trimming, as SMLib's fillet executive builds them (NVIDIA
//! usd-brep, Apache-2.0; `SmFilletExecutive`, `SmFilletGeom`, `SmFilletCorner`): no boolean.
//!
//! Every picked edge gets a strip: the surface the rolling ball sweeps between the edge's two
//! faces, with its rails (the contact curves, [`super::fillet_geom`]). Each end of a strip is
//! settled by what meets at the edge's vertex there:
//! - a free end (no other picked edge there): each rail runs on until it leaves its face across
//!   one of the face's edges (solved on the exact surfaces, not by intersecting curves), and the
//!   strip is cut by the face both rails run into (the end face);
//! - a tangent neighbour (the next edge of a smooth chain): both strips stop at the section in
//!   the plane square to the edges at the vertex, which they share;
//! - a corner of three picked edges (all convex or all concave, three faces): the ball touching
//!   all three faces; each strip stops at its section through that ball, and a sphere patch
//!   bounded by the three sections fills the corner (SMLib's `ConvexNxN` vertex blend);
//! - a convex and a concave fillet at a vertex of three edges: both stop where their rails cross
//!   on their common face, one at its section there, the other running on to the edge left as
//!   it is, and a three-sided face fills the corner ([`Knee`]).
//!
//! A closed edge (a circle round a cylinder's end) has no ends: its strip is a band whose rails
//! are closed, with a seam (a section) where its faces' seams are. A rail off its face all
//! round (a round on a wide hole's rim) has that face go whole and the band cut by the faces
//! round it; a rail that is a point (a cylinder's end rounded with its own radius) makes the
//! band a sphere cap, its face going whole too.
//!
//! The rails are imprinted into their faces and the end curves into the end faces, the pieces
//! between them and the edges (and the edges' vertices) are dropped, and the strips and corner
//! patches are put in their place. The rails are where the fillet surface touches its faces, so
//! no intersection between tangent surfaces is ever computed. What this doesn't cover yet is
//! [`KernelError::Unsupported`], for the caller to fall back to the tool-and-boolean blend.

use super::blend::FaceSrc;
use super::curve::Curve;
use super::fillet_geom::{self as geom, Ball, Boundary, Contact, Exact, Spine, Strip, StripKind};
use super::math::*;
use super::nurbs::NurbsCurve;
use super::split::{split_face, Arc};
use super::surface::Surface;
use super::topo::{Body, CheckLevel, Coedge, Face, Shell};
use crate::*;
use std::collections::HashMap;
use super::tol;

/// Progress on stderr with `CADRS_FILLET_DEBUG=1`.
fn debug(what: impl FnOnce() -> String) {
    if std::env::var_os("CADRS_FILLET_DEBUG").is_some() {
        thread_local!(static START: std::time::Instant = std::time::Instant::now());
        let ms = START.with(|s| s.elapsed().as_millis());
        eprintln!("fillet [{ms} ms]: {}", what());
    }
}

/// The work a local fillet may do, in ball solves (each sections the strip once: a few
/// Newton steps on the two faces), shared by its strips: a fillet that can't be built with that
/// much is reported, never left running. Counted, not timed, so the same input always gets the
/// same answer whatever the machine's load.
#[derive(Clone, Debug)]
struct Budget(std::rc::Rc<std::cell::Cell<u64>>);

/// Ball solves a fillet may take: seven times the most a fillet that's built takes (1.4·10⁵,
/// OCCT blend complex_g1; SMLib's fillets under 9·10⁴).
const BALL_SOLVES: u64 = 1_000_000;

impl Budget {
    fn new(solves: u64) -> Budget {
        Budget(std::rc::Rc::new(std::cell::Cell::new(solves)))
    }
    /// Takes one solve; false when none are left.
    fn spend(&self) -> bool {
        let left = self.0.get();
        self.0.set(left.saturating_sub(1));
        left > 0
    }
    fn spent(&self) -> bool {
        self.0.get() == 0
    }
}

fn unsupported(why: &'static str) -> KernelError {
    KernelError::Unsupported(why)
}

/// The local fillet gave up at `stage` (`"rolling ball"`, `"corner"`, `"end"`, `"surface"` or
/// `"trimming"`): see [`KernelError::BlendFailed`].
fn failed(stage: &'static str, why: &str) -> KernelError {
    KernelError::BlendFailed { stage, why: why.into() }
}

/// A radius too large for the faces at its edge.
fn too_large(why: &str) -> KernelError {
    KernelError::TooLarge { op: "fillet", why: why.into() }
}

/// Geometry at the edge that doesn't allow the fillet.
fn bad_geometry(why: &str) -> KernelError {
    KernelError::InvalidGeometry { op: "fillet", why: why.into() }
}

/// The ball-solve budget ([`BALL_SOLVES`]) ran out.
fn out_of_budget(why: &str) -> KernelError {
    KernelError::BudgetExhausted { stage: "fillet", why: why.into() }
}

/// The faces on either side of edge `e` (distinct), with their coedge directions.
fn sides(b: &Body, e: usize) -> Result<[(usize, bool); 2]> {
    super::blend::edge_sides(b, e)
}

/// What a rail leaving its face across a free edge (a sheet's boundary) runs into.
const OPEN: usize = usize::MAX;

/// The face across edge `e` from face `fi`: `None` for a free edge.
fn beyond(b: &Body, e: usize, fi: usize) -> Option<usize> {
    let [(a, _), (o, _)] = sides(b, e).ok()?;
    Some(if a == fi { o } else { a })
}

/// The coedge of edge `e` in face `fi` (the first, for a seam).
fn coedge_of(b: &Body, fi: usize, e: usize) -> Option<&Coedge> {
    b.faces[fi].loops.iter().flatten().find(|c| c.edge == e)
}

/// Whether edge `e` is a seam of face `fi` (used twice in its loops).
fn is_seam(b: &Body, fi: usize, e: usize) -> bool {
    b.faces[fi].loops.iter().flatten().filter(|c| c.edge == e).count() == 2
}

/// One picked edge's fillet.
struct EdgeFillet {
    e: usize,
    /// Its faces: the one whose loop runs along the edge first.
    faces: [usize; 2],
    /// The ball centre's offset from each face along its outward normal (at the largest
    /// radius).
    k: [f64; 2],
    /// A radius varying along the edge: (edge parameter, radius), increasing, linear between.
    law: Option<Vec<(f64, f64)>>,
    /// A chamfer: straight sections between the ball's contacts.
    flat: bool,
    /// The ball's offset from the second face over its offset from the first (1 but for a
    /// chamfer of two distances: its contacts then lie unequally far from the edge).
    ratio: f64,
    spine: Spine,
    exact: Option<Exact>,
    /// A ball in the middle of the strip (its reference) and its spine parameter.
    mid: Ball,
    tm: f64,
    closed: bool,
    /// Whether the strip's ends (at its lower and higher spine parameter) are the edge's ends;
    /// otherwise the ball rolls on from one face onto a smoothly joined one there and the next
    /// strip of the edge carries on.
    at_vertex: [bool; 2],
    /// The fillet's work budget (shared by all its strips).
    budget: Budget,
    /// How far a contact may lie from the spine (farther is a far branch of a curved face):
    /// a few radii, more where the faces meet at a sharp angle and the ball sits deep in it.
    reach: f64,
}

impl EdgeFillet {
    fn r(&self) -> f64 {
        self.k[0].abs()
    }

    /// The spine parameter t taken into the edge's turn round a closed edge (whose law is
    /// periodic), as it is otherwise.
    fn law_param(&self, t: f64) -> f64 {
        let (lo, hi) = (self.spine.t[0].min(self.spine.t[1]), self.spine.t[0].max(self.spine.t[1]));
        if self.closed && hi > lo { lo + wrap(t - lo, 0.0, hi - lo) } else { t }
    }

    /// The radius at spine parameter t (the end values past the ends).
    fn radius_at(&self, t: f64) -> f64 {
        let Some(law) = &self.law else { return self.r() };
        let t = self.law_param(t);
        let (first, last) = (law[0], law[law.len() - 1]);
        if t <= first.0 {
            return first.1;
        }
        if t >= last.0 {
            return last.1;
        }
        for w in law.windows(2) {
            if t <= w[1].0 {
                let f = (t - w[0].0) / (w[1].0 - w[0].0).max(tol::TINY);
                return w[0].1 + (w[1].1 - w[0].1) * f;
            }
        }
        last.1
    }

    /// The radius's derivative along the spine at t (0 past the law's ends).
    fn radius_slope(&self, t: f64) -> f64 {
        let Some(law) = &self.law else { return 0.0 };
        let t = self.law_param(t);
        law.windows(2).find(|w| t >= w[0].0 && t <= w[1].0).map_or(0.0, |w| (w[1].1 - w[0].1) / (w[1].0 - w[0].0).max(tol::TINY))
    }

    /// How ball `ball` (at spine parameter t) moves along the spine: its contacts' velocities
    /// ([`geom::ball_tangent`]); `None` where its faces are tangent there.
    fn velocity(&self, b: &Body, t: f64, ball: &Ball) -> Option<[V3; 2]> {
        let f = [&b.faces[self.faces[0]], &b.faces[self.faces[1]]];
        let (p, tn, dp, dtn) = self.spine.at_d(t);
        let s = self.k[0].signum() * self.radius_slope(t);
        let v = geom::ball_tangent(f, self.k_at(t), [s, s * self.ratio], ball, &p, &tn, &dp, &dtn)?;
        Some([v[0].0, v[1].0])
    }

    /// The offsets at spine parameter t.
    fn k_at(&self, t: f64) -> [f64; 2] {
        let r = self.radius_at(t) * self.k[0].signum();
        [r, r * self.ratio]
    }

    /// The ball at spine parameter t from contacts near `guess` (default: from the edge).
    fn ball(&self, b: &Body, t: f64, guess: Option<[P2; 2]>) -> Option<Ball> {
        if !self.budget.spend() {
            return None;
        }
        let f = [&b.faces[self.faces[0]], &b.faces[self.faces[1]]];
        let (p, d) = self.spine.at(t);
        let g = match guess {
            Some(g) => g,
            None => initial_guess(b, self, t)?,
        };
        let Some(mut ball) = geom::ball_in_plane(f, self.k_at(t), &p, &d, g) else {
            debug(|| format!("edge {}: no ball at t {t} (radius {})", self.e, self.radius_at(t)));
            return None;
        };
        // Not a far branch: the contacts lie near the spine (a section plane through a circle's
        // axis holds the ball on the other side too).
        ball.flat = self.flat;
        ball.q.iter().all(|q| (q.p - p).norm() <= self.reach).then_some(ball)
    }

    /// The ball at t marched from `start` (at t0) in steps no longer than the radius, nor
    /// turning the spine more than a quarter radian (round a spine tighter than the ball, a
    /// longer step lands the ball on the spine's far side).
    fn ball_from(&self, b: &Body, start: &Ball, t0: f64, t: f64) -> Option<Ball> {
        let len = self.spine_len(t0, t);
        // Where the spine turns tighter than the ball (its curvature radius under the radius),
        // also no more than a quarter radian of turn a step: a big ball round a small circle
        // would jump to the spine's far side.
        let turn = self.spine_turn(t0, t);
        let tight = turn > 0.0 && len < self.r() * turn;
        let by_turn = if tight { (turn / 0.25).ceil() } else { 0.0 };
        let n = ((len / self.r()).ceil().max(by_turn) as usize).clamp(1, 400);
        let mut cur = *start;
        for i in 1..=n {
            let ti = t0 + (t - t0) * i as f64 / n as f64;
            cur = self.ball(b, ti, Some([cur.q[0].uv, cur.q[1].uv]))?;
        }
        Some(cur)
    }

    /// The ball at the edge's end j, or as near it as the ball gets (its contact may leave a
    /// face's surface before the end, where the rail leaves the face anyway).
    fn end_ball(&self, b: &Body, j: usize) -> Result<Ball> {
        let ed = &b.edges[self.e];
        let (tm, te) = (self.tm, ed.t[j]);
        if let Some(x) = self.ball_from(b, &self.mid, tm, te) {
            return Ok(x);
        }
        let n = 64;
        let mut last = self.mid;
        for i in 1..=n {
            let t = tm + (te - tm) * i as f64 / n as f64;
            match self.ball(b, t, Some([last.q[0].uv, last.q[1].uv])) {
                Some(x) => last = x,
                None if i > 1 => return Ok(last),
                None => break,
            }
        }
        Err(failed("end", "the fillet's ball doesn't reach the edge's end"))
    }

    /// How far the spine's tangent turns between t0 and t1 (radians, summed over samples).
    fn spine_turn(&self, t0: f64, t1: f64) -> f64 {
        // In closed form for a line (none) and a circle (its angle, up to a half turn: past
        // that the samples below see a closed spine's wrap as the short way round).
        match &self.spine.curve {
            Curve::Line { .. } => return 0.0,
            Curve::Circle { .. } if (t1 - t0).abs() <= PI => return (t1 - t0).abs(),
            _ => {}
        }
        let n = 8;
        let mut turn = 0.0;
        let mut prev = self.spine.at(t0).1;
        for i in 1..=n {
            let d = self.spine.at(t0 + (t1 - t0) * i as f64 / n as f64).1;
            turn += prev.angle(&d);
            prev = d;
        }
        turn
    }

    fn spine_len(&self, t0: f64, t1: f64) -> f64 {
        // In closed form for a line and a circle (up to a half turn, as `spine_turn`).
        match &self.spine.curve {
            Curve::Line { d, .. } => return d.norm() * (t1 - t0).abs(),
            Curve::Circle { r, .. } if (t1 - t0).abs() <= PI => return r * (t1 - t0).abs(),
            _ => {}
        }
        let n = 8;
        let mut l = 0.0;
        let mut prev = self.spine.at(t0).0;
        for i in 1..=n {
            let p = self.spine.at(t0 + (t1 - t0) * i as f64 / n as f64).0;
            l += (p - prev).norm();
            prev = p;
        }
        l
    }
}

/// [`geom::rail_crossings`] for strip `ef` (its balls flat for a chamfer).
fn crossings(ef: &EdgeFillet, f: [&Face; 2], k: [f64; 2], side: usize, bd: &Boundary, near: &P3, reach: f64) -> Vec<(f64, Ball)> {
    let mut out = geom::rail_crossings(f, k, side, bd, near, reach);
    // A varying radius: each crossing again with the radius where it is. The crossing with the
    // radius at spine parameter t lies at F(t); secant steps on F(t) − t find where they agree.
    if ef.law.is_some() {
        for x in out.iter_mut() {
            let at = |t: f64, near: &Ball| -> Option<(f64, f64, Ball)> {
                let y = geom::rail_crossings(f, ef.k_at(t), side, bd, &near.c, 2.0 * ef.r()).into_iter().min_by(|p, q| (p.1.c - near.c).norm().total_cmp(&(q.1.c - near.c).norm()))?;
                Some((ef.spine.param_of(&y.1.c, t) - t, y.0, y.1))
            };
            let t0 = ef.spine.param_of(&x.1.c, ef.tm);
            let Some((h0, s0, b0)) = at(t0, &x.1) else { continue };
            let (mut a, mut ha) = (t0, h0);
            let mut best = (h0.abs(), s0, b0);
            let mut bcur = b0;
            let mut tb = t0 + h0;
            for _ in 0..40 {
                let Some((hb, sb, bb)) = at(tb, &bcur) else { break };
                if hb.abs() < best.0 {
                    best = (hb.abs(), sb, bb);
                }
                if hb.abs() <= tol::REL_FINE * (1.0 + tb.abs()) || (hb - ha).abs() < tol::TINY {
                    break;
                }
                let tn = tb - hb * (tb - a) / (hb - ha);
                (a, ha, bcur) = (tb, hb, bb);
                tb = tn;
            }
            *x = (best.1, best.2);
        }
    }
    for (_, b) in out.iter_mut() {
        b.flat = ef.flat;
    }
    out
}

/// Contacts to start Newton from at spine parameter t: the feet on the faces of the point the
/// ball's centre would have between the faces' tangent planes.
fn initial_guess(b: &Body, ef: &EdgeFillet, t: f64) -> Option<[P2; 2]> {
    let (p, uv, theta, bis) = wedge(b, ef, t)?;
    let c0 = p + bis * (ef.r() / (theta / 2.0).sin());
    Some([geom::foot(&b.faces[ef.faces[0]], &c0, Some(uv[0])), geom::foot(&b.faces[ef.faces[1]], &c0, Some(uv[1]))])
}

/// The wedge between the faces at spine parameter t (clamped to the edge): the edge's point,
/// its parameters on the faces, the angle between the faces (inside the material) and its
/// bisector.
fn wedge(b: &Body, ef: &EdgeFillet, t: f64) -> Option<(P3, [P2; 2], f64, V3)> {
    let ed = &b.edges[ef.e];
    let tc = t.clamp(ed.t[0].min(ed.t[1]), ed.t[0].max(ed.t[1]));
    let (p, d) = ef.spine.at(tc);
    let mut uv = [P2::origin(); 2];
    let mut w = [V3::zeros(); 2];
    for k in 0..2 {
        let f = &b.faces[ef.faces[k]];
        let c = coedge_of(b, ef.faces[k], ef.e)?;
        let q = c.pc.point(tc);
        uv[k] = P2::new(q.x, q.y);
        let n = geom::contact(f, uv[k]).n;
        // Into the face: material on the left of the coedge's direction.
        let along = if c.fwd { d } else { -d };
        w[k] = n.cross(&along);
    }
    let theta = w[0].dot(&w[1]).clamp(-1.0, 1.0).acos();
    if theta < tol::ANGULAR_POINT {
        return None;
    }
    let bis = (w[0] + w[1]).normalize();
    Some((p, uv, theta, bis))
}

/// Whether edge e is convex (true), concave (false) or neither (its faces tangent) at its
/// middle.
fn edge_convexity(b: &Body, e: usize) -> Option<bool> {
    let ed = &b.edges[e];
    let [(f1, c1), (f2, _)] = sides(b, e).ok()?;
    let faces = if c1 { [f1, f2] } else { [f2, f1] };
    let t = ed.mid_t();
    let d = ed.curve.tangent(t).normalize();
    let n = |fi: usize| -> Option<V3> {
        let c = coedge_of(b, fi, e)?;
        let q = c.pc.point(t);
        Some(geom::contact(&b.faces[fi], P2::new(q.x, q.y)).n)
    };
    let (na, nb) = (n(faces[0])?, n(faces[1])?);
    let s = (-nb.cross(&d)).dot(&na);
    (s.abs() > tol::ANGULAR).then_some(s < 0.0)
}

/// Sets up the fillet of radius r along edge e.
fn edge_fillet(b: &Body, e: usize, law: &[(f64, f64)], flat: bool, ratio: f64, budget: &Budget) -> Result<EdgeFillet> {
    let ed = &b.edges[e];
    if ed.degenerate {
        return Err(KernelError::InvalidParameter("A degenerate edge can't be filleted".into()));
    }
    let [(f1, c1), (f2, _)] = sides(b, e)?;
    let faces = if c1 { [f1, f2] } else { [f2, f1] };
    let spine = Spine { curve: ed.curve.clone(), t: ed.t, closed: ed.closed() };
    // Convex or concave: the second face turns away from the first's normal (convex).
    let convexity = |t: f64| -> Option<bool> {
        let d = ed.curve.tangent(t).normalize();
        let n = |fi: usize| -> Option<V3> {
            let c = coedge_of(b, fi, e)?;
            let q = c.pc.point(t);
            Some(geom::contact(&b.faces[fi], P2::new(q.x, q.y)).n)
        };
        let (na, nb) = (n(faces[0])?, n(faces[1])?);
        let wb = -nb.cross(&d);
        let s = wb.dot(&na);
        (s.abs() > tol::ANGULAR).then_some(s < 0.0)
    };
    let mut convex = None;
    for i in 1..8 {
        let t = ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0;
        match (convex, convexity(t)) {
            (_, None) => return Err(unsupported("a fillet along an edge whose faces are tangent")),
            (None, c) => convex = c,
            (Some(a), Some(c)) if a != c => return Err(unsupported("a fillet along an edge that is convex in part")),
            _ => {}
        }
    }
    let convex = convex.ok_or_else(|| unsupported("a fillet along an edge whose faces are tangent"))?;
    // Positive, but for 0 at an end of the edge (the fillet ends there at a point).
    if law.iter().any(|x| !(x.1 > 0.0) && !(x.1 == 0.0 && (x.0 == 0.0 || x.0 == 1.0))) || !law.iter().any(|x| x.1 > 0.0) {
        return Err(KernelError::InvalidParameter("A fillet radius must be positive".into()));
    }
    // The law along the edge's parameter, in increasing order.
    let mut pts: Vec<(f64, f64)> = law.iter().map(|&(f, rr)| (ed.t[0] + (ed.t[1] - ed.t[0]) * f, rr)).collect();
    pts.sort_by(|a, c| a.0.total_cmp(&c.0));
    if ed.closed() {
        // Round a closed edge the law is periodic: it runs from its first point's radius at the
        // edge's start back to that radius at its end (as OCCT's `ChFiDS_FilSpine::Reset` closes
        // a periodic spine's law), whatever radius the last point gives.
        let (lo, hi) = (ed.t[0].min(ed.t[1]), ed.t[0].max(ed.t[1]));
        let at = |t: f64, u: f64| (t - u).abs() <= tol::REL_FINE * (1.0 + (hi - lo));
        if !at(pts[0].0, lo) {
            pts.insert(0, (lo, pts[0].1));
        }
        let r0 = pts[0].1;
        match pts.last_mut() {
            Some(l) if at(l.0, hi) => l.1 = r0,
            _ => pts.push((hi, r0)),
        }
    }
    let r = pts.iter().map(|x| x.1).fold(0.0, f64::max);
    let varies = pts.iter().any(|x| (x.1 - r).abs() > tol::REL_FINE * r);
    let k = if convex { [-r, -r * ratio] } else { [r, r * ratio] };
    // The larger offset, for how far the contacts may lie.
    let rr = r * ratio.max(1.0);
    // A canal surface's envelope is real only where the radius changes slower than the spine
    // runs (Dahl, research/PAPER_SCAN.md item 11: |r′| < |g′|); past that the balls along the
    // edge nest inside one another and no ball touches both faces in its section.
    if varies {
        for w in pts.windows(2) {
            let dt = w[1].0 - w[0].0;
            if dt.abs() <= tol::TINY {
                continue;
            }
            let slope = (w[1].1 - w[0].1) / dt;
            // (Inside the law's pieces: at their ends, where the slope changes, it has none.)
            let slow = (1..4).all(|i| {
                let t = w[0].0 + dt * i as f64 / 4.0;
                slope.abs() < ed.curve.tangent(t).norm()
            });
            if !slow {
                return Err(KernelError::InvalidParameter("The fillet's radius changes faster along the edge than the edge runs".into()));
            }
        }
    }
    let law = varies.then_some(pts);
    let exact = if varies { None } else { geom::exact_kind([&b.faces[faces[0]], &b.faces[faces[1]]], &spine) };
    let tm = ed.mid_t();
    let mut ef = EdgeFillet { e, faces, k, law, flat, ratio, spine, exact, mid: dummy_ball(), tm, closed: ed.closed(), at_vertex: [true, true], budget: budget.clone(), reach: 4.0 * rr + tol::ROUNDOFF };
    // The contacts lie r / tan(θ/2) from the edge in a wedge of angle θ.
    let theta = (0..=8).filter_map(|i| wedge(b, &ef, ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0).map(|w| w.2)).fold(std::f64::consts::PI, f64::min);
    ef.reach = ef.reach.max(2.0 * rr / (theta / 2.0).tan().max(1e-3) + rr);
    ef.mid = ef.ball(b, tm, None).ok_or_else(|| too_large("the fillet is too large for the faces at the edge"))?;
    // The centre on the side the faces' normals say (inside a convex edge, outside a concave
    // one), and the contacts on the faces' sides of the edge (not a far branch of a curved
    // surface).
    let p = ed.point(tm);
    let nsum: V3 = (0..2)
        .filter_map(|k| {
            let c = coedge_of(b, faces[k], e)?;
            let q = c.pc.point(tm);
            Some(geom::contact(&b.faces[faces[k]], P2::new(q.x, q.y)).n)
        })
        .sum();
    if (ef.mid.c - p).dot(&nsum) * ef.k[0] < 0.0 {
        debug(|| format!("edge {e}: the middle ball {:?} is on the wrong side", ef.mid.c));
        return Err(bad_geometry("the fillet's ball is on the wrong side of the edge"));
    }
    for q in &ef.mid.q {
        if (q.p - p).norm() > ef.reach {
            return Err(too_large("the fillet is too large for the faces at the edge"));
        }
    }
    Ok(ef)
}

fn dummy_ball() -> Ball {
    let c = Contact { uv: P2::origin(), p: P3::origin(), n: V3::z() };
    Ball { c: P3::origin(), q: [c, c], flat: false }
}

/// Where a rail leaves its face at a free end.
#[derive(Debug, Clone, Copy)]
struct Exit {
    ball: Ball,
    /// The ball's spine parameter.
    t: f64,
    /// The face across the boundary edge the rail crosses.
    into: usize,
    /// A rail that doesn't leave its face at the vertex (its face runs on past it): it stops
    /// where it meets the end face's surface, on the extension of the edge `y` between its face
    /// and the end face (SMLib's mixed-convexity end): (y, the point's parameter on y).
    ext: Option<(usize, f64)>,
}

/// How a strip ends.
#[derive(Debug, Clone)]
enum End {
    /// Each rail leaves its face; the strip is cut by the face they run into.
    Free { exits: [Exit; 2] },
    /// The radius law reaches 0 at the edge's vertex, or the faces run on into each other
    /// smoothly there (a pinch, [`pinch`]): the strip ends there at a point.
    Point { t: f64 },
    /// Each rail leaves its face across a free edge of a sheet: the strip ends there, open,
    /// along the curve across it from one rail's end to the other's (`pts`).
    Open { exits: [Exit; 2], pts: Vec<P3> },
    /// Each rail leaves its face into a different face: the strip is cut by those and the faces
    /// between them round the vertex, the cut crossing the edges between them at `links`.
    Chain { exits: [Exit; 2], links: Vec<Link> },
    /// A section shared with a tangent neighbour or a corner: the ball, its spine parameter and
    /// the key of its arc edge.
    Section { ball: Ball, t: f64, key: SectionKey },
    /// A patched corner ([`PatchCorner`] `c`): each rail stops where it crosses the rail of
    /// the next strip on its face (at spine parameters `t`, balls `balls`), and the strip by
    /// the curve across it between those points (`pts`).
    Patch { c: usize, t: [f64; 2], balls: [Ball; 2], pts: Vec<P3> },
    /// A mitre with another strip ([`Mitre`] `m`): the rail on the shared face ends at the
    /// corner ball's contact (the section there at spine parameter `t_s`), the other as `other`
    /// says.
    Mitre { m: usize, t_s: f64, other: MitreEnd },
    /// Three chamfers meeting at a corner: each rail stops where it crosses the next strip's
    /// rail on its face (at spine parameters `t`, balls `balls`). All cutting or all filling,
    /// the strip ends there in the lines from those crossings to the point `p` where the three
    /// strips' planes meet (spine parameter `t_p`), the corner the three planes' (as the solid
    /// cut by every chamfer's plane has it); else (`p` none) in the line between them, and a
    /// triangle through the three crossings closes the corner ([`Cap`], Onshape's corner).
    Star { t: [f64; 2], balls: [Ball; 2], p: Option<P3>, t_p: f64 },
    /// The edge runs on smoothly into one left as it is: the strip stops square at the vertex,
    /// cut by the plane there square to the edge (normal `n`), each rail where it meets that
    /// plane (spine parameters `t`, balls `balls`), and a face in the plane closes the end (the
    /// material the next edge keeps).
    Square { t: [f64; 2], balls: [Ball; 2], n: V3 },
    /// A set-back corner ([`Hole`] `c`): the strip stops at its section at spine parameter `t`
    /// (ball `ball`), which is part of the hole's rim.
    Hole { c: usize, t: f64, ball: Ball },
    /// The strip running on at a mixed corner ([`Knee`] `c`): its rail on the common face stops
    /// where it crosses the other strip's, its other rail where it crosses the edge left as it
    /// is (at spine parameters `t`, balls `balls`), and the strip by the curve across it
    /// between those points (`pts`).
    Knee { c: usize, t: [f64; 2], balls: [Ball; 2], pts: Vec<P3> },
}

/// How a strip's other rail ends at a mitre.
#[derive(Debug, Clone)]
enum MitreEnd {
    /// At the mitre's far end (the ball there at spine parameter t).
    AtEnd { t: f64 },
    /// Across the edge left as it is, the strip cut by the face beyond from the mitre's far end
    /// to there (the points of that curve, with their spine parameters).
    Exit { exit: Box<Exit>, k: usize, pts: Vec<(f64, P3)> },
}

/// Where two picked edges meet at an angle at a vertex of three edges, the third left as it is:
/// the strips run into each other along a mitre (SMLib's `Nx2` bevel corner), from the contact
/// on their common face of the ball touching all three faces to where the mitre reaches one of
/// the other faces (on the edge left as it is when the corner is symmetric).
struct Mitre {
    v: usize,
    shared: usize,
    ps: P3,
    q: P3,
    q_on_edge: bool,
    /// The edge left as it is.
    e3: usize,
    /// The mitre's points from `ps` to `q`.
    pts: Vec<P3>,
}

/// A section arc: at a tangent junction (by vertex) or a corner's (by vertex and edge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SectionKey {
    Junction(usize),
    Corner(usize, usize),
    Mitre(usize),
    /// Where the ball rolls from one face onto the next (by its index among those).
    Roll(usize),
    /// A strip's end at a three-sided corner (vertex, edge).
    Tri(usize, usize),
    /// The line from a chamfer corner's point to the rails' crossing on a face (vertex, face).
    Star(usize, usize),
    /// A chamfer's end on a corner triangle (vertex, edge).
    Cap(usize, usize),
    /// A strip's end at a mixed corner (vertex, edge): the section of the strip stopping
    /// there, or the curve across the one running on.
    Knee(usize, usize),
}

impl SectionKey {
    /// The site whose contact vertices the strips ending at this section share: the vertex,
    /// or a number past every vertex for a roll.
    fn site(self) -> usize {
        match self {
            SectionKey::Junction(v) | SectionKey::Corner(v, _) | SectionKey::Mitre(v) | SectionKey::Star(v, _) | SectionKey::Cap(v, _) | SectionKey::Knee(v, _) => v,
            SectionKey::Roll(i) => usize::MAX / 2 + i,
            // Each strip end's own contacts (they meet only on the edge left as it is; a strip
            // with such corners at both ends has one at each).
            SectionKey::Tri(v, e) => usize::MAX / 4 + (e << 24) + v,
        }
    }
}

/// A three-edge corner: the ball touching the three faces.
struct Corner {
    v: usize,
    c: P3,
    /// The faces and the ball's contacts on them.
    contacts: Vec<(usize, Contact)>,
    /// The picked edges meeting there.
    edges: Vec<usize>,
}

/// A corner where every edge at a vertex is filleted but no ball touches all its faces (more
/// than three faces, or convex and concave fillets together): the strips stop where their rails
/// cross on the faces between them, and four-sided patches fill the hole, one at each crossing,
/// meeting at the middles of the strips' end curves and at the hole's centre (SMLib fills such
/// a corner with one n-sided patch, `SmFilletConcaveNxNCorner`).
struct PatchCorner {
    v: usize,
    /// The strips ending there, in turn round the vertex.
    strips: Vec<usize>,
}

/// The picked edges at each vertex: (index in the list, which end).
fn ends_at(b: &Body, efs: &[EdgeFillet]) -> HashMap<usize, Vec<(usize, usize)>> {
    let mut at: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    for (i, ef) in efs.iter().enumerate() {
        if ef.closed {
            continue;
        }
        for j in 0..2 {
            if ef.at_vertex[j] {
                at.entry(b.edges[ef.e].v[j]).or_default().push((i, j));
            }
        }
    }
    at
}

/// The edges (not degenerate) at vertex v.
fn vertex_edges(b: &Body, v: usize) -> Vec<usize> {
    (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate && b.edges[e].v.contains(&v)).collect()
}

/// The unit direction an edge leaves vertex v in.
fn leaving(b: &Body, e: usize, v: usize) -> V3 {
    let ed = &b.edges[e];
    if ed.v[0] == v { ed.curve.tangent(ed.t[0]).normalize() } else { -ed.curve.tangent(ed.t[1]).normalize() }
}

fn ed_len(b: &Body, e: usize) -> f64 {
    let ed = &b.edges[e];
    ed.curve.length(ed.t[0], ed.t[1])
}

/// Which edges of a rail's face [`rail_exit`] looks at.
#[derive(Clone, Copy, PartialEq)]
enum Which {
    /// Those at the end's vertex.
    AtVertex,
    /// The others.
    Elsewhere,
    /// This one.
    Edge(usize),
}

/// The square end j of `ef` ([`End::Square`]): where each rail meets the plane through the
/// edge's vertex square to the edge (the ball's contacts leave the section's plane where a face
/// curves along the edge), by secant steps along the spine.
fn square_end(b: &Body, ef: &EdgeFillet, j: usize) -> Result<End> {
    let te = b.edges[ef.e].t[j];
    let vp = b.verts[b.edges[ef.e].v[j]].p;
    let n = ef.spine.at(te).1.normalize();
    let lost = || failed("end", "the fillet's ball doesn't reach its square end");
    let ball = ef.ball_from(b, &ef.mid, ef.tm, te).ok_or_else(lost)?;
    let sc = 1.0 + vp.coords.abs().max() + ef.r();
    let h = 1e-3 * ef.r() / ef.spine.curve.d2(te).1.norm().max(tol::DEGENERATE);
    let mut t = [te; 2];
    let mut balls = [ball; 2];
    for k in 0..2 {
        let f = |bl: &Ball| (bl.q[k].p - vp).dot(&n);
        let (mut ta, mut ba) = (te, ball);
        let mut fa = f(&ba);
        if fa.abs() > 1e-11 * sc {
            let mut tb = te + h;
            let mut bb = ef.ball(b, tb, Some([ba.q[0].uv, ba.q[1].uv])).ok_or_else(lost)?;
            let mut fb = f(&bb);
            for _ in 0..60 {
                if fb.abs() < 1e-11 * sc {
                    break;
                }
                if (fb - fa).abs() < tol::TINY {
                    return Err(lost());
                }
                let tn = tb - fb * (tb - ta) / (fb - fa);
                let bn = ef.ball_from(b, &bb, tb, tn).ok_or_else(lost)?;
                (ta, fa) = (tb, fb);
                (tb, bb, fb) = (tn, bn, f(&bn));
            }
            if fb.abs() > tol::REL * sc {
                return Err(lost());
            }
            (ta, ba) = (tb, bb);
        }
        t[k] = ta;
        balls[k] = ba;
    }
    Ok(End::Square { t, balls, n })
}

/// Where rail k of `ef` leaves its face near the end j (`end_ball` is the ball at the edge's
/// end): the crossing nearest that ball of the edges `which` picks.
fn rail_exit(b: &Body, ef: &EdgeFillet, j: usize, k: usize, end_ball: &Ball, which: Which) -> Result<Option<Exit>> {
    let ed = &b.edges[ef.e];
    let (v, te) = (ed.v[j], ed.t[j]);
    let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
    let reach = ef.reach + ed_len(b, ef.e);
    let fi = ef.faces[k];
    let mut best: Option<(f64, Exit)> = None;
    for c in b.faces[fi].loops.iter().flatten() {
        let x = &b.edges[c.edge];
        let wanted = match which {
            Which::AtVertex => x.v.contains(&v),
            Which::Elsewhere => !x.v.contains(&v),
            Which::Edge(e) => c.edge == e,
        };
        if c.edge == ef.e || x.degenerate || !wanted {
            continue;
        }
        let bd = Boundary { curve: &x.curve, pc: &c.pc, t: x.t };
        for (_, ball) in crossings(ef, f, ef.k_at(te), k, &bd, &end_ball.c, reach) {
            let d = (ball.c - end_ball.c).norm();
            if best.as_ref().is_some_and(|(bd, _)| *bd <= d) {
                continue;
            }
            let into = beyond(b, c.edge, fi).unwrap_or(OPEN);
            let mut t = ef.spine.param_of(&ball.c, te);
            let mut ball = ball;
            // A varying radius: again with the radius where the crossing is, till it settles.
            if ef.law.is_some() {
                for _ in 0..6 {
                    let Some((_, y)) = crossings(ef, f, ef.k_at(t), k, &bd, &ball.c, reach).into_iter().min_by(|p, q| (p.1.c - ball.c).norm().total_cmp(&(q.1.c - ball.c).norm())) else { break };
                    ball = y;
                    t = ef.spine.param_of(&ball.c, t);
                }
                ball = ef.ball(b, t, Some([ball.q[0].uv, ball.q[1].uv])).unwrap_or(ball);
            }
            best = Some((d, Exit { ball, t, into, ext: None }));
        }
    }
    Ok(best.map(|x| x.1))
}

/// Exit `x` of rail k (found away from the vertex) moved back to where the rail first meets
/// its face's boundary, when it runs along an edge of the face up to there (the face steps away
/// at a vertex the rail passes through, and the rail then lies on the face's edge): the rail
/// really leaves the face at that vertex.
fn first_contact(b: &Body, ef: &EdgeFillet, k: usize, x: Exit) -> Exit {
    let fi = ef.faces[k];
    let tol = tol::REL_POINT * (1.0 + ef.r());
    let span = x.t - ef.tm;
    let on = |t: f64, from: &Ball, t0: f64| -> Option<(bool, Ball)> {
        let bl = ef.ball_from(b, from, t0, t)?;
        Some((on_boundary(b, fi, &bl.q[k].p, tol), bl))
    };
    // Along the boundary just before the exit?
    let Some((true, _)) = on(x.t - 0.01 * span, &x.ball, x.t) else { return x };
    // Bisect between the middle (inside) and the exit's neighbourhood (on the boundary).
    let (mut lo, mut lo_ball) = (ef.tm, ef.mid);
    let (mut hi, mut hi_ball) = (x.t - 0.01 * span, x.ball);
    if !matches!(on(lo, &lo_ball, lo), Some((false, _))) {
        return x;
    }
    for _ in 0..60 {
        let m = 0.5 * (lo + hi);
        match on(m, &lo_ball, lo) {
            Some((true, bl)) => (hi, hi_ball) = (m, bl),
            Some((false, bl)) => (lo, lo_ball) = (m, bl),
            None => return x,
        }
    }
    debug(|| format!("edge {}: rail {k} runs along its face's edge from {:?}", ef.e, hi_ball.q[k].p));
    // The bisection stops within the tolerance of the boundary: where that's at a vertex of the
    // face (the rail coming onto the edge at its end), the ball whose contact is the vertex.
    let near = b.faces[fi].loops.iter().flatten().map(|c| b.verts[b.co_start(c)].p).filter(|p| (p - hi_ball.q[k].p).norm() < 10.0 * tol).min_by(|p, q| (p - hi_ball.q[k].p).norm().total_cmp(&(q - hi_ball.q[k].p).norm()));
    if let Some(p) = near {
        let tv = ef.spine.param_of(&p, hi);
        if let Some(bl) = ef.ball_from(b, &hi_ball, hi, tv).filter(|bl| (bl.q[k].p - p).norm() < 1e-3 * tol) {
            return Exit { ball: bl, t: tv, ..x };
        }
    }
    Exit { ball: hi_ball, t: hi, ..x }
}

/// The free end j of fillet `ef`: where each rail leaves its face (across an edge at the
/// vertex there, else the nearest other edge of the face).
fn free_end(b: &Body, ef: &EdgeFillet, j: usize) -> Result<[Exit; 2]> {
    let ed = &b.edges[ef.e];
    let v = ed.v[j];
    let te = ed.t[j];
    let end_ball = ef.end_ball(b, j)?;
    let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
    let reach = ef.reach + ed_len(b, ef.e);
    let crossing = |k: usize, incident_only: bool| rail_exit(b, ef, j, k, &end_ball, if incident_only { Which::AtVertex } else { Which::Elsewhere });
    // Where rail k stops on the surface of face `into` (the other rail's end face), on the
    // extension past the vertex of the edge between its face and that one (`far_end`: past the
    // edge's other end, an edge shorter than the strip is high).
    let on_extension = |k: usize, into: usize, far_end: bool| -> Option<Exit> {
        let fk = ef.faces[k];
        let mut found = None;
        for c in b.faces[fk].loops.iter().flatten() {
            let y = &b.edges[c.edge];
            if c.edge == ef.e || y.degenerate || !y.v.contains(&v) {
                continue;
            }
            if beyond(b, c.edge, fk) != Some(into) {
                continue;
            }
            // The edge's curve run on past the vertex.
            let speed = y.curve.d2(y.t[if y.v[0] == v { 0 } else { 1 }]).1.norm().max(tol::TINY);
            let more = reach / speed;
            let bd = Boundary { curve: &y.curve, pc: &c.pc, t: [y.t[0] - more, y.t[1] + more] };
            let tv = y.t[if y.v[0] == v { 0 } else { 1 }];
                let outside = |t: f64| match (far_end, y.v[0] == v) {
                (false, true) => t < tv,
                (false, false) => t > tv,
                (true, true) => t > y.t[1],
                (true, false) => t < y.t[0],
            };
            for (ty, ball) in crossings(ef, f, ef.k_at(te), k, &bd, &end_ball.c, reach) {
                // Past the vertex, and inside the face (a curved edge's extension comes
                // round to the face's boundary again), unless the rail runs along that
                // boundary there (the strip takes the face's width).
                let tol = tol::REL_POINT * (1.0 + ef.r());
                if !outside(ty) {
                    continue;
                }
                if on_boundary(b, fk, &ball.q[k].p, tol) {
                    let t = ef.spine.param_of(&ball.c, te);
                    let back = ef.ball_from(b, &ball, t, t + 0.01 * (ef.tm - t));
                    if !back.is_some_and(|x| on_boundary(b, fk, &x.q[k].p, tol)) {
                        continue;
                    }
                }
                let d = (ball.c - end_ball.c).norm();
                if found.as_ref().is_some_and(|(bd, _): &(f64, Exit)| bd.min(d) == *bd) {
                    continue;
                }
                let t = ef.spine.param_of(&ball.c, te);
                found = Some((d, Exit { ball, t, into, ext: Some((c.edge, ty)) }));
            }
        }
        found.map(|f| f.1)
    };
    let near = [crossing(0, true)?, crossing(1, true)?];
    let exits = match near {
        [Some(a), Some(c)] => {
            let mut exits = [a, c];
            // A rail leaving far off (its face's edge from the vertex comes back round to cross
            // it a long way on, the strip's cut running back over the strip on the way) where it
            // meets the other's end face's surface past the vertex near the end: it stops there.
            for k in 0..2 {
                let far = |x: &Exit| (x.ball.c - end_ball.c).norm() > 4.0 * ef.r();
                if far(&exits[k])
                    && exits[1 - k].into == exits[k].into
                    && let Some(y) = on_extension(k, exits[1 - k].into, false).filter(|y| !far(y))
                {
                    debug(|| format!("edge {} end {j}: rail {k} stops on the extension of edge {:?} instead of leaving at {:?}", ef.e, y.ext.map(|x| x.0), exits[k].ball.q[k].p));
                    exits[k] = y;
                }
            }
            exits
        }
        [Some(x), None] | [None, Some(x)] => {
            // One rail leaves its face at the vertex into the end face; the other's face runs on:
            // that rail stops on the end face's surface, on the extension of their common edge.
            let k = if near[0].is_some() { 1 } else { 0 };
            // Else the rail leaves its face over another edge (its face ends short of the end
            // face there: a step in it); where that's far off, it meets the end face's surface
            // past the far end of their common edge (that edge lower than the strip: the end
            // face runs on above it).
            let y = match on_extension(k, x.into, false) {
                Some(f) => f,
                None => {
                    let away = crossing(k, false)?.map(|x| first_contact(b, ef, k, x));
                    let dist = |x: &Exit| (x.ball.c - end_ball.c).norm();
                    match on_extension(k, x.into, true).filter(|y| away.as_ref().is_none_or(|a| dist(a) > 4.0 * ef.r() && dist(y) < dist(a))) {
                        Some(y) => y,
                        None => away.ok_or_else(|| unsupported("a rail that doesn't leave its face"))?,
                    }
                }
            };
            if k == 0 { [y, x] } else { [x, y] }
        }
        [None, None] => {
            debug(|| format!("edge {} end {j}: no rail leaves at the vertex; end ball {:?}, contacts {:?} {:?}", ef.e, end_ball.c, end_ball.q[0].p, end_ball.q[1].p));
            let a = crossing(0, false)?.ok_or_else(|| unsupported("a rail that doesn't leave its face"))?;
            let c = crossing(1, false)?.ok_or_else(|| unsupported("a rail that doesn't leave its face"))?;
            [a, c]
        }
    };
    for (k, x) in exits.iter().enumerate() {
        debug(|| format!("edge {} end {j} (vertex {v}, edges {:?}): rail {k} on face {} leaves at {:?} into face {}{}", ef.e, vertex_edges(b, v), ef.faces[k], x.ball.q[k].p, x.into, if x.ext.is_some() { " (on an edge's extension)" } else { "" }));
    }
    Ok(exits)
}

/// Where two picked edges meet at an angle at a vertex whose third edge turns the other way
/// (convex fillets round a concave edge's end, or the reverse): each strip stops at its section
/// through the point where its rail on its own face meets that third edge, a curve on the
/// common face bridges the two rails' ends (G1 with both), and a three-sided face fills the
/// corner (SMLib's mixed-convexity `SmFilletNx2Corner`: cross sections, a rail-rail
/// interpolation curve and a corner patch, SmFilletCorner.cpp:6487).
struct TriCorner {
    v: usize,
    /// The edges whose strips end there, and the face both strips touch.
    edges: [usize; 2],
    shared: usize,
    /// The points on the third edge the sections start from (one where the corner is
    /// symmetric; else the corner face four-sided, its fourth side the third edge between).
    side: [P3; 2],
    /// The bridge on the common face from the first strip's rail end to the second's, as
    /// points of the face's (u, v).
    bridge: Vec<P2>,
}

fn tri_corner(b: &Body, efs: &[EdgeFillet], a: (usize, usize), bb: (usize, usize), v: usize, e3: usize) -> Result<(TriCorner, End, End)> {
    let shared: Vec<usize> = efs[a.0].faces.iter().copied().filter(|f| efs[bb.0].faces.contains(f)).collect();
    if shared.len() != 1 || e3 == usize::MAX {
        return Err(unsupported("a three-sided fillet corner without one common face"));
    }
    let fs = shared[0];
    let mut ends = Vec::new();
    let mut sides_at: Vec<P3> = Vec::new();
    let mut mates: Vec<(P2, V3)> = Vec::new(); // (u, v) on the common face and the rail's direction on into the corner
    for &(i, j) in [a, bb].iter() {
        let s = &efs[i];
        let ks = if s.faces[0] == fs { 0 } else { 1 };
        let ko = 1 - ks;
        let eb = s.end_ball(b, j)?;
        let x = rail_exit(b, s, j, ko, &eb, Which::Edge(e3))?.ok_or_else(|| unsupported("a three-sided fillet corner whose rail misses the third edge"))?;
        sides_at.push(x.ball.q[ko].p);
        // The rail on the common face runs on towards the vertex: its direction there.
        let te = b.edges[s.e].t[j];
        let h = tol::DIFF_STEP * (1.0 + s.r()) * (te - s.tm).signum() / s.spine.curve.d2(x.t).1.norm().max(tol::DEGENERATE);
        let y = s.ball_from(b, &x.ball, x.t, x.t + h).ok_or_else(|| failed("corner", "the fillet's ball is lost at a three-sided corner"))?;
        mates.push((x.ball.q[ks].uv, (y.q[ks].p - x.ball.q[ks].p).normalize()));
        ends.push(End::Section { ball: x.ball, t: x.t, key: SectionKey::Tri(v, s.e) });
    }
    // The bridge: a cubic in the common face's (u, v), leaving the first rail's end along it
    // and arriving at the second's against it.
    let face = &b.faces[fs];
    let to_uv = |uv: P2, d: V3| -> V2 {
        let (_, su, sv) = face.surf.d1(uv.x, uv.y);
        let m = nalgebra::Matrix2::new(su.dot(&su), su.dot(&sv), su.dot(&sv), sv.dot(&sv));
        m.lu().solve(&V2::new(su.dot(&d), sv.dot(&d))).unwrap_or(V2::zeros())
    };
    let (p0, p1) = (face.surf.eval(mates[0].0.x, mates[0].0.y), face.surf.eval(mates[1].0.x, mates[1].0.y));
    let len = (p1 - p0).norm();
    let t0 = to_uv(mates[0].0, mates[0].1 * len);
    let t1 = to_uv(mates[1].0, -mates[1].1 * len);
    let (u0, u1) = (mates[0].0.coords, mates[1].0.coords);
    let n = 32;
    let bridge: Vec<P2> = (0..=n)
        .map(|i| {
            let s = i as f64 / n as f64;
            let (h00, h10, h01, h11) = (2.0 * s * s * s - 3.0 * s * s + 1.0, s * s * s - 2.0 * s * s + s, -2.0 * s * s * s + 3.0 * s * s, s * s * s - s * s);
            P2::from(u0 * h00 + t0 * h10 + u1 * h01 + t1 * h11)
        })
        .collect();
    let e1 = ends.pop().ok_or_else(|| failed("corner", "a three-sided corner without ends"))?;
    let e0 = ends.pop().ok_or_else(|| failed("corner", "a three-sided corner without ends"))?;
    debug(|| format!("vertex {v}: three-sided corner of edges {} and {} from {:?}", efs[a.0].e, efs[bb.0].e, sides_at[0]));
    Ok((TriCorner { v, edges: [efs[a.0].e, efs[bb.0].e], shared: fs, side: [sides_at[0], sides_at[1]], bridge }, e0, e1))
}

/// Where a convex and a concave fillet meet at an angle at a vertex of three edges, the third
/// left as it is (OCCT blend simple P2, P7): their rails on the common face cross; the strip
/// turning the way the third edge does stops at its section through that crossing, its other
/// end inside its other face; the other strip runs on to where its rail on its other face
/// crosses the third edge, cut across from the crossing to there ([`End::Knee`]); and a
/// three-sided face fills the corner, bounded by those two curves and a cut across the first
/// strip's other face from its section's end to the third edge (OCCT's corner face there has
/// the same three sides).
struct Knee {
    v: usize,
    /// The strips' common face.
    shared: usize,
    /// The edges of the strip stopping at its section and of the one running on.
    edges: [usize; 2],
    /// The first strip's other face, which the cut from its section's end crosses.
    face: usize,
}

/// The mixed corner ([`Knee`]) where the strips of `efs[a.0]` and `efs[bb.0]` end (at their
/// ends `a.1` and `bb.1`) at vertex v, whose third edge `e3` is left as it is; and the strips'
/// ends there, in that order (the corner's index left unset).
fn knee(b: &Body, efs: &[EdgeFillet], a: (usize, usize), bb: (usize, usize), v: usize, e3: usize) -> Result<(Knee, End, End)> {
    let shared: Vec<usize> = efs[a.0].faces.iter().copied().filter(|f| efs[bb.0].faces.contains(f)).collect();
    if shared.len() != 1 || e3 == usize::MAX {
        return Err(unsupported("a convex and a concave fillet meeting without one common face"));
    }
    let fc = shared[0];
    // The strip turning the third edge's way stops at its section (a negative offset is a
    // convex fillet's).
    let convex3 = edge_convexity(b, e3).ok_or_else(|| unsupported("a convex and a concave fillet meeting at a smooth edge"))?;
    let (s, o) = if (efs[a.0].k[0] < 0.0) == convex3 { (a, bb) } else { (bb, a) };
    let (es, eo) = (&efs[s.0], &efs[o.0]);
    let ksc = if es.faces[0] == fc { 0 } else { 1 };
    let koc = if eo.faces[0] == fc { 0 } else { 1 };
    let (kso, koo) = (1 - ksc, 1 - koc);
    let fs = es.faces[kso];
    // The third edge lies between the strips' other faces.
    let [(f3a, _), (f3b, _)] = sides(b, e3)?;
    if ![f3a, f3b].contains(&fs) || ![f3a, f3b].contains(&eo.faces[koo]) {
        return Err(unsupported("a convex and a concave fillet meeting where the third edge isn't between their other faces"));
    }
    let (ets, eto) = (b.edges[es.e].t[s.1], b.edges[eo.e].t[o.1]);
    let (bs0, bo0) = (es.end_ball(b, s.1)?, eo.end_ball(b, o.1)?);
    let (ts, bs, to, bo) = rail_cross(b, es, ksc, bs0, ets, eo, koc, bo0, eto).ok_or_else(|| unsupported("a convex and a concave fillet whose rails don't cross"))?;
    // The crossing on both strips' sides of the vertex (short of it, or as good as at it).
    let before = |ef: &EdgeFillet, t: f64, te: f64| {
        let f = (t - ef.tm) / (te - ef.tm);
        f > 0.0 && f < 1.0 + tol::REL
    };
    if !before(es, ts, ets) || !before(eo, to, eto) {
        return Err(unsupported("a convex and a concave fillet whose rails cross past their vertex"));
    }
    // The section's other end inside its face, and the running strip's other rail across the
    // third edge.
    if !point_in_face(b, fs, &bs.q[kso].p) {
        return Err(unsupported("a convex and a concave fillet meeting where the section leaves its face"));
    }
    let y = rail_exit(b, eo, o.1, koo, &bo0, Which::Edge(e3))?.ok_or_else(|| unsupported("a convex and a concave fillet meeting where the rail misses the third edge"))?;
    let mut ends_o = [Exit { ball: bo, t: to, into: fs, ext: None }, y];
    if koc == 1 {
        ends_o.swap(0, 1);
    }
    let pts = across_points(b, eo, &ends_o[0], &ends_o[1])?;
    let end_o = End::Knee { c: usize::MAX, t: [ends_o[0].t, ends_o[1].t], balls: [ends_o[0].ball, ends_o[1].ball], pts };
    let end_s = End::Section { ball: bs, t: ts, key: SectionKey::Knee(v, es.e) };
    debug(|| format!("vertex {v}: mixed corner, edge {} stops at {:?}, edge {} runs on to {:?}", es.e, bs.q[ksc].p, eo.e, y.ball.q[koo].p));
    let kn = Knee { v, shared: fc, edges: [es.e, eo.e], face: fs };
    Ok(if s == a { (kn, end_s, end_o) } else { (kn, end_o, end_s) })
}

/// The mitre where the strips of `efs[a.0]` and `efs[b.0]` meet at their ends `a.1` and `b.1`
/// (vertex v, whose edges are `all`), and how each strip ends there (their `m` left unset).
fn mitre(b: &Body, efs: &[EdgeFillet], a: (usize, usize), bb: (usize, usize), v: usize, all: &[usize]) -> Result<(Mitre, End, End)> {
    let (s1, j1, s2, j2) = (&efs[a.0], a.1, &efs[bb.0], bb.1);
    let r = s1.r();
    let shared: Vec<usize> = s1.faces.iter().copied().filter(|f| s2.faces.contains(f)).collect();
    if shared.len() != 1 {
        return Err(unsupported("two fillets meeting at an angle without one common face"));
    }
    let fs = shared[0];
    let k1s = if s1.faces[0] == fs { 0 } else { 1 };
    let k2s = if s2.faces[0] == fs { 0 } else { 1 };
    let (k1o, k2o) = (1 - k1s, 1 - k2s);
    let (fa, fb) = (s1.faces[k1o], s2.faces[k2o]);
    let e3 = all.iter().copied().find(|&e| e != s1.e && e != s2.e).ok_or_else(|| unsupported("a mitre without a third edge"))?;
    let [(x, _), (y, _)] = sides(b, e3)?;
    if !((x == fa && y == fb) || (x == fb && y == fa)) {
        return Err(unsupported("a mitre whose third edge isn't between the strips' other faces"));
    }
    let (b1, b2) = (s1.end_ball(b, j1)?, s2.end_ball(b, j2)?);
    // Where the strips' rails on the common face cross (the ball touching all three faces when
    // the radii there agree): the mitre starts there.
    let (t1e, t2e) = (b.edges[s1.e].t[j1], b.edges[s2.e].t[j2]);
    let (t1s, ball1s, t2s, ball2s) = rail_cross(b, s1, k1s, b1, t1e, s2, k2s, b2, t2e).ok_or_else(|| failed("corner", "the fillets' rails don't cross at their mitre"))?;
    let q1p = ball1s.q[k1s].p;
    // The fraction of the way across a ball's section from its contact on side ks.
    let frac = |bl: &Ball, ks: usize, x: &P3| -> f64 { bl.frac(ks, x) };
    // How far a point is from strip 2 (outside its ball, signed) and its fraction there.
    let on2 = |x: &P3, st: &mut (f64, Ball)| -> Option<(f64, f64)> {
        let t = s2.spine.param_of(x, st.0);
        let bl = s2.ball_from(b, &st.1, st.0, t)?;
        *st = (t, bl);
        Some((bl.sdist(x), frac(&bl, k2s, x)))
    };
    // Strip 1's section at t1 meets strip 2 a fraction u1 across (from the common face), nearest
    // `prev`: (u1, the point, strip 2's fraction there, strip 1's ball).
    let lost = || failed("corner", "the fillets' mitre can't be traced");
    let root = |t1: f64, from: &(f64, Ball), prev: f64, st2: &mut (f64, Ball)| -> Result<Option<(f64, P3, f64, Ball)>> {
        let bl = s1.ball_from(b, &from.1, from.0, t1).ok_or_else(lost)?;
        let at = |u: f64| bl.arc_point(if k1s == 0 { u } else { 1.0 - u });
        // The bracket of the section's 64 that holds a crossing nearest `prev`: brackets in
        // order of their distance from it, samples evaluated as they are needed (the mitre
        // moves little from one section to the next: a bracket or two).
        let n = 64;
        let mut f: Vec<Option<f64>> = vec![None; n + 1];
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&i, &j| ((i as f64 + 0.5) / n as f64 - prev).abs().total_cmp(&((j as f64 + 0.5) / n as f64 - prev).abs()).then(i.cmp(&j)));
        let mut found = None;
        for i in order {
            for j in [i, i + 1] {
                if f[j].is_none() {
                    f[j] = Some(on2(&at(j as f64 / n as f64), st2).ok_or_else(lost)?.0);
                }
            }
            let (fa, fb) = (f[i].unwrap_or(0.0), f[i + 1].unwrap_or(0.0));
            if (fa < 0.0) != (fb < 0.0) || fa == 0.0 {
                found = Some((i, fa, fb));
                break;
            }
        }
        let Some((i, flo, fhi)) = found else { return Ok(None) };
        let (lo, hi) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
        // (Strip 2's ball back near the bracket.)
        on2(&at(lo), st2).ok_or_else(lost)?;
        // A sample right on strip 2 is the root; else the crossing in the bracket, with the
        // signs the scan found (evaluated again, from a ball marched from elsewhere, a root
        // right at a sample can come out the other side).
        let u = if flo == 0.0 {
            lo
        } else {
            let mut failed_on = false;
            let u = geom::bracket_root(
                |m| match on2(&at(m), st2) {
                    Some(x) => Some(x.0),
                    None => {
                        failed_on = true;
                        None
                    }
                },
                (lo, flo),
                (hi, fhi),
            );
            if failed_on {
                return Err(lost());
            }
            u
        };
        let x = at(u);
        let (_, u2) = on2(&x, st2).ok_or_else(lost)?;
        Ok(Some((u, x, u2, bl)))
    };
    let good = |x: &Option<(f64, P3, f64, Ball)>| x.as_ref().is_some_and(|x| x.0 <= 1.0 && x.2 <= 1.0 && x.0 > 0.0);
    // March from the corner ball's section towards the vertex and past it, until the mitre
    // reaches a strip's other rail.
    let n = 32;
    let dt = (t1e - t1s) / (n / 2) as f64;
    let mut cur = (t1s, ball1s);
    let mut st2 = (t2s, ball2s);
    let mut prev_u = 0.0;
    let mut last: Option<(f64, P3, f64, Ball, f64)> = None; // (u1, x, u2, ball1, t1)
    let mut end_t = None;
    debug(|| format!("vertex {v}: mitre from t {t1s} (strip 1 end {t1e}), strip 2 at {t2s}"));
    for i in 1..=4 * n {
        let t1 = t1s + dt * i as f64;
        let x = root(t1, &cur, prev_u, &mut st2)?;
        debug(|| format!("  t1 {t1}: {:?} st2 t {}", x.as_ref().map(|x| (x.0, x.2)), st2.0));
        if !good(&x) {
            end_t = Some(t1);
            break;
        }
        let (u, p, u2, bl) = x.ok_or_else(lost)?;
        prev_u = u;
        cur = (t1, bl);
        last = Some((u, p, u2, bl, t1));
    }
    let (Some(mut t_bad), Some(l)) = (end_t, last) else { return Err(failed("corner", "the fillets' mitre doesn't end")) };
    // Where the mitre leaves the strips: bisection between the last good step and the first bad.
    let mut good_end = l;
    let mut t_good = l.4;
    for _ in 0..60 {
        let tm = 0.5 * (t_good + t_bad);
        let from = (good_end.4, good_end.3);
        let mut st = st2;
        let x = root(tm, &from, good_end.0, &mut st)?;
        if good(&x) {
            let (u, p, u2, bl) = x.ok_or_else(lost)?;
            good_end = (u, p, u2, bl, tm);
            t_good = tm;
            st2 = st;
        } else {
            t_bad = tm;
        }
    }
    let (u1, end_p, u2, bl1, t1q) = good_end;
    debug(|| format!("vertex {v}: mitre ends at t1 {t1q}: u1 {u1}, u2 {u2}"));
    if u1.max(u2) < 1.0 - 1e-4 {
        return Err(unsupported("a mitre that turns back before it reaches a rail"));
    }
    let near_edge = |p: &P3| {
        let ed = &b.edges[e3];
        let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
        // An intersection edge is only as close to its faces as its tolerance.
        (ed.point(t) - p).norm() < (tol::REL_POINT * (1.0 + r)).max(ed.tol)
    };
    // The far end: on strip 1's other rail (face fa) or strip 2's (face fb).
    let on_s1 = 1.0 - u1 <= 1.0 - u2;
    let (qp, t2q, bl2) = if on_s1 {
        let qp = bl1.q[k1o].p;
        let mut st = st2;
        on2(&qp, &mut st).ok_or_else(lost)?;
        (qp, st.0, st.1)
    } else {
        // The mitre's own end (strip 2's ball there may be its last one, short of the point
        // when the point lies past its spine's end).
        let qp = if (st2.1.q[k2o].p - end_p).norm() < tol::REL * (1.0 + r) { st2.1.q[k2o].p } else { end_p };
        (qp, st2.0, st2.1)
    };
    let t1q = if on_s1 { t1q } else { s1.spine.param_of(&qp, t1q) };
    // The mitre's points again along strip 1 up to its far end, closer where the mitre turns
    // or runs fast (it meets the far end's face square).
    struct Pt {
        t: f64,
        p: P3,
        u: f64,
        ball: Ball,
        st2: (f64, Ball),
    }
    let m = 24;
    let mut pts: Vec<Pt> = vec![Pt { t: t1s, p: q1p, u: 0.0, ball: ball1s, st2: (t2s, ball2s) }];
    for i in 1..m {
        let t1 = t1s + (t1q - t1s) * i as f64 / m as f64;
        let prev = &pts[pts.len() - 1];
        let mut st = prev.st2;
        let (u, p, _, bl) = root(t1, &(prev.t, prev.ball), prev.u, &mut st)?.ok_or_else(lost)?;
        pts.push(Pt { t: t1, p, u, ball: bl, st2: st });
    }
    pts.push(Pt { t: t1q, p: qp, u: 1.0, ball: bl1, st2: (t2q, bl2) });
    loop {
        let len: f64 = pts.windows(2).map(|w| (w[1].p - w[0].p).norm()).sum();
        let turn = |i: usize| -> f64 {
            if i == 0 || i + 1 >= pts.len() {
                return 0.0;
            }
            (pts[i].p - pts[i - 1].p).angle(&(pts[i + 1].p - pts[i].p))
        };
        let seg = |i: usize| (pts[i + 1].p - pts[i].p).norm();
        let split: Vec<usize> = (0..pts.len() - 1).filter(|&i| seg(i) > len / 48.0 || (seg(i) > len / 2000.0 && turn(i).max(turn(i + 1)) > 4f64.to_radians())).collect();
        debug(|| format!("  mitre resampling: {} points, {} to split", pts.len(), split.len()));
        let mut inserted = 0;
        for &i in split.iter().rev() {
            if pts.len() > 400 {
                break;
            }
            let (a, z) = (&pts[i], &pts[i + 1]);
            let tm = 0.5 * (a.t + z.t);
            if (z.t - a.t).abs() < tol::REL * (1.0 + tm.abs()) {
                continue;
            }
            let mut st = a.st2;
            let (u, p, _, bl) = root(tm, &(a.t, a.ball), 0.5 * (a.u + z.u), &mut st)?.ok_or_else(lost)?;
            pts.insert(i + 1, Pt { t: tm, p, u, ball: bl, st2: st });
            inserted += 1;
        }
        if inserted == 0 {
            break;
        }
    }
    let pts: Vec<P3> = pts.into_iter().map(|x| x.p).collect();
    let q_on_edge = near_edge(&qp);
    debug(|| format!("vertex {v}: mitre of edges {} and {} from {:?} to {:?} ({} points, {})", s1.e, s2.e, q1p, qp, pts.len(), if q_on_edge { "on the third edge" } else if on_s1 { "on the first's rail" } else { "on the second's rail" }));
    // The strip whose other rail doesn't reach the far end leaves its face across the third
    // edge and is cut by the face beyond, from the far end.
    let exit_end = |s: &EdgeFillet, j: usize, ko: usize, end_ball: &Ball, from: Mark| -> Result<MitreEnd> {
        let x = rail_exit(b, s, j, ko, end_ball, Which::Edge(e3))?.ok_or_else(|| unsupported("a mitre whose strip doesn't reach the third edge"))?;
        let pts = march_on(b, s, &from, &Mark::rail(x.t, x.ball, ko), &b.faces[x.into])?;
        // Outside the face beyond (the strip running past the third edge's end there): not a
        // mitre (where that edge turns the other way, a mixed corner instead).
        if !point_in_face(b, x.into, &pts[pts.len() / 2].1) {
            return Err(unsupported("a mitre's end outside the face beyond"));
        }
        Ok(MitreEnd::Exit { exit: Box::new(x), k: ko, pts })
    };
    let w_of = |ks: usize, u: f64| if ks == 0 { u } else { 1.0 - u };
    let (o1, o2) = if q_on_edge {
        (MitreEnd::AtEnd { t: t1q }, MitreEnd::AtEnd { t: t2q })
    } else if on_s1 {
        (MitreEnd::AtEnd { t: t1q }, exit_end(s2, j2, k2o, &b2, Mark { t: t2q, w: w_of(k2s, frac(&bl2, k2s, &qp)), ball: bl2, p: qp })?)
    } else {
        (exit_end(s1, j1, k1o, &b1, Mark { t: t1q, w: w_of(k1s, frac(&bl1, k1s, &qp)), ball: bl1, p: qp })?, MitreEnd::AtEnd { t: t2q })
    };
    let m = Mitre { v, shared: fs, ps: q1p, q: qp, q_on_edge, e3, pts };
    Ok((m, End::Mitre { m: usize::MAX, t_s: t1s, other: o1 }, End::Mitre { m: usize::MAX, t_s: t2s, other: o2 }))
}

/// Where the rail of strip `a` on side `ka` crosses the rail of strip `b` on side `kb` (the
/// same face), from their balls `ba` (spine parameter ta) and `bb` (tb): Gauss–Newton on the
/// two spine parameters.
#[allow(clippy::too_many_arguments)]
fn rail_cross(body: &Body, a: &EdgeFillet, ka: usize, ba: Ball, ta: f64, b: &EdgeFillet, kb: usize, bb: Ball, tb: f64) -> Option<(f64, Ball, f64, Ball)> {
    let (mut ta, mut ba, mut tb, mut bb) = (ta, ba, tb, bb);
    let sc = 1.0 + ba.c.coords.abs().max() + a.r();
    for _ in 0..60 {
        let r = ba.q[ka].p - bb.q[kb].p;
        if r.norm() < 1e-10 * sc {
            return Some((ta, ba, tb, bb));
        }
        // The rails' directions: the balls' contacts' velocities (blends card 1).
        let da = a.velocity(body, ta, &ba)?[ka];
        let db = b.velocity(body, tb, &bb)?[kb];
        // Least squares: [da, −db] (dta, dtb) = −r.
        let m = nalgebra::Matrix2::new(da.dot(&da), -da.dot(&db), -da.dot(&db), db.dot(&db));
        let rhs = nalgebra::Vector2::new(-da.dot(&r), db.dot(&r));
        let d = m.lu().solve(&rhs)?;
        // Steps no longer than the radius.
        let len = (da * d.x).norm().max((db * d.y).norm());
        let f = if len > a.r() { a.r() / len } else { 1.0 };
        let (na, nbt) = (ta + d.x * f, tb + d.y * f);
        ba = a.ball_from(body, &ba, ta, na)?;
        bb = b.ball_from(body, &bb, tb, nbt)?;
        ta = na;
        tb = nbt;
    }
    None
}

/// The point where three planes (point, normal) meet.
fn meet_planes(a: (P3, V3), b: (P3, V3), c: (P3, V3)) -> Option<P3> {
    let m = nalgebra::Matrix3::from_rows(&[a.1.transpose(), b.1.transpose(), c.1.transpose()]);
    let r = V3::new(a.1.dot(&a.0.coords), b.1.dot(&b.0.coords), c.1.dot(&c.0.coords));
    Some(P3::from(m.try_inverse()? * r))
}

/// The ends of the three chamfers `here` (index, end) meeting at vertex v, whose three edges
/// they are ([`End::Star`]): the rails' crossings on the faces between them and, all cutting or
/// all filling, the point where the strips' planes meet; else the triangle through the
/// crossings.
fn star_corner(b: &Body, efs: &[EdgeFillet], here: &[(usize, usize)], v: usize) -> Result<(Vec<End>, Option<Cap>)> {
    let end_balls: Vec<(Ball, f64)> = here.iter().map(|&(i, j)| Ok((efs[i].end_ball(b, j)?, b.edges[efs[i].e].t[j]))).collect::<Result<_>>()?;
    // Each strip's crossings on its two faces: (spine parameter, ball).
    let mut at: Vec<[Option<(f64, Ball)>; 2]> = vec![[None, None]; here.len()];
    for x in 0..here.len() {
        for y in x + 1..here.len() {
            let (sa, sb) = (&efs[here[x].0], &efs[here[y].0]);
            let shared: Vec<usize> = sa.faces.iter().copied().filter(|f| sb.faces.contains(f)).collect();
            let [fs] = shared[..] else { return Err(unsupported("chamfers meeting at a corner without one common face each")) };
            let ka = if sa.faces[0] == fs { 0 } else { 1 };
            let kb = if sb.faces[0] == fs { 0 } else { 1 };
            let ((ba, ta), (bb, tb)) = (end_balls[x], end_balls[y]);
            let (ta, ba, tb, bb) = rail_cross(b, sa, ka, ba, ta, sb, kb, bb, tb).ok_or_else(|| unsupported("chamfers meeting at a corner whose rails don't cross"))?;
            if at[x][ka].is_some() || at[y][kb].is_some() {
                return Err(unsupported("chamfers meeting at a corner without one common face each"));
            }
            at[x][ka] = Some((ta, ba));
            at[y][kb] = Some((tb, bb));
        }
    }
    let lost = || unsupported("chamfers meeting at a corner whose strips don't meet in a point");
    let tol = tol::REL_POINT * (1.0 + efs[here[0].0].r());
    if !here.iter().all(|&(i, _)| (efs[i].k[0] < 0.0) == (efs[here[0].0].k[0] < 0.0)) {
        let mut ends = Vec::new();
        let mut corners: Vec<P3> = Vec::new();
        for a in &at {
            let [Some((t0, b0)), Some((t1, b1))] = *a else { return Err(lost()) };
            if (b0.q[0].p - b1.q[1].p).norm() < tol {
                return Err(unsupported("a chamfer corner whose strip ends in a point"));
            }
            for q in [b0.q[0].p, b1.q[1].p] {
                if !corners.iter().any(|c| (c - q).norm() < tol) {
                    corners.push(q);
                }
            }
            ends.push(End::Star { t: [t0, t1], balls: [b0, b1], p: None, t_p: t0 });
        }
        let [p0, p1, p2] = corners[..] else { return Err(lost()) };
        if (p1 - p0).cross(&(p2 - p0)).norm() < tol * tol {
            return Err(unsupported("a chamfer corner whose triangle degenerates"));
        }
        debug(|| format!("vertex {v}: chamfers of edges {:?} meet in a triangle", here.iter().map(|&(i, _)| efs[i].e).collect::<Vec<_>>()));
        let cap = Cap { v, edges: here.iter().map(|&(i, _)| efs[i].e).collect(), beside: efs[here[0].0].faces[0] };
        return Ok((ends, Some(cap)));
    }
    // Each strip's plane, through its section at a crossing and along the spine there (flat
    // strips only: the other crossing's section in it too).
    let mut planes: Vec<(P3, V3)> = Vec::new();
    for (a, &(i, _)) in at.iter().zip(here) {
        let [Some((t0, b0)), Some((_, b1))] = a else { return Err(lost()) };
        let pts = [b0.q[0].p, b0.q[1].p, b1.q[0].p, b1.q[1].p];
        let n = (pts[1] - pts[0]).cross(&efs[i].spine.at(*t0).1);
        if n.norm() < tol::TINY {
            return Err(lost());
        }
        let n = n.normalize();
        if pts.iter().any(|p| (p - pts[0]).dot(&n).abs() > tol) {
            return Err(unsupported("chamfers meeting at a corner on strips that aren't flat"));
        }
        planes.push((pts[0], n));
    }
    let p = meet_planes(planes[0], planes[1], planes[2]).ok_or_else(lost)?;
    // The point on each strip, between its rails.
    let mut ends = Vec::new();
    for (h, &(i, _)) in here.iter().enumerate() {
        let ef = &efs[i];
        let [Some((t0, b0)), Some((t1, b1))] = at[h] else { return Err(lost()) };
        let t_p = ef.spine.param_of(&p, t0);
        let bp = ef.ball_from(b, &b0, t0, t_p).ok_or_else(lost)?;
        if !(-tol::REL..=1.0 + tol::REL).contains(&bp.frac(0, &p)) || bp.sdist(&p).abs() > tol {
            debug(|| format!("vertex {v}: chamfer corner point {p:?} off edge {}'s strip (w {}, off by {})", ef.e, bp.frac(0, &p), bp.sdist(&p)));
            return Err(lost());
        }
        ends.push(End::Star { t: [t0, t1], balls: [b0, b1], p: Some(p), t_p });
    }
    debug(|| format!("vertex {v}: chamfers of edges {:?} meet at {p:?}", here.iter().map(|&(i, _)| efs[i].e).collect::<Vec<_>>()));
    Ok((ends, None))
}

/// The patched corner at vertex v where the strips `here` (index, end) meet: each strip's
/// rails stop where they cross the next strips' rails on their common faces.
fn patch_corner(b: &Body, efs: &[EdgeFillet], here: &[(usize, usize)], v: usize) -> Result<(PatchCorner, Vec<End>)> {
    // Each face between two of the strips, with the side of each.
    let mut by_face: HashMap<usize, Vec<(usize, usize)>> = HashMap::new(); // face → (position in here, side)
    for (h, &(i, _)) in here.iter().enumerate() {
        for (k, &f) in efs[i].faces.iter().enumerate() {
            by_face.entry(f).or_default().push((h, k));
        }
    }
    if by_face.len() != here.len() || by_face.values().any(|x| x.len() != 2 || x[0].0 == x[1].0) {
        return Err(unsupported("a patched fillet corner whose faces aren't each between two strips"));
    }
    let end_balls: Vec<(Ball, f64)> = here.iter().map(|&(i, j)| Ok((efs[i].end_ball(b, j)?, b.edges[efs[i].e].t[j]))).collect::<Result<_>>()?;
    // The crossing on each face: (spine parameter, ball) for each strip's side there.
    let mut at: Vec<[Option<(f64, Ball)>; 2]> = vec![[None, None]; here.len()];
    let mut faces: Vec<usize> = by_face.keys().copied().collect();
    faces.sort_unstable();
    for f in faces {
        let x = &by_face[&f];
        let ((ha, ka), (hb, kb)) = (x[0], x[1]);
        let (sa, sb) = (&efs[here[ha].0], &efs[here[hb].0]);
        let (ba, ta) = end_balls[ha];
        let (bb, tb) = end_balls[hb];
        let (ta, ba, tb, bb) = rail_cross(b, sa, ka, ba, ta, sb, kb, bb, tb).ok_or_else(|| {
            debug(|| format!("vertex {v}: the rails of edges {} and {} on face {f} don't cross (from {:?} and {:?})", sa.e, sb.e, ba.q[ka].p, bb.q[kb].p));
            unsupported("a patched fillet corner whose rails don't cross")
        })?;
        at[ha][ka] = Some((ta, ba));
        at[hb][kb] = Some((tb, bb));
    }
    // Each strip's curve across from its rail 0 end to its rail 1 end: the section point a
    // fraction w across at the spine parameter g(w) of the way, g a cubic from 0 to 1 whose
    // slopes at the ends are `s` (1 and 1: evenly).
    let across = |ef: &EdgeFillet, (t0, b0): (f64, Ball), (t1, b1): (f64, Ball), s: [f64; 2]| -> Result<Vec<P3>> {
        let g = |w: f64| s[0] * w + (3.0 - 2.0 * s[0] - s[1]) * w * w + (s[0] + s[1] - 2.0) * w * w * w;
        let n = 16;
        let mut pts = vec![b0.q[0].p];
        let mut cur = (t0, b0);
        for i in 1..n {
            let w = i as f64 / n as f64;
            let t = t0 + (t1 - t0) * g(w);
            let x = ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(|| failed("corner", "the fillet's ball is lost at a patched corner"))?;
            pts.push(x.arc_point(w));
            cur = (t, x);
        }
        pts.push(b1.q[1].p);
        Ok(pts)
    };
    // Whether a curve across near its end on face f crosses the strip beside it there (of the same kind: both cutting or both filling): its distance from that strip's
    // surface differs in sign next to the end and halfway across. Where the strips meet at an
    // angle, tangent to the face they share, their curves across leave the rails' crossing on
    // either side of the mitre between them; one crossing the other strip's surface there
    // crosses its curve across too (P3: each strip's curve bulging over the mitre), and the
    // strips cross.
    let crosses = |h: usize, k: usize, pts: &[P3]| -> bool {
        let ef = &efs[here[h].0];
        let Some(&(hn, _)) = by_face[&ef.faces[k]].iter().find(|x| x.0 != h) else { return false };
        let nf = &efs[here[hn].0];
        if (nf.k[0] < 0.0) != (ef.k[0] < 0.0) {
            return false;
        }
        let end = pts[if k == 0 { 0 } else { pts.len() - 1 }];
        let Some((tn, bn)) = at[hn].iter().flatten().find(|x| x.1.q.iter().any(|q| (q.p - end).norm() < 1e-6 * (1.0 + ef.r()))).copied() else { return false };
        let tol = tol::REL_POINT * (1.0 + nf.r());
        let near = [pts[if k == 0 { 1 } else { pts.len() - 2 }], pts[pts.len() / 2]];
        let mut g = (tn, bn);
        let mut side = 0.0;
        for p in &near {
            let t = nf.spine.param_of(p, g.0);
            let Some(bl) = nf.ball_from(b, &g.1, g.0, t) else { return false };
            g = (t, bl);
            let d = bl.sdist(p);
            if d.abs() > tol {
                if d * side < 0.0 {
                    return true;
                }
                side = d;
            }
        }
        false
    };
    let mut ends = Vec::new();
    for (h, &(i, _)) in here.iter().enumerate() {
        let ef = &efs[i];
        let [Some(a0), Some(a1)] = at[h] else { return Err(failed("corner", "a patched fillet corner without its crossings")) };
        let mut s = [1.0, 1.0];
        let mut pts = across(ef, a0, a1, s)?;
        // Crossing the strip beside at an end: the spine parameter moving on faster there (the
        // curve keeping nearer its rail; up to three times, where the cubic stays monotonic),
        // till it keeps to its side; else as it was.
        for k in 0..2 {
            if !crosses(h, k, &pts) {
                continue;
            }
            let mut fixed = false;
            for x in [1.5, 2.0, 2.5, 3.0] {
                s[k] = x;
                pts = across(ef, a0, a1, s)?;
                if !crosses(h, k, &pts) {
                    fixed = true;
                    break;
                }
            }
            if !fixed {
                debug(|| format!("vertex {v}: edge {}'s curve across crosses the strip beside on face {}", ef.e, ef.faces[k]));
                s[k] = 1.0;
                pts = across(ef, a0, a1, s)?;
            }
        }
        if s != [1.0, 1.0] {
            debug(|| format!("vertex {v}: edge {}'s curve across shaped by slopes {s:?}", ef.e));
        }
        ends.push(End::Patch { c: usize::MAX, t: [a0.0, a1.0], balls: [a0.1, a1.1], pts });
    }
    // The strips in turn round the vertex: each shares a face with the next.
    let mut order = vec![0usize];
    while order.len() < here.len() {
        let last = *order.last().ok_or_else(|| failed("corner", "a patched fillet corner without strips"))?;
        let fl = efs[here[last].0].faces;
        let next = (0..here.len()).find(|h| !order.contains(h) && efs[here[*h].0].faces.iter().any(|f| fl.contains(f)));
        match next {
            Some(h) => order.push(h),
            None => return Err(failed("corner", "a patched fillet corner's strips don't go round it")),
        }
    }
    debug(|| format!("vertex {v}: patched corner of {} strips", here.len()));
    Ok((PatchCorner { v, strips: order.iter().map(|&h| here[h].0).collect() }, ends))
}

/// The edges at vertex v in turn round it, each with the face between it and the next; `None`
/// where the faces don't go once round it (a free edge, a seam, a face at the vertex twice).
fn fan(b: &Body, v: usize) -> Option<Vec<(usize, usize)>> {
    let edges = vertex_edges(b, v);
    let e0 = *edges.first()?;
    if edges.iter().any(|&e| b.edges[e].v[0] == b.edges[e].v[1]) {
        return None;
    }
    let mut out = Vec::new();
    let (mut e, mut f) = (e0, sides(b, e0).ok()?[0].0);
    for _ in 0..edges.len() {
        out.push((e, f));
        // The face's other edge at the vertex.
        let at: Vec<usize> = b.faces[f].loops.iter().flatten().map(|c| c.edge).filter(|&x| x != e && !b.edges[x].degenerate && b.edges[x].v.contains(&v)).collect();
        let [next] = at[..] else { return None };
        if next == e0 {
            return (out.len() == edges.len()).then_some(out);
        }
        f = beyond(b, next, f)?;
        e = next;
    }
    None
}

/// The parameter of edge e a length `d` along it from its vertex v (`None` past its other end).
fn param_at(b: &Body, e: usize, v: usize, d: f64) -> Option<f64> {
    let ed = &b.edges[e];
    let (from, to) = if ed.v[0] == v { (ed.t[0], ed.t[1]) } else { (ed.t[1], ed.t[0]) };
    let len = |t: f64| ed.curve.length(from.min(t), from.max(t));
    if !(d < len(to)) {
        return None;
    }
    if let Curve::Line { d: dir, .. } = &ed.curve {
        return Some(from + (to - from).signum() * d / dir.norm());
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..50 {
        let m = 0.5 * (lo + hi);
        if len(from + (to - from) * m) < d { lo = m } else { hi = m }
    }
    Some(from + (to - from) * 0.5 * (lo + hi))
}

/// How far along edge e from its vertex v its (or its spine's) parameter t lies.
fn dist_along(b: &Body, e: usize, v: usize, t: f64) -> f64 {
    let ed = &b.edges[e];
    let tv = if ed.v[0] == v { ed.t[0] } else { ed.t[1] };
    ed.curve.length(tv.min(t), tv.max(t))
}

/// The set-back corner at vertex v where the round strips `here` (index, end) end (blends
/// card 9, the setback): the setback, past every crossing near the vertex of two rails on a
/// face or of a rail and an edge left as it is, by a radius; and each strip's setback
/// parameter there. `rolls` has, for each strip, the spine parameter of the roll at its other
/// end (where the ball rolls over a smooth edge round the vertex onto the face beyond: the
/// rail crossing that edge, so the setback is past it too, and the strip's end on the piece
/// beyond, [`set_back`]).
fn hole_corner(b: &Body, efs: &[EdgeFillet], here: &[(usize, usize)], rolls: &[Option<f64>], v: usize) -> Result<(Hole, Vec<f64>)> {
    let fan = fan(b, v).ok_or_else(|| unsupported("a fillet corner whose faces don't go once round it"))?;
    let strip_of = |e: usize| here.iter().position(|&(i, _)| efs[i].e == e);
    if here.iter().enumerate().any(|(h, &(i, _))| fan.iter().all(|x| x.0 != efs[i].e) || here[..h].iter().any(|&(o, _)| efs[o].e == efs[i].e)) {
        return Err(unsupported("a fillet corner on an edge closed at its vertex"));
    }
    // Round fillets. The patch's pole lies towards the vertex: out of a convex corner, into a
    // concave corner's room, and near the vertex where the edges round it turn both ways (a
    // saddle, the patch spanning it as OCCT's filling does).
    debug(|| format!("vertex {v}: fan {:?}, convexity {:?}, lengths {:?}", fan, fan.iter().map(|&(e, _)| edge_convexity(b, e)).collect::<Vec<_>>(), fan.iter().map(|&(e, _)| ed_len(b, e)).collect::<Vec<_>>()));
    if here.iter().any(|&(i, _)| efs[i].flat) {
        return Err(unsupported("a fillet corner of this kind"));
    }
    let end_balls: Vec<Ball> = here.iter().map(|&(i, j)| efs[i].end_ball(b, j)).collect::<Result<_>>()?;
    let m = fan.len();
    let mut need = 0.0f64;
    for (&(i, _), t) in here.iter().zip(rolls) {
        if let Some(t) = t {
            need = need.max(dist_along(b, efs[i].e, v, *t));
        }
    }
    // How far along each edge left as it is a rail crosses it.
    let mut need_on: HashMap<usize, f64> = HashMap::new();
    for i in 0..m {
        let (l, f) = fan[i];
        let r = fan[(i + 1) % m].0;
        // A strip rolled off face f near the vertex (onto the face beyond a smooth edge round
        // it) has no rail on f there: its roll is where its rail crosses that edge.
        let on_f = |h: usize| efs[here[h].0].faces.contains(&f);
        let side = |h: usize| if efs[here[h].0].faces[0] == f { 0 } else { 1 };
        let (hl, hr) = (strip_of(l), strip_of(r));
        if let (Some(x), Some(y)) = (hl, hr)
            && !(on_f(x) && on_f(y))
        {
            return Err(unsupported("a fillet corner whose rails don't cross"));
        }
        match (hl.filter(|&h| on_f(h)), hr.filter(|&h| on_f(h))) {
            (Some(hl), Some(hr)) => {
                let (sl, sr) = (&efs[here[hl].0], &efs[here[hr].0]);
                let (tl, tr) = (b.edges[sl.e].t[here[hl].1], b.edges[sr.e].t[here[hr].1]);
                let (ta, _, tb, _) = rail_cross(b, sl, side(hl), end_balls[hl], tl, sr, side(hr), end_balls[hr], tr).ok_or_else(|| unsupported("a fillet corner whose rails don't cross"))?;
                need = need.max(dist_along(b, l, v, ta)).max(dist_along(b, r, v, tb));
            }
            (Some(h), None) | (None, Some(h)) => {
                let sharp = if strip_of(l).is_some() { r } else { l };
                let (i, j) = here[h];
                let s = &efs[i];
                let Some(x) = rail_exit(b, s, j, side(h), &end_balls[h], Which::Edge(sharp))? else {
                    // Beside an edge between faces that join smoothly the rail may run on past
                    // the vertex without meeting it: nothing to set back from there.
                    if edge_convexity(b, sharp).is_none() {
                        continue;
                    }
                    return Err(unsupported("a fillet corner whose rail misses the edge beside it"));
                };
                let se = &b.edges[sharp];
                let ts = se.curve.closest(&x.ball.q[side(h)].p, se.t[0], se.t[1]);
                let on = dist_along(b, sharp, v, ts);
                need = need.max(dist_along(b, s.e, v, x.t)).max(on);
                let n = need_on.entry(sharp).or_default();
                *n = n.max(on);
            }
            (None, None) => {}
        }
    }
    let d = need + here.iter().map(|&(i, _)| efs[i].r()).fold(0.0, f64::max);
    // Each strip's section there, short of its edge's middle; each other edge's point short of
    // its far end.
    let mut ends = Vec::new();
    for &(i, j) in here {
        let s = &efs[i];
        let t = param_at(b, s.e, v, d).filter(|_| d < 0.5 * ed_len(b, s.e) || d < free_reach(b, efs, i, j)).ok_or_else(|| {
            debug(|| format!("vertex {v}: setback {d} (need {need}) past edge {}'s middle (length {}, free end {})", s.e, ed_len(b, s.e), free_reach(b, efs, i, j)));
            unsupported("a fillet corner set back past its edges' middles")
        })?;
        ends.push(t);
    }
    debug(|| format!("vertex {v}: setback {d} (need {need})"));
    // Each other edge's setback: short of its far end, but past where a rail crosses it.
    let mut ds: HashMap<usize, f64> = HashMap::new();
    for &(e, _) in fan.iter().filter(|&&(e, _)| strip_of(e).is_none()) {
        let (len, on) = (ed_len(b, e), need_on.get(&e).copied().unwrap_or(0.0));
        let de = if d <= 0.9 * len { d } else { 0.5 * (on + 0.9 * len) };
        if !(de > on) || param_at(b, e, v, de).is_none() {
            return Err(unsupported("a fillet corner set back past an edge's end"));
        }
        ds.insert(e, de);
    }
    debug(|| format!("vertex {v}: set-back corner of edges {:?} round {:?}, setback {d}", here.iter().map(|&(i, _)| efs[i].e).collect::<Vec<_>>(), fan));
    Ok((Hole { v, fan, d: ds }, ends))
}

/// How far from its end j a set-back corner may stop strip i, past its edge's middle: short of
/// where the rails of the edge's strip there leave their faces at its other end, where that end
/// is free (no other picked edge at its other vertex); else 0.
fn free_reach(b: &Body, efs: &[EdgeFillet], i: usize, j: usize) -> f64 {
    let e = efs[i].e;
    let (v, vo) = (b.edges[e].v[j], b.edges[e].v[1 - j]);
    // The edge's strip at its other end (past any rolls).
    let Some(far) = efs.iter().position(|o| o.e == e && !o.closed && o.at_vertex[1 - j]) else { return 0.0 };
    if v == vo || efs.iter().any(|o| o.e != e && !o.closed && (0..2).any(|k| o.at_vertex[k] && b.edges[o.e].v[k] == vo)) {
        return 0.0;
    }
    let s = &efs[far];
    match free_end(b, s, 1 - j) {
        // (Short of the nearer exit by a radius: the strip's end there is cut by its end face.)
        Ok(exits) => exits.iter().map(|x| dist_along(b, s.e, v, x.t)).fold(f64::INFINITY, f64::min) - s.r(),
        Err(_) => 0.0,
    }
}

/// A bilinearly blended (Coons) patch on four boundary curves, as a B-spline surface through
/// a grid of its points: `bottom` from S(0,0) to S(1,0), `top` from S(0,1) to S(1,1), `left`
/// from S(0,0) to S(0,1), `right` from S(1,0) to S(1,1).
fn coons(bottom: &dyn Fn(f64) -> P3, top: &dyn Fn(f64) -> P3, left: &dyn Fn(f64) -> P3, right: &dyn Fn(f64) -> P3) -> Option<Surface> {
    let n = 13;
    let ps: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();
    let (p00, p10, p01, p11) = (bottom(0.0), bottom(1.0), top(0.0), top(1.0));
    let at = |u: f64, v: f64| -> P3 {
        let a = bottom(u).coords * (1.0 - v) + top(u).coords * v;
        let b = left(v).coords * (1.0 - u) + right(v).coords * u;
        let c = p00.coords * ((1.0 - u) * (1.0 - v)) + p10.coords * (u * (1.0 - v)) + p01.coords * ((1.0 - u) * v) + p11.coords * (u * v);
        P3::from(a + b - c)
    };
    // Rows along v at each u, then the rows' control points along u.
    let rows: Vec<NurbsCurve> = ps.iter().map(|&u| NurbsCurve::interpolate(&ps.iter().map(|&v| at(u, v)).collect::<Vec<_>>(), &ps, 3, None, None)).collect();
    let nv = rows[0].cps.len();
    if rows.iter().any(|r| r.cps.len() != nv || r.knots != rows[0].knots) {
        debug(|| format!("coons: rows differ {:?}", rows.iter().map(|r| r.cps.len()).collect::<Vec<_>>()));
        return None;
    }
    let mut cps: Vec<Vec<super::nurbs::H>> = Vec::new();
    let mut ku = Vec::new();
    let mut cols: Vec<NurbsCurve> = Vec::new();
    for j in 0..nv {
        let pts: Vec<P3> = rows.iter().map(|r| super::nurbs::dehom(&r.cps[j])).collect();
        cols.push(NurbsCurve::interpolate(&pts, &ps, 3, None, None));
    }
    debug(|| format!("coons: column degrees {:?}, row degree {}", cols.iter().map(|c| c.degree).collect::<Vec<_>>(), rows[0].degree));
    let nu = cols[0].cps.len();
    for i in 0..nu {
        cps.push(cols.iter().map(|c| c.cps[i]).collect());
    }
    ku.clone_from(&cols[0].knots);
    let s = match super::nurbs::NurbsSurface::new(3, 3, ku, rows[0].knots.clone(), cps) {
        Ok(s) => s,
        Err(e) => {
            debug(|| format!("coons: {e}"));
            return None;
        }
    };
    Some(Surface::Nurbs(std::sync::Arc::new(s)))
}

/// The strip's curve across from rail 0's point of ball `a` to rail 1's of ball `z`: the
/// section point a fraction w across at the spine parameter a fraction w of the way.
fn across_points(b: &Body, ef: &EdgeFillet, a: &Exit, z: &Exit) -> Result<Vec<P3>> {
    let n = 16;
    let mut pts = vec![a.ball.q[0].p];
    let mut cur = (a.t, a.ball);
    for s in 1..n {
        let w = s as f64 / n as f64;
        let t = a.t + (z.t - a.t) * w;
        let x = ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(|| failed("end", "the fillet's ball is lost at its open end"))?;
        pts.push(x.arc_point(w));
        cur = (t, x);
    }
    pts.push(z.ball.q[1].p);
    Ok(pts)
}

/// Where a free end's cut crosses an edge between two of the faces it cuts: the point on the
/// strip and the face after the edge.
#[derive(Debug, Clone)]
struct Link {
    mark: Mark,
    after: usize,
    /// On the edge's extension past the vertex (the faces meet beyond the edge there).
    past: bool,
}

/// The faces round the end j's vertex between those the rails leave into, and where the strip
/// crosses the edges between them: the edges at the vertex, else (where the cut runs down a
/// step in the faces, away from the vertex) whichever edge of the face it's on the cut crosses
/// next.
fn end_chain(b: &Body, ef: &EdgeFillet, j: usize, exits: &[Exit; 2]) -> Result<Vec<Link>> {
    let v = b.edges[ef.e].v[j];
    let edges = vertex_edges(b, v);
    let (mut prev, mut cur) = (ef.faces[0], exits[0].into);
    let mut links: Vec<Link> = Vec::new();
    let mut guess = exits[0].ball;
    let mut tg = exits[0].t;
    let mut here = Mark::rail(exits[0].t, exits[0].ball, 0);
    let mut crossed: Vec<usize> = Vec::new();
    // Where edge d meets the strip, from its end at the vertex when `from_vertex` (the first
    // crossing), else all along it: the points on the ball of the section through each.
    let meets = |d: usize, from_vertex: bool, g0: (f64, Ball)| -> Vec<Mark> {
        let de = &b.edges[d];
        let (t0, t1) = if !from_vertex || de.v[0] == v { (de.t[0], de.t[1]) } else { (de.t[1], de.t[0]) };
        let dist = |s: f64, g: &mut (f64, Ball)| -> Option<(f64, Mark)> {
            let p = de.point(s);
            let t = ef.spine.param_of(&p, g.0);
            let bl = ef.ball_from(b, &g.1, g.0, t)?;
            *g = (t, bl);
            let w = bl.frac(0, &p);
            Some((bl.sdist(&p), Mark { t, w, ball: bl, p }))
        };
        let n = 64;
        let mut g = g0;
        let mut prev_s: Option<(f64, f64, (f64, Ball))> = None;
        let mut found = Vec::new();
        for i in 0..=n {
            let s = t0 + (t1 - t0) * i as f64 / n as f64;
            let Some((f, _)) = dist(s, &mut g) else {
                if from_vertex {
                    break;
                }
                prev_s = None;
                continue;
            };
            if let Some((sp, fp, gp)) = prev_s
                && (fp < 0.0) != (f < 0.0)
            {
                let (mut lo, mut hi, flo) = (sp, s, fp);
                let mut gg = gp;
                for _ in 0..60 {
                    let m = 0.5 * (lo + hi);
                    let Some((fm, _)) = dist(m, &mut gg) else { break };
                    if (fm < 0.0) == (flo < 0.0) { lo = m } else { hi = m }
                }
                found.extend(dist(0.5 * (lo + hi), &mut gg).map(|x| x.1));
                if from_vertex {
                    break;
                }
            }
            prev_s = Some((s, f, g));
        }
        found
    };
    // On the section's arc between the contacts, not on the ball's far side (as far from the
    // first contact the other way round: its fractions from the two contacts then add up to
    // well over 1; on the arc to 1, up to how far the point is out of the section's plane).
    let on_strip = |m: &Mark| (-tol::REL_POINT..=1.0 + tol::REL_POINT).contains(&m.w) && (m.w + m.ball.frac(1, &m.p) - 1.0).abs() < 1e-2;
    for _ in 0..edges.len() + 8 {
        // The edge at the vertex leading on from the current face (not back to the previous).
        let mut next = None;
        for &d in &edges {
            if d == ef.e {
                continue;
            }
            let [(a, _), (o, _)] = sides(b, d)?;
            if (a == cur && o != prev) || (o == cur && a != prev) {
                next = Some((d, if a == cur { o } else { a }));
            }
        }
        let at_vertex = next
            .filter(|&(_, after)| after != ef.faces[0] && after != ef.faces[1])
            .and_then(|(d, after)| meets(d, true, (tg, guess)).into_iter().next().filter(on_strip).map(|m| (d, after, m)));
        // Else the nearest crossing on along the cut of another edge of the face.
        let step = match at_vertex {
            Some(x) => Some(x),
            None => {
                let mut best: Option<(f64, (usize, usize, Mark))> = None;
                for c in b.faces[cur].loops.iter().flatten() {
                    let d = c.edge;
                    if d == ef.e || b.edges[d].degenerate || crossed.contains(&d) || is_seam(b, cur, d) {
                        continue;
                    }
                    let Some(after) = beyond(b, d, cur) else { continue };
                    if after == ef.faces[0] || after == ef.faces[1] {
                        continue;
                    }
                    for m in meets(d, false, (tg, guess)) {
                        debug(|| format!("edge {} end {j}: face {cur} edge {d} (to face {after}) meets the strip at {:?}, w {} / {}", ef.e, m.p, m.w, m.ball.frac(1, &m.p)));
                        if !on_strip(&m) || m.w < here.w - tol::REL_POINT {
                            continue;
                        }
                        let dd = (m.p - here.p).norm();
                        if best.as_ref().is_none_or(|x| dd < x.0) {
                            best = Some((dd, (d, after, m)));
                        }
                    }
                }
                best.map(|x| x.1)
            }
        };
        let Some((d, after, mark)) = step else {
            debug(|| format!("edge {} end {j}: the cut on face {cur} crosses none of its edges (exits into {} and {})", ef.e, exits[0].into, exits[1].into));
            return Err(unsupported("a fillet end that doesn't cross an edge round its vertex"));
        };
        // Reaching the other rail's exit (where that rail leaves its face at a vertex of this
        // one's edge): the cut ends there, on this face.
        if (mark.p - exits[1].ball.q[1].p).norm() < tol::REL_POINT * (1.0 + ef.r()) && !links.is_empty() {
            debug(|| format!("edge {} end {j}: the cut reaches the other rail's exit on face {cur}", ef.e));
            return Ok(links);
        }
        (guess, tg) = (mark.ball, mark.t);
        debug(|| format!("edge {} end {j}: the cut crosses edge {d} at {:?} (w {}) onto face {after}", ef.e, mark.p, mark.w));
        crossed.push(d);
        here = mark;
        links.push(Link { mark, after, past: false });
        if after == exits[1].into {
            return Ok(links);
        }
        prev = cur;
        cur = after;
    }
    Err(unsupported("a fillet end whose faces don't go round its vertex"))
}

/// [`end_chain`] where the strip doesn't cross the faces' edges at the vertex: the cut traced on
/// each face round the vertex in turn (from those the rails leave into) until it reaches the
/// next one's surface, past the vertex on the edge's extension between them (the fillet adds
/// to those faces, or uncovers a face under the corner it takes away); on the last face, or
/// on any it runs to the other rail's exit on, to there.
fn end_chain_traced(b: &Body, ef: &EdgeFillet, j: usize, exits: &[Exit; 2]) -> Result<Vec<Link>> {
    let v = b.edges[ef.e].v[j];
    let edges = vertex_edges(b, v);
    let (mut prev, mut cur) = (ef.faces[0], exits[0].into);
    let mut links: Vec<Link> = Vec::new();
    let mut from = Mark::rail(exits[0].t, exits[0].ball, 0);
    let to = Mark::rail(exits[1].t, exits[1].ball, 1);
    let tol = tol::REL_POINT * (1.0 + ef.r());
    for _ in 0..edges.len() {
        if cur == exits[1].into {
            return Ok(links);
        }
        let mut next = None;
        for &d in &edges {
            let [(a, _), (o, _)] = sides(b, d)?;
            if d != ef.e && ((a == cur && o != prev) || (o == cur && a != prev)) {
                next = Some((d, if a == cur { o } else { a }));
            }
        }
        let (d, after) = next.filter(|x| !ef.faces.contains(&x.1)).ok_or_else(|| unsupported("a fillet end whose faces don't go round its vertex"))?;
        let (_, at) = trace_until(b, ef, &b.faces[cur], &from, &to, Some(&b.faces[after]))?;
        let Some(mark) = at else {
            debug(|| format!("edge {} end {j}: the cut on face {cur} runs to the other rail's exit", ef.e));
            return Ok(links);
        };
        // On the edge itself, or on its extension past the vertex.
        let de = &b.edges[d];
        let tx = de.curve.closest(&mark.p, de.t[0] - 1e6, de.t[1] + 1e6);
        let past = !(de.t[0] - tol::ROUNDOFF..=de.t[1] + tol::ROUNDOFF).contains(&tx);
        if past && (de.curve.point(tx) - mark.p).norm() > tol {
            return Err(unsupported("a fillet end whose faces meet past a vertex away from their edge's curve"));
        }
        debug(|| format!("edge {} end {j}: the cut meets face {after} at {:?}{}", ef.e, mark.p, if past { " past the vertex" } else { "" }));
        links.push(Link { mark, after, past });
        (prev, cur, from) = (cur, after, mark);
    }
    Err(unsupported("a fillet end whose faces don't go round its vertex"))
}

/// The geometry of all strips' ends.
struct Plan {
    efs: Vec<EdgeFillet>,
    ends: Vec<[Option<End>; 2]>,
    /// Strips lying wholly in a set-back corner's hole: not built ([`set_back`]).
    inside: Vec<bool>,
    corners: Vec<Corner>,
    mitres: Vec<Mitre>,
    patches: Vec<PatchCorner>,
    tris: Vec<TriCorner>,
    caps: Vec<Cap>,
    holes: Vec<Hole>,
    knees: Vec<Knee>,
}

/// A corner where some of a vertex's edges are filleted and the others left as they are, or
/// more filleted edges meet than a ball touches the faces of (OCCT fills such a corner with one
/// patch, `ChFi3d_Builder::PerformMoreThreeCorner`): every edge there is set back `d` from the
/// vertex (past where any two rails, or a rail and an edge left as it is, cross near it), each
/// strip stops at its section there, each face round the vertex is cut across from one setback
/// point to the next, and one patch leaning along its neighbours fills the hole (as the smooth
/// corner's, [`super::blend_corner`]).
struct Hole {
    v: usize,
    /// The edges round the vertex in turn, each with the face between it and the next.
    fan: Vec<(usize, usize)>,
    /// The setback along each edge left as it is.
    d: HashMap<usize, f64>,
}

/// The triangle closing a corner of three chamfers, some cutting and some filling (each
/// strip's end there is [`End::Star`] without a point): at vertex `v`, the strips' edges,
/// beside face `beside`.
struct Cap {
    v: usize,
    edges: Vec<usize>,
    beside: usize,
}

/// Where the ball of strip `ef`, run from `from` (spine parameter t0) towards t1, first rolls
/// over a smooth edge of one of its faces onto the face beyond: (side, the face, the ball there,
/// its spine parameter).
fn first_roll(b: &Body, ef: &EdgeFillet, from: &Ball, t0: f64, t1: f64) -> Result<Option<(usize, usize, Ball, f64)>> {
    let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
    let reach = ef.reach + ed_len(b, ef.e);
    let (lo, hi) = (t0.min(t1), t0.max(t1));
    let span = hi - lo;
    let whole = (b.edges[ef.e].t[1] - b.edges[ef.e].t[0]).abs();
    let mut best: Option<(f64, (usize, usize, Ball, f64))> = None;
    for k in 0..2 {
        let fi = ef.faces[k];
        for c in b.faces[fi].loops.iter().flatten() {
            let x = &b.edges[c.edge];
            if c.edge == ef.e || x.degenerate || is_seam(b, fi, c.edge) {
                continue;
            }
            let Some(into) = beyond(b, c.edge, fi) else { continue };
            let bd = Boundary { curve: &x.curve, pc: &c.pc, t: x.t };
            for (_, ball) in crossings(ef, f, ef.k_at(t0), k, &bd, &from.c, reach) {
                let t = ef.spine.param_of(&ball.c, t0);
                // (Not the crossing the strip starts at, found again a hair off: where the rail
                // runs nearly along the edge it rolled over.)
                if !(t > lo + tol::REL * span && t < hi - tol::REL * span) || (t - t0).abs() < 1e-6 * whole {
                    continue;
                }
                // Not a far branch: its contacts near the spine there (a ball on the far side of
                // a cylinder the edge's face is part of crosses that face's far edges too).
                let p = ef.spine.at(t).0;
                if ball.q.iter().any(|q| (q.p - p).norm() > ef.reach) {
                    continue;
                }
                // Smooth there: the face beyond has the same normal.
                let uv = geom::foot(&b.faces[into], &ball.q[k].p, None);
                let n = geom::contact(&b.faces[into], uv).n;
                if n.dot(&ball.q[k].n) < 1.0 - tol::ANGULAR_POINT {
                    continue;
                }
                let d = (t - t0).abs();
                if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                    best = Some((d, (k, into, ball, t)));
                }
            }
        }
    }
    Ok(best.map(|x| x.1))
}

/// Where the ball of strip `ef`, rolling over a smooth edge on side k onto face `into` (ball
/// `ball` at spine parameter t), rolls straight back onto its face (within a point's
/// tolerance along the spine, before te): the ball there on the strip's faces and its spine
/// parameter.
fn graze(b: &Body, ef: &EdgeFillet, k: usize, into: usize, ball: &Ball, t: f64, te: f64) -> Result<Option<(Ball, f64)>> {
    let mut faces = ef.faces;
    faces[k] = into;
    let mut on = EdgeFillet { faces, mid: *ball, tm: t, exact: None, ..clone_fillet(ef) };
    on.mid.q[k] = geom::contact(&b.faces[into], geom::foot(&b.faces[into], &ball.q[k].p, None));
    let Some((k2, back, x, t2)) = first_roll(b, &on, &on.mid, t, te)? else { return Ok(None) };
    if k2 != k || back != ef.faces[k] || on.spine_len(t, t2) > tol::REL_POINT * (1.0 + ef.r()) {
        return Ok(None);
    }
    let fk = &b.faces[ef.faces[k]];
    let mut x = x;
    x.q[k] = geom::contact(fk, geom::foot(fk, &x.q[k].p, Some(ball.q[k].uv)));
    Ok(Some((x, t2)))
}

/// The strips of edge e: one per pair of faces the ball touches along it, in increasing spine
/// parameter (the ball rolls over smooth edges of the edge's faces onto the faces beyond).
#[allow(clippy::too_many_arguments)]
fn pieces_of(b: &Body, e: usize, law: &[(f64, f64)], flat: bool, ratio: f64, budget: &Budget, rolls: &mut Vec<Ball>) -> Result<Vec<(EdgeFillet, [Option<usize>; 2])>> {
    let mut base = edge_fillet(b, e, law, flat, ratio, budget)?;
    if base.closed {
        // A closed edge whose ball rolls over a smooth edge of one of its faces (a round's
        // tangent edge): opened at its vertex, its strips rolling as an open edge's do, the
        // first and last meeting there in a section (as a smooth chain's strips do).
        // (Only where a face of the edge has a smooth edge to roll over, tangent at its middle.)
        let ed = &b.edges[e];
        let smooth = base.faces.iter().any(|&fi| {
            b.faces[fi].loops.iter().flatten().any(|c| {
                let x = &b.edges[c.edge];
                let Some(g) = beyond(b, c.edge, fi).filter(|_| c.edge != e && !x.degenerate && !is_seam(b, fi, c.edge)) else { return false };
                let p = x.point(x.mid_t());
                let n = |f: &Face| geom::contact(f, geom::foot(f, &p, None)).n;
                n(&b.faces[fi]).dot(&n(&b.faces[g])) > 1.0 - tol::ANGULAR_POINT
            })
        });
        let open = EdgeFillet { closed: false, ..clone_fillet(&base) };
        if !smooth || base.law.is_some() || ed.t.iter().all(|&te| first_roll(b, &open, &open.mid, open.tm, te).ok().flatten().is_none()) {
            return Ok(vec![(base, [None, None])]);
        }
        debug(|| format!("edge {e}: closed, its ball rolling: opened at its vertex"));
        base = open;
    }
    // A contact already past a smooth edge of its face in the middle (a face split in two on
    // one surface, the edge's face the narrow part): the strip starts on the face beyond, and
    // rolls from there.
    for k in 0..2 {
        let fi = base.faces[k];
        let p = base.mid.q[k].p;
        if point_in_face(b, fi, &p) {
            continue;
        }
        let other = base.faces[1 - k];
        let beside = b.faces[fi]
            .loops
            .iter()
            .flatten()
            .filter(|c| c.edge != e && !b.edges[c.edge].degenerate && !is_seam(b, fi, c.edge))
            .filter_map(|c| beyond(b, c.edge, fi))
            .find(|&g| g != fi && g != other && same_surface(&b.faces[fi].surf, &b.faces[g].surf) && point_in_face(b, g, &p));
        if let Some(g) = beside {
            debug(|| format!("edge {e}: the ball's contact on face {fi} is on face {g}, beside it on its surface"));
            base.faces[k] = g;
            base.mid.q[k] = geom::contact(&b.faces[g], geom::foot(&b.faces[g], &p, None));
            base.exact = if base.law.is_some() { None } else { geom::exact_kind([&b.faces[base.faces[0]], &b.faces[base.faces[1]]], &base.spine) };
        }
    }
    let ed = &b.edges[e];
    // From the middle towards each end: each roll starts a strip with one face changed.
    let mut sides: [Vec<(EdgeFillet, usize)>; 2] = [Vec::new(), Vec::new()];
    for (j, side) in sides.iter_mut().enumerate() {
        let mut cur = EdgeFillet { mid: base.mid, faces: base.faces, ..clone_fillet(&base) };
        loop {
            let te = ed.t[j];
            let Some((k, into, ball, t)) = first_roll(b, &cur, &cur.mid, cur.tm, te)? else { break };
            debug(|| format!("edge {e} side {j}: the ball rolls from face {} onto face {into} at t {t} (from {})", cur.faces[k], cur.tm));
            // A contact grazing a smooth edge of its face (onto the face beyond and straight
            // back): no strip between, the ball runs on on its faces past there.
            if let Some(back) = graze(b, &cur, k, into, &ball, t, te)? {
                debug(|| format!("edge {e} side {j}: the ball only grazes face {into} (back at t {})", back.1));
                (cur.mid, cur.tm) = back;
                continue;
            }
            if side.len() > 16 {
                return Err(failed("rolling ball", "the fillet's ball rolls over too many faces"));
            }
            let mut faces = cur.faces;
            faces[k] = into;
            rolls.push(ball);
            let id = rolls.len() - 1;
            let mut next = EdgeFillet { faces, mid: ball, tm: t, exact: None, ..clone_fillet(&cur) };
            // The ball there touches the next face at the same point: its (u, v) there.
            next.mid.q[k] = geom::contact(&b.faces[into], geom::foot(&b.faces[into], &ball.q[k].p, None));
            next.exact = if next.law.is_some() { None } else { geom::exact_kind([&b.faces[faces[0]], &b.faces[faces[1]]], &next.spine) };
            side.push((clone_fillet(&cur), id));
            cur = next;
            let _ = j;
        }
        side.push((cur, usize::MAX));
    }
    // Lower side reversed, then the higher.
    let [low, high] = sides;
    let mut out: Vec<(EdgeFillet, [Option<usize>; 2])> = Vec::new();
    // low: [middle strip, …, lowest]; high: [middle strip, …, highest]. The middle strip is in
    // both: keep it once, with the rolls on either side.
    let n_low = low.len();
    for i in (1..n_low).rev() {
        let (f, _) = &low[i];
        let prev_roll = if i + 1 < n_low { Some(low[i].1) } else { None };
        let _ = prev_roll;
        // Strip i of the low side ends (upwards) at roll low[i - 1].1, downwards at the roll
        // that started it (low[i].1 is the roll beyond it, or none at the lowest).
        let down = if low[i].1 == usize::MAX { None } else { Some(low[i].1) };
        let up = Some(low[i - 1].1);
        out.push((clone_fillet(f), [down, up]));
    }
    let down = if low[0].1 == usize::MAX { None } else { Some(low[0].1) };
    let up = if high[0].1 == usize::MAX { None } else { Some(high[0].1) };
    out.push((clone_fillet(&high[0].0), [down, up]));
    for i in 1..high.len() {
        let down = Some(high[i - 1].1);
        let up = if high[i].1 == usize::MAX { None } else { Some(high[i].1) };
        out.push((clone_fillet(&high[i].0), [down, up]));
    }
    for (f, rs) in out.iter_mut() {
        f.at_vertex = [rs[0].is_none(), rs[1].is_none()];
    }
    Ok(out)
}

fn clone_fillet(f: &EdgeFillet) -> EdgeFillet {
    EdgeFillet { e: f.e, faces: f.faces, k: f.k, law: f.law.clone(), flat: f.flat, ratio: f.ratio, spine: f.spine.clone(), exact: f.exact, mid: f.mid, tm: f.tm, closed: f.closed, at_vertex: f.at_vertex, budget: f.budget.clone(), reach: f.reach }
}

/// The strips, ends and corners of the blend of `edges` (`ratios`: each one's
/// [`EdgeFillet::ratio`]).
fn plan(b: &Body, edges: &[RadiusLaw], ratios: &[f64], flat: bool, budget: &Budget) -> Result<Plan> {
    let mut efs: Vec<EdgeFillet> = Vec::new();
    let mut rolls: Vec<Ball> = Vec::new();
    let mut roll_ends: Vec<(usize, usize, usize)> = Vec::new(); // (strip, end, roll)
    for ((e, law), &ratio) in edges.iter().zip(ratios) {
        let e = *e;
        debug(|| format!("edge {e}: strips"));
        for (f, rs) in pieces_of(b, e, law, flat, ratio, budget, &mut rolls)? {
            for (j, x) in rs.iter().enumerate() {
                if let Some(id) = x {
                    roll_ends.push((efs.len(), j, *id));
                }
            }
            efs.push(f);
        }
    }
    let mut ends: Vec<[Option<End>; 2]> = vec![[None, None]; efs.len()];
    let mut inside = vec![false; efs.len()];
    for &(i, j, id) in &roll_ends {
        let ball = rolls[id];
        // Each strip's own contacts (the rolled side's face differs).
        let ef = &efs[i];
        // (The roll's (u, v) is on one of the faces either side of the edge rolled over: from
        // there, or afresh where that lands elsewhere on the other's surface.)
        let q: [Contact; 2] = std::array::from_fn(|k| {
            let g = &b.faces[ef.faces[k]];
            let p = &ball.q[k].p;
            let at = |hint: Option<P2>| geom::contact(g, geom::foot(g, p, hint));
            let c = at(Some(ball.q[k].uv));
            if (c.p - p).norm() <= tol::REL_POINT * (1.0 + ef.r()) { c } else { [c, at(None)].into_iter().min_by(|x, y| (x.p - p).norm().total_cmp(&(y.p - p).norm())).unwrap_or(c) }
        });
        let t = ef.spine.param_of(&ball.c, ef.tm);
        ends[i][j] = Some(End::Section { ball: Ball { c: ball.c, q, flat: ef.flat }, t, key: SectionKey::Roll(id) });
    }
    let mut corners = Vec::new();
    let mut mitres = Vec::new();
    let mut patches = Vec::new();
    let mut tris = Vec::new();
    let mut caps = Vec::new();
    let mut holes: Vec<Hole> = Vec::new();
    let mut knees: Vec<Knee> = Vec::new();
    let at = ends_at(b, &efs);
    let mut verts: Vec<usize> = at.keys().copied().collect();
    verts.sort_unstable();
    for v in verts {
        let here = &at[&v];
        debug(|| format!("vertex {v}: {} picked ends", here.len()));
        let all = vertex_edges(b, v);
        match here.len() {
            // (Or as good as 0: its contacts within the vertices' tolerance of the vertex.)
            1 if efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1]) <= tol::REL_POINT * (1.0 + efs[here[0].0].r()) => {
                let (i, j) = here[0];
                ends[i][j] = Some(End::Point { t: b.edges[efs[i].e].t[j] });
            }
            // The faces run on into each other smoothly at the vertex: the strip narrows to
            // nothing there and ends at it (blends card 8, a pinch).
            1 if pinch(b, &efs[here[0].0], here[0].1).is_some() => {
                let (i, j) = here[0];
                debug(|| format!("vertex {v}: edge {} pinches to its vertex", efs[i].e));
                ends[i][j] = Some(End::Point { t: b.edges[efs[i].e].t[j] });
            }
            1 => {
                let (i, j) = here[0];
                let ef = &efs[i];
                // An edge running on smoothly into one left as it is: the strip stops square.
                let out = leaving(b, ef.e, v);
                if all.iter().any(|&o| o != ef.e && leaving(b, o, v).dot(&-out) > 1.0 - tol::ANGULAR) {
                    ends[i][j] = Some(square_end(b, ef, j)?);
                    continue;
                }
                let exits = free_end(b, ef, j)?;
                ends[i][j] = Some(if exits.iter().any(|x| x.into == OPEN) {
                    if exits.iter().any(|x| x.into != OPEN || x.ext.is_some()) {
                        return Err(unsupported("a fillet ending at a sheet's free edge on one side only"));
                    }
                    End::Open { pts: across_points(b, ef, &exits[0], &exits[1])?, exits }
                } else if exits[0].into != exits[1].into && exits.iter().all(|x| x.ext.is_none()) {
                    End::Chain {
                        links: end_chain(b, ef, j, &exits).or_else(|e| {
                            debug(|| format!("edge {} end {j}: {e}; the cut traced instead", ef.e));
                            end_chain_traced(b, ef, j, &exits).map_err(|_| e)
                        })?,
                        exits,
                    }
                } else {
                    End::Free { exits }
                });
            }
            2 => {
                let (ia, ja) = here[0];
                let (ib, jb) = here[1];
                let (ea, eb) = (&efs[ia], &efs[ib]);
                // Both strips pinch to the vertex (their faces run on into each other there):
                // each ends at it.
                if pinch(b, ea, ja).is_some() && pinch(b, eb, jb).is_some() {
                    debug(|| format!("vertex {v}: edges {} and {} pinch to it", ea.e, eb.e));
                    ends[ia][ja] = Some(End::Point { t: b.edges[ea.e].t[ja] });
                    ends[ib][jb] = Some(End::Point { t: b.edges[eb.e].t[jb] });
                    continue;
                }
                // The directions they leave the vertex in (by which end: a closed edge opened
                // at its vertex has both there).
                let away = |e: usize, j: usize| {
                    let ed = &b.edges[e];
                    if ed.v[0] != ed.v[1] {
                        leaving(b, e, v)
                    } else if j == 0 {
                        ed.curve.tangent(ed.t[0]).normalize()
                    } else {
                        -ed.curve.tangent(ed.t[1]).normalize()
                    }
                };
                let (da, db) = (away(ea.e, ja), away(eb.e, jb));
                if (ea.k[0] < 0.0) != (eb.k[0] < 0.0) {
                    // At an angle at a vertex of three edges: a mixed corner.
                    if all.len() != 3 || ea.flat || da.dot(&-db) > 1.0 - 1e-7 {
                        return Err(unsupported("a convex and a concave fillet meeting"));
                    }
                    let third = all.iter().copied().find(|&e| e != ea.e && e != eb.e).unwrap_or(usize::MAX);
                    let (kn, end_a, end_b) = knee(b, &efs, here[0], here[1], v, third)?;
                    for (i, j, e) in [(ia, ja, end_a), (ib, jb, end_b)] {
                        ends[i][j] = Some(match e {
                            End::Knee { t, balls, pts, .. } => End::Knee { c: knees.len(), t, balls, pts },
                            e => e,
                        });
                    }
                    knees.push(kn);
                    continue;
                }
                if da.dot(&-db) < 1.0 - 1e-7 {
                    if all.len() != 3 {
                        debug(|| format!("vertex {v}: edges {:?}, picked {} and {} meet at an angle", all, ea.e, eb.e));
                        set_back(b, &efs, &mut ends, &mut inside, here, v, &mut holes)?;
                        continue;
                    }
                    // The strips mitred (a straight skeleton, as chamfers); where the mitre
                    // doesn't reach a rail and the edge left as it is turns the other way, a
                    // three-sided corner face instead (SMLib's mixed-convexity corner).
                    let third = all.iter().copied().find(|&e| e != ea.e && e != eb.e);
                    let mixed = third.and_then(|x| edge_convexity(b, x)) == Some(ea.k[0] > 0.0);
                    let mitred = mitre(b, &efs, here[0], here[1], v, &all);
                    if mitred.is_err() && mixed && !ea.flat {
                        let (tc, end_a, end_b) = tri_corner(b, &efs, here[0], here[1], v, third.unwrap_or(usize::MAX))?;
                        ends[ia][ja] = Some(end_a);
                        ends[ib][jb] = Some(end_b);
                        tris.push(tc);
                        continue;
                    }
                    // Else where the mitre fails, a set-back corner.
                    let (m, end_a, end_b) = match mitred {
                        Ok(x) => x,
                        Err(e) => {
                            debug(|| format!("vertex {v}: no mitre ({e}): a set-back corner"));
                            if set_back(b, &efs, &mut ends, &mut inside, here, v, &mut holes).is_err() {
                                return Err(e);
                            }
                            continue;
                        }
                    };
                    ends[ia][ja] = Some(end_a);
                    ends[ib][jb] = Some(end_b);
                    let _ = m.e3;
                    mitres.push(m);
                    let mi = mitres.len() - 1;
                    for e in [ia, ib] {
                        for x in ends[e].iter_mut().flatten() {
                            if let End::Mitre { m, .. } = x
                                && *m == usize::MAX {
                                    *m = mi;
                                }
                        }
                    }
                    continue;
                }
                let (mut ba, mut bb) = (ea.end_ball(b, ja)?, eb.end_ball(b, jb)?);
                let (mut ta, mut tb) = (b.edges[ea.e].t[ja], b.edges[eb.e].t[jb]);
                let near = |x: &Ball, y: &Ball| (x.c - y.c).norm() <= tol::REL_POINT * (1.0 + ea.r());
                if !near(&ba, &bb) {
                    // Edges tangent only within the angular tolerance: each strip's ball in its
                    // own section plane at the vertex differs by about the radius times their
                    // angle. Both in the plane square to their mean direction there instead.
                    let n = (da - db).normalize();
                    let vp = b.verts[v].p;
                    let in_plane = |ef: &EdgeFillet, j: usize, x: &Ball| {
                        let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
                        ef.budget.spend().then(|| geom::ball_in_plane(f, ef.k_at(b.edges[ef.e].t[j]), &vp, &n, [x.q[0].uv, x.q[1].uv])).flatten()
                    };
                    if let (Some(x), Some(y)) = (in_plane(ea, ja, &ba), in_plane(eb, jb, &bb))
                        && near(&x, &y)
                    {
                        debug(|| format!("vertex {v}: edges {} and {} meet in the plane square to their mean direction", ea.e, eb.e));
                        (ta, tb) = (ea.spine.param_of(&x.c, ta), eb.spine.param_of(&y.c, tb));
                        (ba, bb) = (Ball { flat: ea.flat, ..x }, Ball { flat: eb.flat, ..y });
                    }
                }
                if !near(&ba, &bb) {
                    debug(|| format!("vertex {v}: edges {} and {} end in balls at {:?} and {:?} (ends {ja} {jb}; mids {:?} at {:?} and {:?} at {:?}; spine ends {:?} {:?})", ea.e, eb.e, ba.c, bb.c, ea.mid.c, ea.spine.at(ea.tm).0, eb.mid.c, eb.spine.at(eb.tm).0, ea.spine.at(b.edges[ea.e].t[ja]).0, eb.spine.at(b.edges[eb.e].t[jb]).0));
                    return Err(unsupported("tangent fillets whose sections don't meet"));
                }
                let key = SectionKey::Junction(v);
                ends[ia][ja] = Some(End::Section { ball: ba, t: ta, key });
                ends[ib][jb] = Some(End::Section { ball: bb, t: tb, key });
            }
            3 if efs[here[0].0].flat && all.len() == 3 => {
                let (sends, cap) = star_corner(b, &efs, here, v)?;
                for (&(i, j), e) in here.iter().zip(sends) {
                    ends[i][j] = Some(e);
                }
                caps.extend(cap);
            }
            n if n >= 3 && efs[here[0].0].flat => {
                let _ = n;
                return Err(unsupported("chamfers meeting at a corner"));
            }
            n if n >= 3 && n == all.len() && (n > 3 || here.iter().any(|&(i, j)| (efs[i].k[0] < 0.0) != (efs[here[0].0].k[0] < 0.0) || (efs[i].radius_at(b.edges[efs[i].e].t[j]) - efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1])).abs() > tol::REL * (1.0 + efs[i].r()))) => {
                let (pc, pends) = patch_corner(b, &efs, here, v)?;
                for ((i, j), e) in here.iter().zip(pends) {
                    let End::Patch { t, balls, pts, .. } = e else { continue };
                    ends[*i][*j] = Some(End::Patch { c: patches.len(), t, balls, pts });
                }
                patches.push(pc);
            }
            3 if all.len() == 3 && !efs[here[0].0].flat && here.iter().all(|&(i, j)| (efs[i].radius_at(b.edges[efs[i].e].t[j]) - efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1])).abs() < tol::REL * (1.0 + efs[i].r())) => {
                // The three faces, with contacts near the corner to start from (each strip's
                // ball at the vertex).
                let mut faces: Vec<(usize, P2)> = Vec::new();
                for &(i, j) in here {
                    let bl = efs[i].end_ball(b, j)?;
                    for k in 0..2 {
                        if !faces.iter().any(|x| x.0 == efs[i].faces[k]) {
                            faces.push((efs[i].faces[k], bl.q[k].uv));
                        }
                    }
                }
                if faces.len() != 3 {
                    return Err(unsupported("a fillet corner that isn't between three faces"));
                }
                let kk = efs[here[0].0].k_at(b.edges[efs[here[0].0].e].t[here[0].1])[0];
                let fs = [&b.faces[faces[0].0], &b.faces[faces[1].0], &b.faces[faces[2].0]];
                let (c, q) = geom::ball3(fs, [kk; 3], [faces[0].1, faces[1].1, faces[2].1]).ok_or_else(|| failed("corner", "no ball touches the three faces at the fillets' corner"))?;
                let contacts: Vec<(usize, Contact)> = (0..3).map(|i| (faces[i].0, q[i])).collect();
                let on = |fi: usize| contacts.iter().find(|x| x.0 == fi).map(|x| x.1);
                for &(i, j) in here {
                    let ef = &efs[i];
                    let (Some(qa), Some(qb)) = (on(ef.faces[0]), on(ef.faces[1])) else { return Err(failed("corner", "a corner fillet without its faces")) };
                    let t = ef.spine.param_of(&c, b.edges[ef.e].t[j]);
                    ends[i][j] = Some(End::Section { ball: Ball { c, q: [qa, qb], flat: false }, t, key: SectionKey::Corner(v, ef.e) });
                }
                corners.push(Corner { v, c, contacts, edges: here.iter().map(|&(i, _)| efs[i].e).collect() });
            }
            _ => {
                debug(|| format!("vertex {v}: {} picked ends of {} edges ({:?}), convex {:?}", here.len(), all.len(), here.iter().map(|&(i, _)| efs[i].e).collect::<Vec<_>>(), here.iter().map(|&(i, _)| efs[i].k[0] < 0.0).collect::<Vec<_>>()));
                set_back(b, &efs, &mut ends, &mut inside, here, v, &mut holes)?;
            }
        }
    }
    Ok(Plan { efs, ends, inside, corners, mitres, patches, tris, caps, holes, knees })
}

/// The set-back corner at vertex v where the strips `here` (index, end) end ([`hole_corner`]):
/// each strip's end there [`End::Hole`] (`holes`' next). A strip whose ball rolls over a smooth
/// edge round the vertex before the setback (two faces on one surface split at the vertex) lies
/// in the hole: it is left out (`inside`), and the piece beyond its roll ends there instead.
fn set_back(b: &Body, efs: &[EdgeFillet], ends: &mut [[Option<End>; 2]], inside: &mut [bool], here: &[(usize, usize)], v: usize, holes: &mut Vec<Hole>) -> Result<()> {
    // The roll at a strip's end j (its spine parameter and index).
    let roll = |ends: &[[Option<End>; 2]], i: usize, j: usize| match &ends[i][j] {
        Some(End::Section { t, key: SectionKey::Roll(id), .. }) => Some((*t, *id)),
        _ => None,
    };
    let rolls: Vec<Option<f64>> = here.iter().map(|&(i, j)| roll(ends, i, 1 - j).map(|x| x.0)).collect();
    let (h, ts) = hole_corner(b, efs, here, &rolls, v).inspect_err(|e| debug(|| format!("vertex {v}: no set-back corner: {e}")))?;
    let mut set = Vec::new();
    for (&(i0, j), &t) in here.iter().zip(&ts) {
        let e = efs[i0].e;
        let mut i = i0;
        let mut gone = Vec::new();
        // On along the edge's pieces past every roll nearer the vertex than the setback.
        while let Some((tr, id)) = roll(ends, i, 1 - j)
            && dist_along(b, e, v, tr) < dist_along(b, e, v, t)
        {
            let next = (0..efs.len()).find(|&x| x != i && efs[x].e == e && roll(ends, x, j).is_some_and(|y| y.1 == id)).ok_or_else(|| failed("corner", "a set-back corner's strip has no piece past its roll"))?;
            gone.push(i);
            i = next;
        }
        let s = &efs[i];
        let ball = s.ball_from(b, &s.mid, s.tm, t).ok_or_else(|| failed("corner", "the fillet's ball is lost at a set-back corner"))?;
        set.push((i, j, t, ball, gone));
    }
    for (i, j, t, ball, gone) in set {
        if !gone.is_empty() {
            debug(|| format!("vertex {v}: edge {}'s strip ends at the setback past its roll, strips {gone:?} in the hole", efs[i].e));
        }
        for x in gone {
            inside[x] = true;
        }
        ends[i][j] = Some(End::Hole { c: holes.len(), t, ball });
    }
    holes.push(h);
    Ok(())
}

/// A point on a strip: its spine parameter, the fraction of the way across its section (from
/// the first contact to the second), the ball there and the point.
#[derive(Debug, Clone, Copy)]
struct Mark {
    t: f64,
    w: f64,
    ball: Ball,
    p: P3,
}

impl Mark {
    /// Rail k's point of `ball` at spine parameter t.
    fn rail(t: f64, ball: Ball, k: usize) -> Mark {
        Mark { t, w: k as f64, ball, p: ball.q[k].p }
    }
}

/// An end curve: the fillet cut by end face `g`, from the exit on rail 0 to the one on rail 1.
fn march_end(b: &Body, ef: &EdgeFillet, exits: &[Exit; 2], g: &Face) -> Result<Vec<(f64, P3)>> {
    march_on(b, ef, &Mark::rail(exits[0].t, exits[0].ball, 0), &Mark::rail(exits[1].t, exits[1].ball, 1), g)
}

/// The fillet's curve on face `g`'s surface from `a` to `z`: points marched across the strip
/// (the fraction w of each section's arc going evenly from a's to z's, at the spine parameter
/// where that point lies on the face), with their spine parameters; then more between those
/// far apart (near a rail the curve runs along the strip, w changing slowly along it).
fn march_on(b: &Body, ef: &EdgeFillet, a: &Mark, z: &Mark, g: &Face) -> Result<Vec<(f64, P3)>> {
    let n = 24;
    let hint: std::cell::Cell<Option<P2>> = std::cell::Cell::new(None);
    let sd = |p: &P3| -> f64 {
        let uv = geom::foot(g, p, hint.get());
        hint.set(Some(uv));
        let c = geom::contact(g, uv);
        (p - c.p).dot(&c.n)
    };
    let r = ef.r();
    let lost = || failed("end", "the fillet's end can't be traced");
    // The point at fraction w of the section on the face, from the ball `ball` at t moved on to
    // ta.
    let solve = |w: f64, t: f64, ball: &Ball, ta: f64| -> Result<(f64, Ball)> {
        // Secant on the face's signed distance, from the previous point moved on as the ends'
        // parameters are.
        let mut ta = ta;
        let mut ba = ef.ball_from(b, ball, t, ta).ok_or_else(lost)?;
        let mut ga = sd(&ba.arc_point(w));
        // The balls are solved to about 1e-11 of the model's size: the distance is no finer.
        let sc = 1.0 + ball.c.coords.abs().max() + r;
        // On the face already (a face holding the strip's lines, as a plane holds a cylinder's
        // rulings): there, not a step on (the steps would add up past the end).
        if ga.abs() < 1e-10 * sc {
            return Ok((ta, ba));
        }
        let h = 1e-3 * r / ef.spine.curve.d2(ta).1.norm().max(tol::DEGENERATE);
        let mut tb = ta + h;
        let mut bb = ef.ball(b, tb, Some([ba.q[0].uv, ba.q[1].uv])).ok_or_else(lost)?;
        let mut gb = sd(&bb.arc_point(w));
        for _ in 0..60 {
            let stalled = (tb - ta).abs() <= tol::REL_FINE * (1.0 + tb.abs());
            if gb.abs() < 1e-10 * sc || (stalled && gb.abs() < 1e-8 * sc) {
                return Ok((tb, bb));
            }
            if (gb - ga).abs() < tol::TINY || stalled {
                break;
            }
            let step = (-gb * (tb - ta) / (gb - ga)).clamp(-r, r);
            let tn = tb + step;
            let bn = ef.ball(b, tn, Some([bb.q[0].uv, bb.q[1].uv])).ok_or_else(lost)?;
            (ta, ba, ga) = (tb, bb, gb);
            tb = tn;
            bb = bn;
            gb = sd(&bb.arc_point(w));
        }
        let _ = ba;
        debug(|| format!("edge {}: end march lost at w {w}: t {ta}..{tb}, g {ga}..{gb}", ef.e));
        Err(failed("end", "the fillet's end doesn't meet its end face"))
    };
    // (w, t, ball, point)
    let mut out: Vec<(f64, f64, Ball, P3)> = vec![(a.w, a.t, a.ball, a.p)];
    let dt = (z.t - a.t) / n as f64;
    for i in 1..n {
        let w = a.w + (z.w - a.w) * i as f64 / n as f64;
        let (t0, b0) = (out[i - 1].1, out[i - 1].2);
        let (t, ball) = solve(w, t0, &b0, t0 + dt)?;
        out.push((w, t, ball, ball.arc_point(w)));
    }
    out.push((z.w, z.t, z.ball, z.p));
    // Points between those much farther apart than the rest, a few rounds.
    for _ in 0..8 {
        let len: f64 = out.windows(2).map(|x| (x[1].3 - x[0].3).norm()).sum();
        let mut i = 0;
        let mut added = false;
        while i + 1 < out.len() && out.len() < 200 {
            let (p, q) = (&out[i], &out[i + 1]);
            if (q.3 - p.3).norm() > len / 32.0 {
                let w = 0.5 * (p.0 + q.0);
                let (t0, b0) = (p.1, p.2);
                // (Where the solve fails, the points as they are.)
                if let Ok((t, ball)) = solve(w, t0, &b0, 0.5 * (p.1 + q.1)) {
                    let pt = ball.arc_point(w);
                    out.insert(i + 1, (w, t, ball, pt));
                    added = true;
                    i += 1;
                }
            }
            i += 1;
        }
        if !added {
            break;
        }
    }
    Ok(out.into_iter().map(|x| (x.1, x.3)).collect())
}

/// The strip of `ef`: exact where it can be, else a B-spline through sections at stations
/// covering the spine parameters `cover`, refined to `tol`. The sections at the parameters of
/// `pinned` are those balls (an end pinched to its vertex: [`pinch`]).
fn build_strip(b: &Body, ef: &EdgeFillet, cover: &[f64], pinned: &[(f64, Ball)], tol: f64) -> Result<Strip> {
    if let Some(kind) = ef.exact
        && let Some(strip) = geom::exact_strip(kind, &ef.mid, &ef.spine) {
            return Ok(strip);
        }
    let ed = &b.edges[ef.e];
    let (lo, hi) = if ef.closed {
        // A closed strip runs a whole turn from its seam.
        let from = cover.first().copied().unwrap_or(ed.t[0]);
        (from, from + (ed.t[1] - ed.t[0]))
    } else {
        (cover.iter().copied().fold(f64::INFINITY, f64::min), cover.iter().copied().fold(f64::NEG_INFINITY, f64::max))
    };
    if !(hi > lo) {
        return Err(failed("surface", "the fillet strip is empty"));
    }
    let span = hi - lo;
    // Stations: evenly spaced (one per four radii of length, at most 64; more where the sections need them), plus every parameter the ends need
    // (their sections must be exact); an even one too near one of those gives way.
    let n0 = ((ef.spine_len(lo, hi) / (4.0 * ef.r())).ceil() as usize).clamp(8, 64);
    let mut ts: Vec<f64> = (0..=n0).map(|i| lo + span * i as f64 / n0 as f64).collect();
    let is_cover = |t: f64| cover.iter().any(|c| (c - t).abs() <= tol::REL_FINE * (1.0 + span));
    ts.extend(cover.iter().copied());
    let mut stalled = 0;
    // The radius law's points: its slope jumps there, and so do the sections' (kinks from the
    // start, stations exact there). Round a closed edge, taken into the strip's turn.
    let mut kinks: Vec<f64> = Vec::new();
    for &(t, _) in ef.law.iter().flatten() {
        let t = if ef.closed { lo + wrap(t - lo, 0.0, span) } else { t };
        if t - lo > tol::REL_POINT * span && hi - t > tol::REL_POINT * span && !kinks.contains(&t) {
            kinks.push(t);
            ts.push(t);
        }
    }
    loop {
        ts.sort_by(|a, b| a.total_cmp(b));
        let mut keep: Vec<f64> = Vec::new();
        for &t in &ts {
            if let Some(last) = keep.last_mut()
                && t - *last < tol::REL_POINT * span
            {
                if is_cover(t) || kinks.contains(&t) {
                    *last = t;
                }
                continue;
            }
            keep.push(t);
        }
        ts = keep;
        let balls = balls_at(b, ef, &ts, pinned)?;
        debug(|| format!("edge {}: {} stations", ef.e, ts.len()));
        let mut kink_at: Vec<usize> = kinks.iter().filter_map(|k| ts.iter().position(|t| t == k)).collect();
        kink_at.sort_unstable();
        let strip = geom::nurbs_strip(&balls, &ts, ef.closed, &kink_at).ok_or_else(|| failed("surface", "the fillet surface can't be built"))?;
        // Check between stations; add stations where it strays.
        let mut truth: Vec<(f64, Ball)> = Vec::new();
        let wide = |x: &Ball| (x.q[0].p - x.q[1].p).norm();
        // A tenth of the tolerance where a section pinches to a point (the faces turning
        // tangent, not the radius going to 0): the rails come in tangent to each other (and to
        // the faces' edges) there, so a rail that strays from its face by the tolerance crosses
        // that edge in the face's (u, v).
        let tol_here = if balls.iter().zip(&ts).any(|(x, &t)| wide(x) <= tol && ef.radius_at(t) > tol) { 0.1 * tol } else { tol };
        for (w, bl) in ts.windows(2).zip(balls.windows(2)) {
            let t = 0.5 * (w[0] + w[1]);
            // From the wider of the two sections: where the faces turn tangent (the strip
            // pinching to a point at a vertex), the narrow section's ball is ill-conditioned
            // and Newton from it lands on a far branch (blends card 8).
            let i = if wide(&bl[1]) > wide(&bl[0]) { 1 } else { 0 };
            let x = ef.ball_from(b, &bl[i], w[i], t).ok_or_else(|| failed("rolling ball", "the fillet's ball is lost along the edge"))?;
            truth.push((t, x));
        }
        let errs = geom::strip_error(&strip, &truth);
        let bad: Vec<f64> = truth.iter().zip(&errs).filter(|(_, e)| **e > tol_here).map(|((t, _), _)| *t).collect();
        if bad.is_empty() {
            return Ok(strip);
        }
        // Stations already as close as they go: the sections kink there (a face's curvature
        // jumps), so the strip is only continuous there; failing that, good enough within a
        // hundred times the tolerance.
        if ts.len() == stalled {
            let fresh: Vec<f64> = bad.iter().copied().filter(|t| !kinks.iter().any(|k| (k - t).abs() < 1e-5 * span)).collect();
            if fresh.is_empty() {
                let worst = errs.iter().copied().fold(0.0, f64::max);
                if worst <= 100.0 * tol {
                    return Ok(strip);
                }
                return Err(failed("surface", "the fillet's sections kink along the edge"));
            }
            debug(|| format!("edge {}: kinks at {fresh:?}", ef.e));
            kinks.extend(fresh.iter().copied());
            ts.extend(fresh);
            stalled = 0;
            continue;
        }
        stalled = ts.len();
        if ts.len() + bad.len() > 4000 {
            return Err(failed("surface", "the fillet surface needs too many sections"));
        }
        ts.extend(bad);
    }
}

/// The ball at end j of `ef` where its faces run on into each other smoothly at the vertex
/// (their normals there one): both contacts the vertex. None elsewhere.
fn pinch(b: &Body, ef: &EdgeFillet, j: usize) -> Option<Ball> {
    let ed = &b.edges[ef.e];
    let te = ed.t[j];
    let mut q = [Contact { uv: P2::origin(), p: P3::origin(), n: V3::z() }; 2];
    for k in 0..2 {
        let c = coedge_of(b, ef.faces[k], ef.e)?;
        let uv = c.pc.point(te);
        q[k] = geom::contact(&b.faces[ef.faces[k]], P2::new(uv.x, uv.y));
    }
    if q[0].n.dot(&q[1].n) < 1.0 - tol::ANGULAR {
        return None;
    }
    let p = b.verts[ed.v[j]].p;
    for x in q.iter_mut() {
        x.p = p;
    }
    Some(Ball { c: p + q[0].n * ef.k_at(te)[0], q, flat: ef.flat })
}

/// The balls at increasing spine parameters, marched from the middle ball; at a parameter of
/// `pinned`, that ball. Where the faces turn tangent (the strip pinching to a vertex) the
/// balls are ill-conditioned (many contacts nearly touch both faces) and marching may find
/// none at the vertex itself, where the pinned ball is exact.
fn balls_at(b: &Body, ef: &EdgeFillet, ts: &[f64], pinned: &[(f64, Ball)]) -> Result<Vec<Ball>> {
    let tm = ef.tm;
    let i0 = ts.iter().position(|&t| t >= tm).unwrap_or(ts.len() - 1);
    let mut out = vec![dummy_ball(); ts.len()];
    let lost = || failed("rolling ball", "the fillet's ball is lost along the edge");
    let pin = |t: f64| pinned.iter().find(|p| p.0 == t).map(|p| p.1);
    // The ball at ts[i] marched from `from` at ts[j], or the pinned one there (marching may not
    // reach it: a strip pinching to a point at its end). Marched on from the marched balls
    // (better conditioned).
    let step = |from: &Ball, j: usize, i: usize| -> Result<(Ball, Ball)> {
        let x = ef.ball_from(b, from, ts[j], ts[i]);
        match (x, pin(ts[i])) {
            (Some(x), p) => Ok((x, p.unwrap_or(x))),
            (None, Some(p)) => Ok((p, p)),
            (None, None) => Err(lost()),
        }
    };
    let i0_ball = ef.ball_from(b, &ef.mid, tm, ts[i0]).ok_or_else(lost)?;
    out[i0] = pin(ts[i0]).unwrap_or(i0_ball);
    let mut cur = i0_ball;
    for (i, o) in out.iter_mut().enumerate().skip(i0 + 1) {
        (cur, *o) = step(&cur, i - 1, i)?;
    }
    cur = i0_ball;
    for (i, o) in out.iter_mut().enumerate().take(i0).rev() {
        (cur, *o) = step(&cur, i + 1, i)?;
    }
    Ok(out)
}

/// Splits edge `e` at parameter `t` (strictly inside): the edge keeps [t0, t], a new edge takes
/// [t, t1] from a new vertex at the point; every coedge on it becomes two, in its direction.
/// Returns the new vertex.
fn split_edge(b: &mut Body, e: usize, t: f64) -> usize {
    let p = b.edges[e].point(t);
    let nv = b.add_vertex(p);
    let old = b.edges[e].clone();
    let ne = b.edges.len();
    b.edges.push(super::topo::Edge { curve: old.curve.clone(), t: [t, old.t[1]], v: [nv, old.v[1]], degenerate: false, tol: old.tol });
    b.edges[e].t = [old.t[0], t];
    b.edges[e].v = [old.v[0], nv];
    for f in b.faces.iter_mut() {
        for l in f.loops.iter_mut() {
            let mut out: Vec<Coedge> = Vec::with_capacity(l.len() + 1);
            for c in l.drain(..) {
                if c.edge != e {
                    out.push(c);
                    continue;
                }
                let second = Coedge { edge: ne, fwd: c.fwd, pc: c.pc.clone() };
                if c.fwd {
                    out.push(c);
                    out.push(second);
                } else {
                    out.push(second);
                    out.push(c);
                }
            }
            *l = out;
        }
    }
    nv
}

/// The vertex at point `p` on the boundary of face `fi` (as it is now): an existing vertex
/// when the point is one, else a new vertex splitting the edge it lies on. `None` when `p`
/// isn't on the boundary. On it means near the edge's curve or near its p-curve's image on the
/// face (a rail's exit is solved on the p-curve; a read model's edge may stray from it by more
/// than its stated tolerance).
fn vertex_on_boundary(b: &mut Body, fi: usize, p: &P3, tol: f64) -> Option<usize> {
    let mut best = (f64::MAX, usize::MAX, 0.0);
    let surf = &b.faces[fi].surf;
    for c in b.faces[fi].loops.iter().flatten() {
        let ee = &b.edges[c.edge];
        if ee.degenerate {
            continue;
        }
        let t = ee.curve.closest(p, ee.t[0], ee.t[1]);
        let q = c.pc.point(t);
        let d = (ee.point(t) - p).norm().min((surf.eval(q.x, q.y) - p).norm());
        if d < best.0 {
            best = (d, c.edge, t);
        }
    }
    if best.1 == usize::MAX || best.0 > tol.max(b.edges[best.1].tol) {
        return None;
    }
    let (_, e, t) = best;
    for v in b.edges[e].v {
        if (b.verts[v].p - p).norm() < tol.max(b.verts[v].tol) {
            return Some(v);
        }
    }
    Some(split_edge(b, e, t))
}

/// Whether point `p` lies on the boundary of face `fi`.
fn on_boundary(b: &Body, fi: usize, p: &P3, tol: f64) -> bool {
    b.faces[fi].loops.iter().flatten().any(|c| {
        let e = &b.edges[c.edge];
        !e.degenerate && (e.point(e.curve.closest(p, e.t[0], e.t[1])) - p).norm() <= tol.max(e.tol)
    })
}

/// A B-spline strip's rail k from parameter `from` to `to` on the strip: its iso-line v = k,
/// u running with the edge's parameter (turned round when the edge was).
fn rail_pcurve(st: &Strip, k: usize, from: f64, to: f64) -> Option<Curve> {
    let StripKind::Nurbs = st.kind else { return None };
    let v = k as f64;
    if to >= from {
        return Some(Curve::Line { o: P3::new(0.0, v, 0.0), d: V3::new(1.0, 0.0, 0.0) });
    }
    let Curve::Nurbs(n) = &st.rails[k] else { return None };
    let (k0, k1) = (n.knots[0], n.knots[n.knots.len() - 1]);
    Some(Curve::Line { o: P3::new(k0 + k1, v, 0.0), d: V3::new(-1.0, 0.0, 0.0) })
}

/// The p-curve of `c` on [t0, t1] on `s` ([`super::pcurve::pcurve`]), checked along its
/// length: where the fit strays from the curve (a B-spline surface whose projection the fit's
/// samples missed), the curve's points projected densely in turn, interpolated at their
/// parameters.
fn face_pcurve(s: &Surface, c: &Curve, t0: f64, t1: f64, near: Option<P2>) -> Curve {
    let pc = super::pcurve::pcurve(s, c, t0, t1, near);
    const N: usize = 48;
    // The largest distance between the p-curve's image and the curve, and where.
    let worst = |pc: &Curve| -> (f64, f64) {
        (0..=N)
            .map(|i| {
                let t = t0 + (t1 - t0) * i as f64 / N as f64;
                let uv = pc.point(t);
                let d = (s.eval(uv.x, uv.y) - c.point(t)).norm();
                (if d.is_nan() { f64::INFINITY } else { d }, t)
            })
            .fold((0.0, t0), |a, b| if b.0 > a.0 { b } else { a })
    };
    let gap = |pc: &Curve| worst(pc).0;
    let size = c.point(t0).coords.abs().max().max(c.point(t1).coords.abs().max());
    let (g0, tw) = worst(&pc);
    if g0 <= tol::REL_POINT * (1.0 + size) {
        return pc;
    }
    // The curve itself that far off the surface there (a fitted cut on a B-spline strip): no
    // p-curve keeps closer, and the dense projection below can't help.
    let q = pc.point(tw);
    let (fu, fv) = s.project(&c.point(tw), Some((q.x, q.y)));
    if g0 <= 1.5 * (s.eval(fu, fv) - c.point(tw)).norm() + tol::REL_POINT * (1.0 + size) {
        return pc;
    }
    let m = 256;
    let periods = super::pcurve::closed_periods(s);
    let mut hint = { let q = pc.point(t0); Some((q.x, q.y)) };
    let mut ts = Vec::with_capacity(m + 1);
    let mut pts = Vec::with_capacity(m + 1);
    for i in 0..=m {
        let t = t0 + (t1 - t0) * i as f64 / m as f64;
        let q = super::pcurve::project_along(s, &c.point(t), hint, periods);
        hint = Some((q.x, q.y));
        ts.push(t);
        pts.push(P3::new(q.x, q.y, 0.0));
    }
    let dense = super::pcurve::shift_into(s, Curve::nurbs(NurbsCurve::interpolate(&pts, &ts, 3, None, None)), 0.5 * (t0 + t1), near);
    let g1 = gap(&dense);
    debug(|| format!("  a p-curve {g0:.3e} off its curve, {g1:.3e} projected densely"));
    if g1 < g0 { dense } else { pc }
}

/// A p-curve the strip gives for curve `c` on [t0, t1] on `s`, brought into the face's chart
/// (`near`): kept where it stays as close to the curve as a fitted one would (the curve's own
/// fit), else `None` (fitted instead).
fn given_pcurve(s: &Surface, c: &Curve, t0: f64, t1: f64, pc: Curve, near: Option<P2>) -> Option<Curve> {
    let pc = super::pcurve::shift_into(s, pc, 0.5 * (t0 + t1), near);
    const N: usize = 32;
    let size = c.point(t0).coords.abs().max().max(c.point(t1).coords.abs().max());
    let gap = (0..=N)
        .map(|i| {
            let t = t0 + (t1 - t0) * i as f64 / N as f64;
            let uv = pc.point(t);
            let d = (s.eval(uv.x, uv.y) - c.point(t)).norm();
            if d.is_nan() { f64::INFINITY } else { d }
        })
        .fold(0.0, f64::max);
    (gap <= 1e-5 * (1.0 + size)).then_some(pc)
}

/// The arcs of a face's loops (for [`split_face`]).
fn face_arcs(b: &Body, f: &Face) -> Vec<Vec<Arc>> {
    let periods = [f.surf.u_periodic().then_some(TAU), f.surf.v_periodic().then_some(TAU)];
    f.loops
        .iter()
        .map(|l| {
            let mut arcs: Vec<Arc> = l
                .iter()
                .map(|c| {
                    let e = &b.edges[c.edge];
                    let v = if c.fwd { e.v } else { [e.v[1], e.v[0]] };
                    Arc { edge: c.edge, fwd: c.fwd, pc: c.pc.clone(), t: e.t, v, cut: false, deg: e.degenerate }
                })
                .collect();
            if periods != [None, None] {
                one_chart_loop(&mut arcs, periods);
            }
            arcs
        })
        .collect()
}

/// A loop's p-curves on a periodic surface made whole in one chart: each, round the loop, moved
/// by whole periods to go on from where the one before it ends (a read face may have a coedge,
/// such as a seam's second, a period away: valid, but the split needs its loops whole).
fn one_chart_loop(arcs: &mut [Arc], periods: [Option<f64>; 2]) {
    let shift = |d: f64, per: Option<f64>| per.map_or(0.0, |p| (d / p).round() * p);
    let n = arcs.len();
    // Whole already (each coedge on from the one before, in the order stored, up to the
    // closing one): as it is.
    let tol = tol::REL_POINT * (1.0 + uv_size(arcs));
    if (0..n.saturating_sub(1)).all(|i| arcs[i].v[1] == arcs[i + 1].v[0] && (arcs[i].uv_at(1.0) - arcs[i + 1].uv_at(0.0)).norm() <= tol) {
        return;
    }
    let mut done = vec![false; n];
    let mut cur = 0;
    done[0] = true;
    for _ in 1..n {
        let (v, end) = (arcs[cur].v[1], arcs[cur].uv_at(1.0));
        // The next along (the loop's coedges may be stored out of order): the nearest start
        // at the vertex, a seam's own other coedge only when nothing else starts there, and of
        // two starting at the same point (a vertex the loop passes twice) the one turning most
        // to the left (the face's material is on the left).
        let din = end - arcs[cur].uv_at(1.0 - 1e-4);
        let key = |k: usize| {
            let s = arcs[k].uv_at(0.0);
            let (du, dv) = (end.x - s.x, end.y - s.y);
            let d = (du - shift(du, periods[0])).hypot(dv - shift(dv, periods[1]));
            let dep = arcs[k].uv_at(1e-4) - s;
            (arcs[k].edge == arcs[cur].edge, (d / tol::REL_POINT).round(), -(din.x * dep.y - din.y * dep.x).atan2(din.dot(&dep)))
        };
        let Some(next) = (0..n).filter(|&i| !done[i] && arcs[i].v[0] == v).min_by(|&i, &j| {
            let (a, c) = (key(i), key(j));
            a.0.cmp(&c.0).then(a.1.total_cmp(&c.1)).then(a.2.total_cmp(&c.2))
        }) else {
            return;
        };
        let s = arcs[next].uv_at(0.0);
        let (du, dv) = (shift(end.x - s.x, periods[0]), shift(end.y - s.y, periods[1]));
        if du != 0.0 || dv != 0.0 {
            arcs[next].pc = arcs[next].pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
        }
        done[next] = true;
        cur = next;
    }
}

/// Splits face `f` by the new edges `cuts` (p-curves by projection, near the face's own
/// chart) into faces.
fn split(b: &Body, f: &Face, cuts: &[usize], given: &dyn Fn(usize) -> Option<Curve>) -> Vec<Face> {
    let cut_arcs: Vec<Arc> = cuts
        .iter()
        .map(|&e| {
            let ed = &b.edges[e];
            // In the chart of its middle's nearest boundary point; but a cut ending on the face's
            // seam (split there) in the chart of its other end (a seam has two).
            let on_seam = |v: usize| f.loops.iter().flatten().any(|c| (b.co_start(c) == v || b.co_end(c) == v) && f.loops.iter().flatten().filter(|x| x.edge == c.edge).count() == 2);
            let (s0, s1) = (on_seam(ed.v[0]), on_seam(ed.v[1]));
            let at = match (s0, s1) {
                (true, false) => ed.point(ed.t[1]),
                (false, true) => ed.point(ed.t[0]),
                _ => ed.point(ed.mid_t()),
            };
            // With one end on the seam, where its middle lies inside the face (its other end's
            // nearest boundary point may be on the seam too, in either chart).
            let inside = if s0 != s1 { chart_inside(b, f, &ed.point(ed.mid_t())) } else { None };
            let near = inside.or_else(|| near_chart(b, f, &at));
            let pc = match given(e).and_then(|pc| given_pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], pc, near)) {
                Some(pc) => pc,
                None => face_pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], near),
            };
            Arc { edge: e, fwd: true, pc, t: ed.t, v: ed.v, cut: true, deg: false }
        })
        .collect();
    let boundary = face_arcs(b, f);
    let mut cut_arcs = cut_arcs;
    one_chart_cuts(&f.surf, &boundary, &mut cut_arcs);
    let res = split_face(&f.surf, !f.sense, &boundary, &cut_arcs);
    res.faces
        .into_iter()
        .map(|loops| Face {
            surf: f.surf.clone(),
            sense: f.sense,
            loops: loops.into_iter().map(|l| l.into_iter().map(|a| Coedge { edge: a.edge, fwd: a.fwd, pc: a.pc }).collect()).collect(),
        })
        .collect()
}

/// The (u, v) of point `p` (on face `f`'s surface) in the face's chart: of its turns round a
/// periodic surface, the one inside the face; `None` where none is.
fn chart_inside(b: &Body, f: &Face, p: &P3) -> Option<P2> {
    let near = near_chart(b, f, p);
    let (u, v) = f.surf.project(p, near.map(|q| (q.x, q.y)));
    let polys = super::classify::face_polygons(b, f);
    super::classify::periodic_candidates(&f.surf, u, v, &polys).into_iter().find(|&uv| super::classify::in_face(&polys, &f.surf, uv, tol::ROUNDOFF) == super::classify::InFace::Inside)
}

/// The size of arcs' p-curves' box in (u, v).
fn uv_size(arcs: &[Arc]) -> f64 {
    let ps: Vec<P2> = arcs.iter().flat_map(|a| [a.uv_at(0.0), a.uv_at(0.5), a.uv_at(1.0)]).collect();
    let span = |f: &dyn Fn(&P2) -> f64| ps.iter().map(f).fold(f64::NEG_INFINITY, f64::max) - ps.iter().map(f).fold(f64::INFINITY, f64::min);
    if ps.is_empty() { 0.0 } else { span(&|p| p.x).hypot(span(&|p| p.y)) }
}

/// Cuts meeting end to end inside a face of a periodic surface (a closed rail in pieces) made to
/// run on from each other in one chart: from each cut reaching the boundary (or the first), the
/// next across each vertex only cuts share moved by whole periods to start where the one before
/// ends (each cut's own projection may have put it a period away).
fn one_chart_cuts(surf: &Surface, boundary: &[Vec<Arc>], cuts: &mut [Arc]) {
    let periods = [surf.u_periodic().then_some(TAU), surf.v_periodic().then_some(TAU)];
    if periods == [None, None] {
        return;
    }
    let shift = |d: f64, per: Option<f64>| per.map_or(0.0, |p| (d / p).round() * p);
    let on_boundary = |v: usize| boundary.iter().flatten().any(|a| a.v.contains(&v));
    let n = cuts.len();
    let mut done = vec![false; n];
    let mut order: Vec<usize> = (0..n).filter(|&i| cuts[i].v.iter().any(|&v| on_boundary(v))).collect();
    order.extend((0..n).filter(|&i| !cuts[i].v.iter().any(|&v| on_boundary(v))));
    for root in order {
        if done[root] {
            continue;
        }
        done[root] = true;
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            for (end, v) in [(0usize, cuts[i].v[0]), (1, cuts[i].v[1])] {
                if on_boundary(v) {
                    continue;
                }
                let at = cuts[i].uv_at(end as f64);
                // The other cut at that vertex, if only two meet there.
                let others: Vec<usize> = (0..n).filter(|&j| j != i && cuts[j].v.contains(&v)).collect();
                let [j] = others[..] else { continue };
                if done[j] {
                    continue;
                }
                let q = cuts[j].uv_at(if cuts[j].v[0] == v { 0.0 } else { 1.0 });
                let (du, dv) = (shift(at.x - q.x, periods[0]), shift(at.y - q.y, periods[1]));
                if du != 0.0 || dv != 0.0 {
                    cuts[j].pc = cuts[j].pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                }
                done[j] = true;
                stack.push(j);
            }
        }
    }
}

/// The (u, v) of the face's boundary point nearest `p`: where a curve through `p` sits in the
/// face's chart of a periodic surface.
fn near_chart(b: &Body, f: &Face, p: &P3) -> Option<P2> {
    let mut best: Option<(f64, P2)> = None;
    for c in f.loops.iter().flatten() {
        let e = &b.edges[c.edge];
        for i in 0..=8 {
            let t = e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0;
            let d = (e.point(t) - p).norm();
            if best.is_none_or(|x| d < x.0) {
                let q = c.pc.point(t);
                best = Some((d, P2::new(q.x, q.y)));
            }
        }
    }
    best.map(|x| x.1)
}

/// Shifts a loop's p-curves by whole periods so each starts where the one before ends
/// (`u_period` for a closed B-spline strip, whose u isn't periodic as a surface).
fn align(surf: &Surface, b: &Body, l: &mut [Coedge], u_period: Option<f64>) {
    let mut at: Option<P2> = None;
    let pu = if surf.u_periodic() { Some(TAU) } else { u_period };
    for c in l.iter_mut() {
        let e = &b.edges[c.edge];
        let (t0, t1) = super::topo::co_range(e, c.fwd);
        if let Some(a) = at {
            let q = c.pc.point(t0);
            let du = pu.map_or(0.0, |p| ((a.x - q.x) / p).round() * p);
            let dv = if surf.v_periodic() { ((a.y - q.y) / TAU).round() * TAU } else { 0.0 };
            if du != 0.0 || dv != 0.0 {
                c.pc = c.pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
            }
        }
        let q = c.pc.point(t1);
        at = Some(P2::new(q.x, q.y));
    }
}

/// The curve where the fillet surface meets an end face's surface between points `a` and
/// `bp`, the way round nearer `towards` (exact, for surfaces whose intersection has a closed
/// form): (curve, t at a, t at b).
fn exact_end_curve(s: &Surface, g: &Surface, a: &P3, bp: &P3, towards: &P3) -> Option<(Curve, f64, f64)> {
    let analytic = |s: &Surface| matches!(s, Surface::Plane { .. } | Surface::Cylinder { .. } | Surface::Cone { .. } | Surface::Sphere { .. } | Surface::Torus { .. });
    if !analytic(s) || !analytic(g) {
        return None;
    }
    let mut bb = Bbox::empty();
    bb.add(a);
    bb.add(bp);
    bb.add(towards);
    let bb = bb.grow(bb.diagonal() + 1.0);
    let box_of = |s: &Surface| {
        let ((u0, u1), (v0, v1)) = s.param_box();
        ((u0, u1), (v0.max(-1e4), v1.min(1e4)))
    };
    let cs = super::ssi::surface_surface(s, box_of(s), g, box_of(g), &bb);
    let tol = tol::REL_POINT * (1.0 + bb.diagonal());
    for ic in cs {
        // Only closed-form curves (a fitted one is no better than the marched end).
        if matches!(ic.c, Curve::Nurbs(_)) {
            continue;
        }
        let (lo, hi) = if ic.closed { (ic.t0 - TAU, ic.t1 + TAU) } else { (ic.t0, ic.t1) };
        let ta = ic.c.closest(a, lo, hi);
        let tb = ic.c.closest(bp, lo, hi);
        if (ic.c.point(ta) - a).norm() > tol || (ic.c.point(tb) - bp).norm() > tol {
            continue;
        }
        if ic.closed || matches!(ic.c, Curve::Circle { .. } | Curve::Ellipse { .. }) {
            // Of the two ways round (within a turn: the closest points are found over three),
            // the one whose middle is nearer the corner.
            let tb = ta + wrap(tb - ta, 0.0, TAU);
            let tb2 = tb - TAU;
            let mid = |x: f64, y: f64| ic.c.point(0.5 * (x + y));
            let d1 = (mid(ta, tb) - towards).norm();
            let d2 = (mid(ta, tb2) - towards).norm();
            return Some((ic.c, ta, if d1 <= d2 { tb } else { tb2 }));
        }
        return Some((ic.c, ta, tb));
    }
    None
}

/// Adds the edge along `c` from `t0` to `t1` between vertices `v0` and `v1` (turned round when
/// t1 < t0, so its range increases).
/// The plane through `o` square to `n`, as a face (for marching a strip's cut by it).
fn square_plane(o: &P3, n: &V3) -> Face {
    let x = any_perp(n);
    Face { surf: Surface::Plane { f: Frame { o: *o, x, y: n.cross(&x), z: *n } }, sense: true, loops: Vec::new() }
}

/// The edge from vertex `v0` (at `a`) to `v1` (at `z`) along face `f` in the plane through `o`
/// square to `n`: a line where the face is flat there, else a curve through points of the
/// segment moved onto the face, then along it onto the plane (Newton in the face's (u, v)).
#[allow(clippy::too_many_arguments)]
fn section_cut(f: &Face, a: &P3, z: &P3, o: &P3, n: &V3, tol: f64, v0: usize, v1: usize, nb: &mut Body) -> usize {
    let m = 24;
    let mut hint: Option<P2> = None;
    let sc = 1.0 + o.coords.abs().max() + (z - a).norm();
    let pts: Vec<P3> = (0..=m)
        .map(|i| {
            let s = i as f64 / m as f64;
            let x = a + (z - a) * s;
            if i == 0 || i == m {
                return x;
            }
            let mut uv = geom::foot(f, &x, hint);
            for _ in 0..30 {
                let (p, su, sv) = f.surf.d1(uv.x, uv.y);
                let g = (p - o).dot(n);
                let grad = V2::new(su.dot(n), sv.dot(n));
                if g.abs() < 1e-13 * sc || grad.norm_squared() < tol::TINY {
                    break;
                }
                uv -= grad * (g / grad.norm_squared());
            }
            hint = Some(uv);
            f.surf.eval(uv.x, uv.y)
        })
        .collect();
    let d = (z - a).normalize();
    let straight = pts.iter().all(|p| ((p - a) - d * (p - a).dot(&d)).norm() < tol);
    if straight {
        let (c, len) = Curve::line(*a, *z);
        return add_edge(nb, c, 0.0, len, v0, v1);
    }
    let (c, c0, c1) = fit_cut(&pts, Some(f), [None, None]);
    add_edge(nb, c, c0, c1, v0, v1)
}

fn add_edge(b: &mut Body, c: Curve, t0: f64, t1: f64, v0: usize, v1: usize) -> usize {
    if t1 >= t0 {
        b.add_edge(c, t0, t1, v0, v1)
    } else {
        let (rc, a, z) = c.reversed(t0, t1);
        let (a, z) = if a <= z { (a, z) } else { (z, a) };
        b.add_edge(rc, a, z, v0, v1)
    }
}

/// End curve `ee` (on face `g`'s surface) split where it crosses the edge of g at vertex `w`
/// other than `y`: that edge split there too. The part from vertex `from` (an end of ee) to the
/// crossing, the rest, and the crossing's vertex; `None` where it doesn't cross.
#[allow(clippy::too_many_arguments)]
fn end_over_edge(b: &Body, nb: &mut Body, g: usize, ee: usize, from: usize, w: usize, y: usize, tol: f64) -> Option<(usize, usize, usize)> {
    let face = &b.faces[g];
    let x = face.loops.iter().flatten().map(|c| c.edge).find(|&x| x != y && !b.edges[x].degenerate && b.edges[x].v.contains(&w))?;
    let side = edge_side(face, &b.edges[x]);
    let ed = nb.edges[ee].clone();
    let at = |s: f64| ed.t[0] + (ed.t[1] - ed.t[0]) * s;
    // The first sign change along the curve, then bisection.
    let m = 64;
    let mut prev = side(&ed.point(at(0.0)));
    let i = (1..=m).find(|&i| {
        let f = side(&ed.point(at(i as f64 / m as f64)));
        let flips = (f < 0.0) != (prev < 0.0);
        prev = f;
        flips
    })?;
    let (mut lo, mut hi) = ((i - 1) as f64 / m as f64, i as f64 / m as f64);
    let f_lo = side(&ed.point(at(lo)));
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if (side(&ed.point(at(mid))) < 0.0) == (f_lo < 0.0) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let tq = at(0.5 * (lo + hi));
    // On the edge itself, not its extension.
    let xe = &b.edges[x];
    let p = ed.point(tq);
    let tx = xe.curve.closest(&p, xe.t[0], xe.t[1]);
    if (xe.point(tx) - p).norm() > tol {
        return None;
    }
    let q = vertex_on_boundary(nb, g, &p, tol)?;
    if q == ed.v[0] || q == ed.v[1] {
        return None;
    }
    nb.edges[ee].t = [ed.t[0], tq];
    nb.edges[ee].v[1] = q;
    let rest = nb.add_edge(ed.curve, tq, ed.t[1], q, ed.v[1]);
    Some(if ed.v[0] == from { (ee, rest, q) } else { (rest, ee, q) })
}

/// A strip's end at a set-back corner: its section's edge, its rails' end vertices and its
/// faces (in its order), and the section's ball.
#[derive(Clone, Copy)]
struct HoleEnd {
    section: usize,
    rv: [usize; 2],
    faces: [usize; 2],
    ball: Ball,
}

/// The edge from vertex `v0` (at `a`) to `v1` (at `z`) across face `f` ([`face_curve`]).
fn face_cut(f: &Face, a: &P3, z: &P3, tol: f64, v0: usize, v1: usize, nb: &mut Body) -> usize {
    let (c, c0, c1) = face_curve(f, a, z, tol);
    add_edge(nb, c, c0, c1, v0, v1)
}

/// The curve from `a` to `z` across face `f`: a line where the face is flat there, else a curve
/// through points of the segment moved onto the face; with its parameters at a and z.
fn face_curve(f: &Face, a: &P3, z: &P3, tol: f64) -> (Curve, f64, f64) {
    let m = 24;
    let mut hint: Option<P2> = None;
    let pts: Vec<P3> = (0..=m)
        .map(|i| {
            let x = a + (z - a) * (i as f64 / m as f64);
            if i == 0 || i == m {
                return x;
            }
            let uv = geom::foot(f, &x, hint);
            hint = Some(uv);
            f.surf.eval(uv.x, uv.y)
        })
        .collect();
    let d = (z - a).normalize();
    if pts.iter().all(|p| ((p - a) - d * (p - a).dot(&d)).norm() < tol) {
        let (c, len) = Curve::line(*a, *z);
        return (c, 0.0, len);
    }
    fit_cut(&pts, Some(f), [None, None])
}

/// The faces round a set-back corner cut across between its setback points, and the patch
/// filling the hole.
fn hole_face(b: &Body, nb: &mut Body, h: &Hole, vtol: f64, out: &mut Pieces, input: BodyId) -> Result<()> {
    let v = h.v;
    let m = h.fan.len();
    // The setback point on each edge left as it is.
    let mut points: HashMap<usize, usize> = HashMap::new();
    for &(e, f) in &h.fan {
        if out.hole_ends.contains_key(&(v, e)) {
            continue;
        }
        let t = h.d.get(&e).and_then(|&d| param_at(b, e, v, d)).ok_or_else(|| failed("corner", "a fillet corner's setback is past an edge's end"))?;
        let x = vertex_on_boundary(nb, f, &b.edges[e].point(t), vtol).ok_or_else(|| failed("corner", "a fillet corner's setback point isn't on its edge"))?;
        points.insert(e, x);
    }
    // Where face f meets the hole beside edge e.
    let at = |e: usize, f: usize| -> Result<usize> {
        match out.hole_ends.get(&(v, e)) {
            Some(x) if x.faces.contains(&f) => Ok(x.rv[if x.faces[0] == f { 0 } else { 1 }]),
            Some(_) => Err(failed("corner", "a set-back corner's strip isn't on its face")),
            None => points.get(&e).copied().ok_or_else(|| failed("corner", "a set-back corner's edge has no setback point")),
        }
    };
    // The hole's rim runs against the faces' loops: on each face from the side of the edge
    // that leaves the vertex in the face's loop to that of the edge that comes into it.
    let (l0, f0) = h.fan[0];
    let enters = b.faces[f0].loops.iter().flatten().any(|c| c.edge == l0 && b.co_end(c) == v);
    let order: Vec<(usize, usize, usize)> = if enters {
        (0..m).rev().map(|i| (h.fan[(i + 1) % m].0, h.fan[i].1, h.fan[i].0)).collect()
    } else {
        (0..m).map(|i| (h.fan[i].0, h.fan[i].1, h.fan[(i + 1) % m].0)).collect()
    };
    // The rim: each face's cut, then the section of the strip on the edge after it.
    enum Side {
        Face(usize),
        Strip(Ball),
    }
    let mut rim: Vec<(usize, bool, Side)> = Vec::new();
    for (i, &(before, f, after)) in order.iter().enumerate() {
        let (a, z) = (at(before, f)?, at(after, f)?);
        if a == z {
            return Err(bad_geometry("a set-back corner's cut across a face degenerates"));
        }
        let (pa, pz) = (nb.verts[a].p, nb.verts[z].p);
        let cut = face_cut(&b.faces[f], &pa, &pz, vtol, a, z, nb);
        if !inside_face(b, f, &nb.edges[cut]) {
            return Err(unsupported("a set-back fillet corner whose cut leaves its face"));
        }
        out.cuts.entry(f).or_default().push(cut);
        rim.push((cut, true, Side::Face(f)));
        if let Some(x) = out.hole_ends.get(&(v, after)).copied() {
            let next = order[(i + 1) % m].1;
            let from = at(after, f)?;
            let to = at(after, next)?;
            let ed = nb.edges[x.section].v;
            if !(ed == [from, to] || ed == [to, from]) {
                return Err(failed("corner", "a set-back corner's section doesn't join its faces' cuts"));
            }
            rim.push((x.section, ed[0] == from, Side::Strip(x.ball)));
        }
    }
    let vp = b.verts[v].p;
    let normal = |i: usize, p: &P3| -> V3 {
        match &rim[i].2 {
            Side::Face(f) => geom::contact(&b.faces[*f], geom::foot(&b.faces[*f], p, None)).n,
            // Along the fillet's surface (a convex one faces away from its ball's centre, a
            // concave one towards it: either way, as the rim leans into the hole).
            Side::Strip(ball) => (p - ball.c).normalize(),
        }
    };
    let edges: Vec<(usize, bool)> = rim.iter().map(|x| (x.0, x.1)).collect();
    let (surf, pole, seam, pcs) = disc(nb, &edges, &normal, vp)?;
    let first = if edges[0].1 { nb.edges[edges[0].0].v[0] } else { nb.edges[edges[0].0].v[1] };
    let pv = nb.add_vertex(pole);
    let se = nb.add_edge(seam, 0.0, 1.0, pv, first);
    let deg = nb.add_degenerate(pole, 0.0, 1.0, pv);
    let mut pcurves: Vec<(usize, Curve)> = edges.iter().map(|x| x.0).zip(pcs).collect();
    // The seam at u = 0 (and, a period on, u = 1); the pole along v = 0, back from u = 1 to 0
    // (as given from u = 0 to −1: the loop takes it a period on).
    pcurves.push((se, Curve::Line { o: P3::origin(), d: V3::new(0.0, 1.0, 0.0) }));
    pcurves.push((deg, Curve::Line { o: P3::origin(), d: V3::new(-1.0, 0.0, 0.0) }));
    let mut fe: Vec<usize> = edges.iter().map(|x| x.0).collect();
    fe.extend([se, se, deg]);
    let vi = super::blend::vertex_index(b, v);
    out.new_faces.push(NewFace { surf, edges: fe, src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: Some(1.0), beside: f0, pcurves });
    Ok(())
}

/// A patch filling the hole whose rim is the loop of edges `rim` (each with its direction), its
/// neighbour's outward normal at a point of rim edge i `normal(i, p)`: a B-spline disc whose u
/// runs round the rim (by length, from the first edge's start), v from its pole (domed towards
/// `towards`) to the rim, its next row leaning along the neighbours' tangent planes (as the
/// smooth corner's, [`super::blend_corner`]); C0 where the rim's edges meet. Returns the
/// surface, its pole, its seam (the iso-line u = 0 from the pole, on [0, 1]) and the rim edges'
/// p-curves.
fn disc(nb: &Body, rim: &[(usize, bool)], normal: &dyn Fn(usize, &P3) -> V3, towards: P3) -> Result<(Surface, P3, Curve, Vec<Curve>)> {
    let bad = || failed("corner", "a set-back corner's patch can't be built");
    let m = 16;
    // Each rim edge's samples along the rim: edge parameters, points, rim tangents.
    let mut ts: Vec<Vec<f64>> = Vec::new();
    let mut pts: Vec<Vec<P3>> = Vec::new();
    let mut lean: Vec<Vec<V3>> = Vec::new();
    for (i, &(e, fwd)) in rim.iter().enumerate() {
        let ed = &nb.edges[e];
        let (a, z) = if fwd { (ed.t[0], ed.t[1]) } else { (ed.t[1], ed.t[0]) };
        let t: Vec<f64> = (0..=m).map(|s| a + (z - a) * s as f64 / m as f64).collect();
        let mut p: Vec<P3> = t.iter().map(|&x| ed.point(x)).collect();
        let (va, vz) = if fwd { (ed.v[0], ed.v[1]) } else { (ed.v[1], ed.v[0]) };
        p[0] = nb.verts[va].p;
        p[m] = nb.verts[vz].p;
        let dirs: Vec<V3> = t.iter().zip(&p).map(|(&x, q)| normal(i, q).cross(&(ed.curve.tangent(x).normalize() * if fwd { 1.0 } else { -1.0 }))).collect();
        ts.push(t);
        pts.push(p);
        lean.push(dirs);
    }
    let n = rim.len();
    let all: Vec<P3> = pts.iter().flat_map(|p| p[..m].iter().copied()).collect();
    let g = P3::from(all.iter().fold(V3::zeros(), |a, p| a + p.coords) / all.len() as f64);
    let to_v = towards - g;
    let c = g + to_v * 0.45;
    let axis = to_v.try_normalize(tol::DEGENERATE).ok_or_else(bad)?;
    // Leaning into the hole; where two rim edges meet, the mean of their directions.
    for x in lean.iter_mut().flatten() {
        *x = x.try_normalize(tol::DEGENERATE).unwrap_or(V3::zeros());
    }
    for i in 0..n {
        for s in 0..=m {
            let p = pts[i][s];
            if lean[i][s].dot(&(c - p)) < 0.0 {
                lean[i][s] = -lean[i][s];
            }
        }
    }
    for i in 0..n {
        let j = (i + 1) % n;
        let mean = (lean[i][m] + lean[j][0]).try_normalize(tol::DEGENERATE).unwrap_or(lean[j][0]);
        lean[i][m] = mean;
        lean[j][0] = mean;
    }
    // The rim's length parameter at each sample.
    let mut u: Vec<Vec<f64>> = Vec::new();
    let mut acc = 0.0;
    for p in &pts {
        let mut us = vec![acc];
        for s in 1..=m {
            acc += (p[s] - p[s - 1]).norm();
            us.push(acc);
        }
        u.push(us);
    }
    if !(acc > 0.0) {
        return Err(bad());
    }
    for x in u.iter_mut().flatten() {
        *x /= acc;
    }
    // The rim's and the leaning row's pieces, joined C0.
    let row = |of: &dyn Fn(usize, usize) -> P3| -> Option<NurbsCurve> {
        let mut knots: Vec<f64> = Vec::new();
        let mut cps: Vec<super::nurbs::H> = Vec::new();
        for i in 0..n {
            let ps: Vec<P3> = (0..=m).map(|s| of(i, s)).collect();
            let piece = NurbsCurve::interpolate(&ps, &u[i], 3, None, None);
            if piece.degree != 3 || piece.cps.len() != m + 1 {
                return None;
            }
            let k = &piece.knots;
            let from = if i == 0 { 0 } else { 4 };
            let to = if i + 1 == n { k.len() } else { k.len() - 1 };
            knots.extend_from_slice(&k[from..to]);
            cps.extend_from_slice(&piece.cps[if i == 0 { 0 } else { 1 }..]);
        }
        Some(NurbsCurve { degree: 3, knots, cps })
    };
    let rim_row = row(&|i, s| pts[i][s]).ok_or_else(bad)?;
    let lean_row = row(&|i, s| pts[i][s] + lean[i][s] * (0.33 * (pts[i][s] - c).norm())).ok_or_else(bad)?;
    if rim_row.knots != lean_row.knots {
        return Err(bad());
    }
    let perp = |p: &P3| {
        let w = p - c;
        c + (w - axis * w.dot(&axis)) * 0.4
    };
    let h = |p: P3| super::nurbs::H::new(p.x, p.y, p.z, 1.0);
    let cps: Vec<Vec<super::nurbs::H>> = rim_row
        .cps
        .iter()
        .zip(&lean_row.cps)
        .map(|(r, l)| {
            let (r, l) = (super::nurbs::dehom(r), super::nurbs::dehom(l));
            vec![h(c), h(perp(&l)), h(l), h(r)]
        })
        .collect();
    let bez = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    let seam = Curve::Nurbs(std::sync::Arc::new(NurbsCurve { degree: 3, knots: bez.clone(), cps: cps[0].clone() }));
    let surf = super::nurbs::NurbsSurface::new(3, 3, rim_row.knots.clone(), bez, cps).map_err(|_| bad())?;
    // The rim's p-curves: (u(t), 1).
    let mut pcs = Vec::new();
    for i in 0..n {
        let mut t = ts[i].clone();
        let mut q: Vec<P3> = u[i].iter().map(|&x| P3::new(x, 1.0, 0.0)).collect();
        if t[0] > t[m] {
            t.reverse();
            q.reverse();
        }
        pcs.push(Curve::Nurbs(std::sync::Arc::new(NurbsCurve::interpolate(&q, &t, 3, None, None))));
    }
    Ok((Surface::Nurbs(std::sync::Arc::new(surf)), c, seam, pcs))
}

/// A face to add: its surface, its boundary edges (one entry per use) and its source.
#[derive(Clone)]
struct NewFace {
    surf: Surface,
    edges: Vec<usize>,
    src: FaceSrc,
    /// A closed B-spline band's period in u.
    u_period: Option<f64>,
    /// A face of the input it lies beside (its shell is the new face's).
    beside: usize,
    /// Edges whose p-curves are given (a degenerate edge's), turned round to continue the loop.
    pcurves: Vec<(usize, Curve)>,
}

/// Chains the edges of a new face into one loop: an edge with a known direction (`dir`) is
/// used so, the others the way that continues the loop.
fn chain(b: &Body, edges: &[(usize, Option<bool>)]) -> Result<Vec<(usize, bool)>> {
    // Degenerate edges (poles) go in where the loop reaches their vertex.
    type Use = (usize, Option<bool>);
    let (deg, rest): (Vec<Use>, Vec<Use>) = edges.iter().partition(|x| b.edges[x.0].degenerate);
    if !deg.is_empty() {
        let mut lp = chain(b, &rest)?;
        for (e, dir) in deg {
            let v = b.edges[e].v[0];
            let at = lp.iter().position(|&(x, fwd)| (if fwd { b.edges[x].v[1] } else { b.edges[x].v[0] }) == v).ok_or_else(|| failed("trimming", "a fillet face's pole isn't on its loop"))?;
            lp.insert(at + 1, (e, dir.unwrap_or(true)));
        }
        return Ok(lp);
    }
    let n = edges.len();
    let start = edges.iter().position(|x| x.1.is_some()).ok_or_else(|| failed("trimming", "a fillet face with no known edge direction"))?;
    let mut used = vec![false; n];
    used[start] = true;
    let ends = |e: usize, fwd: bool| -> (usize, usize) {
        let v = b.edges[e].v;
        if fwd { (v[0], v[1]) } else { (v[1], v[0]) }
    };
    let (e0, f0) = (edges[start].0, edges[start].1.unwrap_or(true));
    let (first, at) = ends(e0, f0);
    // Depth first: at each vertex the edges going on from it, those of known direction first;
    // back where a choice leads nowhere (a closed strip's rail running on round its seam's
    // vertex, where the seam must come first).
    let options = |at: usize, used: &[bool]| -> Vec<(usize, bool)> {
        let mut known = Vec::new();
        let mut guessed = Vec::new();
        for (i, &(e, dir)) in edges.iter().enumerate() {
            if used[i] {
                continue;
            }
            let opts: &[bool] = match dir {
                Some(true) => &[true],
                Some(false) => &[false],
                None => &[true, false],
            };
            for &fwd in opts {
                if ends(e, fwd).0 == at {
                    if dir.is_some() { known.push((i, fwd)) } else { guessed.push((i, fwd)) }
                }
            }
        }
        known.extend(guessed);
        known
    };
    let mut out: Vec<(usize, bool)> = vec![(e0, f0)];
    let mut path: Vec<usize> = vec![start];
    let mut stack: Vec<Vec<(usize, bool)>> = vec![options(at, &used)];
    let mut budget = 100_000;
    while let Some(choices) = stack.last_mut() {
        budget -= 1;
        if budget == 0 {
            break;
        }
        if out.len() == n {
            let last = out[n - 1];
            if ends(last.0, last.1).1 == first {
                return Ok(out);
            }
        }
        let Some((i, fwd)) = (if out.len() < n { choices.first().copied() } else { None }) else {
            // Back a step.
            stack.pop();
            if path.len() > 1 {
                let i = path.pop().unwrap_or(start);
                used[i] = false;
                out.pop();
            }
            continue;
        };
        choices.remove(0);
        used[i] = true;
        path.push(i);
        out.push((edges[i].0, fwd));
        let at = ends(edges[i].0, fwd).1;
        stack.push(options(at, &used));
    }
    debug(|| format!("chain: no loop through {:?}", edges.iter().map(|&(e, d)| (e, d, b.edges[e].v)).collect::<Vec<_>>()));
    Err(failed("trimming", "a fillet face's edges don't close up"))
}

/// What the strips add to the body: cuts in faces, new faces and new-edge marks.
#[derive(Default, Clone)]
struct Pieces {
    cuts: HashMap<usize, Vec<usize>>,
    new_faces: Vec<NewFace>,
    section_edges: HashMap<SectionKey, usize>,
    /// (vertex, face) → the vertex of a section's contact there.
    contact_verts: HashMap<(usize, usize), usize>,
    /// End curves outside their end faces (a fillet adding material).
    patches: Vec<Patch>,
    /// At a patched corner (vertex, strip): the halves of the strip's curve across and the
    /// vertex between them.
    patch_halves: HashMap<(usize, usize), (usize, usize, usize)>,
    /// At a set-back corner (vertex, edge): the strip's section there and its rails' end vertices
    /// (on its faces).
    hole_ends: HashMap<(usize, usize), HoleEnd>,
    /// Faces a rail runs right along an edge of (the strip takes the whole face): they go
    /// whole, uncut.
    consumed: std::collections::HashSet<usize>,
    /// (edge, face) → the edge's p-curve there where the strip gives it (a B-spline strip's
    /// rail on its face, from the contacts' (u, v)).
    on_face: HashMap<(usize, usize), Curve>,
}

/// Traces the strip's curve on face `g`'s surface from `a` to `z` (both on it) in the strip's
/// (spine parameter, fraction across) plane: predictor steps along the curve, corrected by
/// Newton on the face's signed distance, shorter where it turns. Returns its points.
fn trace(b: &Body, ef: &EdgeFillet, g: &Face, a: &Mark, z: &Mark) -> Result<Vec<P3>> {
    Ok(trace_until(b, ef, g, a, z, None)?.0)
}

/// [`trace`], stopping where the curve reaches the surface of face `stop` (its signed
/// distance changes sign): the points so far and, if it stopped, where (as a mark).
fn trace_until(b: &Body, ef: &EdgeFillet, g: &Face, a: &Mark, z: &Mark, stop: Option<&Face>) -> Result<(Vec<P3>, Option<Mark>)> {
    match stop {
        Some(h) => {
            let sd = |p: &P3| -> f64 {
                let c = geom::contact(h, geom::foot(h, p, None));
                (p - c.p).dot(&c.n)
            };
            trace_to(b, ef, g, a, z, Some(&sd))
        }
        None => trace_to(b, ef, g, a, z, None),
    }
}

/// The side of edge `x` of face `g` a point on g's surface (near the edge) lies on: across the
/// edge, along the face's tangent plane (a stop for [`trace_to`] where the next face round a
/// corner lies on the same surface as g, the edge between them an iso-line of it).
fn edge_side<'a>(g: &'a Face, x: &'a super::topo::Edge) -> impl Fn(&P3) -> f64 + 'a {
    move |p: &P3| {
        let t = x.curve.closest(p, x.t[0] - 1e6, x.t[1] + 1e6);
        let (q, d, _) = x.curve.d2(t);
        let n = geom::contact(g, geom::foot(g, &q, None)).n;
        (p - q).dot(&n.cross(&d))
    }
}

/// [`trace`], stopping where `stop` changes sign: the points so far and, if it stopped, where.
fn trace_to(b: &Body, ef: &EdgeFillet, g: &Face, a: &Mark, z: &Mark, stop: Option<&dyn Fn(&P3) -> f64>) -> Result<(Vec<P3>, Option<Mark>)> {
    let lost = || failed("trimming", "the fillet's curve on a face can't be traced");
    let sd_stop = |p: &P3| -> f64 { stop.map_or(0.0, |f| f(p)) };
    let r = ef.r();
    let mut hint: Option<P2> = None;
    // The face's signed distance of the strip's point (t, w), from the ball near `bl`.
    let mut eval = |t: f64, w: f64, bl: &Ball, tb: f64| -> Option<(f64, Ball, P3)> {
        let x = ef.ball_from(b, bl, tb, t)?;
        let p = x.arc_point(w);
        let uv = geom::foot(g, &p, hint);
        hint = Some(uv);
        let c = geom::contact(g, uv);
        Some(((p - c.p).dot(&c.n), x, p))
    };
    // Scales making (t, w) steps lengths: the spine's speed and the section's arc length.
    let st = ef.spine.curve.d2(a.t).1.norm().max(tol::DEGENERATE);
    let sw = (a.ball.q[0].p - a.ball.c).angle(&(a.ball.q[1].p - a.ball.c)) * r;
    let total = (z.p - a.p).norm();
    // Steps short enough for a cubic through the points to keep to the curve.
    let hmax = (total / 48.0).min(r / 6.0).max(tol::ROUNDOFF);
    let (mut t, mut w, mut bl) = (a.t, a.w, a.ball);
    let mut pts = vec![a.p];
    let mut dir: Option<(f64, f64)> = None;
    let mut h = hmax;
    for _ in 0..4000 {
        // The gradient (in lengths) by central differences.
        let d = tol::DIFF_STEP * hmax;
        let (fp, _, _) = eval(t + d / st, w, &bl, t).ok_or_else(lost)?;
        let (fm, _, _) = eval(t - d / st, w, &bl, t).ok_or_else(lost)?;
        let (fwp, _, _) = eval(t, w + d / sw, &bl, t).ok_or_else(lost)?;
        let (fwm, _, _) = eval(t, w - d / sw, &bl, t).ok_or_else(lost)?;
        let g = ((fp - fm) / (2.0 * d), (fwp - fwm) / (2.0 * d));
        let gn = (g.0 * g.0 + g.1 * g.1).sqrt();
        if gn < tol::DEGENERATE {
            return Err(lost());
        }
        let mut tan = (-g.1 / gn, g.0 / gn);
        let to_end = ((z.t - t) * st, (z.w - w) * sw);
        let flip = match dir {
            Some(dp) => tan.0 * dp.0 + tan.1 * dp.1 < 0.0,
            // From a rail the curve goes onto the strip (unless it runs along the rail).
            None if a.w <= tol::ROUNDOFF && tan.1.abs() > 1e-3 => tan.1 < 0.0,
            None if a.w >= 1.0 - tol::ROUNDOFF && tan.1.abs() > 1e-3 => tan.1 > 0.0,
            None => tan.0 * to_end.0 + tan.1 * to_end.1 < 0.0,
        };
        if flip {
            tan = (-tan.0, -tan.1);
        }
        let dist_end = (to_end.0 * to_end.0 + to_end.1 * to_end.1).sqrt();
        if dist_end <= 1.5 * h.min(hmax) && pts.len() > 1 {
            pts.push(z.p);
            return Ok((pts, None));
        }
        // Predictor, then Newton along the gradient; halve the step when it doesn't converge
        // or turns too much.
        let mut accepted = None;
        while h > tol::REL * hmax {
            let (mut tn, mut wn) = (t + tan.0 * h / st, w + tan.1 * h / sw);
            let mut ok = None;
            let mut cur = bl;
            let mut tc = t;
            for _ in 0..8 {
                let (f0, x, p) = eval(tn, wn, &cur, tc).ok_or_else(lost)?;
                cur = x;
                tc = tn;
                if f0.abs() < 1e-10 * (1.0 + r + p.coords.abs().max()) {
                    ok = Some((tn, wn, x, p));
                    break;
                }
                let k = f0 / (gn * gn);
                tn -= k * g.0 / st;
                wn -= k * g.1 / sw;
            }
            match ok {
                Some(x) if dir.is_none_or(|dp| tan.0 * dp.0 + tan.1 * dp.1 > 0.996) || h < 1e-3 * hmax => {
                    accepted = Some(x);
                    break;
                }
                _ => h *= 0.5,
            }
        }
        let (tn, wn, x, p) = accepted.ok_or_else(lost)?;
        // Past the stop face's surface on the way over the strip's side (the cut meets that
        // face just short of the rail): the stop rules.
        let stops = stop.is_some() && pts.len() > 1 && (sd_stop(&pts[pts.len() - 1]) < 0.0) != (sd_stop(&p) < 0.0);
        if !stops && !(-tol::REL..=1.0 + tol::REL).contains(&wn) {
            // Over the strip's far side: the end when that's where the curve is bound (a cut
            // across the whole strip, to the other rail), else a cut this one can't be.
            let bound = if wn < 0.0 { 0.0 } else { 1.0 };
            if stop.is_none() && (z.w - bound).abs() < tol::ROUNDOFF_FINE && dist_end <= 4.0 * hmax && pts.len() > 1 {
                pts.push(z.p);
                return Ok((pts, None));
            }
            debug(|| format!("  trace off the strip: w {wn} at t {tn} after {} points, from (t {}, w {}) to (t {}, w {}), stop {}", pts.len(), a.t, a.w, z.t, z.w, stop.is_some()));
            return Err(unsupported(WHOLE_STRIP));
        }
        // Reached the stop face's surface: bisect along the step, back onto g each time.
        let (s0, s1) = (sd_stop(&pts[pts.len() - 1]), sd_stop(&p));
        if stop.is_some() && pts.len() > 1 && (s0 < 0.0) != (s1 < 0.0) {
            let (mut lo, mut hi) = ((t, w, bl), (tn, wn, x));
            let mut at = Mark { t: tn, w: wn, ball: x, p };
            for _ in 0..50 {
                let (tm, wm) = (0.5 * (lo.0 + hi.0), 0.5 * (lo.1 + hi.1));
                // Onto g along the gradient.
                let (mut tc, mut wc, mut cur, mut tb) = (tm, wm, lo.2, lo.0);
                let mut pc = p;
                for _ in 0..8 {
                    let (f0, xb, pp) = eval(tc, wc, &cur, tb).ok_or_else(lost)?;
                    (cur, tb, pc) = (xb, tc, pp);
                    if f0.abs() < 1e-11 * (1.0 + r + pp.coords.abs().max()) {
                        break;
                    }
                    let k = f0 / (gn * gn);
                    tc -= k * g.0 / st;
                    wc -= k * g.1 / sw;
                }
                at = Mark { t: tc, w: wc, ball: cur, p: pc };
                if (sd_stop(&pc) < 0.0) == (s0 < 0.0) {
                    lo = (tc, wc, cur);
                } else {
                    hi = (tc, wc, cur);
                }
            }
            pts.push(at.p);
            return Ok((pts, Some(at)));
        }
        dir = Some(tan);
        (t, w, bl) = (tn, wn, x);
        pts.push(p);
        h = (h * 1.5).min(hmax);
    }
    Err(lost())
}

/// Face `f` with its B-spline surface run on past its parameter box by a quarter of the box
/// in each open direction, where the surface is one polynomial span there (OCCT evaluates a
/// Bézier patch past its range alike); the same parameters inside the box. None for other
/// surfaces.
fn grown_face(f: &Face) -> Option<Face> {
    let Surface::Nurbs(n) = &f.surf else { return None };
    let ((u0, u1), (v0, v1)) = n.range();
    let periods = super::pcurve::closed_periods(&f.surf);
    let grow = |(a, z): (f64, f64), closed: bool| if closed { (a, z) } else { (a - 0.25 * (z - a), z + 0.25 * (z - a)) };
    let surf = super::face_build::extended_past(&f.surf, (grow((u0, u1), periods[0].is_some()), grow((v0, v1), periods[1].is_some())))?;
    Some(Face { surf, sense: f.sense, loops: Vec::new() })
}

/// The strip's cut by the faces beyond its face where rail k runs off it, from crossing `a`
/// (vertex) to crossing `z`: one face, or two meeting at an edge (round a corner of the face).
/// Inside those faces the cut splits them; outside (a fillet adding material) they grow, the
/// two then joined along their edge's extension.
#[allow(clippy::too_many_arguments)]
fn cliff(b: &Body, nb: &mut Body, ef: &EdgeFillet, (k, a, va): (usize, &Crossing, usize), (kz, z, vz): (usize, &Crossing, usize), fe: &mut Vec<usize>, out: &mut Pieces) -> Result<()> {
    let tol = tol::REL_POINT * (1.0 + ef.r());
    let (ma, mz) = (Mark::rail(a.t, a.ball, k), Mark::rail(z.t, z.ball, kz));
    let fit = |nb: &mut Body, pts: Vec<P3>, v0: usize, v1: usize, g: &Face, c0: Option<P3>, c1: Option<P3>| -> usize {
        let mut xs = pts;
        let last = xs.len() - 1;
        xs[0] = nb.verts[v0].p;
        xs[last] = nb.verts[v1].p;
        let (c, t0, t1) = fit_cut(&xs, Some(g), [c0, c1]);
        add_edge(nb, c, t0, t1, v0, v1)
    };
    if a.into == z.into {
        let g = a.into;
        // Traced on the face's surface run on past its box where it can be (the cut may lie
        // outside the face, which then grows to it: on the surface's own continuation, not
        // its box's side the projection would stop at).
        let grown = grown_face(&b.faces[g]);
        let face = grown.as_ref().unwrap_or(&b.faces[g]);
        let pts = trace(b, ef, face, &ma, &mz)?;
        let ce = fit(nb, pts, va, vz, face, Some(a.ball.c), Some(z.ball.c));
        if inside_face(b, g, &nb.edges[ce]) {
            // Across the face's seam: in two, at a vertex on the seam.
            let seam = b.faces[g].loops.iter().flatten().map(|c| c.edge).find(|&e| is_seam(nb, g, e));
            if let Some((tc, at, _)) = seam.and_then(|x| seam_crossing_of(nb, g, ce, x)) {
                let cv = vertex_on_boundary(nb, g, &at, tol).ok_or_else(|| failed("trimming", "a fillet's cut across a face's seam isn't on it"))?;
                let (ed, vz) = (nb.edges[ce].clone(), nb.edges[ce].v[1]);
                nb.edges[ce].t = [ed.t[0], tc];
                nb.edges[ce].v[1] = cv;
                let ce2 = nb.add_edge(ed.curve, tc, ed.t[1], cv, vz);
                out.cuts.entry(g).or_default().extend([ce, ce2]);
                fe.extend([ce, ce2]);
                return Ok(());
            }
            out.cuts.entry(g).or_default().push(ce);
            fe.push(ce);
        } else {
            // The face grows to the cut: on its surface's continuation.
            if let Some(x) = grown {
                nb.faces[g].surf = x.surf;
            }
            fe.extend(grow_to_cut(nb, g, ce, out)?);
        }
        return Ok(());
    }
    // Round corners: the faces across the face's edges from the one it leaves over to the one
    // it comes back over (the way round past the stretch's middle), each traced until the
    // next one's surface.
    let chain = if kz == k {
        cliff_faces(b, ef, k, a, z)?
    } else {
        // Across the strip: from the face rail k runs onto to the one the other rail does,
        // over the edge between them.
        debug(|| format!("  across from face {} ({:?}, rail {k} at {}) to face {} ({:?}, rail {kz} at {}); edges {:?} / {:?}", a.into, b.faces[a.into].surf.kind(), a.t, z.into, b.faces[z.into].surf.kind(), z.t, b.faces[a.into].loops.iter().flatten().map(|c| (c.edge, beyond(b, c.edge, a.into))).collect::<Vec<_>>(), b.faces[z.into].loops.iter().flatten().map(|c| (c.edge, beyond(b, c.edge, z.into))).collect::<Vec<_>>()));
        common_edge(b, a.into, z.into).ok_or_else(|| unsupported("a fillet cut across by faces that don't meet"))?;
        vec![a.into, z.into]
    };
    let mut marks = vec![(ma, va)];
    let mut pieces: Vec<Vec<P3>> = Vec::new();
    // Each corner: its vertex on the edge between the faces (or past its end) and how it joins.
    enum Joint {
        OnEdge,
        Past { corner: usize, ext: usize },
    }
    let mut joints: Vec<Joint> = Vec::new();
    for w in chain.windows(2) {
        let (g, h) = (w[0], w[1]);
        let from = marks.last().map(|x| x.0).ok_or_else(|| failed("trimming", "a fillet's cut has no start"))?;
        let x = common_edge(b, g, h).ok_or_else(|| unsupported("a closed fillet whose rail runs off over faces that don't meet"))?;
        // The next face on the same surface (a hole's wall in two halves): up to the edge
        // between them.
        let (pts, at) = if same_surface(&b.faces[g].surf, &b.faces[h].surf) {
            let side = edge_side(&b.faces[g], &b.edges[x]);
            trace_to(b, ef, &b.faces[g], &from, &mz, Some(&side))?
        } else {
            trace_until(b, ef, &b.faces[g], &from, &mz, Some(&b.faces[h]))?
        };
        let at = at.ok_or_else(|| failed("corner", "a fillet's cut doesn't reach the next face round the corner"))?;
        let xe = &b.edges[x];
        let tx = xe.curve.closest(&at.p, xe.t[0] - 1e6, xe.t[1] + 1e6);
        if tx >= xe.t[0] - tol::ROUNDOFF && tx <= xe.t[1] + tol::ROUNDOFF {
            let xv = vertex_on_boundary(nb, g, &at.p, tol.max(tol::POINT)).ok_or_else(|| failed("corner", "a fillet's cut round a corner isn't on the corner's edge"))?;
            joints.push(Joint::OnEdge);
            marks.push((at, xv));
        } else {
            let cv = if (xe.curve.point(tx) - xe.point(xe.t[0])).norm() < (xe.curve.point(tx) - xe.point(xe.t[1])).norm() { xe.v[0] } else { xe.v[1] };
            let tc = if cv == xe.v[0] { xe.t[0] } else { xe.t[1] };
            let xv = nb.add_vertex(xe.curve.point(tx));
            let ext = add_edge(nb, xe.curve.clone(), tc, tx, cv, xv);
            joints.push(Joint::Past { corner: cv, ext });
            marks.push((at, xv));
        }
        pieces.push(pts);
    }
    let last = *chain.last().ok_or_else(|| failed("trimming", "a fillet's cut over no faces"))?;
    let from = marks.last().map(|x| x.0).ok_or_else(|| failed("trimming", "a fillet's cut has no start"))?;
    pieces.push(trace(b, ef, &b.faces[last], &from, &mz)?);
    marks.push((mz, vz));
    // The cut on each face, then how each face takes it.
    let mut cuts: Vec<usize> = Vec::new();
    for (i, pts) in pieces.into_iter().enumerate() {
        let (v0, v1) = (marks[i].1, marks[i + 1].1);
        let c0 = (i == 0).then_some(a.ball.c);
        let c1 = (i + 1 == chain.len()).then_some(z.ball.c);
        cuts.push(fit(nb, pts, v0, v1, &b.faces[chain[i]], c0, c1));
    }
    let inside: Vec<bool> = chain.iter().zip(&cuts).map(|(&g, &c)| inside_face(b, g, &nb.edges[c])).collect();
    let all_past = joints.iter().all(|j| matches!(j, Joint::Past { .. }));
    if inside.iter().all(|&x| x) && joints.iter().all(|j| matches!(j, Joint::OnEdge)) {
        for (&g, &c) in chain.iter().zip(&cuts) {
            out.cuts.entry(g).or_default().push(c);
        }
    } else if inside.iter().all(|&x| !x) && all_past {
        // Each face from where the cut comes onto it to where it leaves, through the corners'
        // extensions.
        for (i, &g) in chain.iter().enumerate() {
            let mut path = Vec::new();
            let mut from = marks[i].1;
            if i > 0 {
                let Joint::Past { corner, ext } = joints[i - 1] else { unreachable!() };
                from = corner;
                path.push((ext, true));
            }
            path.push((cuts[i], true));
            let mut to = marks[i + 1].1;
            if i + 1 < chain.len() {
                let Joint::Past { corner, ext } = joints[i] else { unreachable!() };
                path.push((ext, false));
                to = corner;
            }
            out.patches.push(Patch { face: g, from, to, path, via: None, stretch: Vec::new() });
        }
    } else {
        return Err(unsupported("a fillet's cut round corners partly inside its faces"));
    }
    fe.extend(cuts);
    Ok(())
}

/// Where the cut `ce` (an edge from face `g`'s boundary to its boundary) crosses the line of
/// the face's seam `seam` (its iso-line, run on past the seam's ends): its u (continued along it)
/// passes the seam's. The cut's parameter there, the point on the seam's curve and its
/// parameter; `None` where it doesn't cross, or the seam's curve doesn't reach the crossing
/// (within the cut's own fit to the face).
fn seam_crossing_of(nb: &Body, g: usize, ce: usize, seam: usize) -> Option<(f64, P3, f64)> {
    let face = &nb.faces[g];
    let c = face.loops.iter().flatten().find(|c| c.edge == seam)?;
    let se = &nb.edges[seam];
    let periods = super::pcurve::closed_periods(&face.surf);
    let (u_seam, axis) = {
        let (q0, q1) = (c.pc.point(se.t[0]), c.pc.point(se.t[1]));
        if (q0.x - q1.x).abs() < (q0.y - q1.y).abs() { (q0.x, 0) } else { (q0.y, 1) }
    };
    let per = periods[axis]?;
    let ed = &nb.edges[ce];
    let side = |q: f64| ((q - u_seam) / per).floor();
    let m = 64;
    let mut hint = None;
    // (Parameter, coordinate, the projection's hint there.)
    type Sample = (f64, f64, Option<(f64, f64)>);
    let mut prev: Option<Sample> = None;
    let mut cross = None;
    for i in 0..=m {
        let t = ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / m as f64;
        let q = super::pcurve::project_along(&face.surf, &ed.point(t), hint, periods);
        let h = hint;
        hint = Some((q.x, q.y));
        if let Some((tp, up, hp)) = prev
            && side(up) != side(q[axis])
        {
            cross = Some((tp, t, up, hp));
            break;
        }
        prev = Some((t, q[axis], h));
    }
    let (mut lo, mut hi, ulo, h0) = cross?;
    for _ in 0..60 {
        let t = 0.5 * (lo + hi);
        let q = super::pcurve::project_along(&face.surf, &ed.point(lo), h0, periods);
        let q2 = super::pcurve::project_along(&face.surf, &ed.point(t), Some((q.x, q.y)), periods);
        if side(q2[axis]) == side(ulo) { lo = t } else { hi = t }
    }
    let tc = 0.5 * (lo + hi);
    let c0 = ed.point(tc);
    let tx = se.curve.closest(&c0, se.t[0] - 1e6, se.t[1] + 1e6);
    let at = se.curve.point(tx);
    ((at - c0).norm() <= 1e-4 * (1.0 + c0.coords.abs().max())).then_some((tc, at, tx))
}

/// Face `g` grown to the cut `ce` outside it (from its boundary to its boundary): the stretch of
/// its boundary between the cut's ends nearer the cut's middle replaced by the cut. Where that
/// stretch goes round the face's seam (a boss's wall, the cut below its rim), the cut is split
/// where it crosses the seam's line and the seam runs on to there. Returns the strip's edges
/// along the cut.
fn grow_to_cut(nb: &mut Body, g: usize, ce: usize, out: &mut Pieces) -> Result<Vec<usize>> {
    let bad = || unsupported("a fillet's cut outside a face that doesn't meet its boundary");
    let (va, vz) = (nb.edges[ce].v[0], nb.edges[ce].v[1]);
    let face = &nb.faces[g];
    let l = face.loops.iter().find(|l| l.iter().any(|c| nb.co_start(c) == va) && l.iter().any(|c| nb.co_start(c) == vz)).ok_or_else(bad)?;
    let n = l.len();
    // Both ways from va to vz along the loop, as coedge indices in loop order.
    let ia = l.iter().position(|c| nb.co_start(c) == va).ok_or_else(bad)?;
    let iz = l.iter().position(|c| nb.co_start(c) == vz).ok_or_else(bad)?;
    let run = |from: usize, to: usize| -> Vec<usize> {
        let mut i = from;
        let mut v = Vec::new();
        while i != to {
            v.push(i);
            i = (i + 1) % n;
        }
        v
    };
    let ed = &nb.edges[ce];
    let mid = ed.point(ed.mid_t());
    let dist = |w: &[usize]| -> f64 {
        w.iter()
            .flat_map(|&i| {
                let e = &nb.edges[l[i].edge];
                (1..8).map(move |s| (e.point(e.t[0] + (e.t[1] - e.t[0]) * s as f64 / 8.0) - mid).norm())
            })
            .fold(f64::INFINITY, f64::min)
    };
    let (fwd, back) = (run(ia, iz), run(iz, ia));
    let way = if dist(&fwd) <= dist(&back) { fwd } else { back };
    let seam: Vec<usize> = way.iter().copied().filter(|&i| face.loops.iter().flatten().filter(|c| c.edge == l[i].edge).count() == 2).collect();
    if seam.is_empty() {
        let stretch: Vec<usize> = way.iter().map(|&i| l[i].edge).collect();
        out.patches.push(Patch { face: g, from: va, to: vz, path: vec![(ce, true)], via: None, stretch });
        return Ok(vec![ce]);
    }
    // Round the seam: the stretch up to it and on from it (the seam's two coedges, the far side
    // of the face between them, stay).
    let (s0, s1) = (seam[0], seam[seam.len() - 1]);
    if seam.len() != 2 || l[s0].edge != l[s1].edge {
        return Err(bad());
    }
    let pos = |i: usize| way.iter().position(|&x| x == i).unwrap_or(0);
    let pre: Vec<usize> = way[..pos(s0)].iter().map(|&i| l[i].edge).collect();
    let post: Vec<usize> = way[pos(s1) + 1..].iter().map(|&i| l[i].edge).collect();
    let sv = nb.co_start(&l[s0]);
    if pre.is_empty() || post.is_empty() || nb.co_end(&l[s1]) != sv {
        return Err(bad());
    }
    let l0 = l[way[0]].clone();
    let (tc, at, x_t) = seam_crossing_of(nb, g, ce, l[s0].edge).ok_or_else(bad)?;
    let se = &nb.edges[l[s0].edge];
    let (seam_curve, se_t, se_v) = (se.curve.clone(), se.t, se.v);
    let ed = nb.edges[ce].clone();
    // The seam's curve run on to there from its end at the seam vertex.
    let tsv = if se_v[0] == sv { se_t[0] } else { se_t[1] };
    let tx = x_t;
    let cv = nb.add_vertex(at);
    let x = add_edge(nb, seam_curve, tsv, tx, sv, cv);
    nb.edges[ce].t = [ed.t[0], tc];
    nb.edges[ce].v = [va, cv];
    let ce2 = nb.add_edge(ed.curve.clone(), tc, ed.t[1], cv, vz);
    let to_sv = nb.edges[x].v[0] == cv;
    // The way taken runs from va (the cut's start) or from vz.
    let (pre_path, post_path) = if nb.co_start(&l0) == va {
        (vec![(ce, true), (x, to_sv)], vec![(x, !to_sv), (ce2, true)])
    } else {
        (vec![(ce2, false), (x, to_sv)], vec![(x, !to_sv), (ce, false)])
    };
    let (first, last) = (nb.co_start(&l0), if nb.co_start(&l0) == va { vz } else { va });
    out.patches.push(Patch { face: g, from: first, to: sv, path: pre_path, via: None, stretch: pre });
    out.patches.push(Patch { face: g, from: sv, to: last, path: post_path, via: None, stretch: post });
    Ok(vec![ce, ce2])
}

const WHOLE_STRIP: &str = "a fillet whose cut by a face crosses the whole strip";

/// An open strip's rails between its ends' vertices `rv` (at rail parameters `rt`), [end][rail].
struct StripRails<'a> {
    ef: &'a EdgeFillet,
    st: &'a Strip,
    rv: [[usize; 2]; 2],
    rt: [[f64; 2]; 2],
    vtol: f64,
}

impl StripRails<'_> {
    /// Rail k's crossing `c` of its face's boundary: its vertex there and its rail parameter.
    fn at(&self, nb: &mut Body, k: usize, c: &Crossing, near: f64) -> Result<(usize, f64)> {
        let p = c.ball.q[k].p;
        let v = vertex_on_boundary(nb, self.ef.faces[k], &p, self.vtol).ok_or_else(|| failed("trimming", "a fillet rail's crossing isn't on its face's boundary"))?;
        Ok((v, self.st.rail_param(k, &p, c.t, near)))
    }

    /// Rail k from vertex v0 (parameter t0) to v1 (t1): a cut in its face, an edge of the strip.
    #[allow(clippy::too_many_arguments)]
    fn rail(&self, nb: &mut Body, k: usize, (v0, t0): (usize, f64), (v1, t1): (usize, f64), fe: &mut Vec<usize>, given: &mut Vec<(usize, Curve)>, out: &mut Pieces) {
        // Along an edge of the face from end to end (the strip takes the whole face): that
        // edge is the rail.
        let fi = self.ef.faces[k];
        let rail_at = |s: f64| self.st.rails[k].point(t0 + (t1 - t0) * s);
        let along = nb.faces[fi].loops.iter().flatten().map(|c| c.edge).find(|&e| {
            let ed = &nb.edges[e];
            !ed.degenerate && (ed.v == [v0, v1] || ed.v == [v1, v0]) && [0.25, 0.5, 0.75].iter().all(|&s| {
                let p = rail_at(s);
                (ed.point(ed.curve.closest(&p, ed.t[0], ed.t[1])) - p).norm() < self.vtol.max(ed.tol)
            })
        });
        if let Some(e) = along {
            debug(|| format!("edge {}: rail {k} runs along edge {e} of face {fi}: the strip takes the face", self.ef.e));
            out.consumed.insert(fi);
            fe.push(e);
            return;
        }
        let e = add_edge(nb, self.st.rails[k].clone(), t0, t1, v0, v1);
        out.cuts.entry(self.ef.faces[k]).or_default().push(e);
        fe.push(e);
        given.extend(rail_pcurve(self.st, k, t0, t1).map(|c| (e, c)));
        // Its p-curve on the face, parametrized as the edge is (turned round with it).
        if let Some(on) = &self.st.on_faces {
            let pc = if t1 >= t0 { on[k].clone() } else { on[k].reversed(t0, t1).0 };
            out.on_face.insert((e, fi), pc);
        }
    }

    /// Each rail on its own, cut where it runs off its face and back (`offs`, per rail).
    fn each(&self, b: &Body, nb: &mut Body, offs: &[Vec<(Crossing, Crossing)>; 2], given: &mut Vec<(usize, Curve)>, out: &mut Pieces) -> Result<Vec<usize>> {
        let mut fe = Vec::new();
        for k in 0..2 {
            let mut at = (self.rv[0][k], self.rt[0][k]);
            for (a, z) in &offs[k] {
                let xa = self.at(nb, k, a, at.1)?;
                let xz = self.at(nb, k, z, at.1)?;
                self.rail(nb, k, at, xa, &mut fe, given, out);
                cliff(b, nb, self.ef, (k, a, xa.0), (k, z, xz.0), &mut fe, out)?;
                at = xz;
            }
            self.rail(nb, k, at, (self.rv[1][k], self.rt[1][k]), &mut fe, given, out);
        }
        Ok(fe)
    }

    /// Both rails running off once each where one cut crosses the whole strip: the strip in two,
    /// each rail up to where it leaves and on from where it comes back, the cuts from rail 0's
    /// crossings to rail 1's.
    fn across(&self, b: &Body, nb: &mut Body, offs: &[Vec<(Crossing, Crossing)>; 2], given: &mut Vec<(usize, Curve)>, out: &mut Pieces) -> Result<Vec<usize>> {
        let mut fe = Vec::new();
        let mut x = [[(0usize, 0.0f64); 2]; 2]; // [leave, return][rail]
        for k in 0..2 {
            let (a, z) = &offs[k][0];
            x[0][k] = self.at(nb, k, a, self.rt[0][k])?;
            x[1][k] = self.at(nb, k, z, self.rt[0][k])?;
            self.rail(nb, k, (self.rv[0][k], self.rt[0][k]), x[0][k], &mut fe, given, out);
            self.rail(nb, k, x[1][k], (self.rv[1][k], self.rt[1][k]), &mut fe, given, out);
        }
        for (j, i) in [(0usize, 0usize), (1, 1)] {
            let (c0, c1) = if j == 0 { (&offs[0][0].0, &offs[1][0].0) } else { (&offs[0][0].1, &offs[1][0].1) };
            cliff(b, nb, self.ef, (0, c0, x[i][0].0), (1, c1, x[i][1].0), &mut fe, out)?;
        }
        Ok(fe)
    }
}

/// The edges `es` grouped into the loops they close (by shared vertices).
fn loops_of(nb: &Body, es: &[usize]) -> Vec<Vec<usize>> {
    let mut group: Vec<usize> = (0..es.len()).collect();
    fn root(g: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while g[r] != r {
            r = g[r];
        }
        g[i] = r;
        r
    }
    for i in 0..es.len() {
        for j in i + 1..es.len() {
            let (a, c) = (&nb.edges[es[i]], &nb.edges[es[j]]);
            if a.v.iter().any(|v| c.v.contains(v)) {
                let (ri, rj) = (root(&mut group, i), root(&mut group, j));
                group[ri] = rj;
            }
        }
    }
    let mut parts: Vec<(usize, Vec<usize>)> = Vec::new();
    for (i, &e) in es.iter().enumerate() {
        let r = root(&mut group, i);
        match parts.iter_mut().find(|p| p.0 == r) {
            Some(p) => p.1.push(e),
            None => parts.push((r, vec![e])),
        }
    }
    parts.into_iter().map(|p| p.1).collect()
}

/// Where an open strip's rail k runs off its face and back between spine parameters t0 and t1
/// (its ends): pairs of crossings of its face's edges, in order along the strip.
fn open_cliffs(b: &Body, ef: &EdgeFillet, k: usize, t0: f64, t1: f64, ends: [P3; 2]) -> Result<Vec<(Crossing, Crossing)>> {
    let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
    let fi = ef.faces[k];
    // Only along the edge itself (past its ends the ends' own cuts rule).
    let ed = &b.edges[ef.e];
    let (lo, hi) = (t0.min(t1).max(ed.t[0].min(ed.t[1])), t0.max(t1).min(ed.t[0].max(ed.t[1])));
    if hi <= lo {
        return Ok(Vec::new());
    }
    let margin = 1e-7 * (1.0 + (hi - lo));
    let reach = ef.reach + ed_len(b, ef.e);
    let mut cs: Vec<Crossing> = Vec::new();
    for c in b.faces[fi].loops.iter().flatten() {
        let x = &b.edges[c.edge];
        if c.edge == ef.e || x.degenerate || is_seam(b, fi, c.edge) {
            continue;
        }
        let bd = Boundary { curve: &x.curve, pc: &c.pc, t: x.t };
        for (_, ball) in crossings(ef, f, ef.k_at(ef.tm), k, &bd, &ef.mid.c, reach) {
            let t = ef.spine.param_of(&ball.c, ef.tm);
            if t <= lo + margin || t >= hi - margin || cs.iter().any(|x| (x.t - t).abs() < margin) || ends.iter().any(|q| (q - ball.q[k].p).norm() < 1e-5 * (1.0 + ef.r())) {
                continue;
            }
            // Over a smooth edge the ball rolls on (the strip's next piece), not off.
            if let Some(g) = beyond(b, c.edge, fi) {
                let n = geom::contact(&b.faces[g], geom::foot(&b.faces[g], &ball.q[k].p, None)).n;
                if n.dot(&ball.q[k].n) > 1.0 - tol::ANGULAR_POINT {
                    continue;
                }
            }
            let into = beyond(b, c.edge, fi).ok_or_else(|| unsupported("a fillet whose rail runs off a sheet's free edge"))?;
            cs.push(Crossing { t, ball, into, edge: c.edge });
        }
    }
    if cs.is_empty() {
        return Ok(Vec::new());
    }
    if !cs.len().is_multiple_of(2) {
        debug(|| format!("edge {}: rail {k} crosses its face's edges at {:?} between {lo} and {hi}", ef.e, cs.iter().map(|c| (c.t, c.into, c.edge, c.ball.q[k].p)).collect::<Vec<_>>()));
        return Err(unsupported("a fillet whose rail leaves its face and doesn't come back"));
    }
    // In order along the rail, from its start end.
    cs.sort_by(|a, b| a.t.total_cmp(&b.t));
    if t1 < t0 {
        cs.reverse();
    }
    debug(|| format!("edge {}: rail {k} runs off its face {:?}", ef.e, cs.iter().map(|c| (c.t, c.into)).collect::<Vec<_>>()));
    // Each stretch between a pair really off the face.
    let pairs: Vec<(Crossing, Crossing)> = cs.chunks(2).map(|w| (w[0], w[1])).collect();
    for (a, z) in &pairs {
        let mid = ef.ball_from(b, &a.ball, a.t, 0.5 * (a.t + z.t)).ok_or_else(|| failed("rolling ball", "the fillet's ball is lost along the edge"))?;
        if point_in_face(b, fi, &mid.q[k].p) {
            return Err(unsupported("a fillet whose rail touches its face's edges"));
        }
    }
    Ok(pairs)
}

/// The faces a closed strip's rail k runs off over between crossings `a` and `z`: across the
/// rail's face's edges from `a`'s to `z`'s, the way round past the rail's middle point there.
fn cliff_faces(b: &Body, ef: &EdgeFillet, k: usize, a: &Crossing, z: &Crossing) -> Result<Vec<usize>> {
    if a.edge == z.edge {
        return Ok(vec![a.into]);
    }
    let fi = ef.faces[k];
    let l = b.faces[fi].loops.iter().find(|l| l.iter().any(|c| c.edge == a.edge) && l.iter().any(|c| c.edge == z.edge)).ok_or_else(|| unsupported("a closed fillet whose rail runs off over another loop"))?;
    let n = l.len();
    let ia = l.iter().position(|c| c.edge == a.edge).ok_or_else(|| failed("trimming", "a crossing's edge isn't on its face"))?;
    let way = |step: usize| -> Vec<usize> {
        let mut i = ia;
        let mut out = vec![l[i].edge];
        while l[i].edge != z.edge && out.len() <= n {
            i = (i + step) % n;
            out.push(l[i].edge);
        }
        out
    };
    // The rail's point halfway along the stretch, and the edge of the face nearest it.
    let mid = ef.ball_from(b, &a.ball, a.t, 0.5 * (a.t + z.t)).ok_or_else(|| failed("rolling ball", "the fillet's ball is lost along the edge"))?.q[k].p;
    let near = l
        .iter()
        .map(|c| {
            let e = &b.edges[c.edge];
            ((e.point(e.curve.closest(&mid, e.t[0], e.t[1])) - mid).norm(), c.edge)
        })
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .map(|x| x.1);
    let (fwd, back) = (way(1), way(n - 1));
    let edges = if near.is_some_and(|e| fwd.contains(&e)) && !(near.is_some_and(|e| back.contains(&e)) && back.len() < fwd.len()) { fwd } else { back };
    let mut faces: Vec<usize> = Vec::new();
    for e in edges {
        let g = beyond(b, e, fi).ok_or_else(|| unsupported("a closed fillet whose rail runs off a sheet's free edge"))?;
        if faces.last() != Some(&g) {
            faces.push(g);
        }
    }
    Ok(faces)
}

/// Whether two surfaces are the same (elementary ones by their parameters; B-splines by
/// identity).
fn same_surface(a: &Surface, c: &Surface) -> bool {
    let close = |p: &P3, q: &P3| (p - q).norm() < tol::REL * (1.0 + p.coords.abs().max());
    let par = |u: &V3, v: &V3| u.cross(v).norm() < tol::ANGULAR;
    match (a, c) {
        (Surface::Plane { f }, Surface::Plane { f: g }) => par(&f.z, &g.z) && (g.o - f.o).dot(&f.z).abs() < tol::REL * (1.0 + f.o.coords.abs().max()),
        (Surface::Cylinder { f, r }, Surface::Cylinder { f: g, r: s }) => (r - s).abs() < tol::REL * (1.0 + r) && par(&f.z, &g.z) && (g.o - f.o).cross(&f.z).norm() < tol::REL * (1.0 + f.o.coords.abs().max()),
        (Surface::Sphere { f, r }, Surface::Sphere { f: g, r: s }) => (r - s).abs() < tol::REL * (1.0 + r) && close(&f.o, &g.o),
        (Surface::Nurbs(x), Surface::Nurbs(y)) => std::sync::Arc::ptr_eq(x, y),
        _ => false,
    }
}

/// The edge between faces `a` and `c`, if any.
fn common_edge(b: &Body, a: usize, c: usize) -> Option<usize> {
    b.faces[a].loops.iter().flatten().map(|x| x.edge).find(|&e| beyond(b, e, a) == Some(c))
}

/// Fits a curve through points of the fillet's cut by face `g` (exact at both ends): (curve,
/// its parameter range). Where an end is a rail's (`centres`: the ball's centre there), the
/// curve leaves along the cut's true direction there, square to the strip's and the face's
/// normals: the cut meets the face's boundary edge tangentially at a rail's end, and only an
/// exact tangent keeps its p-curve on the face's side of that edge.
fn fit_cut(xs: &[P3], g: Option<&Face>, centres: [Option<P3>; 2]) -> (Curve, f64, f64) {
    let fit = |ps: &[f64]| -> NurbsCurve {
        let last = ps.len() - 1;
        let tangent = |i: usize, towards: usize, c: Option<P3>| -> Option<V3> {
            let (g, c) = (g?, c?);
            let p = xs[i];
            let ng = geom::contact(g, geom::foot(g, &p, None)).n;
            let d = (p - c).normalize().cross(&ng);
            if d.norm() < tol::ROUNDOFF {
                return None;
            }
            let chord = xs[towards] - p;
            let d = d.normalize() * d.normalize().dot(&chord).signum();
            // As fast as the parameters run there.
            Some(d * (chord.norm() / (ps[towards] - ps[i]).abs().max(tol::TINY)))
        };
        let d0 = tangent(0, 1, centres[0]);
        let d1 = tangent(last, last - 1, centres[1]).map(|d| -d);
        NurbsCurve::interpolate(xs, ps, 3, d0, d1)
    };
    let chord = NurbsCurve::chord_params(xs);
    let last = chord.len() - 1;
    let c = fit(&chord);
    // Points traced in uneven steps can make the interpolant swing off the curve between
    // them; measured against the face the cut lies on, centripetal parameters (which follow
    // uneven spacing more calmly) are taken where they keep closer.
    if let Some(g) = g {
        let off = |c: &NurbsCurve, ps: &[f64]| -> f64 {
            let mut worst: f64 = 0.0;
            let mut hint = None;
            for w in ps.windows(2) {
                for f in [0.25, 0.5, 0.75] {
                    let p = c.point(w[0] + (w[1] - w[0]) * f);
                    let uv = geom::foot(g, &p, hint);
                    hint = Some(uv);
                    worst = worst.max((geom::contact(g, uv).p - p).norm());
                }
            }
            worst
        };
        let size = xs.iter().fold(0.0f64, |m, p| m.max(p.coords.abs().max()));
        let e0 = off(&c, &chord);
        if e0 > 1e-7 * (1.0 + size) {
            // Chord lengths, then a blend of them with even steps.
            let mut best = (e0, c, chord.clone());
            let mut len = vec![0.0; xs.len()];
            for i in 1..xs.len() {
                len[i] = len[i - 1] + (xs[i] - xs[i - 1]).norm().max(tol::TINY);
            }
            let total = len[last];
            let even: Vec<f64> = (0..xs.len()).map(|i| i as f64 / last as f64).collect();
            let lens: Vec<f64> = len.iter().map(|x| x / total).collect();
            let mixed: Vec<f64> = lens.iter().zip(&even).map(|(a, b)| 0.5 * (a + b)).collect();
            for ps in [lens, mixed] {
                let c2 = fit(&ps);
                let e2 = off(&c2, &ps);
                if e2 < best.0 {
                    best = (e2, c2, ps);
                }
            }
            debug(|| {
                let segs: Vec<f64> = xs.windows(2).map(|w| (w[1] - w[0]).norm()).collect();
                let i = (0..segs.len()).max_by(|&a, &b| segs[a].total_cmp(&segs[b])).unwrap_or(0);
                format!("  cut fit off its face by {e0:.3e}, at best {:.3e} ({} points, longest step {:.3e} at {i}, {:?} → {:?})", best.0, xs.len(), segs.get(i).copied().unwrap_or(0.0), xs.get(i), xs.get(i + 1))
            });
            let (_, c, ps) = best;
            return (Curve::nurbs(c), ps[0], ps[last]);
        }
    }
    (Curve::nurbs(c), chord[0], chord[last])
}

/// Whether point `p` (on face `fi`'s surface) lies inside the face.
fn point_in_face(b: &Body, fi: usize, p: &P3) -> bool {
    let f = &b.faces[fi];
    let near = near_chart(b, f, p);
    let (u, v) = f.surf.project(p, near.map(|q| (q.x, q.y)));
    let polys = super::classify::face_polygons(b, f);
    super::classify::periodic_candidates(&f.surf, u, v, &polys).into_iter().any(|uv| super::classify::in_face(&polys, &f.surf, uv, tol::ROUNDOFF) != super::classify::InFace::Outside)
}

/// Where the closed strip's rail k crosses its face's seam, found from the rail's (u, v): the
/// seam is an iso-line at one end of a parameter range the rail's (u, v) runs past.
fn seam_crossing(b: &Body, ef: &EdgeFillet, k: usize, period: f64) -> Result<Option<Ball>> {
    let fi = ef.faces[k];
    let f = &b.faces[fi];
    let Some(seam) = f.loops.iter().flatten().find(|c| is_seam(b, fi, c.edge)) else { return Ok(None) };
    let se = &b.edges[seam.edge];
    let (a, z) = (seam.pc.point(se.t[0]), seam.pc.point(se.t[1]));
    // The seam's constant coordinate and its two values (one per side).
    let axis = if (a.x - z.x).abs() < (a.y - z.y).abs() { 0 } else { 1 };
    let vals: Vec<f64> = f.loops.iter().flatten().filter(|c| c.edge == seam.edge).map(|c| c.pc.point(se.mid_t())[axis]).collect();
    let ed = &b.edges[ef.e];
    let lost = || failed("rolling ball", "the fillet's ball is lost along the edge");
    let n = 128;
    let mut cur = (ef.tm, ef.mid);
    let mut prev: Option<(f64, Ball)> = None;
    for i in 0..=n {
        let t = ed.t[0] + period * i as f64 / n as f64;
        let x = ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(lost)?;
        cur = (t, x);
        if let Some((tp, bp)) = prev {
            let (cp, cn) = (bp.q[k].uv[axis], x.q[k].uv[axis]);
            for &sv in &vals {
                if (cp - sv) * (cn - sv) < 0.0 {
                    // Bisection on the rail's coordinate against the seam's.
                    let (mut lo, mut hi, mut bl) = (tp, t, bp);
                    for _ in 0..60 {
                        let m = 0.5 * (lo + hi);
                        let y = ef.ball_from(b, &bl, lo, m).ok_or_else(lost)?;
                        if (y.q[k].uv[axis] - sv) * (cp - sv) > 0.0 {
                            (lo, bl) = (m, y);
                        } else {
                            hi = m;
                        }
                    }
                    return Ok(ef.ball_from(b, &bl, lo, 0.5 * (lo + hi)));
                }
            }
        }
        prev = Some((t, x));
    }
    Ok(None)
}

/// Where a closed strip's rail k crosses its face's boundary that the search along the
/// boundary missed (that search seeds a variable fillet with one radius, and misses crossings
/// where the radius is far from it): the rail sampled round the turn, each change between on and
/// off its face with no crossing of `have` near it bisected to the boundary, then solved on the
/// face's edge nearest there (the rail's side of that edge). Nothing where the ball is lost on
/// the way round.
fn missed_crossings(b: &Body, ef: &EdgeFillet, k: usize, period: f64, have: &[Crossing]) -> Result<Vec<Crossing>> {
    let fi = ef.faces[k];
    let t0 = b.edges[ef.e].t[0];
    let n = 192;
    let step = period / n as f64;
    let Some(mut cur) = ef.ball_from(b, &ef.mid, ef.tm, t0) else { return Ok(Vec::new()) };
    let mut samples: Vec<(f64, Ball, bool)> = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = t0 + step * i as f64;
        if i > 0 {
            let Some(x) = ef.ball_from(b, &cur, t - step, t) else { return Ok(Vec::new()) };
            cur = x;
        }
        samples.push((t, cur, point_in_face(b, fi, &cur.q[k].p)));
    }
    let mut out: Vec<Crossing> = Vec::new();
    for w in samples.windows(2) {
        let ((ta, ba, ia), (tb, _, ib)) = (w[0], w[1]);
        let near = |c: &Crossing| {
            let d = wrap(c.t - ta, -0.5 * period, period);
            d >= -0.05 * step && d <= 1.05 * step
        };
        if ia == ib || have.iter().chain(&out).any(near) {
            continue;
        }
        // Onto the face's boundary as its polygons have it.
        let (mut lo, mut hi, mut bl) = (ta, tb, ba);
        for _ in 0..50 {
            let m = 0.5 * (lo + hi);
            let Some(y) = ef.ball_from(b, &bl, lo, m) else { break };
            if point_in_face(b, fi, &y.q[k].p) == ia {
                (lo, bl) = (m, y);
            } else {
                hi = m;
            }
        }
        let p = bl.q[k].p;
        // The edge crossed: the face's edge nearest there.
        let edge = b.faces[fi]
            .loops
            .iter()
            .flatten()
            .map(|c| c.edge)
            .filter(|&e| e != ef.e && !b.edges[e].degenerate && !is_seam(b, fi, e))
            .map(|e| {
                let x = &b.edges[e];
                ((x.point(x.curve.closest(&p, x.t[0], x.t[1])) - p).norm(), e)
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .map(|x| x.1);
        let Some(edge) = edge else { continue };
        // Onto the edge itself: bisection on the rail's side of it, from a step either way.
        let side = edge_side(&b.faces[fi], &b.edges[edge]);
        let at = |from: &Ball, tf: f64, t: f64| ef.ball_from(b, from, tf, t);
        let (mut a, mut z) = ((lo - 0.25 * step).max(ta), (hi + 0.25 * step).min(tb));
        let (Some(xa), Some(xz)) = (at(&bl, lo, a), at(&bl, lo, z)) else { continue };
        let (sa, sz) = (side(&xa.q[k].p), side(&xz.q[k].p));
        let mut ball = bl;
        let mut t = lo;
        if (sa < 0.0) != (sz < 0.0) {
            let mut xb = xa;
            for _ in 0..60 {
                let m = 0.5 * (a + z);
                let Some(y) = at(&xb, a, m) else { break };
                if (side(&y.q[k].p) < 0.0) == (sa < 0.0) {
                    (a, xb) = (m, y);
                } else {
                    z = m;
                }
            }
            (t, ball) = (a, xb);
        }
        let into = beyond(b, edge, fi).ok_or_else(|| unsupported("a closed fillet whose rail runs off a sheet's free edge"))?;
        debug(|| format!("edge {}: rail {k} crosses edge {edge} into face {into} at {t} (missed by the boundary search)", ef.e));
        out.push(Crossing { t, ball, into, edge });
    }
    Ok(out)
}

/// Where a closed strip's rail crosses an edge of its face that isn't the face's seam.
#[derive(Debug, Clone, Copy)]
struct Crossing {
    /// Its spine parameter, unwrapped into the turn from the strip's seam.
    t: f64,
    ball: Ball,
    into: usize,
    /// The edge crossed.
    edge: usize,
}

/// Adds a closed strip's rails, seam and face. A rail may run off its face over an edge and
/// back (a fillet round a boss near the end face's rim): the strip is cut there by the face
/// beyond, which it hangs over (SMLib's cliff roll-over), from where the rail leaves to where it
/// comes back.
fn closed_strip(b: &Body, nb: &mut Body, ef: &EdgeFillet, fit_tol: f64, out: &mut Pieces, input: BodyId) -> Result<()> {
    let ed = &b.edges[ef.e];
    let f = [&b.faces[ef.faces[0]], &b.faces[ef.faces[1]]];
    let r = ef.r();
    let tol = tol::REL_POINT * (1.0 + r);
    // A turn of the spine (an exact band's rails are circles sharing its angle).
    let period = ed.t[1] - ed.t[0];
    // Each rail's crossings of its face's boundary: the face's seam, or edges it runs off over.
    let mut seams: [Option<Ball>; 2] = [None, None];
    let mut cross: [Vec<Crossing>; 2] = [Vec::new(), Vec::new()];
    for k in 0..2 {
        let fi = ef.faces[k];
        for c in b.faces[fi].loops.iter().flatten() {
            let x = &b.edges[c.edge];
            if c.edge == ef.e || x.degenerate {
                continue;
            }
            let bd = Boundary { curve: &x.curve, pc: &c.pc, t: x.t };
            let hits = crossings(ef, f, ef.k_at(ef.tm), k, &bd, &ef.mid.c, 4.0 * r + ed_len(b, ef.e));
            for (_, ball) in hits {
                // The strip's own ball there (not another ball touching the faces with a contact
                // on that edge: a far branch round a torus).
                let t = ed.t[0] + wrap(ef.spine.param_of(&ball.c, ed.t[0]) - ed.t[0], 0.0, period);
                if ef.ball_from(b, &ef.mid, ef.tm, t).is_some_and(|x| (x.c - ball.c).norm() > tol) {
                    continue;
                }
                if is_seam(b, fi, c.edge) {
                    match &seams[k] {
                        Some(s) if (s.c - ball.c).norm() > tol => return Err(unsupported("a closed fillet whose rail crosses its face's seam twice")),
                        _ => seams[k] = Some(ball),
                    }
                } else {
                    let into = beyond(b, c.edge, fi).ok_or_else(|| unsupported("a closed fillet whose rail runs off a sheet's free edge"))?;
                    // Once per point (a rail through a corner of its face crosses two edges).
                    let t = ef.spine.param_of(&ball.c, ed.t[0]);
                    if !cross[k].iter().any(|x| (x.ball.c - ball.c).norm() < tol || (wrap(x.t - t, -PI, period)).abs() < tol::REL * period) {
                        cross[k].push(Crossing { t, ball, into, edge: c.edge });
                    }
                }
            }
        }
        // A seam whose crossing the boundary search missed (the rail runs close along the edge
        // it starts from): where the rail's (u, v) runs past the seam's.
        if seams[k].is_none() {
            seams[k] = seam_crossing(b, ef, k, period)?;
        }
        // (Only a varying radius has crossings the search seeded with one radius misses.)
        if ef.law.is_some() {
            let missed = missed_crossings(b, ef, k, period, &cross[k])?;
            cross[k].extend(missed);
        }
    }
    let cliffs = cross.iter().any(|c| !c.is_empty());
    debug(|| format!("edge {}: seam crossings {:?}, {:?}; cliffs {} {}", ef.e, seams[0].map(|x| (ef.spine.param_of(&x.c, ed.t[0]), x.q[0].p)), seams[1].map(|x| (ef.spine.param_of(&x.c, ed.t[0]), x.q[1].p)), cross[0].len(), cross[1].len()));
    let rail_point = |k: usize, t: f64| -> Result<P3> { Ok(ef.ball_from(b, &ef.mid, ef.tm, t).ok_or_else(|| failed("rolling ball", "the fillet's ball is lost along the edge"))?.q[k].p) };
    let inside = |k: usize, t: f64| -> Result<bool> { Ok(point_in_face(b, ef.faces[k], &rail_point(k, t)?)) };
    // The band's seam: at a face's seam, else at the edge's vertex, else, with cliffs, inside
    // both faces.
    let mut ts = match seams.iter().flatten().next() {
        Some(s) => ef.spine.param_of(&s.c, ed.t[0]),
        None => ed.t[0],
    };
    let clear = |t: f64| -> Result<bool> { Ok(inside(0, t)? && inside(1, t)? && cross.iter().flatten().all(|c| (wrap(c.t - t, -PI, period)).abs() > 1e-3 * period)) };
    if cliffs && !clear(ts)? {
        let mut cands: Vec<f64> = Vec::new();
        for side in &cross {
            let mut cs: Vec<f64> = side.iter().map(|c| c.t).collect();
            cs.sort_by(|a, b| a.total_cmp(b));
            for i in 0..cs.len() {
                let next = if i + 1 < cs.len() { cs[i + 1] } else { cs[0] + period };
                cands.push(0.5 * (cs[i] + next));
            }
        }
        let mut found = None;
        for &t in &cands {
            if clear(t)? {
                found = Some(t);
                break;
            }
        }
        ts = found.ok_or_else(|| unsupported("a closed fillet off its faces all round"))?;
    }
    // A rail off its face all the way round (crossing none of its edges: a round on the rim of a
    // hole nearly as wide as the face it opens in): that face goes whole, and the faces round it
    // cut the strip (SMLib's cliff roll-over, all round). The band's seam is then at a corner of
    // that cut.
    let mut ring: Option<(usize, Vec<(Mark, usize)>)> = None;
    for k in 0..2 {
        if cross[k].is_empty() && seams[k].is_none() && !inside(k, ed.t[0])? {
            if cliffs {
                return Err(unsupported("a closed fillet off one face all round and off the other in part"));
            }
            let corners = ring_corners(b, ef, k, period)?;
            ts = corners[0].0.t;
            if !inside(1 - k, ts)? {
                return Err(unsupported("a closed fillet off both its faces"));
            }
            ring = Some((k, corners));
            break;
        }
    }
    // Along each rail from the seam: where it crosses its face's own seam (on it goes) and the
    // stretches off its face (cut by the face beyond).
    enum Event {
        Seam(f64, Ball),
        Off(Crossing, Crossing),
    }
    let mut events: [Vec<Event>; 2] = [Vec::new(), Vec::new()];
    for k in 0..2 {
        if let Some(sb) = &seams[k] {
            let t = ts + wrap(ef.spine.param_of(&sb.c, ts) - ts, 0.0, period);
            if t - ts > tol::REL * period && ts + period - t > tol::REL * period {
                events[k].push(Event::Seam(t, *sb));
            }
        }
        for c in cross[k].iter_mut() {
            c.t = ts + wrap(c.t - ts, 0.0, period);
        }
        cross[k].sort_by(|a, b| a.t.total_cmp(&b.t));
        let n = cross[k].len();
        if !n.is_multiple_of(2) {
            return Err(unsupported("a closed fillet whose rail leaves its face and doesn't come back"));
        }
        // From the seam (on the face) the rail leaves at the first crossing and comes back at
        // the second, and so on.
        for i in (0..n).step_by(2) {
            let (a, z) = (cross[k][i], cross[k][i + 1]);
            debug(|| format!("edge {}: rail {k} off its face from {} (into {}) to {} (from {})", ef.e, a.t, a.into, z.t, z.into));
            if inside(k, 0.5 * (a.t + z.t))? {
                debug(|| format!("edge {}: rail {k} crossings from ts {ts}: {:?}", ef.e, cross[k].iter().map(|c| (c.t, c.into, c.edge)).collect::<Vec<_>>()));
                return Err(failed("trimming", "a closed fillet's rail crossings don't pair up"));
            }
            events[k].push(Event::Off(a, z));
        }
        events[k].sort_by(|a, b| {
            let t = |e: &Event| match e {
                Event::Seam(t, _) => *t,
                Event::Off(a, _) => a.t,
            };
            t(a).total_cmp(&t(b))
        });
    }
    let ball = ef.ball_from(b, &ef.mid, ef.tm, ts).ok_or_else(|| failed("rolling ball", "the fillet's ball is lost along the edge"))?;
    let st = build_strip(b, ef, &[ts], &[], fit_tol)?;
    let st = &st;
    let mut rv = [0usize; 2];
    let mut fe: Vec<usize> = Vec::new();
    let mut pcurves: Vec<(usize, Curve)> = Vec::new();
    for k in 0..2 {
        if ring.as_ref().is_some_and(|x| x.0 == k) {
            continue;
        }
        let q = ball.q[k].p;
        rv[k] = match vertex_on_boundary(nb, ef.faces[k], &q, tol) {
            Some(v) => v,
            None => nb.add_vertex(q),
        };
        let t0 = st.rail_param(k, &q, ts, ts);
        // A rail that is a point (a sphere strip's pole, [`geom::exact_strip`]): its face goes
        // whole, the strip closed there by a degenerate edge along the pole's line.
        if let (Curve::Circle { r: rr, .. }, Surface::Sphere { f, .. }) = (&st.rails[k], &st.surf)
            && *rr <= tol
        {
            if !events[k].is_empty() {
                return Err(bad_geometry("a fillet's rail is a point on its face's boundary"));
            }
            out.consumed.insert(ef.faces[k]);
            let pole = (q - f.o).dot(&f.z).signum() * std::f64::consts::FRAC_PI_2;
            let d = nb.add_degenerate(q, t0, t0 + period, rv[k]);
            pcurves.push((d, Curve::Line { o: P3::new(0.0, pole, 0.0), d: V3::x() }));
            fe.push(d);
            continue;
        }
        // The rail's p-curve on its face from the strip's contacts (a B-spline strip's), run the
        // way the edge is.
        let on_face = |rail: usize, from: f64, to: f64, out: &mut Pieces| {
            if let Some(on) = &st.on_faces {
                let pc = if to >= from { on[k].clone() } else { on[k].reversed(from, to).0 };
                out.on_face.insert((rail, ef.faces[k]), pc);
            }
        };
        if events[k].is_empty() {
            let rail = nb.add_edge(st.rails[k].clone(), t0, t0 + period, rv[k], rv[k]);
            out.cuts.entry(ef.faces[k]).or_default().push(rail);
            on_face(rail, t0, t0 + period, out);
            fe.push(rail);
            continue;
        }
        let (mut at_v, mut at_t) = (rv[k], t0);
        let rail_to = |nb: &mut Body, to_t: f64, to_v: usize, at_v: usize, at_t: f64, fe: &mut Vec<usize>, out: &mut Pieces| {
            let rail = add_edge(nb, st.rails[k].clone(), at_t, to_t, at_v, to_v);
            out.cuts.entry(ef.faces[k]).or_default().push(rail);
            on_face(rail, at_t, to_t, out);
            fe.push(rail);
        };
        for ev in &events[k] {
            match ev {
                Event::Seam(t, sb) => {
                    let p = sb.q[k].p;
                    let v = vertex_on_boundary(nb, ef.faces[k], &p, tol).ok_or_else(|| failed("trimming", "a fillet rail's seam crossing isn't on its face's seam"))?;
                    let tr = st.rail_param(k, &p, *t, *t);
                    rail_to(nb, tr, v, at_v, at_t, &mut fe, out);
                    (at_v, at_t) = (v, tr);
                }
                Event::Off(a, z) => {
                    let (pa, pz) = (a.ball.q[k].p, z.ball.q[k].p);
                    let va = vertex_on_boundary(nb, ef.faces[k], &pa, tol).ok_or_else(|| failed("trimming", "a fillet rail's crossing isn't on its face's boundary"))?;
                    let vz = vertex_on_boundary(nb, ef.faces[k], &pz, tol).ok_or_else(|| failed("trimming", "a fillet rail's crossing isn't on its face's boundary"))?;
                    let ta = st.rail_param(k, &pa, a.t, a.t);
                    let tz = st.rail_param(k, &pz, z.t, z.t);
                    rail_to(nb, ta, va, at_v, at_t, &mut fe, out);
                    cliff(b, nb, ef, (k, a, va), (k, z, vz), &mut fe, out)?;
                    (at_v, at_t) = (vz, tz);
                }
            }
        }
        rail_to(nb, t0 + period, rv[k], at_v, at_t, &mut fe, out);
    }
    let (c, a0, a1) = ball.arc().ok_or_else(|| bad_geometry("a fillet section degenerates"))?;
    let seam = match &ring {
        None => add_edge(nb, c, a0, a1, rv[0], rv[1]),
        Some((k, corners)) => {
            let vs = ring_cut(b, nb, ef, corners, period, &mut fe, out)?;
            out.consumed.insert(ef.faces[*k]);
            // The section from the other rail to the cut's first corner (a fraction w of the
            // arc's angle from rail 0).
            let am = a0 + (a1 - a0) * corners[0].0.w;
            if *k == 1 { add_edge(nb, c, a0, am, rv[0], vs[0]) } else { add_edge(nb, c, am, a1, vs[0], rv[1]) }
        }
    };
    fe.push(seam);
    fe.push(seam);
    let u_period = matches!(st.kind, StripKind::Nurbs).then_some(period);
    out.new_faces.push(NewFace { surf: st.surf.clone(), edges: fe, src: FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(ef.e as u64) }), u_period, beside: ef.faces[0], pcurves });
    Ok(())
}

/// Where the closed strip of `ef`, its rail k off its face all round, is cut by the faces
/// across that face's outer loop: the strip's points (marks) on the edges between consecutive
/// ones, in order along the strip from the first (spine parameters unwrapped from it, within a
/// turn), each with the face its cut runs over to the next.
fn ring_corners(b: &Body, ef: &EdgeFillet, k: usize, period: f64) -> Result<Vec<(Mark, usize)>> {
    let fi = ef.faces[k];
    let no_ring = || unsupported("a closed fillet off its face all round, not cut by the faces round it");
    let lost = || failed("rolling ball", "the fillet's ball is lost along the edge");
    let mut ls = b.faces[fi].loops.iter().filter(|l| !l.iter().any(|c| c.edge == ef.e));
    let (Some(l), None) = (ls.next(), ls.next()) else { return Err(no_ring()) };
    // The faces round it in loop order, and the vertices where one gives way to the next.
    let around: Vec<usize> = l.iter().map(|c| beyond(b, c.edge, fi).ok_or_else(no_ring)).collect::<Result<_>>()?;
    let n = around.len();
    let turns: Vec<(usize, usize, usize)> = (0..n).filter(|&i| around[i] != around[(i + 1) % n]).map(|i| (around[i], around[(i + 1) % n], b.co_end(&l[i]))).collect();
    if turns.len() < 2 {
        return Err(no_ring());
    }
    // Balls round the strip, to start each corner's solve from the strip's point nearest it.
    let t0 = b.edges[ef.e].t[0];
    let m = 96;
    let mut balls: Vec<(f64, Ball)> = Vec::with_capacity(m);
    let mut cur = (ef.tm, ef.mid);
    for i in 0..m {
        let t = t0 + period * i as f64 / m as f64;
        cur = (t, ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(lost)?);
        balls.push(cur);
    }
    let sd = |g: usize, p: &P3| -> f64 {
        let c = geom::contact(&b.faces[g], geom::foot(&b.faces[g], p, None));
        (p - c.p).dot(&c.n)
    };
    let tol = tol::REL_POINT * (1.0 + ef.r());
    let mut corners: Vec<(Mark, [usize; 2])> = Vec::new();
    for (g, h, v) in turns {
        let x = vertex_edges(b, v).into_iter().find(|&x| !b.edges[x].degenerate && sides(b, x).is_ok_and(|s| [s[0].0, s[1].0] == [g, h] || [s[0].0, s[1].0] == [h, g])).ok_or_else(no_ring)?;
        let xe = &b.edges[x];
        let dist = |p: &P3| (xe.point(xe.curve.closest(p, xe.t[0], xe.t[1])) - p).norm();
        let mut best = (f64::INFINITY, 0usize, 0.5);
        for (i, (_, bl)) in balls.iter().enumerate() {
            for j in 0..=20 {
                let w = j as f64 / 20.0;
                let d = dist(&bl.arc_point(w));
                if d < best.0 {
                    best = (d, i, w);
                }
            }
        }
        // Newton on both faces' signed distances of the strip's point (t, w).
        let (mut t, mut w) = (balls[best.1].0, best.2);
        let mut at = balls[best.1];
        let eval = |t: f64, w: f64, from: &(f64, Ball)| -> Result<(f64, f64, Ball, P3)> {
            let x = ef.ball_from(b, &from.1, from.0, t).ok_or_else(lost)?;
            let p = x.arc_point(w);
            Ok((sd(g, &p), sd(h, &p), x, p))
        };
        let mut found = None;
        for _ in 0..40 {
            let (fg, fh, x, p) = eval(t, w, &at)?;
            at = (t, x);
            if fg.abs().max(fh.abs()) < 1e-10 * (1.0 + ef.r() + p.coords.abs().max()) {
                found = Some(Mark { t, w, ball: x, p });
                break;
            }
            let (dt, dw) = (1e-7 * period, 1e-7);
            let (gt, ht, _, _) = eval(t + dt, w, &at)?;
            let (gw, hw, _, _) = eval(t, w + dw, &at)?;
            let j = [[(gt - fg) / dt, (gw - fg) / dw], [(ht - fh) / dt, (hw - fh) / dw]];
            let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
            if det.abs() < tol::TINY {
                break;
            }
            t -= ((fg * j[1][1] - fh * j[0][1]) / det).clamp(-0.1 * period, 0.1 * period);
            w -= ((j[0][0] * fh - j[1][0] * fg) / det).clamp(-0.25, 0.25);
        }
        // On the strip, and on the edge between the faces (not past its end).
        let mk = found.ok_or_else(no_ring)?;
        if !(-tol::REL..=1.0 + tol::REL).contains(&mk.w) || dist(&mk.p) > tol.max(xe.tol) {
            debug(|| format!("edge {}: ring corner on edge {x} at {:?} (w {}) off it by {:e}", ef.e, mk.p, mk.w, dist(&mk.p)));
            return Err(no_ring());
        }
        corners.push((mk, [g, h]));
    }
    let first = corners[0].0.t;
    for c in corners.iter_mut() {
        c.0.t = first + wrap(c.0.t - first, 0.0, period);
    }
    corners.sort_by(|a, c| a.0.t.total_cmp(&c.0.t));
    // Consecutive corners share the face whose cut runs between them.
    let n = corners.len();
    (0..n)
        .map(|i| {
            let (a, z) = (corners[i].1, corners[(i + 1) % n].1);
            let g = a.into_iter().find(|f| z.contains(f)).ok_or_else(no_ring)?;
            debug(|| format!("edge {}: ring corner at t {} w {} {:?}, on to face {g}", ef.e, corners[i].0.t, corners[i].0.w, corners[i].0.p));
            Ok((corners[i].0, g))
        })
        .collect()
}

/// The cut of a closed strip all round by the faces of [`ring_corners`] (`corners`): each face
/// cut from corner to corner (the cut inside it). Returns the corners' vertices.
fn ring_cut(b: &Body, nb: &mut Body, ef: &EdgeFillet, corners: &[(Mark, usize)], period: f64, fe: &mut Vec<usize>, out: &mut Pieces) -> Result<Vec<usize>> {
    let tol = tol::REL_POINT * (1.0 + ef.r());
    let vs: Vec<usize> = corners.iter().map(|(m, g)| vertex_on_boundary(nb, *g, &m.p, tol.max(tol::POINT)).ok_or_else(|| failed("corner", "a fillet's cut round a corner isn't on the corner's edge"))).collect::<Result<_>>()?;
    let n = corners.len();
    for i in 0..n {
        let (ma, g) = corners[i];
        let mut mz = corners[(i + 1) % n].0;
        if i + 1 == n {
            mz.t += period;
        }
        let mut xs = trace(b, ef, &b.faces[g], &ma, &mz)?;
        let last = xs.len() - 1;
        let (va, vz) = (vs[i], vs[(i + 1) % n]);
        xs[0] = nb.verts[va].p;
        xs[last] = nb.verts[vz].p;
        let (c, t0, t1) = fit_cut(&xs, Some(&b.faces[g]), [None, None]);
        let ce = add_edge(nb, c, t0, t1, va, vz);
        if !inside_face(b, g, &nb.edges[ce]) {
            return Err(unsupported("a closed fillet off its face all round, cut outside the faces round it"));
        }
        out.cuts.entry(g).or_default().push(ce);
        fe.push(ce);
    }
    Ok(vs)
}

/// The fillet of radius `r` along `edges` of a solid by local trimming, with what each of its
/// faces continues (an input face, or a new face from an edge or vertex of `input`).
pub fn fillet_edges(b: &Body, input: BodyId, edges: &[usize], r: f64) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    let laws: Vec<RadiusLaw> = edges.iter().map(|&e| (e, vec![(0.0, r), (1.0, r)])).collect();
    fillet_edges_laws(b, input, &laws, false)
}

/// An edge and its radius law: (fraction along the edge, radius) points, linear between.
pub type RadiusLaw = (usize, Vec<(f64, f64)>);

/// The radius of the ball whose contacts lie `d` from edge e along both its faces, if the
/// faces meet at the same angle all along it (a chamfer through those contacts is then the
/// equal-distance chamfer): d·tan(θ/2), θ between the faces' directions away from the edge.
pub fn chamfer_radius(b: &Body, e: usize, d: f64) -> Option<f64> {
    let th = chamfer_angle(b, e)?;
    (th > tol::ANGULAR_POINT && th < PI - tol::ANGULAR_POINT).then(|| d * (th / 2.0).tan())
}

/// The offsets from edge e's two faces (in [`super::blend::edge_sides`] order) of the ball
/// whose contacts lie `d[0]` and `d[1]` from the edge along them, if the faces meet at the same
/// angle θ all along it: in the section, the contact d₀ along the first face is the foot of the
/// point k₀ off it, and k₀ = (d₁ − d₀ cos θ) / sin θ (k₁ likewise); both positive.
pub fn chamfer_offsets(b: &Body, e: usize, d: [f64; 2]) -> Option<[f64; 2]> {
    let th = chamfer_angle(b, e)?;
    if !(th > tol::ANGULAR_POINT && th < PI - tol::ANGULAR_POINT) {
        return None;
    }
    let (sn, cs) = th.sin_cos();
    let k = [(d[1] - d[0] * cs) / sn, (d[0] - d[1] * cs) / sn];
    (k[0] > 0.0 && k[1] > 0.0).then_some(k)
}

/// The angle between edge e's faces' directions away from it, if the same all along it.
pub(crate) fn chamfer_angle(b: &Body, e: usize) -> Option<f64> {
    let mut theta: Option<f64> = None;
    for s in [0.2, 0.5, 0.8] {
        let th = angle_at(b, e, s)?;
        if theta.is_some_and(|x| (x - th).abs() > tol::ANGULAR) {
            return None;
        }
        theta = Some(th);
    }
    theta
}

/// The radius law of a fillet of width `w` (its contacts that far apart) along edge e, whose
/// faces' angle varies along it: r = w / (2 sin(φ/2)), φ between the faces' normals, at
/// samples along the edge (linear between).
pub(crate) fn width_law(b: &Body, e: usize, w: f64) -> Option<Vec<(f64, f64)>> {
    let n = 16;
    (0..=n)
        .map(|i| {
            let s = i as f64 / n as f64;
            // Just inside the ends (an end's p-curves may stop short of its vertex).
            let th = angle_at(b, e, s.clamp(tol::REL_POINT, 1.0 - tol::REL_POINT))?;
            let phi = PI - th;
            (phi > tol::ANGULAR_POINT && phi < PI - tol::ANGULAR_POINT).then(|| (s, w / (2.0 * (phi / 2.0).sin())))
        })
        .collect()
}

/// The radius law of an equal-distance chamfer `d` along edge e, whose faces' angle θ varies
/// along it: the ball whose contacts lie d from the edge, r = d tan(θ/2), at samples along the
/// edge (linear between).
pub(crate) fn chamfer_law(b: &Body, e: usize, d: f64) -> Option<Vec<(f64, f64)>> {
    let n = 16;
    (0..=n)
        .map(|i| {
            let s = i as f64 / n as f64;
            let th = angle_at(b, e, s.clamp(tol::REL_POINT, 1.0 - tol::REL_POINT))?;
            (th > tol::ANGULAR_POINT && th < PI - tol::ANGULAR_POINT).then(|| (s, d * (th / 2.0).tan()))
        })
        .collect()
}

/// The offsets from straight edge e's two faces (as [`chamfer_offsets`]) of the ball whose
/// contacts lie `d[0]` and `d[1]` along the faces from the edge, measured round a face that
/// curves across it (as the blend tool's section is, [`super::blend::along_face`]), where the
/// faces don't change along the edge (planes, cylinders and extrusions along it): the centre
/// on both contacts' normals, C = q₀ + k₀n₀ = q₁ + k₁n₁.
pub fn chamfer_offsets_along(b: &Body, e: usize, d: [f64; 2]) -> Option<[f64; 2]> {
    let ed = b.edges.get(e)?;
    let Curve::Line { d: dir, .. } = &ed.curve else { return None };
    let [(f1, c1), (f2, _)] = sides(b, e).ok()?;
    let spine = Spine { curve: ed.curve.clone(), t: ed.t, closed: false };
    if geom::exact_kind([&b.faces[f1], &b.faces[f2]], &spine) != Some(Exact::Cylinder) {
        return None;
    }
    let t = ed.mid_t();
    let p = ed.point(t);
    let tan = dir.normalize() * if c1 { 1.0 } else { -1.0 };
    let into = |fi: usize, along: V3| -> Option<V3> {
        let c = coedge_of(b, fi, e)?;
        let q = c.pc.point(t);
        geom::contact(&b.faces[fi], P2::new(q.x, q.y)).n.cross(&along).try_normalize(tol::DEGENERATE)
    };
    let q = [super::blend::along_face(&b.faces[f1], &p, &tan, &into(f1, tan)?, d[0]), super::blend::along_face(&b.faces[f2], &p, &tan, &into(f2, -tan)?, d[1])];
    let n: [V3; 2] = std::array::from_fn(|k| {
        let f = &b.faces[[f1, f2][k]];
        geom::contact(f, geom::foot(f, &q[k], None)).n
    });
    // Least squares on a n₀ − b n₁ = q₁ − q₀ (the two lines meet in the section's plane).
    let r = q[1] - q[0];
    let m = nalgebra::Matrix2::new(1.0, -n[0].dot(&n[1]), -n[0].dot(&n[1]), 1.0);
    let x = m.lu().solve(&nalgebra::Vector2::new(n[0].dot(&r), -n[1].dot(&r)))?;
    let miss = (q[0] + n[0] * x.x - q[1] - n[1] * x.y).norm();
    (x.x * x.y > 0.0 && miss < tol::REL * (1.0 + p.coords.abs().max())).then(|| [x.x.abs(), x.y.abs()])
}

/// The angle between edge e's faces' directions away from it, a fraction `s` along it.
fn angle_at(b: &Body, e: usize, s: f64) -> Option<f64> {
    let ed = b.edges.get(e)?;
    let [(f1, c1), (f2, _)] = sides(b, e).ok()?;
    let t = ed.t[0] + (ed.t[1] - ed.t[0]) * s;
    let dir = ed.curve.tangent(t).normalize() * if c1 { 1.0 } else { -1.0 };
    let w = |fi: usize, along: V3| -> Option<V3> {
        let c = coedge_of(b, fi, e)?;
        let q = c.pc.point(t);
        Some(geom::contact(&b.faces[fi], P2::new(q.x, q.y)).n.cross(&along))
    };
    Some(w(f1, dir)?.angle(&w(f2, -dir)?))
}

/// The fillet along each edge with its radius law (fraction along the edge, radius; linear
/// between), by local trimming; `flat`: a chamfer through the ball's contacts.
pub fn fillet_edges_laws(b: &Body, input: BodyId, edges: &[RadiusLaw], flat: bool) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    local_blend(b, input, edges, &vec![1.0; edges.len()], flat)
}

/// The chamfer of each edge with the distances `d` of its contacts from it along its two faces
/// (in [`super::blend::edge_sides`] order), by local trimming: the ball whose offsets from the
/// faces put its contacts there ([`chamfer_offsets`]), cut straight across.
pub fn chamfer_edges(b: &Body, input: BodyId, edges: &[(usize, [f64; 2])]) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    let mut laws = Vec::new();
    let mut ratios = Vec::new();
    for &(e, d) in edges {
        // Unequal distances on a face curving across the edge: measured round it.
        let [(f1, _), (f2, _)] = sides(b, e)?;
        let curved = ![f1, f2].iter().all(|&f| matches!(b.faces[f].surf, Surface::Plane { .. }));
        let k = if curved && d[0] != d[1] { chamfer_offsets_along(b, e, d) } else { chamfer_offsets(b, e, d) };
        let k = k.ok_or_else(|| unsupported("a chamfer along an edge whose faces don't meet at one angle"))?;
        // In the strip's face order (the face whose loop runs along the edge first).
        let [(_, c1), _] = sides(b, e)?;
        let k = if c1 { k } else { [k[1], k[0]] };
        laws.push((e, vec![(0.0, k[0]), (1.0, k[0])]));
        ratios.push(k[1] / k[0]);
    }
    local_blend(b, input, &laws, &ratios, true)
}

/// A face of a closed surface may come with a hole whose loop crosses the surface's seam
/// without the seam edge being split there: the seam then runs through the hole, and the
/// loop's p-curves jump a period where it touches the seam (Dr Browns cap: a window cut into a
/// dome across its seam). Trimming can't place cuts in such a face. This opens each such hole
/// into the outer loop: the seam edge is cut where the hole touches it (its piece across the
/// hole dropped) and the hole's two halves join the seam's two sides, each in the period it
/// lies against. Edge indices stay as they were (the seam's far piece is a new edge). `None`
/// when no face needs it.
fn open_seam_holes(b: &Body) -> Option<Body> {
    let mut out: Option<Body> = None;
    for fi in 0..b.faces.len() {
        let cur = out.as_ref().unwrap_or(b);
        let periods = super::pcurve::closed_periods(&cur.faces[fi].surf);
        for (d, p) in periods.iter().enumerate() {
            let Some(p) = *p else { continue };
            let cur = out.as_ref().unwrap_or(b);
            if let Some(nb) = open_seam_hole(cur, fi, d, p) {
                debug(|| format!("face {fi}: a hole across its seam opened into its outer loop"));
                out = Some(nb);
            }
        }
    }
    out
}

/// [`open_seam_holes`] for one hole of face `fi` crossing its seam in parameter `d` (period
/// `p`).
fn open_seam_hole(b: &Body, fi: usize, d: usize, p: f64) -> Option<Body> {
    let f = &b.faces[fi];
    let uv = |c: &Coedge, end: bool| -> P2 {
        let (t0, t1) = super::topo::co_range(&b.edges[c.edge], c.fwd);
        let q = c.pc.point(if end { t1 } else { t0 });
        P2::new(q.x, q.y)
    };
    let seams: Vec<usize> = f.loops.iter().flatten().map(|c| c.edge).filter(|&e| is_seam(b, fi, e)).collect();
    for (li, l) in f.loops.iter().enumerate() {
        if l.iter().any(|c| seams.contains(&c.edge)) || l.len() < 2 {
            continue;
        }
        let n = l.len();
        // The seam's sides in d: the face's period window starts at the lower.
        let lo = f.loops.iter().flatten().filter(|c| seams.contains(&c.edge)).map(|c| uv(c, false)[d]).fold(f64::INFINITY, f64::min);
        if !lo.is_finite() {
            continue;
        }
        // The period window each coedge lies in (by its middle); where it changes, after coedge
        // i, the loop crosses the seam (at a vertex: the loop's p-curves run on continuously).
        let window = |c: &Coedge| -> f64 {
            let e = &b.edges[c.edge];
            ((c.pc.point(0.5 * (e.t[0] + e.t[1]))[d] - lo) / p).floor()
        };
        let w: Vec<f64> = l.iter().map(window).collect();
        let jumps: Vec<(usize, f64)> = (0..n).filter_map(|i| (w[(i + 1) % n] != w[i]).then_some((i, w[(i + 1) % n] - w[i]))).collect();
        if jumps.len() != 2 || jumps[0].1 != -jumps[1].1 {
            continue;
        }
        let (ja, jb) = (b.co_end(&l[jumps[0].0]), b.co_end(&l[jumps[1].0]));
        // The seam edge through both: its parameters there.
        let (pa, pb) = (b.verts[ja].p, b.verts[jb].p);
        let s = *seams.iter().find(|&&e| {
            let ed = &b.edges[e];
            [pa, pb].iter().all(|q| {
                let t = ed.curve.closest(q, ed.t[0], ed.t[1]);
                (ed.point(t) - q).norm() <= tol::REL_POINT * (1.0 + q.coords.abs().max()) && t > ed.t[0] && t < ed.t[1]
            })
        })?;
        let se = &b.edges[s];
        let (ta, tb) = (se.curve.closest(&pa, se.t[0], se.t[1]), se.curve.closest(&pb, se.t[0], se.t[1]));
        let ((t_lo, v_lo), (t_hi, v_hi)) = if ta < tb { ((ta, ja), (tb, jb)) } else { ((tb, jb), (ta, ja)) };
        // The hole's halves: from after the first jump to the second, and on round.
        let half = |from: usize, to: usize| -> Vec<Coedge> {
            let mut v = Vec::new();
            let mut i = (from + 1) % n;
            loop {
                v.push(l[i].clone());
                if i == to {
                    break;
                }
                i = (i + 1) % n;
            }
            v
        };
        let halves = [half(jumps[0].0, jumps[1].0), half(jumps[1].0, jumps[0].0)];
        // The seam's two sides (their coedges in the outer loops) and where each lies in d.
        let sides: Vec<(usize, usize, f64)> = f
            .loops
            .iter()
            .enumerate()
            .flat_map(|(lj, ll)| ll.iter().enumerate().filter(|(_, c)| c.edge == s).map(move |(ci, c)| (lj, ci, c)))
            .map(|(lj, ci, c)| (lj, ci, uv(c, false)[d]))
            .collect();
        if sides.len() != 2 || sides.iter().any(|x| x.0 == li) {
            continue;
        }
        // Each half shifted into the face's period, and the side it lies against.
        let mut placed: Vec<(usize, Vec<Coedge>)> = Vec::new();
        for h in halves {
            let m = h[h.len() / 2].pc.point(0.5 * (b.edges[h[h.len() / 2].edge].t[0] + b.edges[h[h.len() / 2].edge].t[1]))[d];
            let shift = -((m - lo) / p).floor() * p;
            let mut dv = V3::zeros();
            dv[d] = shift;
            let h: Vec<Coedge> = h.into_iter().map(|c| Coedge { pc: if shift != 0.0 { c.pc.transformed(&Affine::translation(dv)) } else { c.pc }, ..c }).collect();
            let at = uv(&h[0], false)[d];
            let side = sides.iter().position(|x| (x.2 - at).abs() < 1e-6 * p)?;
            if placed.iter().any(|x| x.0 == side) {
                return None;
            }
            placed.push((side, h));
        }
        let mut nb = b.clone();
        let s2 = nb.add_edge(se.curve.clone(), t_hi, se.t[1], v_hi, se.v[1]);
        nb.edges[s2].tol = se.tol;
        nb.edges[s].t = [se.t[0], t_lo];
        nb.edges[s].v = [se.v[0], v_lo];
        // Each side: its seam coedge becomes the near piece, the half, the far piece.
        let mut edits: Vec<(usize, usize, Vec<Coedge>)> = Vec::new();
        for (side, h) in placed {
            let (lj, ci, _) = sides[side];
            let c = &f.loops[lj][ci];
            let (first, last) = if c.fwd { (s, s2) } else { (s2, s) };
            let want = if c.fwd { (v_lo, v_hi) } else { (v_hi, v_lo) };
            if (nb.co_start(&h[0]), nb.co_end(&h[h.len() - 1])) != want {
                return None;
            }
            let mut seq = vec![Coedge { edge: first, fwd: c.fwd, pc: c.pc.clone() }];
            seq.extend(h);
            seq.push(Coedge { edge: last, fwd: c.fwd, pc: c.pc.clone() });
            edits.push((lj, ci, seq));
        }
        // Splice from the back so indices hold.
        edits.sort_by_key(|x| std::cmp::Reverse((x.0, x.1)));
        for (lj, ci, seq) in edits {
            nb.faces[fi].loops[lj].splice(ci..=ci, seq);
        }
        nb.faces[fi].loops.remove(li);
        return Some(nb);
    }
    None
}

/// [`fillet_edges_laws`] with each edge's [`EdgeFillet::ratio`].
fn local_blend(b: &Body, input: BodyId, edges: &[RadiusLaw], ratios: &[f64], flat: bool) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    if edges.iter().any(|(e, law)| *e >= b.edges.len() || law.is_empty()) {
        return Err(KernelError::InvalidParameter("unknown edge or empty radius law".into()));
    }
    let fixed = open_seam_holes(b);
    let b = fixed.as_ref().unwrap_or(b);
    let budget = Budget::new(BALL_SOLVES);
    let r = edges.iter().zip(ratios).flat_map(|((_, l), q)| l.iter().map(move |x| x.1 * q.max(1.0))).fold(0.0, f64::max);
    let size = b.bbox().diagonal().max(r);
    // B-spline strips within 1e-5 of the size (their sections exact at the stations): well inside
    // the volume and area agreement wanted, with spans few enough to integrate quickly.
    let fit_tol = 1e-5 * (1.0 + size);
    let vtol = tol::REL_POINT * (1.0 + r);
    let out_of_work = || out_of_budget("the local fillet takes more work than its budget");
    let p = plan(b, edges, ratios, flat, &budget).map_err(|e| if budget.spent() { out_of_work() } else { e })?;
    debug(|| format!("planned {} strips, {} corners", p.efs.len(), p.corners.len()));
    // Rails running off their faces mid-strip are cut by the faces beyond; where that fails,
    // the strips as they are (the rail may only graze an edge).
    let r = trim(b, input, &p, size, fit_tol, vtol, true).or_else(|e| {
        if budget.spent() {
            return Err(out_of_work());
        }
        debug(|| format!("with cliffs: {e}; again without"));
        // Only where the rails really stay on their faces: a rail across one of the face's
        // loops (a hole crossing the strip) leaves loops crossing, which only the full check
        // sees.
        let (out, src) = trim(b, input, &p, size, fit_tol, vtol, false).map_err(|e| if budget.spent() { out_of_work() } else { e })?;
        // (The input checked only when the result fails the full check: its own defects carry
        // over, and then the fast check is what holds.)
        let problem = match out.check_with(CheckLevel::Full).into_iter().next() {
            Some(x) if b.check_with(CheckLevel::Full).is_empty() => Some(x),
            Some(_) => out.check_with(CheckLevel::Fast).into_iter().next(),
            None => None,
        };
        match problem {
            Some(x) => {
                debug(|| format!("without cliffs: {x}"));
                Err(e)
            }
            None => Ok((out, src)),
        }
    });
    debug(|| format!("ball solves: {}", BALL_SOLVES - budget.0.get()));
    r
}

/// The strips, ends and corners of plan `p` put into `b`'s faces (`cliffs`: rails running off
/// their faces mid-strip cut by the faces beyond).
#[allow(clippy::too_many_arguments)]
fn trim(b: &Body, input: BodyId, p: &Plan, size: f64, fit_tol: f64, vtol: f64, cliffs: bool) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    let mut nb = b.clone();
    let mut out = Pieces::default();
    for (i, ef) in p.efs.iter().enumerate() {
        if p.inside[i] {
            continue;
        }
        // The spine parameters the strip must cover and, where an end face isn't met in closed
        // form, the marched end curves.
        let mut cover = Vec::new();
        // The sections at pinched ends, exact there.
        let mut pinned: Vec<(f64, Ball)> = Vec::new();
        let mut marched: [Option<Vec<(f64, P3)>>; 2] = [None, None];
        for j in 0..2 {
            match &p.ends[i][j] {
                Some(End::Free { exits }) => {
                    if exits[0].into != exits[1].into {
                        return Err(unsupported("a fillet whose rails end on different faces"));
                    }

                    cover.extend([exits[0].t, exits[1].t]);
                    if ef.exact.is_none() {
                        let pts = march_end(b, ef, exits, &b.faces[exits[0].into])?;
                        cover.extend(pts.iter().map(|x| x.0));
                        marched[j] = Some(pts);
                    }
                }
                Some(End::Section { t, .. }) => cover.push(*t),
                Some(End::Patch { t, .. }) => cover.extend(t.iter().copied()),
                Some(End::Hole { t, .. }) => cover.push(*t),
                Some(End::Knee { t, .. }) => cover.extend(t.iter().copied()),
                Some(End::Star { t, t_p, .. }) => cover.extend([t[0], t[1], *t_p]),
                Some(End::Square { t, balls, n }) => {
                    cover.extend(t.iter().copied());
                    let pts = march_on(b, ef, &Mark::rail(t[0], balls[0], 0), &Mark::rail(t[1], balls[1], 1), &square_plane(&b.verts[b.edges[ef.e].v[j]].p, n))?;
                    cover.extend(pts.iter().map(|x| x.0));
                    marched[j] = Some(pts);
                }
                Some(End::Open { exits, .. }) => cover.extend([exits[0].t, exits[1].t]),
                Some(End::Point { t }) => {
                    cover.push(*t);
                    pinned.extend(pinch(b, ef, j).map(|x| (*t, x)));
                }
                Some(End::Chain { exits, links }) => {
                    cover.extend([exits[0].t, exits[1].t]);
                    cover.extend(links.iter().map(|l| l.mark.t));
                }
                Some(End::Mitre { t_s, other, .. }) => {
                    cover.push(*t_s);
                    match other {
                        MitreEnd::AtEnd { t } => cover.push(*t),
                        MitreEnd::Exit { exit, pts, .. } => {
                            cover.push(exit.t);
                            cover.extend(pts.iter().map(|x| x.0));
                        }
                    }
                }
                None if !ef.closed => return Err(failed("end", "a fillet end without a plan")),
                None => {}
            }
        }
        if ef.closed {
            closed_strip(b, &mut nb, ef, fit_tol, &mut out, input)?;
            continue;
        }
        let mut st = build_strip(b, ef, &cover, &pinned, fit_tol)?;
        debug(|| format!("edge {}: {:?} strip, ends {:?}", ef.e, st.kind, p.ends[i].iter().map(|e| e.as_ref().map(|e| matches!(e, End::Free { .. }))).collect::<Vec<_>>()));
        // Each end: the rails' end vertices and the edge(s) across the strip.
        let mut rv = [[0usize; 2]; 2]; // [end][side]
        let mut rt = [[0.0f64; 2]; 2];
        let mut across: Vec<usize> = Vec::new();
        let mut given: Vec<(usize, Curve)> = Vec::new();
        for j in 0..2 {
            let Some(end) = &p.ends[i][j] else { continue };
            match end {
                End::Point { t } => {
                    // Both rails end at the vertex; the strip's section there is the point, a
                    // pole of its surface.
                    let v = b.edges[ef.e].v[j];
                    let q = b.verts[v].p;
                    for k in 0..2 {
                        rv[j][k] = v;
                        rt[j][k] = st.rail_param(k, &q, *t, *t);
                    }
                    let deg = nb.add_degenerate(q, 0.0, 1.0, v);
                    given.push((deg, Curve::Line { o: P3::new(*t, 0.0, 0.0), d: V3::new(0.0, 1.0, 0.0) }));
                    across.push(deg);
                }
                End::Free { exits } => {
                    let v = b.edges[ef.e].v[j];
                    let mut ext: Option<(usize, usize)> = None; // (side, extension edge)
                    // (For an extension from the edge's far end: that end and the edge.)
                    let mut far_end: Option<(usize, usize)> = None;
                    for k in 0..2 {
                        let x = &exits[k];
                        let q = x.ball.q[k].p;
                        rv[j][k] = match x.ext {
                            None => vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("end", "a fillet rail's end isn't on its face's boundary"))?,
                            Some((y, ty)) => {
                                // A new vertex inside the face, joined to the vertex along the edge's
                                // extension; on its boundary (the rail running along one of its
                                // edges there, the strip as wide as the face) the edge split there.
                                let pv = match vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol) {
                                    Some(x) => x,
                                    None => nb.add_vertex(q),
                                };
                                // (Past the edge's far end: from that end.)
                                let ye = &b.edges[y];
                                let iv = if ye.v[0] == v { 0 } else { 1 };
                                let far = if iv == 0 { ty > ye.t[1] } else { ty < ye.t[0] };
                                let (w, tv) = if far { (ye.v[1 - iv], ye.t[1 - iv]) } else { (v, ye.t[iv]) };
                                let xe = add_edge(&mut nb, ye.curve.clone(), tv, ty, w, pv);
                                out.cuts.entry(ef.faces[k]).or_default().push(xe);
                                ext = Some((k, xe));
                                far_end = far.then_some((w, y));
                                pv
                            }
                        };
                        rt[j][k] = st.rail_param(k, &q, x.t, x.t);
                    }
                    let g = exits[0].into;
                    let (pa, pb) = (nb.verts[rv[j][0]].p, nb.verts[rv[j][1]].p);
                    let corner = b.verts[v].p;
                    let ee = match exact_end_curve(&st.surf, &b.faces[g].surf, &pa, &pb, &corner) {
                        Some((c, ta, tb)) => add_edge(&mut nb, c, ta, tb, rv[j][0], rv[j][1]),
                        None => {
                            let pts = match &marched[j] {
                                Some(pts) => pts.clone(),
                                None => march_end(b, ef, exits, &b.faces[g])?,
                            };
                            let mut xs: Vec<P3> = pts.iter().map(|x| x.1).collect();
                            let last = xs.len() - 1;
                            xs[0] = pa;
                            xs[last] = pb;
                            let (c, c0, c1) = fit_cut(&xs, Some(&b.faces[g]), [Some(exits[0].ball.c), Some(exits[1].ball.c)]);
                            add_edge(&mut nb, c, c0, c1, rv[j][0], rv[j][1])
                        }
                    };
                    match ext.map(|(k, xe)| (k, xe, far_end)) {
                        // The extension from the far end of the edge between the rail's face and
                        // the end face: the end curve cuts the end face up to where it crosses
                        // the face's next edge there, and past that a face on the end face's
                        // surface closes the strip's end, between the end curve, the extension
                        // and that edge.
                        Some((k, xe, Some((w, y)))) => {
                            let o = 1 - k;
                            let (ein, eout, q) = end_over_edge(b, &mut nb, g, ee, rv[j][o], w, y, vtol).ok_or_else(|| unsupported("a fillet end past a short edge that doesn't cross the end face's next edge"))?;
                            if !inside_face(b, g, &nb.edges[ein]) {
                                return Err(unsupported("a fillet end past a short edge outside its end face"));
                            }
                            debug(|| format!("edge {}: end {j} runs past edge {y} of end face {g}, over its next edge at {:?}", ef.e, nb.verts[q].p));
                            out.cuts.entry(g).or_default().push(ein);
                            let piece = nb.faces[g].loops.iter().flatten().map(|c| c.edge).find(|&x| nb.edges[x].v == [w, q] || nb.edges[x].v == [q, w]).ok_or_else(|| failed("end", "a fillet end's crossing of its end face's edge isn't on it"))?;
                            out.new_faces.push(NewFace { surf: b.faces[g].surf.clone(), edges: vec![eout, xe, piece], src: FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(ef.e as u64) }), u_period: None, beside: g, pcurves: Vec::new() });
                            across.push(ein);
                            across.push(eout);
                            continue;
                        }
                        // The end face grows by the region the end curve and the extension
                        // close off at the vertex (in place of its edge from the other rail's exit
                        // to the vertex).
                        Some((k, xe, None)) => {
                            if inside_face(b, g, &nb.edges[ee]) {
                                return Err(unsupported("a mixed fillet end inside its end face"));
                            }
                            let o = 1 - k;
                            // From the other rail's exit along the end curve, then back along
                            // the extension to the vertex.
                            let path = vec![(ee, o == 0), (xe, false)];
                            out.patches.push(Patch { face: g, from: rv[j][o], to: v, path, via: None, stretch: Vec::new() });
                        }
                        // Inside the end face the fillet cuts it; outside, it adds to it (in
                        // place of the face's boundary round the edge's vertex).
                        None if inside_face(b, g, &nb.edges[ee]) => out.cuts.entry(g).or_default().push(ee),
                        None => out.patches.push(Patch { face: g, from: rv[j][0], to: rv[j][1], path: vec![(ee, true)], via: Some(v), stretch: Vec::new() }),
                    }
                    across.push(ee);
                }
                End::Open { exits, pts } => {
                    for k in 0..2 {
                        let x = &exits[k];
                        let q = x.ball.q[k].p;
                        rv[j][k] = vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("end", "a fillet rail's end isn't on its face's boundary"))?;
                        rt[j][k] = st.rail_param(k, &q, x.t, x.t);
                    }
                    let mut xs = pts.clone();
                    let last = xs.len() - 1;
                    xs[0] = nb.verts[rv[j][0]].p;
                    xs[last] = nb.verts[rv[j][1]].p;
                    let (c, c0, c1) = fit_cut(&xs, None, [None, None]);
                    across.push(add_edge(&mut nb, c, c0, c1, rv[j][0], rv[j][1]));
                }
                End::Chain { exits, links } => {
                    for k in 0..2 {
                        let x = &exits[k];
                        let q = x.ball.q[k].p;
                        rv[j][k] = vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("end", "a fillet rail's end isn't on its face's boundary"))?;
                        rt[j][k] = st.rail_param(k, &q, x.t, x.t);
                    }
                    // One cut on each face round the vertex, from mark to mark; where the faces
                    // meet past the vertex, an edge from it along their edge's extension (`ext`).
                    let v = b.edges[ef.e].v[j];
                    let mut marks = vec![(Mark::rail(exits[0].t, exits[0].ball, 0), rv[j][0], exits[0].into)];
                    let mut ext: Vec<Option<usize>> = Vec::new();
                    for l in links {
                        if l.past {
                            let xv = nb.add_vertex(l.mark.p);
                            let d = common_edge(b, marks[marks.len() - 1].2, l.after).filter(|&d| b.edges[d].v.contains(&v)).ok_or_else(|| failed("end", "a fillet end's faces don't meet at its vertex"))?;
                            let de = &b.edges[d];
                            let tx = de.curve.closest(&l.mark.p, de.t[0] - 1e6, de.t[1] + 1e6);
                            let tv = if de.v[0] == v { de.t[0] } else { de.t[1] };
                            ext.push(Some(add_edge(&mut nb, de.curve.clone(), tv, tx, v, xv)));
                            marks.push((l.mark, xv, l.after));
                        } else {
                            let xv = vertex_on_boundary(&mut nb, l.after, &l.mark.p, vtol).ok_or_else(|| failed("end", "a fillet end's crossing isn't on its edge"))?;
                            ext.push(None);
                            marks.push((l.mark, xv, l.after));
                        }
                    }
                    marks.push((Mark::rail(exits[1].t, exits[1].ball, 1), rv[j][1], usize::MAX));
                    let mut cuts: Vec<(usize, usize)> = Vec::new(); // (face, cut)
                    for w in marks.windows(2) {
                        let ((ma, va, g), (mz, vz, _)) = (&w[0], &w[1]);
                        // A crossing on the second rail, which runs on to its exit along an edge
                        // of its face (between it and this face): that edge is the strip's.
                        if (ma.w - 1.0).abs() < tol::REL_POINT && mz.w == 1.0 && va != vz {
                            let along = (0..nb.edges.len()).find(|&x| !nb.edges[x].degenerate && (nb.edges[x].v == [*va, *vz] || nb.edges[x].v == [*vz, *va]));
                            if let Some(x) = along {
                                debug(|| format!("edge {}: rail 1 ends along edge {x}", ef.e));
                                across.push(x);
                                continue;
                            }
                        }
                        let pts = march_on(b, ef, ma, mz, &b.faces[*g])?;
                        let mut xs: Vec<P3> = pts.iter().map(|x| x.1).collect();
                        let last = xs.len() - 1;
                        xs[0] = nb.verts[*va].p;
                        xs[last] = nb.verts[*vz].p;
                        let ca = (ma.w == 0.0).then_some(ma.ball.c);
                        let cz = (mz.w == 1.0).then_some(mz.ball.c);
                        let (c, c0, c1) = fit_cut(&xs, Some(&b.faces[*g]), [ca, cz]);
                        let ee = add_edge(&mut nb, c, c0, c1, *va, *vz);
                        cuts.push((*g, ee));
                        across.push(ee);
                    }
                    let inside: Vec<bool> = cuts.iter().map(|&(g, ee)| inside_face(b, g, &nb.edges[ee])).collect();
                    let last = cuts.len() - 1;
                    for (i, &(g, ee)) in cuts.iter().enumerate() {
                        let before = if i > 0 { ext[i - 1] } else { None };
                        let after = if i < last { ext[i] } else { None };
                        if inside[i] {
                            // Inside its face the cut splits it (the faces meeting at the edges
                            // between them, not past the vertex).
                            if before.is_some() || after.is_some() {
                                return Err(unsupported("a fillet end over several faces outside them"));
                            }
                            out.cuts.entry(g).or_default().push(ee);
                            continue;
                        }
                        // Outside, the face grows by the region between its cut and the vertex
                        // (through the extensions where the faces meet past it).
                        let mut path = Vec::new();
                        let mut from = marks[i].1;
                        if let Some(x) = before {
                            from = v;
                            path.push((x, true));
                        }
                        path.push((ee, true));
                        let mut to = marks[i + 1].1;
                        if let Some(x) = after {
                            path.push((x, false));
                            to = v;
                        }
                        if i == last && g != exits[1].into {
                            // The cut ran to the other rail's exit on this face: back to the
                            // vertex along the edge that exit is on.
                            let back = (0..nb.edges.len()).find(|&x| !nb.edges[x].degenerate && (nb.edges[x].v == [to, v] || nb.edges[x].v == [v, to])).ok_or_else(|| unsupported("a fillet end over several faces outside them"))?;
                            path.push((back, nb.edges[back].v[0] == to));
                            to = v;
                        }
                        out.patches.push(Patch { face: g, from, to, path, via: (from != v && to != v).then_some(v), stretch: Vec::new() });
                    }
                }
                End::Patch { c, t, balls, pts } => {
                    let v = p.patches[*c].v;
                    for k in 0..2 {
                        let fi = ef.faces[k];
                        let q = balls[k].q[k].p;
                        let vid = match out.contact_verts.get(&(v, fi)) {
                            Some(&x) => x,
                            None => {
                                let x = nb.add_vertex(q);
                                out.contact_verts.insert((v, fi), x);
                                x
                            }
                        };
                        rv[j][k] = vid;
                        rt[j][k] = st.rail_param(k, &q, t[k], t[k]);
                    }
                    // Three or four strips: one face fills the corner, bounded by the curves
                    // across whole.
                    if p.patches[*c].strips.len() <= 4 {
                        let mut xs = pts.clone();
                        let last = xs.len() - 1;
                        xs[0] = nb.verts[rv[j][0]].p;
                        xs[last] = nb.verts[rv[j][1]].p;
                        let (c0, a0, z0) = fit_cut(&xs, None, [None, None]);
                        let e0 = add_edge(&mut nb, c0, a0, z0, rv[j][0], rv[j][1]);
                        out.patch_halves.insert((v, i), (e0, e0, usize::MAX));
                        across.push(e0);
                        continue;
                    }
                    // The curve across, in two halves (the patches meet at its middle).
                    let h = pts.len() / 2;
                    let mid = nb.add_vertex(pts[h]);
                    let mut first = pts[..=h].to_vec();
                    first[0] = nb.verts[rv[j][0]].p;
                    let mut second = pts[h..].to_vec();
                    let last = second.len() - 1;
                    second[last] = nb.verts[rv[j][1]].p;
                    let (c0, a0, z0) = fit_cut(&first, None, [None, None]);
                    let (c1, a1, z1) = fit_cut(&second, None, [None, None]);
                    let e0 = add_edge(&mut nb, c0, a0, z0, rv[j][0], mid);
                    let e1 = add_edge(&mut nb, c1, a1, z1, mid, rv[j][1]);
                    out.patch_halves.insert((v, i), (e0, e1, mid));
                    across.push(e0);
                    across.push(e1);
                }
                End::Mitre { m, t_s, other } => {
                    let mi = &p.mitres[*m];
                    let v = mi.v;
                    let ks = if ef.faces[0] == mi.shared { 0 } else { 1 };
                    let ko = 1 - ks;
                    let ps = match out.contact_verts.get(&(v, mi.shared)) {
                        Some(&x) => x,
                        None => {
                            let x = nb.add_vertex(mi.ps);
                            out.contact_verts.insert((v, mi.shared), x);
                            x
                        }
                    };
                    rv[j][ks] = ps;
                    rt[j][ks] = st.rail_param(ks, &mi.ps, *t_s, *t_s);
                    let qv = match out.contact_verts.get(&(v, usize::MAX)) {
                        Some(&x) => x,
                        None => {
                            let x = if mi.q_on_edge {
                                let [(fa, _), _] = sides(b, mi.e3)?;
                                vertex_on_boundary(&mut nb, fa, &mi.q, vtol).ok_or_else(|| failed("corner", "a mitre's end isn't on its edge"))?
                            } else {
                                nb.add_vertex(mi.q)
                            };
                            out.contact_verts.insert((v, usize::MAX), x);
                            x
                        }
                    };
                    let key = SectionKey::Mitre(v);
                    let me = match out.section_edges.get(&key) {
                        Some(&x) => x,
                        None => {
                            let mut xs = mi.pts.clone();
                            let last = xs.len() - 1;
                            xs[0] = nb.verts[ps].p;
                            xs[last] = nb.verts[qv].p;
                            let (c, c0, c1) = fit_cut(&xs, None, [None, None]);
                            let x = add_edge(&mut nb, c, c0, c1, ps, qv);
                            out.section_edges.insert(key, x);
                            x
                        }
                    };
                    across.push(me);
                    match other {
                        MitreEnd::AtEnd { t } => {
                            rv[j][ko] = qv;
                            rt[j][ko] = st.rail_param(ko, &mi.q, *t, *t);
                        }
                        MitreEnd::Exit { exit, k, pts } => {
                            let xp = exit.ball.q[*k].p;
                            rv[j][ko] = vertex_on_boundary(&mut nb, ef.faces[ko], &xp, vtol).ok_or_else(|| failed("end", "a fillet rail's end isn't on its face's boundary"))?;
                            rt[j][ko] = st.rail_param(ko, &xp, exit.t, exit.t);
                            let mut xs: Vec<P3> = pts.iter().map(|x| x.1).collect();
                            let last = xs.len() - 1;
                            xs[0] = nb.verts[qv].p;
                            xs[last] = nb.verts[rv[j][ko]].p;
                            let (c, c0, c1) = fit_cut(&xs, Some(&b.faces[exit.into]), [None, Some(exit.ball.c)]);
                            let ee = add_edge(&mut nb, c, c0, c1, qv, rv[j][ko]);
                            if !inside_face(b, exit.into, &nb.edges[ee]) {
                                return Err(unsupported("a mitre's end outside the face beyond"));
                            }
                            out.cuts.entry(exit.into).or_default().push(ee);
                            across.push(ee);
                        }
                    }
                }
                End::Square { t, balls, n } => {
                    let v = b.edges[ef.e].v[j];
                    let vp = b.verts[v].p;
                    let mut sides = Vec::new();
                    for k in 0..2 {
                        let fi = ef.faces[k];
                        let q = balls[k].q[k].p;
                        let side = match vertex_on_boundary(&mut nb, fi, &q, vtol) {
                            // The face ends at the plane, along an edge from the vertex: that
                            // edge's piece up to the rail is the end face's.
                            Some(vq) => {
                                let e = nb.faces[fi].loops.iter().flatten().map(|c| c.edge).find(|&e| {
                                    let ev = nb.edges[e].v;
                                    !nb.edges[e].degenerate && (ev == [v, vq] || ev == [vq, v])
                                });
                                let e = e.ok_or_else(|| unsupported("a square fillet end whose rail meets a far edge of its face"))?;
                                rv[j][k] = vq;
                                e
                            }
                            // Else a cut across the face from the vertex to the rail's end, where
                            // the plane meets it.
                            None => {
                                let vq = nb.add_vertex(q);
                                rv[j][k] = vq;
                                let e = section_cut(&b.faces[fi], &vp, &q, &vp, n, vtol, v, vq, &mut nb);
                                out.cuts.entry(fi).or_default().push(e);
                                e
                            }
                        };
                        rt[j][k] = st.rail_param(k, &q, t[k], t[k]);
                        sides.push(side);
                    }
                    // The strip's cut by the plane: the ball's section where that lies in it
                    // (the faces unchanged along the edge there), else the marched curve.
                    let in_plane = (t[0] - t[1]).abs() < tol::REL_FINE * (1.0 + t[0].abs()) && balls[0].q.iter().all(|q| (q.p - balls[0].c).dot(n).abs() < vtol);
                    let across_e = match balls[0].arc().filter(|_| in_plane) {
                        Some((c, a0, a1)) => add_edge(&mut nb, c, a0, a1, rv[j][0], rv[j][1]),
                        None => {
                            let pts = match &marched[j] {
                                Some(pts) => pts.clone(),
                                None => return Err(failed("end", "a square fillet end without its cut")),
                            };
                            let mut xs: Vec<P3> = pts.iter().map(|x| x.1).collect();
                            let last = xs.len() - 1;
                            xs[0] = nb.verts[rv[j][0]].p;
                            xs[last] = nb.verts[rv[j][1]].p;
                            // (Square to the ball's radius at the rails: a round section's.)
                            let centres = if ef.flat { [None, None] } else { [Some(balls[0].c), Some(balls[1].c)] };
                            let (c, c0, c1) = fit_cut(&xs, Some(&square_plane(&vp, n)), centres);
                            add_edge(&mut nb, c, c0, c1, rv[j][0], rv[j][1])
                        }
                    };
                    across.push(across_e);
                    let x = (balls[0].q[0].p - vp).normalize();
                    let x = (x - n * x.dot(n)).normalize();
                    let surf = Surface::Plane { f: Frame { o: vp, x, y: n.cross(&x), z: *n } };
                    let vi = super::blend::vertex_index(b, v);
                    out.new_faces.push(NewFace { surf, edges: vec![across_e, sides[0], sides[1]], src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: ef.faces[0], pcurves: Vec::new() });
                }
                End::Hole { c, t, ball } => {
                    for k in 0..2 {
                        let q = ball.q[k].p;
                        rv[j][k] = nb.add_vertex(q);
                        rt[j][k] = st.rail_param(k, &q, *t, *t);
                    }
                    let (arc, a0, a1) = ball.arc().ok_or_else(|| bad_geometry("a fillet section degenerates"))?;
                    let section = add_edge(&mut nb, arc, a0, a1, rv[j][0], rv[j][1]);
                    out.hole_ends.insert((p.holes[*c].v, ef.e), HoleEnd { section, rv: rv[j], faces: ef.faces, ball: *ball });
                    across.push(section);
                }
                End::Knee { c, t, balls, pts } => {
                    let kn = &p.knees[*c];
                    for k in 0..2 {
                        let q = balls[k].q[k].p;
                        rv[j][k] = if ef.faces[k] == kn.shared {
                            // The rails' crossing, shared with the section of the other strip.
                            *out.contact_verts.entry((kn.v, kn.shared)).or_insert_with(|| nb.add_vertex(q))
                        } else {
                            vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("corner", "a mixed fillet corner's rail end isn't on the third edge"))?
                        };
                        rt[j][k] = st.rail_param(k, &q, t[k], t[k]);
                    }
                    let mut xs = pts.clone();
                    let last = xs.len() - 1;
                    xs[0] = nb.verts[rv[j][0]].p;
                    xs[last] = nb.verts[rv[j][1]].p;
                    let (cv, c0, c1) = fit_cut(&xs, None, [None, None]);
                    let x = add_edge(&mut nb, cv, c0, c1, rv[j][0], rv[j][1]);
                    out.section_edges.insert(SectionKey::Knee(kn.v, ef.e), x);
                    across.push(x);
                }
                End::Star { t, balls, p: corner, .. } => {
                    let v = b.edges[ef.e].v[j];
                    for k in 0..2 {
                        let q = balls[k].q[k].p;
                        rv[j][k] = *out.contact_verts.entry((v, ef.faces[k])).or_insert_with(|| nb.add_vertex(q));
                        rt[j][k] = st.rail_param(k, &q, t[k], t[k]);
                    }
                    // Lines to the corner's point (shared with the strips beside), or one across
                    // to the corner's triangle.
                    let lines: Vec<(SectionKey, usize, usize)> = match corner {
                        Some(c) => {
                            let pv = *out.contact_verts.entry((v, usize::MAX)).or_insert_with(|| nb.add_vertex(*c));
                            (0..2).map(|k| (SectionKey::Star(v, ef.faces[k]), rv[j][k], pv)).collect()
                        }
                        None => vec![(SectionKey::Cap(v, ef.e), rv[j][0], rv[j][1])],
                    };
                    for (key, va, vz) in lines {
                        let x = match out.section_edges.get(&key) {
                            Some(&x) => x,
                            None => {
                                let (c, len) = Curve::line(nb.verts[va].p, nb.verts[vz].p);
                                let x = add_edge(&mut nb, c, 0.0, len, va, vz);
                                out.section_edges.insert(key, x);
                                x
                            }
                        };
                        across.push(x);
                    }
                }
                End::Section { ball, t, key } => {
                    let v = key.site();
                    for k in 0..2 {
                        let fi = ef.faces[k];
                        let q = ball.q[k].p;
                        let vid = match out.contact_verts.get(&(v, fi)) {
                            Some(&x) => x,
                            None => {
                                // On the face's boundary (a tangent junction where the face
                                // changes) or inside it (a corner's contact).
                                let x = vertex_on_boundary(&mut nb, fi, &q, vtol).unwrap_or_else(|| nb.add_vertex(q));
                                out.contact_verts.insert((v, fi), x);
                                x
                            }
                        };
                        rv[j][k] = vid;
                        rt[j][k] = st.rail_param(k, &q, *t, *t);
                    }
                    let se = match out.section_edges.get(key) {
                        Some(&x) => x,
                        None => {
                            let (c, a0, a1) = ball.arc().ok_or_else(|| bad_geometry("a fillet section degenerates"))?;
                            let x = add_edge(&mut nb, c, a0, a1, rv[j][0], rv[j][1]);
                            out.section_edges.insert(*key, x);
                            x
                        }
                    };
                    across.push(se);
                }
            }
        }
        // The rails, from the strip's start end to its other end, each cut by the faces beyond
        // where it runs off its face over an edge and back: each rail on its own, or, where a
        // face cuts the strip right across (a hole through it), both together, the strip in
        // two.
        let mut offs: [Vec<(Crossing, Crossing)>; 2] = [Vec::new(), Vec::new()];
        if cliffs {
            for (k, o) in offs.iter_mut().enumerate() {
                let (p0, p1) = (nb.verts[rv[0][k]].p, nb.verts[rv[1][k]].p);
                let (t0, t1) = (ef.spine.param_of(&p0, b.edges[ef.e].t[0]), ef.spine.param_of(&p1, b.edges[ef.e].t[1]));
                *o = open_cliffs(b, ef, k, t0, t1, [p0, p1])?;
            }
        }
        // A B-spline strip through the crossings' sections too, so its rails meet the faces'
        // edges where the rails cross them.
        if matches!(st.kind, StripKind::Nurbs) && offs.iter().flatten().any(|(a, z)| !cover.contains(&a.t) || !cover.contains(&z.t)) {
            let mut more = cover.clone();
            more.extend(offs.iter().flatten().flat_map(|(a, z)| [a.t, z.t]));
            st = build_strip(b, ef, &more, &pinned, fit_tol)?;
        }
        let rails = StripRails { ef, st: &st, rv, rt, vtol };
        let (nb0, out0, given0) = (nb.clone(), out.clone(), given.clone());
        let fe = match rails.each(b, &mut nb, &offs, &mut given, &mut out) {
            Ok(fe) => fe,
            Err(KernelError::Unsupported(why)) if why == WHOLE_STRIP && offs[0].len() == 1 && offs[1].len() == 1 => {
                debug(|| format!("edge {}: cut right across", ef.e));
                (nb, out, given) = (nb0, out0, given0);
                rails.across(b, &mut nb, &offs, &mut given, &mut out)?
            }
            Err(e) => return Err(e),
        };
        let mut fe = fe;
        fe.extend(across);
        // One face per loop the edges close (two where the strip is cut in two).
        for part in loops_of(&nb, &fe) {
            let pcurves: Vec<(usize, Curve)> = given.iter().filter(|x| part.contains(&x.0)).cloned().collect();
            out.new_faces.push(NewFace { surf: st.surf.clone(), edges: part, src: FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(ef.e as u64) }), u_period: None, beside: ef.faces[0], pcurves });
        }
    }
    // Corner patches.
    for c in &p.corners {
        let arcs: Vec<usize> = c.edges.iter().map(|&e| out.section_edges[&SectionKey::Corner(c.v, e)]).collect();
        let m = c.contacts.iter().map(|x| x.1.p - c.c).fold(V3::zeros(), |a, d| a + d.normalize());
        let x = if m.norm() > tol::ROUNDOFF { m.normalize() } else { (c.contacts[0].1.p - c.c).normalize() };
        // The poles (±z) as far from the patch as they go: a patch reaching round a pole would
        // wrap its loop once round in u.
        let dirs: Vec<V3> = arcs
            .iter()
            .flat_map(|&a| {
                let ed = &nb.edges[a];
                (0..=4).map(move |i| ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 4.0))
            })
            .filter_map(|p| (p - c.c).try_normalize(tol::DEGENERATE))
            .collect();
        let worst = |z: &V3| dirs.iter().map(|d| d.dot(z).abs()).fold(0.0, f64::max);
        let z0 = any_perp(&x);
        let z1 = x.cross(&z0);
        // (Turned only where the patch comes within 25° of a pole.)
        let z = if worst(&z0) < 0.9 {
            z0
        } else {
            (0..36).map(|i| z0 * (PI * i as f64 / 36.0).cos() + z1 * (PI * i as f64 / 36.0).sin()).min_by(|p, q| worst(p).total_cmp(&worst(q))).unwrap_or(z0)
        };
        let surf = Surface::Sphere { f: Frame { o: c.c, x, y: z.cross(&x), z }, r: (c.contacts[0].1.p - c.c).norm() };
        let vi = super::blend::vertex_index(b, c.v);
        out.new_faces.push(NewFace { surf, edges: arcs, src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: c.contacts[0].0, pcurves: Vec::new() });
    }
    for pc in &p.patches {
        corner_patches(b, &mut nb, p, pc, &mut out, input)?;
    }
    for tc in &p.tris {
        tri_face(b, &mut nb, tc, &mut out, input)?;
    }
    for h in &p.holes {
        hole_face(b, &mut nb, h, vtol, &mut out, input)?;
    }
    for kn in &p.knees {
        knee_face(b, &mut nb, kn, vtol, &mut out, input)?;
    }
    for c in &p.caps {
        let edges: Vec<usize> = c.edges.iter().map(|&e| out.section_edges.get(&SectionKey::Cap(c.v, e)).copied().ok_or_else(|| failed("corner", "a chamfer corner's strip has no end"))).collect::<Result<_>>()?;
        let [o, x, y] = [nb.edges[edges[0]].v[0], nb.edges[edges[0]].v[1], nb.edges[edges[1]].v.into_iter().find(|w| !nb.edges[edges[0]].v.contains(w)).ok_or_else(|| failed("corner", "a chamfer corner's triangle doesn't close"))?].map(|w| nb.verts[w].p);
        let xd = (x - o).normalize();
        let z = xd.cross(&(y - o)).normalize();
        let surf = Surface::Plane { f: Frame { o, x: xd, y: z.cross(&xd), z } };
        let vi = super::blend::vertex_index(b, c.v);
        out.new_faces.push(NewFace { surf, edges, src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: c.beside, pcurves: Vec::new() });
    }
    // A face of a picked edge none of its strips runs on (a narrow face split off on its
    // neighbour's surface, the contact past it from the edge's middle on): wholly under the
    // strip, it goes.
    for ef in &p.efs {
        for (f, _) in sides(b, ef.e)? {
            if !p.efs.iter().any(|x| x.e == ef.e && x.faces.contains(&f)) {
                debug(|| format!("edge {}: face {f} goes under its strip", ef.e));
                out.consumed.insert(f);
            }
        }
    }
    let picked: Vec<usize> = p.efs.iter().map(|ef| ef.e).collect();
    assemble(nb, &out, &picked, size)
}

/// The face of a three-sided corner: the bridge on the common face (a new cut there), the two
/// strips' sections, and a Coons patch through them, degenerate at the point on the third edge.
fn tri_face(b: &Body, nb: &mut Body, tc: &TriCorner, out: &mut Pieces, input: BodyId) -> Result<()> {
    let sec: Vec<usize> = tc.edges.iter().map(|&e| out.section_edges.get(&SectionKey::Tri(tc.v, e)).copied().ok_or_else(|| failed("corner", "a three-sided corner's strip has no section"))).collect::<Result<_>>()?;
    let tol = tol::REL_POINT * (1.0 + b.bbox().diagonal());
    let near = |nb: &Body, p: &P3| -> Option<usize> { (0..nb.verts.len()).filter(|&v| (nb.verts[v].p - p).norm() < tol).min_by(|&x, &y| (nb.verts[x].p - p).norm().total_cmp(&(nb.verts[y].p - p).norm())) };
    let face = &b.faces[tc.shared];
    let pts: Vec<P3> = tc.bridge.iter().map(|uv| face.surf.eval(uv.x, uv.y)).collect();
    let last = pts.len() - 1;
    // The sections' ends on the common face and on the third edge.
    let other_end = |nb: &Body, e: usize, at: usize| if nb.edges[e].v[0] == at { nb.edges[e].v[1] } else { nb.edges[e].v[0] };
    let sv = near(nb, &tc.side[0]).ok_or_else(|| failed("corner", "a three-sided corner's side point has no vertex"))?;
    let sv1 = near(nb, &tc.side[1]).ok_or_else(|| failed("corner", "a three-sided corner's side point has no vertex"))?;
    let (m0, m1) = (other_end(nb, sec[0], sv), other_end(nb, sec[1], sv1));
    let mut xs = pts;
    xs[0] = nb.verts[m0].p;
    xs[last] = nb.verts[m1].p;
    let (c, c0, c1) = fit_cut(&xs, None, [None, None]);
    let rr = add_edge(nb, c, c0, c1, m0, m1);
    out.cuts.entry(tc.shared).or_default().push(rr);
    let vi = super::blend::vertex_index(b, tc.v);
    if sv1 != sv {
        // Four-sided: the third edge between the sections' ends on it is the fourth side (the
        // face beside it keeps it there, the other's piece there goes).
        let k = (0..nb.edges.len()).find(|&x| !nb.edges[x].degenerate && (nb.edges[x].v == [sv, sv1] || nb.edges[x].v == [sv1, sv])).ok_or_else(|| unsupported("a mixed fillet corner whose sections meet the third edge apart, not along it"))?;
        let bottom = along_from(nb, sec[0], sv);
        let top = along_from(nb, sec[1], sv1);
        let right = along_from(nb, rr, m0);
        let left = along_from(nb, k, sv);
        let surf = coons(&*bottom, &*top, &*left, &*right).ok_or_else(|| failed("corner", "a fillet corner's surface can't be built"))?;
        let pcs = vec![
            (sec[0], side_pcurve(nb, sec[0], sv, (1, 0.0))),
            (sec[1], side_pcurve(nb, sec[1], sv1, (1, 1.0))),
            (rr, side_pcurve(nb, rr, m0, (0, 1.0))),
            (k, side_pcurve(nb, k, sv, (0, 0.0))),
        ];
        out.new_faces.push(NewFace { surf, edges: vec![sec[0], rr, sec[1], k], src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: tc.shared, pcurves: pcs });
        return Ok(());
    }
    // The patch: u across from the third edge's point, v from the first section to the second.
    let bottom = along_from(nb, sec[0], sv);
    let top = along_from(nb, sec[1], sv);
    let right = along_from(nb, rr, m0);
    let sp = nb.verts[sv].p;
    let left = move |_: f64| sp;
    let surf = coons(&*bottom, &*top, &left, &*right).ok_or_else(|| failed("corner", "a three-sided corner's surface can't be built"))?;
    let deg = nb.add_degenerate(sp, 0.0, 1.0, sv);
    // The p-curves: the patch's sides, each linear in its edge's parameter (as sampled).
    let pcs = vec![
        (sec[0], side_pcurve(nb, sec[0], sv, (1, 0.0))),
        (sec[1], side_pcurve(nb, sec[1], sv, (1, 1.0))),
        (rr, side_pcurve(nb, rr, m0, (0, 1.0))),
        // Along u = 0 from v = 1 to v = 0.
        (deg, Curve::Line { o: P3::new(0.0, 1.0, 0.0), d: V3::new(0.0, -1.0, 0.0) }),
    ];
    out.new_faces.push(NewFace { surf, edges: vec![sec[0], rr, sec[1], deg], src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: tc.shared, pcurves: pcs });
    Ok(())
}

/// Where curve `c` crosses edge `e` (both on one face), from parameter `t` of c near there:
/// Gauss–Newton on the two curves' parameters. The parameter on c.
fn edge_crossing(c: &Curve, t: f64, e: &super::topo::Edge) -> Option<f64> {
    let p = c.point(t);
    let (mut t, mut s) = (t, e.curve.closest(&p, e.t[0], e.t[1]));
    let sc = 1.0 + p.coords.abs().max();
    for _ in 0..30 {
        let (pc, dc, _) = c.d2(t);
        let (pe, de, _) = e.curve.d2(s);
        let r = pc - pe;
        if r.norm() < 1e-12 * sc {
            break;
        }
        // Least squares: [dc, −de] (dt, ds) = −r.
        let m = nalgebra::Matrix2::new(dc.dot(&dc), -dc.dot(&de), -dc.dot(&de), de.dot(&de));
        let d = m.lu().solve(&nalgebra::Vector2::new(-dc.dot(&r), de.dot(&r)))?;
        t += d.x;
        s += d.y;
    }
    let span = (e.t[1] - e.t[0]).abs();
    let on = (c.point(t) - e.curve.point(s)).norm() < tol::REL_POINT * sc && s > e.t[0].min(e.t[1]) - tol::REL * span && s < e.t[0].max(e.t[1]) + tol::REL * span;
    on.then_some(t)
}

/// The face of a mixed corner: the section of the strip stopping there, the curve across the
/// one running on (both from the rails' crossing), the cut across the first strip's other face
/// from its section's end to the third edge (a new cut there), and a Coons patch through them,
/// degenerate at the crossing.
fn knee_face(b: &Body, nb: &mut Body, kn: &Knee, vtol: f64, out: &mut Pieces, input: BodyId) -> Result<()> {
    let [sec, run] = kn.edges.map(|e| out.section_edges.get(&SectionKey::Knee(kn.v, e)).copied().ok_or_else(|| failed("corner", "a mixed fillet corner's strip has no end")));
    let (sec, run) = (sec?, run?);
    let x = *out.contact_verts.get(&(kn.v, kn.shared)).ok_or_else(|| failed("corner", "a mixed fillet corner without its crossing"))?;
    let other_end = |e: usize| if nb.edges[e].v[0] == x { Some(nb.edges[e].v[1]) } else if nb.edges[e].v[1] == x { Some(nb.edges[e].v[0]) } else { None };
    let (Some(ps), Some(py)) = (other_end(sec), other_end(run)) else { return Err(failed("corner", "a mixed fillet corner's curves don't start at the crossing")) };
    let (pp, yp) = (nb.verts[ps].p, nb.verts[py].p);
    // The cut across the first strip's other face: inside it, or, where it leaves the face
    // across that strip's edge, in two there: the piece inside a cut, the rest standing in for
    // the face's boundary round the vertex (the face grows over the corner the other strip
    // fills, or uncovers).
    let f = &b.faces[kn.face];
    let (c, c0, c1) = face_curve(f, &pp, &yp, vtol);
    let at = |s: f64| c.point(c0 + (c1 - c0) * s);
    let n = 32;
    let out_at = (1..n).find(|&i| !point_in_face(b, kn.face, &at(i as f64 / n as f64)));
    let mut cuts = Vec::new();
    // Each piece's v (as the patch's right side runs) at its ends.
    let mut spans = vec![(0.0, 1.0)];
    match out_at {
        None => cuts.push(add_edge(nb, c.clone(), c0, c1, ps, py)),
        Some(i) => {
            let (mut lo, mut hi) = ((i - 1) as f64 / n as f64, i as f64 / n as f64);
            for _ in 0..40 {
                let m = 0.5 * (lo + hi);
                if point_in_face(b, kn.face, &at(m)) {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let tq = edge_crossing(&c, c0 + (c1 - c0) * lo, &b.edges[kn.edges[0]]).ok_or_else(|| unsupported("a mixed fillet corner whose cut leaves its face elsewhere than across the fillet's edge"))?;
            let qp = c.point(tq);
            let q = vertex_on_boundary(nb, kn.face, &qp, vtol).ok_or_else(|| failed("corner", "a mixed fillet corner's cut doesn't meet its face's edge"))?;
            cuts.push(add_edge(nb, c.clone(), c0, tq, ps, q));
            let rest = add_edge(nb, c.clone(), tq, c1, q, py);
            out.patches.push(Patch { face: kn.face, from: q, to: py, path: vec![(rest, true)], via: Some(kn.v), stretch: Vec::new() });
            cuts.push(rest);
            let sq = (tq - c0) / (c1 - c0);
            spans = vec![(0.0, sq), (sq, 1.0)];
        }
    }
    out.cuts.entry(kn.face).or_default().push(cuts[0]);
    // u from the crossing along the section, v from the section to the other strip's curve.
    let bottom = along_from(nb, sec, x);
    let top = along_from(nb, run, x);
    let right = move |v: f64| c.point(c0 + (c1 - c0) * v);
    let xp = nb.verts[x].p;
    let left = move |_: f64| xp;
    let surf = coons(&*bottom, &*top, &left, &right).ok_or_else(|| failed("corner", "a mixed fillet corner's surface can't be built"))?;
    let deg = nb.add_degenerate(xp, 0.0, 1.0, x);
    let mut pcs = vec![
        (sec, side_pcurve(nb, sec, x, (1, 0.0))),
        (run, side_pcurve(nb, run, x, (1, 1.0))),
        // Along u = 0 from v = 1 to v = 0.
        (deg, Curve::Line { o: P3::new(0.0, 1.0, 0.0), d: V3::new(0.0, -1.0, 0.0) }),
    ];
    // The cut's pieces on u = 1, v running from 0 at the section's end to 1 at the third edge.
    for (&e, (va, vz)) in cuts.iter().zip(spans) {
        let ed = &nb.edges[e];
        let d = (vz - va) / (ed.t[1] - ed.t[0]);
        pcs.push((e, Curve::Line { o: P3::new(1.0, va - d * ed.t[0], 0.0), d: V3::new(0.0, d, 0.0) }));
    }
    let vi = super::blend::vertex_index(b, kn.v);
    let mut edges = vec![sec];
    edges.extend(&cuts);
    edges.extend([run, deg]);
    out.new_faces.push(NewFace { surf, edges, src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: kn.shared, pcurves: pcs });
    Ok(())
}

/// The four-sided patches of a patched corner: one at each crossing of rails, bounded by the
/// halves of the two strips' curves across that end there and by lines from those curves'
/// middles to the hole's centre.
fn corner_patches(b: &Body, nb: &mut Body, p: &Plan, pc: &PatchCorner, out: &mut Pieces, input: BodyId) -> Result<()> {
    let n = pc.strips.len();
    if n <= 4 {
        return corner_face(b, nb, p, pc, out, input);
    }
    let halves: Vec<(usize, usize, usize)> = pc.strips.iter().map(|&i| out.patch_halves.get(&(pc.v, i)).copied().ok_or_else(|| failed("corner", "a patched corner's strip has no curve across"))).collect::<Result<_>>()?;
    let centre = P3::from(halves.iter().map(|h| nb.verts[h.2].p.coords).sum::<V3>() / n as f64);
    let cv = nb.add_vertex(centre);
    let spokes: Vec<usize> = halves
        .iter()
        .map(|h| {
            let (c, l) = Curve::line(nb.verts[h.2].p, centre);
            nb.add_edge(c, 0.0, l, h.2, cv)
        })
        .collect();
    let vi = super::blend::vertex_index(b, pc.v);
    // The point where strips a and b (in turn) cross: the vertex both their halves share.
    for s in 0..n {
        let (a, bn) = (s, (s + 1) % n);
        let (ha, hb) = (halves[a], halves[bn]);
        let shared = [ha.0, ha.1].into_iter().find_map(|x| {
            let xv = nb.edges[x].v;
            [hb.0, hb.1].into_iter().find(|y| nb.edges[*y].v.iter().any(|v| xv.contains(v) && *v != ha.2 && *v != hb.2)).map(|y| (x, y))
        });
        let (ea, eb) = shared.ok_or_else(|| failed("corner", "a patched corner's strips don't meet"))?;
        let pv = nb.edges[ea].v.iter().copied().find(|v| nb.edges[eb].v.contains(v)).ok_or_else(|| failed("corner", "a patched corner's strips don't meet"))?;
        // Curves from the crossing: along a's half to its middle, along b's half to its middle.
        let along = |e: usize, from: usize| -> Box<dyn Fn(f64) -> P3 + '_> {
            let ed = nb.edges[e].clone();
            let fwd = ed.v[0] == from;
            Box::new(move |u: f64| ed.point(if fwd { ed.t[0] + (ed.t[1] - ed.t[0]) * u } else { ed.t[1] + (ed.t[0] - ed.t[1]) * u }))
        };
        let (ma, mb) = (nb.verts[ha.2].p, nb.verts[hb.2].p);
        let bottom = along(eb, pv);
        let left = along(ea, pv);
        let top = move |u: f64| P3::from(ma.coords * (1.0 - u) + centre.coords * u);
        let right = move |v: f64| P3::from(mb.coords * (1.0 - v) + centre.coords * v);
        let surf = coons(&*bottom, &top, &*left, &right).ok_or_else(|| failed("corner", "a patched corner's surface can't be built"))?;
        let ef = &p.efs[pc.strips[a]];
        out.new_faces.push(NewFace { surf, edges: vec![ea, eb, spokes[a], spokes[bn]], src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: ef.faces[0], pcurves: Vec::new() });
    }
    Ok(())
}

/// A point of edge e a fraction u of the way from its vertex `from`.
fn along_from(nb: &Body, e: usize, from: usize) -> Box<dyn Fn(f64) -> P3> {
    let ed = nb.edges[e].clone();
    let fwd = ed.v[0] == from;
    Box::new(move |u: f64| ed.point(if fwd { ed.t[0] + (ed.t[1] - ed.t[0]) * u } else { ed.t[1] + (ed.t[0] - ed.t[1]) * u }))
}

/// The p-curve of edge e on a Coons patch side sampled by [`along_from`] from vertex `from`:
/// `fixed` is (the coordinate held, its value), the other running 0 to 1 with the edge.
fn side_pcurve(nb: &Body, e: usize, from: usize, fixed: (usize, f64)) -> Curve {
    let ed = &nb.edges[e];
    let l = ed.t[1] - ed.t[0];
    let (o, d) = if ed.v[0] == from { (-ed.t[0] / l, 1.0 / l) } else { (ed.t[1] / l, -1.0 / l) };
    if fixed.0 == 1 {
        Curve::Line { o: P3::new(o, fixed.1, 0.0), d: V3::new(d, 0.0, 0.0) }
    } else {
        Curve::Line { o: P3::new(fixed.1, o, 0.0), d: V3::new(0.0, d, 0.0) }
    }
}

/// One face filling a corner of three or four strips: a Coons patch on the strips' curves
/// across (with three, degenerate at one corner).
fn corner_face(b: &Body, nb: &mut Body, p: &Plan, pc: &PatchCorner, out: &mut Pieces, input: BodyId) -> Result<()> {
    let n = pc.strips.len();
    let curves: Vec<usize> = pc.strips.iter().map(|&i| out.patch_halves.get(&(pc.v, i)).map(|h| h.0).ok_or_else(|| failed("corner", "a patched corner's strip has no curve across"))).collect::<Result<_>>()?;
    // Corner points: P[i] where curve i−1 meets curve i; curve i runs from P[i] to P[i+1].
    let mut pts = Vec::new();
    for i in 0..n {
        let (a, c) = (&nb.edges[curves[(i + n - 1) % n]], &nb.edges[curves[i]]);
        let v = a.v.iter().copied().find(|v| c.v.contains(v)).ok_or_else(|| failed("corner", "a patched corner's strips don't meet"))?;
        pts.push(v);
    }
    let vi = super::blend::vertex_index(b, pc.v);
    let beside = p.efs[pc.strips[0]].faces[0];
    let src = FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) });
    if n == 4 {
        // (0,0) = P0, (1,0) = P1, (1,1) = P2, (0,1) = P3.
        let bottom = along_from(nb, curves[0], pts[0]);
        let right = along_from(nb, curves[1], pts[1]);
        let top = along_from(nb, curves[2], pts[3]);
        let left = along_from(nb, curves[3], pts[0]);
        let surf = coons(&*bottom, &*top, &*left, &*right).ok_or_else(|| failed("corner", "a patched corner's surface can't be built"))?;
        let pcs = vec![
            (curves[0], side_pcurve(nb, curves[0], pts[0], (1, 0.0))),
            (curves[1], side_pcurve(nb, curves[1], pts[1], (0, 1.0))),
            (curves[2], side_pcurve(nb, curves[2], pts[3], (1, 1.0))),
            (curves[3], side_pcurve(nb, curves[3], pts[0], (0, 0.0))),
        ];
        out.new_faces.push(NewFace { surf, edges: curves, src, u_period: None, beside, pcurves: pcs });
        return Ok(());
    }
    // Three: degenerate at P0 (u = 0); v from curve 0 (P0→P1) to curve 2 reversed (P0→P2).
    let bottom = along_from(nb, curves[0], pts[0]);
    let right = along_from(nb, curves[1], pts[1]);
    let top = along_from(nb, curves[2], pts[0]);
    let p0 = nb.verts[pts[0]].p;
    let left = move |_: f64| p0;
    let surf = coons(&*bottom, &*top, &left, &*right).ok_or_else(|| failed("corner", "a patched corner's surface can't be built"))?;
    let deg = nb.add_degenerate(p0, 0.0, 1.0, pts[0]);
    let pcs = vec![
        (curves[0], side_pcurve(nb, curves[0], pts[0], (1, 0.0))),
        (curves[1], side_pcurve(nb, curves[1], pts[1], (0, 1.0))),
        (curves[2], side_pcurve(nb, curves[2], pts[0], (1, 1.0))),
        (deg, Curve::Line { o: P3::new(0.0, 1.0, 0.0), d: V3::new(0.0, -1.0, 0.0) }),
    ];
    let mut edges = curves;
    edges.push(deg);
    out.new_faces.push(NewFace { surf, edges, src, u_period: None, beside, pcurves: pcs });
    Ok(())
}

/// A stretch of a face's boundary replaced by new edges: the path of `(edge, forward)` from
/// vertex `from` to `to` stands in for the stretch between them (the run of edges `stretch` when
/// given, else the one passing `via` when given, else the one of fewer edges).
#[derive(Clone)]
struct Patch {
    face: usize,
    from: usize,
    to: usize,
    path: Vec<(usize, bool)>,
    via: Option<usize>,
    stretch: Vec<usize>,
}

/// Whether the middle of edge `e` lies inside face `fi` of `b`.
fn inside_face(b: &Body, fi: usize, e: &super::topo::Edge) -> bool {
    let f = &b.faces[fi];
    let p = e.point(e.mid_t());
    let near = near_chart(b, f, &p);
    let (u, v) = f.surf.project(&p, near.map(|q| (q.x, q.y)));
    let polys = super::classify::face_polygons(b, f);
    super::classify::periodic_candidates(&f.surf, u, v, &polys).into_iter().any(|uv| super::classify::in_face(&polys, &f.surf, uv, tol::ROUNDOFF) == super::classify::InFace::Inside)
}

/// Applies a [`Patch`] to its face's loops.
fn patch(b: &mut Body, p: &Patch) -> Result<()> {
    let bad = || failed("end", "a fillet's end doesn't meet its end face's boundary");
    let fi = p.face;
    if !p.stretch.is_empty() {
        // Those edges' coedges, a run between the path's ends (either way).
        let li = b.faces[fi].loops.iter().position(|l| l.iter().any(|c| c.edge == p.stretch[0])).ok_or_else(bad)?;
        let l = &b.faces[fi].loops[li];
        let n = l.len();
        let ins = |i: usize| p.stretch.contains(&l[i % n].edge);
        let ci = (0..n).find(|&i| ins(i) && !ins(i + n - 1)).ok_or_else(bad)?;
        let m = p.stretch.len();
        if (0..m).any(|k| !ins(ci + k)) || (0..n).filter(|&i| ins(i)).count() != m {
            return Err(bad());
        }
        let (first, last) = (&l[ci], &l[(ci + m - 1) % n]);
        let forward = match (b.co_start(first), b.co_end(last)) {
            (x, y) if (x, y) == (p.from, p.to) => true,
            (x, y) if (x, y) == (p.to, p.from) => false,
            _ => return Err(bad()),
        };
        let path: Vec<(usize, bool)> = if forward { p.path.clone() } else { p.path.iter().rev().map(|&(e, f)| (e, !f)).collect() };
        let new = path_coedges(b, fi, li, ci, &path);
        let mut l = b.faces[fi].loops[li].clone();
        l.rotate_left(ci);
        l.splice(0..m, new);
        b.faces[fi].loops[li] = l;
        return Ok(());
    }
    let li = b.faces[fi].loops.iter().position(|l| l.iter().any(|c| b.co_start(c) == p.from) && l.iter().any(|c| b.co_start(c) == p.to)).ok_or_else(bad)?;
    let l = b.faces[fi].loops[li].clone();
    let n = l.len();
    let at = |v: usize| l.iter().position(|c| b.co_start(c) == v);
    let (ia, ib) = (at(p.from).ok_or_else(bad)?, at(p.to).ok_or_else(bad)?);
    // The stretch from one index to the other: its length and whether it passes `via`.
    let stretch = |from: usize, to: usize| -> (usize, bool) {
        let (mut i, mut len, mut via) = (from, 0, false);
        while i != to {
            via |= Some(b.co_start(&l[i])) == p.via && i != from;
            len += 1;
            i = (i + 1) % n;
        }
        (len, via)
    };
    let (ab, ba) = (stretch(ia, ib), stretch(ib, ia));
    let forward = match p.via {
        Some(_) if ab.1 != ba.1 => ab.1,
        Some(_) => return Err(bad()),
        None => ab.0 <= ba.0,
    };
    // The path in the loop's direction.
    let path: Vec<(usize, bool)> = if forward { p.path.clone() } else { p.path.iter().rev().map(|&(e, f)| (e, !f)).collect() };
    let (keep_from, keep_to) = if forward { (ib, ia) } else { (ia, ib) };
    let mut out: Vec<Coedge> = Vec::with_capacity(n);
    let mut i = keep_from;
    while i != keep_to {
        out.push(l[i].clone());
        i = (i + 1) % n;
    }
    for (e, fwd) in path {
        let ed = &b.edges[e];
        let near = near_chart(b, &b.faces[fi], &ed.point(ed.mid_t()));
        let pc = face_pcurve(&b.faces[fi].surf, &ed.curve, ed.t[0], ed.t[1], near);
        out.push(Coedge { edge: e, fwd, pc });
    }
    b.faces[fi].loops[li] = out;
    Ok(())
}

/// The coedges of `path` standing in for coedge `ci` of loop `li` of face `fi`: their p-curves
/// by projection, each shifted by whole periods of a closed surface to go on from the one
/// before (the first from the coedge before `ci`: a path along a seam then takes the seam's
/// side it comes to).
fn path_coedges(b: &Body, fi: usize, li: usize, ci: usize, path: &[(usize, bool)]) -> Vec<Coedge> {
    let f = &b.faces[fi];
    let l = &f.loops[li];
    let periods = super::pcurve::closed_periods(&f.surf);
    let end_of = |c: &Coedge| {
        let (_, t1) = super::topo::co_range(&b.edges[c.edge], c.fwd);
        let q = c.pc.point(t1);
        P2::new(q.x, q.y)
    };
    let mut at = end_of(&l[(ci + l.len() - 1) % l.len()]);
    let mut out = Vec::new();
    for &(e, fwd) in path {
        let ed = &b.edges[e];
        let (t0, _) = super::topo::co_range(ed, fwd);
        let pc = face_pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], Some(at));
        let q = pc.point(t0);
        let shift = |d: f64, p: Option<f64>| p.map_or(0.0, |p| (d / p).round() * p);
        let (du, dv) = (shift(at.x - q.x, periods[0]), shift(at.y - q.y, periods[1]));
        let pc = if du != 0.0 || dv != 0.0 { pc.transformed(&Affine::translation(V3::new(du, dv, 0.0))) } else { pc };
        let c = Coedge { edge: e, fwd, pc };
        at = end_of(&c);
        out.push(c);
    }
    out
}

/// Closes small gaps between a closed surface's seam and the fitted p-curve of a cut ending on
/// it (projected near the seam, a cut's end comes out a hair off its period's edge): the end of
/// the fitted (clamped spline) p-curve moves onto the seam's. Only gaps the loop's check would
/// refuse and under `lim` (in the surface's metric: a pole's points far apart in (u, v) are one
/// point in space).
fn close_gaps(b: &Body, s: &Surface, l: &mut [Coedge], lim: f64) {
    let n = l.len();
    let (mu, mv) = s.metric();
    let end_uv = |c: &Coedge, end: bool| -> P2 {
        let (t0, t1) = super::topo::co_range(&b.edges[c.edge], c.fwd);
        let q = c.pc.point(if end { t1 } else { t0 });
        P2::new(q.x, q.y)
    };
    // Coedge c's p-curve with its start (or end) moved to `to`.
    let moved = |c: &Coedge, start: bool, to: P2| -> Option<Curve> {
        let Curve::Nurbs(nc) = &c.pc else { return None };
        let (p, k) = (nc.degree, &nc.knots);
        let clamped = k[..=p].iter().all(|x| *x == k[0]) && k[k.len() - 1 - p..].iter().all(|x| *x == k[k.len() - 1]);
        // Its ends the edge's ends (not a piece of a longer curve).
        let e = &b.edges[c.edge];
        let whole = (k[0] - e.t[0]).abs() <= tol::REL_FINE * (1.0 + e.t[0].abs()) && (k[k.len() - 1] - e.t[1]).abs() <= tol::REL_FINE * (1.0 + e.t[1].abs());
        if !clamped || !whole {
            return None;
        }
        let mut nc = (**nc).clone();
        let i = if start == c.fwd { 0 } else { nc.cps.len() - 1 };
        let w = nc.cps[i].w;
        nc.cps[i] = super::nurbs::hom(&P3::new(to.x, to.y, 0.0), w);
        Some(Curve::Nurbs(std::sync::Arc::new(nc)))
    };
    let seam: Vec<bool> = l.iter().map(|c| l.iter().filter(|x| x.edge == c.edge).count() == 2).collect();
    for k in 0..n {
        let j = (k + 1) % n;
        if b.edges[l[k].edge].degenerate || b.edges[l[j].edge].degenerate || seam[k] == seam[j] {
            continue;
        }
        let (a, z) = (end_uv(&l[k], true), end_uv(&l[j], false));
        // Only gaps the loop's check refuses (beyond the vertex's tolerance): within it the
        // p-curves stay as they were fitted.
        let gap = ((a.x - z.x).abs() * mu).max((a.y - z.y).abs() * mv);
        let vtol = b.verts.get(b.co_end(&l[k])).map_or(0.0, |v| v.tol).max(tol::LOOP_GAP);
        if gap <= vtol || gap > lim {
            continue;
        }
        if let Some(pc) = moved(&l[j], true, a) {
            l[j].pc = pc;
        } else if let Some(pc) = moved(&l[k], false, z) {
            l[k].pc = pc;
        }
    }
}

/// Splits the faces by their cuts, drops the pieces between the cuts and the picked edges, and
/// adds the new faces; checks the result.
fn assemble(mut nb: Body, out: &Pieces, picked: &[usize], size: f64) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    for p in &out.patches {
        patch(&mut nb, p)?;
    }
    let mut new_edge = vec![false; nb.edges.len()];
    // (A patch's path too: an edge it takes over from a face the fillet removes stops the
    // removal there.)
    for &e in out.cuts.values().flatten().chain(out.section_edges.values()).chain(out.patches.iter().flat_map(|p| p.path.iter().map(|x| &x.0))) {
        new_edge[e] = true;
    }
    for nf in &out.new_faces {
        for &e in &nf.edges {
            new_edge[e] = true;
        }
    }
    // Each input face's shell (the pieces and new faces beside it go there).
    let mut shell_of = vec![usize::MAX; nb.faces.len()];
    for (si, sh) in nb.shells.iter().enumerate() {
        for &f in &sh.faces {
            shell_of[f] = si;
        }
    }
    let mut pieces: Vec<(usize, Face)> = Vec::new(); // (input face, piece)
    // Each cut face split on its own (in parallel: the faces are independent), taken in order.
    let splits: Vec<Option<Vec<Face>>> = {
        use rayon::prelude::*;
        let nb = &nb;
        (0..nb.faces.len()).into_par_iter().map(|fi| out.cuts.get(&fi).map(|cs| split(nb, &nb.faces[fi], cs, &|e| out.on_face.get(&(e, fi)).cloned()))).collect()
    };
    for (fi, ps) in splits.into_iter().enumerate() {
        match out.cuts.get(&fi) {
            Some(cs) => {
                let ps = ps.unwrap_or_default();
                // Every cut must be used by the pieces, once each way.
                for &c in cs {
                    let n = ps.iter().flat_map(|f| f.loops.iter().flatten()).filter(|x| x.edge == c).count();
                    if n != 2 {
                        debug(|| format!("face {fi} ({:?}) split by {cs:?} into {} pieces: cut {c} used {n} times; loops {:?}", nb.faces[fi].surf.kind(), ps.len(), nb.faces[fi].loops.iter().map(|l| l.iter().map(|c| (c.edge, nb.co_start(c))).collect::<Vec<_>>()).collect::<Vec<_>>()));
                        debug(|| {
                            let f = &nb.faces[fi];
                            let mut o = String::new();
                            for &c in cs {
                                let ed = &nb.edges[c];
                                let near = near_chart(&nb, f, &ed.point(ed.mid_t()));
                                let pc = face_pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], near);
                                o += &format!("cut {c}: pc {:?} → {:?}; ", pc.point(ed.t[0]), pc.point(ed.t[1]));
                                for i in 0..=20 {
                                    let t = ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 20.0;
                                    o += &format!("[{:.3e} {:.3e}] ", pc.point(t).x, pc.point(t).y);
                                }
                            }
                            for co in f.loops.iter().flatten() {
                                let e = &nb.edges[co.edge];
                                let (t0, _) = super::topo::co_range(e, co.fwd);
                                o += &format!("v{} at {:?}; ", nb.co_start(co), co.pc.point(t0));
                            }
                            o
                        });
                        debug(|| format!("cut edges {:?}", cs.iter().map(|&c| (nb.edges[c].v, nb.verts[nb.edges[c].v[0]].p, nb.verts[nb.edges[c].v[1]].p)).collect::<Vec<_>>()));
                        return Err(failed("trimming", "a fillet's rail doesn't split its face"));
                    }
                }
                pieces.extend(ps.into_iter().map(|f| (fi, f)));
            }
            None => pieces.push((fi, nb.faces[fi].clone())),
        }
    }
    // Drop the pieces holding the picked edges, and those reached from them across edges that
    // aren't new.
    let mut removed = vec![false; pieces.len()];
    let mut by_edge: HashMap<usize, Vec<usize>> = HashMap::new();
    for (pi, (_, f)) in pieces.iter().enumerate() {
        for c in f.loops.iter().flatten() {
            by_edge.entry(c.edge).or_default().push(pi);
        }
    }
    let mut stack: Vec<usize> = Vec::new();
    for (pi, (_, f)) in pieces.iter().enumerate() {
        if f.loops.iter().flatten().any(|c| picked.contains(&c.edge)) {
            removed[pi] = true;
            stack.push(pi);
        }
    }
    while let Some(pi) = stack.pop() {
        if !out.cuts.contains_key(&pieces[pi].0) && !out.consumed.contains(&pieces[pi].0) {
            debug(|| format!("the removal reaches face {} (cut faces {:?})", pieces[pi].0, out.cuts.keys().collect::<Vec<_>>()));
            return Err(failed("trimming", "the fillet's trimming leaks into faces it doesn't touch"));
        }
        for c in pieces[pi].1.loops.iter().flatten() {
            if new_edge.get(c.edge).copied().unwrap_or(false) {
                continue;
            }
            for &q in &by_edge[&c.edge] {
                if !removed[q] {
                    removed[q] = true;
                    stack.push(q);
                }
            }
        }
    }
    for &fi in out.cuts.keys() {
        if !out.consumed.contains(&fi) && pieces.iter().zip(&removed).all(|((f, _), &rm)| *f != fi || rm) {
            return Err(too_large("the fillet consumes a whole face"));
        }
    }
    // The new faces' loops: each new edge used against the way a kept piece uses it.
    let mut kept_dir: HashMap<usize, bool> = HashMap::new();
    for ((_, f), &rm) in pieces.iter().zip(&removed) {
        if !rm {
            for c in f.loops.iter().flatten() {
                if new_edge.get(c.edge).copied().unwrap_or(false) {
                    kept_dir.insert(c.edge, c.fwd);
                }
            }
        }
    }
    let mut faces: Vec<Face> = Vec::new();
    let mut src: Vec<Vec<FaceSrc>> = Vec::new();
    let mut shells: Vec<Shell> = nb.shells.iter().map(|s| Shell { faces: Vec::new(), closed: s.closed }).collect();
    let put = |fi: usize, at: usize, shells: &mut Vec<Shell>| {
        if let Some(s) = shells.get_mut(shell_of[fi]) {
            s.faces.push(at);
        }
    };
    for ((fi, f), rm) in pieces.into_iter().zip(removed) {
        if !rm {
            put(fi, faces.len(), &mut shells);
            faces.push(f);
            src.push(vec![FaceSrc::Input(fi)]);
        }
    }
    let first_new = faces.len();
    // Edges shared by two new faces (sections) take their direction from the first face that
    // uses them.
    let mut new_dir: HashMap<usize, bool> = HashMap::new();
    for nf in &out.new_faces {
        let es: Vec<(usize, Option<bool>)> = nf.edges.iter().map(|&e| (e, kept_dir.get(&e).map(|f| !f).or_else(|| new_dir.get(&e).map(|f| !f)))).collect();
        let lp = chain(&nb, &es)?;
        for &(e, fwd) in &lp {
            new_dir.entry(e).or_insert(fwd);
        }
        let mut coedges: Vec<Coedge> = lp
            .iter()
            .map(|&(ei, fwd)| {
                let ee = &nb.edges[ei];
                let pc = match nf.pcurves.iter().find(|x| x.0 == ei) {
                    Some((_, c)) => c.clone(),
                    None => face_pcurve(&nf.surf, &ee.curve, ee.t[0], ee.t[1], None),
                };
                Coedge { edge: ei, fwd, pc }
            })
            .collect();
        // A given p-curve of a degenerate edge runs the way that continues the loop.
        let n = coedges.len();
        for i in 0..n {
            if !nf.pcurves.iter().any(|x| x.0 == coedges[i].edge) || !nb.edges[coedges[i].edge].degenerate {
                continue;
            }
            let prev = &coedges[(i + n - 1) % n];
            let pe = &nb.edges[prev.edge];
            let at = prev.pc.point(super::topo::co_range(pe, prev.fwd).1);
            let ce = &nb.edges[coedges[i].edge];
            let start_fwd = coedges[i].pc.point(ce.t[0]);
            let start_back = coedges[i].pc.point(ce.t[1]);
            coedges[i].fwd = (start_fwd - at).norm() <= (start_back - at).norm();
        }
        align(&nf.surf, &nb, &mut coedges, nf.u_period);
        // A degenerate edge's given p-curve the other way where the loop doesn't close in
        // (u, v) as it is (a sphere strip's pole, run along the way its rail runs).
        let closes = |l: &[Coedge]| -> bool {
            let (Some(first), Some(last)) = (l.first(), l.last()) else { return true };
            let a = first.pc.point(super::topo::co_range(&nb.edges[first.edge], first.fwd).0);
            let z = last.pc.point(super::topo::co_range(&nb.edges[last.edge], last.fwd).1);
            (a - z).norm() <= 1e-6 * (1.0 + a.coords.abs().max())
        };
        if !closes(&coedges) {
            for i in 0..coedges.len() {
                if !nb.edges[coedges[i].edge].degenerate || !nf.pcurves.iter().any(|x| x.0 == coedges[i].edge) {
                    continue;
                }
                let mut l = coedges.clone();
                l[i].fwd = !l[i].fwd;
                align(&nf.surf, &nb, &mut l, nf.u_period);
                if closes(&l) {
                    coedges = l;
                    break;
                }
            }
        }
        put(nf.beside, faces.len(), &mut shells);
        faces.push(Face { surf: nf.surf.clone(), sense: true, loops: vec![coedges] });
        src.push(vec![nf.src.clone()]);
    }
    let nf = faces.len();
    nb.faces = faces;
    nb.shells = shells;
    super::build::fix_senses(&mut nb, first_new..nf);
    super::xbuild::drop_unused(&mut nb);
    // Edges the trimming split for nothing join again here (as every blend's result's do),
    // before the tolerances are fitted to what's left.
    super::unify::merge_edges(&mut nb);
    for fi in 0..nb.faces.len() {
        let mut loops = std::mem::take(&mut nb.faces[fi].loops);
        for l in loops.iter_mut() {
            close_gaps(&nb, &nb.faces[fi].surf, l, 1e-5 * (1.0 + size));
        }
        nb.faces[fi].loops = loops;
    }
    nb.fit_tolerances_searched(1e-4 * (1.0 + size));
    debug(|| {
        let mut o = String::new();
        for (fi, f) in nb.faces.iter().enumerate() {
            for c in f.loops.iter().flatten() {
                let e = &nb.edges[c.edge];
                let d: Vec<f64> = [0.1, 0.5, 0.9].iter().map(|s| { let t = e.t[0] + (e.t[1] - e.t[0]) * s; let uv = c.pc.point(t); (f.surf.eval(uv.x, uv.y) - e.point(t)).norm() }).collect();
                o += &format!("f{fi} e{} tol {:e} d {:?}\n", c.edge, e.tol, d);
            }
        }
        o
    });
    // The structure only: the solids' volumes (their faces' senses) are the caller's full
    // check's, which would only compute them again.
    let errs = nb.check_structure();
    if !errs.is_empty() {
        return Err(KernelError::InvalidResult { op: "fillet", why: format!("the local fillet doesn't close: {}", errs[0]) });
    }
    debug(|| format!("result: {} faces, {} edges", nb.faces.len(), nb.edges.iter().filter(|e| !e.degenerate).count()));
    Ok((nb, src))
}

fn any_perp(v: &V3) -> V3 {
    let a = if v.x.abs() < 0.9 { V3::x() } else { V3::y() };
    v.cross(&a).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::kernel::NativeKernel;
    use crate::Kernel;

    /// The edge of `b` whose middle is nearest `p`.
    fn edge_near(b: &Body, p: P3) -> usize {
        (0..b.edges.len())
            .filter(|&e| !b.edges[e].degenerate)
            .min_by(|&x, &y| {
                let d = |e: usize| (b.edges[e].point(b.edges[e].mid_t()) - p).norm();
                d(x).total_cmp(&d(y))
            })
            .unwrap()
    }

    fn box_body(k: &mut NativeKernel) -> (BodyId, Body) {
        let crate::native::draw::Shape::Body(id) = k.box_(10.0, 20.0, 30.0).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        (id, b)
    }

    /// OCCT blend simple_r3: an R0.5 fillet along a sheet's edge whose rails leave their faces
    /// across the sheet's free edges, which lie 2.5e-6 off their faces' p-curves (the read
    /// model's edge tolerance says 1e-6). The rails' exits, solved on the p-curves, are on the
    /// boundary all the same (it went to the blend tool: ten seconds instead of a hundredth).
    #[test]
    fn a_rail_exit_on_a_free_edge_off_its_curve() {
        use crate::native::draw::Shape;
        let Ok(root) = std::env::var("CADRS_OCCT_DATA") else { return };
        let mut k = NativeKernel::new();
        let s = k.restore(&format!("{root}/brep/4_N1(hlacg05).draw")).unwrap();
        let s = k.tscale(&s, 0.0, 0.0, 0.0, 1000.0).unwrap();
        let Shape::BodyEdge(id, e) = k.explode_edges(&s).unwrap()[0].clone() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let (out, _) = fillet_edges(&b, id, &[e.0 as usize], 0.5).unwrap();
        assert!(out.check().is_empty(), "{:?}", out.check());
    }

    /// The balls' velocities along the spine (blends card 1's tangent, one linear solve) agree
    /// with central differences of solved balls: on a hole's rim (a circle between a plane
    /// and a cylinder) with a radius varying along it, and on a box's straight edge.
    #[test]
    fn ball_velocities_match_differences() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(30.0, 30.0, 30.0).unwrap();
        let cy = k.pcylinder(6.0, 30.0).unwrap();
        let cy = k.ttranslate(&cy, 15.0, 9.0, 0.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &cy).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let budget = Budget::new(BALL_SOLVES);
        for (e, law) in [(edge_near(&b, P3::new(21.0, 9.0, 30.0)), vec![(0.0, 1.0), (1.0, 2.0)]), (edge_near(&b, P3::new(0.0, 0.0, 15.0)), vec![(0.0, 2.0), (1.0, 2.0)])] {
            let ef = edge_fillet(&b, e, &law, false, 1.0, &budget).unwrap();
            let ed = &b.edges[e];
            for f in [0.3, 0.5, 0.7] {
                let t = ed.t[0] + (ed.t[1] - ed.t[0]) * f;
                let x = ef.ball_from(&b, &ef.mid, ef.tm, t).unwrap();
                let h = 1e-5 * (ed.t[1] - ed.t[0]);
                let (xp, xm) = (ef.ball_from(&b, &x, t, t + h).unwrap(), ef.ball_from(&b, &x, t, t - h).unwrap());
                let v = ef.velocity(&b, t, &x).unwrap();
                for c in 0..2 {
                    let fd = (xp.q[c].p - xm.q[c].p) / (2.0 * h);
                    assert!((v[c] - fd).norm() < 1e-5 * (1.0 + fd.norm()), "edge {e} at {t}: contact {c}: {:?} vs {:?}", v[c], fd);
                }
            }
        }
    }

    #[test]
    fn a_box_edge_fillet_is_exact() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let (out, _) = fillet_edges(&b, id, &[e], 2.0).unwrap();
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // A 30-long edge loses (4 − π)·30.
        let want = 6000.0 - (4.0 - PI) * 30.0;
        assert!((v - want).abs() < 1e-9 * want, "{v} vs {want}");
        assert_eq!(k.get(id2).unwrap().faces.len(), 7);
    }

    /// Every edge of an L-shaped block chamfered: at the convex corners the three chamfers'
    /// planes meet in a point, at the inside corner's two ends (a filled vertical edge between
    /// two cuts) a triangle through the rails' three crossings closes the corner, as Onshape's.
    #[test]
    fn chamfers_round_an_inside_corner_meet_in_triangles() {
        use crate::{Curve2, Extent, Loop, Plane, Profile, Region};
        let mut k = NativeKernel::new();
        let pts: Vec<nalgebra::Point2<f64>> = [(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (10.0, 10.0), (10.0, 20.0), (0.0, 20.0)].iter().map(|&(x, y)| nalgebra::Point2::new(x, y)).collect();
        let outer = Loop { curves: (0..6).map(|i| Curve2::Line { a: pts[i], b: pts[(i + 1) % 6], source: None }).collect() };
        let profile = Profile::new(Plane::top(), vec![Region { outer, holes: vec![], source: None }]);
        let id = k.extrude(&profile, Extent::Blind(10.0)).unwrap().bodies[0];
        let b = (**k.get(id).unwrap()).clone();
        let laws: Option<Vec<RadiusLaw>> = (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate).map(|e| chamfer_radius(&b, e, 1.0).map(|r| (e, vec![(0.0, r), (1.0, r)]))).collect();
        let laws = laws.unwrap();
        assert_eq!(laws.len(), 18);
        let (out, _) = fillet_edges_laws(&b, id, &laws, true).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        assert_eq!(out.faces.len(), 8 + 18 + 2);
        let tris = out.faces.iter().filter(|f| f.loops.len() == 1 && f.loops[0].len() == 3 && matches!(f.surf, Surface::Plane { .. })).count();
        assert_eq!(tris, 2);
    }

    /// Chamfers mitred round a turned square block (chamfer_generated's block 17): one of
    /// the mitre's sample points lies right on the other strip, and the bisection from it ran
    /// off to the next sample (a point 1.5e-2 off the mitre's line, its fit off the strips).
    /// Mitred chamfers on flat faces are straight: the result passes the full check.
    #[test]
    fn a_mitre_through_a_sample_point_stays_straight() {
        use crate::{Curve2, Extent, Loop, Plane, Profile, Region};
        let mut k = NativeKernel::new();
        let base = [(10.338769978523262, 15.0660070428087), (-15.0660070428087, 10.338769978523263), (-10.338769978523256, -15.066007042808703), (15.066007042808701, -10.338769978523256)];
        let pts: Vec<nalgebra::Point2<f64>> = base.iter().map(|&(x, y)| nalgebra::Point2::new(x, y)).collect();
        let outer = Loop { curves: (0..4).map(|i| Curve2::Line { a: pts[i], b: pts[(i + 1) % 4], source: None }).collect() };
        let profile = Profile::new(Plane::top(), vec![Region { outer, holes: vec![], source: None }]);
        let id = k.extrude(&profile, Extent::Blind(29.3120407015679)).unwrap().bodies[0];
        let b = (**k.get(id).unwrap()).clone();
        let d = 1.2069142270573336;
        let laws: Vec<RadiusLaw> = [0, 1, 6, 7, 11].iter().map(|&e| (e, vec![(0.0, chamfer_radius(&b, e, d).unwrap()), (1.0, chamfer_radius(&b, e, d).unwrap())])).collect();
        let (out, _) = fillet_edges_laws(&b, id, &laws, true).map_err(|e| format!("{e}")).unwrap();
        let mut merged = out.clone();
        crate::native::unify::merge_edges(&mut merged);
        assert_eq!(merged.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    /// A fillet on an edge that runs on smoothly into one left as it is (a box's top front edge
    /// into the top rim of a rounded vertical edge) stops square at the vertex: its strip ends
    /// at the section there and a face in the section's plane closes the end. Exactly the
    /// edge's (1 − π/4)r² × its length is lost, and the round's own top rim keeps its corner.
    #[test]
    fn a_fillet_stops_square_where_its_edge_runs_on() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(10.0, 0.0, 15.0));
        let (rounded, _) = fillet_edges(&b, id, &[e], 3.0).unwrap();
        let id1 = k.insert(rounded.clone());
        let v1 = k.mass_properties(id1).unwrap().volume;
        let top = edge_near(&rounded, P3::new(3.5, 0.0, 30.0));
        let (out, _) = fillet_edges(&rounded, id1, &[top], 1.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        assert_eq!(out.faces.len(), rounded.faces.len() + 2);
        let id2 = k.insert(out);
        let lost = v1 - k.mass_properties(id2).unwrap().volume;
        let want = (1.0 - PI / 4.0) * 7.0;
        assert!((lost - want).abs() < 1e-6 * want, "{lost} vs {want}");
    }

    /// A chamfer of two distances by local trimming: the ball's offsets from the faces put its
    /// contacts 2 and 3 from the edge (a right triangle of legs 2 and 3 along 30), and the
    /// distances swapped put the 2 on the other face. With three such chamfers at a corner,
    /// their planes still meet in a point.
    #[test]
    fn a_chamfer_of_two_distances() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let mut widths = Vec::new();
        for d in [[2.0, 3.0], [3.0, 2.0]] {
            let (out, _) = chamfer_edges(&b, id, &[(e, d)]).map_err(|e| format!("{e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
            assert_eq!(out.faces.len(), 7);
            // The first face (in edge_sides order) keeps d[0] less of its width.
            let [(f0, _), _] = sides(&b, e).unwrap();
            let id2 = k.insert(out.clone());
            let v = k.mass_properties(id2).unwrap().volume;
            assert!((v - (6000.0 - 90.0)).abs() < 1e-9 * 6000.0, "{v}");
            let infos = k.faces(id2).unwrap();
            let before = k.faces(id).unwrap();
            let face0 = before.iter().find(|f| f.id.0 as usize == f0).unwrap();
            let (n0, o0) = (face0.plane.unwrap().normal, face0.plane.unwrap().origin);
            let same = |f: &&crate::FaceInfo| f.plane.is_some_and(|p| p.normal.dot(&n0).abs() > 0.999999 && (p.origin - o0).dot(&n0).abs() < 1e-9);
            let lost = face0.area - infos.iter().filter(same).map(|f| f.area).sum::<f64>();
            widths.push(lost / 30.0);
        }
        assert!((widths[0] - 2.0).abs() < 1e-9 && (widths[1] - 3.0).abs() < 1e-9, "{widths:?}");
        // All twelve edges, 1 and 1.5 from each edge along its faces in turn.
        let all: Vec<(usize, [f64; 2])> = (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate).map(|e| (e, [1.0, 1.5])).collect();
        let (out, _) = chamfer_edges(&b, id, &all).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        assert_eq!(out.faces.len(), 6 + 12);
    }

    /// A chamfer as wide as the face beside it (SMLib's box1x1x2): the rail on that face runs
    /// along its far edge, so the chamfer takes the whole face, that edge its own.
    #[test]
    fn a_chamfer_takes_a_whole_face() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let Shape::Body(id) = k.box_(1.0, 1.0, 2.0).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let e = edge_near(&b, P3::new(0.5, 0.0, 2.0));
        let r = chamfer_radius(&b, e, 1.0).unwrap();
        let (out, _) = fillet_edges_laws(&b, id, &[(e, vec![(0.0, r), (1.0, r)])], true).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        assert_eq!(out.faces.len(), 6);
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        assert!((v - 1.5).abs() < 1e-9, "{v}");
    }

    /// Every edge of a box chamfered at once: at each corner the three chamfers' planes meet
    /// in a point (the box cut by every chamfer's plane). Each edge's prism of d²/2 less, at
    /// each corner, the three pairs' overlaps (d³/3 each) plus the three's (d³/4): 2d²(a + b +
    /// c) − 6d³ in all.
    #[test]
    fn three_chamfers_meet_in_a_point() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let d = 2.0;
        let laws: Vec<RadiusLaw> = (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate).map(|e| (e, vec![(0.0, chamfer_radius(&b, e, d).unwrap()), (1.0, chamfer_radius(&b, e, d).unwrap())])).collect();
        assert_eq!(laws.len(), 12);
        let (out, _) = fillet_edges_laws(&b, id, &laws, true).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        let id2 = k.insert(out);
        let removed = 6000.0 - k.mass_properties(id2).unwrap().volume;
        let want = 2.0 * d * d * 60.0 - 6.0 * d * d * d;
        assert!((removed - want).abs() < 1e-9 * want, "{removed} vs {want}");
        assert_eq!(k.get(id2).unwrap().faces.len(), 6 + 12);
    }

    /// A block with a slot along the far end of its front face, under the top (OCCT blend
    /// simple Y2, Y3, Y4): the top front edge's fillet ends past the slot, its rail on the
    /// front face leaving it into the slot (over an edge away from the end vertex), so the end
    /// cut runs down the slot's faces (and, wider than the slot, across its floor) from the end
    /// face to that rail. At r 20 the rail meets the slot's corner and runs on along its
    /// floor's edge: it leaves the face at the corner, where the cut ends. The cut on a face holding the strip's straight lines (the slot's top
    /// and floor) once overran its end and folded back. At r 10 (Y1) the rail runs along the
    /// slot's top edge from its corner to the end face: the end cut crosses down onto the
    /// slot's top there and that edge ends the strip.
    #[test]
    fn a_fillet_end_cut_down_a_step() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(100.0, 100.0, 100.0).unwrap();
        let slot = k.box_at(90.0, -1.0, 80.0, 20.0, 102.0, 10.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &slot).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let v0 = k.mass_properties(id).unwrap().volume;
        let e = edge_near(&b, P3::new(50.0, 0.0, 100.0));
        for r in [10.0, 15.0, 20.0, 25.0] {
            let (out, _) = fillet_edges(&b, id, &[e], r).map_err(|e| format!("{e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
            let id2 = k.insert(out);
            // The whole edge's (1 − π/4)r² × 100, less what the slot had already taken of it
            // (its last 10 mm at most; nothing at r 10).
            let lost = v0 - k.mass_properties(id2).unwrap().volume;
            let whole = (1.0 - PI / 4.0) * r * r * 100.0;
            assert!(lost > 0.9 * whole && lost < whole * (1.0 + 1e-6), "r {r}: {lost} of {whole}");
        }
    }

    /// A block with a rectangular hole whose wall on one side is as thick as the fillet round
    /// the hole's rim there (SMLib's brick, case 4): the rail on the top runs along the block's
    /// outer edge from end to end, which becomes the strip's edge; at each end the rail stops on
    /// the top face's boundary where the hole's end wall's edge runs on, and that end wall grows
    /// over the fillet's end. The strip is the edge's whole prism: (1 − π/4)r² × its length.
    #[test]
    fn a_fillet_as_wide_as_its_face_takes_it_whole() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_at(-63.5, -50.8, 0.0, 127.0, 76.2, 50.8).unwrap();
        let hole = k.box_at(-38.1, -38.1, -1.0, 76.2, 50.8, 52.8).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &hole).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let v0 = k.mass_properties(id).unwrap().volume;
        let e = edge_near(&b, P3::new(0.0, -38.1, 50.8));
        let r = 12.7;
        let (out, _) = fillet_edges(&b, id, &[e], r).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        let id2 = k.insert(out);
        let lost = v0 - k.mass_properties(id2).unwrap().volume;
        let want = (1.0 - PI / 4.0) * r * r * 76.2;
        assert!((lost - want).abs() < 1e-6 * want, "{lost} vs {want}");
    }

    /// A block with a half-round recess in its front face up to a third of the way from its
    /// end (OCCT blend simple Q5): the top front edge's fillet runs from the end into the
    /// recess, its rail on the top leaving the top over the recess's rim away from the vertex
    /// (where the rim's circle, run on past the vertex, came round to the rail's line again at
    /// the far face's edge and was once taken), and the end cut runs over the recess's two
    /// faces.
    #[test]
    fn a_fillet_ends_in_a_recess_beside_its_vertex() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(120.0, 60.0, 60.0).unwrap();
        let cy = k.pcylinder(30.0, 62.0).unwrap();
        let cy = k.ttranslate(&cy, 60.0, 0.0, -1.0).unwrap();
        // The recess's wall in two faces, split 30° round from the vertex.
        let cy = k.trotate(&cy, 60.0, 0.0, 0.0, 0.0, 0.0, 1.0, 30.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &cy).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let v0 = k.mass_properties(id).unwrap().volume;
        let e = edge_near(&b, P3::new(105.0, 0.0, 60.0));
        let r = 25.0;
        let (out, _) = fillet_edges(&b, id, &[e], r).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        let id2 = k.insert(out);
        let lost = v0 - k.mass_properties(id2).unwrap().volume;
        let whole = (1.0 - PI / 4.0) * r * r * 30.0;
        assert!(lost > whole && lost < 1.5 * whole, "{lost} of {whole}");
    }

    /// A tower on a slab whose top edge is rounded where the tower's side wall stands (OCCT
    /// blend simple X7): the fillet of the tower's corner edge ends on the slab, its rails
    /// leaving into the slab's top and onto the round, which meet tangentially along the wall's
    /// foot; the end cut runs over the slab's top to the other rail's exit, and the top grows
    /// over the corner the fillet takes away.
    #[test]
    fn a_fillet_end_uncovers_the_face_under_it() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let Shape::Body(slab) = k.box_(100.0, 200.0, 50.0).unwrap() else { unreachable!() };
        let s = (**k.get(slab).unwrap()).clone();
        let rim = edge_near(&s, P3::new(100.0, 100.0, 50.0));
        let (rounded, _) = fillet_edges(&s, slab, &[rim], 10.0).unwrap();
        let rounded = Shape::Body(k.insert(rounded));
        let tower = k.box_at(0.0, 150.0, 50.0, 90.0, 50.0, 50.0).unwrap();
        let Shape::Body(id) = k.bfuse(&rounded, &tower).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let v0 = k.mass_properties(id).unwrap().volume;
        let e = edge_near(&b, P3::new(90.0, 150.0, 75.0));
        let r = 10.0;
        let (out, _) = fillet_edges(&b, id, &[e], r).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        let id2 = k.insert(out);
        let lost = v0 - k.mass_properties(id2).unwrap().volume;
        let want = (1.0 - PI / 4.0) * r * r * 50.0;
        assert!((lost - want).abs() < 1e-6 * want, "{lost} vs {want}");
    }

    /// A pyramid's apex (four convex edges) with some of its edges filleted (OCCT blend
    /// tolblend_simple D3, D4, D5): no ball touches the faces there, so every edge is set back
    /// from the apex, each strip stops at its section there, the faces are cut across between
    /// the setback points and one patch fills the hole. Two opposite edges' strips end on the
    /// base at an ellipse once taken the long way round (over three turns of the closest-point
    /// search): its p-curve on the strip didn't close. Each result passes the full check, its
    /// area within 1 % of OCCT's.
    #[test]
    fn some_edges_at_an_apex_meet_in_a_set_back_corner() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let s = k.wedge(10.0, 7.0, 10.0, [5.0, 5.0, 5.0, 5.0]).unwrap();
        let Shape::Body(id) = k.tscale(&s, 0.0, 0.0, 0.0, 5.0).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let apex = P3::new(25.0, 35.0, 25.0);
        // The apex's edges in turn round it (by their feet's angle about the axis).
        let mut at: Vec<usize> = (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate && b.edges[e].v.iter().any(|&v| (b.verts[v].p - apex).norm() < 1e-9)).collect();
        let foot = |e: usize| b.edges[e].v.iter().map(|&v| b.verts[v].p).find(|p| (p - apex).norm() > 1e-9).unwrap();
        at.sort_by(|&x, &y| (foot(x).z - 25.0).atan2(foot(x).x - 25.0).total_cmp(&(foot(y).z - 25.0).atan2(foot(y).x - 25.0)));
        assert_eq!(at.len(), 4);
        for (pick, area) in [(vec![at[0], at[1], at[2]], 6728.28), (vec![at[0], at[2]], 6752.97), (vec![at[0], at[1]], 6750.15)] {
            let (out, _) = fillet_edges(&b, id, &pick, 2.5).map_err(|e| format!("{e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new(), "{pick:?}");
            let id2 = k.insert(out);
            let a = k.mass_properties(id2).unwrap().surface_area;
            assert!(((a - area) / area).abs() < 1e-2, "{pick:?}: {a} vs {area}");
        }
    }

    /// A block with a round boss on its side, or a round hole through it, just under the top,
    /// and a fillet along the top edge whose radius grows towards the middle so its rail on the
    /// side dips into the circle (OCCT blend buildevol K4, K5): the strip is cut there by the
    /// boss's wall, outside it (the wall grows down to the cut, its seam, at the top, run on to
    /// the cut), or by the hole's wall, in two faces on one cylinder (the cut traced to the edge
    /// between them, as the surface doesn't change there).
    #[test]
    fn a_fillet_cut_by_a_round_wall_across_its_seam() {
        use crate::native::draw::{Ax3, Shape};
        for boss in [true, false] {
            let mut k = NativeKernel::new();
            let bx = k.box_at(-30.0, -15.0, 0.0, 30.0, 30.0, 15.0).unwrap();
            // Along x from the side x = 0, its seam at the top.
            let ax = Ax3 { origin: P3::new(if boss { 0.0 } else { -31.0 }, 0.0, 0.0), z: V3::x(), x: V3::z() };
            let cy = k.pcylinder_on(&ax, 12.0, if boss { 12.0 } else { 32.0 }).unwrap();
            let s = if boss { k.bfuse(&bx, &cy).unwrap() } else { k.bcut(&bx, &cy).unwrap() };
            let Shape::Body(id) = s else { unreachable!() };
            let b = (**k.get(id).unwrap()).clone();
            let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
            let law: RadiusLaw = (e, vec![(0.0, 2.0), (0.5, 5.0), (1.0, 2.0)]);
            let (out, _) = fillet_edges_laws(&b, id, &[law], false).map_err(|e| format!("boss {boss}: {e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new(), "boss {boss}");
        }
    }

    /// A box with a big round hole whose top rim's fillet is so large its rail on the top runs
    /// past the box's sides (SMLib case 418; OCCT blend simple S5, S8): the strip hangs over the
    /// sides, cut by them, and its rail on the hole's wall crosses the wall's seam. The strip's
    /// face is chained depth first: at the band's seam vertex the rail running on round the
    /// wall's seam once came before the band's seam, and the loop didn't close.
    #[test]
    fn a_closed_fillet_hangs_over_the_sides() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(100.0, 100.0, 50.0).unwrap();
        let cy = k.pcylinder(40.0, 52.0).unwrap();
        let cy = k.ttranslate(&cy, 50.0, 50.0, -1.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &cy).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let e = edge_near(&b, P3::new(90.0, 50.0, 50.0));
        let (out, _) = fillet_edges(&b, id, &[e], 15.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    /// A torus zone's rim filleted (OCCT blend simple J5): the closed strip's rail on the torus
    /// crosses its seam once; other balls touching both faces with a contact on the seam (round
    /// the torus's far side) were once taken as a second crossing.
    #[test]
    fn a_torus_zones_rim_fillet_crosses_its_seam_once() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let s = k.ptorus_zone(None, 20.0, 5.0, 0.0, 150.0, 360.0).unwrap();
        let Shape::Body(id) = k.tscale(&s, 0.0, 0.0, 0.0, 10.0).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        // The rim at 150°: the circle of radius 200 + 50 cos 150° at height 50 sin 150°.
        let e = (0..b.edges.len()).find(|&e| matches!(b.edges[e].curve, Curve::Circle { r, .. } if (r - (200.0 + 50.0 * (150f64).to_radians().cos())).abs() < 1e-6)).unwrap();
        let (out, _) = fillet_edges(&b, id, &[e], 10.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    /// A block with a notch, one of the notch's walls leaning: the notch's two top edges
    /// filleted meet at the top of its inside corner, whose edge is concave (OCCT blend simple
    /// M6, N8, O5, O7). Their rails on the walls meet that edge at different depths (the walls
    /// meet the top at different angles), so the corner face is four-sided, the inside edge
    /// between those points its fourth side; and the mitre the strips would make runs off the
    /// face beyond, so they don't take it.
    #[test]
    fn a_mixed_corner_whose_rails_meet_the_third_edge_apart() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(40.0, 40.0, 20.0).unwrap();
        let notch = k.box_at(20.0, 20.0, -5.0, 30.0, 30.0, 30.0).unwrap();
        let notch = k.trotate(&notch, 20.0, 20.0, 10.0, 1.0, 0.0, 0.0, 10.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &notch).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let top = |v: usize| (b.verts[v].p.z - 20.0).abs() < 1e-9;
        let v = (0..b.verts.len()).filter(|&v| top(v)).min_by(|&x, &y| (b.verts[x].p - P3::new(20.0, 20.0, 20.0)).norm().total_cmp(&(b.verts[y].p - P3::new(20.0, 20.0, 20.0)).norm())).unwrap();
        let es: Vec<usize> = vertex_edges(&b, v).into_iter().filter(|&e| b.edges[e].v.iter().all(|&w| top(w))).collect();
        assert_eq!(es.len(), 2);
        let (out, _) = fillet_edges(&b, id, &es, 3.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        // A slot's three top edges, its back wall leaning: the back one has such a corner at
        // each end (each end's contacts its own; they were once shared, by edge, between the
        // two ends).
        let bx = k.box_(40.0, 40.0, 20.0).unwrap();
        let slot = k.box_at(15.0, 20.0, -5.0, 10.0, 30.0, 30.0).unwrap();
        let slot = k.trotate(&slot, 20.0, 20.0, 10.0, 1.0, 0.0, 0.0, 10.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &slot).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let es: Vec<usize> = [P3::new(20.0, 20.0, 20.0), P3::new(15.0, 30.0, 20.0), P3::new(25.0, 30.0, 20.0)].iter().map(|p| edge_near(&b, *p)).collect();
        let (out, _) = fillet_edges(&b, id, &es, 2.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    /// A variable fillet whose radius falls to next to nothing at an end (OCCT blend
    /// tolblend_buildvol A5): its contacts there are within the vertices' tolerance of the
    /// edge's vertex, so the strip ends there at a point (both rails' ends once snapped to the
    /// vertex, the end cut between them nothing).
    #[test]
    fn a_variable_fillet_ending_next_to_nothing() {
        let mut k = NativeKernel::new();
        let crate::native::draw::Shape::Body(id) = k.box_(500.0, 500.0, 500.0).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let e = edge_near(&b, P3::new(0.0, 0.0, 250.0));
        let law: RadiusLaw = (e, vec![(0.0, 50.0), (0.5, 100.0), (1.0, 1e-4)]);
        let (out, _) = fillet_edges_laws(&b, id, &[law], false).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    /// A width fillet along the ellipse where a slanted cut meets a cylinder: the faces' angle
    /// varies round it, so its radius does too (a law of radii from the angle, r = w / (2
    /// sin(φ/2))), and its contacts stay the width apart (within the law's sampling).
    #[test]
    fn a_width_fillet_where_the_angle_varies() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let cy = k.pcylinder(20.0, 40.0).unwrap();
        let slab = k.box_at(-30.0, -30.0, 30.0, 60.0, 60.0, 30.0).unwrap();
        let slab = k.trotate(&slab, 0.0, 0.0, 30.0, 0.0, 1.0, 0.0, 20.0).unwrap();
        let Shape::Body(id) = k.bcut(&cy, &slab).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let e = (0..b.edges.len()).find(|&e| !b.edges[e].degenerate && matches!(b.edges[e].curve, Curve::Ellipse { .. })).unwrap();
        assert!(chamfer_angle(&b, e).is_none());
        let w = 3.0;
        let law = width_law(&b, e, w).unwrap();
        let (out, _) = fillet_edges_laws(&b, id, &[(e, law)], false).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        // The fillet's face's two long edges (its rails) a width apart, at points along them.
        let id2 = k.insert(out.clone());
        let fillet = out.faces.iter().position(|f| matches!(f.surf, Surface::Nurbs(_))).unwrap();
        let rails: Vec<usize> = out.faces[fillet].loops.iter().flatten().map(|c| c.edge).filter(|&x| !out.edges[x].degenerate && out.edges[x].curve.length(out.edges[x].t[0], out.edges[x].t[1]) > 50.0).collect();
        assert_eq!(rails.len(), 2, "{rails:?}");
        let (r0, r1) = (&out.edges[rails[0]], &out.edges[rails[1]]);
        for s in [0.1, 0.3, 0.6, 0.9] {
            let p = r0.point(r0.t[0] + (r0.t[1] - r0.t[0]) * s);
            let q = r1.point(r1.curve.closest(&p, r1.t[0], r1.t[1]));
            assert!(((p - q).norm() - w).abs() < 2e-2 * w, "{} vs {w}", (p - q).norm());
        }
        let _ = id2;
    }

    /// Two distances on an edge between a flat face and one curving across it (a D-shaped
    /// prism's straight vertical edge): the contacts lie the distances along the faces, round
    /// the curved one (as the blend tool's section measures them), the ball's centre where
    /// their normals meet. Swapping the distances gives another cut.
    #[test]
    fn a_two_distance_chamfer_on_a_curved_face() {
        use crate::{Curve2, Extent, Loop, Plane, Profile, Region};
        let mut k = NativeKernel::new();
        let p = nalgebra::Point2::new;
        let outer = Loop { curves: vec![Curve2::Arc { center: p(0.0, 0.0), radius: 10.0, start_angle: 0.0, sweep: PI, source: None }, Curve2::Line { a: p(-10.0, 0.0), b: p(10.0, 0.0), source: None }] };
        let id = k.extrude(&Profile::new(Plane::top(), vec![Region { outer, holes: vec![], source: None }]), Extent::Blind(10.0)).unwrap().bodies[0];
        let b = (**k.get(id).unwrap()).clone();
        let e = edge_near(&b, P3::new(10.0, 0.0, 5.0));
        let mut vols = Vec::new();
        for d in [[1.0, 2.0], [2.0, 1.0]] {
            let k01 = chamfer_offsets_along(&b, e, d).unwrap();
            let (out, _) = chamfer_edges(&b, id, &[(e, d)]).map_err(|e| format!("{e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
            // The contacts the distances along the faces from the edge: back along the flat
            // side, round the arc on the curved one.
            let [(f0, _), _] = sides(&b, e).unwrap();
            let (d_flat, d_arc) = if matches!(b.faces[f0].surf, Surface::Plane { .. }) { (d[0], d[1]) } else { (d[1], d[0]) };
            for want in [P3::new(10.0 - d_flat, 0.0, 10.0), P3::new(10.0 * (d_arc / 10.0).cos(), 10.0 * (d_arc / 10.0).sin(), 10.0)] {
                assert!(out.verts.iter().any(|v| (v.p - want).norm() < 1e-4), "{d:?}: no vertex at {want:?}");
            }
            let _ = k01;
            let id2 = k.insert(out);
            vols.push(k.mass_properties(id2).unwrap().volume);
        }
        assert!((vols[0] - vols[1]).abs() > 1e-6, "{vols:?}");
    }

    /// A hole through the top runs into the fillet: the strip is cut by the hole's wall (a
    /// cliff), and the cut keeps to both surfaces (traced in uneven steps, its interpolant
    /// once swung millimetres off them, and a retry that ignored the hole left its loop
    /// crossing the trimmed face's) — the full check finds nothing.
    #[test]
    fn a_hole_through_the_fillet_cuts_it() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(30.0, 30.0, 30.0).unwrap();
        let cy = k.pcylinder(6.0, 30.0).unwrap();
        let cy = k.ttranslate(&cy, 15.0, 9.0, 0.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &cy).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let v0 = k.mass_properties(id).unwrap().volume;
        let e = edge_near(&b, P3::new(15.0, 0.0, 30.0));
        let (out, _) = fillet_edges(&b, id, &[e], 5.0).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
        let id2 = k.insert(out);
        // Less than the whole edge's (1 − π/4)r² × 30, the hole having taken some of it.
        let lost = v0 - k.mass_properties(id2).unwrap().volume;
        let whole = (1.0 - PI / 4.0) * 25.0 * 30.0;
        assert!(lost > 0.6 * whole && lost < whole, "{lost} of {whole}");
    }

    /// The same hole through a fillet of varying radius: the rail's crossings of the hole's
    /// rim are found with the radius where they are (secant steps on the spine parameter), not
    /// the middle's, and the strip passes through them (OCCT blend buildevol_i2, j8).
    #[test]
    fn a_hole_through_a_variable_fillet_cuts_it() {
        use crate::native::draw::Shape;
        let mut k = NativeKernel::new();
        let bx = k.box_(30.0, 30.0, 30.0).unwrap();
        let cy = k.pcylinder(6.0, 30.0).unwrap();
        let cy = k.ttranslate(&cy, 15.0, 9.0, 0.0).unwrap();
        let Shape::Body(id) = k.bcut(&bx, &cy).unwrap() else { unreachable!() };
        let b = (**k.get(id).unwrap()).clone();
        let e = edge_near(&b, P3::new(15.0, 0.0, 30.0));
        let law: RadiusLaw = (e, vec![(0.0, 3.0), (1.0, 6.0)]);
        let (out, _) = fillet_edges_laws(&b, id, &[law], false).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
    }

    #[test]
    fn two_box_edges_meet_in_a_mitre() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let es: Vec<usize> = [P3::new(5.0, 0.0, 30.0), P3::new(0.0, 10.0, 30.0)].iter().map(|p| edge_near(&b, *p)).collect();
        let r = 2.0;
        let (out, _) = fillet_edges(&b, id, &es, r).map_err(|e| format!("{e}")).unwrap();
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // Each edge loses (1 − π/4)r² along its length less r; the mitred corner square loses
        // r³/3 (each strip's section swept over the triangle on its side of the mitre).
        let want = 6000.0 - (1.0 - PI / 4.0) * r * r * (10.0 + 20.0 - 2.0 * r) - r * r * r / 3.0;
        assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    }

    #[test]
    fn a_variable_fillet_follows_its_law() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let (out, _) = fillet_edges_laws(&b, id, &[(e, vec![(0.0, 1.0), (1.0, 2.0)])], false).map_err(|e| format!("{e}")).unwrap();
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // Each section loses (1 − π/4)r², r running 1 to 2 along the 30-long edge: ∫r² = 70.
        let want = 6000.0 - (1.0 - PI / 4.0) * 70.0;
        assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    }

    /// A radius changing faster than the edge runs (here 1 to 40 over a 30 long edge) has no
    /// envelope: refused before any ball is solved.
    #[test]
    fn a_radius_law_steeper_than_its_edge_is_refused() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let r = fillet_edges_laws(&b, id, &[(e, vec![(0.0, 1.0), (1.0, 40.0)])], false);
        assert!(matches!(&r, Err(KernelError::InvalidParameter(why)) if why.contains("faster")), "{:?}", r.err());
    }

    #[test]
    fn a_variable_fillet_can_end_at_a_point() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let (out, _) = fillet_edges_laws(&b, id, &[(e, vec![(0.0, 0.0), (1.0, 2.0)])], false).map_err(|e| format!("{e}")).unwrap();
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // r running 0 to 2 along the 30-long edge: ∫r² = 40.
        let want = 6000.0 - (1.0 - PI / 4.0) * 40.0;
        assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    }

    #[test]
    fn a_chamfer_cuts_straight_across() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let e = edge_near(&b, P3::new(0.0, 0.0, 15.0));
        let d = 2.0;
        let r = chamfer_radius(&b, e, d).unwrap();
        let (out, _) = fillet_edges_laws(&b, id, &[(e, vec![(0.0, r), (1.0, r)])], true).map_err(|e| format!("{e}")).unwrap();
        assert_eq!(out.faces.len(), 7);
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // A 30-long edge loses a right triangle of legs d.
        let want = 6000.0 - d * d / 2.0 * 30.0;
        assert!((v - want).abs() < 1e-9 * want, "{v} vs {want}");
    }

    #[test]
    fn fillet_faces_are_named_from_their_edges() {
        // Through the kernel (local trimming): each fillet face is generated from its edge,
        // every box face continues an input face, none is lost; the names survive a rebuild.
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let es: Vec<usize> = [P3::new(0.0, 0.0, 15.0), P3::new(5.0, 0.0, 0.0), P3::new(0.0, 10.0, 0.0)].iter().map(|p| edge_near(&b, *p)).collect();
        let ids: Vec<EdgeId> = es.iter().map(|&e| EdgeId(e as u64)).collect();
        let res = k.fillet(id, &ids, 2.0).unwrap();
        let out = k.get(res.bodies[0]).unwrap();
        assert_eq!(out.faces.len(), 10);
        let from_edges: Vec<u64> = res.history.generated.iter().filter_map(|(_, o)| match o { Origin::FromEdge { body, edge } if *body == id => Some(edge.0), _ => None }).collect();
        let mut want: Vec<u64> = es.iter().map(|&e| e as u64).collect();
        let mut got = from_edges.clone();
        want.sort_unstable();
        got.sort_unstable();
        assert_eq!(got, want, "one fillet face per edge");
        assert_eq!(res.history.generated.iter().filter(|(_, o)| matches!(o, Origin::FromVertex { .. })).count(), 1, "one corner face");
        assert_eq!(res.history.modified.len(), 6, "the box's six faces continue");
        assert!(res.history.deleted.is_empty());
        // Each generated face is a fillet face of the result.
        for (f, _) in &res.history.generated {
            assert!(!matches!(out.faces[f.0 as usize].surf, Surface::Plane { .. }));
        }
        // The same fillet again names the same faces.
        let again = k.fillet(id, &ids, 2.0).unwrap();
        assert_eq!(again.history.generated, res.history.generated);
    }

    #[test]
    fn a_box_corner_gets_a_sphere() {
        let mut k = NativeKernel::new();
        let (id, b) = box_body(&mut k);
        let es: Vec<usize> = [P3::new(0.0, 0.0, 15.0), P3::new(5.0, 0.0, 0.0), P3::new(0.0, 10.0, 0.0)].iter().map(|p| edge_near(&b, *p)).collect();
        let r = 2.0;
        let (out, _) = fillet_edges(&b, id, &es, r).unwrap();
        assert_eq!(out.faces.len(), 10);
        let id2 = k.insert(out);
        let v = k.mass_properties(id2).unwrap().volume;
        // Each edge loses (1 − π/4)r² along its length less r at the corner, and the corner
        // cube r³ less the ball's eighth.
        let lens = (30.0 - r) + (10.0 - r) + (20.0 - r);
        let want = 6000.0 - (1.0 - PI / 4.0) * r * r * lens - (1.0 - PI / 6.0) * r * r * r;
        assert!((v - want).abs() < 1e-7 * want, "{v} vs {want}");
    }
}
