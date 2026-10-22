# Mass-property, tessellation and hidden-line algorithms for cadrs_kernel

Algorithm cards drawn from `research/README.md`, `notes/tessellation_mass_queries.md`,
`notes/kernels_recent.md` and the paper texts. Each card says where it plugs into
`crates/cadrs_kernel/src/native/` (`mass.rs`, `tess.rs`, `hlr.rs`, `silhouette.rs`). Each card
also says whether its steps were checked against a paper text ("verified"), come only from the
notes, or are my own derivation. My derivations were checked numerically where marked. The
scripts are throwaway Python, not part of the repo.

Starting point, from the kernel README and the code.

`mass.rs` uses the Timmer–Stern structure: the divergence theorem per face, then Green's theorem
over the uv domain. One pass accumulates 14 quantities: area and the area moments (`Q[0..4]`,
weighted by |N|), and volume, the first moments and the second moments (`Q[4..14]`, weighted by
N = S_u × S_v). The point counts are fixed and generous:

- Untrimmed B-spline rectangles (`rect_integrals`): 16 Gauss points per v-span. Faces with 1–2
  spans are cut in 4 first. The inner integral along u also uses 16 points per span, with the
  same 4-way cut (`u_breaks`). A bicubic face with one span therefore costs 64 × 64 = 4096
  evaluations.
- Trimmed faces (`face_integrals`): 16 outer points per p-curve piece and 16 inner points per
  u-span crossed, measured from one face-wide reference u (`u_ref`).
- Periodic faces: pieces of at most π/8 with 16 outer points; the inner rule is 8 points along
  the ruling (`face_integrals_v`) or 8 per π/8 along u.

The comments give the reason for the 16-point rules: "the area integrand |N| is only smooth".

`tess.rs` samples each edge once. It seeds interior points on a grid whose steps come from a
7 × 7 sample of |S_uu·n| and |S_vv·n| (`steps`; S_uv is ignored and nothing is guaranteed). It
then refines a spade CDT by measuring each triangle against the surface (`split_point`), plus
spade's Ruppert/Chew refinement for aspect.

`hlr.rs` splits every projected line at its 2D crossings, found on a grid. It tests each piece
by rays towards the eye, and tests long pieces at three points, bisecting where they disagree.
`silhouette.rs` traces n·w = 0 by marching squares on non-quadric faces and does not split at
cusps.

## Ranked cards

| Rank | Card | Payoff | Size | Risk |
|---|---|---|---|---|
| 1 | [Exact degree-matched Gauss rules for polynomial faces](#1-exact-degree-matched-gauss-rules-for-polynomial-faces) | Most of the 11x mass gap. 64x fewer evaluations on 1–2-span bicubic patches, 2–4x on many-span and trimmed ones; volume, centroid and inertia exact to rounding | Small–medium | Low |
| 2 | [Area and non-polynomial integrands with an error estimate](#2-area-and-non-polynomial-integrands-with-an-error-estimate) | Needed before card 1 can drop the 16-point rules; gives an OCCT-style error report | Small | Medium (area accuracy at the smaller rule is unmeasured) |
| 3 | [Trigonometric and periodic rules for analytic faces](#3-trigonometric-and-periodic-rules-for-analytic-faces) | Full circles: 5 points instead of 256 outer; inner 2–3 instead of 8 | Small | Low |
| 4 | [Trimmed faces: Gunderman-style antidifferentiation](#4-trimmed-faces-gunderman-style-antidifferentiation) | Fewer spans crossed per inner integral; outer rules matched to the trim's degree, stopping at the trim's own error | Medium | Low–medium |
| 5 | [Planar faces and mesh solids in closed form (Mirtich/Eberly)](#5-planar-faces-and-mesh-solids-in-closed-form-mirticheberly) | No surface evaluations for polygonal faces and `meshsolid` bodies; 6 instead of 128 per edge otherwise | Small | Low |
| 6 | [HLR: quantitative invisibility propagated along each line (Appel)](#6-hlr-quantitative-invisibility-propagated-along-each-line-appel) | About 2 ray tests per line instead of 1–3 per piece plus bisection | Medium | Medium |
| 7 | [Silhouettes: curtain folds (cusps) and concave outlines](#7-silhouettes-curtain-folds-cusps-and-concave-outlines) | Correct split points for visibility; concave outline pieces hidden with no ray test | Small | Low |
| 8 | [Tessellation: certified chord bound (Filip–Magedson–Markot) for seeding and early accept](#8-tessellation-certified-chord-bound-filipmagedsonmarkot-for-seeding-and-early-accept) | Guaranteed seeding; triangles that pass the bound need no surface evaluation | Medium | Low |
| 9 | [Tessellation: analytic interior nodes and closed-form checks](#9-tessellation-analytic-interior-nodes-and-closed-form-checks) | Regular meshes on quadrics and tori; about π/2 fewer points on spheres; no sampled check | Small–medium | Low |
| 10 | [Tessellation: kd-tree seeding for B-spline faces (Balázs–Guthe–Klein)](#10-tessellation-kd-tree-seeding-for-b-spline-faces-balázsgutheklein) | Adaptive seeding where curvature is concentrated. The paper: 4.4x fewer triangles and 3.6x faster than a quadtree | Medium | Medium |

Cards 1–5 together target the 11x mass-property gap. The order to build them is 2, 1, 3, 5, 4.
Card 1 alone does not speed anything up while the area terms still need 16 points.

---

## 1. Exact degree-matched Gauss rules for polynomial faces

**Problem.** On faces whose mass integrands are polynomials in (u, v), `mass` uses 16-point
rules and cuts short faces into 4. A rule matched to the integrand's degree is exact with far
fewer points.

### Why it is fast

An n-point Gauss–Legendre rule is exact for polynomials of degree ≤ 2n − 1. Per knot span, a
non-rational B-spline face of bidegree (p, q) has these integrand degrees. This is my
derivation; the researcher's derivation is checked below.

- S has degree (p, q). S_u has (p − 1, q). S_v has (p, q − 1). So N = S_u × S_v has
  (2p − 1, 2q − 1).
- Volume integrand $h_V = \tfrac13\,x\cdot N$: degree (3p − 2, 3q − 2). The naive count is
  (3p − 1, 3q − 1), but the top coefficient vanishes. Write the u^p coefficient of S as a(v).
  The leading u-term of x·N is then $a\cdot(p\,a \times a') = 0$, and likewise in v.
- First moments ($\tfrac12 x_i^2 N_i$, centroid): degree (4p − 1, 4q − 1).
- Second moments ($\tfrac13 x_i^3 N_i$ and $\tfrac12 x_i^2 x_j N_i$, inertia): degree
  (5p − 1, 5q − 1).

Exact points per span and direction:

| Quantity | Degree in u | Points $n_u$ | p = 2 | p = 3 | p = 5 |
|---|---|---|---|---|---|
| Volume | 3p − 2 | ⌈(3p − 1)/2⌉ | 3 | 4 | 7 |
| Centroid (first moments) | 4p − 1 | 2p | 4 | 6 | 10 |
| Inertia (second moments) | 5p − 1 | ⌈5p/2⌉ | 5 | 8 | 13 |

The same formulas hold in v with q. One pass computes all ten volume quantities at shared
points, so the inertia row sets the rule: **8 × 8 = 64 points per bicubic span.** Today a
bicubic face with 1–2 spans costs 4096 per span (64x more). A face with more spans costs 256
per span (or 128 past 16 v-spans), 2–4x more.

Numerical check (my script, random Bézier spans, against a 20 × 20 rule):

- At the counts above, relative errors were 5e-15 to 2e-13 for (p, q) = (3,3), (2,3) and (5,2).
- One point fewer in u gave 2e-3 (centroid, p = 3) and 1e-4 (inertia, p = 3).
- For volume, one point fewer still gave 6e-15 at p = 3, which confirms the vanishing top
  coefficient.

**The researcher's derivation** (notes [M8]; README "Mass properties" section) says the volume
integrand is bidegree (3p − 1, 3q − 1) and ⌈3p/2⌉ × ⌈3q/2⌉ points are exact.

- I agree that this count is sufficient. One point fewer suffices for odd p, because the degree
  is really 3p − 2.
- The notes also say "the second-moment integrands x²·n_x have (4p−1, 4q−1)" and give 2p
  points. **That is wrong for inertia.** x²·n_x is the first-moment (centroid) integrand under
  Mirtich's choice of F. The inertia integrands are x³·n_x/3 and x²y·n_x/2, of degree
  (5p − 1, 5q − 1), and need ⌈5p/2⌉. With 2p = 6 points a bicubic span's inertia is off by
  about 1e-4 relative, as in the check above.

Other polynomial surfaces, by the same counting (my derivation):

| Surface | Inner direction | Inertia degree | Points |
|---|---|---|---|
| Plane | u | 3 (x linear in u, N constant) | 2 |
| Extrusion of a degree-m curve, along v | v | 3 | 2 |
| Extrusion of a degree-m curve, along u | u | 4m − 1 | 2m |
| B-spline face with p = 1 in a direction | that direction | 4 | 3 |

### Algorithm

1. Classify the face. The polynomial classes are: `Plane`; `Nurbs` with every weight equal
   (non-rational in fact, which is common in STEP files that set the rational flag); and
   `Extrusion` of a line or a non-rational B-spline. Rational faces go to card 2 and analytic
   faces to card 3.
2. Untrimmed rectangle (`iso_rectangle` succeeds): on each knot cell
   $[u_i, u_{i+1}] \times [v_j, v_{j+1}]$, apply the tensor Gauss rule with
   $n_u = \lceil 5p/2\rceil$ and $n_v = \lceil 5q/2\rceil$ points. Use
   $\int\!\!\int h \approx \sum_a\sum_b w_a w_b\, h(u_a, v_b)\,\tfrac{\Delta u\,\Delta v}{4}$
   with the Gauss nodes mapped from [−1, 1]. Drop the 4-way cut and the 8/16-point choice.
3. Trimmed face (general Green path), inner integral: on each u-span crossed, use $n_u$ points
   (exact per span, since h is polynomial there). Keep splitting at the u-knots as `u_breaks`
   does now, but without the 4-way cut.
4. Trimmed face, outer integral along a p-curve piece. Pieces are already split at the u- and
   v-knots (`split_at_knots`), so each piece lies in one knot cell.
   - Within a cell, $H(u,v)=\int_{u_\text{ref}}^{u} h\,ds$ is a polynomial of bidegree
     $(5p,\,5q-1)$. The complete spans give polynomials in v; the partial span gives a
     polynomial in (u, v).
   - For a polynomial p-curve piece of degree r, the integrand $H(u(t),v(t))\,v'(t)$ has degree
     $\le r(5p+5q-1) + r - 1 = r(5p+5q)-1$. So $n_\text{outer} = \lceil r(5p+5q)/2\rceil$
     (15 for a straight piece on a bicubic face).
   - For an iso-line u = const: $n = \lceil 5q\,r/2\rceil$.
   - For an iso-line v = const: v' ≡ 0, so skip the piece entirely. Today the code skips it
     point by point.
   - If $n_\text{outer}$ would exceed a cap (16), or the p-curve is rational, use card 4's rule.
5. Tabulate Gauss–Legendre nodes and weights for n = 1..20. Today there are only `GAUSS4/8/16`.
   Golub–Welsch (cited by Gunderman et al. as their ref. [23]) or any standard table will do.
6. Keep the job layout (`[u0, u, v, w]` jobs, `run_jobs`) so the GPU path still applies. Only
   the counts change.

### Data structures

- A per-face record of (p, q), the knot breaks per direction, a flag for non-rational, and the
  chosen $n_u$, $n_v$ and $n_\text{outer}$ per piece class.
- A Gauss table indexed by n.

### Failure modes and error bounds

- Polynomial faces: the quadrature error is zero. What remains is rounding, about
  (points) × ε_mach × the integrand's magnitude. `body_mass` already centres coordinates on the
  bounding box (`shift`), which keeps the magnitudes small.
- The guarantee needs every point to lie in one polynomial piece. A missed knot (a p-curve
  crossing a knot that `split_at_knots` misses) silently loses exactness. Its bisection assumes
  each piece is monotone.
- The area terms `Q[0..4]` are not polynomial (|N| is a square root). They are card 2's job.
- Test oracle: the same face with n + 4 points per direction must agree to about 1e-13
  relative. A Gauss rule of higher order is itself exact here, so Gonzalez-Ochoa's
  Bernstein-product moments (notes [M5], from memory) are not needed as an oracle.

### Fit in cadrs

- `mass.rs`:
  - `rect_integrals`: drop `cut`'s 4-way split and the GAUSS16/8 choice; use per-span
    $n_u \times n_v$.
  - `inner`: replace GAUSS16 for `Nurbs` with $n_u$ per span.
  - `u_breaks`: remove the `m = 4` split for polynomial faces.
  - `face_integrals`: replace the outer `g` selection with $n_\text{outer}$ per piece.
  - `inner_v`: replace GAUSS8 with 2 points for planes and extrusions.
- Prerequisite: card 2. Otherwise the area terms lose accuracy, because they share the points.

### Sources

- Degree argument: my derivation, numerically checked. It corrects notes/tessellation_mass_queries.md [M8] and agrees with README "Mass properties close their 11x gap…" for volume.
- Hughes, Reali & Sangalli 2010 (isogeometric quadrature): not downloaded; notes only.
- Choice of F per monomial (x²/2, x³/3, x²y/2 …): papers/tessellation_mass_queries/Eberly-2002-polyhedral-mass-properties.txt, §2, table (3) and equations (5). Verified. These match `accumulate` in `mass.rs`.
- Untrimmed patch reduces to a tensor rule: papers/tessellation_mass_queries/Gunderman-Weiss-Evans-2021-mesh-free-quadrature-trimmed-parametric-surfaces.txt, §2.3.2 (p. 7). Verified.

---

## 2. Area and non-polynomial integrands with an error estimate

**Problem.** Area (|N|) and rational faces cannot be integrated exactly. Today they set the
16-point rules for everything. They need their own control so that card 1's small rules can be
used, and the result should carry an error estimate as OCCT's does.

### Why it is fast

It keeps card 1's point counts for the evaluations every quantity shares. Only spans where the
area integrand is badly behaved get more points, and the error estimate is computed only when a
caller asks for a tolerance. OCCT's non-adaptive mode is 11x faster than cadrs but 1e-3 off on
the blade hubs (README "Performance"). This card aims at OCCT-like counts with a reported error.

### Algorithm

1. **Area on shared points.** Accumulate `Q[0..4]` at card 1's points. For a non-rational span,
   |N|² is a polynomial of degree (4p − 2, 4q − 2) and |N| is analytic wherever N ≠ 0. Gauss
   converges exponentially there (Gunderman et al., §1: smooth integrands converge
   exponentially; a $C^k$ integrand gives order k + 1).
2. **Distortion check per span (my heuristic).** At the span's Gauss points, compute
   $\rho = \max|N| / \min|N|$. If $\rho > \rho_\text{max}$ (start at 4; tune on the corpus) or
   any |N| < `tol::TINY`, the span is near a pole or a collapsed row. Recompute the area terms
   of that span on 2 × 2 sub-cells with the same n, recursively, to depth 3.
   - These extra evaluations feed only `Q[0..4]`.
   - The polynomial quantities stay on the parent rule, which is already exact.
3. **Rational faces.** With homogeneous numerators X (3-vector) and weight W, S = X/W. Then
   (my derivation, by the quotient rule; terms parallel to X drop out of the determinant):
   $$x\cdot(S_u\times S_v) = \det[S, S_u, S_v] = \frac{\det[X, X_u, X_v]}{W^3}.$$
   The numerator is polynomial and the denominator positive, so the integrand is analytic but
   not polynomial. Use card 1's counts + 2 points per direction by default.
   - With the shift, use $X - W\cdot\text{shift}$ as the numerator.
   - The moment integrands have $W^4$ and $W^5$ in their denominators. The same rule applies.
4. **Error estimate (tolerance mode)** for `mass_properties_with(tol)`. Use OCCT's definition,
   quoted in notes [M9] from the BRepGProp reference manual (not a local paper):
   "Error is calculated as Abs((M(i+1)-M(i))/M(i+1))" between successive rule orders.
   - Compute each non-exact span (rational, area-distorted) with n and n + 2 points.
   - Report $\max_\text{face} |M_{n+2} - M_n| / |M_{n+2}|$.
   - Subdivide the spans whose difference exceeds tol × |face value| until it passes.
   - Exact spans (card 1) contribute zero to the estimate.
   - A nested pair (Gauss–Kronrod G7/K15 from QUADPACK) saves the second rule's evaluations on
     1-D outer pieces. Its constants are not in our papers.

### Data structures

- A per-span flag for exact or non-exact.
- A per-span area sub-cell tree, only for distorted spans.
- An error accumulator per face.

### Failure modes and error bounds

- No a priori bound for |N| or rational spans; there is only the two-order estimate, as in
  OCCT and as Gunderman et al. state ("no a-priori bound", notes [M7]).
- The estimate can under-report if both orders are wrong in the same way, as at a pole that
  neither rule resolves. The ρ check catches the usual cause, a vanishing N.
- **Risk:** nobody has measured the area error of 8 × 8 points per bicubic span on the corpus
  (blade hubs, SMLib models). Measure before switching the default.
- A free regression case: SMLib's known-issues list reports a 1.4% mass error on faces trimmed
  by rational circles with interior knots (README "Mass properties").

### Fit in cadrs

- `mass.rs`: split `accumulate` into a polynomial part (`Q[4..14]`) and an area part
  (`Q[0..4]`) so the sub-cells can feed only the area.
- Add a `MassOptions { tol: Option<f64> }` path beside `body_mass`.
- The existing `MassProperties` gets an optional error field.
- Prerequisite of card 1.

### Sources

- Exponential convergence for smooth integrands; "If the integrand is p-th order continuous … p + 1-th order convergence": papers/tessellation_mass_queries/Gunderman-Weiss-Evans-2021-mesh-free-quadrature-trimmed-parametric-surfaces.txt, §1 (pp. 3–4) and Table 1. Verified.
- OCCT's error definition and its non-adaptive mode for Eps > 0.001: notes/tessellation_mass_queries.md [M9]. Notes only (OCCT reference manual).
- Rational determinant identity and the ρ heuristic: my derivation and my choice.

---

## 3. Trigonometric and periodic rules for analytic faces

**Problem.** Cylinders, cones, spheres, tori and surfaces of revolution are integrated on
pieces of at most π/8 with 16 outer and 8 inner points. Their integrands are low-degree
trigonometric polynomials in the angle and low-degree polynomials along the ruling, which allow
much smaller rules.

### Why it is fast

- A full circle edge on a cylinder costs 16 pieces × 16 outer × 8 inner = 2048 evaluations
  today. With this card it costs 5 × 2 = 10.
- Partial arcs need half as many pieces (π/4 instead of π/8) at 8 points.
- Inner rules along rulings shrink from 8 points to 2 or 3.

### Algorithm

1. **Degrees per surface** (my derivation; the sphere and torus rows were checked numerically
   with an off-centre body and random axes). k is the trigonometric degree in the angle θ; the
   other direction is a polynomial or trigonometric degree.

| Surface | Volume (θ / other) | Centroid | Inertia |
|---|---|---|---|
| Cylinder (θ, z) | ≤ 2 / 1 | 3 / 2 | 4 / 3 |
| Cone (θ, v along generator) | ≤ 2 / 2 | 3 / 3 | 4 / 4 |
| Sphere (θ, φ), trig in both | 1 / 2 | 3 / 4 | 4 / 5 |
| Torus (θ, φ), trig in both | 1 / 2 | 3 / 4 | 4 / 5 |
| Revolution of a degree-m polynomial profile (θ, t) | 2 / 3m − 2 | 3 / 4m − 1 | 4 / 5m − 1 |

   Measured for sphere and torus with the periodic trapezoid rule: inertia exact from 5 points
   in θ and 6 in φ, and wrong (1e-2 to 1e-1) with one fewer.

2. **Whole periods.** When an inner or outer integral runs over a full period, use the N-point
   trapezoid rule with equally spaced points. It is exact for trigonometric polynomials of
   degree < N (a standard result, not from our papers). Inertia needs N = 5 in θ and N = 6 in φ
   on spheres and tori. Use it for full-circle edges, which are lines in uv with constant speed,
   and for inner integrals over a whole period on closed faces.
3. **Partial arcs.** Gauss–Legendre with n points on a piece of angular half-width h has the
   standard remainder. With $|f^{(2n)}| \le k^{2n}\|c\|_1$ for a trigonometric polynomial of
   degree k with coefficient vector c:
   $$|E| \le \frac{2h^{2n+1}(n!)^4}{(2n+1)\,[(2n)!]^3}\,k^{2n}\,\|c\|_1 .$$
   Computed values (relative to ‖c‖₁):
   - k = 4, pieces ≤ π/4, n = 8: 1.8e-20.
   - k = 6, pieces ≤ π/4, n = 8: 1.2e-17.
   - k = 6, pieces ≤ π/8, n = 6: 5.3e-16.
   - k = 6, pieces ≤ π/2, n = 8: 1.6e-12 (too coarse).

   So use pieces of at most π/4 with 8 points for k ≤ 6, in place of π/8 with 16.
4. **Polynomial directions** (along a cylinder's or extrusion's ruling, a cone's generator): use
   Gauss with ⌈(d + 1)/2⌉ points, where d is the inertia degree in the table: 2 points for
   cylinders and extrusions, 3 for cones. This replaces GAUSS8 in `inner_v`.
5. **Mixed p-curves**, such as a helix (θ = t, z = at + b on a cylinder): the integrand is a
   polynomial times a trigonometric polynomial, not covered by step 3's bound. Use π/4 pieces
   with 8 points and card 2's tolerance mode.

### Data structures

A per-surface-kind table of (k_θ, d_other) per quantity class, and a trapezoid table N = 1..8.

### Failure modes and error bounds

- Step 3's bound is rigorous for trigonometric polynomials and needs ‖c‖₁ only as a scale.
  Bound it by $(|{\rm shift}| + R_\text{max})^3 \times |N|_\text{max}$ for inertia.
- Surfaces of revolution with rational profiles (circle arcs) are not polynomial along the
  profile. Treat them as card 2's rational case.
- The degrees assume centred coordinates and an exact analytic parametrisation, which cadrs has
  for these types.

### Fit in cadrs

- `mass.rs`:
  - `u_breaks`: the `PI / 8.0` piece size becomes π/4 at 8 points.
  - `inner_v`: GAUSS8 becomes 2 or 3 points.
  - The `extra` piece loops in `face_integrals` and `face_integrals_v`: detect full periods and
    use the trapezoid rule; otherwise π/4 pieces with 8 points.
- `MAX_PIECES` stays as the guard.
- Independent of card 1.

### Sources

- Degrees and the remainder evaluation: my derivation and numeric checks.
- "Analytic faces … inner antiderivative is often closed form": notes/tessellation_mass_queries.md [M7] relevance. Notes only.
- Gauss–Legendre remainder and periodic trapezoid exactness: standard numerical analysis, not in the downloaded papers.

---

## 4. Trimmed faces: Gunderman-style antidifferentiation

**Problem.** On trimmed B-spline faces, each outer point integrates from one face-wide `u_ref`
across every span up to the boundary. Trim pieces of high degree or rational form get fixed
16-point rules with no idea of the trim's own accuracy.

### Why it is fast

- Gunderman et al. report errors 10–12 orders below linear and quadratic meshing at equal point
  counts. Propeller (5,136 rational bicubic patches, untrimmed): machine precision while the
  meshing methods "still have errors of at least 10⁻⁵". Bike frame (exact polynomial trims):
  "accuracy improvements of 12 orders of magnitude".
- Antidifferentiation by 10-point Gauss costs 2.4e-6 s per evaluation, against 8e-7 s for a
  power-rule antiderivative and 0.2 s of symbolic preprocessing per polynomial (MATLAB,
  Table B.2).
- For cadrs the gain is in point counts: inner integrals cross fewer spans, and outer rules are
  sized to the trim's degree and error instead of fixed at 16.

### Algorithm

1. **Boundary pieces.** Split each coedge's p-curve at its own breaks, at the surface's u- and
   v-knots, and at period seams. This exists: `pieces`, `split_at_knots` and the `extra` loop.
2. **Reference line per v-row** (my derivation, generalising Gunderman's constant $P_v$).
   - Green's identity $\iint_D h\,du\,dv = \oint_{\partial D} H\,dv$ needs only
     $\partial H/\partial u = h$ at each fixed v. So the reference may depend on v:
     $H(u,v)=\int_{u_\text{ref}(v)}^{u} h(s,v)\,ds$.
   - Per v-span j, set $u_\text{ref}(v) = $ the largest u-knot ≤ the smallest u of any boundary
     piece that overlaps v-span j.
   - H stays polynomial within each knot cell, so card 1's exactness holds. Inner integrals
     cross only the spans the face occupies in that row.
   - The points stay on the untrimmed patch, which is Gunderman's reason for choosing
     $P_v = 0$ and $P_z = \min z_{ij}$ "close to the region of integration" (§2.3.1).
3. **Antiderivative direction.**
   - Integrate along v instead of u when the boundary pieces' summed span-crossing count is
     smaller that way. Gunderman: "the antiderivative can be taken in an arbitrary direction"
     (§2.1.2).
   - cadrs already does this for ruled surfaces (`face_integrals_v`); generalise the choice to
     B-spline faces with $-\oint G\,du$.
4. **Inner rule:** exact per span (card 1) or analytic (card 3).
5. **Outer rule per piece.**
   - (a) A polynomial piece of degree r with $\lceil r(5p+5q)/2\rceil \le 16$: exact count
     (card 1).
   - (b) Otherwise (rational p-curves such as circle arcs in uv, fitted intersection curves,
     high r): a fixed 8-point Gauss rule per piece.
   - (c) In tolerance mode, add card 2's two-order estimate and bisect pieces.
   - Stop refining once the quadrature error is below the trim's geometric error. Gunderman:
     convergence is exponential "until the error in the geometric approximation of the trimming
     curves dominates", then of order $d+1$ under geometric refinement (cubic trims give nearly
     4th order, §3.7).
   - A bound on that geometric error (my derivation): moving a boundary by δ in 3D changes the
     face's volume integral by at most $\delta\,L\,R_\text{max}$. Here δ is the edge tolerance,
     L the edge length and $R_\text{max} = \max|x - \text{shift}|$ over the face. For second
     moments use $\delta\,L\,R_\text{max}^3$.
6. **Untrimmed cells** are tensor rules (card 1). Gunderman §2.3.2: the Green path on an
   untrimmed patch "is equivalent to the tensor-product quadrature rule" once the zero-weight
   left, right and bottom boundaries are dropped.

### Data structures

- Per face: knot breaks, `u_ref[j]` per v-span, and the chosen inner direction.
- Per piece: (t0, t1), knot cell (i, j), class (polynomial with degree r, rational, fitted),
  n_outer.

### Failure modes and error bounds

- Exact for polynomial trims on polynomial faces. Otherwise exponential convergence up to the
  trim's geometric error (Gunderman Table 1).
- Gunderman's teapot with cubic-approximated trims flat-lines at about 1e-6 for surface
  integrals and 1e-9 for volume integrals (§3.7). That is the scale cadrs' fitted intersection
  p-curves will reach.
- The method is robust to gaps between faces, "since we transform integrals onto trimming
  curves on each surface patch" (§2.2, §3.7). Each face is integrated on its own, which cadrs
  also does.
- A per-row $u_\text{ref}$ must be identical for every boundary point at that v, including
  pieces from different loops. Compute it once per face before integrating.

### Fit in cadrs

- `mass.rs`:
  - `face_integrals`: `u_ref` becomes a per-v-span table; add an inner-direction choice; the
    outer rule follows step 5.
  - `run_jobs` jobs carry their own `u0`, so the GPU path is unchanged.
- Prerequisites: cards 1 and 2.

### Sources

- Two-step method, numerical antidifferentiation and equations (3)–(5): papers/tessellation_mass_queries/Gunderman-Weiss-Evans-2021-mesh-free-quadrature-trimmed-parametric-surfaces.txt, §2.1.2–2.1.3 (pp. 5–6). Verified.
- Choice of constants: §2.3.1 (p. 7). Verified.
- Untrimmed simplification: §2.3.2 (p. 7). Verified.
- Results and geometric-error behaviour: §3.5–3.7 (pp. 8–9), Table 1 (p. 3), Table B.2 (p. 14). Verified.
- Per-row reference and the geometric-error bound: my derivation.
- notes/kernels_recent.md E4 (citation only).

---

## 5. Planar faces and mesh solids in closed form (Mirtich/Eberly)

**Problem.** Planar faces go through the general Green path: 16 outer × 8 inner = 128
evaluations per edge piece, for an integrand that is a cubic polynomial. Mesh solids
(`meshsolid`) have thousands of triangular planar faces.

### Why it is fast

For triangles, Eberly's formulas need about 100 flops per triangle and no projection, no Green's
theorem and no surface evaluation. Eberly: "they require significantly less computational time
than does Mirtich's formulas" (§1). No timing is given. One pass over the faces gives the volume,
the centroid and the inertia tensor.

### Algorithm

1. For a planar face whose loops are all straight lines, fan each loop from one apex A in the
   plane (the first vertex of the outer loop). Use the coedge order, which runs counter-clockwise
   about the outward normal.
   - Each directed edge $P_i \to P_{i+1}$ gives a signed triangle $(A, P_i, P_{i+1})$.
   - The sign comes out of $d = E_1 \times E_2$. Holes and non-convex loops need no special
     case, because the formulas are polynomial in the vertices and signed by d.
   - Work in centred coordinates (subtract `shift`).
2. Per triangle with vertices $(x_k, y_k, z_k)$, k = 0..2 (Eberly §5, verified):
   - $E_1 = P_1 - P_0$, $E_2 = P_2 - P_0$, $d = (d_0, d_1, d_2) = E_1\times E_2$.
   - For each coordinate w ∈ {x, y, z}:
     $f_1 = w_0+w_1+w_2$; $f_2 = w_0^2 + w_0w_1 + w_1^2 + w_2 f_1$;
     $f_3 = w_0^3 + w_0^2w_1 + w_0w_1^2 + w_1^3 + w_2 f_2$;
     $g_i = f_2 + w_i(f_1 + w_i)$.
   - Accumulate:
     - `intg[0] += d0·f1x`
     - `intg[1..3] += (d0·f2x, d1·f2y, d2·f2z)`
     - `intg[4..6] += (d0·f3x, d1·f3y, d2·f3z)`
     - `intg[7] += d0·(y0 g0x + y1 g1x + y2 g2x)`
     - `intg[8] += d1·(z0 g0y + z1 g1y + z2 g2y)`
     - `intg[9] += d2·(x0 g0z + x1 g1z + x2 g2z)`
3. Scale by `mult = {1/6, 1/24, 1/24, 1/24, 1/60, 1/60, 1/60, 1/120, 1/120, 1/120}`. The order
   is 1, x, y, z, x², y², z², xy, yz, zx.
   - Mass = intg[0]; centroid = intg[1..3]/mass.
   - $I_{xx} = \text{intg}[5]+\text{intg}[6] - m(c_y^2+c_z^2)$, and so on.
   - $I_{xy} = -(\text{intg}[7] - m\,c_x c_y)$, and so on.
   - Map these to cadrs' Q indices: 4 volume; 5–7 first moments; 8–10 x², y², z²; 11 xy;
     12 yz; 13 zx.
4. Planar faces with arcs or B-spline edges keep the Green path with exact counts (my
   derivation):
   - Inner: h is cubic in u, so 2 points.
   - Outer: H is quartic in (u, v). A line needs 3 points. A polynomial p-curve of degree r
     needs ⌈5r/2⌉ (8 for cubic intersection curves), exact.
   - Circle arcs: H(u(t), v(t))·v'(t) is a trigonometric polynomial of degree 5. Use card 3's
     π/4 pieces with 8 points.
   - Optionally split such a face into its chord polygon (Eberly) plus the circular segments
     (Green), so only the segments need quadrature.
5. Planar area terms (`Q[0..4]`): |N| is constant, so area and area centroid are also polynomial
   (degree 0 and 1). The same exact rules apply. For triangles: area = |d|/2 and centroid = the
   vertex mean.

### Data structures

None beyond the face's vertex loop; meshes pass a vertex and index array.

### Failure modes and error bounds

- Exact up to rounding.
- Precision loss for large coordinates far from the origin. Eberly's remark is to centre the
  model first, as `shift` does (notes [M4]).
- Mirtich's projection step exists for robustness in his non-triangle formulas and is
  unnecessary for triangles (Eberly §1).

### Fit in cadrs

- `mass.rs` `face_integrals`: an early branch for `Surface::Plane` with all-line loops.
- `body_mass` for `meshsolid`-built bodies.
- The general plane path gets the 2-point inner and 3-point outer rules.
- Independent of the other cards.

### Sources

- papers/tessellation_mass_queries/Eberly-2002-polyhedral-mass-properties.txt, §2–5 (pp. 2–6), formulas (12)–(15) and the pseudocode. Verified.
- Mirtich 1996: PostScript only, not downloaded; notes/tessellation_mass_queries.md [M4].
- Steps 4–5: my derivation.

---

## 6. HLR: quantitative invisibility propagated along each line (Appel)

**Problem.** `hlr` decides each piece's visibility with its own ray tests: one per piece, three
on long pieces, and bisection where they disagree. Visibility can only change at a few kinds of
points, so one test per line plus counting is enough.

### Why it is fast

- Ray tests drop from (1–3 per piece + bisection) to about 2 per line: one test and one check.
- Bénard & Hertzmann: QI "greatly decrease[s] the number of ray tests required, and improve[s]
  accuracy" (ch. 4 intro). With full propagation, "at most one ray test is necessary for each
  connected component" (§4.7).
- No measured numbers in the sources, and cadrs' README gives no HLR timings. Bénard &
  Hertzmann also caution that chains add "considerable implementation complexity for a
  questionable amount of benefit" (§4.6) when ray tests are cheap. **Profile `hlr` first** to
  confirm ray tests dominate.

### Algorithm

Definitions (Bénard & Hertzmann §4.7, verified): the QI of a point is the number of surface
sheets between it and the eye. Visible ⇔ QI = 0. Along a line, QI changes only at singular
points: image-space crossings behind a visibility-indicating line, crossings on the surface, and
curtain folds (§4.3).

1. **Events per line L.** Use cadrs' `Src` lines (edges and outlines). `crossings` already gives
   the split parameters. For each crossing of L with another line N at image point X, compare
   depths at X; the crossing is refined by Newton, so both 3D points are known.
   - If L is the near line: Δ = 0. Bénard step 5: "The near edge does not need to be split
     since its visibility will not change."
   - If L is the far line: $\Delta = c_N(s_\text{after}) - c_N(s_\text{before})$. Here
     $c_N(s)\in\{0,1,2\}$ counts the surface sheets at N that lie on image side s of N (my
     formulation of Bénard's rules):
     - Projected side function: $\text{side}(\mathbf v) = \operatorname{sign}(T_N \times \mathbf v)$
       (2D cross product), with $T_N$ N's projected tangent at X.
     - L's sides: $s_\text{before} = \text{side}(-T_L)$ and $s_\text{after} = \text{side}(T_L)$.
     - Sharp or smooth edge N: for each adjacent face f, the interior direction is
       $d_f = \hat n_f \times T_\text{coedge}$ (loops run counter-clockwise about the outward
       normal, so the face lies on the left). It contributes 1 to side(P(d_f)).
     - Free boundary of a sheet: one face.
     - Outline (silhouette) on face f: both local sheets project to the same side, so c = 2 on
       that side and 0 on the other. Find the side by stepping ±ε along ∇g in uv and projecting
       $S(uv \pm \varepsilon) - S(uv)$.
   - Checks against Bénard: a contour in front gives Δ = ±2 ("the occluded far edge must have QI
     of q + 2"), a boundary ±1 ("q + 1"). An edge between two faces seen on opposite sides gives
     Δ = 0.
   - Cusps of L itself (card 7): **reset** events. Bénard: "the far edge will have a QI of at
     least q + 1" and may need a local overlap test.
   - Collinear overlaps, faces seen edge-on (P(d_f) ∥ T_N) and coincident crossings: also reset
     events.
   - Lines of one body piercing faces of another body change QI without an image crossing. Add
     line–face intersection events for lines whose 3D box overlaps another body's face boxes
     (assemblies). cadrs' bisection handles this case today.
2. **One counted ray test per run between reset events**, at the midpoint of the run's longest
   piece and away from crossings. Count all face crossings beyond the skin, front and back,
   instead of stopping at the first hit (`Occluders::visible` returns a bool today). That gives q.
3. **Propagate:** $q \leftarrow q + \Delta$ at each event along L. Pieces with q = 0 are
   visible. Free constraints (§4.7): concave outline pieces (card 7) have q ≥ 1; lines with both
   faces back-facing are hidden.
4. **Check (voting, §4.6).** Do a second counted ray test on the run's last piece. If its q
   differs from the propagated value, fall back to today's per-piece tests for that line.
   Bénard warns that one wrong test can make "a large silhouette curve that disappears" (§7.6.1).
5. Keep today's post-processing: hidden-on-visible removal, overlap ties, merging.

### Data structures

- Per line: a sorted event list (t, Δ or reset) and runs between resets.
- A per-crossing record of (near line, far line, image point, the near line's side counts).
- `Occluders` gets a counting query.

### Failure modes and error bounds

- Propagation is exact given correct Δs and one correct count.
- Errors come from grazing ray hits near outlines: a count of 0 or 2 sheets where it should be 1
  (Bénard §7.6: ray tests are "numerically unstable" on the contour). Place the test points away
  from outlines and crossings, and vote (step 4).
- Tangential crossings in 2D (nearly parallel projected curves) make the side function
  ill-defined. Treat |T_N × T_L| below `tol::ANGULAR` as a reset.
- OCCT's HLRBRep also does not remove superimposed lines (notes [H8]). cadrs already does.

### Fit in cadrs

- `hlr.rs`:
  - `crossings` returns the other line and depth per crossing.
  - `piece_visibility` becomes per-line propagation with the fallback.
  - `Occluders::visible` gets a counting variant.
- Uses card 7's cusp events.
- Independent of the mass and tessellation cards.

### Sources

- papers/tessellation_mass_queries/Benard-Hertzmann-2019-line-drawings-from-3D-models-tutorial.txt, ch. 4 intro (p. 33), §4.3 singular points (pp. 35–36), §4.6 curve-based algorithm and propagation (pp. 38–41), §4.7 QI rules (pp. 41–43), §7.5–7.6 (pp. 65–68). Verified.
- Appel 1967: paywalled; notes/tessellation_mass_queries.md [H1] only.
- The Δ = c(after) − c(before) formulation, reset events and line–face piercing events: my design, consistent with Bénard's rules.

---

## 7. Silhouettes: curtain folds (cusps) and concave outlines

**Problem.** Traced outlines are not split where their projection has a cusp, yet visibility
can change there. Pieces of an outline lying in a local concavity are always hidden but are ray
tested.

### Why it is fast

- Each cusp found is a split point the visibility pass no longer finds by three-point
  disagreement and bisection.
- Each concave piece needs no ray test.
- Cost: one scalar function per traced outline vertex, plus a few Newton steps per cusp.

### Algorithm

1. At each vertex of a traced outline (`Tracer` output in uv), compute the view direction w:
   - parallel view: w = d;
   - perspective: w = e − p.
2. Express w's tangential part in uv. Solve
   $\begin{bmatrix}E&F\\F&G\end{bmatrix}\begin{bmatrix}\alpha\\\beta\end{bmatrix}=\begin{bmatrix}S_u\cdot w\\S_v\cdot w\end{bmatrix}$
   with $E=S_u\cdot S_u$, $F=S_u\cdot S_v$, $G=S_v\cdot S_v$.
3. Cusp function: $C = L\alpha^2 + 2M\alpha\beta + N\beta^2$, with $L = S_{uu}\cdot\hat n$,
   $M = S_{uv}\cdot\hat n$, $N = S_{vv}\cdot\hat n$ and $\hat n$ the outward normal (face sense
   applied). C is $II(w,w)$, the radial curvature times $|w|^2$.
   - Hertzmann & Zorin define $C(p) = \kappa_1(v\cdot w_1)^2 + \kappa_2(v\cdot w_2)^2$ (§4.2),
     which equals $II(v,v)$ by Euler's formula, so principal directions are not needed.
4. Where C changes sign between consecutive vertices, solve $(g, C) = (0, 0)$ for the cusp.
   - Bisect along the polyline, pulling each midpoint onto g = 0 with the existing `to_zero`.
   - Or use Newton on the 2 × 2 system, with C's gradient by central differences (C needs third
     derivatives otherwise).
   - Bénard & Kass's mesh version bisects triangles with sign changes in both functions (§7.5);
     along an already traced curve the 1-D search is enough.
5. Split the outline at each cusp. With an outward normal, a convex outline (sphere-like) has
   C < 0. **Pieces with C > 0 lie in a concavity and are hidden** (q ≥ 1) without a ray test.
   Bénard: "A contour can only be visible when it has a positive radial curvature" in their sign
   convention (§7.4). Curtain folds are exactly where C = 0 on the outline (§7.5, after
   Koenderink).
6. Skip faces whose C is identically 0 along the outline (cylinders, cones and extrusions: w's
   tangential part runs along the ruling, a zero-curvature direction). Quadric outlines are
   already exact in `silhouette.rs`; spheres have C < 0 everywhere.

### Data structures

- A per-outline-vertex value C.
- A cusp list (uv, parameter) per outline.
- A per-piece flag for locally hidden.

### Failure modes and error bounds

- Near-parabolic outlines (C ≈ 0 over a stretch) give many spurious sign changes. Require
  |C| > `tol::REL` × (|S_uu| + |S_vv|) × |w|² on both sides before declaring a cusp.
  Hertzmann & Zorin note that their construction avoids "introducing many spurious cusps"
  (§4.2).
- Higher-order singularities (swallowtails) are not generic and would show as two close cusps;
  treat them as reset events in card 6.
- Curtain folds on edges (not outlines) need no special handling: they coincide with an
  edge–outline intersection (Bénard §7.5, "Curtain folds on other curves").

### Fit in cadrs

- `silhouette.rs`: after `refine`, compute C and split the chains in `clip_chain`. `Silhouette`
  gets cusp parameters and per-piece hidden flags.
- `hlr.rs`: `face_outlines` passes them on as split points and known-hidden pieces.
- Feeds card 6.

### Sources

- papers/tessellation_mass_queries/Hertzmann-Zorin-2000-illustrating-smooth-surfaces.txt, §4.2 cusp function (p. 3) and §4.4 segment split points (pp. 3–4). Verified.
- papers/tessellation_mass_queries/Benard-Hertzmann-2019-line-drawings-from-3D-models-tutorial.txt, §7.4 radial curvature and visibility sign (pp. 63–65), §7.5 curtain folds and detection (pp. 65–67). Verified.
- The II(w,w) form, the uv solve and the sign with outward normals: my derivation.

---

## 8. Tessellation: certified chord bound (Filip–Magedson–Markot) for seeding and early accept

**Problem.** `steps` estimates interior spacing from a 7 × 7 sample of |S_uu·n| and |S_vv·n|.
It ignores S_uv and can under-sample a curvature peak. The refinement then measures every
triangle against the surface. A certified bound gives safe seeding and lets most triangles skip
the measured check.

### Why it is fast

- The bound is computed from control points once per span, with no surface evaluations.
- Triangles whose bound is under the chord tolerance are accepted with no evaluation.
- Only triangles that fail the bound get today's measured check, so results never get coarser
  than today.
- No measured numbers in our sources. FMM's paper is paywalled.

### Algorithm

1. **Rectangle bound (FMM).** For S ∈ C² on a cell of size Δu × Δv cut into two triangles, the
   piecewise-linear interpolant L satisfies
   $$\|S-L\| \le \tfrac18\left(\Delta u^2 M_{uu} + 2\,\Delta u\,\Delta v\,M_{uv} + \Delta v^2 M_{vv}\right),$$
   with $M_{uu}=\sup\|S_{uu}\|$, $M_{uv}=\sup\|S_{uv}\|$ and $M_{vv}=\sup\|S_{vv}\|$ over the
   cell.
   - The formula is from the notes (from memory; FMM 1986 is paywalled).
   - I checked it numerically on 3000 random vector-valued quadratic maps with random cell
     sizes. The largest error/bound ratio was 0.9995, so the 1/8 constant holds and is tight.
2. **Any triangle** (my derivation, for CDT triangles near trims). For a uv triangle T with
   vertices $p_i$ and barycentrics $\lambda_i$, Taylor's theorem along each segment $p\to p_i$
   gives
   $$\|S(p)-L(p)\| \le \tfrac12\sum_i\lambda_i\,D_2(p_i-p),$$
   where $D_2(e) \le M_{uu}e_u^2 + 2M_{uv}|e_ue_v| + M_{vv}e_v^2$. Two corollaries:
   - (a) $\le \tfrac12\hat M\rho^2$, where ρ is the radius of T's smallest enclosing circle and
     $\hat M = \lambda_{\max}\begin{bmatrix}M_{uu}&M_{uv}\\M_{uv}&M_{vv}\end{bmatrix}$. This uses
     $\sum_i\lambda_i|p_i-p|^2 = \sum_i\lambda_i|p_i-c|^2 - |p-c|^2$ with c the enclosing
     centre.
   - (b) $\le \tfrac12(M_{uu}\Delta u_T^2 + 2M_{uv}\Delta u_T\Delta v_T + M_{vv}\Delta v_T^2)$,
     with T's uv bounding-box extents.

   Use the smaller. Both work in cadrs' metric-scaled uv after rescaling the Ms:
   $M_{uu}' = M_{uu}/m_u^2$, $M_{uv}' = M_{uv}/(m_um_v)$, $M_{vv}' = M_{vv}/m_v^2$.
3. **Derivative bounds per span**, standard B-spline derivative control points (not in our
   papers):
   - $P^{(1,0)}_{ij} = p\,(P_{i+1,j}-P_{ij})/(u_{i+p+1}-u_{i+1})$, applied again with p − 1 for
     $S_{uu}$, once in each direction for $S_{uv}$, and likewise in v.
   - By the convex hull property, $M_{uu}\le\max\|P^{(2,0)}_{ij}\|$ over the control points that
     influence the span.
   - Rational surfaces (my derivation, quotient rule). Translate to the span's control-point
     centre c, so that $X' = X - Wc$ and $r = \max\|P_{ij}-c\|$. Then
     $\|S_u\|\le(\max\|X'_u\| + \max|W_u|\,r)/W_\text{min}$ and
     $\|S_{uu}\|\le(\max\|X'_{uu}\| + \max|W_{uu}|\,r + 2\max|W_u|\,\|S_u\|_\text{max})/W_\text{min}$.
     $S_{uv}$ and $S_{vv}$ follow the same pattern.
   - Analytic surfaces: exact closed-form maxima (card 9).
4. **Seeding.** Per span (or per face, taking the maximum), choose $r = \Delta v/\Delta u =
   \sqrt{M_{uu}/M_{vv}}$, which balances the pure terms (my choice). Solve
   $\Delta u^2(M_{uu}+2rM_{uv}+r^2M_{vv}) = 8\varepsilon$.
   - Keep cadrs' angle rule (`0.5 * ang / k`) and take the smaller step.
   - This replaces `steps`' sampled estimate.
5. **Early accept in refinement.** In `split_point`, first compute the bound of step 2 for the
   triangle. If it is ≤ chord, skip the measured chord test. Otherwise run today's test. Never
   reject on the bound alone.
6. Insertion stays as it is. Shewchuk's circumcentre insertion with ratio bound B gives a
   minimum angle of $\arcsin\frac1{2B}$: B = √2 gives 20.7° (Ruppert), B = 1 gives 30° (Chew).
   cadrs already maps `max_aspect` to these through spade. Its boundary segments are never split
   (they are shared with neighbouring faces), so Ruppert's guarantee does not hold next to the
   boundary. That is accepted today.

### Data structures

- A per-span table of $(M_{uu}, M_{uv}, M_{vv})$ from derivative control points, built once per
  face.
- A lookup from a triangle's uv box to the spans it overlaps (take the maximum over them).

### Failure modes and error bounds

- The bound is certified for exact arithmetic on the parametric error ‖S(p) − L(p)‖. That is ≥
  the distance from L(p) to the surface, so it is conservative.
- It over-refines where S_uu is mostly tangential (non-uniform parametrisation, uneven knots,
  whose derivative control points divide by small knot gaps). The paper's limitation (notes
  [T1]): "global over the domain … over-refines where curvature is concentrated". Hence the
  early-accept use only.
- Rational bounds are loose when weights vary strongly (notes [T1]).

### Fit in cadrs

- `tess.rs`: `steps` (replace the 7 × 7 sample), `seed_grid` (per-span steps), and
  `split_point` (early accept).
- `nurbs.rs` gains derivative control-point bounds per span; card 10 can reuse them.

### Sources

- FMM bound: notes/tessellation_mass_queries.md [T1] (from memory; paywalled). Constant checked numerically by me.
- Delaunay refinement constants: papers/tessellation_mass_queries/Shewchuk-2002-Delaunay-refinement-triangular-mesh-generation.pdf, §2 (printed p. 5). Verified on the page image; the text layer lost the symbols.
- Triangle bound, rational bound and the aspect choice: my derivation.

---

## 9. Tessellation: analytic interior nodes and closed-form checks

**Problem.** Cylinders, cones, spheres and tori are seeded by the same sampled grid as B-splines
and checked by measuring triangles against the surface. Their geometry allows exact node
placement (OCCT's analytic "range splitters") and exact deviation formulas.

### Why it is fast

- No sampling for steps, and no surface evaluations in the refinement check.
- Sphere and torus rows adapted to the local radius need fewer points. On a full sphere, rows of
  equal-sized triangles need about π/2 fewer points than a constant-Δθ latitude–longitude grid
  ($\int\cos\varphi\,d\varphi = 2$ against π).
- OCCT's advantage at mesh parity is attributed to these splitters (README "Mass properties…"
  section; notes [T11]). No measured numbers.

### Algorithm

Let ε be the chord tolerance and α the normal-angle limit (`TessParams.angle`; cadrs' edges use
α/2, so match that).

1. **Cylinder of radius r.**
   - Seeding: generators only, at $\Delta\theta = \min(2\arccos(1-\varepsilon/r),\ \alpha)$. No
     rows along z unless `max_edge` or `max_aspect` ask for them.
   - Check for any triangle whose vertices span angle $\Delta\theta_T \le \pi$:
     $\text{dev} \le r(1-\cos(\Delta\theta_T/2))$.
   - Proof (mine): the triangle lies in the convex hull of the arc strip, whose cross-section's
     smallest radius is the chord's, $r\cos(\Delta\theta_T/2)$.
2. **Cone with half-angle α_c.**
   - Seeding: generators at the cylinder rule with $r = \rho_\text{max}$, the face's largest
     radius.
   - Check: $\text{dev} \le \rho_\text{max}(T)\,(1-\cos(\Delta\theta_T/2))\cos\alpha_c$.
   - Proof (mine): points of the hull of generator segments sit at radius ≥ cos(Δθ/2) times
     the cone's radius at their height, and the normal distance is the radial deficit × cos α_c.
3. **Sphere of radius R.**
   - Check, for any triangle with vertices on the sphere: $\text{dev}\le R-\sqrt{R^2-\rho_c^2}$,
     with $\rho_c$ the triangle's 3D circumradius. The plane of the triangle cuts the sphere in
     its circumcircle. The bound is exact when the circumcentre lies inside the triangle.
   - Seeding: target side $h=\sqrt{3(2R\varepsilon-\varepsilon^2)}$ (equilateral, circumradius
     $h/\sqrt3$). Rows at $\Delta\varphi = \min(h/R, \alpha)$, and per row
     $n_\theta = \lceil \Delta\theta_\text{face}\,R\cos\varphi / h\rceil$.
   - Poles go through `tess`'s existing pole handling (`pole_points`).
4. **Torus (R, r).**
   - Exact derivative maxima over a row $[v_0, v_1]$: $M_{uu} = R + r\max\cos v$,
     $M_{uv} = r\max|\sin v|$, $M_{vv} = r$.
   - Rows at $\Delta v$ and per-row $\Delta u$ from card 8's rectangle bound with these values.
   - Check with card 8's triangle bound and these Ms.
5. Feed the nodes to the CDT as today. The CDT does not need a tensor grid, so per-row counts
   are fine.

### Data structures

None new: per-type closed forms in `Domain`.

### Failure modes and error bounds

- Steps 1–3 are exact upper bounds (my proofs). Step 4 is conservative (card 8).
- Faces spanning more than π in one triangle are excluded: the hull argument needs
  $\Delta\theta_T\le\pi$. The angle limit already enforces much less.
- Spheres near the poles: rows with $n_\theta = 1$ or 2 must defer to the pole logic.

### Fit in cadrs

- `tess.rs`: `steps` and `seed_grid` branch on `Surface` kind. `split_point`'s chord test uses
  the closed forms for these kinds.
- Independent of card 8, but shares its torus bound.

### Sources

- OCCT range splitters ("cylinder/cone/sphere/torus: analytic iso-lines"): notes/tessellation_mass_queries.md [T11] and notes/kernels_recent.md (BRepMesh). Notes only (OCCT mesh guide, not a local paper).
- Deviation formulas, proofs and seeding rules: my derivation.

---

## 10. Tessellation: kd-tree seeding for B-spline faces (Balázs–Guthe–Klein)

**Problem.** `seed_grid` steps a B-spline face uniformly with the face's worst curvature, which
over-refines faces whose curvature is concentrated (blade hubs, fillets fitted as B-splines).
An adaptive kd-tree on the control net places nodes only where needed and splits only in the
direction that needs it.

### Why it is fast

Balázs et al. measured on the Golf car body, 0.2 mm accuracy:

- kd-tree against quadtree: 97.3 s against 348.3 s, and 464,354 against 2,058,739 triangles
  (Table 4).
- With the conservative bound only: 796,438 against 1,511,056 triangles (Table 3).
- Ventilation console: 11.6 s against 64.9 s, and 29,113 against 562,949 triangles (Table 4).

A quadtree is closer to cadrs' uniform grid than the kd-tree is, so expect a similar direction
of gain on B-spline-heavy models. The paper's numbers include their own trimming and are not
cadrs measurements.

### Algorithm

1. Extract the face's B-spline surface into Bézier patches per knot cell (knot insertion). Work
   per patch with control points $P_{ij}$, $0\le i\le m$, $0\le j\le n$.
2. For a kd-tree node (a sub-patch from de Casteljau subdivision):
   - $\varepsilon_\text{bilin} = \max_{ij}\|P_{ij} - \tilde S(i/m, j/n)\|$, with $\tilde S$
     the bilinear interpolant of the four corner control points:
     $\tilde S(a,b) = (1-a)(1-b)P_{00} + a(1-b)P_{m0} + (1-a)b\,P_{0n} + ab\,P_{mn}$.
   - $\varepsilon_\text{cons} \le \varepsilon_\text{bilin} + \tfrac14\|P_{00}-P_{m0}-P_{0n}+P_{mn}\|$.
     The second term accounts for the two triangles not being the bilinear patch.
   - The printed $\tilde S$ pairs a with the second index (a typo or convention clash); the
     corner-bilinear form above is the unambiguous reading.
3. If $\varepsilon_\text{cons} > \varepsilon$, split. Find the control point $(k, l)$ with the
   largest $\|P_{kl}-\tilde S(k/m,l/n)\|$. Split at parameter $k/m$ in u or $l/n$ in v, choosing
   "the one for which the line subdividing the kd-tree node is closer to" that point (§5,
   Fig. 3). Recurse.
4. The leaves' corners are the interior seed nodes for cadrs' CDT. cadrs keeps its own trimming
   (CDT with constrained boundary), so the paper's half-edge trimming (§6) is not needed.
5. Optional, not guaranteed: the paper's "approximate" error replaces $P_{ij}$ by
   $S(\alpha_i, \beta_j)$ at the control points' parameter values (Greville-like). It gave 42%
   fewer triangles than the conservative bound for 1% more time (Table 3). Use it only together
   with card 8's measured fallback.

### Data structures

- A kd-tree of sub-patches: control net, uv box and split axis.
- A Bézier extraction cache per face.

### Failure modes and error bounds

- The conservative bound is guaranteed for polynomial (non-rational) patches. It relies on the
  linear precision of the Bernstein basis, which rational weights break, so use card 8's bound
  for rational faces.
- It overestimates, "sometimes significant[ly]" (§5).
- Deep trees on degenerate patches (collapsed rows): cap the depth and fall back to the grid.
- The paper also gives a convex-hull bound for 3D trim curves,
  $\varepsilon_\text{line}\le(1-2^{-(n-2)})\max_i\|(P_i-P_0)-\lambda(P_n-P_0)\|$, with γ = 3/4 of
  the tolerance for approximation and the rest for simplification. **As printed the factor is 0
  for n = 2**, which cannot be right: a quadratic Bézier deviates from its chord by up to half
  its middle control point's offset. Check Kahlesz et al. 2002 before using it for edge
  sampling. cadrs' measured edge refinement (`refine_polyline`) is unaffected.

### Fit in cadrs

- `tess.rs`: `seed_grid` for `Surface::Nurbs` (non-rational) takes the kd-tree leaves' corners
  in place of the uniform grid. `steps` is not used for those faces.
- `nurbs.rs`: Bézier extraction and de Casteljau subdivision of a surface patch, if not already
  present.
- Shares derivative bounds with card 8.

### Sources

- papers/tessellation_mass_queries/Balazs-Guthe-Klein-2004-efficient-trimmed-NURBS-tessellation.txt, §5 "NURBS Surfaces" (bounds and split rule, p. 3–4), Tables 2–4 (p. 6). Verified against the text and the PDF page 3 image (the printed $\tilde S$ and $\varepsilon_\text{line}$ as above).
- notes/tessellation_mass_queries.md [T7].

---

## Considered and not carded

- **Gonzalez-Ochoa, McCammon & Peters 1998 (exact Bernstein-product moments).** Card 1's Gauss
  rules are already exact for the same (non-rational) patches with fewer operations to write.
  As an oracle a 20-point Gauss rule does the same job. The method description in the notes is
  from memory; the paper is paywalled.
- **Mørken products / Patrizi–Sestini 2026.** An exact B-spline-form route to the same
  integrals. Not needed for mass properties.
- **Krishnamurthy & McMains 2010, GPU moments.** It weights trimmed cells by the fraction of
  trim-texture samples inside (papers/kernels_recent/Krishnamurthy-2010-GPU-moments-trimmed-NURBS-solids.txt,
  §5). That is less accurate than cadrs' exact Green path. cadrs' existing GPU job path stays
  compatible with cards 1–4.
- **GPU CDT and refinement (Qi–Cao–Tan 2012, Chen–Qi–Tan 2017).** Pays off at millions of
  points, not CAD faces (notes [T13]).
- **Hertzmann–Zorin dual-surface octree** (§4.3, silhouettes for many views from one
  precomputation) and **Seong–Elber–Cohen 2006 visibility charts.** These are for many views
  per model. Drawing views are few, and the 6-variable solver is costly.
- **Spatial index for 2D crossings.** `hlr` already finds crossing pairs on a 2D grid, which is
  the improvement the README's item 18 asks for over OCCT's all-pairs HLRBRep. Bénard §4.6
  suggests a Bentley–Ottmann sweep, O(n log n), only if the grid degrades on long lines.
