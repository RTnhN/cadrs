# Boolean, classification and tolerant-modelling algorithms for cadrs_kernel

Algorithm cards drawn from `research/README.md`, `notes/booleans.md`, `notes/kernels_recent.md`
and the paper texts. Each card says where it plugs into `crates/cadrs_kernel/src/native/`
(`boolean.rs`, `split.rs`, `classify.rs`, `unify.rs`, `tol.rs`) and whether its steps were
checked against the paper text ("verified") or come only from the notes.

Starting point (from the kernel README and the code's doc comments). The boolean is already
0.48x OCCT's time on OCCT's cases. Several pieces the literature recommends are already there:

- A BVH over face boxes for face-pair candidates.
- Coincident faces detected with `same_domain` and imprinted, not intersected.
- Face pairs tangent along a shared edge skipped (`tangent_along_shared_edge`).
- SSI curves that run along an existing edge dropped.
- Classes with on-same and on-opposite states.
- Sub-faces grouped by union-find over uncut edge pieces, so one test per group.
- Near-coincident faces given the gap as their tolerance before the run (`near_coincident_tolerances`).

What is missing:

- The vertex merge is greedy and depends on order (`Pool::vertex_tol`). The new vertex keeps the
  first one's position.
- Tolerances combine by max, not sum.
- New vertices from edge–edge and edge–face hits are never re-tested against other edges.
- Groups are not linked across cut edges, so every group shoots its own rays: at least two rays
  that agree, confirmed by a second member (`Classifier::parity_inside`, up to 37 directions).
- Any failure reruns the whole boolean at 1e-5 and then at 1e-4 (`boolean_bodies_with`).

## Ranked cards

| Rank | Card | Payoff | Size | Risk |
|---|---|---|---|---|
| 1 | [Local re-intersection instead of the global retry](#1-local-re-intersection-instead-of-the-global-retry) | Fewer whole-boolean retries; tolerance growth stays near the gap | Large | Medium |
| 2 | [Connexity-chain vertex merge with sum-of-tolerance tests](#2-connexity-chain-vertex-merge-with-sum-of-tolerance-tests) | Order-independent merges; settles sum vs max; prerequisite for 1 | Small | Low–medium |
| 3 | [Pave blocks, shrunk ranges and common blocks](#3-pave-blocks-shrunk-ranges-and-common-blocks) | Tiny split edges stop making spurious hits; one record per coincident piece | Medium | Medium |
| 4 | [Skip face–face SSI where lower-dimensional contacts already decide it](#4-skip-faceface-ssi-where-lower-dimensional-contacts-already-decide-it) | Removes the most unstable and most expensive SSI calls | Medium | Medium |
| 5 | [Intersection-graph repair before imprint (Huang 2023)](#5-intersection-graph-repair-before-imprint-huang-2023) | Closes gaps that now cause open shells and retries | Medium | Low |
| 6 | [Classify at the intersection edge and propagate across it](#6-classify-at-the-intersection-edge-and-propagate-across-it) | Most rays go away; a built-in consistency check | Medium | Medium |
| 7 | [One cheap ray per remaining component: first hit, cascade, perturbation](#7-one-cheap-ray-per-remaining-component-first-hit-cascade-perturbation) | Faster, more robust rays for what card 6 leaves | Small | Low |
| 8 | [Tighter face-pair culling: trimmed boxes, OBB, separating plane](#8-tighter-face-pair-culling-trimmed-boxes-obb-separating-plane) | Fewer SSI calls; speed only | Small | Low |
| 9 | [Jackson's tolerant validity check](#9-jacksons-tolerant-validity-check) | Precise accept/reject for tolerant results and inputs | Medium | Medium |
| 10 | [Generalized winding numbers on trimmed NURBS as a fallback](#10-generalized-winding-numbers-on-trimmed-nurbs-as-a-fallback) | Classification through gaps when everything else fails | Large | Low |

Payoff is ranked against the stated priorities: fewer failures and retries at low cost first,
then speed. Cards 2 and 3 rank high because card 1 depends on them. Card 2 is also the cheapest
fix for a known order dependence.

### Three disagreements in the research, settled here

- **Sum or max of tolerances.** The OCCT spec says that shapes interfere when "the distance
  between the underlying geometry of shapes" is "less or equal to the sum of tolerances of the
  shapes" (fetched 2026-10-22). Jackson says that "a tolerance equal to the sum of the
  tolerances of the topologies is used" (Jackson 1995 p. 249, verified). cadrs uses the larger of
  the two (`Pool::find_tol`, `ctol` in `run`). Card 2 switches to the sum and keeps the run's
  merge distance as a floor.
- **OCCT's vertex/vertex merge.** The spec gives the new vertex as "the center and the radius of
  the sphere enclosing the tolerance spheres of the source vertices" (fetched). The
  "midpoint with max + D/2" reading in `notes/kernels_recent.md` gives the same sphere for two
  vertices of equal tolerance. It is wrong in general. Card 2 uses the enclosing sphere.
- **One ray per patch (Cherchi) or two rays per boolean (Krishnan).** Both can hold together.
  Krishnan's two rays work because adjacent components across an intersection curve "must have
  opposite classifications" (Krishnan 1997 p. 129, verified), which is propagation. Cherchi
  needs one ray per patch because mesh patches carry no orientation data for the neighbour.
  cadrs has exact face normals at every cut edge. Card 6 classifies across cut edges locally and
  propagates. Card 7's ray is then needed only for components that touch no cut edge, plus one
  seed per connected arrangement.

---

## 1. Local re-intersection instead of the global retry

**Problem.** When a run leaves an open shell or an invalid face, cadrs reruns the whole boolean
at 1e-5 and then at 1e-4 (`boolean_bodies_with`, `tol::Tolerances::retries`). That costs one to
two extra full runs. It also raises every entity's tolerance, though the gap is usually in one
place.

**Why it is fast or robust.**
- Jackson re-compares only the pieces of a split edge, "since the smaller edges may now be
  coincident when they were not before". This "can be done reasonably efficiently since only
  edges with known common vertices need be considered" (Jackson 1995 p. 251).
- OCCT merges new vertices from edge/edge hits, and the drafted section edges from face/face, in
  a nested `BOPAlgo_PaveFiller` that runs only on those new entities (OCCT spec, fetched).
- Segal keeps a *near* value per feature: the smallest distance at which a test said "no
  intersection". He reruns only when a merge pushes a tolerance past it (Segal 1990 §4.3,
  verified).

Costs:
- Segal's tolerant modeller ran "5–10 times slower, even when backtracking is not required". It
  is O(n²) in faces without spatial subdivision. Its worst case with backtracking is exponential
  in the number of features, but backtracking "is only necessary when features approximately
  coincide and then only if there are other features near the coincident ones" (Segal 1990 §5,
  §6, verified).
- That 5–10x is the cost of tolerance checks on every test. cadrs already pays most of it,
  because its tests already take per-entity tolerances. The new cost here is the bookkeeping of
  near values and the local passes.
- Growth is unbounded in general. Iterated unions collapsed a cube "to a single vertex" after 11
  iterations (Segal 1990 §5, verified). Keep the 1e-4 ceiling.

**Algorithm.** It has two levels. Level A runs inside one run and needs cards 2 and 3. Level B
replaces the retry and needs only the existing per-entity tolerances.

*Level A: nested passes inside `run`.*
1. Record near values. In the V–E stage, the E–E stage (`ssi::curve_curve`) and the E–F stage,
   a test can find a closest distance `d` with `d > tol_test` and `d < K·tol_test`. Then set
   `near[x] = min(near[x], d)` for both entities. `K = Tolerances::retries.last() / point`
   (= 100 with the defaults), so only distances the ladder could ever merge are kept. This is
   Segal's rule restricted to the reachable range. The restriction is a cadrs choice, not
   Segal's.
2. After stage 2b (E–E) and stage 2c (E–F), collect the set N of vertices created or grown in
   that stage.
3. Sub-pass over N:
   1. V–V: merge chains among N and the pool's vertices within reach (card 2).
   2. V–E: test each v ∈ N against the other body's edges whose box, grown by
      `tol(v) + tol(e)`, contains v (reuse the vertex BVH `vtrees`).
   3. E–E: re-compare only the edge pieces that now end at a vertex of N against edges that
      share a vertex with them (Jackson).
4. A merge can raise an entity's tolerance to `t_new > near[x]`. Then an earlier "no" for x may
   now be wrong (Segal). Reset `near[x] = ∞` and re-run the V–E and E–E tests of x and of every
   entity whose box lies within `t_new` of x. Do not rerun the boolean from the start, as Segal
   did. He noted spatial subdivision would limit what is reconsidered "but this has not been
   implemented". cadrs has the BVH.
5. Loop steps 3–4 until N is empty. This ends because each extra round only merges, so the
   number of vertices falls (Segal's termination argument, §4.3). Cap it at 8 rounds and charge
   each round to `Work`.
6. After stage 3 (F–F), treat the drafted section-edge ends as N and run step 3 once more. This
   is OCCT's `PostTreatFF`: draft vertices and section edges are intersected in a sub-PaveFiller,
   and the results are mapped back.

*Level B: a local retry instead of the global one.*
1. When `run` fails with an open shell, find the free edges in the assembled body (edges used by
   one face of a closed shell). For a `"doesn't close"` loop, use the uv gap that `uv_gap`
   reports.
2. Pair the free edges, or their end vertices, by distance. Each pair's gap `g` is the
   Hausdorff distance of the two edges, sampled. The local tolerance is
   `t_loc = min(1.1·g, retries.last())`.
3. In copies of the inputs, raise to `t_loc` the tolerance of every vertex and edge whose box
   lies within `t_loc` of the gap, and of the faces that bound it. This is the same mechanism
   `near_coincident_tolerances` already uses. It is Jackson's "the hole can be closed by
   assigning larger tolerances to the edges and re-sewing" (p. 252).
4. Rerun `run` at the default merge distance with these per-entity tolerances. If it still
   fails, fall back to the global ladder.

Level B still reruns `run` in full, but the result's tolerance grows only at the gap. Level A is
what removes the reruns.

**Data structures.**
- `near: Vec<f64>` per pool vertex and edge (∞ at start).
- A "dirty" vertex set N per stage.
- For Level B, a free-edge map built from the assembled `Body`'s coedge counts.

**Failure modes.**
- Cascading merges. Segal: "if backtracking occurs once it typically occurs several times". The
  round cap and the work budget bound this.
- Growth past the ceiling. Fail with `BooleanFailed`, as now.
- Merge data out of date. Segal: when merges invalidate orientation data on the sorted points,
  "the current face pair is simply rerun". The cadrs version: rerun stage 3 for that face pair
  only.

**Fit in cadrs.**
- `boolean.rs::run`: new sub-passes between stages 2b/2c and 3 and after 3.
- `boolean.rs::boolean_bodies_with`: Level B before the `retries` loop.
- `tol.rs`: the `K` ratio comes from `Tolerances`.
- Removes most uses of the retry ladder. The ladder stays as the last resort.
- Prerequisites: card 2 (chain merge, so a sub-pass merge is order-independent) and card 3
  (pave blocks, so a re-split edge keeps consistent pieces).

**Sources.**
- `papers/booleans/Jackson-1995-local-tolerances.txt` p. 251 §4.3(e)–(f), p. 252 §6.1 (verified).
- `papers/booleans/Segal-1990-tolerances-polyhedral.txt` §4.3 "Backtracking", §4.6, §4.7, §5
  (verified).
- OCCT Boolean Operations spec, Edge/Edge and Face/Face steps, sub-PaveFiller (fetched and
  verified).
- Level B's free-edge pairing is a cadrs design built on Jackson's re-sew remark. No source
  describes it as an algorithm.

---

## 2. Connexity-chain vertex merge with sum-of-tolerance tests

**Problem.** `Pool::vertex_tol` merges a new point into the nearest existing vertex within the
larger of the two tolerances. The old vertex keeps its position, and its tolerance grows to
`d + tol`. The result depends on the order of insertion. A ≈ B and B ≈ C can merge A, B and C or
not, depending on which came first: nearness is not transitive (Hoffmann 2001; Segal 1990 Fig. 9).
Using the max where Parasolid and OCCT use the sum misses contacts between two tolerant entities.

**Why it is fast or robust.**
- It is O(n) with the existing hash grid, plus a union-find pass.
- The result does not depend on the order of insertion: a chain is a connected component.
- The enclosing sphere is the smallest tolerance that contains every source sphere, so growth is
  minimal. Segal: "the sphere of minimum radius that encloses two intersecting spheres" (§4.3).

**Algorithm.**
1. Interference test for every pair (V–V, V–E, V–F, E–E, E–F):
   `dist ≤ tol_a + tol_b`. Floor it at the run's merge distance: `max(tol_a + tol_b, run_tol)`.
   Sources: OCCT spec ("less or equal to the sum of tolerances"); Jackson p. 249. The floor is
   cadrs's, so exact inputs (tol = `CONFUSION`) still merge at `POINT`.
2. V–V: link every pair of vertices that interfere under step 1 with a union-find. Each
   component (connexity chain) becomes one vertex (OCCT `BOPAlgo_Tools::MakeBlocksCnx`).
3. The chain's vertex is the minimum sphere enclosing the source spheres (cᵢ, rᵢ). Its centre is
   the new point and its radius the new tolerance (OCCT: "the center and the radius of the sphere
   enclosing the tolerance spheres"). For two spheres with d = |c₂ − c₁|:
   - if `d + r₂ ≤ r₁`: (c₁, r₁);
   - if `d + r₁ ≤ r₂`: (c₂, r₂);
   - else `R = (d + r₁ + r₂)/2` and `c = c₁ + (R − r₁)·(c₂ − c₁)/d`.

   For more than two, run Welzl's algorithm on spheres or fold pairwise (fold order changes the
   result only within round-off). Chains are almost always 2–3 vertices.
4. V–E: keep the vertex's position. Set
   `Tol(V) = max(Tol(V), D + Tol(E))`, with D the distance from the vertex to the curve (OCCT
   spec, Vertex/Edge step, verbatim). V–F is the same with the face.
5. Vertices of one input body are never merged with each other unless the input already shares
   them. Segal: "equal-dimensional features never approximately coincide; such coincidence is
   represented with a single feature" (§3, verified).

To keep the pool streaming, a new point may still be inserted with `vertex_tol`. Then run steps
2–3 once over the vertices touched in that stage, before any split uses their positions. That is
the start of each Level A sub-pass in card 1.

**Data structures.** The existing `Pool` grid. A union-find over vertex indices (`unify.rs` has a
`Dsu` to reuse). `vtols` as now.

**Failure modes.**
- A long chain can span far more than any one tolerance (A–B–C–D…). OCCT accepts this. Segal
  bounds it with backtracking. cadrs should reject a merged radius above
  `Tolerances::retries.last()` and fall back to keeping the vertices apart.
- Moving the centre can move a vertex off a curve it was on within tolerance. Re-check the
  vertex's edges with step 4 after the merge (the V–E sub-pass of card 1).

**Fit in cadrs.**
- `boolean.rs`: `Pool::find_tol`, `Pool::vertex_tol`, the `ctol` of the face-pair loop, and
  `tol` in the V–E and E–F stages.
- `tol.rs`: change the module doc's "the larger of their own and the entities'" to the sum.
- Removes the order dependence and the asymmetry of "first vertex keeps its position".
- The change of rule can change results in the conformance suite. Gate it there and in
  `kernel_perf.py`.

**Sources.**
- OCCT spec, "Vertex/Vertex", "Vertex/Edge", interference definition (fetched and verified).
- `papers/booleans/Jackson-1995-local-tolerances.txt` p. 249 §3.3 (verified).
- `papers/booleans/Segal-1990-tolerances-polyhedral.txt` §3 restrictions, §4.3 "Perturbation of
  Merged Features" (verified).
- The two-sphere formula is standard geometry, not from a source.

---

## 3. Pave blocks, shrunk ranges and common blocks

**Problem.**
- A split edge is a list of `(t, vertex)` in `WEdge::splits`. Its pieces are rebuilt late
  ("Pieces of the split edges", "Coincident pieces … become one edge").
- Nothing stops a piece shorter than its end tolerances from interfering again.
- Coincident pieces are found by comparing curves after splitting, not recorded when they are
  found.

**Why it is fast or robust.**
- The *shrunk range* is "the part of 3D curve that can interfere with other shapes": the range
  minus the end vertices' tolerance spheres (OCCT spec). A piece whose shrunk range is empty is a
  point. It can't make new hits, so the E–E and E–F stages skip it.
- A *common block* records once that several pave blocks, and the faces they lie on, are one
  edge. Later stages ask the record and never re-derive coincidence numerically. This is
  Hoffmann's rule: "the same logical question may arise repeatedly, in the form of different
  floating-point computations giving contradictory answers" (Hoffmann 2001).

**Algorithm.**
1. *Pave* = (vertex, parameter t on an edge). *Pave block* = the curve between two consecutive
   paves. Each edge starts with one pave block (its ends).
2. When a stage puts vertex v at parameter t on edge e, insert the pave and split the pave block
   that holds t into two. Do this when the hit is found, not at the end.
3. Shrunk range of block [t₀, t₁] with end vertices V₀, V₁: the largest [s₀, s₁] ⊆ [t₀, t₁] with
   `|C(s₀) − P(V₀)| ≥ Tol(V₀) + Tol(C)` and likewise at s₁. The spec's condition for the
   theoretical range is "Distance (P1, P1c) is equal or less than Tol(V1) + Tol(C)". Find s₀ by
   bisection on the distance from the vertex. For lines and circles it is closed form: arc length
   `Tol(V)+Tol(E)` from the end.
4. If s₀ ≥ s₁, the block is *micro*: mark it and skip it in E–E, E–F and in the tests of card 1.
   OCCT reports these as `BOPAlgo_AlertTooSmallEdge`. Its ends should be one vertex chain (card 2).
5. E–E with an EDGE common part (the curves run within tolerance over an interval): chain the
   coincident pave blocks of both edges by union-find into a *common block*. Do the same for E–F
   (an edge lying on a face), adding the face to the common block's face list. This is the
   "EDGE common part" branch in the spec.
6. A VERTEX common part of E–E (a crossing) makes a new vertex. It goes to the V–V/V–E sub-pass
   (card 1).
7. Per face, keep `FaceInfo`: the pave blocks *On* the face (its boundary, with common-block
   partners) and *In* the face (from E–F and F–F), and the vertices On and In. The F–F stage
   reuses these. Paves use "existing vertices, i.e. vertices On or In … for faces". A section
   edge is drafted only where no existing pave block of the two faces already runs (spec,
   Face/Face).
8. When edges are built, each common block makes one edge with the largest tolerance of its
   members. Each micro block makes none (its ends are merged).

**Data structures.**
- `PaveBlock { edge, v: [usize; 2], t: [f64; 2], shrunk: Option<[f64; 2]>, common: Option<usize> }`.
- `CommonBlock { blocks: Vec<usize>, faces: Vec<usize> }`.
- `FaceInfo { on: Vec<usize>, inside: Vec<usize>, verts_on: Vec<usize>, verts_in: Vec<usize> }`.

**Failure modes.**
- A common block can join blocks whose ends are not the same chains. OCCT requires coinciding
  connexity chains. Check that the end vertices are equal after card 2's merge, else split the
  longer block at the other's end first.
- Overlapping but unequal blocks: split both at each other's ends (Segal §4.7: "edges that lie on
  the intersection line must be broken at intersection points that fall between their
  endpoints").

**Fit in cadrs.**
- `boolean.rs`: `WEdge::splits`, `Pool::split`, and the late steps "Pieces of the split edges"
  and "Coincident pieces become one edge".
- `overlap_runs` (contact stretch reduced to its ends) becomes the producer of EDGE common parts.
- Prerequisite for card 1 (stable pieces to re-test) and card 4 (FaceInfo).

**Sources.**
- OCCT spec, terms "pave", "pave block", "shrunk range", "common block", "FaceInfo", Edge/Edge and
  Face/Face steps (fetched and verified for the quoted phrases).
- The alert name is from `notes/booleans.md`.
- `papers/booleans/Segal-1990-tolerances-polyhedral.txt` §4.7 (verified).
- `papers/booleans/Hoffmann-2001-robustness-in-geometric-computations.txt` (quote from the
  notes, not re-checked here).

---

## 4. Skip face–face SSI where lower-dimensional contacts already decide it

**Problem.** Contact between near-tangent faces is where SSI is slowest and least stable. The
contouring finds noise along the contact, then curves that the later stages must drop.

**Why it is fast or robust.**
- Jackson: the surface intersector "need not consider curves (i.e. contact areas) already known
  to be common by virtue of edge-edge or edge-face comparison. This can be important, since
  computation of near-tangent intersection curves can be very unstable" (p. 251).
- OCCT's Glue modes skip it globally:
  - `BOPAlgo_GlueShift` skips "FACE/FACE intersections for partial coincidence".
  - `BOPAlgo_GlueFull` skips "VERTEX/FACE, EDGE/FACE and FACE/FACE".
  - The speed-up "can go up to 90%", "about 70 percent" in both examples.
  - The modes are unchecked: "The algorithm does not check this itself" (fetched).
- On a case with edges 5.28004e-5 apart, a fuzzy value of 6e-5 made OCCT 45x faster: the near
  coincidence became a coincidence, and SSI no longer traced it (fetched; one case).
- ESOLID spent 54–98% of its total time in the intersection stage (notes, Keyser 2002).

cadrs already skips coincident faces, faces tangent along a shared edge, and drops SSI curves
along existing edges. This card makes the skip decision per face pair from the contacts already
found, not just from two special cases.

**Algorithm.** It runs per candidate pair (fa, fb) after stages 2a–2c, with cards 2–3 in place.
1. Collect the known contact K(fa, fb): common blocks lying on both faces (an edge of fa On fb, an
   edge of fb On fa, edge pairs in one common block), and vertices On or In both faces.
2. Rule out the pair if K covers the whole possible contact. Use a cheap side test:
   1. Sample fb's surface over fa's trimmed region (the uv polygon `WFace::polys`, a few points
      per triangle of a coarse grid inside it). Get the signed distance to fb's surface,
      restricted to the parts that project inside fb.
   2. If every sample is either within `tol_a + tol_b` of a K entity, or keeps one strict sign
      with |d| > `tol_a + tol_b`, the faces touch only along K.
   3. Then skip SSI for this pair. The cuts come from K, which the edge stages already imprint.
   This generalises the existing `tangent_along_shared_edge`. The sampling test is a cadrs
   design; Jackson gives the rule, not a test.
3. Otherwise run SSI. Pass the known K points as fixed vertices (OCCT: section edges use existing
   On/In vertices) and the K curves as excluded regions. When a traced curve comes within
   `tol_a + tol_b` of a K curve over a stretch, cut the stretch out, as `overlap_runs` does. Do
   not leave it to imprint.
4. Where SSI and an E–F hit disagree about the end of a curve on a shared boundary edge, keep the
   E–F hit (card 5, rule A). The lower dimension has "lower errors" (Huang 2023).
5. Option (as OCCT's Fuzzy): for a pair whose sampled distance stays within `f` of zero over a
   region, with `f ≤ retries.last()`, raise the two faces' tolerance to `f` and treat them as
   `same_domain`. `near_coincident_tolerances` does this today for whole faces at 1e-7 relative;
   this extends it to pairs found during the run.

**Data structures.** FaceInfo and common blocks (card 3). A per-pair `Contact { blocks, verts }`
built from them.

**Failure modes.**
- A wrong skip loses a real crossing inside the faces. Two faces can touch along a known edge and
  also cross elsewhere. Step 2's sign test catches this when sampling is dense enough. Use the
  `ssi` grid density for the sampling.
- OCCT warns that Glue on real intersections "is likely to lead to incorrect result" (notes). That
  is why this card decides per pair with a check, never globally.
- Card 6's consistency check is a second net: two adjacent components with the same class across
  a cut mean a missed or extra curve (Krishnan 1997 §6.3, verified).

**Fit in cadrs.**
- `boolean.rs::run` stage 3: the pair filter beside `same_domain` and `tangent_along_shared_edge`,
  and the post-filters of SSI curves ("A curve running within the tolerance of another…", "Where
  the faces touch along an edge of one…").
- `ssi::surface_surface_tol` gains optional fixed points and excluded curves.
- Removes some special cases and much noise filtering after SSI.
- Prerequisite: card 3.

**Sources.**
- `papers/booleans/Jackson-1995-local-tolerances.txt` p. 251 §4.3(c), (e), (f) (verified).
- OCCT spec, Gluing and Fuzzy sections and Face/Face step (fetched and verified).
- ESOLID share from `notes/booleans.md` Q6 (not re-checked).
- `papers/booleans/Huang-2023-brep-boolean-repair.txt` §4.3.1 (verified).

---

## 5. Intersection-graph repair before imprint (Huang 2023)

**Problem.** Each F–F pair is intersected alone. Curves that should meet on a shared boundary edge
miss each other by round-off. Faces that only touch leave sub-tolerance curve stubs. Either one
opens a loop, the shell stays open, and the boolean retries.

**Why it is fast or robust.** All three rules are local and run on the few curve ends near face
boundaries. Each rule moves a vertex to a lower-dimensional intersection (edge–face or
edge–edge), which is more accurate. Each is proved by set identities (Rule 3.1: e ∈ f ⇒ e ∩ f = e;
Rule 3.2: v ∈ e ∩ f ⇒ v = e ∩ f in a small neighbourhood). The paper reports no timings. Its
claim is that flawless parts are never modified.

**Algorithm.**

*Intersection graph.* Nodes are curve-end vertices merged within tolerance. Edges are the
trimmed F–F intersection edges, each with predecessor and successor (itself at a free end), and
branching allowed. Each intersection edge carries a tolerance:
`t = k·d(f₁)·d(f₂)·d(e₀)·t₀`, where d is the maximum polynomial degree of the face or curve
geometry, t₀ the default positional tolerance, and `k = max(1, t_f1/t₀, t_f2/t₀, t_e0/t₀)` with
t_x the fitting errors.

For cadrs, t₀ = `tol::POINT`. Take t_e0 from the SSI fit error. Take t_f from the face tolerance.
Clamp t to `Tolerances::retries.last()`; the clamp is cadrs's.

*Rule A: disconnection on a shared edge* (Huang §4.3.1, Alg. 1–2). For intersection edges e₁ and
e₂, all of these hold:
- they have nearest ends v₁ ∈ e₁, v₂ ∈ e₂ with |v₁ − v₂| ≤ t(e₁) + t(e₂);
- they share a source face f₀, with e₁ = f₀ ∩ f₁ and e₂ = f₀ ∩ f₂;
- f₁ and f₂ share an edge e₁₂, and both v₁ and v₂ lie on e₁₂.

Then compute `edgeFaceIntersection(e₁₂, f₀)`. Take the hit v minimising |v − v₁| + |v − v₂|. Move
v₁ and v₂ to v and merge them.

*Rule B: a short edge that should be a vertex* (§4.3.2, Alg. 3–4). For intersection edge
e₀ = f₁ ∩ f₂, all of these hold:
- length(e₀) < t(e₀);
- in the graph e₀ is connected at no more than one end;
- its ends v₁ and v₂ lie on boundary edges e₁ ∈ f₁ and e₂ ∈ f₂;
- `edgeEdgeIntersection(e₁, e₂)` has a hit v₀ with |v₁ − v₀| and |v₂ − v₀| both < t(e₁) + t(e₂).

Then move v₁ and v₂ to v₀, merge them, and drop e₀'s curve. The faces touch at a point or
tangentially.

*Rule C: deviation at a shared vertex* (§4.3.3, Alg. 5; run during F–F trimming). For e₀ = f₀ ∩ f₁,
crossing boundary edges e₁ and e₂ of f₀ at v₁ and v₂, all of these hold:
- e₁ and e₂ share a vertex v₀;
- |v₁ − v₀| and |v₂ − v₀| are both < t(e₀);
- v₀ lies on f₁.

Then replace v₁ and v₂ by v₀. That removes the sliver piece between them.

**Data structures.** The intersection graph: vertex nodes and an edge list with
predecessor/successor, plus source faces per intersection edge. A per-edge tolerance.

**Failure modes.**
- Real short edges. Rule B needs the edge to be connected at no more than one end ("If we remove
  them, the intersection graph will be split up"); otherwise it is kept.
- The tolerance formula is heuristic. The paper says its ratios "can reach a value of more than
  1000", so the clamp matters.
- Rules A and C overlap card 4 step 4 and card 2. Apply them only to curve ends left unmatched
  after card 2's merge.

**Fit in cadrs.**
- `boolean.rs::run` stage 3, after `ssis` and before "Pieces of the split edges".
- Rule C belongs where intersection curves are cut at the vertices lying on them (the
  "Intervals" block).
- Removes a class of open-shell results, so fewer retries.
- Needs `ssi::curve_surface` and `ssi::curve_curve`, which exist.

**Sources.** `papers/booleans/Huang-2023-brep-boolean-repair.txt` §3.3 (rules), §4.1 (tolerance
formula), §4.3.1–4.3.3 with Algorithms 1–5, Example 5.1 (torus and hexagonal prism) (verified).

---

## 6. Classify at the intersection edge and propagate across it

**Problem.**
- Union-find groups in `run` stage 5 join sub-faces only across *uncut* edge pieces, so each side
  of every cut is its own group.
- Each group shoots rays: `parity_inside` uses up to 37 directions and wants two that agree, and
  a second member confirms. On a part with k separate cut loops, that is at least 2k groups times
  several rays.
- Rays are also the fragile step near the boundary.

**Why it is fast or robust.**
- Krishnan: "two adjacent components must have opposite classifications". With propagation the
  method "requires only two (one for each solid) ray shooting operations per CSG operation"
  (Krishnan 1997 pp. 128–129).
- Liang, Huang et al. 2025 classify the faces at an intersection edge from local data at one
  point Q: normals, coedge directions, curvature. One evaluation gives "usually 4" faces at once.
  They use rays only for components with no intersection edge. Their kernel (on ACIS's
  intersector) beat OCCT by 3–40x on ship-model booleans: 46.8 ms vs 421 ms on M1–M2,
  407 ms vs 1566 ms on C1–C2, and 23.7 s vs 597.8 s for a 68-part hull. These timings cover the
  whole method, not the classification alone.
- The same local test is HHK's area labelling. With orientation it gives oninB/onoutB, the on-same
  and on-opposite states.

**Algorithm.**
1. *Components.* Join sub-faces across uncut edge pieces, as now. Also record, for each cut edge
   piece, the up-to-four sub-faces around it (two per body, more if non-manifold) with their
   coedge directions.
2. *Local classification at a cut edge* (Liang/Huang §2.3). Pick a point Q inside the cut piece,
   away from its ends: the parameter midpoint, moved if it is within `tol` of a vertex. For each
   incident sub-face i compute:
   - the outward normal n_i (face sense applied);
   - the coedge tangent c_i in the face's loop direction;
   - the in-face direction `m_i = n_i × c_i` (points into the face's material side);
   - the normal curvature K_i along m_i, for ties.
3. Take `N = c_0` of any one face as the edge axis. Sort the sub-faces counter-clockwise about N
   by the angle of m_i in the plane ⟂ N. Two m_i within `tol::ANGULAR` of each other are tangent
   faces. Order them by K_i (second order); if K_i also tie, they are coincident and go to the
   on states.
4. Walk the sorted faces. If face i's c_i points the same way as N, the angular sector from m_i
   to the next face is *outside* face i's body; otherwise it is inside. ("若当前面的 ncoedge 与 N
   同向时, 则认为其到其之后的面之间的区域为当前面所在模型的外部".) A sub-face of A lies in one
   sector of B's two faces. That sector's label for B is the sub-face's class: inside or outside
   B.
5. *On states.* Two sub-faces of A and B with equal m_i (same tangent plane and curvature) lie on
   one surface. Record their pairing (cadrs already finds these as `coincident`):
   - normals agree: **on-same** (HHK oninB);
   - normals disagree: **on-opposite** (HHK onoutB, "cannot be part of the surface of A∩B").

   Keep rules (HHK; Krishnan §6.2.5; matches Liang/Huang Table 2):
   - Union: A out, B out, and one copy of on-same.
   - Intersect: A in, B in, and one copy of on-same.
   - A − B: A out, B in (flipped), and A's on-opposite.
   - Drop on-same for subtract and on-opposite for union and intersect.
6. *Propagate.* Build a graph whose nodes are components. Link components of one body across a cut
   with the relation "opposite class with respect to the other body". Classify every component
   that touches a cut edge with step 4. Then flood: every neighbour must get the opposite class.
   Count disagreements.
7. *Consistency check.* Two components adjacent across a cut that get the same class mean the
   surfaces only touch there, or a curve is wrong. Krishnan: "two adjacent components actually
   have the same orientation with respect to the other solid" signals tangential contact
   (§6.3). Liang/Huang: inconsistent local answers can come from tolerance, so take the majority
   over the cut pieces of a component and ignore outliers. Report disagreements in
   `CADRS_BOOL_DEBUG`.
8. *Seeds.* A component with no cut edge (a whole shell of A inside or outside B) still needs one
   point classification: card 7, one ray. One per connected component of A's shells, not per
   group.

**Data structures.**
- Per cut edge piece: the list of (sub-face, n, c, m, K) at Q.
- A component graph with "opposite" links.
- Class enum: `In | Out | OnSame | OnOpposite`. cadrs's `Class` already has these four.

**Failure modes.**
- Q at a singular point (a pole, a cusp of the curve, a vertex): move Q along the piece. Liang:
  "点 Q 的选取应避开奇异点".
- If the cut lies on a surface's parameter boundary, evaluate at a nearby interior point
  (Liang/Huang note 2).
- Tangent faces: when m_i ties within `ANGULAR` and K_i ties within a relative 1e-6, there is not
  enough information. Defer that component to card 7's ray.
- Non-manifold edges (more than four faces) need the full radial sort. The sort in steps 3–4
  handles any count.

**Fit in cadrs.**
- `boolean.rs::run` stage 5: replaces the per-group `Classifier` calls for every component that
  touches a cut. Groups no longer need a confirming second member.
- `classify.rs` keeps `Classifier` for the seeds.
- `split.rs`: cut arcs already carry the edge and its direction (`Arc`), so (n, c) at Q is
  available after splitting.
- The radial sort is also what `boolean.rs` uses to split edges shared by more than two faces
  ("pairing the faces in their order around the edge"). Share one routine.

**Sources.**
- `papers/booleans/Huang-2025-brep-boolean-efficiency-BVH-clustering.txt` §2.3 (face grouping
  graph G(F,E), local classification at Q, notes 1–4), Tables 1–2, 4–6 (verified; Chinese text
  read directly).
- `papers/booleans/Krishnan-1997-dissertation-boundary-evaluation-sculptured-solids.txt`
  §6.2.4–6.3, pp. 128–131 (verified).
- `papers/booleans/Hoffmann-Hopcroft-Karasick-1989-robust-set-operations.txt` §6, pp. 17, 21
  (verified).

---

## 7. One cheap ray per remaining component: first hit, cascade, perturbation

**Problem.** `Classifier::parity_inside` counts every crossing along up to 37 rays in
non-axis-aligned directions. It waits for two votes that agree and retries when a ray grazes an
edge. Each parametric face is tested by a triangulated grid plus Newton (`ray_parametric`). Parity
needs *all* crossings right, so one missed far hit flips the answer.

**Why it is fast or robust.**
- Cherchi uses only the *first* hit along the ray. The sign of the surface normal against the ray
  direction there decides in or out, so far-away faces cannot corrupt the answer.
- Axis-aligned rays turn box tests into 2D tests.
- In 3.8K booleans and 80K rays: 97% found a valid start at the first triangle tried, 2% at the
  second, 0.56% at the third, at most five triangles. "We never hit the last step" (the exact
  fallback). Perturbation never triggered on real shapes. On a mesh against its own translate, a
  single perturbation was always enough.
- Their classification was "up to 100× speedup" over topological flooding.
- This card applies to the seeds card 6 leaves, so the count is small anyway. The gain is
  robustness and fewer rays per seed (one, not two or more agreeing).

**Algorithm.** For a component P and the other body B:
1. *Start point.* Pick the sub-face of P with the largest uv area. Take a point x inside its
   trimmed domain, farthest from the boundary (`interior_points` already returns these first).
   Require x to be farther than `tol_P + max tol of B` (3D, through the metric) from every edge
   of P. Qi–Shapiro–Stewart: a classification is safe only outside the thickened boundary. Use
   the next candidate point, then the next sub-face, up to 5 (Cherchi stopped at 5).
2. *Direction.* Use ±x, ±y, ±z, ending at `p∞ = x + (box extent + 1)·axis`, beyond B's box. Pick
   the axis along which B's box is thinnest seen from x, so the ray meets the fewest face boxes.
3. *First hit.* Collect B's faces whose boxes the axis ray meets: a 2D test, since the ray is
   axis-aligned. Cherchi uses an octree for this; cadrs can use its face BVH. Intersect each with
   `ray_parametric`, or the exact line–quadric solve. Keep only hits inside the face (uv in-face
   test). Take the hit with the smallest t > 0.
4. *Decide.* With the outward normal n of B at the hit and ray direction d:
   - `n·d > 0`: x is inside B (the ray leaves B there);
   - `n·d < 0`: outside;
   - no hit: outside.

   Cherchi's form is the sign of the tetrahedron volume of (hit triangle, p∞). The normal dot
   product is the curved-surface analogue.
5. *Ambiguous hit.* Perturb when the hit is within `tol` of a face boundary (a B edge or vertex),
   when |n·d| < `tol::ANGULAR` (grazing), or when two hits on different faces lie within `tol`
   in t. Then "perturb the coordinates of p∞" without moving x, and repeat. Cherchi steps p∞ by
   `nextafter`. With curved faces and a 1e-6 tolerance, a step of `10·tol` across the ray in
   the two other axes is the practical analogue; that step is cadrs's, not Cherchi's. The ray is
   then no longer axis-aligned, so box tests become 3D.
6. *Last resort.* After 4 perturbations and 5 start points, fall back to the current
   `parity_inside` majority. Then card 10.

**Data structures.** The face BVH (exists). A per-face precomputed in-face polygon (exists:
`face_polygons`).

**Failure modes.**
- B open (a gap): the first hit may be missing or wrong. Card 10 handles it.
- Start point in a sliver face narrower than 2·tol: no valid start. Use a neighbour sub-face of
  the same component.
- Tangent rays on cylinders along an axis: rule out the axis parallel to the face's axis, choose
  another of the six.

**Fit in cadrs.**
- `classify.rs`: a new `Classifier::first_hit_inside(p) -> Option<bool>` beside `parity_inside`.
  `None` means fall back.
- `boolean.rs::run` stage 5 seeds.
- `shell_points_out` (shell orientation, also by parity) can use the same routine: the first hit
  from a point just inside a face along its outward normal must be outward.

**Sources.**
- `papers/booleans/Cherchi-2022-interactive-robust-mesh-booleans.txt` §5.1–5.3, Algorithm 1,
  Fig. 5–6, and the statistics quoted (verified).
- `papers/booleans/Qi-Shapiro-epsilon-regular-sets.txt` (from `notes/booleans.md` Q2, not
  re-checked).
- Krishnan 1997 p. 135 also shoots "multiple random rays to ensure correct parity" (verified). It
  is the approach cadrs has now.

---

## 8. Tighter face-pair culling: trimmed boxes, OBB, separating plane

**Problem.**
- Candidate pairs come from the face BVH and `Bbox::overlaps` of face boxes.
- A B-spline face's box is the box of its poles (`edge_bbox` doc), so it covers the whole
  untrimmed patch.
- A plane's or cylinder's face box is loose against a slanted neighbour.
- Each false pair costs an `ssi::surface_surface_tol` call. The overlap box limits the contour,
  but the call still sets up.

**Why it is fast.**
- Krishnan's two-step cull: boxes of the control points, "tightened by computing the tightest
  fitting axis-aligned rectangle in the domain that encloses the trimming region", then a
  separating-plane LP on control points. It leaves "few pairs of patches that are most likely to
  intersect".
- The LP has 4 unknowns and `2(ds+1)(dt+1)` constraints. Seidel's algorithm is linear in the
  constraints, so the whole cull is O(k) for k box-overlapping pairs.
- OCCT's OBB option can "significantly speed-up the intersection stage … by reducing the number of
  interfering objects".
- No source gives pair counts before and after, so the payoff must be measured with
  `CADRS_PROFILE=1` (the "face-face" phase).

**Algorithm.** It runs per candidate pair, in order of cost. Stop at the first test that
separates.
1. *Trimmed sub-patch box (B-spline faces).* Take the uv rectangle of the face's trimming loops
   (`uvbox` already computes it). Extract the sub-patch over it by knot insertion, or use the
   poles whose basis support meets the rectangle; the poles hull still contains the sub-patch.
   Use its pole box as the face box. For analytic faces, bound the trimmed region's 3D box from
   the edge boxes plus the surface's bulge (for a cylinder, the arc of the uv range in u).
   Compute once per face at import.
2. *AABB overlap* grown by `tol_a + tol_b`, as now (card 2's sum).
3. *OBB.* Per face, use the box in a local frame: a plane's normal frame; a cylinder's or cone's
   axis frame; for B-splines, the principal axes of the poles. Test two OBBs by the separating
   axis theorem: 15 axes (3 + 3 face normals, 9 edge cross products).
4. *Analytic separation.*
   - Plane P against any face with a convex hull H (poles or OBB corners): separated if every
     corner of H has `n·x − d` of one sign with |·| > `tol_a + tol_b`.
   - Two analytic faces: use the closed forms in `ssi`. For a cylinder vs a plane parallel to its
     axis, compare the plane distance with the radius.
5. *Separating plane on control points (B-spline × B-spline, B-spline × hull).* Find (a, b, c, d)
   with `a·X + b·Y + c·Z + d·W < 0` on the homogeneous poles (X, Y, Z, W) of patch 1 and `> 0` on
   patch 2. For `tol`, shift by `±(tol_a + tol_b)·|(a,b,c)|·W`. Solve with Seidel's randomized LP
   (expected linear time in fixed dimension). A plain simplex is also fine at these sizes: 16–100
   constraints for bicubic to higher-degree patches. Feasible means no intersection.
6. Only pairs that pass every test go to SSI.

**Data structures.** Per face: trimmed pole box, OBB (centre, three axes, half extents),
homogeneous trimmed poles for B-splines. All cached on `WFace` at import.

**Failure modes.**
- Near-touching pairs pass every test, so this card only removes clear misses. Card 4 handles
  near contact.
- Rational patches: use homogeneous coordinates as Krishnan does, valid with positive weights
  (cadrs's `NurbsSurface::new` checks them).
- The sub-patch extraction can cost more than it saves on small faces. Do it only when the uv
  rectangle is under, say, half the patch's domain in either direction (a cadrs heuristic).

**Fit in cadrs.**
- `boolean.rs::run` stage 3, pair loop "The pairs of faces to intersect": tests 3–5 go after
  `faces[fa].bbox.overlaps(...)`.
- `WFace::bbox` becomes the trimmed box.
- `super::bvh` stays the broad phase.
- Also helps `classify.rs`, whose ray–box tests use the same boxes.

**Sources.**
- `papers/booleans/Krishnan-1997-dissertation-boundary-evaluation-sculptured-solids.txt` §6.2,
  pp. 118–119 (Z-sweep with nested segment trees, O(n log² n + k) time, O(n² log² n) space; LP
  set-up) and §4.6 p. 91 (verified).
- OCCT spec, OBB option wording (fetched and verified).
- The analytic and OBB details in steps 1, 3 and 4 are standard techniques, not from the sources.
- cadrs's BVH replaces Krishnan's segment-tree sweep, which he chose "for ease of implementation"
  despite its space cost.

---

## 9. Jackson's tolerant validity check

**Problem.**
- `Body::check` accepts an edge end or p-curve within `POINT`, or within the entity's own
  tolerance, and loop gaps within `LOOP_GAP` (README, Tolerances).
- It has no rule for when two tolerant entities may overlap.
- A result with grown tolerances can be self-intersecting within its own tolerances and still
  pass. Or it can be rejected for a gap the tolerances explain, which triggers a retry.

**Why it is robust.** The criterion is exact about what tolerance allows. Jackson gives three
rules:
- non-adjacent entities' tolerance regions must not meet;
- each connected component of the meeting of two adjacent entities' regions must reach a shared
  lower-dimensional entity;
- the ordering vertex ≥ edge ≥ face.

The cost is one more pass over nearby entity pairs, using the BVH. It is cheaper than a rejected
result followed by a retry.

**Algorithm.** It runs on a body whose vertices and edges carry tolerances. Regions are: a vertex
is a sphere, an edge a tube around its curve (or around one fixed p-curve for a tolerant edge), a
face its surface thickened by its tolerance (always `CONFUSION` in cadrs, as in Parasolid then).
1. *Ordering.* For every vertex v and edge e at v: `tol(v) ≥ tol(e)`. For every edge e and face f
   using it: `tol(e) ≥ tol(f)`. ("The vertex tolerance must be at least as great as that of
   connected edges, and the edge tolerance must be at least as great as that of connected faces".)
   Repair by raising the vertex, never by lowering anything.
2. *Incidence.*
   - Each p-curve end lies within `tol(v)` of its vertex ("the ends of the parameter-space curves
     on an edge must lie inside the appropriate vertex sphere").
   - Each p-curve lies within `tol(e)` of the edge curve.
   - Consecutive coedges of a loop need not meet "as long as they are within vertex tolerance".
     That replaces the fixed `LOOP_GAP` with `tol(v)`.
3. *Edge–edge.* For edge pairs whose tubes' boxes overlap, find the stretches where
   `dist(e₁(s), e₂) ≤ tol(e₁) + tol(e₂)` (sum, card 2). Use `ssi::curve_curve` with that
   tolerance and keep the overlap intervals. Each stretch must contain a common end vertex
   ("each connected component of that intersection must intersect a common vertex of the two
   edges"). Otherwise the edges cross inside, and the result is invalid.
4. *Face–face.* For face pairs whose boxes overlap within the sum of tolerances, the contact must
   lie within the tolerance regions of their common edges and vertices ("contractible onto a
   collection of common edges or vertices"). Run the side test of card 4 step 2 with K = the
   common boundary: any sample within tolerance of the other face but farther than `tol(e)` from
   every common edge means the faces touch elsewhere, and the result is invalid.
5. *Face against non-adjacent edge, vertex against non-adjacent anything.* Use the same test with
   K empty.

**Data structures.** The BVH over face and edge boxes grown by their tolerances. Results as error
messages naming the two entities (for `CADRS_BOOL_DEBUG`).

**Failure modes.**
- Jackson admits "the recursive compression of topology … has not been fully implemented" and
  gives no algorithm for it. The check finds problems; it does not fix them. Feed its failures to
  card 1 Level B as gap locations.
- Steps 3–4 are sampling-based for curved geometry. Exact contractibility is not decidable cheaply.
  The sample density decides what is missed.

**Fit in cadrs.**
- `topo.rs` `Body::check` (outside the five files named, but that is where the check lives).
- The boolean's final step "Nor any other invalid body ([`Body::check`] …)" in `run`.
- `tol.rs` docs for `LOOP_GAP`, which becomes a fallback for entities without their own tolerance.
- Also serves STEP import and healing (`xbuild`).

**Sources.** `papers/booleans/Jackson-1995-local-tolerances.txt` §3.1–3.3, pp. 248–249, Fig. 3–4,
§7.2 (verified). Steps 3–5 as tests are a cadrs design; Jackson states the rules, not the tests.

---

## 10. Generalized winding numbers on trimmed NURBS as a fallback

**Problem.** When an input or an intermediate result is not watertight, ray parity and first-hit
rays give wrong answers near the gap. Today the only response is the retry ladder.

**Why it is robust.**
- The generalized winding number (GWN) is a sum over faces, so it is "indifferent to model
  watertightness". A small gap changes the field only near the gap.
- Spainhour & Weiss evaluate it exactly on trimmed NURBS, through boundary line integrals with
  adaptive Gauss quadrature, at about 0.15 ms per query (28-patch vase) and 1.29 ms per query
  (192-patch filleted box-sphere), serial.
- That is far slower than one ray, so it is a fallback only, used on the few components cards 6
  and 7 cannot settle.

**Algorithm** (Spainhour & Weiss 2025, Algorithm 1). For query q and each face S of the body:
1. Translate S so q is at the origin. Set `r = 0.01 × (uv bounding-box diagonal of S)` and extend
   the untrimmed patch by r in parameter space.
2. *Far field.* If q is outside S's 3D box, pick a coordinate axis that misses the box. Evaluate
   `w_S = (1/4π) ∮_∂S F·dΓ` with, for the z-axis,
   `F = ( y z / ((x²+y²)‖x‖), −x z / ((x²+y²)‖x‖), 0 )`. Permute coordinates for the x- or y-axis
   forms.
3. *Near field.* Otherwise, intersect the z-axis line (rotate S so a chosen line is the z-axis;
   use the line along S's average normal, or a random one if the average is near zero) with S.
   Per intersection (u₀, v₀, z₀) with normal n = S_u × S_v:
   - interior point and n ≠ 0: `w_S -= 0.5` if `n_z·z₀ > 0`, `w_S += 0.5` if `n_z·z₀ < 0`;
   - tangent to the line: rotate S randomly and restart;
   - within r of the trim boundary, or n ≈ 0: cut a uv disk of radius r around the hit out of S.
     Add the disk's GWN recursively, and continue with S minus the disk.

   Then add the boundary integral of step 2 over the (possibly modified) trim curves.
4. *Quadrature.* Use a fixed-order Gauss–Legendre rule per trim curve, bisected recursively until
   |whole − (left + right)| < ε_q, with ε_q = 1e-6. Cache nodes and tangents per curve per level
   across queries, except for disk-modified curves. The line–surface intersection tolerance is
   ε_ls = 1e-6.
5. *Coincident q* (q on S): the ±0.5 terms cancel, so add no correction. Treat q on a trim curve by
   cutting a much smaller disk whose GWN counts as zero.
6. *Containment* (from the paper's discussion): inside if the total GWN rounds to 1, outside if it
   rounds to 0. For cadrs, use |w − round(w)| < 0.25 as "decided" and send anything else to
   `BooleanFailed`. That threshold is cadrs's; the paper rounds and notes fractional values near
   gaps.

**Data structures.** Per face: the average normal, the extended patch, and per trim curve a
memoised list of Gauss nodes per bisection level. No hierarchy (the paper has none yet: the cost
per query is linear in faces).

**Failure modes.**
- Points near an open boundary get fractional values, and that is by design. A component whose
  test point is near the gap must take another point.
- Near-tangent lines: random rotation, "the most computationally expensive type of edge case".
- Analytic faces must go through the same formulation. Trim curves on quadrics are exact in
  cadrs, and the integrand needs only points and tangents of ∂S in 3D.

**Fit in cadrs.**
- `classify.rs`: `Classifier::winding(p) -> f64` as the step after card 7's fallback, and in
  `Classifier::classify` when the body is known open.
- Used by `boolean.rs` stage 5 when a component's ray results disagree, before giving up.
- Also useful for `meshsolid` and for classifying imported non-watertight STEP before healing.
- Embarrassingly parallel per query (rayon).

**Sources.**
- `papers/kernels_recent/Spainhour-2025-Containment-trimmed-NURBS-winding-numbers.txt` §3.1–3.4,
  Algorithm 1, Fig. 8–10, §5.1 parameter choices (verified: r = 1% of uv box diagonal,
  ε_q = ε_ls = 1e-6, 0.15 and 1.29 ms per query).
- The 2D predecessor (`papers/kernels_recent/Spainhour-2024-Containment-rational-curves-winding-numbers.txt`)
  fits point-in-trim with p-curves that miss by `LOOP_GAP`. Not checked for this card.

---

## Not carded, and why

- **Variadic winding-number vectors (Zhou 2016).** Labelling cells by `w_n ← w_c + s_p·δ_i`
  across patches gives every operation and `Kernel::cells` from one arrangement (verified, Zhou
  2016 §5, eq. 7). For two-body booleans, card 6's propagation already does this. It matters for
  `cells` and many-tool unions, not for failures. Revisit when `cells` is profiled.
- **Exact predicates (Shewchuk) and plane-based BSP booleans.** These apply only to the planar
  subset. The synthesis puts them under "later" or "study only".
- **Symbolic perturbation by operation (Manifold).** Union perturbs along the normal, subtract and
  intersect against it, so touching boxes fuse and equal-height cuts go through. That is the
  intended outcome of card 6's on-same and on-opposite table at tolerance. It comes from the
  Manifold wiki via the notes; it was not verified against a paper.
- **Union-specific shell regrouping (Liang/Huang §2.4.3).** After a union, a solid that the
  intersection touched stays one solid, so shells deleted from it need no containment tests.
  Small and verified, but `boolean.rs`'s shell grouping (`grouped into shells … and solids`) was
  not profiled as a cost. Measure first.

## What to measure before and after

`CADRS_PROFILE=1` already splits a boolean into import, edge-edge, edge-face, face-face, split,
classify, assemble, unify and shells. Record these per conformance and `kernel_perf.py` case,
before and after:

- the number of runs per boolean (1, 2 or 3), the target of cards 1, 2 and 5;
- face pairs sent to SSI and pairs skipped by each test, the target of cards 4 and 8;
- rays shot per boolean, the target of cards 6 and 7;
- the largest entity tolerance in each result, for growth under card 1.
