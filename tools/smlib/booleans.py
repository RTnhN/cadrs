"""SMLib's boolean tests (tests/prog_test/src/smMergeTest.cpp, the `booleans` suite of
prog_test.cpp, run by `my_test_merge`): manifold union, intersection and difference of two solids
(`my_test_bbu`, `SmMerge::ManifoldBoolean`), checked against SMLib's asserted edge counts and,
with the oracle, SMLib's own result.

What converts:
- the primitive pairs (`SmPrimitiveCreation`: box/box, cone/cone, sphere/sphere, torus/torus).
  SMLib's test unions them; intersection and difference are added, from the oracle only;
- `my_test_stress`'s BoxUCylTan union;
- `boolean_trees.smp`: the chain union of `my_many_prim_test` (45 edges);
- `box_box_grid`, `box_cyl_grid`, `box_cone_grid.smp`: one case per pair of
  `my_two_prim_test` whose SMLib union is a manifold solid (pairs that touch are left out) and
  one per grid for those pairs together (the box/box one checks the asserted total less the
  pairs left out).

The oracle runs each chain step by step (`boolean` with an output file) here, not in the
driver.
"""
import json
import os
import re
import subprocess
import sys
import tempfile

from smlib_to_rust import Case, Fixture, drop_dead, strip_comments

SOURCE = "smMergeTest.cpp"
TITLE = "boolean"

NOT_CONVERTED = [
    ("my_test_non_manifold (smMergeTest.cpp:989), 17 cases",
     "a box with wires (5) or vertex shells (5), two wires (2): non-manifold cell complexes, out of scope"),
    ("my_test_non_manifold: box ∪ / − plane.smb, merge with plane.smb",
     "SMLib's results keep the sheet inside the box (7 faces, not a manifold solid): non-manifold, out of scope"),
    ("my_test_non_manifold: box ∩ plane.smb, plane.smb − box",
     "SMLib gives a sheet (one face, area 49 and 51): sheet results, for Phase 1b"),
    ("Piecewise merge (smMergeTest.cpp:1851, 2 cases) and `my_test_bbu 20` (box merged with itself)",
     "SmMerge::PiecewiseMerge / SM_BO_MERGE: cell complexes, not a boolean the kernel has"),
    ("my_stitching_demo (smMergeTest.cpp:343)",
     "fillet_ts.smb is stitched (StitchFaces 0.001, 0.005) before the difference; unstitched, SMLib's difference isn't a solid: the stitch suite's"),
    ("my_mouse_demo (mouse_1..5.smb)",
     "a SM_BO_MERGE chain of five one-face sheets then MakeManifold: a sheet complex (23 faces, 71 edges in SMLib), not a solid"),
    ("my_test_stress: box − box rotated 0.1°",
     "a rotated copy of box.smb: the oracle's `boolean` takes no transform"),
    ("my_test_stress: extrudeHoles1/2, BoltMBox1/2",
     "SM_BO_MERGE, results with vertex shells (ValidateCounts on regions and shells)"),
    ("my_test_BrepPerShell (smMergeTest.cpp:495)",
     "NonManifoldBoolean into a cell complex with wires and vertex shells, then CreateOneBrepPerBody"),
    ("boolean_trees.smp merge (59 edges)", "SM_BO_MERGE"),
    ("nurb_nurb_1/2.smb (`my_test_bbu 21`)",
     "a SM_BO_MERGE of two offset-surface sheets: a sheet complex, not a solid"),
]

PRELUDE = r'''
use cadrs_kernel::native::draw::{Ax3, Shape};
use cadrs_kernel::native::math::{P3, V3};
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BodyId, BoolOp, Kernel};

const SUITE: &str = "booleans";

const U: BoolOp = BoolOp::Union;
const I: BoolOp = BoolOp::Intersect;
const D: BoolOp = BoolOp::Subtract;

fn all_planar(b: &Body) -> bool {
    b.faces.iter().all(|f| matches!(f.surf, Surface::Plane { .. }))
}

/// `a op b`, one body.
fn bop(k: &mut NativeKernel, op: BoolOp, a: BodyId, b: BodyId) -> Result<BodyId, String> {
    let r = k.boolean(op, a, &[b]).map_err(|e| format!("{e:?}"))?;
    match r.bodies.as_slice() {
        [one] => Ok(*one),
        v => Err(format!("{} bodies", v.len())),
    }
}

/// Checks `out` (counts compared only when every input is planar).
fn check(k: &NativeKernel, inputs: &[Body], out: BodyId, e: &Expect) -> Result<(), String> {
    let input = inputs.iter().find(|b| !all_planar(b)).unwrap_or(&inputs[0]);
    check_result(input, k.get(out).map_err(|e| format!("{e:?}"))?, e)
}

/// SMLib's primitives (`SmPrimitiveCreation`), full turns about +z from `at`.
enum Prim {
    /// `CreateBox(l, w, h)`: min corner at `at`.
    Box(f64, f64, f64),
    /// `CreateCone(h, base radius, top radius)`: base centre at `at`.
    Cone(f64, f64, f64),
    /// `CreateSphere(r)`: centre at `at`.
    Sphere(f64),
    /// `CreateTorus(R, r)`: centre at `at`.
    Torus(f64, f64),
}

fn prim(k: &mut NativeKernel, p: &Prim, at: [f64; 3]) -> Result<BodyId, String> {
    let ax = Ax3 { origin: P3::new(at[0], at[1], at[2]), z: V3::z(), x: V3::x() };
    let s = match *p {
        Prim::Box(l, w, h) => k.box_on(&ax, l, w, h),
        Prim::Cone(h, r0, r1) => k.pcone_on(&ax, r0, r1, h),
        Prim::Sphere(r) => k.psphere_on(&ax, r),
        Prim::Torus(big, r) => k.ptorus_on(&ax, big, r),
    }
    .map_err(|e| format!("{e:?}"))?;
    match s {
        Shape::Body(b) => Ok(b),
        _ => Err("the primitive isn't a solid".into()),
    }
}

/// `a op b` on two primitives.
fn primitives(a: (Prim, [f64; 3]), b: (Prim, [f64; 3]), op: BoolOp, e: &Expect) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (ba, bb) = (prim(&mut k, &a.0, a.1)?, prim(&mut k, &b.0, b.1)?);
    let inputs = [(**k.get(ba).map_err(|e| format!("{e:?}"))?).clone(), (**k.get(bb).map_err(|e| format!("{e:?}"))?).clone()];
    let r = bop(&mut k, op, ba, bb)?;
    check(&k, &inputs, r, e)
}

/// `files[0] op files[1] op ...`, left to right (SMLib's `my_many_prim_test` and
/// `my_two_prim_test`).
fn chain(files: &[&str], op: BoolOp, e: &Expect) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let bodies = files.iter().map(|f| model(SUITE, f).map(|m| m.body)).collect::<Result<Vec<_>, _>>()?;
    let mut acc = k.insert(bodies[0].clone());
    for b in &bodies[1..] {
        let id = k.insert(b.clone());
        acc = bop(&mut k, op, acc, id)?;
    }
    check(&k, &bodies, acc, e)
}

/// `my_two_prim_test` on a grid part: B-rep 2i ∪ B-rep 2i+1 for each i of `pairs`, each a
/// valid solid; their edges total `total` when every input is planar.
fn grid(file: &str, pairs: &[usize], total: Option<usize>) -> Result<(), String> {
    let (mut sum, mut planar, mut failed) = (0, true, Vec::new());
    for &i in pairs {
        let (a, b) = (model(SUITE, &format!("{file}#{}", 2 * i))?.body, model(SUITE, &format!("{file}#{}", 2 * i + 1))?.body);
        planar &= all_planar(&a) && all_planar(&b);
        let mut k = NativeKernel::new();
        let (ia, ib) = (k.insert(a.clone()), k.insert(b.clone()));
        match bop(&mut k, U, ia, ib).and_then(|r| check(&k, &[a, b], r, &Expect::default()).map(|_| r)) {
            Ok(r) => sum += edge_count(k.get(r).map_err(|e| format!("{e:?}"))?),
            Err(e) => failed.push(format!("pair {i}: {e}")),
        }
    }
    if !failed.is_empty() {
        return Err(failed.join("; "));
    }
    match total {
        Some(t) if planar && sum != t => Err(format!("{sum} edges in all, SMLib gives {t}")),
        _ => Ok(()),
    }
}
'''

PRIM = {"CreateBox": ("Box", 3), "CreateCone": ("Cone", 3), "CreateSphere": ("Sphere", 1), "CreateTorus": ("Torus", 2)}
OPS = [("U", "union"), ("I", "intersection"), ("D", "difference")]
MODEL = "TestFiles/pt_TestFiles/"


def fnum(s):
    return float(s.strip().rstrip("fF"))


def rust(x):
    r = repr(float(x))
    return r if "." in r or "e" in r else r + ".0"


def top_blocks(code):
    """The bodies of the `if (TRUE) { ... }` blocks of `code` (not nested ones)."""
    out, i = [], 0
    pat = re.compile(r"\bif\s*\(\s*TRUE\s*\)\s*\{")
    while True:
        m = pat.search(code, i)
        if not m:
            return out
        depth, j = 0, m.end() - 1
        while j < len(code):
            depth += {"{": 1, "}": -1}.get(code[j], 0)
            j += 1
            if depth == 0:
                break
        out.append(code[m.end():j - 1])
        i = j


def function(src, name):
    """The body of `name`'s definition in `src` (comments and dead branches dropped)."""
    code = drop_dead(strip_comments(src))
    m = re.search(r"\bSmStatus\s+%s\s*\([^)]*\)\s*\{" % name, code)
    depth, j = 0, m.end() - 1
    while j < len(code):
        depth += {"{": 1, "}": -1}.get(code[j], 0)
        j += 1
        if depth == 0:
            break
    return code[m.end():j - 1]


def line_of(src, needle):
    return src[:src.find(needle)].count("\n") + 1


def prim_spec(kind, args, at):
    """The primitive as the oracle's spec and as Rust."""
    name, n = PRIM[kind]
    a = [fnum(x) for x in args[:n]]
    spec = "%s:%s:%s" % (name.lower(), ":".join("%g" % x for x in a), ":".join("%g" % x for x in at))
    return spec, "(Prim::%s(%s), [%s])" % (name, ", ".join(rust(x) for x in a), ", ".join(rust(x) for x in at))


# A primitive's volume or area may be this far (relative) off its closed form in SMLib for the
# oracle's value to be compared (the comparison is to 1e-4).
PRIM_TOL = 5e-5


def closed_form(spec):
    """(volume, area) of a primitive spec."""
    kind, *a = spec.split(":")
    a = [float(x) for x in a]
    pi = 3.141592653589793
    if kind == "box":
        l, w, h = a[:3]
        return l * w * h, 2 * (l * w + w * h + h * l)
    if kind == "cone":
        h, r0, r1 = a[:3]
        return pi * h / 3 * (r0 * r0 + r0 * r1 + r1 * r1), pi * (r0 * r0 + r1 * r1) + pi * (r0 + r1) * ((r0 - r1) ** 2 + h * h) ** 0.5
    if kind == "sphere":
        return 4 / 3 * pi * a[0] ** 3, 4 * pi * a[0] ** 2
    if kind == "torus":
        return 2 * pi * pi * a[0] * a[1] ** 2, 4 * pi * pi * a[0] * a[1]
    raise ValueError(spec)


def prim_error(spec, key):
    """How far (relative) SMLib's primitive's volume or area is off its closed form, or None
    without the oracle."""
    r = oracle_run(["props", spec])
    if not r or not r.get("ok"):
        return None
    want = closed_form(spec)[0 if key == "volume" else 1]
    return abs(r[key] - want) / want


def oracle_run(args):
    """SMLib's report (the oracle's JSON) for one command, or None without the oracle."""
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    if not ob or not os.path.exists(ob):
        return None
    r = subprocess.run([ob] + args, capture_output=True, text=True, timeout=600)
    lines = [l for l in r.stdout.splitlines() if l.startswith("{")]
    return json.loads(lines[-1]) if lines else {"ok": False, "status": "no output (exit %d)" % r.returncode}


def primitive_cases(src, merge):
    cases = []
    for b in top_blocks(merge):
        if "SmPrimitiveCreation sPC2" not in b:
            continue
        c1 = re.search(r"sPC\.(Create\w+)\(([^;]*)\);", b)
        c2 = re.search(r"sPC2\.(Create\w+)\(([^;]*)\);", b)
        tr = re.search(r"sRef\.Translate\(SmVector3d\(([^)]*)\)\)", b)
        if not (c1 and c2 and tr and c1.group(1) in PRIM and c2.group(1) in PRIM):
            continue
        at = [fnum(x) for x in tr.group(1).split(",")]
        s1, r1 = prim_spec(c1.group(1), c1.group(2).split(","), [0.0, 0.0, 0.0])
        s2, r2 = prim_spec(c2.group(1), c2.group(2).split(","), at)
        e = re.search(r"sEdges\.GetSize\(\)\s*==\s*(\d+)", b)
        kind = PRIM[c1.group(1)][0].lower()
        line = line_of(src, "sPC.%s(%s)" % (c1.group(1), c1.group(2)))
        for op, what in OPS:
            asserted = {"edges": int(e.group(1))} if (e and op == "U") else {}
            src_note = "SMLib's test (smMergeTest.cpp:%d)" % line if op == "U" else "added: SMLib's test unions this pair (smMergeTest.cpp:%d)" % line
            doc = "SMLib boolean: %s `%s` %s `%s` (%s)." % (what, s1, {"U": "∪", "I": "∩", "D": "−"}[op], s2, src_note)
            c = Case("prim_%s_%s" % (kind, what), doc, "primitives(%s, %s, %s, {expect})" % (r1, r2, op), [], asserted)
            r = oracle_run(["boolean", s1, s2, op])
            bake(c, r)
            # SMLib's curved primitives are approximations: what they are off their closed form
            # (area or volume) isn't compared.
            for k in ("volume", "area"):
                off = max(prim_error(s, k) for s in (s1, s2))
                if off is not None and off > PRIM_TOL and k in c.expect:
                    del c.expect[k]
                    c.note = (c.note + "; " if c.note else "") + "SMLib's primitive's %s is %.1e off its closed form: %s not compared" % (k, off, k)
            cases.append(c)
    return cases


def stress_cases(src):
    cases = []
    body = function(src, "my_test_stress")
    for b in top_blocks(body):
        m = re.search(r'my_boolean_stress_test\(\s*sContext\s*,\s*_T\("\.\./\.\./([^"]+)"\)\s*,\s*_T\("\.\./\.\./([^"]+)"\)\s*,\s*(\d+)', b)
        if not m or m.group(3) != "0":
            continue
        f1, f2 = Fixture(m.group(1)), Fixture(m.group(2))
        vc = re.search(r"ValidateCounts\((\d+),(\d+),", b)
        stem = os.path.splitext(os.path.basename(m.group(1)))[0].rstrip("0123456789").lower()
        doc = "SMLib boolean: `%s` ∪ `%s` (`my_test_stress`, smMergeTest.cpp:%d; SMLib's test merges with tolerance 0, the oracle with SmMerge's default)." % (
            f1.name, f2.name, line_of(src, m.group(1)))
        cases.append(Case("stress_%s_union" % stem, doc, 'chain(&["%s", "%s"], U, {expect})' % (f1.name, f2.name),
                          [f1, f2], {"faces": int(vc.group(1)), "edges": int(vc.group(2))} if vc else {},
                          ("boolean", [f1, f2, "U"])))
    return cases


def oracle_chain(ctx, fixtures, op):
    """SMLib's `a op b op ...` left to right, step by step: its report of the last step, or
    {ok: False}."""
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    if not ob or not os.path.exists(ob):
        return None
    with tempfile.TemporaryDirectory(prefix="smlib_chain_") as tmp:
        acc = ctx.path(fixtures[0].rel)
        res = None
        for k, f in enumerate(fixtures[1:]):
            out = os.path.join(tmp, "step%d.smb" % k)
            res = oracle_run(["boolean", acc, ctx.path(f.rel), op, out])
            if not res.get("ok"):
                return res
            acc = out
        return res


def bake(c, r):
    """SMLib's chain result into the case, as the driver does for single runs."""
    if r is None:
        return
    if not r.get("ok"):
        c.note = "SMLib (the oracle) fails: %s" % r.get("status", "?")
    elif r.get("solid") is False:
        c.note = "SMLib's own result isn't a manifold solid (volume %s, %s faces): not compared" % (r.get("volume"), r.get("faces"))
    else:
        if c.expect.get("edges") is not None and c.expect["edges"] != r.get("edges"):
            c.note = "SMLib's test asserts %d edges, the oracle gives %d" % (c.expect["edges"], r["edges"])
        for k in ("volume", "area", "faces", "edges", "solid"):
            if k in r:
                c.expect[k] = r[k]


def part_cases(ctx, src, merge):
    cases = []
    for b in top_blocks(merge):
        m = re.search(r'ReadPartFromFile\(\s*sContext\s*,\s*_T\("\.\./\.\./([^"]+\.smp)"\)', b)
        if not m:
            continue
        path = m.group(1)
        stem = os.path.splitext(os.path.basename(path))[0]
        total = re.search(r"lTotalEdges\s*==\s*(\d+)", b)
        total = int(total.group(1)) if total else None
        line = line_of(src, 'MYPRINTF(_T("\\nReading test file ../../%s"));' % path)
        many = re.search(r"my_many_prim_test\(\s*sBreps\s*,\s*\w+\s*,\s*(\d+)", b)
        two = re.search(r"my_two_prim_test\(\s*sBreps\s*,\s*(\d+)", b)
        if many and many.group(1) == "0":
            # (((B[n-1] ∪ B[n-2]) ∪ ...) ∪ B0), the part's empty B-reps left out.
            n = count_breps(ctx, path)
            fx = [Fixture("%s#%d" % (path, i)) for i in reversed(range(n))]
            fx = [f for f in fx if not empty(ctx, f)]
            doc = "SMLib boolean: union of `%s`'s B-reps, last to first (`my_many_prim_test`, smMergeTest.cpp:%d; SMLib asserts %s edges)." % (
                os.path.basename(path), line, total)
            c = Case("%s_union" % stem, doc, "chain(&[%s], U, {expect})" % ", ".join('"%s"' % f.name for f in fx),
                     fx, {"edges": total} if total is not None else {})
            bake(c, oracle_chain(ctx, fx, "U"))
            cases.append(c)
        elif two and two.group(1) == "0":
            n = count_breps(ctx, path) // 2
            kept, dropped = [], []  # (pair, SMLib's edges)
            for i in range(n):
                a, bb = Fixture("%s#%d" % (path, 2 * i)), Fixture("%s#%d" % (path, 2 * i + 1))
                r = oracle_chain(ctx, [a, bb], "U")
                if r is not None and r.get("ok") and r.get("solid") is False:
                    # Touching parts: SMLib's union is non-manifold, out of scope.
                    dropped.append((i, r.get("edges")))
                    continue
                kept.append(i)
                doc = "SMLib boolean: `%s` ∪ `%s` (`my_two_prim_test`, smMergeTest.cpp:%d)." % (a.name, bb.name, line)
                c = Case("%s_%02d" % (stem, i), doc, 'chain(&["%s", "%s"], U, {expect})' % (a.name, bb.name), [a, bb])
                bake(c, r)
                cases.append(c)
            NOT_CONVERTED.extend(("`%s` pair %d (B-reps %d, %d)" % (os.path.basename(path), i, 2 * i, 2 * i + 1),
                                  "the parts touch; SMLib's union is non-manifold (%s edges), out of scope" % e) for i, e in dropped)
            left = total - sum(e for _, e in dropped) if total is not None and all(e is not None for _, e in dropped) else None
            doc = "SMLib boolean: the unions of `%s` (`my_two_prim_test`, smMergeTest.cpp:%d): SMLib asserts %s edges for all %d%s (compared for planar inputs)." % (
                os.path.basename(path), line, total, n,
                "; without the non-manifold pairs %s, %s" % (", ".join(str(i) for i, _ in dropped), left) if dropped else "")
            cases.append(Case("%s_all" % stem, doc, 'grid("%s", &[%s], %s)' % (
                os.path.basename(path), ", ".join(str(i) for i in kept), "Some(%d)" % left if left is not None else "None"),
                [Fixture("%s#0" % path)]))
    return cases


_SMP = {}


def smp_breps(ctx, path):
    """[is the B-rep empty] for each B-rep of a part, from its `SmBrep` records' face counts
    (read by the oracle's `props` when it is there, else from the file)."""
    if path not in _SMP:
        ob = os.environ.get("CADRS_SMLIB_ORACLE")
        out = []
        if ob and os.path.exists(ob):
            i = 0
            while True:
                r = subprocess.run([ob, "props", "%s#%d" % (ctx.path(path), i)], capture_output=True, text=True, timeout=120)
                lines = [l for l in r.stdout.splitlines() if l.startswith("{")]
                res = json.loads(lines[-1]) if lines else {"ok": False}
                if not res.get("ok"):
                    break
                out.append(res.get("faces", 0) == 0)
                i += 1
        _SMP[path] = out
    return _SMP[path]


def count_breps(ctx, path):
    n = len(smp_breps(ctx, path))
    if n == 0:
        raise ValueError("%s: can't count its B-reps without the oracle" % path)
    return n


def empty(ctx, f):
    path, i = f.rel.split("#")
    return smp_breps(ctx, path)[int(i)]


def extract(ctx):
    src = ctx.source(SOURCE)
    merge = function(src, "my_test_merge")
    cases = primitive_cases(src, merge) + stress_cases(src)
    try:
        cases += part_cases(ctx, src, merge)
    except ValueError as e:
        print("booleans: parts not converted: %s" % e, file=sys.stderr)
    return cases
