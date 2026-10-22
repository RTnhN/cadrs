# Offsets, shells, sweeps, lofts, drafts and local operations in B-rep kernels

Scope: published algorithms for (a) offsetting B-rep solids and trimming offset surfaces, (b) shelling,
(c) sweeping profiles along curves, (d) lofting/skinning, (e) draft and face-replacement local
operations, measured against what `cadrs_kernel::native` does today (README lines 153–220, 344–362:
sharp offsets re-intersect the faces' offset surfaces; rounded offsets and shells are built as the body ∪
each face's slab ∪ a tube per convex edge ∪ a ball per vertex, i.e. many booleans; sweeps use
double-reflection RMFs and one skinned B-spline surface per profile curve over a tangent run, with
mitred and round corners; lofts skin compatible sections exactly or sample-interpolate; drafts and face
replacement follow SMLib's local operations; sweeps are 2.4× and drafts 1.9× slower than OCCT).

Downloaded PDFs are in `../papers/offsets_sweeps_lofts/` (every file was
verified to start with `%PDF`). Paywalled papers were not downloaded.

Research caveat: the shared web-search budget ran out part-way through (after about 45 searches), so
the second half of the work relied on fetching known documentation pages and reading the downloaded
PDFs. Items that could not be verified are listed under "What I could not find".

## Must-read ranking (top 5 first)

1. Rossignac & Requicha 1986, "Offsetting operations in solid modelling" — the theory behind the
   kernel's slab/tube/ball construction, including the pruning rules it does not yet exploit
   (entry A1).
2. Parasolid's offsetting / hollowing / local-operations documentation (v12 ch. 12, 19, 20; V35 ch. 58–62)
   — the only public description of what a production kernel does when an offset distance exceeds a
   concave radius, when faces collapse, and how drafts are built from isocline surfaces (entries A5, B1, E1).
3. Wang, Jüttler, Zheng & Liu 2008, "Computation of rotation minimizing frames" — cadrs already
   uses the double-reflection method; the paper's error bound and its extensions (closed spines, boundary
   conditions) are what remain to use (entry C1).
4. Seong, Elber & Kim 2006, "Trimming local and global self-intersections in offset curves/surfaces
   using distance maps" — the cleanest published criterion for which part of an offset face survives
   (entry A4).
5. Siggel, Kleinert, Stollenwerk & Maierl 2019 (TiGL), "Gordon surfaces" in B-spline form, plus the
   Piegl–Tiller / Park family of approximate-skinning papers — lofts with guides and incompatible
   sections (entries D1, D2).

Then: OCCT's MakePipeShell / MakeOffsetShape / ThruSections / DraftAngle documentation (C4, A6, D5,
E2), Chiang–Hoffmann–Lynch 1992 (A3), Cao et al. 2024 (A8), Adsul–Machchhar–Sohoni 2012 (C7),
Peternell–Pottmann–Steiner–Zhao 2005 (C6), Abdel-Malek–Blackmore–Joy 2006 (C5).

---

## (a) Offsetting B-rep solids and trimming offset surfaces

### Takeaway
Every published B-rep offset method is a variant of one scheme: the boundary of the offset solid is
a subset of the union of (i) the normal offsets of the faces, (ii) pipe (canal) surfaces of radius d round
the edges that are singular in the right convexity, and (iii) spheres round the singular vertices
(Rossignac & Requicha 1986). Methods differ only in how the surviving subset is found: by booleans of
slabs/tubes/balls (what cadrs does), by pairwise re-intersection of neighbouring offset surfaces
guided by the original topology (Parasolid "accurate"/"trim+extend", OCCT "Intersection" join,
Forsyth 1995), by a distance-map test on each offset surface (Seong–Elber–Kim), or by a global
distance field / level set (Chiang–Hoffmann–Lynch; all the mesh and GPU methods). Production kernels
(Parasolid) combine re-intersection with explicit repair rules: faces whose offset is wholly inside-out
(distance > minimum radius of curvature, e.g. blends) are deleted, partially self-intersecting swept or
spun surfaces are cut back with a new edge per self-intersection, holes are patched with B-surfaces or
closed by extending neighbours, and vertices with more than three edges are split.

### Cited Findings

#### A1. Rossignac & Requicha 1986 — MUST READ
- Citation: Jaroslaw R. Rossignac, Aristides A. G. Requicha, "Offsetting operations in solid
  modelling", Computer Aided Geometric Design 3 (1986) 129–148, North-Holland. — [IBM Research record](https://researcher.ibm.com/publications/offsetting-operations-in-solid-modelling)
- Open-access PDF (author's page): https://faculty.cc.gatech.edu/~jarek/papers/Offsets.pdf — local copy
  `../papers/offsets_sweeps_lofts/Rossignac-Requicha-1986-offsetting-operations-in-solid-modelling.md`
- Algorithm (from the PDF, sections 2.6, 4.1–4.4): the solid offset S↑r is the Minkowski sum of S with a
  ball of radius r (regularised); the negative offset is the complement of the expanded complement,
  S↓r = c*((c*S)↑r); rounding is R_r(S) = (S↓r)↑r and filleting F_r(S) = (S↑r)↓r, which "rounds the convex
  edges and vertices" and "fillets the concave edges and vertices" respectively. The boundary of the
  offset is contained in the "n-offset" of ∂S: for a smooth face the normal offset surface q + r·n; for a
  singular edge the surface swept by a circle of radius r in the normal plane (a canal/pipe surface;
  tori and cylinders for circular and straight edges, otherwise approximated by tori and cylinders);
  for a singular vertex a sphere of radius r. Pruning rules for the tentative faces: "Negative n-offsets
  (towards the interior of S) of faces need not be considered when computing tentative faces for
  S↑r", "Concave edges need not be n-offset for S↑r. Similarly, convex edges are not needed for S↓r.
  BRep edges that separate tangent faces are not singular curves ... and therefore never need to be
  n-offset", and singular points that B-reps omit (cone apexes) must be added. Boundary evaluation is then:
  generate tentative faces → intersect them pairwise (a grid of u- and v-generator lines/circles of one
  surface intersected with the other by solving degree ≤ 4 equations, both families needed or
  near-parallel intersections are missed, then interpolate the points with a piecewise-circular curve) →
  classify tentative edges against the offset solid by a curve/solid classification that segments the
  curve at its crossings with "superfaces" and classifies a midpoint, using the point/set distance
  d(p, S) = r test. Only superfaces, not the body's actual faces and connectivity, are used.
- Limitations stated in the paper: natural quadrics and tori only; canal surfaces round free-form edges
  approximated; self-intersections of n-offsets are treated by pairwise comparison of the approximating
  cylinders/tori; the classification is CSG-style (no maintained B-rep topology); it was an experimental
  Pascal modeller.
- Relevance to cadrs: `offset_round`, `slab`. The slab ∪ tube ∪ ball definition is exactly this paper's
  n-offset superset, so (1) tubes are needed only on convex edges for outward offsets and only on
  concave edges for insets, never on tangent (smooth) edges, and balls only at vertices that are singular
  in that convexity — pruning the boolean count; (2) the paper's "classify tentative pieces by distance
  to the original body" is the alternative to n sequential booleans: generate all tentative faces, split
  them in one arrangement, keep pieces whose sample point is at distance exactly r from the body
  (`distance::min_distance` already exists).

#### A2. Forsyth 1995 (paywalled; description from secondary sources only)
- Citation: Mark Forsyth (D-Cubed Ltd, Cambridge), "Shelling and offsetting bodies", Proc. 3rd ACM
  Symposium on Solid Modeling and Applications (SMA '95), pp. 373–381, Dec. 1995. DOI
  [10.1145/218013.218088](https://dl.acm.org/doi/10.1145/218013.218088). No open-access PDF found
  (ACM DL returned HTTP 403 to the fetcher; the DOI page is paywalled). Not downloaded.
- What is verifiable: Hoffmann & Shapiro's handbook chapter cites it as a primary reference for
  "Offsetting and shelling [as] global solid operations, considered in [BW89, For95, PS95, RSB96]" —
  [Hoffmann & Shapiro, ch. 57 PDF](https://www.csun.edu/~ctoth/Handbook/chap57.pdf) (local copy
  `Hoffmann-Shapiro-2017-solid-modeling-handbook-chapter.pdf`). The SMLib manual describes shelling as
  "a few slight modifications to the Offset Solid algorithm, including allowing a zero offset distance
  for a face" and warns that "as the offset distance increases, there may be massive changes in the
  topology of the object" — [SMLib manual (search snippet; the page itself now redirects to the NVIDIA
  Omniverse hub)](https://docs.nvidia.com/smlib/manual/smlib/shell-offset/index.html).
- Unverified recollection (not checked against the paper; treat as a hypothesis): Forsyth's method
  offsets each face's surface, uses the body's edge/vertex adjacency to decide which offset surfaces to
  re-intersect, and inserts rolling-ball (cylinder/sphere/pipe) faces where neighbouring offset surfaces
  separate at convex edges and vertices, i.e. the "re-intersect offset surfaces" strategy with
  topology-guided gap filling. The report writer should not state this as fact.
- Relevance to cadrs: `local` (sharp offsets) and the shell builder; it is the paper to obtain (library
  access) before redesigning `offset_round` around re-intersection instead of booleans.

#### A3. Chiang, Hoffmann & Lynch 1992
- Citation: Ching-Shoei Chiang, Christoph M. Hoffmann, Robert E. Lynch, "How to compute offsets
  without self-intersection", Proc. SPIE 1830, Curves and Surfaces in Computer Vision and Graphics III
  (1992); also Purdue CS Technical Report 911 (1991) — [Purdue e-Pubs](https://docs.lib.purdue.edu/cstech/911).
- Open-access PDF: https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chiang-International_society_for_optics_and_photonics-1992-How_to_Compute_Offsets_Without_Self_Intersection.pdf
  — local copy `Chiang-Hoffmann-Lynch-1992-how-to-compute-offsets-without-self-intersection.pdf`
- Algorithm (from the PDF): distinguishes the local offset (each point moved along its normal) from the
  global offset (points at global distance d, the "grass fire" front). The accepted strategy is "compute
  the local offset, then remove the parts that are not at global distance d by trimming branches or
  regions bounded by self-intersections"; the paper instead integrates the global distance function in
  the ambient space up to distance d and extracts the global offset as a level set. It proves that the
  self-intersections where local offset regions border global ones lie on the medial axis (skeleton),
  which is why trimming criteria are hard in general.
- Limitations: requires discretising the ambient space (one dimension higher than the offset);
  approximate; no B-rep output.
- Relevance to cadrs: `offset_round` validity reasoning only — the medial-axis statement tells you
  that an offset by d is topology-preserving if d is below the local feature size, which is the test
  behind "the offset distance exceeds a concave radius".

#### A4. Seong, Elber & Kim 2006 — MUST READ
- Citation: Joon-Kyung Seong, Gershon Elber, Myung-Soo Kim, "Trimming local and global
  self-intersections in offset curves/surfaces using distance maps", Computer-Aided Design 38(3)
  (2006) 183–193. DOI [10.1016/j.cad.2005.08.002](https://doi.org/10.1016/j.cad.2005.08.002).
- Open-access PDFs (authors' pages): https://gershon.cs.technion.ac.il/papers/offset_srf_trim.pdf and
  https://cggc.cs.technion.ac.il/files/gallery-pdfs/Elber-1-s2.0-S0010448505001491-main.pdf — local copies
  `Seong-Elber-Kim-2006-trimming-self-intersections-offset-surfaces-distance-maps.pdf`,
  `Seong-Elber-Kim-2006-trimming-offsets-CAD-published.pdf`
- Algorithm (from the PDF): an offset point O_d(r,t) is invalid iff some surface point S(u,v) lies
  closer than d to it. Form the squared distance F(u,v,r,t) = ‖S(u,v) − O_d(r,t)‖² − d² as a rational
  function; the self-intersecting regions of the offset are the projection onto the (r,t) plane of the
  zero set F = 0 in 4-D, whose boundary is the uv-silhouette of the zero set, i.e. the simultaneous
  solution of F = 0, F_u = 0, F_v = 0 (three polynomial equations in four unknowns, solved by Bernstein
  subdivision). Global self-intersections appear as matching pairs of zero-set components; the trivial
  solution (u,v) = (r,t) is divided out. A numerical marching post-process refines the trimming curves.
  Extends Elber's earlier curve-offset trimming.
- Limitations: one freeform (B-spline) surface at a time, not a B-rep; multivariate constraint solving
  is expensive; only polynomial/rational inputs.
- Relevance to cadrs: `local` (sharp offsets of B-spline faces) and `slab` — a direct test of which uv
  region of an offset face survives, replacing a boolean against the body for free-form faces; the F_u =
  F_v = 0 silhouette formulation is reusable with the kernel's existing contouring in `ssi`.

#### A5. Parasolid offsetting / hollowing documentation — MUST READ
- Citation: Siemens Parasolid v12.0 Functional Description, ch. 12 "Local Operations" and ch. 19 "Local
  Ops: Hollowing & Offsetting"; Parasolid V35 Functional Description ch. 58 "Offsetting", ch. 59
  "Hollowing", ch. 60 "Thickening", ch. 62 "Local operations" (third-party mirror of the official
  documentation). URLs: [v12 ch. 12](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.13.html),
  [v12 ch. 19](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.20.html),
  [V35 ch. 58](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.059.html),
  [V35 ch. 59](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.060.html),
  [V35 ch. 60](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.061.html),
  [V35 ch. 62](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.063.html). HTML only; not
  downloaded.
- Findings (quoted from the pages):
  - Self-intersection methods (v12): "Accurately – the self-intersection is removed creating a new edge
    where the accurate surface self-intersects (the default method)"; "Approximately – the
    self-intersection is removed and replaced with a new approximate offset surface around the region of
    the self-intersection"; the accurate method "guarantees that the resulting offset surface remains
    accurate, but may leave sharp edges". V35: "Faces whose offsets are completely self-intersecting are
    healed by intersecting neighboring surfaces. Faces whose offsets are partially self-intersecting are
    healed using ... Trim: Self-intersections are removed by trimming the offset surface. Repair: Any
    remaining self-intersections are healed by constructing a B-surface patch which covers the hole around
    the region of self-intersection. Extend: Any remaining holes are healed by extending and intersecting
    neighboring surfaces" (`PK_offset_method_sx_trim_c`, `_sx_repair_1_c`, `_sx_repair_2_c`; repair_2 recommended).
  - Distance beyond the radius: "If the offset distance is greater than the minimum radius of curvature
    of surface, then that surface becomes either wholly inside-out or self-intersecting. In the former
    case Parasolid simply deletes the face. For example blends whose radius is less than the offset
    distance are removed from the model ... If the surface is either a swept or spun surface, then
    Parasolid extracts that region of the surface which becomes self-intersecting. This generally
    introduces a new edge for each self-intersection ... In other cases Parasolid attempts to delete the
    whole face. The success of this depends on the configuration of the model." V35 adds that "if you
    offset a blend face by a distance greater than the blend radius, it will be deleted", and faces that
    are only partially tightly curved are also deleted (trackable with `track_del`).
  - Topology changes a local operation may make: vertex splitting ("A multi-edge vertex can be split,
    and a new edge introduced"), repair of self-intersecting faces (edges cross; "Repair often involves
    the removal of faces"), repair of clashing faces (e.g. offsetting a chamfer face far enough removes
    the chamfer), repair of mirrored faces ("faces to drift past each other"), and new edges on sheet
    bodies with degeneracies (an offset cone). The same "common algorithm" serves hollowing, offsetting,
    tapering, tweaking surfaces and transforming faces.
  - Gaps between offset faces: "Parasolid created new edge geometry where the offset surfaces
    intersected. For outward offsets (positive) this can mean extending the underlying surface(s) so that
    the offset surfaces meet"; "offset steps" (side faces perpendicular to the offset faces) are created at
    smooth boundary edges between offset and non-offset faces (`offset_step`), which "splits the end
    vertices of those edges"; `ortho_vx_split` chooses whether a split laminar vertex is trimmed back
    to the laminar edge or split orthogonally; `grow` chooses which of two overflowing features of
    different convexity is grown to heal the model; `blend_edges`/`blend_radius` put blends on offset
    edges; `fix_degens` repairs surface degeneracies before offsetting; edge/vertex tolerances can be
    capped (`edge_limit`, `vertex_limit`); offsetting sheet boundaries may produce tolerant edges
    (SP-curves approximated within the supplied tolerance).
- Limitations: Parasolid "does not allow total freedom of topology changes" and "is not guaranteed to
  support these topology changes for every possible configuration".
- Relevance to cadrs: `local`, `slab`, `surfacing` (thicken). This is the reference behaviour for the
  sharp-offset path: delete faces whose offset is inside-out (d > min radius), cut swept/spun faces at
  their self-intersection with one new edge, extend neighbours to close gaps, and split >3-edge
  vertices — rules that let the sharp path replace booleans for most inputs.

#### A6. OpenCASCADE offset documentation
- Citation: Open CASCADE Technology, "Modeling Algorithms" user guide, section on offsets and shelling
  (`BRepOffsetAPI_MakeOffsetShape`, `BRepOffsetAPI_MakeThickSolid`, `BRepOffset_MakeOffset`) —
  [user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html),
  [BRepOffset_MakeOffset reference](https://occt3d.com/dev/doc/refman/html/class_b_rep_offset___make_offset.html),
  [MakeOffsetShape CDL (older source)](https://diyhpl.us/cgit/oce/tree/src/BRepOffsetAPI/BRepOffsetAPI_MakeOffsetShape.cdl).
- Findings: two algorithms. The "analytical continuation" method (join type `Intersection`): "At the
  first step, the offsets are computed. After this, the analytical continuations are computed for each
  offset. Pairwise intersection is computed according to the original topological information. The
  offset shape is assembled." The simple method: "Each surface is mapped to its geometric offset
  surface", whose "possible drawback ... is that it leads, in general case, to tolerance increasing".
  Shelling "rounds or intersects adjacent faces along its edges depending on the convexity of the edge"
  (join `GeomAbs_Arc` is the default, `Intersection` the alternative). `CheckInputData` requires a
  non-null offset, a connected offset shell, continuous input surfaces and normals on the grid. The
  older CDL warns: "the offset value should be sufficiently small to avoid self-intersections", "all
  faces should be based on surfaces with at least C1 continuity", and the algorithm "inherits the
  limitations of the surface intersection algorithm" — [search summary of the CDL](https://diyhpl.us/cgit/oce/tree/src/BRepOffsetAPI/BRepOffsetAPI_MakeOffsetShape.cdl).
- Relevance to cadrs: `local`, `offset_round`. OCCT's Arc join is the per-edge pipe + per-vertex sphere
  construction built directly (no booleans) with intersections only between neighbours, which is why its
  offset cases are cheap; cadrs already beats it on "2D offsets and offset shapes 0.17", so the lesson
  is mostly about failure modes, not speed.

#### A7. Patrikalakis & Maekawa — singularities and self-intersections of offset surfaces
- Citation: N. M. Patrikalakis, T. Maekawa, "Shape Interrogation for Computer Aided Design and
  Manufacturing", Springer 2002, ch. 11 (web hyperbook), and MIT OCW 2.158J lecture 13 (2003) —
  [hyperbook ch. 11.3](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node223.html),
  [lecture notes PDF](https://ocw.mit.edu/courses/2-158j-computational-geometry-spring-2003/3a77ce2173073840510f548601cd36b4_lecnotes13.pdf)
  — local copy `Patrikalakis-Maekawa-2003-MIT-lecture-13-offset-curves-surfaces.pdf`.
- Findings: the offset's principal curvatures are 1/(ρ_i + d) (inverting the signed principal radius plus
  the signed distance), so the offset is irregular where a principal radius equals −d; "the cutter radius
  must not exceed the smallest concave principal radius of curvature of the surface to avoid gouging".
  Self-intersections are pairs (s,t), s ≠ t, with r(s) + d·n(s) = r(t) + d·n(t), solved by the
  Bernstein-subdivision IPP solver after dividing out the trivial factor s = t; self-intersection curves of
  offset surfaces are traced by ODEs whose direction is perpendicular to the two progenitor normals
  (first derived by Aomura & Uehara). The lecture notes work out self-intersections of offsets of explicit
  quadratic surfaces.
- Relevance to cadrs: `local` — a cheap per-face pre-test (minimum concave principal radius < |d| ⇒
  the face's offset is inside-out or self-intersecting) that decides between "delete face", "cut at
  self-intersection" and "offset normally" before any intersection is attempted.

#### A8. Cao et al. 2024 — robust mitred/round offset of meshes
- Citation: Hongyi Cao, Gang Xu, Renshu Gu, Jinlan Xu, Xiaoyu Zhang, Timon Rabczuk, Yuzhe Luo, Xifeng
  Gao, "Robust and Feature-Preserving Offset Meshing", arXiv:2412.15564 (Dec. 2024; the PDF carries an
  ACM-format placeholder DOI). — [arXiv](https://arxiv.org/abs/2412.15564)
- Open-access PDF: https://arxiv.org/pdf/2412.15564 — local copy
  `Cao-et-al-2024-robust-feature-preserving-offset-meshing.pdf`
- Algorithm (from the PDF): generate each vertex's offset points with a feature-capturing energy, build
  "an offset polyhedron for each of its triangle, edge and vertex", convert the polyhedron set to an
  intersection-free triangle soup "by filtering out those triangles not part of" the offset (exact
  arithmetic in the critical predicates, floating point elsewhere), fill the holes left by the
  acceleration step, then build connectivity. Supports inward and outward, constant and variable
  distances, and claims "the first-ever solution for mitered offset that can well preserve sharp
  features"; evaluated on Thingi10K.
- Limitations: meshes only; no exact surfaces.
- Relevance to cadrs: `offset_round`, `slab` — it is the discrete twin of slab ∪ tube ∪ ball, but it
  resolves the union in one arrangement + classification pass rather than n booleans, which is the
  restructuring most likely to make rounded offsets cheap. (A companion paper by the same group,
  "A Parallel Feature-preserving Mesh Variable Offsetting Approach Based On Dynamic Programming",
  arXiv:2310.08997, is downloaded as
  `Cao-Xu-et-al-2023-parallel-feature-preserving-mesh-variable-offsetting.pdf`; its offset positions come
  from dynamic + quadratic programming rather than distance fields; not analysed further.)

#### A9. Jung, Shin & Choi 2004 — self-intersection removal in mesh offsets
- Citation: Wonhyung Jung, Hayong Shin, Byoung K. Choi, "Self-intersection removal in triangular mesh
  offsetting", Computer-Aided Design and Applications 1(1–4) (2004) 477–484 — [KAIST record](https://koasas.kaist.ac.kr/handle/10203/139556);
  Korean version "옵셋팅을 위한 정규 삼각망 추출" (KSCAM 2004) PDF
  https://koreascience.kr/article/JAKO200413842032780.pdf — local copy
  `Jung-Shin-Choi-2004-regular-triangular-net-extraction-for-offsetting-korean.pdf` (English abstract).
- Algorithm: region growing from a seed valid triangle; the valid region is grown to neighbours until a
  self-intersection segment is met, then crosses it to the adjacent valid sub-triangle, so "detection of
  valid triangles is performed only in valid regions and their adjacent self-intersections, and omitted in
  the rest regions (invalid regions)"; self-intersections are found with k-d tree bucketing; output is a
  regular 2-manifold net.
- Relevance to cadrs: not B-rep; useful only if a mesh-level offset is wanted for previews or for
  `meshsolid`. The "grow from a known-valid seed and never traverse the invalid region" idea does
  transfer to face-piece classification after an arrangement.

#### A10. Chen, Panozzo & Dumas 2019; Woerl, Schwanecke & Schoemer 2020 — discrete/GPU offsets
- Zhen Chen, Daniele Panozzo, Jérémie Dumas, "Half-Space Power Diagrams and Discrete Surface
  Offsets", IEEE TVCG (2019), arXiv:1804.08968 — PDF https://arxiv.org/pdf/1804.08968, local copy
  `Chen-Panozzo-Dumas-2019-half-space-power-diagrams-discrete-surface-offsets.pdf`. Dexel (ray)
  representation; "a two-stage sweeping procedure ... entirely avoiding volumetric distance field
  computations"; trivially parallel; approximate, for interactive fabrication.
- Ann-Christin Woerl, Ulrich Schwanecke, Elmar Schoemer, "Variable-Radius Offset Surface Approximation
  on the GPU", WSCG 2020, CSRN 3001 — PDF https://dspace.zcu.cz/bitstream/11025/38456/1/G07.pdf,
  local copy `Woerl-Schwanecke-Schoemer-2020-variable-radius-offset-surface-GPU.pdf`. Per-triangle
  ray-interval unions on the GPU, output through marching cubes; generalises to Minkowski sums.
- Relevance to cadrs: low; an approximate offset for validity previews only.

#### A11. Surveys and classical references (no open-access PDF)
- Takashi Maekawa, "An overview of offset curves and surfaces", Computer-Aided Design 31(3) (1999)
  165–173 — [library record](https://ibl.kb.nl/articles/0000000010055/31/3); covers the post-1992
  literature (per [arXiv:1210.1863](https://arxiv.org/pdf/1210.1863)). Not downloaded (paywalled).
- B. Pham, "Offset curves and surfaces: a brief survey", Computer-Aided Design 24(4) (1992) 223–229 —
  cited by the MIT lecture notes as the pre-1992 survey. Not downloaded.
- R. T. Farouki, "Exact offset procedures for simple solids", Computer Aided Geometric Design 2(4)
  (1985) 257–279 — [library record](https://ibl.kb.nl/articles/0000000010359/2/4); US patent 5,402,531
  describes it as treating offset solids of simple shapes such as cylinders and cones and the
  difficulties for complex solids — [patent](https://patents.google.com/patent/US5402531). Not downloaded.
- Rossignac & Requicha, "Constant-radius blending in solid modelling" (1984) — the companion on
  blending via offsets; PDF https://faculty.cc.gatech.edu/~jarek/papers/Filleting.pdf, local copy
  `Rossignac-Requicha-1984-constant-radius-blending.pdf`.
- "Shelling Algorithm in Solid Modeling", Springer LNCS (2004), DOI 10.1007/978-3-540-30497-5_46 —
  only the title was retrievable (Springer redirected to a login page); authors and content unknown.
- Zou Qiang (Zhejiang Univ.), "A note on solid modeling: history, state of the art, future",
  arXiv:2302.14373 (2023, in Chinese) — local copy `Zou-2023-note-on-solid-modeling-history-chinese.pdf`;
  a history overview, not an algorithm paper.
- Hoffmann & Shapiro, "Solid modeling", ch. 57 of the Handbook of Discrete and Computational Geometry
  (3rd ed., 2017): "when offsetting a curve or surface by a fixed distance, the self-intersections must lie
  on the medial axis" — [PDF](https://www.csun.edu/~ctoth/Handbook/chap57.pdf).

### Inferences
- The kernel's rounded offset is a Minkowski sum with a ball; by Rossignac's rules it can drop the
  tubes on concave edges (outward), on convex edges (inward) and on all smooth edges, and the balls at
  vertices whose edges are all smooth or of the wrong convexity — probably a large share of the
  booleans on filleted parts.
- Parasolid's observable behaviour (delete inside-out faces, cut spun/swept faces at one new edge, extend
  neighbours, split vertices) is consistent with the "re-intersect offset surfaces" strategy plus local
  repair; nobody documents a slab-boolean strategy in a production kernel, which suggests the
  re-intersection path is the one to make the default and the boolean path the fallback.
- The cheapest reliable scheme for valid B-rep offsets, judging by the sources, is: (1) per-face
  curvature pre-test (A7), (2) offset analytic faces exactly and B-spline faces with the distance-map test
  (A4), (3) re-intersect neighbours guided by the original edges, inserting pipe/sphere faces where
  convex neighbours separate (A1/A6), (4) a single arrangement-and-classification pass instead of
  sequential booleans when the topology changes (A8), (5) fall back to booleans.

### Gaps
- Forsyth 1995 could not be read; its actual algorithm is unverified.
- No ACIS (Spatial) documentation on `api_offset_body` / shelling was found at all.
- No paper by "Zhao et al., Offsetting of solids" or "Zhang & Yan, Fast and robust offset" was found;
  the closest matches are Peternell–Pottmann–Steiner–Zhao (swept volumes, C6) and Cao et al. (A8).
- The SMLib manual page is dead (redirects to NVIDIA Omniverse); only a search snippet survives.

---

## (b) Shelling / thin walls and their failure cases

### Takeaway
Shelling is documented everywhere as "offset every face by −t (or +t), pierce faces by 0, then keep the
material between the original and offset boundary"; the failure cases are the offset failure cases plus
the pierce-face trimming: faces that vanish (radius < t), adjacent offset faces that no longer intersect,
tangent pierce faces that need side faces, and large topology changes.

### Cited Findings
- B1. Parasolid hollowing (v12 ch. 19 / V35 ch. 59): `PK_BODY_hollow_2` "hollows a solid body by
  offsetting all the faces in the body by a specified distance"; `PK_FACE_hollow_3` offsets chosen faces
  by chosen distances; "Offset values may be zero, in which case the faces are treated as pierce faces";
  "each pierce face is reduced in size by performing a local boolean using any shared edges in their
  pre-offset and offset positions, so as to leave only the solid portion of the pierce face that remains
  after hollowing"; "A tangent pierce face is a pierce face that shares at least one smooth edge with
  another face that is not a pierce face ... it needs to be identified, so that side faces can be created
  along the smooth boundary edges" (`pierce_tangent`, and then all other offsets must be equal); V35 adds
  step faces along smooth edges, local hollowing, blends on offset edges, and the `grow` control for
  overflowing faces — [v12 ch. 19](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.20.html),
  [V35 ch. 59](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.060.html).
- B2. Parasolid thickening (V35 ch. 60): "thickening a sheet may create self-intersecting surfaces on
  either the side face, or the offset face itself ... Self-intersecting side faces are repaired
  automatically. Parasolid replaces the self-intersecting section with a new surface ... Self-intersecting
  offset surfaces are repaired by constructing B-surface patches, if the offset_method is set to
  PK_offset_method_sx_repair_2_c"; options for a punch direction for side faces, user-supplied side
  surfaces and triangular side faces — [V35 ch. 60](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.061.html).
- B3. T-FLEX CAD (a Parasolid-based application) documents the failure cases plainly: "an offset surface
  is created for each face of the source sheet or solid body, at the specified offset distance"; some
  faces "may vanish during offsetting. The system handles some cases, but many vanishing faces, or several
  adjacent ones, cause an error"; "The result depends on intersection lines between adjacent offset faces.
  In some geometries, two neighboring offset faces don't intersect, and the calculation fails"; a "Pierce
  smooth surfaces" switch handles removed faces tangent to fillets — [T-FLEX shell help](https://www.tflex.com/help/eng/T-FLEX%20CAD/15/shell.htm).
- B4. OCCT: `BRepOffsetAPI_MakeThickSolid` hollows a solid by offsetting its faces except the given
  closing faces (`AddFace`: "Add Closing Faces, <F> has to be in the initial shape S"); the user guide says
  shelling "rounds or intersects adjacent faces along its edges depending on the convexity of the edge";
  `MakeThickSolidBySimple` builds a solid between a shell and its simple offset —
  [user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html),
  [reference](https://occt3d.com/dev/doc/refman/html/class_b_rep_offset___make_offset.html).
- B5. Rossignac & Requicha 1986 define the thin wall implicitly: the shell of thickness t is
  S −* (S↓t); the rounding/filleting identities R_r = (S↓r)↑r, F_r = (S↑r)↓r give rounded cavities for free
  — [PDF](https://faculty.cc.gatech.edu/~jarek/papers/Offsets.pdf).
- B6. Shapr3D/Onshape user guidance (secondary, forum/vendor): a sweep or shell fails when a path radius
  is "smaller somewhere than half of the width of the swept cross-section" — [Shapr3D forum](https://discourse.shapr3d.com/t/loft-sweep-error-swept-body-has-self-intersection/13041);
  Autodesk Fusion reports "The operation would cause a large topology change" for shells —
  [Autodesk forum](https://forums.autodesk.com/t5/fusion-design-validate-document/hollow-out-body/td-p/5474095).

### Inferences
- cadrs's "cavity built from its definition: the body less each kept face's slab, tubes and balls" is
  B5 implemented with booleans; Parasolid's pierce-face rule (trim each pierce face by a local boolean
  between its pre-offset and offset edges) is the local alternative that avoids a full boolean for open
  shells.
- The two documented failure classes (vanishing faces, non-intersecting neighbours) map to the two
  pre-tests in (a): curvature radius < t, and neighbours whose offset surfaces diverge (convex edges
  outward) which need a pipe face rather than an intersection.

### Gaps
- No academic paper on shelling beyond Forsyth 1995 (paywalled) and the unidentified 2004 LNCS paper
  was found; vendor documentation is the only detailed public source.

---

## (c) Sweeping profiles along curves

### Takeaway
The published state of the art for frames is the double-reflection RMF (fourth-order, the method cadrs
already uses); the open problems are corner handling (OCCT documents the same two joins cadrs has and
the exact conditions under which they fail), self-intersection of the swept surface (local
self-intersection when a profile point lies beyond the path's centre of curvature; Adsul et al. give a
point-query invariant for solid sweeps), and swept-volume boundaries for solids (envelope theory and
Jacobian rank deficiency). Exact sweeps of lines and arcs giving analytic surfaces are standard (pipe
surfaces along lines and circles are cylinders and tori: Rossignac 1986).

### Cited Findings

#### C1. Wang, Jüttler, Zheng & Liu 2008 — MUST READ
- Citation: Wenping Wang, Bert Jüttler, Dayue Zheng, Yang Liu, "Computation of rotation minimizing
  frames", ACM Transactions on Graphics 27(1), article 2 (2008) — [JKU bibtex](https://www.ag.jku.at/bibtex/jue08b.txt),
  [HKU repository](https://hub.hku.hk/handle/10722/152386). (The HKU PDF link returned non-PDF content;
  the downloaded copy is the HKU CS Tech Report TR-2007-07 version, "Computation of Rotation Minimizing
  Frame in Computer Graphics".)
- Open-access PDF: https://numerika.fmf.uni-lj.si/bib2html/e-documents/clanki/Computation_of_Rotation_Minimizing_Frame_in_Computer_Graphics.pdf
  — local copy `Wang-Juttler-Zheng-Liu-2007-computation-of-rotation-minimizing-frames-TR.pdf`
- Algorithm (from the PDF): given sample points x_i with unit tangents t_i and a frame U_0 = (r_0, s_0,
  t_0), "Step 1: Let R1 denote the reflection in the bisecting plane of the points x_0 and x_1. Use R1 to
  map U_0 to a left-handed" frame; Step 2 reflects that frame in the bisecting plane of its tangent image
  and t_1 to obtain U_1. The method is exact for spherical and planar curves (Lemma 4.1: on a sphere the
  outward normal is a rotation-minimizing vector), hence the O(h⁴) global error, "comparing favorably to
  the second order approximation error of two currently prevailing methods – the projection method by
  Klok and the rotation method by Bloomenthal, while all these methods have comparable per-frame
  computational cost", and matching 4th-order Runge–Kutta at lower cost. Section 6 covers extensions
  (RMF with boundary conditions, e.g. closed spines and prescribed end frames, via variational
  principles); the RMF is preserved by conformal maps, and the ruled surface x(u) + v·r(u) along an RMF
  vector is developable.
- Limitations: discrete; the error bound presumes samples with exact tangents; for closed spines an
  angular mismatch must be distributed (Section 6).
- Relevance to cadrs: `sweep::frames` — already implemented; what remains is (1) choosing the sample
  count from the O(h⁴) bound and the spine's curvature instead of a fixed step, (2) the boundary-condition
  extension for closed paths and for matching frames at corner joins, (3) the developable-ribbon
  property for exact sweeps of a line segment along an RMF.

#### C2. Klok 1986; Bloomenthal 1990 (the second-order methods)
- F. Klok, "Two moving coordinate frames for sweeping along a 3D trajectory", Computer Aided
  Geometric Design 3(3) (1986) 217–229 — projection method; the central problem is "defining a triple of
  coordinate axes at each point of the trajectory to specify the position and orientation of the swept
  object" — [SciSpace author record](https://scispace.com/authors/f-klok-3s9b1eapyy). No OA PDF.
- J. Bloomenthal, "Calculation of reference frames along a space curve", Graphics Gems (1990) —
  rotation method; cited in C1 as reference [6]. No OA copy found.
- Relevance: historical; C1 supersedes both.

#### C3. Siltanen & Woodward 1992
- P. Siltanen, C. Woodward, "Normal orientation methods for 3D offset curves, sweep surfaces, skinning",
  Proc. Eurographics 1992, pp. 449–457 — existence confirmed only through a citing paper
  ([arXiv:1406.0618](https://arxiv.org/pdf/1406.0618)); no abstract or PDF found. Not downloaded.

#### C4. OpenCASCADE `BRepOffsetAPI_MakePipeShell` (header documentation)
- Citation: OCCT, `BRepOffsetAPI_MakePipeShell.hxx` — [header mirror](https://eic-code-browser.sdcc.bnl.gov/lxr/source/include/opencascade/BRepOffsetAPI_MakePipeShell.hxx),
  [older CDL](https://diyhpl.us/cgit/oce/tree/src/BRepOffsetAPI/BRepOffsetAPI_MakePipeShell.cdl).
- Findings (quoted): frame modes are Frenet / "corrected Frenet", a fixed trihedron ("all sections will
  be parallel"), a fixed binormal ("Angular relations between the section(s) and <BiNormal> will be
  constant"), a surface support ("Sets support to the spine to define the BiNormal of the trihedron, like
  the normal to the surfaces"), an auxiliary spine ("For each Point of the Spine P, an Point Q is evaluated
  on <AuxiliarySpine>" by length ratio or by the normal plane; the normal is PQ, or chosen so the section
  stays in contact with the auxiliary spine, or the auxiliary spine becomes a boundary), and a discrete
  trihedron. Transition modes: `Transformed` (default) — "The pipe is 'transformed' at the fractures of
  the spine. This mode assumes building a self-intersected shell"; `RightCorner` — "Two pieces of the pipe
  corresponding to two adjacent segments of the spine are extended and intersected at a fracture of the
  spine", valid only "if intersection of two pieces of the pipe" is connected and planar, which "can be
  violated if the spine is non-linear in some neighborhood of the fracture"; `RoundCorner` — "The corner
  is treated as rotation of the profile around an axis which passes through the point of the" fracture
  (axis = cross product of the adjacent tangents per the CDL), and "will assuredly provide a good result
  only if a profile was set with option WithCorrection = True, i.e. it is strictly orthogonal to the
  spine". Profiles may be placed `WithContact` (translated onto the spine) and `WithCorrection` (rotated
  orthogonal to the tangent); `SetForceApproxC1` "attempt[s] to approximate a C1-continuous surface if a
  swept surface proved to be C0"; `SetMaxDegree`/`SetMaxSegments` bound the V degree and span count.
  Limitations: "Mode with auxiliary spine is incompatible with hometetic laws", "Mode with auxiliary spine
  and keep contact produce only CO surface".
- Relevance to cadrs: `sweep::corner`, `sweep::path`. cadrs's mitred and round joins are RightCorner and
  RoundCorner; the documented preconditions (extended pieces must meet in a single planar curve; the
  profile must be orthogonal for the round join) are the checks to run before choosing a join, and the
  "Transformed" fallback (accept a self-intersecting shell) is what OCCT does when they fail — one reason
  its pipe cases are faster: it does not try to make every corner valid.

#### C5. Abdel-Malek, Blackmore & Joy 2006 — swept-volume survey
- Citation: Karim Abdel-Malek, Denis Blackmore, Kenneth Joy, "Swept volumes: foundation, perspectives,
  and applications", International Journal of Shape Modeling 12(1) (2006) 87–127 — [NJIT record](https://digitalcommons.njit.edu/fac_pubs/18963)
  (a thesis bibliography gives 2000, vol. 11; the library record's 2006, vol. 12 is used here).
- Open-access PDF (author's page, submitted manuscript): https://user.engineering.uiowa.edu/~amalek/papers/swept-volume-review.pdf
  — local copy `Abdel-Malek-Blackmore-Joy-2006-swept-volumes-foundation-perspectives-applications.pdf`
- Findings: classifies methods as envelope theory, the sweep-envelope differential equation (SEDE),
  Jacobian rank deficiency (singularity theory), voxel methods, and free-form solid sweeping. Envelope
  theory ("instantaneous screw axis theorem") gives characteristic curves — the point sets on the
  boundary of the generator at each time — and for the natural quadrics (planes, cylinders, cones,
  spheres) these were derived in closed form; the faceted swept volume is "established by warping the
  corresponding characteristic curves together with the partial boundaries of the generator at the
  initial and final positions". The rank-deficiency method equates the determinant of the sweep
  Jacobian to zero to find the singular set that bounds the volume. Older implementations compute the
  "necessary boundary" at several steps and join them.
- Relevance to cadrs: `sweep::body` (sweeping sheets and solids), `evolved` — background for swept
  volumes of solids; the closed-form characteristic curves of quadrics are the exact-sweep cases.

#### C6. Peternell, Pottmann, Steiner & Zhao 2005 — swept volumes by characteristic curves
- Citation: Martin Peternell, Helmut Pottmann, Tibor Steiner, Hongkai Zhao, "Swept Volumes",
  Computer-Aided Design & Applications 2(1–4) (2005). — PDF (author's page)
  https://www.math.uci.edu/~zhao/publication/mypapers/pdf/swept_volume.pdf — local copy
  `Peternell-Pottmann-Steiner-Zhao-2005-swept-volumes.pdf`
- Algorithm (from the PDF): at sampled times compute the characteristic set (points of ∂S whose
  velocity is tangent to ∂S, generally a curve with several components), giving points plus normals in a
  superset Γ of the swept boundary; a formula for the evolution speed of the characteristic curve picks
  the time step for uniform sampling; "A kind of marching algorithm on an underlying grid is used to trim
  away the part of the envelope surface Γ which lies in the interior of the swept volume. Special care is
  taken to correctly resolve edges"; output is a point-set surface, mesh or NURBS approximation. Unlike
  earlier work it needs no slicing step and no specific boundary representation, and trimming does not use
  surface/surface intersection.
- Limitations: approximate (grid), not a B-rep.
- Relevance to cadrs: `sweep::body` — a robust way to decide which parts of an exact swept surface
  are on the boundary (self-intersection trimming) when the exact envelope is known but its global
  trimming is hard.

#### C7. Adsul, Machchhar & Sohoni 2012 — procedural solid sweeps and self-intersection
- Citation: Bharat Adsul, Jinesh Machchhar, Milind Sohoni (IIT Bombay), "A procedural framework and
  mathematical analysis for solid sweeps", arXiv:1204.0908 (2012) — [arXiv](https://arxiv.org/abs/1204.0908)
- Open-access PDF: https://arxiv.org/pdf/1204.0908 — local copy
  `Adsul-Machchhar-Sohoni-2012-procedural-framework-solid-sweeps.pdf`
- Algorithm (from the PDF): for a solid whose faces meet smoothly, swept by a one-parameter rigid
  motion σ(t) = A(t)·x + b(t), the envelope is the image of the "funnel" F = {(u,v,t) : f(u,v,t) = 0}
  where f is the envelope condition (velocity · normal = 0) on each face's parameter domain; "the face
  structure of the envelope mimics locally that of the solid", so envelope faces, edges and vertices are
  derived from the solid's. The envelope is represented procedurally (evaluated on demand from the
  funnel, like de Casteljau for splines) rather than fitted. Local self-intersections (folds of the
  envelope) are detected by "a subtle mathematical invariant ... which is computationally benign and
  requires only point queries". Earlier methods (inverse trajectory membership tests, intersecting
  curves of contact at discrete times) are surveyed as expensive.
- Limitations: smooth face junctions assumed; procedural surfaces must later be fitted for exchange.
- Relevance to cadrs: `sweep::body` and `sweep::path` — the funnel formulation gives exact rails for
  solid sweeps, and the point-query fold test is a candidate self-intersection check for profile sweeps
  too (a profile point whose offset from the spine exceeds the spine's radius of curvature moves backwards:
  see C9).

#### C8. Bartoň, Pottmann & Wallner 2014 (reverse engineering of sweeps)
- Citation: M. Bartoň, H. Pottmann, J. Wallner, "Detection and reconstruction of freeform sweeps",
  Computer Graphics Forum 33(2) (Eurographics 2014) 23–32 — [EG DL](https://diglib.eg.org/handle/10.1111/v33i2pp023-032);
  PDF https://www.geometrie.tuwien.ac.at/ig/publications/2014/sweep2014/sweep2014.pdf — local copy
  `Barton-Pottmann-Wallner-2014-detection-reconstruction-freeform-sweeps.pdf`
- Decides whether a freeform surface is (approximately) the sweep of a planar profile: curve
  "movability" analysis, curvature-plot substring matching, exhaustive planar cuts with quick rejection,
  then global optimisation. Relevance: `exchange`/healing (recognising sweeps in imported STEP), not the
  sweep builder.

#### C9. Self-intersection conditions for profile sweeps (practitioner sources)
- "The self-intersection error appears when the sweep path has a smaller radius somewhere than half of
  the width of the swept cross-section" — [Shapr3D forum](https://discourse.shapr3d.com/t/loft-sweep-error-swept-body-has-self-intersection/13041);
  "there is some limit beyond which the inside of the profile is actually moving BACKWARDS along the
  sweep" and non-tangent joins make the inner edges intersect — [Autodesk forum](https://forums.autodesk.com/t5/fusion-design-validate-document/sweep-issue/m-p/7878640/highlight/true);
  Onshape: check the curve's minimum radius; "If your tube radius is bigger than the minimum, it will
  self-intersect and fail" — [Onshape forum](https://forum.onshape.com/discussion/17816/sweep-3d-fit-spline).
- US patent 9,817,924 B2 "Sweeping a profile along small circular arcs of a path" states that
  conventional CAD "requires the minimum radius of a path to be larger than the maximum radius of the
  profile" and proposes a construction that avoids the self-intersection in that case —
  [Google Patents](https://patents.google.com/patent/US9817924).
- Exact analytic sweeps: a circle swept along an edge is a canal surface, "closely related to right
  circular" cylinders, and canal surfaces along general curves are "approximated by tori and
  cylinders" (Rossignac & Requicha 1986, section 2.6/4.2) — [PDF](https://faculty.cc.gatech.edu/~jarek/papers/Offsets.pdf).
  OCCT's `GeomFill_Pipe` likewise special-cases "circular section of constant radius, constant section,
  and a section evolving between two given curves" — [user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html).

### Inferences
- The local self-intersection condition for a profile sweep with an RMF is: at spine parameter s with
  curvature κ(s) and curvature normal N(s), a profile point p (in the frame's normal plane) folds when
  κ(s)·(p · N(s)) ≥ 1, i.e. the point lies at or beyond the centre of curvature. This is the formal form of
  the practitioner rule in C9 and of Adsul's fold invariant; it is my derivation, not a quoted result.
- OCCT's pipe speed advantage likely comes from (1) a single approximation pass over the whole spine
  with a bounded degree/segment count rather than exact rails, and (2) not validating corners (the
  "Transformed" default accepts a self-intersecting shell). cadrs could keep exactness but cut cost by
  choosing frame samples from the O(h⁴) bound and by reusing the rails as iso-curves (already done).

### Gaps
- Siltanen & Woodward 1992 and Klok 1986 could not be read.
- No paper specifically on corner joins of multi-segment sweeps was found; OCCT's header is the only
  source describing the mitre/round join conditions.

---

## (d) Lofting / skinning with incompatible sections

### Takeaway
Exact skinning of incompatible sections (degree elevation + merging knot vectors) explodes the control
net to O(m·n²); the published remedies are approximate skinning (refit every section on a common
knot vector within tolerance and remove knots: Piegl & Tiller 1996/2000/2002, Park et al. 2000/2003,
Wang et al. 2008, Jha 2005), T-spline skinning (Li et al. 2015), and for guide curves the Gordon surface
in B-spline form (Siggel et al. 2019, TiGL): S = skin(profiles) + skin(guides) − tensor interpolant of the
intersection grid, which requires the network to be compatible (each profile meets each guide at the
same parameter) — achieved by reparametrisation, degree elevation and knot insertion.

### Cited Findings

#### D1. Siggel, Kleinert, Stollenwerk & Maierl 2019 — Gordon surfaces in B-spline form — MUST READ
- Citation: Martin Siggel, Jan Kleinert, Tobias Stollenwerk, Reinhold Maierl, "TiGL: An Open Source
  Computational Geometry Library for Parametric Aircraft Design", Mathematics in Computer Science 13
  (2019) 367–389; arXiv:1810.10795 — [arXiv](https://www.arxiv.org/abs/1810.10795), [DLR record](https://elib.dlr.de/124524)
- Open-access PDF: https://www.arxiv.org/pdf/1810.10795 — local copy
  `Siggel-et-al-2019-TiGL-Gordon-surface-curve-network-interpolation.pdf`; code Apache-2.0 at
  github.com/DLR-SC/tigl.
- Algorithm (from the PDF, section 3.2): N profile curves f_i(u) and M guide curves g_j(v) must satisfy
  the compatibility condition f_i(u_j) = s(u_j, v_i) = g_j(v_i), "i.e. all profile curves must intersect a
  guide curve at the same curve parameter, and all guides curves must intersect a profile curve at the same
  parameter". Gordon's surface is then S = S_f + S_g − T, where S_f skins the profiles, S_g skins the guides
  and T is the tensor-product interpolant of the intersection points α_ij, "a generalization of the
  Coons-patch method to more than" four boundary curves. For B-splines "it is required that all profile
  curves share the same degree and a common knot vector ... In practice, both can always be achieved by
  using degree elevation and knot insertion of the input curves"; the three control nets P, Q, T are found
  from three linear interpolation systems (eqs. 15–17), and "the interpolation of the intersection points
  (17) must use the same interpolation parameters, degrees and knot vectors as the interpolation of the
  profile curves (15) and the guide curves (16)". Section 3.2.3 (not read in full) handles curve-network
  compatibility, i.e. reparametrising the curves so the intersections fall at common parameters. The
  authors note only Ayam and SINTEF GoTools implement Gordon surfaces "but without addressing the curve
  network compatibility problem". TiGL's issue tracker states the network rules: "Every section curve's
  start and end point must lie on a guide curve and vice versa. A section curve must always intersect all
  guide curves" — [TiGL issue 987](https://github.com/DLR-SC/tigl/issues/987).
- Limitations: requires a complete, properly closed network; sensitive to parametrisation (a blog on
  Gordon surfaces calls spline Gordon surfaces "very sensitive to the parameterization" —
  [quaoar.su](https://quaoar.su/blog/page/stroim-poverhnost-gordona)); TiGL hard-crashes on an invalid
  network (issue 987).
- Relevance to cadrs: `loft_build` — the missing "loft with guide curves" feature; it builds on the
  existing exact skinning (`loft_pieces`) plus a tensor interpolation, both already available in
  `nurbs.rs`. OCCT also has `GeomFill_Gordon` (Boolean-sum formula with intersection detection and
  reparametrisation) — [OCCT 8.0 reference](https://dev.opencascade.org/doc/refman/html/class_geom_fill___gordon.html).

#### D2. Piegl & Tiller and the approximate-skinning family — MUST READ (no OA PDFs)
- L. Piegl, W. Tiller, "Algorithm for approximate NURBS skinning", Computer-Aided Design 28(9) (1996)
  699–706; "Reducing control points in surface interpolation", IEEE Computer Graphics and Applications
  20(5) (2000) 70–74; "Surface skinning revisited", The Visual Computer 18(4) (2002) 273–283, DOI
  [10.1007/s003710100156](https://doi.org/10.1007/s003710100156). Citations verified in the reference
  list of Li et al. 2015 (local PDF) and [dblp](https://dblp.dagstuhl.de/pid/75/3689.html).
- Method, as described by Li et al. 2015 (the originals are paywalled): the 2000 paper lets the user
  "interpolate or approximate the initial cross sections using knot removal algorithm", so "a significant
  number of control points can be eliminated under a given tolerance"; the 1996/2002 approach: "sample
  points from each cross section are obtained, and then approximated using a non-rational B-spline curve
  based on a candidate knot vector. After that, removable knots are eliminated if the corresponding error
  is small enough and new knots are inserted if the corresponding error exceeds the given tolerance".
  Exact skinning of n sections with m control points each costs O(m·n²) control points. The 2002 abstract:
  the fix "avoids all the anomalies at the expense of increasing the number of control points and the
  compute time" — [Springer abstract via unpaywall](https://unpaywall.org/10.1007%2Fs003710100156).
  Piegl's page gives a brush-handle example: 81 sections, 81 × 1455 control points by exact skinning
  reduced to 81 × 48 (96.7 %) — [USF research page](https://cse.usf.edu/~lespiegl/research.htm).
- Related (all Computer-Aided Design, cited from Li et al. 2015's references): H. Park, K. Kim, "Smooth
  surface approximation to serial cross-sections", 28(12) (1996) 995–1005; H. Park, K. Kim, S. C. Lee,
  "Method for approximate NURBS curve compatibility based on multiple curve refitting", 32(4) (2000)
  237–252 (refit on a common knot vector, reduce its size, energy term for stability); H. Park, "Lofted
  B-spline surface interpolation by linearly constrained energy minimization", 35(14) (2003) 1261–1268;
  W. K. Wang et al., "Reducing control points in lofted B-spline surface interpolation using common knot
  vector determination", 40(10–11) (2008) 999–1008. None downloaded (paywalled).
- Kailash Jha, "Energy Based Multiple Refitting for Skinning", International Journal of CAD/CAM 5(1)
  (2005) 11–18 — PDF https://koreascience.kr/article/JAKO200503018236487.pdf, local copy
  `Jha-2005-energy-based-multiple-refitting-for-skinning.pdf`: refits all sections on one knot vector by
  minimising an energy with terms up to the second derivative, reducing control points by binary search
  on the count for a given accuracy; motivated by the exact-then-reduce approach giving "poor qualities of
  surface when curves are irregular in parameterization".
- Relevance to cadrs: `loft_build` — the "sampled interpolation" path should become approximate
  skinning on a common knot vector with knot removal; it bounds control-net growth when sections have
  different numbers of curves or knots, and removes the parametrisation anomalies the 2002 paper
  catalogues.

#### D3. Li, Chen, Cai, Nasri & Zheng 2015 — T-spline skinning of incompatible sections
- Citation: Yusha Li, Wenyu Chen, Yiyu Cai, Ahmad Nasri, Jianmin Zheng, "Surface skinning using
  periodic T-spline in semi-NURBS form", Journal of Computational and Applied Mathematics 273 (2015)
  116–131. — PDF (AUB repository) https://scholarworks.aub.edu.lb/server/api/core/bitstreams/dcf3a534-c658-40bc-a4f0-574c05e08e2b/content
  — local copy `Li-Chen-Cai-Nasri-Zheng-2015-periodic-T-spline-skinning.pdf`
- Method: skin closed, incompatible sections with a periodic T-spline whose T-junctions absorb the
  differing knot vectors, with explicit formulae ("avoid solving a large system"), local modification and
  controllable smoothness across sections. Limitation: T-splines are not a STEP/IGES exchange type.
- Relevance: low for cadrs (NURBS-only kernel), but its introduction is the best short survey of the
  compatibility literature.

#### D4. Woodward 1986/1988
- C. D. Woodward, "Methods for Cross-Sectional Design of B-Spline Surfaces", Eurographics 1986, DOI
  [10.2312/eg.19861011](https://diglib.eg.org/handle/10.2312/eg19861011) — sweeps, skinning and surface
  interpolation with the CUBES package (the EG DL PDF download failed: not a PDF). The frequently cited
  C. D. Woodward, "Skinning techniques for interactive B-spline surface interpolation", Computer-Aided
  Design 20(8) (1988) 441–451, could not be verified from any source found — treat the citation as
  unconfirmed.

#### D5. OpenCASCADE `BRepOffsetAPI_ThruSections`
- [Reference page](https://occt3d.com/dev/doc/refman/html/class_b_rep_offset_a_p_i___thru_sections.html):
  `CheckCompatibility` computes "origin and orientation on wires to avoid twisted results" and makes the
  wires have the same number of edges; `ruled = true` gives ruled faces between consecutive wires,
  otherwise faces are "smoothed out by approximation" with `SetParType`, `SetContinuity`,
  `SetCriteriumWeight`, `SetMaxDegree` (U-degree cap); vertices allowed only as first or last section.
- Relevance: `loft_build` already matches this (ruled for two sections, exact or approximate otherwise).

### Inferences
- A loft with guide curves in cadrs is: reparametrise sections and guides so intersections share
  parameters (D1 §3.2.3), elevate/merge knots, skin sections (existing), skin guides (existing, other
  direction), interpolate the intersection grid as a tensor surface (banded interpolation exists),
  subtract. Tangency conditions at the ends are then boundary conditions on the two skins, as the kernel
  already supports for sampled lofts.

### Gaps
- Piegl & Tiller's three papers and Park's papers are paywalled; their descriptions here are second-hand
  (from Li et al. 2015 and Piegl's page).
- No source was found on lofting between sections with different numbers of curves beyond OCCT's "make
  the wires have the same number of edges" (which splits edges); the academic literature treats each
  section as one curve.

---

## (e) Drafts/tapers and face-replacement local operations

### Takeaway
Production kernels build a draft by replacing each drafted face's surface with an "isocline" surface
(every point's steepest tangent makes the draft angle with the pull direction) fixed along a reference
edge, face or surface, then re-intersecting the neighbourhood with the same local-operation engine used
for offsets; topology changes (splitting cylinders/cones, clashing faces, merged faces) are handled by a
bounded set of repair rules. OCCT's much narrower draft (planes rotated about their trace on the neutral
plane, cylinders turned into cones, propagated across tangent neighbours, no topology change allowed)
is cheap precisely because it is restricted. No academic paper on draft algorithms was found; the
"editability"/"local operations" literature is about feature re-evaluation and naming, not geometry.

### Cited Findings

#### E1. Parasolid draft / taper documentation — MUST READ
- Citation: Parasolid v12.0 Functional Description ch. 20 "Local Ops: Adding Draft", ch. 21 "Local Ops:
  Double-Sided Tapering", ch. 23 "Local Ops: Generic Face Editing" —
  [ch. 20](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.21.html),
  [ch. 21](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.22.html),
  [ch. 23](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.24.html).
- Findings (quoted): "There are two stages: analysis – divides the body into a set of faces requiring the
  addition of draft, and a set of faces which don't. Typically some faces are split; tapering – the
  surfaces of the faces which require drafting are replaced by isocline surfaces." Analysis uses
  `PK_FACE_imprint_cus_isoclin` to imprint isocline curves ("a boundary between an all steep region and a
  NOT steep region"; "Faces are deemed to be steep when, at all points, the face normal makes an angle
  with the draw direction which is greater than (90 − draft angle)"). "An isocline surface is a surface
  such that at any point on the surface the steepest surface tangent is at an angle of the draft angle to
  the draw direction." `PK_FACE_taper` takes a reference entity per face: an edge ("stays fixed while the
  other edges of the face are altered"), a face ("the face acquires the same isocline surface as the
  reference face" — adjacent faces are then merged), or a surface ("The intersection of the face with the
  reference surface stays fixed"); edges plus a surface give step tapering, where reference edges not on
  the surface "produce a new step face which extends from the reference edge to the tapered face" (step
  surfaces are either tapered or ruled "normal" surfaces perpendicular to the original face). A planar
  face drafted about an edge gets a plane; a curved one "a ruled B-surface". Methods:
  `PK_taper_method_isocline_c` (default; "fails if the taper condition is not satisfied"), `curve_c` and
  `surface_c` (work when the condition is not satisfied, "aesthetic tapering"), `offset_c` (reference
  curve projected onto a single top surface so it appears offset in the taper direction). `PK_BODY_taper`
  does double-sided tapering about a parting sheet body with "mitering" (adding material so faces meet at
  the parting surface). Topology: "tapering a cylindrical or conical face requires" splits and new faces;
  tapering can make faces clash, resolved by "splitting faces or creating new faces as required". Generic
  editing (`PK_FACE_change`) combines taper, offset, transform and surface replacement in one call and
  takes help geometry: "The solution that is chosen is the one whose topology has attached geometry that
  is closest to the supplied geometry" (`edge_help`, `vertex_help`).
- Relevance to cadrs: `local` (drafts, `Kernel::replace_faces`). The two-stage structure (imprint
  isocline curves, then replace surfaces and re-intersect) and the exact surface choices (plane for a
  planar face about an edge, ruled surface for curved faces, cone for a cylinder) are what to mirror; the
  help-geometry rule is the documented answer to "which crossing does the new surface's domain pick".

#### E2. OpenCASCADE `BRepOffsetAPI_DraftAngle`
- [Reference page](https://occt3d.com/dev/doc/refman/html/class_b_rep_offset_a_p_i___draft_angle.html):
  "This algorithm treats planar, cylindrical and conical faces"; a planar face "is tapered by inclining it
  through the angle Angle about the line of intersection" with the neutral plane; a cylindrical or conical
  face becomes conical and the taper propagates to tangent planar/cylindrical/conical neighbours, which
  "must also respect the above criteria"; "Direction indicates the side of NeutralPlane from which matter
  is removed if Angle is positive"; "Do not use shapes, which with a draft angle added to a face would
  modify the topology"; failures are reported per face (`ProblematicShape`, `Draft_ErrorStatus` for face,
  edge or vertex recomputation) and must be `Remove`d. The user guide: "intersection between the face
  and the neutral plane is invariant" — [user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html).
- Relevance to cadrs: `local` — the OCCT draft cases in the perf suite exercise exactly this analytic
  subset (plane → rotated plane, cylinder → cone, neighbours re-intersected as conics); a fast path for
  analytic faces with no topology change would remove most of the 1.9× gap, keeping the general
  SMLib-style path for the rest.

#### E3. Parasolid local operations overview (v12 ch. 12 / V35 ch. 62)
- "Local operations are generally used on solid bodies to change a face or faces"; hollowing,
  offsetting, tapering/drafting, tweaking surfaces and transforming faces "rely on a common algorithm"
  and "can perform a wide range of topology changes" — vertex splitting, repair of self-intersecting
  faces, clashing faces, mirrored faces, degenerate sheet bodies (see A5) —
  [v12 ch. 12](http://www.q-solid.com/Parasolid_Docs/chapters/fd_chap.13.html),
  [V35 ch. 62](http://www.q-solid.com/Parasolid_Docs_V35/chapters/fd_chap.063.html). Tag persistence
  during local operations is a separate section (12.3).
- Relevance: `local` — one engine for offset, draft, replace-face and move-face with a shared repair
  vocabulary, as cadrs already has; the repair classes are a checklist of what `Move face` currently
  refuses ("refuses topology changes", commit 57657d8).

#### E4. Replace-face descriptions (industry)
- Autodesk patent US 7,366,581 B2 "Replace face operator for solid body modeling": "selecting one or
  more outgoing faces on the solid target body and replacing the selected faces with one or more incoming
  faces from an operator body" (which may be a sheet body) — [Google Patents](https://patents.google.com/patent/US7366581).
  Algorithm text not read.
- KeyCreator "Shadow Taper": "introduces new ruled faces in order to remove regions from the nominated
  faces where the draft is shadowed along the pull direction"; it "detects special geometry cases on
  spheres, cones, and Torii and makes a cone or plane for the taper surface rather than a spline" —
  [KeyCreator help](https://help.kubotekkosmos.com/keycreator?id=107687); KeyCreator notes that its
  direct-edit features "will not change the topology of the model, with the exception of Thicken Sheet and
  Replace Face" — [KeyCreator help](https://kubotek3d.com/help?&id=A10105).
- Relevance: `local` — the analytic special cases (cone/plane instead of a spline draft surface) are
  the same speed lever as E2.

#### E5. Chen & Hoffmann 1995; Hoffmann & Juan 1992 (feature editing, not geometry)
- Xiangping Chen, Christoph M. Hoffmann, "On editability of feature-based design", Computer-Aided
  Design 27(12) (1995) 905–914, DOI 10.1016/0010-4485(95)00013-5; Purdue TR CSD-TR-94-067 PDF
  https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chen-Computer_aided_Design-1995-On_editability_of_feature-based_design.pdf
  — local copy `Chen-Hoffmann-1995-on-editability-of-feature-based-design.pdf`. Drafts are "modifying
  features" that "add chamfers and rounds to edges, or draft angles to a set of faces"; editing a feature
  re-evaluates later features against persistent names, with rules for failed or multiple matches.
- C. M. Hoffmann, R. Juan, "Erep: An editable high-level representation for geometric design", Purdue
  TR 1992 — PDF https://www.cs.purdue.edu/cgvlab/www/resources/papers/Hoffmann-CS_technical_reports-1992-Erep_An_Editable_High_Level_Representation_for_Geometric_Design.pdf
  — local copy `Hoffmann-Juan-1992-Erep-editable-representation.pdf`.
- Relevance: `cadrs_core` rebuild/naming rather than the kernel; no face-replacement geometry.

### Inferences
- Parasolid's isocline definition is the general draft; OCCT's draft is its restriction to faces whose
  isocline surface is again a plane or cone. Checking for that restriction first (planar or
  cylindrical/conical face, neighbours analytic, no topology change) and running the general local
  operation only otherwise is the obvious split.
- "Draft of faces adjacent to blends" is not documented anywhere found; the industry pattern visible in
  Parasolid's offset options (`blend_edges` re-blends offset edges) and T-FLEX's "pierce smooth surfaces"
  suggests the practical approach is: remove the blend, draft, re-blend — but that is an inference.

### Gaps
- No academic paper describing a draft/taper algorithm (isocline surface construction, re-intersection)
  was found; Parasolid's documentation is the only detailed public description.
- Nothing by Hoffmann & Joan-Arinyo or Gao & Shah on local operations surfaced; the searches returned
  product help and patents instead.
- Handling of drafts next to blends: no source.

---

## What I could not find

- Forsyth 1995 "Shelling and offsetting bodies" (ACM DL paywalled, HTTP 403 to the fetcher; no
  author/repository copy). Only the DOI, pages and secondary citations are recorded.
- Any public ACIS / Spatial documentation of its offset, shell or thicken algorithms.
- A readable SMLib shell/offset manual page (the NVIDIA-hosted page now redirects to the Omniverse
  documentation hub).
- The papers named as "Zhao et al., Offsetting of solids" and "Zhang & Yan, Fast and robust offset";
  no publication with those authors/titles surfaced.
- Open-access copies of Maekawa 1999, Pham 1992, Farouki 1985, Klok 1986, Bloomenthal 1990,
  Siltanen & Woodward 1992, Piegl & Tiller 1996/2000/2002, Park 2000/2003, Wang et al. 2008 (lofts),
  Woodward 1988 (whose citation itself could not be confirmed).
- The contents of "Shelling Algorithm in Solid Modeling" (Springer LNCS 2004).
- Any paper on corner joins of multi-segment sweeps, on draft/taper surface construction, or on drafting
  faces adjacent to blends.
- The web-search budget was exhausted before Stroud's "Boundary Representation Modelling Techniques"
  (Springer 2006; it has chapters on local operations and offsetting), Peternell & Pottmann's rational
  canal-surface parametrisations, Varadhan & Manocha / Lien Minkowski-sum offsets, and the
  Hoffmann book "Geometric and Solid Modeling" (the Purdue PDF URL I tried was not a PDF) could be
  checked; these remain unverified leads.
