"""SMLib's sweep tests (tests/prog_test/src/smSweepTest.cpp, the `sweeps` suite of prog_test.cpp,
run by `my_sweep_regression`): `my_sweep_demo`'s 34 cases (`SmTopologySweep::DoSweep`,
translational or rotational) and `my_test_dosweep_null_region`.

SMLib sweeps pieces of one non-manifold B-rep: faces into solids, edges into faces, vertices
into edges ("higher"), or copies them to the swept position ("same"); with no pieces named the
whole B-rep. Each converted case builds the cadrs equivalent of what is swept "higher":
- a face swept into a solid: the face as a profile, extruded, swept as a prism or revolved;
- edges swept into faces: a surface extrude of the edges (sheets);
- vertices swept into edges: the draw harness's `prism`/`revol` of a vertex. cadrs has no wire
  bodies yet (Phase 1b), so these cases are known failures;
- a whole solid swept (cases 25-34): the union of the solid at both ends and each face's sweep
  (a sphere's sweep about an axis: its disk through the axis revolved, with the ball at each
  end). A solid S moved by M_t, t in [0, 1], sweeps S_0 ∪ S_1 ∪ the sweeps of its faces.

SMLib's own swept volume is 0 (it leaves the swept region void) and its asserted counts are of
its cell complexes, so volumes, areas and lengths come from closed forms (`_geom.py`: Pappus,
zonotopes, numeric integrals). Counts are checked only where the result is all planes and is
the same as SMLib's (or derived here, said in the case).
"""
import math
import re

from smlib import _geom as g
from smlib_to_rust import Case, case_blocks, rust_f64

SOURCE = "smSweepTest.cpp"
TITLE = "sweep"

NOT_CONVERTED = [
    ("my_sweep_demo 0 (the `default` block)", "it only makes the test B-rep, sweeps nothing"),
    ("2, 4, 6, 9, 12, 13, 15, 16, 17, 18",
     "only \"same\" targets: SMLib copies the vertices, edges or faces to their swept positions (a transform, no swept geometry); the counts are of its cell complex of the copies"),
    ("my_test_dosweep_null_region: explicit against NULL region",
     "the two calls differ only in SMLib's `pRegionToSweepInto` argument (an API contract); the sweeps themselves are converted (null_region_*)"),
]

PRELUDE = r'''
#[path = "smlib_common/kinds.rs"]
mod kinds;
use kinds::*;

use cadrs_kernel::native::draw::{Ax3, Shape};
use cadrs_kernel::native::math::{P3, V3};
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BodyId, BodyKind, BoolOp, Chain, ExtrudeSpec, Kernel, Plane, Profile};
use nalgebra::Unit;

/// The face of SMLib's test B-rep (`my_create_test_brep`, smSweepTest.cpp:192, from
/// `my_test_tsurf_creation`, smTopoTest.cpp:1259): a 10 × 10 square less its top right
/// quarter, with a triangular and a round hole. SMLib splits it in two by an edge from (0,0) to
/// (5,5); `upper` is face 0, the half with the triangle.
fn test_face(upper: bool) -> Profile {
    let tri = polygon(&[[1.0, 5.0], [4.0, 5.0], [1.0, 8.0]]);
    let r = if upper {
        region(polygon(&[[0.0, 0.0], [5.0, 5.0], [5.0, 10.0], [0.0, 10.0]]), vec![tri])
    } else {
        region(polygon(&[[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [5.0, 5.0], [5.0, 10.0], [0.0, 10.0]]), vec![tri, circle([6.0, 2.5], 1.5)])
    };
    Profile::new(Plane::top(), vec![r])
}

/// An open chain of segments in the test B-rep's plane.
fn chain(segs: &[([f64; 2], [f64; 2])]) -> Chain {
    Chain { curves: segs.iter().map(|&(a, b)| seg(a, b)).collect(), source: None }
}

/// A profile's curves swept into sheets (a surface extrude): SMLib's edges swept higher.
fn surface_sweep(k: &mut NativeKernel, p: &Profile, m: Move) -> Result<BodyId, String> {
    let Move::Trans(v) = m else { return Err("SMLib's edge sweeps here are translations".into()) };
    let v = V3::from(v);
    let spec = ExtrudeSpec { body: BodyKind::Surface, ..ExtrudeSpec::blind(Unit::new_normalize(v), v.norm()) };
    one(k.extrude_with(p, &spec).map_err(kerr)?)
}

/// SMLib's vertices swept higher `reps` times: one edge each time.
fn vertices(pts: &[[f64; 3]], m: Move, reps: usize, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let s = point_sweeps(&mut k, pts, m, reps)?;
    check_wire(&k, &s, w)
}

/// SMLib's edges swept higher: the test face's boundary, the edge splitting it and the wire
/// edge, as sheets.
fn edges(m: Move, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let mut p = test_face(false);
    p.chains = vec![chain(&[([0.0, 0.0], [5.0, 5.0])]), chain(&[([10.0, 0.0], [15.0, 2.0])])];
    let b = surface_sweep(&mut k, &p, m)?;
    check_body(&k, b, w)
}

/// SMLib's faces swept higher: the test face (or its face 0) as a solid.
fn faces(upper: bool, m: Move, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = m.sweep_profile(&mut k, &test_face(upper))?;
    check_body(&k, b, w)
}

/// The whole test B-rep swept: its face into a solid (the edge and vertex inside it make no
/// faces or edges, as SMLib's case 8 says), its wire edge into a sheet and its vertex shell into
/// a wire.
fn whole_test_brep(m: Move, solid: &Want, sheet: &Want, wire: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let b = m.sweep_profile(&mut k, &test_face(false))?;
    check_body(&k, b, solid)?;
    let p = Profile { plane: Plane::top(), regions: vec![], chains: vec![chain(&[([10.0, 0.0], [15.0, 2.0])])] };
    let s = surface_sweep(&mut k, &p, m)?;
    check_body(&k, s, sheet)?;
    let wr = point_sweeps(&mut k, &[[-5.0, 0.0, 0.0]], m, 1)?;
    check_wire(&k, &wr, wire)
}

/// A ball (centre `c`, radius `r`) swept about an axis: its disk through the axis revolved,
/// and below a whole turn the ball at both ends.
fn sphere(c: [f64; 3], r: f64, m: Move, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let ax = m.axis().ok_or("a ball is swept about an axis here")?;
    let cv = P3::from(c);
    let d = ax.dir.into_inner();
    let out = (cv - ax.origin) - d * (cv - ax.origin).dot(&d);
    let pl = Plane { origin: cv, x_dir: Unit::new_normalize(out), normal: Unit::new_normalize(out.cross(&d)) };
    let mut pieces = vec![m.sweep_profile(&mut k, &Profile::new(pl, vec![region(circle([0.0, 0.0], r), vec![])]))?];
    if !m.full() {
        let s0 = body_of(k.psphere_on(&Ax3 { origin: cv, z: V3::z(), x: V3::x() }, r).map_err(kerr)?)?;
        let s1 = body_of(m.apply(&mut k, &Shape::Body(s0))?)?;
        pieces.extend([s0, s1]);
    }
    let b = unite(&mut k, &pieces)?;
    check_body(&k, b, w)
}

/// A box's six faces (planar faces of the draw harness).
fn box_faces(k: &NativeKernel, lo: [f64; 3], s: [f64; 3]) -> Result<Vec<Shape>, String> {
    let c = |i: u8, j: u8, l: u8| [lo[0] + s[0] * i as f64, lo[1] + s[1] * j as f64, lo[2] + s[2] * l as f64];
    let quads = [
        [(0, 0, 0), (0, 1, 0), (0, 1, 1), (0, 0, 1)],
        [(1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)],
        [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)],
        [(0, 1, 0), (1, 1, 0), (1, 1, 1), (0, 1, 1)],
        [(0, 0, 0), (1, 0, 0), (1, 1, 0), (0, 1, 0)],
        [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)],
    ];
    quads
        .iter()
        .map(|q| {
            let pts: Vec<[f64; 3]> = q.iter().chain(&q[..1]).map(|&(i, j, l)| c(i, j, l)).collect();
            k.mkplane(&k.polyline(&pts).map_err(kerr)?).map_err(kerr)
        })
        .collect()
}

/// A box (less a box inside it) swept whole: the solid at both ends and every face's sweep
/// (faces the motion slides in their own plane sweep nothing), united.
fn boxes(outer: ([f64; 3], [f64; 3]), cavity: Option<([f64; 3], [f64; 3])>, m: Move, w: &Want) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (lo, s) = outer;
    let mut solid = body_of(k.box_at(lo[0], lo[1], lo[2], s[0], s[1], s[2]).map_err(kerr)?)?;
    let mut fs = box_faces(&k, lo, s)?;
    if let Some((lo, s)) = cavity {
        let inner = body_of(k.box_at(lo[0], lo[1], lo[2], s[0], s[1], s[2]).map_err(kerr)?)?;
        solid = one(k.boolean(BoolOp::Subtract, solid, &[inner]).map_err(kerr)?)?;
        fs.extend(box_faces(&k, lo, s)?);
    }
    let mut pieces = vec![solid];
    if !m.full() {
        pieces.push(body_of(m.apply(&mut k, &Shape::Body(solid))?)?);
    }
    for f in &fs {
        if let (Move::Trans(v), Shape::Face { plane, .. }) = (m, f) {
            let v = V3::from(v);
            if v.dot(&plane.normal).abs() <= 1e-9 * v.norm() {
                continue;
            }
        }
        pieces.push(body_of(m.sweep(&mut k, f)?)?);
    }
    let b = unite(&mut k, &pieces)?;
    check_body(&k, b, w)
}
'''


def f3(v):
    return "[%s]" % ", ".join(rust_f64(x) for x in v)


def fl(v):
    """A vector for a doc comment."""
    return "(%s)" % ", ".join("%g" % float(x) for x in v)


def want(kind="SOLID", **kw):
    fields = []
    for k in ("faces", "edges"):
        if kw.get(k) is not None:
            fields.append("%s: Some(%d)" % (k, kw[k]))
    for k in ("volume", "area", "length"):
        if kw.get(k) is not None:
            fields.append("%s: Some(%s)" % (k, rust_f64(kw[k])))
    return "&Want { %s }" % ", ".join(fields + ["..%s" % kind]) if fields else "&%s" % kind


def move_rust(m):
    if m[0] == "trans":
        return "Move::Trans(%s)" % f3(m[1])
    return "Move::Rot { o: %s, d: %s, deg: %s }" % (f3(m[1]), f3(m[2]), rust_f64(m[3]))


def vec3(code, name):
    m = re.search(r"\b(?:SmVector3d|SmPoint3d)\s+%s\s*\(([^)]*)\)" % name, code)
    return [float(x) for x in m.group(1).split(",")] if m else None


def number(code, name, n):
    """`double name = value;`, a `(nTest == k) ? a : ...` chain evaluated for case n."""
    m = re.search(r"\bdouble\s+%s\s*=\s*([^;]+);" % name, code)
    if not m:
        return None
    e = m.group(1)
    if "?" not in e:
        return float(e)
    for k, v in re.findall(r"\(\s*nTest\s*==\s*(\d+)\s*\)\s*\?\s*([-\d.]+)", e):
        if int(k) == n:
            return float(v)
    return float(re.search(r":\s*([-\d.]+)\s*$", e.strip()).group(1))


def motion(code, n):
    v = vec3(code, "sVec")
    if v:
        return ("trans", v)
    o = vec3(code, "sCenter") if "sRSG(sCenter" in code.replace(" ", "") else vec3(code, "sBase")
    return ("rot", o, vec3(code, "sAxis"), number(code, "dAngle", n))


def validate_counts(code, n):
    """SMLib's asserted (faces, edges, vertices, wire, lamina, manifold, regions, shells) for
    case n (the branch of an `if (nTest ...)` taken)."""
    calls = re.findall(r"ValidateCounts\s*\(([^)]*)\)", code)
    if not calls:
        return None
    pick = calls[0]
    if len(calls) == 2:
        cond = re.search(r"if\s*\(\s*(nTest[^)]*)\)", code[code.find("DoSweep"):]).group(1)
        expr = re.sub(r"\bnTest\b", str(n), cond).replace("||", " or ").replace("&&", " and ")
        pick = calls[0] if eval(expr, {"__builtins__": {}}) else calls[1]
    return tuple(int(x) for x in pick.split(","))


def counts_doc(vc):
    if not vc:
        return "SMLib asserts no counts"
    return "SMLib asserts %d faces, %d edges, %d vertices (%d wire, %d lamina, %d manifold edges), %d regions, %d shells" % vc


def slug(s):
    return re.sub(r"[^a-z0-9]+", "_", s.lower()).strip("_")


def title(code, n, group):
    ts = re.findall(r'_T\(\s*"\s*:\s*([^"\\]*)', code)
    return ts[group.index(n)] if len(ts) == len(group) else ts[0]


def line_of(src, needle, start=0):
    return src[:src.index(needle, start)].count("\n") + 1


def rot_len(pts, o, d, deg):
    """Total length of the arcs points sweep about an axis."""
    o, d = [float(x) for x in o], [float(x) for x in d]
    n = math.sqrt(sum(x * x for x in d))
    d = [x / n for x in d]
    total = 0.0
    for p in pts:
        w = [p[i] - o[i] for i in range(3)]
        t = sum(w[i] * d[i] for i in range(3))
        r = math.sqrt(max(0.0, sum((w[i] - t * d[i]) ** 2 for i in range(3))))
        total += r * math.radians(deg)
    return total


def test_brep_case(n, code, src_line, name, ttl, vc):
    """A sweep of `my_create_test_brep`'s pieces: a Case, or (None, reason)."""
    m = motion(code, n)
    reps = re.search(r"SetRepetitions\((\d+)\)", code)
    reps = int(reps.group(1)) if reps else 1
    call = re.search(r"DoSweep\s*\(\s*pBrep\s*,\s*pBrep->GetInfiniteRegion\(\)\s*,(.*?)\)\s*\)", code, re.S)
    args = [a.strip() for a in call.group(1).split(",")][:6]
    hf, he, hv, sf, se, sv = [a != "NULL" for a in args]
    upper = "sFaces.Add(sBrepFaces[0])" in code.replace(" ", "")
    doc_base = "SMLib sweep %d, %s (smSweepTest.cpp:%d): %s%s; %s." % (
        n, ttl.strip(), src_line, "translated by %s" % fl(m[1]) if m[0] == "trans" else "turned %g° about the axis through %s along %s" % (m[3], fl(m[1]), fl(m[2])),
        ", %d times" % reps if reps > 1 else "", counts_doc(vc))
    mv = move_rust(m)
    if not any((hf, he, hv, sf, se, sv)):
        # The whole B-rep (case 8).
        r = g.test_region()
        vol, area = g.oblique_prism(r, m[1])
        wl = math.dist(*g.WIRE)
        h = math.sqrt(sum(x * x for x in m[1]))
        call = "whole_test_brep(%s, %s, %s, %s)" % (mv, want("SOLID", volume=vol, area=area), want("SHEET", faces=1, edges=4, area=wl * h),
                                                    want("WIRE", edges=1, length=h))
        return Case(name, doc_base + " cadrs: the face as a solid, the wire edge as a sheet, the vertex shell as a wire (Phase 1b)", call)
    if hf:
        r = g.test_region("upper" if upper else "all")
        if m[0] == "trans":
            vol, area = g.oblique_prism(r, m[1])
        else:
            vol, area = g.revolution(r, m[1], m[2], m[3])
        what = "its face 0 (the half with the triangle)" if upper else "its face (both halves)"
        return Case(name, doc_base + " cadrs: %s as a solid; SMLib's counts hold its wire edge, vertex shell and the faces inside" % what,
                    "faces(%s, %s, %s)" % ("true" if upper else "false", mv, want("SOLID", volume=vol, area=area)))
    if he:
        r = g.test_region()
        L = r.perimeter() + math.dist(*g.DIAGONAL) + math.dist(*g.WIRE)
        h = math.sqrt(sum(x * x for x in m[1]))
        return Case(name, doc_base + " cadrs: the face's boundary, the splitting edge and the wire edge as sheets (12 faces)",
                    "edges(%s, %s)" % (mv, want("SHEET", area=L * h)))
    if hv:
        pts = [list(g.ACORN) + [0.0]] if "GetShellVertices" in code else [list(p) for p in g.VERTICES]
        if m[0] == "trans":
            length = len(pts) * reps * math.sqrt(sum(x * x for x in m[1]))
        else:
            length = reps * rot_len(pts, m[1], m[2], m[3])
        note = " (SMLib also copies the edges; not converted)" if se else ""
        return Case(name, doc_base + " cadrs: each of its %d vertices swept into %s (Phase 1b: wires)%s" % (
            len(pts), "an edge" if reps == 1 else "%d edges" % reps, note),
                    "vertices(&%s, %s, %d, %s)" % ("[%s]" % ", ".join(f3(p) for p in pts), mv, reps, want("WIRE", edges=len(pts) * reps, length=length)))
    return None


def whole_case(n, code, src_line, name, ttl, vc):
    """A whole solid swept (cases 25-34)."""
    m = motion(code, n)
    mv = move_rust(m)
    doc = "SMLib sweep %d, %s (smSweepTest.cpp:%d): %s; %s" % (
        n, ttl.strip(), src_line, "translated by %s" % fl(m[1]) if m[0] == "trans" else "turned %g° about the axis through %s along %s" % (m[3], fl(m[1]), fl(m[2])),
        counts_doc(vc))
    if "CreateSphere" in code:
        r = number(code, "dRadius", n)
        c = vec3(code, "sCenter")
        o, d, deg = m[1], m[2], m[3]
        dn = math.sqrt(sum(x * x for x in d))
        w = [c[i] - o[i] for i in range(3)]
        t = sum(w[i] * d[i] for i in range(3)) / dn
        R = math.sqrt(sum(x * x for x in w) - t * t)
        th = math.radians(min(deg, 360.0))
        kw = {}
        if deg >= 360:
            kw = {"volume": 2 * math.pi ** 2 * R * r * r, "area": 4 * math.pi ** 2 * R * r}
            how = "a torus (Pappus)"
        elif deg + 2 * math.degrees(math.asin(r / R)) <= 360:
            kw = {"volume": math.pi * r * r * R * th + 4 / 3 * math.pi * r ** 3, "area": 2 * math.pi * r * R * th + 4 * math.pi * r * r}
            how = "a torus piece and two half balls (Pappus)"
        else:
            kw = {"volume": g.sphere_turn_overlap(R, r, 360 - deg)}
            how = "the torus less the gap's wedge plus the end balls' parts in it (a numeric integral); SMLib's sweep overlaps itself and isn't resolved, so its area isn't the swept solid's"
        what = "ball of radius %g at %s" % (r, fl(c))
        return Case(name, "%s. cadrs: the %s swept: %s." % (doc, what, how), "sphere(%s, %s, %s, %s)" % (f3(c), rust_f64(r), mv, want("SOLID", **kw)))
    if "my_make_hollow_box" in code:
        outer, cav = ((0.0, 0.0, 0.0), (10.0, 8.0, 6.0)), ((1.0, 1.0, 0.5), (8.0, 6.0, 5.0))
        vol, area = g.zonotope([(10, 0, 0), (0, 8, 0), (0, 0, 6), m[1]])
        # The cavity is swallowed: no point of it stays clear of the walls along the sweep.
        assert m[0] == "trans" and abs(m[1][0]) > 8
        return Case(name, "%s. cadrs: the box less the box inside it (`my_make_hollow_box`, smSweepTest.cpp:399) swept: a zonotope, the cavity swallowed." % doc,
                    "boxes((%s, %s), Some((%s, %s)), %s, %s)" % (f3(outer[0]), f3(outer[1]), f3(cav[0]), f3(cav[1]), mv,
                                                              want("SOLID", faces=12, edges=24, volume=vol, area=area)))
    if "my_make_block_with_lines" in code:
        lo, sz = (8.0, 0.0, 0.0), (4.0, 3.0, 1.5)
        src = "the block of `my_make_block_with_lines` (smSweepTest.cpp:288; its lines inside faces change no geometry and aren't built, so SMLib's counts, which hold them, aren't compared)"
        if m[0] == "trans":
            vol, area = g.zonotope([(sz[0], 0, 0), (0, sz[1], 0), (0, 0, sz[2]), m[1]])
            return Case(name, "%s. cadrs: %s swept: a zonotope, 12 faces and 24 edges." % (doc, src),
                        "boxes((%s, %s), None, %s, %s)" % (f3(lo), f3(sz), mv, want("SOLID", faces=12, edges=24, volume=vol, area=area)))
        if m[3] >= 360:
            vol = g.box_turn_volume(lo, sz, m[1], m[2])
            return Case(name, "%s. cadrs: %s turned whole: each face revolved about an axis off its plane (a numeric integral for the volume)." % (doc, src),
                        "boxes((%s, %s), None, %s, %s)" % (f3(lo), f3(sz), mv, want("SOLID", volume=vol)))
        return Case(name, "%s. cadrs: %s turned: each face revolved about an axis off its plane (no closed form: only checked to be a valid solid)." % (doc, src),
                    "boxes((%s, %s), None, %s, &SOLID)" % (f3(lo), f3(sz), mv))
    return None


def null_region_cases(src):
    line = line_of(src, "static SmStatus my_test_dosweep_null_region")
    a = 7 / math.sqrt(2)
    v = [a, a, 0.0]
    cases = []
    cases.append(Case("null_region_vertex", "SMLib sweep `my_test_dosweep_null_region` (smSweepTest.cpp:%d): a corner of a 16 box swept along (1,1,0)/√2 · 7, 1 to 3 times (SMLib asserts 12 + n edges, 8 + n vertices: the box and n wire edges). cadrs: the corner swept into a wire 3 times (Phase 1b)." % line,
                      "vertices(&[[0.0, 0.0, 0.0]], %s, 3, %s)" % (move_rust(("trans", v)), want("WIRE", edges=3, length=21.0))))
    vol, area = g.zonotope([(10, 0, 0), (0, 8, 0), (0, 0, 6), v])
    vx, vy, vz = 8 - a, 6 - a, 5.0
    void_v = vx * vy * vz
    void_a = 2 * (vx * vy + vy * vz + vz * vx)
    cases.append(Case("null_region_hollow", "SMLib sweep `my_test_dosweep_null_region` (smSweepTest.cpp:%d): the hollow box (`my_make_hollow_box`) swept whole along (1,1,0)/√2 · 7 (SMLib asserts no counts: its merge is off). cadrs: a hexagonal prism (8 faces, 18 edges) with the part of the cavity the walls never cross left as a void (6 faces, 12 edges): the cavity's points whose path back stays in it." % line,
                      "boxes((%s, %s), Some((%s, %s)), %s, %s)" % (f3((0, 0, 0)), f3((10, 8, 6)), f3((1, 1, 0.5)), f3((8, 6, 5)), move_rust(("trans", v)),
                                                                want("SOLID", faces=14, edges=30, volume=vol - void_v, area=area + void_a))))
    return cases


def extract(ctx):
    src = ctx.source(SOURCE)
    start = re.search(r"SmStatus my_sweep_demo\s*\(", src).start()
    end = src.index("// end my_sweep_demo\n", start)
    blocks = dict(case_blocks(src[start:end]))
    cases = []
    for n in sorted(blocks):
        group = [n]
        k = n
        while not blocks[k].strip():
            k += 1
            group.append(k)
        # Earlier fallthrough labels share this code.
        j = n - 1
        while j in blocks and not blocks[j].strip():
            group.insert(0, j)
            j -= 1
        code = blocks[k]
        ttl = title(code, n, group)
        src_line = line_of(src, "case %d:" % n, start)
        vc = validate_counts(code, n)
        name = "case_%02d_%s" % (n, slug(ttl))
        c = test_brep_case(n, code, src_line, name, ttl, vc) if "my_create_test_brep" in code else whole_case(n, code, src_line, name, ttl, vc)
        if c is not None:
            cases.append(c)
    return cases + null_region_cases(src)
