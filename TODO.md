# TODO

The focus is **correctness for a release** of cadrs on the native kernel (`cadrs_kernel`, pure
Rust, no OpenCASCADE). New features wait until the release blockers are done.

## 1. Release blockers: correctness

### Real documents rebuild at least as well as on main (the old OpenCASCADE-based release)

All 149 stored documents (173 Part Studios) are rebuilt headlessly with kernel-10 and with main
(the old OpenCASCADE-based release) and compared per feature. Neither crashes or hangs.

| | Feature errors | New vs main |
|---|---|---|
| kernel-10 at `bdf82f3` (first run) | 140 | 63 in 28 Part Studios |
| kernel-10 at `dc8d23f` | ~84 | 4 |
| main (old release) | 106 | — |

Done when no Part Studio has an error that main doesn't also have, and none rebuilds much slower.
Rebuild a single document with `cadrs --document "<name>" --check`.

Fixed (dropped from the list): open sketch loops (d4 desk device v2/v3, d4_desk_device_test, emax,
squish's profile), the open-shell and 3-face booleans, plant_pot's extrude direction, the stroller,
pvc and rianatest blends, Dr Browns cap's Fillet 1 (rounds round a window across the dome's seam)
and the cable organizer's Fillet 2 (full rounds meeting at a corner), references (mate connectors, pibox thicken, t20, trimmer's unnamed
faces), mirror_test's mesh import, squish's Extrude 6 (the fuzzy boolean) and the kids cart and
trimmer.

Still new against main:
- [ ] Dr Browns: the cap's seam through a window (fb11, blocked; see Kernel correctness).
- [ ] Flagged boolean results: volumemaker_f5, shape_type_a_a1, nightlight's Boolean 7.

Differences without errors, still to check:
- [ ] Part counts that differ from main (garbage_cabinet 7 vs 3, TubeAdapter 3 vs 1 from chamfer
  slivers, racingrigrumble 10 vs 8, and others in compare runs).
- [ ] Small volume differences (pibox mount 1802.4 vs 1800.1 mm³, stroller chamfer 1429 vs 762 mm³).
- [ ] Rebuild time: 896 s locally against main's 423 s on gray, before the speed passes of
  2026-10-22/23 (their commit messages: the user documents 358 s to 295 s). Measure again
  against main. pico2, nightlight_sphere, ekgpulse, squish and trimmer were 2–5× slower.
- [ ] Add a document check to the release process. A small headless runner rebuilds every
  document and compares against a saved result. Private documents stay out of the repo; only
  minimal dumped bodies go in as regression fixtures.

### Kernel correctness (independent review 4, 7.45/10 at `b589424`)

- [ ] Fillets and blends:
  - occt_blend: 89 known failures (most "area differs" ones pick another edge than the script's
    explode order, and five expect the reference kernel's own failure);
  - SMLib fillets: 43/100 pass (the sheet inputs' results compared as sheets, on their area);
    group A, the tangent-chain cases whose face pair changes at a vertex (10 cases: 116, 226,
    231, 234, 237, 401, 410, 411, 412, 421), each stops on another limit past the junction (a
    fillet larger than the round it runs over, a rail running off a face, a crease of 1° taken as
    tangent); groups E, G and C are still open;
  - no occt_blend case falls back to the tool-and-boolean blend any more; fillets take about
    1.1× OCCT's time on occt_blend and 1.2× on smlib_fillets.
- [ ] Booleans: 59 occt_boolean known failures (section lengths, sub-shape counts, open shells, areas).
  A boolean now checks that its result's faces don't cross and flags a result that does
  (`take_invalid_results`; `CADRS_BOOL_STRICT=1` makes it an error). Three booleans still
  return a flagged result: volumemaker_f5, shape_type_a_a1 and nightlight's Boolean 7.
- [ ] Offsets: 133 occt_offset known failures; smlib_offset has 4 (offset_04, 05, 07, 08).
- [ ] Remaining known failures: about 486 in all (`tools/occt_known_failures.txt`,
  `tools/smlib_known_failures/`); besides the above they are in feat (47), pipe (27), sewing
  (20), heal (12), thrusection (10), the SMLib ssi suites and a few others.
- [ ] Dr Browns cap: a merged face's new seam can run through one of its holes. The boolean
  merges the pieces of one face back (`unify::merge_group`) and gives the merged face a seam at a
  vertex of its boundary chain; in the cap that put the seam at u = 0, inside the span of a
  window across the dome's seam, and the fillet can't place its cuts (`fillet::open_seam_holes`
  works around it). The seam fix (fb11) makes the cap pass, but it breaks
  `smlib_offset offset_12_offset_bracket2` ("open shell"): offset_12 has a near-tangent
  cylinder, and the tube `offset/round.rs` builds there has a radius 1.6e-3 off. Fix the tube
  first, then land fb11. Earlier tries: cutting the seam at the hole afterwards
  (`cut_seams_at_holes`) and splitting the chain at a clear u each broke offset_12; no seam vertex
  clear of the holes exists in the cap's chain. The inputs are in
  `doc_fixtures/drbrowns_window_{target,tool}.body` (private, outside the repository).
- [ ] `explode` in the test harness should list degenerated edges in the scripts' order. That
  would turn 13 blend topology mismatches into real comparisons.
- [ ] The BRep-text reader: 5 dataset files the reference reads; CFI_5_b12fgg.rle edge count.
- [ ] STEP/IGES: 52 of the 389 dataset files don't read clean (`tests/exchange_corpus.rs`).
- [ ] Fillet speed: 1.11× the reference on occt_blend and 1.17× on smlib_fillet (CPU times, each
  case run by both in turn: the shared machine's load moves either side alone by tens of
  percent); the goal is at or below it. The result's check is about 40% of a local fillet's time.

### Release mechanics

- [ ] Branch protection: the required check is now "Kernel (checks, clippy, fast gate)".
- [ ] Full gate green on the release commit (`tools/kernel_gate.sh`), and an independent review of it.
- [ ] Merge `kernel-10` into main once the document check matches or beats main.

## 2. After the release

- [ ] An optional G2 corner mode for fillets: where three edges of different radii meet, ease each
  strip's radius to one vertex sphere's (Dahl §3.5.2) instead of the patched C0 corner (18° kink
  on a box with radii 1/2/3). Built and parked (`h3_corners.patch` in agent A's scratchpad): it
  makes each eased strip a B-spline, 2–5× slower on complex_e3/d2, with areas a little further
  from the reference's, so it stays out of the default path.

- [ ] Native B-rep threads: external and internal, multi-start, taper, partial length, run-outs.
  Built from exact helix curves and ruled surfaces. The Pali Ijo kernel's design
  (`packages/pali_ijo/doc/brep_threads.md` in github.com/braydennilsen/PaliCAD, MIT; the rest of
  PaliCAD is PolyForm Internal Use, so don't use it) is a good reference.
- [ ] Ideas from the Pali Ijo kernel (`packages/pali_ijo` only, MIT; credit "adapted from Pali Ijo,
  MIT, Copyright (c) 2026 Brayden Nilsen" wherever an algorithm is followed closely):
  - Mesh repair for imported meshes: vertex welding, removing degenerate faces, stitching
    junctions, sealing open boundaries, splitting connected components (`lib/src/mesh/mesh_build.dart`,
    `mesh_regions.dart`, `lib/src/brep/heal.dart`). This also helps the mirror_test import above.
  - Mesh tools for STL/3MF bodies: mesh booleans, convex hull and convexity check, Minkowski
    sums and differences (round offsets of meshes), plane cuts.
  - Exchange: 3MF and OBJ read and write, next to STEP, IGES and STL.
  - Chamfer by distance and angle: the second setback is `d1·sin(angle)/sin(beta + angle)`, where
    beta is the section angle between the faces; reject values that don't fit the section.
    Check it against ours.
  - Tessellation: keep patch boundaries watertight when the faces on either side need different
    sample counts (shared boundary samples), and use derivative bounds with aligned knots to
    avoid over-subdividing cubic spans.
  - Measurement: sphericity, next to volume, area, centre of mass and bounds.
- [ ] Feature gaps from review 4:
  - draft on freeform faces;
  - an N-argument cell decomposition;
  - booleans of mixed body kinds.
