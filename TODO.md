# TODO

The focus is **correctness for a release** of cadrs on the native kernel (`cadrs_kernel`, pure
Rust, no OpenCASCADE). New features wait until the release blockers are done.

## 1. Release blockers: correctness

### Real documents rebuild at least as well as on main (OCCT)

All 149 stored documents (173 Part Studios) were rebuilt headlessly with kernel-10 at `bdf82f3`
and with main's OCCT kernel. Neither crashed or hung.

| | Feature errors | Total rebuild time |
|---|---|---|
| kernel-10 | 140 | 840 s |
| main | 106 | 423 s |

- 63 errors in 28 Part Studios are new on kernel-10; many are cascades of a first failure.
- 29 of main's errors are fixed on kernel-10.

Done when no Part Studio has an error that main doesn't also have, and none rebuilds much slower.
Rebuild a single document with `cadrs --document "<name>" --check`.

- [ ] Sketch loops rejected as "not closed", with gaps down to 7e-5 mm:
  - d4 desk device v2 and v3, Extrude 1 (Transform 1, Sketch 23 and Extrude 32 cascade from it);
  - d4_desk_device_test, Extrude 18;
  - emax gt2815/05, Extrude 3;
  - squish_bottle_adapter, Extrude 6.
- [ ] Booleans that leave an open shell:
  - Modeling Practice Drawings 93;
  - babygatefix;
  - cable_mount;
  - d4 desk device v2/v3, Extrude 16;
  - ekgpulse bracket;
  - gridfinity die tap 2x2;
  - potentiometer knob;
  - racingrigrumble;
  - squish_bottle_adapter;
  - trimmer;
  - weller led block.
- [ ] Booleans that leave an edge used by 3 faces: 32mm pvc pipe snap, gridfinity die tap 2x2 (Mirror 1), rianatest.
- [ ] A boolean face that doesn't close (p-curve 1.1e2 off its edge): Dr Browns cap.
- [ ] Fillets and chamfers refused where OCCT builds them:
  - babygatefix, Fillet 4 ("too large");
  - plant_pot_dish_square, Fillet 1 ("too large");
  - stroller / wheel, Chamfer 1 ("tangent: nothing to blend").
- [ ] References that no longer resolve (persistent naming against the new kernel's topology):
  - mate connectors in alu_extrusion_parts, cable organizer and desk_current_sensor_gadget;
  - selected faces in pibox / piboxv7 and t20 esp32 mount.
- [ ] Mesh import "not consistently oriented or not manifold": mirror_test, Import 1.
- [ ] Slow rebuilds (kernel-10 vs main):
  - pibox / nightlight_sphere: 161 s vs 14 s;
  - ekgpulse / bracket Copy 1: 106 s vs 24 s;
  - gridfinity tap handle holder: 40 s vs 3.7 s;
  - innerdoor_honda92: 12 s vs 0.5 s;
  - gearlevertest: 16 s vs 0.9 s;
  - racingrigrumble: 15 s vs 1.4 s.
- [ ] Add a document check to the release process. A small headless runner rebuilds every
  document and compares against a saved result. Private documents stay out of the repo; only
  minimal dumped bodies go in as regression fixtures.

### Kernel correctness (independent review 4, 7.45/10 at `b589424`)

- [ ] Fillets and blends:
  - occt_blend: 173 known failures (40 wrong areas, 21 open shells, 16 variable fillets of curved edges, 11 degenerate sections, …);
  - SMLib fillets: 34/100 pass;
  - 16 cases still need the tool-and-boolean fallback.
- [ ] Booleans: 166 occt_boolean known failures (section lengths, sub-shape counts, open shells, areas).
- [ ] Offsets: 193 occt_offset known failures (101 of them B-spline wire offsets); smlib_offset 23/29.
- [ ] `explode` should list degenerated edges as OCCT's does. That would turn 13 blend topology
  mismatches into real comparisons.
- [ ] The BRep reader:
  - 5 dataset files that OCCT reads;
  - CFI_5_b12fgg.rle edge count;
  - loose edges and vertices next to faces;
  - wire-only files.
- [ ] STEP/IGES: 128 of OCCT's 389 dataset files don't read clean (`tests/exchange_corpus.rs`).
- [ ] Performance outliers: fillets are about 10× slower than OCCT; a few cases are 100–180× slower.

### Release mechanics

- [ ] Branch protection: the required check is now "Kernel (checks, clippy, fast gate)".
- [ ] Full gate green on the release commit (`tools/kernel_gate.sh`), and an independent review of it.
- [ ] Merge `kernel-10` into main once the document check matches or beats main.

## 2. After the release

- [ ] Native B-rep threads: external and internal, multi-start, taper, partial length, run-outs.
  Built from exact helix curves and ruled surfaces. The Pali Ijo kernel's design
  (`packages/pali_ijo/doc/brep_threads.md` in github.com/braydennilsen/PaliCAD, MIT; the rest of
  PaliCAD is PolyForm Internal Use, so don't use it) is a good reference.
- [ ] Mesh tools, also with Pali Ijo as a reference: repair (weld, stitch, seal), convex hull,
  3MF/OBJ export.
- [ ] Feature gaps from review 4:
  - OCCT form features between non-planar faces;
  - draft on freeform faces;
  - Move face by translate and rotate;
  - an N-argument cell decomposition;
  - booleans of mixed body kinds.
