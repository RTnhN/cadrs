"""SMLib's polygon boolean regression (tests/prog_test/src/smTessTest.cpp:3559,
`my_poly_boolean_regression`, run by the `tessellation` suite through `my_test_tess`) on the
B-rep kernel: SMLib tessellates its inputs and takes the boolean of the meshes
(`SmPolyMerge::ManifoldBoolean`); the kernel takes the same boolean of the exact solids. SMLib's
test asserts only that each boolean succeeds; here each result is a valid closed solid with:
- `my_box_box_stress` test 1 (smTessTest.cpp:3211): a unit cube less a copy moved by (d, 0, 0.5),
  d = 0.000999 then divided by ten five times (down to 1e-8, under the kernel's point
  tolerance): the closed form 0.5 + 0.5 d;
- test 2: an L-shaped prism (an 8-point profile swept by (0, 0, −1)) less a 5 × 0.7 × 0.5 box:
  the closed form 5 − 0.42;
- test 3: a unit cube ∪ a copy moved by 0.5 along x: 1.5;
- test 5: each pair of `box_cyl_grid.smp` (B-reps 2i, 2i + 1) differenced, against SMLib's own
  B-rep difference (the oracle's `boolean … D`): volume, area, faces, edges. Pairs whose SMLib
  difference isn't a manifold solid are left out. The model is the booleans suite's copy
  (tests/data/smlib/booleans), which unions the same pairs.
"""
import os
import re

from smlib_to_rust import Case, Fixture, rust_f64, strip_comments
from smlib._oracle import oracle_run

SOURCE = "smTessTest.cpp"
TITLE = "polygon boolean"

NOT_CONVERTED = [
    ("test 4 (box_box_grid.smp unions) and test 6 (box_cone_grid.smp unions)", "the same unions as the booleans suite's `box_box_grid_*` and `box_cone_grid_*` cases"),
    ("test 7 (Tess/solid1.pbp − Tess/tool1.pbp)", "SMLib polygon B-rep files (.pbp): the kernel has no reader for them"),
    ("`my_box_box_stress` tests 2 and 3", "dead: test 1 returns first"),
]

PRELUDE = r'''
use cadrs_kernel::native::draw::{Ax3, Shape};
use cadrs_kernel::native::math::{P3, V3};
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BodyId, BoolOp, Kernel};

/// The suite whose copy of the grid models this one reads.
const GRID_SUITE: &str = "booleans";

fn e<E: std::fmt::Debug>(x: E) -> String {
    format!("{x:?}")
}

fn solid(s: Shape) -> Result<BodyId, String> {
    match s {
        Shape::Body(b) => Ok(b),
        _ => Err("not a solid".into()),
    }
}

/// An l × w × h box with its min corner at `at`.
fn cube(k: &mut NativeKernel, at: [f64; 3], l: f64, w: f64, h: f64) -> Result<BodyId, String> {
    let ax = Ax3 { origin: P3::new(at[0], at[1], at[2]), z: V3::z(), x: V3::x() };
    solid(k.box_on(&ax, l, w, h).map_err(e)?)
}

/// `a op b`, one valid solid of `volume` (when given: the value and the error allowed) or of
/// SMLib's values.
fn boolean(k: &mut NativeKernel, op: BoolOp, a: BodyId, b: BodyId, volume: Option<(f64, f64)>, x: &Expect) -> Result<(), String> {
    let input = (**k.get(a).map_err(e)?).clone();
    let r = k.boolean(op, a, &[b]).map_err(e)?;
    let [one] = r.bodies.as_slice() else { return Err(format!("{} bodies", r.bodies.len())) };
    let out = k.get(*one).map_err(e)?;
    check_result(&input, out, x)?;
    if let Some((v, tol)) = volume {
        let got = k.mass_properties(*one).map_err(e)?.volume;
        if (got - v).abs() > tol {
            return Err(format!("volume {got:.12}, the closed form {v:.12} (within {tol:.1e})"));
        }
    }
    Ok(())
}

/// `my_box_box_stress` test 1: the unit cube less a copy moved by (d, 0, 0.5). Faces closer than
/// the kernel's point tolerance (1e-6) are one face: the d × 0.5 slab between them may go, so
/// the volume is 0.5 + 0.5 d within 0.5 d there, within 1e-12 otherwise.
fn box_box_stress(d: f64) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (a, b) = (cube(&mut k, [0.0; 3], 1.0, 1.0, 1.0)?, cube(&mut k, [d, 0.0, 0.5], 1.0, 1.0, 1.0)?);
    let tol = if d <= cadrs_kernel::native::tol::POINT { 0.5 * d + 1e-12 } else { 1e-12 };
    boolean(&mut k, BoolOp::Subtract, a, b, Some((0.5 + 0.5 * d, tol)), &Expect::default())
}

/// Test 2: the L-shaped prism less a box.
fn l_prism() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let pts = [[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 2.0, 0.0], [2.0, 2.0, 0.0], [2.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 0.0]];
    let f = k.mkplane(&k.polyline(&pts).map_err(e)?).map_err(e)?;
    let a = solid(k.prism(&f, 0.0, 0.0, -1.0).map_err(e)?)?;
    let b = cube(&mut k, [-1.0, 1.2, -0.3], 5.0, 0.7, 0.5)?;
    // The profile's area is 5; the box takes 2 × 0.7 of it (x in [0, 1] and [2, 3]) 0.3 deep.
    boolean(&mut k, BoolOp::Subtract, a, b, Some((5.0 - 2.0 * 0.7 * 0.3, 1e-9)), &Expect::default())
}

/// Test 3: the unit cube ∪ a copy moved by 0.5 along x.
fn cubes() -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (a, b) = (cube(&mut k, [0.0; 3], 1.0, 1.0, 1.0)?, cube(&mut k, [0.5, 0.0, 0.0], 1.0, 1.0, 1.0)?);
    boolean(&mut k, BoolOp::Union, a, b, Some((1.5, 1e-9)), &Expect::default())
}

/// Test 5: B-rep 2i less B-rep 2i + 1 of the grid model `file`, against SMLib's difference.
fn grid_difference(file: &str, i: usize, x: &Expect) -> Result<(), String> {
    let (a, b) = (model(GRID_SUITE, &format!("{file}#{}", 2 * i))?.body, model(GRID_SUITE, &format!("{file}#{}", 2 * i + 1))?.body);
    let mut k = NativeKernel::new();
    let (ia, ib) = (k.insert(a), k.insert(b));
    boolean(&mut k, BoolOp::Subtract, ia, ib, None, x)
}
'''

GRID = "TestFiles/pt_TestFiles/Solids/box_cyl_grid.smp"


def extract(ctx):
    src = strip_comments(ctx.source(SOURCE))
    assert "my_poly_boolean_regression" in src and "box_cyl_grid.smp" in src
    # The grid model must still be the booleans suite's (this suite reads its copy).
    assert "box_cyl_grid" in ctx.source("smMergeTest.cpp"), "the booleans suite no longer copies box_cyl_grid.smp"
    cases = []
    m = re.search(r"double dMove = ([0-9.eE+-]+);\s*for \(ULONG i=0; i<(\d+); i\+\+\)", src)
    d0, n = float(m.group(1)), int(m.group(2))
    d = d0
    for i in range(n):
        cases.append(Case("box_box_stress_%d" % i, "SMLib `my_box_box_stress` test 1, step %d: the unit cube less a copy moved by (%g, 0, 0.5): volume 0.5 + 0.5 × %g." % (i, d, d),
                          "box_box_stress(%s)" % rust_f64(d)))
        d = d / 10.0
    cases.append(Case("l_prism_less_box", "SMLib `my_poly_boolean_regression` test 2: an L-shaped prism less a 5 × 0.7 × 0.5 box (4.58).", "l_prism()"))
    cases.append(Case("cube_union", "SMLib `my_poly_boolean_regression` test 3: the unit cube ∪ a copy moved by 0.5 (1.5).", "cubes()"))
    i = 0
    while True:
        a, b = Fixture("%s#%d" % (GRID, 2 * i)), Fixture("%s#%d" % (GRID, 2 * i + 1))
        r = oracle_run(["props", ctx.path(b.rel)])
        if r is None:
            print("poly_booleans: no oracle: box_cyl_grid differences left out", file=__import__("sys").stderr)
            break
        if not r.get("ok"):
            break
        res = oracle_run(["boolean", ctx.path(a.rel), ctx.path(b.rel), "D"])
        doc = "SMLib `my_poly_boolean_regression` test 5: `box_cyl_grid.smp` B-rep %d less B-rep %d." % (2 * i, 2 * i + 1)
        c = Case("box_cyl_grid_difference_%02d" % i, doc, 'grid_difference("box_cyl_grid.smp", %d, {expect})' % i)
        if res.get("ok") and res.get("solid") is not False:
            for k in ("volume", "area", "faces", "edges", "solid"):
                if k in res:
                    c.expect[k] = res[k]
            cases.append(c)
        else:
            NOT_CONVERTED.append(("test 5, `box_cyl_grid.smp` pair %d" % i, "SMLib's difference %s" % (
                "isn't a manifold solid" if res.get("ok") else "fails (%s)" % res.get("status"))))
        i += 1
    return cases
