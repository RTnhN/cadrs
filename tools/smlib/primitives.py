"""SMLib's primitive tests (tests/prog_test/src/smSweepTest.cpp, the `primitives` suite of
prog_test.cpp, run by `my_primsol_regression`): `my_primsol_demo`'s cases (`SmPrimitiveCreation`:
skins, boxes, linear and rotational sweeps of curves, cones, spheres, tori, a curve sweep, and
the draft sweep of its dead `if (FALSE)` block) and the `CreateCylindricalBox` input checks.

Each case builds the cadrs equivalent with the kernel's own operations (box, cone, sphere and
torus of the draw harness; extrude, prism, revolve, draft, loft, sweep; sheet booleans of the
draw harness for SMLib's `Boolean2D`) and checks it against closed forms (`_geom.py`): SMLib's
curved primitives are spline approximations, so its own volumes aren't a reference. SMLib's
asserted counts are checked where the result has no closed (periodic) faces.
"""
import math
import re

import numpy as np

from smlib import _geom as g
from smlib_to_rust import Case, case_blocks, rust_f64, strip_comments

SOURCE = "smSweepTest.cpp"
TITLE = "primitive creation"

NOT_CONVERTED = [
    ("my_primsol_demo 9", "empty (and never run: `my_primsol_demo_count` is 8)"),
]

PRELUDE = r'''
#[path = "smlib_common/kinds.rs"]
mod kinds;
use kinds::*;

use cadrs_kernel::native::draw::{Ax3, Shape};
use cadrs_kernel::native::math::{P3, V3};
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{
    BodyId, BodyKind, Curve2, DraftSpec, Extent, FaceId, Kernel, KernelError, LoftEnd, LoftSection, LoftSpec, Loop, PathCurve, Plane,
    Profile, Region, SweepControl, SweepSpec,
};
use nalgebra::{Point2, Unit};

/// A profile in the plane z = 0.
fn top(regions: Vec<Region>) -> Profile {
    Profile::new(Plane::top(), regions)
}

/// An arc in plane coordinates.
fn arc(c: [f64; 2], r: f64, start_deg: f64, sweep_deg: f64) -> Curve2 {
    Curve2::Arc { center: Point2::new(c[0], c[1]), radius: r, start_angle: start_deg.to_radians(), sweep: sweep_deg.to_radians(), source: None }
}

/// The square of half side `h` about `c` grown by `r` with round corners.
fn rounded_square(c: [f64; 2], h: f64, r: f64) -> Loop {
    let (x0, x1, y0, y1) = (c[0] - h, c[0] + h, c[1] - h, c[1] + h);
    Loop {
        curves: vec![
            seg([x0, y0 - r], [x1, y0 - r]),
            arc([x1, y0], r, -90.0, 90.0),
            seg([x1 + r, y0], [x1 + r, y1]),
            arc([x1, y1], r, 0.0, 90.0),
            seg([x1, y1 + r], [x0, y1 + r]),
            arc([x0, y1], r, 90.0, 90.0),
            seg([x0 - r, y1], [x0 - r, y0]),
            arc([x0, y0], r, 180.0, 90.0),
        ],
    }
}

/// `CreateDraftSweep`: the profile extruded `h`, then its sides drafted by `deg` (negative
/// leans them out: the material grows as it rises).
fn draft_sweep(p: Profile, h: f64, deg: f64, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = one(k.extrude(&p, Extent::Blind(h)).map_err(kerr)?)?;
    let sides: Vec<FaceId> = k
        .get(b)
        .map_err(kerr)?
        .faces
        .iter()
        .enumerate()
        .filter(|(_, f)| !matches!(&f.surf, Surface::Plane { f: fr } if fr.z.z.abs() > 1.0 - 1e-9))
        .map(|(i, _)| FaceId(i as u64))
        .collect();
    let spec = DraftSpec { faces: sides, angle: deg.to_radians(), pull: Unit::new_normalize(V3::z()), neutral: P3::origin(), neutral_normal: None, tangent_propagation: false };
    let d = one(k.draft(b, &spec).map_err(kerr)?)?;
    check_body(&k, d, w)
}

/// `CreateSkinPrimitive`: a solid loft through the sections.
fn skin(sections: Vec<Profile>, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let spec = LoftSpec { body: BodyKind::Solid, sections: sections.into_iter().map(LoftSection::Profile).collect(), start: LoftEnd::default(), end: LoftEnd::default(), source: 0 };
    let b = one(k.loft_with(&spec).map_err(kerr)?)?;
    check_body(&k, b, w)
}

/// SMLib's frame (origin, x, y) as a placement, its x turned `deg` about its z.
fn frame(o: [f64; 3], x: [f64; 3], y: [f64; 3], deg: f64) -> Ax3 {
    let x = V3::from(x).normalize();
    let z = x.cross(&V3::from(y)).normalize();
    let yy = z.cross(&x);
    let a = deg.to_radians();
    Ax3 { origin: P3::from(o), z, x: x * a.cos() + yy * a.sin() }
}

/// `CreateBox(l, w, h, frame)`: the min corner at the frame's origin.
fn boxed(o: [f64; 3], x: [f64; 3], y: [f64; 3], size: [f64; 3], w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = body_of(k.box_on(&frame(o, x, y, 0.0), size[0], size[1], size[2]).map_err(kerr)?)?;
    check_body(&k, b, w)
}

/// SMLib's revolved primitives, from `a0` to `a1` degrees about the frame's z.
enum Prim {
    /// `CreateCone(h, base radius, top radius, ...)`.
    Cone(f64, f64, f64),
    /// `CreateSphere(r, ...)`.
    Sphere(f64),
    /// `CreateTorus(R, r, ...)`.
    Torus(f64, f64),
}

fn primitive(p: Prim, o: [f64; 3], x: [f64; 3], y: [f64; 3], a0: f64, a1: f64, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let ax = frame(o, x, y, a0);
    let s = match p {
        Prim::Cone(h, r0, r1) => k.pcone_a(Some(&ax), r0, r1, h, a1 - a0),
        Prim::Sphere(r) => k.psphere_a(Some(&ax), r, a1 - a0),
        Prim::Torus(big, r) => k.ptorus_a(Some(&ax), big, r, a1 - a0),
    };
    let b = body_of(s.map_err(kerr)?)?;
    check_body(&k, b, w)
}

/// `CreateLinearSweep` / `CreateRotationalSweep` of closed curves (capped): a prism or a revolve.
fn sweep(p: Profile, m: Move, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = m.sweep_profile(&mut k, &p)?;
    check_body(&k, b, w)
}

/// `CreateCurveSweep`: the profile swept along cubic Bézier pieces.
fn curve_sweep(p: Profile, path: &[[[f64; 3]; 4]], w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let spec = SweepSpec {
        body: BodyKind::Solid,
        path: path.iter().map(|b| PathCurve::Bezier3 { poles: b.iter().map(|q| P3::from(*q)).collect(), weights: vec![1.0; 4] }).collect(),
        control: SweepControl::None,
        faces: vec![],
    };
    let b = one(k.sweep_with(&p, &spec).map_err(kerr)?)?;
    check_body(&k, b, w)
}

/// A rectangle (`CreateRectangle`: a corner at `o`, `w` along x, `h` along y) as a face.
fn rect_face(k: &NativeKernel, o: [f64; 3], w: f64, h: f64) -> Result<Shape, String> {
    let pts = [o, [o[0] + w, o[1], o[2]], [o[0] + w, o[1] + h, o[2]], [o[0], o[1] + h, o[2]], o];
    k.mkplane(&k.polyline(&pts).map_err(kerr)?).map_err(kerr)
}

/// A disk (`CreateCircle`) as a face.
fn disk_face(k: &NativeKernel, c: [f64; 3], r: f64) -> Result<Shape, String> {
    k.mkplane(&k.mkedge_full(&k.circle(c[0], c[1], c[2], r))).map_err(kerr)
}

/// The face of `my_test_tsurf_creation` (smTopoTest.cpp:1259) moved by (dx, dy).
fn tsurf_face(k: &NativeKernel, dx: f64, dy: f64) -> Result<Shape, String> {
    let at = |p: [f64; 2]| [p[0] + dx, p[1] + dy, 0.0];
    let outer: Vec<[f64; 3]> = [[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [5.0, 5.0], [5.0, 10.0], [0.0, 10.0], [0.0, 0.0]].into_iter().map(at).collect();
    let tri: Vec<[f64; 3]> = [[1.0, 5.0], [1.0, 8.0], [4.0, 5.0], [1.0, 5.0]].into_iter().map(at).collect();
    let c = at([6.0, 2.5]);
    let f = k.mkplane(&k.polyline(&outer).map_err(kerr)?).map_err(kerr)?;
    let f = k.add_hole(&k.polyline(&tri).map_err(kerr)?, &f).map_err(kerr)?;
    let circ = k.wire(&[&k.mkedge_full(&k.circle(c[0], c[1], c[2], 1.5))]).map_err(kerr)?;
    k.add_hole(&circ, &f).map_err(kerr)
}

/// `Boolean2D`: a boolean of two planar faces into a sheet.
fn sheet_boolean(union: bool, a: fn(&NativeKernel) -> Result<Shape, String>, b: fn(&NativeKernel) -> Result<Shape, String>, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (fa, fb) = (a(&k)?, b(&k)?);
    let r = if union { k.bfuse(&fa, &fb) } else { k.bcut(&fa, &fb) };
    let s = body_of(r.map_err(kerr)?)?;
    check_body(&k, s, w)
}

/// SMLib's `CreateCylindricalBox` (SmPrimitiveCreation.cpp:424): the piece of a thick-walled
/// cylinder about the frame's z between two radii, a length and two angles, refusing bad
/// dimensions. cadrs has no such primitive: `None` until one is wired in here.
fn cylindrical_box(_k: &mut NativeKernel, _len: f64, _r_in: f64, _r_out: f64, _a0: f64, _a1: f64) -> Option<Result<BodyId, KernelError>> {
    None
}

/// A `CreateCylindricalBox` check: built and as `w` when `ok`, refused otherwise.
fn cyl_box(len: f64, r_in: f64, r_out: f64, a0: f64, a1: f64, ok: bool, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    match cylindrical_box(&mut k, len, r_in, r_out, a0, a1) {
        None => Err("cadrs has no cylindrical-box primitive".into()),
        Some(Ok(b)) if ok => check_body(&k, b, w),
        Some(Ok(_)) => Err("made a box from dimensions SMLib refuses".into()),
        Some(Err(e)) if ok => Err(kerr(e)),
        Some(Err(_)) => Ok(()),
    }
}
'''


def f(x):
    return rust_f64(x)


def f2(p):
    return "[%s, %s]" % (f(p[0]), f(p[1]))


def f3(v):
    return "[%s]" % ", ".join(f(x) for x in v)


def want(kind="SOLID", **kw):
    fields = []
    for k in ("faces", "edges"):
        if kw.get(k) is not None:
            fields.append("%s: Some(%d)" % (k, kw[k]))
    for k in ("volume", "area", "length"):
        if kw.get(k) is not None:
            fields.append("%s: Some(%s)" % (k, f(kw[k])))
    return "&Want { %s }" % ", ".join(fields + ["..%s" % kind]) if fields else "&%s" % kind


def fl(v):
    """A vector for a doc comment."""
    return "(%s)" % ", ".join("%g" % float(x) for x in v)


def nums(s):
    return [float(x.strip().rstrip("fF")) for x in s.split(",")]


def point_args(s):
    return [nums(p) for p in re.findall(r"SmPoint3d\s*\(([^)]*)\)", s)]


def curves(code):
    """{name: ("line", a, b) | ("circle", centre, r)} of the block's B-spline curves."""
    out = {}
    for m in re.finditer(r"CreateLineSegment\s*\(\s*crContext\s*,\s*3\s*,(.*?),\s*(\w+)\s*\)\s*\)", code, re.S):
        a, b = point_args(m.group(1))
        out[m.group(2)] = ("line", a, b)
    for m in re.finditer(r"(\w+)\.SetCanonical\s*\(\s*SmPoint3d\s*\(([^)]*)\)", code):
        frame, c = m.group(1), nums(m.group(2))
        cm = re.search(r"CreateCircleSegment\s*\(\s*crContext\s*,\s*3\s*,\s*%s\s*,\s*([\d.]+)\s*,.*?,\s*(\w+)\s*\)\s*\)" % frame, code, re.S)
        out[cm.group(2)] = ("circle", c, float(cm.group(1)))
    return out


def added(code, arr="s3DCurves"):
    return re.findall(r"%s\.Add\s*\(\s*(\w+)\s*\)" % arr, code)


def loops(cs):
    """Closed polygons (chained lines, in z = 0) and circles."""
    lines = [(tuple(c[1][:2]), tuple(c[2][:2])) for c in cs if c[0] == "line"]
    polys = []
    while lines:
        a, b = lines.pop(0)
        poly = [a, b]
        while poly[-1] != poly[0]:
            k = next(i for i, (p, q) in enumerate(lines) if poly[-1] in (p, q))
            p, q = lines.pop(k)
            poly.append(q if p == poly[-1] else p)
        polys.append(poly[:-1])
    circles = [(tuple(c[1][:2]), c[2]) for c in cs if c[0] == "circle"]
    return polys, circles


def region_of(cs):
    """The geometry (`_geom.Region`) and the Rust region of closed curves: the largest polygon
    outside, the rest holes."""
    polys, circles = loops(cs)
    polys.sort(key=lambda p: -abs(g.poly_area_centroid(p)[0]))
    reg = g.Region(polys[0], polys[1:], circles)
    holes = ["polygon(&[%s])" % ", ".join(f2(p) for p in h) for h in polys[1:]] + ["circle(%s, %s)" % (f2(c), f(r)) for c, r in circles]
    rust = "region(polygon(&[%s]), vec![%s])" % (", ".join(f2(p) for p in polys[0]), ", ".join(holes))
    return reg, rust


def line_of(src, needle, start=0):
    return src[:src.index(needle, start)].count("\n") + 1


def counts_doc(vc):
    if not vc:
        return "SMLib asserts no counts"
    return "SMLib asserts %d faces, %d edges, %d vertices (%d wire, %d lamina, %d manifold edges)" % tuple(vc[:6])


def validate_counts(code):
    m = re.findall(r"(\w+)->ValidateCounts\s*\(([^)]*)\)", code)
    return [(b, tuple(int(x) for x in a.split(","))) for b, a in m]


def draft_case(src, fn_start):
    """The dead `if (FALSE)` block (`CreateDraftSweep`), never run by SMLib."""
    i = src.index("if(FALSE)", fn_start)
    code = strip_comments(src[i:src.index("switch(nTest)", i)])
    cs = curves(code)
    reg, rust = region_of([cs[n] for n in added(code)])
    m = re.search(r"CreateDraftSweep\(\s*s3DCurves\s*,\s*sDir\s*,\s*([-\d.]+)\s*,\s*([-\d.]+)", code)
    h, deg = float(m.group(1)), float(m.group(2))
    # The material grows by z·tan|deg| at height z: the outer square widens, the holes narrow
    # (the triangle about its incentre, the circles in radius).
    t = math.tan(math.radians(abs(deg)))
    side = math.sqrt(abs(g.poly_area_centroid(reg.outer)[0]))
    tri = reg.poly_holes[0]
    ta = abs(g.poly_area_centroid(tri)[0])
    tp = sum(l for l, _ in g.poly_edges(tri))
    ri = 2 * ta / tp
    circ = reg.circle_holes

    def area_at(z):
        d = z * t
        return (side + 2 * d) ** 2 - ta * ((ri - d) / ri) ** 2 - sum(math.pi * (r - d) ** 2 for _, r in circ)

    def perim_at(z):
        d = z * t
        return 4 * (side + 2 * d) + tp * (ri - d) / ri + sum(2 * math.pi * (r - d) for _, r in circ)
    vol = g.gauss(lambda zs: np.array([area_at(z) for z in zs]), 0.0, h, panels=4, order=6)
    area = area_at(0) + area_at(h) + g.gauss(lambda zs: np.array([perim_at(z) for z in zs]), 0.0, h, panels=4, order=6) / math.cos(math.radians(deg))
    vc = validate_counts(code)
    doc = ("SMLib primitive: `CreateDraftSweep` of a 5 square less a triangle and two circles, %g high, drafted %g° (smSweepTest.cpp:%d: a dead `if (FALSE)` block, never run; "
           "its counts, %s, are case 3's). cadrs: the extrude with its sides drafted out (negative: the material grows as it rises).") % (
        h, deg, line_of(src, "if(FALSE)", fn_start), counts_doc(vc[0][1]))
    return Case("prim_00_draft_sweep", doc, "draft_sweep(top(vec![%s]), %s, %s, %s)" % (rust, f(h), f(deg), want("SOLID", volume=vol, area=area)))


def rotate(v, axis, ang):
    """v turned by ang radians about the unit axis (Rodrigues)."""
    v, k = np.array(v, float), np.array(axis, float) / np.linalg.norm(axis)
    return v * math.cos(ang) + np.cross(k, v) * math.sin(ang) + k * np.dot(k, v) * (1 - math.cos(ang))


def box_cases(code, line):
    m = re.findall(r"CreateBox\(\s*([^,]+),\s*([^,]+),\s*([^,]+),", code)
    l, w, h = nums(",".join(m[0]))
    vol, area = l * w * h, 2 * (l * w + w * h + h * l)
    vc = dict(validate_counts(code))
    out = []
    t1 = nums(re.findall(r"sRefFrame\.Translate\(\s*SmVector3d\(([^)]*)\)", code)[0])
    out.append(Case("prim_02_box", "SMLib primitive 2: `CreateBox(%g, %g, %g)` at %s (smSweepTest.cpp:%d)." % (l, w, h, fl(t1), line),
                    "boxed(%s, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], %s, %s)" % (f3(t1), f3((l, w, h)), want("SOLID", faces=6, edges=12, volume=vol, area=area))))
    t2 = nums(re.findall(r"sRefFrame\.Translate\(\s*SmVector3d\(([^)]*)\)", code)[1])
    rot = re.search(r"RotateAboutAxisAtPoint\(\s*([\d.]+)\s*,.*?SmPoint3d\(([^)]*)\)", code, re.S)
    ang, at = float(rot.group(1)), nums(rot.group(2))
    ax = nums(re.search(r"SmVector3d\s+sAx\(([^)]*)\)", code).group(1))
    o = np.array(t1) + np.array(t2)
    o = np.array(at) + rotate(o - np.array(at), ax, ang)
    x, y = rotate([1, 0, 0], ax, ang), rotate([0, 1, 0], ax, ang)
    out.append(Case("prim_02_box_turned", "SMLib primitive 2: the second `CreateBox(%g, %g, %g)`, its frame moved to %s and turned %g rad about %s there (smSweepTest.cpp:%d); %s." % (
        l, w, h, fl(o), ang, fl(ax), line, counts_doc(vc.get("pBrep"))),
        "boxed(%s, %s, %s, %s, %s)" % (f3(o), f3(x), f3(y), f3((l, w, h)), want("SOLID", faces=6, edges=12, volume=vol, area=area))))
    # The rectangle less the circle (`Boolean2D` SM_2D_DIFFERENCE).
    rect = re.search(r"CreateRectangle\(\s*crContext\s*,\s*\w+\s*,\s*([\d.]+)\s*,\s*([\d.]+)", code)
    rw, rh = float(rect.group(1)), float(rect.group(2))
    tr = re.findall(r"sRefFrame2\.Translate\(\s*SmVector3d\(([^)]*)\)", code)
    ro = nums(tr[0])
    cc = list(np.array(ro) + np.array(nums(tr[1])))
    cr = float(re.search(r"CreateCircle\(\s*crContext\s*,\s*\w+\s*,\s*([\d.]+)", code).group(1))
    # The disk takes the rectangle's corner (cc) and the strip below it: ∫ (√(r² − u²) − 2) du
    # over 0 ≤ u ≤ √(r² − 2²) (the rectangle's top is 2 below the centre).
    gap = cc[1] - (ro[1] + rh)
    um = math.sqrt(cr * cr - gap * gap)
    cut = 0.5 * (um * math.sqrt(cr * cr - um * um) + cr * cr * math.asin(um / cr)) - gap * um
    assert abs(cc[0] - (ro[0] + rw)) < 1e-12 and cc[1] - ro[1] >= cr and gap > 0
    out.append(Case("prim_02_rectangle_less_circle", "SMLib primitive 2: `Boolean2D` difference of `CreateRectangle(%g, %g)` at %s and `CreateCircle(%g)` at %s (smSweepTest.cpp:%d): a sheet of one face, 5 edges. cadrs: a sheet boolean of two planar faces (Phase 1b)." % (
        rw, rh, fl(ro), cr, fl(cc), line),
        "sheet_boolean(false, |k| rect_face(k, %s, %s, %s), |k| disk_face(k, %s, %s), %s)" % (f3(ro), f(rw), f(rh), f3(cc), f(cr), want("SHEET", faces=1, edges=5, area=rw * rh - cut))))
    mv = nums(re.search(r"sTranslate\.Translate\(\s*SmVector3d\(([^)]*)\)", code).group(1))
    vc1 = vc.get("pBrep1")
    out.append(Case("prim_02_tsurf_union", "SMLib primitive 2: `Boolean2D` union of `my_test_tsurf_creation`'s face and a copy moved by %s (smSweepTest.cpp:%d); %s, one face. cadrs: a sheet boolean of two planar faces (Phase 1b)." % (
        fl(mv), line, counts_doc(vc1)),
        "sheet_boolean(true, |k| tsurf_face(k, 0.0, 0.0), |k| tsurf_face(k, %s, %s), %s)" % (f(mv[0]), f(mv[1]), want("SHEET", faces=vc1[0], edges=vc1[1], area=g.tsurf_union_area(mv[0], mv[1])))))
    return out


def frame_of(code, name):
    m = re.search(r"%s\.SetCanonical\(\s*SmPoint3d\(([^)]*)\)\s*,\s*SmVector3d\(([^)]*)\)\s*,\s*SmVector3d\(([^)]*)\)" % name, code, re.S)
    return nums(m.group(1)), nums(m.group(2)), nums(m.group(3))


def revolved_cases(n, code, line):
    """Cases 5-7: two cones, spheres or tori, a whole one and a piece."""
    out = []
    for m in re.finditer(r"(\w+)\.(CreateCone|CreateSphere|CreateTorus)\(([^;]*?)(\w+)\s*\)\s*\)", code, re.S):
        args = [a for a in (x.strip() for x in re.sub(r"//[^\n]*", "", m.group(3)).split(",")) if a]
        frm = frame_of(code, m.group(4))
        kind = m.group(2)
        if kind == "CreateCone":
            h, r0, r1, a0, a1 = nums(",".join(args[:5]))
            th = math.radians(a1 - a0)
            slant = math.hypot(r1 - r0, h)
            vol = th / 2 * h / 3 * (r0 * r0 + r0 * r1 + r1 * r1)
            area = th / 2 * (r0 * r0 + r1 * r1) + th / 2 * (r0 + r1) * slant + (0 if a1 - a0 >= 360 else 2 * h * (r0 + r1) / 2)
            prim = "Prim::Cone(%s, %s, %s)" % (f(h), f(r0), f(r1))
            what = "`CreateCone(%g, %g, %g)`" % (h, r0, r1)
        elif kind == "CreateSphere":
            r, a0, a1 = nums(",".join(args[:3]))
            frac = (a1 - a0) / 360
            vol, area = frac * 4 / 3 * math.pi * r ** 3, frac * 4 * math.pi * r * r + (0 if frac >= 1 else math.pi * r * r)
            prim = "Prim::Sphere(%s)" % f(r)
            what = "`CreateSphere(%g)`" % r
        else:
            big, r, a0, a1 = nums(",".join(args[:4]))
            frac = (a1 - a0) / 360
            vol, area = frac * 2 * math.pi ** 2 * big * r * r, frac * 4 * math.pi ** 2 * big * r + (0 if frac >= 1 else 2 * math.pi * r * r)
            prim = "Prim::Torus(%s, %s)" % (f(big), f(r))
            what = "`CreateTorus(%g, %g)`" % (big, r)
        whole = a1 - a0 >= 360
        name = "prim_%02d_%s_%s" % (n, kind[6:].lower(), "whole" if whole else "%g_%g" % (a0, a1))
        doc = "SMLib primitive %d: %s from %g° to %g° in the frame at %s, x %s, y %s (smSweepTest.cpp:%d; SMLib asserts no counts)." % (n, what, a0, a1, fl(frm[0]), fl(frm[1]), fl(frm[2]), line)
        out.append(Case(name, doc, "primitive(%s, %s, %s, %s, %s, %s, %s)" % (prim, f3(frm[0]), f3(frm[1]), f3(frm[2]), f(a0), f(a1), want("SOLID", volume=vol, area=area))))
    return out


def curve_sweep_case(code, line):
    """Case 8: `OffsetProfile` (LEFT 0.8, round corners, as a shell) then `CreateCurveSweep`
    along a cubic B-spline. The offset profile is rebuilt from what SMLib's counts say it is
    (16 faces, 42 edges, 28 vertices = a square ring grown 0.8 with round corners, 12 sides, and
    the ring inside the first circle, 2 sides; the second circle's ring dropped)."""
    cs = curves(code)
    order = added(code)
    sq = [cs[n] for n in order if cs[n][0] == "line"]
    pts = [tuple(c[1][:2]) for c in sq] + [tuple(c[2][:2]) for c in sq]
    xs, ys = [p[0] for p in pts], [p[1] for p in pts]
    c0 = ((min(xs) + max(xs)) / 2, (min(ys) + max(ys)) / 2)
    h = (max(xs) - min(xs)) / 2
    off = float(re.search(r"OffsetProfile\(\s*crContext\s*,\s*s3DCurves\s*,\s*[\d.e+-]+\s*,\s*([\d.]+)", code).group(1))
    circ = [cs[n] for n in order if cs[n][0] == "circle"][0]
    cc, cr = tuple(circ[1][:2]), circ[2]
    a1 = (2 * h + 2 * off) ** 2 - (4 - math.pi) * off * off - (2 * h) ** 2
    a2 = math.pi * (cr * cr - (cr - off) ** 2)
    cen = ((a1 * c0[0] + a2 * cc[0]) / (a1 + a2), (a1 * c0[1] + a2 * cc[1]) / (a1 + a2))
    sh = lambda p: (p[0] - cen[0], p[1] - cen[1])
    # The path: my_create_nurb (smCurveTest.cpp:244) of the points moved by its translation,
    # degree 3, knots i/4 (sic), then scaled by 10.
    pm = re.search(r"my_create_nurb\(\s*crContext\s*,\s*SmPoint3d\(([^)]*)\)\s*,\s*sPnts1\s*,\s*(\d+)", code)
    tr, deg = nums(pm.group(1)), int(pm.group(2))
    sc = nums(re.search(r"SmVector3d\s+sScale\(([^)]*)\)", code).group(1))
    ctrl = [[(p[i] + tr[i]) * sc[i] for i in range(3)] for p in point_args(code[code.index("sPnts1.Add"):code.index("my_create_nurb")])]
    nk = len(ctrl) - deg + 1
    knots = [0.0] * (deg + 1) + [i / nk for i in range(1, nk - 1)] + [1.0] * (deg + 1)
    bez = g.bspline_to_bezier(deg, knots, ctrl)
    length = sum(g.bezier_length(b) for b in bez)
    p0, tan = bez[0][0], bez[0][1] - bez[0][0]
    nrm = tan / np.linalg.norm(tan)
    x = np.array([1.0, 0, 0]) - nrm * nrm[0]
    x /= np.linalg.norm(x)
    y = np.cross(nrm, x)
    prof = "Profile::new(plane(%s, %s, %s), vec![region(rounded_square(%s, %s, %s), vec![polygon(&[%s])]), region(circle(%s, %s), vec![circle(%s, %s)])])" % (
        f3(p0), f3(x), f3(y), f2(sh(c0)), f(h), f(off),
        ", ".join(f2(sh(p)) for p in [(c0[0] - h, c0[1] - h), (c0[0] + h, c0[1] - h), (c0[0] + h, c0[1] + h), (c0[0] - h, c0[1] + h)]),
        f2(sh(cc)), f(cr), f2(sh(cc)), f(cr - off))
    path = "&[%s]" % ", ".join("[%s]" % ", ".join(f3(q) for q in b) for b in bez)
    vc = validate_counts(code)
    doc = ("SMLib primitive 8: `CreateCurveSweep` of an `OffsetProfile` along a cubic B-spline (smSweepTest.cpp:%d); %s. cadrs: the profile rebuilt from those counts (a %g square ring "
           "grown %g with round corners, and the ring inside the r %g circle; SMLib drops the other circle's ring), its centroid on the path's start and square to it, swept: "
           "V = A · L (the path's length, %.6f). Its counts (4 caps, SMLib's 16 faces have 2) aren't compared.") % (line, counts_doc(vc[0][1]), 2 * h, off, cr, length)
    return Case("prim_08_curve_sweep", doc, "curve_sweep(%s, %s, %s)" % (prof, path, want("SOLID", volume=(a1 + a2) * length)))


def extract(ctx):
    src = ctx.source(SOURCE)
    start = re.search(r"SmStatus my_primsol_demo\s*\(", src).start()
    end = src.index("// end my_primsol_demo\n", start)
    cases = [draft_case(src, start)]
    sw = src.index("switch(nTest)", start)
    blocks = dict(case_blocks(src[sw:end]))
    for n in sorted(blocks):
        code = blocks[n]
        line = line_of(src, "case %d:" % n, sw)
        vc = validate_counts(code)
        if "CreateSkinPrimitive" in code:
            cs = curves(code)
            names = added(code)
            counts = [int(x) for x in re.findall(r"sCounts\.Add\(\s*(\d+)\s*\)", code)]
            secs, i = [], 0
            for c in counts:
                group = [cs[x] for x in names[i:i + c]]
                i += c
                z = group[0][1][2]
                if group[0][0] == "circle":
                    reg = "region(circle(%s, %s), vec![])" % (f2(group[0][1]), f(group[0][2]))
                else:
                    polys, _ = loops([(t, a, b) for t, a, b in group])
                    reg = "region(polygon(&[%s]), vec![])" % ", ".join(f2(p) for p in polys[0])
                secs.append("Profile::new(plane(%s, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), vec![%s])" % (f3((0, 0, z)), reg))
            doc = ("SMLib primitive 1: `CreateSkinPrimitive` through a circle, a square and a circle (smSweepTest.cpp:%d; its counts are commented out). cadrs: a solid loft; the "
                   "skin's surface differs between kernels, so only a valid solid is checked.") % line
            cases.append(Case("prim_01_skin", doc, "skin(vec![%s], &SOLID)" % ", ".join(secs)))
        elif "CreateBox" in code:
            cases += box_cases(code, line)
        elif "CreateLinearSweep" in code:
            cs = curves(code)
            reg, rust = region_of([cs[x] for x in added(code)])
            m = re.search(r"SmVector3d\s+sDir\(([^)]*)\)", code)
            d = np.array(nums(m.group(1)))
            dist = float(re.search(r"CreateLinearSweep\(\s*s3DCurves\s*,\s*sDir\s*,\s*([\d.]+)", code).group(1))
            v = d / np.linalg.norm(d) * dist
            vol, area = g.oblique_prism(reg, v)
            doc = "SMLib primitive 3: `CreateLinearSweep` (capped) of a 5 square less a triangle and two circles along %s · %g (smSweepTest.cpp:%d); %s. cadrs: a prism (counts not compared: the circles' sides are closed faces)." % (
                fl(d), dist, line, counts_doc(vc[0][1]))
            cases.append(Case("prim_03_linear_sweep", doc, "sweep(top(vec![%s]), Move::Trans(%s), %s)" % (rust, f3(v), want("SOLID", volume=vol, area=area))))
        elif "CreateRotationalSweep" in code:
            cs = curves(code)
            reg, rust = region_of([cs[x] for x in added(code)])
            m = re.search(r"CreateRotationalSweep\(\s*s3DCurves\s*,\s*SmPoint3d\(([^)]*)\)\s*,.*?SmVector3d\(([^)]*)\)\s*,.*?([\d.]+)\s*,", code, re.S)
            o, d, deg = nums(m.group(1)), nums(m.group(2)), float(m.group(3))
            vol, area = g.revolution(reg, o, d, deg)
            doc = "SMLib primitive 4: `CreateRotationalSweep` (capped) of a 5 square less a triangle, %g° about the axis through %s along %s (smSweepTest.cpp:%d; one side of the square on the axis); %s." % (
                deg, o, d, line, counts_doc(vc[0][1]))
            cases.append(Case("prim_04_rotational_sweep", doc, "sweep(top(vec![%s]), Move::Rot { o: %s, d: %s, deg: %s }, %s)" % (
                rust, f3(o), f3(d), f(deg), want("SOLID", faces=vc[0][1][0], edges=vc[0][1][1], volume=vol, area=area))))
        elif re.search(r"Create(Cone|Sphere|Torus)\(", code):
            cases += revolved_cases(n, code, line)
        elif "CreateCurveSweep" in code:
            cases.append(curve_sweep_case(code, line))
    # CreateCylindricalBox (my_primsol_regression).
    rows = re.findall(r"\{\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*(SM_\w+)\s*\}\s*,?\s*(?://\s*([^\n]*))?", src[end:])
    line = line_of(src, "CreateCylindricalBox rejects", end)
    for i, (ln, ri, ro, a0, a1, st, why) in enumerate(rows):
        ln, ri, ro, a0, a1 = map(float, (ln, ri, ro, a0, a1))
        ok = st == "SM_SUCCESS"
        th = math.radians(a1 - a0)
        w = want("SOLID", volume=th / 2 * (ro * ro - ri * ri) * ln, area=th * (ro * ro - ri * ri) + th * (ro + ri) * ln + 2 * (ro - ri) * ln) if ok else "&SOLID"
        doc = "SMLib primitive: `CreateCylindricalBox(%g, %g, %g, %g, %g)` (smSweepTest.cpp:%d, case %d) %s. cadrs has no cylindrical-box primitive (missing)." % (
            ln, ri, ro, a0, a1, line, i, "is made" if ok else "is refused (%s)" % why.strip())
        cases.append(Case("cylindrical_box_%d" % i, doc, "cyl_box(%s, %s, %s, %s, %s, %s, %s)" % (f(ln), f(ri), f(ro), f(a0), f(a1), "true" if ok else "false", w)))
    return cases
