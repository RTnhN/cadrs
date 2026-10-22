# Fillet, chamfer and vertex-blend algorithm cards for cadrs_kernel

Goal: close the fillet gap (9.2x slower than OCCT 7.8 on OCCT's blend cases, 10.3x on SMLib's;
`crates/cadrs_kernel/README.md`, "Performance") and remove slow fallbacks to the subtracted tool
body (`blend::blend`). Only algorithms with concrete steps are here. History, surveys, PDE and
implicit blends, Dupin cyclides and G2 sections are left out: none of them is on the measured
slow path.

Nobody has profiled the fillet yet, so the ranking is a judgement, not a measurement. Two cheap
measurements decide it:
- Ball solves per edge. `fillet.rs` already counts them (`Budget`, cap `BALL_SOLVES` = 1e6).
  OCCT's walker takes about 20 sections per straight edge and about 22 per full turn of a curved one (card 1). If cadrs takes many more, cards
  1 to 3 are the payoff.
- Fallback rate. `blend::fillet` runs the local fillet, then `local_problem` (a full
  `Body::check`), and on any failure builds tool bodies and runs a boolean. A fallback case pays
  for both. Running the cases with `CADRS_LOCAL_FILLET_ONLY` set (`blend.rs`) counts them. If
  fallbacks are common, cards 5, 6 and 9 come first.

"Verified" below means checked against the paper text in `research/papers/` or against OCCT
7.8.0 source fetched for this write-up (`BlendFunc_ConstRad.cxx`, `Blend_Walking_2.gxx`,
`Blend_Walking_4.gxx`, `BRepBlend_SurfRstConstRad.cxx`, `ChFi3d_FilBuilder.cxx`,
`ChFi3d_Builder.cxx`, `ChFi3d_Builder_0.cxx`, `ChFi3d_Builder_1.cxx`,
`BRepFilletAPI_MakeFillet.cxx`, all at
`https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V7_8_0/src/<package>/<file>`).
"Notes only" means the card relies on `notes/blends.md` or `notes/kernels_recent.md` and was not
re-checked.

Two corrections to the notes, from the OCCT source:
- The step angle test compares the *squared* cosine with 0.98, so the limit is
  $\arccos\sqrt{0.98} \approx 8.1°$, not 11.5°. The 2D test uses $\cos^2 \ge 0.88$, about 20.3°.
- The sagitta estimate is $|\hat t_{prev}-\hat t_{cur}|^2\,|c|^2/64$ ($|c|$ the chord length,
  squared), which is the square of the usual arc sagitta $|c|\,\Delta\theta/8$. The notes drop the
  square on $|c|$.

## Ranked summary

| Rank | Card | Payoff | Size | Risk |
|---|---|---|---|---|
| 1 | Walked sections with tangent predictor and OCCT step control | High if ball solves dominate: fewer stations, 1 to 3 Newton iterations each | Medium | Low |
| 2 | Hermite strip and rail p-curves from section derivatives | High: no "truth" solves for the error check, no 48-point p-curve projections | Medium | Medium |
| 3 | Cheap Newton iterates (one evaluation per iterate, resolution-based stop, Gauss then SVD) | Medium: constant factor on every ball solve | Small | Low |
| 4 | Exits found during the walk, re-framed by an inverse solve | Medium: replaces sampling of every boundary curve | Medium | Medium |
| 5 | Cliff-edge blend functions (surface/edge and edge/edge) | Medium to high on overflow cases: they stay local instead of falling back | Medium | Medium |
| 6 | Trimming path before topology edits (HP/Opitz) | Medium to high on growing/shrinking neighbour faces: removes fallbacks | Large | Medium to high |
| 7 | Fail-fast fit test from offset regularity | Low to medium: stops doomed local attempts early | Small | Low |
| 8 | Pinch split where the contact curves meet | Low to medium: removes a failure (and its fallback) | Small | Low |
| 9 | Corner ladder with a Generalized Coons patch | Low to medium: deterministic n-edge and mixed corners, fewer fallbacks | Medium | Medium |

---

## 1. Walked sections with tangent predictor and OCCT step control

Problem: place the fillet's sections (ball positions) along the edge with as few ball solves as
possible, and start each Newton solve close enough that it converges in one to three iterations.

### Why it is fast
- Stations are chosen once, by a local test on quantities the solve already produced (contact
  points and their tangents). No second pass of extra solves checks the result.
- The predictor uses the exact tangent of the solution curve, so the corrector starts within
  $O(h^2)$ of the answer.
- Step sizes in OCCT: the first and largest step is 5% of the guide's parameter length, and the
  sagitta limit is 5% of the radius. On a straight edge between planes or cylinders the sagitta is
  near zero, so every step grows to the cap: about 20 sections per edge. On a circular rail the
  chord makes half the turning angle with each end tangent, so the 8.1° limit allows about 16°
  per step: about 22 sections per full turn, fewer if the sagitta limit is looser.
- No timings are published (notes, KQ5). The constants are the only sanity reference.

### Algorithm
Unknowns $X=(u_1,v_1,u_2,v_2)$, the contact parameters on faces 1 and 2. Guide (spine) $g(t)$,
plane normal $n=g'/|g'|$. $P_i=S_i(u_i,v_i)$, $N_i=\partial_u S_i\times\partial_v S_i$ (not
normalised). The in-plane unit normal is
$m_i = (N_i-(n\cdot N_i)\,n)/|n\times N_i|$. Signed radii $r_1,r_2$ ($\pm R$, sign by which side
of each face the ball lies; OCCT's `choix`).

Equations (verified, `BlendFunc_ConstRad::ComputeValues`):
- $E_1 = n\cdot\big(\tfrac{P_1+P_2}{2}-g(t)\big) = 0$ (midpoint of the contacts in the section
  plane).
- $E_{2..4} = (P_1 - r_1 m_1) - (P_2 - r_2 m_2) = 0$ (both contacts reach the same centre).

Jacobian $J=\partial E/\partial X$ (4x4, analytic):
- Row 1: $\tfrac12\,(n\cdot\partial_{u_1}P_1,\ n\cdot\partial_{v_1}P_1,\ n\cdot\partial_{u_2}P_2,\ n\cdot\partial_{v_2}P_2)$.
- Rows 2 to 4, column $x\in\{u_1,v_1\}$: $\partial_x P_1 - r_1\,\partial_x m_1$; column
  $x\in\{u_2,v_2\}$: $-(\partial_x P_2 - r_2\,\partial_x m_2)$.
- $\partial_x m = (\partial_x w - m\,(m\cdot\partial_x w))/|w|$ with $w=N-(n\cdot N)n$,
  $\partial_x w = \partial_x N - (n\cdot\partial_x N)\,n$,
  $\partial_u N = S_{uu}\times S_v + S_u\times S_{uv}$, $\partial_v N = S_{uv}\times S_v + S_u\times S_{vv}$.
  (OCCT writes the same derivative in an expanded form with $|n\times N|^{-3}$; this form is
  equivalent.)

Derivative in the guide parameter, $D=\partial E/\partial t$ (verified):
- $D_1 = n'\cdot\big(\tfrac{P_1+P_2}{2}-g\big) - |g'|$, with $n' = (g''-n\,(n\cdot g''))/|g'|$.
- $D_{2..4} = -r_1\,\partial_t m_1 + r_2\,\partial_t m_2$, where $\partial_t m_i$ comes from $n'$
  through $w_i = N_i-(n\cdot N_i)n$: $\partial_t w_i = -(n'\cdot N_i)\,n-(n\cdot N_i)\,n'$.
- Variable radius (OCCT `BlendFunc_EvolRad`, notes only): $r_i=r_i(t)$ from the law, which adds
  $-r_1'\,m_1 + r_2'\,m_2$ to $D_{2..4}$. This addition is my derivation, not checked against
  EvolRad's source.

Tangent of the solution curve: solve $J\,X' = -D$ (card 3 for the solver). Then
$P_i' = \partial_u S_i\,u_i' + \partial_v S_i\,v_i'$ are the rails' 3D tangents and
$(u_i',v_i')$ their uv tangents. If the solve fails the point is a "tangency point": it has no
tangent.

Walk (verified, `Blend_Walking_4.gxx` `InternalPerform`, `Blend_Walking_2.gxx` `TestArret` and
`CheckDeflection`, `ChFi3d_FilBuilder.cxx` `SimulParams`):
1. `pasmax` $= 0.05\,(t_{last}-t_{first})$. `fleche` $= 0.05\,R$; for a radius law, $R$ is the
   largest of 6 samples at $t_{first}+k\,0.2\,(t_{last}-t_{first})$, $k=0..5$.
2. First step: `pasmax`, or the last step of the existing line when continuing one; never below
   $100\,\text{tolgui}$. If $t+\text{step}$ passes the bound, step $=(\text{bound}-t)/2$.
3. Guide check: if the guide tangent turns by more than $\cos^2 < 0.88$ (about 20.3°) over the
   step, halve the step and retry (no solve).
4. Predictor: $X_0 = X_{prev} + \Delta t\,X'_{prev}$ (uv parts only). If $X_0$ leaves the
   surfaces' parameter bounds, or the previous point had no tangent, use $X_{prev}$.
5. Corrector: Newton on $E(X;t)=0$, at most 30 iterations, inside the parameter bounds. No
   convergence: step too large.
6. Classify both contacts in their faces (uv point-in-face). OUT or ON on either: card 4. If it
   is the walk's first point, treat as step too large.
7. Accept test, per rail $i$, with chord $c=P_{i,cur}-P_{i,prev}$ and the 3D rail tangents:
   - $|c| \le 0.01\,\text{tol3d}$: same point (stop the walk).
   - $c\cdot T_{prev} < 0$: backward (treated as too large; sets a "rebrousse" flag).
   - $\cos^2(c,T_{prev}) < 0.98$ or $\cos^2(c,T_{cur}) < 0.98$ or $c\cdot T_{cur}<0$: too large.
   - Optional 2D test on the uv step against the uv tangent: $\cos^2<0.88$: too large; both
     $|\Delta u|,|\Delta v|$ under the surface resolution of tol3d: same point.
   - Sagitta $f=|\hat T_{prev}-\hat T_{cur}|^2\,|c|^2/64$. $f>\text{fleche}^2$: too large.
     $f\le 0.25\,\text{fleche}^2$: too small.
   - Combine the two rails: any too large: reject. Else any OK: accept. Else both too small:
     accept and grow. Else: same point (stop).
8. Step update: reject: step $/= 2$; if $|\text{step}| < \text{tolgui}$, end the walk at the last
   accepted point. Accept (OK): keep the step. Accept (too small): step $=\min(1.5\,\text{step},
   \text{pasmax})$. Land exactly on the bound when within tolgui of it.
9. After re-framing on a boundary (card 4), the deflection test is replaced by a length test:
   reject if any $|\Delta u|$ or $|\Delta v|$ exceeds 5% of that surface's parameter range.

Tolerances (verified, `ChFi3d_Builder_1.cxx` constructor):
`SetParams(Ta, tolesp=1e-4, tol2d=1e-5, tolapp3d=1e-4, tolapp2d=1e-5, fleche=1e-3)`;
the walker's tol3d is `tolapp3d` and tolgui is the guide's resolution of `tolapp3d`
(`ChFi3d_FilBuilder.cxx`, `PerformFirstSection`).

### Data structures
- A walk point: $t$, $X$, $P_1,P_2$, centre, uv and 3D tangents of both rails, a "no tangent"
  flag. A walked line is a `Vec` of these, appended or prepended by direction.
- Walk state: step, `pasmax`, `fleche`, tol3d, tolgui, the two face domains for classification.

### Failure modes
- Newton fails or the point jumps backward: halve until the step is below tolgui, then stop. The
  caller sees a short line and fails the stripe ("CallPerformSurf : Path failed!").
- Rank-deficient $J$ (contacts coincide, faces tangent): the point has no tangent; the predictor
  reuses the previous solution, and the deflection tests skip the missing tangent.
- Start: OCCT's `StartSol` tries about $\max(10, 3\cdot\text{nbedges})$ guide samples, faces of the
  edge first, then neighbouring faces (notes only).

### Fit in cadrs
- Module: `fillet_geom` (solve) and `fillet::build_strip` (station placement).
- `ball_in_plane(f, k, p0, t, guess)` is already this system with the section plane through
  `p0` square to `t`; it needs a second entry point that also returns $X'$ (card 2 uses it).
- Replace the station refinement in `build_strip` (stations added until `strip_error` against
  extra "truth" balls passes) with the walk above. Keep `EdgeFillet::ball_from` for continuation.
- `fillet::trace_to` (end curves) uses its own step cap `min(total/48, r/6)`; the same
  accept/reject rules can replace that fixed cap.
- Prerequisite: count ball solves per edge on the OCCT and SMLib cases first.

### Sources
- OCCT 7.8.0 `BlendFunc/BlendFunc_ConstRad.cxx` (`ComputeValues`, `IsSolution`),
  `Blend/Blend_Walking_2.gxx`, `Blend/Blend_Walking_4.gxx`, `ChFi3d/ChFi3d_FilBuilder.cxx`
  (`SimulParams`), `ChFi3d/ChFi3d_Builder_1.cxx`: verified against source.
- `notes/blends.md` KQ1 and KQ5 (StartSol, EvolRad): notes only.
- `papers/blends/Filkins-1991-MIT-thesis-curvature-continuous-blends-linkage-curves.txt`,
  section 4.2 and p. 51: midpoint adaptive sampling of linkage curves (the scheme cadrs uses
  now). Verified; it gives no constants.

---

## 2. Hermite strip and rail p-curves from section derivatives

Problem: build the B-spline fillet surface and the rails' p-curves from the walked sections
without extra ball solves and without projecting sample points onto the faces.

### Why it is fast
- Each section already has its $t$-derivative from card 1's tangent solve, at the cost of one 4x4
  linear solve. With values and derivatives, a cubic Hermite span has error
  $O(h^4)$ and needs about half the stations of a value-only interpolant for the same tolerance
  (standard interpolation theory, not from a source).
- The rails' uv tangents come with the walk, so each rail p-curve is a Hermite curve in the
  face's uv. `fillet::face_pcurve` today samples 48 points (`const N: usize = 48`) and projects
  them, one foot-point Newton each.
- OCCT does the same in its approximation step: `BlendFunc_ConstRad::Section(P, Poles, DPoles,
  Poles2d, DPoles2d, Weights, DWeights)` returns each section's poles, 2D poles and weights *with
  their derivatives* along the guide, and `BRepBlend_AppSurf` fits through them (verified for the
  derivative output; the approximator's defaults, max degree 11, max 50 segments, C1, come from
  the notes).

### Algorithm
1. Walk the edge (card 1). Each accepted point $k$ gives $t_k$, $X_k$, $X'_k$, centre $C_k$,
   contacts $P_{1,k},P_{2,k}$.
2. Section poles (homogeneous, rational quadratic arc, as `fillet_geom::section_cps`):
   $H_0=(P_1,1)$, $H_1=(w\,Q, w)$, $H_2=(P_2,1)$, with $Q$ the meeting point of the arc's end
   tangents and $w=\cos(\phi/2)$, $\phi$ the arc's opening angle. If $\phi$ nears $\pi$, split
   into two arcs (OCCT `GetShape` picks the pole count from the largest angle `maxang`).
3. Pole derivatives: differentiate step 2 in $t$ using $P_i' $ from $X'$, $C'$ from
   $C = P_1 - r_1 m_1$ (so $C' = P_1' - r_1 m_1' - r_1' m_1$, with
   $m_1' = \partial_{u_1}m_1\,u_1' + \partial_{v_1}m_1\,v_1' + \partial_t m_1$), $Q'$ and $w'$ by
   the chain rule. A finite difference of step 2 at $t\pm\delta$ is the fallback; OCCT's own
   fallback uses $\delta=10^{-7}$ (`deltat = 1.e-7` in `BlendFunc_ConstRad.cxx`).
4. For every pole row $j$, build a $C^1$ piecewise cubic Hermite B-spline in $t$ through
   $(H_{j,k}, H'_{j,k})$: knots $t_k$ doubled, Bézier inner poles $H_{j,k}+\tfrac{h_k}{3}H'_{j,k}$
   and $H_{j,k+1}-\tfrac{h_k}{3}H'_{j,k+1}$, $h_k=t_{k+1}-t_k$. Interpolating in homogeneous
   coordinates keeps every station's section an exact circle.
5. Rail p-curves: on face $i$, the same Hermite construction on $(u_i,v_i)_k$ with derivatives
   $(u_i',v_i')_k$. These are exact at stations and $O(h^4)$ between them.
6. Error control without truth solves (my proposal, not from a source): estimate the span error
   from the jump of the third derivative between neighbouring spans,
   $e_k \approx \tfrac{h_k^4}{384}\,\big|H'''_{k+1}-H'''_{k}\big|/\bar h$, and split only spans
   over tolerance, solving one new section at the span midpoint. Card 1's sagitta test already
   keeps most spans under it.
7. Tolerance: OCCT approximates to `tolapp3d` = 1e-4 absolute and `tolapp2d` = 1e-5 (verified).
   cadrs's `fit_tol` is `1e-5 * (1 + size)` (`fillet::local_blend`): tighter than OCCT on small
   parts, looser above 9 mm. Keep cadrs's value; the payoff here is in station count and
   projections, not tolerance.

### Data structures
- Per walk point: the section's poles, weights and their $t$-derivatives; per rail: uv and uv'.
- The strip: one rational B-spline surface, degree 3 in $t$ (C1), degree 2 across, plus two
  Hermite uv p-curves.

### Failure modes
- Points without a tangent (card 1): fall back to a finite difference of neighbouring points, or
  to value-only interpolation over that span.
- Curvature jumps of a face along the edge (kinks): `nurbs_strip` already takes `kinks`; keep the
  surface only C0 there by not sharing the derivative across the kink.
- Arc angle near $\pi$ or a pinch (card 8): split.

### Fit in cadrs
- Replaces `fillet_geom::nurbs_strip` / `interp_h` (value-only interpolation) and the
  `strip_error` loop in `fillet::build_strip`. Replaces `fillet::face_pcurve` sampling for rails
  (`fillet::rail_pcurve`).
- Exact strips (`exact_strip`, cylinder and torus) are untouched.
- Prerequisite: card 1 (walk points with tangents).

### Sources
- OCCT 7.8.0 `BlendFunc/BlendFunc_ConstRad.cxx` (`Section` with `DPoles`, `deltat = 1.e-7`),
  `ChFi3d/ChFi3d_Builder_1.cxx` (tolerances): verified.
- `notes/blends.md` KQ1 (AppSurf degree 11, 50 segments, C1): notes only.
- `notes/blends.md` KQ1 inference: C3D's `MbFilletSurface` keeps the fillet procedural (two
  curves on surface plus radius); notes only.

---

## 3. Cheap Newton iterates

Problem: every ball solve (sections, crossings, corners) pays for surface derivatives and a
linear solve per iteration. Cut the cost per iterate and the iterations per solve.

### Why it is fast
- OCCT evaluates each surface once per iterate. `ComputeValues` caches the last $X$ and $t$
  and returns at once when `Value` and `Derivatives` ask for the same point (verified). With
  D2 needed for the Jacobian, that halves surface evaluations when values and Jacobian are
  requested separately.
- The stopping test is per-variable in parameter space: the surface's U and V resolution of
  tol3d (`GetTolerance`: `UResolution(Tol)`, `VResolution(Tol)`), with tol3d = 1e-4 in the
  fillet builder. cadrs stops some solves at $10^{-12}$ relative (for example the crossing solve
  in `fillet::crossings`). Newton is quadratic, so the gap is one or two iterations per solve,
  but it is paid on every solve.
- Iteration cap 30 for the walk, 35 for re-framing (verified for 30; 35 from notes).

### Algorithm
1. One routine computes $E$, $J$ and (when asked) $D=\partial E/\partial t$ from a single D2
   evaluation of each surface. Cache the last $(X,t)$ and its outputs.
2. Newton step: Gauss elimination with partial pivoting, pivot threshold $10^{-9}$. Verify the
   residual of the linear solve against the per-variable tolerances. If Gauss fails or the
   residual is too large, solve by SVD with relative singular value cut-off $10^{-6}$ and verify
   again (verified, `IsSolution`, `Section`).
3. Converged when each $|\Delta x_j|$ is below its resolution tolerance and
   $|E_1|\le$ tol3d, $|E_{2..4}|\le$ tol3d (`IsSolution`: $|E_1|\le Tol$,
   $E_2^2+E_3^2+E_4^2\le Tol^2$).
4. Keep cadrs's step damping (`fillet_geom::damp`, a 3D step cap); OCCT instead clamps to the
   parameter bounds.
5. Polish once at the end if the caller needs `CONFUSION`-level contact points (vertices on
   boundaries): one extra iteration from a converged point is cheap.

### Data structures
- A small solver struct per surface pair holding the cached point and derivatives. The
  surfaces' resolutions (`UResolution`, `VResolution`) per face, computed once.

### Failure modes
- Degenerate normal ($|N| < 10^{-15}$, a pole): OCCT computes the normal by a limit
  (`BlendFunc::ComputeNormal`); $|n\times N| < 10^{-15}$ sets the inverse norm to 1 and goes on
  ("unsatisfactory, but it is not necessary to crash"). Verified.
- SVD fallback also fails: report a tangency point, not an error.

### Fit in cadrs
- `fillet_geom::ball_in_plane`, `ball3`, `rail_crossings`, and the private Newton loops in
  `fillet.rs` (`crossings`, `first_contact`, `section_cut`, ...). `fillet_geom::offset` already
  derives offset derivatives from second derivatives; the change is caching and the stop rule.
- Prerequisite: a per-solve iteration count in the profile.

### Sources
- OCCT 7.8.0 `BlendFunc/BlendFunc_ConstRad.cxx` (cache in `ComputeValues`, `GetTolerance`,
  `IsSolution`, `Eps = 1.e-15`, Gauss `1.e-9`, SVD `1.e-6`), `Blend/Blend_Walking_4.gxx`
  (`math_FunctionSetRoot rsnld(Func, tolerance, 30)`): verified.
- `notes/kernels_recent.md` 1.1: OCCT 8.0 devirtualised elementary evaluation with POD results;
  notes only. Relevant if surface evaluation dominates the profile.

---

## 4. Exits found during the walk, re-framed by an inverse solve

Problem: find where a rail leaves its face (onto a neighbour, or at the fillet's end) without
sampling every boundary curve of the face.

### Why it is fast
- cadrs's `fillet_geom::rail_crossings` samples the other offset's signed distance along each
  boundary curve and refines each sign change by Newton. That cost grows with the boundary's
  length and curve count, and is paid whether or not the rail comes near the curve.
- OCCT classifies each new walk point's uv in its face (one point-in-face test per point) and
  solves on a boundary arc only when a point lands OUT or ON. Work is proportional to the number
  of actual exits.

### Algorithm
1. During card 1's walk, after each corrected point, classify $(u_1,v_1)$ in face 1 and
   $(u_2,v_2)$ in face 2 with tolerance $\min$ of the two parameter tolerances (verified,
   `domain->Classify`).
2. If a contact is OUT or ON face $i$: for each boundary arc of face $i$ near the last inside
   point, solve the inverse system with Newton. Unknowns (4): the arc parameter $s$, the guide
   parameter $t$, and $(u,v)$ on the other face. Equations: card 1's $E_1..E_4$ with
   $(u_i,v_i)=\gamma(s)$, $\gamma$ the arc's p-curve. Start from the predictor at the last inside
   point. Tolerance $0.1\,$tol3d, 35 iterations (notes only; that $X_2$ is the guide parameter
   is verified, `solrst1(2)` is compared with the walk parameter).
3. Accept the re-framed point only if it lies behind the current step within $10\,$tolgui:
   $(t_{cur}-t_{rst})\cdot\text{sens} \ge -10\,$tolgui. Otherwise halve the step and retry
   (verified).
4. Snap to a vertex when the arc parameter is within 0.4 of the arc span of the vertex, capped by
   the vertex tolerance; if the solve fails near an arc end, try the neighbouring arc (notes
   only).
5. Both contacts re-framed at the same $t$ within $10\,$tolgui: the state is "on both
   restrictions" (verified for the $10\,$tolgui test).
6. The walk ends at the re-framed point. If a recovery fails and the step is still above
   $2\,$tolgui, it counts as step too large; below, the walk stops there (verified).
7. Ends where the two contacts coincide within $2\,$tol3d are singular extremities (notes only).

### Data structures
- Per face: its boundary arcs as p-curves in uv and a fast uv point-in-face test (cadrs has
  `classify::in_face`).
- A re-framed point: arc index, arc parameter, guide parameter, other contact, vertex snap.

### Failure modes
- The walk steps over a thin face or a short arc: the next point is IN another face and the
  exit is missed. OCCT relies on its step limits; a cheap guard is to also test the chord
  between consecutive points against the face's arcs (a 2D segment test).
- Re-framed point ahead of the walk: wrong root; halve and retry.

### Fit in cadrs
- Replaces the sampling half of `fillet_geom::rail_crossings` and its callers `fillet::crossings`,
  `rail_exit`, `open_cliffs`, `seam_crossing` when the strip is walked. Keep the 3-unknown
  crossing Newton (on $t$ of the boundary, $u,v$ of the other face) as the inverse solve; it is
  the same system without the section plane.
- Prerequisite: card 1.

### Sources
- OCCT 7.8.0 `Blend/Blend_Walking_4.gxx` (classification, `Recadre` calls, $10\,$tolgui and
  $2\,$tolgui tests): verified.
- `notes/blends.md` KQ1 (`Blend_Walking_3.gxx`: $0.1\,$tol3d, 35 iterations, 0.4 vertex snap,
  $2\,$tol3d singular end, 1% guide extension): notes only.

---

## 5. Cliff-edge blend functions (surface/edge and edge/edge)

Problem: when the ball would roll over a sharp edge of one of its faces, keep the fillet local by
letting the ball ride on that edge, instead of failing or subtracting a tool body.

### Why it is fast
- It turns an overflow case into one more walked segment: 3 unknowns instead of 4, same walker.
- The alternative in cadrs, when the cut-by-faces handling (`fillet::cliff`) cannot settle the
  case (for example the `WHOLE_STRIP` error, "a fillet whose cut by a face crosses the whole
  strip"), is the tool body plus a boolean.
- Parasolid-based systems ship the same option ("Retain cliff edges": the ball "stays tangent to
  one of the original faces and is supported by the retained edge"), next to notch and smooth
  overflow (T-FLEX help, notes only).

### Algorithm
Surface/edge function (verified, `BRepBlend_SurfRstConstRad::Value`, `NbVariables` = 3):
unknowns $X=(u,v,w)$: contact $(u,v)$ on face $S$, and parameter $w$ on the edge curve
$\gamma(w)$ (the edge as a curve on the other face). Section plane: normal $n$, offset
$\theta=-n\cdot g(t)$.
- $F_1 = n\cdot S(u,v) + \theta = 0$
- $F_2 = n\cdot \gamma(w) + \theta = 0$
- With $m$ the in-plane unit normal of $S$ (card 1) and $r$ the signed radius, centre
  $C = S(u,v) - r\,m$: $F_3 = |C-\gamma(w)|^2 - r^2 = 0$ (OCCT writes
  $(|v_{ref}|^2-1)\,r^2$ with $v_{ref}=(S - \gamma + r\cdot(-m))/r$).
- Jacobian (verified rows 1 and 2): $\partial F_1 = (n\cdot S_u,\ n\cdot S_v,\ 0)$,
  $\partial F_2 = (0,\ 0,\ n\cdot\gamma')$; row 3:
  $2\,(C-\gamma)\cdot(\partial_x C)$ for $x\in\{u,v\}$ with $\partial_x C = \partial_x S - r\,\partial_x m$,
  and $-2\,(C-\gamma)\cdot\gamma'$ for $w$.
- The section is the arc from $S(u,v)$ to $\gamma(w)$ about $C$. It is tangent to $S$ but meets the
  other face at the edge with a crease, as intended.

Edge/edge function (`RstRstConstRad`, notes only): both contacts pinned to edges, unknowns
$(w_1,w_2)$, both points in the section plane and on the ball. Used when the ball rides on two
edges at once.

When to switch (OCCT `ChFi3d_Builder_2.cxx`, notes only): the walk re-frames a contact onto a
boundary arc (card 4). If the face beyond that arc is not tangent to the current face (`IsG1`
fails, so the ball cannot roll on), continue the stripe as a new segment with the surface/edge
function on that arc. If both sides are blocked, use edge/edge. Each segment is a separate
`SurfData` in the stripe.

### Data structures
- A stripe as a sequence of segments, each tagged face/face, face/edge or edge/edge, with its own
  walked line and the arc it is pinned to.

### Failure modes
- The ball leaves the edge (the edge turns away): re-frame on the face again and resume face/face.
- The edge ends at a vertex: card 4's vertex snap, then the corner logic.
- OCCT has no recovery beyond the plate corner fill; failures surface as walking failures.

### Fit in cadrs
- `fillet::pieces_of` already splits an edge into strips where the ball rolls over a *smooth*
  edge (`first_roll`). Add the sharp-edge case: a strip piece whose one rail is the edge itself.
- `fillet_geom`: a `ball_on_edge(face, k, edge_curve, plane)` solver beside `ball_in_plane`.
- Removes: the fallback to `blend::blend` for fillets that would roll over a sharp edge.
- Prerequisite: a fallback count (preface) that shows these cases; cards 1 and 4 make it cleaner
  but are not required.

### Sources
- OCCT 7.8.0 `BRepBlend/BRepBlend_SurfRstConstRad.cxx` (`Value`, `Derivatives`, `NbVariables`):
  verified.
- `notes/blends.md` KQ4 (`SurfRstConstRad`, `RstRstConstRad`, `SearchFace`/`IsG1`, SurfData per
  face, T-FLEX overflow policies): notes only.
- `notes/kernels_recent.md` 2.6, US 8,935,130 (notch and cliff blends recognised by arcs tangent
  at one end): notes only; it is about recognition, but it confirms the configuration.

---

## 6. Trimming path before topology edits (HP / Opitz)

Problem: when the fillet face must make neighbouring faces grow or shrink (faces the strip hangs
over, end faces that meet past the vertex, edges swallowed by the blend), update the topology
locally instead of falling back to a boolean.

### Why it is fast
- It keeps such cases on the local path: no tool body, no boolean, no global re-classification.
- The whole path is computed on geometry first, so a failure costs no topology rollback.
  "The complete path is computed before any topology is changed", because Euler edits "are
  generally not reversible in a unique way" (US 6,133,922, as quoted in the notes).

### Algorithm (notes only; the patents are not in `papers/`)
Freitag (US 5,615,317), shrink-only:
1. Compute the blend's left and right boundaries LB, RB (the rails) on the two faces.
2. At each end vertex V, for each other face edge at V: intersect it with the boundary. No
   intersection: kill-edge-vertex (KEV), which pulls the far vertex onto V, and test the next
   edge. Repeat until a crossing exists.
3. For every face at V not adjacent to the blended edge, intersect it with the blend surface;
   remove its edges, then the face (kill-face-vertex), when none intersect.
4. Check that the number of intersections found matches the number expected.

Opitz (US 6,133,922), growing faces:
1. Blend geometry first (rails on both faces). Walk the edges leaving each end vertex on each
   adjacent face; mark edges with no crossing for removal; find the RB and LB crossing points.
2. KEV the marked edges (the body is temporarily inconsistent).
3. ADEF: add the blend face in place of the edge.
4. Per end vertex, loop:
   a. Collect gaps: topologically adjacent edges whose geometric end points differ.
   b. Compute a trimming path from RB to LB by depth-first search over half-curves of
      face/face intersections through the current point. Order candidates by turning angle,
      then by length. Reject paths that self-intersect or cross existing edges. Extend short
      edges by ray shooting; mark faces the path leaves as growing.
   c. Integrate the path: a segment that crosses an existing edge shrinks that face (reattach
      edges); otherwise the face grows and the segment becomes a new edge (ADEF).
   d. Collect new gaps; repeat until none.

### Data structures
- Half-curves: each face/face intersection curve near V split at its crossings, stored per face
  pair with direction.
- A path: list of (half-curve, from, to, face). Marks: removed edges, growing faces.
- Euler operators on `topo::Body`: KEV, ADEF (add edge and face), split/extend edge.

### Failure modes
- No path found: the patents report failure (the prior art "show[s] an error message and the
  blend fails"). In cadrs, fall back to the tool body as now.
- The DFS is described for a single-loop hole; several loops need one search per loop (an
  inference).

### Fit in cadrs
- `fillet.rs`: `grow_to_cut`, `end_chain`, `end_chain_traced`, `cliff`, `hole_face` and
  `assemble` already cover named special cases ("faces the strip hangs over grow to its cut",
  "end faces that meet past the vertex grow over the fillet's end"). The trimming path is the
  general form: one search per end vertex that produces the cut curves, then one pass of edits.
- Removes: fallbacks where a neighbour face has to grow or vanish in a configuration the special
  cases do not name.
- Prerequisite: cadrs needs face/face intersection curves near V (`ssi`), which it has.

### Sources
- `notes/blends.md` KQ4 and catalogue entry 2 (US 6,133,922; US 5,615,317): notes only.
- `notes/kernels_recent.md` 2.7 (US 7,236,168, blend removal with KEV/KEF in reverse order):
  notes only; same Euler-operator bookkeeping in the other direction.

---

## 7. Fail-fast fit test from offset regularity

Problem: stop a local fillet that cannot succeed before it spends ball solves (the budget is
1,000,000) and a full body check, and go straight to the fallback or an error.

### Why it is fast
- The test costs a few curvature evaluations per edge. A doomed attempt costs failed Newton
  solves, halved steps and the `local_problem` full check before the fallback starts.

### Algorithm (derived; the self-intersection papers are not readable)
The ball's centre lies on the offset $O = S + k\,\hat N$ of each face (cadrs's own definition in
`fillet_geom`, with $k=\mp r$). $dO = (I - k\,W)\,dS$ with $W$ the shape operator, so the offset
is singular, and the rolling ball cannot touch the face smoothly, where $1 - k\,\kappa_j = 0$ for
a principal curvature $\kappa_j$ (sign convention as in `fillet_geom`'s Weingarten terms).
1. Sample each face of the edge at the edge's points (for example the walk's start sections, or
   8 samples along the edge) and at the expected contact points (the edge point moved by about
   $r$ across the face).
2. Compute principal curvatures $\kappa_{1,2}$ from the first and second fundamental forms.
3. If $\min_j (1 - k\,\kappa_j) \le \epsilon$ at any sample ($\epsilon$ a margin such as 0.05; a
   choice, not a sourced constant), the radius exceeds a local radius of curvature on the ball's
   side: fail fast with a clear message ("radius larger than the face's curvature radius").
4. For variable radius, test with the law's maximum on each span.

### Data structures
- None new: surface D2 at sample points.

### Failure modes
- It is a sufficient test for trouble, not a full validity check: a ball can still be blocked
  globally (a far wall), which needs the walk.
- Sampling can miss a short, highly curved region; use the walk's own stations once card 1 exists.

### Fit in cadrs
- `fillet::edge_fillet` (before any ball solve) and `blend::check_fits` (which already checks
  that a tool fits). OCCT's corner code refuses "fillets have too big radiuses" at the same kind
  of point (notes only).
- Removes: the time of doomed local attempts; it does not remove fallbacks.

### Sources
- `notes/blends.md` KQ3 and KQ5 (Lukács 1998, Hermann 1992, Hermann–Lukács–Várady 1997: self
  intersection when the radius exceeds the local curvature radius): abstracts only, notes only.
- The condition itself is standard offset-surface geometry, not taken from a paper in
  `papers/`.

---

## 8. Pinch split where the contact curves meet

Problem: when the two contact curves touch (the fillet narrows to a point, for example where the
two faces become tangent), the strip degenerates and the fit or imprint fails. Split the strip
there instead of failing.

### Why it is fast
- One scan over already walked points and a 1D root find per candidate. It avoids a failure, and
  with it the fallback.

### Algorithm (verified, `ChFi3d_FilBuilder.cxx` `SplitSurf` and its callers)
1. While walking, track the smallest contact distance $d_{min}=\min_k |P_{1,k}-P_{2,k}|$.
2. If $d_{min} \le 100\,$tolapp3d, run the split (`maybesingular`).
3. Scan the walked points inside the stripe's range. Where
   $d_k < d_{k-1}$ and $d_k < d_{k+1}$ (a local minimum), find the exact minimum on
   $[t_{k-1}, t_{k+1}]$ with a 1D root finder from the midpoint, tolerance tol2d, 50 iterations.
   OCCT's function (`ChFi3d_SearchSing`) works on the approximated surface's two boundary
   iso-curves.
4. Keep the root if the distance there is below tolapp3d and it lies more than tol2d inside the
   range and past the previous root.
5. Split the strip at each kept root into two pieces sharing a vertex at the pinch.

### Data structures
- The walked line's per-point contact distance; the strip as a list of pieces.

### Failure modes
- A point with no tangent at the pinch (card 1): the walk handles it by reusing the previous
  solution; the split then has a vertex there.
- Root finder fails: OCCT skips the split ("Failed calculation of the minimum length").

### Fit in cadrs
- `fillet::build_strip` and `fillet_geom::nurbs_strip` (which already accept `kinks`, the same
  idea for curvature jumps). Rails meeting at a point become a vertex in `assemble`.
- Prerequisite: none; easier with card 1.

### Sources
- OCCT 7.8.0 `ChFi3d/ChFi3d_FilBuilder.cxx` (`maybesingular = (Func.GetMinimalDistance() <=
  100*tolapp3d)`, `SplitSurf`): verified.

---

## 9. Corner ladder with a Generalized Coons patch

Problem: fill vertex corners (n filleted edges, mixed convexity, some edges unfilleted)
deterministically, so they do not fall back, and without an iterative patch fit.

### Why it is fast
- Exact rungs (sphere, torus) cost nothing to fit. The transfinite patch is closed form: a point
  costs n ribbon evaluations and n blend weights, with no least squares and no iteration (Salvi
  et al. 2012). OCCT's last rung, `GeomPlate`, is iterative and warns of "unpredicable
  undulations" (notes).

### Algorithm
Ladder (OCCT `PerformThreeCorner`, `PerformTwoCorner`, `PerformMoreThreeCorner`; notes only):
1. Three stripes, all radii equal within tolapp3d: sphere corner (cadrs has this).
2. Three stripes with a pivot face that is a plane, the two adjacent radii equal and their axes
   orthogonal: torus corner.
3. Two stripes and one sharp edge: intersect the fillets directly ("bevel" when both ends are on
   the same face, "knee" otherwise) by p-curve intersection on the common face; all three faces
   planar, equal radii within 1e-7 and the third face perpendicular: exact toric knee
   (`ToricRotule`). Tangents within 1°: treat as the n-edge case.
4. Otherwise: set back and fill the hole with an n-sided patch (below). OCCT uses
   `GeomFill_ConstrainedFilling(8 or 11, 20)` for 3 or 4 sides and `GeomPlate` (degree 3, 10
   curve points, 1 iteration, approximated with 9 spans, max degree 8) beyond.

Setback (US 8,004,517, notes only): cut each edge blend back from the vertex by a distance
from the widths of the neighbouring edge blends, plus a positive correction for tangent or cusp
edges. The hole is bounded alternately by profile curves (sections of the edge blends) and spring
curves (curves on the primary faces joining adjacent rails), up to $2n$ sides, zero-length sides
allowed.

Generalized Coons patch (verified, Salvi, Várady & Rockwood 2012, sections 2 and 4):
- Inputs per side $i$ ($n$ sides, cyclic indices): boundary curve $P_i(s)$, $s\in[0,1]$, and a
  cross-derivative field $T_i(s)$ pointing into the patch. For a profile curve, $T_i$ is the edge
  blend's derivative across that section (along the spine); for a spring curve, the face's
  tangent perpendicular to the curve. Their magnitudes are a choice (not specified by the
  paper).
- Domain: a regular $n$-gon in $(u,v)$.
- Side parameters $s_i(u,v)\in[0,1]$, 0 on side $i-1$ and 1 on side $i+1$ (for example a
  bilinear or radial line sweep). Blend $\alpha(t)=\frac{(1-t)^2}{t^2+(1-t)^2}$.
  Distance parameters $d_i = (1-s_{i-1})\,\alpha(s_i) + s_{i+1}\,\alpha(1-s_i)$.
- Ribbons $R_i(s_i,d_i) = P_i(s_i) + d_i\,T_i(s_i)$.
- Corner corrections
  $Q_{i,i-1}(s_i,s_{i-1}) = P_i(0) + (1-s_{i-1})\,T_i(0) + s_i\,T_{i-1}(1) + s_i(1-s_{i-1})\,W_{i,i-1}$,
  twist $W_{i,i-1}=\partial_s T_i(0) = -\partial_s T_{i-1}(1)$ (use Gregory's rational twists if
  the two disagree).
- Weights $D_{i,i-1}=\prod_{k\notin\{i,i-1\}} d_k$,
  $B_{i,i-1} = D_{i,i-1}^2 / \sum_l D_{l,l-1}^2$, $B_i = B_{i,i-1}+B_{i+1,i}$.
- Surface $S(u,v)=\sum_i R_i\,B_i - \sum_i Q_{i,i-1}\,B_{i,i-1}$. It interpolates every $P_i$
  and $T_i$, so G1 to the edge blends and faces holds by construction (the paper's conditions
  (2) to (10)).
- If only positions are available, Salvi 2020 gives a C0 variant (a weighted sum of $n$ C0 Coons
  ribbons with Wachspress coordinates), not enough for G1 corners.

### Data structures
- A corner: the vertex, its cyclic list of hole sides (profile or spring, with curve, cross
  field and the face or strip it borders).
- A new `Surface` kind for the transfinite patch, or a B-spline fitted to it for export and for
  `ssi`/`tess`/`mass`. STEP export needs the B-spline (Salvi, Várady & Rockwood, CAD&A 2021, not
  downloaded).

### Failure modes
- Concave or very uneven holes distort the regular-polygon parameterisation (the papers propose
  non-regular domains; notes).
- Incompatible twists at a corner: rational twists.
- Odd configurations (n = 3 with a zero-length side) degenerate a ribbon; the C0 paper notes the
  same for its opposite curve at $n=3$.

### Fit in cadrs
- `fillet::hole_corner`, `hole_face`, `disc` (the B-spline disc today) and `fillet::coons`
  (four-sided); `blend_corner::fill_corner` (the smooth corner's B-spline disc). The patch
  replaces the disc fit; the ladder adds the torus rungs in `fillet::plan`'s corner dispatch.
- Removes: fallbacks at corners `plan` does not classify, if any show up in the fallback count.
- Prerequisite: a decision on the surface representation (procedural kind or fitted B-spline).

### Sources
- `papers/blends/Salvi-Varady-Rockwood-2012-new-schemes-multi-sided-transfinite-interpolation.txt`,
  sections 2, 4.1 to 4.3: verified.
- `papers/blends/Salvi-2020-multi-sided-generalization-C0-Coons-patch.txt`, section 3: verified.
- `notes/blends.md` KQ2 (OCCT corner rules from `ChFi3d_FilBuilder_C2.cxx`, `_C3.cxx`,
  `ChFi3d_Builder_CnCrn.cxx`; US 8,004,517 setback): notes only.

---

## Left out

- Exact rolling-ball cases (OCCT `ChFiKPart`): OCCT covers plane/plane, plane/cylinder and
  plane/cone; cylinder/cylinder throws "particular case not written". cadrs's
  `fillet_geom::exact_kind` (straight edge between surfaces unchanged along it, circle between
  coaxial surfaces of revolution) already covers these, so there is no speed gap here.
- Dupin cyclide blends (`papers/blends/Chandru-Dutta-Hoffmann-1989-...txt`): exact variable
  blends between natural quadrics, but variable fillets are not in the measured gap.
- G2, conic and PDE sections (Boeing patent, Filkins, You et al., Kim & Elber, Flends): they
  change the section shape, not the cost of the slow path.
- Voronoi-surface and high-dimensional spine tracing (Chandru et al., Chuang et al.): same
  Newton-plus-marching as card 1 with more unknowns; slower, not faster.
