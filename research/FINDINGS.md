# Findings

The research is distilled into 49 implementation cards in [algorithms/](algorithms/), with the full report in [README.md](README.md), the per-area notes in [notes/](notes/), and 169 papers as Markdown in [papers/](papers/). The PDFs are git-ignored.

Each card gives the algorithm's steps, equations and constants, where it plugs into cadrs, and its sources. It also says whether it was checked against the paper text or OCCT source, or comes only from the notes. The build order in [algorithms/README.md](algorithms/README.md) starts with five cheap measurements, because none of the payoffs has been timed in cadrs.

## Status (2026-10-23, kernel-perf)

The ratios quoted below are the 2026-10-21 baseline. Since then the kernel has been through speed and fillet passes (git log, `kernel:` commits of 2026-10-22 and 2026-10-23). Against the reference (CPU time, same cases): fillets went from 9.2x to about 1.1x on the blend suite and from 10.3x to about 1.2x on the SMLib fillet suite, and the pipe cases' geometric mean fell to 0.70 of its earlier time (`eec99ce`; not yet compared with the reference again). What each card or scan item came to, as far as the commit messages say:

- **Done (build order, [algorithms/README.md](algorithms/README.md)):**
  - #1 sweep sections spaced by curvature (`eec99ce`);
  - #3 exact Gauss rules for mass properties (`eec99ce`; the area-integrand error estimate is not recorded as done);
  - #5 one solve for the 3D curve and both p-curves (`e21a46e`);
  - #8 balls only at pointed vertices (`eec99ce`);
  - #11 STEP p-curve shortcuts and parallel face building (`eec99ce`).
- **Partly done:** #6 cheaper ball solves (warm-started mitre root with Illinois regula falsi, one 4x4 solve for the ball tangent: ball solves 22.1 M to 4.2 M, `3b24c14`); #10 and #14 strip and rail p-curves from contact points, and rail exits on a face's p-curve image (`3b24c14`, `16baa15`); #13 a different SSI cull (cell pairs on one side of the other surface, `e8258d7`) rather than the Gauss-map test.
- **Open:** #2 closed-form analytic drafts, #4 trigonometric and planar mass rules, #7 sum-of-tolerance vertex merge, #9 walker with a tangent predictor, #12, #15 to #18 and the rest of the build order. Not timed again since the passes above.
- **PAPER_SCAN ([PAPER_SCAN.md](PAPER_SCAN.md)):**
  - done: #8 `Body::check` on changed faces only (`6b274d2`); #18 and #19 metamorphic and stress booleans (`tests/metamorphic_booleans.rs`, `stress_booleans.rs`, `6866a57`); #21 the opening oracle for fillets (`6b274d2`);
  - partly done: #11 radius laws that change faster than the edge runs are refused (|r'| < |g'|, `6b274d2`); #6 face splitting and STEP face building run in parallel, fillet, sweep, loft and offset do not;
  - measured and dropped: #1 multi-tool booleans (2.6% of document time); #2 BVH in the parity rays (at most 1.4%); #5 the Ruppert refinement path (unused); #20 mesh against exact volume (the test already existed);
  - open: the rest (#3, #4, #7, #9, #10, #12 to #17).

## The most valuable leads

- **Sweeps (2.4x slower than OCCT):** the worst pipe cases take about 135 ms against OCCT's 2 to 6 ms. The section-count estimate in `sweep/frames.rs` aims for about 1e-8 accuracy, but the refinement loop right after it accepts 1e-5. Counts are capped at 1024, so the gap may just be too many sections, not the algorithm. The two tolerances and the cap are confirmed in the code.
- **Mass properties (11x):** cards 1 and 2 of [mass_tessellation_hlr.md](algorithms/mass_tessellation_hlr.md) replace the 16-point rules with exact Gauss rules matched to each knot span. That's about 64x fewer evaluations on bicubic patches, with no error for volume, centroid and inertia. The surface-area integrand needs its own error check first.
- **Fillets (9 to 10x):** the probable cause is too many sections or Newton steps along each edge, which OCCT keeps to about 20 sections on a straight edge. The fix is OCCT's walker with a tangent predictor, plus strip p-curves built from data the walker already has. Counting ball solves per edge confirms this cheaply before anyone builds it.
- **Drafts (1.9x) and STEP import (1.9x):** the draft gap is five outlier cases, where edges could be computed in closed form instead of by general surface intersection. Only large STEP files are slow, 7 to 14x, and the time is in model building, not parsing. OCCT's shortcut is to reuse the file's p-curves rather than refitting them.
- **Booleans and intersection:** fitting the 3D curve and both p-curves in one solve removes the re-projection booleans and fillets do today. Switching vertex merging to sum-of-tolerances also settles an open question.

## Corrections to the report

- **Inertia:** it needs 8 Gauss points per bicubic span, not 6. The notes had used the centroid integrand.
- **OCCT's fillet constants:** its step-angle test is on the squared cosine, about 8°.
- **Kernel README:** `crates/cadrs_kernel/README.md` was out of date in three places (since refreshed). The numeric intersection already subdivides cell pairs before marching, torus recognition exists, and offset tubes are already limited to convex edges.

## The papers

| | Count |
|---|---|
| Papers | 169 |
| With Markdown (docling) | 169 |
| With text (pdftotext) | 159 |

The other 10 are scans or image-only PDFs, so their Markdown comes from OCR. The paper text adds about 24 MB to the repo.

Of the 44 papers the report listed as missing, 33 were later found. That includes author copies, repository copies, tech-report versions and archived copies of dead links. The 11 still missing need a library login; [download.md](download.md) lists them with links and file names.
