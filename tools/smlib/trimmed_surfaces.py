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
- The distance solves (`my_test_tsurface_tsurface_solve`, `SmTopologySolver::BrepBrepSolve`)
  with `SM_SO_MINIMIZE` and `SM_SO_MAXIMIZE` on `ss_plane_sph1` (a band of a sphere r 2 at
  (10, 2, 2) bounded by four half circles, and the square x = 5, 0 ≤ y, z ≤ 5),
  `ss_2parallel_planes` and `ss_2adj_planes` (squares of side 5): `Kernel::min_distance` and
  `max_distance` against closed forms (the sphere's nearest point (8, 2, 2); the farthest pair
  a corner of the square and a point on one of the band's half circles; the squares' gap and
  corners). SMLib's asserted counts (1/3 for the sphere, 69/36 for the parallel squares, ...)
  count its sampled solutions along degenerate sets; the values are what both agree on.
"""
import math

from smlib_to_rust import Case, Fixture, rust_f64

SOURCE = "smAdvancedTest.cpp, smTopoTest.cpp"
TITLE = "trimmed-surface"

NOT_CONVERTED = [
    ("the normalize, projected and directed solves of `my_test_tsurface_tsurface_solve` (SM_SO_NORMALIZE, SM_SO_PROJECTED_MINIMIZE, SM_SO_DIRECTED_MINIMIZE/MAXIMIZE, SM_SO_SIGNED_DIRECTED_MINIMIZE: 4/8/4 on ss_plane_sph1)",
     "SMLib's projected solver operations (distances in a view plane, along a direction) count its sampled solutions; the kernel's distance API has the 3D minimum and maximum (converted above)"),
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
use cadrs_kernel::native::smb;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{DistanceTarget, FaceId, Kernel, PointClass};

const SUITE: &str = "trimmed_surfaces";

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
    if let Some(i) = invalid(&b) {
        return Err(i);
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

/// `my_test_tsurface_tsurface_solve` on the two one-face B-reps of `file`: the minimum (or with
/// `max`, the maximum) distance between the faces, within `tol`, and the points reaching it on
/// each face.
fn solve(file: &str, max: bool, want: f64, tol: f64) -> Result<(), String> {
    let bytes = std::fs::read(data(SUITE, file)).map_err(|x| format!("{file}: {x}"))?;
    let p = smb::read_part(&bytes)?;
    if p.breps.len() != 2 {
        return Err(format!("{} B-reps", p.breps.len()));
    }
    let mut k = NativeKernel::new();
    let a = k.insert(p.breps[0].body.clone());
    let b = k.insert(p.breps[1].body.clone());
    let (ta, tb) = (DistanceTarget::Face(a, FaceId(0)), DistanceTarget::Face(b, FaceId(0)));
    let r = if max { k.max_distance(&ta, &tb) } else { k.min_distance(&ta, &tb) }.map_err(e)?;
    if (r.distance - want).abs() > tol {
        return Err(format!("{} {:.9}, the closed form gives {want:.9}", if max { "maximum distance" } else { "minimum distance" }, r.distance));
    }
    close("the points' distance", (r.a - r.b).norm(), r.distance)?;
    for (side, pt) in [(0, r.a), (1, r.b)] {
        let s = &p.breps[side].body.faces[0].surf;
        let d = s.distance(&pt);
        if d > 1e-7 {
            return Err(format!("point {side} {pt:?} is {d:.1e} off its face's surface"));
        }
    }
    Ok(())
}
'''

# my_test_tsurf_creation's face: the L (100 − 25), less the triangle (1,5)-(1,8)-(4,5) and the
# circle of radius 1.5.
FACE_AREA = 75.0 - 4.5 - math.pi * 1.5 ** 2
PERIMETER = 40.0 + (3.0 + 3.0 + 3.0 * math.sqrt(2.0)) + 2 * math.pi * 1.5


def plane_sph1_max():
    """The farthest pair of ss_plane_sph1: a corner of the square and a point of the band's
    boundary (half circles of radius √2 in the planes y = 2 ± √2 on the side x ≥ 10 and
    z = 2 ± √2 on the side x ≤ 10). The sphere points farthest from the corners (antipodal,
    the only critical points inside) all lie off the band."""
    r2 = math.sqrt(2.0)
    c = (10.0, 2.0, 2.0)
    sub = lambda a, b: tuple(x - y for x, y in zip(a, b))
    dot = lambda a, b: sum(x * y for x, y in zip(a, b))
    arcs = [((10.0, 2.0 + r2, 2.0), (1, 0, 0), (0, 0, 1), -math.pi / 2, math.pi / 2),
            ((10.0, 2.0 - r2, 2.0), (1, 0, 0), (0, 0, 1), -math.pi / 2, math.pi / 2),
            ((10.0, 2.0, 2.0 - r2), (1, 0, 0), (0, 1, 0), math.pi / 2, 3 * math.pi / 2),
            ((10.0, 2.0, 2.0 + r2), (1, 0, 0), (0, 1, 0), math.pi / 2, 3 * math.pi / 2)]
    best = 0.0
    for k in [(5.0, y, z) for y in (0.0, 5.0) for z in (0.0, 5.0)]:
        w = sub(c, k)
        n = math.sqrt(dot(w, w))
        p = tuple(ci + 2.0 * wi / n for ci, wi in zip(c, w))
        assert not ((p[0] >= 10 and abs(p[1] - 2) >= r2) or (p[0] <= 10 and abs(p[2] - 2) <= r2))
        for c0, a, b, t0, t1 in arcs:
            d = sub(c0, k)
            A, B = r2 * dot(d, a), r2 * dot(d, b)
            ts = [t0, t1] + [math.atan2(B, A) + 2 * math.pi * j for j in range(-2, 3)]
            for t in ts:
                if t0 <= t <= t1:
                    best = max(best, dot(d, d) + 2.0 + 2.0 * (A * math.cos(t) + B * math.sin(t)))
    return math.sqrt(best)


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
        Case("tsurf_solve_plane_sph1_min", "SMLib trimmed surfaces: `ss_plane_sph1` minimize: the sphere band's point (8, 2, 2) is 3 from the square x = 5.",
             'solve("ss_plane_sph1.smp", false, %s, 1e-9)' % rust_f64(3.0), [Fixture("TestFiles/pt_TestFiles/ss_plane_sph1.smp")]),
        Case("tsurf_solve_plane_sph1_max", "SMLib trimmed surfaces: `ss_plane_sph1` maximize: from the square's corner (5, 5, 5) to the band's half circle in y = 2 − √2 (to 1e-3: the file's half circles in y = 2 ± √2 are B-splines on the sphere off their planes by up to 2e-3).",
             'solve("ss_plane_sph1.smp", true, %s, 1e-3)' % rust_f64(plane_sph1_max()), [Fixture("TestFiles/pt_TestFiles/ss_plane_sph1.smp")]),
        Case("tsurf_solve_2parallel_planes_min", "SMLib trimmed surfaces: `ss_2parallel_planes` minimize: the squares y = 0 and y = 5 are 5 apart.",
             'solve("ss_2parallel_planes.smp", false, %s, 1e-9)' % rust_f64(5.0), [Fixture("TestFiles/pt_TestFiles/ss_2parallel_planes.smp")]),
        Case("tsurf_solve_2parallel_planes_max", "SMLib trimmed surfaces: `ss_2parallel_planes` maximize: opposite corners, √75 apart.",
             'solve("ss_2parallel_planes.smp", true, %s, 1e-9)' % rust_f64(math.sqrt(75.0)), [Fixture("TestFiles/pt_TestFiles/ss_2parallel_planes.smp")]),
        Case("tsurf_solve_2adj_planes_min", "SMLib trimmed surfaces: `ss_2adj_planes` minimize: the squares x = 5 and y = 0 share an edge.",
             'solve("ss_2adj_planes.smp", false, %s, 1e-9)' % rust_f64(0.0), [Fixture("TestFiles/pt_TestFiles/ss_2adj_planes.smp")]),
        Case("tsurf_solve_2adj_planes_max", "SMLib trimmed surfaces: `ss_2adj_planes` maximize: corners (5, 5, z) and (0, 0, 5 − z), √75 apart.",
             'solve("ss_2adj_planes.smp", true, %s, 1e-9)' % rust_f64(math.sqrt(75.0)), [Fixture("TestFiles/pt_TestFiles/ss_2adj_planes.smp")]),
        Case("tsurf_analytic_section", "SMLib trimmed surfaces: `my_test_analytic_tsurf_creation`, a plane through a quarter cylinder r 2: a quarter circle π long.",
             "analytic_tsurf_section(%s)" % rust_f64(math.pi)),
    ]
