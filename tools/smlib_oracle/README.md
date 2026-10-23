# SMLib oracle

A second, independent B-rep kernel for the native kernel's tests: SMLib from NVIDIA's
[usd-brep](https://github.com/NVIDIA-Omniverse/usd-brep) (Apache-2.0), built here with a
serial stand-in for the few oneTBB pieces it uses (`tbb_shim/`), so no TBB headers are needed.

    cmake -S tools/smlib_oracle -B target/smlib_oracle -DUSD_BREP_DIR=<usd-brep checkout> -DCMAKE_BUILD_TYPE=Release
    cmake --build target/smlib_oracle -j
    CADRS_SMLIB_ORACLE=$PWD/target/smlib_oracle/smlib_oracle cargo test -r -p cadrs_kernel --test smlib_oracle -- --nocapture

`-DSMLIB_MESSAGES=ON` builds SMLib with its error messages (each failure's file and line on
stderr with `ORACLE_VERBOSE=1`), in a separate build directory.

The native kernel writes its bodies as `.smb` (`native::oracle::smb_text`); `smlib_oracle`
reads them and prints one line of JSON. The commands that make a body print the body's volume,
area, face and edge counts, and whether it is a manifold solid (`report()`):

    smlib_oracle props  in.smb
    smlib_oracle fillet in.smb out.smb radius x y z [x y z ...]   # the edges nearest the points

## Commands that mirror SMLib's own tests

Each one drives SMLib the way a test in `usd-brep/tests/prog_test/src` does, with that test's
tolerances and flags. An input is a `.smb` file or `file.smp#i` (the i-th brep of a part file,
default 0), or one of `my_test_merge`'s primitives (smMergeTest.cpp, `SmPrimitiveCreation`, full
turns about +z, tolerance 1e-5): `box:l:w:h:x:y:z` (min corner at x y z), `cone:h:r0:r1:x:y:z`
(base centre at x y z) or `torus:R:r:x:y:z`. `props` and `fillet` read through SM_API with the UV trim curves rebuilt; the others
read as the tests do (`SmBrep::ReadFromFile`, no rebuild; `--rebuild-uv` to rebuild). Options
are `--key` or `--key=value` anywhere on the line. `ORACLE_VERBOSE=1` puts diagnostics on
stderr (SMLib's error callback, per-face areas, and per-command details).

| Command | Test it mirrors | Fixed parameters (from the test) |
|---|---|---|
| `boolean a b U\|I\|D [out.smb]` | `my_test_bbu` (smMergeTest.cpp) | parts whose joint box centre is far out (`\|c\|² > 1e7`) are moved to the origin first and back after |
| `offset in dist [keepFace] [out.smb]` | `my_test_offset` (smOffsetTest.cpp:30) | `keepFace` is 1-based (0 = none) and is shelled, not offset; dist < 0 is an inset; `SetExtendConvexEdges(FALSE)`, `SetMergeResults(TRUE)`; tolerance `--tol`, default 1e-4 (the tests use 1e-4 or `SM_ZONE_TOL_3D` = 1e-5) |
| `shell in dist face... [out.smb]` | `ShellBrep` (smOffsetTest.cpp:190) | faces are 0-based; tolerance = the brep's; extend convex corners and stitch (`bExtendCorners = TRUE`, every test; `--no-extend` for the other branch); then `my_test_bbu` difference (offset − input, or input − inset) |
| `replace in face plane ox oy oz nx ny nz [out.smb]`<br>`replace in face cyl ox oy oz ax ay az r [out.smb]`<br>`replace in face sms file.sms [out.smb]` | `my_local_op` (smFilletTest.cpp:3518) | face is 0-based; `SmPlane`/`SmCylinder::CreateCanonical` then `AdjustSTEPUVDomain`, or `SmBSplineSurface::ReadFromFile` reparameterized to the face surface's natural domain; then `SmBrep::LocalOperation`. The tests give the frame's X axis and the domain: pass them as `--xdir=x,y,z` and `--dom=u0,v0,u1,v1` (cylinder u in degrees); by default X is the world axis least along the normal/axis and the domain covers the box, padded by 10% of its diagonal (cylinder u 0–360) |
| `chamfer in out.smb dist x y z [...] \| all` | `my_test_model` with `bMakeChamfer` (smFilletTest.cpp) | `SmConstantRadiusFS` (dist = ball radius; `--solver=dist` for `SmConstantDistanceFS` with dist·√2 as the test passes), `SmLinearCrossSectionFSG`, `SmMakeSurfaceBlendSIH(1.0)`, angle 30°, tangency 2°, approx tol `SmTol::GetApproxTol3d(brep)`, local merge (`--global-merge` for global); `all` = every edge (the NxN chamfer cases) |
| `sweep in trans dx dy dz [out.smb]`<br>`sweep in rot ox oy oz ax ay az deg [out.smb]` | smSweepTest.cpp cases 25–34 | whole-body `SmTopologySweep::DoSweep` (all six arrays NULL) into the brep's infinite region, `SetDoMerge(TRUE)`, `SetDoStitching(TRUE)`; rotation eps = the brep's tolerance (`--eps`). `in` may be `sphere:r:cx:cy:cz` (`SmPrimitiveCreation::CreateSphere`, cases 25/33/34) or `spherenp:r:cx:cy:cz` (`CreateSphereNoPole`, cases 27–29) |
| `stitch in tol [tol ...]` | `my_stitching_demo` (smMergeTest.cpp:343) | `m_bEditingEnabled` set around `StitchFaces`; one run per tolerance in order (the test runs 0.001 then 0.005) |
| `ssi file.smp [tx ty tz]` | `my_test_one_ssi` / `my_test_ssi` (smAdvancedTest.cpp:1023, smTopoTest.cpp:2171) | every face of brep 0 × every face of brep 1 (a surface-only part: every surface pair, on natural domains), face UV domains, both `bUseSurfaceEdges` TRUE, approx tol 1e-3 (`--tol`), angle tol default; the translation is the test's |
| `section in nx ny nz k` | `my_test_tsurf_section` (smTopoTest.cpp, run by `my_test_sec_adv`, smAdvancedTest.cpp:27) | planes through `box.Evaluate(t,t,t)`, t = i/(k−1); approx tol 1e-2 (`--tol`); `--move=x,y,z` translates first as the test does |
| `silhouette in vx vy vz` | `my_test_tsurf_silhouette` (smTopoTest.cpp) | per face `CreateSilhouetteCurves` on the face UV domain then `SmFace::CreateCurvesByTrimming` (`bDoTrim`); approx tol 1e-3, angle 20°; parallel view unless `--persp` (then the vector is the eye point; the tests use `--persp`); `--move` |
| `tess in chord angleDeg [maxEdge [maxAspect]]` | `my_test_one_tess` (smTessTest.cpp:82) | `SmCurveTessDriver(0, chord, angle)`, `SmSurfaceTessDriver(chord, angle, maxEdge, 0.0, 0.001, maxAspect)`, on a copy of the brep; maxEdge < 0 = fraction of the box size as in the test; triangles collected through an `SmPolygonOutputCallback` (`SM_PO_TRIANGLES`) |
| `cci file.smp` | `my_test_cci_adv` / `my_test_GlobalCC_intersect` | every pair of the part's curves over their natural intervals, tol `SM_ZONE_TOL_3D` (`--tol`) |
| `psection surf nx ny nz k` | `my_test_planar_section` (smSurfaceTest.cpp:667) | `SmSurface::CreatePlanarSectionCurves` at k planes through `box.Evaluate(t,t,t)`, t = i/(k−1), of the surface's `CalculateBoundingBox` (its control hull, grown by the zone tolerance); default tolerances; per plane its point and curves |
| `pproject surf k` | `my_test_parallel_projection` (smSurfaceTest.cpp:761) | quadratic circles of radius i/k of half the box's larger width about the middle of the box's top, `CreateParallelProjectionCurves` along −z; per circle its centre, radius and curves |
| `ptsolve surf min\|max\|normalize nx ny nz` | `my_test_surface_point_extrema` (smSurfaceTest.cpp:404) | `GlobalPointSolve` (`SM_SR_ALL`, `SM_ZONE_TOL_3D`) from each point of the nx × ny × nz grid over the box, stepped as the test steps it; per target its solutions' uv, point and distance. `--move=x,y,z` transforms the surface first by `SetCanonical((x,y,z), +x, +z)` (`my_test_sp_nurb`) |
| `closest surf x y z ...` | `my_test_surface_closest_point` (smSurfaceTest.cpp:632) | `GlobalPointSolve` `SM_SO_MINIMIZE`, `SM_SR_SINGLE`, tolerance 1e-6, natural domain, no drop distance; per point the nearest point and distance |

The surface commands take a bare B-spline surface, `surf`: a `.sms` or `.srf` file
(`SmBSplineSurface::ReadFromFile`), or `grid:tx:ty:tz:nu:nv:du:dv:x0:y0:z0:...`
(`my_create_nurb_surf`: nu × nv control points, v fastest, moved by t, clamped uniform knots);
`--dom=a,b,c,d` restricts it to the fractions (a, b) to (c, d) of its natural uv domain.

Outputs (`ssi` curves have their length, ends and 8 samples evenly in the parameter, ends
included; `type` is `SmTsectCurveType`; SMLib statuses are `1000` for success):

    $ smlib_oracle boolean Solids/box_cyl_grid.smp#0 Solids/box_cyl_grid.smp#1 D
    {"ok": true, "status": "ok", "volume": 687.223742597, "area": 481.962715718, "faces": 6, "edges": 11, "solid": true}
    $ smlib_oracle offset Solids/offset_bracket2.smb 1.5 --tol=1e-5
    {"ok": true, "status": "ok", "volume": 12462.4415394, "area": 5175.04899581, "faces": 167, "edges": 364, "solid": true}
    $ smlib_oracle offset Solids/off_extrusion.smb -0.3 1
    {"ok": true, "status": "ok", "volume": 407.319330943, "area": 644.656847527, "faces": 15, "edges": 39, "solid": true}
    $ smlib_oracle shell Shell/problemShell1.smb -4.58 4
    {"ok": true, "status": "ok", "volume": 358711.793625, "area": 160149.510287, "faces": 11, "edges": 24, "solid": true}
    $ smlib_oracle replace Fillet/nurb_box.smb 1 plane 0 0 12 0 -0.7071067811865476 0.7071067811865476 --dom=-15,-15,25,25
    {"ok": true, "status": "ok", "volume": 1531.2126465, "area": 894.550664354, "faces": 6, "edges": 12, "solid": true}
    $ smlib_oracle replace Solids/box.smb 4 sms offset_nurb.sms
    {"ok": true, "status": "ok", "volume": 1151.34567842, "area": 673.407453468, "faces": 6, "edges": 12, "solid": true}
    $ smlib_oracle chamfer Solids/pyramid.smb out.smb 10 all
    {"ok": true, "status": "ok", "volume": 869257.022598, "area": 56449.2999203, "faces": 18, "edges": 32, "solid": true, "chamfered": 8}
    $ smlib_oracle chamfer Solids/box.smb out.smb 1 10 5 10
    {"ok": true, "status": "ok", "volume": 995, "area": 593.142135624, "faces": 7, "edges": 15, "solid": true, "chamfered": 1}
    $ smlib_oracle sweep sphere:4:8:0:0 rot -15 0 0 0 1 0 45
    {"ok": true, "status": "ok", "volume": 0, "area": 655.083511239, "faces": 3, "edges": 5, "solid": true, "sweep_status": 1000}
    $ smlib_oracle sweep Solids/box.smb trans 11 22 33
    {"ok": true, "status": "ok", "volume": 0, "area": 2580.85732089, "faces": 12, "edges": 24, "solid": true, "sweep_status": 1000}
    $ smlib_oracle stitch Solids/fillet_ts.smb 0.001 0.005
    {"ok": true, "status": "ok", "volume": 924.82857533, "area": 605.235964878, "faces": 9, "edges": 17, "solid": true, "runs": [{"tol": 0.001, "status": 1000, "stitched": 15, "lamina": 4, "max_vertex_gap": 0.000573511368129, "max_edge_gap": 0.000562100008541}, {"tol": 0.005, "status": 1000, "stitched": 2, "lamina": 0, "max_vertex_gap": 0, "max_edge_gap": 0.0017562337572}]}
    $ smlib_oracle ssi ssi_2pl.smp
    {"ok": true, "status": "ok", "pairs": 1, "failed_pairs": 0, "count": 1, "curves": [{"length": 3, "start": [5, 5, 2], "end": [5, 2, 2], "samples": [[5, 5, 2], [5, 4.57142857143, 2], [5, 4.14285714286, 2], [5, 3.71428571429, 2], [5, 3.28571428571, 2], [5, 2.85714285714, 2], [5, 2.42857142857, 2], [5, 2, 2]], "pair": [0, 0], "type": 2, "deviation": 0}]}
    $ smlib_oracle section Solids/nurb_fillet.smb 0 0 1 3
    {"ok": true, "status": "ok", "count": 12, "planes": [{"point": [-13.4783567688, -8.00002, -7.38230334891], "status": 1000, "count": 0, "length": 0}, {"point": [2.95197329006, 1, -1.7763568394e-15], "status": 1000, "count": 12, "length": 30.9105594536}, {"point": [19.3823033489, 10.00002, 7.38230334891], "status": 1000, "count": 0, "length": 0}]}
    $ smlib_oracle silhouette torus.smb 60 -90 0.00001 --persp --move=60,90,0
    {"ok": true, "status": "ok", "count": 4, "length": 172.341710348, "faces": [{"face": 0, "status": 1000, "untrimmed": 4, "count": 4, "length": 172.341710348}]}
    $ smlib_oracle tess Solids/nurb_fillet.smb 0.2 30
    {"ok": true, "status": "ok", "triangles": 2634, "area": 1076.38139734, "volume": 1212.43295671, "faces": 20, "failed": false, "faces_without_triangles": 0, "output_status": 1000}
    $ smlib_oracle cci cci_bug_rhino1.smp
    {"ok": true, "status": "ok", "curves": 2, "count": 2, "points": [{"curves": [0, 1], "t": [0, 2], "point": [-10, 0, 0], "gap": 0, "range": true, "t_end": [20, 22], "end": [10, 0, 0]}, {"curves": [0, 1], "t": [52.2458698357, 2], "point": [-10, 0, 0], "gap": 0, "range": false}]}

(Paths are under `usd-brep/TestFiles/pt_TestFiles`.)

### Checked against the tests' expectations

- `boolean`, union over every pair of `Solids/box_box_grid.smp` and `box_cyl_grid.smp`: 659 and
  499 edges in all, the totals `my_two_prim_test` asserts.
- `offset`: off_extrusion 0.3 / 1.0 / −0.3 keeping face 1, boolean_trees_result 0.3, and
  offset_bracket2 0.5 / 1.5 give the face and edge counts the tests assert. off_extrusion 2.2
  gives 80 faces, 155 edges, not a solid; the test asserts 40/78, but SMLib's own run of that test
  (`usd-brep/rust/golden/release/pt_offset/progTest.txt`) prints the same 80/155.
- `shell`: problemShell, 1, 2, 3, 5, 10 give 11 faces and 24 edges as asserted. problemShell4
  (+5.08) gives 5 faces and 10 edges; the test asserts 7 edges ("Possible bug here"), and the
  golden run prints 5/10 too.
- `sweep`: cases 25, 27, 28, 29, 33 and 34 give the asserted face and edge counts.
- `chamfer`: pyramid 10 all → 32 edges, Fillet/ext_w_hole 0.5 all → 76 edges, as asserted.
- `ssi`: 14 of 16 cases tried give the asserted counts, ssi_pln_rev2 included (3 without
  `USE_ANALYTICS`, 2 with; SmConfig.h sets it). ssi_tor_purpen_tor_sing gives 6 against the
  asserted 14, and the golden run prints `Expected 14, Found 6` as well.
- `silhouette`: the torus and the three vase_rev cases give the asserted 4, 8, 5, 5. sphere.smb
  (eye 100 90 0, moved −60 −150 0) gives 2 against the asserted 5.
- `section`: torus.smb gives 9 / 0 / 12 curves for normals y / z / x (7 planes) against the
  asserted 14–16 / 13 / 16–17. The surface sections are there (`ORACLE_VERBOSE` shows 2 curves
  per cutting plane) but `SmFace::CreateCurvesByTrimming` keeps none on this one-face torus. The
  release golden run cannot confirm or rule this out (its asserts are compiled out and it prints
  no counts).
- `cci`: cci_bug_rhino1 gives the asserted 2.
- `tess`: the four nurb_fillet runs report no failed faces, as asserted.

### Notes

- A whole-body sweep leaves the swept body's region flagged void (the `.smb` written shows both
  regions void), so the SM_API volume is 0; the area and counts are right. `tess` of the written
  result gives the volume (sphere sweep above: 1173.8; Pappus gives about 1176).
- `chamfer`'s distance is the ball radius of the constant-radius solver with a linear section, so
  each face is set back by `dist` on a right-angle edge (box above: 1000 − ½·1·1·10 = 995).
- The tests' own driver code is not linked in: each command repeats the few calls the test makes.
