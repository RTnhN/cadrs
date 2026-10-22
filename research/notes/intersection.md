# Surface–surface, curve–surface and curve–curve intersection, and polynomial-system root finding for a B-rep kernel

Scope: methods for computing *all* branches of the intersection of two parametric or analytic
surfaces (no missed loops, tangential or singular contacts), refining and representing the curve
with p-curves on both surfaces, and the lower-dimensional problems (curve/surface, curve/curve,
point projection on NURBS, polynomial systems). Compiled 2026-10-22 for `cadrs_kernel::native`
(modules `ssi`, `ssi_numeric`, `ssi_coincident`, `pcurve`, `nurbs`, `distance`).

Research conditions: the shared web-search budget ran out after the first two rounds of
searches, so the rest of the work was done with direct fetches of known URLs (MIT hyperbook,
OCCT sources on GitHub, Semantic Scholar DOI metadata, local text extraction of the downloaded
PDFs). Several open-access PDFs were behind bot checks (HAL/Anubis, Cloudflare on ACM, BYU,
Graphics Interface, MDPI; MIT DSpace rate-limited with HTTP 429); their URLs are given so they
can be fetched from a browser. Downloaded papers are in
`../papers/intersection/` (11 PDFs, all verified to start with
`%PDF`).

Where a description comes from my own recollection of a paper rather than from a page or PDF I
read in this session, it is marked "(recollection)"; treat those sentences as unverified.

---

## Ranked entries (top 5 "must read" first)

### 1. Hohmeyer — loop-detection criterion and the Gauss-map subdivision SSI (MUST READ)

- **Citation.** Michael E. Hohmeyer, *Robust and Efficient Surface Intersection for Solid
  Modeling*, Ph.D. thesis, UC Berkeley, Technical Report UCB/CSD-92-681, 1992. Conference
  version: M. E. Hohmeyer, "A surface intersection algorithm based on loop detection", *Proc.
  1st ACM Symposium on Solid Modeling Foundations and CAD/CAM Applications (SMA '91)*, 1991
  (pages not verified).
- **Canonical URL.** https://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/6140.html
- **Open-access PDF.** http://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/Archive/CSD-92-681.pdf
- **Local copy.** `../papers/intersection/Hohmeyer-1992-robust-efficient-surface-intersection-thesis.md`
  (scanned, no text layer; pages 34–40, 66–67, 75, 89–90 were read as images).
- **Algorithm.** The thesis (§3.2, Lemma 1 and Theorem 4) proves: let N1, N2 be bounds on the
  Gauss maps (sets of normals) of two C¹ surfaces S1, S2; if there are vectors P1, P2 with
  P1·n1 > 0, P1·n2 < 0, P2·n1 > 0, P2·n2 > 0 for all n1 ∈ N1, n2 ∈ N2, then the intersection
  is a set of curves and points, all isolated points lie on patch boundaries, no curve has a
  singularity, and no curve is a closed loop. The proof sets P = P1 × P2 and shows the unit
  tangent T = n1 × n2 / |n1 × n2| satisfies T·P > 0 everywhere, so ∮T ds = 0 is impossible
  for a loop. §3.5.1 gives the recursive algorithm `Intersect(A, B)`: if the spatial bounds
  overlap, then if the Gauss-map bounds satisfy the criterion → intersect "simple" surfaces;
  else if both surfaces are within ε of a point and both Gauss maps within ε of one normal →
  report a singular point; else subdivide each surface and recurse on all pairs (Theorem 5:
  this always terminates in either no-loop subproblems or a discovered singularity). A simple
  pair has at most one curve, whose two end points p0, p1 come from edge–surface intersections;
  §3.4 traces it by sweeping planes F·P = w from w0 = P·p0 to w1 = P·p1 in steps δw, solving
  F(s,t) = G(u,v), F(s,t)·P = w by Newton from the previous solution — exactly one solution per
  plane, so there is no component jumping and the points come out ordered; δw "does not need to
  be chosen carefully". Supporting algorithms (ch. 4): the pseudo-normal patch N(u,v) = F_u × F_v
  is computed once as a Bézier surface for the root patch and subdivided with it (§4.4.2);
  Gauss-map separability is a linear program (Seidel's LP, §4.1, §4.3.2 "separating circles":
  find P with P·v_i > 0, P·w_j < 0); §4.4.6 notes that once the surfaces have been split at a
  point where they share a normal q, the separating plane has one degree of freedom left, which
  makes the test much faster. Chapter 6 handles singular points with the auxiliary system
  (6.4)–(6.7): F_s×F_t·G_u = 0, F_s×F_t·G_v = 0, (F−G)·F_s = 0, (F−G)·F_t = 0 — four equations
  in four unknowns whose solutions are a superset of the singular points (Sederberg's "magic
  points", Kriezis's "collinear normal points"); Theorem 7: its Jacobian is non-singular at node
  singularities and at isolated intersections where the surfaces do not share the same
  fundamental form. §3.3 (read): "if the bounds on the Gauss maps are separable by a plane then
  there can be no loops"; the converse holds by Farkas' lemma for convex bounds, so this is the
  strongest possible criterion given only convex Gauss-map bounds; on the example of two
  cylinders, Sederberg's, Sinha's, de Montaudouin's and Kriezis's criteria need arbitrarily
  fine subdivision while this one passes at any angular separation; the surfaces may be
  parametric, implicit or "other", and may have holes in their domains (Sederberg's criterion
  requires the loop interior to be a disc).
- **Limitations.** Finds *all* curves only when there are no singular intersection points; with
  singularities it finds and reports at least one (abstract). Needs Gauss-map bounds: the
  pseudo-normal patch of an m×n patch has degree up to 3m×3n (Krishnan & Manocha 1996, p. 4),
  and near tangential contact the recursion gets deep (K&M report 8 subdivision levels on their
  Fig. 1 loop example). The thesis assumes bounds on normals that are convex; tori and
  revolutions need case-by-case bounds (§4.4.1 only says this "can be constructed on a case by
  case basis"). Tested on real manufacturing data and pathological examples (abstract), but no
  timings were extracted.
- **Relevance to cadrs.** `ssi` / `ssi_numeric`: the plane-sweep trace (solve F = G, F·P = w) is
  a drop-in replacement for marching with step-size heuristics, and it gives p-curve points on
  both surfaces for free; the loop-detection recursion on Gauss-map bounds is the principled way
  to guarantee that the signed-distance contouring does not miss a small loop. For cadrs's
  analytic surfaces (cylinder, cone, sphere, torus, revolution of a line/circle) the Gauss-map
  bound of a uv box is cheap in closed form, so the criterion costs little.

### 2. Patrikalakis & Maekawa — characteristic points, IPP solver, marching with tangential cases, validated tracing (MUST READ)

- **Citation.** N. M. Patrikalakis and T. Maekawa, *Shape Interrogation for Computer Aided
  Design and Manufacturing*, Springer, 2002 (hyperbook with W. Cho, 2009). Chapters 4
  (Nonlinear polynomial solvers), 5 (Intersection problems), 6 (Differential geometry of
  intersection curves), 7 (Distance functions). Companion papers: E. C. Sherbrooke and
  N. M. Patrikalakis, "Computation of the solutions of nonlinear polynomial systems", *CAGD*
  10(5):379–405, 1993, DOI 10.1016/0167-8396(93)90019-Y; C. Y. Hu, T. Maekawa, N. M.
  Patrikalakis, X. Ye, "Robust interval algorithm for surface intersections", *Computer-Aided
  Design* 29(9):617–627, 1997 (cited as HMPY97 in Mukundan 2004); H. Mukundan, K. H. Ko, T.
  Maekawa, T. Sakkalis, N. M. Patrikalakis, "Tracing surface intersections with validated ODE
  system solver", *ACM Symposium on Solid Modeling and Applications 2004*, pp. 249–254, DOI
  10.2312/sm.20041397.
- **Canonical URLs.** Book: https://link.springer.com/book/9783642040733 ; Mukundan 2004:
  https://diglib.eg.org/handle/10.2312/sm20041397 ; Sherbrooke 1994 MIT thesis (same content as
  the 1993 paper): http://hdl.handle.net/1721.1/12300
- **Open-access.** Whole book as HTML: https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/
  (key nodes: 4.4 Projected Polyhedron node42.html, 4.9 Interval PP node52.html, 5.5.2
  point/surface node72–76, 5.7.2 curve/surface node91–94, 5.8.1 parametric/implicit node100–105
  (tracing node102, characteristic points node103, singular points node104, starting points
  node105), 5.8.2 parametric/parametric node106–109 (lattice 107, subdivision 108, marching
  109), 5.9 overlapping node111, 5.10 self-intersection node112, ch. 6 tangential intersection
  curves node117 ff.). Mukundan 2004 PDF:
  https://diglib7.eg.org/bitstream/handle/10.2312/sm20041397/249-254_mukundan.pdf. Sherbrooke
  thesis PDF: https://dspace.mit.edu/bitstreams/add86c41-188c-45cc-a01e-aa72856f9d69/download
  (DSpace returned HTTP 429 during this session). Mukundan's 2005 S.M. thesis
  "Surface-surface intersection with validated error bounds":
  https://dspace.mit.edu/handle/1721.1/33167 (bitstream
  /bitstreams/0514eb3a-4734-410b-9de7-41fa448928f3/download, also 429).
- **Local copy.** `../papers/intersection/Mukundan-2004-tracing-surface-intersection-validated-ODE.md`
  (the book itself is HTML only).
- **Algorithm.** *Projected Polyhedron (PP, node42):* map the box to [0,1]ⁿ, write every
  equation in the multivariate Bernstein basis (convert in exact arithmetic — power→Bernstein
  conversion is ill-conditioned), project the control points of each equation onto each
  (x_i, f) plane, intersect the 2-D convex hull with the axis to get an interval per variable,
  intersect across equations (empty → no root), take the product box; if a side did not shrink
  enough, split evenly along the non-shrinking dimensions; rescale with multivariate de
  Casteljau and repeat; accept a box when it is below the size tolerance and every hull still
  crosses the axis; then polish isolated roots with Newton. Quadratic convergence in 1-D, at
  best linear in higher dimensions; slow on underconstrained (positive-dimensional) systems,
  which is why it is used only to find characteristic points and marching does the rest.
  *Interval PP (node52):* the same with rounded interval arithmetic on the coefficients, so a
  root cannot be lost to round-off (their example (x−0.1)(x−0.6)(x−0.7): floating point loses
  the root 0.7 at iteration 9, RIA keeps it). *Parametric × implicit SSI (node100–105):*
  substitute the patch into f(x,y,z) = 0 to get an algebraic curve f(u,v) = 0; trace it as the
  ODE du/dt = λ f_v, dv/dt = −λ f_u with λ chosen so t is arc length on the surface via E, F, G
  (node102); starting points are *border points* (univariate roots on the four edges),
  *turning points* (f = 0 with f_u = 0, and f = 0 with f_v = 0 — 2×2 systems) and *singular
  points* (f = f_u = f_v = 0, an overconstrained 3×2 system for IPP); every closed loop has at
  least two turning points, so these seed every branch (node103/105); at a singular point the
  branch tangents are the roots of the quadratic from the second derivatives (node104: two real
  roots → crossing, double → cusp, complex → isolated point). *Parametric × parametric
  (node109):* characteristic points are border points (curve/surface intersections) and
  *collinear normal points* (the 4×4 system "surfaces' normals parallel", which Sederberg
  showed lie inside every loop and include all singular points); the patches are split at those
  points and then only border points are needed; the marching ODE is the arc-length-normalised
  c = ±(N_P × N_Q)/|N_P × N_Q| mapped into each parameter plane by two 2×2 systems
  (Mukundan 2004 eq. 2: σ' = det(c, P_t, N_P)/(N_P·N_P), t' = det(P_σ, c, N_P)/(N_P·N_P), same
  for u', v'). *Tangential intersection (ch. 6, Ye & Maekawa 1999, Mukundan 2004 §2.2):* when
  the normals are parallel the marching direction comes from equal normal curvatures,
  b11 σ'² + 2 b12 σ' t' + b22 t'² = 0 with b_ij from the first and second fundamental forms; the
  discriminant d = b12² − b11 b22 classifies: d < 0 isolated tangential contact point, d > 0
  branching (direction not unique), d = 0 with all b_ij = 0 contact of order ≥ 2 (cannot be
  evaluated), d = 0 otherwise a unique tangential direction c = (νP_σ + P_t)/|…|, ν = −b12/b11.
  *Validated tracing (Mukundan 2004):* the ODE system y' = f(y) in (σ, t, u, v) is integrated
  with a two-phase validated interval ODE solver (Phase I: a priori enclosure and step size from
  the validation criterion; Phase II: tightened enclosure), which prevents "straying" (leaving
  the branch) and "looping" (returning to earlier points) because a step is only accepted if
  existence and uniqueness are verified in the enclosure, and gives a numerically verified
  parameter-space error bound that is mapped to a 3-D model-space bound for a gap-free B-rep.
  *Overlap (node111):* run the IPP solver at coarse accuracy; many overlapping root boxes along
  the curves signal overlap; Theorem 5.9.1: if two curve segments overlap along a finite
  length, the overlap extends to a boundary point (all derivatives match), so the overlap
  endpoints are boundary points.
- **Limitations.** PP/IPP is exponential in the number of variables (tensor Bernstein, see
  Fünfzig et al. 2010) and "extremely time consuming in the case of tangential, as well as
  higher order intersections"; "there is also no guarantee that intervals generated do contain
  a root" (Mukundan 2004, §1, citing HMSP96); topology resolution of the roots by adjacency is
  complicated. The marching ODE needs a starting point on every branch (completeness rests on
  finding all characteristic points) and the collinear-normal system is 4-D. Mukundan 2004
  assumes an identified branch with bounded start point and does not address topology.
- **Relevance to cadrs.** `ssi_numeric`: characteristic points (border, turning, collinear
  normal) are the missing guarantee for the signed-distance contour — compute them with a
  Bernstein/B-spline subdivision solver on the uv box and split the box there before
  contouring, and use the second-fundamental-form quadratic for tangential branches instead of
  N1 × N2. `ssi_coincident`: Theorem 5.9.1 justifies deciding overlap from boundary points.
  `nurbs`: the PP solver is the reference design for a multivariate Bernstein root finder.

### 3. Krishnan & Manocha — algebraic lower-dimensional formulation, loop detection via complex tracing (MUST READ)

- **Citation.** S. Krishnan and D. Manocha, "An efficient surface intersection algorithm based
  on lower-dimensional formulation", *ACM Transactions on Graphics* 16(1):74–106, 1997, DOI
  10.1145/237748.237751. S. Krishnan and D. Manocha, "Algebraic loop detection and evaluation
  algorithms for curve and surface interrogations", *Graphics Interface '96*, pp. 87–94, DOI
  10.20380/GI1996.11. D. Manocha and S. Krishnan, "Algebraic pruning: a fast technique for
  curve and surface intersection", *CAGD* 14(9):823–845, 1997. D. Manocha and J. F. Canny, "A
  new approach for surface intersection", *Int. J. Computational Geometry & Applications*
  1(4):491–516, 1991 (also SMA '91, DOI 10.1145/112515.112544, gold OA).
- **Canonical URLs.** https://doi.org/10.1145/237748.237751 ;
  https://graphicsinterface.org/proceedings/gi1996/gi1996-11/ ; author page
  https://gamma.cs.unc.edu/CSG/intersect.html (lists ftp links that no longer resolve).
- **Open-access PDFs.** GI '96: https://graphicsinterface.org/wp-content/uploads/gi1996-11.pdf
  (downloaded). TOG 1997: https://dl.acm.org/doi/pdf/10.1145/237748.237751 (ACM "bronze" OA,
  blocked for scripts in this session). SMA '91 Manocha–Canny:
  https://dl.acm.org/doi/pdf/10.1145/112515.112544 (gold OA, same block).
- **Local copy.** `../papers/intersection/Krishnan-Manocha-1996-algebraic-loop-detection-GI.md`
- **Algorithm (GI '96, read in full).** The intersection curve of two rational tensor-product
  Bézier patches F(s,t) and G(u,v) is three equations in four unknowns (eq. 1). Implicitize F
  by the Dixon resultant of xW − X, yW − Y, zW − Z: a matrix M(x,y,z,w) of order 2mn whose
  entries are linear forms a x + b y + c z + d w; substituting G(u,v) gives a bivariate matrix
  polynomial M(u,v) whose singular set (det = 0, never expanded) is the plane projection of the
  intersection curve in G's domain, one-to-one with the space curve (degree up to 108 for two
  bicubics). Base points of F (X = Y = Z = W = 0) make the resultant vanish identically; they
  use the maximal-rank minor. Boundary start points: set u = 0 (etc.) to get a matrix
  polynomial M(v) = Σ v^i M_i, whose roots are the eigenvalues of the block companion matrix
  (eq. 6; generalized eigenproblem if M_d is singular), computed by QR — all real *and complex*
  roots. Tracing: predict the next point from tangent/curvature, then correct by inverse power
  iteration on the companion matrix of M(u2, v) to find the eigenvalue nearest v2 (LU of
  C − v2 I, exploiting structure). Loop detection (Lemma 1): if a real closed component exists,
  two complex-conjugate paths of the curve meet it at a real *u-turning point* (a point where
  the tangent is parallel to the v axis); proof by Bézout counting of intersections with
  iso-parametric lines L1→L2→L3 (two real roots merge to a double root at the turning point and
  continue as a conjugate pair). So the algorithm traces the complex boundary roots through the
  (u, Re v, Im v) space until they hit the real plane at a turning point, which yields one start
  point per loop; the method also works when the curve is singular (self-intersecting), unlike
  Gauss-map criteria. The TOG 1997 paper combines this with marching and reports an order of
  magnitude speed-up over previously published robust algorithms with output-sensitive
  complexity (abstract via search).
- **Limitations.** Needs polynomial/rational surfaces of modest degree (matrix of order 2mn;
  Dixon resultant construction is non-trivial and base points need special handling); the
  plane curve degree is quadratic in the patch degree, so eigen-solves on large companion
  matrices dominate; numerical conditioning of the matrix polynomial in floating point is not
  analysed in the GI paper; the comparison text notes previous Gauss-map methods "may not work
  well if the intersection curve is self-intersecting", which this handles, but complex
  tracing can itself stray. Not applicable as-is to procedural surfaces (sweeps with
  rotation-minimizing frames) unless they are first converted to B-splines.
- **Relevance to cadrs.** `ssi` for analytic × B-spline pairs: cadrs already has the implicit
  form of planes, quadrics and tori, so substituting a B-spline patch into it gives exactly
  the plane curve f(u,v) = 0 of this paper and of Patrikalakis §5.8.1 — all turning points
  (loop seeds) are then roots of {f = 0, f_v = 0} on the uv box, a 2×2 Bernstein system, which
  is the cheapest rigorous loop guarantee available for most of cadrs's boolean cases.

### 4. Grandine & Klein — topology resolution, then the intersection as a DAE boundary-value problem (MUST READ)

- **Citation.** T. A. Grandine and F. W. Klein IV, "A new approach to the surface intersection
  problem", *Computer Aided Geometric Design* 14(2):111–134, 1997, DOI
  10.1016/S0167-8396(96)00024-9.
- **Canonical URL.** https://doi.org/10.1016/S0167-8396(96)00024-9 (publisher page refused
  automated access; the paper is paywalled).
- **Open-access PDF.** None found. Related: T. A. Grandine, "Applications of contouring", *SIAM
  Review* 42(2), 2000 (not fetched); J. Hass, R. T. Farouki, C. Y. Han, X. Song, T. W.
  Sederberg, "Guaranteed consistency of surface intersections and trimmed surfaces using a
  coupled topology resolution and domain decomposition scheme", *Advances in Computational
  Mathematics* 27:1–26, 2007 (cited by Gao, Lin & Li 2026 as the other main topology-first
  method; no OA copy found).
- **Local copy.** none.
- **Algorithm (from the abstract and three secondary descriptions).** Two ideas: (1) *topology
  resolution* — the structure of the intersection curves, including closed interior loops, is
  determined before any curve is computed numerically; (2) the curves are then computed as the
  numerical solution of a differential-algebraic equation, approximately parametrised by arc
  length (abstract via search). Patrikalakis & Maekawa (node109) describe the solver side:
  "Grandine and Klein determine the intersection structure, including closed loops, before
  tracing; their method extends the PP algorithm to B-splines in floating point, with equations
  normalized to [−1, 1] and knot vectors normalized to [0, 1] at each iteration for numerical
  robustness." Mukundan et al. 2004 (§1): "Grandine and Klein formulate the intersection
  problem as a differential algebraic equation which can be solved as a boundary value problem
  instead of an initial value problem. The tracing of the intersection curve is based on
  approximation methods. Moreover the algorithm has a hard time dealing with tangential
  intersections." Gao, Lin & Li 2026 (§2.1): current research "employs algebraic methods to
  compute these characteristic points of intersection, thereby decomposing the intersection
  curve into several monotonic segments at the characteristic points to determine the
  topology [Grandine and Klein 1997][Hass et al. 2007]". (Recollection, unverified in this
  session: the characteristic points are the boundary crossings plus the *turning points of the
  curve with respect to one chosen parameter*, found as roots of the 4-variable system {F − G =
  0, that parameter's derivative along the curve = 0}; sorting all these events by that
  parameter and counting branches between consecutive events yields the pairing of end points
  of monotone segments; each segment is then solved as a two-point BVP with a B-spline
  collocation method, so the output is a spline curve with p-curves in both domains directly.)
- **Limitations.** Paywalled; tangential intersections are hard (Mukundan); topology resolution
  needs a reliable polynomial solver for the turning-point system (they use floating-point PP
  with renormalisation, i.e. no interval guarantee); the BVP approach needs the topology to be
  right first, and nearly-tangential configurations produce ill-conditioned turning points.
- **Relevance to cadrs.** `ssi_numeric`: the "monotone segments between events" idea maps
  directly onto the marching-squares contour: find the events (border crossings and
  turning points of the signed-distance zero set) first, then contour each monotone strip
  without any risk of missing a small loop between grid lines; the BVP/collocation step is an
  alternative to "refine points then fit a B-spline" that produces the fitted curve and both
  p-curves in one solve (`pcurve`).

### 5. Elber & Kim; Mourrain & Pavone; Sherbrooke & Patrikalakis — multivariate Bernstein/B-spline subdivision solvers (MUST READ as a group)

- **Citations.** G. Elber and M.-S. Kim, "Geometric constraint solver using multivariate
  rational spline functions", *Proc. 6th ACM Symposium on Solid Modeling and Applications*,
  2001, pp. 1–10, DOI 10.1145/376957.376958. B. Mourrain and J.-P. Pavone, "Subdivision
  methods for solving polynomial equations", *Journal of Symbolic Computation* 44(3):292–306,
  2009, DOI 10.1016/j.jsc.2008.04.016 (INRIA report RR-5658, 2005). E. C. Sherbrooke and
  N. M. Patrikalakis, "Computation of the solutions of nonlinear polynomial systems", *CAGD*
  10(5):379–405, 1993, DOI 10.1016/0167-8396(93)90019-Y. C. Fünfzig, D. Michelucci, S. Foufou,
  "Optimizations for tensorial Bernstein-based solvers by using polyhedral bounds", *Int. J.
  Shape Modeling* 16(1–2), 2010. C. Liang, B. Mourrain, J.-P. Pavone, "Subdivision methods for
  the topology of 2d and 3d implicit curves", in *Geometric Modeling and Algebraic Geometry*,
  Springer 2008, pp. 199–214, DOI 10.1007/978-3-540-72185-7_11.
- **Canonical URLs.** https://doi.org/10.1145/376957.376958 ;
  https://doi.org/10.1016/j.jsc.2008.04.016 (listed "bronze" OA by Semantic Scholar) ;
  https://doi.org/10.1016/0167-8396(93)90019-Y ; HAL for Mourrain–Pavone:
  https://inria.hal.science/inria-00070350 (Anubis bot check blocked scripted download).
- **Open-access PDFs / local copies.**
  Elber–Kim: https://gershon.cs.technion.ac.il/papers/multivar_solution.pdf →
  `../papers/intersection/Elber-Kim-2001-geometric-constraint-solver-multivariate-splines.md`.
  Fünfzig et al.: https://cfuenfzig.lima-city.de/Publications/OptimizationsForTensorialBernsteinBasedSolvers_IJSM2010.pdf →
  `../papers/intersection/Fuenfzig-Michelucci-Foufou-2010-tensorial-Bernstein-solvers.md`.
  Liang–Mourrain–Pavone: https://cs.nyu.edu/~yap/classes/modeling/06f/lect/mourrain-etal-subdiv-curves.pdf →
  `../papers/intersection/Liang-Mourrain-Pavone-2008-subdivision-topology-implicit-curves.md`.
  Sherbrooke thesis: MIT DSpace handle 1721.1/12300 (429 during this session).
- **Algorithm.** *Elber & Kim 2001 (read):* constraints F_i(x) = 0 (and inequalities) are
  represented as multivariate B-spline/Bézier scalar functions; a sub-domain can contain a
  zero only if the control coefficients of every F_i change sign (convex hull property), so
  the domain is subdivided (B-spline refinement at the midpoint) and sub-domains without a
  sign change are pruned; at the size tolerance one point per sub-domain is kept and improved
  by a Newton step derived geometrically (the normal of the graph hypersurface, eq. 1, gives
  the tangent hyperplane, intersected with the zero hyperplane); for positive-dimensional
  solution sets the sampled points are fitted with a hypersurface (§2.3), parametrised by one
  of the input constraints. §2.4 gives the *single-solution termination test*: generalising
  Sederberg & Meyers' hodograph test and the surface tangent/bounding cone tests, a sub-domain
  contains at most one solution when the cones bounding the normals of the implicit
  hypersurfaces F_i = 0 are such that the hypersurfaces can meet only once; then subdivision
  stops early and Newton finishes, and clusters of close roots are separated by subdividing
  until each region holds one. Applications: ray traps, bisectors, sweep envelopes, 5-axis
  accessibility. *Mourrain & Pavone 2009 (from the Pavone abstract, Fünfzig §7 and Liang
  2008):* a Bernstein-basis subdivision solver in a box of Rⁿ, "an improvement of the Interval
  Projected Polyhedron algorithm", with a *preconditioning* step — the system P(x) is replaced
  by P(x)·P'(m)⁻¹ (same roots) so that the Jacobian at the box centre is nearly the identity,
  which makes domain reduction near double roots very effective — and a univariate root solver
  (Descartes' rule in the Bernstein basis) for the projected polynomials; complexity is
  analysed in terms of intrinsic differential invariants. *Liang, Mourrain, Pavone 2008
  (read):* for an implicit curve (n−1 equations in n = 2, 3 variables) on a box, represent the
  polynomials in the Bernstein basis of that box; Theorem 1 (Descartes/sign variation) bounds
  the number of real roots on an edge; a *regularity test* (extending Alberti–Comte–Mourrain's
  surface criterion to curves) certifies that the topology of the curve inside a cell is
  determined by its points on the cell border, so the curve is reconstructed by joining those
  border points; cells that fail are subdivided down to a threshold ε, which isolates singular
  points in ε-boxes (their topology is not certified); handles degree-80 curves with 200-bit
  coefficients, where resultant/sub-resultant sweeps (Grandine–Klein style, GLMT05) fail.
  *Fünfzig et al. 2010 (read):* the tensor Bernstein basis has (1+d1)…(1+dn) functions, so
  Bernstein solvers are impractical beyond 6–7 variables; they bound each monomial with the
  "Bernstein polytope" (O(n²) faces for degree 2) and reduce the domain with linear programs
  instead; performance equals Bernstein solvers for n = 2.
- **Limitations.** All of these are exponential in dimension and slow at tangential/multiple
  roots unless preconditioned; the single-solution test needs normal-cone bounds; the
  termination for 1-D solution sets (curves) needs the Liang regularity test or a "no-loop"
  test (Bartoň & Elber 2011, see entry 15); interval guarantees need rounded interval
  arithmetic on the coefficients (IPP), which Elber–Kim do not provide.
- **Relevance to cadrs.** `nurbs`: one Bernstein/B-spline box solver (with sign-change
  pruning, Elber–Kim's normal-cone single-solution test, Mourrain–Pavone preconditioning and
  Bézier-clipping-style projection) serves curve/curve, curve/surface, point projection,
  collinear-normal points and the turning points needed by `ssi_numeric`; the Liang box
  regularity test is the certified replacement for marching-squares saddle handling in the
  contour and in `silhouette`. `distance`: the same solver replaces the branch-and-bound on
  cell boxes for the stationary points of the distance function.

---

### 6. Sederberg & Meyers (collinear normals, bounding pyramids); Sinha, Klassen & Wang; Sederberg & Zundel (loop destruction)

- **Citations.** T. W. Sederberg and R. J. Meyers, "Loop detection in surface patch
  intersections", *CAGD* 5(2):161–171, 1988, DOI 10.1016/0167-8396(88)90029-5. P. Sinha, E.
  Klassen, K. K. Wang, "Exploiting topological and geometric properties for selective
  subdivision", *Proc. 1st ACM Symposium on Computational Geometry*, 1985 (pages/DOI not
  verified). T. W. Sederberg, H. N. Christiansen, S. Katz, "An improved test for closed loops
  in surface intersections", *Computer-Aided Design* 21(8):505–508, 1989 (cited as THS89 in
  Krishnan & Manocha). T. W. Sederberg and A. K. Zundel, "Loop destruction using Bézier
  clipping" (BYU record title "Surface intersection loop destruction"), in *The Mathematics of
  Surfaces VII*, Oxford Univ. Press, 1996, pp. 463–478. M. Daniel, "Using a convex pyramid to
  bound surface normal vectors", *Computer Graphics Forum* 15(4), 1996.
- **Canonical URLs.** https://doi.org/10.1016/0167-8396(88)90029-5 ;
  https://scholarsarchive.byu.edu/facpub/1150 ;
  https://diglib.eg.org/items/eab6db5b-ccbb-4456-9f71-bec32430a4d6
- **Open-access PDF.** Sederberg–Zundel:
  https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=2149&context=facpub (Cloudflare
  "just a moment" page for scripts; open in a browser). Sederberg–Meyers: none (Semantic
  Scholar: closed). Local copy: none.
- **Algorithm (from Krishnan & Manocha 1996 §1.1, Patrikalakis node109, Elber–Kim §2.4).**
  Sinha et al. proved that if two C¹ surfaces intersect in a closed loop there is a normal on
  one surface parallel to a normal on the other; Sederberg et al. strengthened this to: there
  is a line perpendicular to both surfaces (collinear normals at two points that are also
  inside the loop), provided no normal of one surface is perpendicular to a normal of the
  other. The practical test bounds the normals of each patch in a cone (or a pyramid, which is
  tighter and cheaper to intersect — Daniel 1996) and declares "no loop possible" when the
  two normal bounds do not overlap (and, with the Sederberg–Christiansen–Katz refinement, when
  no normal of one is perpendicular to a normal of the other); failing patches are subdivided
  into four and all sub-pairs retested until every pair passes, after which each pair's
  intersection consists of open branches found from edge–surface intersections. Sederberg &
  Zundel use Bézier clipping of the collinear-normal system to subdivide only where a loop can
  exist ("loop destruction") instead of blind quad-subdivision (from the abstract and title;
  the PDF was not readable here).
- **Limitations.** The cone test is conservative: cylinders at any angle need arbitrarily
  fine subdivision (Hohmeyer §3.3 and Fig. 3.5); patches must be tangent-plane continuous and
  the loop interior must be a disc (no holes); singular/self-intersecting curves are not
  handled; the pseudo-normal (hodograph) surface used for the bound has degree (2m−1)×(2n−1).
- **Relevance to cadrs.** `ssi` / `ssi_numeric`: the normal-cone test is a cheap pre-filter
  before contouring: if the two faces' normal cones (closed form for analytic surfaces, hull of
  the pseudo-normal control net for B-splines) are disjoint, no closed loop exists and only
  border starting points are needed; otherwise split the uv box and retest or solve the
  collinear-normal system.

### 7. Sederberg & Nishita — Bézier clipping; Bartoň & Jüttler — quadratic clipping; Nishita–Sederberg–Kakimoto — ray/patch

- **Citations.** T. W. Sederberg and T. Nishita, "Curve intersection using Bézier clipping",
  *Computer-Aided Design* 22(9):538–549, 1990, DOI 10.1016/0010-4485(90)90039-F. T. Nishita,
  T. W. Sederberg, M. Kakimoto, "Ray tracing trimmed rational surface patches", *SIGGRAPH '90*,
  pp. 337–345. M. Bartoň and B. Jüttler, "Computing roots of polynomials by quadratic
  clipping", *CAGD* 24(3):125–141, 2007, DOI 10.1016/j.cagd.2007.01.003.
- **Canonical URLs.** https://doi.org/10.1016/0010-4485(90)90039-F ;
  https://doi.org/10.1016/j.cagd.2007.01.003 ; https://history.siggraph.org/?p=122078
- **Open-access PDF.** None found for the three (Semantic Scholar: closed). Open
  implementation of fat-line Bézier clipping: LibreOffice `basegfx/source/workbench/bezierclip.cxx`
  (https://docs.libreoffice.org/basegfx/html/bezierclip_8cxx_source.html). A German lab
  report implementing quadratic and cubic clipping with comparisons:
  https://panthema.net/2012/0320-Finding-Roots-of-Polynomials-by-Clipping/Ausarbeitung.pdf
  (not downloaded). Local copy: none.
- **Algorithm (recollection, consistent with the secondary sources read).** Bézier clipping
  for two curves P(t), Q(s): take the "fat line" L of Q (the line through Q's end points,
  offset by d_min, d_max of its control points); the signed distance of P's control points to
  L, plotted against t, is a Bézier function whose convex hull's intersection with the band
  [d_min, d_max] gives the t-range that can contain intersections; clip P to that range (de
  Casteljau), swap roles, iterate; if a clip removes less than ~20 % the curve is bisected
  (handles multiple intersections); convergence is quadratic for simple roots (secondary source
  via search: "proven to have a second-order convergence rate"). For ray × rational patch
  (Nishita et al.) the ray is two planes, the patch's signed distances to them are two Bézier
  functions over (u,v), and clipping is done alternately in u and v — robustly finds all hits
  without preprocessing (SIGGRAPH archive text). Quadratic clipping (Bartoň–Jüttler, from the
  search summary): approximate the degree-n Bernstein polynomial by its best quadratic via
  degree reduction, bound the error, and intersect the two bounding quadratics with the axis to
  shrink the interval; convergence order 3 at simple roots and 3/2 at double roots, better than
  Bézier clipping on close root pairs; Liu et al. extended it to cubic clipping and Jiang et
  al. (2016) to SLEFE-based clipping.
- **Limitations.** Bézier clipping degrades to bisection at tangential (double) intersections
  (convergence linear); needs polynomial (Bernstein) input — rational curves use homogeneous
  distances; the "20 % rule" and fat-line choice are heuristics; curve/curve of a line, circle
  and NURBS still needs the NURBS split into Bézier spans.
- **Relevance to cadrs.** `nurbs`: 1-D and 2-D Bézier/quadratic clipping are the fast local
  solvers for curve/curve (line, circle, ellipse, NURBS pairs), curve/surface (two distance
  functions over uv) and point projection; `ssi_numeric` border points (a surface's boundary
  curve against the other surface) are exactly Nishita's ray/patch problem with the ray
  replaced by a curve.

### 8. Quadric–quadric intersection in closed form: Levin; Wang, Goldman & Tu; Dupont, Lazard, Lazard & Petitjean; degenerate and conic cases

- **Citations.** J. Z. Levin, "A parametric algorithm for drawing pictures of solid objects
  composed of quadric surfaces", *Communications of the ACM* 19(10):555–563, 1976, DOI
  10.1145/360349.360355; J. Levin, "Mathematical models for determining the intersections of
  quadric surfaces", *Computer Graphics and Image Processing* 11(1):73–87, 1979 (recollection
  for the 1979 details). W. Wang, R. Goldman, C. Tu, "Enhancing Levin's method for computing
  quadric-surface intersections", *CAGD* 20(7):401–422, 2003, DOI
  10.1016/S0167-8396(03)00081-5. W. Wang, B. Joe, R. Goldman, "Computing quadric surface
  intersections based on an analysis of plane cubic curves", *Graphical Models* 64(6):335–367,
  2002, DOI 10.1016/S1077-3169(02)00018-7. L. Dupont, D. Lazard, S. Lazard, S. Petitjean,
  "Near-optimal parameterization of the intersection of quadrics: I. The generic algorithm;
  II. A classification of pencils; III. Parameterizing singular intersections", *Journal of
  Symbolic Computation* 43(3):168–191, 192–215, 216–232, 2008. S. Lazard, L. M. Peñaranda,
  S. Petitjean, "Intersecting quadrics: an efficient and exact implementation", *Computational
  Geometry* 35(1–2):74–99, 2006 (INRIA RR-5670, 2005; the QI library). R. T. Farouki, C. A.
  Neff, M. A. O'Connor, "Automatic parsing of degenerate quadric-surface intersections", *ACM
  Transactions on Graphics* 8(3):174–203, 1989, DOI 10.1145/77055.77058. R. N. Goldman and
  J. R. Miller, "Combining algebraic rigor with geometric robustness for the detection and
  calculation of conic sections in the intersection of two natural quadric surfaces", *SMA
  '91*, DOI 10.1145/112515.112545; J. R. Miller and R. N. Goldman, "Geometric algorithms for
  detecting and calculating all conic sections in the intersection of any two natural quadric
  surfaces", *CVGIP: Graphical Models and Image Processing* 57(1):55–66, 1995, DOI
  10.1006/gmip.1995.1006. C.-K. Shene and J. K. Johnstone, "On the lower degree intersections
  of two natural quadrics", *ACM TOG* 13(4):400–424, 1994, DOI 10.1145/195826.197316. A.
  Trocado and L. Gonzalez-Vega, "Computing the intersection of two quadrics through projection
  and lifting", arXiv:1903.06983, 2019. R. F. Sarraga, "Algebraic methods for intersections of
  quadric surfaces in GMSOLID", *CVGIP* 22(2):222–238, 1983.
- **Canonical / OA URLs.** Levin 1976: https://dl.acm.org/doi/pdf/10.1145/360349.360355 (ACM
  bronze OA, script-blocked). Wang–Goldman–Tu: HKU repository
  https://hub.hku.hk/handle/10722/152301 (403 for scripts) and the HKU graphics group PDF
  `https://i.cs.hku.hk/GraphicsGroup/publications/pdf/Enhancing Levin's method for computing quadric surface intersections.pdf`
  (returned HTML). Dupont et al.: HAL PDFs
  https://inria.hal.science/inria-00186089/file/JSC1.pdf ,
  https://inria.hal.science/inria-00186090/file/JSC2.pdf ,
  https://inria.hal.science/inria-00186091/file/JSC3.pdf ,
  https://inria.hal.science/inria-00070341/file/RR-5670.pdf (all behind HAL's Anubis bot
  check in this session; open in a browser). Farouki et al.:
  https://dl.acm.org/doi/pdf/10.1145/77055.77058 (bronze, blocked). Trocado & Gonzalez-Vega:
  https://arxiv.org/abs/1903.06983 → local
  `../papers/intersection/Computing-intersection-two-quadrics-2019-arxiv.md`.
- **Algorithms.** *Levin:* in the pencil Q1 + λQ2 of two quadrics there is always a real ruled
  quadric (Wang–Goldman–Tu give a geometric proof of this key fact); parametrise its rulings
  and substitute into the other quadric, giving the intersection as p(u) = a(u) ± d(u)√s(u)
  with s(u) a quartic (Wang–Goldman–Tu abstract via search). *Wang, Goldman & Tu 2003:* the
  roots of s(u) classify the morphology of an irreducible intersection curve (reducibility,
  singularity, number of connected components), and when the curve is singular an enhanced
  version produces a *rational* parametrisation; a later paper (Trocado & Gonzalez-Vega §1)
  notes that whether the refined algorithm is numerically robust "is open to question" and
  that Levin's method "often fails to find the intersection curve when it is singular and
  generates a parameterization that involves the square root of a quartic", and can be
  topologically wrong in floating point. *Wang, Joe & Goldman 2002:* reduce the intersection
  to the analysis of a plane cubic curve and classify all cases (title/abstract only).
  *Dupont et al. 2008 (abstracts via search):* exact algorithm over the rationals producing
  proper parametrisations of all components in all cases, with the number of distinct square
  roots in the coefficients minimal except in a few identified cases ("near-optimal"); part II
  classifies pencils by the Segre characteristic / inertia of the pencil's members, part III
  parametrises the singular cases (nodal/cuspidal quartics, conics, lines, points); part IV /
  the QI library is a C++ implementation handling all algebraic components and returning
  rational parametrisations whenever they exist. *Farouki, Neff & O'Connor 1989:* automatic
  parsing of all degenerate (reducible) quadric intersections into lines, conics, cubics with
  exact arithmetic (recollection of the content beyond the title). *Goldman & Miller 1991 /
  Miller & Goldman 1995, Shene & Johnstone 1994:* for *natural* quadrics (plane, sphere,
  cylinder, cone) detect and construct every conic section in the intersection by purely
  geometric tests (coaxial, tangent, "Dupin cyclide"-type conditions) rather than by factoring
  the pencil's quartic, which is what makes them robust in floating point. *Trocado &
  Gonzalez-Vega 2019 (read):* write both quadrics as z² + p1(x,y) z + p0 = 0 and z² + q1 z + q0
  = 0, project with the resultant S0(x,y) (degree ≤ 4, the "cutcurve" restricted to where both
  discriminants Δi ≥ 0, whose boundaries are the silhouettes Δi = 0); lift with the first
  subresultant z = (p0 − q0)/(q1 − p1) where p1 ≠ q1; singular points of the cutcurve are
  either on the line p1 = q1 (degree-2 equations) or projections of tangential intersection
  points (found via subresultants of S0 and its partials, Theorem 5.14); regular branches are
  discretised by solving the quartic S0(α, y) = 0 for sample α. Maple implementation,
  0.14–6.5 s per example; the general quadric form is "easy to derive" but not worked out.
- **Limitations.** Levin-type parametrisations contain √(quartic) and misclassify singular
  curves in floating point; the exact methods (Dupont, Farouki) need rational input and exact
  arithmetic and give algebraic-number coefficients; none of these covers tori (degree 4).
- **Relevance to cadrs.** `ssi` (analytic pairs): cadrs already does quadrics in closed form —
  the Miller–Goldman/Shene–Johnstone geometric conic detection is the robust way to make the
  degenerate cases (cylinder–cylinder of equal radius with intersecting axes, cone–sphere
  tangent, etc.) exact rather than fitted, and Wang–Goldman–Tu's root-of-s(u) classification is
  the right tool to decide the number of components and the presence of a singular point
  before any sampling.

### 9. Torus intersections

- **Citations.** K.-J. Kim and M.-S. Kim, "Procedural method for detecting conic sections in
  the intersection of two tori", *Transactions of the Society of CAD/CAM Engineers (Korea)*
  5(4):336–346, 2000 (Korean with English abstract). J. Caravantes et al., "Tools for analyzing
  the intersection curve between a torus and a quadric through projection and lifting", *CAGD*
  126, 2026 (record only). T. Reithmann, *Torus intersections with simple surfaces* (title
  paraphrased), Ph.D. thesis, Univ. Mainz, 2009 (record: algebraic methods reducing to degree ≤
  4 polynomials with exact arithmetic, including degenerate cases).
- **URLs / local copy.** Kim & Kim PDF:
  https://koreascience.or.kr/article/JAKO200011920959088.pdf → local
  `../papers/intersection/Kim-Kim-2000-torus-torus-conic-sections.md`.
  Caravantes: https://portalinvestigacion.um.es/documentos/69db0189226b77548f67a37f .
  Reithmann: https://openscience.ub.uni-mainz.de/handle/20.500.12030/4502
- **Algorithm (Kim & Kim abstract, read).** Any conic in a torus is a circle; the circles on a
  torus are profile circles, cross-sectional circles and (Yvon-)Villarceau circles; a
  classification of these three families gives a procedure that detects and constructs every
  circle common to two tori using only circle–circle and circle–line intersections, vector
  additions and inner products — "this simple structure makes our algorithm robust and
  efficient". The Caravantes 2026 paper applies the projection-and-lifting scheme of entry 8
  to torus × quadric. No closed-form general torus–torus method was found; OCCT treats a torus
  analytically only when coaxial/aligned with the other surface (entry 10).
- **Limitations.** Kim & Kim cover only the circular components; the remaining components of a
  torus–torus intersection (degree 16 space curve) still need numeric tracing.
- **Relevance to cadrs.** `ssi`: extend the coaxial surfaces-of-revolution case to the full
  list of circle cases (Villarceau circles are the ones a coaxial test misses) so that fillet
  tools (tori) meeting other fillet tools yield exact circles instead of fitted B-splines.

### 10. OpenCASCADE — what IntPatch / IntWalk / GeomInt / IntAna / Extrema actually do (read from source)

- **Citation.** Open CASCADE Technology, packages `IntPatch`, `IntWalk`, `IntPolyh`, `GeomInt`,
  `IntAna`, `IntCurveSurface`, `Extrema` (master branch, read 2026-10-22). Reference manual:
  https://occt3d.com/dev/doc/refman/html/classIntWalk__PWalking.html (404 at the time; use
  https://dev.opencascade.org/doc/refman/html/class_int_walk___p_walking.html).
- **Sources read.** https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx ,
  `.../IntPatch/IntPatch_ImpPrmIntersection.cxx`, `.../IntPatch/IntPatch_PrmPrmIntersection.cxx`,
  `.../IntWalk/IntWalk_PWalking.cxx`, `.../IntWalk/IntWalk_IWalking.gxx`,
  `.../GeomInt/GeomInt_IntSS_1.cxx`, `.../GeomInt/GeomInt_LineConstructor.cxx`,
  `.../IntCurveSurface/IntCurveSurface_HInter.cxx`,
  `https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingData/TKGeomBase/IntAna/IntAna_IntQuadQuad.cxx`,
  `.../TKGeomBase/Extrema/Extrema_GenExtPS.cxx`. Local copy: none (LGPL sources).
- **Dispatch (`IntPatch_Intersection`).** Plane, cylinder, sphere and cone are "analytic";
  a torus is analytic only when its axis is parallel to a plane's normal, its centre is on a
  sphere's centre/axis, or it is coaxial with a cylinder/cone (also two tori of equal radii in
  the list-of-start-points overload); cones with half-angle near 0 or above 1.55 rad are
  treated as parametric; everything else (B-spline, Bézier, extrusion, revolution, offset) is
  parametric. Analytic × analytic → `IntPatch_ImpImpIntersection` (`IntAna_QuadQuadGeo` for
  the special pairs, `IntAna_IntQuadQuad` otherwise); if it is "not done" the code falls back
  to parametric × parametric. Analytic × parametric → `IntPatch_ImpPrmIntersection`;
  parametric × parametric → `IntPatch_PrmPrmIntersection`. Defaults: deflection `myFleche` 0.01
  (clamped to [1e-3, 10]), `myUVMaxStep` 0.01 (≤ 0.5; 0.0001 near singular points),
  TolArc/TolTang clamped to [1e-8, 0.5] in `SetTolerances` (but overwritten by the raw
  arguments in every `Perform` — an inconsistency in the source). Analytic lines are converted
  to walking lines with 200 points (`IntPatch_ALineToWLine`); walking lines are purged
  (`IntPatch_WLineTool::ComputePurgedWLine`), joined only for cylinder × cylinder
  (`JoinWLines`) and extended to boundaries/critical points (`ExtendTwoWLines`: cone apexes,
  sphere poles).
- **Analytic × analytic (`IntAna_IntQuadQuad`).** Cylinder or cone against a general quadric
  (10 implicit coefficients): substitute the cylinder (θ, Z) into the quadric → quadratic in Z
  with trigonometric coefficients; the discriminant DIS(θ) = C1 + C_SS sin² + C_CC cos² +
  2C_SC sin cos + 2C_S sin + 2C_C cos is solved with `math_TrigonometricFunctionRoots`, roots
  normalised to [0, 2π), sorted and checked by substitution at 1e-8 ("the direct search gives
  unreliable results"); the θ intervals where DIS > 0 (sign sampled at three points per
  interval) become `IntAna_Curve`s of the form Z±(θ) (flags `DEUX_Z_PAR_THETA`,
  `UN_SEUL_Z_PAR_THETA`, `Z_POSITIF/NEGATIF`); one discriminant root = tangency (tangent point
  plus curves if DIS ≥ 0 elsewhere); a double root splits an interval into Z+ and Z− branches.
  Cone: z = apex frame, A(t) z² + B(t) z + C(t) = 0, branches going to infinity where A(t) = 0
  are flagged open; near-zero leading coefficients set `done = false` (callers must check).
  Curves are chained end-to-end with 1e-7 (parameter) / 1e-10 (3-D) tolerances
  (`InternalSetNextAndPrevious`), and `AddSpecialPoints` widens the θ interval when a cone apex
  or sphere pole is a true intersection just outside the computed boundary.
- **Analytic × parametric (`IntPatch_ImpPrmIntersection` + `IntWalk_IWalking`).** The quadric's
  implicit function is evaluated on the parametric surface (`IntPatch_TheSurfFunction`), i.e.
  exactly cadrs's "signed distance over the other's uv box" but with the *implicit* function.
  Start points: (a) zeros of the function along the parametric domain's boundary arcs
  (`IntPatch_TheSOnBounds`, giving path points and boundary segments; tangent ones flagged);
  (b) interior points from an interior solver (`solins`) seeded by a user point or the domain
  (for a plane, first sample the domain for a sign change; if none, "whole surface is on one
  side of plane" and the interior search is skipped). `IntWalk_IWalking` then walks in the
  2-D parameter plane: boundary points with state 12 start *open* lines, interior grid points
  with state 13 start *closed* lines (`ComputeCloseLine`: if the walk returns to its start,
  "line closed good case"; else the point is re-declared open, state 12, and the line is
  continued in the other direction); a processed point's state is negated; `TestArretPassage`
  marks interior grid points the segment passes near (distance ratios 0.25/0.5) so that each
  loop is walked once; up to `MaxNbIter = 10` passes of `FillPntsInHoles` pair unmatched
  line ends and Newton-solve their midpoints as new seeds. Step: predictor along the previous
  2-D direction by `Step`, `Cadrage` clamps to the boundary iso-line, corrector
  `IntWalk_MarchFunction` adds the equation "correct along the normal to the marching direction
  in UV"; step halved when the residual exceeds the function tolerance, when the 3-D angle
  cosine < `CosRef3D = 0.98` or the 2-D cosine < `CosRef2D = 0.88`; `TestDeflection` estimates
  the chord deviation as |d1 − d4|²·Norme/64 against 0.25·fleche² and grows the step toward
  `pas` when small; `IsTangent()` → `IntWalk_ArretSurPoint` ends the line at a tangent point.
  Post-processing splits lines at seams and poles (`DecomposeResult`, U jump > π/2), joins
  lines only at tangency points, inserts restriction vertices (`IntPatch_RstInt::PutVertexOnLine`),
  drops walking lines coincident with restriction (boundary) lines, and shrinks the step to
  1e-4 for high-degree or nearly-C0 B-spline surfaces (`GetLocalStep`).
- **Parametric × parametric (`IntPatch_PrmPrmIntersection` + `IntWalk_PWalking`).** Start
  points come from a polyhedral intersection: each surface is triangulated
  (`IntPatch_Polyhedron`, sample counts from `SamplePnts(Deflection)`), `IntPolyh_Intersection`
  returns section lines and tangent zones; seeds are taken at the midpoint, then near the
  start, end and quarter points of each section line (longest first) and from tangent-zone
  points, mapped to (u,v) by barycentric coordinates; a seed is skipped if `IsPointOnLine`
  finds it on an existing walked line or if its distance is below `SeuildPointLigne =
  15·Increment²` ("10 est insuffisant"). Each seed is refined (`PerformFirstPoint`) and walked
  in both directions; lines with ≤ 2 points are dropped, lines with < 40 points get
  `SeekAdditionalPoints`, `PutToBoundary` extends to the face boundary, duplicates keep the
  longer line (`DublicateOfLinesProcessing`), and periodic parameters are shifted by a period
  on jumps ≥ half a period. `IntWalk_PWalking` steps all four parameters by `pasuv[k]` times
  the direction (one parameter fixed as the "iso"), corrects by Newton on the 4-unknown system
  with that iso fixed (`IntWalk_TheInt2S`), and adapts the step with `TestDeflection` status
  codes: `OK` (after 5 consecutive OK steps grow by a quarter of the gap to `pasInit`, capped
  10 %; `pasMax` ×1.1 while < 0.1), `PasTropGrand` (restore previous point, shrink),
  `StepTooSmall` (grow ≤ 1.5×), `PointConfondu`, `ArretSurPoint`, `ArretSurPointPrecedent`;
  after 10 iterations without a new point, steps are halved; stops when the chord angle exceeds
  3π/4 on both surfaces, when the 3-D derivative along the direction vanishes on both
  (singularity), or when the parameter-space dot product of consecutive segments is negative.
  Tangential (contact) branches: `IsTangentExtCheck` rejects a start where offset samples of
  each surface project within 2·tol onto the other; `ComputeContactDirection` takes the null
  direction of the difference of the two second fundamental forms (2×2 eigen-analysis, the
  same quadratic as Patrikalakis ch. 6) and `RefineContactPoint` solves a 6-equation
  (position gap + normal mismatch, each scaled by a tolerance) 4-unknown bounded Newton
  (`MathSys::NewtonBounded` on `IntWalk_ContactFunction`, requiring rank 4). Loop closure: a
  new point within 1e-14 squared 3-D distance of the first point closes the line (a comment
  says this 3-D check exists because "the UV-based closure check in TestArret fails" in some
  cases); while still near the start, points are skipped and `IncKey` grows (abort at 5000).
- **From polyline to curve with p-curves (`GeomInt_IntSS`).** With approximation on, a walking
  line is first split (`DecompositionOfWLine`, tol 2e-7), a parametrisation type is chosen per
  piece (`ApproxInt_KnotTools::DefineParType`), and `GeomInt_WLApprox` (ApproxInt) fits the 3-D
  curve and both 2-D p-curves *simultaneously* with `SetParameters(myTolApprox = 1e-7, tol2d,
  degmin 4, degmax 8, 0, 30 iterations, …)`; the result is a `Geom_BSplineCurve` plus p-curves
  from the same multi-curve, with `TolReached3d/2d` recorded; special cases: a plane p-curve
  is fitted and the 3-D poles obtained with `ElSLib::Value`; cylinder: fit in UV then
  `buildSurfaceCurve(1000, 8, C1)` with the 2-D error converted to 3-D scale; if the fit fails
  the fallback is a degree-1 B-spline through the points (`MakeBSpline`/`MakeBSpline2d`,
  index knots). Missing p-curves come from `GeomProjLib::Curve2d` (retry with a larger
  tolerance; tiny spans become a line between the projected end points). `IntPatch_GLine`
  results (line, circle, ellipse, parabola, hyperbola) are kept as exact conics, trimmed, and
  classified at sample points (18 samples per circle/ellipse). `GeomInt_LineConstructor` then
  keeps the parameter ranges whose midpoints classify inside both face domains (tolerance
  35·PConfusion, micro-circles below the 3-D tolerance rejected), with a note that the
  implicit–parametric intersector "does not respect domain of the quadric surface".
- **Curve/surface and point projection.** `IntCurveSurface_HInter`: conics (line, circle,
  ellipse, parabola, hyperbola) against analytic surfaces by `IntAna_IntConicQuad`; general
  curves on quadrics by an exact quadric-along-curve solver (`TheQuadCurvExactHInter`); the
  general case approximates the curve by a polygon and the surface by a polyhedron, filters
  with interference tests and refines with Newton (`TheExactHInter`); overlapping segments:
  `AppendSegment` is literally "// Not implemented". `Extrema_GenExtPS` (point projection):
  sample the surface on a grid placed by knot spans and degree (B-spline), by the basis
  curve's knots (revolution/extrusion) or uniformly, pad with sentinel rows, pick local minima
  per cell (closest point on each cell edge, face-midpoint candidate if the projection is
  inside the cell, duplicates suppressed), and polish each with `math_FunctionSetRoot` on the
  two orthogonality equations (`Extrema_FuncExtPS`); 300 samples is the cap "used for singular
  points"; sample counts are rescaled when the U/V step ratio leaves [0.1, 10].
- **Limitations / what makes it faster or slower.** No topological guarantee anywhere: loops
  are found only if the polyhedral intersection or the interior grid hits them, tangent zones
  are heuristics, and many magic constants (15·Inc², 1e-14, 0.98/0.88, 40 points, 2500 cells)
  tune behaviour; the 429/403 blocks meant the ApproxInt and IntPolyh internals were not read.
  It is fast because analytic × analytic never samples (exact `IntAna_Curve`s), analytic ×
  parametric walks a 2-D problem, and the B-spline fit produces curve and both p-curves in one
  least-squares pass.
- **Relevance to cadrs.** `ssi`: cadrs's dispatch is already more exact than OCCT's (tori,
  coaxial revolutions, plane × extrusion as conics). `ssi_numeric`: OCCT's implicit ×
  parametric path is the same problem cadrs contours; its weaknesses (grid-dependent loop
  seeds) are exactly what entries 1–5 fix. `pcurve`: the simultaneous 3-D + two 2-D
  least-squares fit (degree 4–8, tolerance 1e-7, parametrisation chosen per piece) is the
  design to copy so that p-curves never have to be re-projected. `distance`: the knot-span
  grid plus per-cell edge/face candidates is a cheaper start than branch-and-bound for the
  common case.

### 11. Bajaj, Hoffmann, Lynch & Hopcroft — tracing surface intersections; Hoffmann's dimensionality paradigm

- **Citations.** C. L. Bajaj, C. M. Hoffmann, R. E. Lynch, J. E. H. Hopcroft, "Tracing surface
  intersections", *CAGD* 5(4):285–307, 1988, DOI 10.1016/0167-8396(88)90010-6. C. M. Hoffmann,
  "A dimensionality paradigm for surface interrogations", *CAGD* 7(1–4):517–532, 1990 (Purdue
  TR 88-837, 1988).
- **URLs.** https://doi.org/10.1016/0167-8396(88)90010-6 (closed);
  https://cs.purdue.edu/cgvlab/www/publications/bajaj1988tracing ; Hoffmann TR PDF:
  https://www.cs.purdue.edu/cgvlab/www/resources/papers/Hoffmann-Computer_aided_geo_design-1990-A_dimensionality_paradigm_for_surface_interrogation.pdf
  → local `../papers/intersection/Hoffmann-1990-dimensionality-paradigm-surface-interrogation.md`.
- **Algorithm.** Bajaj et al. (abstract not retrievable; Gao et al. 2026 §2.1 summarise:
  "tracing methods generate intersections by stepping from initial points on a curve branch,
  following the local differential geometry of the curve [Bajaj et al. 1988]; these methods
  require careful selection of valid initial points … determining the tracking direction at
  singular points is challenging"). (Recollection: they trace implicit/implicit and
  parametric/implicit intersections with a Newton-corrected predictor using the curve's
  derivatives up to third order, treat singular points by a local power-series/quadratic-form
  analysis of the branches, and implemented it in Fortran.) Hoffmann's paradigm (TR abstract,
  read): instead of closed-form expressions, reformulate the interrogation in a
  higher-dimensional space with more variables but simpler equations (e.g. an offset or blend
  as a system {surface, normal, distance}) and trace/solve the system there, avoiding symbolic
  elimination and numerically delicate operations.
- **Limitations.** Start points are the user's problem; no loop guarantee; singular points
  need the local analysis to pick branches.
- **Relevance to cadrs.** `ssi_numeric` / `fillet_geom`: the higher-dimensional formulation is
  how to intersect a procedural surface (sweep with RMF frames, offset) without converting it:
  trace the system in (u, v, s, t) directly, as cadrs already does implicitly when refining
  contour points onto both surfaces.

### 12. Barnhill & Kersey (marching), Houghton et al. (subdivision), Lasser (Bernstein–Bézier subdivision): status of marching/subdivision/lattice methods

- **Citations.** R. E. Barnhill and S. N. Kersey, "A marching method for parametric
  surface/surface intersection", *CAGD* 7(1–4):257–280, 1990, DOI
  10.1016/0167-8396(90)90035-P. R. E. Barnhill, G. Farin, M. Jordan, B. R. Piper,
  "Surface/surface intersection", *CAGD* 4(1–2):3–16, 1987. E. G. Houghton, R. F. Emnett,
  J. D. Factor, C. L. Sabharwal, "Implementation of a divide-and-conquer method for
  intersection of parametric surfaces", *CAGD* 2(1–3):173–183, 1985, DOI
  10.1016/0167-8396(85)90022-6. D. Lasser, "Intersection of parametric surfaces in the
  Bernstein–Bézier representation", *Computer-Aided Design* 18(4):186–192, 1986. J. R.
  Rossignac and A. A. G. Requicha, "Piecewise-circular curves for geometric modeling", *IBM J.
  Res. Dev.* 31(3), 1987 (lattice evaluation, cited by Gao et al.).
- **URLs.** DOIs above (all closed; Semantic Scholar lists no OA copies). Local copy: none.
- **Algorithms and status (abstracts via search; Patrikalakis nodes 107–108).** Barnhill &
  Kersey march on rectangular and triangular domains using only evaluated positions and
  tangents (no surface equations), with handling of tangent points, branch points and
  "tangent tracks", a step-length approximation and relaxation of points onto surface
  boundaries; start points come from subdivision with bounding boxes/octrees (keywords).
  Houghton et al. and Lasser recursively subdivide both patches (Bernstein–Bézier
  subdivision), intersect the resulting near-planar pieces and link the segments; the
  hyperbook's assessment: with finite subdivision, correct connectivity near singular or
  near-singular points cannot be guaranteed, small loops are missed with polyhedral
  approximations and extraneous loops can appear, and high-precision evaluation causes data
  proliferation, so pure subdivision "is impractical for many high-accuracy CAD/CAM
  applications"; lattice methods (isoparametric curves of one surface intersected with the
  other, then linked) depend on the grid resolution, miss small loops and isolated points near
  tangency, and can connect points wrongly. The recommended practice is adaptive subdivision
  plus local methods for characteristic points, then marching.
- **Relevance to cadrs.** `ssi_numeric`: this is the method family cadrs's contouring belongs
  to; the historical verdict (grid-dependent, loops missed near tangency) is the argument for
  adding characteristic points (entries 2–4) rather than finer grids.

### 13. Affine-arithmetic subdivision SSI on the GPU; Mapper-based topology (2013–2026)

- **Citations.** H. Lin, Y. Qin, H. Liao, Y. Xiong, "Affine arithmetic-based B-spline surface
  intersection with GPU acceleration", *IEEE TVCG* 20(2):172–181, 2014 (online 2013), DOI
  10.1109/TVCG.2013.237. L. H. de Figueiredo, "Surface intersection using affine arithmetic",
  *Graphics Interface '96*, pp. 168–175. C. Gao, H. Lin, G. Li, "Topology understanding of
  B-spline surface/surface intersection with Mapper", arXiv:2604.12631, April 2026. (Also:
  W. Huber and H. Barth, subdivision with parallelepiped bounds, 1999, record only.)
- **URLs / local copies.** Lin et al.: https://doi.org/10.1109/TVCG.2013.237 (closed; no
  author copy found). Gao et al.: https://arxiv.org/abs/2604.12631 → local
  `../papers/intersection/Topology-BSpline-SSI-Mapper-2026-arxiv.md`.
  De Figueiredo: Graphics Interface proceedings are open at graphicsinterface.org (exact file
  not located).
- **Algorithms.** Lin et al. (abstract via search): extend affine arithmetic to the
  recursively and locally defined B-spline basis functions so that a uv box of surface A and
  an st box of surface B get affine-form bounding volumes that are tight and *correlated*;
  boxes whose bounds cannot meet are pruned, the rest subdivided, on the GPU; strip-shaped
  intersection regions are thinned and the points refined; the abstract claims affine
  arithmetic "is able to not only find every branch of the intersection, but also deal with
  some singular cases, such as surface tangency". Gao, Lin & Li 2026 (read): take the
  pruned-box point sets P1(u,v), P2(s,t) from the Lin et al. subdivision and apply a two-step
  Mapper (topological data analysis): filter by projection onto the principal direction,
  cover the range with overlapping intervals (parameters from Carrière et al. 2018 so the
  Mapper graph converges to the Reeb graph), cluster each preimage with DBSCAN, build the
  graph, then split nodes along the orthogonal direction; "characteristic nodes" (branching,
  turning) partition the intersection into simple segments and give the number of components
  and cycles. Limitation stated (§5.4): closely clustered singular points fall into one
  interval and are merged into one node.
- **Limitations.** Affine-arithmetic subdivision is complete only down to the box tolerance
  (two branches closer than the tolerance stay merged); GPU-friendly but memory-hungry; the
  Mapper step is statistical (DBSCAN radius, overlap ratio) rather than certified.
- **Relevance to cadrs.** `ssi_numeric` for B-spline × B-spline: affine-arithmetic box
  pruning in (u,v,s,t) is the modern, parallel (rayon) way to get a superset of the curve
  that cannot miss a branch, replacing a fixed contour grid; it is also directly usable for
  curve/surface and self-intersection. The Mapper paper is a signal that topology from the box
  cloud is still an open problem, so cadrs should keep the characteristic-point approach.

### 14. Point projection / inversion on NURBS: Piegl & Tiller, Ma & Hewitt, Selimovic, Hu & Wallner, Liao et al.

- **Citations.** L. Piegl and W. Tiller, *The NURBS Book*, 2nd ed., Springer 1997, §6.1
  (point inversion and projection; recollection). Y. L. Ma and W. T. Hewitt, "Point inversion
  and projection for NURBS curve and surface: control polygon approach", *CAGD* 20(2):79–99,
  2003, DOI 10.1016/S0167-8396(03)00021-9. I. Selimovic, "Improved algorithms for the
  projection of points on NURBS curves and surfaces", *CAGD* 23(5):439–445, 2006, DOI
  10.1016/j.cagd.2006.01.007. S.-M. Hu and J. Wallner, "A second order algorithm for
  orthogonal projection onto curves and surfaces", *CAGD* 22(3):251–260, 2005, DOI
  10.1016/j.cagd.2004.12.001. X. Li et al., "Convergence analysis on the Hu–Wallner method"
  (title paraphrased), *Symmetry* 9(10):210, 2017 (MDPI, open access). K. Liao et al.,
  "Algebraic point projection for immersed boundary analysis on low degree NURBS curves and
  surfaces", *Algorithms* 13(4):82, 2020, MDPI, open access.
- **URLs.** Hu–Wallner author PDF: http://www.geometrie.tugraz.at/wallner/sproj.pdf (listed
  "green OA" by Semantic Scholar; TU Graz was unreachable in this session — timeouts). Li et
  al.: https://www.mdpi.com/2073-8994/9/10/210 ; Liao et al.:
  https://www.mdpi.com/1999-4893/13/4/82 (MDPI returned "Access Denied" to scripts). Ma &
  Hewitt and Selimovic: closed. Local copy: none.
- **Algorithms (from the search summaries and the hyperbook).** Piegl & Tiller (recollection):
  sample the curve/surface coarsely, take the nearest sample as the start, Newton on
  f(u) = C'(u)·(C(u) − P) = 0 (surface: the two equations S_u·(S − P) = 0, S_v·(S − P) = 0 with
  the Jacobian containing second derivatives), with stopping criteria on point coincidence,
  zero cosine, parameter change and domain clamping. Ma & Hewitt: subdivide into Bézier
  pieces, select candidate pieces by control-polygon/control-net tests (the projection of P
  onto a piece lies within the piece only if P's position relative to the control polygon
  permits it), then Newton–Raphson. Selimovic: improves Ma & Hewitt with selective subdivision
  based on the distance to the end (corner) points of the entity and a hyperplane test that
  eliminates a piece when all its control points lie on one side of a plane through the
  candidate point (Chen et al. later noted that this needs all control points on different
  sides of a hyperplane and proposed circular clipping instead). Hu & Wallner: a second-order
  method that, instead of Newton on the tangent, steps along the curvature (osculating circle
  / second-order Taylor of the curve), so each iteration is a geometric projection onto the
  local approximation; Li et al. 2017 prove second-order convergence for projection and
  third-order for inversion independent of the initial value for planar curves. Liao et al.
  2020: for low-degree NURBS, write the orthogonality condition algebraically and solve it
  exactly per Bézier span. Hyperbook §5.4.2.2/5.5.2.3: bounding box plus subdivision until
  boxes are small, then minimise the *squared* distance numerically (the derivative of the
  distance diverges near zero); §5.5.2.4 distance-function stationary points via the IPP
  solver.
- **Limitations.** Newton needs a good start and converges to the wrong stationary point near
  curvature centres; control-polygon pruning is weak for highly curved or rational pieces;
  Hu–Wallner's geometric step needs curvature (second derivatives) and is per-iteration more
  expensive; none of them guarantees the *global* minimum without a box/subdivision stage.
- **Relevance to cadrs.** `nurbs` / `distance`: point projection is the inner loop of
  cadrs's "refine points onto both surfaces" and of edge/face distance queries; a per-span
  Bézier prune (Ma–Hewitt/Selimovic) followed by Hu–Wallner's second-order step is the standard
  fast path, with the IPP/Bernstein solver as the guaranteed fallback for the global minimum.

### 15. Topologically guaranteed subdivision solutions for curves and single-component tests (2007–2012)

- **Citations.** M. Bartoň and G. Elber, "Topologically guaranteed univariate solutions of
  underconstrained polynomial systems via no-loop and single-component tests", *Computer-Aided
  Design* 43(8), 2011 (also SPM 2010); I. Hanniel and G. Elber, "Subdivision termination
  criteria in subdivision multivariate solvers using dual hyperplanes representations",
  *Computer-Aided Design* 39(5):369–378, 2007; M. Aizenshtein, M. Bartoň, G. Elber, "Global
  solutions of well-constrained transcendental systems using expression trees and a single
  solution test", *CAGD* 29(5):265–279, 2012. (All from recollection of titles/venues; DOIs
  not verified in this session, and no OA copies located because the search budget was gone.)
- **URLs.** Elber's publication list https://gershon.cs.technion.ac.il/ (directory listing not
  available); check for PDFs there. Local copy: none.
- **Algorithm (recollection).** For a system of n−1 equations in n variables (an
  intersection curve) solved by Bernstein subdivision, a cell is "done" when (a) a *no-loop
  test* holds — a vector exists that makes a positive inner product with every tangent of the
  solution curve in the cell, derived from the normal cones of the constraint hypersurfaces
  (the same argument as Hohmeyer's Theorem 4 inside a box) — and (b) a *single-component
  test* holds — the curve meets the cell boundary in exactly two points; then the curve in the
  cell is a single arc between those boundary points and can be traced or approximated with
  guaranteed topology, without subdividing to the geometric tolerance. Hanniel & Elber give
  the single-solution termination criterion for zero-dimensional systems via dual
  hyperplanes.
- **Relevance to cadrs.** `ssi_numeric` / `silhouette`: this is the box-level version of the
  Hohmeyer criterion, usable directly in a uv-box contour: a box passes if the normals bound
  test holds and the zero set crosses its border exactly twice; only failing boxes are split.

### 16. Elber — Gauss maps of freeform surfaces (visibility and SSI support)

- **Citation.** G. Elber, "Freeform surface analysis using Gauss maps and visibility
  sets" (title paraphrased from the abstract; 1995, University of Utah GDC publication
  `elber95a`), 9 pages.
- **URL / local copy.** https://www2.cs.utah.edu/gdc/publications/papers/elber95a.pdf →
  `../papers/intersection/Elber-1995-elber95a.md`.
- **Algorithm (abstract read).** A symbolic method to compute the Gauss map of a freeform
  (possibly trimmed) rational surface to arbitrary precision — the normal field N = S_u × S_v
  as a rational surface, mapped to the sphere — and to exploit it for visibility sets; the
  abstract cites Gauss maps and visibility sets as helpful in robust surface–surface
  intersection, referencing Hohmeyer.
- **Relevance to cadrs.** `ssi`: the exact pseudo-normal surface of a B-spline patch (cadrs
  can build it with its symbolic product machinery) is what the loop-detection tests of
  entries 1, 6 and 15 need as input.

---

## Marching/tracing methods: how start points are found so no branch or loop is missed; step control; tangential and singular points

### Takeaway
Every method that guarantees completeness finds a set of *characteristic points* first
(border crossings plus either collinear-normal points, turning points of the projected curve,
or a Gauss-map/normal-cone test that proves no loop can exist in a region) and only then
traces; step control is either validated (interval ODE solver), plane-sweep (Hohmeyer's
F·P = w), or heuristic (OCCT's deflection/angle rules). Tangential branches need the
second-fundamental-form quadratic for the direction and a rank-4 contact Newton for the
correction.

### Cited Findings
- Hohmeyer's loop-detection theorem: with Gauss-map bounds N1, N2 and vectors P1, P2 such that
  P1·n1 > 0, P1·n2 < 0, P2·n1 > 0, P2·n2 > 0 for all normals, the intersection has no loops,
  no singular points and isolated points only on patch boundaries; the recursion "either
  decomposes the intersection problem into sub-problems containing no loops or discovers a
  singularity" (Theorem 5). — [Hohmeyer 1992, thesis pp. 34–40](http://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/Archive/CSD-92-681.pdf)
- Hohmeyer's trace: step planes F·P = w from w0 to w1, Newton on {F(s,t) = G(u,v), F·P = w}
  from the previous solution; "the system of equations has exactly one solution in the region
  … there is no possibility of jumping components … the value δw does not need to be chosen
  carefully." — [Hohmeyer 1992, p. 39](http://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/Archive/CSD-92-681.pdf)
- "If the bounds on the Gauss maps are separable by a plane then there can be no loops"; on
  two cylinders the Sederberg, Sinha, de Montaudouin and Kriezis criteria require arbitrarily
  fine subdivision while this criterion passes at any angular separation. — [Hohmeyer 1992,
  §3.3](http://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/Archive/CSD-92-681.pdf)
- Sinha et al.: a closed loop implies parallel normals on the two surfaces; Sederberg et al.:
  it implies a line perpendicular to both surfaces, provided no normal of one is perpendicular
  to a normal of the other; bounding cones/pyramids make the test cheap; Hohmeyer used
  pseudo-normal patches and linear programming; applying Hohmeyer's criterion to their Fig. 1
  loop example "takes 8 levels of subdivision"; Gauss-map methods "may not work well if the
  intersection curve is self-intersecting". — [Krishnan & Manocha 1996, §1.1](https://graphicsinterface.org/wp-content/uploads/gi1996-11.pdf)
- Krishnan & Manocha, Lemma 1: "If the curve in the real domain consists of a closed
  component, then two arbitrary complex conjugate paths meet at one of the real points
  (corresponding to a turning point) on the loop"; boundary roots by companion-matrix
  eigenvalues, tracing by inverse power iterations. — [Krishnan & Manocha 1996, §3](https://graphicsinterface.org/wp-content/uploads/gi1996-11.pdf)
- Collinear normal points "lie inside every loop and include all singular points"; after
  splitting the patches at them, only border points are needed as starting points. —
  [Patrikalakis & Maekawa, node109](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node109.html)
- Starting points of an algebraic curve are border points, turning points (f = 0 with f_u = 0
  or f_v = 0) and singular points (f = f_u = f_v = 0); bounds on their numbers (e.g. 6, 6, 5
  for a biquadratic) are far larger than what lies in the real square, so solvers restricted
  to the box (IPP, interval Newton) are preferred. — [node103](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node103.html),
  [node105](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node105.html)
- Tracing ODE on a surface: du/dt, dv/dt proportional to (f_v, −f_u), normalised to arc length
  with E, F, G; step-size selection "is the hard part": too large a step strays or loops,
  worst at constrictions where the gradient is small. — [node102](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node102.html)
- Singular point classification by the quadratic in the branch direction from second
  derivatives: two real roots → self-crossing, double → cusp, complex → isolated point. —
  [node104](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node104.html)
- Tangential marching direction from b11 σ'² + 2 b12 σ' t' + b22 t'² = 0 with four discriminant
  cases (isolated contact, branching, order-≥2 contact, unique direction); a validated interval
  ODE solver "verifies the existence and the uniqueness of the solution within that bound" and
  is used "in preventing straying or looping and in resolving singularities"; produces a
  verified parameter-space and 3-D error bound. — [Mukundan et al. 2004](https://diglib7.eg.org/bitstream/handle/10.2312/sm20041397/249-254_mukundan.pdf)
- Mukundan et al. on IPP: it "can exhaustively and robustly find all roots. However, a
  topology resolution of the roots based on adjacency information is complicated … tends to
  be extremely time consuming in the case of tangential, as well as higher order
  intersections. There is also no guarantee that intervals generated do contain a root". —
  [Mukundan et al. 2004, §1](https://diglib7.eg.org/bitstream/handle/10.2312/sm20041397/249-254_mukundan.pdf)
- OCCT's PrmPrm walker: seeds from the polyhedral section lines (midpoint, then near start,
  end, quarter points) and tangent zones; a seed is walked only if farther than 15·Increment²
  from existing lines; step adaptation by status codes (OK → grow after 5 consecutive OK;
  PasTropGrand → restore and shrink; StepTooSmall → ×1.5); contact direction from the null
  direction of the difference of second fundamental forms; loop closure when a new point is
  within 1e-14 (squared) of the first. — [IntPatch_PrmPrmIntersection.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx),
  [IntWalk_PWalking.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx)
- OCCT's ImpPrm walker: interior grid points with state 13 seed closed lines; a closed walk
  that fails to close is re-declared open and continued backwards; interior points passed by
  a line are negated so each loop is walked once; angle limits CosRef3D 0.98 / CosRef2D 0.88,
  deflection test |d1−d4|²·Norme/64 < 0.25·fleche². — [IntWalk_IWalking.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx)
- Barnhill & Kersey handle "tangent points, branch points, and tangent tracks" with a
  step-length approximation and relaxation onto boundaries, using only evaluated positions and
  tangents. — [Georgia Southern record](https://scholars.georgiasouthern.edu/en/publications/a-marching-method-for-parametric-surfacesurface-intersection-3/)

### Inferences
- For cadrs's geometry (analytic surfaces with closed-form normal bounds, and B-splines whose
  normal field is a computable rational surface), Hohmeyer's separability test is cheap, and
  with his plane-sweep trace the marching step size problem disappears for transversal
  branches; tangential branches still need the Patrikalakis/OCCT second-fundamental-form
  machinery.
- For analytic × B-spline (the common boolean case) the cheapest *complete* seeding is
  Patrikalakis's turning points {f = 0, f_v = 0} of the implicit function on the uv box, which
  is a 2×2 Bernstein system; the 4-D collinear-normal system is only needed for
  B-spline × B-spline.

### Gaps
- Grandine & Klein's exact turning-point system and segment-pairing rule could not be read
  (paywalled); Bajaj et al. 1988 and Barnhill & Kersey 1990 are known here only by abstract.
- Hohmeyer's chapter 7 (degeneracy, backtracking) was not read beyond the table of contents.

## Subdivision/lattice methods and algebraic methods for low-degree surfaces: practical status

### Takeaway
Pure subdivision and lattice methods are historically judged incomplete (small loops and
tangential features missed, extraneous loops, data proliferation) and survive only as the
seeding stage of hybrid methods; affine-arithmetic subdivision (2013–2026) revives them for
B-spline × B-spline on GPUs but still lacks a certified topology step. Algebraic methods are
practical for quadrics (exact, closed form) and for the loop seeds of low-degree patches
(resultant matrices, eigenvalues), not for general high-degree patches.

### Cited Findings
- Lattice methods: "the initial grid resolution is chosen beforehand, and it can cause the
  method to miss important features … small loops and isolated points, which arise near or at
  tangency … the way discrete points are joined into branches can be wrong." — [node107](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node107.html)
- Subdivision methods: correct connectivity near singular points is hard to guarantee with
  finite subdivision; small loops missed with polyhedral approximations; extraneous loops;
  "pure subdivision is impractical for many high-accuracy CAD/CAM applications"; recommended:
  adaptive subdivision with local techniques for characteristic points, then marching. —
  [node108](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node108.html)
- "The algebraic degree of the intersection curve of two bicubic Bezier patches can be as high
  as 324 in space and the corresponding curve in the domain can be of degree 108. The
  approaches based on symbolic methods and algebraic solvers are not practical for such high
  degree curves." — [Krishnan & Manocha 1996, §1.1](https://graphicsinterface.org/wp-content/uploads/gi1996-11.pdf)
- A 1980s solid-modelling patent citing Houghton et al. argues that a stricter flatness
  criterion "is not a general solution" to missed intersections in subdivision. — [US patent
  4890242 (via search)](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/4890242)
- Lin et al. 2014: affine arithmetic extended to B-spline basis functions, GPU-accelerated,
  with thinning of strip-shaped regions and point refinement; claims to find every branch and
  handle tangency. — [TVCG 2014 abstract via search](https://doi.org/10.1109/TVCG.2013.237)
- Gao, Lin & Li 2026 use Lin et al.'s subdivision for the point sets and a two-step Mapper
  graph for topology; limitation: closely clustered singular points are merged. — [arXiv:2604.12631](https://arxiv.org/abs/2604.12631)
- Liang, Mourrain & Pavone handle implicit curves "of total degree about 80 with coefficients
  of bit-size 200, which resultant-based techniques are not able to treat", certifying the
  topology in regular cells from border points and isolating singular points in ε-boxes. —
  [Liang et al. 2008](https://cs.nyu.edu/~yap/classes/modeling/06f/lect/mourrain-etal-subdiv-curves.pdf)
- Quadrics: Levin's pencil method gives p(u) = a(u) ± d(u)√s(u); Wang, Goldman & Tu classify
  the curve from the roots of s(u) and give a rational parametrisation when it is singular;
  Dupont et al. give an exact "near-optimal" parametrisation in all cases, implemented in the
  QI library. — [Wang–Goldman–Tu abstract via search](https://doi.org/10.1016/S0167-8396(03)00081-5);
  [HAL inria-00186089](https://hal-univ-tlse3.archives-ouvertes.fr/INRIA/inria-00186089);
  [HAL inria-00070341](https://hal-univ-tlse3.archives-ouvertes.fr/INRIA/inria-00070341v1)
- Trocado & Gonzalez-Vega: cutcurve/silhouette analysis with subresultants; singular points on
  the line p1 = q1 reduce to degree-2 equations; Maple timings 0.14–6.5 s. — [arXiv:1903.06983](https://ar5iv.labs.arxiv.org/html/1903.06983)
- Kim & Kim: all circles common to two tori from profile, cross-sectional and Villarceau
  circles by circle–circle and circle–line intersections only. — [Kim & Kim 2000](https://koreascience.or.kr/article/JAKO200011920959088.pdf)
- OCCT: a torus is analytic only when coaxial/aligned; otherwise it is walked parametrically;
  cylinder/cone × quadric are solved exactly by the trigonometric discriminant of a quadratic
  in Z. — [IntPatch_Intersection.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx),
  [IntAna_IntQuadQuad.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingData/TKGeomBase/IntAna/IntAna_IntQuadQuad.cxx)

### Inferences
- cadrs's analytic dispatch already exceeds OCCT's; the remaining exactness gains are the
  Miller–Goldman style conic detection for degenerate natural-quadric pairs and the torus
  circle cases of Kim & Kim.
- Affine-arithmetic box pruning in 4-D is the natural rayon-parallel replacement for a fixed
  uv contour grid when both surfaces are B-splines.

### Gaps
- No open-access copy of Lin et al. 2014 or of Dupont et al. was obtainable by script;
  Houghton 1985, Lasser 1986 and Barnhill–Kersey 1990 are abstract-only.

## Contouring approaches (zero set of a signed distance / 2-D contouring): how topology is guaranteed

### Takeaway
A contour is only guaranteed correct when the region is first decomposed at events — border
crossings, turning points (extrema of one parameter along the curve) and singular points —
so that each piece holds a single monotone arc (Grandine & Klein, Patrikalakis), or when each
cell passes a certified regularity/no-loop test in the Bernstein basis so that its border
points determine the arc (Liang–Mourrain–Pavone, Bartoň–Elber); marching squares with saddle
heuristics gives neither.

### Cited Findings
- Grandine & Klein: topology resolution "determines the structure of the intersection curves,
  including closed interior loops, before they are computed numerically", then the curves are
  the numerical solution of a DAE approximately parametrised by arc length. — [abstract via
  search](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/3666035)
- Their solver "extends the PP algorithm to B-splines in floating point, with equations
  normalized to [−1, 1] and knot vectors normalized to [0, 1] at each iteration". —
  [node109](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node109.html)
- "Grandine and Klein formulate the intersection problem as a differential algebraic equation
  which can be solved as a boundary value problem instead of an initial value problem … the
  algorithm has a hard time dealing with tangential intersections." — [Mukundan et al. 2004](https://diglib7.eg.org/bitstream/handle/10.2312/sm20041397/249-254_mukundan.pdf)
- "Current research primarily employs algebraic methods to compute these characteristic points
  … decomposing the intersection curve into several monotonic segments at the characteristic
  points to determine the topology [Grandine and Klein 1997][Hass et al. 2007]." — [Gao, Lin &
  Li 2026, §2.1](https://arxiv.org/abs/2604.12631)
- Liang et al.: "a simple regularity test … allows us to detect easily when the topology of
  the curve in a cell is uniquely determined from its intersection with the border of the
  cell", using Descartes' rule in the Bernstein basis (Theorem 1) for the border roots. —
  [Liang et al. 2008](https://cs.nyu.edu/~yap/classes/modeling/06f/lect/mourrain-etal-subdiv-curves.pdf)
- Overlap detection via coarse IPP boxes and Theorem 5.9.1 (overlaps end at boundary points). —
  [node111](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node111.html)
- OCCT's implicit × parametric path is a 2-D contour-walk of the quadric's implicit function
  over the parametric domain, seeded from boundary zeros and an interior grid, with hole-filling
  passes pairing unmatched line ends. — [IntPatch_ImpPrmIntersection.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx)

### Inferences
- cadrs's `ssi_numeric` contour can keep its data flow and add two stages: (1) solve for the
  events of the signed-distance function d(u,v) on the uv box — border roots of d, roots of
  {d = 0, d_v = 0} (and {d = 0, d_u = 0}), and the overconstrained {d = d_u = d_v = 0} — with a
  Bernstein box solver; (2) contour each sub-box bounded by events as a monotone arc (or apply
  the Liang regularity test per cell and split only failing cells). This makes loops and
  tangential contacts (d = 0 with ∇d = 0) explicit instead of hoped-for.
- For B-spline × analytic the signed distance to a plane/sphere/cylinder/cone/torus restricted
  to a B-spline patch is itself a (piecewise) polynomial or rational B-spline function of
  (u,v) (via cadrs's symbolic products), so all of the above is exact in the Bernstein basis.

### Gaps
- The Grandine–Klein paper and Hass et al. 2007 could not be read; the exact formulation of
  their turning-point system and of the Hass et al. domain decomposition is unverified.
- Bajaj & Xu's tracing/contouring work (listed in the assignment) was not located within the
  available tool budget.

## Polynomial root finding for the 1-D and 2-D subproblems; point inversion/projection

### Takeaway
The state of practice is a Bernstein-basis box solver (PP/IPP, Elber–Kim, Mourrain–Pavone)
for guaranteed enumeration, with Bézier/quadratic clipping as the fast 1-D kernel, and
Newton/Hu–Wallner polishing; point projection combines per-span control-net pruning (Ma &
Hewitt, Selimovic) with second-order iteration (Hu & Wallner), or, in OCCT, a knot-span grid
with per-cell candidates and `math_FunctionSetRoot`.

### Cited Findings
- PP algorithm: project control points onto each (x_i, f) plane, intersect the 2-D convex hull
  with the axis, intersect intervals across equations, split along non-shrinking dimensions,
  rescale by de Casteljau; "quadratic convergence in one dimension", at best linear in higher
  dimensions; slow for underconstrained systems. — [node42](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node42.html)
- IPP with rounded interval arithmetic keeps the root 0.7 of (x−0.1)(x−0.6)(x−0.7) that the
  floating-point version loses at iteration 9. — [node52](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node52.html)
- Elber & Kim: B-spline constraint functions, sign-change pruning, geometric Newton
  improvement, single-solution termination from normal cones generalising Sederberg & Meyers'
  hodograph test, hypersurface fitting of positive-dimensional solutions. — [Elber & Kim 2001](https://gershon.cs.technion.ac.il/papers/multivar_solution.pdf)
- Mourrain & Pavone "can be seen as an improvement of the Interval Projected Polyhedron
  algorithm" with a univariate Bernstein root finder. — [Pavone's Academia page via search](https://independent.academia.edu/JeanPascalPavone)
- "At double roots, Mourrain's method benefits from preconditioning, which … achieves a nearly
  unit jacobian matrix at the center of the domain"; tensor Bernstein solvers are impractical
  beyond 6–7 variables; the Bernstein polytope + LP alternative matches them for n = 2. —
  [Fünfzig et al. 2010](https://cfuenfzig.lima-city.de/Publications/OptimizationsForTensorialBernsteinBasedSolvers_IJSM2010.pdf)
- Bézier clipping: fat-line clipping iterated between the two curves, "proven to have a
  second-order convergence rate"; cubic Béziers can meet up to nine times. — [Springer 2022
  via search](https://vciba.springeropen.com/articles/10.1186/s42492-022-00114-3),
  [blog via search](https://losingfight.com/blog/2011/07/07/how-to-implement-boolean-operations-on-bezier-paths-part-1/)
- Quadratic clipping: degree-reduced quadratic plus bounding quadratics; convergence rates 3
  (single) and 3/2 (double root); "performs very well in the case of two roots which are
  relatively close". — [modeldb citation via search](https://modeldb.science/citations/238370)
- Manocha & Krishnan's algebraic pruning: "an eigenvalue formulation … computes only selected
  eigenvalues in the intersection domain, using a matrix formulation, power iterations, and
  geometric properties of Bézier curves and surfaces". — [ACNP record via search](https://acnpsearch.tweb-dev.unibo.it/singlejournalindex/11737425)
- Point projection: Ma & Hewitt subdivide into Bézier pieces, select candidates by control
  polygon/net, then Newton–Raphson; Selimovic improves this "using selective sub-division …
  based on distance to the end (corner) point"; Chen et al. criticised the hyperplane condition
  and proposed circular clipping. — [MDPI Algorithms 2020 via search](https://www.mdpi.com/1999-4893/13/4/82)
- Hu & Wallner's method is second-order convergent for projection and third-order for
  inversion, independent of the initial value (planar curves). — [Symmetry 2017 via search](https://www.mdpi.com/2073-8994/9/10/210)
- Hyperbook: minimise the squared distance, not the distance, "because the derivative of the
  plain distance can diverge as it approaches zero"; subdivision errors can mislead box tests,
  so rational arithmetic is suggested. — [node66](https://web.mit.edu/hyperbook/Patrikalakis-Maekawa-Cho/node66.html)
- OCCT `Extrema_GenExtPS`: grid by knot spans/degree, sentinel borders, per-cell edge and face
  candidates, `math_FunctionSetRoot` polish; 300 samples cap "used for singular points". —
  [Extrema_GenExtPS.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingData/TKGeomBase/Extrema/Extrema_GenExtPS.cxx)
- OCCT curve/surface: conics by `IntAna_IntConicQuad`, general curves on quadrics exactly,
  otherwise polygon × polyhedron interference then Newton; overlapping segments
  "// Not implemented". — [IntCurveSurface_HInter.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntCurveSurface/IntCurveSurface_HInter.cxx)

### Inferences
- One Bernstein box solver in cadrs (`nurbs`) with: sign-change pruning, PP projection
  shrinking, Mourrain–Pavone Jacobian preconditioning, Elber–Kim's normal-cone single-solution
  stop, and Bézier/quadratic clipping in 1-D would serve curve/curve, curve/surface, point
  projection, `distance` stationary points, silhouettes and all SSI characteristic points;
  rounded interval arithmetic on the control coefficients (IPP) is the only way to make its
  enumeration a guarantee rather than a heuristic.

### Gaps
- No open-access text of Sederberg & Nishita 1990, Bartoň & Jüttler 2007, Ma & Hewitt 2003,
  Selimovic 2006 or Hu & Wallner 2005 was obtained; the Mourrain–Pavone JSC paper was read only
  through secondary descriptions.

## Quadric–quadric and torus intersections in closed form, degenerate and tangential cases

### Takeaway
Numerically the robust formulations are the *geometric* ones for natural quadrics (Goldman &
Miller, Shene & Johnstone: detect conics by configuration tests, never by factoring the
quartic) and Wang–Goldman–Tu's root analysis of s(u) for morphology; exact algebraic
parametrisations (Dupont et al., Farouki et al.) are correct but need rational input and exact
arithmetic; for tori only the circle components have a closed-form treatment (Kim & Kim).

### Cited Findings
- See entries 8–9 above (Levin, Wang–Goldman–Tu, Wang–Joe–Goldman, Dupont et al., Farouki et
  al., Goldman–Miller, Shene–Johnstone, Trocado & Gonzalez-Vega, Kim & Kim) with URLs.
- "Levin's method often fails to find the intersection curve when it is singular and generates
  a parameterization that involves the square root of a quartic"; whether the Wang et al.
  refinement "is numerically robust is open to question". — [Trocado & Gonzalez-Vega 2019](https://ar5iv.labs.arxiv.org/html/1903.06983)
- OCCT checks each discriminant root by substitution at 1e-8 because "the direct search gives
  unreliable results", samples the sign three times per interval, and widens intervals for
  cone apex/sphere pole hits "computed with some inaccuracy". — [IntAna_IntQuadQuad.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingData/TKGeomBase/IntAna/IntAna_IntQuadQuad.cxx)

### Inferences
- cadrs's closed-form pairs should keep an explicit classification step (Wang–Goldman–Tu
  morphology or a geometric conic test) before sampling, so tangential quadric contacts become
  a vertex or an exact conic rather than a near-degenerate fitted curve.

### Gaps
- The Dupont et al. and Wang et al. full texts were not accessible to scripts; the 1979 Levin
  paper and Farouki et al. 1989 are cited from memory/metadata only.

## What OpenCASCADE and other kernels do, and what makes them faster or more robust

### Takeaway
OCCT is a three-way dispatch (analytic × analytic exact; analytic × parametric 2-D walk of the
implicit function; parametric × parametric 4-D walk seeded by a polyhedral intersection), with
heuristic loop/tangency handling and a simultaneous least-squares fit of the 3-D curve and both
p-curves; its speed comes from never sampling analytic pairs and from the one-pass fit, its
robustness from many tuned fallbacks rather than from any topological guarantee.

### Cited Findings
- Dispatch rules, torus analytic conditions, fallback from GeomGeom to ParamParam, default
  deflection 0.01 and UVMaxStep 0.01, TolArc/TolTang clamp inconsistency. — [IntPatch_Intersection.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx)
- Simultaneous 3-D + 2×2-D fit with tolerance 1e-7, degrees 4–8, parametrisation type per
  piece; degree-1 polyline fallback; p-curves by `GeomProjLib::Curve2d` with retry; "Keep 3D
  error control for the distance to the other surface." — [GeomInt_IntSS_1.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_IntSS_1.cxx)
- Segment classification by midpoints with tolerance 35·PConfusion; "Implicit-Parametric
  intersector does not respect domain of the quadric surface." — [GeomInt_LineConstructor.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineConstructor.cxx)
- Walking details (seeds, step codes, contact direction, closure) as in the first question's
  findings. — [IntWalk_PWalking.cxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx),
  [IntWalk_IWalking.gxx](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/master/src/ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx)
- Hohmeyer's thesis was "tested on real manufacturing data and on pathological examples" and
  "needs no tolerances beyond those of machine arithmetic". — [Berkeley TR page](https://www2.eecs.berkeley.edu/Pubs/TechRpts/1992/6140.html)

### Inferences
- Where cadrs is slower than OCCT (fillets 9–10×, sweeps 2.4× per the README), the SSI work
  is dominated by B-spline × analytic and B-spline × B-spline pairs; the two OCCT ideas worth
  copying are the one-pass fit of curve + p-curves and seeding from a coarse polyhedral
  intersection, and the two ideas worth *not* copying are the grid-based loop seeds and the
  tolerance-tuned duplicate/closure tests, which entries 1–5 replace with guarantees.

### Gaps
- Published descriptions of Parasolid's or ACIS's SSI were not searched (budget); SMLib's
  approach is known to the project from its fillet executive only.

---

## What I could not find

- An open-access copy of Grandine & Klein 1997 (CAGD) or of Hass, Farouki, Han, Song &
  Sederberg 2007 (Adv. Comput. Math.); the Grandine–Klein algorithm is described above from
  its abstract, from Patrikalakis & Maekawa, Mukundan et al. and Gao et al., plus marked
  recollection.
- Sederberg & Meyers 1988, Sederberg–Christiansen–Katz 1989, Sinha–Klassen–Wang 1985 (no DOI
  or OA copy located; described from Krishnan & Manocha's and Elber–Kim's summaries).
- Sederberg & Zundel "Loop destruction using Bézier clipping": the BYU ScholarsArchive PDF
  exists (https://scholarsarchive.byu.edu/cgi/viewcontent.cgi?article=2149&context=facpub) but
  is behind a Cloudflare interstitial for scripts; download it from a browser.
- Dupont, Lazard, Lazard & Petitjean (JSC 2008, parts I–III) and Lazard, Peñaranda & Petitjean
  (RR-5670): HAL PDFs are behind the Anubis bot check; LORIA/INRIA author pages were
  unreachable (connection refused) during the session.
- Krishnan & Manocha TOG 1997 and Manocha & Canny SMA '91, Levin 1976, Farouki–Neff–O'Connor
  1989: ACM "bronze/gold" open-access PDFs blocked by Cloudflare for scripts (URLs given).
- Wang, Goldman & Tu 2003 (HKU repository, 403) and Wang, Joe & Goldman 2002 (closed).
- Lin, Qin, Liao & Xiong 2014 (TVCG): no author copy; de Figueiredo 1996 (GI): file not located.
- Mourrain & Pavone 2009 (JSC) full text (HAL blocked); Bartoň & Elber 2011, Hanniel & Elber
  2007, Aizenshtein–Bartoň–Elber 2012: DOIs not verified, no PDFs (Elber's site gave no
  listing); Bajaj & Xu tracing papers: not located.
- Sherbrooke 1994 and Mukundan 2005 MIT theses: DSpace bitstreams returned HTTP 429
  (retry-after 900 s); URLs given above.
- Hu & Wallner 2005 author PDF (TU Graz, timeouts); Ma & Hewitt 2003, Selimovic 2006,
  Bartoň & Jüttler 2007, Sederberg & Nishita 1990: closed, no OA copy.
- Efremov, Havran & Seidel 2005 (numerically stable Bézier clipping for ray/NURBS): not
  searched before the budget ran out; Kriezis, Patrikalakis & Wolter 1992 (CAD 24(1):41–55):
  record only, no DOI confirmed.
- Any published description of Parasolid/ACIS SSI internals.
