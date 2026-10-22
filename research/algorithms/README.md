# High-performance algorithms for cadrs_kernel

These are the algorithms from the research worth building into the kernel, written as implementation cards. Each card gives the problem, why the method is fast, its steps, equations and constants, the data it needs, its failure modes, where it plugs into cadrs, and its sources. Each card also says which parts were checked against the paper text or OCCT source, which come only from the research notes, and which are the card writer's own derivation.

| File | Area | Cards |
|---|---|---|
| [blends.md](blends.md) | Fillets, chamfers, vertex blends | 9 |
| [intersection_solvers.md](intersection_solvers.md) | Surface intersection, the shared box solver, point projection, distance | 9 |
| [booleans.md](booleans.md) | Booleans, classification, tolerant modelling | 10 |
| [mass_tessellation_hlr.md](mass_tessellation_hlr.md) | Mass properties, tessellation, hidden lines, silhouettes | 10 |
| [offsets_sweeps_import.md](offsets_sweeps_import.md) | Offsets, shells, sweeps, lofts, drafts, STEP import | 11 |

The background is in [../README.md](../README.md), the research report, and in [../notes/](../notes/).

## Measure first

None of these payoffs has been timed in cadrs. These counters are cheap, and they decide the order of the work below.

1. **Sweep section counts** on `pipe/specific_r4` and the other `pipe/specific_*` cases. They take about 135 ms against OCCT's 2 to 6 ms. The section-count estimate in `sweep/frames.rs` aims for about 1e-8, its refinement loop accepts 1e-5 × (1 + reach), and `MAX_SECTIONS` is 1024. If these cases sit at the cap, the gap is a constant, not the algorithm.
2. **Ball solves per fillet edge and the fallback rate.** `fillet.rs` already counts the solves in its budget. OCCT takes about 20 sections per straight edge. For the fallback rate, run the fillet cases with `CADRS_LOCAL_FILLET_ONLY=1` and count failures.
3. **`ssi::surface_surface` calls per draft** on the five slow draft cases (`draft/angle_a5`, `a6`, `a9`, `b1`, `c3`).
4. **Time per `XBuilder::finish` phase** on the large STEP files, which are 7 to 14 times slower than OCCT (`trj6_pm4_ec_203`, `as1_motor`).
5. **Boolean runs per call** (1, 2 or 3), face pairs sent to intersection, and rays per boolean. See the last section of [booleans.md](booleans.md).

## Build order

Sizes and payoffs are the card writers' estimates.

### Do next: small, low risk, aimed at a measured gap

| # | Card | Gap it targets | Size |
|---|---|---|---|
| 1 | [Sweep sections by local curvature, frames from the RMF error bound](offsets_sweeps_import.md#1-sweep-sections-spaced-by-local-curvature-frames-by-the-rmf-error-bound) | Sweeps 2.4x | Small |
| 2 | [Closed-form edges and vertices for analytic drafts](offsets_sweeps_import.md#2-closed-form-edges-and-vertices-for-analytic-drafts) | Drafts 1.9x | Small |
| 3 | [Area and rational integrands with an error estimate](mass_tessellation_hlr.md#2-area-and-non-polynomial-integrands-with-an-error-estimate), then [exact degree-matched Gauss rules](mass_tessellation_hlr.md#1-exact-degree-matched-gauss-rules-for-polynomial-faces) | Mass properties 11x | Small, then small to medium |
| 4 | [Trigonometric rules for analytic faces](mass_tessellation_hlr.md#3-trigonometric-and-periodic-rules-for-analytic-faces) and [closed forms for planar faces](mass_tessellation_hlr.md#5-planar-faces-and-mesh-solids-in-closed-form-mirticheberly) | Mass properties | Small |
| 5 | [Fit the 3D curve and both p-curves in one solve](intersection_solvers.md#1-fit-the-3d-curve-and-both-p-curves-in-one-solve) | Every numeric intersection edge in booleans and fillets | Small |
| 6 | [Cheap Newton iterates in the ball solve](blends.md#3-cheap-newton-iterates) | Fillets 9 to 10x | Small |
| 7 | [Connexity-chain vertex merge with sum-of-tolerance tests](booleans.md#2-connexity-chain-vertex-merge-with-sum-of-tolerance-tests) | Boolean retries; settles sum versus max | Small |
| 8 | [Balls only at pointed vertices](offsets_sweeps_import.md#5-balls-only-at-pointed-vertices-rossignacrequicha-pruning) | Booleans per rounded offset and shell | Small |

### Then: medium work on the big gaps

| # | Card | Gap it targets | Size |
|---|---|---|---|
| 9 | [Walked fillet sections with a tangent predictor and OCCT step control](blends.md#1-walked-sections-with-tangent-predictor-and-occt-step-control) | Fillets, if measurement 2 shows too many stations | Medium |
| 10 | [Hermite strip and rail p-curves from section derivatives](blends.md#2-hermite-strip-and-rail-p-curves-from-section-derivatives) | Fillets | Medium |
| 11 | [STEP p-curve and SameParameter shortcuts](offsets_sweeps_import.md#3-step-import-occts-p-curve-and-sameparameter-shortcuts) and [parallel face building](offsets_sweeps_import.md#4-build-imported-faces-in-parallel) | STEP import, large files | Medium |
| 12 | [Classify at the cut edge and propagate](booleans.md#6-classify-at-the-intersection-edge-and-propagate-across-it), then [one ray per remaining component](booleans.md#7-one-cheap-ray-per-remaining-component-first-hit-cascade-perturbation) | Rays per boolean | Medium, then small |
| 13 | [Gauss-map separability with polyhedral normal bounds](intersection_solvers.md#2-gauss-map-plane-separability-with-polyhedral-normal-bounds) | Cell pairs in the numeric intersection | Small to medium |
| 14 | [Fillet exits found during the walk](blends.md#4-exits-found-during-the-walk-re-framed-by-an-inverse-solve) and [cliff-edge blend functions](blends.md#5-cliff-edge-blend-functions-surfaceedge-and-edgeedge) | Fillet fallbacks | Medium |
| 15 | [Skip face–face intersection where edges already decide contact](booleans.md#4-skip-faceface-ssi-where-lower-dimensional-contacts-already-decide-it) and [tighter face-pair culling](booleans.md#8-tighter-face-pair-culling-trimmed-boxes-obb-separating-plane) | Boolean speed, if measurement 5 shows intersection time dominating | Medium, small |

### Larger structural work

| # | Card | What it buys | Size |
|---|---|---|---|
| 16 | [Shared Bernstein/B-spline box solver](intersection_solvers.md#3-shared-bernsteinb-spline-box-solver) | The base for certified intersection, projection and distance (intersection cards 4, 6, 7, 9) | Medium to large |
| 17 | [Certified analytic × B-spline intersection](intersection_solvers.md#4-certified-analytic--b-spline-intersection-regular-cells-and-characteristic-points) | No missed loops in the commonest boolean pair | Medium |
| 18 | [Local re-intersection instead of the global retry](booleans.md#1-local-re-intersection-instead-of-the-global-retry), needing [pave blocks](booleans.md#3-pave-blocks-shrunk-ranges-and-common-blocks) | Fewer whole-boolean retries | Large |
| 19 | [Trimming path before topology edits (HP/Opitz)](blends.md#6-trimming-path-before-topology-edits-hp--opitz) | Fillets that grow or shrink neighbour faces stay local | Large |
| 20 | [Offset boundary from tentative faces, kept by distance](offsets_sweeps_import.md#6-offset-boundary-from-tentative-faces-kept-by-distance) | Rounded offsets and shells with no booleans when nothing clashes | Large |

The other cards are smaller wins or new features. These include point projection and LUB-tree distance, rounded intervals, Gunderman trimmed quadrature, HLR quantitative invisibility, the tessellation bounds and seeding, mitre joins, lofting, Part 21 parsing and sewing. Each file's ranked table places them.

## Corrections the cards make to the research report

- **Inertia needs more Gauss points than the notes say.** For a bidegree (p, q) span it needs ⌈5p/2⌉ points per direction, which is 8 for bicubic, not 2p. The notes used the centroid integrand. The card's derivation was checked numerically.
- **Tolerances combine by sum, not max.** The OCCT spec and Jackson both use the sum, and cadrs takes the max. OCCT's merged vertex is the sphere enclosing the source spheres.
- **OCCT's fillet step constants are on squared cosines.** The 3D step-angle limit is cos² ≥ 0.98, about 8°. The sagitta estimate squares the chord.
- **The kernel README is out of date in three places.** The numeric intersection already subdivides cell pairs with a normal-cone test, torus recognition exists, and offset tubes are already limited to convex edges.
