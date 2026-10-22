# Findings

The research is distilled into 49 implementation cards in [algorithms/](algorithms/), with the full report in [README.md](README.md), the per-area notes in [notes/](notes/), and 169 papers as Markdown in [papers/](papers/). The PDFs are git-ignored.

Each card gives the algorithm's steps, equations and constants, where it plugs into cadrs, and its sources. It also says whether it was checked against the paper text or OCCT source, or comes only from the notes. The build order in [algorithms/README.md](algorithms/README.md) starts with five cheap measurements, because none of the payoffs has been timed in cadrs.

## The most valuable leads

- **Sweeps (2.4x slower than OCCT):** the worst pipe cases take about 135 ms against OCCT's 2 to 6 ms. The section-count estimate in `sweep/frames.rs` aims for about 1e-8 accuracy, but the refinement loop right after it accepts 1e-5. Counts are capped at 1024, so the gap may just be too many sections, not the algorithm. The two tolerances and the cap are confirmed in the code.
- **Mass properties (11x):** cards 1 and 2 of [mass_tessellation_hlr.md](algorithms/mass_tessellation_hlr.md) replace the 16-point rules with exact Gauss rules matched to each knot span. That's about 64x fewer evaluations on bicubic patches, with no error for volume, centroid and inertia. The surface-area integrand needs its own error check first.
- **Fillets (9 to 10x):** the probable cause is too many sections or Newton steps along each edge, which OCCT keeps to about 20 sections on a straight edge. The fix is OCCT's walker with a tangent predictor, plus strip p-curves built from data the walker already has. Counting ball solves per edge confirms this cheaply before anyone builds it.
- **Drafts (1.9x) and STEP import (1.9x):** the draft gap is five outlier cases, where edges could be computed in closed form instead of by general surface intersection. Only large STEP files are slow, 7 to 14x, and the time is in model building, not parsing. OCCT's shortcut is to reuse the file's p-curves rather than refitting them.
- **Booleans and intersection:** fitting the 3D curve and both p-curves in one solve removes the re-projection booleans and fillets do today. Switching vertex merging to sum-of-tolerances also settles an open question.

## Corrections to the report

- **Inertia:** it needs 8 Gauss points per bicubic span, not 6. The notes had used the centroid integrand.
- **OCCT's fillet constants:** its step-angle test is on the squared cosine, about 8°.
- **Kernel README:** `crates/cadrs_kernel/README.md` is out of date in three places, and it hasn't been edited. The numeric intersection already subdivides cell pairs before marching, torus recognition exists, and offset tubes are already limited to convex edges.

## The papers

| | Count |
|---|---|
| Papers | 169 |
| With Markdown (docling) | 169 |
| With text (pdftotext) | 159 |

The other 10 are scans or image-only PDFs, so their Markdown comes from OCR. The paper text adds about 24 MB to the repo.

Of the 44 papers the report listed as missing, 33 were later found. That includes author copies, repository copies, tech-report versions and archived copies of dead links. The 11 still missing need a library login; [download.md](download.md) lists them with links and file names.
