"""SMLib's analytic surface–surface intersection tests (tests/prog_test/src/smAdvancedTest.cpp,
`my_ssi_analytic_demo`, the `ssi-analytic` suite): 17 numbered cases intersecting surfaces built
in code (`my_create_test_*`: cone patches on a reference frame turned 0.1 rad about (1, 1, 1),
SMLib's B-spline surfaces of `surf_rev.smb`, `surf_sphere150.smb`, `surf_extrude.smb`, bilinear
planes) with `GlobalSurfaceIntersect` over their uv domains.

The surfaces are ported by hand (the `PRELUDE`'s `analytic`): cone patches as
`Surface::Cylinder` (or `Surface::Extrusion` of a circle where SMLib makes a surface of
extrusion), the files' B-splines as read, transformed as the test transforms them. The checks are
those of the ssi-advanced suite (tests/smlib_common/intersect.rs): every point on both surfaces,
none against some as SMLib asserts, and the total length where it has a closed form. Most cases
are cylinders lying on each other (the same radius and axis): SMLib returns the boundary of the
region they share (and lines where they meet edge to edge), whose length is the closed form.
There is no oracle for surfaces built in code.
"""
import math
import re

from smlib_to_rust import Case, Fixture, strip_comments
from smlib._oracle import want_literal

SOURCE = "smAdvancedTest.cpp"
TITLE = "analytic surface-surface intersection"

NOT_CONVERTED = [
    ("case 1 (SurfOfRevolution × Sphere turned π)", "the half sphere is turned about the X axis of the frame `SmSphere::IsNurbSurfaceSphere` gives it, which isn't known"),
    ("case 4 (SurfOfRevolution × its mirror)", "the copy is placed by `CreateDirectrixAtGeneratorParam` and the frame `IsNurbSurfaceSurfOfRevolution` detects"),
    ("case 8 (SurfOfRevolution × Cone)", "the cone is built on the frame `IsNurbSurfaceSurfOfRevolution` detects"),
]

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::math::{Affine, Frame, P3, V3};
use cadrs_kernel::native::surface::Surface;
use nalgebra::{Rotation3, Unit};

const SUITE: &str = "ssi_analytic";

/// `b` after `a`.
fn then(a: &Affine, b: &Affine) -> Affine {
    Affine { m: b.m * a.m, t: b.m * a.t + b.t }
}

/// A turn by `deg` degrees about the line through `o` along `axis`
/// (`SmAxis2Placement::RotateAboutAxisAtPoint`).
fn turn(o: P3, axis: V3, deg: f64) -> Affine {
    let m = Rotation3::from_axis_angle(&Unit::new_normalize(axis), deg.to_radians()).into_inner();
    Affine { m, t: o.coords - m * o.coords }
}

/// SMLib's `my_create_refframe` as a map: origin (−2, 0, 0), axes turned 0.1 rad about
/// (1, 1, 1).
fn refframe() -> Affine {
    let m = Rotation3::from_axis_angle(&Unit::new_normalize(V3::new(1.0, 1.0, 1.0)), 0.1).into_inner();
    Affine { m, t: V3::new(-2.0, 0.0, 0.0) }
}

/// The world frame moved by `m`.
fn frame_of(m: &Affine) -> Frame {
    Frame { o: m.point(&P3::origin()), x: m.vector(&V3::x()), y: m.vector(&V3::y()), z: m.vector(&V3::z()) }
}

/// Moved `h` along the reference frame's axis and turned `deg` about it.
fn along_axis(h: f64, deg: f64) -> Affine {
    let r = refframe();
    let (o, z) = (r.point(&P3::origin()), r.vector(&V3::z()));
    then(&Affine::translation(z * h), &turn(o, z, deg))
}

/// `my_create_test_cyl` (`my_create_test_rotcyl`): `CreateConePatch` on the reference frame,
/// radius 2, height 5, angles 0 to `deg`, then moved by `m`.
fn cyl(deg: f64, m: &Affine) -> Patch {
    let f = frame_of(&then(&refframe(), m));
    Patch::rect(Surface::Cylinder { f, r: 2.0 }, ((0.0, deg.to_radians()), (0.0, 5.0)))
}

/// `my_create_test_extruded_surf2`: the same cylinder as a surface of extrusion.
fn ext_cyl(deg: f64, m: &Affine) -> Patch {
    let f = frame_of(&then(&refframe(), m));
    Patch::rect(Surface::Extrusion { c: Curve::Circle { c: f.o, x: f.x, y: f.y, r: 2.0 }, d: f.z }, ((0.0, deg.to_radians()), (0.0, 5.0)))
}

/// A bilinear plane (`CreateBilinearSurface`) from (x0, y0, 0) to (x0 + 10, y0 + 10, 0) on the
/// reference frame.
fn square(x0: f64, y0: f64) -> Patch {
    let r = refframe();
    let f = Frame { o: r.point(&P3::new(x0, y0, 0.0)), x: r.vector(&V3::x()), y: r.vector(&V3::y()), z: r.vector(&V3::z()) };
    Patch::rect(Surface::Plane { f }, ((0.0, 10.0), (0.0, 10.0)))
}

/// The B-spline surface of `file`'s one face over the fractions `lo` to `hi` of its domain
/// (`SmExtent2d::Evaluate`, `TrimWithDomain`), moved by `m`.
fn file_surface(file: &str, lo: (f64, f64), hi: (f64, f64), m: &Affine) -> Result<Patch, String> {
    let model = model(SUITE, file)?;
    let [face] = model.body.faces.as_slice() else { return Err(format!("{file}: {} faces", model.body.faces.len())) };
    let Surface::Nurbs(n) = &face.surf else {
        // The reader gave the face the analytic surface its B-spline is: the whole face is the
        // same surface, a part of SMLib's domain isn't.
        if (lo, hi) != ((0.0, 0.0), (1.0, 1.0)) {
            return Err(format!("{file}: the reader made the B-spline analytic; SMLib's sub-domain isn't known in its parameters"));
        }
        let p = Patch::face(&model.body, 0);
        return Ok(Patch { surf: p.surf.transformed(m), ..p });
    };
    let ((u0, u1), (v0, v1)) = n.range();
    let uv = ((u0 + (u1 - u0) * lo.0, u0 + (u1 - u0) * hi.0), (v0 + (v1 - v0) * lo.1, v0 + (v1 - v0) * hi.1));
    Ok(Patch::rect(face.surf.transformed(m), uv))
}

/// A surface of extrusion's vector (`GetExtrusionVector`): along its straight direction (the
/// B-spline's degree-1 one), from its low end to its high end.
fn extrusion_vector(p: &Patch) -> Result<V3, String> {
    let ((u0, u1), (v0, v1)) = p.uv;
    if let Surface::Extrusion { d, .. } = &p.surf {
        return Ok(d * (v1 - v0));
    }
    let Surface::Nurbs(n) = &p.surf else { return Err("not an extrusion".into()) };
    if n.dv == 1 {
        Ok(p.surf.eval(u0, v1) - p.surf.eval(u0, v0))
    } else if n.du == 1 {
        Ok(p.surf.eval(u1, v0) - p.surf.eval(u0, v0))
    } else {
        Err("no straight direction".into())
    }
}

/// `SmVector3d::MakeUnitOrthoVectors(NULL, x, y, z)`: x along `v`, z = x × (0, 1, 0) (or
/// × (0, 0, 1) when v is along y), y = z × x.
fn unit_ortho(v: V3) -> (V3, V3, V3) {
    let x = v.normalize();
    let mut z = x.cross(&V3::y());
    if z.norm_squared() < 1e-24 {
        z = x.cross(&V3::z());
    }
    let z = z.normalize();
    (x, z.cross(&x), z)
}

/// `my_ssi_analytic_demo` case `case`: its two surfaces intersected.
fn analytic(case: u32, want: &Want) -> Result<(), String> {
    let id = Affine::identity();
    let (a, b) = match case {
        2 => (cyl(240.0, &id), cyl(240.0, &along_axis(4.0, 180.0))),
        3 => (ext_cyl(200.0, &id), ext_cyl(200.0, &along_axis(3.0, 200.0))),
        5 | 13 => {
            // `my_create_test_extruded_surf`, and a copy shifted along the
            // `MakeUnitOrthoVectors` of its extrusion vector by (5, 0, 1) (case 13: and by the
            // extrusion's height less 5 along it).
            let a = file_surface("surf_extrude.smb", (0.0, 0.0), (1.0, 1.0), &refframe())?;
            let ev = extrusion_vector(&a)?;
            let (x, _, z) = unit_ortho(ev);
            let mut shift = x * 5.0 + z;
            if case == 13 {
                shift += x * (ev.norm() - 5.0);
            }
            let b = Patch::rect(a.surf.transformed(&Affine::translation(shift)), a.uv);
            (a, b)
        }
        6 => (ext_cyl(200.0, &id), ext_cyl(200.0, &along_axis(3.0, 90.0))),
        7 => (cyl(180.0, &id), cyl(180.0, &along_axis(5.0, 0.0))),
        9 => (
            file_surface("surf_rev.smb", (0.1, 0.5), (0.5, 1.0), &refframe())?,
            file_surface("surf_sphere150.smb", (0.1, 0.5), (0.5, 1.0), &refframe())?,
        ),
        10 => (cyl(180.0, &id), cyl(180.0, &along_axis(4.0, 0.0))),
        11 => (cyl(180.0, &id), cyl(180.0, &along_axis(4.0, 90.0))),
        12 => (cyl(180.0, &id), cyl(180.0, &along_axis(4.0, 180.0))),
        14 => (square(0.0, 0.0), square(10.0, 5.0)),
        15 => {
            let b = file_surface("surf_extrude.smb", (0.0, 0.0), (1.0, 1.0), &refframe())?;
            let a = file_surface("surf_extrude.smb", (0.0, 0.0), (1.0, 1.0), &then(&Affine::translation(V3::new(1.0, 0.0, 7.0)), &refframe()))?;
            (a, b)
        }
        16 => {
            let b = file_surface("surf_sphere150.smb", (0.0, 0.0), (1.0, 1.0), &refframe())?;
            let m = then(&Affine::translation(V3::new(0.0, 1.0, 0.0)), &turn(P3::new(-2.0, 0.0, 0.0), V3::y(), 180.0));
            let a = file_surface("surf_sphere150.smb", (0.0, 0.0), (1.0, 1.0), &then(&m, &refframe()))?;
            (a, b)
        }
        17 => (square(0.0, 0.0), square(11.0, 5.0)),
        _ => return Err(format!("case {case} isn't ported")),
    };
    let found = intersect_patches(&[a], &[b])?;
    check_found(&found, want)
}
'''

R = 2.0
DEG = math.pi / 180.0

# The closed-form length of what SMLib finds, where there is one: cylinders of radius 2 lying on
# each other give the boundary of the region they share (arcs and lines), plus lines where they
# meet edge to edge.
CASES = {
    2: ("two 240° cylinder patches, the second moved 4 along the axis and turned 180°: two shared regions [0°, 60°] and [180°, 240°] × height 1",
        2 * (2 * R * 60 * DEG + 2 * 1.0), []),
    3: ("two 200° cylinders of extrusion, the second moved 3 and turned 200°: the shared region [0°, 40°] × height 2 and the line where they meet at 200°",
        2 * R * 40 * DEG + 2 * 2.0 + 2.0, []),
    5: ("`surf_extrude.smb` and a copy shifted (5, 0, 1) along its extrusion frame", None, ["surf_extrude.smb"]),
    6: ("two 200° cylinders of extrusion, the second moved 3 and turned 90°: the shared region [90°, 200°] × height 2",
        2 * R * 110 * DEG + 2 * 2.0, []),
    7: ("two 180° cylinder patches, the second moved 5 along the axis: they meet along a half circle", R * math.pi, []),
    9: ("parts of `surf_rev.smb` and `surf_sphere150.smb` on the reference frame", None, ["surf_rev.smb", "surf_sphere150.smb"]),
    10: ("two 180° cylinder patches, the second moved 4 along the axis: the shared region, height 1", 2 * R * math.pi + 2 * 1.0, []),
    11: ("two 180° cylinder patches, the second moved 4 and turned 90°: the shared region [90°, 180°] × height 1",
         2 * R * 90 * DEG + 2 * 1.0, []),
    12: ("two 180° cylinder patches, the second moved 4 and turned 180°: they meet along two lines of length 1", 2 * 1.0, []),
    13: ("`surf_extrude.smb` and a copy shifted along its extrusion by its height and 1 across", None, ["surf_extrude.smb"]),
    14: ("two coplanar 10 × 10 squares meeting along an edge of length 5", 5.0, []),
    15: ("`surf_extrude.smb` and a copy moved (1, 0, 7), on the reference frame", None, ["surf_extrude.smb"]),
    16: ("`surf_sphere150.smb` and a copy moved 1 along y and turned 180° about y", None, ["surf_sphere150.smb"]),
    17: ("two coplanar 10 × 10 squares 1 apart", 0.0, []),
}


def asserted_counts(src):
    """{case: count} from `SM_ASSERT(nSolutions == n)` in each case of the demo's switch."""
    body = strip_comments(src[src.find("PT_EXPORT SmStatus my_ssi_analytic_demo"):src.find("end my_ssi_analytic_demo")])
    out = {}
    marks = list(re.finditer(r"\bcase\s+(\d+)\s*:", body))
    for k, m in enumerate(marks):
        end = marks[k + 1].start() if k + 1 < len(marks) else len(body)
        a = re.search(r"SM_ASSERT\(\s*nSolutions\s*==\s*(\d+)\s*\)", body[m.end():end])
        if a:
            out[int(m.group(1))] = int(a.group(1))
    return out


def extract(ctx):
    counts = asserted_counts(ctx.source(SOURCE))
    cases = []
    for n, (what, length, files) in sorted(CASES.items()):
        count = counts[n]
        doc = "SMLib analytic SSI %d: %s; SMLib asserts %d curves%s." % (
            n, what, count, "" if length is None else ", closed-form length %.6g" % length)
        cases.append(Case("analytic_%02d" % n, doc, "analytic(%d, %s)" % (n, want_literal(count > 0, length, [])),
                          [Fixture("TestFiles/pt_TestFiles/" + f) for f in files]))
    return cases
