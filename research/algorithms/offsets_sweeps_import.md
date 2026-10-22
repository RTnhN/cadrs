# Offset, shell, sweep, loft, draft and STEP-import algorithms for cadrs_kernel

Algorithm cards drawn from `research/README.md`, `notes/offsets_sweeps_lofts.md`, the STEP-speed,
sewing and recognition parts of `notes/exchange_healing_naming.md`, the paper texts in
`papers/offsets_sweeps_lofts/`, and five OCCT 7.8.0 source files fetched for this write-up
(`Draft/Draft_Modification_1.cxx`, `XSTEPResource/STEP`,
`StepToTopoDS/StepToTopoDS_TranslateEdgeLoop.cxx`, `ShapeFix/ShapeFix_Shape.cxx`,
`BRepBuilderAPI/BRepBuilderAPI_Sewing.cxx`, all at tag V7_8_0). Each card says where it plugs into
`crates/cadrs_kernel/src/native/` and whether its steps were checked against a paper or source
text ("verified") or come only from the notes.

**Where the time goes today** (`crates/cadrs_kernel/PERF.md`, 2026-10-21, CPU time on one
thread). The category means hide a few slow cases:

- **Pipes**, 2.35 geometric mean but 1.21 median. Twelve cases (`pipe/specific_q7`…`s6`) take
  128–145 ms each against OCCT's 1.8–6.2 ms. Most other pipe cases are about 5 ms against 4 ms.
  An almost constant 135 ms points to a fixed cap being reached, most likely `MAX_SECTIONS` = 1024
  in `sweep/frames.rs` (see card 1). Nobody has checked this yet.
- **Drafts**, 1.88 mean but 1.00 median. Five cases dominate: `angle_a5` 7.1 ms vs 0.83,
  `a6` 21.8 vs 1.30, `a9` 22.7 vs 0.80, `b1` 69.3 vs 1.43, `c3` 12.6 vs 2.94.
- **STEP read**, 1.92 mean. Small files are faster than OCCT (`r103ec` 0.12, `trj3_as1` 0.11).
  Large ones are 7–14x slower: `trj6_pm4_ec_203` takes 6727 ms against 479, `as1_motor` 2022
  against 142, `bug24595` 1018 against 92. A gap that grows with the model points to per-face or
  per-edge work, or to something superlinear, in `xbuild`. It is not the lexer: these files are
  3.5 MB at most. The multithreaded column ("Native mt") is no faster, so nothing runs in parallel
  yet.
- **Offsets** are 0.17 of OCCT overall, but `offset/shape_type_a_a9` (372 ms vs 2.6, 146x) and
  `smlib offset_11_offset_bracket2` (7398 ms vs 52, 142x) are among the 20 worst cases, and
  `offset/faces_type_a_a9` times out at 180 s. These are the cases where the boolean count
  hurts.

**Already in cadrs** (so not carded, or carded only for what is still missing):

- Drafts already build OCCT's analytic surfaces: a plane turned about its hinge line, a cylinder
  or cone becoming a cone through its circle on the neutral plane (`local::draft`).
- Rounded offsets already put tubes only on convex edges, never on smooth ones
  (`offset_round::convex` returns None for tangent faces). Insets put them only on concave
  edges. Balls are still made at every vertex outward (`grown_except`), and at every vertex of
  a concave edge inward (`rounds`).
- Torus recognition exists (`recognize::torus`, checked on an 11×11 grid at 1e-7 of the size).
  The research report says it is missing; it is not.
- Sewing already uses a BVH for vertices and splits free edges at T-junctions
  (`surfacing::sew_faces`, `iges.rs`).
- Sweeps already use double-reflection rotation-minimizing frames (RMF) and exact rails.

## Ranked cards

| Rank | Card | Payoff | Size | Risk |
|---|---|---|---|---|
| 1 | [Sweep sections spaced by local curvature, frames by the RMF error bound](#1-sweep-sections-spaced-by-local-curvature-frames-by-the-rmf-error-bound) | Likely most of the 2.35x pipe gap (the 12 cases at ~135 ms) | Small | Low |
| 2 | [Closed-form edges and vertices for analytic drafts](#2-closed-form-edges-and-vertices-for-analytic-drafts) | The 1.9x draft gap (5 outlier cases) | Small | Low |
| 3 | [STEP import: OCCT's p-curve and SameParameter shortcuts](#3-step-import-occts-p-curve-and-sameparameter-shortcuts) | Large-file STEP read (7–14x slower now) | Medium | Low–medium |
| 4 | [Build imported faces in parallel](#4-build-imported-faces-in-parallel) | STEP read on large files; about 2x reported for the same idea | Small–medium | Low–medium |
| 5 | [Balls only at pointed vertices (Rossignac–Requicha pruning)](#5-balls-only-at-pointed-vertices-rossignacrequicha-pruning) | Fewer booleans per rounded offset, inset and shell; most on filleted parts | Small | Low |
| 6 | [Offset boundary from tentative faces, kept by distance](#6-offset-boundary-from-tentative-faces-kept-by-distance) | Rounded offsets and shells with no booleans when nothing clashes | Large | Medium–high |
| 7 | [Mitre joins checked first, then glued instead of cut](#7-mitre-joins-checked-first-then-glued-instead-of-cut) | Two to three local booleans fewer per sweep corner; failures caught early | Medium | Medium |
| 8 | [Parallel Part 21 parsing with ids as table offsets](#8-parallel-part-21-parsing-with-ids-as-table-offsets) | Small unless a profile says the lexer matters; less memory | Medium | Low |
| 9 | [Approximate skinning on a common knot vector](#9-approximate-skinning-on-a-common-knot-vector) | Bounded control nets for sampled lofts | Medium | Low–medium |
| 10 | [Gordon surfaces for lofts with guide curves](#10-gordon-surfaces-for-lofts-with-guide-curves) | New feature (guides); no measured gap | Medium–large | Medium |
| 11 | [OCCT's sewing candidate search and merge rules](#11-occts-sewing-candidate-search-and-merge-rules) | Faster, more predictable IGES and Sew; no effect on the STEP gap | Small | Low |

Cards 1–4 target measured gaps. Cards 5–6 cut the boolean count, which offsets need for their
outlier cases and for robustness, not for their mean. Before cards 1–3, put a counter or timer on
the suspect: the section count per run (card 1), `ssi::surface_surface` calls per draft (card 2),
and time per `XBuilder::finish` phase (cards 3–4).

---

## 1. Sweep sections spaced by local curvature, frames by the RMF error bound

**Problem.** `PathSweep::new` spaces its skinning sections evenly. Their number comes from the
largest curvature on the run and a 1e-8 mm target, so one tight bend on a long path drives every
run towards `MAX_SECTIONS` = 1024. The frames are always `FINE` = 256 double-reflection steps per
segment, whatever the curve.

**Why it is fast.**
- Cubic interpolation of a rail of curvature κ at spacing h errs by about $h^4\kappa^3/384$ (the
  Hermite bound; the code's comment uses the same formula). Spacing each section for its own κ
  makes the count $N=\int (\kappa^3/384\varepsilon)^{1/4}\,ds$ instead of
  $L\,(\kappa_{max}^3/384\varepsilon)^{1/4}$. For a path that is gentle except for one bend, that
  is several times fewer sections.
- The code estimates with ε = 1e-8, but the doubling loop that follows accepts
  `SECTION_TOL·(1+reach)` = 1e-5·(1+reach). Since $h\propto\varepsilon^{1/4}$, using one
  tolerance for both would cut the estimate by up to $(10^{-5}/10^{-8})^{1/4}\approx 5.6$x.
  Check whether 1e-8 is deliberate before changing it.
- Double reflection's one-step error is $\|r(h)-r_1\| = K h^5/720 + O(h^6)$, with
  $K = 2\kappa_1^2\tau_0 + \kappa_0^2\tau_0^3 + \kappa_1\kappa_0\tau_1 - \kappa_2\kappa_0\tau_0$.
  K = 0 exactly on spherical curves, and so on all planar curves (τ = 0). Planar spines
  therefore need no fine frame steps at all: frames at the section points are exact.
- Each frame step costs about 40 flops, but spine evaluation per step does not, and 256 steps
  per segment get re-sampled by every `rail_error` doubling.

**Algorithm.**
1. Sample the run at the current fine steps $s_i$, giving points $x_i$ and unit tangents $t_i$
   (already done in `PathSweep::new`).
2. Rail curvature per step: $\kappa_i = \angle(t_i,t_{i+1})/|x_{i+1}-x_i|$, and
   $\kappa^{rail}_i = \kappa_i(1+R\,\kappa_i)$, where R is the profile's reach (as the code does
   now, but per step instead of the maximum).
3. Local spacing $h_i = (384\,\varepsilon/\kappa^{rail\,3}_i)^{1/4}$, capped at the run length
   and at $\pi/24$ of turn per section (the existing `turn / (PI/24)` floor). Use
   ε = `SECTION_TOL·(1+R)`, the tolerance the doubling check already enforces.
4. Equidistribute: cumulate $\rho(s) = 1/h(s)$ and place $N=\lceil\int\rho\,ds\rceil$ (at least 8)
   section parameters $v_k$ at equal steps of the cumulative ρ. The parameters stay by length, so
   `me.v` becomes non-uniform. `NurbsCurve::interpolate(&at, &v, …)` already takes any v, and
   every profile curve and rail shares the same v, so the side surfaces stay compatible.
5. Keep the doubling check (`rail_error`), but refine only the intervals whose midpoint error
   exceeds ε: insert a section at their midpoint. Stop under the current rule (the error falls
   by less than 4x per refinement) or at `MAX_SECTIONS`.
6. Frames. If the run is planar (all $x_i$ within tol of one plane), compute frames only at
   the section points: the double reflection is exact there. Otherwise pick the fine count n
   from the error bound. Global angle error, accumulated over L/h steps:
   $E \approx (h^4/720)\int |K|\,ds$ (from eq. 14; this summation is our derivation). A profile
   point at reach R moves by R·E, so $h \le (720\,\varepsilon_f/(R\int|K|ds))^{1/4}$, with
   $\varepsilon_f$ = 0.1·ε.
   K needs $\kappa'$, $\kappa''$ and $\tau'$, which are noisy to estimate. A cheaper a-posteriori
   test: run double reflection with n and with 2n steps and compare the reference vectors at
   the shared points. The error at n is about $\tfrac{16}{15}\max|r_n-r_{2n}|$ (fourth order).
   Then $n_{need} = n\,(E_n/(\varepsilon_f/R))^{1/4}$. Start with n = 16 per segment, not 256.
7. Closed paths (unchanged method, verified source): spread the closing angle linearly in arc
   length, $\alpha(s) = \alpha_{tot}\,s/L$. This is the minimum of total squared angular speed
   (paper §6, Euler equation $\alpha''=0$). The paper recommends it over the curvature-weighted
   rule, which is ill-posed on straight parts.

Double reflection step (Table 1, unchanged from today):
$v_1 = x_{i+1}-x_i$, $c_1 = v_1\cdot v_1$, $r^L = r_i - (2/c_1)(v_1\cdot r_i)v_1$,
$t^L = t_i - (2/c_1)(v_1\cdot t_i)v_1$, $v_2 = t_{i+1}-t^L$, $c_2 = v_2\cdot v_2$,
$r_{i+1} = r^L - (2/c_2)(v_2\cdot r^L)v_2$, $s_{i+1} = t_{i+1}\times r_{i+1}$. It is degenerate
when $x_{i+1}=x_i$, or when $(x_1-x_0)\cdot(t_1+t_0)=0$ and $(x_1-x_0)\times(t_1-t_0)=0$
(§4.4).

**Data structures.** A per-run `Vec<f64>` of section parameters (already `me.v`, now
non-uniform). The cumulative density as a `Vec<f64>` over the fine steps.

**Failure modes.**
- A spine with a curvature spike between fine samples is under-sampled. Keep the `rail_error`
  check, which samples the true motion between sections.
- Frames that are not smooth (locked direction, Frenet near an inflection): the current
  "less than 4x per doubling" stop still applies.
- Too few frame steps through a highly twisted region shows up as a large Richardson
  difference. Refine there; the bound is local.
- The paper's bound assumes exact tangents. cadrs evaluates them exactly from the curve.

**Fit in cadrs.** `sweep/frames.rs`: `PathSweep::new` (fine steps, `by_err`, the doubling loop),
`rail_error` (made per interval). It replaces the uniform `v` and the fixed `FINE`. It removes no
module. First log `n` for `pipe/specific_r4` and its siblings to confirm they hit
`MAX_SECTIONS`. If they don't, profile `skin` and the face assembly instead.

**Sources.**
- `papers/offsets_sweeps_lofts/Wang-Juttler-Zheng-Liu-2007-computation-of-rotation-minimizing-frames-TR.txt`:
  Table 1 (the algorithm), §4.4 (degenerate cases), Theorem 4.4 / eq. 14 (one-step error and
  K), Theorem 4.5 (O(h⁴) global), §6 (closed frames, linear-in-arc-length twist). Verified.
- The $h^4\kappa^3/384$ section bound is the code's own comment (standard cubic Hermite bound),
  not from a paper here. The summed global error and the Richardson rule are our derivations.
- The tolerance mismatch is from reading `frames.rs`, not from the research.

---

## 2. Closed-form edges and vertices for analytic drafts

**Problem.** `local::draft` already makes OCCT's analytic surfaces. It then hands them to
`replace_surfaces_in`, which recomputes every moved edge with the general
`ssi::surface_surface` (with a sampled search box) and every vertex with a general meet. OCCT
recomputes the same edges in closed form and uses the general intersector only when that fails.

**Why it is fast.** OCCT's draft is a pure geometry substitution
(`BRepTools_Modifier`, no topology change). Its edges between planes, and between a plane and
a coaxial cylinder or cone, are lines and circles built directly; it falls back to a general
intersection only for other pairs. Each closed-form edge costs O(1) instead of a surface
intersection over a box, and the old edge picks the branch with no candidate filtering. The
median draft case is already at parity (1.00), so the gain is in the five outlier cases. Their
cost profile is not yet known (see "Fit in cadrs").

**Algorithm.**
0. Eligibility (OCCT's `NewSurface` conditions, verified). Let D be the pull direction and N the
   neutral plane. Every drafted face is a plane, cylinder or cone. For a cylinder or cone,
   $|D\cdot n_N| > 1-\epsilon_a$ and $|D\cdot a| > 1-\epsilon_a$ (axis a parallel to D, neutral
   plane square to it), and N ∩ surface is a circle. OCCT uses $\epsilon_a$ =
   `Precision::Angular()` = 1e-12, and rejects a cylinder's ellipse unless
   major < 1e5·minor. Faces joined G1 to a drafted face are drafted too when they are planes,
   cylinders or extrusions (`InternalAdd` recursion on `Continuity ≥ G1`); cadrs's
   `tangent_closure` covers this.
1. New surfaces (already in cadrs; OCCT's formulas for reference, verified):
   - Plane. Hinge line $L = P\cap N$ with direction $\hat x$; $\hat y = n\times\hat x$;
     $a=D\cdot\hat x$, $b=D\cdot\hat y$, $c=D\cdot n$ (b and c negated for a reversed face).
     $\delta=\sqrt{1-a^2}$, which must exceed $|\sin\alpha|$. $\phi=\operatorname{atan2}(b/\delta,c/\delta)$,
     $\theta_0=\arccos(\sin\alpha/\delta)$, $\theta=\theta_0-\phi$; if $\cos\theta<0$ then
     $\theta=-\theta_0-\phi$; wrap θ into $(-\pi,\pi]$. Rotate the plane about L by θ (skip if
     $|\theta|$ ≤ 1e-12).
   - Cylinder. Centre c of the circle N ∩ cylinder, Z = its axis parameter. Start with α' = α;
     negate it once for a reversed face on a direct frame (or a forward face on an indirect
     one), and once more if the axis points against D. Then $R' = R + Z\tan\alpha'$. If
     $R' < 0$, take $|R'|$ and keep α'; otherwise negate α'. The new surface is the cone with
     the cylinder's frame, semi-angle α' and reference radius |R'|.
   - Cone. Same, from the neutral circle's radius. Angle 0 gives a cylinder. Unchanged if the
     new semi-angle equals the old within 1e-12.
2. New vertices. Let $F_1..F_k$ be the vertex's faces with new surfaces.
   - Three or more planes: solve $[n_1;n_2;n_3]\,x=[d_1;d_2;d_3]$ by Cramer's rule with the three
     best-conditioned planes; accept if $|\det|$ > `tol::ANGULAR` and every other face's plane
     is within `tol::POINT`.
   - Two planes and one quadric: line $L = P_1\cap P_2$, then $L\cap Q$ (a quadratic in the line
     parameter for a cylinder or cone). Take the root nearest the old vertex.
   - One plane and a coaxial cone or cylinder pair: the circle of step 3, intersected with the
     third surface.
   - Otherwise: today's `meet`.
3. New edges, by surface pair (closed form first, `ssi::surface_surface` last). $p_0$, $p_1$
   are the new end vertices.

   | Pair | Edge |
   |---|---|
   | plane–plane | The line through $p_0$, $p_1$ (direction $n_1\times n_2$ if they coincide). |
   | plane–cylinder/cone, plane normal ∥ axis | Circle: centre = axis ∩ plane, radius r(v) at that height. Arc from $p_0$ to $p_1$ in the old edge's direction. |
   | plane–cylinder/cone, plane contains a ruling | The ruling (line) through $p_0$. |
   | plane–cone, general | The exact conic `ssi` already builds for plane × cone, called directly. Pick the branch through $p_0$. |
   | cone/cylinder–cone/cylinder, coaxial | Circle at the height where the radii agree. |
   | anything else | `ssi::surface_surface`, as today. |

   Check each closed-form edge at its midpoint (distance to both surfaces < `tol::POINT`). On
   a failed check, fall back to the general path for that edge only.
4. p-curves. Planes take the affine projection; rims and rulings of cones and cylinders are
   iso-lines (`pcurve::exact` covers both). Only plane–cone conics and general edges need a
   fit.

**Data structures.** None new. A small `enum EdgeKind { Line, Circle, Conic, General }` chosen
per edge from the two surface kinds.

**Failure modes.**
- The hinge is parallel to the pull ($\delta \le |\sin\alpha|$): OCCT returns no surface and
  flags the face (`Draft_FaceRecomputation`). cadrs should do the same before any intersection.
- A vertex where the faces no longer meet, or an edge whose new curve leaves a face: OCCT gives
  `Draft_VertexRecomputation` or `Draft_EdgeRecomputation` with the bad shape, and refuses
  topology changes outright. Keep cadrs's current error, or the general path.
- A cone's apex inside the face: OCCT trims the cone's v range to the apex side
  (`Vapex = −RefRadius / sin(SemiAngle)`, with a Confusion margin). Check the same thing.

**Fit in cadrs.** `local.rs`: `replace_surfaces_in`, in the "Vertices" block (before `meet`) and
the "Edges" block (before `ssi::surface_surface`). It removes the general SSI call for analytic
pairs, for drafts and also for sharp offsets of analytic bodies, which share
`replace_surfaces`. Prerequisite: time `draft/angle_b1`, `a6` and `a9`, and count SSI calls and
`check` time. If the time is elsewhere (p-curve fits, tolerance fitting, `check`), this card
still applies but moves less.

**Sources.**
- OCCT `src/Draft/Draft_Modification_1.cxx` (V7_8_0): `NewSurface`, `FindRotation`,
  `InternalAdd`, `Propagate`, `Perform`. Verified (fetched).
- `BRepOffsetAPI_DraftAngle` reference ("treats planar, cylindrical and conical faces", "do not
  use shapes, which … would modify the topology"): notes only (`notes/offsets_sweeps_lofts.md`
  E2).
- No paper describes draft construction (notes, section e).

---

## 3. STEP import: OCCT's p-curve and SameParameter shortcuts

**Problem.** `xbuild` ignores every p-curve in the file and computes its own for every
(face, edge). The exact one comes from `pcurve::pcurve`; otherwise it is a piecewise cubic fit of
projected points (`pcurve.rs`: "refined until it is within 1e-10 of the projection";
`xbuild::fit_span`). It then fits tolerances. OCCT keeps the file's p-curves, recomputes them
only on planes, does not run SameParameter while translating, and runs each `ShapeFix` step only
when its check trips.

**Why it is fast.** On a B-spline-heavy file, almost all p-curves are fits: projections
(Newton, plus a seed search) at 12+ points per span, refined recursively. OCCT does none of this
for p-curves the file provides. On planes it needs no stored p-curve at all: the projection is
affine. cadrs's slowdown grows with model size (7–14x on the 1–7 s files), which fits a
per-edge cost of this kind. OCCT's defaults, from the fetched sources:

- File p-curves on planes are dropped (`CheckPCurves`: `if (isPlane) { RemovePCurves… }`), and
  `hasPcurve = false` for planes.
- Elsewhere the file's p-curve is used as given (`B.UpdateEdge(E, C2d, Face, 0.)`), checked by
  `CheckPCurve`, and projected (`ShapeFix_EdgeProjAux`) only if missing. If projection fails,
  the p-curve is removed.
- Shape processing after reading is one operator, `FixShape`. Nearly every mode is −1, "only if
  needed". `FixEdgeSameParameterMode 0` (off per wire), `ModifyTopologyMode 0`,
  `MinTolerance3d 1e-7`, `AutoCorrectPrecisionMode 1`.
- The shape-level SameParameter pass calls `ShapeFix::SameParameter(shape, enforce = false)`.

**Algorithm** (cadrs must keep its invariant that a p-curve is parametrised by its edge's t).
1. Read the file's p-curves (`SURFACE_CURVE` / `SEAM_CURVE` → `PCURVE` →
   `DEFINITIONAL_REPRESENTATION`) into the `XFace` loops as optional hints. `step.rs` reads them
   today only to reach the 3D curve.
2. Plane: the exact affine projection (`pcurve::exact`), file p-curve ignored, as OCCT.
3. Analytic surface with an iso edge (rulings, rims, meridians): exact, as today.
4. Any other surface, when the file gives a p-curve $\gamma(s)$:
   a. Same range: if $[s_0,s_1]$ equals the edge's $[t_0,t_1]$ within `tol::REL`, test same
      parameter. At m = max(8, 2·poles of γ) points $t_j$ (OCCT samples 23 in its own
      SameParameter fallback), require $|S(\gamma(t_j)) - C(t_j)| \le \tau$ with
      τ = max(edge tolerance, `tol::POINT`). If it passes, take γ as the p-curve: no projection,
      no fit. Exporters built on OCCT, Parasolid and ACIS normally write same-parameter
      p-curves (an expectation, not measured).
   b. Same-range but not same parameter, or a different range: map linearly onto $[t_0,t_1]$ and
      use $\gamma$ as the Newton seed for each projection in the fit
      (`project(..., hint)` already takes one). This skips the seed search (the 24×24 `seed`
      grid in xbuild).
5. Fit tolerance. Fit the remaining p-curves to the tolerance `check` enforces, not to 1e-10: a
   target of 0.1·max(edge tolerance, `tol::POINT`) = 1e-7 mm. Cubic fits converge as $h^4$,
   so this needs about $(10^{-7}/10^{-10})^{1/4}\approx 5.6$x fewer spans. `pcurve_checked_tol`
   already takes `etol`; find which path still uses 1e-10.
6. Tolerances only where something changed. Run `fit_tolerances` on the edges and vertices
   whose p-curves were fitted, moved or rejected, not on every edge. A file's vertex and edge
   tolerances stand otherwise (OCCT's `read.stdsameparameter.mode` = 0).
7. Repairs only on detection, as now for loops (`chains` before `repair_loop`). Apply the same
   rule to seams, singular points and senses: run each only when its cheap test fails.

**Data structures.** `XFace` loops gain `Option<Curve>` per coedge (the file's p-curve). A
per-edge flag for "p-curve from the file / exact / fitted", so step 6 knows what to refit.

**Failure modes.**
- A wrong file p-curve, such as a shared p-curve on two faces (OCCT's "SYRKO" case) or a seam
  with its two uses swapped: the step 4a test rejects it, and the fit runs. For seams, OCCT picks
  the forward p-curve with `ShapeAnalysis_Curve::SelectForwardSeam` and fails the edge if
  neither fits.
- p-curves with a zero range or outside a bounded curve: OCCT drops them (w1 == w2) or clamps
  them (`CheckPCurves`). Do the same before step 4a.
- Precision: OCCT uses the file's `UNCERTAINTY_MEASURE_WITH_UNIT` (mode 0) or a user value with
  default 1e-4 (notes). cadrs's `JOIN` = 1e-5 is stricter. Keep it, but report the file's value.

**Fit in cadrs.** `step.rs` (read the p-curves), `xbuild.rs` (`XFace`, `pc_from`, `pc_raw`,
`fit_pieces`, `fit_span`, `finish_mapped`), `pcurve.rs` (fit tolerance). It removes most fits on
B-spline faces. Prerequisite: a per-phase timer in `XBuilder::finish` (analytic recognition,
p-curves, trace and resolve, seams, orientation, tolerances) on `trj6_pm4_ec_203` and
`as1_motor`. If one phase is superlinear (for example `pc_cache` growth, or a pass over all
edges per face), fix that first.

**Sources.**
- OCCT `src/XSTEPResource/STEP` (the full `FromSTEP.FixShape.*` list),
  `src/StepToTopoDS/StepToTopoDS_TranslateEdgeLoop.cxx` (`CheckPCurves`, `isPlane`,
  `ComputePCurve`, `EdgeProjAux`, `SelectForwardSeam`), `src/ShapeFix/ShapeFix_Shape.cxx`
  (`SameParameter(myResult, Standard_False)`). Verified (fetched).
- That non-enforced `ShapeFix::SameParameter` skips edges already flagged SameParameter is a
  recollection of `ShapeFix.cxx`, not fetched. Unverified.
- `read.precision.mode`/`val` (1e-4) and `read.stdsameparameter.mode` off: notes only
  (`notes/exchange_healing_naming.md` §3, from the OCCT STEP user guide).

---

## 4. Build imported faces in parallel

**Problem.** `XBuilder::finish` builds faces one after another: analytic recognition, p-curves,
uv tracing, period resolution and seams. The faces are almost independent. The PERF "Native mt"
column is no faster than one thread on any STEP case.

**Why it is fast.** Per-face work is embarrassingly parallel once vertices and edges exist. The
one published STEP parser that beats OCCT does this: a thread pool per level (shape → faces →
edges → curves). It reports about 2x over its serial version and about 1.5x over OpenCASCADE.
That comes from the abstract only (the paper is in Chinese and unread). Tessellation already
does the same with rayon.

**Algorithm.**
1. Serial: vertices, then edges with their ranges (`edge`, `edge_range`). Edges are shared, so
   they are made once, before any face.
2. Parallel over faces (`par_iter` over face indices), each task reading only the shared
   vertices and edges:
   - the surface, through `analytic` (recognition);
   - every coedge's p-curve (card 3's order: exact, file, fitted), into a per-face `Vec`;
   - loop chaining checks, the uv trace, period resolution (`trace`, `resolve`).
   Each task returns a `FaceDraft { surf, pcurves, loops, edits }`, where `edits` lists the
   changes it wants to shared topology: an edge split for a seam (`split_edge`), a seam to add,
   a degenerate edge at a pole.
3. Serial, in face order: apply the `edits` (splits, seams, point edges), redoing the p-curves
   of the edges a split touched. Then `orient_shells` and tolerances.
4. Replace `pc_cache: RefCell<HashMap>` with the per-face results. A cache shared across
   threads would need a lock, and it is only needed because faces are traced more than once.

Determinism: each task is a pure function of the shared vertices and edges, and results are
gathered by face index, so the output does not depend on thread count or scheduling. That is a
README requirement: the same inputs give the same bytes.

**Data structures.** `FaceDraft` per face; `Vec<FaceDraft>` collected in index order. Edge
splits are recorded as `(edge, t)` and applied sorted by (edge, t) so the order is fixed.

**Failure modes.**
- Two faces asking to split one edge at nearly the same t: merge the requests within
  `tol::POINT` before applying them.
- A face whose loop repair reorders shared edges: repairs that change only the face's own loop
  list are local and fine. Anything that changes an edge goes into `edits`.
- Memory: parallel tasks hold their fits at once. That is bounded by the face count.

**Fit in cadrs.** `xbuild.rs`: `finish_mapped` and the face loop, `pc_from`, `trace`, `add_seam`,
`split_edge`, `close_whole`. It needs rayon in the kernel, which `tess` already uses. Do this
after card 3: fewer fits make the parallel part smaller but also leave the serial part
dominant, which is what to measure.

**Sources.** Bi, Liu, Yang, Zeng, Yong 2023, J. CAD & CG 35(12) 1842–1850 (abstract only;
`notes/exchange_healing_naming.md` §3). Not verified, no local copy. The OCCT 7.8/7.9
multithreaded STEP mode is described as beta and "not available in any official versions"
(notes).

---

## 5. Balls only at pointed vertices (Rossignac–Requicha pruning)

**Problem.** `grown_except` puts a ball at every vertex of the body, and the inset and cavity
builders (`rounds`) put one at every vertex of a concave edge. Each ball is one more boolean
union or difference. On filleted parts most vertices sit where smooth edges meet, and their
balls add nothing.

**Why it is fast.** The offset boundary lies in the union of the faces' normal offsets, tubes
round the singular edges and spheres at the singular points (Rossignac & Requicha §2.6). A
sphere contributes boundary only over the vertex's normal cone, the directions n for which the
vertex is the nearest point. If that cone has no interior, the sphere adds a set of measure zero
already on the neighbouring tubes or slabs. Every ball dropped is one boolean fewer. The share
is unmeasured; on parts with filleted edges it should be most vertices (the research report's
"probably a large share" is an estimate).

**Algorithm.** For each vertex v that is not a pole or apex of a closed edge:
1. Collect unit tangent directions leaving v: for each non-degenerate edge at v, its tangent at
   v pointing away. For each face at v, also add the unit bisector of its corner, inside the
   face's sector in the tangent plane, so a reflex corner (> π) is seen.
2. Hemisphere test. Find n with $\max_k n\cdot t_k < -\varepsilon$, ε = 1e-9 (`tol::ANGULAR`).
   With 3–8 vectors, brute force is enough. Try $n = -\sum t_k/|\sum t_k|$, then for each pair
   $\pm(t_i\times t_j)$ tilted slightly towards $-\sum t_k$. A strict hemisphere exists exactly
   when one of these passes, since an LP optimum lies on two active constraints. If none
   passes, neither the body's tangent cone nor its complement is pointed. No ball is needed,
   outward or inward.
3. Which side. If n exists, the point $v + \delta n$ (δ = 1e-3 of the shortest edge at v) is
   classified against the body (`classify` with an exact ray, or the local test from the faces
   at v).
   - Outside: the body's tangent cone is pointed, so the vertex needs a ball for an outward
     offset or grown body.
   - Inside: the complement's cone is pointed, so it needs a ball for an inset or cavity.
4. Edges stay as now: tubes on convex edges outward, on concave edges inward, none on smooth
   edges. That is Rossignac's rule, already implemented.
5. Optional, same principle: per face, the principal-radius test (card 6 step 2) lets a face
   whose offset is wholly inside-out skip its slab.

Examples: a box corner (three convex edges) keeps its ball outward and gets none inward. A
vertex where only smooth edges meet, as on a fully filleted box, gets none either way. The end
of an L-section's concave edge on its end face gets no ball: the tangent cone's hull is a
half-space.

**Data structures.** None. A per-vertex `enum Pointed { Body, Complement, Neither }` computed
once and shared by `grown_except`, `rounds`, `inner_core` and `cavity_by_regions`.

**Failure modes.**
- Curved faces bending away within the corner: the tangent test is local. A vertex whose
  normal cone has interior only at second order (tangent faces that separate) is treated as
  smooth. The tubes' flat ends then meet without a ball and may leave a sliver. Guard: if the
  outward normals of the faces at v differ by more than 1e-6 rad but step 2 says "Neither",
  keep the ball.
- Cone apexes and sphere poles are singular points the B-rep may not list (Rossignac §4.1:
  "the apex of a cone is a singular point and must be n-offset"). Keep a ball at any apex
  inside a face's closure, as `singular_points` finds them.
- A wrong "Neither" drops boundary, and the result is wrong without failing. Gate the change on
  the offset and shell suites (`offset_local.rs`, `smlib_offset.rs`, OCCT offset cases).

**Fit in cadrs.** `offset_round.rs`: the vertex loop of `grown_except`, `rounds`, `inner_core`,
`cavity_by_regions`. It removes most balls on filleted parts. No prerequisite.

**Sources.**
- `papers/offsets_sweeps_lofts/Rossignac-Requicha-1986-offsetting-operations-in-solid-modelling.txt`:
  §2.6 (boundary ⊂ {p : d(p, S) = r}; the faces, singular curves and points, and their
  n-offsets), §4.1 (pruning rules: negative n-offsets not needed for $S\uparrow r$; concave
  edges not needed for $S\uparrow r$, convex not for $S\downarrow r$; "edges that separate tangent
  faces … never need to be n-offset"; cone apex). Verified.
- The pointed-cone criterion for balls is our derivation from §2.6. The paper gives a ball to
  every singular point. Not verified as a published rule.

---

## 6. Offset boundary from tentative faces, kept by distance

**Problem.** A rounded offset is the body ∪ n slabs ∪ tubes ∪ balls, combined one boolean at a
time (`combine_all`, up to three rounds with retries). The outlier cases (146x, 142x, a 180 s
time-out) are this path. Shells subtract the same kind of union.

**Why it is fast.** When nothing clashes, the offset boundary is just the tentative faces
assembled along the body's own adjacency: face → offset face, convex edge → tube patch, pointed
vertex → sphere patch. That is OCCT's Arc join, built with no booleans. Only pieces that
actually cross something far away need intersecting, and then each piece is kept or dropped by
one distance query (Rossignac's classification), not by a boolean against the union of
everything else. Cost: O(F + E + V) to build, plus a BVH pass over the piece boxes, plus SSI only
for clashing pairs.

**Algorithm** (outward, constant d; inward is the same with the complement's convexity).
1. Tentative faces, with their uv domains:
   - each face F: $q + d\,n(q)$ over F's own loops (the outer face of the slab, `slab.rs`);
   - each convex edge E: the canal patch $E(t) + d\,(\cos\phi\,n_1(t)+\sin\phi\,b(t))$, with φ
     limited to the wedge between the two faces' normals $n_1(t)$, $n_2(t)$. Between planes this
     is a cylinder sector; along a circle, a torus sector. Its long edges are exactly the offset
     curves of E on the two offset faces, so the pieces share edges;
   - each pointed vertex (card 5): the sphere patch over the spherical polygon of the faces'
     normals at v, bounded by the tube patches' end arcs;
   - each concave edge: nothing (Rossignac §4.1). Instead, mark the pair of offset faces to be
     re-intersected: they overlap there, and their intersection is the new edge (as
     `offset_sharp` does).
2. Local validity per piece. For a face with principal curvatures $\kappa_{1,2}$ (positive where
   convex), the offset's normal keeps its direction iff $(1+d\kappa_1)(1+d\kappa_2) > 0$. It is
   inside-out where $d\kappa < -1$ (critical curvature $\kappa_{crit}=-1/d$). If inside-out
   everywhere, drop the face (Parasolid: "blends whose radius is less than the offset distance
   are removed"). If only in part, mark it for step 4. For analytic faces this is a radius
   comparison (a concave cylinder of radius ρ < d). cadrs already handles that one with its
   wedge.
3. Clash detection. Build a BVH over the pieces' boxes (`bvh.rs`). Candidate pairs are
   overlapping boxes of pieces that are not adjacent (adjacency = sharing an edge or a vertex in
   the assembly of step 1), plus the concave-edge pairs. Rule out pairs with a separating plane
   or a distance test (`distance::min_distance` between pieces > `tol::POINT`). If no pair
   survives and step 2 marked nothing, assemble the pieces into the shell and stop: zero
   booleans.
4. Otherwise: intersect only the surviving pairs (`ssi`), split the pieces in uv along the
   curves (the boolean's `split` machinery), and group the sub-pieces into components that no
   intersection curve separates (union-find over uncut edges).
5. Classify one sample per component. Take p at the component's deepest interior uv point and
   compute $D = \operatorname{dist}(p, S)$ with `distance::min_distance` against the original
   body. Keep the component iff $D \ge d - \delta$, with δ = 10·`tol::POINT`. By construction p
   is at distance d from its generator, so a smaller D means another part of S is nearer and p
   lies inside the offset (Rossignac Algorithm 6: < r in, > r out, = r on). Prefer growing from
   a known-kept seed across uncut edges, as Jung, Shin & Choi do, so most components need no
   query.
6. Assemble the kept components with the boolean's shell assembly (`build`/`boolean` face
   assembly), and run `Body::check`. On failure, fall back to today's `combine_all`.

**Data structures.** A `Tentative { surf, uv_loops, generator: Face|Edge|Vertex, adj: [piece] }`
list mirroring the body's topology; the BVH; a union-find over split sub-pieces; per component,
a kept flag.

**Failure modes.**
- B-spline faces whose offsets self-intersect only in part: steps 2 and 4 find the region, but
  the trim curve is hard to get. Seong, Elber & Kim solve $F = F_u = F_v = 0$ on the squared
  distance (study only, per the research report). Fall back to booleans for such faces.
- Near-tangent clashes (two pieces touching): D ≈ d on both sides. Rossignac resolves the
  "= r" case with neighbourhoods, and so does the boolean. Fall back.
- Vertices with more than three edges where neighbours drift past each other (Parasolid's
  vertex splitting, mirrored faces): no single rule. Fall back.
- Free-form edges: a canal surface round a B-spline edge folds where the edge curves tighter
  than d (`edge_tube` notes this). Detect with κ(t)·d ≥ 1 and fall back.

**Fit in cadrs.** `offset_round.rs`: replaces `grown_except`'s piece list and `combine_all` on
the common path; `inset_rounded`, `inner_core` and `cavity_by_regions` likewise, with
complement convexity. It reuses `slab::face_slab`'s offset face, `edge_tube`'s geometry (the
patch, not the solid), `bvh`, `split`, `distance` and the boolean's assembly. Prerequisite:
card 5's pointed-vertex test. Start with bodies whose faces are all analytic.

**Sources.**
- `papers/offsets_sweeps_lofts/Rossignac-Requicha-1986-offsetting-operations-in-solid-modelling.txt`:
  §2.6, §3 Algorithm 2 (tentative faces → tentative edges → classify), §4.1, §4.3 Algorithms 6
  and 7 (point classification by distance; distance by projection on singular points, curves
  and superfaces). Verified.
- `papers/offsets_sweeps_lofts/Patrikalakis-Maekawa-2003-MIT-lecture-13-offset-curves-surfaces.txt`:
  eqs. 13.16–13.19, offset principal curvatures $\kappa/(1+d\kappa)$, critical curvature $-1/d$.
  Verified.
- OCCT Arc join ("rounds or intersects adjacent faces along its edges depending on the convexity
  of the edge") and Parasolid's deletion and repair rules: notes only (A5, A6).
- Jung, Shin & Choi 2004 seed growing: notes only (A9); the local copy is the Korean version.
- Seong, Elber & Kim 2006:
  `papers/offsets_sweeps_lofts/Seong-Elber-Kim-2006-trimming-self-intersections-offset-surfaces-distance-maps.txt`.
  Cited as the fallback idea only; not checked for this card.

---

## 7. Mitre joins checked first, then glued instead of cut

**Problem.** `sweep::corner::join` mitres by extending each piece with a prism of its cap and
cutting both at the bisecting plane with local box booleans, then uniting them. When OCCT's
RightCorner precondition holds, the two cut faces coincide exactly. The join is then a glue
(`path::glue`), not a boolean. When the precondition fails, it is better to know before the
booleans run.

**Why it is fast.** It replaces two cuts and one union per corner with an exact end-face
construction and a glue, both linear in the profile's size. A failing corner (fold, too short a
segment) is caught by an O(profile) test instead of a failed boolean with retries. OCCT's
default join (`Transformed`) does no corner work at all and accepts a self-intersecting shell;
cadrs keeps valid corners, so this card cuts their cost rather than dropping them.

**Algorithm.** For a corner at point c with incoming direction $d_1$, outgoing $d_2$, turn angle
θ (MAX_CORNER = 179°), and the profile square to the path at the corner:
1. Bisector plane: normal $m = (d_1+d_2)/|d_1+d_2|$ through c. Turn direction
   $w = (d_2 - d_1)/|d_2-d_1|$, which lies in the profile plane.
2. Extent. For a profile point p, let $\rho = (p-c)\cdot w$ (signed; positive towards the inside
   of the turn). Its extension past the corner is $\ell(p) = -\rho\tan(\theta/2)$ on each
   side. Points on the inner side trim back by $\rho_{max}\tan(\theta/2)$, where
   $\rho_{max} = \max_p \rho$ over the profile.
3. Preconditions (all O(profile + nearby spine samples)):
   a. The profile is square to the path at both sides (OCCT: RoundCorner and RightCorner are
      reliable only "if a profile was set with option WithCorrection").
   b. Straight ends: over arc length $\lambda = \rho_{max}\tan(\theta/2)$ + margin before and
      after c, the spine stays within `tol::POINT` of its end tangent line. For a line segment
      this is exact. For a curve with curvature κ, test $\kappa\lambda^2/2 \le$ `tol::POINT`.
      This is OCCT's "the intersection of two pieces of the pipe is connected and planar",
      which "can be violated if the spine is non-linear in some neighborhood of the fracture".
   c. Both segments are longer than λ, so the inner trim stays inside the piece.
   d. No fold near the corner: $\kappa(s)\,(p\cdot N(s)) < 1$ for every profile point at the
      ends of each piece (our derivation; card 1's rail curvature already samples κ).
4. If all hold, build the mitre directly. Each piece's last section is the profile placed on
   the bisector plane by projection along $d_1$ (or $d_2$). For a prism piece, each side face's
   end edge is that side surface ∩ bisector plane: a line for a planar side, an ellipse for a
   cylindrical one. That is exact, the same construction as an extrude to a plane. Both pieces
   then end in the same planar face. Glue them (`path::glue`: the caps cancel, their edges
   become one).
5. If (a)–(c) fail, use today's local booleans. If (d) fails, report the fold before trying
   (the sweep self-intersects).

Round join: the knuckle (the outer half cap turned about $d_1\times d_2$ through c) needs no
boolean where the profile lies wholly on the outside of the turn (ρ ≤ 0 everywhere). Otherwise
keep today's overlap union.

**Data structures.** None new. `Corner` already holds c, $d_1$, $d_2$ and the placements.

**Failure modes.**
- The profile is not square to the path: the cut faces don't coincide. Fall back, or as OCCT
  does, warn that the result is guaranteed only with correction.
- θ near 180°: the bisector plane runs almost along the path, and $\tan(\theta/2)$ blows up.
  MAX_CORNER already refuses past 179°.
- Profiles with holes: every loop is projected the same way; the glue matches loop by loop.

**Fit in cadrs.** `sweep/corner.rs`: `join` (the `Mitre` arm), `local_box` and `end_cap` are
used only on the fallback. `sweep/path.rs`: `sweep_spine` builds the end section on the
bisector instead of square. Prerequisite: none. Card 1's curvature samples feed test (d).

**Sources.**
- OCCT `BRepOffsetAPI_MakePipeShell.hxx` (transition modes and their preconditions): notes only
  (C4). Not fetched.
- Fold condition κ·(p·N) ≥ 1: the notes' derivation (C9), consistent with practitioner rules
  ("path radius smaller than half the width") and US 9,817,924. Not verified from a paper.
- No paper on multi-segment sweep corner joins exists in the research (notes, Gaps).

---

## 8. Parallel Part 21 parsing with ids as table offsets

**Problem.** `step::parse` lexes serially into `HashMap<u64, Record>`. Each record owns a
`Vec<(String, Vec<Param>)>`: one allocation per keyword and per list. `by_kind` is a
`HashMap<String, Vec<u64>>`.

**Why it is fast.** Fast readers tokenise at 0.4–1.7 GB/s single-threaded and 3–5x faster on 8
threads, all self-reported (openbim-step, ifc-lite; web-ifc opens a 70 MB file of 1.1 M
entities in 448 ms on an M1). The techniques are: a byte-class table, `memchr` to skip
strings, borrowed slices, exact-size parameter lists, ids turned into integer offsets, and
splitting at record boundaries. The bench files are at most 3.5 MB, so even 100 MB/s parses
them in 35 ms. Parsing is not where the 2–7 s go. This card is for memory and for very large
assemblies; do it only if card 3's profile shows the lexer.

**Algorithm.**
1. Find the `DATA;` section and its `ENDSEC;` with `memchr`.
2. Split the DATA bytes into k chunks (k = 4 × threads) at $i\cdot L/k$. Move each split point
   forward to a record start: a `;`, optional whitespace or comments, then `#digits` and `=`.
3. Lex each chunk in parallel into a flat arena. Records go into `Vec<RawRecord { id: u64,
   kw: u16, params: Range<u32> }>`, and parameters into `Vec<Param>` with lists as
   `Param::List(Range<u32>)` into the same arena. Keywords are interned through a static table
   of the roughly 150 keywords `step.rs` reads (others get "unknown", kept as a byte span).
   Strings stay as byte spans and are decoded on use (`decode_string`).
4. Check that each chunk ended exactly at the next chunk's start. If one did not (a string
   containing `;\n#12=`), re-lex that pair of chunks serially. That is openbim-step's rule: fall
   back to sequential when a slice does not end exactly at a boundary.
5. Concatenate the chunks in order. Ids: if max id ≤ 4 × count, use a dense `Vec<u32>` from id
   to record index (an id becomes an offset); otherwise a `HashMap`. `by_kind`:
   `Vec<Vec<u32>>` indexed by keyword id. File order is kept, and sorting each list by id
   reproduces today's "ascending".
6. Optional lazy mode: store only (id, kw, byte span) and lex parameters on first `rec(id)`, so
   reading one product of a large assembly touches only its records (STEPcode's lazy loader;
   ruststep's holder/table split).

The output is the same records in the same order as the serial parse, so it is deterministic.

**Data structures.** Byte buffer (borrowed). `Vec<RawRecord>`, `Vec<Param>` arena, `Vec<u32>`
id table, keyword interning table.

**Failure modes.**
- Multi-line strings and comments containing record-like text: step 4 catches them.
- Complex records `#8=(A() B() …);`: the boundary rule is unchanged, since the record still
  starts with `#id=`.
- Ids written out of order or sparse: they hash instead.
- Files with `\X2\` escapes: decoding is lazy but must give today's strings.

**Fit in cadrs.** `step.rs`: `Lexer`, `parse`, `StepFile`, `Record`. `Record::part` and `kind`
change from `String` compares to `u16` compares. The reader functions (`curve`, `surface`,
`edge`, …) change only in signature.

**Sources.** openbim-step release notes, web-ifc, STEPcode, ruststep: notes only
(`notes/exchange_healing_naming.md` §3). All numbers are self-reported by the maintainers; none
was verified. No paper on Part 21 parsing was read.

---

## 9. Approximate skinning on a common knot vector

**Problem.** `loft_build::loft_sampled` samples every section and interpolates them with cubics
on one knot vector, so the pole count equals the sample count. Exact skinning of incompatible
sections merges knot vectors, and nets grow as O(m·n²). Neither bounds the control net by an
accuracy.

**Why it is fast.** It fits all sections on one common knot vector with as few control points
as the tolerance allows, found by binary search. Piegl's page reports a brush handle of 81
sections going from 81×1455 control points (exact skinning) to 81×48 (notes; the papers are
paywalled). Smaller nets make every later operation on the loft's faces cheaper: SSI,
tessellation, mass.

**Algorithm** (Jha 2005, with the knot rule Jha cites).
1. Sample each section $C_k$ ($k=0..K$) at $m+1$ points with chord-length parameters
   $\bar u_{k,i}$ (the existing `sample_sections`). The degree is p = 3.
2. Binary search on the control point count n: $n_{min}=p$, $n_{max}=m+1$.
   Loop: $n = \lfloor(n_{max}+n_{min})/2\rfloor$.
   a. Knot vector per section by averaging. With $d = (m+1)/(n-p+1)$, for $j=1..n-p$:
      $i=\lfloor jd\rfloor$, $\alpha=jd-i$, $u_{p+j} = (1-\alpha)\bar u_{i-1}+\alpha\bar u_i$.
      The ends are clamped (p+1 zeros and ones). Every knot span then holds at least one
      parameter.
   b. Common knot vector: the average of the K+1 section knot vectors.
   c. Fit each section by minimising
      $E = \alpha\!\int\!|C'|^2 + \beta\!\int\!|C''|^2 + \gamma\!\int\!|C'''|^2 + \lambda\sum_i|P_i - C(\bar u_i)|^2$,
      subject to end-point constraints $C(0)=P_0$, $C(1)=P_m$ via Lagrange multipliers. The
      result is a banded linear system per section; the matrix is shared by all sections when
      their parameters coincide.
   d. If every section's max error ≤ ε: $n_{max}=n$, else $n_{min}=n$. Stop when
      $n_{max}-n_{min}\le 1$ and return $n_{max}$'s fit. If even $n=m+1$ fails, interpolate
      exactly (Jha's fallback).
3. Skin across sections: interpolate each control-point column through the section parameters
   $v_k$ (the existing `skin` / `skin_conditioned`, keeping end conditions). Optionally, binary
   search the v count the same way (Jha's ApproximateSkinning).
4. Optional: knot removal on the final surface within ε (Piegl & Tiller 2000; paywalled, notes
   only).

Constants: ε = `tol::LOOP_GAP` × (1 + section size), so the loft's edges still meet within what
`check` accepts. Jha gives no fixed α, β, γ; the paper studies their effect. Start with α = 0,
β = 1e-6·λ·L², γ = 0 (our choice), and test fairness on the loft suites.

**Data structures.** Per section: parameters and samples. One common knot vector. Banded
normal-equation matrices; `nurbs.rs` already has banded interpolation.

**Failure modes.**
- Sections with kinks: insert a knot of multiplicity p at each kink parameter before fitting
  (the same rule as in TiGL's reparametrisation, card 10), or keep exact skinning for such
  sections.
- Wildly different parametrisations across sections: a common averaged knot vector can still
  leave a section badly fitted. The binary search then climbs to $m+1$, so it degrades to
  interpolation; it never gives a wrong surface.
- Fairness terms too large flatten real features. The ε check catches that.

**Fit in cadrs.** `loft_build.rs`: `loft_sampled` (replace its per-section interpolation) and
`make_compatible` (an alternative to knot merging when the merged net would exceed a bound,
say 4× the largest input).

**Sources.**
- `papers/offsets_sweeps_lofts/Jha-2005-energy-based-multiple-refitting-for-skinning.txt`:
  §3 knot selection (eqs. 4–6), §4 MultipleRefit binary search (NCMin = p, NCMax = m+1, common
  knot = average), §5 energy (eq. 7), §6 approximate skinning. Verified; the OCR is poor, so
  check eq. 5's index ranges against Piegl & Tiller's averaging formula.
- Piegl & Tiller 1996/2000/2002 and Park et al.: notes only (D2), described second-hand by Li et
  al. 2015. The 81×1455 → 81×48 figure is from Piegl's web page (notes).

---

## 10. Gordon surfaces for lofts with guide curves

**Problem.** `loft_build` has no guide curves. A loft through N profiles that must also follow M
guides needs one surface interpolating the whole curve network.

**Why it is fast.** Gordon's surface is three linear interpolations and a control-point sum. In
B-spline form it is one surface, with no booleans and no iteration beyond the reparametrisation.
OCCT 8.0 added `GeomFill_Gordon` (notes).

**Algorithm** (Siggel et al. 2019, Algorithm 1).
1. Network check. Every profile starts on the first guide and ends on the last, and every guide
   starts on the first profile and ends on the last (eqs. 4–5). Every profile meets every guide.
   TiGL crashes on networks that break this (issue 987, notes); cadrs must return an error.
2. Intersections: parameters $\tilde u_{l,k}$ on profile k and $\tilde v_{k,l}$ on guide l with
   $\tilde f_k(\tilde u_{l,k}) = \tilde g_l(\tilde v_{k,l})$ (eq. 18). Use curve–curve closest
   points (`ssi` curve–curve hits) and accept within `tol::LOOP_GAP`.
3. Common parameters: $u_l = \frac1N\sum_k \tilde u_{l,k}$ and $v_k = \frac1M\sum_l\tilde v_{k,l}$.
4. Reparametrise each profile with a monotone cubic $\sigma_k$ through $(u_l, \tilde u_{l,k})$
   (eq. 21). Require $\sigma_k' > 0$ on the whole range, or fail ("bijectivity is checked").
5. Approximate $\tilde f_k(\sigma_k(u))$ with n control points, n = the largest pole count among
   the profiles, on one uniform knot vector shared by all profiles. Kinks get knots of
   multiplicity ν (the degree). Use the constrained least-squares (eq. 22), solved through the
   Lagrange system (eq. 23):
   $\begin{bmatrix}\hat N^T\hat N & N^T\\ N & 0\end{bmatrix}\begin{bmatrix}p\\\lambda\end{bmatrix} = \begin{bmatrix}\hat N^T\hat c\\ c\end{bmatrix}$.
   The constraints make each curve pass exactly through its network points
   $\tilde f_k(\sigma_k(u_l))$. Take $n_s > n$ samples ($n_s$ = 4n is our choice; the paper
   only requires $n_s > n$). Do the same for the guides with m control points.
6. Three interpolations, all with the same parameters, degrees and knot vectors in each
   direction:
   - $S_f$ skins the profiles at $v_k$ (eq. 15);
   - $S_g$ skins the guides at $u_l$ (eq. 16);
   - $T$ interpolates the grid $\alpha_{l,k}$ at $(u_l, v_k)$ (eq. 17).
   "The interpolation of the intersection points (17) must use the same interpolation
   parameters, degrees and knot vectors as (15) and (16)."
7. Raise all three to the largest u and v degrees, insert knots until all share one knot vector
   pair, and set $P_s = P_f + P_g - P_T$ (eq. 12, $s = S_f + S_g - T$).
8. Tangent end conditions, if any, go on the two skins $S_f$ and $S_g$ as boundary derivatives,
   as `skin_conditioned` already does for sampled lofts (notes' inference).

**Data structures.** The N×M intersection grid (parameters and points). Per curve its σ and its
reparametrised B-spline. Three `NurbsSurface`s.

**Failure modes.**
- Non-monotone σ when the intersection parameters differ a lot between curves: TiGL errors.
  Do the same, or fall back to sampling more points along σ.
- An incomplete or open network (a profile missing a guide): error before step 2.
- Parametrisation sensitivity: spline Gordon surfaces are "very sensitive to the
  parameterization" (notes, quaoar.su). Chord-length intersection parameters help.
- Exact reparametrisation multiplies degrees (degree 4 ∘ degree 3 = 12). That is why TiGL
  approximates.

**Fit in cadrs.** `loft_build.rs`: a new `loft_guided` next to `loft_with`, reusing `skin`,
`make_compatible`, `NurbsCurve::interpolate` and degree elevation and knot insertion from
`nurbs.rs`. The loft spec needs guides. It depends on card 9's constrained least-squares
helper.

**Sources.**
`papers/offsets_sweeps_lofts/Siggel-et-al-2019-TiGL-Gordon-surface-curve-network-interpolation.txt`:
§3.2 (eqs. 4–8), §3.2.1 (eqs. 9–12), §3.2.2 (eqs. 13–17), §3.2.3 (eqs. 18–23, the
reparametrisation), §3.2.4 Algorithm 1. Verified. TiGL issue 987 and OCCT 8.0's
`GeomFill_Gordon`: notes only.

---

## 11. OCCT's sewing candidate search and merge rules

**Problem.** `surfacing::sew_faces` merges vertices through a BVH. It splits free edges at
T-junctions in a loop of up to 10,000 passes that rebuilds an end-point BVH each pass. It then
merges edges with "same ends and middle". OCCT does the same job in single passes with fixed
sampling rules. STEP import of manifold B-reps does not sew, so this card speeds IGES and the
Sew feature, not the STEP gap.

**Why it is fast.** One spatial index for each phase, built once. Cutting projects all candidate
vertices onto each free bound in one pass. Edge candidates come from shared end nodes through a
map, so there is no geometric search at all, and each candidate is tested with 8 points.

**Algorithm** (OCCT 7.8 `BRepBuilderAPI_Sewing`).
1. Tolerances: τ = max(user tol, 1e-7); min tolerance $\tau_{min}$ = max(1e-4·τ, 1e-7); max
   tolerance unbounded by default.
2. Face analysis: sample each edge at 5 points. If $2\cdot\max$(distance from the chord's
   midpoint) ≤ $\tau_{min}$, the edge is small: glue its ends and make it degenerate. Drop a face
   whose edges are all small.
3. Vertex gluing: put vertex points in a cell grid of cell size τ (`BRepBuilderAPI_CellFilter`).
   Query ±τ boxes, and merge only vertices whose edges may meet (`IsMergedVertices`). The merged
   node takes the average position and tolerance $\max_i(|c - P_i| + tol_i)$.
4. Cutting, in one pass: a box tree of vertex nodes (boxes enlarged by τ/2). For each free bound,
   select the nodes in its curve box enlarged by τ, excluding its own end nodes. Project them
   onto the curve; accept a projection if its distance < τ. Reuse an end node if the projection
   is within max(0.1·distance, $\tau_{min}$); otherwise insert a new vertex of tolerance 1e-7.
   Then split the bound into sections.
5. Candidates: for each section, look up the sections whose two end nodes are the same pair
   (either order) through a node → sections map (`MergedNearestEdges`).
6. Distance test per candidate: sample both at 8 uniform parameters. Compute the max distance
   forward and reversed, and take the smaller as the orientation. Accept if max < τ and at least
   4 of the 8 points project between the reference's ends. Otherwise fall back to point
   projection. Reject sections whose 8-point polyline is shorter than $\tau_{min}$. With several
   candidates, sort by max then min distance and keep mutual best pairs.
7. Merge: the longer edge (by arc length) is the reference and keeps its 3D curve and range. The
   other gives its p-curves, re-ranged to the reference. If the resulting tolerance exceeds τ,
   swap roles and keep the smaller. The last resort samples 23 points:
   $tol = \max(\sqrt{d^2_{max}}(1+10^{-7}), 10^{-7})$. Nullify an edge above the max
   tolerance.
8. Vertex tolerance of a merged pair: with $R_m \ge R_n$, $dR = R_m-R_n$, $D=|P_m-P_n|$. If
   $D\le dR$ keep $(P_m,R_m)$; else $R=(R_m+R_n+D)/2$ and centre
   $=(P_m+P_n-(P_n-P_m)\,dR/D)/2$, the smallest sphere enclosing both.

**Data structures.** A cell grid for vertices; a box tree for nodes; `HashMap<(node, node),
Vec<section>>` for candidates.

**Failure modes.**
- Non-manifold input (an edge shared by three faces): OCCT's manifold mode refuses it ("Sewing
  will not be done in case of multiple edges"). cadrs's "pairs only, closest first" is the same
  policy.
- Closed surfaces: two p-curves on one periodic face. OCCT accepts only if the wrap-around gap
  ≤ the direct gap (`IsMergedClosed`, comparing 2D boxes).
- A tolerance that is too small leaves unsewn borders (OCCT forum: 71-surface IGES). Report free
  edges after sewing.

**Fit in cadrs.** `surfacing.rs`: `sew_faces` (replace the T-junction loop with step 4 and the
edge merge with steps 5–7). `iges.rs`: the trimmed-surface sewing. `xbuild`: tolerance of merged
vertices (step 8) wherever two file vertices are joined (`vertex_at`, `JOIN`).

**Sources.** OCCT `src/BRepBuilderAPI/BRepBuilderAPI_Sewing.cxx` (V7_8_0): `FaceAnalysis` (npt =
5), `VerticesAssembling` / cell filter, `Cutting` (eps = τ·0.5, `BndLib_Add3dCurve` with τ),
`MergedNearestEdges`, `EvaluateDistances` (npt = 8, `nbFound >= npt*0.5`), `SameParameterEdge`
(lengths, nbp = 23), `ComputeToleranceVertex`, defaults at lines 1725–1733. Verified (fetched).
The `CreateCuttingNodes` reuse threshold (0.1·distance) and the 23-point tolerance formula are
from the notes; the surrounding code was seen but those exact lines were not re-read.

---

## Not carded

- **Analytic recognition on import**: the torus case exists (`recognize.rs`). Lukács, Martin &
  Marshall's least-squares fallback (`papers/exchange_healing_naming/Lukacs-1998-faithful-fitting.txt`)
  is for quality, not speed. OCCT does not recognise surfaces while reading STEP (`FromSTEP.exec.op`
  is `FixShape` only), so cadrs's recognition is a cost OCCT does not pay. It only builds its
  121-point grid once a candidate exists; put it in card 3's per-phase profile before deciding
  anything.
- **Parasolid's offset repair rules** (cut swept or spun faces at one new edge, extend
  neighbours, split vertices) and Forsyth 1995: vendor documentation or unread paper. They
  belong with `Move face`'s topology changes, not with speed.
- **Swept volumes of solids** (Peternell et al., Adsul et al., Abdel-Malek et al.): no measured
  gap.
