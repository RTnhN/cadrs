"""SMLib's advanced sectioning tests (tests/prog_test/src/smAdvancedTest.cpp, `my_test_sec_adv`,
the `section-advanced` suite): `torus.smb`, moved, cut by 7 planes square to one axis through the
points `box.Evaluate(t, t, t)`, t = i/6, of its bounding box (`my_test_tsurf_section`,
smTopoTest.cpp), counting the trimmed section curves.

Here each plane is the top face of a slab through the same point (sections take solids) and the
section is `boolean::section` (through the DRAW harness's `bsection`), keeping the edges on that
face. Each plane checks:
- every section edge lies on the plane and on the torus;
- the edges join into closed loops, as many as the plane's section has (a range where the plane
  is within 1e-5 of touching the torus's inner equator: a figure eight or one loop);
- their total length against the closed form.

The closed form, not the oracle's length: SMLib's own counts (the oracle gives 9, 0 and 12
curves for normals y, z and x) disagree with the test's asserts (14 or 16, 13, 16 or 17), and
its lengths aren't the section's: `SmFace::CreateCurvesByTrimming` drops curves on this one-face
torus (none at all for the planes square to its axis, which cut two circles of total length 4πR,
and none for the planes through its axis). torus.smb is R 10, r 5 about z through the origin.
"""
import math
import re

from smlib_to_rust import Case, Fixture, drop_dead, rust_f64, strip_comments
from smlib._oracle import oracle_run

SOURCE = "smAdvancedTest.cpp"
TITLE = "advanced sectioning"

# torus.smb: major and minor radius, about z through the origin.
R_MAJ, R_MIN = 10.0, 5.0

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;

use cadrs_kernel::native::draw::{Ax3, DEdge, Shape};
use cadrs_kernel::native::math::{any_perpendicular, Affine, P3, V3};
use cadrs_kernel::native::NativeKernel;

const SUITE: &str = "section";

/// One cutting plane: the point it passes through, the section's length and the range of its
/// loop count.
struct Plane {
    at: [f64; 3],
    length: f64,
    loops: (usize, usize),
}

/// The closed chains the edges make (ends within `tol`); an open chain is an error.
fn loops(edges: &[DEdge], tol: f64) -> Result<usize, String> {
    // Union-find over the edges, joined where their ends meet.
    let n = edges.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(p: &mut Vec<usize>, i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        p[i] = r;
        r
    }
    let ends: Vec<[P3; 2]> = edges.iter().map(|e| [e.start(), e.end()]).collect();
    let mut degree = vec![[0usize; 2]; n];
    for i in 0..n {
        for a in 0..2 {
            if (ends[i][a] - ends[i][1 - a]).norm() <= tol && a == 0 {
                // A closed edge: its ends meet each other.
                degree[i] = [1, 1];
            }
            for j in 0..n {
                if j == i {
                    continue;
                }
                for b in 0..2 {
                    if (ends[i][a] - ends[j][b]).norm() <= tol {
                        degree[i][a] += 1;
                        let (ri, rj) = (root(&mut parent, i), root(&mut parent, j));
                        parent[ri] = rj;
                    }
                }
            }
        }
    }
    if let Some(i) = (0..n).find(|&i| degree[i].iter().any(|&d| d == 0)) {
        return Err(format!("section edge {i} ends in the air ({:?} – {:?})", ends[i][0], ends[i][1]));
    }
    let mut roots: Vec<usize> = (0..n).map(|i| root(&mut parent, i)).collect();
    roots.sort_unstable();
    roots.dedup();
    Ok(roots.len())
}

/// SMLib's `my_test_tsurf_section`: `file` moved by `mv`, cut by planes square to `n`.
fn section(file: &str, mv: [f64; 3], n: [f64; 3], planes: &[Plane]) -> Result<(), String> {
    let m = model(SUITE, file)?;
    let body = m.body.transformed(&Affine::translation(V3::new(mv[0], mv[1], mv[2])));
    let surfs: Vec<_> = body.faces.iter().map(|f| f.surf.clone()).collect();
    let size = body.bbox().diagonal();
    let mut k = NativeKernel::new();
    let solid = Shape::Body(k.insert(body));
    let n = V3::new(n[0], n[1], n[2]).normalize();
    let x = any_perpendicular(&n).normalize();
    let y = n.cross(&x);
    let mut errs = Vec::new();
    for (i, pl) in planes.iter().enumerate() {
        let o = P3::new(pl.at[0], pl.at[1], pl.at[2]);
        let c = |a: f64, b: f64| {
            let q = o + x * a + y * b;
            [q.x, q.y, q.z]
        };
        // The plane as the top face of a slab behind it (sections take solids); only the
        // section edges on that face count.
        let corner = c(-size, -size);
        let ax = Ax3 { origin: P3::new(corner[0], corner[1], corner[2]) - n * size, z: n, x };
        let slab = k.box_on(&ax, 2.0 * size, 2.0 * size, size).map_err(|e| format!("{e:?}"))?;
        let edges = match k.bsection(&solid, &slab) {
            Ok(Shape::Wire(es)) => es
                .into_iter()
                .filter(|e| (e.curve.point(0.5 * (e.t0 + e.t1)) - o).dot(&n).abs() <= 1e-6 * size)
                .collect(),
            Ok(Shape::Empty) => vec![],
            Ok(other) => return Err(format!("plane {i}: a section of {other:?}")),
            Err(e) => {
                errs.push(format!("plane {i}: {e:?}"));
                continue;
            }
        };
        // On the plane and on the torus.
        let mut off = None;
        'edges: for (j, e) in edges.iter().enumerate() {
            for s in 0..=8 {
                let t = e.t0 + (e.t1 - e.t0) * s as f64 / 8.0;
                let p = e.curve.point(t);
                let dp = (p - o).dot(&n).abs();
                let ds = surfs.iter().map(|s| intersect::dist(s, &p)).fold(f64::INFINITY, f64::min);
                if dp > 1e-6 * size || ds > intersect::ON_TOL {
                    off = Some(format!("plane {i}, edge {j}: {p:?} is {dp:.1e} off the plane and {ds:.1e} off the model"));
                    break 'edges;
                }
            }
        }
        if let Some(o) = off {
            errs.push(o);
            continue;
        }
        let len: f64 = edges.iter().map(|e| e.length()).sum();
        if (len - pl.length).abs() > 1e-3 * pl.length.max(1.0) {
            errs.push(format!("plane {i}: section length {len:.6}, the closed form gives {:.6}", pl.length));
        }
        match loops(&edges, 1e-6 * size) {
            Ok(nl) if nl < pl.loops.0 || nl > pl.loops.1 => errs.push(format!("plane {i}: {nl} loops, want {}..={}", pl.loops.0, pl.loops.1)),
            Ok(_) => {}
            Err(e) => errs.push(format!("plane {i}: {e}")),
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs.join("; ")) }
}
'''


def branch_length(R, r, c, n=200000):
    """Length of the x ≥ 0 half of the section of the torus (R, r about z) by the plane y = c:
    over the torus's v, x = √((R + r cos v)² − c²), z = r sin v, where defined."""
    k = (abs(c) - R) / r
    if k >= 1.0:
        return 0.0
    if k <= -1.0:
        vs = [2 * math.pi * i / n for i in range(n + 1)]
    else:
        a = math.acos(k)
        # Clustered towards the ends, where x turns back like a square root.
        vs = [-a * math.cos(math.pi * i / n) for i in range(n + 1)]
    pts = []
    for v in vs:
        rho = R + r * math.cos(v)
        pts.append((math.sqrt(max(rho * rho - c * c, 0.0)), r * math.sin(v)))
    return sum(math.hypot(q[0] - p[0], q[1] - p[1]) for p, q in zip(pts, pts[1:]))


def plane_section(R, r, axis, off):
    """(length, (min loops, max loops)) of the section of the torus (R, r about z) by the plane
    square to `axis` (0, 1, 2) at `off` from its centre."""
    near = 1e-4
    if axis == 2:
        if abs(off) >= r:
            return 0.0, (0, 1 if abs(abs(off) - r) < near else 0)
        return 4 * math.pi * R, (2, 2)
    c = abs(off)
    if c >= R + r:
        return 0.0, (0, 1 if c - (R + r) < near else 0)
    length = 2 * branch_length(R, r, c)
    if c < R - r - near:
        return length, (2, 2)
    if c <= R - r + near:
        return length, (1, 2)
    return length, (1, 1)


def extract(ctx):
    src = ctx.source(SOURCE)
    body = strip_comments(src[src.find("my_test_sec_adv()"):src.find("end my_test_sec_adv")])
    blocks = re.findall(
        r'ReadFromFile\(\s*sContext\s*,\s*_T\(\s*"\.\./\.\./(TestFiles/pt_TestFiles/(\w+)\.smb)"\s*\)[^;]*;.*?'
        r'Translate\(\s*SmVector3d\(([^)]*)\)\s*\).*?sVec\(([^)]*)\).*?my_test_tsurf_section\([^,]+,\s*([0-9.eE+-]+)\s*,\s*sVec\s*,\s*(\d+)\s*,.*?SM_ASSERT\(([^;]*)\);',
        body, re.S)
    cases = []
    for i, (path, stem, mv, vec, tol, k, asserted) in enumerate(blocks, 1):
        mv = [float(x) for x in mv.split(",")]
        n = [float(x) for x in vec.split(",")]
        k = int(k)
        axis = max(range(3), key=lambda j: abs(n[j]))
        r = oracle_run(["section", ctx.path(path), n[0], n[1], n[2], k, "--move=%s" % ",".join(repr(x) for x in mv)])
        if r and r.get("ok"):
            pts = [p["point"] for p in r["planes"]]
            smlib = "SMLib (the oracle) finds %d curves, lengths %s" % (r["count"], ", ".join("%.4g" % p["length"] for p in r["planes"]))
        else:
            # The box without the oracle's: the torus's own.
            lo = [mv[0] - R_MAJ - R_MIN, mv[1] - R_MAJ - R_MIN, mv[2] - R_MIN]
            hi = [mv[0] + R_MAJ + R_MIN, mv[1] + R_MAJ + R_MIN, mv[2] + R_MIN]
            pts = [[lo[j] + (hi[j] - lo[j]) * t / (k - 1) for j in range(3)] for t in range(k)]
            smlib = "no oracle"
        planes = []
        for p in pts:
            length, (lo_, hi_) = plane_section(R_MAJ, R_MIN, axis, p[axis] - mv[axis])
            planes.append("Plane { at: [%s], length: %s, loops: (%d, %d) }" % (", ".join(rust_f64(x) for x in p), rust_f64(length), lo_, hi_))
        doc = "SMLib advanced section %d: `%s.smb` moved by (%g, %g, %g), %d planes square to (%g, %g, %g); SMLib's test asserts `%s`, %s." % (
            i, stem, mv[0], mv[1], mv[2], k, n[0], n[1], n[2], " ".join(asserted.split()), smlib)
        cases.append(Case("section_%d_%s_%s" % (i, stem, "xyz"[axis]), doc,
                          'section("%s.smb", [%s], [%s], &[\n        %s,\n    ])' % (
                              stem, ", ".join(rust_f64(x) for x in mv), ", ".join(rust_f64(x) for x in n), ",\n        ".join(planes)),
                          [Fixture(path)]))
    return cases
