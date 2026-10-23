"""SMLib's tessellation tests (tests/prog_test/src/smTessTest.cpp, `my_test_tess`, the
`tessellation` suite): `my_test_one_tess` meshes `nurb_fillet.smb` with four sets of driver
parameters (chord height, angle, maximum edge, maximum aspect) and asserts no face failed; the
stitching example meshes `fillet_ts.smb` after `StitchFaces` at 0.001 then 0.005 (chord 0.6,
40°, edges up to 4, aspect 4).

Here `tess::mesh` with the same controls (`TessParams`: chord, angle, maximum edge, maximum
aspect ratio; SMLib's chord of 0, "angle only", is an error to the kernel, so those cases mesh at
a chord of 0.1 mm) and each case checks:
- no face without triangles;
- the mesh is watertight (`tess::check_faces`: every side of every triangle shared by exactly
  two triangles, once each way);
- every triangle's centroid and side middles lie within the chord tolerance of its face;
- every triangle side is no longer than the maximum edge, when there is one;
- the mesh volume against SMLib's mesh volume (the oracle's `tess`), within the two meshes'
  chord errors (the larger of the measured deviation and the chord, twice, times the area).
The aspect ratio is reported (stderr), not checked: the kernel meets it by refinement that never
splits the boundary shared with the neighbouring face, so triangles on a long boundary segment
may keep a larger one.

The meshing stage of `my_test_decimation_regression` converts the same way (each model stitched
at 1e-5 first, SMLib's surface angle as the kernel's one angle); its decimation doesn't.

Not converted: the poly-solver, tri-strip, STL, PolyBrep merge and ray-firing tests (SMLib's
polygon library, which the kernel doesn't have).
"""
import os
import re

from smlib_to_rust import Case, Fixture, strip_comments, rust_f64
from smlib._oracle import oracle_run, opt_f64

SOURCE = "smTessTest.cpp"
TITLE = "tessellation"

NOT_CONVERTED = [
    ("face/normal tables, poly solvers 1–4, CreateTriStripFaces, SimpleMerge, STL and .pbp input/output, ray firing, decimation (its meshing converted), poly booleans (the smlib_poly_booleans suite) (tests 1–4, 9–16)",
     "SMLib's polygon library (SmPolyBrep), which the kernel has no counterpart of"),
]

PRELUDE = r'''
use rayon::prelude::*;

use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::surfacing;
use cadrs_kernel::native::tess;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::native::NativeKernel;

const SUITE: &str = "tessellation";

/// The chord used where SMLib's is 0 ("angle only"; a chord of 0 is an error to the kernel).
const ANGLE_ONLY_CHORD: f64 = 0.1;

/// Distance from `p` to the surface: exact where it has an implicit form, else by projection
/// from `hint` (the (u, v) of a mesh vertex nearby, from the mesher), and again from the other end of the range
/// when that stops on one (the foot across the seam of a closed surface).
fn dist(s: &Surface, p: &P3, hint: (f64, f64)) -> f64 {
    if let Some(d) = s.signed_distance(p) {
        return d.abs();
    }
    let ((u0, u1), (v0, v1)) = s.param_box();
    let mut starts = vec![hint];
    let mut best = f64::INFINITY;
    let mut k = 0;
    while k < starts.len() && k < 8 {
        let (u, v) = s.project(p, Some(starts[k]));
        best = best.min((s.eval(u, v) - p).norm());
        if u <= u0 || u >= u1 {
            starts.push((if u <= u0 { u1 } else { u0 }, v));
        }
        if v <= v0 || v >= v1 {
            starts.push((u, if v <= v0 { v1 } else { v0 }));
        }
        k += 1;
    }
    best
}

/// SMLib's `my_test_one_tess` on `file` (sewn first at `sew` tolerances, as SMLib's stitching
/// example) with its driver parameters (chord, angle in degrees, maximum edge and aspect, 0 for
/// none), checked against SMLib's mesh volume `volume`.
fn tess(file: &str, sew: &[f64], chord: f64, angle_deg: f64, max_edge: f64, max_aspect: f64, volume: Option<f64>) -> Result<(), String> {
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
    let params = tess::TessParams::new(defl, angle_deg.to_radians()).and_then(|p| p.with_limits(max_edge, max_aspect)).map_err(|e| e.to_string())?;
    let t = std::time::Instant::now();
    let (faces, _) = tess::mesh(&body, &params).map_err(|e| e.to_string())?;
    let elapsed = t.elapsed();
    if let Some(f) = faces.iter().position(|m| m.indices.is_empty()) {
        return Err(format!("face {f} has no triangles"));
    }
    let check = tess::check_faces(&faces);
    // A sheet's mesh is open along the sheet's border only.
    let sheet = !body.is_solid();
    if if sheet { check.bad > 0 } else { !check.is_watertight() } {
        return Err(format!("not watertight: of {} mesh edges {} are open and {} have more than two triangles or two the same way round", check.edges, check.open, check.bad));
    }
    // Per face: volume, area, deviation from the surface, longest side, worst aspect ratio.
    let per: Vec<[f64; 5]> = faces
        .par_iter()
        .enumerate()
        .map(|(fi, m)| {
            let s = &body.faces[fi].surf;
            let uv: Vec<(f64, f64)> = m.uvs.iter().map(|q| (q.x, q.y)).collect();
            let mut r = [0.0f64; 5];
            for t in &m.indices {
                let [a, b, c] = t.map(|i| m.positions[i as usize]);
                let [ia, ib, ic] = t.map(|i| i as usize);
                r[0] += a.coords.dot(&b.coords.cross(&c.coords)) / 6.0;
                let area = (b - a).cross(&(c - a)).norm() / 2.0;
                r[1] += area;
                // From each corner's (u, v) (a pole's is one of many), the nearest.
                for p in [P3::from((a.coords + b.coords + c.coords) / 3.0), nalgebra::center(&a, &b), nalgebra::center(&b, &c), nalgebra::center(&c, &a)] {
                    r[2] = r[2].max([ia, ib, ic].map(|h| dist(s, &p, uv[h])).into_iter().fold(f64::INFINITY, f64::min));
                }
                let l = [(b - a).norm(), (c - b).norm(), (a - c).norm()].into_iter().fold(0.0, f64::max);
                r[3] = r[3].max(l);
                if area > 0.0 {
                    r[4] = r[4].max(l * l / (2.0 * area));
                }
            }
            r
        })
        .collect();
    let vol: f64 = per.iter().map(|r| r[0]).sum();
    let area: f64 = per.iter().map(|r| r[1]).sum();
    let [dev, longest, aspect] = [2, 3, 4].map(|k| per.iter().map(|r| r[k]).fold(0.0, f64::max));
    let tris: usize = faces.iter().map(|m| m.indices.len()).sum();
    eprintln!("{file}: {tris} triangles in {elapsed:?}, deviation {dev:.3e}, longest side {longest:.3}, worst aspect ratio {aspect:.1}");
    if dev > defl * (1.0 + 1e-6) + 1e-9 {
        return Err(format!("the mesh is {dev:.3e} off the surfaces, the chord tolerance is {defl}"));
    }
    if max_edge > 0.0 && longest > max_edge * (1.0 + 1e-9) {
        return Err(format!("a triangle side is {longest:.4} long, the maximum edge is {max_edge}"));
    }
    if let Some(v) = volume {
        let tol = 2.0 * dev.max(defl) * area;
        if (vol - v).abs() > tol {
            return Err(format!("mesh volume {vol:.6}, SMLib's mesh {v:.6} (tolerance {tol:.3e})"));
        }
    }
    Ok(())
}

/// `my_test_cone_face_normal_table` (smTessTest.cpp:2322): a cone (radius 1 to 2, height 10)
/// over `start`..`end` degrees meshed at 10° (SMLib's `ConvertToPolyBrep`, chord 0 here 0.01):
/// every vertex of every face's mesh carries its face's outward normal (unit, the surface's
/// normal at the vertex's (u, v), turned by the face's sense), and the mesh is watertight.
fn cone_normals(start: f64, end: f64) -> Result<(), String> {
    use cadrs_kernel::native::draw::{Ax3, Shape};
    use cadrs_kernel::native::math::V3;
    let mut k = NativeKernel::new();
    let a = start.to_radians();
    let ax = Ax3 { origin: P3::origin(), z: V3::z(), x: V3::new(a.cos(), a.sin(), 0.0) };
    let Shape::Body(b) = k.pcone_a(Some(&ax), 1.0, 2.0, 10.0, end - start).map_err(|e| format!("{e:?}"))? else {
        return Err("the cone isn't a body".into());
    };
    let body: Body = (**k.get(b).map_err(|e| format!("{e:?}"))?).clone();
    let params = tess::TessParams::new(0.01, 10f64.to_radians()).map_err(|e| e.to_string())?;
    let (faces, _) = tess::mesh(&body, &params).map_err(|e| e.to_string())?;
    if !tess::check_faces(&faces).is_watertight() {
        return Err("not watertight".into());
    }
    for (fi, m) in faces.iter().enumerate() {
        let f = &body.faces[fi];
        if m.normals.len() != m.positions.len() || m.uvs.len() != m.positions.len() {
            return Err(format!("face {fi}: {} normals, {} uvs for {} vertices", m.normals.len(), m.uvs.len(), m.positions.len()));
        }
        for (i, n) in m.normals.iter().enumerate() {
            let q = m.uvs[i];
            let want = f.surf.normal(q.x, q.y) * if f.sense { 1.0 } else { -1.0 };
            if (n.norm() - 1.0).abs() > 1e-9 || (n - want).norm() > 1e-6 {
                return Err(format!("face {fi} vertex {i}: normal {n:?}, the face's {want:?}"));
            }
        }
    }
    Ok(())
}

/// `my_test_tess` test 3 (smTessTest.cpp:2442): `BoxFillet.stl` (ASCII) read and its triangles
/// welded into a valid closed solid (`meshsolid::mesh_solid`, at the kernel's point tolerance:
/// SMLib's merge tolerance of 1 would move vertices) whose volume is the file's triangles' signed
/// volume.
fn read_stl() -> Result<(), String> {
    let text = std::fs::read_to_string(data(SUITE, "BoxFillet.stl")).map_err(|e| e.to_string())?;
    let tris = cadrs_kernel::exchange::read_stl_ascii(&text);
    if tris.is_empty() {
        return Err("no triangles".into());
    }
    let want: f64 = tris.iter().map(|[a, b, c]| a.coords.dot(&b.coords.cross(&c.coords)) / 6.0).sum();
    let body = cadrs_kernel::native::meshsolid::mesh_solid(&tris, 0.0)?;
    if let Some(bad) = invalid(&body) {
        return Err(bad);
    }
    if !body.is_solid() {
        return Err("not a solid".into());
    }
    let v = cadrs_kernel::native::mass::body_mass(&body).volume;
    if (v - want.abs()).abs() > 1e-9 * want.abs() {
        return Err(format!("volume {v}, the triangles' {want}"));
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
        if not ch:
            doc += " Chord 0 (angle only) meshes at %g mm." % 0.1
        cases.append(Case("tess_%d_%s" % (k, stem), doc, 'tess("%s.smb", &[], %s, %s, %s, %s, %s)' % (stem, rust_f64(ch), rust_f64(ang), rust_f64(maxe), rust_f64(maxa), opt_f64(vol)), [Fixture(path)]))
    # The stitching example.
    m = re.search(r'ReadFromFile\(\s*sContext\s*,\s*_T\(\s*"\.\./\.\./(TestFiles/pt_TestFiles/Solids/fillet_ts\.smb)"\s*\)', body)
    if m:
        path = m.group(1)
        r = oracle_run(["tess", ctx.path(path), 0.6, 40.0, 4.0, 4.0])
        vol = r["volume"] if r and r.get("ok") and not r.get("failed") else None
        doc = ("SMLib tessellation example: `fillet_ts.smb` stitched at 0.001 then 0.005 (`surfacing::sew_faces`), chord 0.6, angle 40°, "
               "max edge 4 and aspect 4; SMLib asserts no failed faces%s." % (
                   "" if vol is None else ", its mesh volume (unstitched) %.6g" % vol))
        cases.append(Case("tess_stitched_fillet_ts", doc, 'tess("fillet_ts.smb", &[0.001, 0.005], 0.6, 40.0, 4.0, 4.0, %s)' % opt_f64(vol), [Fixture(path)]))
    cases += decimation_cases(ctx, src)
    m = re.search(r"my_test_cone_face_normal_table\(\s*0\.0\s*,\s*360\.0\s*\).*?my_test_cone_face_normal_table\(\s*30\.0\s*,\s*180\.0\s*\)", src, re.S)
    assert m, "the cone normal-table calls"
    for a, b in ((0.0, 360.0), (30.0, 180.0)):
        cases.append(Case("cone_normals_%d_%d" % (a, b), "SMLib `my_test_cone_face_normal_table(%g, %g)`: a cone meshed, every vertex with its face's outward normal." % (a, b),
                          "cone_normals(%s, %s)" % (rust_f64(a), rust_f64(b))))
    assert "BoxFillet.stl" in src
    cases.append(Case("stl_box_fillet", "SMLib `my_test_tess` test 3: `BoxFillet.stl` read and welded into a valid solid of the triangles' volume.",
                      "read_stl()", [Fixture("TestFiles/pt_TestFiles/Tess/BoxFillet.stl")]))
    return cases


def decimation_cases(ctx, src):
    """The meshing stage of `my_test_decimation_regression` (smTessTest.cpp:3850): each model
    stitched at SM_ZONE_TOL_3D and meshed (`ConvertToPolyBrep`) with the test's chord, curve and
    surface angles; the decimation that follows is SMLib's polygon library's. Calls meshing the
    same model the same way (quadric decimation or not) are one case."""
    body = src[src.find("SmStatus my_test_decimation_regression()"):src.find("end my_test_decimation_regression")]
    seen, cases = {}, []
    calls = re.finditer(r'my_test_decimation\(\s*sContext\s*,\s*_T\("\.\./\.\./(TestFiles/pt_TestFiles/[\w/]+\.smb)"\)\s*,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,\s*([0-9.eE+-]+)\s*,', body)
    for k, m in enumerate(calls, 1):
        path, ch, cang, sang = m.group(1), float(m.group(2)), float(m.group(3)), float(m.group(4))
        key = (path, ch, cang, sang)
        if key in seen:
            seen[key].doc = seen[key].doc.replace(" (SMLib", " (also its test %d; SMLib" % k, 1)
            continue
        stem = os.path.splitext(os.path.basename(path))[0]
        r = oracle_run(["tess", ctx.path(path), ch, sang])
        vol = r["volume"] if r and r.get("ok") and not r.get("failed") else None
        pts = ctx.points(Fixture(path))
        sheet = not pts.get("solid", True)
        if sheet:
            vol = None
        doc = ("SMLib decimation test %d, its meshing: `%s.smb` stitched at 1e-5, chord %g, curve angle %g°, surface angle %g° "
               "(SMLib: the mesh is made%s). The kernel meshes at one angle, the surface one%s." % (
                   k, stem, ch, cang, sang, "; a sheet, its mesh open along its border" if sheet else "" if vol is None else ", its volume %.6g at the surface angle" % vol,
                   "; chord 0 (angle only) at 0.1 mm" if not ch else ""))
        c = Case("tess_decimation_%d_%s" % (k, stem), doc, 'tess("%s.smb", &[%s], %s, %s, 0.0, 0.0, %s)' % (stem, "" if sheet else "1e-5", rust_f64(ch), rust_f64(sang), opt_f64(vol)), [Fixture(path)])
        seen[key] = c
        cases.append(c)
    assert cases, "no my_test_decimation calls found"
    return cases
