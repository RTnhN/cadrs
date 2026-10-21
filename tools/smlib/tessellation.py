"""SMLib's tessellation tests (tests/prog_test/src/smTessTest.cpp, `my_test_tess`, the
`tessellation` suite): `my_test_one_tess` meshes `nurb_fillet.smb` with four sets of driver
parameters (chord height, angle, maximum edge, maximum aspect) and asserts no face failed; the
stitching example meshes `fillet_ts.smb` after `StitchFaces` at 0.001 then 0.005 (chord 0.6,
40°, edges up to 4, aspect 4).

Here `tess::mesh_body` (chord and angle; it has no maximum edge or aspect ratio, and a chord of 0,
SMLib's "angle only", becomes 0.1 mm) and each case checks:
- no face without triangles;
- the mesh is watertight (its vertices welded, every edge used by two triangles, once each way);
- every triangle's centroid and edge middles lie within the chord tolerance of its face;
- the mesh volume against SMLib's mesh volume (the oracle's `tess`), within the two meshes'
  chord errors (the larger of the measured deviation and the chord, twice, times the area).

Not converted: the poly-solver, tri-strip, STL, PolyBrep merge and ray-firing tests (SMLib's
polygon library, which the kernel doesn't have).
"""
import re

from smlib_to_rust import Case, Fixture, strip_comments, rust_f64
from smlib._oracle import oracle_run, opt_f64

SOURCE = "smTessTest.cpp"
TITLE = "tessellation"

NOT_CONVERTED = [
    ("face/normal tables, poly solvers 1–4, CreateTriStripFaces, SimpleMerge, STL and .pbp input/output, ray firing, decimation, poly booleans (tests 1–4, 9–16)",
     "SMLib's polygon library (SmPolyBrep), which the kernel has no counterpart of"),
]

PRELUDE = r'''
use std::collections::HashMap;

use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::surfacing;
use cadrs_kernel::native::tess;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::Kernel;

const SUITE: &str = "tessellation";

/// The chord used where SMLib's is 0 (angle only).
const ANGLE_ONLY_CHORD: f64 = 0.1;

/// Distance from `p` to the surface (the better of two projections where it has no implicit
/// form).
fn dist(s: &Surface, p: &P3) -> f64 {
    if let Some(d) = s.signed_distance(p) {
        return d.abs();
    }
    let (u, v) = s.project(p, None);
    let d0 = (s.eval(u, v) - p).norm();
    let (u, v) = s.project(p, Some(s.project_coarse(p, 16)));
    d0.min((s.eval(u, v) - p).norm())
}

/// SMLib's `my_test_one_tess` on `file` (sewn first at `sew` tolerances, as SMLib's stitching
/// example), checked against SMLib's mesh volume `volume`.
fn tess(file: &str, sew: &[f64], chord: f64, angle_deg: f64, volume: Option<f64>) -> Result<(), String> {
    let mut body: Body = model(SUITE, file)?.body;
    if let Some((last, first)) = sew.split_last() {
        for &t in first {
            body = surfacing::sew_faces(body, t).map_err(|e| format!("{e:?}"))?;
        }
        // The last closes the solid (oriented outward).
        let mut k = NativeKernel::new();
        let id = k.insert(body);
        let r = surfacing::sew(&mut k, &[id], *last).map_err(|e| format!("{e:?}"))?;
        body = (**k.get(r.bodies[0]).map_err(|e| format!("{e:?}"))?).clone();
    }
    let defl = if chord > 0.0 { chord } else { ANGLE_ONLY_CHORD };
    let (faces, _) = tess::mesh_body(&body, defl, angle_deg.to_radians());
    if let Some(f) = faces.iter().position(|m| m.indices.is_empty()) {
        return Err(format!("face {f} has no triangles"));
    }
    // Weld and count directed edges.
    let size = body.bbox().diagonal();
    let q = |p: &P3| ((p.x / (1e-8 * size)).round() as i64, (p.y / (1e-8 * size)).round() as i64, (p.z / (1e-8 * size)).round() as i64);
    let mut ids: HashMap<(i64, i64, i64), usize> = HashMap::new();
    let mut directed: HashMap<(usize, usize), usize> = HashMap::new();
    let (mut vol, mut area, mut dev) = (0.0, 0.0, 0.0f64);
    for (fi, m) in faces.iter().enumerate() {
        let s = &body.faces[fi].surf;
        let vid: Vec<usize> = m.positions.iter().map(|p| {
            let n = ids.len();
            *ids.entry(q(p)).or_insert(n)
        }).collect();
        for t in &m.indices {
            let [a, b, c] = t.map(|i| m.positions[i as usize]);
            vol += a.coords.dot(&b.coords.cross(&c.coords)) / 6.0;
            area += (b - a).cross(&(c - a)).norm() / 2.0;
            for p in [P3::from((a.coords + b.coords + c.coords) / 3.0), nalgebra::center(&a, &b), nalgebra::center(&b, &c), nalgebra::center(&c, &a)] {
                dev = dev.max(dist(s, &p));
            }
            let [i, j, k] = t.map(|i| vid[i as usize]);
            for e in [(i, j), (j, k), (k, i)] {
                if e.0 != e.1 {
                    *directed.entry(e).or_insert(0) += 1;
                }
            }
        }
    }
    let mut open = 0;
    for (&(i, j), &n) in &directed {
        if n != 1 || directed.get(&(j, i)) != Some(&1) {
            open += 1;
        }
    }
    if open > 0 {
        return Err(format!("not watertight: {open} of {} mesh edges aren't used once each way", directed.len()));
    }
    if dev > defl * (1.0 + 1e-6) + 1e-9 {
        return Err(format!("the mesh is {dev:.3e} off the surfaces, the chord tolerance is {defl}"));
    }
    if let Some(v) = volume {
        let tol = 2.0 * dev.max(defl) * area;
        if (vol - v).abs() > tol {
            return Err(format!("mesh volume {vol:.6}, SMLib's mesh {v:.6} (tolerance {tol:.3e})"));
        }
    }
    Ok(())
}
'''


def extract(ctx):
    src = strip_comments(ctx.source(SOURCE))
    body = src[src.find("PT_EXPORT SmStatus my_test_tess()"):src.find("end my_test_tess")]
    cases = []
    # my_test_one_tess calls, each with the model its block read last.
    reads = list(re.finditer(r'ReadFromFile\(\s*sContext\s*,\s*_T\(\s*"\.\./\.\./(TestFiles/pt_TestFiles/Solids/(\w+)\.smb)"\s*\)', body))
    calls = re.finditer(r'my_test_one_tess\(\s*pBrep1\s*,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,', body)
    for k, m in enumerate(calls, 1):
        path, stem = [r for r in reads if r.start() < m.start()][-1].groups()
        ch, ang, maxe, maxa = (float(x) for x in m.groups())
        r = oracle_run(["tess", ctx.path(path), ch, ang, maxe, maxa])
        vol = r["volume"] if r and r.get("ok") and not r.get("failed") else None
        doc = "SMLib tessellation %d: `%s.smb`, chord %g, angle %g°, max edge %g, max aspect %g (SMLib: no failed faces%s)." % (
            k, stem, ch, ang, maxe, maxa, "" if vol is None else ", mesh volume %.6g" % vol)
        if maxe or maxa:
            doc += " The kernel's mesher has no maximum edge or aspect ratio: meshed by chord and angle only."
        cases.append(Case("tess_%d_%s" % (k, stem), doc, 'tess("%s.smb", &[], %s, %s, %s)' % (stem, rust_f64(ch), rust_f64(ang), opt_f64(vol)), [Fixture(path)]))
    # The stitching example.
    m = re.search(r'ReadFromFile\(\s*sContext\s*,\s*_T\(\s*"\.\./\.\./(TestFiles/pt_TestFiles/Solids/fillet_ts\.smb)"\s*\)', body)
    if m:
        path = m.group(1)
        r = oracle_run(["tess", ctx.path(path), 0.6, 40.0, 4.0, 4.0])
        vol = r["volume"] if r and r.get("ok") and not r.get("failed") else None
        doc = ("SMLib tessellation example: `fillet_ts.smb` stitched at 0.001 then 0.005 (`surfacing::sew_faces`), chord 0.6, angle 40°, "
               "max edge 4 and aspect 4 (not in the kernel's mesher); SMLib asserts no failed faces%s." % (
                   "" if vol is None else ", its mesh volume (unstitched) %.6g" % vol))
        cases.append(Case("tess_stitched_fillet_ts", doc, 'tess("fillet_ts.smb", &[0.001, 0.005], 0.6, 40.0, %s)' % opt_f64(vol), [Fixture(path)]))
    return cases
