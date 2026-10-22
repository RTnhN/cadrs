# TODO

The focus is **correctness for a release** of cadrs on the native kernel (`cadrs_kernel`, pure
Rust, no OpenCASCADE). New features wait until the release blockers are done.

## 1. Release blockers: correctness

### Real documents rebuild at least as well as on main (OCCT)

All 149 stored documents (173 Part Studios) are rebuilt headlessly with kernel-10 and with main's
OCCT kernel and compared per feature. Neither crashes or hangs.

| | Feature errors | New vs main |
|---|---|---|
| kernel-10 at `bdf82f3` (first run) | 140 | 63 in 28 Part Studios |
| kernel-10 at `dc8d23f` | ~84 | 4 |
| main (OCCT) | 106 | — |

Done when no Part Studio has an error that main doesn't also have, and none rebuilds much slower.
Rebuild a single document with `cadrs --document "<name>" --check`.

Fixed (dropped from the list): open sketch loops (d4 desk device v2/v3, d4_desk_device_test, emax,
squish's profile), the open-shell and 3-face booleans, plant_pot's extrude direction, the stroller,
pvc and rianatest blends, Dr Browns cap's Fillet 1 (rounds round a window across the dome's seam)
and the cable organizer's Fillet 2 (full rounds meeting at a corner), references (mate connectors, pibox thicken, t20, trimmer's unnamed
faces), mirror_test's mesh import.

Still new against main:

Differences without errors, still to check:
- [ ] Part counts that differ from main (garbage_cabinet 7 vs 3, TubeAdapter 3 vs 1 from chamfer
  slivers, racingrigrumble 10 vs 8, and others in compare runs).
- [ ] Small volume differences (pibox mount 1802.4 vs 1800.1 mm³, stroller chamfer 1429 vs 762 mm³).
- [ ] Rebuild time: 896 s locally against main's 423 s on gray. pico2, nightlight_sphere,
  ekgpulse, squish and trimmer are still 2–5× slower.
- [ ] Add a document check to the release process. A small headless runner rebuilds every
  document and compares against a saved result. Private documents stay out of the repo; only
  minimal dumped bodies go in as regression fixtures.

### Kernel correctness (independent review 4, 7.45/10 at `b589424`)

- [ ] Fillets and blends:
  - occt_blend: 96 known failures (most "area differs" ones pick another edge than OCCT's
    explode order, and five expect OCCT's own failure);
  - SMLib fillets: 36/100 pass;
  - no occt_blend case needs the tool-and-boolean fallback any more; four SMLib cases still do
    (case_407, 416, 417, 419).
- [ ] Booleans: 166 occt_boolean known failures (section lengths, sub-shape counts, open shells, areas).
- [ ] Offsets: 193 occt_offset known failures (101 of them B-spline wire offsets); smlib_offset 23/29.
- [ ] A merged face's new seam can run through one of its holes. The boolean merges the
  pieces of one face back (`unify::merge_group`) and gives the merged face a seam at a vertex of
  its boundary chain. In Dr Browns cap that put the seam at u = 0, inside the span of a window
  across the dome's seam. The fillet then can't place its cuts; `fillet::open_seam_holes` works
  around it. Three fixes each fixed the cap, or broke `smlib_offset offset_12_offset_bracket2`
  ("open shell") on the Ryzen box, where it still passes on gray:
  (a) cutting the seam at the hole afterwards (`cut_seams_at_holes` on the merged face or the
  boolean's output) fixed the cap but broke offset_12;
  (b) choosing a seam vertex clear of the holes kept offset_12, but the cap's chain has no such
  vertex;
  (c) splitting the chain at a clear u fixed the cap but broke offset_12 again.
  The inputs are in `doc_fixtures/drbrowns_window_{target,tool}.body` (private, outside the
  repository).
- [ ] `explode` should list degenerated edges as OCCT's does. That would turn 13 blend topology
  mismatches into real comparisons.
- [ ] The BRep reader:
  - 5 dataset files that OCCT reads;
  - CFI_5_b12fgg.rle edge count;
  - loose edges and vertices next to faces;
  - wire-only files.
- [ ] STEP/IGES: 128 of OCCT's 389 dataset files don't read clean (`tests/exchange_corpus.rs`).
- [ ] Fillet speed: 1.55× OCCT on occt_blend and 1.94× on smlib_fillet (PERF.md); the goal is at or
  below OCCT.

### Release mechanics

- [ ] Branch protection: the required check is now "Kernel (checks, clippy, fast gate)".
- [ ] Full gate green on the release commit (`tools/kernel_gate.sh`), and an independent review of it.
- [ ] Merge `kernel-10` into main once the document check matches or beats main.

## 2. After the release

- [ ] An optional G2 corner mode for fillets: where three edges of different radii meet, ease each
  strip's radius to one vertex sphere's (Dahl §3.5.2) instead of the patched C0 corner (18° kink
  on a box with radii 1/2/3). Built and parked (`h3_corners.patch` in agent A's scratchpad): it
  makes each eased strip a B-spline, 2–5× slower on complex_e3/d2, with areas a little further
  from OCCT's, so it stays out of the default path.

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
  - OCCT form features between non-planar faces;
  - draft on freeform faces;
  - Move face by translate and rotate;
  - an N-argument cell decomposition;
  - booleans of mixed body kinds.
