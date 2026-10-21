//! Coincident surfaces: where two surfaces are one over a region, their intersection is that
//! region's boundary (SMLib's `SM_TC_REGION_BOUNDARY` curves): the sides of each surface's
//! parameter box that lie on the other surface inside its box.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::ssi::IntCurve;
use super::surface::Surface;

type UvBox = ((f64, f64), (f64, f64));

/// Whether (u, v) lies in the box (angles modulo a turn), within a relative `e`.
fn in_box(s: &Surface, bx: UvBox, (u, v): (f64, f64), e: f64) -> bool {
    let ok = |x: f64, (lo, hi): (f64, f64), periodic: bool| {
        let m = (hi - lo) * e + 1e-12 * (1.0 + lo.abs().max(hi.abs()));
        if periodic {
            hi - lo >= TAU - 1e-9 || wrap(x, lo - m, TAU) <= hi + m
        } else {
            x >= lo - m && x <= hi + m
        }
    };
    ok(u, bx.0, s.u_periodic()) && ok(v, bx.1, s.v_periodic())
}

/// Whether all of `s` over `bx` lies on `o` (within `tol`): a 9 × 9 grid of its points, each
/// projected from a coarse start.
pub fn lies_on(s: &Surface, bx: UvBox, o: &Surface, tol: f64) -> bool {
    let ((u0, u1), (v0, v1)) = bx;
    if ![u0, u1, v0, v1].iter().all(|x| x.is_finite()) || (u1 - u0).abs() > 1e3 || (v1 - v0).abs() > 1e3 {
        return false;
    }
    let mut hint = None;
    (0..9).all(|i| {
        (0..9).all(|j| {
            let p = s.eval(u0 + (u1 - u0) * i as f64 / 8.0, v0 + (v1 - v0) * j as f64 / 8.0);
            let near = |uv: (f64, f64)| (o.eval(uv.0, uv.1) - p).norm() <= tol;
            let uv = o.project(&p, hint);
            let ok = near(uv) || {
                let c = o.project_coarse(&p, 16);
                near(o.project(&p, Some(c)))
            };
            hint = Some(uv);
            ok
        })
    })
}

/// The boundary of the region where `a` over `ua` and `b` over `ub` coincide, within `tol`.
pub fn region_boundary(a: &Surface, ua: UvBox, b: &Surface, ub: UvBox, tol: f64) -> Vec<IntCurve> {
    let mut out: Vec<IntCurve> = Vec::new();
    for (s, bx, o, obx) in [(a, ua, b, ub), (b, ub, a, ua)] {
        for c in sides_on(s, bx, o, obx, tol) {
            // A side found from both surfaces (shared edges of the boxes), or the two sides of
            // a full turn's seam: once.
            let dup = out.iter().any(|d| {
                [0.2, 0.5, 0.8].iter().all(|f| {
                    let p = c.c.point(c.t0 + (c.t1 - c.t0) * f);
                    (d.c.point(d.c.closest(&p, d.t0, d.t1)) - p).norm() < 10.0 * tol
                })
            });
            if std::env::var_os("CADRS_SSI_DEBUG").is_some() {
                eprintln!("coincident side: {:?} .. {:?} length {:.4}{}", c.c.point(c.t0), c.c.point(c.t1), c.c.length(c.t0, c.t1), if dup { " (again)" } else { "" });
            }
            if !dup {
                out.push(c);
            }
        }
    }
    out
}

/// The parts of the sides of `s`'s box `bx` that lie on `o` inside its box `obx`.
fn sides_on(s: &Surface, bx: UvBox, o: &Surface, obx: UvBox, tol: f64) -> Vec<IntCurve> {
    let ((u0, u1), (v0, v1)) = bx;
    let mut out = Vec::new();
    // (fixed u?, its value, the other coordinate's range)
    for (fix_u, val, (t0, t1)) in [(true, u0, (v0, v1)), (true, u1, (v0, v1)), (false, v0, (u0, u1)), (false, v1, (u0, u1))] {
        if !(t1 > t0) || !val.is_finite() {
            continue;
        }
        let at = |t: f64| if fix_u { s.eval(val, t) } else { s.eval(t, val) };
        let on = |t: f64, hint: Option<(f64, f64)>| -> (bool, (f64, f64)) {
            let p = at(t);
            let uv = o.project(&p, hint);
            let d = (o.eval(uv.0, uv.1) - p).norm();
            (d <= tol && in_box(o, obx, uv, 1e-9), uv)
        };
        const N: usize = 64;
        let mut hint = None;
        let flags: Vec<(f64, bool)> = (0..=N)
            .map(|i| {
                let t = t0 + (t1 - t0) * i as f64 / N as f64;
                let (f, uv) = on(t, hint);
                hint = Some(uv);
                (t, f)
            })
            .collect();
        // The runs of samples on `o`, their ends found by bisection.
        let edge = |inside: f64, outside: f64| {
            let (mut a, mut b) = (inside, outside);
            for _ in 0..50 {
                let m = 0.5 * (a + b);
                if on(m, None).0 {
                    a = m;
                } else {
                    b = m;
                }
            }
            a
        };
        let mut k = 0;
        while k <= N {
            if !flags[k].1 {
                k += 1;
                continue;
            }
            let mut e = k;
            while e < N && flags[e + 1].1 {
                e += 1;
            }
            let ra = if k == 0 { flags[0].0 } else { edge(flags[k].0, flags[k - 1].0) };
            let rb = if e == N { flags[N].0 } else { edge(flags[e].0, flags[e + 1].0) };
            k = e + 1;
            if (at(rb) - at(ra)).norm() <= 10.0 * tol && (at(0.5 * (ra + rb)) - at(ra)).norm() <= 10.0 * tol {
                continue; // a point (a pole), or nothing
            }
            let iso = match s {
                Surface::Nurbs(n) => Some(Curve::Nurbs(Arc::new(nurbs_iso(n, fix_u, val)))),
                _ if fix_u => s.iso_u(val),
                _ => s.iso_v(val),
            };
            let c = match iso {
                Some(c) => IntCurve { c, t0: ra, t1: rb, closed: (at(ra) - at(rb)).norm() <= tol },
                None => {
                    // A side with no iso curve of its own (a surface of revolution's): interpolated
                    // through its points (exact on `s`).
                    let pts: Vec<P3> = (0..=128).map(|i| at(ra + (rb - ra) * i as f64 / 128.0)).collect();
                    let ts: Vec<f64> = (0..=128).map(|i| ra + (rb - ra) * i as f64 / 128.0).collect();
                    let n = NurbsCurve::interpolate(&pts, &ts, 3, None, None);
                    let r = n.range();
                    IntCurve { c: Curve::Nurbs(Arc::new(n)), t0: r.0, t1: r.1, closed: false }
                }
            };
            out.push(c);
        }
    }
    out
}

/// The iso curve u = `val` (v = `val` when not `fix_u`) of a B-spline surface, exactly: the
/// control net's rows (columns) combined by the basis functions at `val`, in homogeneous
/// coordinates.
fn nurbs_iso(n: &super::nurbs::NurbsSurface, fix_u: bool, val: f64) -> NurbsCurve {
    use super::nurbs::{basis_ders, find_span, H};
    let (p, knots, count) = if fix_u { (n.du, &n.ku, n.cps.len()) } else { (n.dv, &n.kv, n.cps[0].len()) };
    let span = find_span(count - 1, p, val, knots);
    let basis = &basis_ders(span, val, p, 0, knots)[0];
    let cps: Vec<H> = if fix_u {
        (0..n.cps[0].len()).map(|j| (0..=p).map(|k| n.cps[span - p + k][j] * basis[k]).sum()).collect()
    } else {
        (0..n.cps.len()).map(|i| (0..=p).map(|k| n.cps[i][span - p + k] * basis[k]).sum()).collect()
    };
    let (degree, knots) = if fix_u { (n.dv, n.kv.clone()) } else { (n.du, n.ku.clone()) };
    NurbsCurve { degree, knots, cps }
}
