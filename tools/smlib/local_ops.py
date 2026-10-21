"""SMLib's local-operation tests (tests/prog_test/src/smFilletTest.cpp, `my_local_op`,
smFilletTest.cpp:3518): `SmBrep::LocalOperation` gives one face a new surface and rebuilds the
edges and vertices around it. Here that is `native::local::replace_surfaces` (the `Kernel` trait
has no replace-face operation): the face is picked by its centroid, the new surface built as the
test builds it, and the face keeps the side it faced (SMLib keeps the face's use).

SMLib's test asserts nothing but success, and marks lCount 3 and 5 "needs fixing". Those two
check closed-form volume and area (SMLib's own lCount 3 isn't a solid; its lCount 5 agrees with
the closed form to 3e-6); lCount 1 and 7 check the oracle's.
"""
import json
import math
import os
import re
import subprocess
import sys

from smlib_to_rust import Case, Fixture, drop_dead, if_blocks, num, rust_f64, strip_comments

SOURCE = "smFilletTest.cpp"
TITLE = "local operation"

NOT_CONVERTED = [
    ("lCount 0, 2, 4, 6", "only read and draw a model"),
]

PRELUDE = r'''
use cadrs_kernel::native::local::replace_surfaces;
use cadrs_kernel::native::math::{Frame, P3, V3};
use cadrs_kernel::native::smb;
use cadrs_kernel::native::surface::Surface;

const SUITE: &str = "local_ops";

/// A face's new surface, placed as SMLib's test places it (`SmAxis2Placement::SetCanonical(o,
/// x, y)`: the plane's normal or the cylinder's axis is x × y, u = 0 along x).
enum New {
    Plane([f64; 3], [f64; 3], [f64; 3]),
    Cylinder([f64; 3], [f64; 3], [f64; 3], f64),
    /// A B-spline surface file of the suite (`.sms`).
    Sms(&'static str),
}

fn frame(o: [f64; 3], x: [f64; 3], y: [f64; 3]) -> Frame {
    let (x, y) = (V3::new(x[0], x[1], x[2]), V3::new(y[0], y[1], y[2]));
    Frame::from_xz(P3::new(o[0], o[1], o[2]), x, x.cross(&y))
}

/// The outward normal of `body`'s face `f` near `p`.
fn outward(body: &cadrs_kernel::native::topo::Body, f: usize, p: &P3) -> V3 {
    let face = &body.faces[f];
    let (u, v) = face.surf.project(p, None);
    face.surf.normal(u, v) * if face.sense { 1.0 } else { -1.0 }
}

/// SMLib's `SmBrep::LocalOperation`: face `face` (SMLib's index) of `file` given the surface
/// `new`, then checked.
fn local_op(file: &str, face: usize, new: New, e: &Expect) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let [id] = faces(&m, &[face])?[..] else { return Err("no face".into()) };
    let fi = id.0 as usize;
    let surf = match new {
        New::Plane(o, x, y) => Surface::Plane { f: frame(o, x, y) },
        New::Cylinder(o, x, y, r) => Surface::Cylinder { f: frame(o, x, y), r },
        New::Sms(s) => smb::read_surface(&std::fs::read(data(SUITE, s)).map_err(|e| format!("{s}: {e}"))?)?,
    };
    let mut surfs: Vec<Surface> = m.body.faces.iter().map(|f| f.surf.clone()).collect();
    surfs[fi] = surf;
    let mut out = replace_surfaces(&m.body, &surfs).map_err(|e| format!("{e:?}"))?;
    // The face faces the side it faced before (SMLib keeps its face use).
    let before = outward(&m.body, fi, &m.face_centroids[face].ok_or("no centroid")?);
    let at = smb::face_centroids(&out)[fi];
    out.faces[fi].sense = true;
    out.faces[fi].sense = outward(&out, fi, &at).dot(&before) > 0.0;
    check_result(&m.body, &out, e)
}
'''

VARS = {"sX": (1.0, 0.0, 0.0), "sY": (0.0, 1.0, 0.0), "sZ": (0.0, 0.0, 1.0)}

# Closed forms where SMLib's own result is wrong (its test: "needs fixing"). box.smb is the cube
# [0, 10]³, face 1 its top (z = 10); cylinder.smb runs along x from 0 to 10, radius 5, face 1
# its end disc at x = 0.
CLOSED = {
    # The top becomes the half cylinder of radius 5 about y through (5, 0, 12) that the test's
    # domain (u 0–180° from −x towards +z) names: a 10 × 10 × 12 block with a half round on top.
    3: ("the 10 × 10 × 12 block under a half round of radius 5 and length 10",
        1200.0 + 125.0 * math.pi, 580.0 + 75.0 * math.pi),
    # The end disc at x = 0 becomes the plane x = −3: the cylinder 3 longer.
    5: ("the cylinder made 3 longer: radius 5, length 13",
        325.0 * math.pi, 180.0 * math.pi),
}


def vec(expr, local):
    """A vector expression of the test: `sX`, `-sZ`, or a local vector."""
    expr = expr.strip()
    sign = -1.0 if expr.startswith("-") else 1.0
    v = local.get(expr.lstrip("-").strip()) or VARS[expr.lstrip("-").strip()]
    return tuple(sign * x + 0.0 for x in v)


def nums(s):
    return [num(x) for x in s.split(",")]


def function(src, name):
    code = drop_dead(strip_comments(src))
    m = re.search(r"\bSmStatus\s+%s\s*\([^)]*\)\s*\{" % name, code)
    depth, j = 0, m.end() - 1
    while j < len(code):
        depth += {"{": 1, "}": -1}.get(code[j], 0)
        j += 1
        if depth == 0:
            break
    return code[m.end():j - 1]


def oracle_run(args):
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    if not ob or not os.path.exists(ob):
        return None
    r = subprocess.run([ob] + args, capture_output=True, text=True, timeout=600)
    lines = [l for l in r.stdout.splitlines() if l.startswith("{")]
    return json.loads(lines[-1]) if lines else {"ok": False, "status": "no output (exit %d)" % r.returncode}


def rarr(v):
    return "[%s]" % ", ".join(rust_f64(x) for x in v)


def fmt(v):
    return ",".join("%r" % x for x in v)


def block_case(ctx, src, n, b):
    if "LocalOperation" not in b:
        return None
    path = re.search(r'ReadFromFile\(\s*crContext\s*,\s*_T\(\s*"\.\./\.\./([^"]+\.smb)"\s*\)', b).group(1)
    face = int(re.search(r"sFaces\[\s*(\d+)\s*\]", b).group(1))
    fx = Fixture(path)
    local = {}
    for name, a in re.findall(r"SmVector3d\s+(\w+)\s*\(([^)]*)\)", b):
        v = nums(a)
        if re.search(r"\b%s\.Unitize\(\)" % name, b):
            l = math.sqrt(sum(x * x for x in v))
            v = [x / l for x in v]
        local[name] = tuple(v)
    start = src.find("SmStatus my_local_op(")
    line = src[:src.find("if(lCount == %d)" % n, start)].count("\n") + 1
    fixtures = [fx]
    oracle_args = None
    pos = re.search(r"sPos\.SetCanonical\(\s*SmPoint3d\(([^)]*)\)\s*,\s*([-\w]+)\s*,\s*([-\w]+)\s*\)", b)
    dom = None
    lo = re.search(r"SmPoint2d\s+sUVMin\s*(?:=\s*SmPoint2d)?\s*\(([^)]*)\)", b)
    hi = re.search(r"SmPoint2d\s+sUVMax\s*(?:=\s*SmPoint2d)?\s*\(([^)]*)\)", b)
    if lo and hi:
        dom = nums(lo.group(1)) + nums(hi.group(1))
        dom = [dom[0], dom[1], dom[2], dom[3]]
    if "SmPlane::CreateCanonical" in b or "SmCylinder::CreateCanonical" in b:
        o = nums(pos.group(1))
        x, y = vec(pos.group(2), local), vec(pos.group(3), local)
        z = (x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0])
        if "SmPlane::CreateCanonical" in b:
            new = "New::Plane(%s, %s, %s)" % (rarr(o), rarr(x), rarr(y))
            what = "the plane through %s with normal %s" % (tuple(o), tuple(round(c, 6) for c in z))
            oracle_args = [fx, face, "plane"] + o + list(z)
        else:
            r = num(re.search(r"SmCylinder::CreateCanonical\(\s*crContext\s*,\s*sPos\s*,\s*([0-9.eE+-]+)", b).group(1))
            new = "New::Cylinder(%s, %s, %s, %s)" % (rarr(o), rarr(x), rarr(y), rust_f64(r))
            what = "the cylinder of radius %s about %s through %s" % (r, tuple(z), tuple(o))
            oracle_args = [fx, face, "cyl"] + o + list(z) + [r]
        oracle_args += ["--xdir=%s" % fmt(x)] + (["--dom=%s" % fmt(dom)] if dom else [])
    elif "SmBSplineSurface::ReadFromFile" in b:
        sms = re.search(r'SmBSplineSurface::ReadFromFile\(\s*crContext\s*,\s*_T\(\s*"\.\./\.\./([^"]+\.sms)"\s*\)', b).group(1)
        sf = Fixture(sms)
        fixtures.append(sf)
        new = 'New::Sms("%s")' % sf.name
        what = "the B-spline surface `%s`" % sf.name
        oracle_args = [fx, face, "sms", sf]
    else:
        print("local_ops: block %d not converted" % n, file=sys.stderr)
        return None
    doc = "SMLib local operation %d (smFilletTest.cpp:%d): face %d of `%s` becomes %s." % (n, line, face, fx.name, what)
    name = "local_op_%d_%s" % (n, os.path.splitext(fx.name)[0].lower())
    call = 'local_op("%s", %d, %s, {expect})' % (fx.name, face, new)
    if n in CLOSED:
        why, v, a = CLOSED[n]
        c = Case(name, doc + " Closed form: %s." % why, call, fixtures, {"volume": v, "area": a})
        args = [ctx.path(x.rel) if isinstance(x, Fixture) else (repr(x) if isinstance(x, float) else str(x)) for x in oracle_args]
        r = oracle_run(["replace"] + args)
        if r is not None:
            c.note = ("SMLib (the oracle) gives volume %s, area %s, %s faces, %s edges%s" % (
                r.get("volume"), r.get("area"), r.get("faces"), r.get("edges"), "" if r.get("solid") else ", not a solid")
                      if r.get("ok") else "SMLib (the oracle) fails: %s" % r.get("status"))
        return c
    return Case(name, doc, call, fixtures, {}, ("replace", oracle_args))


def extract(ctx):
    src = ctx.source(SOURCE)
    body = function(src, "my_local_op")
    cases = []
    for n, b in if_blocks(body):
        c = block_case(ctx, src, n, b)
        if c is not None:
            cases.append(c)
    return cases
