"""SMLib's C API tests (NVIDIA usd-brep tests/SM_API_test/src, `run_sm_api` of SM_API_test.cpp)
that assert geometric outcomes, on the kernel's own API. Every model is built in code (the C
API's boxes, spheres, tori, cylinders, circles and lines become the DRAW harness's primitives,
edges and wires), so the suite has no data.

Most of SM_API_test checks the C API's contract (argument validation, ownership, error codes,
copy semantics): not applicable here (research/SMLIB_COVERAGE.md lists every function). What
converts, each case citing its source line, with SMLib's expected values (tables parsed from the
source where it has them: `ShellCase`, `TestMergeBrepsOperation`, `aExpectedAreas`; the other
literals are checked to be in the source) or, where SMLib asserts only success, the closed form:
- booleans: `TestSmMergeBreps` (union, intersection, difference of two boxes, the left-to-right
  grouping of three, 1144/48/144/2532/944), `TestSmBooleanBoundsExample` (a plate less 16
  cylinders, its volume and tight box), `TestSmBooleanWithOptions`, `TestSmNonManifoldBoolean`,
  `TestSmPiecewiseMerge`/`TestSmBooleanLists` (a box with an inscribed sphere, boxes overlapping);
- shells: `TestSmShellBrepFull` (584, 584, 728, 488);
- mass properties: `TestSmMassProperties` (a rotated box: volume, area, centroid, inertia about
  the centroid and the origin; SMLib's products of inertia are +∫xy dm, the kernel's −∫xy dV),
  `TestSmPrecisePropertySelection` (areas 52 far from the origin, 100π, 20π², 21π);
- queries: `TestSmBrepDistance`, `TestSmGetClosestPoint`, `TestSmFindEdge`, `TestSmFindFaces`,
  `TestSmGetFaces`/`TestSmGetEdges`;
- intersections: `TestSmIntersectCurveSurface` (a circle and a tilted plane: (2, 5, 0) and
  (8, 5, 0)), `TestSmIntersectCurveFace`, `TestSmIntersectCurveBrep` (lines through a box and a
  holed box: the points, none along the hole's axis, two where a diagonal grazes two edges);
- sweeps and primitives: `TestSmSweepAlongPlanarPath` (a circle along one and two segments),
  `TestSmCurveSweep`, `TestSmCreateLinearSweep*`, `TestSmCreateRotationalSweep`;
- fillets, offsets, healing, exchange: `TestSmCircularFillet` (every edge of a box: the rounded
  box's closed form), `TestSmCreateOffsetSurface` (a sphere and a torus offset past their
  radius fail), `TestSmHealBrep`, `TestSmImportExport` (written and read back),
  `TestSmTessellateBoundaries`.
"""
import re

from smlib_to_rust import Case, rust_f64, strip_comments

SOURCE = "TestSmBrep.cpp and the other TestSm*.cpp"
SOURCE_DIR = "tests/SM_API_test/src"
TITLE = "C API (SM_API_test)"

NOT_CONVERTED = [
    ("API contract checks (NULL, ownership, sentinels, invalid input, error codes, copy semantics) across all TestSm* functions", "the C API's own contract, not a geometric outcome"),
    ("TestSmSweepAlongPlanarPath with one end cap (capEnds 1, 2)", "the kernel's sweep caps both ends or neither"),
    ("TestSmSweepAlongPlanarPath: the 2 × 1 rectangle along 20 mm", "the rectangle's plane relative to the path isn't given by the test (it asserts at least one face)"),
    ("TestSmIntersectCurveFace/CurveBrep/CurveSurface: curves lying on a face (a circle on the top face, a line along an edge)", "the kernel's curve intersections return points, not overlap ranges"),
    ("TestSmPrecisePropertySelection: area moments of a face with thickness", "not a kernel operation"),
    ("ProjectAndTrim, DropCurveToSrf, LiftUVCurve, curve offsets, knot removal, ruled/linear skins, swung primitives, surface fillets, bevel corners, non-uniform scale, curvature, third derivatives", "feature gaps (research/SMLIB_COVERAGE.md)"),
]

PRELUDE = r'''
use std::f64::consts::PI;

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::draw::{Ax3, Shape};
use cadrs_kernel::native::math::{Frame, P3, V3};
use cadrs_kernel::native::ssi;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BodyId, BodyKind, BoolOp, Curve2, DistanceTarget, HealOp, Kernel, Loop, OffsetSpec, OpResult, PathCurve, Plane, Profile, Region, ShellSpec, SweepControl, SweepSpec, Tessellation};
use nalgebra::{Point2, Vector3};

fn e<E: std::fmt::Debug>(x: E) -> String {
    format!("{x:?}")
}

fn close(what: &str, got: f64, want: f64, tol: f64) -> Result<(), String> {
    if (got - want).abs() > tol {
        return Err(format!("{what} {got:.9}, SMLib's {want:.9} (within {tol:.1e})"));
    }
    Ok(())
}

fn solid(s: Shape) -> Result<BodyId, String> {
    match s {
        Shape::Body(b) => Ok(b),
        _ => Err("not a body".into()),
    }
}

fn one(r: OpResult) -> Result<BodyId, String> {
    match r.bodies.as_slice() {
        [b] => Ok(*b),
        v => Err(format!("{} bodies", v.len())),
    }
}

fn ax(o: [f64; 3]) -> Ax3 {
    Ax3 { origin: P3::new(o[0], o[1], o[2]), z: V3::z(), x: V3::x() }
}

/// `SmApiCreateBox(origin, l, w, h)`: the min corner at `origin`.
fn bx(k: &mut NativeKernel, o: [f64; 3], l: f64, w: f64, h: f64) -> Result<BodyId, String> {
    solid(k.box_on(&ax(o), l, w, h).map_err(e)?)
}

/// A valid manifold solid's volume.
fn volume(k: &NativeKernel, b: BodyId) -> Result<f64, String> {
    if let Some(bad) = invalid(k.get(b).map_err(e)?) {
        return Err(bad);
    }
    if k.solid_count(b).map_err(e)? != 1 {
        return Err(format!("{} solids", k.solid_count(b).map_err(e)?));
    }
    Ok(k.mass_properties(b).map_err(e)?.volume)
}

/// `TestMergeBrepsOperation` and the three-box merges (TestSmBrep.cpp:766, 1044): SMLib's
/// `merge_breps` groups the list last to first (`[A, B]` difference is B − A, `[C, B, A]` is
/// (A − B) − C).
fn merge(boxes: &[[f64; 6]], op: BoolOp, want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let ids = boxes.iter().map(|b| bx(&mut k, [b[0], b[1], b[2]], b[3], b[4], b[5])).collect::<Result<Vec<_>, _>>()?;
    let mut acc = *ids.last().ok_or("no boxes")?;
    for &b in ids.iter().rev().skip(1) {
        acc = one(k.boolean(op, acc, &[b]).map_err(e)?)?;
    }
    close("volume", volume(&k, acc)?, want, 1e-9 * want)
}

/// `TestSmShellBrepFull` (TestSmBrep.cpp:2544): the 10 mm box shelled by `d` (outward when
/// positive) with sharp corners, opened at its top face `opened` times (0: a closed hollow).
fn shell_box(d: f64, opened: usize, want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
    let top = k.faces(b).map_err(e)?.into_iter().find(|f| (f.center.z - 10.0).abs() < 1e-9).ok_or("no top face")?.id;
    let spec = ShellSpec { remove: vec![top; opened], thickness: d.abs(), outward: d > 0.0, hollow: opened == 0, sharp: true };
    let r = one(k.shell_with(b, &spec).map_err(e)?)?;
    if let Some(bad) = invalid(k.get(r).map_err(e)?) {
        return Err(bad);
    }
    close("volume", k.mass_properties(r).map_err(e)?.volume, want, 1e-5)
}

/// `TestSmMassProperties` (TestSmBrep.cpp:483): a 2 × 6 × 10 box turned 30° about the vertical
/// through its centre (1, 3, 5), density 2. `about_origin`: the moments about the world origin.
/// SMLib's products of inertia are +∫xy dm: the kernel's tensor holds −∫xy dV.
fn mass_box(about_origin: bool) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (c, s) = (PI / 6.0).sin_cos();
    let (c, s) = (s, c);
    let centre = V3::new(1.0, 3.0, 5.0);
    let x = V3::new(c, s, 0.0);
    let y = V3::new(-s, c, 0.0);
    let o = centre - (x * 1.0 + y * 3.0 + V3::z() * 5.0);
    let b = solid(k.box_on(&Ax3 { origin: P3::from(o), z: V3::z(), x }, 2.0, 6.0, 10.0).map_err(e)?)?;
    let m = k.mass_properties(b).map_err(e)?;
    let rho = 2.0;
    let mass = rho * 120.0;
    let (axx, byy, czz) = (mass * 4.0 / 12.0, mass * 36.0 / 12.0, mass * 100.0 / 12.0);
    let (ixx, iyy, izz) = (axx * 0.25 + byy * 0.75 + czz, axx * 0.75 + byy * 0.25 + czz, axx + byy);
    // SMLib's +∫xy dm (negative here).
    let ixy = 0.4330127018922193 * (axx - byy);
    close("volume", m.volume, 120.0, 1e-9)?;
    close("area", m.surface_area, 184.0, 1e-9)?;
    close("mass", rho * m.volume, mass, 1e-9)?;
    for (i, w) in [1.0, 3.0, 5.0].iter().enumerate() {
        close("centroid", m.center_of_mass[i], *w, 1e-9)?;
    }
    let (t, want) = if about_origin {
        let (cx, cy, cz) = (1.0, 3.0, 5.0);
        let i = m.inertia_about(P3::origin()) * rho;
        (i, [ixx + mass * (cy * cy + cz * cz), iyy + mass * (cx * cx + cz * cz), izz + mass * (cx * cx + cy * cy), mass * cy * cz, mass * cz * cx, ixy + mass * cx * cy])
    } else {
        (m.inertia * rho, [ixx, iyy, izz, 0.0, 0.0, ixy])
    };
    for (name, got, w) in [("Ixx", t[(0, 0)], want[0]), ("Iyy", t[(1, 1)], want[1]), ("Izz", t[(2, 2)], want[2]), ("Iyz", -t[(1, 2)], want[3]), ("Izx", -t[(2, 0)], want[4]), ("Ixy", -t[(0, 1)], want[5])] {
        close(name, got, w, 1e-9 * w.abs().max(1.0))?;
    }
    Ok(())
}

/// `TestSmPrecisePropertySelection` (TestSmBrep.cpp:373): the area of a box far out, a sphere,
/// a torus and a disc with a hole, the body's equal to its faces' sum.
fn areas(case: usize, want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let area = match case {
        0 | 1 | 2 => {
            let b = match case {
                0 => bx(&mut k, [1.0e5, -2.0e5, 3.0e5], 2.0, 3.0, 4.0)?,
                1 => solid(k.psphere_on(&ax([0.0; 3]), 5.0).map_err(e)?)?,
                _ => solid(k.ptorus_on(&ax([0.0; 3]), 5.0, 1.0).map_err(e)?)?,
            };
            let a = k.mass_properties(b).map_err(e)?.surface_area;
            let sum: f64 = k.faces(b).map_err(e)?.iter().map(|f| f.area).sum();
            close("the faces' areas' sum", sum, a, 1e-9 * a)?;
            a
        }
        _ => {
            let outer = k.wire(&[&k.mkedge_full(&k.circle(0.0, 0.0, 0.0, 5.0))]).map_err(e)?;
            let hole = k.wire(&[&k.mkedge_full(&k.circle(0.0, 0.0, 0.0, 2.0))]).map_err(e)?;
            let f = k.mkplane(&outer).map_err(e)?;
            let f = k.add_hole(&hole, &f).map_err(e)?;
            k.sprops(&f).map_err(e)?
        }
    };
    close("area", area, want, 1e-9 * want)
}

/// `TestSmBrepDistance` (TestSmBrepDistance.cpp:25): boxes (min corner, size) `a` and `b`.
fn distance(a: [f64; 4], b: [f64; 4], want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (ia, ib) = (bx(&mut k, [a[0], a[1], a[2]], a[3], a[3], a[3])?, bx(&mut k, [b[0], b[1], b[2]], b[3], b[3], b[3])?);
    let d = k.min_distance(&DistanceTarget::Body(ia), &DistanceTarget::Body(ib)).map_err(e)?.distance;
    let r = k.min_distance(&DistanceTarget::Body(ib), &DistanceTarget::Body(ia)).map_err(e)?.distance;
    close("distance", d, want, 1e-9)?;
    close("the distance the other way", r, d, 1e-9)
}

/// `TestSmGetClosestPoint`, `TestSmFindEdge`, `TestSmFindFaces`, `TestSmGetFaces`/`GetEdges`
/// (TestSmGeneral.cpp:707–830) on the 10 mm box.
fn queries_on_box() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
    let d = k.min_distance(&DistanceTarget::Point(P3::new(5.0, 5.0, 15.0)), &DistanceTarget::Body(b)).map_err(e)?;
    close("closest point distance", d.distance, 5.0, 1e-9)?;
    close("closest point", (d.b - P3::new(5.0, 5.0, 10.0)).norm(), 0.0, 1e-9)?;
    let (faces, edges) = (k.faces(b).map_err(e)?, k.edges(b).map_err(e)?);
    if faces.len() != 6 || edges.len() != 12 {
        return Err(format!("{} faces, {} edges", faces.len(), edges.len()));
    }
    for p in [[5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [10.0, 10.0, 5.0], [0.0, 0.0, 5.0], [5.0, 10.0, 10.0]] {
        let p = P3::new(p[0], p[1], p[2]);
        let on = edges.iter().filter_map(|ed| k.min_distance(&DistanceTarget::Point(p), &DistanceTarget::Edge(b, ed.id)).ok()).any(|r| r.distance <= 1e-6);
        if !on {
            return Err(format!("no edge through {p:?}"));
        }
    }
    let p = P3::new(5.0, 5.0, 10.0);
    if !faces.iter().filter_map(|f| k.min_distance(&DistanceTarget::Point(p), &DistanceTarget::Face(b, f.id)).ok()).any(|r| r.distance <= 1e-9) {
        return Err("no face through (5, 5, 10)".into());
    }
    Ok(())
}

/// `TestSmIntersectCurveSurface` (TestSmIntersectors.cpp:301): the circle of radius 3 about
/// (5, 5, 0) in z = 0 against the plane through (0, 0, −5), (10, 0, −5), (0, 10, 5): z = y − 5.
fn circle_plane() -> Result<(), String> {
    let c = Curve::Circle { c: P3::new(5.0, 5.0, 0.0), x: V3::x(), y: V3::y(), r: 3.0 };
    let y = V3::new(0.0, 1.0, 1.0).normalize();
    let f = Frame { o: P3::new(0.0, 0.0, -5.0), x: V3::x(), y, z: V3::x().cross(&y) };
    // (Exactly two: a crossing at the circle's seam, (8, 5, 0), counts once.)
    let mut pts: Vec<P3> = ssi::curve_surface(&c, 0.0, 2.0 * PI, &Surface::Plane { f }).into_iter().map(|t| c.point(t)).collect();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x));
    let want = [P3::new(2.0, 5.0, 0.0), P3::new(8.0, 5.0, 0.0)];
    if pts.len() != 2 || pts.iter().zip(&want).any(|(p, w)| (p - w).norm() > 1e-9) {
        return Err(format!("points {pts:?}, SMLib's surface points are {want:?}"));
    }
    Ok(())
}

/// The 10 mm box, holed by a cylinder of radius 2 about the vertical through (5, 5)
/// (`sm_CreateHoledBox`, TestSmIntersectors.cpp:401).
fn holed_box(k: &mut NativeKernel) -> Result<BodyId, String> {
    let b = bx(k, [0.0; 3], 10.0, 10.0, 10.0)?;
    let c = solid(k.pcylinder_on(&ax([5.0, 5.0, -1.0]), 2.0, 12.0).map_err(e)?)?;
    one(k.boolean(BoolOp::Subtract, b, &[c]).map_err(e)?)
}

/// The distinct points where the segment from `a` to `b` crosses the body (`ray_hits`, kept on
/// the segment; a hit where faces meet is one point).
fn segment_hits(k: &NativeKernel, body: BodyId, a: [f64; 3], b: [f64; 3]) -> Result<Vec<P3>, String> {
    let (a, b) = (P3::new(a[0], a[1], a[2]), P3::new(b[0], b[1], b[2]));
    let len = (b - a).norm();
    let mut out: Vec<P3> = Vec::new();
    for h in k.ray_hits(body, a, (b - a) / len).map_err(e)? {
        if h.t >= -1e-9 && h.t <= len + 1e-9 && !out.iter().any(|q| (q - h.point).norm() <= 1e-7) {
            out.push(h.point);
        }
    }
    Ok(out)
}

/// `TestSmIntersectCurveFace` and `TestSmIntersectCurveBrep` (TestSmIntersectors.cpp:429, 596).
fn curve_brep(case: &str) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (body, a, b, want): (BodyId, [f64; 3], [f64; 3], Vec<[f64; 3]>) = match case {
        // A line through the box: its faces x = 0 and x = 10.
        "through_box" => (bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?, [-5.0, 5.0, 5.0], [15.0, 5.0, 5.0], vec![[0.0, 5.0, 5.0], [10.0, 5.0, 5.0]]),
        // A diagonal grazing the two vertical edges x = y = 0 and x = y = 10: 2 points.
        "diagonal_through_edges" => (bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?, [-5.0, -5.0, 5.0], [15.0, 15.0, 5.0], vec![[0.0, 0.0, 5.0], [10.0, 10.0, 5.0]]),
        // The hole's axis: nothing.
        "hole_axis" => (holed_box(&mut k)?, [5.0, 5.0, -5.0], [5.0, 5.0, 15.0], vec![]),
        // A vertical line beside the hole: the bottom and the top face (SMLib's top face: (1, 1, 10)).
        "beside_hole" => (holed_box(&mut k)?, [1.0, 1.0, -5.0], [1.0, 1.0, 15.0], vec![[1.0, 1.0, 0.0], [1.0, 1.0, 10.0]]),
        _ => return Err(format!("{case} isn't ported")),
    };
    let mut pts = segment_hits(&k, body, a, b)?;
    pts.sort_by(|p, q| (p.x + p.y + p.z).total_cmp(&(q.x + q.y + q.z)));
    if pts.len() != want.len() || pts.iter().zip(&want).any(|(p, w)| (p - P3::new(w[0], w[1], w[2])).norm() > 1e-9) {
        return Err(format!("points {pts:?}, want {want:?}"));
    }
    Ok(())
}

/// `TestSmBooleanBoundsExample` (Examples/SM_API/boolean_bounds.cpp): a 12 × 12 × 1.5 plate less
/// 16 cylinders of radius 0.75 on a 3 mm grid, one at a time: its volume (relative 1e-6) and
/// tight box (1e-4).
fn plate_holes() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let mut plate = bx(&mut k, [0.0; 3], 12.0, 12.0, 1.5)?;
    for r in 0..4 {
        for c in 0..4 {
            let cyl = solid(k.pcylinder_on(&ax([(c as f64 + 0.5) * 3.0, (r as f64 + 0.5) * 3.0, -1.0]), 0.75, 3.5).map_err(e)?)?;
            plate = one(k.boolean(BoolOp::Subtract, plate, &[cyl]).map_err(e)?)?;
        }
    }
    let want = 12.0 * 12.0 * 1.5 - 16.0 * PI * 0.75 * 0.75 * 1.5;
    close("volume", volume(&k, plate)?, want, want * 1e-6)?;
    let bb = k.bounding_box(plate).map_err(e)?;
    for (i, (lo, hi)) in [(0.0, 12.0), (0.0, 12.0), (0.0, 1.5)].iter().enumerate() {
        close("box min", bb.min[i], *lo, 1e-4)?;
        close("box max", bb.max[i], *hi, 1e-4)?;
    }
    Ok(())
}

/// `TestSmBooleanWithOptions` and `TestSmNonManifoldBoolean` (TestSmBrep.cpp:2287, 2461): the
/// 10 mm box with the sphere of radius 5 inscribed in it (touching all six faces).
fn box_inscribed_sphere(op: BoolOp, want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
    let s = solid(k.psphere_on(&ax([5.0; 3]), 5.0).map_err(e)?)?;
    let r = one(k.boolean(op, b, &[s]).map_err(e)?)?;
    close("volume", volume(&k, r)?, want, 1e-6 * want)
}

/// A circle of radius `r` about `c` in its z plane, as a face (`face`) or a wire.
fn circle_profile(k: &NativeKernel, c: [f64; 3], r: f64, face: bool) -> Result<Shape, String> {
    let w = k.wire(&[&k.mkedge_full(&k.circle(c[0], c[1], c[2], r))]).map_err(e)?;
    if face { k.mkplane(&w).map_err(e) } else { Ok(w) }
}

/// `TestSmSweepAlongPlanarPath` and `TestSmCurveSweep` (TestSmBrep.cpp:2872, 1731): a circle of
/// radius `r` about the origin in z = 0 swept (`Kernel::sweep_with`, the profile square to the
/// path) along the polyline `path` in y = 0; capped: a solid of `want` (within `rel`), else a
/// sheet of one face.
fn sweep_path(r: f64, path: &[[f64; 3]], capped: bool, want: f64, rel: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let disc = Curve2::Circle { center: Point2::origin(), radius: r, source: None };
    let profile = Profile::new(Plane::top(), vec![Region { outer: Loop { curves: vec![disc] }, holes: vec![], source: None }]);
    // The xz plane: sketch (u, v) is (u, 0, v).
    let xz = Plane { origin: nalgebra::Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() };
    let path = path.windows(2).map(|w| PathCurve::Sketch { plane: xz, curve: Curve2::Line { a: Point2::new(w[0][0], w[0][2]), b: Point2::new(w[1][0], w[1][2]), source: None } }).collect();
    let spec = SweepSpec { body: if capped { BodyKind::Solid } else { BodyKind::Surface }, path, control: SweepControl::None, faces: vec![] };
    let b = one(k.sweep_with(&profile, &spec).map_err(e)?)?;
    if capped {
        return close("volume", volume(&k, b)?, want, rel * want);
    }
    let body = k.get(b).map_err(e)?;
    if let Some(bad) = invalid(body) {
        return Err(bad);
    }
    if body.is_solid() || body.faces.len() != 1 {
        return Err(format!("{} faces, solid: {}; SMLib gives one open face", body.faces.len(), body.is_solid()));
    }
    Ok(())
}

/// `TestSmCreateLinearSweep*` and `TestSmCreateRotationalSweep` (TestSmPrimitives.cpp:261–400):
/// SMLib asserts success; the closed forms.
fn primitive_sweep(case: &str) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let half_disc = |k: &NativeKernel| -> Result<Shape, String> {
        let arc = k.mkedge(&k.circle(0.0, 0.0, 0.0, 10.0), 0.0, PI);
        let chord = k.polyline(&[[-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).map_err(e)?;
        let w = k.wire(&[&arc, &chord]).map_err(e)?;
        k.mkplane(&w).map_err(e)
    };
    match case {
        "half_disc" => {
            let f = half_disc(&k)?;
            let b = solid(k.prism(&f, 0.0, 0.0, 10.0).map_err(e)?)?;
            close("volume", volume(&k, b)?, 500.0 * PI, 1e-9)
        }
        "cylinder" => {
            let f = circle_profile(&k, [0.0; 3], 10.0, true)?;
            let b = solid(k.prism(&f, 0.0, 0.0, 10.0).map_err(e)?)?;
            close("volume", volume(&k, b)?, 1000.0 * PI, 1e-9)
        }
        "plane_sheet" => {
            let w = k.polyline(&[[0.0; 3], [4.0, 2.0, 0.0]]).map_err(e)?;
            let b = solid(k.prism(&w, 0.0, 0.0, 10.0).map_err(e)?)?;
            close("area", k.mass_properties(b).map_err(e)?.surface_area, 10.0 * 20f64.sqrt(), 1e-9)
        }
        "sphere" => {
            let f = half_disc(&k)?;
            let b = solid(k.revol(&f, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 360.0).map_err(e)?)?;
            close("volume", volume(&k, b)?, 4000.0 * PI / 3.0, 1e-9)
        }
        _ => Err(format!("{case} isn't ported")),
    }
}

/// `TestSmCircularFillet` (TestSmFillets.cpp:26): every edge of the 5 × 10 × 15 box rounded by
/// 0.5: the box less 2r on each side grown by a ball of radius r.
fn fillet_box() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 5.0, 10.0, 15.0)?;
    let edges: Vec<_> = k.edges(b).map_err(e)?.into_iter().map(|ed| ed.id).collect();
    let r = one(k.fillet(b, &edges, 0.5).map_err(e)?)?;
    let (a, bb, c, r0) = (4.0, 9.0, 14.0, 0.5);
    let want = a * bb * c + 2.0 * r0 * (a * bb + bb * c + c * a) + PI * r0 * r0 * (a + bb + c) + 4.0 / 3.0 * PI * r0.powi(3);
    close("volume", volume(&k, r)?, want, 1e-7 * want)
}

/// `TestSmCreateOffsetSurface` (TestSmSurfaces.cpp:548): a sphere of radius 10 offset by −20 and
/// a torus (10, 2) by −5 collapse: SMLib fails, so must the kernel.
fn offset_collapse(torus: bool) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (b, d) = if torus { (solid(k.ptorus_on(&ax([0.0; 3]), 10.0, 2.0).map_err(e)?)?, -5.0) } else { (solid(k.psphere_on(&ax([0.0; 3]), 10.0).map_err(e)?)?, -20.0) };
    match k.offset(b, &OffsetSpec { distance: d, faces: vec![], sharp: false }) {
        Err(_) => Ok(()),
        Ok(r) => Err(format!("the offset by {d} gives {} bodies; SMLib fails", r.bodies.len())),
    }
}

/// `TestSmHealBrep`, `TestSmImportExport`, `TestSmTessellateBoundaries` (TestSmHeal.cpp:22,
/// TestSmImportExport.cpp:64, TestSmBrep.cpp:1205) on boxes.
fn box_round_trips(case: &str) -> Result<(), String> {
    let mut k = NativeKernel::new();
    match case {
        "heal" => {
            let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
            let r = one(Kernel::heal(&mut k, b, &[HealOp::Fix { precision: 1e-7, max_tolerance: 1e-3 }]).map_err(e)?)?;
            close("volume", volume(&k, r)?, 1000.0, 1e-9)?;
            if k.faces(r).map_err(e)?.len() != 6 {
                return Err("the healed box hasn't 6 faces".into());
            }
            Ok(())
        }
        "write_read" => {
            let b = bx(&mut k, [0.0; 3], 5.0, 10.0, 20.0)?;
            let bytes = k.write_body(b).map_err(e)?;
            let mut k2 = NativeKernel::new();
            let r = k2.read_body(&bytes).map_err(e)?;
            let (a, z) = (k.get(b).map_err(e)?, k2.get(r).map_err(e)?);
            if (a.faces.len(), a.edges.len(), a.verts.len()) != (z.faces.len(), z.edges.len(), z.verts.len()) {
                return Err("the counts changed".into());
            }
            close("volume", volume(&k2, r)?, 1000.0, 1e-9)
        }
        "boundaries" => {
            let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
            let m = k.tessellate(b, Tessellation { deflection: 5.0, angle: PI / 4.0 }).map_err(e)?;
            if m.edges.len() != 12 || m.edges.iter().any(|(_, p)| p.len() != 2) {
                return Err(format!("{} edge polylines of {:?} points; SMLib: 12 of 2", m.edges.len(), m.edges.iter().map(|(_, p)| p.len()).collect::<Vec<_>>()));
            }
            Ok(())
        }
        _ => Err(format!("{case} isn't ported")),
    }
}
/// `TestSmOffsetBrepFull` (TestSmBrep.cpp:2745; SMLib asserts success): the 10 mm box offset by
/// 1 with its convex edges extended (`bDoExtendedOffset`): the 12 mm box, 1728.
fn offset_box() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 10.0, 10.0, 10.0)?;
    let r = one(k.offset(b, &OffsetSpec { distance: 1.0, faces: vec![], sharp: true }).map_err(e)?)?;
    close("volume", volume(&k, r)?, 1728.0, 1e-9)?;
    let bb = k.bounding_box(r).map_err(e)?;
    for i in 0..3 {
        close("box min", bb.min[i], -1.0, 1e-9)?;
        close("box max", bb.max[i], 11.0, 1e-9)?;
    }
    Ok(())
}

/// `TestSmCreateSurfaceRevolution*` (TestSmSurfaces.cpp:415–548; SMLib asserts success): curves
/// turned a whole turn into sheets, their areas.
fn revolved_sheet(case: &str) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (w, o, d, want) = match case {
        // A half circle of radius 10 about its diameter (the x axis): a sphere, 400π.
        "sphere" => (k.mkedge(&k.circle(0.0, 0.0, 0.0, 10.0), 0.0, PI), [-10.0, 0.0, 0.0], [1.0, 0.0, 0.0], 400.0 * PI),
        // The line x = 5 from z = 0 to 10 about z: a cylinder, 100π.
        "cylinder" => (k.polyline(&[[5.0, 0.0, 0.0], [5.0, 0.0, 10.0]]).map_err(e)?, [0.0; 3], [0.0, 0.0, 1.0], 100.0 * PI),
        // (5, 0, 0)–(0, 0, 10) about z: a cone, π·5·√125.
        "cone" => (k.polyline(&[[5.0, 0.0, 0.0], [0.0, 0.0, 10.0]]).map_err(e)?, [0.0; 3], [0.0, 0.0, 1.0], PI * 5.0 * 125f64.sqrt()),
        // (5, 0, 0)–(3, 0, 10) about z: a truncated cone, π(5 + 3)·√104.
        "trunc_cone" => (k.polyline(&[[5.0, 0.0, 0.0], [3.0, 0.0, 10.0]]).map_err(e)?, [0.0; 3], [0.0, 0.0, 1.0], PI * 8.0 * 104f64.sqrt()),
        _ => return Err(format!("{case} isn't ported")),
    };
    let b = solid(k.revol(&w, o[0], o[1], o[2], d[0], d[1], d[2], 360.0).map_err(e)?)?;
    let body = k.get(b).map_err(e)?;
    if let Some(bad) = invalid(body) {
        return Err(bad);
    }
    close("area", k.mass_properties(b).map_err(e)?.surface_area, want, 1e-9 * want)
}

/// `TestSmCut` and `TestSmIntersectBrepWithPlane` (TestSmBrep.cpp:1494, TestSmIntersectors.cpp:203;
/// SMLib asserts success): the 5 × 10 × 15 box with every edge rounded by 0.5 cut by the plane
/// z = 7: two valid solids, the lower of 346.972 (the rounded box below z = 7, closed form), and
/// the section a rounded rectangle of area 49 + π/4 (each piece's flat face there).
fn cut_rounded() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = bx(&mut k, [0.0; 3], 5.0, 10.0, 15.0)?;
    let edges: Vec<_> = k.edges(b).map_err(e)?.into_iter().map(|ed| ed.id).collect();
    let r = one(k.fillet(b, &edges, 0.5).map_err(e)?)?;
    let plane = Plane { origin: nalgebra::Point3::new(0.0, 0.0, 7.0), ..Plane::top() };
    let cut = one(k.split(r, &cadrs_kernel::SplitTool::Plane(plane), 1).map_err(e)?)?;
    let pieces: Vec<BodyId> = k.split_solids(cut).map_err(e)?.into_iter().map(one).collect::<Result<_, _>>()?;
    if pieces.len() != 2 {
        return Err(format!("{} pieces", pieces.len()));
    }
    // The rounded box: the 4 × 9 × 14 box grown by a ball of radius 0.5; below z = 7, 6.5 of its
    // full section (36 + 13 + π/4) and the rounded bottom.
    let (a, bb, r0) = (4.0, 9.0, 0.5);
    let section = a * bb + 2.0 * r0 * (a + bb) + PI * r0 * r0;
    let lower = 6.5 * section + a * bb * 0.5 + 2.0 * (a + bb) * (PI * r0 * r0 / 4.0) + PI * (r0 * r0 * 0.5 - 0.5f64.powi(3) / 3.0);
    let mut vols: Vec<f64> = pieces.iter().map(|&p| volume(&k, p)).collect::<Result<_, _>>()?;
    vols.sort_by(f64::total_cmp);
    close("lower piece", vols[0], lower, 1e-7 * lower)?;
    for &p in &pieces {
        let cap: f64 = k.faces(p).map_err(e)?.iter().filter(|f| (f.center.z - 7.0).abs() < 1e-9 && f.plane.is_some()).map(|f| f.area).sum();
        close("section area", cap, section, 1e-7 * section)?;
    }
    Ok(())
}
'''


def line_of(src, needle):
    i = src.find(needle)
    assert i >= 0, needle
    return src.count("\n", 0, i) + 1


def extract(ctx):
    path = lambda f: ctx.path("tests/SM_API_test/src/" + f)
    read = lambda f: open(path(f), encoding="latin-1").read()
    brep, gen, inter, prim = read("TestSmBrep.cpp"), read("TestSmGeneral.cpp"), read("TestSmIntersectors.cpp"), read("TestSmPrimitives.cpp")
    fil, surf, dist = read("TestSmFillets.cpp"), read("TestSmSurfaces.cpp"), read("TestSmBrepDistance.cpp")
    heal, io = read("TestSmHeal.cpp"), read("TestSmImportExport.cpp")
    bounds = open(ctx.path("Examples/SM_API/boolean_bounds.cpp"), encoding="latin-1").read()
    code = strip_comments(brep)
    cases = []
    # TestMergeBrepsOperation(op, volume, manifold) on boxes A = 10³ at 0 and B = 4 × 8 × 6 at (8, 6, 3).
    a, b = "[0.0, 0.0, 0.0, 10.0, 10.0, 10.0]", "[8.0, 6.0, 3.0, 4.0, 8.0, 6.0]"
    assert "CreateMergeTestBox(8.0, 6.0, 3.0, 4.0, 8.0, 6.0" in code
    ops = {"SM_BO_UNION": ("Union", "union"), "SM_BO_INTERSECTION": ("Intersect", "intersection"), "SM_BO_DIFFERENCE": ("Subtract", "difference")}
    for m in re.finditer(r"TestMergeBrepsOperation\((SM_BO_\w+),\s*([0-9.]+),\s*TRUE\)", code):
        op, name = ops[m.group(1)]
        cases.append(Case("merge_%s" % name, "SMLib `TestSmMergeBreps` (TestSmBrep.cpp:%d): the 10 mm box and a 4 × 8 × 6 box at (8, 6, 3), %s, volume %s." % (
            line_of(brep, m.group(0)), name, m.group(2)), "merge(&[%s, %s], BoolOp::%s, %s)" % (a, b, op, rust_f64(float(m.group(2))))))
    for needle, boxes, op, want, doc in [
        ("CheckMergeTestVolume(pOutput, 2532.0)", ["[0.0, 0.0, 0.0, 10.0, 10.0, 10.0]", "[5.0, 5.0, 5.0, 10.0, 10.0, 10.0]", "[8.0, 8.0, 8.0, 10.0, 10.0, 10.0]"], "Union", 2532.0, "three 10 mm boxes at 0, (5, 5, 5), (8, 8, 8) united"),
        ("CheckMergeTestVolume(pOutput, 944.0)", ["[1.0, 1.0, 1.5, 2.0, 2.0, 2.0]", b, a], "Subtract", 944.0, "the list [C, B, A] differenced last to first: (A − B) − C"),
    ]:
        cases.append(Case("merge_%s3" % ("union" if op == "Union" else "difference"), "SMLib `TestSmMergeBreps` (TestSmBrep.cpp:%d): %s, volume %g." % (line_of(brep, needle), doc, want),
                          "merge(&[%s], BoolOp::%s, %s)" % (", ".join(boxes), op, rust_f64(want))))
    # The ShellCase table.
    lists = {"sFacesToShell": 1, "sDuplicateFacesToShell": 2, "sNoFacesToShell": 0}
    for k, m in enumerate(re.finditer(r"\{\s*(-?[0-9.]+),\s*(TRUE|FALSE),\s*&(\w+),\s*([0-9.]+)\s*\}", code)):
        d, faces, want = float(m.group(1)), lists[m.group(3)], float(m.group(4))
        what = {0: "closed (hollow)", 1: "opened at a face", 2: "opened at a face listed twice"}[faces]
        cases.append(Case("shell_box_%d" % k, "SMLib `TestSmShellBrepFull` (TestSmBrep.cpp:%d): the 10 mm box shelled by %g, %s, corners extended: volume %g." % (
            line_of(brep, m.group(0)), d, what, want), "shell_box(%s, %d, %s)" % (rust_f64(d), faces, rust_f64(want))))
    assert len([c for c in cases if c.name.startswith("shell_")]) == 4
    for about, doc in [(False, "about its centroid"), (True, "about the world origin (parallel axes)")]:
        cases.append(Case("mass_rotated_box" + ("_origin" if about else ""), "SMLib `TestSmMassProperties` (TestSmBrep.cpp:%d): a 2 × 6 × 10 box turned 30° about its centre, density 2: volume, area, mass, centroid and inertia %s." % (
            line_of(brep, "SmStatus TestSmMassProperties()"), doc), "mass_box(%s)" % ("true" if about else "false")))
    m = re.search(r"aExpectedAreas\[\]\s*=\s*\{([^}]*)\}", code)
    exprs = [x.strip() for x in m.group(1).split(",")]
    import math
    vals = [eval(x.replace("SM_PI", "math.pi"), {"math": math}) for x in exprs]
    for k, (what, v) in enumerate(zip(["a 2 × 3 × 4 box at (1e5, −2e5, 3e5)", "a sphere of radius 5", "a torus (5, 1)", "a disc of radius 5 less one of radius 2"], vals)):
        cases.append(Case("area_%d" % k, "SMLib `TestSmPrecisePropertySelection` (TestSmBrep.cpp:%d): the area of %s, %s." % (line_of(brep, "aExpectedAreas"), what, exprs[k]),
                          "areas(%d, %s)" % (k, rust_f64(v))))
    for name, a_, b_, want, doc in [
        ("apart", "[0.0, 0.0, 0.0, 1.0]", "[4.0, 0.0, 0.0, 1.0]", 3.0, "unit cubes 3 apart"),
        ("touching", "[0.0, 0.0, 0.0, 1.0]", "[1.0, 0.0, 0.0, 1.0]", 0.0, "unit cubes touching"),
        ("crossing", "[0.0, 0.0, 0.0, 10.0]", "[8.0, 3.0, 3.0, 4.0]", 0.0, "a 10 mm box and a 4 mm box crossing its face"),
        ("inside", "[0.0, 0.0, 0.0, 10.0]", "[3.0, 3.0, 3.0, 2.0]", 3.0, "a 2 mm box inside a 10 mm box, 3 from its faces"),
    ]:
        assert {"apart": "sPosB(4.0", "touching": "sPosTouch(1.0", "crossing": "sPosCross(8.0, 3.0, 3.0)", "inside": "sPosInside(3.0, 3.0, 3.0)"}[name] in dist, name
        cases.append(Case("distance_%s" % name, "SMLib `TestSmBrepDistance` (TestSmBrepDistance.cpp:%d): %s: %g (and the same the other way)." % (line_of(dist, "SmStatus TestSmBrepDistance()"), doc, want),
                          "distance(%s, %s, %s)" % (a_, b_, rust_f64(want))))
    cases.append(Case("box_queries", "SMLib `TestSmGetClosestPoint`, `TestSmFindEdge`, `TestSmFindFaces`, `TestSmGetFaces`, `TestSmGetEdges` (TestSmGeneral.cpp:%d): the 10 mm box's nearest point to (5, 5, 15) (5 away), its 6 faces and 12 edges, an edge through each of 5 edge middles, a face through (5, 5, 10)." % line_of(gen, "SmStatus TestSmGetClosestPoint()"),
                      "queries_on_box()"))
    assert "c1(0,0,-5), c2(10,0,-5), c3(0,10,5)" in inter
    cases.append(Case("intersect_circle_plane", "SMLib `TestSmIntersectCurveSurface` (TestSmIntersectors.cpp:%d): a circle of radius 3 and a tilted plane: (2, 5, 0) and (8, 5, 0)." % line_of(inter, "SmStatus TestSmIntersectCurveSurface()"), "circle_plane()"))
    for name, doc in [("through_box", "a line through the 10 mm box: (0, 5, 5) and (10, 5, 5) (SMLib: at least 2)"),
                      ("diagonal_through_edges", "a diagonal through two vertical edges of the box: exactly 2 points"),
                      ("hole_axis", "the axis of the holed box's hole: no point"),
                      ("beside_hole", "a vertical line beside the hole: (1, 1, 0) and (1, 1, 10), SMLib's top-face point")]:
        cases.append(Case("intersect_" + name, "SMLib `TestSmIntersectCurveFace`/`TestSmIntersectCurveBrep` (TestSmIntersectors.cpp:%d): %s." % (line_of(inter, "SmStatus TestSmIntersectCurveBrep()"), doc), 'curve_brep("%s")' % name))
    assert "kRadius = 0.75" in bounds
    cases.append(Case("boolean_plate_holes", "SMLib `TestSmBooleanBoundsExample` (Examples/SM_API/boolean_bounds.cpp): a 12 × 12 × 1.5 plate less 16 cylinders of radius 0.75: volume and tight box.", "plate_holes()"))
    cases.append(Case("boolean_box_less_inscribed_sphere", "SMLib `TestSmBooleanWithOptions` (TestSmBrep.cpp:%d): the 10 mm box less its inscribed sphere: 1000 − 500π/3·... (1000 − 4π·125/3)." % line_of(brep, "SmStatus TestSmBooleanWithOptions()"),
                      "box_inscribed_sphere(BoolOp::Subtract, %s)" % rust_f64(1000 - 4 * math.pi * 125 / 3)))
    cases.append(Case("boolean_box_with_inscribed_sphere", "SMLib `TestSmNonManifoldBoolean` (TestSmBrep.cpp:%d): the 10 mm box ∪ its inscribed sphere: the box." % line_of(brep, "SmStatus TestSmNonManifoldBoolean()"),
                      "box_inscribed_sphere(BoolOp::Union, 1000.0)"))
    cases.append(Case("boolean_overlapping_boxes", "SMLib `TestSmPiecewiseMerge`/`TestSmBooleanLists` (TestSmBrep.cpp:%d): 10 mm boxes at 0 and (5, 0, 0) united: 1500." % line_of(brep, "SmStatus TestSmPiecewiseMerge()"),
                      "merge(&[[0.0, 0.0, 0.0, 10.0, 10.0, 10.0], [5.0, 0.0, 0.0, 10.0, 10.0, 10.0]], BoolOp::Union, 1500.0)"))
    sweep_line = line_of(brep, "SmStatus TestSmSweepAlongPlanarPath()")
    cases.append(Case("sweep_circle_capped", "SMLib `TestSmSweepAlongPlanarPath` (TestSmBrep.cpp:%d): a unit circle swept 10 along z, both ends capped: 10π (SMLib: 1%%)." % sweep_line,
                      "sweep_path(1.0, &[[0.0; 3], [0.0, 0.0, 10.0]], true, %s, 0.01)" % rust_f64(10 * math.pi)))
    cases.append(Case("sweep_circle_open", "SMLib `TestSmSweepAlongPlanarPath` (TestSmBrep.cpp:%d): the same sweep uncapped: one open face." % sweep_line,
                      "sweep_path(1.0, &[[0.0; 3], [0.0, 0.0, 10.0]], false, 0.0, 0.0)"))
    cases.append(Case("sweep_circle_two_segments", "SMLib `TestSmSweepAlongPlanarPath` (TestSmBrep.cpp:%d): a unit circle along (0, 0, 0)–(0, 0, 10)–(5, 0, 15), capped: π(10 + √50) (SMLib: 2%%)." % sweep_line,
                      "sweep_path(1.0, &[[0.0; 3], [0.0, 0.0, 10.0], [5.0, 0.0, 15.0]], true, %s, 0.02)" % rust_f64(math.pi * (10 + math.sqrt(50)))))
    cases.append(Case("sweep_curve_straight_path", "SMLib `TestSmCurveSweep` (TestSmBrep.cpp:%d): a circle of radius 2 along (0, 0, 0)–(0, 0, 3)–(0, 0, 7)–(0, 0, 10): 40π." % line_of(brep, "SmStatus TestSmCurveSweep()"),
                      "sweep_path(2.0, &[[0.0; 3], [0.0, 0.0, 3.0], [0.0, 0.0, 7.0], [0.0, 0.0, 10.0]], true, %s, 1e-9)" % rust_f64(40 * math.pi)))
    for name, fn, doc in [("half_disc", "TestSmCreateLinearSweep()", "a half disc of radius 10 swept 10: 500π"),
                          ("cylinder", "TestSmCreateLinearSweepCylinder()", "a circle of radius 10 swept 10, capped: 1000π"),
                          ("plane_sheet", "TestSmCreateLinearSweepPlane()", "the segment (0, 0, 0)–(4, 2, 0) swept 10 along z: a sheet of area 10√20"),
                          ("sphere", "TestSmCreateRotationalSweep()", "a half disc of radius 10 turned about its chord: a sphere, 4000π/3")]:
        cases.append(Case("primitive_" + name, "SMLib `%s` (TestSmPrimitives.cpp:%d; SMLib asserts success): %s." % (fn.rstrip("()"), line_of(prim, "SmStatus " + fn), doc), 'primitive_sweep("%s")' % name))
    cases.append(Case("fillet_box_all_edges", "SMLib `TestSmCircularFillet` (TestSmFillets.cpp:%d; SMLib asserts success): every edge of a 5 × 10 × 15 box rounded by 0.5: 743.7294." % line_of(fil, "SmStatus TestSmCircularFillet()"), "fillet_box()"))
    for t, doc in [(False, "a sphere of radius 10 offset by −20"), (True, "a torus (10, 2) offset by −5")]:
        cases.append(Case("offset_collapses_%s" % ("torus" if t else "sphere"), "SMLib `TestSmCreateOffsetSurface` (TestSmSurfaces.cpp:%d): %s collapses: SMLib fails, and so must the kernel." % (line_of(surf, "SmStatus TestSmCreateOffsetSurface()"), doc),
                          "offset_collapse(%s)" % ("true" if t else "false")))
    for name, doc in [("heal", "`TestSmHealBrep` (TestSmHeal.cpp:%d): a sound box healed is the box" % line_of(heal, "SmStatus TestSmHealBrep()")),
                      ("write_read", "`TestSmImportExport` (TestSmImportExport.cpp:%d): a box written and read back keeps its counts and volume" % line_of(io, "SmStatus TestSmImportExport()")),
                      ("boundaries", "`TestSmTessellateBoundaries` (TestSmBrep.cpp:%d): the box's 12 edges each meshed as 2 points" % line_of(brep, "SmStatus TestSmTessellateBoundaries()"))]:
        cases.append(Case("box_" + name, "SMLib %s." % doc, 'box_round_trips("%s")' % name))
    cases.append(Case("offset_box_extended", "SMLib `TestSmOffsetBrepFull` (TestSmBrep.cpp:%d; SMLib asserts success): the 10 mm box offset by 1, convex edges extended: 1728." % line_of(brep, "SmStatus TestSmOffsetBrepFull()"), "offset_box()"))
    for name, fn, doc in [("sphere", "TestSmCreateSurfaceRevolutionSphere", "a half circle of radius 10 about its diameter: 400π"),
                          ("cylinder", "TestSmCreateSurfaceRevolutionCylinder", "the line x = 5, z 0 to 10, about z: 100π"),
                          ("cone", "TestSmCreateSurfaceRevolutionCone", "(5, 0, 0)–(0, 0, 10) about z: π·5·√125"),
                          ("trunc_cone", "TestSmCreateSurfaceRevolutionTruncCone", "(5, 0, 0)–(3, 0, 10) about z: 8π√104")]:
        cases.append(Case("revolved_sheet_" + name, "SMLib `%s` (TestSmSurfaces.cpp:%d; SMLib asserts success): %s." % (fn, line_of(surf, "SmStatus %s()" % fn), doc), 'revolved_sheet("%s")' % name))
    cases.append(Case("cut_rounded_box", "SMLib `TestSmCut`, `TestSmIntersectBrepWithPlane` (TestSmBrep.cpp:%d, TestSmIntersectors.cpp:%d; SMLib asserts success): the rounded 5 × 10 × 15 box cut at z = 7: the lower piece 346.972, the section 49 + π/4." % (
        line_of(brep, "SmStatus TestSmCut()"), line_of(inter, "SmStatus TestSmIntersectBrepWithPlane()")), "cut_rounded()"))
    return cases
