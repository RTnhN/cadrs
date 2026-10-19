//! Marching along the intersection of two surfaces from a seed point.
//!
//! The curve is followed along t = n_a × n_b. Each step predicts the next point a distance h
//! along t and corrects it onto both surfaces with Newton's method on (u_a, v_a, u_b, v_b): the
//! points of the two surfaces must agree, and the step must keep its length along t (the point
//! stays on the plane square to t at distance h). The step grows where the curve runs straight and
//! shrinks where it turns (an angle between successive tangents of 0.03 to 0.1 radians). A
//! march stops on the boundary of a's parameter box (the end solved on it exactly), where the
//! surfaces turn tangent (t vanishes), or when it comes back to its start (a closed curve).

use super::math::*;
use super::surface::Surface;

/// A marched curve: its points in order, and whether it closes on itself.
pub struct Marched {
    pub pts: Vec<P3>,
    pub closed: bool,
}

/// The parameter box of the first surface (the second is unbounded).
#[derive(Clone, Copy)]
pub struct UvBox {
    pub u: (f64, f64),
    pub v: (f64, f64),
}

impl UvBox {
    fn inside(&self, u: f64, v: f64) -> bool {
        let (eu, ev) = ((self.u.1 - self.u.0) * 1e-9, (self.v.1 - self.v.0) * 1e-9);
        u >= self.u.0 - eu && u <= self.u.1 + eu && v >= self.v.0 - ev && v <= self.v.1 + ev
    }
}

#[derive(Clone, Copy)]
struct State {
    p: P3,
    a: (f64, f64),
    b: (f64, f64),
}

fn tangent(a: &Surface, b: &Surface, s: &State) -> Option<V3> {
    let na = a.normal(s.a.0, s.a.1);
    let nb = b.normal(s.b.0, s.b.1);
    na.cross(&nb).try_normalize(1e-7)
}

/// Newton for a point on both surfaces satisfying one more condition `extra` (its value and
/// its gradient with respect to a's (u, v)).
fn correct(a: &Surface, b: &Surface, mut s: State, extra: &dyn Fn(&P3, &V3, &V3) -> (f64, f64, f64), scale: f64) -> Option<State> {
    for _ in 0..20 {
        let (pa, au, av) = a.d1(s.a.0, s.a.1);
        let (pb, bu, bv) = b.d1(s.b.0, s.b.1);
        let d = pa - pb;
        let (g, gu, gv) = extra(&pa, &au, &av);
        if d.norm() < 1e-12 * scale && g.abs() < 1e-12 * scale {
            s.p = pa;
            return Some(s);
        }
        let j = nalgebra::Matrix4::new(
            au.x, av.x, -bu.x, -bv.x, //
            au.y, av.y, -bu.y, -bv.y, //
            au.z, av.z, -bu.z, -bv.z, //
            gu, gv, 0.0, 0.0,
        );
        let f = nalgebra::Vector4::new(-d.x, -d.y, -d.z, -g);
        // The 4×4 is square but the first three rows can be dependent (surfaces nearly tangent):
        // least squares through the normal equations.
        let jt = j.transpose();
        let dx = (jt * j).try_inverse().map(|inv| inv * jt * f)?;
        s.a = (s.a.0 + dx[0], s.a.1 + dx[1]);
        s.b = (s.b.0 + dx[2], s.b.1 + dx[3]);
        if !dx.iter().all(|x| x.is_finite()) {
            return None;
        }
    }
    let (pa, pb) = (a.eval(s.a.0, s.a.1), b.eval(s.b.0, s.b.1));
    if (pa - pb).norm() < 1e-9 * scale {
        s.p = pa;
        Some(s)
    } else {
        None
    }
}

/// Marches from the seed (a point on both surfaces near `uv_a` on `a`) both ways.
pub fn march(a: &Surface, bx: UvBox, b: &Surface, uv_a: P2, scale: f64) -> Option<Marched> {
    let p0 = a.eval(uv_a.x, uv_a.y);
    let ub = b.project(&p0, None);
    let start = correct(a, b, State { p: p0, a: (uv_a.x, uv_a.y), b: ub }, &|_, _, _| (0.0, 0.0, 0.0), scale)?;
    if !bx.inside(start.a.0, start.a.1) {
        return None;
    }
    let h_max = scale / 40.0;
    let h_min = scale * 1e-6;
    let mut halves: [Vec<P3>; 2] = [Vec::new(), Vec::new()];
    for (side, sign) in [1.0, -1.0].into_iter().enumerate() {
        let mut s = start;
        let mut t_prev = tangent(a, b, &s)? * sign;
        let mut h = h_max / 4.0;
        let mut out = Vec::new();
        for _ in 0..20000 {
            let Some(t) = tangent(a, b, &s).map(|t| if t.dot(&t_prev) < 0.0 { -t } else { t }) else { break };
            let target = s.p + t * h;
            let p_from = s.p;
            let step = |q: &P3, qu: &V3, qv: &V3| ((q - p_from).dot(&t) - h, qu.dot(&t), qv.dot(&t));
            // Predict: a's parameters moved along t's components.
            let (_, au, av) = a.d1(s.a.0, s.a.1);
            let guess = solve2(au.dot(&au), au.dot(&av), au.dot(&av), av.dot(&av), au.dot(&(t * h)), av.dot(&(t * h))).unwrap_or((0.0, 0.0));
            let pred = State { p: target, a: (s.a.0 + guess.0, s.a.1 + guess.1), b: b.project(&target, Some(s.b)) };
            let Some(next) = correct(a, b, pred, &step, scale) else {
                if h <= h_min {
                    break;
                }
                h *= 0.5;
                continue;
            };
            // Turning too fast: a shorter step.
            let t_next = tangent(a, b, &next).map(|x| if x.dot(&t) < 0.0 { -x } else { x });
            let turn = t_next.map_or(0.0, |x| x.dot(&t).clamp(-1.0, 1.0).acos());
            if turn > 0.1 && h > h_min {
                h *= 0.5;
                continue;
            }
            // Leaving a's box: the end on its side.
            if !bx.inside(next.a.0, next.a.1) {
                if let Some(end) = boundary_end(a, b, bx, &s, &next, scale) {
                    out.push(end.p);
                }
                break;
            }
            // Back at the start: closed.
            if side == 0 && out.len() > 3 && (next.p - start.p).norm() < h * 0.6 {
                return Some(Marched { pts: std::iter::once(start.p).chain(out).collect(), closed: true });
            }
            out.push(next.p);
            t_prev = t;
            s = next;
            if turn < 0.03 {
                h = (h * 1.5).min(h_max);
            }
        }
        halves[side] = out;
    }
    let [fwd, mut back] = halves;
    back.reverse();
    let pts: Vec<P3> = back.into_iter().chain(std::iter::once(start.p)).chain(fwd).collect();
    (pts.len() >= 2).then_some(Marched { pts, closed: false })
}

/// Where the march from `s` (inside) to `out` (outside) crosses a's box: the side crossed fixed
/// and Newton on the rest.
fn boundary_end(a: &Surface, b: &Surface, bx: UvBox, s: &State, out: &State, scale: f64) -> Option<State> {
    // The side: the first one the straight uv step crosses.
    let (du, dv) = (out.a.0 - s.a.0, out.a.1 - s.a.1);
    let mut best: Option<(f64, bool, f64)> = None; // (fraction, fix u, value)
    for (val, fix_u) in [(bx.u.0, true), (bx.u.1, true), (bx.v.0, false), (bx.v.1, false)] {
        let (from, d) = if fix_u { (s.a.0, du) } else { (s.a.1, dv) };
        if d.abs() < 1e-300 {
            continue;
        }
        let f = (val - from) / d;
        if (0.0..=1.0).contains(&f) && best.is_none_or(|b| f < b.0) {
            best = Some((f, fix_u, val));
        }
    }
    let (f, fix_u, val) = best?;
    let guess = State { p: s.p + (out.p - s.p) * f, a: (s.a.0 + du * f, s.a.1 + dv * f), b: (s.b.0 + (out.b.0 - s.b.0) * f, s.b.1 + (out.b.1 - s.b.1) * f) };
    let mut g = guess;
    if fix_u {
        g.a.0 = val;
    } else {
        g.a.1 = val;
    }
    // The fixed parameter as the extra condition, in terms of a's (u, v).
    let fixed = move |_: &P3, _: &V3, _: &V3| -> (f64, f64, f64) { (0.0, if fix_u { 1.0 } else { 0.0 }, if fix_u { 0.0 } else { 1.0 }) };
    let r = correct(a, b, g, &fixed, scale)?;
    // (The extra row only pins the fixed parameter's change at zero.)
    Some(r)
}
