# SMLib test coverage

Which of SMLib's acceptance tests (NVIDIA usd-brep, Apache-2.0) the kernel's generated suites
(`crates/cadrs_kernel/tests/smlib_*.rs`, from `tools/smlib_to_rust.py` and `tools/smlib/*.py`) cover.
Audited 2026-10-23 against kernel-perf 7c89042 and the usd-brep checkout at
`/home/casper/work/nvbrep/usd-brep`.

Status of each item (a function, a numbered case, a table row or, for the Python file, a test method):
- **covered**: a generated test checks the same outcome (suite and test named);
- **partial**: some of its asserts are converted, or a weaker check (what is missing is said);
- **missing**: not converted, but the kernel has the operation: a conversion candidate;
- **gap**: not converted because the kernel lacks the operation (a feature gap, named);
- **n/a**: not applicable: SMLib's own containers and utilities (SmTArray, SmMap, SmObject...), C/Python
  API plumbing (argument validation, handles, ownership), drawing, timing loops with no assert, USD-only,
  dead code (`if (0)`, `#if 0`, never called), or a model absent from the checkout.

Two facts hold throughout:
- `SM_ASSERT` is a no-op in SMLib's release build (SmMessages.h:714), which is what produced the golden
  output (`usd-brep/rust/golden/release/pt_*/progTest.txt`). Only `SER(...)` and explicit `return SM_ERR`
  checks are enforced there. Counts that exist only as `SM_ASSERT` are SMLib's intent, not checked
  behaviour, and some are stale.
- SMLib's piece counts (curves split at seams, sampled solutions along degenerate sets) are compared as
  "none against some" in most converted suites. Such cases are counted as covered when the geometric
  outcome (lengths, points on both surfaces, volumes) is checked.

## Totals

Cases per source (the decimation tests of smTessTest count once per stage, 8 cases in both missing and gap).

| Source | Runs from | covered | partial | missing | gap | n/a |
|---|---|---:|---:|---:|---:|---:|
| smAdvancedTest.cpp | sec/sil/ssi/cci-advanced, ssi-analytic, trimmed-surfaces | 109 | 4 | 11 | 4 | 6 |
| smSweepTest.cpp | sweeps, primitives | 45 | 2 | 0 | 10 | 4 |
| smFilletTest.cpp | fillets, local-ops | 100 | 4 | 0 | 4 | 6 |
| smOffsetTest.cpp | offset | 26 | 1 | 0 | 0 | 6 |
| smMergeTest.cpp | booleans, stitch | 83 | 3 | 1 | 28 | 0 |
| smTopoTest.cpp | topology, brep-import, helpers of the advanced suites | 4 | 9 | 70 | 69 | 83 |
| smTessTest.cpp | tessellation | 5 | 50 | 48 | 28 | 6 |
| smCurveTest.cpp | unit (`my_unit_test_suite`) | 4 | 27 | 52 | 90 | 43 |
| smSurfaceTest.cpp | unit (`my_unit_test_suite`) | 0 | 11 | 175 | 69 | 33 |
| smFeatureTest.cpp | never called | 0 | 0 | 0 | 0 | 16 |
| **prog_test total** | | **376** | **111** | **357** | **302** | **203** |
| SM_API_test (`run_sm_api`, 114 functions run) | SM_API_test.sh | 4 | 5 | 134 | 34 | 373 |
| test_omni_solid.py (400 tests) | pytest | 0 | 0 | 189 | 57 | 154 |

Verified from the earlier audit:
- smSurfaceTest.cpp has no suite. Only `my_test_csi_analy` is converted, partly, in `smlib_cci`.
  smFeatureTest.cpp has no suite, and nothing calls `my_test_feature`, so it is n/a.
- smTopoTest.cpp yields one test of its own (`smlib_brep_import`). Its helpers drive the section,
  silhouette, ssi-advanced and trimmed-surfaces suites. Its own `my_test_topology` (29 checks of
  SMLib's topology internals) and `my_topo_regression_test` are not converted.
- smTessTest.cpp yields 5 tests (`my_test_tess` tests 4-8). The rest are mostly SmPolyBrep, though not
  all: STL reading, ray firing, near-coincident cube booleans, meshing for decimation and cone normals
  have kernel counterparts.
- SM_API_test and test_omni_solid.py were unused. Neither has file fixtures (all geometry is built in
  code), and most of their cases are API plumbing.

Corrections to the generators' NOT_CONVERTED notes, found by the audit:
- `my_test_cci_tol` is run by `my_unit_test_suite` (smAdvancedTest.cpp:4973). cci.py says it is not.
- offset lCount 18: `Solids/airfoil.brep` exists in the checkout. The case is unreached because the
  loop stops at 17.
- `my_test_trim_surfaces`'s ProjectAndTrim of `blue.sm` is dead code, and its files are absent.
- ssi_advanced.py resolves `#ifdef USE_ANALYTICS` as undefined, but SmConfig.h:262 defines it. This
  affects only the alternate counts, which are compared as none against some.

## Converted since the audit

The tables further down are as audited. These conversions came after them and take precedence
(new tests, of which known failures in brackets):

| Source items | Suite: tests | Checks |
|---|---|---|
| smCurveTest `my_test_cci_tol` (5 asserts) | smlib_cci: `cci_tol_*` (5, [1]) | distinct points (closer than 10 tol counted once) as SMLib's counts, seam duplicates aside (8 → 4) |
| smCurveTest `my_test_analytic_curves` (line, circle, ellipse) | smlib_cci: `cci_point_solve_*` (3) | 11 points per curve found again within tol |
| smCurveTest `my_test_curve_properties` (circle × plane) | smlib_cci: `cci_circle_near_tangent_plane` (1) | 2 points |
| smSurfaceTest `my_test_csi_tangency` (44 steps) | smlib_cci: `csi_tangency_{circles,lines}_{crossing,near_miss}` (4, [2]) | 2 points while crossing, 1 near miss within 1.0001e-3 |
| smSurfaceTest `my_test_csi_tol` (40 steps) | smlib_cci: `csi_tol_seam`, `csi_tol_knot` (2) | 1 point per step |
| smSurfaceTest `my_test_sp_nurb` (point intersect ×4, MINIMIZE ×2, 15 closest points) | smlib_surfaces: `sp_nurb_*` (7) | points found again; nearest points no farther than SMLib's (new oracle `ptsolve`, `closest`) |
| smSurfaceTest `my_test_section` (5 surfaces) | smlib_surfaces: `section_*` (5) | per plane: none/some, total length, SMLib's samples (new oracle `psection`) |
| smSurfaceTest `my_test_projection` (3 surfaces) | smlib_surfaces: `projection_*` (3) | per circle, as the sections (new oracle `pproject`) |
| smSurfaceTest `my_test_analytic_surfaces` (9 of 10 surfaces), `my_test_sp_analy` (live block) | smlib_surfaces: `analytic_*` (9, [1]), `sp_analy_revolution` (1) | grid points found again within 1e-6 |
| smTessTest `my_poly_boolean_regression` tests 1–3, 5 (`my_box_box_stress` 6 steps, L prism − box, cube ∪ cube, `box_cyl_grid.smp` 25 differences less 3 non-manifold) | smlib_poly_booleans (30) | valid solids; closed-form volumes (snapping below the point tolerance allowed); SMLib's B-rep difference (volume, area) for the grid |
| smTessTest `my_test_decimation_regression` (8 tests, their meshing; 5 and 6 the same) | smlib_tessellation: `tess_decimation_*` (7) | mesh watertight (a sheet's open only on its border), within chord, SMLib's mesh volume |
| smAdvancedTest `my_test_tsurface_regression` 25 (`ellips_ts2.smb`): benchmark point solve, point classify, point intersect | smlib_trimmed_surfaces: `tsurf_25_*` (3) | distance 2870.476090 as SMLib's run prints; 389 grid points inside / on the face (new oracle `tgrid`) |

Feature gaps found on the way: no parabola or hyperbola curves (`my_test_analytic_curves`,
`my_test_analytic_surfaces`); no "every foot of the normal" point solve (SM_SO_NORMALIZE counts of
`my_test_sp_nurb`).

## Feature gaps (operations SMLib tests that the kernel lacks)

Grouped by area, with the sources that exercise them:
- **Non-manifold results:**
  - SM_BO_MERGE and piecewise merge, solids with wires or vertex shells, and "same"-target sweeps
    (smMergeTest, smSweepTest, Python, SM_API).
  - Face-to-face blends between separate B-reps or bare surfaces (smFilletTest 101, 104, 106 and 107;
    Python `surface_surface_fillet`).
- **Curve solvers:**
  - Local-extremum enumeration (curve-curve, curve-point).
  - Directed, projected, signed and pivot distance solves on curves, surfaces and B-reps (smCurveTest,
    smTopoTest, smAdvancedTest).
  - Curve-surface and surface-surface local extrema (smSurfaceTest).
- **Curve-on-surface:**
  - Drop, project and lift curves, and trim faces by them (DropCurve, DropAndTrimCurve, LiftCurve,
    ProjectAndTrim).
  - Imprinting a projected curve on a body (smSurfaceTest, smTopoTest, SM_API).
- **NURBS editing:**
  - Knot removal, degree change, curve and surface extension, point-grid approximation, end-point
    deformation.
  - Gordon and swung surfaces, ruled or linear skins (smCurveTest, smSurfaceTest, SM_API, Python).
- **Curve types and analysis:**
  - Parabola and hyperbola curves, conversion to lines and arcs, continuity evaluation.
  - Curve property census, curvature from derivatives, third derivatives.
- **Polygon library:**
  - SmPolyBrep solvers, merge, tri-strips, decimation, `.pbp` I/O, polygon booleans, sections and
    trims (smTessTest).
  - Partial tessellation with a failure report.
- **Other:**
  - Defeaturing (`SmFeatureExecutive`, smFeatureTest; never run).
  - Fillet preview, `remove_fillet`, setback distances and bevel corners on fillets.
  - Non-uniform B-rep scale, a helix curve on its own, and a native `.smb` writer for users.

---




Sources: /home/casper/work/nvbrep/usd-brep/tests/prog_test/src/smAdvancedTest.cpp and smSweepTest.cpp.
Our side: tools/smlib/{section,silhouette,ssi_advanced,ssi_analytic,trimmed_surfaces,cci,primitives,sweeps}.py, the
generated tests/smlib_<suite>.rs, tools/smlib_known_failures/<suite>.txt (TREE = the cadrs checkout).

Conventions: "covered (known fail)" = converted, listed in smlib_known_failures, so it is a kernel bug, not a coverage gap.
Wherever SMLib asserts a piece count of its own cell complex or of trimmed pieces (SM_ASSERT(lNumFound == n), ValidateCounts), our
tests check the geometric outcome instead (closed form, or length and sample points against the oracle) and compare the count
only as none-vs-some or where the result is all-planar. I count such cases as covered (the converted outcome is what both agree
on) and say so in Notes. The known-failure files list only these names: ssi_advanced 01, 15, 16, 17 (slow), 29, 35, 41, 42;
ssi_analytic 09, 16. section, silhouette, cci, trimmed_surfaces, primitives and sweeps have none.

### smAdvancedTest.cpp

What runs (prog_test.cpp:53-63 suite table): `section-advanced` = my_test_sec_adv (:27), `silhouette-advanced` = my_test_sil_adv
(:123), `ssi-advanced` = my_test_ssi_adv (:1087), `trimmed-surfaces` = my_test_tsurface_regression (:1438), `cci-advanced` =
my_test_cci_adv (:2130), `ssi-analytic` = my_test_ssi_analytic (:4902) -> my_ssi_analytic_demo (:2877, a `for nTest=1..17` switch),
`unit` = my_unit_test_suite (:4914). All of them are live. Helper functions before my_test_ssi_adv (my_test_one_ssi :1023) and
the my_create_test_* builders (:2165-2870) are only called by those entry points. my_test_java_interface (:19) is only declared;
it is reached only under `#ifdef SMJAVA` (:4917) and no definition exists in the repo.

USE_ANALYTICS caveat: SmConfig.h:262 does `#define USE_ANALYTICS 1`, so SMLib's own run takes the `#ifdef USE_ANALYTICS`
branches in my_test_ssi_adv (asserted counts of numbered runs 28, 37, 41, 48, 50), but tools/smlib/ssi_advanced.py resolves the
`#ifdef`s as if USE_ANALYTICS were undefined (docstring line 14; it keeps 29, 38, 42, 49, 51). Only the count differs (and ours
is compared only as none-vs-some), so no outcome is lost, but the docstring/doc comments quote the wrong counts for 5 runs, and
the generated test names ssi_29/36/39/45... are the non-analytic alternates. Worth a one-line fix in the generator.

| Item (function or case, file:line) | Fixtures (TestFiles/pt_TestFiles) | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_sec_adv case 1: torus moved (0,0,0), plane normal (0,1,0), 7 planes, asserts 14 or 16 curves (:48) | torus.smb | covered | section: section_1_torus_y | each plane: edges on plane and torus, closed loops, total length vs closed form; SMLib's count not compared (oracle disagrees with the assert: 9) |
| my_test_sec_adv case 2: torus moved (0,-40,0), normal (0,0,1), asserts 13 (:79) | torus.smb | covered | section: section_2_torus_z | same |
| my_test_sec_adv case 3: torus moved (-40,-40,0), normal (1,0,0), asserts 16 or 17 (:110) | torus.smb | covered | section: section_3_torus_x | same. (source has `lCount==2` guard on 2 and 3, all run since lCount starts 0) |
| my_test_sil_adv 1-4: sphere, views (1,0,0) 2 / (0,1,0) 2 / (0,0,1) 1 / (1,1,1) 1 (:132-229) | sphere.smb | covered | silhouette: silhouette_01..04_sphere_par | geometry (point on surface, n.w=0) + total length vs oracle `silhouette`; count only none-vs-some (sphere count unconfirmed: oracle 2 vs asserted 5) |
| my_test_sil_adv 5-8: cone, 4 views, counts 0/2/2/2 | cone.smb | covered | silhouette_05..08_cone_par | same (5 asserts none) |
| my_test_sil_adv 9-12: plane_circ, counts 0, 0 or 1, 0, 0 | plane_circ.smb | covered | silhouette_09..12_plane_circ_par | all none-ish |
| my_test_sil_adv 13-16: plane_square, counts 6/6/0/0 | plane_square.smb | covered | silhouette_13..16_plane_square_par | edge-on sides of uv box |
| my_test_sil_adv 17-19: toruswvy (1,1,1) 19, (0,0,1) 4, (1,0,0) 48 | toruswvy.smb | covered | silhouette_17..19_toruswvy_par | |
| my_test_sil_adv 20-22: torus (0,1,0) 28, (1,0,0) 16, (1,0,1e-7) 14 | torus.smb | covered | silhouette_20..22_torus_par | |
| my_test_sil_adv 23-25: vase_rev (1,1,1) 13, (1,.3,.1) 5, (1,0,0) 5 | vase_rev.smb | covered | silhouette_23..25_vase_rev_par | |
| my_test_sil_adv 26-28: toruswvy perspective, eye (100,100,100) 6, (0,60,100) 4, (160,60,0) 10 | toruswvy.smb | covered | silhouette_26..28_toruswvy_persp | |
| my_test_sil_adv 29-31: torus perspective, eyes (-60,190,0) 4, (100,90,0) 5, (60,-90,1e-5) 4 | torus.smb | covered | silhouette_29..31_torus_persp | |
| my_test_sil_adv 32-34: vase_rev perspective, eyes (40,220,100) 8, (160,30,10) 5, (160,120,0) 5 | vase_rev.smb | covered | silhouette_32..34_vase_rev_persp | |
| my_test_ssi_adv runs 1-17 (:1094-1180): ext_ext_coin 11, ext_cyl_none 0, ext_cyl_touch 2, ext_cyl_2int 2, ext_ext_none 0, ext_ext_inside 4, ext_ext_opposite 4, ext_ext_touch 4, ext_ext_overlap 4, ext_ext_same 4, rev_rev_360 3, rev_rev_opposite 3, rev_rev_90 3, rev_rev_rot180 6, rev_rev_180_pts 6, rev_rev_coin 5, rev_rev_coin_touch 7 | ssi_*.smp | covered (known fail: 1, 15, 16, 17 slow) | ssi_advanced: ssi_01..ssi_17 | every point on both surfaces, total length vs oracle `ssi` (1e-3), SMLib sample points near ours, count none-vs-some. 1/16/17: coincident-region boundary conventions; 15: point contacts missed |
| my_test_ssi_adv runs 18A, 18B, 19 (:1186-1200): rev_sph_full 4, rev_cyl_full 6, rev_cone_full 6 | ssi_rev_sph_full, ssi_rev_cyl_full, ssi_rev_cone_full | covered | ssi_18..20 | |
| my_test_ssi_adv runs 20-27 (:1202-1239): tor_purpen_tor_sing 14, tor_sph_tan 2, tor_sph_tan2 2, tor_in_tor_tang_hyper 1, tor_in_tor_tan_ellip 1, 2pl 1, cylcyl1 2, pln_rev1 2 | ssi_*.smp | covered | ssi_21..28 | run 20's asserted 14 is stale (SMLib finds 6) |
| my_test_ssi_adv runs 28/29 (USE_ANALYTICS alt pair), 30, 31, 32, 33 (:1241-1266) | ssi_pln_rev2, ssi_pln_rev3, ssi_plncyl1, ssi_plncyl2, ssi_rev_rev1 | covered (known fail: pln_rev2) | ssi_29_pln_rev2 (known fail, "oracle error"), ssi_30..33 | one executed run for 28/29 (SMLib: 28, count 2) |
| my_test_ssi_adv run 34 bumppy_plane2 50 (:1270) | ssi_bumppy_plane2.smp | covered | ssi_34_bumppy_plane2 | |
| my_test_ssi_adv run 35 bumppy_plane 84 (:1277) | ssi_bumppy_plane.smp | n/a | - | `if (FALSE)` (slow, missing a segment); listed in NOT_CONVERTED |
| my_test_ssi_adv runs 36 (rev_rev_sing 6), 37/38 (cyl_cyl_sing alt pair), 39 (cyl_cyl_sing2 4), 40 (coincident_planes 4), 41/42 (coincident_cyls alt pair 3), 43 (coincident_spheres 1), 44 (coincident_spheres2 4), 45 (coincident_torii 8) (:1284-1334) | ssi_*.smp | covered (known fail: rev_rev_sing, coincident_spheres2, coincident_torii) | ssi_35..42 (ours are numbered by executed run; 35 rev_rev_sing, 36 cyl_cyl_sing, 37 sing2, 38 planes, 39 cyls, 40 spheres, 41 spheres2 fail, 42 torii fail) | 35: curves lie off a surface by > 1e-3 |
| my_test_ssi_adv runs 46-60 (:1337-1420): tor_sph_touch 1, rhino_bug1 1, sph_plane1 1 (48/49), sph_plane2 (50/51), cone_plane1 2, cone_cyl1 2, cone_cyl2 2, cone_2pln 1, cyl_bicub 2, tor_overlap 6, spiral 1, clasic_ruled 4, cylcyl_90_tan 1 | ssi_*.smp | covered | ssi_43..55 | |
| my_test_tsurface_regression block 1 (:1440, `if (FALSE)` my_test_trim_surfaces) | - | n/a | - | dead code |
| my_test_tsurface_regression blocks 2, 8, 14 (:1449, :1509, :1572): my_test_trim_surfaces (smTopoTest.cpp) | built in code | gap | - | ProjectAndTrim of a B-spline curve onto a trimmed plane, MakeFaceWithCurves from uv curves: kernel has no "project a curve onto a face and split/trim it by it" (Kernel::project is hidden-line view projection; Kernel::split_faces needs a SplitTool, not a curve-on-surface projection). NOT_CONVERTED says the same. Three identical runs |
| block 3 (:1455): my_test_analytic_tsurf_creation | built in code | partial | trimmed_surfaces: tsurf_analytic_section | converted as a plane/quarter-cylinder section (pi long quarter circle); SMLib's IntersectInsertRelate imprint and its topology counts are not checked |
| block 4 (:1460, `if (FALSE)` my_test_tsurf_creation + my_test_ssi) | - | n/a | - | dead code |
| block 7 (:1491): CreateConePatch r1 h1 full turn, Area printed | built in code | covered | trimmed_surfaces: tsurf_cone_patch_area | closed form 2pi (SMLib only prints) |
| block 10 (:1513): ss_plane_sph1 DIRECTED_MINIMIZE 4, DIRECTED_MAXIMIZE 8, SIGNED_DIRECTED_MINIMIZE 4 (:1537-1542) | ss_plane_sph1.smp | gap | - | directed and signed-directed distance operations: Kernel::min_distance/max_distance have only plain 3D min/max |
| block 11 (:1552): my_test_tsurf_insert_edge | built in code | n/a | - | per the generator "nothing asserted" (timing/InsertEdge); not re-verified by me |
| block 12 (:1557): my_test_tsurf_creation (10x10 face, triangle and circle holes) | built in code | covered | trimmed_surfaces: tsurf_creation_volume_area, tsurf_creation_point_classes, tsurf_creation_props | extruded by 1; volume, area, point classification |
| block 15 (:1575) ss_2parallel_planes: MIN 69, MAX 36, NORMALIZE 1 | ss_2parallel_planes.smp | partial | trimmed_surfaces: tsurf_solve_2parallel_planes_min, _max | min/max as closed-form distances (5, sqrt75); NORMALIZE solve missing (gap) |
| block 16 (:1613) ss_2adj_planes: MIN 34, MAX 18, NORMALIZE 0 | ss_2adj_planes.smp | partial | tsurf_solve_2adj_planes_min, _max | same (0 and sqrt75) |
| block 17 (:1650) ss_plane_sph1: MIN 1, MAX 3, NORMALIZE 1, PROJECTED_MIN 4, PROJECTED_MAX 33, DIRECTED_MIN 4, DIRECTED_MAX 8, SIGNED 4, plus a second directed set 1/1/1 | ss_plane_sph1.smp | partial | tsurf_solve_plane_sph1_min, _max | normalize/projected/directed/signed solves not converted (gap: no such operations) |
| blocks 18-23 (:1705-2000): MIN/MAX (and normalize/projected/directed) on ss_plane_sph2 (MIN 6, MAX 3), ss_plane_sph3 (2, 3), ss_plane_sph4 (1, 3), ss_plane_tor1 (1, 3), ss_plane_tor2 (>10, 3), ss_tor_tor (1, 1) | ss_plane_sph2/3/4, ss_plane_tor1/2, ss_tor_tor .smp | missing | - | kernel has Kernel::min_distance/max_distance (DistanceTarget on faces/bodies); six fixtures not converted. Asserts are counts of solutions, so values need an oracle subcommand (see candidates). The extra operations in each block are gap |
| block 24 (:2059, `if (FALSE)` cyl_hole.smp, silhouette 6, my_topo_regression_test) | cyl_hole.smp | n/a | - | dead code |
| block 25 (:2094) ellips_ts2: my_topo_regression_test(389,362,52,60,389,59,7,36,68,30,8) (smTopoTest.cpp:2852: point classify, point min/max/normalize, point intersect, curve classify, curve solve intersect/min/max, projection, section(1,1,1)) | ellips_ts2.smb | missing | - | 11 solution-count asserts on one trimmed ellipsoid; kernel has classify, min/max_distance, section; counts of sampled solutions need an oracle. Fixture exists in TestFiles/pt_TestFiles but is referenced nowhere in TREE |
| block 25 silhouette: my_test_tsurf_silhouette(pBrep, 1e-4, (1,0,-0.1), parallel) asserts 2 (:2100) | ellips_ts2.smb | missing | - | Kernel::silhouettes; oracle `silhouette` exists; same template as silhouette.py |
| my_test_cci_adv (:2130): GlobalCurveIntersect of the 2 curves of cci_bug_rhino1.smp asserts 2 | cci_bug_rhino1.smp | covered | cci: cci_adv_rhino_bug1 | oracle `cci` points must lie on one of ours; count not compared |
| my_ssi_analytic_demo case 1 (:2888) SurfOfRevolution x Sphere turned pi: 0 solutions, IsDegenerate check | surf_rev.smb, surf_sphere150.smb | missing | - | kernel has ssi; not converted because the half sphere's turning axis (frame from SmSphere::IsNurbSurfaceSphere) is unknown. Could be dumped from SMLib |
| case 2 (:2995) two rev cylinders, 8 | built in code | covered | ssi_analytic: analytic_02 | no oracle for in-code surfaces: points on both surfaces, none-vs-some, boundary-of-shared-region length closed form |
| case 3 (:3099) two extrusion cylinders, 5 | built in code | covered | analytic_03 | |
| case 4 (:3210) rev x its mirror (CreateDirectrixAtGeneratorParam), 1 | surf_rev.smb | missing | - | frame detected by IsNurbSurfaceSurfOfRevolution unknown |
| case 5 (:3358) two extrusions from surf_extrude.smb, 3 | surf_extrude.smb | covered | analytic_05 | |
| case 6 (:3488) extrusion cylinders, 4 | built in code | covered | analytic_06 | |
| case 7 (:3604) rev cylinders, 1 | built in code | covered | analytic_07 | |
| case 8 (:3713) Cone x SurfOfRevolution, 2 | surf_rev.smb | missing | - | same frame problem |
| case 9 (:3824) SurfOfRevolution x Sphere, 0 and !IsDegenerate | surf_rev.smb, surf_sphere150.smb | covered (known fail) | analytic_09 | the reader makes surf_sphere150's B-spline a sphere, so the uv sub-domain differs: a test problem, not a kernel one |
| cases 10, 11, 12, 13 (:3940, :4048, :4159, :4268): rev cylinders 4, 4, 2; extrusions 3 | built in code | covered | analytic_10..13 | |
| case 14 (:4408) planar patches, 1 | built in code | covered | analytic_14 | |
| case 15 (:4537) extruded B-spline surfaces, 3 | built in code | covered | analytic_15 | |
| case 16 (:4649) intersect (coincident cylinders on each other), 1 | built in code | covered (known fail) | analytic_16 | SMLib returns the shared region's boundary; ssi returns nothing |
| case 17 (:4764) planar patches, 0 | built in code | covered | analytic_17 | |
| my_test_java_interface (:19, called :4920 under `#ifdef SMJAVA`) | - | n/a | - | Java binding, SMJAVA not defined, no definition in repo |

my_unit_test_suite (:4914) is audited by the smCurveTest/smSurfaceTest agents; it only calls these (not counted in totals).
smCurveTest.cpp: my_test_analytic_curves :4885, my_test_array :1261, my_test_map :1563, my_test_object :743, my_test_vector2d :1758,
my_test_vector3d :1907, my_test_matrix :3161, my_test_axis2placement :2291, my_test_extent1d :2364, my_test_extent2d :2688,
my_test_extent3d :3115, my_test_pseudobox :777, my_test_solution :899, my_test_creation :4978, my_test_bsplinecurve :8104,
my_test_bsplinecurve_nlib :9098, my_test_cci_analy :3484, my_test_cci_nurb :4157, my_test_cci_tol :4054, my_test_cci_tangent :3379,
my_test_cci_coincidence :3945, my_test_cci_self :3226, my_test_cci_projection :5242, my_test_cc_nurb :4328, my_test_cc_analy :4421,
my_test_methods :4705, my_test_cp_nurb :5131, my_test_cp_analy :5369, my_test_time :6558, my_test_polynomial :1629,
my_test_curve_properties :7991, my_test_crv_angle_min :8142, my_test_crv_directed_max :8152, my_test_crv_directed_min :8233,
my_test_crv_projected_max :8377, my_test_crv_projected_min :8389, my_test_crv_projected_tangency :8533,
my_test_crv_signed_angle_min :8582, my_test_crv_signed_directed_min :8847, my_test_signed_pivot_min :8938.
smSurfaceTest.cpp: my_test_bsplinesurface_nlib :5715, my_test_analytic_surfaces :5419, my_test_bsplinesurface :5097,
my_test_csi_analy :3514, my_test_csi_nurb :3874, my_test_csi_tol :1171, my_test_csi_tangency :1100, my_test_csi_coincidence :1283,
my_test_cs_nurb :1403, my_test_drop_curve :2053, my_test_ss_nurb :3421, my_test_surf_methods :3984, my_test_sp_nurb :4042,
my_test_sp_analy :4165, my_test_surface_suite :949, my_test_section :4429, my_test_projection :4332.
(Only my_test_solution and my_test_drop_curve have their status checked with SER; the rest are called bare. cci.py converts
my_test_cci_analy/tangent/coincidence/nurb and my_test_csi_analy from this list.)

Totals: covered 109, partial 4, missing 11, gap 4, n/a 6

(covered 3 + 34 + 55 + 2 (tsurf blocks 7, 12) + 1 + 14; partial: block 3, 15, 16, 17; missing: blocks 18-23 (6), block 25 (2), analytic 1, 4, 8; gap: trim_surfaces x3, block 10; n/a: bumppy_plane, tsurf blocks 1, 4, 11, 24, java.)

### smSweepTest.cpp

What runs: prog_test.cpp:53-54 `sweeps` = my_sweep_regression (:3234), `primitives` = my_primsol_regression (:1261). my_primsol_regression
calls my_primsol_demo (:435) `my_primsol_demo_count()` = 8 times with nTestArg 1000, which advances a static counter, so cases 1..8
run in order; case 9 is empty (never reached) and a `if (FALSE)` block at the top ("Test 2 Linear sweep twice no caps",
CreateDraftSweep) is dead. It then runs 7 CreateCylindricalBox input cases (the code's own table, :1281). my_sweep_regression runs
my_sweep_demo (:1336) for ii = 0..34 (`my_sweep_demo_count()` = 35; the status it returns is ignored), then
my_test_dosweep_null_region (:3187; also a SER). In my_sweep_demo most `pBrep->ValidateCounts(...)` return values are not checked
(only an internal assert could fire); `ucheck_mappings_sweep` is also called and not converted.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_primsol_demo case 1 (:553): CreateSkinPrimitive of circle/square/circle (ValidateCounts commented out) | built in code | covered | primitives: prim_01_skin | closed-form volume/area |
| case 2 (:633) blocks: two CreateBox(10,15,20) (x2) | built in code | covered | prim_02_box, prim_02_box_turned | ValidateCounts(6,12,8) compared (6 faces, 12 edges) plus volume/area |
| case 2: Boolean2D difference of CreateRectangle(5,3) and CreateCircle(4) | built in code | covered | prim_02_rectangle_less_circle | sheet boolean, 1 face 5 edges |
| case 2: Boolean2D union of the tsurf face and a moved copy, ValidateCounts(1,25,25,...) | built in code | covered | prim_02_tsurf_union | 1 face, 25 edges, area |
| case 3 (:724) CreateLinearSweep of square less triangle and 2 circles, ValidateCounts(11,27,18,0,0,27,2,2) | built in code | partial | prim_03_linear_sweep | volume and area from a prism; face/edge counts not compared (periodic faces) |
| case 4 (:826) CreateRotationalSweep 20 deg, ValidateCounts(8,18,12,...) | built in code | covered | prim_04_rotational_sweep | counts 8 faces 18 edges compared, volume, area |
| case 5 (:922) CreateCone twice (whole and 30..60 deg); ValidateCounts commented out | built in code | covered | prim_05_cone_whole, prim_05_cone_30_60 | closed forms |
| case 6 (:982) CreateSphere twice (0..180, 30..60) | built in code | covered | prim_06_sphere_0_180, prim_06_sphere_30_60 | |
| case 7 (:1043) CreateTorus twice (whole, 30..60) | built in code | covered | prim_07_torus_whole, prim_07_torus_30_60 | |
| case 8 (:1100) CreateCurveSweep, ValidateCounts(16,42,28,0,0,42,2,2) | built in code | partial | prim_08_curve_sweep | volume/area closed form; counts not compared |
| case 9 (:1226) | - | n/a | - | empty (and not reached: count is 8) |
| dead block (:462, `if (FALSE)`) CreateDraftSweep, ValidateCounts(11,27,18,...) | built in code | n/a | (prim_00_draft_sweep exists anyway) | dead in SMLib; we converted it as a draft body |
| CreateCylindricalBox table, 7 cases (:1281-1311): ok, zero length, zero inside radius, outside <= inside, start angle out of range, end before start, sweep > 360 | none | covered | primitives: cylindrical_box_0..6 | Kernel::cylindrical_box accept/reject |
| my_sweep_demo case 0 / default (:3085) | built in code | n/a | - | only builds the test brep, sweeps nothing |
| my_sweep_demo 1 (translate vertices higher, edges same) | built in code | covered | sweeps: case_01_translate_vertices_higher_edges_same | wire body; counts commented out in SMLib |
| 3 (translate vertices higher; ValidateCounts(2,25,26,14,...)) | built in code | covered | case_03_translate_vertices_higher | |
| 5 (translate edges higher) | built in code | covered | case_05_translate_edges_higher | surface extrude |
| 7 (translate 2 faces higher) | built in code | covered | case_07_translate_2_faces_high | |
| 8 (translate whole brep, non-manifold faces + wire + vertex) | built in code | covered | case_08_translate_whole_brep_non_manifold_faces_wire_vertex | |
| 10, 11, 14 (rotate vertices higher: 3x, full circle, 3x) | built in code | covered | case_10, case_11, case_14 | |
| 19-24 (rotate 1 face higher: partial, through corner, through edge, and each full) | built in code | covered | case_19..case_24 | |
| 25-34 (whole-brep translate/rotate: sphere 25, hollow box 26, 27-29 (45, 345, 360 deg), block with internal edges 30-32, sphere with pole on silhouette 33/34) | built in code | covered | case_25..case_34 (case_26 and case_30 compare 12 faces / 24 edges) | the union-of-ends formulation; closed forms |
| 2, 4, 6, 9, 12, 13, 15, 16, 17, 18 ("same" targets: pieces of the non-manifold test brep copied to their swept position; counts like ValidateCounts(2,12,26,1,10,1,1,15)) | built in code | gap | - | outcome is the cell complex of a non-manifold brep (copied vertices/edges/faces beside the original); the kernel has neither non-manifold bodies nor selecting pieces of a body to sweep/copy. 10 cases |
| my_test_dosweep_null_region: selected-vertex sweep repeated 1..3 times, asserts 6 faces, 12+n edges, 8+n vertices (:3213) | built in code | covered | sweeps: null_region_vertex | wire edges swept 3 times, length 21 |
| my_test_dosweep_null_region: whole hollow box swept, merge off (:3197) | built in code | covered | sweeps: null_region_hollow | SMLib asserts no counts, only non-empty and equal to the NULL-region result |
| my_test_dosweep_null_region: explicit region vs NULL region equality (:3200) | - | n/a | - | API contract of SmTopologySweep's `pRegionToSweepInto` argument |

Totals: covered 45, partial 2, missing 0, gap 10, n/a 4

(my_primsol_regression 19 covered, 2 partial, 2 n/a; my_sweep_demo 24 covered, 10 gap, 1 n/a; null_region 2 covered, 1 n/a.)

### Conversion candidates

Ranked by payoff.

1. **Distance solves on six more trimmed-surface fixtures** (smAdvancedTest.cpp blocks 18-23, :1705-2000; 12 asserted min/max outcomes
   plus about 50 more solve counts that have no kernel op). Build one Case per fixture and min/max: ss_plane_sph2, sph3, sph4,
   ss_plane_tor1, tor2, ss_tor_tor. SMLib asserts only solution counts (6/3, 2/3, 1/3, 1/3, >10/3, 1/1), no values, so expected values need
   the oracle: add an oracle.cpp subcommand `solve file.smp min|max` running `SmTopologySolver::BrepBrepSolve` on brep 0 vs brep 1 and
   returning the best distance (or the solution list). None exists now (commands: props fillet boolean offset shell replace chamfer
   sweep stitch ssi section silhouette tess cci). Kernel API: `Kernel::min_distance` / `max_distance` with `DistanceTarget::Body` or
   face targets. Generator: practical, trimmed_surfaces.py already has the `solve(file, max, value, tol)` helper and per-fixture Case
   template; the new cases are the same shape with oracle-supplied value.
2. **ellips_ts2 silhouette** (block 25, :2100, assert 2 curves) and **my_topo_regression_test on ellips_ts2** (smTopoTest.cpp:2852, 11
   asserts: 389,362,52,60,389,59,7,36,68,30,8). Silhouette: reuse silhouette.py's Case template, view dir (1,0,-0.1), tol 1e-4, no move;
   oracle `silhouette ellips_ts2.smb 1 0 -0.1` exists; kernel `Kernel::silhouettes`. Topo regression: the counts are of sampled point/curve
   solutions (classify, extrema, curve solve, projection, section); closed forms are hard, oracle needs the same `solve` subcommand as (1)
   plus point/curve variants; low value except the section (existing `section` command, `Kernel`/`boolean::section` with normal (1,1,1)).
   Silhouette half: easy and regular; topo half: hand-written.
3. **ssi_analytic cases 1, 4, 8** (3 outcomes: 0 solutions with degenerate check; 1; 2). The kernel ssi exists; the blocker is the
   surfaces' frames (SmSphere::IsNurbSurfaceSphere, IsNurbSurfaceSurfOfRevolution, CreateDirectrixAtGeneratorParam). Add an oracle
   subcommand `analytic N` that runs my_ssi_analytic_demo's case N surfaces and dumps the frames/ssi result (the fixtures and
   built-in-code surfaces cannot be passed to the existing `ssi`, which takes a .smp), then hand-port. Irregular hand-written code; the
   cases are 100-130 lines each, a generator is not practical.
4. **Trim surfaces by projected curve** (my_test_trim_surfaces, three identical runs, smTopoTest.cpp): a feature gap, not a conversion:
   needs "project a B-spline curve onto a face and split it" (ProjectAndTrim, MakeFaceWithCurves). Add to the kernel first.
5. **Directed / projected / normalize / signed distance operations** (SM_SO_DIRECTED_*, SM_SO_PROJECTED_*, SM_SO_NORMALIZE, SM_SO_SIGNED_*;
   about 60 asserted counts across blocks 10, 15-23) are a kernel feature gap with count-only assertions: low payoff.
6. **Sweep "same" cases** (my_sweep_demo 2, 4, 6, 9, 12, 13, 15, 16, 17, 18) are a gap (non-manifold bodies, sweeping selected pieces); low payoff.
7. Housekeeping: fix ssi_advanced.py to resolve `USE_ANALYTICS` as defined (SmConfig.h:262), so the quoted counts and run numbering match SMLib's
   actual run (affects the doc comments of 5 runs only). Optionally compare ValidateCounts for primitive cases 3 and 8 (11/27/18 and 16/42/28)
   against our face/edge counts if the kernel's periodic-face splitting is made to match SMLib's seam convention (not a kernel defect).



Sources: `usd-brep/tests/prog_test/src/smFilletTest.cpp`, `smOffsetTest.cpp`, `smMergeTest.cpp`.
Ours: `tools/smlib/{fillets,local_ops,offset,booleans,stitch}.py`, `tests/smlib_{fillets,local_ops,offset,booleans,stitch}.rs`.

Generated-test census (counted from the .rs files): fillets 100 tests (57 known failures, 23 of them `#[ignore]` slow), local_ops 4, offset 29 (4 known failures, 2 slow), booleans 90 (0 known failures), stitch 4 (1 known failure).

Two cross-cutting observations that affect "covered" below:
1. `check_result` (tests/smlib_common/mod.rs:232-243) compares face and edge counts only when every input face is planar. SMLib's asserted edge counts on curved models (sphere union 6, box_cyl_grid 499, box_cone_grid 447, and every fillet case whose model has a curved face) are recorded in the Expect but not enforced; those tests check validity, closedness/Euler, and oracle volume/area.
2. In `smOffsetTest.cpp` and `my_test_stress`, `ValidateCounts(...)` is called as a bare statement (return value ignored, mismatch only prints), and `ShellBrep(...)`/`my_offset_demo` results are discarded by `my_offset_regression`. So SMLib itself enforces only "no crash / ShellBrep and my_test_bbu return success" for the offset suite. Our offset tests are stronger than SMLib's.

---

### smFilletTest.cpp

Entry points (prog_test.cpp:43-52, all `bRunByDefault` TRUE): suite `fillets` = `my_test_fillet_regression(SM_FS_CONST_RADIUS)` (smFilletTest.cpp:3414) which loops `my_test_suite_0..4` (lCount 1-12, 101-128, 201-237, 301-308, 401-421; suite 2 is run, the CONST_DIST jump over it never happens because only the const-radius variant is registered); suite `local-ops` = `my_local_op_regression` (3496), 8 iterations of `my_local_op` (lCount 0-7). The SM_FS_CONST_DIST and SM_FS_VARIABLE_RADIUS variants of `my_test_fillet_regression` exist but no suite registers them: n/a. Only lCount 11 has an `eSolverType == VARIABLE_RADIUS` skip, none touches the const-radius path. Helpers (`my_test_model` 172, `my_read_fillet_definition_file` 4285, `my_read_fsg`/`my_read_edge_fillet_solver`/`my_read_edges_fillet_solver`, `my_test_two_brep_fillet` 5079 / `my_test_surface_fillet` 5228) run only through the cases below. Case numbering is unique across the suites (offsets of 100), and `my_local_op` reuses lCount 0-7, so `if_blocks` merges those local-op blocks into fillet cases 1-7 (harmless: fillet extraction looks for `my_test_model(`; checked that cases 1-7 resolve to the suite_0 fixtures).

I re-parsed every live `my_test_model` block (file, radius, interval list, INCLUDE/EXCLUDE) and compared with the generated test doc lines: 0 mismatches over all `my_test_model` cases. Flags not modelled by our tests: `bDoGlobalMerge` (TRUE in 214-217, 403, 406, 418-420; an internal merge strategy, no kernel meaning), `bMakeBevelCorner` (TRUE only in 124), `SM_ZONE_TOL_3D`/1e-3 tolerance argument.

Case counts: 106 live numbered cases (suite_0 12, suite_1 28, suite_2 37, suite_3 8, suite_4 21; lCount 421 has two textual blocks, one case) -> 100 generated tests. NOT_CONVERTED in fillets.py lists exactly the 6 missing ones (101; 104, 106, 107; 108, 109) and that list is accurate.

| Item (function or case, file:line) | Fixtures (TestFiles/pt_TestFiles/) | Status | Our test(s) | Notes |
|---|---|---|---|---|
| suite_0 lCount 1 (smFilletTest.cpp:589) asserts 27 edges | Fillet/cyl_cyl_219.smb | covered | `smlib_fillets::case_001_cyl_cyl_219` | - |
| suite_0 lCount 2 (smFilletTest.cpp:608) asserts 39 edges | Fillet/cyl_cyl_2191.smb | covered | `smlib_fillets::case_002_cyl_cyl_2191` | known failure |
| suite_0 lCount 3 (smFilletTest.cpp:627) asserts 27 edges | Fillet/box_on_box.smb | covered | `smlib_fillets::case_003_box_on_box` | - |
| suite_0 lCount 4 (smFilletTest.cpp:645) asserts 32 edges | Fillet/brick.smb | covered | `smlib_fillets::case_004_brick` | - |
| suite_0 lCount 5 (smFilletTest.cpp:668) asserts 21 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_005_l_shaped_box` | - |
| suite_0 lCount 6 (smFilletTest.cpp:686) asserts 21 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_006_l_shaped_box` | - |
| suite_0 lCount 7 (smFilletTest.cpp:704) asserts 19 edges | Fillet/cyl_on_box.smb | covered | `smlib_fillets::case_007_cyl_on_box` | - |
| suite_0 lCount 8 (smFilletTest.cpp:723) asserts 8 edges | Fillet/case_int.smb | covered | `smlib_fillets::case_008_case_int` | known failure, slow/ignored |
| suite_0 lCount 9 (smFilletTest.cpp:745) asserts 12 edges | Fillet/case_ext.smb | covered | `smlib_fillets::case_009_case_ext` | known failure |
| suite_0 lCount 10 (smFilletTest.cpp:763) asserts 17 edges | Fillet/tilt_cyl_on_box.smb | covered | `smlib_fillets::case_010_tilt_cyl_on_box` | - |
| suite_0 lCount 11 (smFilletTest.cpp:781) asserts 12 edges | Fillet/box1x1x2.smb | covered | `smlib_fillets::case_011_box1x1x2` | chamfer (`bMakeChamfer`) |
| suite_0 lCount 12 (smFilletTest.cpp:806) asserts 13 edges | Fillet/extrusion1002.smb | covered | `smlib_fillets::case_012_extrusion1002` | known failure, slow/ignored |
| suite_1 lCount 101 (smFilletTest.cpp:898) asserts 4 edges | Fillet/eg1_srf1.smb, Fillet/eg1_srf2.smb | gap | - | two B-reps, rolling-ball strip between two sheets (`my_test_two_surface_fillet`, SM_BT_MINIMAL r=10); asserts 4 edges. Kernel has no face-to-face blend of separate bodies |
| suite_1 lCount 102 (smFilletTest.cpp:910) asserts 76 edges | Fillet/ext_w_hole.smb | covered | `smlib_fillets::case_102_ext_w_hole` | chamfer, all-but-[] edges; known failure |
| suite_1 lCount 103 (smFilletTest.cpp:930) asserts 76 edges | Fillet/bolt.smb | covered | `smlib_fillets::case_103_bolt` | known failure |
| suite_1 lCount 104 (smFilletTest.cpp:947) asserts 4 edges | Fillet/nurb_nurb_1.smb, Fillet/nurb_nurb_2.smb | gap | - | `my_test_one_surface_fillet` nurb_nurb_1/2 r=5 SM_BT_BLEND, asserts 4 edges; same missing op |
| suite_1 lCount 105 (smFilletTest.cpp:966) asserts 21 edges | Drop/FilletTests/self_intersection_blend2.fdf | covered | `smlib_fillets::case_105_extrusion_old` | fdf `self_intersection_blend2.fdf` edge 17 const 1.3, 3 = approx circular section; known failure, slow/ignored |
| suite_1 lCount 106 (smFilletTest.cpp:982) asserts 3 edges | Fillet/nurb_sph_1.smb, Fillet/nurb_sph_2.smb | gap | - | `my_test_one_surface_fillet` nurb_sph_1/2 r=1, asserts 3 edges; same |
| suite_1 lCount 107 (smFilletTest.cpp:999) asserts 8 edges | Fillet/nurb_plane_w_notch1.smb, Fillet/nurb_plane_w_notch2.smb | gap | - | `my_test_one_surface_fillet` nurb_plane_w_notch1/2 r=2, asserts 8 edges; same |
| suite_1 lCount 108 (smFilletTest.cpp:1018) | - | n/a | - | empty block |
| suite_1 lCount 109 (smFilletTest.cpp:1021) | - | n/a | - | empty block |
| suite_1 lCount 110 (smFilletTest.cpp:1024) asserts 48 edges | Fillet/nurb_box.smb | covered | `smlib_fillets::case_110_nurb_box` | all edges; the `if (0)` box variant is dead, nurb_box live; known failure, slow/ignored |
| suite_1 lCount 111 (smFilletTest.cpp:1053) asserts 52 edges | Fillet/box_on_box.smb | covered | `smlib_fillets::case_111_box_on_box` | known failure |
| suite_1 lCount 112 (smFilletTest.cpp:1074) asserts 45 edges | Fillet/letter_g.smb | covered | `smlib_fillets::case_112_letter_g` | known failure |
| suite_1 lCount 113 (smFilletTest.cpp:1097) asserts 40 edges | Fillet/box_on_box.smb | covered | `smlib_fillets::case_113_box_on_box` | known failure |
| suite_1 lCount 114 (smFilletTest.cpp:1115) asserts 18/17 edges | Fillet/cone_34.smb | covered | `smlib_fillets::case_114_cone_34` | const-radius run asserts 17 (the 18 is the VARIABLE_RADIUS branch); generated doc comment says SMLib asserts 18, expectation is 17 (right) |
| suite_1 lCount 115 (smFilletTest.cpp:1142) asserts 36 edges | Fillet/filleted_box.smb | covered | `smlib_fillets::case_115_filleted_box` | known failure |
| suite_1 lCount 116 (smFilletTest.cpp:1171) asserts 36 edges | Fillet/filleted_box.smb | covered | `smlib_fillets::case_116_filleted_box` | known failure |
| suite_1 lCount 117 (smFilletTest.cpp:1196) asserts 35 edges | Fillet/ext_w_hole.smb | covered | `smlib_fillets::case_117_ext_w_hole` | - |
| suite_1 lCount 118 (smFilletTest.cpp:1214) asserts 25 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_118_l_shaped_box` | known failure |
| suite_1 lCount 119 (smFilletTest.cpp:1232) asserts 28 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_119_l_shaped_box` | known failure |
| suite_1 lCount 120 (smFilletTest.cpp:1251) asserts 18 edges | Fillet/shell_fillet.smb | covered | `smlib_fillets::case_120_shell_fillet` | known failure |
| suite_1 lCount 121 (smFilletTest.cpp:1268) asserts 57 edges | Fillet/6_sided_corner.smb | covered | `smlib_fillets::case_121_6_sided_corner` | known failure |
| suite_1 lCount 122 (smFilletTest.cpp:1293) asserts 8 edges | Fillet/cyl_sphere.smb | covered | `smlib_fillets::case_122_cyl_sphere` | known failure |
| suite_1 lCount 123 (smFilletTest.cpp:1313) asserts 60 edges | Fillet/corner_box.smb | covered | `smlib_fillets::case_123_corner_box` | - |
| suite_1 lCount 124 (smFilletTest.cpp:1330) asserts 48 edges | Solids/box.smb | partial | `smlib_fillets::case_124_box` | SMLib sets `bMakeBevelCorner` (flat bevel corners); our test fillets with default round corners (volume/area from the oracle run without bevel). Kernel has no bevel-corner option |
| suite_1 lCount 125 (smFilletTest.cpp:1350) asserts 32 edges | Solids/pyramid.smb | covered | `smlib_fillets::case_125_pyramid` | chamfer all edges (NxN); known failure |
| suite_1 lCount 126 (smFilletTest.cpp:1370) asserts 53 edges | Fillet/torii_fillet.smb | covered | `smlib_fillets::case_126_torii_fillet` | - |
| suite_1 lCount 127 (smFilletTest.cpp:1395) asserts 14 edges | Fillet/extrusion0206.smb | covered | `smlib_fillets::case_127_extrusion0206` | chamfer; also asserts 8 faces (in Expect); known failure |
| suite_1 lCount 128 (smFilletTest.cpp:1432) asserts 22 edges | Fillet/shell_fillet2.smb | covered | `smlib_fillets::case_128_shell_fillet2` | known failure |
| suite_2 lCount 201 (smFilletTest.cpp:1539) asserts 5 edges | Fillet/test1.smb | covered | `smlib_fillets::case_201_test1` | known failure, slow/ignored |
| suite_2 lCount 202 (smFilletTest.cpp:1564) asserts 5 edges | Fillet/fillet0114.smb | covered | `smlib_fillets::case_202_fillet0114` | assert 5 edges at r=1.0 (11 at r=3.175 is commented out); known failure, slow/ignored |
| suite_2 lCount 203 (smFilletTest.cpp:1594) asserts 7 edges | Fillet/torii_unioned.smb | covered | `smlib_fillets::case_203_torii_unioned` | known failure, slow/ignored |
| suite_2 lCount 204 (smFilletTest.cpp:1612) asserts 23 edges | Fillet/test4.smb | covered | `smlib_fillets::case_204_test4` | known failure |
| suite_2 lCount 205 (smFilletTest.cpp:1631) asserts 20 edges | Fillet/test5.smb | covered | `smlib_fillets::case_205_test5` | known failure |
| suite_2 lCount 206 (smFilletTest.cpp:1651) asserts 35 edges | Fillet/test6.smb | covered | `smlib_fillets::case_206_test6` | known failure |
| suite_2 lCount 207 (smFilletTest.cpp:1672) asserts 26 edges | Fillet/test7.smb | covered | `smlib_fillets::case_207_test7` | known failure |
| suite_2 lCount 208 (smFilletTest.cpp:1691) asserts 9 edges | Fillet/test8.smb | covered | `smlib_fillets::case_208_test8` | known failure, slow/ignored |
| suite_2 lCount 209 (smFilletTest.cpp:1709) asserts 21 edges | Fillet/test9.smb | covered | `smlib_fillets::case_209_test9` | - |
| suite_2 lCount 210 (smFilletTest.cpp:1728) asserts 55 edges | Fillet/sphere_dif_box.smb | covered | `smlib_fillets::case_210_sphere_dif_box` | known failure |
| suite_2 lCount 211 (smFilletTest.cpp:1745) asserts 13 edges | Fillet/cyl_cyl_shell.smb | covered | `smlib_fillets::case_211_cyl_cyl_shell` | known failure, slow/ignored |
| suite_2 lCount 212 (smFilletTest.cpp:1764) asserts 6 edges | Fillet/sphere_plane.smb | covered | `smlib_fillets::case_212_sphere_plane` | - |
| suite_2 lCount 213 (smFilletTest.cpp:1782) asserts 13 edges | Fillet/bague.smb | covered | `smlib_fillets::case_213_bague` | known failure, slow/ignored |
| suite_2 lCount 214 (smFilletTest.cpp:1800) asserts 10 edges | Fillet/cap1.smb | covered | `smlib_fillets::case_214_cap1` | known failure |
| suite_2 lCount 215 (smFilletTest.cpp:1820) asserts 10 edges | Fillet/cap1.smb | covered | `smlib_fillets::case_215_cap1` | known failure, slow/ignored |
| suite_2 lCount 216 (smFilletTest.cpp:1839) asserts 14 edges | Fillet/caps_coons.smb | covered | `smlib_fillets::case_216_caps_coons` | known failure, slow/ignored |
| suite_2 lCount 217 (smFilletTest.cpp:1858) asserts 37 edges | Fillet/caps_nside.smb | covered | `smlib_fillets::case_217_caps_nside` | known failure, slow/ignored |
| suite_2 lCount 218 (smFilletTest.cpp:1880) asserts 16 edges | Fillet/fillet1010.smb | covered | `smlib_fillets::case_218_fillet1010` | known failure |
| suite_2 lCount 219 (smFilletTest.cpp:1901) asserts 15 edges | Fillet/fillet0227.smb | covered | `smlib_fillets::case_219_fillet0227` | known failure, slow/ignored |
| suite_2 lCount 220 (smFilletTest.cpp:1923) asserts 48 edges | Fillet/vase.smb | covered | `smlib_fillets::case_220_vase` | - |
| suite_2 lCount 221 (smFilletTest.cpp:1952) asserts 64 edges | Fillet/vase1.smb | covered | `smlib_fillets::case_221_vase1` | known failure |
| suite_2 lCount 222 (smFilletTest.cpp:1977) asserts 48 edges | Fillet/vase2.smb | covered | `smlib_fillets::case_222_vase2` | - |
| suite_2 lCount 223 (smFilletTest.cpp:2005) asserts 112 edges | Fillet/vase3.smb | covered | `smlib_fillets::case_223_vase3` | known failure, slow/ignored |
| suite_2 lCount 224 (smFilletTest.cpp:2022) asserts 15 edges | Fillet/0311.smb | covered | `smlib_fillets::case_224_0311` | known failure |
| suite_2 lCount 225 (smFilletTest.cpp:2040) asserts 38 edges | Fillet/box_diff_cyl.smb | covered | `smlib_fillets::case_225_box_diff_cyl` | no SMLib assert (validity/oracle only) |
| suite_2 lCount 226 (smFilletTest.cpp:2058) asserts 64 edges | Fillet/blade_hub.smb | covered | `smlib_fillets::case_226_blade_hub` | known failure, slow/ignored |
| suite_2 lCount 227 (smFilletTest.cpp:2079) asserts 45 edges | Fillet/fillet1018.smb | covered | `smlib_fillets::case_227_fillet1018` | known failure, slow/ignored |
| suite_2 lCount 228 (smFilletTest.cpp:2104) asserts 48 edges | Fillet/crown.smb | covered | `smlib_fillets::case_228_crown` | - |
| suite_2 lCount 229 (smFilletTest.cpp:2123) asserts 24 edges | Fillet/loft_brick.smb | covered | `smlib_fillets::case_229_loft_brick` | known failure |
| suite_2 lCount 230 (smFilletTest.cpp:2144) asserts 43 edges | Fillet/box_extrusion1.smb | covered | `smlib_fillets::case_230_box_extrusion1` | - |
| suite_2 lCount 231 (smFilletTest.cpp:2163) asserts 45 edges | Fillet/box_extrusion2.smb | covered | `smlib_fillets::case_231_box_extrusion2` | no SMLib assert (validity/oracle only); known failure, slow/ignored |
| suite_2 lCount 232 (smFilletTest.cpp:2183) asserts 53 edges | Fillet/box_w_slot.smb | covered | `smlib_fillets::case_232_box_w_slot` | - |
| suite_2 lCount 233 (smFilletTest.cpp:2201) asserts 15 edges | Fillet/coons_01_03_29.smb | covered | `smlib_fillets::case_233_coons_01_03_29` | - |
| suite_2 lCount 234 (smFilletTest.cpp:2221) asserts 40 edges | Fillet/nurb_cone.smb | covered | `smlib_fillets::case_234_nurb_cone` | known failure |
| suite_2 lCount 235 (smFilletTest.cpp:2246) asserts 24 edges | Fillet/cube_sphere2.smb | covered | `smlib_fillets::case_235_cube_sphere2` | chamfer; known failure |
| suite_2 lCount 236 (smFilletTest.cpp:2268) asserts 26 edges | Fillet/Heart.smb | covered | `smlib_fillets::case_236_heart` | - |
| suite_2 lCount 237 (smFilletTest.cpp:2286) asserts 11 edges | Fillet/knife.smb | partial | `smlib_fillets::case_237_knife` | SMLib asserts 11 edges ("was 12, okay with 11"); our test expects the oracle's 12; known failure, slow/ignored |
| suite_3 lCount 301 (smFilletTest.cpp:2360) asserts 22 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_301_l_shaped_box` | Nx1 zero-radius: linear law 2.0 to 0.0 on edges 0,13 via `fillet_variable` |
| suite_3 lCount 302 (smFilletTest.cpp:2414) asserts 18 edges | Solids/box.smb | covered | `smlib_fillets::case_302_box` | edge 0 r=0.5, edge 1 r=1.5 (Nx2 bevel comment) |
| suite_3 lCount 303 (smFilletTest.cpp:2468) asserts 168 edges | Solids/box.smb | partial | `smlib_fillets::case_303_box` | SMLib sets corners back (`SetSetBackDist(1.25)`) and asserts 168 edges; our test uses plain corners and the oracle's 48. `fillet_smooth(.., setback)` could do it |
| suite_3 lCount 304 (smFilletTest.cpp:2534) asserts 22 edges | Solids/box.smb | covered | `smlib_fillets::case_304_box` | known failure |
| suite_3 lCount 305 (smFilletTest.cpp:2596) asserts 21 edges | Solids/box.smb | partial | `smlib_fillets::case_305_box` | edges 1 and 5 use `SmLinearCrossSectionFSG` (chamfer-like section, dist 2.0); our test fillets them circularly. Mixed fillet/chamfer per edge not expressible in one op |
| suite_3 lCount 306 (smFilletTest.cpp:2658) asserts 44 edges | Fillet/extrusion0210.smb | covered | `smlib_fillets::case_306_extrusion0210` | edges 10-14 r=0.5, edge 24 linear law 2.0 to 0.5; known failure |
| suite_3 lCount 307 (smFilletTest.cpp:2728) asserts 22 edges | Fillet/l_shaped_box.smb | covered | `smlib_fillets::case_307_l_shaped_box` | same law on l_shaped_box edges 0,13 |
| suite_3 lCount 308 (smFilletTest.cpp:2779) asserts 26 edges | Fillet/nurb_box1.smb | covered | `smlib_fillets::case_308_nurb_box1` | - |
| suite_4 lCount 401 (smFilletTest.cpp:2880) asserts 27/28 edges | Drop/FilletTests/ext_large_radius1.fdf | covered | `smlib_fillets::case_401_extrusion_old` | fdf `ext_large_radius1.fdf` edge 0 r=3.8; source accepts 27 or 28 edges, test uses 27; known failure |
| suite_4 lCount 402 (smFilletTest.cpp:2894) asserts 27 edges | Drop/FilletTests/l_box_w_boss_large_radius.fdf | covered | `smlib_fillets::case_402_l_shape_with_boss` | fdf `l_box_w_boss_large_radius.fdf` |
| suite_4 lCount 403 (smFilletTest.cpp:2903) asserts 24 edges | Fillet/fillet1019.smb | covered | `smlib_fillets::case_403_fillet1019` | no SMLib assert; GlobalMerge flag has no kernel meaning; known failure |
| suite_4 lCount 404 (smFilletTest.cpp:2922) asserts 29 edges | Fillet/cyl_on_box1.smb | covered | `smlib_fillets::case_404_cyl_on_box1` | - |
| suite_4 lCount 405 (smFilletTest.cpp:2940) asserts 17 edges | Fillet/cyl_on_box1.smb | covered | `smlib_fillets::case_405_cyl_on_box1` | known failure |
| suite_4 lCount 406 (smFilletTest.cpp:2958) asserts 23 edges | Fillet/2cyls_on_box.smb | covered | `smlib_fillets::case_406_2cyls_on_box` | - |
| suite_4 lCount 407 (smFilletTest.cpp:2977) asserts 27 edges | Fillet/cyl_on_box_w_fillet.smb | covered | `smlib_fillets::case_407_cyl_on_box_w_fillet` | - |
| suite_4 lCount 408 (smFilletTest.cpp:2996) asserts 27 edges | Fillet/cyl_on_box_w_fillet1.smb | covered | `smlib_fillets::case_408_cyl_on_box_w_fillet1` | - |
| suite_4 lCount 409 (smFilletTest.cpp:3014) asserts 40 edges | Fillet/cyl_on_box_w_fillet2.smb | covered | `smlib_fillets::case_409_cyl_on_box_w_fillet2` | - |
| suite_4 lCount 410 (smFilletTest.cpp:3032) asserts 128 edges | Fillet/blade_hub_1.smb | covered | `smlib_fillets::case_410_blade_hub_1` | known failure, slow/ignored |
| suite_4 lCount 411 (smFilletTest.cpp:3053) asserts 92 edges | Fillet/blade_hub_2.smb | covered | `smlib_fillets::case_411_blade_hub_2` | known failure, slow/ignored |
| suite_4 lCount 412 (smFilletTest.cpp:3073) asserts 137 edges | Fillet/blade_hub_3.smb | covered | `smlib_fillets::case_412_blade_hub_3` | known failure, slow/ignored |
| suite_4 lCount 413 (smFilletTest.cpp:3092) asserts 20 edges | Fillet/cliff_3.smb | covered | `smlib_fillets::case_413_cliff_3` | - |
| suite_4 lCount 414 (smFilletTest.cpp:3110) asserts 17 edges | Fillet/cliff_4.smb | covered | `smlib_fillets::case_414_cliff_4` | - |
| suite_4 lCount 415 (smFilletTest.cpp:3128) asserts 26 edges | Fillet/cyl_on_box_w_hole.smb | covered | `smlib_fillets::case_415_cyl_on_box_w_hole` | - |
| suite_4 lCount 416 (smFilletTest.cpp:3146) asserts 24 edges | Fillet/rollover_corner.smb | covered | `smlib_fillets::case_416_rollover_corner` | - |
| suite_4 lCount 417 (smFilletTest.cpp:3165) asserts 35 edges | Fillet/cyl_on_box_w_tan_hole.smb | covered | `smlib_fillets::case_417_cyl_on_box_w_tan_hole` | - |
| suite_4 lCount 418 (smFilletTest.cpp:3184) asserts 20 edges | Fillet/box_diff_cyl_big_rad.smb | covered | `smlib_fillets::case_418_box_diff_cyl_big_rad` | - |
| suite_4 lCount 419 (smFilletTest.cpp:3203) asserts 37 edges | Fillet/box_diff_3cyls.smb | covered | `smlib_fillets::case_419_box_diff_3cyls` | no SMLib assert; GlobalMerge flag has no kernel meaning |
| suite_4 lCount 420 (smFilletTest.cpp:3222) asserts 27 edges | Fillet/cyl_on_box_w_hole.smb | covered | `smlib_fillets::case_420_cyl_on_box_w_hole` | - |
| suite_4 lCount 421 (smFilletTest.cpp:3241) asserts 39 edges | Fillet/blade_fillet_fails.smb | covered | `smlib_fillets::case_421_blade_fillet_fails` | `SmFilletExecutive` r=5 on edges 7,6,5,9,8 with circular section + self-intersection handler; asserts 39 edges (3 lines of option-setting in source: stepback 1.0, classification on); known failure, slow/ignored |

Totals (fillets suite, counting cases): covered 96, partial 4, missing 0, gap 4, n/a 2.
Of the 96 covered, 56 are known failures (the kernel's result does not yet match; the test asserts the failure); the 4 partial are 124/237/303/305.

### my_local_op (local-ops suite)

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_local_op lCount 0 (smFilletTest.cpp:3531) | Fillet/nurb_box.smb | n/a | - | read + draw only |
| lCount 1 (3546) | Fillet/nurb_box.smb | covered | `smlib_local_ops::local_op_1_nurb_box` | face 1 -> plane (0,0,12) normal (0,1,1)/sqrt2 over domain; SMLib asserts only success; ours checks oracle volume/area/counts. Kernel `replace_faces` |
| lCount 2 (3599) | Solids/box.smb | n/a | - | read + draw only |
| lCount 3 (3613) | Solids/box.smb | covered | `smlib_local_ops::local_op_3_box` | face 1 -> cylinder r=5; marked "needs fixing" in SMLib (its result is not a solid); ours uses closed-form volume/area |
| lCount 4 (3664) | Solids/cylinder.smb | n/a | - | read + draw only |
| lCount 5 (3681) | Solids/cylinder.smb | covered | `smlib_local_ops::local_op_5_cylinder` | face 1 -> plane x=-3; "Needs fixing" in SMLib; closed form + oracle |
| lCount 6 (3734) | Solids/box.smb | n/a | - | read + draw only |
| lCount 7 (3751) | Solids/box.smb + offset_nurb.sms | covered | `smlib_local_ops::local_op_7_box` | face 4 -> B-spline surface reparameterized to the face domain |

Totals (local-ops): covered 4, partial 0, missing 0, gap 0, n/a 4.

---

### smOffsetTest.cpp

Entry point: suite `offset` = `my_offset_regression` (smOffsetTest.cpp:133, default on): 17 iterations of `my_offset_demo` (lCount 1-17) then 16 iterations of `my_shell_demo` (lCount 1-16). Note `my_offset_demo` lCount 18 (`airfoil_offset`, line 1033) is never reached (loop is `i<17`, lCount reaches 17 at the last call), and `airfoil_offset` reads `airfoil.brep` relative to cwd, so offset.py's NOT_CONVERTED reason "its model isn't in the checkout" is inaccurate: a copy exists at TestFiles/pt_TestFiles/Solids/airfoil.brep, but the case is dead anyway. `my_shell_demo`: lCount 0 and 12-15 are inside `#if 0` / `if (0 && ...)`; live cases are 1-11 (lCount 12-16 calls are no-ops). Helpers: `my_test_offset` (30, SM_OO_SOLID_OFFSET with extend-convex FALSE, merge TRUE), `ShellBrep` (190, offset or inset with shell faces kept, then `my_test_bbu` difference with the input). I checked every case's file, distance, kept/open face and bExtendCorners against the generated doc lines: all match.

| Item (function or case, file:line) | Fixtures (TestFiles/pt_TestFiles/) | Status | Our test(s) | Notes |
|---|---|---|---|---|
| offset_demo 1 (smOffsetTest.cpp:781) ShellBrep -2.54, face 1 open, extend FALSE (rounded) | Solids/OffsetBoxCylOnCorner_edited.smb | covered | `smlib_offset::offset_01_offsetboxcyloncorner_edited` | ValidateCounts(17,42,28,..) ignored by SMLib; ours faces/edges (planar only) + oracle volume/area. `shell_with` |
| offset_demo 2 (795) ShellBrep -2.54, face 5 | Solids/OffsetBoxWithBoss.smb | covered | `offset_02_offsetboxwithboss` | |
| offset_demo 3 (808) ShellBrep -2.54, face 0 | Solids/OffsetBoxSphOnCorner.smb | covered | `offset_03_offsetboxsphoncorner` | |
| offset_demo 4 (820) my_test_offset 0.3 | Solids/off_extrusion.smb | covered | `offset_04_off_extrusion` | known failure (slow): rounded offset union leaves open shell |
| offset_demo 5 (831) my_test_offset 2.2 | Solids/off_extrusion.smb | covered | `offset_05_off_extrusion` | known failure (tubes fold over); SMLib's own result is not a manifold solid, asserts dropped |
| offset_demo 6 (855) my_test_offset -0.3 keep face 1, then `my_test_bbu(pOrig,pOffset,2)` | Solids/off_extrusion.smb | partial | `offset_06_off_extrusion` | inset converted; the follow-up difference (input minus inset, boolean must succeed) is not run. Kernel `boolean(Subtract)` could do it |
| offset_demo 7 (871) my_test_offset 1.0 | Solids/off_extrusion.smb | covered | `offset_07_off_extrusion` | known failure (slow) |
| offset_demo 8 (882) my_test_offset 0.3 ("fails under LINUX") | Solids/boolean_trees_result.smb | covered | `offset_08_boolean_trees_result` | known failure |
| offset_demo 9-14 (893, 904, 915, 926, 937, 948): radii 0.5, 1.5, 2.5, 3.5, 5.1, 7.1 | Solids/offset_bracket2.smb | covered | `offset_09` ... `offset_14_offset_bracket2` | lCount 14 "produces 6 regions" in SMLib (bug) |
| offset_demo 15 (959): offset 1.5 and 2.6, each `my_test_bbu(...,2)` with cutter | Solids/offset_bracket2.smb, offset_bracket_box.smb | covered | `offset_15a_offset_bracket2`, `offset_15b_offset_bracket2` | two outcomes (302 / 297 edges asserted in ValidateCounts, ours oracle 312 / 305) |
| offset_demo 16 (999) ShellBrep -12.7, face 0, extend FALSE | Solids/InsetCone.smb | covered | `offset_16_insetcone` | |
| offset_demo 17 (1018) ShellBrep -12.7, face 2, extend TRUE ("zero-area loop") | Solids/InsetTaperedExtrude.smb | covered | `offset_17_insettaperedextrude` | SMLib result not a solid; faces/edges only |
| offset_demo 18 (1033) airfoil_offset | Solids/airfoil.brep (cwd-relative path in source) | n/a | - | never executed by the regression (17 iterations) |
| shell_demo 1-11 (517, 531, 545, 559, 574, 588, 603, 618, 632, 646, 661) ShellBrep (extend TRUE): problemShell -2.54 f1; problemShell1 -4.58 f4; problemShell2 -12.7 f1; problemShell21 -12.7 f2; problemShell3 -5.08 f0; problemShell4 +5.08 f2 (outward); problemShell5 -5.08 f5; problemShell6 -2.54 f5; problemShell10 -12.7 f5; problemShell16 -12.7 f1; problemShell10 -12.7 f1 | Shell/problemShell{,1,2,21,3,4,5,6,10,16,10}.smb | covered | `shell_01_problemshell` ... `shell_11_problemshell10` | one test each, no known failures. lCount 6 asserts 7 edges in SMLib "possible bug", ours oracle 10 |
| shell_demo 0 (481, `#if 0`), 12 (677, 699 `#if 0`), 13 (713), 14 (727), 15 (741 `0 &&`) | Shell/ShellBrep_471, BoxVRFillet, problemShell15, 18, ShellBrep_983, OpenBox | n/a | - | compiled out; NOT_CONVERTED correct |

Totals (smOffsetTest.cpp, counting cases): covered 26, partial 1, missing 0, gap 0, n/a 6.
(Cases: offset_demo 17 live, of which 16 covered + 1 partial; shell_demo 11 covered; n/a = offset_demo 18 + shell_demo 0, 12, 13, 14, 15.)

---

### smMergeTest.cpp

Entry points: suite `booleans` = `my_test_merge` (smMergeTest.cpp:1528, default on); suite `stitch` = `my_stitching_demo` (343, default off) which `my_test_merge` also calls (1965). `my_test_merge` lines 1538-1716 are two `if (FALSE)` blocks (dead). Live blocks in order (return values of the sub-demos are discarded; only SM_ASSERT and SER-inside-callee abort): `my_test_non_manifold()` (1721) and again (1982, identical), the four primitive pairs, two Piecewise Merge blocks, box merged with copy, `my_stitching_demo`, `my_mouse_demo`, `my_test_stress`, `my_test_BrepPerShell`, boolean_trees union and merge, three grid files via `my_two_prim_test`, and a nurb offset-surface merge. Helper `my_test_bbu` (2184): ManifoldBoolean op 0 U / 1 I / 2 D / 3 MERGE / 4 XOR (translates inputs near |c|^2>1e7 first). The kernel has U/I/D; MERGE (cell-complex merge) is out of scope, as booleans.py's NOT_CONVERTED says. `my_boolean_stress_test` (201), `my_two_prim_test` (38), `my_many_prim_test` (137) are helpers.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| non_manifold 11: box U plane.smb (1328) | Solids/box.smb, plane.smb | gap | - | SMLib result keeps the sheet inside the box (non-manifold) |
| non_manifold 12: box I plane.smb (1356, bbu at 1374) | same | covered | `smlib_booleans::box_intersection_plane` | result is a sheet |
| non_manifold 13: box - plane.smb (1384) | same | gap | - | non-manifold result in SMLib; (success+validity-only check might be possible, see candidates) |
| non_manifold 14: plane.smb - box (1412, bbu at 1430) | same | covered | `smlib_booleans::plane_difference_box` | sheet result |
| non_manifold 15: merge box/plane (1440) | same | gap | - | SM_BO_MERGE |
| non_manifold 1-5 box+wires: U, I, box-wires, wires-box, merge (995, 1029, 1062, 1095, 1129) | built in code (MakeWireEdgeVertex) | gap | - | wire bodies / non-manifold cell complexes |
| non_manifold 6-10 box+vertex shell: U, I, box-v, v-box, merge (1163, 1196, 1229, 1262, 1295) | built in code | gap | - | vertex shells |
| non_manifold 16, 17: union of two wires (1468, 1493) | built in code | gap | - | wires |
| (my_test_non_manifold runs twice, 1721 and 1982: identical, counted once) | | | | |
| Two boxes U (1726; asserts 24 edges) | Prim: box 10x10x10 U 10x7x5 at (2,2,0) | covered | `prim_box_union`, plus added `prim_box_intersection`, `prim_box_difference` | 24 edges compared (planar) |
| Two cones U (1754; no assert, 9 commented) | Prim: cone | covered | `prim_cone_union` (+ I, D added) | |
| Two spheres U (1782; asserts 6 edges) | Prim: sphere r10 at (0,0,0)/(5,2,0) | partial | `prim_sphere_union` (+ I, D added) | Expect.edges=6 recorded but not compared (non-planar); volume/faces checked |
| Two tori U (1810; assert commented out) | Prim: torus | covered | `prim_torus_union` (+ I, D added) | |
| Piecewise Merge #1 (1852): `PiecewiseMerge(NULL,TRUE,FALSE)` of box and a moved copy | Solids/box.smb | gap | - | no piecewise merge in kernel |
| Piecewise Merge #2 (1881): `PiecewiseMerge(NULL,TRUE,TRUE)` | Solids/box.smb | gap | - | |
| Merge of box with copy of itself (1943; bbu op 3) | Solids/box.smb | gap | - | SM_BO_MERGE of coincident solids |
| my_stitching_demo step 1: read + StitchFaces 0.001 (350-386) | Solids/fillet_ts.smb | covered | `smlib_stitch::stitch_1_fillet_ts_0001` | SMLib asserts nothing; ours: valid, <= oracle's lamina edges |
| step 2: StitchFaces 0.005 (413) | Solids/fillet_ts.smb | covered | `smlib_stitch::stitch_2_fillet_ts_0005` | closed solid, volume 924.8286, area 605.236 |
| step 3: read fillet_cut.smb (440) | Solids/fillet_cut.smb | covered | `smlib_stitch::stitch_3_read_fillet_cut` | |
| step 4: `my_two_prim_test(...,2)` stitched part - cut (461) | fillet_ts.smb, fillet_cut.smb | covered | `smlib_stitch::stitch_4_cut` | known failure (boolean leaves an open shell) |
| my_mouse_demo (1970; 654): merge mouse_1..5 (five one-face sheets) via `my_many_prim_test` op 3, `MakeManifold`, `OrientTrimmedSurfaces` | Solids/mouse_1..5.smb | gap | - | SM_BO_MERGE chain of sheets; only success asserted. See candidates (sew) |
| my_test_stress S1 (883, bbu 908): box - box rotated 0.1 deg about z (op 2) | Solids/box.smb x2 | missing | - | kernel `transform` + `boolean(Subtract)`; oracle `boolean` takes no transform; SMLib asserts only success (near-coincident faces stress) |
| my_test_stress S2 (920): merge extrudeHoles1/2 (op 3), ValidateCounts(8,19,20,0,6,12,2,8), asserts 2 regions, 7 shells | Boolean/extrudeHoles1.smb, 2.smb | gap | - | MERGE, vertex shells |
| my_test_stress S3 (947): merge BoltMBox1/2 (op 3), ValidateCounts(83,162,85,0,0,114,6,6) | Boolean/BoltMBox1.smb, 2.smb | gap | - | MERGE |
| my_test_stress S4 (963): union BoxUCylTan1/2 (tol 0), ValidateCounts(9,21,14,..) | Boolean/BoxUCylTan1.txt, 2.txt | covered | `smlib_booleans::stress_boxucyltan_union` | SMLib's counts not enforced (return ignored); ours uses the oracle's 24 edges |
| my_test_BrepPerShell (1988; 495): non-manifold box with wire/vertex/circle then `NonManifoldBoolean` diff and union, `CreateOneBrepPerBody` | built in code | gap | - | non-manifold booleans; only validity asserted |
| boolean_trees union (1992): `my_many_prim_test` op 0, asserts 45 edges | Solids/boolean_trees.smp | covered | `smlib_booleans::boolean_trees_union` | 45 compared only if inputs planar |
| boolean_trees merge (2014): op 3, asserts 59 edges | Solids/boolean_trees.smp | gap | - | MERGE |
| box_box_grid pairs 0-24 except 3 (2035; `my_two_prim_test` op 0) | Solids/box_box_grid.smp | covered | `box_box_grid_00..24` (24 tests) | per-pair: valid solid + oracle volume/area |
| box_box_grid pair 3 (B-reps 6,7) | same | gap | - | parts touch; SMLib union is non-manifold |
| box_box_grid total asserts 659 edges | same | covered | `box_box_grid_all` | planar: sum compared (636 + dropped pair's edges) |
| box_cyl_grid pairs 0-24 except 6 (2051) | Solids/box_cyl_grid.smp | covered | `box_cyl_grid_00..24` (24) | |
| box_cyl_grid pair 6 | same | gap | - | non-manifold union |
| box_cyl_grid total asserts 499 edges | same | partial | `box_cyl_grid_all` | non-planar: total not compared |
| box_cone_grid pairs 0-24 except 8, 9 (2076) | Solids/box_cone_grid.smp | covered | `box_cone_grid_00..24` (23) | |
| box_cone_grid pairs 8, 9 | same | gap | - | non-manifold union |
| box_cone_grid total asserts 447 edges | same | partial | `box_cone_grid_all` | non-planar: total not compared |
| nurb offset-surface merge (2107): `SmOffsetSurface` r=5 faces of nurb_nurb_1/2 made into one-face breps, bbu op 3, asserts 11 edges | Fillet/nurb_nurb_1.smb, nurb_nurb_2.smb | gap | - | MERGE of two offset-surface sheets |

Totals (smMergeTest.cpp incl. stitching demo, counting cases; the two `my_test_non_manifold` runs and the three `my_test_bbu`-based grid totals counted once): covered 83, partial 3, missing 1, gap 28, n/a 0.
(gap 28 = 15 non-manifold cases + 2 piecewise + 1 merge-with-self + 1 mouse + 2 stress merges + 1 BrepPerShell + 1 trees merge + 4 touching grid pairs + 1 nurb merge; covered includes the stitching demo's 4 steps, of which step 4 is a known failure; partial = sphere pair and the box_cyl / box_cone grid totals.)

Grand totals over the three sources: covered 96+4+26+83 = 209, partial 4+0+1+3 = 8, missing 0+0+0+1 = 1, gap 4+0+0+28 = 32, n/a 2+4+6+0 = 12 (total 262 cases).

---

### Conversion candidates

Ranked by payoff (asserted outcomes x kernel relevance). Honest summary: the three suites are covered case by case; what remains is mostly operations the kernel lacks (MERGE / non-manifold, face-to-face blend) plus a handful of small fidelity gaps.

1. Enforce SMLib's edge counts on curved models (harness change, not a new case). What: `check_result` skips the face/edge count whenever any face is non-planar, so SMLib's asserted counts are not tested for most of the 100 fillet cases (I could not cheaply classify the .smb fixtures by planarity: surface type 15002 covers both planar patches and rational cylinders), the sphere union (6), BoxUCylTan (24 oracle), boolean_trees (45), box_cyl_grid (499) and box_cone_grid (447) totals, and the nurb 11. SMLib asserts: yes, SM_ASSERT in source for all of these (no oracle needed for the number; the oracle already supplies it where SMLib's own value drifted). Kernel API: none new; compare a seam-insensitive count (edges minus seam duplicates, or count after `unify`/merge of split periodic faces). Generator: trivial, the Expect values are already emitted; only the comparison in smlib_common needs a normalized count. Payoff: highest (dozens of asserted outcomes), cost moderate (normalization may expose many kernel/SMLib topology differences, so they would need triage into known_failures).

2. Fillet 303: set-back corners. What: `SetSetBackDist(1.25)` on all 12 box edges, SMLib asserts 168 edges (our test uses plain corners and the oracle's 48, so SMLib's number is unchecked). Kernel API: `Kernel::fillet_smooth(body, edges, &FilletSpec::radius(1.0), 1.25)`. Oracle: no setback option in `fillet`/`chamfer` commands (would need `SetSetBackDist` added to oracle.cpp), but the expected 168 is in the source. Generator: practical (executive_case already detects `SetSetBackDist`; emit `fillet_smooth` and keep the source's 168). 1 case.

3. Offset demo 6 follow-up: `my_test_bbu(pOrig, pOffset, 2, pOrig)` after the -0.3 inset (smOffsetTest.cpp:866): boolean difference input minus inset must succeed. SMLib asserts only success. Kernel API: `boolean(BoolOp::Subtract, input, [inset])`; oracle `boolean a b D` exists. Generator: practical, offset.py already has a `cutter` path for lCount 15; add "subtract the input" for lCount 6. 1 case, near-coincident faces make it a robustness test.

4. my_test_stress S1: box.smb minus box.smb rotated 0.1 deg about z (smMergeTest.cpp:883-909). SMLib asserts success only. Kernel API: `transform` (rotation) + `boolean(Subtract)`; check valid solid and volume against the closed form (the rotated box's sliver). Oracle `boolean` has no transform argument (add `--rot`), but the closed form suffices. Irregular hand-written case (not extractable from the generator's regexes); 1 test, valuable as near-coincident-face stress.

5. Fillet 305 / mixed cross sections: edges 1 and 5 use `SmLinearCrossSectionFSG` (chamfer-like, dist 2.0) with edge 0 circular r=1; SMLib asserts 21 edges. Our test fillets all three. Kernel: `chamfer_with`/`fillet_with` take one kind per call; a per-edge mix would need either two sequential ops (not equivalent) or a new API. Oracle: `chamfer` supports only a uniform distance; add a per-edge section kind to oracle.cpp. Low payoff (1 case), classify as gap unless the kernel gains mixed blends.

6. Fillet 124 bevel corners (`SetBevel(TRUE)` on all corners, box 1.0, asserts 48 edges): kernel has no bevel-corner option (gap). 1 case. Fillet 237: SMLib asserts 11 edges ("could be 10"), test expects the oracle's 12: could switch to a tolerance (10..12) or the source value. Trivial.

7. my_mouse_demo (smMergeTest.cpp:654): five one-face sheets merged and `MakeManifold`, SMLib asserts only success. A partial stand-in is `surfacing::sew(bodies, tol)` over mouse_1..5 plus validity and the oracle's 23 faces / 71 edges (note in booleans.py). Not the same operation (SM_BO_MERGE imprints); hand-written, 1 test.

8. Box U plane.smb and box - plane.smb (non_manifold 11, 13; smMergeTest.cpp:1328, 1384): SMLib's results are non-manifold (sheet kept inside the box, 7 faces); a success + validity + "volume equals the box" check might be possible if the kernel's boolean accepts a sheet tool the way it does for the two converted siblings (12, 14). Needs the kernel to define the semantics; oracle `boolean` can produce SMLib's result for comparison. 2 cases, generator: `non_manifold` blocks all have the same shape (regular), practical.

9. Gaps with no kernel operation (not candidates until the kernel grows them): face-to-face rolling-ball blend of two separate B-reps (fillet 101, 104, 106, 107; asserts 4, 4, 3, 8 edges in source, no oracle command: add one mirroring `my_test_two_brep_fillet`); SM_BO_MERGE / cell-complex merge (piecewise x2, box-with-copy, trees merge 59, extrudeHoles, BoltMBox with ValidateCounts in source, nurb offset merge asserting 11); non-manifold wire / vertex-shell booleans (15 cases in `my_test_non_manifold`, `my_test_BrepPerShell`); union of touching solids with a non-manifold result (4 grid pairs: box_box 3, box_cyl 6, box_cone 8 and 9). Cases are either regular (non_manifold, grids: generator-friendly) or hand-built in code (BrepPerShell, piecewise).

Smaller notes for the generators' docs/NOT_CONVERTED:
- offset.py: airfoil entry reason is wrong (file exists at Solids/airfoil.brep; the real reason is that lCount 18 is never reached and the path is cwd-relative).
- booleans.py: "my_test_non_manifold, 17 cases" counts all 17 blocks while 2 of them (box I plane, plane - box) are converted; the actual unconverted count is 15 (12 wire/vertex-shell blocks + box U plane, box - plane, plane merge).
- fillets.py doc line for case 114 ("SMLib's test asserts 18 edges") is the VARIABLE_RADIUS branch; the run uses 17, which equals the expectation.
- `my_test_non_manifold` runs twice inside `my_test_merge` (1721 and 1982) with identical content, so the duplicate adds no coverage.



Paths: SM = /home/casper/work/nvbrep/usd-brep/tests/prog_test/src; TREE = the cadrs checkout.
Fixtures are under /home/casper/work/nvbrep/usd-brep/TestFiles/pt_TestFiles (abbreviated TF/).

Facts that change what "asserted" means (read first):
- `SM_ASSERT` is a no-op unless `SM_DEBUG_CODE` is defined (SmMessages.h:378 debug, :714 release: `if(a) {}`). The golden run
  (usd-brep/rust/golden/release/pt_*/progTest.txt) is a release build, so every `SM_ASSERT(count == N)` below is compiled out there.
  `SER(...)` (return on error status) and the explicit `return SM_ERR` checks do run. Counts that exist only as SM_ASSERT are marked "(SM_ASSERT)".
- Golden stdout exists for `topology`, `tessellation`, `trimmed-surfaces`, `brep-import`; it confirms which blocks run.
- Oracle subcommands (tools/smlib_oracle/oracle.cpp): props fillet boolean offset shell replace chamfer sweep stitch ssi section silhouette tess cci.
  There is NO distance/solver, no classify, no topology subcommand.

---

### smTopoTest.cpp

Entry points. `my_test_topology` (SM:4616, suite `topology`, prog_test.cpp:48) calls 24 `my_test_*` static functions plus an inline
Euler-operator block. `my_test_brep_import` (SM:5037, suite `brep-import`) calls only `my_test_MakeFace` (SM:4993).
`my_test_trim_surfaces` (SM:4749) is run by `my_test_tsurface_regression` (smAdvancedTest.cpp:1450, 1510, 1573; the call at :1442 is under
`if (FALSE)`). Helpers `my_test_tsurf_creation`, `my_test_analytic_tsurf_creation`, `my_test_tsurf_insert_edge`,
`my_test_tsurface_tsurface_solve`, `my_test_tsurf_section/silhouette`, `my_test_ssi`, `my_topo_regression_test` are called from smAdvancedTest
(tsurface-regression blocks 3, 11, 12, 15-23, 25; sec/sil/ssi suites) and from smSweepTest (`my_test_tsurf_creation`, covered by sweeps).
Never called anywhere (dead helpers): `my_test_tsurface_point_extrema` grid code (after an unconditional `return`, see below),
`my_test_tsurface_curve_solve` etc. are only reached through `my_topo_regression_test`, which runs only for `ellips_ts2.smb`
(smAdvancedTest.cpp:2106; the `cyl_hole.smp` call at :2080 is inside `if (FALSE)`).

### my_test_topology (SM:4616) and its sub-tests

| Item (function, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_poles_without_optional_outputs :4553, 2 cases (UV swap 0/1) | none (SmSphere r2) | n/a | | `SmSphere::GetSingularities` pole mask; SMLib-internal flags. |
| my_test_closed_sphere_properties :4520, 2 cases | none | n/a | | `SmHealData::HealBrep` face-props flag `m_bClosedSurf`; internal. |
| my_test_heal_single_vertex_sphere :4576, 2 cases (swap 0/1) | none | missing | | Sphere r2 face with a single-vertex loop at (0,0,2), `HealBrep(ALL)` -> 1 face, 1 edge (seam), 2 vertices, manifold solid, valid. Kernel `face_from_surface`/`face_from_bare_surface` (face_build.rs:41,335) builds that result; `heal::fix_missing_seams` (heal.rs:441) is the repair. Check counts (non-degenerate edge), `Body::check`, volume 4/3 pi r^3. |
| my_test_doubly_periodic_vertexuse_corners :4384, 3 calls (domains 1x1, 1e-4 x 1, 1 x 1e-4; 16 corner combos each) | none | n/a | | `SmVertexuse` UV choice on a doubly periodic surface; internal bookkeeping. |
| my_test_single_periodic_vertexuse :4312, 4 calls | none | n/a | | Same, single periodic. |
| my_test_face_line_intersect_growth :4245, 5 hole counts (0,3,4,8,16) x 2 modes (all intervals / ray) = 10 | none (`CreateRectangle` + cut holes) | gap | | `SmFace::GlobalLineIntersect`: line in the plane of a sheet with N holes -> N+1 in-face intervals with exact (t, u, v). Kernel has no face-line interval query (`ray_hits` is for transversal crossings of a body). |
| my_test_loop_uv_gap_neighborhood :4115, 12 sub-cases (5 patch cases + 4 iso/weight cases + 3 poles) | none | n/a | | `my_check_loop_uv_gap` on a rational bicubic patch; SmLoop-internal trim healing. |
| my_test_stale_edgeuse_owner :174, 1 | none | n/a | | `ValidatePointers` corruption detection (SmBrepValidationTestAccess). |
| my_test_validate_required_pointers :71, 5 sub-checks (empty, wire shell, null owner, null faceuse, fail-fast) | none | n/a | | Same family; `Body::check` (topo.rs:736) is the kernel's checker but the corruptions are SMLib pointer fields. |
| my_test_squeeze_edge_tracks_removal :222, 1 | none | n/a | | `SmTrackTopologyChanges` / `SmHealData::UpdateTargetLists`. |
| my_test_tracks_changed_objects :261, 1 | none | n/a | | Same. |
| my_test_seam_uv_redistribution_at_poles :325, 8 (2 axes x 4 faces) | none | n/a | | `SmFace::SplitAtSeam`/`CreateUVTrimCurves`; only outcome is "manifold solid and valid". |
| my_test_move_seam_refreshes_face_props :461, 2 (cache vs MoveSeam) | none | n/a | | `SmFaceProps` cache refresh. |
| my_test_tracks_nested_geometry_and_resets :548, 1 | none | n/a | | Change tracker. |
| my_test_missing_poles_refreshes_all_faces :637, 1 | none | n/a | | `SmHealData::Fix_BadLoops` bookkeeping. |
| my_test_get_loop_crossings :3006, 4 cases (empty sheet 2; vertex loop 0; crack 0; hole 2) | none (20x20 `CreateRectangle`) | gap | | `SmLoop::GetLoopCrossings` of y=10 against outer/inner loops; kernel has no loop-vs-line crossing API (the closest is `boolean::section` of the sheet). |
| my_test_face_loop_nesting :3395, 7 reports (nested hole ok; unnested hole; disjoint holes ok; crossing holes; holes touching at one UV point; disjoint ok; loops sharing a vertex) | none | missing | | `AssertValid` must report bad nesting / intersecting / connected inner loops. Kernel `Body::check_with(Full)` (topo.rs:736, nesting rule at :1311/:1386, self-crossing :853) is the counterpart; needs hand-built faces with deliberately bad loops, and "crossing"/"touching" reports may not exist (check first). |
| my_test_coaxial_cylinder_merge :3577, 4 variants (seam +x / -y, 0-180, 0-356, 0-360 deg) | none | missing | | Two cylinder faces r50 on one cylinder, sewn, `RemoveTopologicalEdgesAndVertices` -> 1 valid face, 4 vertices, bbox x +-50, y 0, z -10..13. Kernel: `face_from_bare_surface` x2 + `surfacing::sew_faces` + `HealOp::Unify` (unify.rs `unify_faces`/`merge_edges`). |
| my_test_coaxial_merge_domain :3668, 4 (reverse x broad) | none | missing | | Three cylinder strips (z 0-1,1-2,2-3, different seams) merge to 1 valid face, 2 passes; domain is SMLib's, so check face count, validity, area (180 deg x 3 mm x r50). |
| my_test_surface_cover_domains :3726, 4 checks | none | n/a | | `SmSurface::CoverOtherCoincidentSurface`. |
| my_test_edge_geometry_repairs :3767, 2 checks | none | n/a | | `SmEdge::ValidateGeometry` interval repair. |
| my_test_reversed_line_join :3861, 16 cases (4 flags) | none | n/a | | `SmLine::JoinWith` with inside-out flags; kernel has no curve-class join (only `Nurbs::joined`). |
| my_test_coaxial_merge_orientation :3899, 32 cases (swap/inside-out/creation order) x 3 passes (before, after, repeat) | none | missing | | As coaxial merge with swapped/reversed parametrisations; validity + volume/area. Kernel Surface has no swap/inside-out flags, so only the reversed-face-orientation and creation-order variants are expressible (fewer than 32). |
| inline block :4640-4745 (MakeShellVertex x2, MakeWireEdgeVertex, MakeWireEdge, MakeEdgeTopology x3) | none | n/a | | Euler operators on a bare `SmBrep`; nothing asserted besides `SER`. |

Totals (topology table): covered 0, partial 0, missing 49, gap 14, n/a 66

### Trimmed-surface builders and helpers (my_test_trim_surfaces, my_test_brep_import, helpers of smAdvancedTest suites)

| Item (function, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_tsurf_cyl_creation_fast(TRUE / FALSE) :689, run 100x each from my_test_trim_surfaces :4760-4778 | none | gap | | `MakeFaceWithCurvesFast`/`MakeFaceWithCurves`: 6+3+1 UV-curve loops (outer with notch, triangle hole, circle hole) on a half cone-patch (r5, h10, 180 deg, reparametrised 10x10). Only `SER` + timing. Kernel cannot build a face on an arbitrary surface from uv loops (only planar `mkplane/add_hole`, and `face_from_surface` with a uv rectangle). |
| my_test_tsurf_cyl_creation_2d :831 / _3d :953 | none | gap | | Same face from 2D / 3D curves (`TrimSurfaceWithModelSpaceCurves`). All four builders should give one face: same area. |
| my_test_trimming_tools :1408, face (a): L-shape with triangle and circle holes on 10x10 plane | none | covered | smlib_trimmed_surfaces: `tsurf_creation_volume_area`, `tsurf_creation_point_classes` | Same face as `my_test_tsurf_creation` (built via draw harness polyline/mkplane/add_hole, extruded by 1). |
| my_test_trimming_tools face (b): 20x20 plane with circle r5 outer and circle r3 hole (inner reversed) | none | missing | | Annulus: area 16 pi, valid, `classify_point` in/out/hole. Same harness as face (a): circles via `k.circle`+`mkedge_full`, `mkplane`, `add_hole`. Only `SER`+`SM_ASSERT_VALID` in SMLib. |
| bilinear 1x1 plane with 4 uv line curves (0,0)-(0,.5)-(.5,.5)-(.5,0), `SM_OT_OPPOSITE`, SM:4812-4851 (case 2) | none | missing | | `MakeFaceWithCurves` + valid; closed form area 0.25. Kernel: `face_from_surface` with `UvRect` domain, `mass::face_area`. |
| `ProjectAndTrim` of "blue.sm" curve, SM:4853-4900 | blue.sm, blueCurve1.sm (absent) | n/a | | Inside `if (FALSE)`; files not in TestFiles. NOTE: tools/smlib/trimmed_surfaces.py NOT_CONVERTED lists `ProjectAndTrim` as an unconverted live part of `my_test_trim_surfaces`; it is dead code. |
| my_test_analytic_tsurf_creation :1080 (called smAdvancedTest.cpp:1457) | none | partial | smlib_trimmed_surfaces: `tsurf_analytic_section` | SMLib: plane face and quarter-cylinder face, `SmTopologyIntersector::IntersectInsertRelate`; asserts nothing but `SER`. Ours checks the section edge length (pi). |
| my_test_tsurf_creation :1259 (called from tsurface-regression 12, insert_edge, smSweepTest) | none | covered | smlib_trimmed_surfaces: `tsurf_creation_volume_area`, `tsurf_creation_point_classes`; sweeps use it | |
| my_test_tsurf_insert_edge :1590 (smAdvancedTest.cpp:1554) live block 1: `SmBrepCutting::DoCut` of the tsurf face by plane cutters (5.1,5.1,0) n(-1,-1,0) then (6.5,6.5,0) n(1,1,0) | none | missing | | `SER` only. Kernel `Kernel::split_faces` with `SplitTool::Plane` (types.rs:675); check face count 2 then 3, valid, total area preserved. |
| my_test_tsurf_insert_edge live block 2: `MergeCurvesOnSurface` of the line y=5.5 on the tsurf face | none | gap | | Splitting a face by a 3D curve lying on its surface; `SplitTool` has only Plane, Face, Body (a sheet of one line is not a Body). Partial workaround: plane y=5.5 (equivalent for a planar face). |
| my_test_tsurf_insert_edge `#if 0` region :1697-1884 (copy/transform and 6 more blocks) | | n/a | | Unreachable (`return SM_SUCCESS` at :1884 region start precedes it). |
| my_test_MakeFace :4993 via my_test_brep_import :5037 | MakeFace/ApproximateSphere.smp | covered | smlib_brep_import: `make_face_approximate_sphere` | ValidateCounts(1,1,2,0,0,1,2,2). Golden prints it. |

### my_test_tsurface_tsurface_solve (SM:2585), asserted counts in tsurface-regression (smAdvancedTest.cpp:1510-1960)

64 live `SM_ASSERT(lCount == N)` (SM_ASSERT: compiled out in release). Each block ends with 3 more under `#if ADDITIONAL_PROJ_TESTS`
(never defined): dead, not counted. Block 10 uses ss_plane_sph1 (directed ops only), block 17 the same file (all ops).

| Item | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| MINIMIZE / MAXIMIZE on ss_plane_sph1 (blk 17: 1 / 3), ss_2parallel_planes (blk 15: 69 / 36), ss_2adj_planes (blk 16: 34 / 18); 6 asserts | TF/ss_plane_sph1.smp, ss_2parallel_planes.smp, ss_2adj_planes.smp | partial | smlib_trimmed_surfaces: `tsurf_solve_*` (6 tests) | Closed-form distance values instead of SMLib's solution counts (counts are sampling artefacts). |
| MINIMIZE / MAXIMIZE on ss_plane_sph2 (6 / 3), sph3 (2 / 3), sph4 (1 / 3), tor1 (1 / 3), tor2 (>10 / 3), tor_tor (1 / 1); 12 asserts | TF/ss_plane_sph2.smp, sph3, sph4, tor1, tor2, ss_tor_tor.smp | missing | | `Kernel::min_distance`/`max_distance` between face 0 of brep 0 and 1 (as `solve()` in trimmed_surfaces.py). Expected values are not in the source; closed forms are hard on tori, so needs a `distance` oracle subcommand (BrepBrepSolve MINIMIZE/MAXIMIZE, report the best value). None exists. |
| SM_SO_NORMALIZE (blk 15: 1, 16: 0, 17: 1, 18: 1, 19: 0, 20: 0, 21: 4, 22: 18, 23: 16); 9 asserts | all nine | gap | | Normalize solver (points where the surface normals line up): no kernel operation. |
| PROJECTED_MIN/MAX, DIRECTED_MIN/MAX, SIGNED_DIRECTED_MIN: blk 10 (3: 4/8/4), 17 (5), 18 (4), 19 (5), 20 (5), 21 (5), 22 (5), 23 (5); 37 asserts | sph1..tor_tor | gap | | Viewed-plane and directed distance solvers: no kernel operation; counts are solution multiplicities. (Already listed in trimmed_surfaces.py NOT_CONVERTED.) |

### Sec/sil/ssi helpers and my_topo_regression_test

| Item | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_ssi :2171 (called by my_test_ssi_adv smAdvancedTest.cpp:1069; :1481 is `if (FALSE)`) | per case smp files | covered | smlib_ssi_advanced | Detail belongs to the smAdvancedTest audit. |
| my_test_tsurf_section :1898 (my_test_sec_adv; planes at box points) | torus.smb etc. | partial | smlib_section | Lengths/loops against closed forms, not curve counts (documented in section.py). |
| my_test_tsurf_silhouette :1977 (my_test_sil_adv, 34 cases) | 7 models | partial | smlib_silhouette | Total length vs oracle + on-surface; counts only none/some. |
| my_topo_regression_test block 25 (smAdvancedTest.cpp:2106) on ellips_ts2.smb (1 face): point classify 33x33 uv grid per face counts points in the face (389) | TF/ellips_ts2.smb | missing | | Count of grid points whose `PointClassify` is `SM_PC_FACE`. Needs a uv-in-face query: kernel has none public (`boolean.rs:582 point_in_face` is private); workaround: evaluate the surface at grid points and `Kernel::classify` on the sheet. (SM_ASSERT) |
| ... point intersect (`BrepPointSolve INTERSECT`) at the same grid: 389 | same | missing | | Must equal the classify count; kernel: `min_distance(Point, Face) <= tol`. (SM_ASSERT) |
| ... point_extrema MINIMIZE 362 / MAXIMIZE 52 / NORMALIZE 60 | same | n/a | | `my_test_tsurface_point_extrema` returns after a fixed benchmark point (SM:2255-2310, `if (TRUE)`, comment "currently turned off" is stale), so one count is returned for all three ops: at most 10 x 10 solutions; 362/52/60 cannot all hold. Asserts are dead in release; would fail in debug. |
| ... that benchmark point (2811.4739500810206, 590.21281639457879, 145.44695932094584): minimum distance to the brep, printed 2870.476090 (golden pt_trimmed-surfaces/progTest.txt) | same | missing | | Real expected value, not asserted: `Kernel::min_distance(DistanceTarget::Point, DistanceTarget::Body)` == 2870.476090 (1e-6). |
| ... curve_classify: 19 concentric uv ellipses per face, intervals classed in-face: 59 | same | gap | | `CurveOnClassify` of a uv curve against a face; no kernel operation. (SM_ASSERT) |
| ... curve_solve INTERSECT, 5x5 vertical lines through the bbox: 7 | same | missing | | `Kernel::ray_hits` per line, keep t in [0, bbox height]. (SM_ASSERT) |
| ... curve_solve MINIMIZE 36 / MAXIMIZE 68 | same | gap | | Number of local extrema line-to-brep: kernel gives only the global one. |
| ... CreateParallelProjectionCurves of 10 circles along -z: 30 pieces | same | gap | | Projecting a curve onto a body's faces (3D + uv curves); kernel `project` is hidden-line removal only. |
| ... section, 10 planes normal (1,1,1): 8 curves | same | missing | | Oracle `section in 1 1 1 10` exists; kernel `boolean::section`. Counts are SMLib's trimmed pieces (see section.py: counts disagree there), so compare total length. (SM_ASSERT) |
| ... `my_test_tsurf_silhouette(pBrep,1e-4,(1,0,-0.1),parallel)`: 2 | same | missing | | Oracle `silhouette in 1 0 -0.1` exists; kernel `Kernel::silhouettes`. Compare lengths like silhouette.py. (SM_ASSERT) |
| my_topo_regression_test block 24 (smAdvancedTest.cpp:2080) on cyl_hole.smp: 11 counts + silhouette 6 | TF/cyl_hole.smp | n/a | | Inside `if (FALSE)`. 12 asserts. |

Totals: covered 4, partial 9, missing 70, gap 69, n/a 83
(by table: topology 0/0/49/14/66; builders/helpers: covered 3, partial 1, missing 3, gap 5, n/a 2; solve: partial 6, missing 12, gap 46;
sec/sil/ssi/topo-regression: covered 1, partial 2, missing 6, gap 4, n/a 15 = 3 dead extrema asserts + 12 dead cyl_hole asserts)

---

### smTessTest.cpp

Entry points. Suite `tessellation` -> `my_test_tess` (SM:2365). It runs, in order: face/normal-table tests (3 calls), `my_test_poly_solver_1..4` (SM:394, 942, 1355, 1756),
tess tests 1-14 (SM:2377-2931, numbered by its own `MYPRINTF`), then `my_test_decimation_regression` (SM:3850, test 15) and
`my_poly_boolean_regression` (SM:3559, test 16). The golden stdout confirms every block runs. Our `tessellation.py` converts only the 5 tests that
mesh a B-rep with the kernel (tests 4-8, `my_test_one_tess` and the stitched example). Everything else in the file is SmPolyBrep (polygon-mesh library) code,
EXCEPT that several of those tests have a kernel counterpart through meshes the kernel does have: `tess::mesh`/`TriMesh` with normals, `exchange::read_stl_ascii`
(exchange.rs:239), `meshsolid::mesh_solid` (a welded closed mesh -> Body), `ray_hits`, booleans. Almost all of these tests assert only `SER` success
(the only real asserts: `rbFailedFaces == FALSE`, `bHitsSomething == TRUE`, `lNumberLaminaRemaining == 0`, `sSolutions.GetSize() > 0` (SM_ASSERT), and
the normal-table checks). `PolyBrep`-only means the object under test is `SmPolyBrep`/`SmPolySolver`/`SmPolyMerge`/`SmPolyDecimate`/`SmRayTracer`.

| Item (function / test, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_polybrep_glue_preserves_face_normal_table :2216, 1 | none | n/a | | `SmPolyBrep::GlueVertices` normal-table bookkeeping; PolyBrep-only. |
| my_test_cone_face_normal_table(0,360) and (30,180) :2322, 2 | none (`CreateCone` r 1/2, h 10 or partial) | missing | | Real check: after meshing (`ConvertToPolyBrep` chord 0, 10 deg, 30 deg), every mesh corner maps back to its B-rep face with its normal. Kernel analog: mesh a full and a partial (30-180 deg) cone via `tess::mesh`; every `FaceMesh.normals[i]` unit, equal to the surface normal (apex excepted), `TriMesh.triangle_faces` valid, watertight. No normals check exists in smlib_tessellation. |
| my_test_poly_solver_1 :394, 5 tests (SM_SO_3D_SIGNED_DIRECTED_MINIMIZE, SIGNED_DIRECTED_MINIMIZE, DIRECTED_MAXIMIZE, DIRECTED_MINIMIZE, ...: rectangle/polybrep/polybrep solves) | Tess/Extrusion.pbp, Tess/BoxFillet.stl | gap | | `SmPolySolver::PolyBrepRectangleIntersect`/`PolyBrepPolyBrepSolve`; assert only `sSolutions.GetSize() > 0` (SM_ASSERT). Mesh-vs-mesh solvers; the kernel has B-rep distance (`min_distance`) only. |
| my_test_poly_solver_2 :942, 4 tests (PROJECTED_MIN/MAX, MAX, MIN polybrep-polybrep) | Extrusion.pbp, BoxFillet.stl | gap | | Same. |
| my_test_poly_solver_3 :1355, 5 tests (`PolyBrepPointSolve`, extrusion_old.smb converted to polybrep) | Solids/extrusion_old.smb (+ Extrusion.pbp in test 5) | gap | | Point-to-mesh solves; same. |
| my_test_poly_solver_4 :1756, 4 tests (PROJECTED_MIN/MAX, MAX, MIN `PolyBrepPointSolve`/`PolyBrepPlaneSolve`) | extrusion_old.smb, Extrusion.pbp | gap | | Same. |
| tess test 1 `CreateTriStripFaces` (10 points) :2394 | none | n/a | | PolyBrep constructor; asserts nothing. |
| tess test 2 `SimpleMerge` of BoxFillet.stl polyfaces into a new polybrep :2410 | Tess/BoxFillet.stl | n/a | | PolyBrep face move; nothing asserted. |
| tess test 3 `ReadFromSTLFile(BoxFillet.stl, ASCII, merge tol 1)` :2442 | Tess/BoxFillet.stl (120 facets, ASCII) | missing | | Kernel reads it: `read_stl_ascii` + `mesh_solid(tris, tol)` -> closed solid (SMLib asserts only success). Check: valid solid, closed oriented mesh, volume vs the STL's signed volume. |
| tess test 4 (stitch fillet_ts.smb at 0.001 then 0.005, tess chord 0.6, angle 40, edge 4, aspect 4, no failed faces) :2480 | Solids/fillet_ts.smb | covered | smlib_tessellation: `tess_stitched_fillet_ts` | Plus oracle mesh volume. |
| tess tests 5-8 `my_test_one_tess` nurb_fillet.smb with (chord, angle, maxEdge, aspect) = (0.2,30,0,0), (0,30,0,0), (0,30,3,0), (0,30,0,3), no failed faces :2593-2678 | Solids/nurb_fillet.smb | covered | smlib_tessellation: `tess_1_nurb_fillet` ... `tess_4_nurb_fillet` | `my_test_one_tess` also writes the mesh with `SmPolygonSLAOutput` (STL) and deletes it; no assert. Kernel has `exchange::write_stl_ascii`/`write_stl_binary` (not exercised by the SMLib-derived test). |
| tess test 9 poly brep STL read -> `WriteToFile` .pbp -> `ReadFromFile` -> write :2678 | Tess/BoxFillet.stl | n/a | | `.pbp` is SmPolyBrep persistence; no kernel counterpart (STL read part is row "test 3"). |
| tess test 10 ray firing at a polybrep grid (`SmRayTracer`, ray along (1,1,1) from bbox centre minus 15, length 40): `bHitsSomething == TRUE` :2715 | Tess/Extrusion.pbp | missing | | `Kernel::ray_hits` on the body (extrusion_old.smb is the model Extrusion.pbp was meshed from; or `mesh_solid` of the pbp triangles). Weak: assert a hit. |
| tess test 11 `SmPolyBrep::DoSectioning` with plane x=6: `sLineSegPnts.GetSize() > 0` :2776 | Tess/Extrusion.pbp | gap | | Section of a polygon mesh; kernel sections B-reps (`boolean::section`) only. |
| tess test 12 `TrimWithPlane` of a polybrep (extrusion_old.smb converted, or BoxFillet.stl) :2830 | Solids/extrusion_old.smb, BoxFillet.stl | gap | | Mesh clipping; no kernel operation. |
| tess test 13 `OrientedStitch` of SewSolidBefore.pbp, `lNumberLaminaRemaining == 0` :2877 | Tess/SewSolidBefore.pbp | missing | | Kernel counterpart: `mesh_solid` (welds, orients, requires a closed mesh -> error if lamina remain). Needs a small `.pbp` triangle reader in the test helper (text format: PolyFace/PolyLoop/vertex tables). |
| tess test 14 `OrientedStitch` of OrientedSewWSplit.pbp (needs vertex splitting), no lamina left :2907 | Tess/OrientedSewWSplit.pbp | missing | | `mesh_solid` step 3 (touching parts split per part) is the counterpart; same reader. |
| tess test 15 / my_test_decimation_regression :3850, 8 tests, stage 1 (read, `StitchFaces` at `SM_ZONE_TOL_3D`, tessellate to a polybrep with the test's chord/angles: nurb_fillet (0,10,25) and (0,10,40), sphere.smb (0,15,30) and (0,15,40), nurby_box_cyl_sphere.smb (0,15,40) x2, ellips_ts2.smb (0.01,5,25) and (0.01,5,30)) | Solids/nurb_fillet.smb, Solids/sphere.smb, Tess/nurby_box_cyl_sphere.smb, ellips_ts2.smb | missing | | Meshing these models is a kernel operation: `surfacing::sew_faces` then `tess::mesh` with the same chord/angle (chord 0 = angle only -> 0.1 mm like tessellation.py), then the existing `tess()` helper (watertight, deviation, volume vs oracle `tess in chord angle`). Golden prints the triangle counts (e.g. 4066, 3254, 240). Only 1 of the 4 files (nurb_fillet) is used by our tests, with other parameters. |
| tess test 15 stage 2: `SmPolyDecimate::DoMeasuredDecimation` (tests 1,3,5,7) / `DoQuadricDecimation` (2,4,6,8) to 80% reduction, 8 tests | same | gap | | Mesh decimation is absent from the kernel. Nothing is asserted (golden: 4066 -> 812 triangles etc.). |
| tess test 16 / my_poly_boolean_regression :3559, test 1 = `my_box_box_stress` test 1: 6 `SmPolyMerge` differences of unit cubes with offsets (dMove,0,0.5), dMove = 0.000999 / 10^k, k=0..5 :3211 | none | missing | | Kernel boolean difference of two unit boxes with near-coincident faces (offsets 9.99e-4 ... 9.99e-9, below/near kernel tolerance): result valid, volume = 1 - (1 - dMove) * 0.5. Closed form; hand-written in the booleans harness (`bop`). SMLib asserts only validity. |
| `my_box_box_stress` tests 2 and 3 (:3264, :3312) | none | n/a | | Unreachable: test 1's block ends with `return SM_SUCCESS`. |
| poly boolean test 2: linear sweep of an 8-vertex polygon (0,0)(3,0)(3,2)(2,2)(2,1)(1,1)(1,2)(0,2) by (0,0,-1), minus box 5 x 0.7 x 0.5 at (-1,1.2,-0.3) :3580 | none | missing | | Prism minus box: volume 5 - box overlap; closed form. Kernel `prism`/boolean difference. |
| poly boolean test 3: union of two unit cubes offset 0.5 in x :3644 | none | missing | | Volume 1.5, 1 shell, valid. Trivial, closed form. |
| poly boolean test 4: `box_box_grid.smp`, union of B-rep pairs (2i, 2i+1) converted to polybreps and `ManifoldBoolean` :3700 (25 pairs of 51 B-reps) | Solids/box_box_grid.smp | partial | smlib_booleans: `box_box_grid_*` (one per pair whose union is a manifold solid, plus `_all`) | The polygon route asserts nothing (a failure is ignored, `== SM_SUCCESS` only gates adding the result). Touching pairs are left out of our suite. 25 cases. |
| poly boolean test 5: `box_cyl_grid.smp`, DIFFERENCE (op 2) of the pairs :3735 | Solids/box_cyl_grid.smp | missing | | Our `box_cyl_grid_*` tests are unions. Difference per pair with the oracle's `boolean a b D` (exists) and `Kernel::boolean`; compare volume/face count. Generator practical: reuse booleans.py's grid extraction with `op = D`. 25 cases. |
| poly boolean test 6: `box_cone_grid.smp`, union of the pairs :3770 | Solids/box_cone_grid.smp | partial | smlib_booleans: `box_cone_grid_*` | As test 4. 25 cases. |
| poly boolean test 7: `solid1.pbp` minus `tool1.pbp` (`my_test_ppu_poly` op 2, polybreps read directly) :3805 | Tess/solid1.pbp, Tess/tool1.pbp | missing | | Planar-faced mesh solids: `.pbp` reader + `mesh_solid` + `Kernel::boolean` difference; result checks (valid, volume) would need the oracle or exact triangle volumes (no oracle subcommand reads .pbp). |

Totals: covered 5, partial 50, missing 48, gap 28, n/a 6

(missing 48 = cone normals 2, STL read 1, ray 1, OrientedStitch 2, decimation stage 1: 8, box_box_stress 6, sweep minus box 1, cube union 1, box_cyl_grid difference 25, solid1/tool1 1. gap 28 = solvers 18, DoSectioning 1, TrimWithPlane 1, decimation stage 2: 8.
The 8 decimation tests are counted twice, once per stage.)

Is it SmPolyBrep-only? The generator's NOT_CONVERTED lumps tests 1-4, 9-16 together as "SmPolyBrep, which the kernel has no counterpart of". That is right for the object under test of: solvers (18), SimpleMerge, TriStrip, glue, pbp IO, sectioning, TrimWithPlane, decimation stage 2 (about 31 cases). It is wrong for these, whose operation has a kernel equivalent: STL import (test 3), OrientedStitch (13, 14), ray firing (10), the near-coincident cube booleans (box_box_stress, tests 2, 3), box_cyl_grid difference (test 5), the decimation tests' tessellation stage, and the cone normal-table checks.

---

### Conversion candidates

Ranked by asserted outcomes x kernel relevance.

1. **box_cyl_grid difference (poly boolean test 5), 25 cases, plus near-coincident cube differences (6) and the two closed-form polygon booleans (2).**
   Build: add `op = D` to the existing grid extraction in tools/smlib/booleans.py (`grid()` in the Rust prelude takes pairs and unions); add 6 + 2 hand-written boolean cases with closed-form volumes.
   SMLib asserts: nothing but success (the polygon route; failures are even ignored in tests 4-6). Expected values: oracle `boolean a b D` exists (booleans suite already uses it); box_box_stress volumes are closed forms (0.5 + 0.5 dMove).
   Kernel API: `Kernel::boolean` (via the `bop` harness used in smlib_booleans.rs), `mass`. Generator: the grid part is a parse of the same call (practical); box_box_stress is regular (dMove / 10 loop, 6 iterations) so a generator can emit it; the sweep-minus-box and cube-union cases are irregular hand-written.
2. **ellips_ts2.smb topology regression (my_topo_regression_test, smAdvancedTest.cpp:2106), 6 convertible outcomes.**
   Build: one new module (or extend trimmed_surfaces.py, which has the harness) with: benchmark min distance 2870.476090 (printed in golden stdout, so a source-derived value), the point-classify/point-intersect count (389) on a 33 x 33 uv grid, line intersections (7) for 5 x 5 vertical lines, section (8 curves, compare length via oracle `section ellips_ts2.smb 1 1 1 10`), silhouette (2, via oracle `silhouette ... 1 0 -0.1`).
   SMLib asserts: the counts above are only `SM_ASSERT` (compiled out in release); the three point_extrema counts (362/52/60) cannot hold (dead after the benchmark early return) and should not be converted.
   Kernel API: `Kernel::min_distance(DistanceTarget::Point, DistanceTarget::Body)`, `Kernel::classify` on the sheet or `ray_hits`, `boolean::section`, `Kernel::silhouettes`. Counts that depend on SMLib's trimming (section, silhouette) should compare lengths, not counts (as section.py/silhouette.py do). Hand-written (the five helpers are irregular); the grid loops are simple.
3. **ellips_ts2 / sphere / nurby_box_cyl_sphere / nurb_fillet meshing with the decimation test's parameters (8 cases).**
   Build: extend tessellation.py with a `decimation` section: parse `my_test_decimation(file, chord, crvAngle, srfAngle, ...)` calls (regular), emit `tess(file, &[1e-5 sew], chord or 0.1, srfAngle, 0, 0, volume)`; the Rust `tess()` helper exists. SMLib asserts only that tessellation and decimation succeed; the oracle `tess in chord angle` gives mesh volume (nurb_fillet already used). Note the test passes crvAngle and srfAngle separately; kernel `TessParams::new(chord, angle)` has one angle (use the surface angle).
4. **Cone normal-table checks (2 cases).** Hand-written: mesh a cone (r 1 -> 2, h 10, full and 30-180 deg) and check normals/watertight/triangle_faces, as `tess()` does for deviations. Expected values: the surface normal itself (no oracle needed). Source cases are regular (two argument pairs).
5. **STL / mesh-solid round trips (tess tests 3, 13, 14; 3 cases) and ray hit (test 10).**
   Build: BoxFillet.stl: `exchange::read_stl_ascii` + `mesh_solid` -> valid closed solid; a small `.pbp` reader for SewSolidBefore.pbp / OrientedSewWSplit.pbp / Extrusion.pbp (text tables of faces, loops, vertices) feeding `mesh_solid` (must not report lamina); `ray_hits` with the test's ray on the extruded model must hit. SMLib asserts: success / zero lamina / `bHitsSomething`. Irregular, hand-written; the .pbp parser is the cost.
6. **Coaxial cylinder merge (SM:3577, 3668, 3899: 4 + 4 + up to 32 cases).** Build with `face_from_bare_surface` per face + `surfacing::sew_faces` + `HealOp::Unify`; checks: one face, 4 vertices and the bbox of the first test (all numeric values are in the source), 3 strips -> 1 face of area 180 deg x r50 x 3 mm, validity. Hand-written; the orientation family (32 cases) only partly expressible. Medium payoff: tests `unify`, which the kernel uses in healing and booleans.
7. **Loop nesting reports (face_loop_nesting, 7 cases) and heal single-vertex sphere (2 cases).** Hand-built `Body` faces with bad loops against `Body::check_with(Full)`; sphere r2 `face_from_surface` counts (1 face, 1 edge, 2 vertices, 4/3 pi r^3). All values in source. Check first that the kernel checker reports crossing/touching holes (it checks self-crossing and nesting).
8. **Trimmed-surface builders (3 cases): annulus (r5 outer, r3 hole) on a 20 x 20 plane, 0.5 x 0.5 uv sub-square face, DoCut of the tsurf face by two planes (`split_faces`).** Closed forms: area 16 pi; area 0.25; areas preserved. Reuse the trimmed_surfaces.py harness (`polyline`, `mkplane`, `add_hole`, `circle`). Small and cheap but low assert value (SMLib asserts validity only).
9. **MIN/MAX distances on ss_plane_sph2/3/4, tor1, tor2, tor_tor (12 asserts).** Needs a new oracle subcommand `distance file.smp min|max` (SmTopologySolver::BrepBrepSolve returning the best value) because the source only has solution counts, then `solve()` in trimmed_surfaces.py compares values. Without the oracle, no expected values exist. Payoff medium (distance on tori and spheres is a real kernel capability), generator practical (regular).

Gaps (feature absent, not worth converting now): polygon-mesh solvers (18 cases, tests assert only `> 0` solutions), mesh section and plane trim (2), mesh decimation (8), `GlobalLineIntersect` face intervals (10), loop crossings (4), `MakeFaceWithCurves` on curved surfaces from uv/3D loops (4 + 1 `MergeCurvesOnSurface`), curve-on-face classification (59), projected/directed/normalize solvers (46), curve projection onto a body (30).



### smCurveTest.cpp (usd-brep/tests/prog_test/src/smCurveTest.cpp, 9491 lines)

What runs: the `unit` suite of prog_test.cpp:56 calls `my_unit_test_suite` (smAdvancedTest.cpp:4914). Its first block ("SmCurve_test file tests", smAdvancedTest.cpp:~4930-5010) runs every `my_test_*` of this file except the dead ones listed below, in this order: analytic_curves, array, map, object, vector2d, vector3d, matrix, axis2placement, extent1d/2d/3d, pseudobox, solution, creation, bsplinecurve (-> curve_interface + curve_properties), bsplinecurve_nlib, cci_analy, cci_nurb, cci_tol, cci_tangent, cci_coincidence, cci_self, cci_projection, cc_nurb, cc_analy, methods (-> polynomial), cp_nurb, cp_analy, time, polynomial, curve_properties, crv_angle_min, crv_directed_max/min, crv_projected_max/min, crv_projected_tangency, crv_signed_angle_min, crv_signed_directed_min, signed_pivot_min. Never called (dead): `my_test_CC_extrema` (571, inside a `/* */` comment, "never used"), `my_test_CC_intersect` (695, no caller). (`my_test_curve_interface` is static but is called by `my_test_bsplinecurve` at 8107, so it runs.)

Notes on how it runs:
- No fixtures. Every curve is built in code (`my_create_line/circle/ellipse/nurb`, `SmBSplineCurve::Create*Segment`); no TestFiles paths are read. Output files (`curve1.smc`...) are written only if `sbDoWriteTests` (FALSE).
- Expectations are `SM_ASSERT(count == N)` (counts of solutions returned by `GlobalCurveIntersect` / `GlobalCurveSolve` / `GlobalPointSolve` / `GlobalPropertyAnalysis`) or `if (...) SER(SM_ERR)` value checks. Most SmLib statuses are discarded by `my_unit_test_suite` (only `my_test_solution` is wrapped in SER); a failing `SM_ASSERT` is the signal.
- `SM_DEBUG_CODE`/`SM_GFX_CODE` are defined by premake5.lua:64/83/99, so the debug-only blocks are compiled, but they only draw or dump.
- The expected counts are SMLib's own (seam duplicates, near-miss pairs, coincident ranges counted its way), so they are not directly comparable with a kernel that returns distinct points; see Notes.
- `tools/smlib/cci.py` converts the intersection functions (`cci_analy`, `cci_tangent`, `cci_coincidence`, `cci_nurb`). It does not convert anything else from this file. Its NOT_CONVERTED entry says `my_test_cci_tol` is "not run by SMLib's suites": that is wrong, it is called at smAdvancedTest.cpp:4973 in the unit suite (the entry should be corrected, and the 5 asserts converted).
- Helpers (not test items, excluded from totals): `my_draw_2crv_sol` 33, `my_draw_crv_pnt_sol` 103, `my_create_*` ~140-352, `my_test_GlobalCC_intersect` 353 (counts solutions of `GlobalCurveIntersect` at `SM_ZONE_TOL_3D`, a range counts as 1), `my_test_drop_point` 455, `my_test_point_extrema` 498, `my_test_GlobalCC_extrema` 623 (counts solutions of `GlobalCurveSolve`).

Kernel facts used below (crates/cadrs_kernel/src/native): exact `Curve` = Line, Circle, Ellipse, Nurbs only (curve.rs:16; no parabola/hyperbola variant, step.rs:634 turns them into quadratic arcs); `Curve::{point,d2,length,closest,to_nurbs,split via NurbsCurve}`; `NurbsCurve::{interpolate, interpolate_ext, chord_params, split, joined, insert_knot, elevated, reparam, reversed, restricted, ders}` (nurbs.rs); `ssi::curve_curve`, `ssi::curve_surface` (ssi.rs:1238, 1440); `distance::extremum` over `Piece::Curve` (global min/max only, distance.rs:345); `offset::wire::offset_wire` (offset/wire.rs:891); `math::quadratic` only (math.rs:276). No: local-extremum enumeration, curve property analysis, knot removal/point approximation/curve extension (only a private `bezier_curve_extended`, face_build.rs:243), directed/projected/signed/pivot curve-curve solves, parabola/hyperbola. Oracle: `smlib_oracle cci file.smp` only (reads part files); there is no oracle command for in-code curve pairs, point/curve extrema or properties.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| my_test_object 743 | none | n/a | | SmObject context/Dump container test |
| my_test_pseudobox 777 | none | n/a | | SmPseudoBox utility class |
| my_test_solution 899 (+ create_curve 853, create_solution_array 875) | none | n/a | | SmSolutionArray container, 53 SER calls, no geometry result |
| my_test_array 1261 | none | n/a | | SmTArray |
| my_test_map 1563 | none | n/a | | SmMap |
| my_test_vector2d 1758 | none | n/a | | SmVector2d class methods |
| my_test_vector3d 1907 | none | n/a | | SmVector3d class methods |
| my_test_axis2placement 2291 | none | n/a | | SmAxis2Placement (kernel has Frame/Affine; no SMLib-specific behaviour) |
| my_test_extent1d 2364 | none | n/a | | SmExtent1d interval class |
| my_test_extent2d 2688 | none | n/a | | SmExtent2d box class |
| my_test_extent3d 3115 | none | n/a | | SmExtent3d box class |
| my_test_matrix 3161 | none | n/a | | SmMatrix::SolveLinearSystem (singular must fail, then solves); kernel uses nalgebra / nurbs::solve |
| my_test_polynomial 1629: quadratic, 4 calls (6 asserts: 2 roots of x^2+3x+2, 6x^2+7x+2 two roots, double root -1, no real roots) | none | missing | | `math::quadratic` (math.rs:276) does it; only a 1-line unit test exists (math.rs:353) |
| my_test_polynomial: cubic, 4 calls (roots counts 3, triple root 1, double root 2, one real root 1) | none | gap | | no cubic solver in the kernel |
| my_test_polynomial: quartic, 4 calls (4, quadruple 1, two double 2, 2) | none | gap | | no quartic solver |
| my_test_cci_self 3226 (misnamed): `CreateOffsetsOfManyCurves` of 3 composites (outer box side 1, open composite of lines+nurbs side 2, circle hole side 1) at 20 distances 0.1..2.0, fillet corners, trim, then `BuildCompositesFromCurves`; only SER status | none | missing | | `offset::wire::offset_wire` (arc joins) per wire; check Ok and every result point at distance d from its source; SMLib offsets one side of an open composite, the kernel runs out and back, so only the closed wires and the hole compare 1:1 |
| my_test_cci_tangent 3379: 5 asserts (line/circle tangent 1; circle/line 1; circle in ellipse 2; circle round ellipse 5; two cubic nurbs tangent at ends 4) | none | partial | smlib_cci: `cci_tangent_line_circle`, `cci_tangent_circle_line`, `cci_tangent_circle_in_ellipse`, `cci_tangent_circle_out_ellipse`, `cci_tangent_nurbs` | only "some points, on both curves"; exact counts (SMLib's seam-duplicate counts) not compared |
| my_test_cci_analy 3484 case 1 (100000 GlobalPointSolve calls, no assert) | none | n/a | | benchmark (cci.py NOT_CONVERTED) |
| my_test_cci_analy cases 2-14 (13 sweeps of 20-40 gap steps each through the 1e-5 tolerance; asserts 0/1/2 per step at lines 3560-3910) | none | partial | smlib_cci: `cci_analy_02` .. `cci_analy_14` | per step none-vs-some only (steps within 5% of tol of a flip skipped); exact counts 1 vs 2 not compared |
| my_test_cci_coincidence 3945: 7 asserts (nurbs coincident 3; lines overlap 1; circles r .5/.5 1, same circle 1, turned 2, opposite 1, opposite turned 2) | none | partial | smlib_cci: `cci_coincident_nurbs`, `_lines`, `_circles_radii`, `_circles`, `_circles_turned`, `_circles_opposite`, `_circles_opposite_turned` | only "some points"; the range/point counts are not checked; known-failure note in tools/smlib_known_failures/cci.txt header (coincident curves return no points) is stale-looking: file has no live entries |
| my_test_cci_tol 4054: 5 asserts (circle r .500005 vs line 2; line vs circle r .499999 1; circle in ellipse 2; circle .999999 vs ellipse 8; two near-coincident cubics 3) | none | missing | | `ssi::curve_curve` at tol 1e-5; add as `expect()` cases in cci.py UNIT (see candidates) |
| my_test_cci_nurb 4157: 17-point zigzags 16 crossings, shifted 8, coincident 8 | none | covered (16, 8) / partial (coincident 8) | smlib_cci: `cci_nurb_zigzag_16`, `cci_nurb_zigzag_shifted_8`, `cci_nurb_zigzag_coincident` | distinct-point count compared for 16 and 8 only |
| my_test_cci_nurb: 21-point zigzags 10, shifted 10, coincident 2 | none | covered (10, 10) / partial (coincident 2) | smlib_cci: `cci_nurb_zigzag21_10`, `_shifted_10`, `_coincident` | same |
| my_test_cc_nurb 4328: two 21-pole cubic zigzags: MINIMIZE 2, MAXIMIZE 2 (shift A), MINIMIZE 3, MAXIMIZE 2 (shift B), NORMALIZE 81, line AT_DISTANCE 10 (6 asserts) | none | gap | | SMLib returns ALL local min / max / mutual-normal / at-distance solutions of curve pair (`GlobalCurveSolve`). Kernel only has the global extremum (`distance::extremum`) |
| my_test_cc_analy 4421 case 1 (bilinear Bezier surface Split, no assert) | none | n/a | | SmCubicBezierSurface API |
| my_test_cc_analy case 2 (mirror a nurb in a plane, no assert) | none | missing | | `Curve::transformed(Affine)` with a mirror; trivial (check reflected points) |
| my_test_cc_analy case 3: MINIMIZE between circle/ellipse 2, circle(plane yz)/ellipse 1, circle/nurb 1, circle/circle 1 (4 asserts) | none | gap | | local-minima counts (including closed-curve param 0/1 duplicates); global min only available |
| my_test_cc_analy case 4: `my_test_point_extrema(AT_DISTANCE)` on line, nurb, circle, ellipse (4 calls, no assert) | none | gap | | "points on curve at distance d from probe" has no kernel API |
| my_test_cc_analy case 4: `my_test_drop_point` x1000 params on nurb, circle, ellipse (3 curves; each point must have >= 1 solution) | none | missing | | `Curve::closest(p,t0,t1)` returns a param; check distance < 1e-5 and t near the source param |
| my_test_cc_analy case 5: AT_DISTANCE circle/line 2, circle/line 2, circle/nurb 0 (3 asserts) | none | gap | | boundary points of the "local neighbourhood within distance"; no kernel API |
| my_test_methods 4705: CreatePlaneProjection of a circle onto a plane, ApproximateCurve of the result (no assert, SER only) | none | gap | | parallel projection of a curve onto a plane + tolerance approximation of the result are not offered |
| my_test_methods: IsRational nurb false / circle true (2 checks) | none | missing | | `NurbsCurve::is_rational` / `Curve::to_nurbs` |
| my_test_methods: GetKnotsExpert / GetControlPointsExpert / SetExpert round trip (no check) | none | n/a | | SMLib storage accessors |
| my_test_methods: Length of unit circle = 2 pi (1e-9) and of line = 2 (2 checks) | none | missing | | `Curve::length` (curve.rs:128) |
| my_test_analytic_curves 4885: line (-1..2), circle (0..90 deg), ellipse (0..270 deg): 11 points each re-found by `GlobalPointSolveSTEP`, the found param must evaluate within tol of the point; then ReverseParameterization | none | missing | | `Curve::closest` + `point` for Line/Circle/Ellipse (3 cases) |
| my_test_analytic_curves: parabola, hyperbola (same checks) | none | gap | | no native parabola/hyperbola curve (only imported as quadratic arcs) |
| my_test_creation 4978: `CreateLineSegment` degree 3 and 2 (no check) | none | missing | | `Curve::to_nurbs` of a Line; trivial |
| my_test_creation: `CreateCircleSegment` full quintic and 90 deg quadratic arcs, canonical round trip (`GetCanonical`/`CreateCanonical`), no check | none | missing | | `Curve::to_nurbs(t0,t1)` of a Circle: check points on the circle, weights/degree; round trip itself is SMLib API (n/a) |
| my_test_creation: cubic with knot multiplicities {4,1,1,1,4} and weights (CreateCanonical/GetCanonical, no check) | none | missing | | `NurbsCurve::new` + `ders`; no expected value in source |
| my_test_creation: `if (FALSE)` ellipse segments 5035 | none | n/a | | dead code |
| my_test_cp_nurb 5131: `my_test_point_extrema` MINIMIZE x2 (5x19 grids of probe points), MAXIMIZE x2, NORMALIZE x2, AT_DISTANCE x2 on a 2D nurb and a transformed copy (8 calls, success only, no assert) | none | missing (MIN, MAX: 4 calls) / gap (NORMALIZE, AT_DISTANCE: 4 calls) | | MIN: `Curve::closest`; MAX: `distance::extremum` with `point_pieces` vs a `Piece::Curve`; both against brute force. NORMALIZE (all feet of perpendiculars) and AT_DISTANCE are local-solution lists the kernel lacks |
| my_test_cci_projection 5242: SM_SO_PROJECTED_INTERSECT circle vs line, line tangent parallel to the projection direction (no assert, comment doubts the answer) | none | gap | | projected curve-curve intersection absent (no assert in SMLib either) |
| my_test_cci_projection: `if (FALSE)` nurb vs nurb block 5245 | none | n/a | | dead code |
| my_test_cp_analy 5369 live: SM_SO_3D_SIGNED_DIRECTED_MINIMIZE circle/circle (rotated 20 deg): 2; shifted pair: 1; third pair: 1 (3 asserts) | none | gap | | signed directed (along a direction) minimum between two curves |
| my_test_cp_analy: `if (FALSE)` BuildCompositesFromCurves 5513 | none | n/a | | dead code |
| my_test_cp_analy: `#if 0` 5567-6547 (47 SM_ASSERTs, comment "unreachable code") | none | n/a | | dead code |
| my_test_time 6558: 2 `if (FALSE)` timing blocks | none | n/a | | benchmark, dead |
| my_test_curve_interface 6700 (called from my_test_bsplinecurve 8107), 43 `if (...) SER(SM_ERR)` checks, in groups: | none | | | Java-wrapper API test |
| - InterpolatePoints (4 pts, chord length, cubic) / CreateInterpolatingCurve x2 / 3 interpolations before CreateByJoining: non-null (6 checks, 6731-6903) | none | missing | | `NurbsCurve::interpolate` + `chord_params`; "non-null" is weak, add on-curve check |
| - CreateByJoining of 3 curves non-null (6915); JoinWith x2 (status not checked) | none | missing | | `NurbsCurve::joined` |
| - IsRational, degree 3, 10 control points, cp[12]==8, GetControlPoint x==8 w==1, 14 natural knots, 4 distinct knots with mult 3 (12 checks, 6934-6984) | none | n/a | | SMLib knot/control-point storage layout |
| - Evaluate first derivative y==13 (x2), EvaluatePoint(0.5).y==2 (3 checks, 6995-7008) | none | missing | | `NurbsCurve::ders` / `point` on the joined curve; domain [0,3] by join |
| - Trim [0,2] -> 11 knots, Trim [1,2] no-keep -> 8 knots (2 checks) | none | n/a | | knot counts of SMLib's representation |
| - EditParameterization x2, ReverseParameterization (value at param: 6, 6, 8; 3 checks) | none | missing | | `NurbsCurve::reparam`, `reversed` |
| - CreateExtendedCurve x3 (extension 1.5 by C-infinity continuation: values -260, -21, -3.4375, domain [0, 1.3746575...]; 8 checks, 7056-7121) | none | gap | | no public curve extension (only private `bezier_curve_extended`) |
| - ScaleKnotVector y(6)==2; SetControlPoint y(6)==4.5 (2 checks) | none | missing | | `NurbsCurve::reparam` + direct control-point edit |
| - AssertValid (1) | none | n/a | | SMLib internal |
| - Length == 11.888509121634552 (1e-6) (1) | none | missing | | `Curve::length` of the joined curve (expected in source) |
| - DropPoint found / param 6 / distance 0 (3 checks) | none | missing | | `Curve::closest` |
| - CutWithPlane returns 1 piece (1) | none | missing | | `ssi::curve_surface` with a plane; count crossing params |
| - 4x WriteToFile (guarded by `sbDoWriteTests` FALSE) | none | n/a | | not run |
| my_test_curve_properties 7991 (calls my_test_creation again): circle vs plane y = 0.99999 (near-tangent), `GlobalPropertyAnalysis(PLANE_INTERSECTION)`: 2 solutions | none | missing | | `ssi::curve_surface(Circle, Plane)` (handles tangency); expect 2 params |
| my_test_curve_properties: 2D 17-pole nurb, `my_test_one_curve_prop` total over 18 property analyses == 124 | none | gap | | plane classification x2, plane intersection, silhouette (parallel/perspective), tangent parallel to vector, inflection, min/max tangent length, min/max radius of curvature, given tangent length, given radius, perpendicular to X/Y/Z/any axis, tangent-to-circle (7219-7990). Kernel has no curve property solver |
| my_test_curve_properties: 3D 17-pole nurb total == 113 | none | gap | | same |
| my_test_bsplinecurve 8104 (only calls interface + properties + builds one nurb) | none | n/a | | wrapper, no checks of its own |
| my_test_crv_angle_min 8142 | none | n/a | | empty body |
| my_test_crv_directed_max 8152: SM_SO_DIRECTED_MAXIMIZE 1, 1 (2 asserts) | none | gap | | directed (along a direction) extremum between curves |
| my_test_crv_directed_min 8233: live SM_SO_DIRECTED_MINIMIZE 1 (8279) | none | gap | | |
| my_test_crv_directed_min: `#if 0` 8284-8368 (3 asserts: 2, 1, 1) | none | n/a | | dead code |
| my_test_crv_projected_max 8377 | none | n/a | | empty body |
| my_test_crv_projected_min 8389: SM_SO_PROJECTED_MINIMIZE 2, 1 (2 asserts) | none | gap | | |
| my_test_crv_projected_tangency 8533: SM_SO_PROJECTED_TANGENCY 4 (1 assert) | none | gap | | |
| my_test_crv_signed_angle_min 8582: SM_SO_SIGNED_ANGLE_MINIMIZE, 11 asserts (10 x 1, one x 2 at 8720) | none | gap | | |
| my_test_crv_signed_directed_min 8847: 2 asserts (1, 1) | none | gap | | |
| my_test_signed_pivot_min 8938: SM_SO_SIGNED_PIVOT_MINIMIZE, 6 asserts (all 1) | none | gap | | |
| my_test_bsplinecurve_nlib 9098: C1NonRationalCubicInterpolate (SER only) | none | missing | | `NurbsCurve::interpolate` degree 3, check passes through points |
| my_test_bsplinecurve_nlib: InterpolatePoints chord length (SER only) | none | missing | | `NurbsCurve::interpolate` |
| my_test_bsplinecurve_nlib: InsertOneKnot(0.3456789, x2) (SER only) | none | missing | | `NurbsCurve::insert_knot` x2, assert shape unchanged |
| my_test_bsplinecurve_nlib: RefineCurve with 6 knots (SER only) | none | missing | | repeated `insert_knot`, assert shape unchanged |
| my_test_bsplinecurve_nlib: ApproximatePoints with tol (SER only) | none | gap | | no tolerance point fit |
| my_test_bsplinecurve_nlib: DeformToEndPointTargets, 5 modes x 3 curve types (nurb, line, circle) = 15 calls (SER only) | none | gap | | end-point + tangent deformation absent |
| my_test_bsplinecurve_nlib: PiecewiseArcInterpolate (SER only) | none | gap | | |
| my_test_bsplinecurve_nlib: RemoveKnots (tol, constrain end derivative) and RemoveOneKnot (SER only) | none | gap | | no knot removal |

Totals: covered 4, partial 27, missing 52, gap 90, n/a 43

(How cases were counted: each `SM_ASSERT` / `SER(SM_ERR)` check group above is one case, a benchmark/dead/empty/container function is one case, the dead `#if 0` blocks count as one case each (cp_analy 5567 holds 47 asserts), `my_test_one_curve_prop` is two cases (the two totals 124 / 113). The 12 cci_analy sweeps and 7+5 CCI unit cases are counted from cci.py's names. Partial = 5 tangent + 13 analy + 7 coincidence + 2 coincident-zigzag.)

### Conversion candidates (ranked by payoff = asserted outcomes x kernel relevance)

1. my_test_cci_tol (5 asserts, trivially cheap, kernel relevance high: tolerance-edge curve-curve intersection).
   - Build: add 5 `UNIT` entries to tools/smlib/cci.py (`expect()` form, or the sweep helper): line (1,-2)-(3,-2) vs circle r 0.500005 at (2.5,-1.5): some; line vs circle r 0.499999 at (1.5,-1.5): some (SMLib 1); circle r 0.499999 vs ellipse(1,0.5) concentric: some (SMLib 2); circle r 0.999999 vs same ellipse: some (SMLib 8, 4 of them at seam, compare distinct count only if you work out the distinct points); two cubic nurbs with 7 poles (second lifted by 5e-6 in z): some (3).
   - SMLib asserts (4054-4156): counts 2, 1, 2, 8, 3 in the source. No oracle needed; `ssi::curve_curve` at tol 1e-5, already used by `cci()` in the generated test. Generator: yes, add to the existing table (curve builders `circle`, `ellipse`, `line`, `nurb` exist in the PRELUDE; the nurb cubic pair needs the two point lists and a z offset, and the quintic/quadratic circle form is irrelevant to exact circles).
   - Also fix the NOT_CONVERTED entry in cci.py (cci_tol is run).
2. Point inversion / curve extrema against brute force (about 30 outcomes: analytic_curves 3, drop_point 3x1000 params, cp_nurb MIN x2 and MAX x2, cc_analy case 3 global minima x4, cc_nurb global MIN/MAX x4).
   - Build: hand-written test `curve_closest`: for line / circle arc / ellipse arc / nurbs, 11 params (and 1000 for the nurb, circle, ellipse of drop_point), `Curve::closest` must return a param whose point is within 1e-5 of the probe; for cp_nurb's 5x19 grids compare `Curve::closest` and `distance::extremum` (Min/Max with `point_pieces` and `Piece::Curve`) against a dense-sample brute force; for cc_nurb / cc_analy case 3 compare the global `distance::extremum` of the two curves against sampled brute force.
   - SMLib asserts: only counts of local solutions (cc_nurb 2/2/3/2, case 3: 2/1/1/1) and "at least one" for drop_point; the global value has no expected number in the source, so use brute force, or add an oracle subcommand (`ccx file|spec min|max` driving `GlobalCurveSolve`, see `my_test_GlobalCC_extrema` 623, and `GlobalPointSolve` for `closest`) to compare SMLib's values with ours. The oracle currently only has `cci`. Curves are in-code, so the oracle would need the construction recipes (`my_create_nurb` etc.) in oracle.cpp, as `box:`/`torus:` primitives are today.
   - Generator: not practical (irregular code); hand-written cases (a table of curve recipes shared with the CCI prelude).
3. Curve interface numerics (my_test_curve_interface, 20 convertible checks with literal expected values: derivative 13, y(0.5)=2, reparam values 6/6/8, ScaleKnotVector 2, SetControlPoint 4.5, Length 11.888509121634552, DropPoint param 6 distance 0, CutWithPlane 1, interpolation non-null).
   - Build: reproduce the 4-point chord-length cubic interpolants, `NurbsCurve::joined`, `reparam`, `reversed`, `length`, `closest`, `ssi::curve_surface` with a plane; use SMLib's literals (the join places the three segments on [0,3]).
   - Caveat: the parameter conventions (SMLib's chord-length normalisation per segment, join keeping each segment's [0,1]) must be matched; verify with an oracle spot check if the first run disagrees. Hand-written (about 15 statements); no generator.
4. Tighten the 27 partial CCI cases (tangent 5, coincidence 7, coincident zigzags 2, analy sweeps 13).
   - Build: compare the number of distinct points / merged ranges with SMLib's literal counts where the count is geometric (tangent line/circle 1, circle-in-ellipse 2, coincident lines 1 range, zigzags). SMLib counts seam duplicates and near-miss pairs its own way (cci.py docstring), so only the geometrically unambiguous ones are worth it. Needs a range output from `ssi::curve_curve` (coincident ranges currently return no points, per the stale-looking known-failures header). Same generator tables.
5. Plane-tangent circle (my_test_curve_properties first block): 1 assert, `ssi::curve_surface(Circle, Plane)`, expect 2 params (circle r 1, plane y = 0.99999); hand-written, 10 lines, no oracle.
6. my_test_cci_self composite offsets: `offset_wire` on the box outline (left), the open lines+nurbs composite (right) and the circular hole at the 20 distances; assert Ok and distance-d property. 20 iterations; hand-written; open-wire semantics differ (round caps), so restrict to the closed wires.
7. Shape-preserving knot tests for the nlib smoke calls (InsertOneKnot x2, RefineCurve with 6 knots, C1 cubic and chord-length interpolation passing through the points): 4 cases, `insert_knot` / `interpolate`; no SMLib expectation (SER only), so these are kernel property tests in the SMLib shape.
8. Polynomial solver (quadratic 4 calls cover `math::quadratic`; cubic/quartic need a new internal solver first): 12 calls with literal root counts, hand-written; low kernel relevance.
9. Feature gaps with large asserted payoff but needing new kernel operations (not convertible today): curve-pair solves with directed / projected / signed-angle / signed-directed / pivot objective (28 asserts live, crv_* and cp_analy), curve property analysis (curve_properties totals 124 and 113 over 18 property kinds), local extremum enumeration (cc_nurb 6, cc_analy 7, cp_nurb 8 calls), AT_DISTANCE / NORMALIZE point and curve solves, curve extension, knot removal, tolerance point approximation, end-point deformation (15 calls), parabola/hyperbola as native curves (2 cases). Each would need an oracle subcommand for expected values unless the count is in the source (they all are: SM_ASSERT counts), but the counts are SMLib-specific (closed-curve duplicates, near-miss pairs), so they would be weak cross-checks even if the operation existed.



Paths: SRC = /home/casper/work/nvbrep/usd-brep/tests/prog_test/src; fixtures are under /home/casper/work/nvbrep/usd-brep/TestFiles/pt_TestFiles (the C++ says `../../TestFiles/pt_TestFiles`).
TREE = the cadrs checkout.

### smSurfaceTest.cpp

Only `suite "unit"` (prog_test.cpp:56) reaches this file: `my_unit_test_suite` (smAdvancedTest.cpp:4914) runs, in the "smsurf_test file tests" block (smAdvancedTest.cpp:5030-5048), in this order: `my_test_analytic_surfaces`, `my_test_bsplinesurface`, `my_test_csi_analy`, `my_test_csi_nurb`, `my_test_csi_tol`, `my_test_csi_tangency`, `my_test_csi_coincidence`, `my_test_cs_nurb`, `my_test_drop_curve`, `my_test_ss_nurb`, `my_test_surf_methods`, `my_test_sp_nurb`, `my_test_sp_analy`, `my_test_surface_suite` (= `my_test_analytic_surfaces` again), `my_test_section`, `my_test_projection`; and earlier in the curve block `my_test_bsplinesurface_nlib` (smAdvancedTest.cpp:4950). `my_test_bsplinesurface` (5097) calls `my_test_surface_interface` and then only prints an area and runs a 33x33 point round trip; the many `if (FALSE)` calls to other tests inside it are dead. `my_test_drop_curve` calls `my_test_drop_curve_near_seam` and `my_test_drop_iso_curves`.
Only `my_test_csi_analy` is converted (tools/smlib/cci.py, suite smlib_cci.rs, 11 `csi_*` tests), and only as "none against some + every point on the surface" (cci.py docstring/NOT_CONVERTED: SMLib's counts include seam duplicates). cci.py also lists `my_test_csi_nurb` as not ported. Nothing else of this file is in any generator (grep of tools/ for the function names and for the fixtures shovel.srf, Surface.sms, Drop/*.dat finds nothing; Surface.sms exists only as a reader fixture, tests/data/smlib/reader/Surface.sms, used by src/native/smb.rs tests). The oracle (tools/smlib_oracle) has no surface-level subcommand (commands: boolean offset shell replace chamfer sweep stitch ssi section silhouette tess cci), but nearly every assert here has its expected value in the C++ source, so none is needed.
Kernel facts used below: `ssi::curve_surface(curve,t0,t1,surf) -> Vec<f64>` (isolated parameters only, no overlap ranges; ssi.rs:1238); `ssi::surface_surface` (+ `_full`, `_tol`); `Surface::{eval,project,distance,iso_u,iso_v,to_nurbs}` and `ssi::sdist`; `pcurve::pcurve` (p-curve of a curve lying on a surface); `Surface::Extrusion/Revolution` variants; `NurbsSurface::through_curves`; `NurbsCurve::{interpolate,insert_knot,elevated}`; `offset::sharp::offset_surface`; `smb::{read_surface,read_curve}` (bare legacy layout: the Drop/*.dat, shovel.srf and Surface.sms files have the same layout, so the reader should take them; not verified by running); `delete_faces`, `heal`. The kernel has NO: surface-level knot insertion/refinement/removal/degree change, point-grid surface approximation/interpolation, Gordon/swung/sweep NURBS surface builders, extended surface, curve-surface or surface-surface local-extremum solvers (min/max/normalize/directed), DropCurve/LiftCurve/parallel-projection curve builders, coincident curve-on-surface ranges.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `my_test_surface_suite` 949 (1 case) | none | n/a | none | only calls `my_test_analytic_surfaces`, which therefore runs twice |
| `my_test_analytic_surfaces` 5419 / `my_test_analytic_surface` 5298 (12 cases: plane, cylinder, cone, 0-radius semi-infinite cone, inside-out 0-radius cone, torus, sphere, surf-of-revolution of a parabola, surf-of-extrusion of a B-spline, 2 identical surf-of-revolution of a B-spline, surf-of-extrusion of a B-spline) | none | missing | none | per case: 11x11 uv grid -> point -> `GlobalPointSolveSTEP` INTERSECT -> `EvaluateSTEPPoint` within 1.5e-8, then `LiftCurve` of the uv diagonal returns non-null. Kernel: `Surface::project`/`ssi::sdist` round trip (`dist(eval(uv), project(eval(uv))) < 1e-8`) on `Surface::{Plane,Cylinder,Cone,Torus,Sphere,Revolution,Extrusion}`; the STEP/NURBS parametrisation conversion has no kernel equivalent. Cases 10 and 11 are the same code |
| `my_test_csi_tangency` 1100 (2 modes x 22 positions = 44 cases) | none | missing | none | quintic cylinder (`cone_patch` exists in cci.py PRELUDE) against circles (mode 0) and lines (mode 1) that graze it within +-1e-3: SM_ASSERT count 2 where the curve is separated by more than tol from tangent, else 1. Kernel `ssi::curve_surface` returns parameters of sign changes and touches, so assert the SMLib count only loosely (>=1 point when within tol, 0 beyond, every point within 1.0001e-3 of both) |
| `my_test_csi_tol` 1171 (20 iterations x 2 surfaces = 40 cases) | none | missing | none | quadratic cylinder at x=0 and at x=2.5 against a line moved by 1.0001e-5/10 per step: count must be 2 for y<=eff-zero else 1 (seam), and 1 at the knot. Same count-semantics caveat as above |
| `my_test_csi_coincidence` 1283 (5 groups x 2 surfaces = 10 cases) | none | gap | none | lines/circles lying on a quadratic and a quintic cylinder (full, short, along the seam: 1, 1, 2 coincident segments, circles at the end and middle: 1). The kernel's `curve_surface` has no overlap ranges, so the asserted segment counts cannot be checked |
| `my_test_cs_nurb` 1403: 6 `SM_SO_3D_SIGNED_DIRECTED_MINIMIZE` cases (count == 1) + 1 `SM_SO_NORMALIZE` case (count 4) (7 cases) | none (9x7 cubic surface from `my_load_surf3_pts`, 3 directions, line and circle) | gap | none | no curve-surface local-extremum / directed-distance solver in the kernel |
| `my_test_cs_nurb` 1403: 11 unasserted solver calls (circle1 MAXIMIZE, circles 2-6 MIN/MAX) + the mirror surface | none | n/a | none | no assert; results discarded |
| `my_test_drop_iso_curves` 1584, called at 2791, 2800, 2809, 2818, 2827 (cones/cylinders) and 2870 (nurb surface) (6 cases) | none | missing | none | per surface: iso curves at u,v = 0, 0.5, 1 dropped back: exactly 1 piece for the 0.5 curves, >=1 for the edges, skipped when degenerate. Kernel: `Surface::iso_u/iso_v` + `pcurve::pcurve` and check 1 piece / lift within tol (the kernel's `pcurve` is one curve, so "exactly 1" is true by construction: assert the lift instead) |
| `my_test_drop_curve_near_seam` 1950 (2 swap x 5 kinds x 2 reverse = 20 cases) | none | missing | none | `SmCylinder` r=0.25 length 20, curves: 3 near-axial lines (end offset -2e-5, 0, +2e-5), a 270-degree circle, a helix; DropCurve at tol 1e-5 must give >=1 curve (1 for the helix), no assert/warning diagnostics, and every knot-span sample lifts back within 1.01 tol. Kernel: `pcurve::pcurve(&Surface::Cylinder, &curve, t0, t1, None)` then eval(p-curve(t)) vs curve(t); the seam-jump check is the point |
| `my_test_drop_curve` 2053: dead `if (FALSE)` blocks 2062 (curveDropped.txt) and 2317 (/Customers/Gestel/...) (2 cases) | absent files | n/a | none | dead code |
| `my_test_drop_curve` 2116, 2154, 2192, 2237, 2276 (5 cases: expected piece count 1, 1, 1, 0, 1) | Drop/PS10_drp_Crv1..5.dat, PS10_drp_Srf1..5.dat | gap | none | `DropAndTrimCurve` (drop an off-surface curve and trim at the domain boundary, count the pieces): kernel `pcurve` only handles curves already on the surface and does no trimming/splitting. Case 4 expects 0 pieces |
| `my_test_drop_curve` 2364, 2412, 2460, 2508, 2556, 2606 (6 LiftCurve cases) | Drop/PS_LiftCrv_Crv1..4.dat + Srf1..4.dat, PS_lift_crv1,2.dat + PS_lift_srf1,2.dat | gap | none | `LiftCurve` of a uv curve to 3D within 1e-2; assert only non-null. No lift builder in the kernel (trivial to emulate by sampling `Surface::eval`, low value) |
| `my_test_drop_curve` 2654, 2716 (2 cases) | none (surf1 6x4 cubic, lines) | gap | none | long lines dropped on a small doubly curved surface needing trimming; no count assert, only non-null pieces |
| `my_test_drop_curve` 2831-3043 (7 cases: circle 2869, line 2907, circle 2946, line 2984, circle 3002, line 3012, line 3043) and 3054 (irregular closed curve), 3116, 3179, 3236, 3292, 3354 (5 lift-and-drop cases) (13 cases, each count == 1) | none | gap | none | `DropCurve` on the 6x4 cubic surface, count of pieces == 1; same gap as above |
| `my_test_ss_nurb` 3421: NORMALIZE count 1; MINIMIZE count > 50 (2 cases) | none | gap | none | surface-surface local-extremum solver absent. The count > 50 is a whole set of locally minimal pairs |
| `my_test_ss_nurb` 3421: 2 unasserted calls (MINIMIZE, MAXIMIZE) | none | n/a | none | no assert |
| `my_test_csi_analy` 3514 (4 planes + 7 cylinders/cones = 11 cases; counts 25, 25, 25, 5, 45, 25, 60, 45, 25, 13, 13) | none | partial | smlib_cci.rs `csi_plane`, `csi_plane_tilted`, `csi_plane_skew`, `csi_plane_vertical`, `csi_cylinder`, `csi_half_cylinder`, `csi_cone_up`, `csi_cone_down_270`, `csi_cylinder_quintic`, `csi_cone_side`, `csi_cone_side_down` | converted as "points lie on the surface, none against some" only; exact counts not compared (SMLib counts seam duplicates and uses the B-spline's control-hull grid; cci.py NOT_CONVERTED). `csi_plane_vertical` (plane x=14) is weaker still (lines lie in the plane, SMLib: 5) |
| `my_test_csi_analy` 3514: dead `if (FALSE)` blocks (Lift_surface000.sm parallel projection, a 35-pole surface silhouette) (2 cases) | absent files | n/a | none | dead code |
| `my_test_csi_nurb` 3874: 3 surface cases (count 92, then 85 after a transform, then 92 after another) (3 cases) | none | missing | none | 9x7 cubic surface (`my_create_crease_surf`, sbCreases FALSE, so a plain uniform cubic from `my_load_surf3_pts`), 10x10 grid. cci.py says not ported. Kernel: `ssi::curve_surface` per line as in cci.py `csi()`; counts would be loose |
| `my_test_csi_nurb` 3874: 9 curve cases (`my_create_3dcrv` translated to x = -3, -3.5, -4, -4.5, -5, -5.5, -6, -2, -2.5: counts 7, 7, 6, 6, 6, 8, 8, 4, 6) (9 cases) | none | missing | none | same surface translated to x = -6; curve is a 13-point cubic. The counts are exact in the source ("upon inspection 6 is correct"); `ssi::curve_surface` + distinct points could assert them if the kernel agrees; else loosen |
| `my_test_csi_nurb` 3874: dead `if (FALSE)` block (49 on a sub-domain) (1 case) | none | n/a | none | dead code |
| `my_test_surf_methods` 3984 (2 cases: `CreateConeThroughBox`; quintic cone GetKnotsExpert/SetExpert rebuild) | none | n/a | none | SMLib API plumbing + Dump, no value assert (only IsRational) |
| `my_test_sp_nurb` 4042: 4 point-inversion round trips (33x33 uv grid -> point -> `GlobalPointSolve` INTERSECT) | none (surf3 cubic; moved; domain-reduced) | missing | none | the solution-count check is in an error branch that only fires with graphics on, so SMLib asserts nothing in a normal run; a stronger kernel test (`project(eval(uv))` distance < tol) is easy: `Surface::project` / `ssi::sdist` |
| `my_test_sp_nurb`: 2 MINIMIZE cases (10x10x2 sample points, count == 200 and a per-point "at least one solution" assert) | none | missing | none | `Surface::project` / `sdist` finds a nearest point for every sample; compare against brute-force sampling |
| `my_test_sp_nurb`: 2 NORMALIZE cases (count 156 and 86; with creases 243/171 not run) | none | gap | none | all common-normal feet of a point on a surface (not just the nearest) are not enumerated by the kernel |
| `my_test_sp_nurb` 4117: 15 closest-point cases (`my_test_surface_closest_point`: nearest point within 0.03 of each of pt1..pt15, tol 1e-6) | Surface.sms (also in crates/cadrs_kernel/tests/data/smlib/reader/Surface.sms) | missing | none | thinly closed surface; `smb::read_surface` + `ssi::sdist(&surf, &p) <= 0.03` (or `Surface::project` then distance). All 15 points are literals in the source |
| `my_test_sp_analy` 4165: surface-of-revolution STEP block (101x101 = 10201 grid points; STEP<->NURBS uv round trip, `EvaluateSTEPPoint` == `EvaluatePoint`, `GlobalPointSolve` INTERSECT finds >=1 solution evaluating back to the point) (1 case) | none (B-spline revolved about (1,1,0) through (0,2,0)) | missing | none | the function `return`s at the end of this block (4250). Kernel: `Surface::Revolution` project round trip; the STEP conversions have no equivalent. Also prints a timing |
| `my_test_sp_analy` 4165: five cylinder/cone cases after the `return` (point_intersect + MINIMIZE count 216 each) (5 cases) | none | n/a | none | dead code after `return SM_SUCCESS` at 4250 |
| `my_test_projection` 4332: circles (20 radii) projected along -z onto shovel.srf, no assert; surf 5x4 (10 circles, count == 15); surf2 (20 circles, count == 29) (3 cases) | shovel.srf for the first | missing | none | `CreateParallelProjectionCurves`: kernel can build it as `ssi::surface_surface(surf, Surface::Extrusion{c: circle, d: -z})`; expected curve counts 15/29 are exact in the source but count curve pieces the SMLib way (compare loosely: >0 curves each, on both surfaces) |
| `my_test_section` 4429: shovel.srf at 40 planes (no assert); 4 asserted: surf 5x4 normals (0,0,1) 10 planes -> 10, (1,1,0) 10 planes -> 10; surf2 20 planes (0,0,1) -> 21, 10 planes (1,1,1) -> 7 (5 cases) | shovel.srf for the first | missing | none (the section suite is solids, `smlib_section.rs` torus.smb) | `CreatePlanarSectionCurves` of a bare surface: `ssi::surface_surface(surf, Plane)` through `box.Evaluate(t,t,t)`. Counts are exact; the source comments that degenerate point curves are counted (10 not 8), so compare the curve count loosely or by total length |
| `my_test_surface_interface` 4620 (via `my_test_bsplinesurface`): ApproximatePoints (surf1 6x4, tol 1e-3) + GetDegree == 3, UMin == 0, control net 6 x ... = 24 poles with no weights, 10 knots with knot0 == 0, not rational, not periodic (1 case) | none | gap | none | no tolerance-driven point-grid approximation |
| `my_test_surface_interface`: EvaluatePoint at (0.5,0.5) x == 1.4454876380771180; EvaluatePointOnExtendedSurface at (1.2,1.2) x == 3.3196332743395720; EvaluateNormal x == -0.081949394674615525 (all +-1e-8) (3 cases) | none | n/a | none | the numbers are those of SMLib's own approximated surface and extension |
| `my_test_surface_interface`: DropPoint (NORMALIZE fails, MINIMIZE succeeds with uv.y == 1) (1 case) | none | missing | none | `Surface::project`; needs a stand-in surface (e.g. `through_curves` of the same rows) |
| `my_test_surface_interface`: GlobalLineIntersect (count 1), LocalLineIntersect (found, dist <= 1e-8) (2 cases) | none | missing | none | line vs surface: `ssi::curve_surface` with `Curve::line` |
| `my_test_surface_interface`: GlobalCurveIntersect (cubic interpolating (1,1,1),(5,1,1),(5,5,1),(9,5,1); count 1) (1 case) | none | missing | none | `NurbsCurve::interpolate` + `ssi::curve_surface` |
| `my_test_surface_interface`: GlobalSurfaceIntersect with the surf4 approximation (2 curves, deviation <= tol) (1 case) | none | missing | none | `ssi::surface_surface`; counts depend on the stand-in surfaces |
| `my_test_surface_interface`: GlobalPointSolve MINIMIZE (count 1) (1 case) | none | missing | none | `Surface::project` |
| `my_test_surface_interface`: GlobalCurveSolve MINIMIZE (count 1), GlobalSurfaceSolve MINIMIZE (count 1) (2 cases) | none | gap | none | local-extremum solvers absent |
| `my_test_surface_interface`: Reparameterize to (1,2,3,4) (knot0 == 1), SwapUV (knot0 == 2) (2 cases) | none | gap | none | no surface reparameterize / swap-uv in the kernel (curves have `reparam`) |
| `my_test_surface_interface`: CreateIsoParametricCurve x5 (non-null) (1 case) | none | missing | none | `Surface::iso_u/iso_v` |
| `my_test_surface_interface`: CreateSkinnedSurface x2 (1 case) | none | missing | none | `NurbsSurface::through_curves` |
| `my_test_surface_interface`: CreateExtendedSurface (0.5, C-infinity) (1 case) | none | gap | none | no surface extension (its assert tests a stale status variable, so it asserts nothing real) |
| `my_test_surface_interface`: ApproximateOffsetSurface x2 (1 case) | none | missing | none | `offset::sharp::offset_surface`; smoke only |
| `my_test_bsplinesurface` 5097: Area print of a 1-radius cylinder (no assert) (1 case) | none | n/a | none | print only |
| `my_test_bsplinesurface` 5097: 33x33 point round trip on a 5x4 cubic (1 case) | none | missing | none | as the sp_nurb round trips |
| `my_test_bsplinesurface` 5097: ~13 dead `if (FALSE)` blocks (calls to other tests, tessellation, derivatives, bbox) (1 case) | none | n/a | none | dead code; the tessellation helper `my_test_surface_tessellation` (915) is an empty stub |
| `my_test_bsplinesurface_nlib` 5715: ApproximateOffsetSurface 5744 (1 case) | none | missing | none | `offset::sharp::offset_surface`; asserts success only |
| `my_test_bsplinesurface_nlib`: AdjustBoundaryCurve 5722, ApproximatePoints 5763, ApproximateRandomPoints 5782, ApproximateRationalSurface 5808, ApproximateSubSurface 5840, CalculateCrossBoundaryDerivative 5910, CreateGordonSurface 5920, CreateSweepSurface 5960, CreateSwungSurface 5995, DegreeElevate 6016, DegreeReduction 6029, InsertOneKnot 6055, InterpolatePoints 6065, RefineSurface 6087, RemoveKnots 6109, RemoveOneKnot 6141 (16 cases) | none | gap | none | each asserts only SM_SUCCESS (no value). The kernel has no such surface-level operations (curves have `insert_knot`/`elevated`/`interpolate`; surfaces only `through_curves`) |
| `my_test_bsplinesurface_nlib`: GetMeasures 6045 (1 case) | none | n/a | none | SMLib-internal measure query, no assert |
| helpers `my_create_*`, `my_load_surf1..4_pts`, `my_test_GlobalCSI`, `my_test_cs_solve`, `my_test_ss_solve`, `my_test_surface_*`, `my_test_planar_section`, `my_test_parallel_projection`, `my_CenterSurface`, `my_surface_from_file` (1 case) | none | n/a | none | helpers, not tests |

Totals: covered 0, partial 11, missing 175, gap 69, n/a 33

### smFeatureTest.cpp

None of this file is run. `my_test_feature` (smFeatureTest.cpp:1053, declared in inc/smFeatureTest.h:18) is the file's only entry point and nothing calls it: it is absent from the suite table in prog_test.cpp:47-64 (suites topology booleans ... silhouette-advanced), from `my_unit_test_suite` (smAdvancedTest.cpp:4914), from prog_test_app.cpp, and from every build/script file (grep over usd-brep for `my_test_feature` and `smFeatureTest` finds only the header, `prog_test.h:55` `#include <smFeatureTest.h>` and the file itself). It exercises `SmFeatureExecutive` (source/SMLib/src/SmFeatureExecutive.cpp: AddFeature, DoDefeature, RestoreFeatures with rebuild types) and checks the face/edge/vertex counts of the rebuilt base and feature breps with `ValidateCounts`. The fixtures all exist (TestFiles/pt_TestFiles/Defeature/*.smb) and are not used by any of our suites (reader tests load some: LShape_I/II, Platte). The kernel has no defeature executive (nearest: `delete_faces`, surfacing.rs:588, "delete face (Heal)", and `heal`), so it would be a feature gap if SMLib ever ran these. Statuses are n/a (dead code); the expected counts are in the source if wanted.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `my_test_feature` 1053 (entry point) | none | n/a | none | never called |
| `my_test_rebuild_fillets` 22 (case 0: base 10/18/12, feature 3/5/3 faces/edges/vertices) (1 case) | Defeature/defillet.smb | n/a | none | dead; kernel gap: defeature + rebuild (only `delete_faces` is near) |
| `my_test_add_by_faces` 147 (case 0: base 13/27/20, feature 3/3/2) (1 case) | Defeature/extrusion.smb | n/a | none | dead; features added by face lists |
| `my_test_multi_feature` 285 (case 0: base 13/27/20, feature 3/3/2) (1 case) | Defeature/extrusion.smb | n/a | none | dead; three features with edge loops |
| `my_test_multi_feature_adv` 435 (case 0: base 13/27/20, feature 3/3/2) (1 case) | Defeature/extrusion.smb | n/a | none | dead; features with several loops |
| `my_test_multi_rebuilds` 601 (cases 0, 1, 2: base 13/27/20 extend-surface rebuild; 9/22/18 with no rebuild, 4 lamina; 10/22/18 after `RestoreFeatures`) (3 cases) | Defeature/extrusion.smb | n/a | none | dead |
| `my_test_extended_surface` 790 (cases 0-9: base/feature counts 9/21/14 + 6/12/8; 11/26/17 + 7/15/10; 13/30/19 + 8/17/11; 15/36/23 + 9/21/14 twice; 8/17/11 + 7/15/10; 12/32/22 + 10/24/16; case 8 and case 9 (25/62/40) in the source) (10 cases) | Defeature/inset_cube.smb, inset_cube_1x1fillet.smb, inset_cube_1x2fillet.smb, inset_cube_1x3fillet.smb, inset_cube_3x1fillet.smb, flathead_screw_top.smb, brep1.smb, LShape_I.smb, LShape_II.smb, Platte.smb | n/a | none | dead; defeature by extended surfaces |

Totals: covered 0, partial 0, missing 0, gap 0, n/a 16

### Conversion candidates

Ranked by asserted outcomes x kernel relevance. For all of them the expected values are literals in the C++ source, so no oracle subcommand is needed (none exists for surface-level calls; the nearest, `ssi` and `cci`, take .smp parts). A generator that parses the C++ blocks is practical only where noted; the rest are irregular hand-written code that should be ported by hand into one new module, e.g. tools/smlib/surface.py with the same Case/PRELUDE scheme as cci.py (reuse its `cone_patch`, `bilinear`, `Patch`, `csi`).

1. `my_test_csi_tangency` + `my_test_csi_tol` + `my_test_csi_coincidence` (44 + 40 + 10 outcomes, 94). Build: quintic/quadratic cylinders (cci.py `cone_patch`) against the line and circle sweeps. SMLib asserts exact counts (2 near tangent when separated by more than tol, 1 otherwise; 2 or 1 at the seam; coincident segment counts 1/1/2/1/1). Kernel: `ssi::curve_surface` + `intersect::distinct`; assert on-surface, none/some by distance from tangency, and the exact count where the kernel can match it. Coincidence needs overlap ranges that `curve_surface` does not return (gap). Hand-written: the loops are short, parameters are literal, a generator would be over-engineering.
2. `my_test_drop_curve_near_seam` (20 cases) and `my_test_drop_iso_curves` (6). Build: cylinder r=0.25 with 5 curve kinds x swap x reverse; assert every sample of the p-curve lifts within 1.01 tol of the 3D curve, no jump across the seam, 1 curve for the helix. Kernel: `pcurve::pcurve` (+ `Surface::iso_u/iso_v` for the iso case). Self-contained, no fixtures, tight tolerance: a real robustness check of the p-curve builder. Hand-written.
3. `my_test_sp_nurb` (15 closest points on Surface.sms + 200-point MINIMIZE + 4 round trips, 21 outcomes) and the sp_analy revolution block (1). Build: `smb::read_surface` of Surface.sms (already copied to tests/data/smlib/reader/), assert `ssi::sdist(surf,p) <= 0.03` for pt1..pt15; project(eval(uv)) round trip on the 6x4 surface (surf3 points: `my_load_surf3_pts`, lines 89-165); nearest-point against brute-force sampling. Hand-written; the 15 points are copyable literals (4120-4134).
4. `my_test_section` (4 asserted: 10, 10, 21, 7) + `my_test_projection` (15, 29), plus the two shovel.srf smoke runs: 9 outcomes. Build: planes through `bbox.Evaluate(t,t,t)` and circle extrusions along -z against the 5x4 cubic surfaces (the point tables are in the source at 4354-4373 and 4453-4472; `my_create_nurb` is in smCurveTest.cpp:244); kernel `ssi::surface_surface(surf, Plane)` and `surface_surface(surf, Surface::Extrusion{circle, -z})`. SMLib counts degenerate point curves, so compare totals loosely, or the curve count if it matches. The shovel.srf runs need `smb::read_surface` to accept the legacy .srf (it should). Hand-written.
5. `my_test_csi_nurb` (12 outcomes: 92/85/92 line-grid counts, 9 curve counts 7,7,6,6,6,8,8,4,6): build the 9x7 cubic surface from `my_load_surf3_pts` (uniform knots, no creases), translate by (-6,0,0) and `my_create_3dcrv` (13 poles at smSurfaceTest.cpp:208-232, a cubic B-spline built by `my_create_nurb`, smCurveTest.cpp:244). Same loose-count approach as `csi_*`; upgrading `csi_*` from "some/none" to exact counts is not possible (seam duplicates), so these stay approximate.
6. `my_test_analytic_surfaces` (12 cases; runs twice in SMLib): 11x11 uv grid round trip through `Surface::project` for each analytic type, plus a UV-diagonal lift check by sampling. The STEP-domain parameters (degrees, sphere latitudes, semi-infinite cones) map to the kernel's own parametrisation (`Surface::Cone{r:0}`, `Sphere`, `Revolution`, `Extrusion`). Hand-written, cheap.
7. `my_test_surface_interface` (9 missing outcomes: line/curve/surface intersections, DropPoint, iso curves, skinned, offset) and `my_test_bsplinesurface_nlib` offset smoke: low value (SMLib's own checks are mostly status codes); only the intersection counts (1, 1, 2 curves) are real asserts and they depend on SMLib's ApproximatePoints surfaces, so they would need an interpolated stand-in.

Not worth converting without new kernel features (feature gaps, each from SMLib code that is actually run): curve-surface/surface-surface local-extremum solvers (`my_test_cs_nurb` 7 asserts, `my_test_ss_nurb` 2, `sp_nurb` NORMALIZE counts 2, interface 2); DropCurve/DropAndTrimCurve/LiftCurve with trimming (`my_test_drop_curve`, 26 outcomes with piece-count asserts on the 20 + 5 Drop/*.dat and doubly curved cases); coincident curve-on-surface ranges (`csi_coincidence` 10); surface-level NURBS algorithms exercised by `my_test_bsplinesurface_nlib` (16 smoke calls) and `Reparameterize`/`SwapUV`/`CreateExtendedSurface`/`ApproximatePoints` in the interface test. The Drop/*.dat fixtures could be read with `smb::read_curve/read_surface` once a DropCurve exists. `smFeatureTest.cpp` needs nothing: SMLib never runs it, and the kernel has no defeature executive.



Source: /home/casper/work/nvbrep/usd-brep/tests/SM_API_test/ (src/*.cpp, inc/, SM_API_test.sh), plus Examples/SM_API/boolean_bounds.cpp and Examples/SMLib/step_derivatives.cpp (compiled into the app). Counting rule: one case per asserted scenario; groups of identical checks (table rows, invalid-input lists) are counted individually.


Overall totals: covered 4, partial 5, missing 134, gap 34, n/a 373 (cases; bracketed number in each row is its case count).


### SM_API_test.cpp

Runner. `SM_API_test.sh <platform>` executes `SM_API_test_app` (premake5.lua:45), whose `main` (:1743) calls `run_sm_api` (:29): 115 sequential `stat = TestSmX()` calls (114 distinct functions; `TestSmIntersectBrepWithPlane` twice), pass/fail counted. 14 `TestSm*` functions in the sources are never called (listed in the rows): 10 aggregators/empty, plus `TestSmBoolean2DPlanarUnion`, `TestSmBoolean2DStatusPropagation`, `TestSmPipeSweep` (only reachable through the unrun aggregator `TestSmBreps`) and `TestSmCurveSweepFromFaces` (call commented out, :253). Two examples are compiled in (`Examples/SM_API/boolean_bounds.cpp`, `Examples/SMLib/step_derivatives.cpp`). Verified: no `TestSm*` name occurs in tools or crates/cadrs_kernel/tests, and no generator module reads SM_API_test. **None of the 114 run functions uses a fixture file**: every geometry is built in code through the C API (the only files are `sm_api_native_io_*.smb`, written and deleted at run time by TestSmImportExport). Almost every assertion is a status code or an API contract; real geometric values are few and listed below.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `main` :1743 / `run_sm_api` :29: 115 calls (114 distinct fns; `TestSmIntersectBrepWithPlane` runs at :159 and :1086), counts pass/fail [1] | none (built in code) | n/a | - | Runner only. Built as `SM_API_test_app` (premake5.lua:45); `SM_API_test.sh` runs it. |
| `TestSmBreps` (TestSmBrep.cpp:3156): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). calls the individual fns, which the runner also calls |
| `TestSmCurves` (TestSmCurves.cpp:1063): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmSurfaces` (TestSmSurfaces.cpp:671): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmPrimitives` (TestSmPrimitives.cpp:709): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmIntersectors` (TestSmIntersectors.cpp:768): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmTrimmedSurfaces` (TestSmTrimmedSurfaces.cpp:381): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmFillets` (TestSmFillets.cpp:306): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmGeneral` (TestSmGeneral.cpp:1408): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). |
| `TestSmHeal` (TestSmHeal.cpp:39): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). wraps `TestSmHealBrep`, which the runner calls |
| `TestSmPolygons` (TestSmPolygons.cpp:24): aggregator, never called by the runner [1] | none (built in code) | n/a | - | Dead (not in `run_sm_api`). empty, returns SM_SUCCESS |

Totals: covered 0, partial 0, missing 0, gap 0, n/a 11

### TestSmBrep.cpp

Booleans, merge lists, mass properties, tessellation, projection/cut, sweeps, shell/offset/stitch wrappers. Longest file (3339 lines). Most geometric content: TestSmMergeBreps (volumes 1144/48/144/2532/944), TestSmShellBrepFull (584/584/728/488), TestSmMassProperties (rotated box inertia), TestSmPrecisePropertySelection (face areas), TestSmSweepAlongPlanarPath (caps, volumes), TestSmMaterialCensus, BooleanBoundsExample. The rest asserts success only, on boxes/spheres where a closed form exists.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmBoolean` :41 API contract: self-union, EXTRACT_SEPARATE, poly self-union rejected with INVALID_INPUT; operands/poly volume unchanged [5] | none (built in code) | n/a | - | C API argument validation. |
| `TestSmBoolean` :41 box 5x10x15 at origin minus sphere r10 at (3,5,0) returns SM_SUCCESS [1] | none (built in code) | missing | - | Kernel::boolean(Subtract). Asserts success only; add: valid solid, volume in (0, 750). No closed form (numeric box/sphere volume). |
| `TestSmBooleanBoundsExample` :64 -> Examples/SM_API/boolean_bounds.cpp: 12x12x1.5 plate, 16 cylinders r0.75 (4x4 grid, pitch 3, centres ((c+.5)*3,(r+.5)*3), z -1..2.5) subtracted one by one; modes: no per-cut bounds / ordinary / tight [3] | none (built in code) | missing | - | Asserts per mode: manifold solid, volume = 216 - 16*pi*0.75^2*1.5 = 173.5885 (rel 1e-6), bbox min <= 1e-4 and max >= (12,12,1.5)-1e-4 after every cut (tight: within 1e-4), bbox queries leave volume/faces/edges unchanged. Kernel: boolean(Subtract, plate, [16 tools]) or 16 sequential, mass_properties, bounding_box. Same assertions in all 3 modes except the bbox-tightness: 3 cases. |
| `TestSmMassProperties` :483 box 2x6x10 rotated 30 deg about its centroidal Z, density 2: volume 120, area 184, mass 240, centroid (1,3,5), MOI about centroid Ixx 2560 Iyy 2240 Izz 800, POI Iyz~0 Izx~0 Ixy -277.13 (sign asserted negative) [1] | none (built in code) | missing | - | Kernel::transform + mass_properties (inertia about centroid at unit density; x2) . SIGN: SMLib POI.z = +int(xy)*rho = -277.1 here; our `inertia` off-diagonals are -int(xy) (types.rs:938), so expect +138.56 per unit density. Tolerances: vol/area/mass 0.5, centroid 1e-3, MOI 1% , POI 1%+5. |
| `TestSmMassProperties` :483 same box about the world origin (parallel axis): centroid, Ixx/Iyy/Izz (+ M(cy^2+cz^2)...), POI incl. Iyz=M*cy*cz, Izx=M*cz*cx, Ixy [1] | none (built in code) | missing | - | `MassProperties::inertia_about(point)`. Expected values closed form in source. |
| `TestSmMassProperties` :483 rejects density 0,-1,1e-13,NaN,inf; NaN accuracy (volume, area, mass props); NaN origin [8] | none (built in code) | n/a | - | C API validation. |
| `TestSmMassProperties` :483 accuracy 10.0 still succeeds; box area 184 within 1e-6 [1] | none (built in code) | missing | - | mass_properties.surface_area of the box (folded into the first MassProperties case). |
| `TestSmMassProperties` :483 face with SM_AI_MASS_PROPERTIES attribute makes mass props INVALID_INPUT [1] | none (built in code) | n/a | - | SMLib attribute system. |
| `TestSmMassProperties` :483 planar disc r5 (a sheet): mass props INVALID_INPUT; area 25*pi (rel 1e-3) [1] | none (built in code) | missing | - | Area of a planar-circle sheet = 25*pi (mass_properties.surface_area of a sheet body; contract differs: kernel gives volume 0, not an error, so the INVALID_INPUT half is n/a). |
| `TestSmMassProperties` :483 unit sphere's single face: centroid (0,0,0) 1e-3, area 4*pi rel 1e-3; zero-estimate `ComputePreciseProperties` at the pole must FAIL (SMLib solver limitation) [1] | none (built in code) | missing | - | Centroid/area of a sphere: closed form. The expected FAILURE is an SMLib integrator quirk (n/a, folded in). |
| `TestSmPrecisePropertySelection` :373 face-area sums: box 2x3x4 at (1e5,-2e5,3e5) =52; sphere r5 =100*pi; torus R5 r1 =20*pi^2; planar disc r5 minus hole r2 (Boolean2d difference) =21*pi; each rel 1e-3, Brep area = sum of faces [4] | none (built in code) | missing | - | mass_properties.surface_area (box far from origin tests precision). Annulus via the sheet-boolean harness (smlib_primitives uses it for Boolean2D). |
| `TestSmPrecisePropertySelection` :244 flag-selection ComputePreciseProperties (63 flag sets x orientation x accuracy x thickness) equals full integration, bitwise for ALL; invalid flag sets (4) fail [2] | none (built in code) | n/a | - | SMLib face-integrator selection API. |
| `TestSmPrecisePropertySelection` :373 thick sphere (thickness 0.1, offset surfaces r5.05/4.95): area 4*pi*(5.05^2+4.95^2), area first moments ~0, second moments 4*pi/3*(5.05^4+4.95^4) (and 2x for xyz-sums) [1] | none (built in code) | gap | - | Kernel has no surface-integral (area/first/second area moments) with thickness; MassProperties is volume-based. |
| `TestSmBrepDistance` (TestSmBrepDistance.cpp:25) unit cubes at 0 and (4,0,0): 3.0 (1e-4); reversed equal (1e-9); touching at (1,0,0): 0; 10^3 box vs 4^3 box at (8,3,3) crossing: 0; 10^3 box vs 2^3 box at (3,3,3) inside: 3.0 [5] | none (built in code) | missing | - | Kernel::min_distance(DistanceTarget::Body, Body) (boundary distance, matches the inside=3 case). Values in source. |
| `TestSmBrepDistance` NULL operands fail [2] | none (built in code) | n/a | - |  |
| `TestSmMaterialCensus` :3296 census (solids, voids): box (1,0); box 10 minus box 4 at (3,3,3) (1,1); two disjoint 2-cubes (2,0) [3] | none (built in code) | missing | - | Kernel::solid_count for lumps; voids only via the Body shell structure (hollow: volume 936, 1 solid with a void shell). A `voids` query is not in the Kernel trait: use native::topo or add one. |
| `TestSmMaterialCensus` :3296 hollow box MERGEd with a 2-cube inside its cavity (BooleanMerge): (2 solids, 1 void) [1] | none (built in code) | gap | - | Non-manifold merge of a solid nested in a void; kernel booleans return manifold results (cells/compound keep separate bodies). |
| `TestSmMaterialCensus` :3296 NULL brep fails [1] | none (built in code) | n/a | - |  |
| `TestSmMergeBreps` :916 invalid inputs: empty, singleton, NULL entry, duplicate entry, bad op (kernel `merge_breps` x5 incl. op 4) and API x7 (empty, singleton, NULL, duplicate, UNKNOWN/EXCLUSIVE_OR/PARTIAL_MERGE); inputs preserved [12] | none (built in code) | n/a | - | List-ownership contract of SmMerge::merge_breps / SmApiMergeBreps. |
| `TestSmMergeBreps` :916 A = box 10^3 at 0, B = box 4x8x6 at (8,6,3): UNION 1144, INTERSECTION 48, DIFFERENCE 144 (= B - A: the LAST list entry minus the earlier ones), each manifold, volume rel 2e-4 [3] | none (built in code) | missing | - | Kernel::boolean(Unite/Intersect/Subtract). Note the list-order semantics of DIFFERENCE. |
| `TestSmMergeBreps` :916 SM_BO_MERGE of A and B: not a manifold solid, has faces [1] | none (built in code) | gap | - | Non-manifold B-rep result; closest is Kernel::cells (separate bodies). |
| `TestSmMergeBreps` :916 union of 3 boxes 10^3 at (0,0,0),(5,5,5),(8,8,8): volume 2532, manifold [1] | none (built in code) | missing | - | 2532 = 3000 - (125+8+343) + 8. Kernel::boolean(Unite) twice. |
| `TestSmMergeBreps` :916 DIFFERENCE list [2^3 at (1,1,1.5), B, A]: volume 944, manifold [1] | none (built in code) | missing | - | = 1000 - 48 - 8. |
| `TestSmTessellate` :1348 / `TestSmTessellateFailures` :1127: box with face 3's UV trim curves shifted by (1000,1000): partial/strict modes, SmTessellationReport (lamina 4, 1 failure at stage BOUNDARY_PREPARATION, mesh of 10 triangles-faces), all-faces-shifted, NULL/invalid [5] | none (built in code) | n/a | - | Fault injection into p-curves and SMLib's partial-tessellation report API; Kernel::tessellate has no partial/report mode. |
| `TestSmTessellateBoundaries` :1205 box 10^3: boundary polylines at tol 5: 12 edges, 24 points, 2 points per edge [1] | none (built in code) | missing | - | TriMesh.edges (one polyline per edge; straight edges 2 points). |
| `TestSmTessellateBoundaries` :1205 edge curves removed (1 edge, then all) -> SM_ERR with counts 0; tol 0 / NaN / NULL brep invalid [5] | none (built in code) | n/a | - |  |
| `TestSmTessellateMaxEdgeCap` :1266 maxEdge = -4 (relative, capped) gives the same mesh as absolute 0.1*sqrt(300) [1] | none (built in code) | n/a | - | SMLib's relative max-edge convention. |
| `TestSmTessellateParams` :1316 cylinder r5 h10: chord 0.001 gives more vertices than default [1] | none (built in code) | missing | - | Kernel::tessellate with two Tessellation qualities. Trivial. |
| `TestSmTessellate` :1348 tail: box 5x10x15 all edges filleted 0.5 tessellates [1] | none (built in code) | missing | - | Success only. |
| `TestSmProjectBrepOntoPlane` :1376 box 5x10x15 filleted 0.5, moved +30 z, projected on z=0: succeeds; scaled normal (0,0,4) gives the same curve count [1] | none (built in code) | missing | - | Weak (count only). Kernel::silhouettes (parallel view along +z) or `project`; true outline: 5x10 rounded rectangle, perimeter 30 - 8*0.5 + 2*pi*0.5. |
| `TestSmProjectBrepOntoPlane` :1376 zero normal / NULL brep INVALID_INPUT; normal not normalised in place [3] | none (built in code) | n/a | - |  |
| `TestSmCreateSilhouetteCurves` :1432 box 10^3, plane z=20 normal +z: succeeds with >0 curves [1] | none (built in code) | missing | - | Op is covered on SMLib's models by smlib_silhouette.rs (34 cases, `Kernel::silhouettes`); this exact call (count>0) is not. |
| `TestSmCreateSilhouetteCurves` :1432 scaled normal same count; zero normal INVALID; append-to-array semantics [3] | none (built in code) | n/a | - |  |
| `TestSmCut` :1495 filleted box 5x10x15 cut by plane z=7: succeeds [1] | none (built in code) | missing | - | Kernel::split(Plane) (keep one side); no volume asserted. Add: volume of the lower part. |
| `TestSmCut` :1495 zero normal INVALID and brep unchanged; scaled normal gives the same face count [2] | none (built in code) | n/a | - |  |
| `TestSmProjectAndTrim` :1542 curve through (-10,-2),(0,5),(10,0),(20,2) at z=20 projected along -z onto filleted box 10x5x15 and trimmed (ref point (5,1,20)): succeeds; a curve at x=100..103 (misses) fails with the brep unchanged [2] | none (built in code) | gap | - | Project a curve onto a body's faces and split/trim them (imprint). Kernel::split_faces takes plane/face/body tools only. |
| `TestSmProjectAndTrim` :1542 zero direction INVALID unchanged; scaled direction same face count [2] | none (built in code) | n/a | - |  |
| `TestSmProjectCurve` :1616 same curve projected along -z onto the box: >0 curves; a shifted curve (x+1000) -> SM_ERR with output unchanged [2] | none (built in code) | gap | - | Curve-on-body projection returning 3D curves. |
| `TestSmProjectCurve` :1616 zero direction INVALID; append/borrowed-array semantics [3] | none (built in code) | n/a | - |  |
| `TestSmCurveSweep` :1731 circle r2 swept (no options) along the path (0,0,0)-(0,0,3)-(0,0,7)-(0,0,10): SM_SUCCESS [1] | none (built in code) | missing | - | Kernel::sweep_with. The path is a straight line, so closed form: cylinder volume 40*pi = 125.66, 3 faces. SMLib asserts success only. |
| `TestSmCurveSweepFromFaces` :1774 (disc r2 swept along the same path) [1] | none (built in code) | n/a | - | Dead: call commented out at SM_API_test.cpp:253. Same closed form as above. |
| `TestSmTaperExtrude` :1817 square 10x10 (4 segments), height 10, draft 5 deg, end caps 3, no fillet: succeeds; list with a NULL entry INVALID [1] | none (built in code) | missing | - | extrude then Kernel::draft (or loft). Success only; closed form if drafted inward: h/3*(A1+A2+sqrt(A1*A2)) with side 10 - 2*10*tan5deg = 8.2502 (sign convention of SMLib's angle must be checked). |
| `TestSmTaperExtrude` NULL entry rejected [1] | none (built in code) | n/a | - |  |
| `TestSmPipeSweep` :1856 circle pipe r1 along zig-zag path (0,0,0),(5,5,0),(10,0,0),(15,5,0), caps [1] | none (built in code) | n/a | - | Dead: only called from the unrun aggregator `TestSmBreps`. Kernel op: sweep_with. |
| `TestSmNonManifoldSweep` :2151 face 0 of box 10^3 swept 5 along +z into the same brep; sweeps with a scaled direction give the same bbox/face count as a unit direction [1] | none (built in code) | partial | - | The op (SmTopologySweep::DoSweep) is covered by smlib_sweeps.rs (`my_sweep_demo` cases: face to solid, translation); this input is not. Success only. |
| `TestSmNonManifoldSweep` :2151 zero direction, scaled dir equal, NULL entry in faces/edges/vertices (3), foreign target (2), stale face after Cut (1) [8] | none (built in code) | n/a | - | Selection-list validation / handle liveness. |
| `TestSmNonManifoldRotationalSweep` :2236 box 5^3 face 0 swept 90 deg about Z through origin: succeeds [1] | none (built in code) | partial | - | As above, rotational cases of smlib_sweeps.rs. |
| `TestSmNonManifoldRotationalSweep` :2236 NULL entries (3), foreign target (2), stale entry (1) [6] | none (built in code) | n/a | - |  |
| `TestSmBooleanWithOptions` :2287 box 10^3 minus sphere r5 centred (5,5,5) (inscribed, tangent to all 6 faces) with options (cookie F, imprint F, post-process T, classify F, keep F): succeeds [1] | none (built in code) | missing | - | Kernel::boolean(Subtract) with tangent contacts at 6 points (a robustness case). Closed form volume 1000 - 500*pi/3 = 476.401. |
| `TestSmDeleteFaceKeepsInfiniteRegion` :2443 prism (20x12 profile, height 100) with/without top cap deleted, merged with a cylinder wall surface (r30 at (0,-44,60)), MakeManifold keeping the bottom cap's region; infinite-region invariants [2] | none (built in code) | n/a | - | SMLib region/infinite-region topology; no kernel analogue. |
| `TestSmNonManifoldBoolean` :2461 box 10^3 UNION inscribed sphere r5 at (5,5,5): succeeds [1] | none (built in code) | missing | - | Kernel::boolean(Unite); result is the box: volume 1000 (sphere is inside, tangent). |
| `TestSmPiecewiseMerge` :2485 box 10^3 at 0 and at (5,0,0), flag FALSE: succeeds [1] | none (built in code) | missing | - | Kernel::boolean(Unite): volume 1500 (assuming the union semantics). |
| `TestSmBooleanLists` :2509 lists [box 10^3 at 0], [box 10^3 at (5,0,0)], op 1 (= SM_BO_UNION): succeeds [1] | none (built in code) | missing | - | Union volume 1500. |
| `TestSmShellBrepFull` :2544 box 10^3, shell distance 1 (extend corners, stitch) with face 0 as the opening: volume 584 (solid result); duplicate face list: 584; no faces: 728; distance -1 no faces: 488 (tol 1e-5) [4] | none (built in code) | missing | - | Kernel::shell_with(ShellSpec{outward, sharp}): 584 = 12^3 - 10^3 - 12*12*1 slab; 728 = 12^3 - 10^3; 488 = 10^3 - 8^3. Table of 4 rows -> trivially generated. |
| `TestSmShellBrepFull` :2544 distance 0 INVALID; a face of another brep INVALID; input box (counts, volume 1000, bbox, face area 100) unchanged after each call [3] | none (built in code) | n/a | - |  |
| `TestSmOffsetBrepFull` :2745 box 10^3 offset 1.0 (flags TRUE,TRUE): succeeds [1] | none (built in code) | missing | - | Kernel::offset. Success only; with extended corners the volume is 12^3 = 1728 (rounded corners: 1000+600+23.5pi+4.19, if the flags mean the opposite). |
| `TestSmStitchIntoSolid` :2765 box 10^3: succeeds [1] | none (built in code) | missing | - | surfacing::sew / Kernel::sew_solid on an already closed solid. Success only. |
| `TestSmStitchIntoShell` :2789 box 10^3, tol 10: succeeds [1] | none (built in code) | missing | - | Success only. |
| `TestSmUnifyNormals` :2813 box 10^3, start face 0: succeeds [1] | none (built in code) | missing | - | Kernel::heal (face senses / shell orientation fix). Success only. |
| `TestSmAdvancedStitch` :2843 box 10^3, tol 0.001, flags TRUE,TRUE,TRUE,FALSE: succeeds [1] | none (built in code) | missing | - | Success only. |
| `TestSmSweepAlongPlanarPath` :2872 rectangle 2x1 (4 segments) swept along the line (0,0,0)-(20,0,0), caps: succeeds with >=1 face [1] | none (built in code) | missing | - | Kernel::sweep_with. Closed form volume 2*1*20 = 40. |
| `TestSmSweepAlongPlanarPath` :2872 circle r1 along (0,0,0)-(0,0,10), capEnds 0/1/2/3: faces 1+start+end, manifold iff both caps, start cap at z=0 / end cap at z=10, 1 side face; open results have \|volume\|<1e-8; capped volume 10*pi (1%) [4] | none (built in code) | missing | - | sweep_with Surface (sheet) / Solid; 4 rows of a table (capEnds, expected faces 1/2/2/3, manifold F/F/F/T). The sheet volume 0 matches MassProperties for surface bodies. |
| `TestSmSweepAlongPlanarPath` :2872 circle r1 along two connected segments (0,0,0)-(0,0,10)-(5,0,15), both caps: 4 faces, 5 edges, 3 vertices, 2 regions (1 material), manifold, volume pi*(10+sqrt(50)) (2%) [1] | none (built in code) | missing | - | sweep_with with a two-curve path (kink). SMLib's counts include seams; compare volume only (and faces if the kernel has the same split). |
| `TestSmSweepAlongPlanarPath` :2872 empty curve lists fail; capEnds -1 and 4 INVALID; input curves unchanged [4] | none (built in code) | n/a | - |  |
| `TestSmBoolean2DPlanarUnion` :98 two 10x10 planes at x=0 and x=5, 2D union: area 150 (0.03), bbox (0,0,0)-(15,10,0) [1] | none (built in code) | n/a | - | Dead (only via aggregator `TestSmBreps`), but a clean value: sheet union area 150. Convertible if wanted. |
| `TestSmBoolean2DStatusPropagation` :180 non-coplanar 2D union -> INVALID_INPUT, operands intact [1] | none (built in code) | n/a | - | Dead; API contract. |

Totals: covered 0, partial 2, missing 53, gap 7, n/a 83

### TestSmCurves.cpp

Curve creation and queries through the C API. Of 17 functions only OffsetCurve, DropCurveToSrf, ProjectCurveToSurface assert geometry (curve offset ends, projected sine curve ends); two functions are big input-validation tables (HelixInvalidInputs, 54 table rows; CurveInvalidInputs, 33 checks). Curve offset/drop/project/lift/make-compatible/knot-removal are kernel gaps.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmCreateLineSegment` :44 segment, zero-direction line, ellipse with negative radii / far centre (1e16, minor 1e10 ok, swapped invalid) [5] | none (built in code) | n/a | - | C API creation/validation. |
| `TestSmCreateCircle` :87 circle r10; `TestSmCreateArc` :107 arc 0-135 r10 + Evaluate with NaN/past-end/NULL [4] | none (built in code) | n/a | - | Success only / API validation. |
| `TestSmCreateCurve` :143 interpolate (0,0),(10,-3),(20,3),(30,-6): success, knot type unspecified; 0..3 points INVALID [5] | none (built in code) | n/a | - | Kernel Nurbs::interpolate exists; SMLib asserts no geometry. |
| `TestSmCreateCanonicalCurve` :184 rational cubic, 7 poles, knots 0,.3,.6,.8,1 mult 4,1,1,1,4, weights 1.5/.5 alternating; 2 poles degree 3 rejected [2] | none (built in code) | n/a | - | Constructor validation (Nurbs::new/validate; fuzz_readers covers). |
| `TestSmOffsetCurve` :252 offset of the interpolated spline by 3.0 succeeds; line along x offset 2: ends \|y\|=2, z=0; line along z offset 2: ends \|x\|=2, y=0, z=0 and 30 [3] | none (built in code) | gap | - | No curve offset in the Kernel API (planar wire offset exists only inside native::offset::wire for profiles). Line cases have closed forms. |
| `TestSmDropCurveToSrf` :318 spline at z=0 dropped on the 5x7 B-spline surface (35 poles inline): >=1 piece, 2D and 3D lists same length [1] | none (built in code) | gap | - | No curve-onto-surface drop/projection (curve-on-surface p-curve + 3D curve). |
| `TestSmDropCurveToSrf` :318 2D/3D-only outputs, accumulation (2x), distinct curves, NULL inputs [3] | none (built in code) | n/a | - |  |
| `TestSmCreateRectangle` :424 4 segments; far centre / tiny height; NaN, inf [4] | none (built in code) | n/a | - |  |
| `TestSmCreateRegularPolygon` :462 hexagon r5: 6 segments; tiny radius far away; NaN/inf [4] | none (built in code) | n/a | - | Profile construction is above the Kernel trait (cadrs sketch). |
| `TestSmProjectCurveToSurface` :498 curve at z=2 onto the 10x10 patch: >=1 3D curve; 2D/3D lengths; sine curve y=5+1.5sin x, x in [-5,15], z=.5 projected AND dropped onto the patch: exactly 1 piece, start.x<1e-3, end.x in [9.5,10], z=0 [3] | none (built in code) | gap | - | Curve projection onto a surface (clipped to its domain). Closed form for the sine case. |
| `TestSmProjectCurveToSurface` :498 accumulation, NULL inputs [2] | none (built in code) | n/a | - |  |
| `TestSmLiftUVCurve` :576 project a 3D curve to uv then lift back: >=1 curve [1] | none (built in code) | gap | - | Lifting a uv curve to a 3D B-spline on a surface (kernel evaluates surfaces but has no such curve op). |
| `TestSmMakeCurvesCompatible` :615 circles r5 at z0 and r3 at z5: succeeds [1] | none (built in code) | gap | - | No public op to make B-spline curves compatible (internal to loft_build). |
| `TestSmOrderCurves` :638 order the 4 rectangle segments: 4 remain [1] | none (built in code) | n/a | - | Profile loops are order-free in cadrs. |
| `TestSmCreateHelixInvalidInputs` :665 54 table rows (non-finite origin/height/radii/turns/tol, extremes) -> INVALID_INPUT and NULL result [54] | none (built in code) | n/a | - | Table-driven (regular) but pure input validation; the kernel has no helix constructor. |
| `TestSmCreateCurveInvalidInputs` :815 circle (14 bad centres/radii), just-above-zero radius ok, interp points: 3 points, NaN, 3 bad parameterisation enums, duplicate/near-duplicate/all-same points x chord/centripetal (6), uniform with duplicates ok, EvaluateContinuity checks (4) and failure propagation (2) [33] | none (built in code) | n/a | - | Validation and SMLib status propagation. |
| `TestSmRemoveCurveKnots` :1028 linear curve, 3 collinear poles, interior knot 0.5: after removal 4 knots remain [1] | none (built in code) | gap | - | Nurbs has insert_knot only, no knot removal. |

Totals: covered 0, partial 0, missing 0, gap 10, n/a 117

### TestSmSurfaces.cpp

Surface creation (B-spline from poles/points, extrude, ruled, revolve, offset, skin). All success-only except two expected failures in OffsetSurface (sphere r10 offset -20, torus offset -5), uv evaluation validation, and degenerate inputs. The 35-pole net is inline.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmCreateSurface` :27 5x7 net: success; 2-row nets rejected [3] | none (built in code) | n/a | - | Constructor validation; NurbsSurface::new covers. |
| `TestSmCreateSurfaceFromPoints` :91 interpolate an ordered 5x7 point grid; `...FromRandomPoints` :145 fit scattered points [2] | none (built in code) | gap | - | No surface interpolation / scattered-data fit in the kernel (through_curves needs curves). Success only. |
| `TestSmCreateSurfaceFromCornerPoints` :199 bilinear patch (0,0,0),(10,0,0),(0,12,0),(10,12,0); point/normal/derivative evaluation with NaN, out-of-domain, NULL uv/surface rejected [10] | none (built in code) | n/a | - | API validation. |
| `TestSmCreateExtrude` :246 extrude spline along z: success; zero vector, degenerate sweep path fail [3] | none (built in code) | n/a | - | Success/validation only. |
| `TestSmCreateRuledSurface` :302 ruled surface between two 6-point splines (z=0, z=10) [1] | none (built in code) | missing | - | Kernel::loft_with(BodyKind::Surface, 2 sections). Success only. |
| `TestSmCreateSurfaceRevolution` :355 spline revolved 180 deg about x: success [1] | none (built in code) | missing | - | revolve_with Surface. Degenerate curve and NULL rejected (2, n/a). |
| `TestSmCreateSurfaceRevolution` :355 degenerate / NULL curve fail [2] | none (built in code) | n/a | - |  |
| `TestSmCreateSurfaceRevolutionSphere` :415 semicircle r10 360 deg; `...Torus` :441 circle r5 at (50,0,0) turned 90 about x then revolved about z; `...Cylinder` :470 line (5,0,0)-(5,0,10); `...Cone` :496 line (5,0,0)-(0,0,10); `...TruncCone` :522 line (5,0,0)-(3,0,10) [5] | none (built in code) | missing | - | Success only. Kernel::revolve_with Surface; closed-form areas: 400*pi, 4*pi^2*50*5 = 9869.6, 2*pi*5*10 = 314.16, pi*5*sqrt(125) = 175.62, pi*(5+3)*sqrt(104) = 256.3. Shapes are the kind smlib_primitives already revolves. |
| `TestSmCreateOffsetSurface` :548 offset of the 5x7 spline surface by -12 succeeds [1] | none (built in code) | gap | - | Kernel offsets bodies, not bare surfaces; success only. |
| `TestSmCreateOffsetSurface` :548 sphere r10 offset -20 FAILS (collapsed); torus R10 r2 offset -5 FAILS [2] | none (built in code) | missing | - | Kernel::offset(sphere body, -20) and torus -5 should return an error. Real geometric outcomes. |
| `TestSmCreateOffsetSurface` :548 NULL input INVALID [1] | none (built in code) | n/a | - |  |
| `TestSmCreateSkinSurface` :641 skin through circles r5 (z0), r3 (z5), r5 (z10): success [1] | none (built in code) | missing | - | loft_with(Surface, 3 profile sections). Success only. |
| `SmTestConeOffset` :698 and `SmTestSphereOffset` :756 (inside `#if 0`) [2] | none (built in code) | n/a | - | Dead code. |

Totals: covered 0, partial 0, missing 10, gap 3, n/a 21

### TestSmPrimitives.cpp

Primitive creation and input validation, sweeps (linear, rotational, draft, swung), skin primitives. Real values only in TestSmCreateSkinPrimitive (linear skin volume 1130*pi/3, annulus skin 295*pi, manifold, face counts) and CylindricalBox rejections (already converted from smSweepTest). Linear sweeps/revolves assert success only, but closed forms exist.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmCreateSphere` :30 sphere r10 ok; 10 invalid-parameter rejections (zero radius sphere/cone/cylinder/torus, partial sphere/cone/torus, cylindrical box, elliptic-end cone, INVALID codes of Sphere and SphereNoPole) [12] | none (built in code) | n/a | - |  |
| `TestSmCreateSphere` :30 SphereNoPole with radius -5 succeeds (inside-out) [1] | none (built in code) | n/a | - | SMLib negative-radius quirk; positive `CreateSphereNoPole` is exercised by the sweeps suite (cases 27-29). |
| `TestSmCreateBox` :88 box 5x10x20; zero and negative dims INVALID [3] | none (built in code) | n/a | - |  |
| `TestSmCreateBox` :88 CreateCylindricalBox rejections (10,5,2,0,90), (10,2,5,400,450), (10,2,5,90,10), (10,2,5,0,400) [4] | none (built in code) | covered | smlib_primitives::cylindrical_box_3,4,5,6 | smlib_primitives.rs `cylindrical_box_3`..`cylindrical_box_6` (same arguments, from smSweepTest.cpp:1278). |
| `TestSmCreateBox` :88 CreateCylindricalBox with start = end angle (45,45) rejected [1] | none (built in code) | missing | - | Add one row to the `cylindrical_box` table. |
| `TestSmCreateCone` :132 cone r5->10 h20 and its surface variant; zero radii, negative radii/height rejected (8 calls) [10] | none (built in code) | n/a | - |  |
| `TestSmCreateTorus` :185 torus R10 r5 [1] | none (built in code) | n/a | - | Success only; the primitives suite covers the torus volume. |
| `TestSmCreatePlane` :204 plane 10x15 (brep and surface); zero size rejected (2) [4] | none (built in code) | n/a | - |  |
| `TestSmCreatePlanarCircle` :238 disc r10; r0 rejected [2] | none (built in code) | n/a | - |  |
| `TestSmCreateRotationalSweep` :261 semicircle r10 revolved 360 deg about x (a sphere) [1] | none (built in code) | missing | - | Success only; closed form sphere r10 V 4188.8. Revolve of an arc is in smlib_primitives (my_primsol_demo). |
| `TestSmCreateRotationalSweep` :261 degenerate curve rejected; linear sweep with 0 repetitions rejected [2] | none (built in code) | n/a | - |  |
| `TestSmCreateLinearSweep` :311 half disc (semicircle r10 + chord) swept 10 along z, capped [1] | none (built in code) | missing | - | Kernel::extrude. Closed form V = pi*100/2*10 = 1570.8 (SMLib asserts success only). |
| `TestSmCreateLinearSweepPlane` :345 segment (0,0,0)-(4,2,0) swept 10 along z, no caps (a sheet) [1] | none (built in code) | missing | - | extrude Surface; area sqrt(20)*10 = 44.72. |
| `TestSmCreateLinearSweepCylinder` :373 circle r10 swept 10, capped [1] | none (built in code) | missing | - | V = 3141.6. |
| `TestSmCreateDraftSweep` :401 rectangle 10x20 drafted -5 deg over height 10, caps [1] | none (built in code) | partial | - | smlib_primitives covers the draft sweep of `my_primsol_demo`'s dead block; this rectangle is not. Success only. |
| `TestSmCreateSwungPrimitive` :441 arc r5 in XY and arc r8 in XZ swung (scale 1.0) [1] | none (built in code) | gap | - | No swung (two-profile) primitive in the kernel. |
| `TestSmCreateSkinPrimitive` :478 circles r5/r8/r3 at z0/5/10, quadratic skin: manifold solid, 1 start face, 1 side face, 1 end face; input curves unchanged [1] | none (built in code) | missing | - | Kernel::loft_with(Solid, 3 sections); face counts may differ (seams). |
| `TestSmCreateSkinPrimitive` :478 same circles, LINEAR skin (order 1): manifold, volume 1130*pi/3 (rel 1e-4) [1] | none (built in code) | gap | - | The loft is smooth only: no ruled (degree 1) mode in LoftSpec. Piecewise-cone volumes would be asserted. |
| `TestSmCreateSkinPrimitive` :478 skin of a circle and a line segment profile fails, outputs cleared [1] | none (built in code) | missing | - | loft_with(circle section, open section) must error. |
| `TestSmCreateSkinPrimitive` :478 annulus profiles (outer r5/8/3, inner r2/4/1 at z0/5/10), linear: manifold, volume 295*pi (rel 1e-4) [1] | none (built in code) | gap | - | Ruled loft of regions with holes (volume = 1130pi/3 - 49*5pi/3). |
| `TestSmCreateSkinFromFaces` :665 skin planar discs r5 (z0) and r8 (z10), degree 3: succeeds [1] | none (built in code) | missing | - | loft_with(LoftSection::Face x2). Success only. |

Totals: covered 4, partial 1, missing 8, gap 3, n/a 35

### TestSmIntersectors.cpp

Intersection wrappers. Real values: curve-surface (2 points at x=2,8), curve-face (hole misses, (1,1,10) hit), curve-brep (diagonal gives exactly 2 points, along-edge gives 1 curve). Curve-curve, surface-surface, brep-plane, brep-brep assert success only.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmIntersectCurves` :78 splines through (0,0),(10,-3),(20,3),(30,-6) and (10,-5),(15,-3),(10,3),(20,6): success [1] | none (built in code) | missing | - | native::ssi_cci::curve_curve. No count asserted (add the kernel's own count check). |
| `TestSmIntersectSurfaces` :120 5x7 B-spline surface (35 poles inline) vs the cylinder from the line (-50,-40,0..100) revolved about z: success and >=1 curve [1] | none (built in code) | missing | - | ssi::surface_surface (tests/smlib_common/intersect.rs helpers). Fixture inline. |
| `TestSmIntersectBrepWithPlane` :203 (run twice) filleted box 5x10x15 sectioned by plane z=7: success [1] | none (built in code) | missing | - | boolean::section / Kernel::split. No count asserted. True section: closed rounded rectangle. |
| `TestSmIntersectBrepWithPlane` :203 scaled normal same count; zero normal INVALID; append semantics; NULL brep [4] | none (built in code) | n/a | - |  |
| `TestSmIntersectBreps` :269 box 5x10x15 vs sphere r10 at (3,5,0): success (curves+points) [1] | none (built in code) | missing | - | Weak; add curve closure / on-both-surfaces check via boolean::section. |
| `TestSmIntersectCurveSurface` :301 circle r3 at (5,5,0) with the tilted plane patch (0,0,-5),(10,0,-5),(0,10,5),(10,10,5): points/curves on the surface within 0.01 (true: 2 points (2,5,0),(8,5,0)); circle r2 at (5,5,0) with the plane z=0 patch: >=1 coincident curve on the surface [2] | none (built in code) | missing | - | ssi::curve_surface (params) and ssi_coincident. Closed forms. |
| `TestSmIntersectCurveFace` :429 line (-5,5,5)-(15,5,5) vs each face of box 10^3: a hit on some faces, on the surface within 0.01 (true (0,5,5),(10,5,5)); holed box (cyl r2 at (5,5)) top face: axis line 0 hits; off-axis line at (1,1): exactly 1 point (1,1,10) (1e-6) [3] | none (built in code) | missing | - | Kernel::ray_hits gives hits per body; per-face trimmed hits need native face classification. Values in source. |
| `TestSmIntersectCurveFace` :429 circle r2 at (5,5,10) lying on the top face: coincident curve found [1] | none (built in code) | gap | - | Curve-on-face overlap (curves, not points) is not exposed. |
| `TestSmIntersectCurveBrep` :596 line (-5,5,5)-(15,5,5) vs box: >=2 points on the boundary; axis line through the holed box's hole: 0 hits; diagonal (-5,-5,5)-(15,15,5) through two vertical edges: exactly 2 points [3] | none (built in code) | missing | - | Kernel::ray_hits (segment). The diagonal case tests de-duplication of edge hits. |
| `TestSmIntersectCurveBrep` :596 line (2,0,0)-(8,0,0) along a box edge: any hit on the brep; and exactly 1 overlap curve [2] | none (built in code) | gap | - | Curve overlapping a body's edge/faces returns curves. |

Totals: covered 0, partial 0, missing 12, gap 3, n/a 4

### TestSmTrimmedSurfaces.cpp

Trimmed-surface creation. TrimSurfaceWith3dCurves and CreatePlanarFaces use the same L-shaped plane with a triangular and a circular hole as SMLib's `my_test_tsurf_creation` (already converted in smlib_trimmed_surfaces.rs, geometry only). TrimProjectParallel is a kernel gap.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmTrimSurfaceWith3dCurves` :30 10x10 plane trimmed by 3 loops (6 lines of the L-shape, a triangle, a circle r1.5 at (6,2.5)): 1 face, loops [6,3,1], 10 boundary edges, outer loop 6 edges/6 vertices, loop closure by vertices [1] | none (built in code) | partial | smlib_trimmed_surfaces::tsurf_creation_volume_area, tsurf_creation_point_classes | Same face as smlib_trimmed_surfaces `tsurf_creation_*` (area 70.5 - 2.25pi, extruded; classify). Loop/edge/vertex counts are not asserted there; Kernel::faces/edges (loops) could. |
| `TestSmTrimSurfaceWith3dCurves` :30 NULL face INVALID [1] | none (built in code) | n/a | - |  |
| `TestSmTrimProjectParallel` :178 25x25 plane patch: line (-10,15,10)-(30,15,10) projected along -z with keep-point (5,5,0), then line x=15 on the result: both succeed (true result [0,15]^2, area 225) [2] | none (built in code) | gap | - | Trim a surface by projected curves keeping the region of a point; kernel splits with plane/face/body tools only. |
| `TestSmTrimProjectParallel` :178 surface owned by a box rejected and box left intact; zero direction; degenerate surface [3] | none (built in code) | n/a | - |  |
| `TestSmCreatePlanarFaces` :318 the same 10 curves (unordered, no loop counts) give planar faces [1] | none (built in code) | partial | smlib_trimmed_surfaces::tsurf_creation_* | Same face as above (covered geometrically). Loop detection from an unordered curve soup is not exercised by the Profile API. |

Totals: covered 0, partial 2, missing 0, gap 2, n/a 4

### TestSmBrepCopy.cpp

Copy semantics of SmBrep (C++ copy constructors and SmApiBrepCopy) on 5 primitives. API-plumbing.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmBrepCopyRepresentation` :77 box 4x5x6, cylinder r3 h8, cone 3->1 h8, sphere r3, torus 5/1: API copy, copy after one face is NURBS, after TurnToNurbs, strict copy ctor, MergeBrep copy: face/edge counts, surface and curve types, uv domains, sample points equal (1e-8) [21] | none (built in code) | n/a | - | C++ copy-constructor / SmApiBrepCopy representation semantics. |

Totals: covered 0, partial 0, missing 0, gap 0, n/a 21

### TestSmHeal.cpp

One call: heal a box.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmHealBrep` :22 box 10^3 healed: succeeds [1] | none (built in code) | missing | - | Kernel::heal(body, ops) on a sound box: unchanged, valid, volume 1000. Trivial. |

Totals: covered 0, partial 0, missing 1, gap 0, n/a 0

### TestSmImportExport.cpp

Round trip of a box through the native .smb writer/reader (ascii and binary).

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmImportExport` :64 box 5x10x20 written to .smb ascii and read back: face/edge/vertex counts equal [1] | none (built in code) | missing | - | Kernel::write_body/read_body round trip (+ volume 1000). The files are runtime outputs in the cwd, not fixtures. |
| `TestSmImportExport` :64 same through the binary .smb format [1] | none (built in code) | n/a | - | One native format in the kernel (no ascii/binary switch). |

Totals: covered 0, partial 0, missing 1, gap 0, n/a 1

### TestSmFillets.cpp

Fillet wrappers on a box. All success-only except SetBevelCorners (face-count comparison). Closed forms are available for all-edge fillet.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmCircularFillet` :26 box 5x10x15, all edges r0.5 [1] | none (built in code) | missing | - | Kernel::fillet (all 12 edges). Success only in SMLib; closed form with spherical corners: V = 750 - (1-pi/4)*r^2*108 - 8*r^3*(1-pi/6) = 743.7293. |
| `TestSmChamferFillet` :47 same box, all edges chamfer 0.5 [1] | none (built in code) | missing | - | Kernel::chamfer. Success only; oracle `chamfer in out 0.5 all` can supply the volume. |
| `TestSmFilletEdges` :68 box 10^3, first 4 edges (SMLib edge order), r1, (1,1,1.0) [1] | none (built in code) | missing | - | fillet_with. Success only; picking 'the first 4 SMLib edges' needs an order mapping. |
| `TestSmFilletEdgesPerEdge` :96 first 4 edges with radii 0.5,0.75,1.0,1.25 (circular) [1] | none (built in code) | missing | - | fillet_with per edge. Success only. |
| `TestSmVariableRadiusFillet` :132 first 4 edges, radius 0.5 -> 2.0 [1] | none (built in code) | missing | - | Kernel::fillet_variable. Success only. |
| `TestSmSurfaceSurfaceFillet` :161 planar patch z=0 (10x10) and the vertical patch x=5 (y 0..10, z -5..5): fillet r2, r2, tol .01: result brep [1] | none (built in code) | gap | - | Fillet between two bare surfaces; the kernel blends body edges. |
| `TestSmFilletPreview` :201 box 10^3 first 4 edges r1: preview surfaces [1] | none (built in code) | gap | - | No blend-surface preview API. |
| `TestSmSetBevelCorners` :252 box 10^3 all edges r1 with and without corner vertex 0 bevelled: both have >6 faces and different face counts [1] | none (built in code) | gap | - | No flat/bevel corner option in the fillet spec (corner patches only). |
| `TestSmRemoveFillet` :272 box 10^3, fillet r1 all edges (>6 faces), RemoveFillet succeeds [1] | none (built in code) | missing | - | Kernel::delete_faces (heal) on the blend faces: assert back to 6 faces and volume 1000 (SMLib asserts only the status). |

Totals: covered 0, partial 0, missing 6, gap 3, n/a 0

### TestSmErrorCodes.cpp

Status-code and NULL-argument contracts of the C API, plus SMLib virtual-geometry status propagation. All n/a.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmInvalidInputCodes` :201 23 NULL/degenerate-argument calls return SM_ERR_INVALID_INPUT [23] | none (built in code) | n/a | - | C API. |
| `TestSmInvalidInputCodes` :201 closest-point (all/one) status propagation via a subclassed SmLine, 4 outcomes x 2 calls [8] | none (built in code) | n/a | - | SMLib virtual-geometry status plumbing. |
| `TestIntersectionOutputStatus` :142 curve-surface/face/brep output status and array contents on solver failure, 6 cases x 3 wrappers (14 combos) [14] | none (built in code) | n/a | - |  |

Totals: covered 0, partial 0, missing 0, gap 0, n/a 45

### TestSmGeneral.cpp

Transforms, queries (closest point, find edge/face), PolyBrep transforms, curvature helper, STEP derivative checks, ownership/cleanup contracts. Geometric values: closest point, find-edge points, principal-curvature tensor values, finite-difference derivative checks.

| Item (function or case, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| `TestSmTransform` :398 box rotated 45 deg about y [1] | none (built in code) | n/a | - | Success only (Kernel::transform). |
| `TestSmTransform` :398 line translated (1,2,3): ends (1,2,3),(3,2,3); surface patch translated: midpoint shifted [2] | none (built in code) | n/a | - | Below the Kernel trait (Curve::transformed, Surface::transformed). |
| `TestSmScale` :462 box 5^3 at (-10,10,0) scaled (1,2,4) about the origin and about (-10,10,0) [2] | none (built in code) | gap | - | Kernel::scale is uniform; no non-uniform B-rep scaling. |
| `TestSmScale` :462 circle: uniform scale 2 -> centre (4,0,0) r2; scale 0.5 about (1,0,0) -> (2.5,0,0) r1; non-uniform scale of a circle rejected and unchanged [3] | none (built in code) | n/a | - | Curve-level. |
| `TestSmRotate` :522 box rotated 45 deg about z; circle's axis rotated 45 deg about y -> (sqrt.5,0,sqrt.5) [2] | none (built in code) | n/a | - |  |
| `TestSmTranslate` :560 box translate; translating an owned edge curve / face surface / face rejected (3); circle translate; PolyBrep translate/rotate(31 deg)/scale(2,3,4) with normal transform rules; zero/negative/NaN/inf PolyBrep scales rejected (4) [10] | none (built in code) | n/a | - | Ownership guards and SMLib PolyBrep object model. |
| `TestSmGetClosestPoint` :707 box 10^3, query (5,5,15): distance within [4,6] (true 5) [1] | none (built in code) | missing | - | Kernel::min_distance(Point, Body) = 5. |
| `TestSmGetEdges` :736 box has >=12 edges; `TestSmGetFaces` :763 exactly 6 faces [2] | none (built in code) | missing | - | Kernel::edges/faces. Trivial. |
| `TestSmFindEdge` :790 five probe points on box edge midpoints: edge index and parameter, point on the edge within 1e-6 [5] | none (built in code) | missing | - | edges() + min_distance(Point, Edge). Values in source. |
| `TestSmFindFaces` :830 point (5,5,10) finds a face [1] | none (built in code) | missing | - | faces() + min_distance(Point, Face) or classify. |
| `TestSmPolyBrepMassPropertiesWinding` :862 tessellated box (1,2,3)+(2,4,6): reversed-winding mesh has negative signed volume, manifold; mass props identical (volume 48) [1] | none (built in code) | n/a | - | SMLib PolyBrep. |
| `TestSmPrincipalDirectionPairing` :926 from hand-fed derivatives Du=(1,0,0), Dv=(1,1,0), Duu,Duv,Dvv = (25,-35,49)/169 z: K=0, H=.5, K1=1, K2=0, E,F,G=(1,1,2), L,M,N, directions (5,-12,0)/13 and (12,5,0)/13, normal-section curvatures [1] | none (built in code) | gap | - | No curvature/principal-direction API in the kernel (native::surface has metric only). Table of expected values in source. |
| `TestSmConeSTEPCrossDerivative` :981 cone (placement default, 1.0, 2.0, 3.0): Duv vs finite difference of Du, 4 u x 3 v points (1e-5 step, rel 1e-6) [12] | none (built in code) | missing | - | Re-targeted: native::surface Surface::d2 (suv) vs a difference of d1, in the kernel's own parameter units (SMLib's STEP degrees are n/a). 12 grid points. |
| `TestSmEdgeCurveOwnership` :1018 SetCurve ownership transfer with/without notification [2] | none (built in code) | n/a | - |  |
| `TestSmPlanarFacesFailureCleanup` :1095 open 2-segment loop: CreatePlanarFaces fails, result NULL [1] | none (built in code) | n/a | - |  |
| `TestSmCurveSweepFailureCleanup` :1131 open profile segment with caps TRUE fails; closed circle profile with a scale law succeeds [2] | none (built in code) | missing | - | sweep_with: open profile + cap request must error; circle with a scale law succeeds. Scale law is not in SweepControl: the success half may be a gap. |
| `TestSmCurveSweepFailureCleanup` :1131 scale curve/path/boundary curves unchanged; FromFaces/FromEdges wrappers: caps TRUE fails, FALSE succeeds (2x2) [6] | none (built in code) | n/a | - |  |
| `TestSmSTEPDerivativeUnits` :1334 cylinder r2, cone (1,2,3), sphere r2, torus 5/1, surface of revolution of a cubic generator: derivatives of order 1..3 (all mixed) vs central differences at u = 20,110,250 deg; unused slots untouched [15] | none (built in code) | missing | - | Re-targeted: d1 and d2 for 5 surfaces x 3 u via Surface::d1/d2. Order-3 derivatives need new API (gap, folded in); STEP degree units n/a. |
| `TestSmSTEPDerivativesExample` (TestSmSTEPDerivativesExample.cpp:12 -> Examples/SMLib/step_derivatives.cpp) same 5 surfaces at one point each plus cylinder r10 at (30 deg, 2): P=(10cos,10sin,2), Du per degree, Dv=(0,0,1); STEP->NURBS uv conversion same point [5] | none (built in code) | missing | - | Same checks as above at other points; the degree-unit and uv conversion halves are n/a. |
| `TestSmStatusAssertMacros` :1499 AERS/AERN/AER macros; DefeatureAndRebuild on empty brep asserts; FindLoopCurveOrientations with no curves fails [3] | none (built in code) | n/a | - |  |

Totals: covered 0, partial 0, missing 43, gap 3, n/a 31

### Conversion candidates

Shape of the source for a generator: the SM_API tests are hand-written C++ with fixtures built in code and assertions interleaved with API-contract checks (ownership, NULL, sentinel pointers). There are no `TestFiles` inputs and no repeated block like `my_test_model(...)` to pattern-match. Parsing the C++ is not practical. The regular parts are literal tables inside the source (ShellCase 4 rows, CapCase 4 rows, `TestMergeBrepsOperation` 4 calls, `sm_ExpectCensus` 4 calls, the 54-row helix table, 5 find-edge probes, 6 distance asserts, CylindricalBox angle list). Recommended form: one new module `tools/smlib/sm_api.py` carrying an explicit, hand-ported case table (each row citing its TestSmBrep.cpp line) and emitting `tests/smlib_sm_api.rs` through the existing driver (as `ssi_analytic.py` does with its `PRELUDE`). All expected values below are in the C++ source or a one-line closed form. No oracle subcommand is needed for any of them; the oracle (`boolean`, `offset`, `shell`, `chamfer`, `sweep`, `tess` take `.smb` or `box:l:w:h:x:y:z` inputs) is optional for the success-only cases.

Ranked by (asserted outcomes x kernel relevance):

1. **Boolean/shell/merge volumes** (about 21 values). TestSmMergeBreps (A=10^3 at 0, B=4x8x6 at (8,6,3): union 1144, intersection 48, difference 144 = B - A [the LAST list entry minus the earlier ones]; three 10^3 boxes at 0,(5,5,5),(8,8,8): 2532; list difference [2^3 at (1,1,1.5), B, A]: 944; tol rel 2e-4); TestSmShellBrepFull (box 10^3, thickness 1 outward sharp: opening face 584, duplicate face list 584, no opening 728, inward -1: 488; abs 1e-5); BooleanBoundsExample (plate 12x12x1.5 minus 16 cylinders r0.75: 173.5885, bbox (0,0,0)-(12,12,1.5) within 1e-4, three bounding-box modes); BooleanWithOptions (box minus inscribed sphere: 1000 - 500*pi/3 = 476.401); NonManifoldBoolean / PiecewiseMerge / BooleanLists (box unions: 1000, 1500, 1500); TestSmBoolean box minus sphere (success, validity). Kernel: `Kernel::boolean`, `shell_with(ShellSpec{outward, sharp})`, `mass_properties`, `bounding_box`. Generator: table-driven, trivial.
2. **Mass properties / areas** (about 12). TestSmMassProperties: 2x6x10 box rotated 30 deg about its centroidal Z, density 2: V 120, A 184, M 240, centroid (1,3,5), Ixx 2560, Iyy 2240, Izz 800, Ixy -277.13 (SMLib's sign; ours is -int(xy), so +138.56 at unit density), then the same about the world origin (parallel axis, `inertia_about`); PrecisePropertySelection face areas (box 52 at (1e5,-2e5,3e5), sphere 100*pi, torus 20*pi^2, annulus 21*pi), disc 25*pi, sphere 4*pi + centroid. Kernel: `transform` + `mass_properties`. Needs the sign convention care; the far-offset box is a precision test.
3. **Intersection values** (about 9). CurveSurface (circle r3 vs tilted plane: points (2,5,0), (8,5,0); coincident circle on z=0 patch), CurveFace (line through a box: faces x=0 and x=10 at (0,5,5), (10,5,5); holed box top face: axis line 0 hits, line at (1,1) exactly 1 point (1,1,10) within 1e-6), CurveBrep (through-line >= 2 points; axis through the hole 0; diagonal (-5,-5,5)-(15,15,5) exactly 2 points: edge-hit de-duplication). Kernel: `ray_hits`, `ssi::curve_surface`, `ssi_coincident`; the coincident-curve cases (circle on top face, line along an edge: 3 cases) are gaps.
4. **Sweep-along-path caps** (6). TestSmSweepAlongPlanarPath: circle r1 along z 10 with capEnds 0..3 (faces 1/2/2/3, manifold only with both caps, start cap z=0, end cap z=10, open volumes 0, capped 10*pi at 1%), the two-segment path (0,0,0)-(0,0,10)-(5,0,15): volume pi*(10+sqrt 50) at 2%, plus rectangle 2x1 along 20 (volume 40). Also TestSmCurveSweep (cylinder 40*pi). Kernel: `sweep_with`. The kink-volume depends on the kernel's corner treatment (SMLib's 2% tolerance); compare with care.
5. **Distance and point queries** (13). BrepDistance (3.0, symmetric, touching 0, crossing 0, inside 3.0), GetClosestPoint (5), FindEdge (5 midpoints), FindFaces. Kernel: `min_distance` (Point/Edge/Face/Body targets). Easy.
6. **Closed-form modelling checks for success-only calls** (about 20): linear sweeps (half-disc V 1570.8, cylinder 3141.6, plane sheet 44.72), revolves (areas 400*pi, 9869.6, 314.16, 175.62, 256.3), CurveSweep 40*pi, circular fillet of the 5x10x15 box with spherical corners (V 743.7293), RemoveFillet round trip (6 faces, V 1000), OffsetSurface failures (sphere r10 by -20 and torus R10 r2 by -5 must error), CylindricalBox equal angles, box-sphere boolean, HealBrep, Import/Export round trip (`write_body`/`read_body`, counts + volume 1000), TessellateBoundaries (12 edges x 2 points), Tessellate params (finer chord, more vertices). SMLib asserts only success; adding the closed form is our strengthening.
7. **Derivative consistency** (27 grid points): ConeSTEPCrossDerivative (12), STEPDerivativeUnits (15 = 5 surfaces x 3 u) and the STEP example. Re-targeted to `Surface::d1/d2` against differences in the kernel's own units. Orders up to 2 only.

Kernel gaps exposed by this suite (feature requests, not tests): project a curve onto a body and imprint/trim (ProjectAndTrim, ProjectCurve; 4 cases), curve-on-surface operations (DropCurveToSrf, ProjectCurve to surface, LiftUVCurve: 6 cases), curve offset (3), knot removal, make-curves-compatible, interpolating a surface through a point grid or scattered points, trim a surface by projected curves (TrimProjectParallel), ruled/linear skin (2 volumes: 1130*pi/3 and 295*pi), swung primitive, bare-surface fillet and fillet preview, bevel corners, non-uniform B-rep scale, surface curvature from derivatives (TestSmPrincipalDirectionPairing, exact values in source), third-order surface derivatives, face area moments with thickness, partial tessellation with a failure report, non-manifold MERGE results and nested solid-in-void census, curve overlaps with faces (3 cases). 34 cases.

Not worth converting (n/a, about 370 of the cases): C API argument validation, ownership/handle/sentinel contracts, copy semantics, SMLib status propagation through subclassed geometry, region/infinite-region topology, PolyBrep container behaviour, the 54-row helix table (pure INVALID_INPUT checks; the kernel has no helix constructor).




### test_omni_solid.py

6484 lines, 38 `unittest.TestCase` classes, **400 test methods** (counted with `ast`; no pytest fixtures, no `parametrize`; loops with `self.subTest` give the extra cases). Run with plain `python -m unittest` against the `_omni_solid` pybind module (`source/SmPyLib/src/SmPy*.cpp` wrap the C `SmApi*` entry points: Primitives, Curves, Surfaces, Booleans, Sweeps, Fillets, Offset, Stitching, Intersectors, Queries, Transforms, Tessellation, PolyBrep, FileIo, Heal, CutProjectTrim, Usd).

**Not converted (verified).** `tools/smlib/*.py` list SOURCE files only from `tests/prog_test/src` (smTopoTest, smFilletTest, smAdvancedTest, smSweepTest, smCurveTest, smSurfaceTest, smTessTest, smMergeTest, smOffsetTest). Nothing in `tools/` or `crates/cadrs_kernel/tests/*.rs` mentions `SmPy`, `omni_solid` or `test_omni`, and none of this file's asserted values (e.g. 1144/48/144, 2532, 944, 10*pi, 40*pi) appears in a generated suite. So covered = 0, partial = 0.

**How USD dependence is marked.** There are no skip markers or fixtures for it: `import _omni_solid as sm` is unconditional and `sm.usd` is always bound (SmPyMain.cpp:253 `bind_usd(m)`). USD tests are identified only by use of `sm.usd.*` (71 `sm.usd` references in 28 tests), `.usda` temp files, and the `_inject_xform_ops` helper (file text, line 30, which avoids `pxr`). USD-dependent tests: all 26 of `TestUsd` (3 of them, 5635/5681/5706, only use helpers `_tessellate_moved_reference`, etc.) plus 5 in `TestErrors` = **31**. Subprocess scripts (`_IMPORT_BREPS_SCRIPT` 5384, `TestBorrowed...` 3697+, 4108+) are embedded Python strings run in a child process (crash-safety checks). The only skip markers in the file: `skipUnless(np is not None)` at 789 (numpy), and `skipTest` at 1079, 1083 (legacy `.smb` fixture) and 1102 (`.smp` fixture).

**Fixtures.** Almost every test builds its own geometry with `sm.create_*` (`create_box` appears in 166 tests / 211 calls); only 3 use files: `TestFiles/pt_TestFiles/box_with_large_fillet.smb` (present in the checkout, test 1075) and `TestFiles/bz_TestFiles/Regressions2009/10.Wilson/SmallBox.smp` (**absent** from the checkout: `bz_TestFiles` does not exist, so test 1091 always skips). USD tests use `TestFiles/brep_validator/*.usda`, `usd_TestFiles/RevitWallHealerCrash.usda` and generated temp files.

**Character of the suite.** It tests the Python binding, not SMLib's numerics: 226 of 400 tests have no `assertAlmostEqual`/numeric `assertEqual` at all; most are smoke (`isinstance(r, sm.Brep)`, `len(r.faces()) > 0`, `isinstance(x, list)`). Only about 44 assert volume/area/mass values (class counts: Queries 12, Sweeps 8, TopologyAccessors 6, NativeBrepFileIo 4, Transforms 4, PolyBrepMass 2, OffsetShell 2, TrimmedSurfaces 2, others 1). Many others are ownership/lifetime/argument-validation checks of the pybind layer (C handles consumed or borrowed, `SM_ERR_INVALID_INPUT` strings), which have no Rust analogue.

Status key: **missing** = kernel has the operation (API given), not converted; **gap** = kernel lacks it; **n/a** = Python-binding plumbing (P), USD-only (U), model absent (A). "smoke" = asserts only type / `faces()>0` / list-ness.

| Item (class, file:line) | Fixtures | Status | Our test(s) | Notes |
|---|---|---|---|---|
| TestVec3 60,66; TestVec2 72,77 (4) | none | n/a (P) | - | Python value types, repr |
| TestEnums 82-94 (3); TestNewEnums 5367,5372 (2) | none | n/a (P) | - | enum members exist / `int(CapEnds)` values |
| TestStableApiExcludesDevSurface 1276 (1); TestDiscoverability 6177-6238 (4) | none | n/a (P) | - | `hasattr(sm,...)`, submodule aliases, module docstring |
| TestCapEndsAndOffsetType 6384-6420 (5: cap_ends_values, offset_type_values, enum_members_not_module_level, draft_sweep_rejects_int_cap_ends, create_offset_profile_rejects_int_offset_type) | none | n/a (P) | - | enum typing |
| TestCapEndsAndOffsetType 6421,6427,6433,6442 (4) | none | missing | - | default caps = BOTH for draft_sweep(circle r5,h10,5deg), taper_extrude(80deg), sweep_along_planar_path, offset_profile band; smoke (`faces>0`). Kernel: `draft`/`extrude_with`/`sweep_with`/`offset` |
| TestPrimitives 105-186: create_box 106, pyramid 143 (2) | none | missing | - | box 6F/12E/8V; pyramid(10,10) 5F/5V manifold. `Kernel::extrude` of a rect / draw harness `Shape` |
| TestPrimitives 113,118,123,128,133,138,152,157,167,177,182 (11: sphere, cone, cylinder, torus, plane, planar_circle, cylindrical_box, partial_sphere, partial_cone, partial_cylinder, partial_torus) | none | missing | - | smoke (`faces>0`; plane `faces==1`). Kernel: `revolve`/`cylindrical_box`/draw harness primitives (smlib_primitives already builds these from smSweepTest) |
| TestPrimitives 162 sphere_no_pole, 172 cone_elliptic_ends (2) | none | gap | - | SMLib-specific parameterisations (poleless sphere, elliptic-end cone); smoke only |
| TestPrimitives 187 create_blend_primitive_exposed (1) | none | n/a (P) | - | `callable(getattr(sm,...))` |
| TestSkinPrimitiveGeometry 367 (1) | none | missing | - | **Closed-form**: skin of 181 sections x 4 line segments, degree 1, capped; volume = 1000*(2+cos(pi/360)) (delta 5e-3), tight bbox from section points (1e-5). Kernel: `loft_with` (degree-1 piecewise). Generator-free, one hand test |
| TestSkinPrimitiveInputCurvesUnchanged 194-365 (7); TestOffsetProfileInputOwnership 411-566 (7); TestSweepInputCurvesUnchanged 2946-3050 (9); TestBorrowedGeometryHandleLifetime 3693-4192 (6) | none | n/a (P) | - | Python curve/face handles survive or are consumed by calls; child-process crash probes; no geometric outcome |
| TestTessellateBoundaries 568 (1) | none | missing | - | box(10): counts `[2]*12`, 24 points, edges == `box.edges()`. Kernel: `native::tess` edge sampling |
| TestTessellateBoundaries 580 (1) | none | missing | - | sphere r5: coarser angle gives fewer points; `sum(counts)==len(points)` |
| TestTessellateBoundaries 588 (1) | none | n/a (P) | - | invalid args raise |
| TestTessellate 606,613,626 (3) | none | missing | - | box -> polybrep faces/verts >0; sphere r5 mesh manifold; finer angle gives more faces (cylinder r5 h10, sphere r5). `Kernel::tessellate` (watertight check) |
| TestTessellate 672 failure_reporting, 685 open_mesh_statistics (2) | none | gap | - | tessellation report dict (`lamina_edge_count==4`, `spine_edge_count==0`, failures list) has no kernel equivalent |
| TestTessellate 642,654,696,710,717,727,734,750,790 (9) | none | n/a (P) | - | kwargs defaults/dict, `to_mesh_arrays`, numpy dtype, method vs function form |
| TestNativeBrepFileIo 928,944,958 (3); 1114 (1) | tmp `.smb`, unicode path | gap | - | write then read SMLib native BRep (ASCII/binary/unicode path), face_count and volume equal. `native::smb` only reads (`read_brep`, `read_part`), no writer |
| TestNativeBrepFileIo 969,997,1062 (3) | tmp files | n/a (P) | - | explicit-selector / format-mismatch / part-container rejection without crash |
| TestNativeBrepFileIo 1046,1055 (2) | tmp empty `.smp` v42/v40 | missing | - | `read_part` of empty containers == `([],[],[],[])`. `smb::read_part` |
| TestNativeBrepFileIo 1075 (1) | `pt_TestFiles/box_with_large_fillet.smb` (present) | missing | - | `read_brep` faces>0, edges>0. `smb::read_brep`; not referenced by any suite |
| TestNativeBrepFileIo 1091 (1) | `bz_TestFiles/.../SmallBox.smp` (absent) | n/a (A) | - | model not in checkout; always skips |
| TestSmApiExtraBindings 1132,1136,1178,1181,1184 (5) | none | n/a (P) | - | `get_faces_via_api` len equals `faces()`, enum/Axis2Placement construct |
| TestSmApiExtraBindings 1140 (1) | none | missing | - | box mesh volume>0 (weak) |
| TestSmApiExtraBindings 1146,1162 (2) | none | gap | - | `poly_boolean_difference` on PolyBrep meshes; kernel has no mesh-boolean API (only `mesh_solid`) |
| TestPolyBrepMassProperties 1203 (1) | none | missing | - | **Closed form** box (1,2,3)+(2,4,6), density 2.5: V=48, A, mass, centroid (2,4,6), centroidal and origin-(0,0,0) inertia/products to 1e-9, linear in density. Kernel `mass_properties` on the exact body (tessellated box is exact) |
| TestPolyBrepMassProperties 1244 (1) | none | gap | - | mesh vs exact sphere r10 within 2e-3 / 5e-3: mesh mass props, no kernel API |
| TestPolyBrepMassProperties 1258 (1) | none | n/a (P) | - | invalid input error text |
| TestCurves 1403,1407,1526,1702,1724,1728,1732,1737,1742,1747 (10) | none | missing | - | smoke: line segment, circle, arc, curve through points, line, ellipse, `create_rectangle` (4 segs), `create_regular_polygon(6)` (6 segs), approx, interp. Kernel: `native::curve::Curve::line`, `nurbs::NurbsCurve::interpolate` |
| TestCurves 1411 (1) | none | missing | - | analytic Circle accessors (center/radius/normal/axes) across 5 (center,radius) incl. 1e-3 and 1e4, delta 1e-9. `native::curve` circle |
| TestCurves 1760,1790 (2) | none | missing | - | interp through points: passes through points (1e-6) for UNIFORM/CHORD/CENTRIPETAL; parameterisations differ >1e-4. `NurbsCurve::interpolate` + `chord_params` (uniform/centripetal must be built by hand) |
| TestCurves 1902 (1) | none | missing | - | `evaluate_curve` returns dict (smoke) |
| TestCurves 1556,1572,1583,1610,1627,1683 (6) | none | gap | - | `create_helix` (constant/conical radius, handedness, endpoint tangents, invalid-input matrix, boundary acceptance, tolerance enforcement with SM_ERR_NOT_WITHIN_TOLERANCE): no standalone helix curve constructor in kernel (helix only fitted inside sweep path code) |
| TestCurves 1707 (1) | none | gap | - | `remove_curve_knots` (shape preserved to 1e-7): `nurbs` has insert/elevate only |
| TestCurves 1907 evaluate_continuity (1) | none | gap | - | `evaluate_continuity` classification C0/G1/C1/C_INFINITY: no kernel equivalent |
| TestCurves 1944 (1) | none | gap | - | `convert_to_lines_and_arcs`: no polyline/arc fitting op |
| TestCurves 1469,1477,1508,1516,1815,1824,1829,1841,1867,1883,1912,1920,1933 (13) | none | n/a (P) | - | determinism, `SM_ERR_INVALID_INPUT` validation (radius <= 1e-12, <3 points, coincident points, bad enum), keyword-only args, registered enum members |
| TestSurfaces 1966-2040 (11: from_points, ruled, revolution, from_corner_points, from_ordered_points, extrude, sweep_surface, skin_surface, evaluate_point (5,5 centre ~5.0), normal, derivatives) | none | missing | - | smoke (isinstance Surface, 3-tuple lengths). Kernel: `native::nurbs::NurbsSurface::through_curves`, `surface::Surface::{eval,normal,d1}` |
| TestBooleans 2107,2113,2119,2125 (4) | none | missing | - | boxes (0,0,0,10)/(5,5,5,10): union faces>6, diff/intersect faces>0, generic UNION. Smoke; `Kernel::boolean` |
| TestBooleans 2152 (1) | none | missing | - | **Values**: box(0,0,0,10) vs box(8,6,3, 4x8x6): union 1144, intersection 48, difference (second-first) 144; MERGE non-manifold is a gap (non-manifold output) |
| TestBooleans 2182,2197 (2) | none | missing | - | **Values**: 3-box union 2532 (inclusion-exclusion); 3-box difference 944 + bbox (0..10)^3 |
| TestBooleans 2217 (1) | none | missing | - | `boolean_with_options` defaults: faces>0 |
| TestBooleans 2223,2231,2239 (3) | none | gap | - | cookie-cutter / post-process / keep-other-brep options: no kernel options |
| TestBooleans 2247,2266,2274,2282 (4) | none | missing | - | `boolean_2d` of two coplanar circles r10 at x=0,8: UNION/INTERSECTION/DIFFERENCE/EXCLUSIVE_OR, smoke. Kernel sheet booleans (draw harness) |
| TestBooleans 2290,2300,2310,2320 (4) | none | missing | - | **Values**: rectangle pairs, per op: area + bbox. Partial overlap 150/50/50/100; disjoint 200/0(none)/100/200; first subset 100/16/0(none)/84; second subset 100/16/84/84. 16 asserted areas. Good generator candidate (table-driven) |
| TestBooleans 2130,2133,2138,2145,2255 (5) | none | n/a (P) | - | merge input validation / not consumed; non-coplanar error text |
| TestBooleans 2330,2339 (2), 2346 (1), 2354 (1), 2361 (1), 2368 (1), 2375 (1) | none | gap | - | `boolean_lists` (with surfaces), CSG tree evaluate, `boolean_merge`, `boolean_with_curves` (returns intersection edges), `non_manifold_boolean`, `piecewise_merge`: all smoke; kernel has only manifold union/subtract/intersect (and `cells`) |
| TestSweeps 2513,2519,2525,2532,2681 (5) | none | missing | - | linear_sweep(circle r5,10), rotational (line, 360), draft_sweep, pipe_sweep(curve through points, r1), taper_extrude(80deg): `faces>0`. Kernel `extrude`/`revolve`/`draft`/`sweep_with` |
| TestSweeps 2692 (1) | none | missing | - | sweep circle r1 along z-line 10, 4 cap modes: face count 1+caps, cap faces at z=0/10, manifold only with both caps, volume 10*pi (0.1*pi) else 0 |
| TestSweeps 2744,2771,2809,2828,2860 (5) | none | missing | - | **Values**: two-segment path 4F/5E/3V vol pi*(10+sqrt50) (2 %); nested circles parity vol 60*pi, 4 cap modes; open square profile stays sheet (vol 0); tangent line-arc-line 5F vol pi*9*(33+2*pi) (1 %); closed mitered path 4F vol 40*pi, equal for all caps. `sweep_with` with multi-segment path |
| TestSweeps 2911,2918 (2) | none | missing | - | `curve_sweep` of circle along curve (smoke) and along straight line with cap options |
| TestSweeps 2539,2616 (2) | none | gap | - | pipe_sweep along `create_helix` (R2,H10,5 turns, pipe r0.3): 3 faces, cap area pi*0.09; low-turn helix topology. Needs the helix constructor (gap) though the sweep itself exists |
| TestSweeps 2396,2421,2435,2451,2462,2475 (6) | none | gap | - | `non_manifold_linear/rotational_sweep` of box vertices (counts (faces, edges+1, verts+1), whole-brep (16,30,16)/(14,28,16)) and their rejects: no non-manifold sweep |
| TestSweeps 2897,2904 (2) | none | gap | - | `*_sweep_with_repetitions`: no repetition parameter on `ExtrudeSpec`/`RevolveSpec` |
| TestSweeps 2687 (1) | none | n/a (P) | - | `end_caps=` kwarg rejected |
| TestFillets 3052,3057,3062,3068,3083 (5) | none | missing | - | box(10): circular fillet r1, chamfer 1, fillet_edges (xsect 1), variable radius 0.5..2.0, per-edge radii [1,2]: all `faces>6`. Kernel `fillet`, `chamfer`, `fillet_variable`, `fillet_with` |
| TestFillets 3075 (1) | none | gap | - | `remove_fillet` (undo) |
| TestFillets 3090 (1), 3098 (1) | none | gap | - | `surface_surface_fillet` of two patches r2,2; `fillet_preview` returns surfaces |
| TestOffsetShell 3121 (1) | none | missing | - | `offset_brep(box,1)` smoke. Kernel `offset` |
| TestOffsetShell 3127 (1) | none | missing | - | **Values** 8 subcases (dist +-1, create_solid, select top face): counts (11|12, 24, 16), volume 584/728 (+1) and 424/488 (-1), exact bbox. Kernel `shell_with` |
| TestOffsetShell 3172 (1) | none | missing | - | **Values** cylinder r31 h86 wall +-3: volume pi*((r+w)^2(h+w)-r^2 h) and inward analogue, rel 1e-5 |
| TestOffsetShell 3198,3207 (2) | none | n/a (P) | - | failure leaves source intact; foreign face rejected |
| TestStitching 3224,3231,3254,3247 (4: stitch_into_solid, stitch_into_shell, stitch_advanced, unify_normals) | none | missing | - | smoke result types; kernel `sew_solid`, `heal` (`HealOp`), `native::unify` |
| TestStitching 3238 (1) | none | n/a (P) | - | return-flag echo |
| TestIntersectors 3266-3307 (6: breps, brep-plane (curves>0), curves, surfaces, curve-surface, curve-brep) | none | missing | - | smoke (`list` types). Kernel `native::ssi*`, `ssi_cci`, `split`/section |
| TestTransforms 3427,3438,3443,3449 (4) | none | missing | - | translate then closest-point dist 0; rotate 90 keeps 6 faces; `scale(box,(2,2,2))` volume 8000 (+-1); reflection (negative scales) volume 192*abs(prod) +-0.01, manifold. `transform`/`scale`/`Motion` (reflection supported) |
| TestTransforms 3465,3478,3501 (3) | none | missing | - | in-place transform of standalone circle/line (centre, radius, endpoints); negative scale on exact roots. `native::curve::Curve::transformed` |
| TestTransforms 3594 (1) | none | gap | - | PolyBrep transform contract (mesh object) |
| TestTransforms 3433,3515,3553,3562,3623,3654 (6) | none | n/a (P) | - | root-transform contract, dispatch/rejection of owned geometry, analytic-surface scale rejection |
| TestQueries 4195,4201,4208 (3) | none | missing | - | box closest point z=10, dist 5; line closest (5,0); surface closest. `min_distance`, `Curve::closest`, `Surface::project` |
| TestQueries 4223,4247,4277,4293,4315 (5) | none | gap | - | SMLib returns all seam/periodic uv aliases (cylinder >= 2, torus all), only selected point, ignores face trims, natural domain on filleted corner: kernel `project` returns one (u,v) |
| TestQueries 4356,4360,4410,4423 (4) | none | missing | - | is_manifold_solid box true / plane false; volume 1000 (0.1), sphere r5 4/3 pi r^3 (1.0). `body_class`, `mass_properties` |
| TestQueries 4364,4378,4390,4398 (4) | none | gap | - | `material_census` (solid/void counts) for box, hollow box (void), disjoint solids, nested solid in cavity: kernel has `solid_count` only |
| TestQueries 4429,4436,4443,4450,4457 (5) | none | missing | - | `brep_relationship`: separate / touching / interpenetrating / a_contains_b / b_contains_a for unit/10-boxes. From `min_distance` + `classify` + `overlaps`/`joins` |
| TestQueries 4468,4478,4483,4488 + 4473 (5) | none | missing | - | **Values** `brep_distance`: 3.0 (gap between unit boxes), symmetric, touching 0, interpenetrating 0, enclosed wall clearance 3.0 (1e-4). `min_distance` |
| TestQueries 4507,4524,4577,4581,4596,4622,4665 (7) | none | missing | - | **Values** box(10): V1000, A600, mass1000; density scaling x3 of mass and inertia; sphere/torus area (4 pi r^2, 4 pi^2 r) rel 1e-3 and equals sum of face areas; disk sheet area; L-shaped union V3000 centroid (25/3,5,25/3); box 2x4x6 inertia about origin (parallel axis, 1e-3). `mass_properties`, face area |
| TestQueries 4369,4373,4463,4496,4502,4415,4546,4563,4605,4613 (10) | none | n/a (P) | - | method vs function equality, submodule alias, invalid accuracy/density errors, non-solid raises |
| TestTopologyAccessors 4764-5236 (35: bbox, centre (6,7,8), volume 1000, counts 6/12, face bbox/area 100/interior point/classify_uv/closest/centroid (+sphere with poles)/surface/uv_domain/topology/boundary_loops (annulus from `boolean_2d`)/seam loop/point_at_uv/normal unit/outward normal; reversed-cap cylinder area > 0 (2 tests); edge length [3,7,10] of box 10x7x3, curve/endpoints/faces (2)/point_at/parameter_range/classify_parameter/tangent/closest; vertex point/edges 3/faces 3) | none | missing | - | all on boxes/cylinder/sphere; asserts are mostly bounds checks and counts; kernel `faces`/`edges`/`vertices`/`adjacent_faces`/`bounding_box`/`face_normals_at`/`min_distance`. Cheap with a box fixture |
| TestTopologyAccessors 4791,4799,5109,5208,5232,5237 (6) | none | n/a (P) | - | copy independence, `.brep()` back-reference identity, repr |
| TestBrepOps 5249,5255,5261 (3) | none | missing | - | `cut(box, z=5 plane)` faces>0, `project_brep_onto_plane` curves>0, `create_silhouette_curves` list. Kernel `split`, `project`, `silhouettes` |
| TestTrimmedSurfaces 5271, 5291 (2) | none | missing | - | `create_planar_faces([circle])` faces>0; `trim_project_parallel` of 25x25 patch KEEP_POINT faces>0. `face_from_surface`/`split` |
| TestTrimmedSurfaces 5277,5300,5310 (3) | none | n/a (P) | - | error cases |
| TestHeal 5340 (1) | none | missing | - | `heal_brep(box)` keeps 6 faces; `Kernel::heal` |
| TestClosestPointAll 5350,5357 (2) | none | gap | - | all closest points/params/dists (multiple results) for box (5,5,15) and circle centre-offset point; kernel returns one |
| TestCurveMethods 6240-6291 (9: parameter range, endpoints, midpoint (5,0,0), length 10, circle length 2 pi 5 (1e-3), is_closed x2, is_planar x2) | none | missing | - | native curve accessors (`Curve::length`, `point`); trivial |
| TestCurveMethods 6288,6292 (2) | none | n/a (P) | - | repr, dict keys |
| TestSurfaceMethods 6307-6378 (6: uv domain, evaluate z=0, unit normal, derivative shape, planar, periodic) | none | missing | - | `Surface::{param_box,eval,normal,d1,kind,u_periodic}` |
| TestSurfaceMethods 6325,6374 (2) | none | n/a (P) | - | consistent domain accessor aliases, repr |
| TestDegenerateRadiiHardening 6472,6478 (2) | none | missing | - | sphere radius 0 and torus minor radius 0 must raise (error not crash). Kernel `revolve` with degenerate profile returns error |
| TestDegenerateRadiiHardening 6458,6465 (2) | none | gap | - | spindle torus (minor >= major) and negative-radius sphere are valid solids with `faces>0` |
| TestErrors 6087,6107,6114 (3) | none | n/a (P) | - | status-code strings (SM_ERR_INVALID_INPUT 1007) for None args, zero rotation axis, bad tuple size |
| TestErrors 6118,6122,6127,6138,6153 (5) | none | n/a (U) | - | all call `sm.usd.import_breps("/nonexistent/file.usda")`; error text contains `SmApiUsdImportBreps`, symbolic status, no accumulation |
| TestUsd 5503-6084 (26) | `.usda` temp files; `brep_validator/CubeBrepArray.usda`, `Test_BA_120_Wrong_face_array_size.usda`, `usd_TestFiles/RevitWallHealerCrash.usda` | n/a (U) | - | USD export/import round trip (volume, topology, transforms, NURBS preservation, unbounded plane ranges), `tessellate_file` (mesh per brep, materials, visibility/purpose, partial failures), `append_breps`, malformed BrepArray rejection |

Totals: covered 0, partial 0, missing 189, gap 57, n/a 154 (n/a = 122 Python-binding plumbing + 31 USD + 1 absent model file) = 400 tests.

Per-class counts (missing/gap/n-a): Vec3 0/0/2, Vec2 0/0/2, Enums 0/0/3, Primitives 13/2/1, SkinInputCurves 0/0/7, SkinGeometry 1/0/0, OffsetProfileOwnership 0/0/7, TessellateBoundaries 2/0/1, Tessellate 3/2/9, NativeBrepFileIo 3/4/4, SmApiExtraBindings 1/2/5, PolyBrepMass 1/1/1, StableApi 0/0/1, Curves 14/9/13, Surfaces 11/0/0, Booleans 16/10/5, Sweeps 13/10/1, SweepInputCurves 0/0/9, Fillets 5/3/0, OffsetShell 3/0/2, Stitching 4/0/1, Intersectors 6/0/0, Transforms 7/1/6, Borrowed 0/0/6, Queries 24/9/10, TopologyAccessors 35/0/6, BrepOps 3/0/0, TrimmedSurfaces 2/0/3, Heal 1/0/0, ClosestPointAll 0/2/0, NewEnums 0/0/2, Usd 0/0/26, Errors 0/0/8, Discoverability 0/0/4, CurveMethods 9/0/2, SurfaceMethods 6/0/2, CapEnds 4/0/5, DegenerateRadii 2/2/0.

### Regularity for a Python->Rust generator (ast analysis)

Frequency of call patterns (tests containing the pattern, of 400): primitive creation 215 (create_box alone 166, 211 call sites); topology accessors (`faces()/edges()/vertices()/*_count`) 136; curve creation 104; volume/area/mass 51; assertRaises 51; booleans 30 (+10 `boolean_2d`); tessellate 34; closest/distance/relationship 29; sweeps 31; surface creation 20; transforms 17; fillet 14; file io 11; stitch/heal 9; shell/offset 7; intersect 7. Top calls: `create_box` 211, `Brep` 77, `usd` 71, `create_line_segment` 54, `create_circle` 42, `BooleanOp2D` 31, `tessellate` 24, `create_sphere` 22, `CapEnds` 22.

Verdict: **not worth a general generator.** The file is hand-written unittest code with helpers (`_assert_volume`, `_counts`, `_owner_snapshot`, `_assert_curve_unchanged`), `subTest` loops, lambdas and nested helper scripts; `ast` can extract names but the assertions are heterogeneous. Two thirds of the "missing" tests are smoke checks that add nothing beyond what the 16 SMLib-C++ suites already assert. A small hand-written module, `tests/smlib_python_values.rs` (about 25 tests), captures nearly all the value-bearing content. A table-driven helper (`fn check(volume, area, bbox, counts)` on a body) suffices, because every value is a literal in the source. If any generation is done, only these tables are regular:
- booleans 2D rectangle tables (2290-2330): 4 tests x 4 ops (op, area, bbox) as `((BooleanOp2D, area, bbox|None))` tuples, parsable with `ast.literal_eval`-style walking;
- `merge_breps` volumes (2152-2215): 3 tests, tuples of (op, volume);
- `shell_brep` table (3127): dict literal of 8 expected (counts, volume, bounds);
- planar-path sweeps (2692-2860): five near-identical `sweep_along_planar_path` patterns with (cap case, faces, solid, volume).
All else (smoke) is a one-line `create_*` + "faces > 0" and should not be converted.

### Conversion candidates

1. **Closed-form value tests** (highest payoff, about 25 tests, about 80 asserted values, no oracle needed). Build by hand with the box/cylinder/sphere helpers and check against the literals in the source: `merge_breps` 1144/48/144, 2532, 944 (Kernel::boolean); 2D rectangle boolean areas (16 values); shell table 8 cases and cylinder shell volumes (`shell_with`); skin volume 1000*(2+cos(pi/360)) (`loft_with`); planar-path sweep volumes pi*(10+sqrt50), 60 pi, 40 pi, 9 pi (33+2 pi), 10 pi (`sweep_with`); mass properties (box V/A/mass, density scaling, L-shape centroid (25/3,5,25/3), origin inertia, sphere/torus areas, disk sheet); box PolyBrep closed form (use exact body); brep_distance 3/0/0/3; brep_relationship five cases; scale 8000, reflection 192. Hand-written, irregular.
2. **Box-based topology / accessor checks** (35 tests): bbox, centre (6,7,8), counts 6/12/8, face area 100, edge lengths [3,7,10], vertex edges/faces 3, `classify_uv`, normals. Cheap on one box fixture via `faces`/`edges`/`vertices`/`adjacent_faces`; low value over existing kernel unit tests.
3. **Smoke-only primitives, surfaces, curves, intersectors, fillets, sweeps, stitching, heal, brep ops** (about 70 tests): `faces() > 0` etc. Convert only as one parametrised "builds without error" test per family; each already has stronger coverage in the smlib_* suites.
4. **Native .smb/.smp reading** (3 tests): `box_with_large_fillet.smb` (file present, unused by any suite) with `smb::read_brep`; empty `.smp` v40/v42 containers with `smb::read_part`. `SmallBox.smp` is absent.
5. **Feature gaps this file exposes** (57 tests): standalone `create_helix` (7 incl. pipe along helix), `remove_curve_knots`, `convert_to_lines_and_arcs`, `evaluate_continuity`, SMLib native writer/round-trip (4), PolyBrep boolean/mass/transform (4), `material_census` (4), multi-valued closest point and seam aliases (7), non-manifold boolean (1) and sweeps (6), `boolean_with_options`/lists/CSG tree/curves (8), `remove_fillet`/`surface_surface_fillet`/`fillet_preview` (3), sweep repetitions (2), tessellation report (2), sphere_no_pole, cone_elliptic_ends, spindle torus, inside-out sphere. The oracle CLI (`tools/smlib_oracle`) does not apply: this file has no C++ counterpart to drive, and every expected value is a literal in the Python.
