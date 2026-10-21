"""SMLib's advanced surface–surface intersection tests (tests/prog_test/src/smAdvancedTest.cpp,
`my_test_ssi_adv`, the `ssi-advanced` suite): `my_test_one_ssi(file.smp, translation, count)`
reads a part's two B-reps, moves them and intersects every face of the first with every face of
the second (`GlobalSurfaceIntersect` on the faces' uv boxes, `my_test_ssi`, smTopoTest.cpp).

Here each face pair goes through `ssi::surface_surface_points` (tests/smlib_common/intersect.rs)
and the case checks:
- every point of every curve lies on both surfaces;
- the total length of the curves inside both faces against SMLib's (the oracle's `ssi`), to 1e-3;
- every sample point of SMLib's curves (8 per curve) lies near one of ours;
- the count only as none against some: SMLib splits curves at seams and returns degenerate
  curves and coincident regions' boundaries its own way. The count is the test's without
  `USE_ANALYTICS` (the analytic intersectors' counts differ); ssi_tor_purpen_tor_sing's
  asserted 14 is stale (SMLib's golden run finds 6), some either way.
"""
import os
import re

from smlib_to_rust import Case, Fixture, drop_dead, strip_comments
from smlib._oracle import oracle_run, want_literal

SOURCE = "smAdvancedTest.cpp"
TITLE = "advanced surface-surface intersection"

NOT_CONVERTED = [
    ("ssi_bumppy_plane.smp (84 curves)", "`if (FALSE)` in SMLib's test: \"extremely slow and missing one segment\""),
]

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use cadrs_kernel::native::math::{Affine, V3};
use cadrs_kernel::native::smb;

const SUITE: &str = "ssi_advanced";

/// SMLib's `my_test_one_ssi`: the part's two B-reps moved by `t`, every face of the first
/// intersected with every face of the second.
fn ssi(file: &str, t: [f64; 3], want: &Want) -> Result<(), String> {
    let bytes = std::fs::read(data(SUITE, file)).map_err(|e| format!("{file}: {e}"))?;
    let part = smb::read_part(&bytes)?;
    let [a, b, ..] = part.breps.as_slice() else { return Err(format!("{file}: {} B-reps", part.breps.len())) };
    let m = Affine::translation(V3::new(t[0], t[1], t[2]));
    let (a, b) = (a.body.transformed(&m), b.body.transformed(&m));
    let found = intersect_patches(&Patch::faces(&a), &Patch::faces(&b))?;
    check_found(&found, want)
}
'''


def preprocess(code, defined=()):
    """`code` with its `#ifdef`/`#if`/`#else`/`#endif` resolved: only the macros in `defined`
    are set (SMLib's test without USE_ANALYTICS, no graphics)."""
    out, stack = [], []
    for line in code.split("\n"):
        s = line.strip()
        m = re.match(r"#\s*(ifdef|ifndef|if)\b\s*(\w*)", s)
        if m:
            kind, name = m.groups()
            on = name in defined
            stack.append(on if kind in ("ifdef", "if") else not on)
            continue
        if re.match(r"#\s*else\b", s):
            stack[-1] = not stack[-1]
            continue
        if re.match(r"#\s*endif\b", s):
            stack.pop()
            continue
        if all(stack):
            out.append(line)
    return "\n".join(out)


def function(src, name):
    """The body of `name`'s definition in `src`."""
    i = src.find(name + "(void)")
    j = src.find("end " + name, i)
    return src[i:j]


def extract(ctx):
    src = ctx.source(SOURCE)
    body = drop_dead(strip_comments(preprocess(function(src, "my_test_ssi_adv"))))
    calls = re.findall(r'my_test_one_ssi\(_T\("\.\./\.\./(TestFiles/pt_TestFiles/(\w+)\.smp)"\)\s*,\s*SmVector3d\(([^)]*)\)\s*,\s*(\d+)\s*\)', body)
    cases = []
    for k, (path, stem, t, n) in enumerate(calls, 1):
        t = [float(x) for x in t.split(",")]
        count = int(n)
        if stem == "ssi_tor_purpen_tor_sing":
            count = 6  # The asserted 14 is stale (SMLib's golden run prints "Expected 14, Found 6").
        fx = Fixture(path)
        r = oracle_run(["ssi", ctx.path(path)] + t)
        length, samples, note = None, [], None
        if r and r.get("ok"):
            length = sum(c["length"] for c in r["curves"])
            samples = [p for c in r["curves"] for p in c["samples"]]
            if (r["count"] > 0) != (count > 0):
                note = "SMLib's test asserts %d curves, the oracle finds %d" % (count, r["count"])
        elif r is not None:
            note = "SMLib (the oracle) fails: %s" % r.get("status", "?")
        doc = "SMLib advanced SSI %d: `%s.smp` moved by (%g, %g, %g), %d curves asserted%s." % (
            k, stem, t[0], t[1], t[2], count, "" if length is None else ", SMLib's length %.6g" % length)
        c = Case("ssi_%02d_%s" % (k, stem[4:] if stem.startswith("ssi_") else stem), doc,
                 'ssi("%s.smp", [%s], %s)' % (stem, ", ".join("%r" % x for x in t), want_literal(count > 0, length, samples)),
                 [fx])
        c.note = note
        cases.append(c)
    return cases
