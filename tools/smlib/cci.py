"""SMLib's curve–curve and curve–surface intersection tests: the `cci-advanced` suite
(smAdvancedTest.cpp, `my_test_cci_adv`: the two curves of `cci_bug_rhino1.smp`) and the unit
tests it sits beside (`my_test_cci_analy`, `my_test_cci_tangent`, `my_test_cci_coincidence`,
`my_test_cci_nurb` in smCurveTest.cpp; `my_test_csi_analy` in smSurfaceTest.cpp), all through
`GlobalCurveIntersect` at `SM_ZONE_TOL_3D` (1e-5): a near miss within it is an intersection.

Ported by hand (the `PRELUDE`): SMLib's B-spline lines, circles and ellipses (`my_create_line`,
`my_create_circle`, `my_create_ellipse`) as the kernel's exact curves (the counts don't depend
on the parametrisation), `my_create_nurb` with its knots (internal knots i/(n − p + 1)), the
bilinear and cone patches as B-spline planes, cylinders and cones. Curve–curve is
`ssi::curve_curve`, curve–surface `ssi::curve_surface`. Each check:
- every point returned is within the tolerance on both curves (on the surface, in the patch);
- none against some as SMLib asserts (SMLib counts seam duplicates, near-miss pairs and
  coincident ranges its own way); the distinct points' count where SMLib's test calls its count
  exact and the curves cross (the `my_test_cci_nurb` crossings);
- the gap sweeps of `my_test_cci_analy` step by step, skipping the steps within 5% of the
  tolerance of where SMLib's expectation turns between none and some (there the answer is the
  tolerance's rounding);
- cci_bug_rhino1: SMLib's (the oracle's `cci`) point solutions lie on one of ours.
"""
from smlib_to_rust import Case, Fixture, rust_f64
from smlib._oracle import oracle_run, points_literal

SOURCE = "smCurveTest.cpp, smSurfaceTest.cpp, smAdvancedTest.cpp"
TITLE = "curve-curve and curve-surface intersection"

NOT_CONVERTED = [
    ("my_test_cci_analy 1", "times 100000 `GlobalPointSolve` calls: a benchmark, no assert"),
    ("my_test_cci_tol (smCurveTest.cpp)", "not run by SMLib's suites"),
    ("my_test_csi_nurb (smSurfaceTest.cpp:3874)",
     "its surface comes from `my_create_crease_surf` over the `my_load_surf1_pts` data with creases, not ported"),
    ("my_test_csi_analy counts", "SMLib's line grid comes from its B-spline's bounding box (the control hull) and its counts include seam duplicates: only none against some is compared"),
]

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use std::f64::consts::TAU;
use std::sync::Arc;

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::math::{Affine, Frame, P3, V3};
use cadrs_kernel::native::nurbs::{hom, NurbsCurve, NurbsSurface};
use cadrs_kernel::native::smb;
use cadrs_kernel::native::ssi;
use cadrs_kernel::native::surface::Surface;
use nalgebra::{Rotation3, Unit};

const SUITE: &str = "cci";

/// SMLib's `SM_ZONE_TOL_3D`.
const TOL: f64 = 1e-5;

/// A bounded curve.
struct C {
    c: Curve,
    t0: f64,
    t1: f64,
}

fn p3(a: [f64; 3]) -> P3 {
    P3::new(a[0], a[1], a[2])
}

/// `my_create_line`.
fn line(a: [f64; 3], b: [f64; 3]) -> C {
    C { c: Curve::Line { o: p3(a), d: p3(b) - p3(a) }, t0: 0.0, t1: 1.0 }
}

/// `my_create_circle`: in the xy plane.
fn circle(r: f64, c: [f64; 3]) -> C {
    C { c: Curve::Circle { c: p3(c), x: V3::x(), y: V3::y(), r }, t0: 0.0, t1: TAU }
}

/// `my_create_ellipse`: in the xy plane, `a` along x.
fn ellipse(a: f64, b: f64, c: [f64; 3]) -> C {
    C { c: Curve::Ellipse { c: p3(c), x: V3::x(), y: V3::y(), a, b }, t0: 0.0, t1: TAU }
}

/// `my_create_nurb`: the points moved by `tr`, clamped knots on [0, 1] with the internal ones at
/// i/(n − p + 1).
fn nurb(tr: [f64; 3], pts: &[[f64; 3]], p: usize) -> C {
    let nk = pts.len() - p + 1;
    let mut knots = vec![0.0; p + 1];
    knots.extend((1..nk - 1).map(|i| i as f64 / nk as f64));
    knots.extend(vec![1.0; p + 1]);
    let cps = pts.iter().map(|q| hom(&(p3(*q) + p3(tr).coords), 1.0)).collect();
    let n = NurbsCurve::new(p, knots, cps).expect("a valid B-spline");
    C { c: Curve::nurbs(n), t0: 0.0, t1: 1.0 }
}

/// Turned by `rad` radians about the line through the origin along `axis`.
fn turned(c: C, axis: [f64; 3], rad: f64) -> C {
    let m = Rotation3::from_axis_angle(&Unit::new_normalize(V3::new(axis[0], axis[1], axis[2])), rad).into_inner();
    C { c: c.c.transformed(&Affine { m, t: V3::zeros() }), ..c }
}

/// Placed on the frame (origin, x, y) (`SmAxis2Placement::SetCanonical`, then `Transform`).
fn placed(c: C, o: [f64; 3], x: [f64; 3], y: [f64; 3]) -> C {
    let (x, y) = (V3::new(x[0], x[1], x[2]).normalize(), V3::new(y[0], y[1], y[2]).normalize());
    let m = nalgebra::Matrix3::from_columns(&[x, y, x.cross(&y)]);
    C { c: c.c.transformed(&Affine { m, t: p3(o).coords }), ..c }
}

/// `GlobalCurveIntersect`: where `a` and `b` meet within TOL, each point checked.
fn cci(a: &C, b: &C) -> Result<Vec<P3>, String> {
    let hits = ssi::curve_curve(&a.c, a.t0, a.t1, &b.c, b.t0, b.t1, TOL);
    let mut out = Vec::new();
    for (ta, tb) in hits {
        let slack = 1e-9 * (a.t1 - a.t0).abs().max(b.t1 - b.t0).max(1.0);
        if ta < a.t0 - slack || ta > a.t1 + slack || tb < b.t0 - slack || tb > b.t1 + slack {
            return Err(format!("({ta}, {tb}) lies off the curves' ranges"));
        }
        let (pa, pb) = (a.c.point(ta), b.c.point(tb));
        let gap = (pa - pb).norm();
        if gap > TOL * (1.0 + 1e-6) + 1e-12 {
            return Err(format!("the curves are {gap:.2e} apart at ({ta}, {tb})"));
        }
        out.push(nalgebra::center(&pa, &pb));
    }
    Ok(out)
}

/// `a` and `b` meet (or not, `any`), at `distinct` distinct points when given.
fn expect(a: &C, b: &C, any: bool, distinct: Option<usize>) -> Result<(), String> {
    let pts = cci(a, b)?;
    if !pts.is_empty() != any {
        return Err(format!("{} points, SMLib finds {}", pts.len(), if any { "some" } else { "none" }));
    }
    if let Some(n) = distinct {
        let d = intersect::distinct(&pts, 1e-6);
        if d.len() != n {
            return Err(format!("{} distinct points ({} in all), SMLib finds {n}", d.len(), pts.len()));
        }
    }
    Ok(())
}

/// A gap sweep of `my_test_cci_analy`: `n` steps of x from `x0` by `dx` (added up as SMLib
/// does), the curves `make(x)`, SMLib's count `want(x)`; steps within 5% of TOL of where its
/// expectation turns between none and some are skipped.
fn sweep(n: usize, x0: f64, dx: f64, make: impl Fn(f64) -> (C, C), want: impl Fn(f64) -> usize) -> Result<(), String> {
    let mut x = x0;
    let mut errs = Vec::new();
    for i in 0..n {
        let d = 0.05 * TOL;
        if (want(x - d) > 0) == (want(x + d) > 0) {
            let (a, b) = make(x);
            match cci(&a, &b) {
                Ok(p) if p.is_empty() == (want(x) > 0) => errs.push(format!("step {i} (x = {x:.2e}): {} points, SMLib asserts {}", p.len(), want(x))),
                Ok(_) => {}
                Err(e) => errs.push(format!("step {i} (x = {x:.2e}): {e}")),
            }
        }
        x += dx;
    }
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}

/// `my_test_cci_analy` case `case`.
fn analy(case: u32) -> Result<(), String> {
    const E: f64 = 1e-12; // SM_EFF_ZERO
    let (t, dx) = (TOL, TOL / 10.0);
    match case {
        2 => sweep(40, -2.0 * t, dx, |x| (turned(circle(0.5, [-0.5 + x, 0.0, 0.0]), [1.0, 0.0, 0.0], 30.0), circle(1.0, [0.0; 3])), |x| {
            if x < -(t + E * (1.0 + t)) { 0 } else if x <= t + E * (1.0 + t) { 1 } else { 0 }
        }),
        3 => sweep(40, -2.0 * t, dx, |x| (circle(0.5, [0.5 + x, 0.0, 0.0]), circle(1.0, [0.0; 3])), |x| {
            if x < -0.999 * t { 0 } else if x <= 1e-9 { 4 } else { 2 }
        }),
        4 => sweep(40, -2.0 * t, dx, |x| (circle(0.5, [0.0, 0.5 + x, 0.0]), circle(1.0, [0.0; 3])), |x| {
            if x < -0.999 * t { 0 } else if x <= 1e-9 { 1 } else { 2 }
        }),
        5 => sweep(40, -2.0 * t, dx, |x| (circle(0.5, [-x, 0.5, 0.0]), circle(1.0, [0.0; 3])), |x| if x.abs() < 1.1 * t / 10.0 { 1 } else { 2 }),
        6 => sweep(40, -2.0 * t, dx, |x| (circle(0.5, [-1.5 + x, 0.0, 0.0]), circle(1.0, [0.0; 3])), |x| if x < -t { 0 } else { 2 }),
        7 => sweep(40, -2.0 * t, dx, |x| (line([x, 0.0, 0.05 + x], [-2.0 + x, 0.0, -0.05 + x]), circle(1.0, [0.0; 3])), |x| {
            if x < -1.0999 * t || x > 1.09999 * t { 0 } else { 1 }
        }),
        8 => sweep(40, -2.0 * t, dx, |x| (line([x, 0.0, 0.1 + x], [-2.0 + x, 0.0, -0.1 + x]), circle(1.0, [0.0; 3])), |x| {
            if x < -1.1999 * t || x > 1.19999 * t { 0 } else { 1 }
        }),
        9 => sweep(40, -2.0 * t, dx, |x| (line([x, 0.0, 1.0], [-2.0 + x, 0.0, -1.0]), circle(1.0, [0.0; 3])), |x| {
            if x < -1.4001 * t || x > 1.4001 * t { 0 } else { 1 }
        }),
        10 => sweep(30, -1.5 * t, dx, |x| (line([-1.0 + x, 0.0, 0.0], [-2.0, 0.0, -2.0]), circle(1.0, [0.0; 3])), |x| {
            if x < -1.001 * t || x > 1.1999 * t { 0 } else { 1 }
        }),
        11 => sweep(30, -1.5 * t, dx, |x| (line([-1.0 + x, 0.0, 0.0], [-2.0, 0.0, 0.0]), circle(1.0, [0.0; 3])), |x| if x < -1.001 * t { 0 } else { 1 }),
        12 => sweep(30, -1.5 * t, dx, |x| (line([-1.0 + x, 0.0, -1.0], [-1.0 + x, 0.0, 1.0]), circle(1.0, [0.0; 3])), |x| {
            if x < -(t + E) || x > t + E { 0 } else { 1 }
        }),
        13 => sweep(20, -t, dx, |x| (line([-1.0 + x, -1.0, 0.0], [-1.0 + x, 1.0, 0.0]), circle(1.0, [0.0; 3])), |x| if x <= E { 1 } else { 2 }),
        14 => {
            // The line from the circle's centre (SMLib: 2 or 3 near its seam, else 1), and the
            // circle at the line's end (1, near a knot).
            sweep(20, 0.0, dx, |y| (line([0.0, y, 0.0], [2.5, y, 0.0]), circle(1.0, [0.0; 3])), |_| 1)?;
            sweep(20, 0.0, dx, |y| (circle(1.0, [2.5, 0.0, 0.0]), line([0.0, y, 0.0], [2.5, y, 0.0])), |_| 1)
        }
        _ => Err(format!("case {case} isn't ported")),
    }
}

/// The two curves of `my_test_cci_tangent`'s and `my_test_cci_nurb`'s B-splines.
const NURB_A: [[f64; 3]; 7] = [[0.0, 0.1, 0.0], [0.2, 0.1, 0.0], [0.4, 0.2, 0.0], [0.6, -0.3, 0.0], [1.0, 0.0, 0.0], [1.1, 0.1, 0.0], [1.0, 0.5, 0.0]];
const NURB_B: [[f64; 3]; 7] = [[0.0, 0.1, 0.0], [0.2, 0.1, 0.0], [0.4, 0.3, 0.0], [0.6, -0.4, 0.0], [1.0, 0.0, 0.0], [1.1, 0.1, 0.0], [1.0, 0.5, 0.0]];
const COIN_A: [[f64; 3]; 7] = [[0.0, 0.1, 0.0], [0.2, 0.1, 0.0], [0.4, 0.2, 0.0], [0.6, -0.3, 0.0], [1.0, 0.0, 0.0], [1.1, 0.2, 0.0], [1.0, 0.5, 0.0]];
const COIN_B: [[f64; 3]; 7] = [[0.0, 0.1, 0.0], [0.2, 0.1, 0.0], [0.4, 0.2, 0.0], [0.6, -0.3, 0.0], [1.0, 0.2, 0.0], [1.1, -0.1, 0.0], [1.0, 0.3, 0.0]];
const ZIG17_A: [[f64; 3]; 17] = [
    [-1.0, 0.5, 0.0], [0.0, 0.5, 0.0], [1.0, 1.0, 0.0], [0.0, 1.5, 0.0], [-1.0, 2.0, 0.0], [1.0, 4.5, 0.0], [0.0, 3.5, 0.0], [-1.0, 4.0, 0.0], [1.0, 5.0, 0.0],
    [-1.0, 6.0, 0.0], [0.0, 6.5, 0.0], [1.0, 7.0, 0.0], [0.0, 7.5, 0.0], [-1.0, 8.0, 0.0], [0.0, 8.5, 0.0], [1.0, 9.0, 0.0], [-1.0, 9.5, 0.0],
];
const ZIG17_B: [[f64; 3]; 17] = [
    [1.0, 0.5, 0.0], [0.0, 0.5, 0.0], [-1.0, 1.0, 0.0], [0.0, 1.5, 0.0], [1.0, 2.0, 0.0], [-1.0, 4.5, 0.0], [0.0, 3.5, 0.0], [1.0, 4.0, 0.0], [-1.0, 5.0, 0.0],
    [1.0, 6.0, 0.0], [0.0, 6.5, 0.0], [-1.0, 7.0, 0.0], [0.0, 7.5, 0.0], [1.0, 8.0, 0.0], [0.0, 8.5, 0.0], [-1.0, 9.0, 0.0], [1.0, 9.5, 0.0],
];

/// The 21-point zigzags of `my_test_cci_nurb`'s second block.
const ZIG21_A: [[f64; 3]; 21] = [
    [-1.0, 0.5, 0.0], [0.0, 0.5, 0.0], [1.0, 1.0, 0.0], [0.0, 1.5, 0.0], [-1.0, 2.0, 0.0], [0.0, 2.5, 0.0], [1.0, 3.0, 0.0], [0.0, 3.5, 0.0], [-1.0, 4.0, 0.0], [0.0, 4.5, 0.0],
    [1.0, 5.0, 0.0], [0.0, 5.5, 0.0], [-1.0, 6.0, 0.0], [0.0, 6.5, 0.0], [1.0, 7.0, 0.0], [0.0, 7.5, 0.0], [-1.0, 8.0, 0.0], [0.0, 8.5, 0.0], [1.0, 9.0, 0.0], [0.0, 9.5, 0.0],
    [-1.0, 9.5, 0.0],
];
const ZIG21_B: [[f64; 3]; 21] = [
    [1.0, 0.5, 0.0], [0.0, 0.5, 0.0], [-1.0, 1.0, 0.0], [0.0, 1.5, 0.0], [1.0, 2.0, 0.0], [0.0, 2.5, 0.0], [-1.0, 3.0, 0.0], [0.0, 3.5, 0.0], [1.0, 4.0, 0.0], [0.0, 4.5, 0.0],
    [-1.0, 5.0, 0.0], [0.0, 5.5, 0.0], [1.0, 6.0, 0.0], [0.0, 6.5, 0.0], [-1.0, 7.0, 0.0], [0.0, 7.5, 0.0], [1.0, 8.0, 0.0], [0.0, 8.5, 0.0], [-1.0, 9.0, 0.0], [0.0, 9.5, 0.0],
    [1.0, 9.5, 0.0],
];

/// The named case of `my_test_cci_tangent`, `my_test_cci_coincidence` or `my_test_cci_nurb`.
fn unit(case: &str) -> Result<(), String> {
    let o = [0.0; 3];
    match case {
        "tangent_line_circle" => expect(&line([1.0, 2.0, 0.0], [3.0, 2.0, 0.0]), &circle(0.5, [1.5, 2.5, 0.0]), true, None),
        "tangent_circle_line" => expect(&circle(0.5, [2.5, 2.5, 0.0]), &line([1.0, 2.0, 0.0], [3.0, 2.0, 0.0]), true, None),
        "tangent_circle_in_ellipse" => expect(&circle(0.5, [-2.0, 2.0, 0.0]), &ellipse(1.0, 0.5, [-2.0, 2.0, 0.0]), true, None),
        "tangent_circle_out_ellipse" => expect(&circle(1.0, [-2.0, 2.0, 0.0]), &ellipse(1.0, 0.5, [-2.0, 2.0, 0.0]), true, None),
        "tangent_nurbs" => expect(&nurb([-0.5, 2.0, 0.0], &NURB_A, 3), &nurb([-0.5, 2.0, 0.0], &NURB_B, 3), true, None),
        "coincident_nurbs" => expect(&nurb(o, &COIN_A, 3), &nurb(o, &COIN_B, 3), true, None),
        "coincident_lines" => expect(&line([-2.0, 0.0, 0.0], [-1.0, 0.0, 0.0]), &line([-1.5, 0.0, 0.0], [-0.5, 0.0, 0.0]), true, None),
        "coincident_circles_radii" => expect(&circle(0.500005, [1.5, -1.5, 0.0]), &circle(0.499999, [1.5, -1.5, 0.0]), true, None),
        "coincident_circles" => expect(&circle(0.5, [2.0, 0.0, 0.0]), &circle(0.5, [2.0, 0.0, 0.0]), true, None),
        "coincident_circles_turned" => expect(&circle(0.5, o), &placed(circle(0.5, o), o, [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]), true, None),
        "coincident_circles_opposite" => expect(&circle(0.5, o), &placed(circle(0.5, o), o, [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]), true, None),
        "coincident_circles_opposite_turned" => expect(&circle(0.5, o), &placed(circle(0.5, o), o, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]), true, None),
        "nurb_zigzag_16" => expect(&nurb([10.0, -5.0, 0.0], &ZIG17_A, 3), &nurb([10.0, -5.0, 0.0], &ZIG17_B, 3), true, Some(16)),
        "nurb_zigzag_shifted_8" => expect(&nurb([6.0, -5.0, 0.0], &ZIG17_A, 3), &nurb([7.0, -5.0, 0.0], &ZIG17_B, 3), true, Some(8)),
        "nurb_zigzag_coincident" => expect(&nurb([13.0, -4.0, 0.0], &ZIG17_A, 3), &nurb([13.0, -5.0, 0.0], &ZIG17_B, 3), true, None),
        "nurb_zigzag21_10" => expect(&nurb([0.0, -5.0, 0.0], &ZIG21_A, 3), &nurb([0.0, -5.0, 0.0], &ZIG21_B, 3), true, Some(10)),
        "nurb_zigzag21_shifted_10" => expect(&nurb([-4.0, -5.0, 0.0], &ZIG21_A, 3), &nurb([-3.0, -5.0, 0.0], &ZIG21_B, 3), true, Some(10)),
        "nurb_zigzag21_coincident" => expect(&nurb([3.0, -4.0, 0.0], &ZIG21_A, 3), &nurb([3.0, -5.0, 0.0], &ZIG21_B, 3), true, None),
        _ => Err(format!("{case} isn't ported")),
    }
}

/// `my_test_cci_adv`: the first two curves of `file` over their natural intervals; SMLib's point
/// solutions `points` lie on ours.
fn cci_part(file: &str, points: &[[f64; 3]]) -> Result<(), String> {
    let bytes = std::fs::read(data(SUITE, file)).map_err(|e| format!("{file}: {e}"))?;
    let part = smb::read_part(&bytes)?;
    let [a, b, ..] = part.curves.as_slice() else { return Err(format!("{file}: {} curves", part.curves.len())) };
    let range = |c: &Curve| c.nurbs_range().ok_or("not a B-spline");
    let ((a0, a1), (b0, b1)) = (range(a)?, range(b)?);
    let pts = cci(&C { c: a.clone(), t0: a0, t1: a1 }, &C { c: b.clone(), t0: b0, t1: b1 })?;
    if pts.is_empty() {
        return Err("no points, SMLib finds some".into());
    }
    for q in points {
        let q = p3(*q);
        let d = pts.iter().map(|p| (p - q).norm()).fold(f64::INFINITY, f64::min);
        if d > 1e-4 {
            return Err(format!("SMLib's point {q:?} is {d:.1e} from ours"));
        }
    }
    Ok(())
}

/// A bilinear B-spline plane (`my_create_plane`: corners o, u, v and u + v − o, moved by `tr`).
fn bilinear(tr: [f64; 3], o: [f64; 3], u: [f64; 3], v: [f64; 3]) -> Patch {
    let t = p3(tr).coords;
    let (o, u, v) = (p3(o) + t, p3(u) + t, p3(v) + t);
    let w = u + (v - o);
    let cps = vec![vec![hom(&o, 1.0), hom(&v, 1.0)], vec![hom(&u, 1.0), hom(&w, 1.0)]];
    let n = NurbsSurface::new(1, 1, vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0], cps).expect("a valid B-spline");
    Patch::rect(Surface::Nurbs(Arc::new(n)), ((0.0, 1.0), (0.0, 1.0)))
}

/// `SmBSplineSurface::CreateConePatch` on the frame (o, x, y): radius r0 at its base to r1 at
/// height h, angles a0 to a1 degrees.
fn cone_patch(o: [f64; 3], x: [f64; 3], y: [f64; 3], r0: f64, r1: f64, a0: f64, a1: f64, h: f64) -> Patch {
    let (x, y) = (V3::new(x[0], x[1], x[2]), V3::new(y[0], y[1], y[2]));
    let f = Frame { o: p3(o), x, y, z: x.cross(&y) };
    let us = (a0.to_radians(), a1.to_radians());
    if r0 == r1 {
        Patch::rect(Surface::Cylinder { f, r: r0 }, (us, (0.0, h)))
    } else {
        let l = h.hypot(r1 - r0);
        Patch::rect(Surface::Cone { f, r: r0, a: ((r1 - r0) / l).asin() }, (us, (0.0, l)))
    }
}

/// `my_test_surface_curve_intersect`: lines along z through an nx × ny grid over the patch's
/// box, each intersected with the surface (`ssi::curve_surface`) and kept in the patch; every
/// point checked on the surface, none against some as SMLib finds.
fn csi(p: &Patch, nx: usize, ny: usize, any: bool) -> Result<(), String> {
    const E: f64 = 1e-12;
    let bb = p.bbox();
    let mut total = 0;
    let mut i = 0;
    while i < nx {
        let mut dxs = (bb.max.x - bb.min.x) / (nx as f64 - 1.0 + E);
        if dxs < E {
            i = nx;
        }
        if nx < 2 {
            dxs = 0.0;
        }
        for j in 0..ny {
            let dys = (bb.max.y - bb.min.y) / (ny as f64 - 1.0 + E);
            let (x, y) = (bb.min.x + i as f64 * dxs, bb.min.y + j as f64 * dys);
            let (mut z0, mut z1) = (bb.min.z, bb.max.z);
            if z1 - z0 < E {
                z0 -= 1.0;
                z1 += 1.0;
            }
            let (l, len) = Curve::line(P3::new(x, y, z0), P3::new(x, y, z1));
            let mut pts = Vec::new();
            for t in ssi::curve_surface(&l, 0.0, len, &p.surf) {
                let q = l.point(t);
                if !p.contains(&q) {
                    continue;
                }
                let d = dist(&p.surf, &q);
                if d > TOL {
                    return Err(format!("line ({x}, {y}): {q:?} is {d:.1e} off the surface"));
                }
                pts.push(q);
            }
            total += intersect::distinct(&pts, 1e-6).len();
        }
        i += 1;
    }
    if (total > 0) != any {
        return Err(format!("{total} points, SMLib finds {}", if any { "some" } else { "none" }));
    }
    Ok(())
}
'''

# (name, call, doc)
UNIT = [
    ("tangent_line_circle", "a line tangent to a circle (1)"),
    ("tangent_circle_line", "a circle tangent to a line (1)"),
    ("tangent_circle_in_ellipse", "a circle inside an ellipse, tangent at both ends of its minor axis (2)"),
    ("tangent_circle_out_ellipse", "a circle round an ellipse, tangent at both ends of its major axis (5, seam duplicates)"),
    ("tangent_nurbs", "two cubic B-splines tangent at their ends, crossing twice between (4)"),
    ("coincident_nurbs", "two cubic B-splines coincident over their first part, then crossing (3)"),
    ("coincident_lines", "two overlapping lines (1 range)"),
    ("coincident_circles_radii", "circles of radius 0.500005 and 0.499999 (1 range)"),
    ("coincident_circles", "the same circle twice (1 range)"),
    ("coincident_circles_turned", "a circle and a copy starting a quarter turn on (2)"),
    ("coincident_circles_opposite", "a circle and a copy running the other way (1)"),
    ("coincident_circles_opposite_turned", "a circle and a copy running the other way from a quarter turn on (2)"),
    ("nurb_zigzag_16", "two 17-point cubic zigzags crossing at 16 points"),
    ("nurb_zigzag_shifted_8", "the zigzags shifted 1 apart in x: 8 crossings"),
    ("nurb_zigzag_coincident", "the zigzags shifted 1 apart in y: partly coincident (8; SMLib's \"broken case\")"),
    ("nurb_zigzag21_10", "two 21-point cubic zigzags crossing at 10 points"),
    ("nurb_zigzag21_shifted_10", "the 21-point zigzags shifted 1 apart in x: 10 crossings"),
    ("nurb_zigzag21_coincident", "the 21-point zigzags shifted 1 apart in y: a coincident range and a near miss (2)"),
]

ANALY = {
    2: "a circle of radius 0.5 tilted 30 rad about x, its left end at gap x from the unit circle's",
    3: "a circle of radius 0.5 inside the unit circle, touching it at (1, 0) at gap x",
    4: "a circle of radius 0.5 inside the unit circle, touching it at (0, 1) at gap x",
    5: "a circle of radius 0.5 crossing the unit circle near (0, 1), shifted x along the tangent",
    6: "a circle of radius 0.5 outside the unit circle, touching it at (−1, 0) at gap x",
    7: "a line crossing the unit circle's plane at a shallow angle near (−1, 0) at gap x",
    8: "a line crossing the unit circle's plane at a steeper angle near (−1, 0) at gap x",
    9: "a line crossing the unit circle's plane at 45° near (−1, 0) at gap x",
    10: "a line out of the plane ending near (−1, 0) at gap x",
    11: "a line in the plane ending near (−1, 0) at gap x",
    12: "a line square to the circle's plane passing (−1, 0) at gap x",
    13: "a line tangent to the unit circle at (−1, 0), moved x across",
    14: "a line from the unit circle's centre at height y near its seam, and a circle at the line's end",
}

CSI = [
    ("plane", "bilinear([0.0; 3], [0.0; 3], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0])", 5, 25, "a 2 × 2 plane in z = 0"),
    ("plane_tilted", "bilinear([0.0, -5.0, 0.0], [0.0; 3], [2.0, 0.0, 2.0], [0.0, 2.0, 0.0])", 5, 25, "a plane tilted 45° about y"),
    ("plane_skew", "bilinear([9.0, 0.0, 0.0], [0.0; 3], [2.0, 0.0, 2.0], [0.0, 2.0, 2.0])", 5, 25, "a skew parallelogram"),
    ("plane_vertical", "bilinear([14.0, 0.0, 0.0], [0.0; 3], [0.0, 0.0, 2.0], [0.0, 2.0, 0.0])", 5, 5, "a plane x = 14: the lines lie in it"),
    ("cylinder", "cone_patch([-5.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 1.0, 0.0, 360.0, 1.0)", 5, 45, "a unit cylinder about −y"),
    ("half_cylinder", "cone_patch([-5.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 1.0, 0.0, 180.0, 1.0)", 5, 25, "half a unit cylinder about −y"),
    ("cone_up", "cone_patch([5.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 0.0, 0.0, 360.0, 1.0)", 10, 60, "a cone about z, apex up"),
    ("cone_down_270", "cone_patch([5.0, -5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.0, 1.0, 0.0, 270.0, 1.0)", 10, 45, "three quarters of a cone about z, apex down"),
    ("cylinder_quintic", "cone_patch([0.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 1.0, 0.0, 360.0, 1.0)", 5, 25, "a unit cylinder about −y (SMLib's quintic form)"),
    ("cone_side", "cone_patch([5.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.0, 1.0, 0.0, 360.0, 1.0)", 5, 13, "a cone about −y, apex at the base"),
    ("cone_side_down", "cone_patch([-5.0, -5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, 0.0, 0.0, 360.0, 1.0)", 5, 13, "a cone about −y, apex at the top"),
]


def extract(ctx):
    cur = ctx.source("smCurveTest.cpp")
    for f in ("my_test_cci_analy", "my_test_cci_tangent", "my_test_cci_coincidence", "my_test_cci_nurb"):
        assert "PT_EXPORT SmStatus %s" % f in cur, f
    assert "my_test_csi_analy" in ctx.source("smSurfaceTest.cpp")
    cases = []
    rhino = Fixture("TestFiles/pt_TestFiles/cci_bug_rhino1.smp")
    r = oracle_run(["cci", ctx.path(rhino.rel)])
    pts = [p["point"] for p in r["points"] if not p.get("range")] if r and r.get("ok") else []
    cases.append(Case("cci_adv_rhino_bug1",
                      "SMLib advanced CCI: the two curves of `cci_bug_rhino1.smp` (SMLib: 2, a coincident range and a point%s)." % (
                          "" if not pts else ", the point at %s" % ", ".join("(%g, %g, %g)" % tuple(p) for p in pts)),
                      'cci_part("cci_bug_rhino1.smp", %s)' % points_literal(pts), [rhino]))
    for n, what in sorted(ANALY.items()):
        cases.append(Case("cci_analy_%02d" % n, "SMLib `my_test_cci_analy` %d: %s, swept through the tolerance." % (n, what), "analy(%d)" % n))
    for name, what in UNIT:
        cases.append(Case("cci_" + name, "SMLib CCI unit test: %s." % what, 'unit("%s")' % name))
    for name, make, grid, count, what in CSI:
        cases.append(Case("csi_" + name, "SMLib `my_test_csi_analy`: %s against a %d × %d grid of lines along z (SMLib: %d points)." % (what, grid, grid, count),
                          "csi(&%s, %d, %d, true)" % (make, grid, grid)))
    return cases
