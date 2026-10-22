"""SMLib's advanced silhouette tests (tests/prog_test/src/smAdvancedTest.cpp, `my_test_sil_adv`,
the `silhouette-advanced` suite): 34 cases on 7 models (sphere, cone, two planes, two tori, a
vase), each moved, whose faces' silhouette curves (`CreateSilhouetteCurves` on the face's uv
domain, then `CreateCurvesByTrimming`; `my_test_tsurf_silhouette`, smTopoTest.cpp) are counted,
for a direction of sight (25 cases) or an eye point (9 cases, `bPerspective`).

Here `Kernel::silhouettes` (native::silhouette) gives every face's silhouette curves, seen or
hidden, for the same parallel or perspective view. It checks:
- every point of the curves is its face's surface at the (u, v) given with it (1e-6), where the
  normal is square to the line of sight (|n·w| ≤ 1e-3, w the direction of sight, or from the
  eye point to the point; at a cone's apex there is no normal);
- the curves' total length against SMLib's (the oracle's `silhouette`), to 1e-3;
- none where SMLib finds none (the count is compared only as none against some: SMLib counts
  trimmed pieces, and the sphere's count is unconfirmed, 2 against the asserted 5).
A face seen edge-on (the planes seen along their edges) lies wholly on its silhouette: both give
the sides of its uv box that lie in it (the square's four sides, nothing of the disc).
"""
import re

from smlib_to_rust import Case, Fixture, rust_f64, strip_comments
from smlib._oracle import oracle_run, want_literal

SOURCE = "smAdvancedTest.cpp"

# Passing cases that take over a minute.
SLOW = set()
TITLE = "advanced silhouette"

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use cadrs_kernel::native::math::{Affine, P3, V3};
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{Kernel, SilhouetteView};

const SUITE: &str = "silhouette";

/// The curves' chord tolerance (mm).
const TOL: f64 = 1e-4;
/// A point of a curve lies on its face's surface within this.
const ON: f64 = 1e-6;
/// And the surface's normal there is this square to the line of sight (|n·w|).
const SQUARE: f64 = 1e-3;

/// SMLib's `my_test_tsurf_silhouette`: `file` moved by `mv`, seen along `view` (or from the eye
/// point `view` when `persp`).
fn silhouette(file: &str, mv: [f64; 3], view: [f64; 3], persp: bool, want: &Want) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let body = m.body.transformed(&Affine::translation(V3::new(mv[0], mv[1], mv[2])));
    let (eye, dir) = (P3::new(view[0], view[1], view[2]), V3::new(view[0], view[1], view[2]).normalize());
    let v = if persp { SilhouetteView::Perspective(eye) } else { SilhouetteView::Parallel(dir) };
    let mut k = NativeKernel::new();
    let id = k.insert(body.clone());
    let curves = k.silhouettes(id, &v, TOL).map_err(|e| format!("{e:?}"))?;
    let mut found = Found::default();
    for (j, c) in curves.iter().enumerate() {
        let s = &body.faces[c.face.0 as usize].surf;
        if c.uv.len() != c.points.len() {
            return Err(format!("curve {j}: {} points, {} (u, v)", c.points.len(), c.uv.len()));
        }
        for (p, q) in c.points.iter().zip(&c.uv) {
            let gap = (s.eval(q.x, q.y) - p).norm();
            let sight = if persp { (p - eye).normalize() } else { dir };
            // No normal at an apex or a pole (where a generator of a cone ends).
            let (_, su, sv) = s.d1(q.x, q.y);
            let apex = su.cross(&sv).norm() <= 1e-9 * su.norm_squared().max(sv.norm_squared());
            let nd = if apex { 0.0 } else { s.normal(q.x, q.y).dot(&sight).abs() };
            if gap > ON || nd > SQUARE {
                return Err(format!("curve {j} (face {}): {p:?} is {gap:.1e} off its surface, n·w {nd:.2e}", c.face.0));
            }
        }
        found.length += c.length();
        found.pieces.push(c.points.clone());
    }
    check_found(&found, want)
}
'''


def function(src, name):
    i = src.find(name + "(void)")
    return src[i:src.find("end " + name, i)]


def extract(ctx):
    src = ctx.source(SOURCE)
    body = strip_comments(function(src, "my_test_sil_adv"))
    blocks = re.findall(
        r'ReadFromFile\(\s*sContext\s*,\s*_T\("\.\./\.\./(TestFiles/pt_TestFiles/(\w+)\.smb)"\)[^;]*;.*?'
        r'Translate\(\s*SmVector3d\(([^)]*)\)\s*\).*?bPerspective\s*=\s*(TRUE|FALSE)\s*;.*?sVec\(([^)]*)\).*?'
        r'my_test_tsurf_silhouette\(.*?SM_ASSERT\(([^;]*)\);',
        body, re.S)
    cases = []
    for i, (path, stem, mv, persp, vec, asserted) in enumerate(blocks, 1):
        mv = [float(x) for x in mv.split(",")]
        v = [float(x) for x in vec.split(",")]
        persp = persp == "TRUE"
        counts = [int(x) for x in re.findall(r"lNumFound\s*==\s*(\d+)", asserted)]
        args = ["silhouette", ctx.path(path), v[0], v[1], v[2], "--move=%s" % ",".join(repr(x) for x in mv)] + (["--persp"] if persp else [])
        r = oracle_run(args)
        length, note = None, None
        if r and r.get("ok"):
            length = r["length"]
            if r["count"] not in counts:
                note = "SMLib's test asserts %s, the oracle finds %d curves" % (" or ".join(map(str, counts)), r["count"])
        elif r is not None:
            note = "SMLib (the oracle) fails: %s" % r.get("status", "?")
        if all(c > 0 for c in counts):
            any_ = True
        elif all(c == 0 for c in counts):
            any_ = False
        else:
            # `0 or 1`: SMLib's own run decides.
            any_ = bool(r and r.get("ok") and r["count"] > 0)
        doc = "SMLib advanced silhouette %d: `%s.smb` moved by (%g, %g, %g), %s (%g, %g, %g); SMLib's test asserts %s%s." % (
            i, stem, mv[0], mv[1], mv[2], "eye at" if persp else "seen along", v[0], v[1], v[2],
            " or ".join(map(str, counts)), "" if length is None else ", SMLib's length %.6g" % length)
        c = Case("silhouette_%02d_%s_%s" % (i, stem, "persp" if persp else "par"), doc,
                 'silhouette("%s.smb", [%s], [%s], %s, %s)' % (
                     stem, ", ".join(rust_f64(x) for x in mv), ", ".join(rust_f64(x) for x in v), "true" if persp else "false",
                     want_literal(any_, length, [])),
                 [Fixture(path)])
        c.note = note
        c.slow = c.name in SLOW
        cases.append(c)
    return cases
