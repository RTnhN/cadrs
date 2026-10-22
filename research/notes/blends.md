# Fillets, chamfers and vertex blends in B-rep kernels: algorithms and sources

Scope: rolling-ball (constant and variable radius) blends, chamfers, vertex blends, blend chains,
overflow and topology bookkeeping, for `cadrs_kernel::native` (`fillet`, `fillet_geom`, `blend`,
`blend_chain`, `blend_corner`). Today the kernel does local trimming with contact curves, sphere
patches at three-edge corners, setback patches elsewhere, and falls back to a subtracted tool body;
fillets run 9.2x (OCCT blend cases) and 10.3x (SMLib cases) slower than OCCT
([README](../../crates/cadrs_kernel/README.md), lines 184-185, 357).

Research conditions: the shared web-search budget ran out part-way through, so the second half of
this work was done by fetching known URLs (OCCT source on GitHub, Google Patents, Crossref and
Semantic Scholar metadata APIs, publisher and author pages). Elsevier/ScienceDirect, Springer,
the SZTAKI eprints, Cardiff ORCA and the Oslo DUO repository all refused or had no full text, so
most of the classical journal papers are described from their abstracts, from the sources that
cite them, or (where marked) from my background knowledge. The OCCT description is from the
actual source and is the most reliable part of these notes.

Downloaded PDFs (all verified to start with `%PDF`) are in
`../papers/blends/`; the local path is given with each entry below.

---

## Key question 1: Rolling-ball blend tracing (marching vs. offset SSI vs. closed form); step control; termination

### Takeaway
Every production-grade method traces the spine or the contact curves by marching with a Newton
corrector on a small nonlinear system (4 unknowns for two parametric faces), with step control by
chord/tangent angle and sagitta tests; the "offset-surface intersection" formulation is used as
the *definition* of the spine but is rarely computed as an SSI because offsets of free-form
surfaces are not rational. Analytic pairs (plane/plane, plane/cylinder, plane/cone with a line or
circle spine) are special-cased to exact cylinders and tori; cylinder/cylinder is *not* closed-form
in OCCT.

### Cited Findings
- OCCT's constant-radius blend function `BlendFunc_ConstRad` has unknowns X = (u1, v1, u2, v2) and
  4 equations, split 1 + 3: E(1) says the midpoint of the two contact points lies on the section
  plane through the guide point whose normal is the guide tangent; E(2..4) is the vector equation
  "ball centre reached from S1 (pts1 + ray1·n1) equals the centre reached from S2 (pts2 + ray2·n2)",
  with n1, n2 the surface normals projected perpendicular to the plane normal. Newton is
  `math_FunctionSetRoot` with 30 iterations; the Jacobian is analytic (`Derivatives`), and when the
  Gauss solve of the tangent fails it falls back to SVD (threshold 1e-6) and flags the point as a
  tangency ("istangent") point — [OCCT BlendFunc_ConstRad.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/BlendFunc/BlendFunc_ConstRad.cxx)
- OCCT's marching loop `Blend_Walking::InternalPerform`: the first step is `pasmax` (set by the
  fillet builder to 5% of the guide length; in `ChFi3d_Builder_2.cxx` `MaxStep = (bidl-bidf)*0.05/nbed`),
  or the previous step when continuing, floored at `100*tolgui`; on `Blend_StepTooLarge` the step is
  halved, on `Blend_StepTooSmall` it grows `min(1.5*stepw, pasmax)`; the walk stops when
  `|stepw| < tolgui`; the next guess is a linear extrapolation with the 2D tangents (`evalpinit`),
  falling back to the previous solution if it leaves the bounds — [OCCT Blend_Walking_4.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/Blend/Blend_Walking_4.gxx); [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx); [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx)
- OCCT's step acceptance (`TestArret`/`CheckDeflection`): points closer than `0.01*tolpoint3d` are
  `SamePoints`; the chord must make an angle below arccos 0.98 (about 11.5°) with both the previous
  and current 3D tangents (and below arccos 0.88 in 2D) else `StepTooLarge`; a negative cosine is
  `Backward` (treated as too large and sets a "rebrousse" flag); the sagitta estimate is
  `|t_prev − t_cur|²·|chord|/64`, with `≤ 0.25·fleche²` → `StepTooSmall`, `> fleche²` → `StepTooLarge`;
  `fleche` is 5% of the largest radius; an optional guard limits a step to 5% of each surface's
  parameter range ("Pas max : 5% du domaine") — [OCCT Blend_Walking_2.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/Blend/Blend_Walking_2.gxx); [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx)
- OCCT termination at a face boundary: when a solution classifies OUT/ON on a face, `Recadre`
  re-solves an inverse function (`BlendFunc_ConstRadInv`) on the restriction arc with Newton
  (tolerance `0.1*tolpoint3d`, 35 iterations), tries the neighbouring arc if it fails near an arc
  end, snaps to a vertex within 0.4 of the arc span (capped by vertex tolerance), and the state
  becomes `OnRst1`/`OnRst2`/`OnRst12` (both, when the two parameters agree within `10*tolgui`);
  the walk ends there (`MakeExtremity`), or with a singular extremity when the two contact points
  coincide within `2*tolpoint3d`. The guide is artificially extended by 1% at each end "to help
  rsnld" — [Blend_Walking_3.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/Blend/Blend_Walking_3.gxx); [Blend_Walking_4.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/Blend/Blend_Walking_4.gxx); [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx)
- OCCT's starting solution: `StartSol` samples about `max(10, 3*nbed)` points along the spine,
  tries the faces adjacent to each edge first, then neighbouring faces ("Now one tries the
  neighbor faces") with a section plane from the guide, and throws `StartSol echec` if none
  converges — [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx)
- OCCT's exact ("KPart") cases: plane/plane with a line spine, plane/cylinder (line or circle
  spine), plane/cone (circle spine only); every other pair throws
  `Standard_NotImplemented("particular case not written")` — cylinder/cylinder and cone/cone are
  walked numerically. Corner KParts: a toric corner when a plane meets a cylinder-type face, a
  sphere, and a "rotule" between three planes — [OCCT ChFiKPart_ComputeData.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFiKPart/ChFiKPart_ComputeData.cxx)
- OCCT's section shape options are `ChFi3d_Rational` (default, classical NURBS circle),
  `ChFi3d_QuasiAngular` (rational circle whose parametrisation matches the angle) and
  `ChFi3d_Polynomial`; the fillet surface is a sweep approximation of the sections along the
  walked line (`BRepBlend_AppSurf`/`AppSurface`, default max degree 11, max 50 segments,
  continuity set by `SetContinuity`, default `GeomAbs_C1`) — [OCCT BRepBlend API](https://pythonocc-documentation.readthedocs.io/en/review-gen-apidoc-rtd/apidoc/OCC.Core.BRepBlend.html); [OCCT BRepFilletAPI_MakeFillet.cdl](https://git.rbts.co/OpenCascade/occt/raw/commit/9cd1334737c3bba62d7df207ded13457bb1f0e6d/src/BRepFilletAPI/BRepFilletAPI_MakeFillet.cdl)
- OCCT splits a walked surface where the two contact curves pinch: after approximation,
  `maybesingular = (Func.GetMinimalDistance() <= 100*tolapp3d)` triggers `SplitSurf`, which finds
  local minima of the contact-point distance, refines them with `math_FunctionRoot` and inserts a
  new SurfData at each root — [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx)
- The spine of a constant-radius blend is "the intersection curve of the two surfaces formed by
  offsetting each base surface by the radius of the ball" (Klass & Kuhn 1992, as summarised in a
  citing patent); Farouki & Sverrisson 1996 "finds the spine curve by intersecting offset curves
  directly on the original surfaces, so it never has to build the offset surfaces, which are
  generally non-rational" (as described by later citing papers) — [US7518609 background](https://patents.google.com/patent/US7518609); [search summary of citing papers](https://www.scitepress.org/Papers/2010/28230/28230.pdf)
- The Autodesk "extended rolling ball" patent summarises the classical methods: Várady 1989 =
  "geometric marching over blending arcs"; Klass & Hermann = "geometric marching plus trimline
  tangent computation"; Choi & Ju 1989 = parametric base surfaces with multiple mapping and
  substitution; Rossignac & Requicha 1984 = approximation of blends as sphere, cylinder and torus
  sequences; all produce G1 fillets that stop abruptly at the end of the intersection curve —
  [US5774359](https://patents.google.com/patent/US5774359)
- Chandru, Dutta & Hoffmann (1989) define the spine as the intersection of the *Voronoi surface*
  of the two primaries with a reference surface and propose to "determine the spine curve
  numerically only, by tracing it in higher dimensions" (10-D: point, two foot points, distance),
  because explicit elimination of the Voronoi surface was intractable even for a cylinder and an
  inclined plane; they note the Voronoi surface of a plane and a cylinder or cone is always a cone,
  and conjecture a hyperbolic paraboloid for cone/cone, cone/cylinder, cylinder/cylinder —
  [Purdue TR CSD-TR-851](https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chandru-CS_technical_report-1989-On_Variable_Radius_Blending_Using_Dupin_Cyclides.pdf)
- Rossignac & Requicha's 1986 CSG approach "approximates the fixed radius blends by smoothly
  joined pieces of cylinders and tori" and the exact blend surface is "in general degree 16 or
  higher" — [Chandru TR](https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chandru-CS_technical_report-1989-On_Variable_Radius_Blending_Using_Dupin_Cyclides.pdf)
- A Japanese JSPE paper traces SSI curves (including those of offsets, for rolling-ball fillets)
  as solutions of ODEs "with automatic stepwise control" — [search summary of JSPE 57(8)](https://www.jstage.jst.go.jp/browse/jjspe/57/8/_contents/-char/ja)
- C3D (2025) added a rolling-ball method next to its older "rolling disk" method: for the
  rolling ball "the path curves are defined as the locus of the points where the ball touches the
  original surfaces" and "a directrix is not required"; a directrix is still required when the
  radius or section varies; fillet points are now "calculated with higher precision" and defects
  from "inaccurate definition" of fillet-to-face intersection points were removed —
  [C3D Labs 2025](https://c3dlabs.com/blog/products/c3d-geometric-kernel-new-features-and-development-trends/)
- C3D's fillet surface class `MbFilletSurface` (base `MbSmoothSurface`) is defined by two
  `MbSurfaceCurve` support curves, two distances, a section form `MbeSmoothForm`, a conic shape
  coefficient ("always non-null and sets the section between the conic limits or an arc"), an
  optional guide `curve0` (the intersection curve of the tangents, or a guide that only defines
  the parametrisation), and `weights0`; u runs along the mid-curve with accessors for the opening
  angle, local axis and centre point per u — [C3D MbFilletSurface](https://c3dlabs.com/doc/class_mb_fillet_surface.html)

### Inferences
- OCCT's structure (4-unknown Newton on sections normal to a guide curve, chord/sagitta step
  control, boundary re-framing with an inverse function) is the de-facto production design; its
  cost is dominated by the number of Newton solves per step (each needs D1 of both surfaces,
  ~4-5 evaluations per accepted step with rejections) and by the final sweep approximation to a
  B-spline. A kernel that keeps the fillet as a *procedural* surface (two contact curves + radius,
  as C3D's `MbFilletSurface` and cadrs's own contact-curve strip) avoids the approximation step
  entirely and only pays for the contact-curve fit.
- For cylinder/cylinder (equal radii, intersecting axes) and cylinder/plane the exact rolling-ball
  surface is a torus or a cylinder; OCCT does not exploit cylinder/cylinder, so a kernel that does
  (cadrs's `fillet_geom` "exact prism or revolution of the rolling-ball section") already has an
  advantage there. Dupin cyclides (Pratt 1990, Chandru et al. 1989) give exact rational blends
  between natural quadrics in more configurations (see KQ3).
- Step control should combine OCCT's tangent-angle and sagitta tests with a cheap 2D guard; the
  key robustness feature is the restriction re-framing (`Recadre`) that ends the walk exactly on
  a face boundary instead of overshooting.

### Gaps
- I could not open Klass & Kuhn 1992, Farouki & Sverrisson 1996, Choi & Ju 1989, Barnhill–Farin–Chen
  1993 or Sanglikar et al. 1990 (all paywalled); their step-control details are not verified.
- The IISc Voronoi-surface marching paper (`mecheng.iisc.ac.in/~asitava/sharad.pdf`) returned
  HTTP 502 on every attempt; only the search snippet is available ("marches from a starting point
  on the edge; setting the radius to zero gives the SSI curve").

---

## Key question 2: Vertex blends (setback blends, n-sided patches) and which give valid B-rep topology

### Takeaway
The standard taxonomy is Várady–Rockwood's: at a vertex with n blended edges, either intersect
the edge blends directly (bevel/"biseau"), or cut them back by a *setback* and fill a 2n-sided
hole whose boundary alternates *profile* (cross) curves on the edge blends and *spring* curves
on the primary faces; the hole is filled by a sphere/torus when the geometry allows, otherwise
by an n-sided transfinite/Gregory/plate patch. OCCT does exactly this (sphere/torus → circular
guide blend → `GeomFill_ConstrainedFilling` → `GeomPlate`), and the HP patents give the
Euler-operator bookkeeping that keeps the B-rep valid when faces grow or vanish.

### Cited Findings
- Várady & Rockwood 1997: setback blends "merge edge blends by broadening them at certain
  distances from the vertex"; the vertex blend is represented by "2n-sided patches" built from
  standard polynomial patches joined by a "setback split"; special cases with an odd number of
  sides arise — [SZTAKI eprint 1451 (abstract only)](https://eprints.sztaki.hu/1451/); DOI
  [10.1016/S0010-4485(96)00070-X](https://doi.org/10.1016/S0010-4485(96)00070-X)
- The Geomagic/Várady patent (reverse engineering, but the same geometry) describes the setback
  vertex blend precisely: "Profile curves and spring curves alternate. Profile curves border the
  edge blends and the vertex blend... Spring curves border the primary faces and the vertex blend,
  joining the longitudinal boundaries of adjacent edge blends. An n-edge vertex can have up to 2n
  sides"; any curve may have zero length (one spring curve plus two zero-length ones gives a
  four-sided blend); the simplest case is "a three-sided patch formed by intersecting the trimlines
  of two edge blends that meet on the same primary face... represented as an octant of a sphere";
  setbacks are computed from "the widths of neighboring features", with an optional positive
  correction "for degenerate cases such as cusps or tangential edges"; model variants A1 (one
  spring curve), A2 (full setback), B (no setbacks, exactly n-sided), C (no vertex blend) —
  [US8004517](https://patents.google.com/patent/US8004517)
- OCCT dispatch at a vertex (`PerformFilletOnVertex`): 1 stripe → `PerformOneCorner` (or
  `PerformIntersectionAtEnd` if more than 3 sharp edges); 2 stripes → `PerformTwoCorner`;
  3 stripes → `PerformThreeCorner`; 4+ or mixed/degenerate → `PerformMoreThreeCorner` ("Last
  chance..."); all-degenerate → `PerformSingularCorner` — [ChFi3d_Builder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKFillet/ChFi3d/ChFi3d_Builder.cxx)
- OCCT `PerformThreeCorner`: finds a *pivot* stripe (the one whose neighbours have opposite
  concavity); if the three end points coincide within `tol2d` the corner is "pointu"; a toric
  corner is used when the pivot face is a plane, the two adjacent radii are equal and the axes are
  orthogonal; a sphere when all three radii agree within `tolapp3d`; otherwise a corner fillet is
  walked along a circular guideline (`ChFi3d_CircularSpine`, falling back to a cubic Bézier spine)
  with `ConstRad` or `EvolRad`+`Law_S`; if that fails the hole is filled by
  `GeomFill_ConstrainedFilling(11,20)` from the pivot boundary, the two end boundaries and the face
  boundary (three boundaries when pointu); stripes are cut back by removing SurfData beyond the
  corner, not by a computed setback distance; falls back to the plate method when no pivot is found
  or the neighbours' base faces differ — [ChFi3d_FilBuilder_C3.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder_C3.cxx)
- OCCT `PerformTwoCorner` (two fillets, one sharp edge): if the two edge tangents are within 1° it
  delegates to `PerformMoreThreeCorner`; the fillets are intersected directly when the spine
  status is `AllSame` ("biseau"/bevel) or both `OnSame` ("rotule"/knee) via
  `PerformTwoCornerbyInter` (pcurve intersection on the common face); `ToricRotule` requires all
  three faces planar, constant spines, equal radii within 1e-7 and the third face perpendicular;
  otherwise `GeomFill_ConstrainedFilling(8,20)` over three or four boundaries; failure messages
  include "TwoCorner : No common face to loop the contour", "concavites inverted : fail" —
  [ChFi3d_FilBuilder_C2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder_C2.cxx); [ChFi3d_Builder_C2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_C2.cxx)
- OCCT `PerformMoreThreeCorner` ("Process case of a top with n edges"): intersects adjacent
  stripes (`ChFi3d_SearchFD`), limits how far parameters may recede ("determine the minimum recoil
  distance that can't be exceeded"), builds Hermite connections projected onto intermediate faces
  (`BRepAlgo_NormalProjection`, tol 1e-4), then `GeomPlate_BuildPlateSurface` (degree 3, 10 curve
  points, 1 iteration) with G1 constraints where edges are not both sharp, approximated with
  `GeomPlate_MakeApprox` (9 spans, max degree 8), approximation error `CriterionError()*1.1`;
  the source warns that "using 'bad' initial surface leads to much more 'bad' solution" and G1
  constraints can cause "unpredicable undulations" — [ChFi3d_Builder_CnCrn.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_CnCrn.cxx)
- OCCT extends the guide spine past a vertex before walking: scale factors 0.5 (one corner), 0.3
  (two corners, plus 1.5× radius) and 0.1 (three corners) — [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx)
- OCCT `PerformOneCorner` (a fillet ending at a vertex against unfilleted faces) intersects the
  tangency curves with the end face, splits at a seam, and refuses transversal crossings between
  stripes ("OneCorner : fillets have too big radiuses", comment "Prevent the builder from creating
  intersecting fillets - they are bad"); `PerformIntersectionAtEnd` walks up to 15 faces around
  the vertex, and when the fillet misses an edge it extends the fillet surface by
  `GeomLib::ExtendSurfByLength` (0.5× the larger interference length) or the adjacent face
  (`GeomInt_IntSS`); the cap for the unimplemented case throws "OneCorner : bouchon non ecrit" —
  [ChFi3d_Builder_C1.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKFillet/ChFi3d/ChFi3d_Builder_C1.cxx)
- Modern n-sided patches from the Várady–Salvi group (all with open PDFs, see catalogue): the
  Generalized Coons patch and Composite Ribbon patch (Várady, Rockwood, Salvi 2011/2013/2014),
  Charrot–Gregory with G2 (Salvi & Várady 2014), Generalized Bézier patches (Várady, Salvi, Karikó
  2016), and S-patch hole filling: "a method is presented for automatically creating S-patches of
  any degree or any number of sides, suitable for inclusion in a curve network with tangential
  continuity to the adjacent surfaces" — [BME 3DGeo publications](http://3dgeo.iit.bme.hu/pages/research/multisided); [Salvi 2019 S-patches](http://3dgeo.iit.bme.hu/papers/multisided/spatch.pdf)
- The Ricoh patent (Harada & Konno) builds a sliding-circle blend as biquartic Gregory patches
  (29 control points, 16 boundary + 13 interior from cross-boundary derivatives), adjacent patches
  meeting with G1, with the arc midpoint used to set the centre control point —
  [US5345546](https://patents.google.com/patent/US5345546)
- C3D documents three vertex-fillet types: "a vertex fillet by spherical surface, a vertex fillet
  by smooth surface, and a vertex fillet by triangular surface" — [C3D CornerValues (search summary)](https://c3dlabs.com/doc/struct_corner_values.html)
- A vertex blend "connects all the neighboring blends smoothly" when three or more blended edges
  meet, and "is typically spherical in simple cases" — [US6760038 / US7236168 (search summary)](https://patents.google.com/patent/US7236168)

### Inferences
- Valid topology comes from *where the hole boundary lives*, not from the patch type: the
  setback split puts every boundary curve of the corner patch either on an edge blend (profile
  curve: an iso-section of the blend) or on a primary face (spring curve: a curve on surface),
  so each corner edge has a p-curve on both of its faces by construction. OCCT keeps the same
  invariant (`ChFi3d_mkbound` boundaries are "straight lines uv" on the stripes).
- A sphere/torus/circular-guide corner first, then an n-sided transfinite patch (Generalized
  Coons or Gregory), then a plate/least-squares surface as a last resort, is the ordering both
  OCCT and the Várady literature converge on; a Generalized Coons / S-patch would be a
  deterministic and cheaper replacement for OCCT's iterative `GeomPlate`.
- OCCT's "cut back by index" (no computed setback distance) is a robustness weakness; the
  Geomagic/Várady rule "setback = max width of the two adjacent edge blends, plus a positive
  correction for tangent or cusp edges" is a usable formula.

### Gaps
- Várady & Hoffmann 1998 "Vertex blending: problems and solutions" and Várady & Rockwood 1997
  have no open full text (SZTAKI: "Full text not available"; Purdue: "no available PDF").
- Braid 1997 "Non-local blending of boundary models" (CAD 29(2) 89-100, DOI
  10.1016/S0010-4485(96)00038-3): no abstract or full text obtainable; ScienceDirect returned 403.
- "Zhou/Varady/Martin on vertex blends" could not be identified as a paper; the closest matches
  are Várady, Martin & Vida 1989 "Topological considerations in blending boundary representation
  solid models" (DOI 10.1007/978-3-642-61542-9_13) and the 1999 SZTAKI thesis "Vertex blending
  surfaces in computer aided geometric design" (eprint 2219, no full text).

---

## Key question 3: Variable-radius, conic and G2 (curvature-continuous) fillets

### Takeaway
Variable-radius rolling-ball (VRRB) blends are defined by a spine on which the ball centre moves
with a radius law; the exact spine is traced numerically in a high-dimensional space (Chuang et
al.) or defined by a reference surface cut of the Voronoi surface (Chandru et al.); OCCT handles
them with the same walker and `BlendFunc_EvolRad` plus a `Law_Function`. Exact rational VRRB
blends exist between natural quadrics (Dupin cyclides; Dahl & Krasauskas). G2 fillets are built
by replacing the circular section with a curvature-matched section (clothoid–arc–clothoid in
Boeing's patent; conic with rho in Parasolid-based systems) and verified by the Pegna–Wolter
linkage-curve theorem.

### Cited Findings
- Chuang, Lin & Hwang 1995: "for a given pair of base surfaces in parametric form, a reference
  curve, and a radius function of the rolling sphere, it presents an exact representation for the
  variable-radius spine curve and proposes a marching procedure"; the spine and linkage curves then
  build "parametric variable-radius spherical and circular blends" — [NYCU record](https://ir.lib.nycu.edu.tw/handle/11536/2146); DOI [10.1007/BF02434038](https://doi.org/10.1007/BF02434038)
- Chuang & Hwang 1997: because "the radius is hard to specify and the spine curve is hard to
  trace", geometric constraints that fix the radius are "converted into a nonlinear system that
  defines the spine exactly" and traced "in a high-dimensional space"; a second method enforces the
  constraints while tracing directly in 3D — [NYCU record](https://ir.lib.nycu.edu.tw/handle/11536/824?mode=full); DOI [10.1007/s003710050106](https://doi.org/10.1007/s003710050106)
- Chandru, Dutta & Hoffmann 1989: the variable-radius blend is "the portion of a canal surface,
  bounded by the curves of tangency"; its spine is "the intersection of the Voronoi surface of F
  and G and a given reference surface" (a plane when one primary is a plane); Pegna's earlier
  reference-curve iteration "does not use the exact distance" and "may exhibit sideway drifts";
  Dupin cyclides are "natural approximants since their principal lines of curvature are always
  circles", and whole cyclides serve as exact VR blends in some cases (also found by Pratt 1988);
  general approximation uses biarc + Liming's method on the conic spines — [Purdue TR](https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chandru-CS_technical_report-1989-On_Variable_Radius_Blending_Using_Dupin_Cyclides.pdf)
- Lukács 1998: spherical VRRB surfaces are analysed by envelope theory; "envelope surfaces are
  special cases of discriminant sets which... under certain natural conditions... are G1 continuous
  provided the defining equations are G1 (and piecewise C2) continuous as well" — [SZTAKI eprint 1762 / workshop abstract](https://eprints.sztaki.hu/1762/); DOI [10.1016/S0167-8396(98)00006-5](https://doi.org/10.1016/S0167-8396(98)00006-5)
- Dahl & Krasauskas 2012 and Dahl 2014: "Rational variable-radius rolling ball blends between
  pairs of natural quadrics can be built from rational curves in their bisector surface... When
  edges and corners are blended together in a composite corner, adjacent blends must meet with at
  least G1 continuity... control spheres setting the blending radius at key points... watertight,
  i.e., the boundary curves of the blend lies on the original surfaces" — [SINTEF record](https://www.sintef.no/publikasjoner/publikasjon/1181408/); DOIs [10.1016/j.cagd.2012.06.008](https://doi.org/10.1016/j.cagd.2012.06.008), [10.1007/978-3-319-08635-4_12](https://doi.org/10.1007/978-3-319-08635-4_12)
- OCCT variable radius: the builder branches on `fsp->IsConstant()`, building `BRepBlend_EvolRad`
  with `fsp->Law(HGuide)` (and `EvolRadInv`), and `Fleche` is 5% of the maximum of six radius
  samples along the guide; `BRepFilletAPI_MakeFillet::Add` takes an array of (parameter, radius)
  pairs for an "evolutive" fillet — [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx); [OCCT user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html)
- OCCT's chamfer KParts cover symmetric, two-distance and distance-angle chamfers
  (`ChFiKPart_MakeChamfer`, `MakeChAsym` with `DisOnP`) for the same plane/cylinder/cone pairs —
  [ChFiKPart_ComputeData.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFiKPart/ChFiKPart_ComputeData.cxx)
- Pegna & Wolter 1992 Linkage Curve Theorem: two surfaces joined with tangent-plane continuity
  along a linkage curve are second-order smooth there "if the normal curvatures along the linkage
  curve on each surface agree in one direction other than the tangent direction to the linkage
  curve" — [search summary citing the paper](https://link.springer.com/doi/10.1007/BF01198253); DOI [10.1115/1.2916918](https://doi.org/10.1115/1.2916918)
- Boeing (Grandine & Hogan) G2 fillet: spine = the face intersection curve; at each point the
  section plane normal is the intersection tangent plus weighted face normals (so the plane varies
  smoothly, unlike the rolling ball's); an 8×8 Newton system finds the two end points constrained
  to lie on a circle of the larger "cut radius"; the section is clothoid–arc–clothoid, the clothoids
  matching position, tangent and curvature of each face and reaching curvature 1/fillet-radius;
  the faces are trimmed at the cut-radius points; stated drawbacks of rolling balls: the ball
  "meets both surface edges simultaneously only when the surface normals and edge tangents are
  coplanar", and plane/plane gives a cylinder with a curvature jump — [US7518609](https://patents.google.com/patent/US7518609)
- T-FLEX (a Parasolid-based CAD) exposes "Curvature continuous" G2 blends for constant, variable
  and "variable-elliptical" (conic, "Convexity coefficient (Rho)") roundings; enabling it "distorts
  Radius" for the circular types — [T-FLEX help](https://tflex.com/help/eng/T-FLEX%20CAD/17/14_edgeblend_additional_continuity.htm)
- Filkins 1991 (MIT, Patrikalakis): blends between NURBS surfaces "with up to curvature continuous
  boundary conditions along... the linkage curves", approximated by lofting low-degree B-spline
  surfaces whose cross-link curves "match the curvature of a normal section of the underlying
  surface at the end point", with adaptive sampling; applied to propeller blade/hub fillets —
  [archive.org PDF](https://archive.org/download/algorithmsforble00filk/algorithmsforble00filk.pdf)
- Hartmann 2001 (Visual Computer) "Parametric Gn blending of curves and surfaces" and CAGD 2001
  "Gn-continuous connections between normal ringed surfaces" exist (DOIs 10.1007/PL00013398 and
  10.1016/S0167-8396(01)00065-6); You & Zhang 2006 summarise: Aumann's normal ringed surfaces give
  curvature-continuous connections of cones and cylinders, Hartmann achieved G2 implicit/parametric
  transitions with "a simple additional condition" — [You & Zhang 2006 PDF](https://koreascience.or.kr/article/JAKO200634514931903.pdf)
- Kim & Elber (1995/1997) build free-form blends symbolically: rail curves on the two surfaces,
  cross-boundary tangent fields chosen from several recipes (e.g. T = dC/dt × n(u(t),v(t)), or
  linear combinations of S_u and S_v), normalised by refinement so the blend has "accurate tangent
  plane continuity for the entire boundary... as opposed to... at only a finite set of locations"
  — [Utah PDF](https://www2.cs.utah.edu/gdc/publications/papers/kim95.pdf)
- Eager & Dannenhoffer 2019 ("Flends"): fillets/chamfers "are restrictive and fragile... the extent
  of a fillet on the two surfaces are linked" and "yield curvature discontinuities"; they build
  B-spline transition surfaces between user-chosen offset edges, solid-modeler agnostic —
  [AIAA 2019-1717 PDF](https://acdl.mit.edu/ESP/Publications/AIAApaper2019-1717.pdf)

### Inferences
- For a kernel that already has a radius law (`fillet`: "width fillets... take a law of radii"),
  the cheapest exact-section VR blend is OCCT's: the same 4-unknown Newton with R = law(t) and
  a `Fleche` from the max radius. The spine then costs nothing extra.
- Conic (rho) sections and G2 sections only change the *section generator* (section in the normal
  plane through the two contact points), not the tracing; but a true G2 fillet moves the contact
  curves (the section's end tangents no longer lie at ball-contact points), which is why Boeing
  introduces a larger "cut radius" and T-FLEX warns that the radius is "distorted".
- Dupin cyclides are the only known closed-form VR blends; worth having as exact cases between
  natural quadrics (plane/cone, cone/cylinder, sphere/plane), replacing a numeric walk + fit.

### Gaps
- No open full text of Chuang et al. 1995/1997, Lukács 1998, Hermann–Lukács–Várady 1997
  (Springer PDF is HTML behind a paywall), Pegna & Wolter 1992, Hartmann 2001, or Dahl 2014 (the
  UiO DUO link now redirects to nva.sikt.no with no file).
- No published description of how Parasolid/ACIS compute conic or G2 cross sections was found;
  only the end-user options (T-FLEX, Siemens release notes).

---

## Key question 4: Overflow onto neighbouring faces, blends on previous fillets, blend-chain propagation

### Takeaway
Two mechanisms are documented: (a) the walker detects that a contact curve reaches a face
boundary, re-frames the solution on that boundary and continues on the neighbouring face
(OCCT "obstacle" handling, SurfData per face), and (b) at the topology level the blend face is
intersected with the surrounding faces, consuming edges and whole faces by Euler operators (HP
patents). Parasolid-family systems expose three overflow policies (smooth overflow, retained
cliff edge, notch). Chain propagation is a tangency test between consecutive edges with a
G1 face-tangency test at the shared vertex.

### Cited Findings
- OCCT chain propagation (`PerformElement`): the spine grows downstream then upstream from the
  seed edge; a candidate edge is accepted when `FaceTangency` holds for the opposing faces and the
  tangent directions agree within an angle test (stricter `ta` when `TangentOnVertex`); an edge
  with more than one face on the far side gives `ChFiDS_BreakPoint`; a closed chain with
  `FaceTangency` is `ChFiDS_Closed`; free ends default to `ChFiDS_FreeBoundary`;
  `PerformExtremity` counts G1 connections at the vertex and forces `BreakPoint` when faces minus
  G1 connections exceed 3; the API extends "to all edges in a smooth continuity with the original
  edge" — [ChFi3d_Builder_1.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_1.cxx); [OCCT user guide](https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html)
- OCCT overflow during the walk: a stripe is a sequence of `ChFiDS_SurfData`; when the blend runs
  off an edge onto a neighbouring face the new SurfData is inserted (`InsertAfter`/`InsertBefore`),
  the obstacle edge is recorded (`SetIndexOfC1`), `SearchFace`/`IsG1` find a tangent neighbour and
  `SearchFaceOnV` handles vertices; four obstacle cases (both sides, S1 only, S2 only, none);
  `ChFi3d_BuildPlane` "builds a tangent plane so the path can continue when no neighbouring face
  exists"; surface/restriction and restriction/restriction blend functions (`SurfRstConstRad`,
  `RstRstConstRad`) walk with one or both contact points pinned to an edge (the "cliff edge"
  case) — [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx); [ChFi3d_FilBuilder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder.cxx); [BRepBlend API](https://pythonocc-documentation.readthedocs.io/en/review-gen-apidoc-rtd/apidoc/OCC.Core.BRepBlend.html)
- OCCT twisted surfaces: after the walk, surfaces flagged twisted are rebuilt from neighbouring
  traces with `GeomFill_ConstrainedFilling(11,20)` ("It is attempted to reprocess the squares that
  bore"); if impossible `ChFiDS_TwistedSurface` is set and it throws — [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx)
- HP (Freitag) US5615317: after computing the blend boundaries LB/RB by intersecting the blend
  with the object, sub-algorithm A1 at each end vertex intersects the adjacent face's edge at V
  with the boundary; no intersection → delete the edge with "kill edge vertex", which pulls the far
  endpoint onto V, and test again; A2 intersects every face at V not adjacent to E with the blend
  surface, removing edges and then whole faces ("kill object face vertex") when none intersect;
  a count check confirms all expected intersections; "the number of removed edges and faces is
  not limited"; prior systems "show an error message and the blend fails" —
  [US5615317](https://patents.google.com/patent/US5615317)
- HP (Opitz) US6133922 extends this to *growing* faces: remove obsolete edges (KEV), insert the
  blend face with ADEF (body temporarily inconsistent), then for each end vertex collect "gaps"
  (topologically adjacent edges whose geometric endpoints differ), compute a trimming path from RB
  to LB by a depth-first search over half-curves of face/face intersections through the current
  point, ordered by angle then length, rejecting self-intersecting paths and paths crossing
  existing edges, extending short edges by ray shooting; then integrate the path (a segment
  crossing an existing edge shrinks that face; otherwise the face grows and a new edge is added),
  collect new gaps and repeat; "the complete path is computed before any topology is changed"
  because topological edits "are generally not reversible in a unique way"; the prior "swallowed
  topology" method "cannot handle growing faces or edges", which arise "when several edges meeting
  at a vertex are blended simultaneously" — [US6133922](https://patents.google.com/patent/US6133922)
- Parasolid-based T-FLEX overflow options, tried in order: *Smooth overflow* (when the blend
  overflows an existing rounding, "the blend surface is deformed across a transition zone so that
  it meets the existing rounding tangentially"); *Retain cliff edges* (when the rounding "creeps"
  over a sharp edge, "the sharp edge is kept, and the blend is formed by a ball that stays tangent
  to one of the original faces and is supported by the retained edge"; sub-option "Process Cliff
  End"); *Notch* ("the surfaces of the notched element are trimmed or extended to meet the
  rounding"); plus "Inside tight corners" modes for sidewall radii smaller than the ball ("Tight
  blends", "Tight faces", "Tight partial"); "Disabling unneeded complex options shortens model
  regeneration time" — [T-FLEX help](https://tflex.com/help/eng/T-FLEX%20CAD/17/blendedges_overflowprocessing.htm)
- Parasolid v34.1 notes mention "notch overflow blend operations", "face-face blends", "cap faces"
  at blend ends, and that blends "can extend neighboring faces" — [Siemens blog](https://blogs.sw.siemens.com/plm-components/parasolid-v34-1-release-highlights/)
- Siemens US9690878 ("mutually dependent blends"): a blend's shape depends on its "unders"; blends
  can be unders of other blends; a "blend ribbon" is a set of mutually dependent blends; "ribbon
  breakers" are chosen from the user's edit and reblendable/non-reblendable blends are designated
  by propagating outward from the breakers, alternating; this is for variational editing, not
  construction — [US9690878](https://patents.google.com/patent/US9690878)
- Autodesk US5774359 extends a fillet past the end of the face/face intersection: offset A by R,
  build the spherical duct of radius R along a boundary curve of B, intersect to get the extended
  spine, sample it to a chordal tolerance, find contact points, and loft rational arcs; "the fillet
  ends are described as stopping 'abruptly'" in prior methods — [US5774359](https://patents.google.com/patent/US5774359)
- OCCT's topology assembly: stripes are written into a `TopOpeBRepDS` data structure
  (`ChFi3d_FilDS`), stripe/stripe intersections are checked first ("First we'll check ain't there
  / intersections between fillets"), then `myCoup->Perform(myDS)` splits faces, `MergeSolid`
  rebuilds solids, `BRepLib::SameParameter` and `ShapeFix::SameParameter` run on new faces;
  vertex tolerances above 1e-4 are bumped by 3e-4 and edge tolerance set 1e-5 above —
  [ChFi3d_Builder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKFillet/ChFi3d/ChFi3d_Builder.cxx)
- C3D 2019: the kernel "must know the arrangement of the involved faces and the adjacent faces
  that may also be affected... Some elements may be removed by the operation, and others may be
  split into two or more parts... Not every construction outcome can be predicted before the
  operation runs" — [C3D Labs 2019](https://c3dlabs.com/blog/products/fillets-nothing-more-than-more-fillets/)

### Inferences
- The two HP patents are the clearest published statement of what cadrs's `fillet` module
  needs for "faces the strip hangs over grow to its cut" and "end faces that meet past the vertex
  grow over the fillet's end": compute the whole trimming path (RB→LB) around each end vertex by
  intersecting the blend/neighbour faces *before* touching topology, then apply Euler operators
  in one pass. Opitz's growing-face DFS is the general form of the README's "set back with one
  patch filling the hole".
- "Cliff edge" blends (one contact pinned to an edge) are a distinct blend function (OCCT
  `SurfRstConstRad`: 3 unknowns, the ball tangent to one face and touching an edge) and are the
  cheap answer when the ball would roll over a sharp edge; a kernel needs this function to avoid
  the tool-body fallback in the overflow case.

### Gaps
- No source describes how Parasolid or ACIS *implement* smooth overflow (deforming the blend in a
  transition zone); only the user-visible behaviour is documented.
- Braid 1997 ("non-local blending") presumably addresses exactly overflow/consumed faces, but no
  abstract could be retrieved.

---

## Key question 5: Performance characteristics and robustness failure modes

### Takeaway
No paper reports fillet timings; the only performance evidence is structural (what OCCT spends
its time on) and the cadrs benchmark itself. Failure modes are well catalogued in OCCT's exception
strings and the C3D posts: no starting solution, Newton divergence at the walk, pinched/twisted
blend surfaces, intersecting fillets ("too big radiuses"), no common face at a corner, plate
undulation, and tolerance inflation after approximation.

### Cited Findings
- OCCT per-stripe cost structure: `StartSol` (≥10 Newton attempts), the walk (one 4×4 Newton
  per trial step, 30 iterations max, with halving on rejection), `Recadre` inverse solves at each
  boundary (35 iterations), then `BRepBlend_AppSurf` sweep approximation (degree ≤ 11, ≤ 50
  segments, sections as rational arcs), then `SplitSurf` singularity search, then
  `GeomPlate`/`GeomFill_ConstrainedFilling` at corners, then DS splitting and two SameParameter
  passes — [Blend_Walking_4.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/Blend/Blend_Walking_4.gxx); [ChFi3d_Builder_2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_2.cxx); [ChFi3d_Builder.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKFillet/ChFi3d/ChFi3d_Builder.cxx)
- OCCT failure strings (robustness map): "StartSol echec", "CallPerformSurf : Path failed!",
  "PerformSurf : Failed approximation!", `ChFiDS_WalkingFailure`, `ChFiDS_TwistedSurface`,
  "OneCorner : fillets have too big radiuses", "OneCorner : bouchon non ecrit",
  "IntersectionAtEnd : the max number of faces reached" (15), "TwoCorner : No common face to loop
  the contour", "concavites inverted : fail", "PerformMoreThreeCorner: pb in the parsing of edges
  and faces", and the GeomPlate warning that G1 constraints cause "unpredicable undulations" —
  [ChFi3d_Builder_C1.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKFillet/ChFi3d/ChFi3d_Builder_C1.cxx); [ChFi3d_FilBuilder_C2.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_FilBuilder_C2.cxx); [ChFi3d_Builder_CnCrn.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ChFi3d/ChFi3d_Builder_CnCrn.cxx)
- OCCT tolerance problems after filleting: a tracker issue reports "fillet faces have big
  tolerance (0.00049)" violating the rule that edge tolerances exceed face tolerance, and edges
  that fail SameParameter; a 2022 commit fixed a crash by reducing `ChFi3d_Builder::tolesp` and
  separating `tolgui` (guide parameter), `tolpoint2d`, `tolpoint3d` — [OCCT tracker 21595](https://tracker.dev.opencascade.org/view.php?id=21595); [OCCT commit 03275c0](https://git.rbts.co/OpenCascade/occt/src/commit/03275c0be165da7abe8e0044d79c10a0038a8340)
- OCCT default approximation tolerances in `FilletSurf_Builder`: `Ta 1e-2` (angular), `Tapp3d 1e-4`,
  `Tapp2d 1e-5`; `BRepFilletAPI_MakeFillet::SetParams(Tang, Tesp, T2d, TApp3d, TolApp2d, Fleche)` —
  [pythonocc FilletSurf doc](https://cdn.jsdelivr.net/gh/tpaviot/pythonocc-core@3ceb6b92/doc/apidoc/0.17.3/OCC.FilletSurf.html)
- Self-intersection of rolling-ball blends when the radius exceeds the local radius of curvature
  is treated by Hermann 1992 (SPIE 1830, DOI 10.1117/12.131744, "Rolling ball blends and
  self-intersection") and Hermann–Lukács–Várady 1997 ("Continuity and self-intersections of
  variable radius rolling ball blend surfaces", DOI 10.1007/978-0-387-35187-2_31); the Patrikalakis
  book covers local and global self-intersection of pipe surfaces (11.6.2-11.6.3) and offsets —
  [Crossref/Semantic Scholar metadata](https://api.crossref.org/works/10.1117/12.131744); [MIT hyperbook TOC](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/)
- C3D 2019: "3D fillets are one of the hardest tasks for a geometric kernel"; the full-round
  (three-face) fillet "needed a new algorithm" based on "numerical methods of equation solving";
  the company adds new fillet cases on a regular basis and invites failing cases from users —
  [C3D Labs 2019](https://c3dlabs.com/blog/products/fillets-nothing-more-than-more-fillets/)
- C3D 2025 planned work: "improving variable-radius edge filleting and handling self-intersections
  where curvature changes sharply" — [C3D Labs 2025](https://c3dlabs.com/blog/products/c3d-geometric-kernel-new-features-and-development-trends/)
- Rhino users report failures such as a fillet failing "once a chamfer was applied to a seventh
  edge" — [McNeel forum (search summary)](https://discourse.mcneel.com/t/fillet-edge-operation-failed/156093)
- OCCT's own timing for the cadrs conformance suite was 0.84 s for 83 cases; cadrs's fillets are
  9.2x/10.3x slower than OCCT on OCCT's and SMLib's blend cases — [cadrs README](../../crates/cadrs_kernel/README.md)

### Inferences
- Because OCCT walks *and* then fits a B-spline sweep, a kernel that keeps a procedural fillet
  (contact curves + radius) should be able to beat it on the geometry phase; the 9-10x gap in
  cadrs is therefore more likely in the contact-curve tracing/fitting and the imprint/loop
  rebuild than in a missing closed form. Profiling by phase (start solution, march, fit,
  imprint, corner fill) is the first step; OCCT's step sizes (5% of guide length initial step,
  sagitta 5% of radius) are a sanity reference for how many sections a walk needs.
- The robustness pattern shared by OCCT and C3D is a *ladder* of methods per configuration
  (analytic → walked → filled → plate; bevel → knee → corner fillet → plate) with explicit
  fallbacks; cadrs's "local trimming, else tool-body boolean" is a two-rung ladder.

### Gaps
- No published benchmarks of fillet performance for any kernel; no paper or vendor document
  states what dominates fillet cost. SMLib's "fillet executive" (which the README cites) has no
  public description beyond its test cases.
- The Vida–Martin–Várady 1994 survey's "open research questions" section could not be read.

---

## Ranked source catalogue

Entries 1-5 are the "must read" set. Local PDF paths are given where a legitimate open copy
was downloaded; "paywalled" means no open copy was found.

### 1. OpenCASCADE ChFi3d / BRepBlend / BlendFunc source (the only fully readable production fillet implementation)
- Citation: Open CASCADE S.A.S., *OCCT 7.8.0 source: packages `ChFi3d`, `ChFiKPart`, `ChFiDS`, `BRepBlend`, `Blend`, `BlendFunc`, `BRepFilletAPI`* (1994-2024).
- Canonical URL: https://github.com/Open-Cascade-SAS/OCCT (files cited above under V7_8_0 and master); user guide https://occt3d.com/dev/doc/overview/html/occt_user_guides__modeling_algos.html
- Open access: yes (LGPL source). Not downloaded (not a PDF).
- Algorithm: `BRepFilletAPI_MakeFillet::Add(edge, radius)` seeds a `ChFiDS_Spine`, which `PerformElement` grows along G1-tangent edges in both directions; `ExtentAnalyse` extends the spine past the end vertices (0.5/0.3/0.1 of some length depending on how many stripes meet there); `PerformSetOfSurf` tries the analytic `KPart` (plane/plane, plane/cylinder, plane/cone with line/circle spine; corners: torus, sphere, "rotule") and otherwise `PerformSetOfKGen`, which for each elementary spine finds a start solution, walks `BRepBlend_Walking` with `BlendFunc_ConstRad`/`EvolRad` (4 unknowns, 1+3 equations, Newton 30 its, step halved/×1.5 by chord-angle and sagitta tests, re-framed onto face boundaries by inverse functions, continued on the neighbouring face as a new `SurfData`), approximates the section sweep with `BRepBlend_AppSurf`, splits pinched surfaces, then `PerformFilletOnVertex` handles corners (bevel/knee intersections, toric/spherical corners, circular-guide corner fillet, `GeomFill_ConstrainedFilling`, `GeomPlate`), and finally `TopOpeBRepDS` splits the faces and rebuilds solids with SameParameter fixes.
- Limitations: cylinder/cylinder not analytic; every fillet becomes an approximated B-spline sweep with inflated tolerances; many hard-coded heuristics (1% guide extension, 15-face limit, 1° tangent test); "bouchon non ecrit" cap case; GeomPlate corners may undulate; French-named code with little documentation.
- Relevance to cadrs: `fillet`/`fillet_geom` (the walker's step control, start-solution search and boundary re-framing), `blend_chain` (`PerformElement` tangency rules and end statuses), `blend_corner` (the corner ladder: torus → sphere → circular-guide fillet → Coons/plate), and the overflow handling (SurfData per face, `SurfRst`/`RstRst` cliff-edge functions) that would retire the tool-body fallback.

### 2. Opitz (HP), "Method of integrating a blend or chamfer surface replacing an edge of a geometrical body into a solid model", US 6,133,922 (filed 1997, granted 2000); and Freitag (HP), "Method for blending edges of a geometric object in a computer-aided design system", US 5,615,317 (1997)
- Canonical URLs: https://patents.google.com/patent/US6133922 ; https://patents.google.com/patent/US5615317 (full text free).
- Algorithm (Opitz): blend geometry first (contact curves on both faces); walk the edges leaving each end vertex on each adjacent face, mark edges with no crossing for removal, find RB/LB crossing points; `KEV` the marked edges (far endpoints collapse onto the vertex, body temporarily inconsistent); `ADEF` the blend face in place of the edge; then per end vertex loop: collect gaps (adjacent edges with different geometric endpoints), compute a trimming path RB→LB by DFS over half-curves of face/face intersections (candidates ordered by angle then length, self-intersecting or edge-crossing paths rejected, short edges extended by ray shooting, faces marked "growing"), integrate the path (crossing an existing edge → face shrinks, edges reattached; otherwise face grows → new edge by ADEF), collect new gaps, repeat until none. Freitag's earlier A1/A2 is the shrink-only version (kill-edge-vertex until a crossing exists, kill faces with no crossing left, check the count of expected intersections).
- Limitations: the growing-face DFS is described for a single-loop hole; overflow onto curved neighbouring faces is handled only implicitly ("trims infinite face portions"); no numerical details for the face/face intersections; nothing about smoothing the blend across an existing fillet.
- Relevance to cadrs: `fillet` (the imprint/loop rebuild where "faces the strip hangs over grow to its cut" and end faces grow over the fillet end) and `blend_corner` (the setback hole filled "with one patch" is exactly the trimming-path hole); this is the documented way to make local trimming cover the cases that now fall back to `blend`.

### 3. Vida, Martin, Várady, "A survey of blending methods that use parametric surfaces", Computer-Aided Design 26(5):341-365, 1994
- Canonical URL: https://doi.org/10.1016/0010-4485(94)90023-X ; SZTAKI record https://eprints.sztaki.hu/682/ (no full text); 59-page 1992 report version https://eprints.sztaki.hu/158/ (no full text). Paywalled.
- Algorithm/content (from the abstract and citing works): proposes a terminology and classification of parametric blends (rolling-ball vs. trimline-based, spine/profile, range of influence), evaluates the geometric techniques (Rossignac–Requicha torus/cylinder approximations, Várady's geometric marching over blending arcs, Klass–Kuhn offset-intersection spines, Choi–Ju's mapping/substitution, Gregory/Coons patch constructions), discusses the *topology* of blending in B-rep models (which faces are affected, how trimlines are intersected), and ends with open questions and a comprehensive bibliography.
- Limitations: 1994; predates setback vertex blends (1997) and variable-radius envelope theory (1998); no full text available to me, so the above is second-hand.
- Relevance to cadrs: all five modules; the canonical map of the design space and the bibliography to mine.

### 4. Chuang, Lin, Hwang, "Variable-radius blending of parametric surfaces", The Visual Computer 11(10):513-525, 1995; Chuang & Hwang, "Variable-radius blending by constrained spine generation", The Visual Computer 13(7):316-329, 1997
- Canonical URLs: https://doi.org/10.1007/BF02434038 ; https://doi.org/10.1007/s003710050106 ; NYCU records https://ir.lib.nycu.edu.tw/handle/11536/2146 and https://ir.lib.nycu.edu.tw/handle/11536/824 (abstracts). Paywalled.
- Algorithm (from abstracts): given two parametric faces, a reference curve and a radius function, write the spine exactly as the solution set of a nonlinear system (footpoints on both surfaces at distance r(t) along the normals, centre on the reference-curve section) and march along it with a predictor-corrector in the joint parameter space (the 1992 thesis describes "a marching method to generate the spine and linkage curves" and NURBS patches from them); the 1997 paper adds geometric constraints that fix the radius (so the user need not specify r(t)), converts them into a system defining the spine exactly and traces it "in a high-dimensional space", and gives a cheaper variant that enforces the constraints while tracing in 3D.
- Limitations: free-form (polynomial) surfaces only in the examples; the high-dimensional tracing is reported as expensive (a later paper advertises avoiding "numerical tracing in a nine-dimensional space"); no treatment of topology.
- Relevance to cadrs: `fillet`/`fillet_geom` radius-law fillets and `blend` variable sections; replaces the lofted-chain tool body with a traced spine.

### 5. Várady & Rockwood, "Geometric construction for setback vertex blending", Computer-Aided Design 29(6):413-425, 1997; Várady & Hoffmann, "Vertex blending: problems and solutions", Mathematical Methods for Curves and Surfaces II, Vanderbilt UP, 1998, pp. 501-527; plus the open Várady–Salvi–Rockwood n-sided patch papers
- Canonical URLs: https://doi.org/10.1016/S0010-4485(96)00070-X ; https://eprints.sztaki.hu/1756/ ; https://cs.purdue.edu/cgvlab/www/publications/varady1998vertex (all without full text). Open companions: Salvi, Várady, Rockwood, "New schemes for multi-sided transfinite surface interpolation" (2012) and the journal version "Transfinite surface interpolation over irregular n-sided domains", CAD 43(11), 2011, https://doi.org/10.1016/j.cad.2011.08.028 ; Salvi, Várady, Rockwood, "Ribbon-based transfinite surfaces", CAGD 31(9), 2014, https://doi.org/10.1016/j.cagd.2014.06.006 ; Salvi, "G1 hole filling with S-patches made easy" (2019, arXiv:2002.11109); Várady, Salvi, Vaitkus, "Genuine multi-sided parametric surface patches – A survey", CAGD 110, 2024, https://doi.org/10.1016/j.cagd.2024.102286 .
- Local PDFs: `../papers/blends/Salvi-Varady-Rockwood-2012-new-schemes-multi-sided-transfinite-interpolation.md`, `.../Salvi-Varady-Rockwood-2013-transfinite-interpolation-specific-curvenet-configurations.pdf`, `.../Salvi-Varady-2014-multi-sided-surfaces-curvature-continuity.pdf`, `.../Salvi-Varady-2015-comparison-two-n-patch-representations.pdf`, `.../Salvi-2019-G1-hole-filling-with-S-patches.pdf`, `.../Salvi-2020-multi-sided-generalization-C0-Coons-patch.pdf`, `.../Varady-Salvi-2013-KEPAF-F10_04-ribbon-patches.pdf`, `.../Vaitkus-2023-arxiv-2311.12822-ribbon-biharmonic-patches.pdf`.
- Algorithm: cut each edge blend back from the vertex by a setback (chosen from the widths of the adjacent blends, with extra setback at tangent/cusp configurations), bound the hole by alternating profile curves (sections of the edge blends) and spring curves (curves on the primary faces joining adjacent edge-blend rails), giving up to 2n sides with zero-length sides allowed; fill it with a 2n-sided patch assembled from standard quadrilateral patches via the "setback split" (1997), or, in the modern formulation, with one transfinite n-sided patch: a Generalized Coons patch (side interpolants with corner corrections, Boolean-sum logic), a Composite Ribbon patch (ribbons spanning three consecutive sides, Wachspress or harmonic domain coordinates), a Charrot–Gregory patch (corner ribbons, extended to G2), an S-patch (multi-sided Bézier triangle generalisation, interior points by biharmonic masks) or a Generalized Bézier patch with a simple control structure. All take the boundary curves and cross-derivative ribbons as input, so G1 to the edge blends and faces holds by construction.
- Limitations: the transfinite patches are not tensor-product NURBS; exporting them to STEP needs a CAD-compatible conversion (Salvi, Várady, Rockwood, CAD&A 18(1), 2021, https://doi.org/10.14733/cadaps.2021.156-169); their p-curves on the primary faces are the spring curves (curves on surface), not iso-lines; domain parameterisation matters for concave or very uneven holes.
- Relevance to cadrs: `blend_corner` and `fillet`'s "set back with one patch filling the hole": a Generalized Coons or S-patch replaces an ad-hoc patch and gives a deterministic, non-iterative corner; the setback formula replaces OCCT-style index truncation.

### 6. Bajaj, Hoffmann, Lynch, Hopcroft, "Tracing surface intersections", CAGD 5(4):285-307, 1988
- Canonical URL: https://doi.org/10.1016/0167-8396(88)90010-6 ; open PDF (scanned, no text layer) https://www.cs.purdue.edu/cgvlab/www/resources/papers/Bajaj-Computer_aided_geometric_design-1988-Tracing_surface_intersections.pdf
- Local PDF: `../papers/blends/Bajaj-Hoffmann-Lynch-Hopcroft-1988-tracing-surface-intersections.md` (image-only scan).
- Algorithm (from my background knowledge of the paper; not re-verified from the scan): the intersection curve of two implicit or parametric surfaces is traced as the solution of a system F(x) = 0 in a higher-dimensional space (e.g. 2+2 parameters plus 3 coordinates), with a predictor that follows the tangent (the null vector of the Jacobian) and a Newton corrector; at singular points (rank-deficient Jacobian) the local branches are found by a power-series/ Puiseux analysis of the curve's Taylor expansion, and tracing continues on each branch; step size is adapted from the curvature estimate. Cited by Chandru et al. as the robust way to trace a VR spine "in 10-dimensional space".
- Limitations: algebraic/polynomial surfaces; singularity analysis is symbolic and costly; no boundary (trimmed face) handling.
- Relevance to cadrs: `fillet_geom`/`ssi` (predictor-corrector tracing of spines and contact curves with branch handling at pinch points).

### 7. Chandru, Dutta, Hoffmann, "On variable radius blending using Dupin cyclides", Purdue CSD-TR-851, 1989 (journal version: "Variable radius blending using Dupin cyclides", Geometric Modeling for Product Engineering, 1990); companion "On the geometry of Dupin cyclides", The Visual Computer 5(5):277-290, 1989; Pratt, "Cyclides in computer aided geometric design", CAGD 7(1-4):221-242, 1990 (DOI 10.1016/0167-8396(90)90033-N)
- Canonical URL: https://docs.lib.purdue.edu/cstech/725 ; open PDF https://www.cs.purdue.edu/cgvlab/www/resources/papers/Chandru-CS_technical_report-1989-On_Variable_Radius_Blending_Using_Dupin_Cyclides.pdf ; "Geometry of Dupin cyclides" open at https://repository.ias.ac.in/5471
- Local PDF: `../papers/blends/Chandru-Dutta-Hoffmann-1989-variable-radius-blending-Dupin-cyclides.md`
- Algorithm: define the Voronoi surface V(F,G) of the two primaries (points equidistant from both, obtained by eliminating footpoints from an 8-equation system; worked example: cylinder and plane give a hyperbolic paraboloid), the spine as V(F,G) ∩ reference surface, and the VR blend as the envelope of spheres centred on the spine with radius = common distance; since explicit elimination is intractable, trace the spine numerically in 10-D (point, two footpoints, distance), then approximate the blend by Dupin cyclide patches (all lines of curvature are circles; a cyclide is determined by a conic spine and a radius function) joined along circles, fitting the conic spines by biarcs + Liming's method; case study: cylinder and inclined plane, where whole cyclides are exact VR blends.
- Limitations: natural quadrics only; approximation (not exact) outside the special cases; later work (Shene 2000) shows blending and offsetting do not commute for cyclides; Garnier et al. note cyclides suit circular quadrics but not elliptic ones.
- Relevance to cadrs: `fillet_geom` exact cases (cyclide blends between plane/cone, cone/cylinder, sphere/plane would extend the current "cylinder or torus" exact set) and `blend` variable-radius tools.

### 8. Rossignac & Requicha, "Constant-radius blending in solid modeling", Computers in Mechanical Engineering 3:65-73, 1984; and "Offsetting operations in solid modelling", CAGD 3(2):129-148, 1986
- Canonical URLs: no DOI for 1984 (ASME CIME); 1986 https://doi.org/10.1016/0167-8396(86)90017-8 (DOI from my background knowledge). Paywalled; Rossignac's page https://gatech.academia.edu/JarekRossignac lists it ("impacted surface/surface intersection technology in Unigraphics").
- Algorithm (from the citing sources): a constant-radius round is the CSG expression (A↓r)↑r (shrink then grow by r) and a fillet (A↑r)↓r, so blends are defined by offsetting the solid; the blend surfaces are then approximated by sequences of spheres, cylinders and tori; the ball rolls in contact with the primary surfaces along the intersection of their offsets.
- Limitations: global (not local) definition, so every edge is blended; CSG primitives only; approximation by tori.
- Relevance to cadrs: `offset_round`/`blend` (the kernel's rounded offsets already follow this definition: body ∪ slabs ∪ tubes ∪ balls); historical basis for the tool-body approach.

### 9. Klass & Kuhn, "Fillet and surface intersections defined by rolling balls", CAGD 9(3):185-193, 1992; Hermann, "Rolling ball blends and self-intersection", SPIE 1830 (Curves and Surfaces in Computer Vision and Graphics III), 1992
- Canonical URLs: https://doi.org/10.1016/0167-8396(92)90016-I ; https://doi.org/10.1117/12.131744 . Paywalled.
- Algorithm (second-hand): the spine is the intersection of the two offset surfaces at distance R; the trimlines are the footpoints; Klass & Hermann compute trimline tangents at each ball position from the surface derivatives so that the marching can use a Hermite predictor and the trimlines can be fitted as splines; Hermann treats the case where the blend self-intersects (radius larger than the local radius of curvature of a primary) and approximates the blend with bicubic patches.
- Limitations: unknown step-control details; self-intersection handling is a post-check.
- Relevance to cadrs: `fillet_geom` (tangent-based predictor for contact curves), `blend` fit checks.

### 10. Farouki & Sverrisson, "Approximation of rolling-ball blends for free-form parametric surfaces", CAD 28(11):871-878, 1996
- Canonical URL: https://doi.org/10.1016/0010-4485(96)00008-5 . Paywalled.
- Algorithm (second-hand, from citing works): avoids constructing non-rational offset surfaces by intersecting the offset *curves* directly on the original surfaces, computing the spine and contact curves to a prescribed precision with error bounds, then approximating the blend within a specified tolerance.
- Limitations: free-form parametric surfaces only; approximation.
- Relevance to cadrs: `fillet_geom` (error-bounded contact-curve fitting).

### 11. Lukács, "Differential geometry of G1 variable radius rolling ball blend surfaces", CAGD 15(6):585-613, 1998; Hermann, Lukács, Várady, "Continuity and self-intersections of variable radius rolling ball blend surfaces", IFIP Product Modeling, 1997 (DOI 10.1007/978-0-387-35187-2_31); "Techniques for variable radius rolling ball blends", 1995 (SZTAKI eprint 911)
- Canonical URL: https://doi.org/10.1016/S0167-8396(98)00006-5 . Paywalled (SZTAKI: no full text).
- Content (from the workshop abstract): VRRB surfaces are envelopes of a one-parameter family of spheres; envelopes are discriminant sets that are G1 under natural conditions when the defining equations are G1 and piecewise C2, which gives sufficient conditions on the spine and radius law for a G1 blend and characterises the local self-intersections (the characteristic circle degenerating).
- Relevance to cadrs: `blend` (conditions on the radius law and spine to keep variable blends G1 and non-self-intersecting; a cheap validity check before building a tool).

### 12. Pegna & Wolter, "Geometrical criteria to guarantee curvature continuity of blend surfaces", ASME J. Mech. Des. 114(1):201-210, 1992; Filkins, "Algorithms for blending surface generation", MIT thesis, 1991
- Canonical URLs: https://doi.org/10.1115/1.2916918 (paywalled); Filkins open at https://archive.org/download/algorithmsforble00filk/algorithmsforble00filk.pdf
- Local PDF: `../papers/blends/Filkins-1991-MIT-thesis-curvature-continuous-blends-linkage-curves.md`
- Algorithm: Linkage Curve Theorem — a G1 join along a curve is G2 iff the normal curvatures agree in one non-tangent direction; Filkins uses it to build G2 blends between NURBS surfaces by lofting: approximate the linkage curves, construct cross-link curves whose end curvature matches the normal section of the underlying surface, sample adaptively, and fit a low-degree B-spline surface through the cross-links.
- Limitations: the loft is an approximation; sampling density drives cost.
- Relevance to cadrs: `blend` curvature sections (a validation criterion and a construction recipe for G2 sections).

### 13. Grandine & Hogan (Boeing), "System, method and computer program product for modeling a transition between adjoining surfaces", US 7,518,609 (2009)
- Canonical URL: https://patents.google.com/patent/US7518609 (free).
- Algorithm: see KQ3; clothoid–arc–clothoid G2 section in a smoothly varying plane, 8×8 Newton for the end points on a "cut radius" circle, faces trimmed at the cut points.
- Limitations: sections combined into a surface by sampling; needs the face intersection curve; expired patent.
- Relevance to cadrs: `blend` G2/curvature sections.

### 14. Kim & Elber, "A symbolic approach to freeform parametric surface blends", J. Visualization and Computer Animation 8(2):69-80, 1997 (Utah TR 1995)
- Canonical URL: https://doi.org/10.1002/(SICI)1099-1778(199703)8:2<69::AID-VIS162>3.0.CO;2-D ; open PDF https://www2.cs.utah.edu/gdc/publications/papers/kim95.pdf
- Local PDF: `../papers/blends/Kim-Elber-1995-symbolic-freeform-surface-blends.md`
- Algorithm: rail curves given on each surface; the cross-boundary tangent field T_i(t) is computed symbolically from the surface derivatives along the rail (several recipes: curve tangent × normal, projections of a guiding curve, combinations of S_u and S_v), normalised to unit length by refinement with a bounded error, then a Hermite/ruled-type blend surface interpolates the rails and tangent fields so G1 holds along the entire boundary rather than at samples.
- Limitations: trimline-based, not rolling-ball; shape depends on tangent magnitude heuristics.
- Relevance to cadrs: `blend_corner` and `surfacing::fill` (exact G1 cross-boundary fields for corner patches), `blend` lofted chain tools.

### 15. Harada & Konno (Ricoh), "Method and apparatus for generating fillet surface between two surfaces", US 5,345,546 (1994); Harada, Konno, Chiyokura, "Variable-radius blending by using Gregory patches in geometric modeling", Eurographics 1991
- Canonical URL: https://patents.google.com/patent/US5345546 (free); Eurographics DOI 10.2312/egtp.19911038.
- Algorithm: a user spine R(t) sampled at 2n points; at each, Newton on (u,v) of both surfaces so that a circle of radius r(t) in the normal plane of the spine is tangent to both tangent planes; movement vectors from differentiating the constraints; quartic Bézier trajectory curves; biquartic Gregory patches (29 control points) between consecutive arcs, G1 between patches.
- Limitations: fixed uniform sampling; Gregory patches are rational with singular corners.
- Relevance to cadrs: `blend` (sliding-circle variable blends as patches).

### 16. Edelsbrunner, Facello, Gloth, Terék, Várady (Geomagic), "Methods, apparatus and computer program products that model three-dimensional surface structures", US 8,004,517 (2011)
- Canonical URL: https://patents.google.com/patent/US8004517 (free).
- Algorithm: reverse-engineering context, but contains the explicit setback vertex blend construction (profile/spring curves, 2n sides, degenerate sides, sphere octant for three-sided case, setback from feature widths, models A1/A2/B/C).
- Relevance to cadrs: `blend_corner` (setback choice and hole boundary layout).

### 17. Rockwood & Owen (Evans & Sutherland), "System for blending surfaces in geometric modeling", US 5,251,160 (1993); Rockwood & Owen, "Blending surfaces in solid modeling", SIAM Geometric Modeling 1987; Hoffmann & Hopcroft, "The potential method for blending surfaces and corners", 1987
- Canonical URL: https://patents.google.com/patent/US5251160 (free).
- Algorithm: implicit blends N<P_i> = R{1 − (Σ Q_i)^(1/t)} with Q_i = [1 − P_i/R_i]_+^t over the primitives' implicit functions, and "displacement" blends D = h solved by Newton; blends on blends by nesting; inside/outside by sign.
- Limitations: implicit (not parametric) surfaces; no circular cross-section; not a B-rep operation.
- Relevance to cadrs: low; only `blend` tool-body construction for free-form "potential" blends.

### 18. Dahl & Krasauskas, "Rational fixed radius rolling ball blends between natural quadrics", CAGD 29(9):691-706, 2012; Dahl, "Rational parametrizations of edge and corner blends for isogeometric analysis", SAGA (Springer Geometry and Computing), 2014, pp. 217-237
- Canonical URLs: https://doi.org/10.1016/j.cagd.2012.06.008 ; https://doi.org/10.1007/978-3-319-08635-4_12 . Paywalled (the Oslo thesis link is dead).
- Algorithm (from the abstract): rational parametrisations of constant- and variable-radius rolling-ball blends between pairs of natural quadrics via rational curves in their bisector surface (Laguerre/Minkowski-space methods); composite corner blends joined G1 with control spheres fixing the radius at key points; boundary curves lie exactly on the primaries ("watertight").
- Relevance to cadrs: `fillet_geom` exact rational cases beyond cylinder/torus, and `blend_corner` exact corner blends between quadrics.

### 19. Eager & Dannenhoffer, "Flends: Generalized fillets via B-splines", AIAA SciTech 2019-1717
- Canonical URL: https://doi.org/10.2514/6.2019-1717 ; open PDF https://acdl.mit.edu/ESP/Publications/AIAApaper2019-1717.pdf
- Local PDF: `../papers/blends/Syracuse-AIAA-2019-1717-Bspline-transition-surfaces-ESP.md`
- Algorithm: the user chooses the offset edges on each face where the transition meets them; a B-spline transition surface is built between the two offset edges with controllable curvature; implemented on OpenCASCADE via ESP.
- Relevance to cadrs: `blend` (an alternative "transition surface" when a rolling-ball fillet fails).

### 20. You, Ugail, Tang, You, Zhang, "Sweeping based controllable surface blending", GRAPP 2010; You & Zhang, "Blending surface modelling using sixth order PDEs", Int. J. CAD/CAM 6(1), 2006; You, Tian, Tang, Chang, Zhang, "Analytical C2 continuous surface blending", Mathematics 12(19):3096, 2024
- Open PDFs: https://www.scitepress.org/Papers/2010/28230/28230.pdf ; https://koreascience.or.kr/article/JAKO200634514931903.pdf ; https://eprints.bournemouth.ac.uk/40506/1/mathematics-12-03096.pdf
- Local PDFs: `../papers/blends/Scitepress-2010-sweeping-based-controllable-surface-blending.md`, `.../KoreaScience-2006-curvature-continuous-blend.pdf`, `.../Bournemouth-2024-mathematics-12-03096-curvature-continuous-blending.pdf`
- Algorithm: PDE blends — a vector-valued 4th-order (G1) or 6th-order (C2) ODE/PDE generator is swept between the two trimlines so that position/tangent/curvature constraints hold exactly at the trimlines; the 2024 paper reduces the PDE to ODEs via elementary functions found in the boundary constraints and solves analytically.
- Limitations: not rolling-ball; shape depends on PDE parameters; mostly for animation/free-form.
- Relevance to cadrs: low; a possible `surfacing::fill` G2 generator, not for `fillet`.

### 21. Vendor documentation: C3D Labs (2019, 2025 blog posts; `MbFilletSurface` reference), T-FLEX/Parasolid overflow options, Siemens Parasolid release notes
- URLs: https://c3dlabs.com/blog/products/fillets-nothing-more-than-more-fillets/ ; https://c3dlabs.com/blog/products/c3d-geometric-kernel-new-features-and-development-trends/ ; https://c3dlabs.com/doc/class_mb_fillet_surface.html ; https://tflex.com/help/eng/T-FLEX%20CAD/17/blendedges_overflowprocessing.htm ; https://blogs.sw.siemens.com/plm-components/parasolid-v34-1-release-highlights/
- Content: see KQ1, KQ4; procedural fillet surface definition (two curves on surfaces + distances + conic coefficient), rolling-disk vs rolling-ball, three vertex-fillet kinds, overflow policy ladder (smooth / cliff edge / notch), tight-corner modes, notch overflow and cap faces.
- Relevance to cadrs: `fillet_geom` (procedural fillet surface as the stored representation), `fillet` (overflow policies to implement: cliff-edge blend function and notch trimming).

---

## What I could not find
- Any open full text of the core CAD/CAGD journal papers: Vida–Martin–Várady 1994, Braid 1997,
  Várady–Rockwood 1997, Várady–Hoffmann 1998, Choi–Ju 1989, Klass–Kuhn 1992, Farouki–Sverrisson
  1996, Lukács 1998, Hermann–Lukács–Várady 1997, Chuang et al. 1995/1997, Pegna–Wolter 1992,
  Barnhill–Farin–Chen 1993, Sanglikar et al. 1990, Hartmann 2001, Dahl & Krasauskas 2012, Dahl
  2014. Elsevier, Springer and the SZTAKI/Cardiff/Oslo repositories either had no file or
  refused the request (403). Descriptions of these in the notes are from abstracts and citing
  works, or marked as background knowledge.
- Braid 1997 "Non-local blending of boundary models": not even an abstract (ScienceDirect 403;
  Crossref and Semantic Scholar carry no abstract).
- "Zhou/Varady/Martin" vertex blend paper: no such paper identified (see KQ2 gaps).
- The Bajaj et al. 1988 PDF is a scan with no text layer; its content is from background knowledge.
- The IISc Voronoi-surface rolling-ball paper (`~asitava/sharad.pdf`): server returned 502.
- Hoffmann & Hopcroft 1987 "The potential method for blending surfaces and corners": the
  Purdue pages 404; no open copy.
- Parasolid: no public functional description of blending (Siemens docs are behind a login);
  ACIS: doc.spatial.com redirects to a ticket/login page. Only end-user option names (overflow,
  notch, cliff edge, cross curves, setbacks, hold line) were found, via T-FLEX and release notes.
- SMLib's "fillet executive" (cited by the cadrs README) has no public description.
- No published performance data for fillets in any kernel, and no paper that profiles where
  fillet time goes.
- OCCT user-guide text on fillet surface types beyond the enum names; the `Blend_Walking_1.gxx`
  path on `master` moved (404) — the V7_8_0 tag paths cited above work.
- The 2010-2026 academic literature on *B-rep fillet construction* appears thin: the only
  2010+ items found are the n-sided patch work (Várady/Salvi group), Dahl & Krasauskas (exact
  quadric blends), PDE blends (You et al.) and Flends (2019); the web-search budget ran out before
  a dedicated sweep of CAD/CAGD 2015-2026 could be done, so this may be incomplete.
