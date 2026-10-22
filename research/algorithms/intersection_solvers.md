# Intersection, root-finding, projection and distance: algorithm cards for `cadrs_kernel`

Compiled 2026-10-22 from `research/README.md`, `notes/intersection.md`,
`notes/tessellation_mass_queries.md` (projection, distance), `notes/kernels_recent.md` (C1 TIGHT,
C4 Mizrahi–Elber) and the local paper texts. Paths below are relative to `research/`.

"Verified" means the step, formula or number was checked against the local paper text. Two-column
PDFs were re-read with plain `pdftotext` (reading order) because the `-layout` `.txt` interleaves
columns. Equations in Elber–Kim 2001 and Hohmeyer 1992 did not survive extraction (font-encoded or
`formula-not-decoded`); for those the prose is verified and the formula comes from the notes (which
read Hohmeyer from page images) or from a derivation shown on the card. "Derivation" marks a step
that is not in any source and was worked out for this file; each one is short enough to check.

## What the code does today (from doc comments and signatures)

The kernel README (lines 153–220) describes the numeric SSI as contouring a signed distance. The
current `ssi_numeric.rs` module doc describes something closer to the literature:

- Branch finding: each uv box is cut at knots and quarter turns; cell pairs are subdivided while
  their boxes overlap; a pair stops when its two normal cones hold no pair of parallel normals
  (Sederberg–Meyers / Sinha); seeds are edge–surface crossings on cell borders. Cell normal cones
  are circular, built from 3 × 3 sampled normals widened by 3/4 of the largest neighbour angle
  (`cell`, `separated`, `CONE_MARGIN = 1e-3`, `PAIR_BUDGET = 60_000`, `TINY = 2e-3`).
- Tracing: march along $n_a \times n_b$ with Newton on four parameters, adaptive step
  (`h_max = size/40`, `h_min = size·1e-7`); tangent points chained separately (`TANGENT_SIN = 1e-7`,
  `TRANSVERSAL_SIN = 1e-3`).
- Fitting: `ssi::fit_curve` interpolates a cubic B-spline through exact points and inserts points
  where it strays more than 1e-9. `IntCurve` carries only the 3D curve; consumers
  (`boolean.rs:1570`, `:1618`, `fillet::face_pcurve`) rebuild p-curves with `pcurve::pcurve`, which
  projects samples onto the surface and refines to 1e-10 (`pcurve::fitted`).
- Projection: analytic surfaces in closed form; extrusions and NURBS by a 24 × 24 grid plus Newton
  (`Surface::project_grid`).
- Distance: best-first branch and bound on cell pairs with `patch_bbox` boxes (a NURBS cell's box
  is its control hull's box; others are sampled and grown by $h^2/8\,|S''|$), cells split down to
  1/50 of a piece, then Levenberg–Marquardt damped Newton (`distance::polish`).

The cards below target exactly these places.

## Ranked summary

Payoff is an estimate; nothing here has been profiled (the synthesis report's item 1 still
applies). Size: small is days, medium is one to three weeks, large is more than a month.

| Rank | Card | Payoff | Size | Risk |
|---|---|---|---|---|
| 1 | [Fit the 3D curve and both p-curves in one solve](#1-fit-the-3d-curve-and-both-p-curves-in-one-solve) | Removes the per-edge p-curve projection for every numeric SSI edge | Small | Low |
| 2 | [Gauss-map plane separability with polyhedral normal bounds](#2-gauss-map-plane-separability-with-polyhedral-normal-bounds) | Fewer cell pairs (cylinders and tori pass at any crossing angle); gives the sweep direction for card 5 | Small–medium | Low–medium |
| 3 | [Shared Bernstein/B-spline box solver](#3-shared-bernsteinb-spline-box-solver) | Enabler for cards 4, 6, 7, 9; replaces ad hoc subdivision in curve/curve and curve/surface | Medium–large | Medium |
| 4 | [Certified analytic × B-spline intersection](#4-certified-analytic--b-spline-intersection-regular-cells-and-characteristic-points) | No missed loops or touching points in the commonest boolean pair; regular cells need no sampling | Medium | Medium |
| 5 | [Plane-sweep tracing, tangential direction, step control](#5-plane-sweep-tracing-tangential-direction-step-control) | No step-size heuristics on transversal branches; correct direction on tangent branches | Medium | Medium |
| 6 | [Point projection on NURBS: span pruning and a second-order step](#6-point-projection-on-nurbs-span-pruning-and-a-second-order-step) | Cold projections without a 576-point grid; fewer wrong basins | Medium | Low |
| 7 | [LUB-tree minimum distance with hull bounds and normal-cone pruning](#7-lub-tree-minimum-distance-with-hull-bounds-and-normal-cone-pruning) | Tighter lower bounds, fewer Newton launches in `distance` | Medium | Low |
| 8 | [Rounded-interval coefficients for exclusion tests](#8-rounded-interval-coefficients-for-exclusion-tests) | Turns cards 2–4 and 7's pruning into a guarantee | Small–medium | Low |
| 9 | [Exact extremum events for maximum and Hausdorff distance](#9-exact-extremum-events-for-maximum-and-hausdorff-distance) | Exact `max_distance`; slow, so a fallback or check | Medium | Medium |

Dependencies: 2 → 5 (the sweep direction $P$); 3 → 4, 6, 7, 9; 2 and 7 share the normal-bound code
and a convex-hull distance routine (Gilbert/GJK); 8 hardens 3. Card 1 is independent.

---

## 1. Fit the 3D curve and both p-curves in one solve

**Problem.** Numeric SSI curves leave `ssi` as a 3D B-spline only, so each consumer re-derives
both p-curves by projection, although the tracer already had both surfaces' (u, v) at every point.

**Why it is fast.** The interpolation matrix depends only on the parameters and knots, so one
banded factorisation (O(n), already in `nurbs`) serves 3 + 2 + 2 right-hand-side columns. No
projections at all for SSI edges. The p-curves also come out with the same parameter as the 3D
curve, which is what the topology wants. OCCT does this in `GeomInt_IntSS` /
`GeomInt_WLApprox`: 3D curve and both 2D curves from one multi-curve least-squares fit,
`SetParameters(myTolApprox = 1e-7, tol2d, degmin 4, degmax 8, 0, 30 iterations, …)`. No timing is
published; OCCT's speed on SSI is attributed partly to this single pass (notes, entry 10).

**Algorithm.** Input: the traced points $Pt_i = (p_i, a_i, b_i)$, $i = 0..m$, each exact on both
surfaces (`ssi_numeric::solve`).

1. Unwrap periodic parameters. For each periodic direction with period $T$, add $kT$ so that
   $|a_i - a_{i-1}| < T/2$ (OCCT shifts by a period on jumps of at least half a period; notes,
   entry 10). At a pole or apex the free coordinate is undefined: take it from the neighbouring
   point (`pcurve::settle_poles` does this already).
2. Parameters $t_i$: chord length of the 3D points (`ssi::chord`), shared by all seven components.
3. Knots: as `NurbsCurve::interpolate` (degree 3), or `interpolate_periodic` for closed curves.
4. Solve $N X = [\,p \mid a \mid b\,]$, an $(m+1) \times 7$ right-hand side, with one factorisation
   of the collocation matrix $N_{ij} = N_{j,3}(t_i)$.
5. A closed 3D curve whose uv winds once round a period (a loop round a cylinder: $u$ grows by
   $2\pi k$) is not periodic in uv. Fit $\tilde u(t) = u(t) - 2\pi k\,(t - t_0)/(t_1 - t_0)$ as
   periodic, convert to the clamped form, and add the linear term back (a degree-1 polynomial lies
   in every clamped spline space of degree ≥ 1). (Derivation.)
6. Check at the midpoints $t_{i+1/2}$ of every span three errors: $|C(t) - X^*(t)|$ against the
   intersection (as `fit_curve` does now: a point refined onto both surfaces on the plane square
   to the chord), $|C(t) - S_a(c_a(t))|$ and $|C(t) - S_b(c_b(t))|$. Where any exceeds the
   tolerance (cadrs's 1e-9 relative, `fit_curve`; OCCT uses 1e-7), insert the refined point (it
   has all seven components) and refit. A uv error is measured in 3D, through the surface.
7. Return the 3D curve with both p-curves on the same $[t_0, t_1]$.

**Data structures.** A generic-dimension B-spline interpolation (or three calls sharing one LU);
`IntCurve` gains `pc: Option<[Curve; 2]>` (2D curves with z = 0, as `pcurve` makes them).

**Failure modes.** OCCT falls back to a degree-1 B-spline through the points when the fit fails
(`MakeBSpline`/`MakeBSpline2d`, index knots); missing p-curves come from projection with a larger
tolerance (notes, entry 10). The p-curves can need more points than the 3D curve (high uv
curvature near a pole): step 6's max over the three errors handles it. Points from tangent chains
(`ssi::fit_points`) carry no exact uv refinement; keep projection for them.

**Fit in cadrs.** `ssi::fit_curve` (`ssi.rs:1051`) and `ssi::numeric` build the 2D curves; the
consumers in `boolean.rs:1570`, `:1618` and `fillet::face_pcurve` use them when the face's surface
is `a` or `b` of that SSI and call `pcurve::pcurve` otherwise. Exact p-curves (`pcurve::exact`,
planes and rims) stay preferred where they apply. Shares the ordered points of card 5.

**Sources.** `notes/intersection.md` entry 10 and the "What OpenCASCADE … does" section
(`GeomInt_IntSS_1.cxx`, read from OCCT source by the researcher); `README.md` "Intersection
gains…". Not verified: there is no paper, and the ApproxInt internals were not read (notes say
they were blocked). Steps 5 and 6's three-error check are derivations.

---

## 2. Gauss-map plane separability with polyhedral normal bounds

**Problem.** The pair recursion in `ssi_numeric` stops only when two circular normal cones are
disjoint. Cylinders, cones and tori whose normal sets are thin arcs keep failing that test and get
subdivided far more than needed.

**Why it is fast.** Hohmeyer's criterion is the strongest possible given convex normal bounds (by
Farkas' lemma, failing it means some surfaces with the same bounds do have a loop). On two
cylinders, the cone criteria of Sederberg, Sinha, de Montaudouin and Kriezis "must be subdivided
arbitrarily small", while this one "will pass … for any angular separation" (Hohmeyer §3.3,
verified). For circular cones the criterion is identical to cadrs's current `separated`; the gain
comes only from non-circular bounds, which are exact and cheap for analytic cells. Measured, on
two quarter-cone Bézier patches meeting at a parameter $d$: running time grows as $1/\sqrt d$
(§4.4.5, verified). Spatial separation by a separating-plane LP after the box test needs
sub-patches of size about $\sqrt\delta$ at separation $\delta$, O(1/δ) work, against
O(1/δ²) for boxes alone; adding the "bounding planes" test halves the time again (§4.2.4,
verified). Krishnan & Manocha report 8 levels of subdivision for Hohmeyer's criterion on their
loop example (GI '96 §1.1, verified).

**Algorithm.**

*Criterion* (Hohmeyer Theorem 4; prose verified, inequalities from the notes). Let $N_1, N_2$
bound the oriented normals of two $C^1$ surface pieces. If there are $P_1, P_2$ with
$P_1\cdot n_1 > 0,\ P_1\cdot n_2 < 0,\ P_2\cdot n_1 > 0,\ P_2\cdot n_2 > 0$ for all
$n_1 \in N_1, n_2 \in N_2$, then the intersection has no closed loop, no singular point, and
isolated points only on the piece boundaries. With $P = P_1 \times P_2$ the curve tangent
$T = n_1 \times n_2$ satisfies $T\cdot P > 0$ everywhere, since by Binet–Cauchy
$(n_1\times n_2)\cdot(P_1\times P_2) = (n_1\cdot P_1)(n_2\cdot P_2) - (n_1\cdot P_2)(n_2\cdot P_1) > 0$
(checked algebraically). Points on the curve are ordered by $x\cdot P$. If the piece boundaries
meet the other surface in exactly two points, there is exactly one curve and those are its ends
(§3.2, verified).

*Test as a min-norm problem* (derivation). Let $W_1 = N_1 \cup (-N_2)$ and $W_2 = N_1 \cup N_2$
(finite sets of unit generators). For a finite set $W$, $\max_{|P|=1}\min_{w\in W} P\cdot w$ equals
the distance from the origin to $\mathrm{conv}(W)$ when that distance is positive, attained at
$P = z/|z|$ where $z$ is the min-norm point of $\mathrm{conv}(W)$. So:

1. $z_1 = \text{minnorm}(\mathrm{conv}\,W_1)$, $z_2 = \text{minnorm}(\mathrm{conv}\,W_2)$ (Gilbert /
   GJK on ≤ ~12 points; the same routine card 7 needs for hull distances).
2. Pass if $|z_1| > m$ and $|z_2| > m$, where $m$ covers how far true normals may stray from the
   generators. For exact generators $m$ = a rounding margin (cadrs's `CONE_MARGIN = 1e-3` is a
   reasonable start). For sampled generators widened by angle $\delta$ (today's cells),
   $m = 2\sin(\delta/2)$, since a unit $n$ within $\delta$ of a generator $w$ has
   $P\cdot n \ge P\cdot w - |n - w|$.
3. Then $P_1 = z_1, P_2 = z_2, P = P_1 \times P_2$.

Hohmeyer solves the same thing as two LPs with Seidel's algorithm, run interleaved and stopped as
soon as one is infeasible: "a factor of 2 speedup can typically be obtained for criterion
failures" (§4.4.4, verified). With GJK, run the cheaper failing case first in the same spirit.

*Normal generators per cell* (Hohmeyer §4.4.1 only says quadrics and tori are done "case by case";
the sets below are derivations). The cone spanned by an arc of a great circle shorter than $\pi$ is
spanned by its two end points.
- Plane: $\{n\}$.
- Cylinder, $u \in [u_0, u_1]$, $u_1 - u_0 < \pi$ (cells are cut at quarter turns already):
  $\{\hat r(u_0), \hat r(u_1)\}$ with $\hat r(u) = \cos u\,x + \sin u\,y$, any $v$ range.
- Cone of half-angle $\beta$: normals $n(u) = \cos\beta\,\hat r(u) - \sin\beta\,z$ lie on a small
  circle; bound the arc by $\{n(u_0), n(u_1), \cos\beta\,\hat r(u_m)/\cos(\Delta u/2) - \sin\beta\,z\}$,
  $u_m$ the mid angle (the third point is where the arc's end tangents meet in the plane
  $z = -\sin\beta$).
- Sphere and torus over a $(u, v)$ box: $n = \cos v\,\hat r(u) + \sin v\,z$; use the two
  parallels $v_0, v_1$ each bounded as the cone case (end points and tangent-meet point), six
  generators.
- NURBS: the control points of the pseudo-normal $N = S_u \times S_v$, computed once per root patch
  and subdivided with it (§4.4.2, verified: "compute this surface once for the root surface and
  then compute the pseudo-normal surfaces for any sub-surfaces by subdividing"). Degree for a
  polynomial B-spline of degree $m \times n$: $(2m-1)\times(2n-1)$, knot multiplicities
  $\mu' = \mu + m$, $\nu' = \nu + n$; for a rational patch $3m-1 \times 3n-1$ (81 control points
  for a rational bicubic) (§4.4.2, verified).
- Extrusion along $d$ of a curve $c$: $n \parallel c'(t) \times d$; generators $c'_k \times d$ from
  the control points of the hodograph of the curve's cell.

*Recursion* (Hohmeyer §3.5.1 `Intersect`, verified):

```
intersect(A, B):
  if spatially separable(A, B): return            # AABB, then bounding planes, then separating plane
  if gauss_separable(A, B) -> P: simple pair: border seeds; trace with P (card 5)
  else if A and B within eps of a point and normals within eps of one normal: report singular point
  else: split both, recurse on all pairs
```

Theorem 5 (verified): this terminates in no-loop sub-problems or finds a singularity. Spatial
tests in order (§4.2, verified): boxes; then "bounding planes" with normal $(C - A)\times(D - B)$
from the corner control points (cadrs's `apart` slabs along the cell normal axes play this
role); then a separating plane between the two hulls (Gilbert distance between hulls > 0).
After a split at a point $q$ where the surfaces share a normal, the separating plane must contain
$q$ and has one degree of freedom left, which "becomes much faster" (§4.4.6, verified).

Singular-point candidates for B-spline × B-spline (collinear normal points, Hohmeyer §6.1): four
equations in $(s,t,u,v)$,
$F_s\times F_t\cdot G_u = 0,\ F_s\times F_t\cdot G_v = 0,\ (F - G)\cdot F_s = 0,\ (F - G)\cdot F_t = 0$,
a superset of the singular points; Theorem 7 (prose verified, equations from notes): the Jacobian
is non-singular at node singularities and at isolated intersections where the surfaces do not
share the same fundamental form. Solve with card 3 or Newton from the small pair's centre
(`ssi_numeric::singular_point` exists).

**Data structures.** Per cell: a small vector of normal generators plus a margin (replacing
`axis`/`half`); per NURBS root patch: the pseudo-normal Bézier/B-spline patch; a GJK min-norm and
hull-distance routine.

**Failure modes.** Near tangency the normals are nearly parallel and no criterion passes; the
recursion then goes to the singular-point branch (`TINY`). Sampled generators are not rigorous
bounds; the exact analytic sets and the pseudo-normal hull are. The pseudo-normal hull
"overestimate[s] the pseudo-normal surface considerably" for high degrees; Hohmeyer suggests
Sederberg's bounds on $S_u$, $S_v$ (degree one less) as the fix (ch. 8, verified). Gauss-map
methods "may not work well if the intersection curve is self-intersecting" (Krishnan–Manocha,
verified).

**Fit in cadrs.** `ssi_numeric::cell` (generators), `separated` → `separable(a, b) -> Option<V3>`
returning $P$; `search` uses it; `PAIR_BUDGET` is hit less often. Shared with card 5 (needs $P$),
card 7 (normal bounds for pruning) and `silhouette`.

**Sources.** `papers/intersection/Hohmeyer-1992-robust-efficient-surface-intersection-thesis.md`
§3.2 Lemma 1 and Theorem 4, §3.3, §3.5.1 Theorem 5, §4.2.1–4.2.4, §4.4.1–4.4.6, §6.1 Theorem 7
(prose verified; formulas not decoded in the OCR, taken from `notes/intersection.md` entry 1);
`papers/intersection/Krishnan-Manocha-1996-algebraic-loop-detection-GI.txt` §1.1 (verified). The
min-norm reformulation and the analytic generator sets are derivations.

---

## 3. Shared Bernstein/B-spline box solver

**Problem.** Curve/curve (`ssi_cci`), curve/surface (`ssi::curve_surface`), characteristic points,
projection and distance extrema each find roots of small polynomial systems on a box, today with
sampling plus Newton. One solver that cannot skip a root (modulo card 8) serves them all.

**Why it is fast.** Sub-boxes where any equation's coefficients have one sign are discarded at a
cost of one pass over the coefficients, so "many subdomains are … eliminated early" (Elber–Kim
§1, verified). Domain reduction by projected envelopes converges quadratically at simple roots
(Mourrain–Pavone, via Fünfzig §7, verified). Preconditioning by $J(m)^{-1}$ makes double roots
cheap: on two touching circles the range-reduction factor per step was 1024 against about 3.3 for
simple roots (Fünfzig Table 1, verified). The single-solution test stops subdivision early and
hands each isolated root to Newton. Limits: the tensor Bernstein basis has $\prod (d_i + 1)$
coefficients, so these solvers are "impractical for systems with more than n = 6 or 7 variables"
(Fünfzig abstract, verified); use it for 1–4 unknowns.

**Algorithm.** Input: equations $f_1..f_k$ in $n \le 4$ unknowns on a box $D$, each a tensor of
Bernstein coefficients (split B-splines into Bézier pieces at knots first, or refine at the
midpoint as Elber–Kim do), $k = n$ for isolated roots, $k = n - 1$ for curves; optional
inequality constraints $g > 0$.

Build coefficients in Bernstein form directly (control points, de Casteljau, products via
$B^m_i B^n_j = \frac{\binom mi \binom nj}{\binom{m+n}{i+j}} B^{m+n}_{i+j}$, a standard identity), never
through the power basis: the conversion is ill-conditioned (Patrikalakis–Maekawa node42, notes).

Per box, in a work queue (Fünfzig Fig. 1, verified):

1. **Exclude.** If some $f_i$ has all coefficients $> 0$ or all $< 0$, drop the box (Elber–Kim
   §1 and Alg. 2.1, verified). For an inequality $g > 0$, drop when all of $g$'s coefficients are
   $\le 0$.
2. **Precondition** ($k = n$). Evaluate $J(m)$ at the box centre; if its condition number is
   moderate, replace $f$ by $J(m)^{-1} f$ (a linear combination of coefficient tensors; same
   roots; Jacobian ≈ identity at $m$) (Fünfzig §7, verified; Mourrain–Pavone 2009 via Fünfzig).
3. **Reduce.** For each variable $x_j$ and each $f_i$: the lower envelope coefficients
   $\underline b_{i_j} = \min_{i_k, k\ne j} b_{i_1..i_n}$ and upper $\overline b_{i_j} = \max(\cdot)$
   define univariate Bernstein polynomials that bound $f_i$ from below and above on the box. A
   root needs $\underline b(x_j) \le 0 \le \overline b(x_j)$. Cheap version (Projected Polyhedron):
   intersect the convex hull of the points $(i_j/d_j, b)$ with the axis. Tighter version
   (Mourrain–Pavone): first and last roots of the two envelope polynomials. Intersect the
   intervals over all $i$; empty → drop (Fünfzig §4, §7, verified). Repeat while a pass shrinks
   the box enough: Fünfzig's LP variant repeated "as long as reduction factor > 2.0" (Table 1,
   verified).
4. **Single-solution test** ($k = n$).
   - $n = 1$: count sign variations $V$ of the coefficients. The number of roots in the open
     interval is $\le V$ and has the parity of $V$ (Descartes in the Bernstein basis, Liang et al.
     Theorem 1, verified). $V = 0$: drop. $V = 1$: exactly one root: bracketed Newton (bisection
     fallback).
   - $n = 2$: Elber–Kim cone test. For each $f_i$ build the gradient coefficient vectors
     $(\partial_u f, \partial_v f)$ after elevating both partials to a common degree; cone axis
     $a_i$ = their average, half-angle $\alpha_i$ = the largest angle to the axis (§2.4, prose
     verified). If the two double-sided cones are disjoint,
     $\alpha_1 + \alpha_2 < \angle(a_1, a_2) < \pi - \alpha_1 - \alpha_2$, the tangent cones meet
     only at the origin and there is at most one common root (Theorem 1 in §2.4: "at most one
     common solution", prose verified).
   - $n = 3, 4$: the general cone test needs Hanniel–Elber's dual-hyperplane LP (not local).
     Cheap sufficient test instead (derivation, mean-value theorem): bound every entry of
     $A\,J(x)$, $A = J(m)^{-1}$, over the box from the Bernstein coefficients of the partials; if
     every matrix in that interval matrix is strictly diagonally dominant, $A J$ is non-singular
     on the box, so $f(x) = f(y)$ forces $x = y$ and there is at most one root.
5. **Newton** when the test passes, or when the box is below the size tolerance: Newton from the
   centre; Elber–Kim's geometric step (intersection of the tangent hyperplanes of the graphs) is
   ordinary Newton for $k = n$ and the minimum-norm (QR) solution for $k < n$ (§2.2, verified).
   Accept a root that converges inside the box plus a margin; otherwise keep subdividing. Merge
   roots closer than the tolerance.
6. **Split** along the longest side at the middle (Fünfzig Fig. 1; Elber–Kim subdivide at the
   midpoint), de Casteljau along that axis, and queue both halves.
7. A box that reaches the size tolerance without passing step 4 holds a multiple root, a tangency
   or a cluster: report it as such (centre plus box), do not guess.

Constants from the sources: Fünfzig stop all sides $< \delta = 10^{-3}$ on a $[-10, 10]^2$ start
box (Table 1); Mizrahi–Elber, for a unit-size problem, subdivision tolerance $10^{-3}$, Newton
tolerance $10^{-10}$, tracing step $10^{-4}$, end-point merge $10^{-5}$ (Remark 1, verified). For
cadrs: subdivision tolerance a fraction of the box (start at $10^{-3}$), Newton to
`tol::ROUNDOFF` relative to size.

**Data structures.** `BernTensor { dims: [usize; 4], coef: Vec<f64> }` per equation (or an `[f64]`
slice with strides), its box, derivative tensors (cheap: differences of neighbours times degree),
de Casteljau split along one axis in place; a queue (depth-first keeps memory small; boxes are
independent, so rayon over the queue works). Sizes: a quadric substituted into a bicubic patch
has degree 6 × 6, 49 coefficients; a torus, 12 × 12, 169; the 4-variable difference of two bicubic
patches, 256 per coordinate.

**Failure modes.** Tangential and multiple roots: slow without preconditioning (Fünfzig §7,
verified); with it the double root of system (c) was handled in 8 reductions and 1 bisection
(Table 1). Rounding: Fünfzig §5 (verified) shows how a rounding error $\varepsilon$ turns the line
$y - x = 0$ into $y - x = \varepsilon$ and loses both roots of $x^2 - x$; their conservative fix is
to push each hyperplane outward by the accumulated error (card 8). Positive-dimensional solution
sets (curves, overlaps) make a zero-dimensional solver return chains of boxes; send those to
cards 2/4/5. Patrikalakis–Maekawa: PP "tends to be extremely time consuming in the case of
tangential, as well as higher order intersections" (via Mukundan 2004 §1, verified).

**Fit in cadrs.** New module `native/bern.rs` (Bézier extraction and products may live in
`nurbs.rs`). First users: `ssi::curve_surface` (n = 1 per curve span against an implicit
surface; n = 3 against a NURBS), `ssi_cci` (n = 2), card 4's border, turning and singular points,
card 6's projection equations, card 9's antipodal systems, `silhouette`'s $n\cdot w = 0$.

**Sources.** `papers/intersection/Elber-Kim-2001-geometric-constraint-solver-multivariate-splines.txt`
§1, §2.1 Alg. 2.1, §2.2, §2.4 (prose verified; equations lost in extraction);
`papers/intersection/Fuenfzig-Michelucci-Foufou-2010-tensorial-Bernstein-solvers.txt` abstract,
§1 Fig. 1, §2, §4, §5, §7 Table 1 (verified);
`papers/intersection/Liang-Mourrain-Pavone-2008-subdivision-topology-implicit-curves.txt` §2.1
Theorem 1, §2.2 (verified); `papers/kernels_recent/Mizrahi-Elber-Topologically-guaranteed-bivariate-solver.txt`
§3 and Remark 1 (verified). Mourrain–Pavone 2009 itself is not local. The $n = 3, 4$ dominance test
is a derivation.

---

## 4. Certified analytic × B-spline intersection (regular cells and characteristic points)

**Problem.** For a plane, quadric or torus against a B-spline patch, a sampled search can miss a
small loop or a touching point between samples. The lattice-method verdict: grid resolution "can
cause the method to miss … small loops and isolated points, which arise near or at tangency"
(Patrikalakis–Maekawa node107, notes).

**Why it is fast.** The implicit equation of the analytic surface substituted into the patch is a
polynomial $f(u, v)$ in Bernstein form, so every test is a sign check on a few dozen
coefficients. Cells where $f$ has one sign drop out; cells where $\partial_u f$ or $\partial_v f$
has one sign are certified and need only their border roots, no interior sampling. Liang et al.
handle curves "of total degree about 80 with coefficients of bit-size 200, which resultant-based
techniques are not able to treat" (§1, verified). The 2 × 2 turning-point system is "the cheapest
rigorous loop guarantee available" for this pair class (notes, entry 3).

**Algorithm.**

*Build $f$.* Transform the patch's homogeneous control points $H = (X, Y, Z, W)$ into the analytic
surface's frame. Then, per Bézier patch, with products from card 3:
- plane $z = 0$: $f = Z$, degree $(p, q)$;
- quadric $x^\top Q x = 0$: $f = H^\top \hat Q H$ ($\hat Q$ the 4 × 4 homogeneous form), degree
  $(2p, 2q)$;
- torus (major $R$, minor $r$, axis $z$): $f = (X^2 + Y^2 + Z^2 + (R^2 - r^2)W^2)^2 - 4R^2 W^2 (X^2 + Y^2)$,
  degree $(4p, 4q)$.

The sign of $f$ is the sign of the implicit function because $W > 0$ and the powers of $W$ are
even (derivation). $f$ is not a distance; tolerances on $f$ are scaled by a lower bound of
$|\nabla f|$.

*Certified cells* (Liang–Mourrain–Pavone §3.2–3.3, verified):

1. Start from the patch's uv box. For a cell, if $f$'s coefficients have one sign: empty, drop.
2. If $\partial_v f$'s coefficients have one sign (the cell is x-regular) or $\partial_u f$'s do
   (y-regular): the curve in the cell has no singular point and no turning point in that
   direction, and its topology is fixed by its border points (Proposition 1).
   - Border points: roots of $f$ on the four edges; an edge's coefficients are the outer
     rows/columns of the tensor (verified: "the coefficients of a polynomial, with a minimal …
     indices are its Bernstein coefficients on the corresponding face"). Solve each with card 3,
     $n = 1$.
   - Connect: sort the border points by $u$ (x-regular) or by $v$ (y-regular), count a point twice
     when the curve is tangent to the border there (even multiplicity), and join
     $(p_1, p_2), (p_3, p_4), \dots$ (Proposition 1, verified). Each pair is one branch.
3. Otherwise split the cell (quadtree, midpoints); share edge roots between neighbours (§3.3,
   verified: computed once per face).
4. A cell below the threshold $\varepsilon$ that is still not regular holds a singular point; its
   topology is not certified (verified: "we do not certify the topology at singular points").

*Characteristic points* (seeds, when the existing marcher is kept instead of step 2's join):
- Border points: as above.
- Turning points: roots of $\{f = 0, \partial_v f = 0\}$ (2 × 2, card 3). Along $f = 0$ the
  tangent is $\propto (\partial_v f, -\partial_u f)$, so these are where $u$ is extremal on the
  curve. Every closed loop has at least two (Krishnan–Manocha Lemma 1 and its proof: "every loop
  component must have at least two turning points", verified; Patrikalakis–Maekawa node103/105,
  notes).
- Singular points: $\{f = \partial_u f = \partial_v f = 0\}$, three equations in two unknowns:
  solve $\{\partial_u f, \partial_v f\}$ with $f$ as an extra exclusion test, then check $|f|$.
  Classify by the Hessian quadratic $f_{uu}a^2 + 2f_{uv}ab + f_{vv}b^2 = 0$: $f_{uv}^2 - f_{uu}f_{vv} > 0$
  two branches cross, $= 0$ cusp or higher contact, $< 0$ isolated touching point
  (Patrikalakis–Maekawa node104, notes).
- Marching from all border and turning points reaches every branch; split the uv box at the
  $u$ of every turning and singular point and each strip holds only monotone arcs.

Then refine each branch's points onto both surfaces (existing `solve`) and fit (card 1).

**Data structures.** Per Bézier patch: tensors for $f$, $\partial_u f$, $\partial_v f$ (degrees as
above); a quadtree of cells with shared edge-root lists; per regular cell its sorted border points.

**Failure modes.** A tangential intersection, $f \ge 0$ with $f = \nabla f = 0$ along a whole
curve, is never certified: the quadtree goes to $\varepsilon$ all along it. This is the fillet case
(a fillet surface tangent to its neighbour face). Detect it up front: border points where $f$ and
$\nabla f$ both vanish, or the boolean already knowing the faces share that edge (synthesis plan
item 8, "skip F-F intersection for face pairs whose contact is already common edges"), and send
those to the tangent-chain path (`ssi_numeric` stage 3). Overlapping (coincident) pieces make
$f \equiv 0$ on a cell: all coefficients zero; route to `ssi_coincident`. Liang et al. assume no
common factor and a square-free resultant (§3, verified). Rounding: card 8.

**Fit in cadrs.** `ssi_numeric` stage 1 for pairs where one `Dom` has an exact implicit form
(`off_exact`) and the other is a NURBS (extrusions and revolutions of NURBS curves convert
exactly to NURBS surfaces). Replaces cell-pair subdivision and the Sederberg–Meyers stop for those
pairs; the marcher, tangent handling and fit stay. `ssi::curve_surface` gains the same exactness
for a NURBS curve against an implicit surface ($f(t)$, $n = 1$). Needs card 3; card 8 makes it a
guarantee.

**Sources.** `papers/intersection/Liang-Mourrain-Pavone-2008-subdivision-topology-implicit-curves.txt`
§1, §2.1 Theorem 1, §3.2 Proposition 1 and the connection rule, §3.3 (verified);
`papers/intersection/Krishnan-Manocha-1996-algebraic-loop-detection-GI.txt` Definition 1,
Lemma 1 (verified); Patrikalakis–Maekawa hyperbook node103–105, node107 (`notes/intersection.md`
entry 2; HTML, no local copy, not verified). The degree formulas for $f$ and the sign argument are
derivations.

---

## 5. Plane-sweep tracing, tangential direction, step control

**Problem.** Marching needs step-size heuristics: too large a step strays to another branch or
loops back, worst "at constrictions where the gradient is small" (node102, notes). Along tangent
branches $n_a \times n_b$ vanishes and gives no direction.

**Why it is fast.** In a pair that passed card 2 there is exactly one curve, monotone in $x\cdot P$,
so each sweep plane $x\cdot P = w$ cuts it once: Newton from the previous point cannot jump
branches, the points come out ordered, and "the value δw does not need to be chosen carefully"
(Hohmeyer §3.4, verified). Step size then only controls sampling density, which card 1's error
check already does. No closure test is needed (the pair has no loop).

**Algorithm.**

*Transversal pair with $P$ (Hohmeyer §3.4, verified).* Ends $p_0, p_1$ are the pair's two border
seeds; $w_0 = P\cdot p_0$, $w_1 = P\cdot p_1$.

```
for w = w0 + δw, w0 + 2δw, ..., w1:
    solve for y = (s, t, u, v):
        F(s,t) - G(u,v) = 0          (3 equations)
        P·F(s,t) - w   = 0           (1 equation)
    Jacobian (4×4):  [ F_s  F_t  -G_u  -G_v ]
                     [ P·F_s  P·F_t  0   0 ]
    start: previous y plus δw/(P·T) times the parameter rates of T (below)
    if Newton fails: halve δw and retry (verified: "one can simply reduce δw until it does")
```

For analytic × B-spline the same with two unknowns: $f(u, v) = 0$, $P\cdot S(u, v) = w$, Jacobian
$\begin{pmatrix} f_u & f_v \\ P\cdot S_u & P\cdot S_v \end{pmatrix}$ (derivation; Hohmeyer's criterion
makes no assumption on the surface form, §3.3, verified). Choose $\delta w = (w_1 - w_0)/N$ with
$N$ from the existing chord-deflection rule; card 1 adds points where the fit needs them.

*Parameter rates along a direction $c$* (Mukundan 2004 eq. 2, verified by derivation from
$c = \sigma' P_\sigma + t' P_t$; the text's layout is scrambled):
$\sigma' = \det(c, P_t, N_P)/(N_P\cdot N_P)$, $t' = \det(P_\sigma, c, N_P)/(N_P\cdot N_P)$,
$u' = \det(c, Q_v, N_Q)/(N_Q\cdot N_Q)$, $v' = \det(Q_u, c, N_Q)/(N_Q\cdot N_Q)$, with
$N_P = P_\sigma\times P_t$, $N_Q = Q_u\times Q_v$. Transversal direction
$c = \pm (N_P\times N_Q)/|N_P\times N_Q|$ (eq. 3, verified).

*Tangential branches (normals parallel)* (Mukundan §2.2, verified): the direction satisfies
$b_{11}\sigma'^2 + 2b_{12}\sigma' t' + b_{22}t'^2 = 0$. With $d = b_{12}^2 - b_{11}b_{22}$:
- $d < 0$: isolated tangential contact point;
- $d > 0$: branching, the direction is not unique (two crossing branches);
- $d = 0$ and all $b_{ij} = 0$: contact of order ≥ 2, the method cannot decide;
- $d = 0$ otherwise: one direction. If $b_{11} \ne 0$: $\nu = -b_{12}/b_{11}$,
  $c = (\nu P_\sigma + P_t)/|\nu P_\sigma + P_t|$ (eq. 5); else if $b_{22} \ne 0$:
  $\mu = -b_{12}/b_{22}$, $c = (P_\sigma + \mu P_t)/|\cdot|$ (eq. 6).

Mukundan gives the $b_{ij}$ only by reference. Derivation (equal normal curvatures, as OCCT's
`ComputeContactDirection` uses "the difference of the two second fundamental forms", notes): with
the common unit normal $n$, $L_P = \begin{pmatrix} P_{\sigma\sigma}\cdot n & P_{\sigma t}\cdot n \\
P_{\sigma t}\cdot n & P_{tt}\cdot n \end{pmatrix}$, $L_Q$ likewise for $Q$, $J_P = [P_\sigma\ P_t]$,
$J_Q = [Q_u\ Q_v]$, $M = (J_Q^\top J_Q)^{-1}J_Q^\top J_P$ (maps $(\sigma', t')$ to $(u', v')$), then
$B = L_P - M^\top L_Q M$.

*Step control where card 2 does not pass* (keep the current marcher; OCCT constants from source,
notes entry 10, not verified against a paper): reject a step when the cosine between successive
3D directions is below 0.98 (`CosRef3D`) or between uv directions below 0.88 (`CosRef2D`);
chord-deflection estimate $|d_1 - d_4|^2\cdot\text{Norme}/64$ against $0.25\,\text{fleche}^2$; after 5
consecutive accepted steps grow by a quarter of the gap to the initial step, capped at 10 %; on
"step too small" grow at most 1.5×; default deflection 0.01 clamped to [1e-3, 10], uv step 0.01
(≤ 0.5; 1e-4 near singular points).

*Validated tracing* (Mukundan 2004, verified): an interval ODE solver accepts a step only after
proving existence and uniqueness in an a-priori enclosure, which prevents straying and reports
singularities; but it needed 845–1303 steps per branch near a singularity (Table 1) and the
authors call it "costlier compared to a conventional algorithms for simple problems". Not
recommended as the default.

**Data structures.** `Pt` as today (it already holds both uv); per simple pair: $P$, $w_0$, $w_1$.

**Failure modes.** Newton failing at a plane: halve $\delta w$. $P\cdot T$ near zero cannot happen
inside a certified pair (it is bounded below by the separability margin). Tangential branches go to
the quadratic above; $d$ close to 0 within rounding is the hard case and stays a tangent chain.

**Fit in cadrs.** `ssi_numeric::march`: a new `sweep(ctx, seeds, P)` for certified simple pairs;
`march` keeps the rest and gains the tangential direction in `tangent` (today it "steps straight
on" through tangent points). Needs card 2; feeds card 1.

**Sources.** `papers/intersection/Hohmeyer-1992-robust-efficient-surface-intersection-thesis.md`
§3.3, §3.4 (verified; the trace equations from `notes/intersection.md` entry 1);
`papers/intersection/Mukundan-2004-tracing-surface-intersection-validated-ODE.txt` §2, §2.1 eq.
(3), §2.2 eqs. (4)–(6) and the four cases, §3.3, §5 Table 1 (verified); OCCT step constants from
`notes/intersection.md` entry 10 (OCCT source, not verified here). The $B$ matrix and the 2-unknown
sweep are derivations.

---

## 6. Point projection on NURBS: span pruning and a second-order step

**Problem.** A cold projection onto a NURBS or extrusion evaluates a 24 × 24 grid (576 points)
and runs Newton from the best one (`Surface::project_grid`); the grid decides which stationary
point Newton reaches, with no guarantee it is the global one.

**Why it is fast.** Point-to-surface is the LUB-tree of card 7 with one side a point. Johnson &
Cohen report $10^{-6}$ parametric precision at 10–50 Hz on an SGI Indigo2, against about 1 s for
solving the two orthogonality equations numerically, and stable behaviour for orders 2 to 7 with
time "directly proportional to the order in each direction" (1998 §2.3, §3.5.2–3.5.3, verified).
Whole spans are discarded by one hull-distance test against the best distance so far, and Newton
only starts where a foot point can exist.

**Algorithm.**

1. Per surface, once (cache on the surface): Bézier patches at the knot spans; each with its
   control net (hull vertices), AABB, and the Greville abscissae ("parametric nodes") of its
   control points.
2. Upper bound $U$: with a hint, $|S(\text{Newton from hint}) - p|$. Otherwise map the closest hull
   point of the patch with the smallest lower bound to (u, v) by bilinear interpolation between the
   parametric nodes of the surrounding control points and evaluate (Johnson–Cohen §3.5, Fig. 4,
   verified).
3. Lower bound per patch: AABB distance first, then the Gilbert distance from $p$ to the control
   net's hull (Johnson–Cohen §3.5, verified: Gilbert's algorithm on local hulls "after" an AABB
   test).
4. Drop patches with lower bound $> U$. Split the rest (de Casteljau at the middle, or knot
   insertion over contiguous active regions with a size limit, Johnson–Cohen §3.5, §3.5.1,
   verified). Breadth-first, updating $U$ each level.
5. Prune patches that cannot hold an interior foot point (notes only: Selimovic 2006/2009,
   Johnson–Cohen 2005; not local, unverified). A foot point needs $p - S \parallel N$. If no
   direction from the patch hull to $p$ lies in the patch's double-sided normal cone (card 2's
   generators), the patch has no interior stationary point; keep it only for its border (the
   minimum then lies on a patch edge, covered by the neighbour or by the surface's own boundary
   curve). Selimovic's end-point test: if all control points lie on the far side of the plane
   through a corner $c$ perpendicular to $p - c$, that corner is the closest point of the patch.
6. Leaf solve when a patch is below a size limit or passes the single-solution test (card 3,
   $n = 2$, on $g = (S_u\cdot(S - p),\ S_v\cdot(S - p))$):
   - Newton (Piegl–Tiller form, notes): $g$ as above,
     $J = \begin{pmatrix} S_u\cdot S_u + S_{uu}\cdot(S-p) & S_u\cdot S_v + S_{uv}\cdot(S-p) \\
     S_u\cdot S_v + S_{uv}\cdot(S-p) & S_v\cdot S_v + S_{vv}\cdot(S-p) \end{pmatrix}$; or
   - Hu–Wallner second-order step (notes only; reconstruction, verify convergence order by test):
     first-order step $\Delta = G^{-1}(S_u\cdot(p - S),\ S_v\cdot(p - S))$ with
     $G = \begin{pmatrix}E & F\\ F & G\end{pmatrix}$; tangent $c = S_u\Delta_u + S_v\Delta_v$; normal
     curvature $\kappa = (L\Delta_u^2 + 2M\Delta_u\Delta_v + N\Delta_v^2)/(E\Delta_u^2 + 2F\Delta_u\Delta_v + G\Delta_v^2)$;
     in the plane of $\hat c$ and $n$ through $S$, the circle of curvature $\kappa$ with centre
     $m = S + n/\kappa$; project $p$ into that plane and onto the circle; $s$ = signed arc length
     from $S$ to that point; update $(u, v) \mathrel{+}= (s/|c|)\,\Delta$. If $|\kappa|$ is tiny,
     use $s = |c|$ (the first-order step). Li et al. 2017 prove second-order convergence for
     projection and third-order for inversion on planar curves (notes).
7. Stop when the step is below `tol::ROUNDOFF` relative to size and the angle between $p - S$ and
   $n$ is below `tol::ANGULAR`; clamp to the domain (wrap periodic directions).

Minimise the squared distance, not the distance: "the derivative of the plain distance can
diverge as it approaches zero" (node66, notes).

**Data structures.** Per surface: a cached `Vec<BezierPatch>` with hull vertices, AABB, Greville
nodes and normal generators (shared with cards 2 and 7); a small active list per query.

**Failure modes.** Hull-only pruning is not enough for candidate selection: Chen et al. 2007 give
a counterexample to Ma–Hewitt's control-polygon test (notes); the sphere-of-best-distance prune of
step 4 is safe because it compares a true lower bound with a true upper bound. Newton converges to
the wrong stationary point near a centre of curvature; the leaf only runs where step 5 allows a
single foot point. Hu–Wallner degrades to first order where $\kappa \approx 0$.

**Fit in cadrs.** `Surface::project` / `project_newton` / `project_grid` and `project_coarse` for
`Extrusion` and `Nurbs`; `pcurve::project_along`. Hinted projections (the tracer's) keep Newton or
switch to the second-order step only. Shares the patch cache with card 7, the solver with card 3.

**Sources.** `papers/tessellation_mass_queries/Johnson-Cohen-1998-framework-efficient-minimum-distance.txt`
§2.3, §3.1, §3.5–3.5.3 (verified); Piegl–Tiller, Ma–Hewitt, Selimovic, Hu–Wallner, Chen et al.,
Li et al. from `notes/tessellation_mass_queries.md` [P1]–[P5] and `notes/intersection.md` entry 14
(no local copies; not verified).

---

## 7. LUB-tree minimum distance with hull bounds and normal-cone pruning

**Problem.** `distance::extremum` bounds cell pairs by AABBs (sampled and grown for analytic
cells), splits down to 1/50 of a piece, then launches damped Newton from the centres of surviving
pairs. Loose lower bounds keep many pairs alive, and each survivor costs a Newton solve.

**Why it is fast.** Pruning compares a lower bound per pair with one global upper bound; tighter
lower bounds (hull distance instead of boxes) and an early upper bound cut the active set, and the
control net converges quadratically to the surface under refinement, "so the upper bound
converges in only a few iterations" (Johnson–Cohen 1998 §3.5.2, verified). Cost: best case
$\log_2 N$ bound evaluations; "typical bad case" (parallel flats, concentric spheres) $2N$ lower
and $\log_2 N$ upper bounds (§3.3, verified). Breadth-first was about twice as fast as Quinlan's
depth-first search on polygon models, 20–150 ms for 575–23 581 triangles (§3.6.1, verified).
Surface–surface: 2–20 Hz on an Indigo2 against "one second to several minutes" in earlier work
(§3.5.2, verified). For repeated queries on moving parts, bound coherence "can more than double the
speed" (1999 abstract, verified).

**Algorithm** (Johnson–Cohen 1998 §3.1, verified, with cadrs's pieces):

1. Nodes: the Bézier patches (cards 6's cache) of each face piece, edges' curve spans, vertices.
   cadrs already splits entities into face, edge and vertex pieces (`distance::Piece`), so a
   face–face pair only needs interior extrema.
2. For each active pair: lower bound $L$ = AABB distance; if $L < U$, Gilbert distance between the
   two control-net hulls (§3.5).
3. Upper bound $U$: from the pair with the smallest $L$, take the closest points of the two hulls,
   map each to (u, v) by bilinear interpolation of the Greville nodes, evaluate, measure; polish
   with the existing Newton immediately so $U$ is tight early (§3.5 Fig. 4, verified; the polish
   is cadrs's).
4. Drop pairs with $L > U$ (or $L > U - \text{tol}$ for "all solutions within tolerance").
5. Normal-cone prune (Johnson–Cohen 2005, notes only, unverified): an interior extremum of
   $|P - Q|$ needs $P - Q$ parallel to both normals. Let $D$ be the cone of directions from
   pair cell B's hull to cell A's hull. If $D$ misses A's double-sided normal cone or B's (card 2's
   generators, tested with the same min-norm routine), the pair has no interior extremum: drop it.
   This is valid in cadrs because piece boundaries are separate pieces.
6. Refine survivors: group contiguous surviving patches into active regions, extract each region
   by knot insertion at its boundary and refine it, with a limit on patches per region so diagonal
   strips do not refine whole rows (§3.5, §3.5.1, verified).
7. Repeat level by level (breadth-first) until $U - \min L <$ tolerance or every survivor is a leaf;
   then Newton (or card 3's 4-unknown solve of the antipodal system, card 9) on the leaves.

For `max_distance` swap the roles (cadrs does: farthest-corner bound); step 5 applies unchanged.

*Bound coherence for repeated queries* (1999, verified): keep the leaf pair that gave the minimum
and start the next query with its distance as $U$; store the lower bounds of pairs that were pruned
(only those) in a hash table keyed by pair id, and next time prune a saved pair if
$L_{\text{saved}} - \text{movement} > U$, where movement is the largest displacement of the moved
body's bounding-box corners. Useful for interactive drag/clearance, not for rebuilds.

**Data structures.** Patch cache (card 6) with hulls and Greville nodes; per level a `Vec` of
active pairs `(node_a, node_b, L)`; a hash map of saved lower bounds for coherence.

**Failure modes.** Control hulls are loose for high degree or few spans; refinement fixes it at a
cost (knot insertion allocates; §3.5 notes the region extraction for that reason). Breadth-first
keeps the whole level in memory. There is no closed-form error, only $U - L$. Johnson–Cohen
compare breadth-first with depth-first, not with best-first; cadrs's best-first queue may already
be as good, so measure before changing the search order.

**Fit in cadrs.** `distance::cell_box` → hull lower bound for NURBS cells (analytic cells keep
`patch_bbox`); `distance::extremum` gains the upper-bound step, the normal-cone prune and region
refinement in place of uniform splitting to 1/50. `polish` stays as the leaf solver. Shares the
patch cache and normal generators with cards 2 and 6.

**Sources.** `papers/tessellation_mass_queries/Johnson-Cohen-1998-framework-efficient-minimum-distance.txt`
§3.1–3.6 (verified); `papers/tessellation_mass_queries/Johnson-Cohen-1999-bound-coherence-minimum-distance.txt`
(verified); Johnson–Cohen 2005 tangent cones from `notes/tessellation_mass_queries.md` [D1]
(not local, unverified).

---

## 8. Rounded-interval coefficients for exclusion tests

**Problem.** Every pruning step in cards 2–4 and 7 says "no root here" from floating-point
coefficients. A rounding error of one ulp can flip a sign and drop a box that holds a root.

**Why it is fast.** Only the exclusion tests need intervals; Newton and tracing stay in plain
floats. TIGHT/NFG show correctly rounded intervals can be fast: in the MiSo solver, torus–torus SSI
over $[0,1]^4$ ran 11 105 iterations and produced 6976 4-D boxes in 136 ms with TIGHT against
2186 ms with Filib++ (about 16×); continuous collision queries took 27 ms, 133 ms and 2.5 s against
127 ms, 756 ms and 13.2 s (about 5×) (§4.2.1–4.2.2, verified). TIGHT returns intervals 0 or 1 ulp
wide for single operations (Table 1, verified). The cost relative to plain `f64` is not reported.

**Algorithm.**

1. Represent each Bernstein coefficient as $[\underline b, \overline b]$ (two `f64` tensors).
2. Arithmetic (Rust, derivation): Rust/LLVM gives no portable control of the rounding mode, which
   NFG relies on (it runs in round-upward mode and computes lower bounds as $-((-a)\star b)$,
   §2, verified). Instead compute each bound in round-to-nearest and widen by one ulp:
   $[\,\text{next\_down}(\underline r), \text{next\_up}(\overline r)\,]$ (`f64::next_down`/`next_up`).
   This is valid for $+, -, \times, \div, \sqrt{}$ because IEEE 754 rounds them correctly, and is at
   most one ulp per side wider than TIGHT's result. Products of intervals take the min/max of the
   four endpoint products.
3. de Casteljau at $t = 1/2$: $(a + b)/2$ per step; the halving is exact, the sum is widened.
4. Exclusion: drop a box when every $\underline b > 0$ or every $\overline b < 0$. Envelope reduction
   (card 3 step 3) uses $\underline b$ for the lower envelope and $\overline b$ for the upper.
5. Build the input coefficients (control points, products for card 4's $f$, the pseudo-normal of
   card 2) in interval arithmetic once per root patch; subdivision keeps the widths small.

Transcendental functions are not needed when analytic surfaces enter through their implicit
polynomial (card 4). If they are (a parametric torus as in TIGHT's demo), TIGHT gets sin/cos
ranges by checking the derivative sign at both ends and comparing against precomputed correctly
rounded multiples of $\pi/2$ in $[-2\pi, 2\pi]$ (§3.1.6, verified).

**Data structures.** `IBernTensor { lo: Vec<f64>, hi: Vec<f64>, dims }`; an `Interval` type with
the widened operations.

**Failure modes.** Interval widths grow with depth of computation; correct rounding "does not
eliminate the issue, but it reduces propagation to a minimum and makes it predictable" (§4, the Figure 4 example,
verified). A box can then fail to be excluded although empty, which costs subdivisions, not
correctness. The guarantee covers enumeration only; TIGHT's SSI demo is subdivision only, with no
Newton contraction and no topology (notes C1). Fünfzig's alternative for the LP route: push each
hyperplane outward by the error accumulated in its row (§5, verified).

**Fit in cadrs.** Inside card 3's tensor type (a feature or a second type), card 4's $f$, card 2's
generators when exact bounds are wanted, card 7's hull distances. `tol` keeps its tolerance model;
this only guards the "empty" decisions.

**Sources.** `papers/kernels_recent/Sichetti-2025-TIGHT-intervals.txt` §2, §3.1.6, §4.1 Table 1,
§4.2.1, §4.2.2, Figure 4 example (verified); `papers/intersection/Fuenfzig-Michelucci-Foufou-2010-tensorial-Bernstein-solvers.txt`
§5 (verified). The Rust widening scheme is a derivation.

---

## 9. Exact extremum events for maximum and Hausdorff distance

**Problem.** Branch and bound finds a maximum slowly because far-corner bounds are loose, and gives
no proof that it found every candidate. The extremum of the distance between two pieces is reached
at a finite set of events that can be solved for directly.

**Why it is fast (and where it is not).** The events are square polynomial systems of 1–4 unknowns,
solved by card 3, so the answer is exact to tolerance with no sampling. But the authors call the
computation "not fast": Hausdorff distances between curves took "a few seconds to several dozens
seconds"; for two curves of 50 coefficients the tensor-product formulation of the bisector system
took 326 s and 1470 MB, 23 s and 11.5 MB with their reduced form; the surface cases were not
implemented (§5–§7, Table 1, verified). Use it for low-degree pieces, as a check, or on the few
leaf pairs that card 7 leaves, not as the global search.

**Algorithm** (Elber–Grandine 2008, verified). For pieces $S(u, v)$ and $R(r, s)$ (curves have one
parameter, points none):

1. Minimum and maximum distance occur at boundary events (end points, boundary curves, corners,
   handled as separate pieces in cadrs) or at interior antipodal points (§4):
   - point $q$ – curve $C$: $\langle q - C(t), C'(t)\rangle = 0$ (1 unknown, eq. 1);
   - curve – curve: $\langle C_1(r) - C_2(t), C_1'(r)\rangle = 0$, $\langle C_1(r) - C_2(t), C_2'(t)\rangle = 0$ (eq. 2);
   - curve – surface: $\langle S - C, C'\rangle = 0$, $\langle S - C, S_u\rangle = 0$, $\langle S - C, S_v\rangle = 0$ (eq. 5);
   - surface – surface: $\langle S - R, S_u\rangle = \langle S - R, S_v\rangle = \langle S - R, R_r\rangle = \langle S - R, R_s\rangle = 0$ (eq. 10).
2. Build each equation's Bernstein coefficients by products (card 3; inner products of
   differences of two independent patches are tensor products in all their parameters).
3. Solve with card 3; keep solutions inside the trimmed domains (post-filter in uv); the extremum is
   the min or max over all events.
4. Hausdorff distance additionally needs self-bisector events, e.g. for two curves
   $\langle C_2(t) - C_2(s), C_2(t) + C_2(s) - 2C_1(r)\rangle = 0$ with $\langle C_1(r) - C_2(s), C_2'(s)\rangle = 0$ and
   $\langle C_1(r) - C_2(t), C_2'(t)\rangle = 0$ (eqs. 3–4). The first equation contains the factor
   $(t - s)$ and vanishes on $t = s$: split the curve at its curvature extrema (solve
   $(\kappa^2)' = 0$; $\kappa^2$ is rational) so $s$ and $t$ lie in different pieces, or divide the
   factor out (§2, verified).

For analytic pairs (planes, quadrics, tori) the antipodal condition $\nabla f(p) \parallel \nabla g(q) \parallel p - q$
is small and often closed form (Chen et al. 2006, notes [D3], not local).

**Data structures.** Card 3's tensors; one system per piece pair; a list of events with values.

**Failure modes.** Constraints vanish where the pieces intersect, so intersections must be found
and excluded first (§7, verified). Coefficient counts grow as $O(n^k)$ for $k$ independent forms
(§6, verified); four unknowns of bicubic patches is the practical limit. Degenerate solution sets
(concentric spheres, parallel planes) return boxes, not points: report the extremum value with a
representative point.

**Fit in cadrs.** `distance::extremum` with `Sense::Max` on the leaf pairs from card 7, or as an
exact check in tests; a future Hausdorff query (approximation error of fits) would use step 4.
Needs card 3.

**Sources.** `papers/tessellation_mass_queries/Elber-Grandine-2008-Hausdorff-minimal-distances-freeforms.txt`
§2–§7, eqs. (1)–(5), (10), Table 1 (verified); Chen et al. 2006 from
`notes/tessellation_mass_queries.md` [D3] (not verified).

---

## Left out

- Krishnan–Manocha resultant/eigenvalue tracing and Grandine–Klein BVP tracing: degree blow-up (a
  bicubic pair gives a plane curve of degree up to 108) or paywalled details; characteristic
  points (card 4) give the loop guarantee at far lower cost (synthesis report, "Study only").
- Validated interval ODE tracing (Mukundan 2004): correct but slow (card 5).
- Affine-arithmetic GPU subdivision and Mapper topology (Lin et al. 2014, Gao et al. 2026): no
  certified topology, no benchmarks against marching.
- Bernstein-polytope LP solvers (Fünfzig): only pay off above 6–7 variables; for 2 variables they
  match the tensor solver (§7, verified).
- Bézier clipping and quadratic clipping (Sederberg–Nishita, Bartoň–Jüttler): no local text; card
  3's envelope reduction does the same job in 1-D. Worth a benchmark later as a 1-D accelerator.
