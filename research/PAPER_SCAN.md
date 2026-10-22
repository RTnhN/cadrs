# Paper scan (2026-10-22)

A pass over every .md/.txt/.tex under `research/papers/` and `research/notes/`, against the
implementation cards (`algorithms/`), FINDINGS.md, TODO.md and the kernel source. Only gaps are
listed: ideas no card has, or concrete additions to a card. Effort: S (hours), M (a day or two),
L (more). Unmeasured claims are marked; measure before building.

## Speed

1. **Multi-tool booleans** (NEW, M). `boolean_with` runs one full boolean per tool, so N pattern
   cuts cost N passes. Cull each tool to the faces its box touches, run tools with disjoint boxes
   in parallel, or build one variadic arrangement. Cherchi 2022 §6, QuickCSG §6.
2. **BVH in `classify.rs` parity rays** (EXTENDS card 7, S). `parity_inside` scans every face per
   ray (up to 37 rays) and never uses `bvh.rs`. Huang 2025: shell/sub-shell BVH, centroid splits.
3. **Quadric-pair closed forms** (NEW, M). Classify a quadric pair by det(λA−B) (Dupont I–III,
   Wang–Goldman–Tu, Trocado 2019) and emit exact conics for skew cylinder×cylinder, off-axis
   sphere/cone×cylinder, cone×cone and equal-radius crossings; today all go to marching.
4. **Torus conics** (NEW, S–M). Kim–Kim 2000: profile, cross-section and Villarceau circles for
   torus×torus/plane/sphere/cylinder, with an explicit seam split.
5. **Tessellation refinement cap** (NEW, S). `tess.rs::triangulate` allows ≥10 000 extra vertices
   per face at a 30° minimum angle; Ruppert refinement only provably terminates up to ~20.7°
   (Shewchuk 2002 §6). Size the budget from face area, cap the angle at 20–25°.
6. **Parallel fillet/sweep/loft/offset** (NEW, S–M). None use rayon. Parallelise per-chain and
   per-section solves, collect in index order (determinism test), and report a 12-thread wall-time
   ratio next to the single-thread CPU ratio.
7. **Indexed point-in-trim** (NEW, M, measure first). `classify::in_face` and
   `FaceDomain::contains` scan every loop segment per query (43 call sites). Sloup 2021: monotone
   loop pieces under a kd-tree. Count segment visits on slow documents first.
8. **Body::check on touched faces only** in blend `local_problem` (engineering, S).
9. **Point projection** (EXTENDS cards 6/7, M). Johnson–Cohen 2005 all-local-extrema projection
   with a dynamic subdivision tree, ~6× on coherent queries. **Span-shared basis evaluation** for
   grid points (Balázs 2004, EXTENDS card 10, S).

## Correctness

10. **One vertex–face decision table** feeding every edge–face hit (EXTENDS cards 2/3, M). Smith
    §2.3.3/§4.9: contradictory independent decisions trigger the global tolerance retries. Plus a
    cancellation pass for coincident opposing coedges/sub-faces (Smith §5.1, partial coincidence).
11. **Variable-radius canal checks** (EXTENDS blends cards 1/7, S). Dahl §3.3–3.4: envelope real
    only if |r′| < spine speed; constant radius needs r·κ < 1; contact direction
    −r′t + √(1−r′²)(…) as the walker predictor. Dahl §3.5.2: three-edge corners, unequal radii.
12. **Offset-rail crossing pre-test** (EXTENDS blends cards 2/8, S). Trace offset rails in uv by
    least-squares stepping, test them as uv polylines + 2×2 Newton before building the strip
    (Flends, AIAA 2019 §II.B): catches rails that don't split their face, set-back strips off it.
13. **Wire offsets by winding number** (NEW, M, unverified). Keep raw offset pieces by the positive
    winding rule (Chen–McMains 2005) instead of the distance test in `offset/wire.rs`.
14. **Sewing tolerance ladder** (EXTENDS card 11, S). Sew at increasing tolerances with a cap
    instead of one weld tolerance in `xbuild.rs::sew_loose`.
15. **Tolerant analytic recognition** (EXTENDS `recognize.rs`, M). Lukacs 1998 least-squares
    cylinder/cone/torus fits to recognise near-analytic exporter B-splines at ~1e-5; NURBS →
    revolution by a Plücker/velocity-field axis test (Barton–Pottmann–Wallner 2014). Count the
    corpus payoff first.
16. **Naming fallback by neighbour similarity** (NEW, M). Agbodan 2003 / Marcheix 2006 δ0/δ1
    neighbour-circuit scores with greedy matching, replacing the nearest-position last resort in
    `naming::resolve`; Bidarra 2005 side-of-reference-plane edge names for repeated intersections.
17. **Sweep fold invariant** θ ≤ 0 (Adsul 2012, EXTENDS offsets card 7, S).

## Gates (no OCCT needed)

18. **Metamorphic booleans**: vol(A∪B)+vol(A∩B)=vol(A)+vol(B), A−A=∅, A∪A=A, tool-order
    invariance, STEP round-trip volume. Over ABC / Fusion 360 Gallery (replayable extrude
    sequences with ground-truth volumes).
19. **Stress**: a cube united with copies rotated by tiny angles across the tolerance; 12+
    iterated unions checking tolerance/volume drift; EMBER's iterated drill-bit chain as a
    benchmark.
20. **Mesh vs exact**: closed-form mesh volume vs `mass.rs`, |ΔV| ≤ area × chord, plus a
    triangle-count ceiling per face.
21. **Opening/closing oracle** for fillets: fillet-all-convex = opening, fillet-all-concave =
    closing; compare with cadrs's own offsets (max |dist − d|, Cao 2024).
