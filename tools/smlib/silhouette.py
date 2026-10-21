"""SMLib's advanced silhouette tests (tests/prog_test/src/smAdvancedTest.cpp, `my_test_sil_adv`,
the `silhouette-advanced` suite): 34 cases on 7 models (sphere, cone, two planes, two tori, a
vase), each moved, whose faces' silhouette curves (`CreateSilhouetteCurves` on the face's uv
domain, then `CreateCurvesByTrimming`; `my_test_tsurf_silhouette`, smTopoTest.cpp) are counted,
for a direction of sight (25 cases) or an eye point (9 cases, `bPerspective`).

The kernel has no silhouette curves of its own, only the outlines of a drawing view
(`Kernel::project`, native::hlr: projected to 2D, parallel views only). A parallel case projects
the model with hidden lines and lifts every outline point back to 3D: the point where the line of
sight through it touches the outline's face (where the line comes nearest the face with the
face's normal square to it: it also meets the face where it crosses it). It checks:
- every outline point lifts (the line of sight touches the face within 1e-3, and the normal there
  is square to the direction of sight, |n·v| ≤ 0.05);
- the lifted length against SMLib's (the oracle's `silhouette`), to 1e-3;
- none where SMLib finds none (the count is compared only as none against some: SMLib counts
  trimmed pieces, and the sphere's count is unconfirmed, 2 against the asserted 5).
A perspective case (n·(p − eye) = 0) calls `missing_perspective_silhouette`: the kernel has no
API for it, so those cases are known failures until it does.
"""
import re

from smlib_to_rust import Case, Fixture, rust_f64, strip_comments
from smlib._oracle import oracle_run, want_literal

SOURCE = "smAdvancedTest.cpp"

# Passing cases that take over a minute (the wavy torus's outline lifted point by point).
SLOW = {"silhouette_18_toruswvy_par"}
TITLE = "advanced silhouette"

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use cadrs_kernel::native::math::{any_perpendicular, Affine, P3, V3};
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::projection::{ProjClass, ProjectOptions, ViewFrame};
use cadrs_kernel::Kernel;

const SUITE: &str = "silhouette";

/// The outlines' 2D tolerance.
const PROJ_TOL: f64 = 1e-4;
/// A lifted point's line of sight touches its face within this.
const LIFT_TOL: f64 = 1e-3;
/// And the face's normal there is this square to it (|n·v|).
const SQUARE: f64 = 0.05;

/// MISSING API: silhouette curves seen from an eye point (n·(p − eye) = 0). `native::hlr`
/// projects along a direction only.
fn missing_perspective_silhouette() -> Result<Found, String> {
    Err("missing API: no silhouette curves for a perspective view (native::hlr projects along a direction only)".into())
}

/// Where the line `o + t·d`, t in [t0, t1], touches `s`: the t minimizing the distance plus a
/// measure of how far the surface's normal at the foot is from square to the line (the line
/// also meets the surface where it crosses it, at distance 0 too), `scale` (mm) weighing the
/// two alike near a tangency (distance ≈ δ²/2ρ, n·d ≈ δ/ρ). Returns (t, distance, |n·d|).
fn touch(s: &Surface, o: &P3, d: &V3, t0: f64, t1: f64, scale: f64) -> (f64, f64, f64) {
    let eval = |t: f64| {
        let p = o + d * t;
        let (u, v) = s.project(&p, Some(s.project_coarse(&p, 16)));
        let gap = dist(s, &p);
        let nd = s.normal(u, v).normalize().dot(d).abs();
        (gap + 0.5 * scale * nd * nd, gap, nd)
    };
    let n = 192;
    let h = (t1 - t0) / n as f64;
    let (mut best, mut fb) = (t0, f64::INFINITY);
    for i in 0..=n {
        let t = t0 + h * i as f64;
        let v = eval(t).0;
        if v < fb {
            (best, fb) = (t, v);
        }
    }
    // Golden section around the best sample.
    let (mut a, mut b) = (best - h, best + h);
    let g = 0.5 * (5f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (c, e) = (b - g * (b - a), a + g * (b - a));
        if eval(c).0 < eval(e).0 {
            b = e;
        } else {
            a = c;
        }
    }
    let t = if eval(0.5 * (a + b)).0 < fb { 0.5 * (a + b) } else { best };
    let (_, gap, nd) = eval(t);
    (t, gap, nd)
}

/// SMLib's `my_test_tsurf_silhouette`: `file` moved by `mv`, seen along `view` (or from the eye
/// point `view` when `persp`).
fn silhouette(file: &str, mv: [f64; 3], view: [f64; 3], persp: bool, want: &Want) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let body = m.body.transformed(&Affine::translation(V3::new(mv[0], mv[1], mv[2])));
    let found = if persp {
        missing_perspective_silhouette()?
    } else {
        let bb = body.bbox();
        let d = V3::new(view[0], view[1], view[2]).normalize();
        let frame = ViewFrame::new(d, any_perpendicular(&d));
        let mut k = NativeKernel::new();
        let id = k.insert(body.clone());
        let proj = k.project(&[id], &frame, &ProjectOptions { tolerance: PROJ_TOL, hidden: true }).map_err(|e| format!("{e:?}"))?;
        let up = frame.up();
        let corners: Vec<P3> = (0..8).map(|i| P3::new([bb.min.x, bb.max.x][i & 1], [bb.min.y, bb.max.y][(i >> 1) & 1], [bb.min.z, bb.max.z][i >> 2])).collect();
        let mut found = Found::default();
        for (j, e) in proj.edges.iter().filter(|e| e.class == ProjClass::Outline).enumerate() {
            let Some(f) = e.source.and_then(|s| s.face) else { return Err(format!("outline {j} has no face")) };
            let s = &body.faces[f.0 as usize].surf;
            let mut piece = Vec::new();
            for q in &e.points {
                let o = frame.origin + frame.x * q.x + up * q.y;
                let ts = corners.iter().map(|c| (c - o).dot(&d));
                let (t0, t1) = ts.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), t| (a.min(t), b.max(t)));
                let (t, gap, nd) = touch(s, &o, &d, t0 - 1.0, t1 + 1.0, 0.1 * bb.diagonal());
                let p = o + d * t;
                let (u, v) = s.project(&p, Some(s.project_coarse(&p, 16)));
                let foot = s.eval(u, v);
                if gap > LIFT_TOL || nd > SQUARE {
                    return Err(format!("outline {j} (face {}): the line of sight through {q:?} misses the face by {gap:.1e} (n·v {nd:.2e})", f.0));
                }
                piece.push(foot);
            }
            found.length += piece.windows(2).map(|w| (w[1] - w[0]).norm()).sum::<f64>();
            found.pieces.push(piece);
        }
        found
    };
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
