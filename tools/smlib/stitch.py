"""SMLib's stitching demo (tests/prog_test/src/smMergeTest.cpp:343, `my_stitching_demo`, the
`stitch` suite, not run by default): `fillet_ts.smb`, a solid's faces read unstitched, is stitched
with `StitchFaces` at 0.001 (which leaves lamina edges) then 0.005, `fillet_cut.smb` is read, and
the stitched part less the cylinder (`my_two_prim_test`).

Here stitching is `surfacing::sew_faces` (and `surfacing::sew` for the closed solid), and:
- at 0.001: valid, with no more lamina (one-face) edges than SMLib leaves (the oracle's `stitch`);
- at 0.001 then 0.005: no lamina edges, a valid closed solid with SMLib's volume and area;
- fillet_cut.smb reads as a valid solid;
- the stitched part less the cylinder: a valid solid whose volume lies between the part's less the
  cylinder's and the part's (SMLib's own difference can't be run on the stitched part).
"""
from smlib_to_rust import Case, Fixture, rust_f64
from smlib._oracle import oracle_run

SOURCE = "smMergeTest.cpp"
TITLE = "stitching"

PRELUDE = r'''
use cadrs_kernel::native::surfacing;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{BoolOp, Kernel};

const SUITE: &str = "stitch";

/// Edges used by one face only.
fn lamina(b: &Body) -> usize {
    b.edge_faces().iter().zip(&b.edges).filter(|(f, e)| !e.degenerate && f.len() == 1).count()
}

/// `file`'s faces sewn at `tol` (`StitchFaces`): valid, with at most `max_lamina` lamina edges.
fn stitch_open(file: &str, tol: f64, max_lamina: usize) -> Result<(), String> {
    let b = surfacing::sew_faces(model(SUITE, file)?.body, tol).map_err(|e| format!("{e:?}"))?;
    if let Some(i) = invalid(&b) {
        return Err(i);
    }
    let n = lamina(&b);
    if n > max_lamina {
        return Err(format!("{n} lamina edges, SMLib leaves {max_lamina}"));
    }
    Ok(())
}

/// `file`'s faces sewn at each of `tols` in turn, the last closing a solid (`surfacing::sew`).
fn stitched(k: &mut NativeKernel, file: &str, tols: &[f64]) -> Result<(Body, Body), String> {
    let input = model(SUITE, file)?.body;
    let (last, first) = tols.split_last().ok_or("no tolerances")?;
    let mut b = input.clone();
    for &t in first {
        b = surfacing::sew_faces(b, t).map_err(|e| format!("{e:?}"))?;
    }
    let id = k.insert(b);
    let r = surfacing::sew(k, &[id], *last).map_err(|e| format!("{e:?}"))?;
    let out = (**k.get(r.bodies[0]).map_err(|e| format!("{e:?}"))?).clone();
    Ok((input, out))
}

/// Stitched at `tols`: no lamina edges, a valid closed solid with SMLib's values `e`.
fn stitch_closed(file: &str, tols: &[f64], e: &Expect) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (input, out) = stitched(&mut k, file, tols)?;
    let n = lamina(&out);
    if n > 0 {
        return Err(format!("{n} lamina edges"));
    }
    check_result(&input, &out, e)
}

/// `file` reads as a valid solid.
fn read_solid(file: &str) -> Result<(), String> {
    let b = model(SUITE, file)?.body;
    check_result(&b, &b, &Expect::default())
}

/// The part stitched at `tols`, less `tool`.
fn stitch_then_cut(file: &str, tols: &[f64], tool: &str) -> Result<(), String> {
    let mut k = NativeKernel::new();
    let (_, part) = stitched(&mut k, file, tols)?;
    let cut = model(SUITE, tool)?.body;
    let (vp, vc) = (cadrs_kernel::native::mass::body_mass(&part).volume, cadrs_kernel::native::mass::body_mass(&cut).volume);
    let (a, b) = (k.insert(part.clone()), k.insert(cut));
    let r = k.boolean(BoolOp::Subtract, a, &[b]).map_err(|e| format!("{e:?}"))?;
    let [one] = r.bodies.as_slice() else { return Err(format!("{} bodies", r.bodies.len())) };
    let out = (**k.get(*one).map_err(|e| format!("{e:?}"))?).clone();
    check_result(&part, &out, &Expect::default())?;
    let v = cadrs_kernel::native::mass::body_mass(&out).volume;
    if !(v >= vp - vc - 1e-6 * vp && v < vp) {
        return Err(format!("volume {v:.6}: not between the part's less the cylinder's ({:.6}) and the part's ({vp:.6})", vp - vc));
    }
    Ok(())
}
'''


def extract(ctx):
    src = ctx.source(SOURCE)
    body = src[src.find("PT_EXPORT SmStatus my_stitching_demo"):src.find("end my_stitching_demo")]
    assert "fillet_ts.smb" in body and "fillet_cut.smb" in body and "StitchFaces( 0.001" in body and "StitchFaces( 0.005" in body
    ts = Fixture("TestFiles/pt_TestFiles/Solids/fillet_ts.smb")
    cut = Fixture("TestFiles/pt_TestFiles/Solids/fillet_cut.smb")
    r = oracle_run(["stitch", ctx.path(ts.rel), 0.001, 0.005])
    lam1, expect = None, {}
    if r and r.get("ok"):
        lam1 = r["runs"][0]["lamina"]
        expect = {"volume": r["volume"], "area": r["area"], "solid": r["solid"]}
    e = "&Expect { %s..Expect::default() }" % "".join("%s: Some(%s), " % (k, rust_f64(v) if k != "solid" else ("true" if v else "false")) for k, v in expect.items())
    return [
        Case("stitch_1_fillet_ts_0001", "SMLib stitching 1: `fillet_ts.smb` stitched at 0.001 (SMLib leaves %s lamina edges)." % (lam1 if lam1 is not None else "some"),
             'stitch_open("fillet_ts.smb", 0.001, %d)' % (lam1 if lam1 is not None else 1 << 20), [ts]),
        Case("stitch_2_fillet_ts_0005", "SMLib stitching 2: `fillet_ts.smb` stitched at 0.001 then 0.005: closed%s." % (
            "" if not expect else ", SMLib's volume %.9g and area %.9g" % (expect["volume"], expect["area"])),
             'stitch_closed("fillet_ts.smb", &[0.001, 0.005], %s)' % e, [ts]),
        Case("stitch_3_read_fillet_cut", "SMLib stitching 3: `fillet_cut.smb` (the cylinder cut from the stitched part) reads as a solid.",
             'read_solid("fillet_cut.smb")', [cut]),
        Case("stitch_4_cut", "SMLib stitching 4: the stitched `fillet_ts.smb` less `fillet_cut.smb` (`my_two_prim_test`).",
             'stitch_then_cut("fillet_ts.smb", &[0.001, 0.005], "fillet_cut.smb")', [ts, cut]),
    ]
