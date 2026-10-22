# Exchange, healing and persistent naming: published methods compared with cadrs_kernel

Scope: (1) healing and sewing of imported B-rep models, (2) recognising analytic surfaces and curves
hidden in B-spline form, (3) fast STEP parsing and model building, (4) persistent naming of topology
across parametric rebuilds. Baseline is the kernel README (`crates/cadrs_kernel/README.md`): STEP
AP214 and IGES read/write with assemblies (`step`, `iges`, `exchange`), other systems' faces rebuilt
into valid bodies with fitted p-curves and faces sewn by their edges (`xbuild`), a heal pipeline
(`heal`, `heal_split`: fix loops, drop small edges, add seams, refit p-curves, fit tolerances, convert
to revolution/B-spline, split by angle/closed/C0/Bézier, unify), analytic recognition of planes,
cylinders, cones, spheres, lines and circles, three-level tolerance model (1e-7/1e-6/1e-5 mm) with
per-vertex/edge tolerances "as OCCT", STEP import 1.9x slower than OCCT 7.8, and history-based
naming (`naming.rs`: `FaceName{op, origin, split}`, edges by two faces, vertices by three faces,
geometric fallback, `Lost` rather than a guess).

Downloaded PDFs are in `../papers/exchange_healing_naming/` (every
file verified to start with `%PDF`). The web-search budget of this session ran out part-way through
(after ~26 searches); the remaining work was done by fetching known URLs and OCCT source directly,
so a few citations below are verified only through Crossref metadata and are marked as such.

---

## Top 5 "must read" (ranked)

1. **Jackson 1995, "Boundary representation modelling with local tolerances"** (Parasolid's tolerant
   modelling; the model cadrs's per-vertex/edge tolerances approximate). Local PDF
   `Jackson-1995-local-tolerances-brep.pdf`. Defines what a tolerant edge/vertex *is* and the
   validity rules (tube/sphere intersections contractible onto common vertices), which is the
   cleanest specification for `Body::check_with(CheckLevel::Full)` on healed bodies.
2. **OCCT `BRepBuilderAPI_Sewing` (source, V7_8_0)**: the only fully specified sewing algorithm
   found (phase order, candidate search by UB-tree of vertex boxes, 8-point edge distance, cutting
   at projected vertices, longer edge kept, `ComputeToleranceVertex` enclosing-sphere formula).
   Directly comparable with `xbuild`'s sewing.
3. **OCCT `GeomConvert_SurfToAnaSurf` / `GeomConvert_CurveToAnaCurve` (source, V7_8_0)**: the
   actual iso-curve + Gauss-field recognition with 20x20 verification grid and
   `tol = min(tol, diag/1000)`; plus **Lukács–Martin–Marshall 1998** (local PDF) for the faithful
   distance functions that make cylinder/cone/torus least-squares fits degrade gracefully into
   simpler types. Together these cover what `heal`'s recognition does and the fitting fallback it
   lacks (torus, least-squares refinement).
4. **Marcheix & Pierra 2002, "A survey of the persistent naming problem"** (SMA 2002, paywalled)
   together with **Agbodan–Marcheix–Pierra–Thabaud 2003** (local PDF) and **Marcheix 2006**
   (local PDF): invariant vs contingent entities, face-graph history, and a two-phase similarity
   matching that handles split/merge by *topological cover* instead of index order. The
   invariant/contingent split is exactly cadrs's `Cap/Side` vs `Unnamed{index}` distinction;
   the cover-based matching is what cadrs's `resolve` lacks for split faces.
5. **Kripac 1997** (CAD 29(2), paywalled) and **Capoyleas–Chen–Hoffmann 1996** (CAD 28(1),
   paywalled): the two canonical face-naming schemes (Kripac's topological-ID graph with split/merge
   bookkeeping, Hoffmann's name = creating feature + adjacency with a case analysis for split and
   merge). Both are cited by every later paper; read via the surveys (Marcheix & Pierra 2002,
   Farjana & Han 2018) if the originals stay paywalled. Plus the practical production systems:
   **Onshape's robust references** (dependency records → encoded queries → pattern-matching against
   history with "unstable" id components and external disambiguation) and **FreeCAD's
   RealThunder element map** (string names built from OCCT `Modified/Generated` history, `;:M`/`;:G`/
   `;:U`/`;:L` postfixes, SHA1-compressed).

---

## 1. Healing and sewing of imported B-rep models

### Takeaway
Every mature healer runs the same broad order: (a) vertex clustering and edge pairing to recover
topology (sewing), (b) per-wire fixes (reorder, drop small edges, connect, fix curves, add
degenerate edges, self-intersection, fill gaps), (c) per-face fixes (orientation, missing seam,
small-area wires), (d) per-shell/solid orientation, then (e) SameParameter / tolerance fitting
last, with every tolerance increase capped by a `MaxTolerance`. Tolerances per vertex/edge are
computed as the smallest sphere/tube enclosing the merged entities (OCCT), or declared
per-entity with validity rules on the intersections of tolerance regions (Parasolid). Nobody
publishes a repair-rate comparison across orderings; the academic literature covers facet-level gap
filling (Barequet–Sharir), rectification of invalid B-reps to an interval representation
(Shen–Sakkalis–Patrikalakis), and history-based repair (Yang & Han 2006), while the real
step-order knowledge lives in OCCT's source and resource files.

### Cited Findings (one entry per method)

#### Jackson, D.J. "Boundary representation modelling with local tolerances." Proc. 3rd ACM Symposium on Solid Modeling and Applications (SMA '95), Salt Lake City, 1995, pp. 247–254.
- Canonical: ACM, ISBN 0-89791-672-7 (DOI not resolved; conference proceedings).
- Open PDF: https://ftp1.cs.wisc.edu/pub/users/prem/jackson-SM-95.pdf — local
  `Jackson-1995-local-tolerances-brep.pdf`.
- Algorithm/data model: every face, edge and vertex carries its own tolerance. An edge whose
  tolerance exceeds the default is a "tolerant edge" and is represented not by one 3D curve but
  by its parameter-space curves on each adjacent face; the tolerance region is a tube of radius =
  edge tolerance centred on one (arbitrary but fixed) of those p-curves, and all the face's
  p-curves must lie inside the tube. Vertices are spheres; p-curve ends must lie inside the
  vertex sphere but need not meet end-to-end. Validity rules: edges only intersect at their end
  vertices — "If the intersection of the tolerance tubes of two edges is nonempty, then each
  connected component of that intersection must intersect a common vertex of the two edges"; the
  same for faces: each component of the intersection of two faces' tolerance regions must be
  contractible onto common edges/vertices. Intersection and coincidence tests are performed with
  the sum of the entities' tolerances ("two entities intersect if they approach within the sum of
  their tolerances"). "When necessary, tolerances are locally increased and topology merged to
  avoid ambiguity." Motivation given: with a single global tolerance, "it is not possible to split
  a short edge, of length less than 4 times the tolerance" consistently, and tangential surfaces
  "may fail to meet at a higher resolution" (Fig. 1). [Source: local PDF, sections 2.2, 2.4, 3.1]
- Limitations: a representation and validity paper, not a repair recipe; no algorithm is given
  for *choosing* the tolerance beyond "increase locally until unambiguous". Parasolid's current
  body-checker documentation is behind a login and was not reachable.
- Relevance to cadrs: `heal` (tolerance fitting) and `topo::check_with(CheckLevel::Full)`: the
  tube/sphere contractibility rules are a precise statement of what "adaptive edge and vertex
  consistency" should verify on a healed body; cadrs's "operations take the larger of their own
  and the entities' tolerances" matches Jackson's sum-of-tolerances rule only approximately
  (max vs sum).

#### OCCT `BRepBuilderAPI_Sewing` (Open CASCADE Technology 7.8.0, LGPL 2.1), `src/BRepBuilderAPI/BRepBuilderAPI_Sewing.{hxx,cxx}`.
- Canonical: https://github.com/Open-Cascade-SAS/OCCT (tag V7_8_0); user guide (sewing is barely
  described there) https://occt3d.com/dev/doc/overview/html/occt_user_guides__shape_healing.html
- Open PDF: none (source code). Not downloaded.
- Algorithm (from the source, read via the raw GitHub files):
  1. `FaceAnalysis`: walks face→wire→edge; degenerate edges kept and counted small; every other
     edge sampled at 5 uniform parameters, and if `2*maxdist <= MinTolerance()` from the chord
     midpoint the edge is "small": its ends are glued and it is replaced by a degenerate edge; a
     face all of whose edges are small is dropped (`myLittleFace`). Glued vertices move to the
     group average; tolerance = max distance from the average to a member + the largest member
     tolerance. `FindFreeBoundaries` maps edges to faces; an edge with one face (and not a seam)
     is a free boundary; in non-manifold mode multi-face edges qualify too.
  2. `VerticesAssembling`: vertex positions in a `BRepBuilderAPI_CellFilter` queried with a
     ±tolerance box; candidate pairs checked by `IsMergedVertices` (the wires sharing a face);
     `CreateNewNodes` merges a group into one vertex at the average position with tolerance
     max_i(|centre − P_i| + tol_i).
  3. `Cutting` (option, default on): a `BRepBuilderAPI_BndBoxTree` (UB-tree) of vertex boxes
     enlarged by `myTolerance*0.5`; each free bound's curve box (`BndLib_Add3dCurve`, enlarged by
     `myTolerance`) selects candidate vertices, which are projected with one `Extrema_ExtPC` solve
     each (accepted if dist² < tol²); `CreateCuttingNodes` reuses an existing end vertex if the
     projection is within `max(0.1*distProj, MinTolerance())`, else inserts a new vertex with
     `Precision::Confusion()` tolerance and splits the bound into sections.
  4. `Merging`: `EvaluateDistances` samples each candidate edge at 8 uniform points, computes the
     max distance forward and reversed (orientation = whichever is smaller), requires max < tol
     and at least half the 8 points to project between the reference's ends; falls back to
     point-on-curve projection; sections with chord length ≤ `MinTolerance` are rejected.
     `FindCandidates` sorts by max then min distance with a recursive mutual-best check (manifold)
     or `AnalysisNearestEdges` (non-manifold). For closed surfaces, `IsMergedClosed` compares 2D
     boxes of the p-curves and accepts only if the wrap-around gap ≤ the direct gap. Split vs whole
     merge chosen by `MinSplitTol < BoundEdgeTol + MinTolerance()`.
  5. `SameParameterEdge`: the *longer* edge (`GCPnts_AbscissaPoint::Length`) is the reference and
     keeps its 3D curve and range; the other's p-curves are added (reversed and re-ranged with
     `GeomLib::SameRange`), `BRepLib::SameParameter` runs in a try block; on failure or
     tolerance > `myTolerance` the roles are swapped and the smaller tolerance wins; final fallback
     samples 23 points on the 3D curve and sets tolerance = max(sqrt(maxSqDist)*(1+1e-7),
     Confusion); an edge whose tolerance exceeds `MaxTolerance()` is nullified.
  6. `ComputeToleranceVertex` (two vertices): with R_m ≥ R_n, dR = R_m − R_n, D = |P_m − P_n|: if
     D ≤ dR keep (P_m, R_m); else R = (R_m + R_n + D)/2, centre = (P_m + P_n − (P_n − P_m)·dR/D)/2
     — the smallest sphere enclosing both vertex spheres. Three-vertex version (closed edges):
     centroid, R = max_i(|c − P_i| + R_i). (OCCT bug 26241 records that the pre-6.9.1 formula was
     "too simple" — https://tracker.dev.opencascade.org/view.php?id=26241.)
  7. Then `EdgeProcessing`, `CreateSewedShape` (`BRepTools_Quilt`; faces grouped into shells),
     `EdgeRegularity`, `SameParameterShape`.
  Defaults: `myTolerance = max(tol, Confusion)` (ctor default 1e-6), `myMinTolerance =
  myTolerance*1e-4` floored at Confusion, `myMaxTolerance = Infinite`; "local tolerances" mode
  uses `WorkTolerance = myTolerance + tolEdge1 + tolEdge2`.
  [Sources: raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/BRepBuilderAPI/BRepBuilderAPI_Sewing.cxx and .hxx]
- Limitations: "Only manifold shapes will be found. Sewing will not be done in case of multiple
  edges" (hxx); one global tolerance drives everything unless the local-tolerance mode is on;
  the user must supply the tolerance (OCCT forum reports of a 71-surface IGES model left with
  unsewn borders: https://old.opencascade.com/content/holes-while-importing-iges).
- Relevance to cadrs: `xbuild` (faces "sewn by their edges") and `heal` tolerance fitting. Items
  to compare: whether cadrs cuts edges at other edges' vertices before pairing (OCCT's `Cutting`),
  whether it uses a spatial index for candidate pairs, which edge survives a merge (OCCT: the
  longer), and the enclosing-sphere vertex tolerance formula.

#### OCCT Shape Healing user guide and `ShapeFix_*` sources (7.8/dev).
- Canonical: https://occt3d.com/dev/doc/overview/html/occt_user_guides__shape_healing.html ;
  STEP resource file `src/XSTEPResource/STEP` (V7_8_0).
- Order of fixes (documented): `ShapeFix_Wire::Perform` runs FixReorder → FixSmall →
  FixConnected → FixEdgeCurves (FixReversed2d, FixRemovePCurve, FixAddPCurve, FixAddCurve3d,
  FixRemoveCurve3d, FixSeam, FixShifted, FixSameParameter) → FixDegenerated → FixSelfIntersection
  (only if ClosedWireMode) → FixLacking. `ShapeFix_Face::Perform` (source): wire pass 1 with
  lacking/self-intersection off (and, with `AutoCorrectPrecisionMode`, precision lowered to half
  the least edge size) → FixPeriodicDegenerated (cone apex) → FixMissingSeam (may split the face)
  → wire pass 2 (lacking, self-intersection, notch, tail; FixLoopWire) → SplitWire check →
  FixWiresTwoCoincEdges → FixIntersectingWires → FixOrientation → FixAddNaturalBound →
  FixSplitFace → FixSmallAreaWire. `ShapeFix_Shell` makes face orientations coherent, creating
  several shells for a Möbius strip; `ShapeFix_Solid` (`FixShellMode`, `CreateOpenShellMode`).
- Tolerance policy: `Precision` (working precision, default 1e-7 in the guide's workflow) detects
  problems; `MaxTolerance` caps what a fix may raise a tolerance to — "A problem needing a larger
  value is not fixed", except that the cap is *not* applied in `ShapeFix_SameParameter()` and
  `ShapeFix_Edge::FixVertexTolerance()`; `MinTolerance` is the minimum edge length (shorter edges
  removed if `ModifyTopologyMode`). Three strategies per fix: raise a tolerance, change topology
  (`ModifyTopologyMode`, default false), change geometry (`ModifyGeometryMode`, default true).
  Self-intersection within one edge → raise a vertex tolerance below MaxTolerance; adjacent edges
  → move the common vertex, or raise its tolerance when topology may not change; non-adjacent →
  raise the nearest vertex's tolerance only if below MaxTolerance. `FixSameParameter` raises edge
  tolerance to the measured 3D/2D deviation.
- FixMissingSeam decision (source): only for surfaces closed in U or V; `CheckWire` sums the
  pcurve end-minus-start vectors per wire; a net displacement of about one period (within 10%)
  marks the wire "open" in that direction; two open wires are paired (one reversed if flags
  equal), a single open wire on sphere/torus/B-spline gets a synthesized degenerate edge; the seam
  position prefers the end vertex closest to 0, with preference for a position where ends of both
  wires coincide, else the surface's lower bound; `ShapeFix_ComposeShell` builds the closed face.
- FixOrientation (source): one-edge wires sampled at 10 points and dropped if length ≤ Confusion;
  single wire → `ShapeAnalysis::IsOuterBound`; several wires → 2D classification
  (`BRepTopAdaptor_FClass2d`) of the infinite point and of other wires' vertices/midpoints, with
  period shifts, reversing a wire whose classification disagrees with the infinite point.
- STEP import default sequence (`XSTEPResource/STEP`): `FromSTEP.exec.op : FixShape` only, with
  `Tolerance3d = &Runtime.Tolerance`, `MaxTolerance3d = &Runtime.MaxTolerance`,
  `MinTolerance3d = 1.e-7`, `AutoCorrectPrecisionMode 1`, `ModifyTopologyMode 0`,
  `ModifyGeometryMode 1`, `ClosedWireMode 1`, `PreferencePCurveMode 1`,
  `FixEdgeSameParameterMode 0`, `FixVertexPositionMode 0`, `FixTailMode 0`, everything else −1
  ("only if needed"). `ToSTEP.exec.op : SplitCommonVertex,DirectFaces`.
- Other operators and ordering constraints (guide): DirectFaces, SameParameter,
  BSplineRestriction, ElementaryToRevolution, SplitAngle, SurfaceToBSpline, ToBezier,
  SplitContinuity, SplitClosedFaces, FixGaps, FixFaceSize, DropSmallEdges, FixShape,
  SplitClosedEdges; "Run SplitContinuity after BSplineRestriction", "Call FixGaps when
  FixFaceSize and/or DropSmallEdges are called", "Run SameParameter after operators that split
  edges". `ShapeFix_Wireframe::FixWireGaps`: `FixGap2d` moves 2D curve ends to their midpoint,
  `FixGaps3d` moves 3D ends to a common vertex, `FixGapsByRanges` (off) first tries exact
  intersections/projections; remaining gaps convert the curves to B-spline and shift the ends.
  `ShapeUpgrade_UnifySameDomain`: faces/edges "same-domain" if neighbours lie on coincident
  surfaces/curves; `ConcatBSplines` merges C1 B-spline edges.
  [Sources: user guide URL above; raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/ShapeFix/ShapeFix_Face.cxx; .../src/XSTEPResource/STEP]
- Limitations: the guide never states a measured repair rate; `ShapeFix_Shape::Perform` "applies
  an optimal set of fixes" without a documented order at the shape level; the MaxTolerance
  exemption for SameParameter means an edge can end up far above the cap.
- Relevance to cadrs: `heal` and `heal_split`. cadrs's documented pipeline ("fix (loops reordered
  and closed, small edges dropped, missing seams added, face senses and shell orientation,
  p-curves refitted, tolerances fitted), gaps, same-parameter, tolerance update, direct faces,
  conversion to surfaces of revolution and to B-splines, splits by angle, of closed faces, at C0
  breaks and into Bézier patches, unify and wire tails") matches OCCT's operator set one-to-one;
  what to check is (a) whether cadrs applies the three-strategy policy and a MaxTolerance cap,
  (b) whether `AutoCorrectPrecisionMode` (precision = half the least edge) is mirrored, and (c)
  the seam-position and outer-wire rules above.

#### Barequet, G.; Sharir, M. "Filling gaps in the boundary of a polyhedron." Computer Aided Geometric Design 12(2):207–229, 1995.
- Canonical: https://doi.org/10.1016/0167-8396(94)00011-G
- Open PDF: https://www.cs.jhu.edu/~misha/ReadingSeminar/Papers/Barequet95.pdf — local
  `Barequet-1995-filling-gaps-polyhedron.pdf`.
- Algorithm: (1) Data acquisition: build adjacency by sorting edges on endpoint ids, store in a
  quad-edge structure, find connected components by DFS on the dual graph and orient facets
  consistently (or detect non-orientable components); border edges are found as the formal sum of
  the oriented facet boundaries (e + (−e) = 0), giving edge-disjoint directed border cycles, broken
  into simple cycles. (2) Matching: each border is discretized at equal arc length s into
  "footprints"; every pair of footprints closer than ε (a range-search query; ε is "our a priori
  estimation of the physical size of the gaps", not derived from the input) casts a vote for the
  (border, border, shift) match; votes become scored partial-match suggestions; portions shared by
  several significant matches are simply dropped; choosing a consistent maximum-score subset
  (each border oriented one way, only inverted matches accepted) is proved NP-hard ("Choose-1")
  and solved by an approximation. (3) Filling: matched chains are stitched like merging two sorted
  lists, advancing the chain that gives the smaller new-triangle perimeter; remaining holes (at
  junctions of cracks) are found as the new borders and closed by minimum-area 3D polygon
  triangulation. [Source: local PDF §3–6]
- Limitations: facet (polyhedral) models only; ε is a user constant; NP-hard selection step
  approximated; overlapping matches discarded rather than resolved.
- Relevance to cadrs: `meshsolid` (triangle meshes to solids) and, conceptually, `xbuild`'s edge
  pairing (vote by discretized footprints rather than by edge end-vertices, which is more robust
  when edges are split differently on the two sides — the "cutting" case OCCT handles by
  projection).

#### Barequet, G.; Kumar, S. "Repairing CAD models." Proc. IEEE Visualization '97, Phoenix, 1997, pp. 363–370.
- Canonical: https://doi.org/10.1109/visual.1997.663904 (Crossref). Author's list
  https://barequet.cs.technion.ac.il/papers.html ("reprint available" by request; no PDF link).
- Open PDF: none found. Companion journal versions: Barequet, "Using geometric hashing to repair
  CAD objects", IEEE CS&E 4(4):22–28, 1997, https://doi.org/10.1109/99.641606; Barequet, Duncan,
  Kumar, "RSVP: a geometric toolkit for controlled repair of solid models", IEEE TVCG 4(2):162–177,
  1998, https://doi.org/10.1109/2945.694983 (both paywalled; Crossref-verified).
- Algorithm (as described by the 2017 survey citing it): "Hashing techniques heal gaps by assuming
  that one border is a slightly deformed sub-curve of the other. Then, they find the translation
  and rotation of the sub-curve that yield the best least-squares fit to the appropriate portion of
  the larger curve" — i.e. geometric hashing of border signatures replaces the 1995 voting; RSVP
  adds user control over which repairs are applied. [Source: González-Lluch et al. 2017 §(mesh
  repair), local PDF `Gonzalez-Lluch-2017-cad-quality-survey.pdf`]
- Limitations: still facet-level; details of the 1997 paper could not be verified from a PDF.
- Relevance to cadrs: `meshsolid` only.

#### Shen, G.; Sakkalis, T.; Patrikalakis, N.M. "Boundary representation model rectification." Graphical Models 63(3):177–195, 2001; and Sakkalis, Shen, Patrikalakis, "Representational validity of boundary representation models", CAD 32(12):719–726, 2000.
- Canonical: https://doi.org/10.1006/gmod.2001.0539 ; https://doi.org/10.1016/s0010-4485(00)00047-6
  (Crossref-verified). Shen's MIT PhD thesis "Analysis of boundary representation model
  rectification" (2000): https://dspace.mit.edu/handle/1721.1/88491 (the bitstream download
  returned HTTP 429 during this session; not downloaded).
- Open PDF: the MIT thesis is the open copy (retry the DSpace bitstream
  `/bitstreams/74d39ce7-2e51-4e21-b98d-447ebd40dbcd/download`).
- Algorithm: this is the "topology construction from tolerant geometry" line from the
  Patrikalakis group — the validity paper formalises when a B-rep whose geometry is only known to
  interval/tolerance accuracy still represents a valid solid; the rectification paper and the
  thesis rectify an invalid B-rep into an interval solid model (geometry replaced by interval
  boxes/surfaces so that the topology becomes consistent). Details beyond the abstracts could not
  be read in this session (journal paywalled, thesis download throttled). [Sources: MIT DSpace
  record; dblp/Crossref records]
- Limitations: interval-geometry representation, not the per-entity tolerance representation
  cadrs uses; unknown performance.
- Relevance to cadrs: `topo::check` validity rules for tolerant bodies (what "valid" means when
  edges have tolerance); a theoretical cross-check for Jackson's rules.

#### Yang, J.; Han, S. "Repairing CAD model errors based on the design history." Computer-Aided Design 38(6):627–640, 2006.
- Canonical: https://doi.org/10.1016/j.cad.2006.02.007 (Crossref-verified). Related: Yang, Han,
  Park, "A method for verification of computer-aided design model errors", J. Eng. Design
  16(3):337–352, 2005, https://doi.org/10.1080/09544820500126565.
- Open PDF: none found (KAIST DSpace was unreachable: `dspace.kaist.ac.kr` DNS failure).
- Method (from the surveys): instead of repairing the B-rep, errors found by a checker are mapped
  back to the feature/sketch in the design history (macro-parametric approach) and the model is
  regenerated. [Source: González-Lluch et al. 2017, refs [28],[88]]
- Limitations: needs the design history; not applicable to a neutral STEP file.
- Relevance to cadrs: not `heal`; relevant to `cadrs_core` rebuilds only. (The user's
  "Yang/Han/Lee 'A new sewing approach'" could not be located — see gaps.)

#### Gu, H.; Chase, T.R.; Cheney, D.C.; Bailey, T.; Johnson, D. "Identifying, correcting, and avoiding errors in computer-aided design models which affect interoperability." J. Computing and Information Science in Engineering 1(2):156–166, 2001.
- Canonical: https://doi.org/10.1115/1.1384887 (paywalled; abstract via
  https://findanexpert.unimelb.edu.au/scholarlywork/1230217-...).
- Open PDF: none found.
- Content: diagnostic software and a visual catalogue of interoperability errors from a 140-part
  industrial suite; "Geometry errors turn out to be more common than topology errors". The user's
  "Gu/Chase/Sheng 'Healing of geometric models'" matches no record; the closest are this paper and
  Sheng & Meier, "Generating topological structures for surface models", IEEE CG&A 15(6):35–41,
  1995, https://doi.org/10.1109/38.469517 (Crossref-verified; the classic "build topology from a
  bag of trimmed surfaces by matching trimming curves" paper; paywalled).
- Relevance to cadrs: `xbuild` test corpus design (error taxonomy), not an algorithm.

#### Butlin, G.; Stops, C. "CAD data repair." Proc. 5th International Meshing Roundtable, 1996, pp. 7–12 (FEGS / CADfix).
- Canonical: Sandia IMR archive (https://imr.sandia.gov returned HTTP 403 in this session);
  cited in González-Lluch et al. 2017 ref [49].
- Open PDF: not reachable.
- Content: not verifiable beyond the citation; CADfix's public material describes repair of
  gaps/missing faces into a solid and de-featuring (collapsing short edges, removing fillets and
  small holes) — https://www.iti-global.com/cad-to-cae.
- Relevance to cadrs: `heal` ("drop small edges", "fit tolerances"); nothing algorithmic recovered.

#### Patel, P.S.; Marcum, D.L.; Remotigue, M.G. "Stitching and filling: creating conformal faceted geometry." Proc. 14th International Meshing Roundtable, 2005, pp. 239–256.
- Canonical: https://doi.org/10.1007/3-540-29090-7_14 (Crossref-verified). Related MSState
  dissertation "Automatic CAD model processing for downstream applications"
  https://scholarsjunction.msstate.edu/td/826 (PDF download returned HTML; not stored).
- Method: iterative vertex-pair contraction (stitching) closes small gaps; larger gaps are filled
  by adding faces; "Processed models are guaranteed to be free of intersecting faces or surfaces".
  [Source: scholarsjunction abstract]
- Relevance to cadrs: `meshsolid`.

#### González-Lluch, C.; Company, P.; Contero, M.; Camba, J.D.; Plumed, R. "A survey on 3D CAD model quality assurance and testing tools." Computer-Aided Design 83:64–79, 2017 (arXiv 1611.01765).
- Canonical: https://doi.org/10.1016/j.cad.2016.10.003 ; open PDF https://arxiv.org/pdf/1611.01765
  — local `Gonzalez-Lluch-2017-cad-quality-survey.pdf`.
- Use: bibliography for the healing literature (refs [7],[9],[28],[49],[60],[61],[72],[73],[88],
  [121],[122]); states "Gaps can be stitched by physically moving vertices ... although this change
  introduces an error in the order of the resolution tolerance".
- Relevance to cadrs: reading list only.

#### Parasolid / ACIS / HOOPS Exchange documentation
- Parasolid: Siemens docs behind login; only the Jackson paper and marketing PDFs
  (https://www.plm.automation.siemens.com/en_us/Images/Siemens-PLM-Components-Parasolid-fs_tcm1023-7381.pdf)
  were reachable. Siemens' NX blog notes a solid that passes B-rep checks "is not guaranteed to
  succeed in all later operations" (https://blogs.sw.siemens.com/nx-design/the-virtual-and-the-reality-a-brief-history-of-3d-cad-quality/).
- ACIS (Spatial) healing docs: not reachable without the doc portal; not covered.
- HOOPS Exchange defines sewing by "the maximum tolerance for the sewing (in mm)"
  (https://docs.techsoft3d.com/exchange/latest/api/group__a3d__Healing.html).

### Inferences
- The ordering that every implementation converges on is: cluster vertices → pair/cut/merge edges
  (sewing) → per-wire → per-face → per-shell → SameParameter/tolerance fitting last. Doing
  SameParameter earlier wastes work because later splits invalidate it (OCCT's explicit rule).
- Per-vertex tolerance after a merge should be the enclosing sphere of the merged vertices
  (OCCT's formula), not the max of distances alone; per-edge tolerance should be the measured
  3D-vs-p-curve deviation (SameParameter), capped by a MaxTolerance with the fix refused above it.
- cadrs's "operations take the larger of their own and the entities' tolerances" is weaker than
  Jackson's "sum of tolerances" test for intersection/coincidence; a boolean between two tolerant
  bodies may miss a coincidence that Parasolid would detect.

### Gaps
- No published side-by-side repair-rate comparison of healing step orders exists in what was found.
- The Yang/Han/Lee "new sewing approach" paper and the "Gu/Chase/Sheng" healing paper could not be
  identified; the Butlin & Stops paper and Barequet & Kumar 1997 have no open PDF.
- Parasolid body checker and ACIS healing documentation are login-only.

---

## 2. Recognising analytic surfaces and curves hidden in B-spline form

### Takeaway
Production code (OCCT 7.6+ `GeomConvert_SurfToAnaSurf`, `ShapeAnalysis_CanonicalRecognition`)
recognises by *sampling*: fit a line/circle to two mid iso-curves, classify the pair
(circle+circle → sphere/torus, line+circle → cylinder/cone), build the candidate from three isos,
then verify on a 20x20 grid of projected distances against `tol = min(tol, diag/1000)`, with a
7x7 Gauss-curvature fallback for cylinders plus optional PSO+Powell least-squares. Research
fitting (Lukács–Martin–Marshall, Benkő–Várady, Vančo–Brunnett) adds what OCCT lacks: faithful
least-squares distance functions that degrade gracefully (torus→sphere/cone/cylinder as radii go
to 0/∞), tests by distributions of normals/curvature rather than isos, and type decisions by
comparing residuals across the type hierarchy. Tolerances are user-given absolute distances;
OCCT additionally tightens relative to the surface diagonal and uses `1000*tol` for the radius
spread test.

### Cited Findings

#### OCCT `GeomConvert_SurfToAnaSurf` (V7_8_0, `src/GeomConvert/GeomConvert_SurfToAnaSurf.cxx`) and `ShapeAnalysis_CanonicalRecognition`.
- Canonical: https://github.com/Open-Cascade-SAS/OCCT ; reference docs
  https://dev.opencascade.org/doc/occt-8.0.0/refman/html/class_shape_analysis___canonical_recognition.html ;
  product page https://opencascade.com/content/canonical-recognition.
- Algorithm (from source): `ConvertToAnalytical(tol)` substitutes [−1,1] for infinite bounds; for a
  `Geom_SurfaceOfRevolution` with a line basis, `CheckVTrimForRevSurf` cuts the V range where the
  basis crosses the axis and keeps the larger part "to avoid two-parts cone-like surface". If the
  adaptor already reports plane/cylinder/cone/sphere/torus it is copied with gap 0. B-spline and
  Bézier surfaces are segmented to the requested domain. Tolerance tightened to
  `min(tol, diagonal/1000)`. Order: (1) `GeomLib_IsPlanarSurface` → plane; (2) the mid-U and
  mid-V iso-curves are converted with `GeomConvert_CurveToAnaCurve::ComputeCurve` to a line or
  circle; (3) two circles → `TryTorusSphere`: take R from one circle, two more isos at 1/3 and 2/3
  of the other direction must be circles with radius within tol of R; centres coincident → sphere;
  else the circle through the three centres (`GetCircle`) gives the major radius and the torus
  axis is the normal of the centres' plane; V-isos tried first, then U-isos. (4) line + circle →
  `TryCylinerCone`: first and last isos converted to circles (R1, R3) and compared with the middle
  (R2): all equal within tol → cylinder with axis from first to last centre; strictly monotonic →
  cone with semi-angle = angle between the line direction and the centre line, apex on the smaller
  side; a non-circle end iso is taken as radius 0. (5) `TryCylinderByGaussField` (only if no
  circle candidate and one iso is a line): 7x7 grid of cell centres, `GeomLProp_SLProps` order 2;
  reject if curvature undefined, any umbilic, or sqrt|K| > tol, or min principal curvature not
  ~0; radius = 1/|k_max|, axis = mean min-curvature direction, each sample must agree with the
  running averages within tol; radius spread tested as 3σ/√n against `1000*tol`; optional
  `GetCylByLS`: if the max radial deviation > Confusion, a particle-swarm (`math_PSO`) search ±20%
  on axis point and radius, then `math_Powell` with directions perpendicular to the axis.
  (6) `ComputeGap`: every candidate checked on a 20x20 grid of cell centres, each point projected
  by `ElSLib::Parameters/Value`, gap = max distance, early exit when gap > tol. Selection mode:
  Simplest (first accepted in order plane, cylinder, cone, sphere, torus), Target (requested type
  only), MinGap (smallest gap). [Source: raw GitHub source file]
- Code quirks noted during reading: the diagonal sample evaluates `Value(U1+U2, (V1+V2)/2)`
  (looks like a missing /2); `TryCylinerCone` uses `(Last−First)/2` as a parameter instead of
  `First + range/2`; a `NbU*NbU` buffer filled to `NbU*NbV`.
- Limitations: recognition is iso-based, so a B-spline cylinder whose iso-curves are not
  generators/circles (e.g. a re-parametrised or skewed patch) falls to the 7x7 Gauss test, and
  only cylinders have that fallback; torus only from circle-circle isos; no extrusion-surface
  handling; no periodic handling beyond the given bounds; canonical recognition accepts only
  plane/cylinder/cone/sphere for shapes (torus only inside SurfToAnaSurf).
- Relevance to cadrs: `heal` ("imported B-spline surfaces and curves are recognised as planes,
  cylinders, cones, spheres, lines and circles"). cadrs lacks the torus case and a least-squares
  refinement; the `diag/1000` tightening and the 20x20 verification grid are concrete numbers to
  compare against ours.

#### OCCT `GeomConvert_CurveToAnaCurve` (V7_8_0).
- Algorithm: unwrap trimmed curves; modes Target/Simplest/MinGap. Line: B-spline/Bézier poles are
  the samples, otherwise 23 uniform points; `IsLinear` takes the most distant pair (O(n²)),
  builds the line through it and requires every sample within tol; final line through the end
  points. Circle: three points at c1, (2c1+c2)/3, (c1+2c2)/3 through `gce_MakeCirc` (reject
  radius below resolution), re-oriented so the parameter origin is at P0, verified on 21 uniform
  samples; parameters normalised into [cf, cf+2π). Ellipse: 5 samples (span shortened by 1/5 for
  closed curves), centroid-shifted, planarity by `IsArrayPntPlanar`, conic A x²+Bxy+Cy²+Dx+Ey=−1
  solved by `math_Gauss`, ellipse iff AC−B²/4>0 and Q1(A+C)<0, rejected if Rmax−Rmin < Confusion
  (then it is a circle), verified on 20 samples. MinGap order: line at tol, then circle at
  min(tol, line gap), then ellipse at min(tol, circle gap) — "a later, more complex shape must fit
  at least as well as the simpler candidates"; short curves (both chords < 2*tol) take whichever
  of line/circle has the smaller gap. [Source: raw GitHub source file]
- Limitations: parabola/hyperbola are commented-out placeholders; three-point circle construction
  is sensitive to the chosen thirds on noisy curves.
- Relevance to cadrs: `heal` curve recognition and `step`/`iges` import (recognising circles in
  B-spline edges so that `ssi` and `fillet` get exact geometry).

#### Lukács, G.; Martin, R.R.; Marshall, A.D. "Faithful least-squares fitting of spheres, cylinders, cones and tori for reliable segmentation." ECCV 1998, LNCS 1406, pp. 671–686.
- Canonical: https://doi.org/10.1007/BFb0055697 . Open copies: Cardiff ORCA accepted manuscript
  (CC-BY) https://orca.cardiff.ac.uk/id/eprint/48135/ (returned 403 to automated fetches); the
  Springer PDF https://rd.springer.com/content/pdf/10.1007/BFb0055697.pdf was served without a
  paywall and is stored locally as `Lukacs-1998-faithful-fitting.pdf`. SZTAKI tech-report version
  "Geometric least-squares fitting of spheres, cylinders, cones and tori" (1997):
  https://eprints.sztaki.hu/1458/.
- Algorithm: nonlinear least squares on the *geometric* distance d(s, p_i) of each point to the
  parametrised surface family s, but with the square roots in d replaced so that the modified
  distance is "faithful": it "is 0 where the true distance is 0, and, secondly, at these points,
  the derivatives with respect to the parameters are the same for the true distance and the
  modified definition" (Eq. 5–7). Parametrisations are chosen so that critical values do not lie
  on the border of the parameter domain. The torus distance (Eq. 28) is written as a sphere
  distance minus a correction such that "In special cases the torus may degenerate into a sphere,
  as the major radius vanishes, or into a cone, as the minor radius tends to infinity. Our
  equations appropriately reduce to those for sphere or cone fitting"; as the curvature parameter
  s→0 the expression "degenerates to the distance function of a cylinder". Initial estimates come
  from the segmentation (seed regions grown from a depth map); the paper contrasts with Taubin's
  algebraic fits, which are only usable when f is linear in the parameters. [Source: local PDF
  §3–6]
- Limitations: needs an initial guess and a nonlinear solver (Levenberg–Marquardt type) per
  surface; written for point clouds, so the sampling of a B-spline surface must be chosen by the
  caller; no planes (trivial) and no general quadrics.
- Relevance to cadrs: `heal` recognition fallback when iso-based tests fail (skewed or
  re-parametrised patches), and the missing torus case; the type decision becomes "fit the
  richest type, then read off the degenerate parameters".

#### Benkő, P.; Martin, R.R.; Várady, T. "Algorithms for reverse engineering boundary representation models." Computer-Aided Design 33(11):839–851, 2001.
- Canonical: https://doi.org/10.1016/S0010-4485(01)00100-2 ; accepted manuscript at Cardiff ORCA
  https://orca.cardiff.ac.uk/id/eprint/48124/ (403 to automated fetch; SZTAKI record
  https://eprints.sztaki.hu/2654/ has no full text). Not downloaded.
- Content: the full reverse-engineering pipeline (segmentation → surface fitting → constrained
  fitting → B-rep construction), including the classification of regions into planes, cylinders,
  cones, spheres, tori, extrusions and surfaces of revolution. Details could not be read in this
  session.
- Relevance to cadrs: the "surfaces of revolution and extrusion" recognition (cadrs converts to
  revolution but the README does not say it *recognises* revolutions from B-splines).

#### Benkő, P.; Várady, T. "Segmentation methods for smooth point regions of conventional engineering objects." Computer-Aided Design 36(6):511–523, 2004 (and "Direct segmentation of smooth, multiple point regions", GMP 2002, pp. 169–178).
- Canonical: https://doi.org/10.1016/s0010-4485(03)00159-3 (Crossref-verified);
  https://doi.org/10.1109/gmap.2002.1027508 .
- Open PDF: none found (SZTAKI author list https://eprints.sztaki.hu/view/creators/Benk==0151=3AP=2E=3A=3A.default.html).
- Content (from the abstracts/records): "direct segmentation" classifies a point region by tests
  on the distribution of estimated normals and principal curvatures (planes: all normals equal;
  cylinders/cones: normals lie on a great circle / a cone of the Gauss sphere; spheres: constant
  curvature; translational/rotational sweeps: tests on the normal and position fields), so that
  the surface type is decided *before* fitting, as opposed to fitting every type and comparing.
  The exact test thresholds could not be read.
- Relevance to cadrs: `heal` recognition — a Gauss-map based pre-classification of a sampled
  B-spline patch (cheap: evaluate normals on a grid) would decide which analytic fit to try, which
  OCCT does only for cylinders (its 7x7 Gauss-field test).

#### Vančo, M.; Brunnett, G. "Direct segmentation of algebraic models for reverse engineering." Computing 72:207–220, 2004 (also in "Geometric Modelling", Springer).
- Canonical: https://doi.org/10.1007/978-3-7091-0587-0_17 (Crossref returned the book-chapter
  DOI; journal volume 72 pp. 207–220).
- Open PDF: none found.
- Content: not readable in this session; the same direct-segmentation idea (type from local
  differential properties, then algebraic fit).
- Relevance to cadrs: as above.

#### Schnabel, R.; Wahl, R.; Klein, R. "Efficient RANSAC for point-cloud shape detection." Computer Graphics Forum 26(2):214–226, 2007.
- Canonical: https://doi.org/10.1111/j.1467-8659.2007.01016.x (Crossref-verified; author PDF
  usually at cg.cs.uni-bonn.de; not fetched).
- Content: minimal-sample candidates (plane: 3 points with normals; sphere: 2; cylinder: 2;
  cone: 3; torus: 4) scored by inliers within ε and a normal-angle threshold, with an octree
  localisation; the standard robust alternative to least squares for noisy data.
- Relevance to cadrs: not needed for exact B-splines (no outliers); mention only as the
  robust-fit option for `meshsolid` surfaces.

#### Analysis Situs blog, "On recognizing canonical surfaces: checking" (Slyadnev).
- https://quaoar.su/blog/page/on-recognizing-canonical-surfaces-checking — says recognition is
  "geometric, not parametric" ("If something looks like a plane, we declare it a plane"), that
  kinematic (extrusion/revolution) surfaces must be considered, that freeform entities can be
  mapped to analytic ones on the fly without rebuilding the B-rep, and that "there is no
  comprehensive open-source analytical recognition toolbox" beyond `GeomLib_IsPlanarSurface`.
  No tolerances or algorithms given.

### Inferences
- A two-stage recogniser is the state of practice: cheap classification (planarity test; iso
  types; Gauss-map/curvature statistics on a coarse grid) → candidate construction from a few
  samples → dense verification (OCCT's 20x20 grid, max projected distance ≤ tol). cadrs should
  compare its sample counts and whether the tolerance is tightened relative to the patch size.
- Where OCCT constructs the candidate from three isos, a least-squares refinement with a
  faithful distance function (Lukács et al.) would reduce the gap for patches that are analytic
  only to within file precision, and would add the torus case cadrs lacks.
- For curves the "tighten tolerance to the simpler candidate's gap" rule (line → circle → ellipse)
  is a cheap way to prefer the simplest type without a separate model-selection criterion.

### Gaps
- Benkő–Várady and Vančo–Brunnett thresholds (how "normals on a great circle" is tested
  numerically) could not be read; the Cardiff ORCA copies refuse automated download — fetch them
  by hand.
- No paper was found that specifically treats recognising analytic surfaces *from B-spline control
  nets* (e.g. exact tests on poles/weights for rational circles); OCCT uses poles only for the
  line test.

---

## 3. Fast STEP parsing and model building

### Takeaway
The fast Part 21 readers (web-ifc, openbimrs/step, ifc-lite, STEPcode's lazy loader) share one
design: a byte-class/SIMD tokenizer over the whole buffer, entity ids turned into integer offsets
into a "tape" so instances are decoded on demand, strings borrowed not copied, and parallel
parsing by splitting the buffer at guessed record boundaries. Reported self-measured throughputs
are 0.4–1.7 GB/s for tokenising/scanning and 3–5x on 8 threads. OCCT's reader is three-stage
(load = parse and memorise, check, translate) with `ShapeFix` on every result and an opt-in
multithreaded mode (parsing/transferring/healing on OCC threads or TBB) that was announced for
7.9 but, as of August 2025, "is not available in any official versions". The only paper found
reports a hierarchical thread-pool STEP parser "about one times faster" than serial and ~1.5x
faster than OpenCASCADE.

### Cited Findings

#### Bi Yuli; Liu Jiazheng; Yang Yijun; Zeng Wei; Yong Junhai. "A Parallel Algorithm to Parse STEP Files Based on Multiple Threads." Journal of Computer-Aided Design & Computer Graphics 35(12):1842–1850, 2023 (Chinese).
- Canonical: https://doi.org/10.3724/SP.J.1089.2023.2023-00037 (https://jcad.cn/en/article/doi/10.3724/SP.J.1089.2023.2023-00037).
- Open PDF: the journal page offers full text in Chinese (not downloaded; abstract only read).
- Method (abstract): "hierarchical parallel data parsing algorithm based on multithreading" that
  "uses the hierarchical structure of model data" and a thread pool to parallelise parsing at
  each layer (shape → faces → edges → curves), implemented in the OpenGE engine; "the running
  speed of the parallel parsing program is about one times faster than that of the serial
  parsing program based on single thread and about one and a half times faster than that of
  OpenCASCADE". [Source: journal abstract page]
- Limitations: speedup against serial is only ~2x; the file-splitting and cross-thread reference
  resolution are not described in the abstract.
- Relevance to cadrs: `step` (model building): a per-level thread pool is the same shape as
  cadrs's rayon tessellation; a 1.5x over OCCT would turn the current 1.9x deficit into a lead.

#### openbimrs/step (Rust crate `openbim-step`), release notes 0.6–0.11.
- Canonical: https://github.com/openbimrs/step/releases ; https://docs.rs/crate/openbim-step
- Design (self-reported): per-byte lexer loop driven by a byte-class lookup table instead of
  range comparisons, per-token lexers inlined into the tokenizer; `memchr` vectorised search to
  jump to the next backslash/apostrophe in strings; escape-free string bodies copied in one step;
  `Str` and `InstanceId` stored inline up to 22 bytes/digits; parameter lists allocated at exact
  final length; `parse_events_borrowed` returns `Cow` slices of the input; `parse_parallel_with`
  starts slices at guessed record boundaries and falls back to sequential if a slice does not end
  exactly at a boundary; `scan` reads only the header and yields each record's id, name and byte
  span, decoding on demand. Figures: tokenizer 407–541 MB/s (from 183–317), 8 threads 3–5x,
  16 threads with mimalloc 600–1000 MB/s, scan ~1.4–1.7 GB/s on large IFC files, owned-model peak
  memory −28% to −34% (600→431 MB on a 109 MB Revit IFC; 97 MB AP214 file −32%), heap allocations
  7.5M→2.6M; tested on OCCT's `linkrods.step` and `screw.step` among 4,316 files.
  [Source: release notes page]
- Limitations: all numbers are the maintainers' own; the 0.8.0 notes say the build VM was under
  load; memory for a fully decoded scan is 1.2–1.6x ifc-lite's.
- Relevance to cadrs: `step` lexer/parser: concrete targets (byte-class table, memchr, borrowed
  strings, exact-size parameter vectors, split-at-record-boundary parallel parse).

#### web-ifc (ThatOpen/engine_web-ifc) tape reader.
- Canonical: https://github.com/ThatOpen/engine_web-ifc ; benchmark table
  https://github.com/ThatOpen/engine_web-ifc/blob/main/benchmark.md
- Design (secondary description from the search results; the README itself does not describe
  it): the file is loaded in chunks, entity ids replaced by `uint32_t`, type names by CRC codes,
  arguments stored by type on a tape so that "With the entity ID as offset, the loader jumps to
  any entity"; "much smaller memory footprint compared to an object oriented model, and loading
  is much faster". Benchmark (Apple M1): e.g. 70 MB file, 1,139,622 entities, open 448 ms,
  execute-all 5,530 ms; 55 MB, 945,194 entities, open 339 ms; 47 MB schependomlaan.ifc,
  714,485 entities, open 301 ms. [Sources: README, benchmark.md]
- Limitations: IFC geometry (extrusions/CSG), not B-rep sewing; "open" excludes geometry.
- Relevance to cadrs: `step` parse stage: ~150 MB/s *including* index building on one core is
  the bar for the parse-only part.

#### STEPcode lazy loading (`src/cllazyfile`, "lazyInstMgr").
- Canonical: https://github.com/stepcode/stepcode (the raw header URL guessed in this session
  returned 404; the Fedora/mankier package pages describe STEPcode only as a C++/Python
  generator https://www.mankier.com/package/stepcode).
- Content: STEPcode's lazy file module scans sections and records instance offsets, loading an
  instance only when referenced — known from the repository, but no documentation page could be
  fetched to quote; treat as unverified.
- Relevance to cadrs: `step` — lazy entity resolution is what makes reading one product from a
  large assembly file cheap.

#### ruststep (ricosjp) and other Rust crates.
- Canonical: https://github.com/ricosjp/ruststep ; https://docs.rs/ruststep/latest/ruststep/
- Design: `espr` compiles EXPRESS to Rust structs; the `parser` module tokenises into an `ast`
  (`nom` 7.1.3 is a dependency); the `tables` module handles the "exchange structure graph as
  tables"; a derived `Holder` struct per ENTITY holds unresolved references, resolved from the
  table on demand. README: "still in experimental stage", serialisation back to STEP and EXPRESS
  rules unimplemented (issues #13, #43). No performance claims. Other crates: `step-p21`
  (https://docs.rs/step-p21, tokenizer + struct mapping, no figures), `bimifc-parser`/`ifc-lite`
  (nom + memchr, "Zero-copy tokenization and SIMD-accelerated entity discovery", ifc-lite claims
  ~1,259 MB/s tokenisation, self-reported, https://cdn.jsdelivr.net/npm/@ifc-lite/parser@3.4.0/README.md),
  `ifcfast` (claims 20–27x faster than IfcOpenShell, self-reported, https://pypi.org/project/ifcfast/).
- Relevance to cadrs: `step` — the Holder/Table split (parse everything into typed holders, resolve
  `#id` references lazily through a table) is the idiomatic Rust design for lazy resolution.

#### OCCT STEP reader (user guide and `StepToTopoDS_TranslateEdgeLoop.cxx`, V7_8_0) and the multithreading status.
- Canonical: https://occt3d.com/dev/doc/overview/html/occt_user_guides__step.html ;
  https://occt3d.com/dev/content/open-cascade-development-insights-part-2/index.html
- Stages: Load ("Loading the file only memorizes the data, it does not translate it"), optional
  Check, set parameters, Translate roots/entities, fetch. `read.precision.mode` 0 = file
  (`uncertainty` of each `shape_representation` "is used as a fundamental value of precision
  during translation"), 1 = user `read.precision.val` (default 1e-4, mm regardless of the file's
  unit); `read.maxprecision.val` (default 1) with mode Preferred (may be exceeded "to ensure the
  shape validity") or Forced; `read.surfacecurve.mode` 0 = take both 2D and 3D, 3 = "3D curves are
  used to rebuild 2D ones"; `read.stdsameparameter.mode` off by default (when on, calls
  `BRepLib::SameParameter` via `ShapeFix_Edge`). "The best of the original and the recomputed
  pcurve is put into" the edge; a p-curve is recomputed when SameParameter gives a tolerance above
  a threshold. Shape Processing after translation "consists of a single operator", `ShapeFix`
  (with the resource parameters listed in section 1). The guide warns that STEP translation "does
  not improve the quality of the geometry in the original STEP file".
- Edge-loop translation (source): vertices within `Precision::Confusion()` are bound to one;
  adjacent edges that do not share a vertex but whose vertices lie within `preci` are merged
  (else "not connected" warning); file p-curves on *planar* surfaces are always dropped and
  recomputed; missing p-curves are projected with `ShapeFix_EdgeProjAux::Compute(preci)` and
  removed if projection fails; seams need two p-curves chosen by
  `ShapeAnalysis_Curve().SelectForwardSeam`; `CheckPCurves` drops p-curves with w1 == w2 and
  clamps ranges; each p-curve is validated by `XSAlgo::AlgoContainer()->CheckPCurve(..., preci,
  IsSeam)`.
- Multithreading: the OCCT dev article describes an opt-in parallel mode for "Parsing,
  Transferring, and Healing" on OCC Threads or TBB, "presented in beta mode", single-thread
  default, "OCCT3D version 7.8 introduced improvements in STEP thread-safety processing. Some
  scenarios may still have data race problems", listed for 7.9.0 — but in an August 2025 reply the
  author says it "is not available in any official versions", was tested only on specific
  projects, is unstable and postponed. The STEP guide says translation is thread-safe only when
  each call passes its own `DESTEP_Parameters` rather than global `Interface_Static`. A 2018
  commit removed quadratic behaviour in STEPCAF label lookup ("using map mechanism instead of brute
  force iteration") and added STEP read/write performance tests
  (https://git.rbts.co/OpenCascade/occt/commits/commit/1d54b80764aaf3eca366cfe888cb5da952c1adb1/tests/perf).
- Limitations: OCCT's lexer is flex/bison generated and single-threaded; all entities are
  materialised at load.
- Relevance to cadrs: `step` and `xbuild`: cadrs is 1.9x slower than this reader *including* its
  ShapeFix pass; the places OCCT saves work are (a) dropping and recomputing p-curves only on
  planes, (b) SameParameter off by default, (c) a single FixShape pass with "-1 = only if needed"
  modes.

### Inferences
- Parsing is rarely the bottleneck in a B-rep STEP import; with a 0.4–1.5 GB/s tokenizer a
  100 MB file lexes in well under a second, so cadrs's 1.9x is most likely in model building
  (p-curve fitting, sewing, checks). Profile `xbuild` separately from the lexer before touching
  the parser.
- The published OCCT-beating result (1.5x) came from a per-level thread pool during model
  building, i.e. translating faces in parallel; cadrs already has rayon for tessellation and could
  build faces (p-curves, loops) in parallel before the single-threaded sewing/assembly step.
- Lazy resolution (tape + offsets, or Holder/Table) matters mostly for assemblies where only some
  products are needed and for memory.

### Gaps
- No peer-reviewed benchmark of Part 21 parsers exists in what was found; all throughput numbers
  are self-reported by crate maintainers.
- The Chinese 2023 paper's full text (file splitting and cross-thread reference handling) was
  not read.
- STEPcode's lazy loader could not be documented from a fetched source.

---

## 4. Persistent naming of topology across parametric rebuilds

### Takeaway
Three families: (a) **history/feature names** (Kripac 1997; Capoyleas–Chen–Hoffmann 1996; Wu et
al. 2001; Mun & Han 2005; FreeCAD/RealThunder; OCCT TNaming) — name a face by the feature and
feature-face that created it, name edges/vertices by their adjacent faces, and add bookkeeping for
split (index in some order) and merge (list of merged names); (b) **topological similarity
matching** (Agbodan/Marcheix/Pierra 2003, Marcheix 2006, Cheon et al. 2012) — name invariant
faces structurally, name contingent faces by step+iteration number *plus their neighbourhood*,
and at re-evaluation match old to new faces by maximising a topological cover/similarity measure
over the face graph; (c) **persistent-entity references** (Bidarra–Nyirenda–Bronsvoort 2005;
Onshape) — refer only to persistent feature faces or operation ids and disambiguate multiple
intersections with automatically generated half-space references (Bidarra) or "external
disambiguation" entities (Onshape). Split cases are handled best by (b) and (c): index-ordered
schemes (RealThunder, cadrs's `split = parent·16+k+1` by centroid order) break when the number
or order of pieces changes, which both Marcheix and the FreeCAD wiki document explicitly.

### Cited Findings

#### Kripac, J. "A mechanism for persistently naming topological entities in history-based parametric solid models." Computer-Aided Design 29(2):113–122, 1997.
- Canonical: https://doi.org/10.1016/s0010-4485(96)00040-1 (Crossref-verified; paywalled).
- Open PDF: none found.
- Method (as summarised by Mun & Han 2005 and Marcheix & Pierra 2002): a Topological ID Graph
  records, per feature step, the faces with ids and the split/merge relations between old and new
  faces; edges and vertices are identified through their adjacent faces; at re-evaluation the
  graph is walked to find the entity with the same id-history, with special cases for a face
  split into several or several merged into one. [Source: Mun & Han 2005 local PDF §2; Marcheix
  2006 local PDF §2]
- Limitations (per Mun & Han): does not address the case "where there are several closed" loops
  / merged faces fully; relies on a global face-id graph that grows with history.
- Relevance to cadrs: `naming` — the id-graph is what cadrs's `History` + `FaceName` already is;
  Kripac's explicit split/merge records (which pieces came from which face, in which step) are the
  bookkeeping cadrs stores only as `split` indices.

#### Capoyleas, V.; Chen, X.; Hoffmann, C.M. "Generic naming in generative, constraint-based design." Computer-Aided Design 28(1):17–26, 1996; and Chen, X.; Hoffmann, C.M. "On editability of feature-based design." CAD 27(12):905–914, 1995.
- Canonical: https://doi.org/10.1016/0010-4485(95)00014-3 ; https://doi.org/10.1016/0010-4485(95)00013-5
  (Crossref-verified; paywalled). Purdue CS technical-report versions exist in Purdue e-Pubs
  (`docs.lib.purdue.edu/cstech`) but the repository search returned 403 and Hoffmann's
  `cs.purdue.edu/homes/cmh/distribution` page returned 404 in this session.
- Open PDF: not reached (try Purdue e-Pubs by hand: CSD-TR-94-011 / 95-xxx).
- Method (as summarised by the surveys): a face is named by the feature that generated it and its
  generic role in that feature ("the top face of the slot"); where a feature face is split, the
  pieces are named by the adjacent faces that bound them (adjacency-based disambiguation), and a
  case analysis covers split, merge and the "n-to-m" change by orienting and ordering; edges and
  vertices are named as intersections of named faces. This is the scheme Onshape's and cadrs's
  "feature + origin" names descend from. [Sources: Marcheix 2006 §2; Farjana & Han 2018 abstract]
- Relevance to cadrs: `naming` — `Side{region, curve}` / `Cap{region, end}` are generic names;
  Hoffmann's adjacency-based split disambiguation is the alternative to centroid ordering.

#### Marcheix, D.; Pierra, G. "A survey of the persistent naming problem." Proc. 7th ACM Symposium on Solid Modeling and Applications (SMA '02), 2002, pp. 13–22.
- Canonical: https://doi.org/10.1145/566282.566288 (Crossref-verified; paywalled).
- Open PDF: none found (LIAS publications host the 2003 and 2006 papers but not this one).
- Content: classifies naming schemes by what they name (atomic entities vs aggregates), by the
  information used (topology, geometry, history) and by how they match after re-evaluation;
  introduces the invariant/contingent vocabulary used in the LIAS papers below.
- Relevance to cadrs: `naming` — the taxonomy for the design note.

#### Agbodan, D.; Marcheix, D.; Pierra, G.; Thabaud, C. "A topological entity matching technique for geometric parametric models." Proc. Shape Modeling International 2003, pp. 235–244.
- Canonical: IEEE SMI 2003 (DOI not resolved). Open PDF:
  https://www.lias-lab.fr/publications/7076/2003-ICSMA-AGBODAN.pdf — local
  `Agbodan-2003-topological-entity-matching.pdf`.
- Algorithm: entities are split into *invariant* ("completely and unambiguously characterized by
  the structure of a constructive gesture and its input parameters": the end face of a sweep, the
  lateral shell of a slot) and *contingent* ("results from an interaction between the pre-existing
  geometric model and invariant entities"). A face graph records every face that exists or has
  existed, with historical links; invariant faces of primitive/transition/extrusion/revolution
  features are named "by a unique topological traversal of the object", contingent faces by
  "a step number and an iterative number" *plus* their topological neighbourhood. At
  re-evaluation, matching between the initial graph AG and the new graph NG is done per
  construction step in two phases: a local phase computing two similarity measures δ0, δ1 for every
  (old, new) face pair from the *cover* of their boundary circuits γF (sets of oriented edges, with
  "partial sub-paths" allowing deleted edges; Table 2 gives (1,1) for equal circuits, (1, ]0,1[)
  or (]0,1[, 1) for inclusion, (]0,1[, ]0,1[) for partial overlap, (0,0) for disjoint), and a
  global phase choosing "a binary relation that maximizes the topological similarity between the
  matched entities". The chosen semantics is that a face J split off a face X is matched as "part
  of" X when they share an invariant ancestor, so traceability is not lost. Edges and vertices are
  then named from the matched faces. Implemented on Open CASCADE. [Source: local PDF §3–4]
- Limitations: "when the model contains non-linear entities, topological neighborhood, even
  extensive, are ambiguous" and a geometric mechanism (feature orientation) is needed; the matching
  is per step over all faces (cost grows with history).
- Relevance to cadrs: `naming::resolve` — the cover measure is a principled replacement for "a
  unique entity with the same base name, else the one the hint puts nearest": split pieces of a
  face match by how much of the old boundary circuit they inherit.

#### Marcheix, D. "A persistent naming of shells." International Journal of CAD/CAM 6(1):125–137, 2006.
- Canonical: http://www.ijcc.org (KoreaScience https://koreascience.kr/article/JAKO200634514931770.pdf).
  Open PDF: https://www.lias-lab.fr/publications/7550/2006-CAD-CAM-Marcheix.pdf — local
  `Marcheix-2006-persistent-naming-of-shells.pdf`.
- Algorithm: extends the 2003 method to aggregates: each form-feature class has a hierarchical
  structure of shells (invariant structure) and the faces carry the historical structure; a shell
  reference ("the lateral shell of the slot") is re-evaluated by matching its faces with the same
  two similarity measures and taking the set; integrated in a TopSolid prototype. [Source: local
  PDF abstract and §1]
- Relevance to cadrs: `naming` for multi-face selections (e.g. "all side faces of Extrude 1"),
  which cadrs's per-face names do not express as one reference.

#### Cardot, A.; Marcheix, D.; Skapin, X. "Persistent naming based on graph transformation rules to reevaluate parametric specification." Proc. CAD'18, pp. 387–391, 2018; and Marcheix, Cardot, Skapin, Dieudonné-Glad, "A persistent naming system based on graph transformation rules", WSCG 2018 posters (CSRN 2803).
- Canonical: https://doi.org/10.14733/cadconfp.2018.387-391 (Crossref-verified). Open PDFs:
  https://www.cad-conference.net/files/CAD18/CAD18_387-391.pdf — local
  `Cardot-2018-naming-graph-transformation-CAD18.pdf`;
  https://dspace.zcu.cz/bitstream/11025/34637/1/Marcheix.pdf — local
  `Marcheix-2018-naming-graph-transformation-rules.pdf`.
- Algorithm: the model is a generalised map (G-map: darts linked by α0..αd involutions; an entity
  is an orbit ⟨i..j⟩ of a dart); every operation is a Jerboa graph-transformation rule, so the
  history of *every* dart is known exactly from the rule application; naming is therefore
  "independent of the model dimension, the dimension of entities ... and withstanding the edition
  of the parametric specification (i.e. adding, deleting or moving operations)"; persistent names
  are orbits traced through rule applications, and matching after re-evaluation follows the
  dart history rather than any geometric or index heuristic. [Source: local PDFs]
- Limitations: requires the whole kernel to be a G-map with rule-based operations — not
  applicable to a B-rep kernel with free-form algorithms except as an ideal: the history should
  be recorded at dart/half-edge granularity, not only face level.
- Relevance to cadrs: `naming` — argues for recording `History` for edges and vertices too (cadrs
  derives edge/vertex names only from faces).

#### Wu, J.; Zhang, T.; Zhang, X.; Zhou, J. "A face based mechanism for naming, recording and retrieving topological entities." Computer-Aided Design 33(10):687–698, 2001.
- Canonical: https://doi.org/10.1016/s0010-4485(00)00099-3 (Crossref-verified; paywalled).
- Open PDF: none found.
- Method (per Mun & Han 2005): faces get a basic name from their feature; split faces with the
  same basic name are distinguished by *PSI* (parametric-space information: ordering in the
  parameter space of the feature's sketch); edges and vertices are named by adjacent faces. "Wu
  did not address the naming case where there are several closed" loops, nor merging, and "did not
  propose a name matching method". [Source: Mun & Han 2005 local PDF §2]
- Relevance to cadrs: `naming` — PSI is the same idea as cadrs's centroid ordering, in sketch
  parameter space rather than 3D.

#### Mun, D.; Han, S. "Identification of topological entities and naming mapping for parametric CAD model exchanges." International Journal of CAD/CAM 5(1):69–81, 2005.
- Open PDF: https://koreascience.kr/article/JAKO200503018236985.pdf — local
  `Mun-Han-2005-naming-mapping-parametric-exchange.pdf`.
- Algorithm: entity name EN(f) = BN(f) : OSI : SN — basic name (feature + feature face), *object
  space information* ("the sequential number in the object space, which is calculated by
  comparing x, y, and z values of given topological entities having the same basic name" — i.e.
  split pieces ordered by 3D position, "[0,0] if no ambiguity") and *secondary name* (the list of
  faces merged into this face: after BN(f1_blank) merges with BN(f2_tool) the name becomes
  "BN(f1_blank):0,0:1;BN(f2_tool)" and Total_Num increments). Edges: EN(e) = EN(f1) # EN(f2) #
  OSI; vertices: EN(v) = EN(f1) {# EN(f2)} # OSI. Then a *naming mapping* translates these names
  between CAD systems in a macro-parametric exchange. [Source: local PDF §3]
- Limitations: OSI ordering by coordinates breaks when a parameter change reorders the pieces in
  space; merge lists grow.
- Relevance to cadrs: `naming` — this is the closest published scheme to cadrs's (`split` index
  ordered by centroid = OSI; "where two input faces merge, the smaller name wins" ≈ SN but cadrs
  drops the other name instead of keeping the list). Keeping the merged names (SN) would let a
  reference to either pre-merge face still resolve.

#### Cheon, S.-U.; Mun, D.; Han, S.; Kim, B.C. "Name matching method using topology merging and splitting history for exchange of feature-based CAD models." Journal of Mechanical Science and Technology 26(10):3201–3212, 2012.
- Canonical: https://doi.org/10.1007/s12206-012-0827-3 (Crossref-verified; paywalled).
- Open PDF: none found.
- Content: extends Mun & Han with an explicit merge/split history used to match names across
  systems. Not readable in this session.
- Relevance to cadrs: `naming` split/merge bookkeeping.

#### Bidarra, R.; Nyirenda, P.J.; Bronsvoort, W.F. "A feature-based solution to the persistent naming problem." Computer-Aided Design & Applications 2(1–4):517–526, 2005; and Bidarra & Bronsvoort, "Persistent naming through persistent entities", Geometric Modeling and Processing 2002.
- Canonical: Computer-Aided Design & Applications 2(1–4), 2005 (the CAD&A DOI could not be
  resolved via Crossref in this session); TU Delft record
  https://publications-cgv01.ewi.tudelft.nl/papers/626. Open PDF:
  https://www.cad-journal.net/files/vol_2/CAD_2(1-4)_2005_517-526.pdf — local
  `Bidarra-2005-feature-based-persistent-naming.pdf`.
- Algorithm: parametric definitions never refer to B-rep faces, only to *feature faces* of
  declarative feature classes ("unlike b-rep entities, they are never split, merged or deleted,
  even though their geometric representation may be"). Two B-rep invariants make this work: "(i)
  every b-rep face carries an attribute with the name of the feature face from which it originates
  (called its owner), and propagates this name every time it gets split; and (ii) two adjacent
  b-rep faces that are coplanar are never merged, unless they have the same owner." An edge is
  named by its two adjacent feature faces plus discriminators; when two feature faces intersect in
  more than one edge, a *partitioning algorithm* automatically creates reference half-spaces
  positioned relative to the two features (e.g. "passing through the axis of the cylindrical face
  (of the slot) and orthogonal to the planar face (of the block)"), giving names like
  `e1 → (<block.front, slot.side>, <block.top, block.bottom, reference.positive>)`; the references
  are constrained to the features so they move with them. "At each boundary evaluation step, a
  generic partitioning algorithm ... identifies each group of intersection edges defined by the
  same two feature faces ... Based on (i) the type of surface of each feature face involved, (ii)
  the type of the canonical shape in the respective features, and ... (iii) the references already
  available ... one or more references are generated" until every edge is in a distinct half-space
  combination. Validated for planar, cylindrical, spherical and toroidal faces. [Source: local PDF
  §3–4]
- Limitations: needs declarative feature classes (every face must have an owning feature face),
  forbids merging coplanar faces of different owners (so `unify` must respect owners), and the
  half-space generator is per surface-type pair (quadrics only).
- Relevance to cadrs: `naming` and `unify` — cadrs's "where two input faces merge, the smaller
  name wins" is the opposite of rule (ii); and the half-space discriminator is a geometric,
  feature-relative way to index multiple intersection edges that is stable under parameter changes,
  unlike "smallest midpoint, rounded to 1 µm".

#### Farjana, S.H.; Han, S. "Mechanisms of persistent identification of topological entities in CAD systems: a review." Alexandria Engineering Journal 57(4):2837–2849, 2018.
- Canonical: https://doi.org/10.1016/j.aej.2018.01.007 (Crossref-verified; open access at Elsevier
  — the `pdfft` download returned HTML in this session, not stored).
- Content: classifies mechanisms as topology-based, geometry-based or combined and compares them
  (per DOAJ record https://doaj.org/article/bf42fdae16584d0ba2d824137aba6599).
- Relevance to cadrs: the most recent survey; fetch by hand.

#### Safdar, M.; Jauhar, T.A.; Kim, Y.; Lee, H.; Noh, C.; Kim, H.; Lee, I.; Kim, I.; Kwon, S.; Han, S. "Feature-based translation of CAD models with macro-parametric approach: issues of feature mapping, persistent naming, and constraint translation." Journal of Computational Design and Engineering 7(5):603–614, 2020.
- Canonical: https://doi.org/10.1093/jcde/qwaa043 (Crossref-verified; open access at OUP — the
  direct PDF URL guessed in this session returned HTML; not stored).
- Content: persistent naming in the context of translating feature models between CAD systems
  (the KAIST line continued); not read.

#### "Hierarchical naming method and mechanism design for topological entities." Journal of Computer-Aided Design & Computer Graphics, 2023 (Chinese).
- Canonical: https://doi.org/10.3724/SP.J.1089.2023-00023 . Abstract: names are built
  "progressively from original names to reference names"; a prototype on OpenCASCADE "effectively
  addresses the ambiguity problem"; authors and details not shown on the English page.

#### OCCT OCAF `TNaming` (user guide; `TNaming_Naming.cxx`, `TNaming_Identifier.cxx`, V7_8_0).
- Canonical: https://occt3d.com/dev/doc/overview/html/occt_user_guides__ocaf.html ; sources on
  GitHub.
- Algorithm: `TNaming_NamedShape` stores pairs (old shape, new shape) with one evolution per
  attribute — PRIMITIVE ("newly created topology, with no previous history"), GENERATED ("the new
  shape is created from a low-level old shape"), MODIFY, DELETE, SELECTED. `TNaming_Selector`
  "creates a name of the selected shape, which is a unique description (data structure) of how to
  find the selected topology", stored as a SELECTED named shape, and `Solve` re-evaluates it after
  a rebuild ("If the naming structure ... is correct, the selector automatically updates the
  selected sub-shape ... else it fails"). `TNaming_Identifier` picks the name type from the
  shape's named-shape evolution: PRIMITIVE → MODIFUNTIL (feature), or CONSTSHAPE if "imported"
  (a label holding more than one shape); GENERATED → GENERATION with the generators found by
  `TNaming_Localizer::FindGenerator`; otherwise `Localizer.Backward` yields primitive ancestors
  and shapes → INTERSECTION (e.g. an edge as the intersection of the features that produced its
  faces), collapsing to MODIFUNTIL when there is one primitive. `TNaming_Naming::Name` first
  checks `RepeatabilityInContext` (the same sub-shape appearing several times, e.g. seam edges) and
  records an ORIENTATION name with a unique context; wires become WIREIN (index 1 outer, −1 inner),
  shells SHELLIN, compounds UNION. After building, the name is regenerated and compared; if it
  matches more than the selected shape, `Filter` wraps it in FILTERBYNEIGHBOURGS with the
  neighbours (each named with Geom=1) and accepts only if "only S appears among the new shapes",
  recursion capped at 3 levels; a `Geom` flag accepts a geometric-only match without uniqueness.
  [Sources: OCAF guide; raw source files]
- Limitations: known in FreeCAD/OCCT practice to be fragile for booleans; names are structural
  descriptions tied to OCAF labels, so a kernel without OCAF must re-create the label tree.
- Relevance to cadrs: `naming` — the INTERSECTION-of-features name for edges/vertices and the
  neighbour filter are the OCCT counterparts of cadrs's `EdgeName{faces}` and `resolve`'s hint;
  ORIENTATION/`Index` is how OCCT handles seams, which cadrs leaves unnamed.

#### FreeCAD / RealThunder "Topological Naming" (element map), merged into FreeCAD 1.0 (2024).
- Canonical: https://github.com/realthunder/FreeCAD_assembly3/wiki/Topological-Naming and
  .../Topological-Naming-Algorithm ; Ondsel project notes
  https://ondsel.com/blog/milestone-toponaming-fix-phase-2-done ,
  http://www.ondsel.com/blog/freecad-topological-naming/ ; FreeCAD dev blog 24 July 2024
  https://blog.freecad.org/2024/07/24/wip-wednesday-24-july-2024/ (wiki page
  https://wiki.freecad.org/Topological_naming_problem was behind an Anubis bot-check).
- Algorithm (`TopoShape::makeShape`): (1) sub-shapes unchanged between input and result keep
  their names; (2) for each input vertex/edge/face, OCCT's `Modified`/`Generated` (via
  `BRepBuilderAPI_MakeShape` history, with special handling for `BRepBuilderAPI_Sewing` and
  `ThruSections`) gives result elements, each named `<source>;:<M|G|MG><index>;<opcode>;:T<tag>:<srcLen>:<type>`
  (e.g. `Face6;:M2;FUS;:T1:5:F`), modified results with positive indices "in the order OCCT
  reports them", generated with negative; several sources → first is the prefix, rest in
  parentheses; (3) reverse pass: unnamed edges of a named face get `<face>;:U<n>;<opcode>`,
  unnamed vertices of a named edge likewise, sorted so the first can name the upper element;
  (4) forward pass: an unnamed edge/face is named only if *all* its lower elements are named, as
  `<first lower>;:L(<other lowers>);<opcode>`. Names are stored in a `StringHasher` (SHA1 digest
  when above `Threshold`, default 0 = keep text), with `;` as the separator and `;:` reserved;
  `getElementHistory` decodes the source name recursively; `getRelatedElements` tries the
  recovered source name then `name;:M`/`source;:M` prefixes. One-to-many is "disambiguated only by
  the order index"; the wiki's own example: a cylinder moved into a cube edge split the edge and
  the fillet reference "bound to the lower of two split edges, because OCCT reported it first ...
  the problem remains for more segments". Overhead on a 148-object model: recompute 40 s → 52 s,
  file 17.8 → 22.6 MiB. [Source: wiki algorithm page]
- Merge status: Ondsel's 2023 plan split RealThunder's LinksStage3 work into pieces merged with
  unit tests; the FreeCAD dev blog (July 2024) records contributors "transferring RealThunder's
  code to solve the save/restore errors with objects with element maps"; community reports say it
  is in FreeCAD 1.0 with "not everything solved" (https://hn.nuxt.dev/item/47097446 — community
  claim). Ondsel: the problem "cannot be _solved_ in the traditional sense".
- Limitations: no geometric matching at all; a fillet "breaks when its referenced edge is modified
  or removed, because names are not reused"; names depend on OCCT's reporting order and on the
  algorithm version string.
- Relevance to cadrs: `naming` — same architecture as cadrs (history → names; edges from faces),
  with the same weak spot (index order for splits). cadrs is already stronger (centroid order plus
  geometric fallback plus `Lost`).

#### Onshape robust references (forum explanation by Onshape staff "lana", August 2025).
- Canonical: https://forum.onshape.com/discussion/comment/130171 (also
  https://forum.onshape.com/discussion/comment/120351 on stable FeatureScript ids).
- Mechanism (quoted): "For each topology created by an operation there is a record of its
  dependencies allowing us to identify this topology. When downstream references this topology,
  query (an encoding) of this record is generated. Query evaluation is a pattern matching of the
  query with current state of history. One of the first matches is operation id." "If we introduce
  an unstable component into ids of some operations, it means that results of those operations
  will all be considered when evaluating a query generated for one of them. This way reference is
  stable to order and number of input selections. If the dependencies are not enough to
  disambiguate results of these operations, external disambiguation is used. At the time of query
  evaluation external reference replaces the unstable component." A user's confirmed summary: a
  robust query evaluates its input query to a transient id, finds the last operation that touched
  it, and builds "which entity had that transient id at that operation"; marking an id component
  unstable adds `*`, and `setExternalDisambiguation` links the id to the entities the operation
  created. Caveat from staff: chains of `opSplit` "don't yet behave as the model predicts". A
  developer reply explains that counter-generated ids shift when code paths change and that
  hand-built descriptive ids give "much more robust parametric references".
- Limitations: proprietary; staff say the model is simplified.
- Relevance to cadrs: `naming` — "dependency record → query → pattern match against history"
  with explicit wildcard components and external disambiguation is the production answer to the
  split/merge question: a split result is not indexed, it is matched by its dependencies (which
  input edges/faces it came from) with a fallback to a user-visible disambiguating entity.

### Inferences
- For splits, the three defensible strategies in the literature are: dependency/adjacency
  matching (Hoffmann; Agbodan's cover; Onshape), feature-relative geometric discriminators
  (Bidarra's half-spaces; cadrs's seed point for sketch-on-face), and ordering (Wu's PSI, Mun's
  OSI, RealThunder's OCCT order, cadrs's centroid order). Ordering is the weakest and is what
  cadrs uses for `split` and `EdgeName.index`; replacing it with "which input edges/vertices bound
  this piece" (available from `History` if generated/modified is recorded for edges) would
  cover the Control-Arm style cases without geometry.
- For merges, keeping the list of merged names (Mun's SN; Kripac's merge record) instead of
  "the smaller name wins" lets a reference to either input resolve after the merge and un-merge
  cleanly when a parameter change separates the faces again.
- Recording history at edge and vertex level (G-map argument; OCCT GENERATED for edges) removes
  the need to name vertices by "three smallest face names", which changes when any adjacent face
  is renamed.

### Gaps
- Kripac 1997, Capoyleas et al. 1996, Chen & Hoffmann 1995, Marcheix & Pierra 2002, Wu et al.
  2001, Cheon et al. 2012: no open PDFs located (Purdue e-Pubs and Hoffmann's page were
  unreachable in this session; they are the most likely open sources for the 1995/1996 papers).
- Farjana & Han 2018 and Safdar et al. 2020 are open access but the direct download URLs tried
  returned HTML; fetch from the publisher pages by hand.
- No Fusion 360 technical description of its naming was found before the search budget ran out.
- The 2023 Chinese hierarchical-naming paper's method is known only from its abstract.

---

## What I could not find
- "Yang/Han/Lee, A new sewing approach" — no record under that title; the closest KAIST papers
  are Yang & Han 2006 (history-based repair) and Yang, Han, Park 2005 (error verification).
- "Gu/Chase/Sheng, Healing of geometric models" — no record; see Gu et al. 2001 and Sheng & Meier
  1995 above.
- Butlin & Stops 1996 (IMR 5) full text; the Sandia IMR archive refused the fetch.
- Any open PDF of Kripac 1997, Capoyleas–Chen–Hoffmann 1996, Chen–Hoffmann 1995, Marcheix–Pierra
  2002, Wu et al. 2001, Benkő–Martin–Várady 2001 (Cardiff ORCA refuses automated download),
  Benkő–Várady 2004, Vančo–Brunnett 2004, Cheon et al. 2012, Shen–Sakkalis–Patrikalakis 2001
  (MIT thesis download throttled with HTTP 429).
- Parasolid tolerant-modelling / body-checker and ACIS healing documentation (login-only).
- A peer-reviewed benchmark of STEP Part 21 parsers; all throughput figures are self-reported.
- Fusion 360's naming mechanism and any 2015–2026 academic paper on persistent naming other than
  Cardot/Marcheix/Skapin 2018, Farjana & Han 2018, Safdar et al. 2020 and the 2023 JCAD paper.

## Local files (all verified `%PDF`)
`../papers/exchange_healing_naming/`:
Agbodan-2003-topological-entity-matching.pdf, Barequet-1995-filling-gaps-polyhedron.pdf,
Bidarra-2005-feature-based-persistent-naming.pdf, Cardot-2018-naming-graph-transformation-CAD18.pdf,
Gonzalez-Lluch-2017-cad-quality-survey.pdf, Jackson-1995-local-tolerances-brep.pdf,
Lukacs-1998-faithful-fitting.pdf, Marcheix-2006-persistent-naming-of-shells.pdf,
Marcheix-2018-naming-graph-transformation-rules.pdf, Mun-Han-2005-naming-mapping-parametric-exchange.pdf.
