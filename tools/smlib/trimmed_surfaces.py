"""SMLib's trimmed-surface regression (tests/prog_test/src/smAdvancedTest.cpp:1438,
`my_test_tsurface_regression`, the `trimmed-surfaces` suite), the parts that port: the checks
SMLib's test makes of trimmed faces it builds, as closed forms.

- `CreateConePatch` (a cylinder r 1, h 1, full turn) and its `Area`: here the lateral face of a
  cylinder of the same size, `mass::face_area` = 2π.
- `my_test_tsurf_creation` (smTopoTest.cpp:1259): a 10 × 10 plane less its top right quarter,
  with a triangular hole and a circular one (`MakeFaceWithCurves`). Here the same face (the DRAW
  harness's `polyline`, `mkplane`, `add`), extruded by 1: its volume and area, and points in the
  material, in the holes, in the missing quarter and on the boundary classified
  (`classify::classify_point`).
- `my_test_analytic_tsurf_creation` (smTopoTest.cpp:1080): a 20 × 20 plane and a quarter of a
  cylinder (r 2, height 2 about z through the plane) imprinted on each other
  (`SmTopologyIntersector::IntersectInsertRelate`). Here the plane (the top face of a slab:
  sections take solids) cut through the quarter cylinder (`boolean::section`): the section's
  edges on the cylinder are a quarter circle, π long.
"""
import math

from smlib_to_rust import Case, rust_f64

SOURCE = "smAdvancedTest.cpp, smTopoTest.cpp"
TITLE = "trimmed-surface"

NOT_CONVERTED = [
    ("distance finding (ss_plane_sph1, ss_2parallel_planes, ss_2adj_planes: `my_test_tsurface_tsurface_solve` with minimize, maximize, normalize, projected and directed solves)",
     "missing API: the kernel has no face–face distance or extremum solver"),
    ("`my_test_trim_surfaces`: `ProjectAndTrim` of a B-spline curve onto the trimmed plane, `MakeFaceWithCurves` from uv curves, the trimming tools",
     "missing API: no projecting a curve onto a body's faces and trimming them by it"),
    ("`my_test_tsurf_cyl_creation_fast/2d/3d` timing loops, `my_test_tsurf_insert_edge`",
     "SMLib's own face-building calls (timings, `InsertEdge`), nothing asserted"),
]

PRELUDE = r'''
use cadrs_kernel::native::classify;
use cadrs_kernel::native::draw::Shape;
use cadrs_kernel::native::mass;
use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::PointClass;

fn e(x: impl std::fmt::Debug) -> String {
    format!("{x:?}")
}

fn body(k: &NativeKernel, s: &Shape) -> Result<Body, String> {
    let Shape::Body(id) = s else { return Err(format!("not a body: {s:?}")) };
    Ok((**k.get(*id).map_err(e)?).clone())
}

fn close(what: &str, got: f64, want: f64) -> Result<(), String> {
    if (got - want).abs() > 1e-6 * want.abs().max(1.0) {
        return Err(format!("{what} {got:.9}, the closed form gives {want:.9}"));
    }
    Ok(())
}

/// The cone patch of `my_test_tsurface_regression` (radius 1 to 1, a full turn, height 1, at
/// (−2, 0, 0)): its lateral face's area.
fn cone_patch_area(want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let ax = k.plane(-2.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let cyl = k.pcylinder_on(&ax, 1.0, 1.0).map_err(e)?;
    let b = body(&k, &cyl)?;
    let lateral: f64 = (0..b.faces.len()).filter(|&f| matches!(b.faces[f].surf, Surface::Cylinder { .. })).map(|f| mass::face_area(&b, f).0).sum();
    close("lateral area", lateral, want)
}

/// `my_test_tsurf_creation`'s face (the L-shaped plane with a triangular and a circular hole),
/// extruded by 1.
fn tsurf(k: &mut NativeKernel) -> Result<Body, String> {
    let outer = k.polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 5.0, 0.0], [5.0, 5.0, 0.0], [5.0, 10.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 0.0]]).map_err(e)?;
    let tri = k.polyline(&[[1.0, 5.0, 0.0], [1.0, 8.0, 0.0], [4.0, 5.0, 0.0], [1.0, 5.0, 0.0]]).map_err(e)?;
    let c = k.circle(6.0, 2.5, 0.0, 1.5);
    let circ = k.mkedge_full(&c);
    let circ = k.wire(&[&circ]).map_err(e)?;
    let f = k.mkplane(&outer).map_err(e)?;
    let f = k.add_hole(&tri, &f).map_err(e)?;
    let f = k.add_hole(&circ, &f).map_err(e)?;
    let s = k.prism(&f, 0.0, 0.0, 1.0).map_err(e)?;
    body(k, &s)
}

/// The extruded face's volume (the face's area) and area.
fn tsurf_creation_props(volume: f64, area: f64) -> Result<(), String> {
    let b = tsurf(&mut NativeKernel::new())?;
    if let Some(i) = b.check().first() {
        return Err(format!("invalid: {i}"));
    }
    let m = mass::body_mass(&b);
    close("volume", m.volume, volume)?;
    close("area", m.surface_area, area)
}

/// Points of the extruded face at mid height, classified.
fn tsurf_creation_classify(points: &[([f64; 2], PointClass)]) -> Result<(), String> {
    let b = tsurf(&mut NativeKernel::new())?;
    let polys: Vec<_> = b.faces.iter().map(|f| classify::face_polygons(&b, f)).collect();
    let mut errs = Vec::new();
    for (q, want) in points {
        let p = P3::new(q[0], q[1], 0.5);
        let got = classify::classify_point(&b, &polys, &p, 1e-6);
        if got != *want {
            errs.push(format!("({}, {}): {got:?}, want {want:?}", q[0], q[1]));
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}

/// `my_test_analytic_tsurf_creation`: the plane z = 0 (20 × 20) through a quarter of a cylinder
/// of radius 2 from z = −1 to 1; the section's edges on the cylinder.
fn analytic_tsurf_section(want: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let ax = k.plane(0.0, 0.0, -1.0, 0.0, 0.0, 1.0);
    let quarter = k.pcylinder_a(Some(&ax), 2.0, 2.0, 90.0).map_err(e)?;
    // The plane as the top face of a slab below it (sections take solids).
    let base = k.plane(-10.0, -10.0, -30.0, 0.0, 0.0, 1.0);
    let plane = k.box_on(&base, 20.0, 20.0, 30.0).map_err(e)?;
    let edges = match k.bsection(&quarter, &plane).map_err(e)? {
        Shape::Wire(es) => es,
        other => return Err(format!("a section of {other:?}")),
    };
    let on_cyl: f64 = edges
        .iter()
        .filter(|ed| {
            let m = ed.curve.point(0.5 * (ed.t0 + ed.t1));
            (m.x.hypot(m.y) - 2.0).abs() < 1e-6 && m.z.abs() < 1e-6 && m.x > -1e-9 && m.y > -1e-9
        })
        .map(|ed| ed.length())
        .sum();
    close("section length on the cylinder", on_cyl, want)
}
'''

# my_test_tsurf_creation's face: the L (100 − 25), less the triangle (1,5)-(1,8)-(4,5) and the
# circle of radius 1.5.
FACE_AREA = 75.0 - 4.5 - math.pi * 1.5 ** 2
PERIMETER = 40.0 + (3.0 + 3.0 + 3.0 * math.sqrt(2.0)) + 2 * math.pi * 1.5


def extract(ctx):
    top = ctx.source("smTopoTest.cpp")
    adv = ctx.source("smAdvancedTest.cpp")
    for needle, src in (("SmStatus my_test_tsurf_creation", top), ("SmStatus my_test_analytic_tsurf_creation", top), ("CreateConePatch", adv)):
        assert needle in src, needle
    pts = [
        ((2.0, 2.0), "Inside"), ((9.0, 1.0), "Inside"), ((3.0, 9.0), "Inside"), ((2.0, 6.0), "Outside"),
        ((6.0, 2.5), "Outside"), ((7.5, 7.5), "Outside"), ((11.0, 1.0), "Outside"), ((0.0, 5.0), "OnBoundary"),
        ((7.5, 2.5), "OnBoundary"),
    ]
    plist = ", ".join("([%s, %s], PointClass::%s)" % (rust_f64(x), rust_f64(y), c) for (x, y), c in pts)
    return [
        Case("tsurf_cone_patch_area", "SMLib trimmed surfaces: `CreateConePatch` r 1, h 1, full turn: its area 2π.",
             "cone_patch_area(%s)" % rust_f64(2 * math.pi)),
        Case("tsurf_creation_volume_area", "SMLib trimmed surfaces: `my_test_tsurf_creation`'s face (area 70.5 − 2.25π) extruded by 1: volume and area.",
             "tsurf_creation_props(%s, %s)" % (rust_f64(FACE_AREA), rust_f64(2 * FACE_AREA + PERIMETER))),
        Case("tsurf_creation_point_classes", "SMLib trimmed surfaces: `my_test_tsurf_creation`'s face extruded by 1, points in the material, the holes, the missing quarter and on the boundary.",
             "tsurf_creation_classify(&[%s])" % plist),
        Case("tsurf_analytic_section", "SMLib trimmed surfaces: `my_test_analytic_tsurf_creation`, a plane through a quarter cylinder r 2: a quarter circle π long.",
             "analytic_tsurf_section(%s)" % rust_f64(math.pi)),
    ]
