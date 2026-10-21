"""SMLib's fillet tests (tests/prog_test/src/smFilletTest.cpp): each case reads its model,
fillets (or chamfers) the edges SMLib's test names by index and checks the result against the
edge count SMLib asserts and, with the oracle, SMLib's own fillet.

Three kinds of block convert:
- `my_test_model(file, ..., radius, index intervals, include/exclude, ..., chamfer, bevel)`:
  one radius on the named edges (or all others);
- `my_read_fillet_definition_file(*.fdf)`: a fillet definition file (read here: edge, face or
  whole-body definitions with a constant radius or a linear law);
- the blocks driving SmFilletExecutive directly (a radius or a linear law per edge).
"""
import os
import re
import sys

from smlib_to_rust import Case, Fixture, Out, if_blocks, num, rust_f64

SOURCE = "smFilletTest.cpp"
TITLE = "fillet"

NOT_CONVERTED = [
    ("101", "a fillet between two sheets' surfaces (`my_test_two_surface_fillet`), not a B-rep edge"),
    ("104, 106, 107", "a fillet between the faces of two separate B-reps (`my_test_one_surface_fillet`)"),
    ("108, 109", "empty"),
]

PRELUDE = r'''
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{ChamferSpec, FilletLaw, FilletSpec, Kernel};

const SUITE: &str = "fillets";

/// How the named edges are blended.
enum Op {
    Fillet(f64),
    Chamfer(f64),
}

/// Reads `file`, blends the edges `ranges` name (index ranges in SMLib's file order, all others
/// when `include` is false) and checks the result.
fn blend(file: &str, ranges: &[(usize, usize)], include: bool, op: Op, e: &Expect) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let ids = pick_edges(&m.body, &edge_points(&m, ranges, include));
    let input = m.body.clone();
    let mut k = NativeKernel::new();
    let b = k.insert(m.body);
    let r = match op {
        Op::Fillet(r) => k.fillet(b, &ids, r),
        Op::Chamfer(d) => k.chamfer(b, &ids, ChamferSpec::EqualDistance(d)),
    }
    .map_err(|e| format!("{e:?}"))?;
    let out = k.get(r.bodies[0]).map_err(|e| format!("{e:?}"))?;
    check_result(&input, out, e)
}

/// Reads `file` and fillets each `(edge, r0, r1)`: radius r0 at the edge's start (SMLib's) to
/// r1 at its end, linearly. One radius for all is `fillet_with`, anything else
/// `fillet_variable`.
fn blend_laws(file: &str, laws: &[(usize, f64, f64)], e: &Expect) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let idx: Vec<usize> = laws.iter().map(|l| l.0).collect();
    let ids = edges(&m, &idx)?;
    if ids.len() != laws.len() {
        return Err("two laws on one edge".into());
    }
    let input = m.body.clone();
    let mut k = NativeKernel::new();
    let b = k.insert(m.body.clone());
    let r0 = laws[0].1;
    let r = if laws.iter().all(|l| l.1 == r0 && l.2 == r0) {
        k.fillet_with(b, &ids, &FilletSpec::radius(r0))
    } else {
        let fl: Vec<FilletLaw> = laws
            .iter()
            .zip(&ids)
            .map(|(&(i, a, c), id)| {
                // Along SMLib's edge: turned round when ours runs the other way.
                let ed = &m.body.edges[id.0 as usize];
                let s = m.body.verts[ed.v[0]].p;
                let (from, to) = m.edge_ends[i];
                let (a, c) = if (s - from).norm() <= (s - to).norm() { (a, c) } else { (c, a) };
                FilletLaw { edge: *id, radii: vec![(0.0, a), (1.0, c)] }
            })
            .collect();
        k.fillet_variable(b, &fl, true)
    }
    .map_err(|e| format!("{e:?}"))?;
    let out = k.get(r.bodies[0]).map_err(|e| format!("{e:?}"))?;
    check_result(&input, out, e)
}
'''

OUT = Out()

MODEL = re.compile(r'pt_TestFiles/([A-Za-z0-9_]+/[A-Za-z0-9_.]+\.(?:smb|smp))')


def top_args(s):
    """The comma-separated arguments of a call's argument text."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    out.append(cur.strip())
    return out


def flag(b, arg):
    """A boolean argument: a literal, or a local set to one in the block."""
    if arg in ("TRUE", "FALSE"):
        return arg == "TRUE"
    m = re.search(r"\b%s\s*=\s*(TRUE|FALSE)\b" % re.escape(arg), b)
    if not m:
        raise ValueError("can't resolve %s" % arg)
    return m.group(1) == "TRUE"


def total_edges(b):
    m = re.search(r"lTotalEdges\s*==\s*(\d+)", b)
    return int(m.group(1)) if m else None


def case_name(n, path):
    return "case_%03d_%s" % (n, os.path.splitext(os.path.basename(path))[0].lower())


def oracle_fillet(ctx, fx, picks, radius, chamfer=False):
    """The oracle's fillet (or chamfer) of the model's edges `picks` (indices)."""
    p = ctx.points(fx)
    if "error" in p:
        return None
    pts = []
    for i in picks:
        if i >= len(p["edges"]):
            return None
        pts += p["edges"][i]
    return ("chamfer" if chamfer else "fillet", [fx, OUT, float(radius)] + [float(x) for x in pts])


def model_case(ctx, n, b):
    """A `my_test_model` block."""
    call = re.search(r"my_test_model\((.*?)\)\);", b, re.S)
    f = MODEL.search(call.group(1)) if call else None
    r = re.search(r"dBallRadius\s*=\s*([0-9.eE+-]+)", b)
    k = re.search(r"lNumIntervals\s*=\s*(\d+)", b)
    e = total_edges(b)
    if not (call and f and r and k and e is not None):
        return None
    args = top_args(call.group(1))
    chamfer, bevel = flag(b, args[-2]), flag(b, args[-1])
    ivs = dict((int(a), int(v)) for a, v in re.findall(r"aIndexArr\[\s*(\d+)\]\s*=\s*(\d+)", b))
    idx = [(ivs[2 * j], ivs[2 * j + 1]) for j in range(int(k.group(1))) if 2 * j + 1 in ivs]
    include = "EXCLUDE_ARRAY" not in call.group(1)
    rad = num(r.group(1))
    path = "TestFiles/pt_TestFiles/" + f.group(1)
    fx = Fixture(path)
    name = os.path.basename(path)
    rs = ", ".join("(%d, %d)" % ab for ab in idx)
    op = "Op::%s(%s)" % ("Chamfer" if chamfer else "Fillet", rust_f64(rad))
    doc = "SMLib fillet test %d: `%s`, %s %s, edges %s%s%s." % (
        n, name, "chamfer" if chamfer else "radius", rad, "" if include else "all but ", idx,
        " (SMLib: bevelled corners)" if bevel else "")
    oracle = None
    p = ctx.points(fx)
    if "error" not in p:
        ne = len(p["edges"])
        picks = [i for i in range(ne) if any(a <= i <= c for a, c in idx) == include]
        oracle = oracle_fillet(ctx, fx, picks, rad, chamfer)
    return Case(case_name(n, path), doc,
                'blend("%s", &[%s], %s, %s, {expect})' % (name, rs, "true" if include else "false", op),
                [fx], {"edges": e}, oracle)


# --- Fillet definition files (`my_read_fillet_definition_file`) -----------------------------

def fdf_lines(text):
    """The file's lines without their `//` comments, blank ones dropped."""
    out = []
    for l in text.splitlines():
        l = l.split("//")[0].strip()
        if l:
            out.append(l)
    return out


def read_fdf(text):
    """A fillet definition file: (model path, expected edges, [definition]) for its one
    topology filleting operation, a definition being (kind, index, law) with kind "edge",
    "face" or "brep" and law ("const", r) or ("linear", r0, r1). Raises ValueError for what
    isn't read (surface filleting, several operations, distance or B-spline laws)."""
    toks = []
    for l in fdf_lines(text):
        toks.append(l.split())
    it = iter(toks)
    nops = int(next(it)[0])
    if nops != 1:
        raise ValueError("%d filleting operations" % nops)
    model = next(it)[0]
    op = next(it)
    if int(op[0]) != 2:
        raise ValueError("surface filleting")
    if any(float(x) != 0.0 for x in op[2:5]):
        raise ValueError("a translated model")
    expected = int(op[1])
    ndefs = int(next(it)[0])
    defs = []
    for _ in range(ndefs):
        kind = int(next(it)[0])
        if kind not in (1, 2, 3):
            raise ValueError("topology definition %d" % kind)
        line = next(it)
        vals = line[1:] if kind in (1, 2) else line
        index = int(line[0]) if kind in (1, 2) else None
        solver = int(vals[1])
        if solver == 2:
            law = ("const", float(vals[2]))
        elif solver == 3 and int(vals[2]) == 1:
            r0, r1 = float(vals[4]), float(vals[5])
            law = ("const", r0) if r0 == r1 else ("linear", r0, r1)
        else:
            raise ValueError("fillet solver %d (law %s)" % (solver, vals[2:4]))
        xs = next(it)  # cross section: 1 linear, 2 circular, 3 approximately circular
        if int(xs[0]) == 1:
            raise ValueError("a linear cross section")
        defs.append(({1: "edge", 2: "face", 3: "brep"}[kind], index, law))
    return model.replace("../../", ""), expected, defs


def fdf_case(ctx, n, b):
    m = re.search(r'my_read_fillet_definition_file\(\s*crContext\s*,\s*_T\("\.\./\.\./([^"]+\.fdf)"\)', b)
    if not m:
        return None
    fdf = m.group(1)
    model, expected, defs = read_fdf(open(ctx.path(fdf)).read())
    asserted = total_edges(b)
    fx = Fixture(model)
    name = os.path.basename(model)
    p = ctx.points(fx)
    laws = []
    for kind, index, law in defs:
        if kind == "edge":
            idx = [index]
        elif kind == "brep":
            idx = list(range(len(p["edges"])))
        else:
            raise ValueError("face definitions aren't read yet")
        for i in idx:
            laws.append((i,) + ((law[1], law[1]) if law[0] == "const" else (law[1], law[2])))
    ls = ", ".join("(%d, %s, %s)" % (i, rust_f64(a), rust_f64(c)) for i, a, c in laws)
    const = all(a == laws[0][1] and c == laws[0][1] for _, a, c in laws)
    doc = "SMLib fillet test %d: `%s` (`%s`), %s." % (
        n, os.path.basename(fdf), name,
        ", ".join("edge %d %s" % (i, ("radius %s" % a) if a == c else ("radius %s to %s" % (a, c))) for i, a, c in laws))
    oracle = oracle_fillet(ctx, fx, [l[0] for l in laws], laws[0][1]) if const else None
    return Case(case_name(n, model), doc, 'blend_laws("%s", &[%s], {expect})' % (name, ls),
                [fx], {"edges": asserted if asserted is not None else expected}, oracle)


# --- Blocks driving SmFilletExecutive directly ---------------------------------------------

def executive_case(ctx, n, b):
    """Blocks building fillet solvers by hand: per-edge radii (`alIndex`/`adBallRadii`), edges
    added to a list (`sEdgesForFillet.Add`), a loop over the edges with an index test, and
    linear laws (`SmLinearFilletLaw(r0, r1)`)."""
    f = MODEL.search(b)
    if not f or "SmFilletExecutive" not in b:
        return None
    path = "TestFiles/pt_TestFiles/" + f.group(1)
    fx = Fixture(path)
    name = os.path.basename(path)
    p = ctx.points(fx)
    ne = len(p.get("edges", []))
    laws = []  # (edge, r0, r1)
    notes = []
    al = re.search(r"alIndex\[\]\s*=\s*\{([^}]*)\}", b)
    ar = re.search(r"adBallRadii\[\]\s*=\s*\{([^}]*)\}", b)
    adds = re.findall(r"sEdgesForFillet\.Add\(sEdges\[(\d+)\]\)", b)
    if al and ar:
        for i, r in zip(al.group(1).split(","), ar.group(1).split(",")):
            laws.append((int(i), num(r.strip()), num(r.strip())))
    elif adds:
        r = num(re.search(r"dBallRadius\s*=\s*([0-9.eE+-]+)", b).group(1))
        laws = [(int(i), r, r) for i in adds]
    else:
        # A loop over the edges: which ones, and the solver each gets.
        loop = b[b.find("for (ULONG i=0; i<sEdges.GetSize(); i++)"):]
        lin = re.findall(r"SmLinearFilletLaw\(\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*\)", loop)
        brk = re.search(r"if \(i > (\d+)\) break;", loop)
        nots = re.search(r"if \(i != (\d+) && i != (\d+)\) continue;", loop)
        rng = re.findall(r"if \(i >= (\d+) && i <= (\d+)\)", loop)
        one = re.findall(r"else if \(i == (\d+)\)", loop)
        radii = [num(x) for x in re.findall(r"dBallRadius\s*=\s*([0-9.eE+-]+)", loop)]
        if nots and lin:
            a, c = num(lin[0][0]), num(lin[0][1])
            laws = [(int(nots.group(1)), a, c), (int(nots.group(2)), a, c)]
        elif brk and radii:
            # `if (i > k) break;` with `dBallRadius = r0; if (i>0) dBallRadius = r1;`
            for i in range(int(brk.group(1)) + 1):
                r = radii[1] if (i > 0 and len(radii) > 1) else radii[0]
                laws.append((i, r, r))
        elif rng and one and lin:
            r = num(re.search(r"ConstantRadiusFS\([^;]*?,\s*([0-9.eE+-]+)\s*,\s*pEU\)", loop, re.S).group(1))
            a, c = int(rng[0][0]), int(rng[0][1])
            laws = [(i, r, r) for i in range(a, c + 1)] + [(int(one[0]), num(lin[0][0]), num(lin[0][1]))]
        elif radii and not lin:
            # Every edge, one radius.
            laws = [(i, radii[0], radii[0]) for i in range(ne)]
            if "SetSetBackDist" in b:
                notes.append("SMLib sets its corners back (`SetSetBackDist`); here plain corners")
        else:
            return None
    if not laws:
        return None
    ls = ", ".join("(%d, %s, %s)" % (i, rust_f64(a), rust_f64(c)) for i, a, c in laws)
    shown = ", ".join("%d: %s" % (i, a if a == c else "%s to %s" % (a, c)) for i, a, c in laws) if len(laws) <= 8 else "all, radius %s" % laws[0][1]
    doc = "SMLib fillet test %d: `%s`, radii by edge {%s}%s." % (n, name, shown, "".join("; " + x for x in notes))
    const = all(a == laws[0][1] and c == laws[0][1] for _, a, c in laws)
    oracle = oracle_fillet(ctx, fx, [l[0] for l in laws], laws[0][1]) if const and "error" not in p else None
    return Case(case_name(n, path), doc, 'blend_laws("%s", &[%s], {expect})' % (name, ls),
                [fx], {"edges": total_edges(b)}, oracle)


def extract(ctx):
    src = ctx.source(SOURCE)
    cases = []
    for n, b in if_blocks(src):
        try:
            if "my_test_model(" in b:
                c = model_case(ctx, n, b)
            elif "my_read_fillet_definition_file" in b:
                c = fdf_case(ctx, n, b)
            elif "SmFilletExecutive" in b:
                c = executive_case(ctx, n, b)
            else:
                c = None
        except ValueError as e:
            print("fillets: block %d not converted: %s" % (n, e), file=sys.stderr)
            c = None
        if c is not None:
            cases.append(c)
    return cases
