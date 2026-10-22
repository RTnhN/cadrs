# Papers to download by hand

These papers matter to the research but couldn't be downloaded by script, because they are paywalled or the site blocks automated downloads. They come from Appendix B of [README.md](README.md).

Papers marked **free** should open in a normal browser. Most of the rest are Elsevier, Springer or IEEE links that need a library or institutional login.

Ten free ones were fetched on 2026-10-22 with `~/tools/playwright/fetch_pdf.sh`, which drives a visible Chrome window; it handles sites that block curl and wget. Use it for the free ones left (`fetch_pdf.sh URL OUT.pdf`).

Save each PDF into the folder named in its heading, then convert it with docling and pdftotext like the others. The command is in the last section of [README.md](README.md).

## `papers/intersection/`

- **free:** Mourrain & Pavone 2009, subdivision methods for polynomial equations: https://inria.hal.science/inria-00070350
- **free:** Sherbrooke 1994 MIT thesis, the interval projected polyhedron solver: http://hdl.handle.net/1721.1/12300
- **free:** Mukundan 2005 MIT thesis: https://dspace.mit.edu/handle/1721.1/33167
- **free, server down on 2026-10-22:** Hu & Wallner 2005, second-order orthogonal projection: http://www.geometrie.tugraz.at/wallner/sproj.pdf
- Grandine & Klein 1997, a new approach to the surface intersection problem: https://doi.org/10.1016/S0167-8396(96)00024-9
- Wang, Goldman & Tu 2003, enhancing Levin's method: https://doi.org/10.1016/S0167-8396(03)00081-5
- Lin et al. 2014, affine-arithmetic B-spline intersection on the GPU: https://doi.org/10.1109/TVCG.2013.237

## `papers/blends/`

- Vida, Martin & Várady 1994, survey of blending methods: https://doi.org/10.1016/0010-4485(94)90023-X
- Várady & Rockwood 1997, setback vertex blending: https://doi.org/10.1016/S0010-4485(96)00070-X
- Braid 1997, non-local blending of boundary models: https://doi.org/10.1016/S0010-4485(96)00038-3
- Chuang, Lin & Hwang 1995, variable-radius blending: https://doi.org/10.1007/BF02434038
- Chuang & Hwang 1997, variable-radius blending by spine generation: https://doi.org/10.1007/s003710050106
- Klass & Kuhn 1992, rolling-ball blends: https://doi.org/10.1016/0167-8396(92)90016-I
- Farouki & Sverrisson 1996, rolling-ball blends: https://doi.org/10.1016/0010-4485(96)00008-5
- Lukács 1998, G1 variable-radius rolling-ball blends: https://doi.org/10.1016/S0167-8396(98)00006-5
- Pegna & Wolter 1992, curvature continuity of blends: https://doi.org/10.1115/1.2916918
- Dahl & Krasauskas 2012, rational rolling-ball blends between quadrics: https://doi.org/10.1016/j.cagd.2012.06.008

## `papers/offsets_sweeps_lofts/`

- Forsyth 1995, shelling and offsetting bodies: https://doi.org/10.1145/218013.218088
- Piegl & Tiller 2002, surface skinning revisited: https://doi.org/10.1007/s003710100156

## `papers/tessellation_mass_queries/`

- **free:** Johnson & Cohen 2005, distance extrema with tangent cones, from the project page: https://www-old.cs.utah.edu/gdc/projects/minimum_distance/
- Hughes, Reali & Sangalli 2010, efficient quadrature for NURBS: https://doi.org/10.1016/j.cma.2008.12.004
- Filip, Magedson & Markot 1986, surface algorithms using derivative bounds: https://doi.org/10.1016/0167-8396(86)90005-1
- Bénard, Hertzmann & Kass 2014, smooth surface contours with accurate topology: https://doi.org/10.1145/2558307

## `papers/booleans/`

- **free, server refused connections on 2026-10-22:** Trettner, Nehring-Wirxel & Kobbelt 2022, EMBER exact mesh booleans: https://www.graphics.rwth-aachen.de/media/papers/339/ember_exact_mesh_booleans_via_efficient_and_robust_local_arrangements.pdf
- Attene 2020, indirect predicates: https://doi.org/10.1016/j.cad.2020.102856
- Urick et al. 2019, watertight Boolean operations, Computer-Aided Design 115:147–160. There's no link: search the title on sciencedirect.com.

## `papers/exchange_healing_naming/`

- **free:** Shen 2000 MIT thesis, B-rep model rectification: https://dspace.mit.edu/handle/1721.1/88491
- **free:** Benkő, Martin & Várady 2001, reverse engineering B-rep models: https://orca.cardiff.ac.uk/id/eprint/48124/
- **free:** Farjana & Han 2018, persistent identification review: https://doi.org/10.1016/j.aej.2018.01.007
- **free:** Bi et al. 2023, parallel STEP parsing, in Chinese: https://doi.org/10.3724/SP.J.1089.2023.2023-00037
- Marussig & Hughes 2018, review of trimming in isogeometric analysis: https://doi.org/10.1007/s11831-017-9220-9
- Kripac 1997, persistently naming topological entities: https://doi.org/10.1016/s0010-4485(96)00040-1
- Capoyleas, Chen & Hoffmann 1996, generic naming in generative design: https://doi.org/10.1016/0010-4485(95)00014-3
- Marcheix & Pierra 2002, survey of the persistent naming problem: https://doi.org/10.1145/566282.566288
