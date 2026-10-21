"""SMLib's offset and shell tests (tests/prog_test/src/smOffsetTest.cpp, run by
`my_offset_regression`): `my_offset_demo` and `my_shell_demo`, checked against SMLib's asserted
counts (`ValidateCounts`, compared on planar models only) and, with the oracle, SMLib's own
result.

Two kinds of block convert:
- `my_test_offset(file, keepFace, d, tol)` (smOffsetTest.cpp:30): every face moved by d (an inset
  when negative) with SMLib's convex edges rounded (`SetExtendConvexEdges(FALSE)`), face
  keepFace (1-based, 0 for none) left where it is: `Kernel::offset` with an `OffsetSpec`, the
  kept face at distance 0. lCount 15 then subtracts `offset_bracket_box.smb` (`my_test_bbu`).
- `ShellBrep(file, d, bExtendCorners, ..., faces)` (smOffsetTest.cpp:190): the faces (0-based)
  opened, walls |d| thick, inside the body when d < 0 and outside when d > 0 (SMLib offsets or
  insets with those faces kept, then takes the difference with the input): `shell_with` with a
  `ShellSpec`, the faces picked by their centroids.
"""
import os
import re
import sys

from smlib_to_rust import Case, Fixture, drop_dead, if_blocks, num, rust_f64, strip_comments

SOURCE = "smOffsetTest.cpp"
TITLE = "offset and shell"

NOT_CONVERTED = [
    ("my_offset_demo lCount 18 (`airfoil_offset`)", "its model, airfoil.brep, isn't in the checkout"),
    ("my_shell_demo lCount 0, 12–15", "compiled out (`#if 0`, `if (0 && ...)`)"),
]

PRELUDE = r'''
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BoolOp, Kernel, OffsetSpec, OpResult, ShellSpec};

const SUITE: &str = "offset";

fn one(k: &NativeKernel, r: OpResult) -> Result<&Body, String> {
    match r.bodies.as_slice() {
        [b] => k.get(*b).map(|b| &**b).map_err(|e| format!("{e:?}")),
        v => Err(format!("{} bodies", v.len())),
    }
}

/// SMLib's `my_test_offset`: every face of `file` moved by `d` (outward when positive), face
/// `keep` (SMLib's index, 0-based here) left in place; then `cutter` subtracted when given.
fn offset(file: &str, d: f64, keep: Option<usize>, cutter: Option<&str>, e: &Expect) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let mut spec = OffsetSpec { distance: d, faces: Vec::new(), sharp: false };
    if let Some(i) = keep {
        spec.faces = faces(&m, &[i])?.into_iter().map(|f| (f, 0.0)).collect();
    }
    let input = m.body.clone();
    let mut k = NativeKernel::new();
    let b = k.insert(m.body);
    let r = k.offset(b, &spec).map_err(|e| format!("offset: {e:?}"))?;
    let r = match cutter {
        Some(c) => {
            let [id] = r.bodies[..] else { return Err(format!("{} bodies", r.bodies.len())) };
            let cut = k.insert(model(SUITE, c)?.body);
            k.boolean(BoolOp::Subtract, id, &[cut]).map_err(|e| format!("subtract: {e:?}"))?
        }
        None => r,
    };
    check_result(&input, one(&k, r)?, e)
}

/// SMLib's `ShellBrep`: the faces `open` (SMLib's indices) removed and walls `|d|` thick,
/// inside the body when `d < 0`, outside when `d > 0`.
fn shell(file: &str, d: f64, open: &[usize], e: &Expect) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let spec = ShellSpec { remove: faces(&m, open)?, thickness: d.abs(), outward: d > 0.0, hollow: false };
    let input = m.body.clone();
    let mut k = NativeKernel::new();
    let b = k.insert(m.body);
    let r = k.shell_with(b, &spec).map_err(|e| format!("shell: {e:?}"))?;
    check_result(&input, one(&k, r)?, e)
}
'''

MODEL = re.compile(r'_T\(\s*"\.\./\.\./(TestFiles/pt_TestFiles/[^"]+\.smb)"\s*\)')

# SMLib's tolerances by name.
TOLS = {"SM_ZONE_TOL_3D": 1e-5}


def drop_if0(code):
    """`code` without its `#if 0 ... #endif` regions (nested `#if`s counted; an `#else` keeps
    the rest)."""
    out, skip, depth = [], False, 0
    for line in code.split("\n"):
        s = line.strip()
        if not skip and re.match(r"#\s*if\s+0\b", s):
            skip, depth = True, 1
            continue
        if skip:
            if re.match(r"#\s*if", s):
                depth += 1
            elif re.match(r"#\s*endif", s):
                depth -= 1
                if depth == 0:
                    skip = False
            elif depth == 1 and re.match(r"#\s*else", s):
                skip = False
            continue
        out.append(line)
    return "\n".join(out)


def function(src, name):
    """The body of `name`'s definition in `src` (comments and dead code dropped)."""
    code = drop_dead(drop_if0(strip_comments(src)))
    m = re.search(r"\bSmStatus\s+%s\s*\([^)]*\)\s*\{" % name, code)
    depth, j = 0, m.end() - 1
    while j < len(code):
        depth += {"{": 1, "}": -1}.get(code[j], 0)
        j += 1
        if depth == 0:
            break
    return code[m.end():j - 1]


def dead_lines(src):
    """The (1-based) line numbers of `src` inside `#if 0` regions."""
    dead, skip, depth = set(), False, 0
    for i, line in enumerate(src.split("\n"), 1):
        s = line.strip()
        if not skip and re.match(r"#\s*if\s+0\b", s):
            skip, depth = True, 1
        elif skip:
            if re.match(r"#\s*if", s):
                depth += 1
            elif re.match(r"#\s*endif", s):
                depth -= 1
                skip = depth > 0
            elif depth == 1 and re.match(r"#\s*else", s):
                skip = False
        if skip:
            dead.add(i)
    return dead


def line_of(src, needle):
    """The line of `needle`'s first occurrence in `src` outside `#if 0` regions."""
    dead = dead_lines(src)
    i = src.find(needle)
    while i >= 0:
        n = src[:i].count("\n") + 1
        if n not in dead:
            return n
        i = src.find(needle, i + 1)
    raise ValueError("%s not found" % needle)


def counts(b):
    """[(faces, edges)] of the block's `ValidateCounts` calls (9999: not asserted)."""
    out = []
    for f, e in re.findall(r"ValidateCounts\(\s*(\d+)\s*,\s*(\d+)\s*,", b):
        out.append((None if int(f) == 9999 else int(f), None if int(e) == 9999 else int(e)))
    return out


def expect_of(fe):
    f, e = fe
    out = {}
    if f is not None:
        out["faces"] = f
    if e is not None:
        out["edges"] = e
    return out


def stem(path):
    return os.path.splitext(os.path.basename(path))[0].lower()


def tol_arg(s):
    s = s.strip()
    return TOLS[s] if s in TOLS else num(s)


# Asserts SMLib's own runs contradict (smlib_oracle README, "Checked against the tests'
# expectations"; SMLib's golden run, usd-brep/rust/golden/release/pt_offset/progTest.txt).
STALE = {
    ("offset", 5): "SMLib's test asserts 40 faces, 78 edges; SMLib itself (the oracle and its golden run) gives 80 faces, 155 edges and no solid; the asserts are dropped",
}


def offset_cases(ctx, src, prefix, body):
    cases = []
    for n, b in if_blocks(body):
        ce = counts(b)
        offs = re.findall(r"my_test_offset\(\s*crContext\s*,\s*_T\(\s*\"\.\./\.\./([^\"]+)\"\s*\)\s*,\s*(\d+)\s*,\s*([-0-9.eE+]+)\s*,\s*([A-Z_0-9.eE+-]+)\s*,", b)
        sh = re.search(r"ShellBrep\(\s*crContext\s*,\s*(?:_T\()?\s*\"\.\./\.\./([^\"]+)\"\s*\)?\s*,\s*([-0-9.eE+]+)\s*,\s*(TRUE|FALSE)\s*,\s*(TRUE|FALSE)\s*,", b)
        if "airfoil_offset" in b:
            continue
        if offs:
            cutter = MODEL.search(b[b.find("pCutter"):]) if "pCutter" in b else None
            for k, (path, keep, d, tol) in enumerate(offs):
                fx = Fixture(path)
                keep, d, tol = int(keep), num(d), tol_arg(tol)
                name = "%s_%02d%s_%s" % (prefix, n, "abcdefgh"[k] if len(offs) > 1 else "", stem(path))
                line = line_of(src, 'Entered my_test_offsets %s"' % re.findall(r"Entered my_test_offsets (\d+)", b)[k])
                what = "`%s` offset by %s%s" % (fx.name, d, ", face %d kept" % (keep - 1) if keep else "")
                exp = expect_of(ce[k]) if k < len(ce) else {}
                fixtures = [fx]
                if cutter:
                    cf = Fixture(cutter.group(1))
                    fixtures.append(cf)
                    what += ", less `%s`" % cf.name
                    call = 'offset("%s", %s, %s, Some("%s"), {expect})' % (fx.name, rust_f64(d), "Some(%d)" % (keep - 1) if keep else "None", cf.name)
                else:
                    call = 'offset("%s", %s, %s, None, {expect})' % (fx.name, rust_f64(d), "Some(%d)" % (keep - 1) if keep else "None")
                doc = "SMLib %s demo %d (smOffsetTest.cpp:%d, `my_test_offset`): %s." % (prefix, n, line, what)
                if (prefix, n) in STALE:
                    doc += " %s." % STALE[(prefix, n)]
                    exp = {}
                oargs = [fx, d] + ([keep] if keep else []) + (["--tol=%r" % tol] if tol != 1e-4 else [])
                c = Case(name, doc, call, fixtures, exp, None if cutter else ("offset", oargs))
                if cutter:
                    bake(c, oracle_offset_cut(ctx, fx, d, keep, tol, cf))
                cases.append(c)
        elif sh:
            path, d, extend = sh.group(1), num(sh.group(2)), sh.group(3) == "TRUE"
            fx = Fixture(path)
            faces_ = [int(x) for x in re.findall(r"\.Add\(\s*(\d+)\s*\)", b)]
            line = line_of(src, 'Entered ShellBrep %s"' % re.search(r"Entered ShellBrep (\w+)", b).group(1))
            name = "%s_%02d_%s" % (prefix, n, stem(path))
            doc = "SMLib %s demo %d (smOffsetTest.cpp:%d, `ShellBrep`): `%s` shelled %s, %s thick, face%s %s open%s." % (
                prefix, n, line, fx.name, "inward" if d < 0 else "outward", abs(d), "s" if len(faces_) > 1 else "",
                ", ".join(map(str, faces_)), "" if extend else " (SMLib: corners rounded, `bExtendCorners` FALSE)")
            exp = expect_of(ce[0]) if ce else {}
            oargs = [fx, d] + faces_ + ([] if extend else ["--no-extend"])
            call = 'shell("%s", %s, &[%s], {expect})' % (fx.name, rust_f64(d), ", ".join(map(str, faces_)))
            cases.append(Case(name, doc, call, [fx], exp, ("shell", oargs)))
        else:
            print("offset: %s block %d not converted" % (prefix, n), file=sys.stderr)
    return cases


def oracle_run(args):
    """SMLib's report (the oracle's JSON) for one command, or None without the oracle."""
    import json
    import subprocess
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    if not ob or not os.path.exists(ob):
        return None
    r = subprocess.run([ob] + args, capture_output=True, text=True, timeout=600)
    lines = [l for l in r.stdout.splitlines() if l.startswith("{")]
    return json.loads(lines[-1]) if lines else {"ok": False, "status": "no output (exit %d)" % r.returncode}


def oracle_offset_cut(ctx, fx, d, keep, tol, cutter):
    """SMLib's offset then `my_test_bbu` difference with the cutter: the last step's report."""
    import tempfile
    with tempfile.TemporaryDirectory(prefix="smlib_offset_") as tmp:
        out = os.path.join(tmp, "offset.smb")
        r = oracle_run(["offset", ctx.path(fx.rel), repr(d), str(keep), out, "--tol=%r" % tol])
        if r is None or not r.get("ok"):
            return r
        return oracle_run(["boolean", out, ctx.path(cutter.rel), "D"])


def bake(c, r):
    """SMLib's chained result into the case, as the driver does for single runs."""
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


def extract(ctx):
    src = ctx.source(SOURCE)
    return (offset_cases(ctx, src, "offset", function(src, "my_offset_demo"))
            + offset_cases(ctx, src, "shell", function(src, "my_shell_demo")))
