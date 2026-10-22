//! Wire tails removed from faces (OCCT's `ShapeFix_Wire::FixTails` with
//! `ShapeAnalysis_Wire::CheckTail`, behind DRAW's `fixshape -maxtaila -maxtailw`).
//!
//! A tail is where a face's boundary runs out and back along itself: two consecutive edges
//! leaving their common vertex in nearly the same direction (the sine of the angle between them
//! at most `max_sine`) and staying within `max_width` of each other for a while. The part of
//! each edge within that width of the other is cut away; the cut ends, at most `max_width`
//! apart, become one vertex (its tolerance covering both). An edge lying wholly within the
//! width goes altogether. The edges are trimmed in place (their p-curves keep their
//! parametrization), so only edges of one face are fixed.

use super::curve::Curve;
use super::math::*;
use super::topo::{co_range, Body, Coedge};
use crate::{KernelError, Result};
use super::tol;

/// OCCT's `Precision::Confusion()`.
const CONFUSION: f64 = 1e-7;

/// One side of a tail: a coedge's edge with its parameter at the tail's vertex (`near`) and at
/// its other end (`far`).
struct Side {
    curve: Curve,
    near: f64,
    far: f64,
}

impl Side {
    fn of(b: &Body, c: &Coedge, at_end: bool) -> Side {
        let e = &b.edges[c.edge];
        let (s, t) = co_range(e, c.fwd);
        let (near, far) = if at_end { (t, s) } else { (s, t) };
        Side { curve: e.curve.clone(), near, far }
    }
    /// The distance from `p` to the side's whole edge.
    fn dist(&self, p: &P3) -> f64 {
        let (lo, hi) = (self.near.min(self.far), self.near.max(self.far));
        (self.curve.point(self.curve.closest(p, lo, hi)) - p).norm()
    }
    fn project(&self, p: &P3) -> f64 {
        let (lo, hi) = (self.near.min(self.far), self.near.max(self.far));
        self.curve.closest(p, lo, hi)
    }
}

/// What a tail check found: the new near-end parameter of each side (equal to `far` when the
/// whole edge goes, to `near` when it stays as it is).
struct Cut {
    at: [f64; 2],
}

/// `ShapeAnalysis_Wire::CheckTail` for the coedge `a` running into the vertex and `b` running
/// out of it.
fn check_tail(sides: &[Side; 2], max_sine: Option<f64>, max_width: f64) -> Option<Cut> {
    let tol2 = max_width + 0.5 * CONFUSION;
    let tol3 = max_width + CONFUSION;
    let tol4 = max_width + 1.5 * CONFUSION;
    let p = [sides[0].curve.point(sides[0].near), sides[1].curve.point(sides[1].near)];
    if (p[0] - p[1]).norm() > tol2 {
        return None;
    }
    if let Some(max_sine) = max_sine {
        // The directions leaving the vertex along each edge.
        let mut d = [V3::zeros(); 2];
        for (k, s) in sides.iter().enumerate() {
            let len = s.curve.length(s.near.min(s.far), s.near.max(s.far));
            if len < 0.5 * CONFUSION {
                return None;
            }
            let t = s.curve.tangent(s.near) * if s.far >= s.near { 1.0 } else { -1.0 };
            d[k] = t.try_normalize(tol::TINY)?;
        }
        if d[0].dot(&d[1]) < 0.0 || d[0].cross(&d[1]).norm_squared() > max_sine * max_sine {
            return None;
        }
    }
    // Each side's stretch within the width of the other: up to `bound` (its parameter), with
    // the foot of that point on the other side.
    let mut bound = [0.0; 2];
    let mut foot = [0.0; 2];
    let mut dists = [0.0; 2];
    let mut whole = [true, true];
    for k in 0..2 {
        let (s, o) = (&sides[k], &sides[1 - k]);
        bound[k] = s.far;
        let q = s.curve.point(s.far);
        dists[k] = o.dist(&q);
        foot[k] = o.project(&q);
        if dists[k] <= tol2 {
            continue;
        }
        whole[k] = false;
        let mut inside = s.near;
        for _ in 0..200 {
            let m = 0.5 * (inside + bound[k]);
            let q = s.curve.point(m);
            let dm = o.dist(&q);
            if dm <= tol2 {
                inside = m;
            } else {
                bound[k] = m;
                foot[k] = o.project(&q);
                if dm <= tol3 {
                    break;
                }
            }
        }
    }
    // The stretches must stay within the width all along.
    for k in 0..2 {
        let (s, o) = (&sides[k], &sides[1 - k]);
        for i in 1..23 {
            let t = s.near + (bound[k] - s.near) * i as f64 / 23.0;
            if o.dist(&s.curve.point(t)) > tol4 {
                return None;
            }
        }
    }
    if whole[0] && whole[1] && (sides[0].curve.point(bound[0]) - sides[1].curve.point(bound[1])).norm() <= tol3 {
        return Some(Cut { at: [sides[0].far, sides[1].far] });
    }
    // One side cut at its stretch's end, the other at that point's foot.
    let mut fi = 0;
    if whole[0] || whole[1] {
        fi = if whole[0] { 0 } else { 1 };
        if dists[1 - fi] < dists[fi] && whole[1 - fi] {
            fi = 1 - fi;
        }
    }
    let mut at = [0.0; 2];
    at[fi] = bound[fi];
    at[1 - fi] = foot[fi];
    let pconf = tol::ROUNDOFF;
    let mut changed = false;
    for k in 0..2 {
        let s = &sides[k];
        if (at[k] - s.far).abs() <= pconf {
            at[k] = s.far;
            changed = true;
        } else if (at[k] - s.near).abs() <= pconf || s.curve.length(s.near.min(at[k]), s.near.max(at[k])) <= CONFUSION {
            at[k] = s.near;
        } else if s.curve.length(at[k].min(s.far), at[k].max(s.far)) <= CONFUSION {
            at[k] = s.far;
            changed = true;
        } else {
            changed = true;
        }
    }
    changed.then_some(Cut { at })
}

/// Removes the tails of every face's loops (see the module docs). `max_angle` is in radians.
/// An edge a tail cuts must belong to that face only.
pub fn fix_tails(b: &mut Body, max_angle: f64, max_width: f64) -> Result<bool> {
    if max_width < 0.0 {
        return Ok(false);
    }
    let max_sine = max_angle.sin();
    let mut uses = vec![0usize; b.edges.len()];
    for c in b.faces.iter().flat_map(|f| f.loops.iter().flatten()) {
        uses[c.edge] += 1;
    }
    let mut done = false;
    for fi in 0..b.faces.len() {
        for li in 0..b.faces[fi].loops.len() {
            // Pairs (n−1, 0), (0, 1), …: the coedge running into a vertex and the one out of it.
            let mut i = 0usize;
            let mut check_angle = true;
            let mut guard = 0;
            loop {
                let n = b.faces[fi].loops[li].len();
                if n < 2 || i >= n || guard > 10 * (n + 10) {
                    break;
                }
                guard += 1;
                let (ia, ib) = ((i + n - 1) % n, i);
                let (ca, cb) = (b.faces[fi].loops[li][ia].clone(), b.faces[fi].loops[li][ib].clone());
                if ca.edge == cb.edge || b.edges[ca.edge].degenerate || b.edges[cb.edge].degenerate {
                    i += 1;
                    check_angle = true;
                    continue;
                }
                let sides = [Side::of(b, &ca, true), Side::of(b, &cb, false)];
                let Some(cut) = check_tail(&sides, check_angle.then_some(max_sine), max_width) else {
                    i += 1;
                    check_angle = true;
                    continue;
                };
                // At least one edge must stay in the loop.
                let whole = (0..2).filter(|&k| cut.at[k] == sides[k].far).count();
                let split = (0..2).filter(|&k| cut.at[k] != sides[k].far && cut.at[k] != sides[k].near).count();
                if n + split < 1 + whole + split {
                    i += 1;
                    check_angle = true;
                    continue;
                }
                if uses[ca.edge] > 1 || uses[cb.edge] > 1 {
                    return Err(KernelError::Unsupported("tails along edges other faces use"));
                }
                done = true;
                // The new ends: a vertex between the two cut points.
                let pa = sides[0].curve.point(cut.at[0]);
                let pb = sides[1].curve.point(cut.at[1]);
                let v = b.add_vertex(P3::from((pa.coords + pb.coords) / 2.0));
                b.verts[v].tol = ((pa - pb).norm() / 2.0 * 1.01).max(b.verts[v].tol);
                let mut remove = Vec::new();
                for (k, c) in [(0, &ca), (1, &cb)] {
                    let s = &sides[k];
                    if cut.at[k] == s.far {
                        remove.push(if k == 0 { ia } else { ib });
                        continue;
                    }
                    let e = &mut b.edges[c.edge];
                    // The near end moves to the cut; the end at the vertex is `near`.
                    let near_is_t1 = (e.t[1] - s.near).abs() <= (e.t[0] - s.near).abs();
                    if near_is_t1 {
                        e.t[1] = cut.at[k];
                        e.v[1] = v;
                    } else {
                        e.t[0] = cut.at[k];
                        e.v[0] = v;
                    }
                }
                // A removed edge's far vertex joins the new one.
                for &r in &remove {
                    let c = &b.faces[fi].loops[li][r];
                    let e = &b.edges[c.edge];
                    let fv = if r == ia { b.co_start(c) } else if c.fwd { e.v[1] } else { e.v[0] };
                    let fp = b.verts[fv].p;
                    for e2 in b.edges.iter_mut() {
                        for ev in e2.v.iter_mut() {
                            if *ev == fv {
                                *ev = v;
                            }
                        }
                    }
                    let d = (fp - b.verts[v].p).norm();
                    b.verts[v].tol = b.verts[v].tol.max(d * 1.01);
                }
                remove.sort_unstable();
                for &r in remove.iter().rev() {
                    b.faces[fi].loops[li].remove(r);
                }
                if remove.contains(&ia) && ia < i {
                    i = i.saturating_sub(1);
                }
                check_angle = false;
            }
        }
    }
    if done {
        super::unify::compact(b);
        // The joined ends lie within their vertices' tolerances.
        for ei in 0..b.edges.len() {
            for k in 0..2 {
                let e = &b.edges[ei];
                let d = (e.point(e.t[k]) - b.verts[e.v[k]].p).norm();
                let v = e.v[k];
                if d * 1.01 > b.verts[v].tol {
                    b.verts[v].tol = d * 1.01;
                }
            }
        }
    }
    Ok(done)
}
