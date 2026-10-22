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
//!   bounded by the three sections fills the corner (SMLib's `ConvexNxN` vertex blend).
//!
//! A closed edge (a circle round a cylinder's end) has no ends: its strip is a band whose rails
//! are closed, with a seam (a section) where its faces' seams are.
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

fn failed(why: impl Into<String>) -> KernelError {
    KernelError::OperationFailed(why.into())
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

    /// The radius at spine parameter t (the end values past the ends).
    fn radius_at(&self, t: f64) -> f64 {
        let Some(law) = &self.law else { return self.r() };
        let (first, last) = (law[0], law[law.len() - 1]);
        if t <= first.0 {
            return first.1;
        }
        if t >= last.0 {
            return last.1;
        }
        for w in law.windows(2) {
            if t <= w[1].0 {
                let f = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-300);
                return w[0].1 + (w[1].1 - w[0].1) * f;
            }
        }
        last.1
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

    /// The ball at t marched from `start` (at t0) in steps no longer than the radius.
    fn ball_from(&self, b: &Body, start: &Ball, t0: f64, t: f64) -> Option<Ball> {
        let len = self.spine_len(t0, t);
        let n = ((len / self.r()).ceil() as usize).clamp(1, 400);
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
        Err(failed("the fillet's ball doesn't reach the edge's end"))
    }

    fn spine_len(&self, t0: f64, t1: f64) -> f64 {
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
                if hb.abs() <= 1e-12 * (1.0 + tb.abs()) || (hb - ha).abs() < 1e-300 {
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
    if theta < 1e-6 {
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
    (s.abs() > 1e-9).then_some(s < 0.0)
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
        (s.abs() > 1e-9).then_some(s < 0.0)
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
    let r = law.iter().map(|x| x.1).fold(0.0, f64::max);
    let varies = law.iter().any(|x| (x.1 - r).abs() > 1e-12 * r);
    let k = if convex { [-r, -r * ratio] } else { [r, r * ratio] };
    // The larger offset, for how far the contacts may lie.
    let rr = r * ratio.max(1.0);
    // The law along the edge's parameter, in increasing order.
    let mut pts: Vec<(f64, f64)> = law.iter().map(|&(f, rr)| (ed.t[0] + (ed.t[1] - ed.t[0]) * f, rr)).collect();
    pts.sort_by(|a, c| a.0.total_cmp(&c.0));
    let law = varies.then_some(pts);
    let exact = if varies { None } else { geom::exact_kind([&b.faces[faces[0]], &b.faces[faces[1]]], &spine) };
    let tm = ed.mid_t();
    let mut ef = EdgeFillet { e, faces, k, law, flat, ratio, spine, exact, mid: dummy_ball(), tm, closed: ed.closed(), at_vertex: [true, true], budget: budget.clone(), reach: 4.0 * rr + 1e-9 };
    // The contacts lie r / tan(θ/2) from the edge in a wedge of angle θ.
    let theta = (0..=8).filter_map(|i| wedge(b, &ef, ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0).map(|w| w.2)).fold(std::f64::consts::PI, f64::min);
    ef.reach = ef.reach.max(2.0 * rr / (theta / 2.0).tan().max(1e-3) + rr);
    ef.mid = ef.ball(b, tm, None).ok_or_else(|| failed("the fillet is too large for the faces at the edge"))?;
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
        return Err(failed("the fillet's ball is on the wrong side of the edge"));
    }
    for q in &ef.mid.q {
        if (q.p - p).norm() > ef.reach {
            return Err(failed("the fillet is too large for the faces at the edge"));
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
    /// The radius law reaches 0 at the edge's vertex: the strip ends there at a point.
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
}

impl SectionKey {
    /// The site whose contact vertices the strips ending at this section share: the vertex,
    /// or a number past every vertex for a roll.
    fn site(self) -> usize {
        match self {
            SectionKey::Junction(v) | SectionKey::Corner(v, _) | SectionKey::Mitre(v) | SectionKey::Star(v, _) | SectionKey::Cap(v, _) => v,
            SectionKey::Roll(i) => usize::MAX / 2 + i,
            // Each strip's own contacts (they meet only on the edge left as it is).
            SectionKey::Tri(_, e) => usize::MAX / 4 + e,
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
    let lost = || failed("the fillet's ball doesn't reach its square end");
    let ball = ef.ball_from(b, &ef.mid, ef.tm, te).ok_or_else(lost)?;
    let sc = 1.0 + vp.coords.abs().max() + ef.r();
    let h = 1e-3 * ef.r() / ef.spine.curve.d2(te).1.norm().max(1e-12);
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
                if (fb - fa).abs() < 1e-300 {
                    return Err(lost());
                }
                let tn = tb - fb * (tb - ta) / (fb - fa);
                let bn = ef.ball_from(b, &bb, tb, tn).ok_or_else(lost)?;
                (ta, fa) = (tb, fb);
                (tb, bb, fb) = (tn, bn, f(&bn));
            }
            if fb.abs() > 1e-9 * sc {
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
    let near = [crossing(0, true)?, crossing(1, true)?];
    let exits = match near {
        [Some(a), Some(c)] => [a, c],
        [Some(x), None] | [None, Some(x)] => {
            // One rail leaves its face at the vertex into the end face; the other's face runs on:
            // that rail stops on the end face's surface, on the extension of their common edge.
            let k = if near[0].is_some() { 1 } else { 0 };
            let fk = ef.faces[k];
            let mut found = None;
            for c in b.faces[fk].loops.iter().flatten() {
                let y = &b.edges[c.edge];
                if c.edge == ef.e || y.degenerate || !y.v.contains(&v) {
                    continue;
                }
                if beyond(b, c.edge, fk) != Some(x.into) {
                    continue;
                }
                // The edge's curve run on past the vertex.
                let speed = y.curve.d2(y.t[if y.v[0] == v { 0 } else { 1 }]).1.norm().max(1e-300);
                let more = reach / speed;
                let bd = Boundary { curve: &y.curve, pc: &c.pc, t: [y.t[0] - more, y.t[1] + more] };
                let tv = y.t[if y.v[0] == v { 0 } else { 1 }];
                let outside = |t: f64| if y.v[0] == v { t < tv } else { t > tv };
                for (ty, ball) in crossings(ef, f, ef.k_at(te), k, &bd, &end_ball.c, reach) {
                    if !outside(ty) {
                        continue;
                    }
                    let d = (ball.c - end_ball.c).norm();
                    if found.as_ref().is_some_and(|(bd, _): &(f64, Exit)| bd.min(d) == *bd) {
                        continue;
                    }
                    let t = ef.spine.param_of(&ball.c, te);
                    found = Some((d, Exit { ball, t, into: x.into, ext: Some((c.edge, ty)) }));
                }
            }
            // Else the rail leaves its face over another edge (its face ends short of the end
            // face there: a step in it).
            let y = match found {
                Some(f) => f.1,
                None => crossing(k, false)?.ok_or_else(|| unsupported("a rail that doesn't leave its face"))?,
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
    /// The point on the third edge both sections start from.
    side: P3,
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
        let h = 1e-6 * (1.0 + s.r()) * (te - s.tm).signum() / s.spine.curve.d2(x.t).1.norm().max(1e-12);
        let y = s.ball_from(b, &x.ball, x.t, x.t + h).ok_or_else(|| failed("the fillet's ball is lost at a three-sided corner"))?;
        mates.push((x.ball.q[ks].uv, (y.q[ks].p - x.ball.q[ks].p).normalize()));
        ends.push(End::Section { ball: x.ball, t: x.t, key: SectionKey::Tri(v, s.e) });
    }
    if (sides_at[0] - sides_at[1]).norm() > 1e-6 * (1.0 + efs[a.0].r()) {
        return Err(unsupported("a three-sided fillet corner whose rails meet the third edge apart"));
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
    let e1 = ends.pop().ok_or_else(|| failed("a three-sided corner without ends"))?;
    let e0 = ends.pop().ok_or_else(|| failed("a three-sided corner without ends"))?;
    debug(|| format!("vertex {v}: three-sided corner of edges {} and {} from {:?}", efs[a.0].e, efs[bb.0].e, sides_at[0]));
    Ok((TriCorner { v, edges: [efs[a.0].e, efs[bb.0].e], shared: fs, side: sides_at[0], bridge }, e0, e1))
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
    let (t1s, ball1s, t2s, ball2s) = rail_cross(b, s1, k1s, b1, t1e, s2, k2s, b2, t2e).ok_or_else(|| failed("the fillets' rails don't cross at their mitre"))?;
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
    let lost = || failed("the fillets' mitre can't be traced");
    let root = |t1: f64, from: &(f64, Ball), prev: f64, st2: &mut (f64, Ball)| -> Result<Option<(f64, P3, f64, Ball)>> {
        let bl = s1.ball_from(b, &from.1, from.0, t1).ok_or_else(lost)?;
        let at = |u: f64| bl.arc_point(if k1s == 0 { u } else { 1.0 - u });
        let n = 64;
        let mut f: Vec<f64> = Vec::with_capacity(n + 1);
        for i in 0..=n {
            f.push(on2(&at(i as f64 / n as f64), st2).ok_or_else(lost)?.0);
        }
        let mut best: Option<(f64, usize)> = None; // (distance from prev, bracket)
        for i in 0..n {
            if (f[i] < 0.0) != (f[i + 1] < 0.0) || f[i] == 0.0 {
                let d = ((i as f64 + 0.5) / n as f64 - prev).abs();
                if best.is_none_or(|x| d < x.0) {
                    best = Some((d, i));
                }
            }
        }
        let Some((_, i)) = best else { return Ok(None) };
        let (mut lo, mut hi) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
        // The sign at lo as the scan found it: evaluated again (the ball marched from
        // elsewhere), a root right at the sample can come out the other side, and the bisection
        // would run off to the bracket's other end. A sample right on strip 2 is the root.
        let flo = f[i];
        // (Strip 2's ball back near the bracket.)
        on2(&at(lo), st2).ok_or_else(lost)?;
        if flo == 0.0 {
            hi = lo;
        }
        for _ in 0..60 {
            if hi == lo {
                break;
            }
            let m = 0.5 * (lo + hi);
            let fm = on2(&at(m), st2).ok_or_else(lost)?.0;
            if (fm < 0.0) == (flo < 0.0) { lo = m } else { hi = m }
        }
        let u = 0.5 * (lo + hi);
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
    let (Some(mut t_bad), Some(l)) = (end_t, last) else { return Err(failed("the fillets' mitre doesn't end")) };
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
        (ed.point(t) - p).norm() < (1e-6 * (1.0 + r)).max(ed.tol)
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
        let qp = if (st2.1.q[k2o].p - end_p).norm() < 1e-9 * (1.0 + r) { st2.1.q[k2o].p } else { end_p };
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
            if (z.t - a.t).abs() < 1e-9 * (1.0 + tm.abs()) {
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
        let ha = 1e-6 * a.r() / a.spine.curve.d2(ta).1.norm().max(1e-12);
        let hb = 1e-6 * b.r() / b.spine.curve.d2(tb).1.norm().max(1e-12);
        let xa = a.ball(body, ta + ha, Some([ba.q[0].uv, ba.q[1].uv]))?;
        let xb = b.ball(body, tb + hb, Some([bb.q[0].uv, bb.q[1].uv]))?;
        let da = (xa.q[ka].p - ba.q[ka].p) / ha;
        let db = (xb.q[kb].p - bb.q[kb].p) / hb;
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
    let tol = 1e-6 * (1.0 + efs[here[0].0].r());
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
        if n.norm() < 1e-300 {
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
        if !(-1e-9..=1.0 + 1e-9).contains(&bp.frac(0, &p)) || bp.sdist(&p).abs() > tol {
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
        let (ta, ba, tb, bb) = rail_cross(b, sa, ka, ba, ta, sb, kb, bb, tb).ok_or_else(|| unsupported("a patched fillet corner whose rails don't cross"))?;
        at[ha][ka] = Some((ta, ba));
        at[hb][kb] = Some((tb, bb));
    }
    // Each strip's curve across from its rail 0 end to its rail 1 end: the section point a
    // fraction w across at the spine parameter a fraction w of the way.
    let mut ends = Vec::new();
    for (h, &(i, _)) in here.iter().enumerate() {
        let ef = &efs[i];
        let [Some((t0, b0)), Some((t1, b1))] = at[h] else { return Err(failed("a patched fillet corner without its crossings")) };
        let n = 16;
        let mut pts = vec![b0.q[0].p];
        let mut cur = (t0, b0);
        for s in 1..n {
            let w = s as f64 / n as f64;
            let t = t0 + (t1 - t0) * w;
            let x = ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(|| failed("the fillet's ball is lost at a patched corner"))?;
            pts.push(x.arc_point(w));
            cur = (t, x);
        }
        pts.push(b1.q[1].p);
        ends.push(End::Patch { c: usize::MAX, t: [t0, t1], balls: [b0, b1], pts });
    }
    // The strips in turn round the vertex: each shares a face with the next.
    let mut order = vec![0usize];
    while order.len() < here.len() {
        let last = *order.last().ok_or_else(|| failed("a patched fillet corner without strips"))?;
        let fl = efs[here[last].0].faces;
        let next = (0..here.len()).find(|h| !order.contains(h) && efs[here[*h].0].faces.iter().any(|f| fl.contains(f)));
        match next {
            Some(h) => order.push(h),
            None => return Err(failed("a patched fillet corner's strips don't go round it")),
        }
    }
    debug(|| format!("vertex {v}: patched corner of {} strips", here.len()));
    Ok((PatchCorner { v, strips: order.iter().map(|&h| here[h].0).collect() }, ends))
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
        let x = ef.ball_from(b, &cur.1, cur.0, t).ok_or_else(|| failed("the fillet's ball is lost at its open end"))?;
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
    let on_strip = |m: &Mark| (-1e-6..=1.0 + 1e-6).contains(&m.w) && (m.w + m.ball.frac(1, &m.p) - 1.0).abs() < 1e-2;
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
                        if !on_strip(&m) || m.w < here.w - 1e-6 {
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
        (guess, tg) = (mark.ball, mark.t);
        debug(|| format!("edge {} end {j}: the cut crosses edge {d} at {:?} (w {}) onto face {after}", ef.e, mark.p, mark.w));
        crossed.push(d);
        here = mark;
        links.push(Link { mark, after });
        if after == exits[1].into {
            return Ok(links);
        }
        prev = cur;
        cur = after;
    }
    Err(unsupported("a fillet end whose faces don't go round its vertex"))
}

/// The geometry of all strips' ends.
struct Plan {
    efs: Vec<EdgeFillet>,
    ends: Vec<[Option<End>; 2]>,
    corners: Vec<Corner>,
    mitres: Vec<Mitre>,
    patches: Vec<PatchCorner>,
    tris: Vec<TriCorner>,
    caps: Vec<Cap>,
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
                if t <= lo + 1e-9 * span || t >= hi - 1e-9 * span {
                    continue;
                }
                // Smooth there: the face beyond has the same normal.
                let uv = geom::foot(&b.faces[into], &ball.q[k].p, None);
                let n = geom::contact(&b.faces[into], uv).n;
                if n.dot(&ball.q[k].n) < 1.0 - 1e-6 {
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

/// The strips of edge e: one per pair of faces the ball touches along it, in increasing spine
/// parameter (the ball rolls over smooth edges of the edge's faces onto the faces beyond).
#[allow(clippy::too_many_arguments)]
fn pieces_of(b: &Body, e: usize, law: &[(f64, f64)], flat: bool, ratio: f64, budget: &Budget, rolls: &mut Vec<Ball>) -> Result<Vec<(EdgeFillet, [Option<usize>; 2])>> {
    let base = edge_fillet(b, e, law, flat, ratio, budget)?;
    if base.closed {
        return Ok(vec![(base, [None, None])]);
    }
    let ed = &b.edges[e];
    // From the middle towards each end: each roll starts a strip with one face changed.
    let mut sides: [Vec<(EdgeFillet, usize)>; 2] = [Vec::new(), Vec::new()];
    for (j, side) in sides.iter_mut().enumerate() {
        let mut cur = EdgeFillet { mid: base.mid, faces: base.faces, ..clone_fillet(&base) };
        loop {
            let te = ed.t[j];
            let Some((k, into, ball, t)) = first_roll(b, &cur, &cur.mid, cur.tm, te)? else { break };
            if side.len() > 16 {
                return Err(failed("the fillet's ball rolls over too many faces"));
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
    for &(i, j, id) in &roll_ends {
        let ball = rolls[id];
        // Each strip's own contacts (the rolled side's face differs).
        let ef = &efs[i];
        let q: [Contact; 2] = std::array::from_fn(|k| geom::contact(&b.faces[ef.faces[k]], geom::foot(&b.faces[ef.faces[k]], &ball.q[k].p, Some(ball.q[k].uv))));
        let t = ef.spine.param_of(&ball.c, ef.tm);
        ends[i][j] = Some(End::Section { ball: Ball { c: ball.c, q, flat: ef.flat }, t, key: SectionKey::Roll(id) });
    }
    let mut corners = Vec::new();
    let mut mitres = Vec::new();
    let mut patches = Vec::new();
    let mut tris = Vec::new();
    let mut caps = Vec::new();
    let at = ends_at(b, &efs);
    let mut verts: Vec<usize> = at.keys().copied().collect();
    verts.sort_unstable();
    for v in verts {
        let here = &at[&v];
        debug(|| format!("vertex {v}: {} picked ends", here.len()));
        let all = vertex_edges(b, v);
        match here.len() {
            1 if efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1]) == 0.0 => {
                let (i, j) = here[0];
                ends[i][j] = Some(End::Point { t: b.edges[efs[i].e].t[j] });
            }
            1 => {
                let (i, j) = here[0];
                let ef = &efs[i];
                // An edge running on smoothly into one left as it is: the strip stops square.
                let out = leaving(b, ef.e, v);
                if all.iter().any(|&o| o != ef.e && leaving(b, o, v).dot(&-out) > 1.0 - 1e-9) {
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
                    End::Chain { links: end_chain(b, ef, j, &exits)?, exits }
                } else {
                    End::Free { exits }
                });
            }
            2 => {
                let (ia, ja) = here[0];
                let (ib, jb) = here[1];
                let (ea, eb) = (&efs[ia], &efs[ib]);
                if (ea.k[0] < 0.0) != (eb.k[0] < 0.0) {
                    return Err(unsupported("a convex and a concave fillet meeting"));
                }
                if leaving(b, ea.e, v).dot(&-leaving(b, eb.e, v)) < 1.0 - 1e-7 {
                    if all.len() != 3 {
                        debug(|| format!("vertex {v}: edges {:?}, picked {} and {} meet at an angle", all, ea.e, eb.e));
                        return Err(unsupported("two fillets meeting at an angle where more edges meet"));
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
                    let (m, end_a, end_b) = mitred?;
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
                let (ba, bb) = (ea.end_ball(b, ja)?, eb.end_ball(b, jb)?);
                if (ba.c - bb.c).norm() > 1e-6 * (1.0 + ea.r()) {
                    debug(|| format!("vertex {v}: edges {} and {} end in balls at {:?} and {:?} (ends {ja} {jb}; mids {:?} at {:?} and {:?} at {:?}; spine ends {:?} {:?})", ea.e, eb.e, ba.c, bb.c, ea.mid.c, ea.spine.at(ea.tm).0, eb.mid.c, eb.spine.at(eb.tm).0, ea.spine.at(b.edges[ea.e].t[ja]).0, eb.spine.at(b.edges[eb.e].t[jb]).0));
                    return Err(unsupported("tangent fillets whose sections don't meet"));
                }
                let key = SectionKey::Junction(v);
                ends[ia][ja] = Some(End::Section { ball: ba, t: b.edges[ea.e].t[ja], key });
                ends[ib][jb] = Some(End::Section { ball: bb, t: b.edges[eb.e].t[jb], key });
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
            n if n >= 3 && n == all.len() && (n > 3 || here.iter().any(|&(i, j)| (efs[i].k[0] < 0.0) != (efs[here[0].0].k[0] < 0.0) || (efs[i].radius_at(b.edges[efs[i].e].t[j]) - efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1])).abs() > 1e-9 * (1.0 + efs[i].r()))) => {
                let (pc, pends) = patch_corner(b, &efs, here, v)?;
                for ((i, j), e) in here.iter().zip(pends) {
                    let End::Patch { t, balls, pts, .. } = e else { continue };
                    ends[*i][*j] = Some(End::Patch { c: patches.len(), t, balls, pts });
                }
                patches.push(pc);
            }
            3 if all.len() == 3 && !efs[here[0].0].flat && here.iter().all(|&(i, j)| (efs[i].radius_at(b.edges[efs[i].e].t[j]) - efs[here[0].0].radius_at(b.edges[efs[here[0].0].e].t[here[0].1])).abs() < 1e-9 * (1.0 + efs[i].r())) => {
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
                let (c, q) = geom::ball3(fs, [kk; 3], [faces[0].1, faces[1].1, faces[2].1]).ok_or_else(|| failed("no ball touches the three faces at the fillets' corner"))?;
                let contacts: Vec<(usize, Contact)> = (0..3).map(|i| (faces[i].0, q[i])).collect();
                let on = |fi: usize| contacts.iter().find(|x| x.0 == fi).map(|x| x.1);
                for &(i, j) in here {
                    let ef = &efs[i];
                    let (Some(qa), Some(qb)) = (on(ef.faces[0]), on(ef.faces[1])) else { return Err(failed("a corner fillet without its faces")) };
                    let t = ef.spine.param_of(&c, b.edges[ef.e].t[j]);
                    ends[i][j] = Some(End::Section { ball: Ball { c, q: [qa, qb], flat: false }, t, key: SectionKey::Corner(v, ef.e) });
                }
                corners.push(Corner { v, c, contacts, edges: here.iter().map(|&(i, _)| efs[i].e).collect() });
            }
            _ => {
                debug(|| format!("vertex {v}: {} picked ends of {} edges ({:?}), convex {:?}", here.len(), all.len(), here.iter().map(|&(i, _)| efs[i].e).collect::<Vec<_>>(), here.iter().map(|&(i, _)| efs[i].k[0] < 0.0).collect::<Vec<_>>()));
                return Err(unsupported("a fillet corner of this kind"));
            }
        }
    }
    Ok(Plan { efs, ends, corners, mitres, patches, tris, caps })
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
/// where that point lies on the face), with their spine parameters.
fn march_on(b: &Body, ef: &EdgeFillet, a: &Mark, z: &Mark, g: &Face) -> Result<Vec<(f64, P3)>> {
    let n = 24;
    let mut out: Vec<(f64, P3)> = vec![(a.t, a.p)];
    let mut ball = a.ball;
    let mut t = a.t;
    let mut hint: Option<P2> = None;
    let mut sd = |p: &P3| -> f64 {
        let uv = geom::foot(g, p, hint);
        hint = Some(uv);
        let c = geom::contact(g, uv);
        (p - c.p).dot(&c.n)
    };
    let r = ef.r();
    let lost = || failed("the fillet's end can't be traced");
    let dt = (z.t - a.t) / n as f64;
    for i in 1..n {
        let w = a.w + (z.w - a.w) * i as f64 / n as f64;
        // Secant on the face's signed distance, from the previous point moved on as the ends'
        // parameters are.
        let mut ta = t + dt;
        let mut ba = ef.ball_from(b, &ball, t, ta).ok_or_else(lost)?;
        let mut ga = sd(&ba.arc_point(w));
        // The balls are solved to about 1e-11 of the model's size: the distance is no finer.
        let sc = 1.0 + ball.c.coords.abs().max() + r;
        // On the face already (a face holding the strip's lines, as a plane holds a cylinder's
        // rulings): there, not a step on (the steps would add up past the end).
        if ga.abs() < 1e-10 * sc {
            t = ta;
            ball = ba;
            out.push((t, ba.arc_point(w)));
            continue;
        }
        let h = 1e-3 * r / ef.spine.curve.d2(ta).1.norm().max(1e-12);
        let mut tb = ta + h;
        let mut bb = ef.ball(b, tb, Some([ba.q[0].uv, ba.q[1].uv])).ok_or_else(lost)?;
        let mut gb = sd(&bb.arc_point(w));
        let mut ok = false;
        for _ in 0..60 {
            let stalled = (tb - ta).abs() <= 1e-12 * (1.0 + tb.abs());
            if gb.abs() < 1e-10 * sc || (stalled && gb.abs() < 1e-8 * sc) {
                ok = true;
                break;
            }
            if (gb - ga).abs() < 1e-300 || stalled {
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
        let _ = (ta, ba);
        if !ok {
            debug(|| format!("edge {}: end march lost at w {w}: t {ta}..{tb}, g {ga}..{gb}", ef.e));
            return Err(failed("the fillet's end doesn't meet its end face"));
        }
        t = tb;
        ball = bb;
        out.push((t, bb.arc_point(w)));
    }
    out.push((z.t, z.p));
    Ok(out)
}

/// The strip of `ef`: exact where it can be, else a B-spline through sections at stations
/// covering the spine parameters `cover`, refined to `tol`.
fn build_strip(b: &Body, ef: &EdgeFillet, cover: &[f64], tol: f64) -> Result<Strip> {
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
        return Err(failed("the fillet strip is empty"));
    }
    let span = hi - lo;
    // Stations: evenly spaced (one per four radii of length, at most 64; more where the sections need them), plus every parameter the ends need
    // (their sections must be exact); an even one too near one of those gives way.
    let n0 = ((ef.spine_len(lo, hi) / (4.0 * ef.r())).ceil() as usize).clamp(8, 64);
    let mut ts: Vec<f64> = (0..=n0).map(|i| lo + span * i as f64 / n0 as f64).collect();
    let is_cover = |t: f64| cover.iter().any(|c| (c - t).abs() <= 1e-12 * (1.0 + span));
    ts.extend(cover.iter().copied());
    let mut stalled = 0;
    let mut kinks: Vec<f64> = Vec::new();
    loop {
        ts.sort_by(|a, b| a.total_cmp(b));
        let mut keep: Vec<f64> = Vec::new();
        for &t in &ts {
            if let Some(last) = keep.last_mut()
                && t - *last < 1e-6 * span
            {
                if is_cover(t) || kinks.contains(&t) {
                    *last = t;
                }
                continue;
            }
            keep.push(t);
        }
        ts = keep;
        let balls = balls_at(b, ef, &ts)?;
        debug(|| format!("edge {}: {} stations", ef.e, ts.len()));
        let kink_at: Vec<usize> = kinks.iter().filter_map(|k| ts.iter().position(|t| t == k)).collect();
        let strip = geom::nurbs_strip(&balls, &ts, ef.closed, &kink_at).ok_or_else(|| failed("the fillet surface can't be built"))?;
        // Check between stations; add stations where it strays.
        let mut truth: Vec<(f64, Ball)> = Vec::new();
        for (w, bl) in ts.windows(2).zip(&balls) {
            let t = 0.5 * (w[0] + w[1]);
            let x = ef.ball_from(b, bl, w[0], t).ok_or_else(|| failed("the fillet's ball is lost along the edge"))?;
            truth.push((t, x));
        }
        let errs = geom::strip_error(&strip, &truth);
        let bad: Vec<f64> = truth.iter().zip(&errs).filter(|(_, e)| **e > tol).map(|((t, _), _)| *t).collect();
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
                return Err(failed("the fillet's sections kink along the edge"));
            }
            debug(|| format!("edge {}: kinks at {fresh:?}", ef.e));
            kinks.extend(fresh.iter().copied());
            ts.extend(fresh);
            stalled = 0;
            continue;
        }
        stalled = ts.len();
        if ts.len() + bad.len() > 4000 {
            return Err(failed("the fillet surface needs too many sections"));
        }
        ts.extend(bad);
    }
}

/// The balls at increasing spine parameters, marched from the middle ball.
fn balls_at(b: &Body, ef: &EdgeFillet, ts: &[f64]) -> Result<Vec<Ball>> {
    let tm = ef.tm;
    let i0 = ts.iter().position(|&t| t >= tm).unwrap_or(ts.len() - 1);
    let mut out = vec![dummy_ball(); ts.len()];
    let lost = || failed("the fillet's ball is lost along the edge");
    out[i0] = ef.ball_from(b, &ef.mid, tm, ts[i0]).ok_or_else(lost)?;
    for i in i0 + 1..ts.len() {
        out[i] = ef.ball_from(b, &out[i - 1], ts[i - 1], ts[i]).ok_or_else(lost)?;
    }
    for i in (0..i0).rev() {
        out[i] = ef.ball_from(b, &out[i + 1], ts[i + 1], ts[i]).ok_or_else(lost)?;
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
/// isn't on the boundary.
fn vertex_on_boundary(b: &mut Body, fi: usize, p: &P3, tol: f64) -> Option<usize> {
    let mut best = (f64::MAX, usize::MAX, 0.0);
    for c in b.faces[fi].loops.iter().flatten() {
        let ee = &b.edges[c.edge];
        if ee.degenerate {
            continue;
        }
        let t = ee.curve.closest(p, ee.t[0], ee.t[1]);
        let d = (ee.point(t) - p).norm();
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
    let gap = |pc: &Curve| -> f64 {
        (0..=N)
            .map(|i| {
                let t = t0 + (t1 - t0) * i as f64 / N as f64;
                let uv = pc.point(t);
                let d = (s.eval(uv.x, uv.y) - c.point(t)).norm();
                if d.is_nan() { f64::INFINITY } else { d }
            })
            .fold(0.0, f64::max)
    };
    let size = c.point(t0).coords.abs().max().max(c.point(t1).coords.abs().max());
    let g0 = gap(&pc);
    if g0 <= 1e-6 * (1.0 + size) {
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

/// The arcs of a face's loops (for [`split_face`]).
fn face_arcs(b: &Body, f: &Face) -> Vec<Vec<Arc>> {
    f.loops
        .iter()
        .map(|l| {
            l.iter()
                .map(|c| {
                    let e = &b.edges[c.edge];
                    let v = if c.fwd { e.v } else { [e.v[1], e.v[0]] };
                    Arc { edge: c.edge, fwd: c.fwd, pc: c.pc.clone(), t: e.t, v, cut: false, deg: e.degenerate }
                })
                .collect()
        })
        .collect()
}

/// Splits face `f` by the new edges `cuts` (p-curves by projection, near the face's own
/// chart) into faces.
fn split(b: &Body, f: &Face, cuts: &[usize]) -> Vec<Face> {
    let cut_arcs: Vec<Arc> = cuts
        .iter()
        .map(|&e| {
            let ed = &b.edges[e];
            let near = near_chart(b, f, &ed.point(ed.mid_t()));
            let pc = face_pcurve(&f.surf, &ed.curve, ed.t[0], ed.t[1], near);
            Arc { edge: e, fwd: true, pc, t: ed.t, v: ed.v, cut: true, deg: false }
        })
        .collect();
    let res = split_face(&f.surf, !f.sense, &face_arcs(b, f), &cut_arcs);
    res.faces
        .into_iter()
        .map(|loops| Face {
            surf: f.surf.clone(),
            sense: f.sense,
            loops: loops.into_iter().map(|l| l.into_iter().map(|a| Coedge { edge: a.edge, fwd: a.fwd, pc: a.pc }).collect()).collect(),
        })
        .collect()
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
    let tol = 1e-6 * (1.0 + bb.diagonal());
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
            // Of the two ways round, the one whose middle is nearer the corner.
            let tb2 = if tb > ta { tb - TAU } else { tb + TAU };
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
                if g.abs() < 1e-13 * sc || grad.norm_squared() < 1e-300 {
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
            let at = lp.iter().position(|&(x, fwd)| (if fwd { b.edges[x].v[1] } else { b.edges[x].v[0] }) == v).ok_or_else(|| failed("a fillet face's pole isn't on its loop"))?;
            lp.insert(at + 1, (e, dir.unwrap_or(true)));
        }
        return Ok(lp);
    }
    let n = edges.len();
    let start = edges.iter().position(|x| x.1.is_some()).ok_or_else(|| failed("a fillet face with no known edge direction"))?;
    let mut used = vec![false; n];
    used[start] = true;
    let ends = |e: usize, fwd: bool| -> (usize, usize) {
        let v = b.edges[e].v;
        if fwd { (v[0], v[1]) } else { (v[1], v[0]) }
    };
    let (e0, f0) = (edges[start].0, edges[start].1.unwrap_or(true));
    let mut out = vec![(e0, f0)];
    let (first, mut at) = ends(e0, f0);
    for _ in 1..n {
        let mut next = None;
        for (i, &(e, dir)) in edges.iter().enumerate() {
            if used[i] {
                continue;
            }
            let opts: &[bool] = match dir {
                Some(true) => &[true],
                Some(false) => &[false],
                None => &[true, false],
            };
            if let Some(&fwd) = opts.iter().find(|&&fwd| ends(e, fwd).0 == at) {
                // A known direction wins over a guessed one.
                if next.is_none() || dir.is_some() {
                    next = Some((i, e, fwd));
                }
                if dir.is_some() {
                    break;
                }
            }
        }
        let (i, e, fwd) = next.ok_or_else(|| failed("a fillet face's edges don't close up"))?;
        used[i] = true;
        out.push((e, fwd));
        at = ends(e, fwd).1;
    }
    if at != first {
        return Err(failed("a fillet face's edges don't close up"));
    }
    Ok(out)
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
    let lost = || failed("the fillet's curve on a face can't be traced");
    let sd_stop = |p: &P3| -> f64 {
        let Some(h) = stop else { return 0.0 };
        let c = geom::contact(h, geom::foot(h, p, None));
        (p - c.p).dot(&c.n)
    };
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
    let st = ef.spine.curve.d2(a.t).1.norm().max(1e-12);
    let sw = (a.ball.q[0].p - a.ball.c).angle(&(a.ball.q[1].p - a.ball.c)) * r;
    let total = (z.p - a.p).norm();
    // Steps short enough for a cubic through the points to keep to the curve.
    let hmax = (total / 48.0).min(r / 6.0).max(1e-9);
    let (mut t, mut w, mut bl) = (a.t, a.w, a.ball);
    let mut pts = vec![a.p];
    let mut dir: Option<(f64, f64)> = None;
    let mut h = hmax;
    for _ in 0..4000 {
        // The gradient (in lengths) by central differences.
        let d = 1e-6 * hmax;
        let (fp, _, _) = eval(t + d / st, w, &bl, t).ok_or_else(lost)?;
        let (fm, _, _) = eval(t - d / st, w, &bl, t).ok_or_else(lost)?;
        let (fwp, _, _) = eval(t, w + d / sw, &bl, t).ok_or_else(lost)?;
        let (fwm, _, _) = eval(t, w - d / sw, &bl, t).ok_or_else(lost)?;
        let g = ((fp - fm) / (2.0 * d), (fwp - fwm) / (2.0 * d));
        let gn = (g.0 * g.0 + g.1 * g.1).sqrt();
        if gn < 1e-12 {
            return Err(lost());
        }
        let mut tan = (-g.1 / gn, g.0 / gn);
        let to_end = ((z.t - t) * st, (z.w - w) * sw);
        let flip = match dir {
            Some(dp) => tan.0 * dp.0 + tan.1 * dp.1 < 0.0,
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
        while h > 1e-9 * hmax {
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
        if !(-1e-9..=1.0 + 1e-9).contains(&wn) {
            // Over the strip's far side: the end when that's where the curve is bound (a cut
            // across the whole strip, to the other rail), else a cut this one can't be.
            let bound = if wn < 0.0 { 0.0 } else { 1.0 };
            if stop.is_none() && (z.w - bound).abs() < 1e-12 && dist_end <= 4.0 * hmax && pts.len() > 1 {
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

/// The strip's cut by the faces beyond its face where rail k runs off it, from crossing `a`
/// (vertex) to crossing `z`: one face, or two meeting at an edge (round a corner of the face).
/// Inside those faces the cut splits them; outside (a fillet adding material) they grow, the
/// two then joined along their edge's extension.
#[allow(clippy::too_many_arguments)]
fn cliff(b: &Body, nb: &mut Body, ef: &EdgeFillet, (k, a, va): (usize, &Crossing, usize), (kz, z, vz): (usize, &Crossing, usize), fe: &mut Vec<usize>, out: &mut Pieces) -> Result<()> {
    let tol = 1e-6 * (1.0 + ef.r());
    let (ma, mz) = (Mark::rail(a.t, a.ball, k), Mark::rail(z.t, z.ball, kz));
    let fit = |nb: &mut Body, pts: Vec<P3>, v0: usize, v1: usize, g: usize, c0: Option<P3>, c1: Option<P3>| -> usize {
        let mut xs = pts;
        let last = xs.len() - 1;
        xs[0] = nb.verts[v0].p;
        xs[last] = nb.verts[v1].p;
        let (c, t0, t1) = fit_cut(&xs, Some(&b.faces[g]), [c0, c1]);
        add_edge(nb, c, t0, t1, v0, v1)
    };
    if a.into == z.into {
        let pts = trace(b, ef, &b.faces[a.into], &ma, &mz)?;
        let ce = fit(nb, pts, va, vz, a.into, Some(a.ball.c), Some(z.ball.c));
        if inside_face(b, a.into, &nb.edges[ce]) {
            out.cuts.entry(a.into).or_default().push(ce);
        } else {
            out.patches.push(Patch { face: a.into, from: va, to: vz, path: vec![(ce, true)], via: None });
        }
        fe.push(ce);
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
        let from = marks.last().map(|x| x.0).ok_or_else(|| failed("a fillet's cut has no start"))?;
        let (pts, at) = trace_until(b, ef, &b.faces[g], &from, &mz, Some(&b.faces[h]))?;
        let at = at.ok_or_else(|| failed("a fillet's cut doesn't reach the next face round the corner"))?;
        let x = common_edge(b, g, h).ok_or_else(|| unsupported("a closed fillet whose rail runs off over faces that don't meet"))?;
        let xe = &b.edges[x];
        let tx = xe.curve.closest(&at.p, xe.t[0] - 1e6, xe.t[1] + 1e6);
        if tx >= xe.t[0] - 1e-9 && tx <= xe.t[1] + 1e-9 {
            let xv = vertex_on_boundary(nb, g, &at.p, tol.max(1e-6)).ok_or_else(|| failed("a fillet's cut round a corner isn't on the corner's edge"))?;
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
    let last = *chain.last().ok_or_else(|| failed("a fillet's cut over no faces"))?;
    let from = marks.last().map(|x| x.0).ok_or_else(|| failed("a fillet's cut has no start"))?;
    pieces.push(trace(b, ef, &b.faces[last], &from, &mz)?);
    marks.push((mz, vz));
    // The cut on each face, then how each face takes it.
    let mut cuts: Vec<usize> = Vec::new();
    for (i, pts) in pieces.into_iter().enumerate() {
        let (v0, v1) = (marks[i].1, marks[i + 1].1);
        let c0 = (i == 0).then_some(a.ball.c);
        let c1 = (i + 1 == chain.len()).then_some(z.ball.c);
        cuts.push(fit(nb, pts, v0, v1, chain[i], c0, c1));
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
            out.patches.push(Patch { face: g, from, to, path, via: None });
        }
    } else {
        return Err(unsupported("a fillet's cut round corners partly inside its faces"));
    }
    fe.extend(cuts);
    Ok(())
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
        let v = vertex_on_boundary(nb, self.ef.faces[k], &p, self.vtol).ok_or_else(|| failed("a fillet rail's crossing isn't on its face's boundary"))?;
        Ok((v, self.st.rail_param(k, &p, c.t, near)))
    }

    /// Rail k from vertex v0 (parameter t0) to v1 (t1): a cut in its face, an edge of the strip.
    #[allow(clippy::too_many_arguments)]
    fn rail(&self, nb: &mut Body, k: usize, (v0, t0): (usize, f64), (v1, t1): (usize, f64), fe: &mut Vec<usize>, given: &mut Vec<(usize, Curve)>, out: &mut Pieces) {
        let e = add_edge(nb, self.st.rails[k].clone(), t0, t1, v0, v1);
        out.cuts.entry(self.ef.faces[k]).or_default().push(e);
        fe.push(e);
        given.extend(rail_pcurve(self.st, k, t0, t1).map(|c| (e, c)));
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
                if n.dot(&ball.q[k].n) > 1.0 - 1e-6 {
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
        let mid = ef.ball_from(b, &a.ball, a.t, 0.5 * (a.t + z.t)).ok_or_else(|| failed("the fillet's ball is lost along the edge"))?;
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
    let ia = l.iter().position(|c| c.edge == a.edge).ok_or_else(|| failed("a crossing's edge isn't on its face"))?;
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
    let mid = ef.ball_from(b, &a.ball, a.t, 0.5 * (a.t + z.t)).ok_or_else(|| failed("the fillet's ball is lost along the edge"))?.q[k].p;
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
            if d.norm() < 1e-9 {
                return None;
            }
            let chord = xs[towards] - p;
            let d = d.normalize() * d.normalize().dot(&chord).signum();
            // As fast as the parameters run there.
            Some(d * (chord.norm() / (ps[towards] - ps[i]).abs().max(1e-300)))
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
                len[i] = len[i - 1] + (xs[i] - xs[i - 1]).norm().max(1e-300);
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
    super::classify::periodic_candidates(&f.surf, u, v, &polys).into_iter().any(|uv| super::classify::in_face(&polys, &f.surf, uv, 1e-9) != super::classify::InFace::Outside)
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
    let lost = || failed("the fillet's ball is lost along the edge");
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
    let tol = 1e-6 * (1.0 + r);
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
                if is_seam(b, fi, c.edge) {
                    match &seams[k] {
                        Some(s) if (s.c - ball.c).norm() > tol => return Err(unsupported("a closed fillet whose rail crosses its face's seam twice")),
                        _ => seams[k] = Some(ball),
                    }
                } else {
                    let into = beyond(b, c.edge, fi).ok_or_else(|| unsupported("a closed fillet whose rail runs off a sheet's free edge"))?;
                    // Once per point (a rail through a corner of its face crosses two edges).
                    let t = ef.spine.param_of(&ball.c, ed.t[0]);
                    if !cross[k].iter().any(|x| (x.ball.c - ball.c).norm() < tol || (wrap(x.t - t, -PI, period)).abs() < 1e-9 * period) {
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
    }
    let cliffs = cross.iter().any(|c| !c.is_empty());
    debug(|| format!("edge {}: seam crossings {:?}, {:?}; cliffs {} {}", ef.e, seams[0].map(|x| (ef.spine.param_of(&x.c, ed.t[0]), x.q[0].p)), seams[1].map(|x| (ef.spine.param_of(&x.c, ed.t[0]), x.q[1].p)), cross[0].len(), cross[1].len()));
    let rail_point = |k: usize, t: f64| -> Result<P3> { Ok(ef.ball_from(b, &ef.mid, ef.tm, t).ok_or_else(|| failed("the fillet's ball is lost along the edge"))?.q[k].p) };
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
            if t - ts > 1e-9 * period && ts + period - t > 1e-9 * period {
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
                return Err(failed("a closed fillet's rail crossings don't pair up"));
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
    let ball = ef.ball_from(b, &ef.mid, ef.tm, ts).ok_or_else(|| failed("the fillet's ball is lost along the edge"))?;
    let st = build_strip(b, ef, &[ts], fit_tol)?;
    let st = &st;
    let mut rv = [0usize; 2];
    let mut fe: Vec<usize> = Vec::new();
    for k in 0..2 {
        let q = ball.q[k].p;
        rv[k] = match vertex_on_boundary(nb, ef.faces[k], &q, tol) {
            Some(v) => v,
            None => nb.add_vertex(q),
        };
        let t0 = st.rail_param(k, &q, ts, ts);
        if events[k].is_empty() {
            let rail = nb.add_edge(st.rails[k].clone(), t0, t0 + period, rv[k], rv[k]);
            out.cuts.entry(ef.faces[k]).or_default().push(rail);
            fe.push(rail);
            continue;
        }
        let (mut at_v, mut at_t) = (rv[k], t0);
        let rail_to = |nb: &mut Body, to_t: f64, to_v: usize, at_v: usize, at_t: f64, fe: &mut Vec<usize>, out: &mut Pieces| {
            let rail = add_edge(nb, st.rails[k].clone(), at_t, to_t, at_v, to_v);
            out.cuts.entry(ef.faces[k]).or_default().push(rail);
            fe.push(rail);
        };
        for ev in &events[k] {
            match ev {
                Event::Seam(t, sb) => {
                    let p = sb.q[k].p;
                    let v = vertex_on_boundary(nb, ef.faces[k], &p, tol).ok_or_else(|| failed("a fillet rail's seam crossing isn't on its face's seam"))?;
                    let tr = st.rail_param(k, &p, *t, *t);
                    rail_to(nb, tr, v, at_v, at_t, &mut fe, out);
                    (at_v, at_t) = (v, tr);
                }
                Event::Off(a, z) => {
                    let (pa, pz) = (a.ball.q[k].p, z.ball.q[k].p);
                    let va = vertex_on_boundary(nb, ef.faces[k], &pa, tol).ok_or_else(|| failed("a fillet rail's crossing isn't on its face's boundary"))?;
                    let vz = vertex_on_boundary(nb, ef.faces[k], &pz, tol).ok_or_else(|| failed("a fillet rail's crossing isn't on its face's boundary"))?;
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
    let (c, a0, a1) = ball.arc().ok_or_else(|| failed("a fillet section degenerates"))?;
    let seam = add_edge(nb, c, a0, a1, rv[0], rv[1]);
    fe.push(seam);
    fe.push(seam);
    let u_period = matches!(st.kind, StripKind::Nurbs).then_some(period);
    out.new_faces.push(NewFace { surf: st.surf.clone(), edges: fe, src: FaceSrc::New(Origin::FromEdge { body: input, edge: EdgeId(ef.e as u64) }), u_period, beside: ef.faces[0], pcurves: Vec::new() });
    Ok(())
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
    (th > 1e-6 && th < PI - 1e-6).then(|| d * (th / 2.0).tan())
}

/// The offsets from edge e's two faces (in [`super::blend::edge_sides`] order) of the ball
/// whose contacts lie `d[0]` and `d[1]` from the edge along them, if the faces meet at the same
/// angle θ all along it: in the section, the contact d₀ along the first face is the foot of the
/// point k₀ off it, and k₀ = (d₁ − d₀ cos θ) / sin θ (k₁ likewise); both positive.
pub fn chamfer_offsets(b: &Body, e: usize, d: [f64; 2]) -> Option<[f64; 2]> {
    let th = chamfer_angle(b, e)?;
    if !(th > 1e-6 && th < PI - 1e-6) {
        return None;
    }
    let (sn, cs) = th.sin_cos();
    let k = [(d[1] - d[0] * cs) / sn, (d[0] - d[1] * cs) / sn];
    (k[0] > 0.0 && k[1] > 0.0).then_some(k)
}

/// The angle between edge e's faces' directions away from it, if the same all along it.
pub(crate) fn chamfer_angle(b: &Body, e: usize) -> Option<f64> {
    let ed = b.edges.get(e)?;
    let [(f1, c1), (f2, _)] = sides(b, e).ok()?;
    let mut theta: Option<f64> = None;
    for s in [0.2, 0.5, 0.8] {
        let t = ed.t[0] + (ed.t[1] - ed.t[0]) * s;
        let dir = ed.curve.tangent(t).normalize() * if c1 { 1.0 } else { -1.0 };
        let w = |fi: usize, along: V3| -> Option<V3> {
            let c = coedge_of(b, fi, e)?;
            let q = c.pc.point(t);
            Some(geom::contact(&b.faces[fi], P2::new(q.x, q.y)).n.cross(&along))
        };
        let th = w(f1, dir)?.angle(&w(f2, -dir)?);
        if theta.is_some_and(|x| (x - th).abs() > 1e-9) {
            return None;
        }
        theta = Some(th);
    }
    theta
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
        let k = chamfer_offsets(b, e, d).ok_or_else(|| unsupported("a chamfer along an edge whose faces don't meet at one angle"))?;
        // In the strip's face order (the face whose loop runs along the edge first).
        let [(_, c1), _] = sides(b, e)?;
        let k = if c1 { k } else { [k[1], k[0]] };
        laws.push((e, vec![(0.0, k[0]), (1.0, k[0])]));
        ratios.push(k[1] / k[0]);
    }
    local_blend(b, input, &laws, &ratios, true)
}

/// [`fillet_edges_laws`] with each edge's [`EdgeFillet::ratio`].
fn local_blend(b: &Body, input: BodyId, edges: &[RadiusLaw], ratios: &[f64], flat: bool) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    if edges.iter().any(|(e, law)| *e >= b.edges.len() || law.is_empty()) {
        return Err(KernelError::InvalidParameter("unknown edge or empty radius law".into()));
    }
    let budget = Budget::new(BALL_SOLVES);
    let r = edges.iter().zip(ratios).flat_map(|((_, l), q)| l.iter().map(move |x| x.1 * q.max(1.0))).fold(0.0, f64::max);
    let size = b.bbox().diagonal().max(r);
    // B-spline strips within 1e-5 of the size (their sections exact at the stations): well inside
    // the volume and area agreement wanted, with spans few enough to integrate quickly.
    let fit_tol = 1e-5 * (1.0 + size);
    let vtol = 1e-6 * (1.0 + r);
    let out_of_work = || failed("the local fillet takes more work than its budget");
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
        // The spine parameters the strip must cover and, where an end face isn't met in closed
        // form, the marched end curves.
        let mut cover = Vec::new();
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
                Some(End::Star { t, t_p, .. }) => cover.extend([t[0], t[1], *t_p]),
                Some(End::Square { t, balls, n }) => {
                    cover.extend(t.iter().copied());
                    let pts = march_on(b, ef, &Mark::rail(t[0], balls[0], 0), &Mark::rail(t[1], balls[1], 1), &square_plane(&b.verts[b.edges[ef.e].v[j]].p, n))?;
                    cover.extend(pts.iter().map(|x| x.0));
                    marched[j] = Some(pts);
                }
                Some(End::Open { exits, .. }) => cover.extend([exits[0].t, exits[1].t]),
                Some(End::Point { t }) => cover.push(*t),
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
                None if !ef.closed => return Err(failed("a fillet end without a plan")),
                None => {}
            }
        }
        if ef.closed {
            closed_strip(b, &mut nb, ef, fit_tol, &mut out, input)?;
            continue;
        }
        let mut st = build_strip(b, ef, &cover, fit_tol)?;
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
                    for k in 0..2 {
                        let x = &exits[k];
                        let q = x.ball.q[k].p;
                        rv[j][k] = match x.ext {
                            None => vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("a fillet rail's end isn't on its face's boundary"))?,
                            Some((y, ty)) => {
                                // A new vertex inside the face, joined to the vertex along the edge's
                                // extension.
                                // On the face's boundary the rail runs along one of its edges (the
                                // fillet takes the whole face): not done yet.
                                if on_boundary(&nb, ef.faces[k], &q, vtol) {
                                    return Err(unsupported("a fillet whose rail runs along an edge of its face"));
                                }
                                let pv = nb.add_vertex(q);
                                let ye = &b.edges[y];
                                let tv = ye.t[if ye.v[0] == v { 0 } else { 1 }];
                                let xe = add_edge(&mut nb, ye.curve.clone(), tv, ty, v, pv);
                                out.cuts.entry(ef.faces[k]).or_default().push(xe);
                                ext = Some((k, xe));
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
                    match ext {
                        // The end face grows by the region the end curve and the extension
                        // close off at the vertex (in place of its edge from the other rail's exit
                        // to the vertex).
                        Some((k, xe)) => {
                            if inside_face(b, g, &nb.edges[ee]) {
                                return Err(unsupported("a mixed fillet end inside its end face"));
                            }
                            let o = 1 - k;
                            // From the other rail's exit along the end curve, then back along
                            // the extension to the vertex.
                            let path = vec![(ee, o == 0), (xe, false)];
                            out.patches.push(Patch { face: g, from: rv[j][o], to: v, path, via: None });
                        }
                        // Inside the end face the fillet cuts it; outside, it adds to it (in
                        // place of the face's boundary round the edge's vertex).
                        None if inside_face(b, g, &nb.edges[ee]) => out.cuts.entry(g).or_default().push(ee),
                        None => out.patches.push(Patch { face: g, from: rv[j][0], to: rv[j][1], path: vec![(ee, true)], via: Some(v) }),
                    }
                    across.push(ee);
                }
                End::Open { exits, pts } => {
                    for k in 0..2 {
                        let x = &exits[k];
                        let q = x.ball.q[k].p;
                        rv[j][k] = vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("a fillet rail's end isn't on its face's boundary"))?;
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
                        rv[j][k] = vertex_on_boundary(&mut nb, ef.faces[k], &q, vtol).ok_or_else(|| failed("a fillet rail's end isn't on its face's boundary"))?;
                        rt[j][k] = st.rail_param(k, &q, x.t, x.t);
                    }
                    // One cut on each face round the vertex, from mark to mark.
                    let mut marks = vec![(Mark::rail(exits[0].t, exits[0].ball, 0), rv[j][0], exits[0].into)];
                    for l in links {
                        let xv = vertex_on_boundary(&mut nb, l.after, &l.mark.p, vtol).ok_or_else(|| failed("a fillet end's crossing isn't on its edge"))?;
                        marks.push((l.mark, xv, l.after));
                    }
                    marks.push((Mark::rail(exits[1].t, exits[1].ball, 1), rv[j][1], usize::MAX));
                    for w in marks.windows(2) {
                        let ((ma, va, g), (mz, vz, _)) = (&w[0], &w[1]);
                        let pts = march_on(b, ef, ma, mz, &b.faces[*g])?;
                        let mut xs: Vec<P3> = pts.iter().map(|x| x.1).collect();
                        let last = xs.len() - 1;
                        xs[0] = nb.verts[*va].p;
                        xs[last] = nb.verts[*vz].p;
                        let ca = (ma.w == 0.0).then_some(ma.ball.c);
                        let cz = (mz.w == 1.0).then_some(mz.ball.c);
                        let (c, c0, c1) = fit_cut(&xs, Some(&b.faces[*g]), [ca, cz]);
                        let ee = add_edge(&mut nb, c, c0, c1, *va, *vz);
                        if !inside_face(b, *g, &nb.edges[ee]) {
                            return Err(unsupported("a fillet end over several faces outside them"));
                        }
                        out.cuts.entry(*g).or_default().push(ee);
                        across.push(ee);
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
                                vertex_on_boundary(&mut nb, fa, &mi.q, vtol).ok_or_else(|| failed("a mitre's end isn't on its edge"))?
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
                            rv[j][ko] = vertex_on_boundary(&mut nb, ef.faces[ko], &xp, vtol).ok_or_else(|| failed("a fillet rail's end isn't on its face's boundary"))?;
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
                    let in_plane = (t[0] - t[1]).abs() < 1e-12 * (1.0 + t[0].abs()) && balls[0].q.iter().all(|q| (q.p - balls[0].c).dot(n).abs() < vtol);
                    let across_e = match balls[0].arc().filter(|_| in_plane) {
                        Some((c, a0, a1)) => add_edge(&mut nb, c, a0, a1, rv[j][0], rv[j][1]),
                        None => {
                            let pts = match &marched[j] {
                                Some(pts) => pts.clone(),
                                None => return Err(failed("a square fillet end without its cut")),
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
                            let (c, a0, a1) = ball.arc().ok_or_else(|| failed("a fillet section degenerates"))?;
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
            st = build_strip(b, ef, &more, fit_tol)?;
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
        let x = if m.norm() > 1e-9 { m.normalize() } else { (c.contacts[0].1.p - c.c).normalize() };
        let z = any_perp(&x);
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
    for c in &p.caps {
        let edges: Vec<usize> = c.edges.iter().map(|&e| out.section_edges.get(&SectionKey::Cap(c.v, e)).copied().ok_or_else(|| failed("a chamfer corner's strip has no end"))).collect::<Result<_>>()?;
        let [o, x, y] = [nb.edges[edges[0]].v[0], nb.edges[edges[0]].v[1], nb.edges[edges[1]].v.into_iter().find(|w| !nb.edges[edges[0]].v.contains(w)).ok_or_else(|| failed("a chamfer corner's triangle doesn't close"))?].map(|w| nb.verts[w].p);
        let xd = (x - o).normalize();
        let z = xd.cross(&(y - o)).normalize();
        let surf = Surface::Plane { f: Frame { o, x: xd, y: z.cross(&xd), z } };
        let vi = super::blend::vertex_index(b, c.v);
        out.new_faces.push(NewFace { surf, edges, src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: c.beside, pcurves: Vec::new() });
    }
    let picked: Vec<usize> = p.efs.iter().map(|ef| ef.e).collect();
    assemble(nb, &out, &picked, size)
}

/// The face of a three-sided corner: the bridge on the common face (a new cut there), the two
/// strips' sections, and a Coons patch through them, degenerate at the point on the third edge.
fn tri_face(b: &Body, nb: &mut Body, tc: &TriCorner, out: &mut Pieces, input: BodyId) -> Result<()> {
    let sec: Vec<usize> = tc.edges.iter().map(|&e| out.section_edges.get(&SectionKey::Tri(tc.v, e)).copied().ok_or_else(|| failed("a three-sided corner's strip has no section"))).collect::<Result<_>>()?;
    let tol = 1e-6 * (1.0 + b.bbox().diagonal());
    let near = |nb: &Body, p: &P3| -> Option<usize> { (0..nb.verts.len()).filter(|&v| (nb.verts[v].p - p).norm() < tol).min_by(|&x, &y| (nb.verts[x].p - p).norm().total_cmp(&(nb.verts[y].p - p).norm())) };
    let face = &b.faces[tc.shared];
    let pts: Vec<P3> = tc.bridge.iter().map(|uv| face.surf.eval(uv.x, uv.y)).collect();
    let last = pts.len() - 1;
    // The sections' ends on the common face and on the third edge.
    let other_end = |nb: &Body, e: usize, at: usize| if nb.edges[e].v[0] == at { nb.edges[e].v[1] } else { nb.edges[e].v[0] };
    let sv = near(nb, &tc.side).ok_or_else(|| failed("a three-sided corner's side point has no vertex"))?;
    let (m0, m1) = (other_end(nb, sec[0], sv), other_end(nb, sec[1], sv));
    let mut xs = pts;
    xs[0] = nb.verts[m0].p;
    xs[last] = nb.verts[m1].p;
    let (c, c0, c1) = fit_cut(&xs, None, [None, None]);
    let rr = add_edge(nb, c, c0, c1, m0, m1);
    out.cuts.entry(tc.shared).or_default().push(rr);
    // The patch: u across from the third edge's point, v from the first section to the second.
    let bottom = along_from(nb, sec[0], sv);
    let top = along_from(nb, sec[1], sv);
    let right = along_from(nb, rr, m0);
    let sp = nb.verts[sv].p;
    let left = move |_: f64| sp;
    let surf = coons(&*bottom, &*top, &left, &*right).ok_or_else(|| failed("a three-sided corner's surface can't be built"))?;
    let deg = nb.add_degenerate(sp, 0.0, 1.0, sv);
    // The p-curves: the patch's sides, each linear in its edge's parameter (as sampled).
    let pcs = vec![
        (sec[0], side_pcurve(nb, sec[0], sv, (1, 0.0))),
        (sec[1], side_pcurve(nb, sec[1], sv, (1, 1.0))),
        (rr, side_pcurve(nb, rr, m0, (0, 1.0))),
        // Along u = 0 from v = 1 to v = 0.
        (deg, Curve::Line { o: P3::new(0.0, 1.0, 0.0), d: V3::new(0.0, -1.0, 0.0) }),
    ];
    let vi = super::blend::vertex_index(b, tc.v);
    out.new_faces.push(NewFace { surf, edges: vec![sec[0], rr, sec[1], deg], src: FaceSrc::New(Origin::FromVertex { body: input, vertex: VertexId(vi as u64) }), u_period: None, beside: tc.shared, pcurves: pcs });
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
    let halves: Vec<(usize, usize, usize)> = pc.strips.iter().map(|&i| out.patch_halves.get(&(pc.v, i)).copied().ok_or_else(|| failed("a patched corner's strip has no curve across"))).collect::<Result<_>>()?;
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
        let (ea, eb) = shared.ok_or_else(|| failed("a patched corner's strips don't meet"))?;
        let pv = nb.edges[ea].v.iter().copied().find(|v| nb.edges[eb].v.contains(v)).ok_or_else(|| failed("a patched corner's strips don't meet"))?;
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
        let surf = coons(&*bottom, &top, &*left, &right).ok_or_else(|| failed("a patched corner's surface can't be built"))?;
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
    let curves: Vec<usize> = pc.strips.iter().map(|&i| out.patch_halves.get(&(pc.v, i)).map(|h| h.0).ok_or_else(|| failed("a patched corner's strip has no curve across"))).collect::<Result<_>>()?;
    // Corner points: P[i] where curve i−1 meets curve i; curve i runs from P[i] to P[i+1].
    let mut pts = Vec::new();
    for i in 0..n {
        let (a, c) = (&nb.edges[curves[(i + n - 1) % n]], &nb.edges[curves[i]]);
        let v = a.v.iter().copied().find(|v| c.v.contains(v)).ok_or_else(|| failed("a patched corner's strips don't meet"))?;
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
        let surf = coons(&*bottom, &*top, &*left, &*right).ok_or_else(|| failed("a patched corner's surface can't be built"))?;
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
    let surf = coons(&*bottom, &*top, &left, &*right).ok_or_else(|| failed("a patched corner's surface can't be built"))?;
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
/// vertex `from` to `to` stands in for the stretch between them (the one passing `via` when
/// given, else the one of fewer edges).
#[derive(Clone)]
struct Patch {
    face: usize,
    from: usize,
    to: usize,
    path: Vec<(usize, bool)>,
    via: Option<usize>,
}

/// Whether the middle of edge `e` lies inside face `fi` of `b`.
fn inside_face(b: &Body, fi: usize, e: &super::topo::Edge) -> bool {
    let f = &b.faces[fi];
    let p = e.point(e.mid_t());
    let near = near_chart(b, f, &p);
    let (u, v) = f.surf.project(&p, near.map(|q| (q.x, q.y)));
    let polys = super::classify::face_polygons(b, f);
    super::classify::periodic_candidates(&f.surf, u, v, &polys).into_iter().any(|uv| super::classify::in_face(&polys, &f.surf, uv, 1e-9) == super::classify::InFace::Inside)
}

/// Applies a [`Patch`] to its face's loops.
fn patch(b: &mut Body, p: &Patch) -> Result<()> {
    let bad = || failed("a fillet's end doesn't meet its end face's boundary");
    let fi = p.face;
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

/// Splits the faces by their cuts, drops the pieces between the cuts and the picked edges, and
/// adds the new faces; checks the result.
fn assemble(mut nb: Body, out: &Pieces, picked: &[usize], size: f64) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    for p in &out.patches {
        patch(&mut nb, p)?;
    }
    let mut new_edge = vec![false; nb.edges.len()];
    for &e in out.cuts.values().flatten().chain(out.section_edges.values()) {
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
    for fi in 0..nb.faces.len() {
        match out.cuts.get(&fi) {
            Some(cs) => {
                let ps = split(&nb, &nb.faces[fi], cs);
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
                        return Err(failed("a fillet's rail doesn't split its face"));
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
        if !out.cuts.contains_key(&pieces[pi].0) {
            return Err(failed("the fillet's trimming leaks into faces it doesn't touch"));
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
        if pieces.iter().zip(&removed).all(|((f, _), &rm)| *f != fi || rm) {
            return Err(failed("the fillet consumes a whole face"));
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
        return Err(failed(format!("the local fillet doesn't close: {}", errs[0])));
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
    /// simple Y2, Y4): the top front edge's fillet ends past the slot, its rail on the front
    /// face leaving it into the slot (over an edge away from the end vertex), so the end cut
    /// runs down the slot's faces (and, wider than the slot, across its floor) from the end
    /// face to that rail. The cut on a face holding the strip's straight lines (the slot's top
    /// and floor) once overran its end and folded back.
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
        for r in [15.0, 25.0] {
            let (out, _) = fillet_edges(&b, id, &[e], r).map_err(|e| format!("{e}")).unwrap();
            assert_eq!(out.check_with(CheckLevel::Full), Vec::<String>::new());
            let id2 = k.insert(out);
            // The whole edge's (1 − π/4)r² × 100, less what the slot had already taken of it
            // (its last 10 mm at most).
            let lost = v0 - k.mass_properties(id2).unwrap().volume;
            let whole = (1.0 - PI / 4.0) * r * r * 100.0;
            assert!(lost > 0.9 * whole && lost < whole, "r {r}: {lost} of {whole}");
        }
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
