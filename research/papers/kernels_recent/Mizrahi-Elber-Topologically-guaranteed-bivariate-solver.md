5

10

15

20

## Abstract

We present a subdivision based algorithm to compute the solution of an under-constrained piecewise polynomial system of n -2 equations with n unknowns, exploiting properties of B-spline basis functions. The solution of such systems is, typically, a two-manifold in R n . To guarantee the topology of the approximated solution in each sub-domain, we provide subdivision termination criteria, based on the (known) topology of the univariate solution on the domain's boundary, and the existence of a one-to-one projection of the unknown solution on a two dimensional plane, in R n . We assume the equation solving problem is regular, while sub-domains containing points that violate the regularity assumption are detected, bounded, and returned as singular locations of small (subdivision tolerance) size. This work extends (and makes extensive use of) topological guarantee results for systems with zero and one dimensional solution sets. Test results in R 3 and R 4 are also demonstrated, using error-bounded piecewise linear approximations of the two-manifolds.

Keywords: Subdivision Solvers, Polynomial Equations, Implicit Surfaces, B-spline Basis Functions.

## 1. Introduction and Related Work

25

The general problem discussed in this paper is the solution of equation systems of the form:

<!-- formula-not-decoded -->

where the multivariate function F is a piecewise polynomial, defined on some compact n -dimensional box D ⊂ R n with values in R n -2 . We assume ¯ 0 ∈ R n -2 is a regular value (as defined in Section 2) of F .

The need for an efficient and robust method for finding all solutions of equation systems (1) in a given domain arises in a variety of fields such as Computer Aided Design (CAD), engineering, robotics, and in fact whenever the geometric constraints of the problem can be formulated as a set of algebraic (non-linear in general) equations. Bisector surfaces [1], sweep-surfaces [2], medical iso-surfaces [3], and the possible states of any 2-DOF (Degrees of Freedom) kinematic system (Ch.06 of [4]) are all examples of application domains that can be mapped to an under-constrained system with a bivariate solution in R n , where n ≥ 3. 35 40 45

The general problem of solving non-linear algebraic constraints with any solution set's dimension (not necessarily two) is typically addressed via either local methods and/or global methods. The local methods refer to a family of numeric iterative algorithms, such as the NewtonRaphson procedure or other prediction-correction meth-

50

ods. Although very useful and typically of quadratic convergence, these techniques depend on the quality of the initial candidate/starting point, and cannot guarantee the global solution: all roots or all connected components of the solution manifold, and its topological properties.

As for global methods, the problem of finding all (real or complex) solutions for equations such as (1) has been mainly addressed via three major approaches. Algebraic geometry techniques (elimination theory and the use of Gr¨ obner bases, [4]) and Homotopy techniques [5] are typically less efficient for finding only the real roots in a bounded domain, due to various practical considerations (for a detailed survey refer to [6, 7]). The third class of methods (and the one discussed in this paper), are the subdivision techniques , which are reviewed in Section 3, and are the framework for the results introduced herein.

The main result in this paper is a topologically guaranteed subdivision termination condition for equation systems with bivariate solutions (two-manifolds). The technique extends and exploits previously achieved topological guarantee results for the zero and one dimensional solution manifolds in [8, 9, 10]. Since these results play a significant role in the development of the methods introduced in this paper, they are reviewed in more detail in Section 3. Subdivision algorithms with topological guarantee are known for implicit regular surfaces in R 3 , for example as in [11, 12]. To the best of our knowledge, the criteria

## Topologically Guaranteed Bivariate Solutions of Under-constrained Multivariate Piecewise Polynomial Systems

Jonathan Mizrahi

Department of Mathematics, Technion, Israel

Gershon Elber

Department of Computer Science, Technion, Israel

Submission ID: P000089

55

60

65

70

75

80

85

90

95

100

introduced in this paper is the first to provide topological guarantee for two-manifold solutions in R n , when the co-dimension is greater than one ( n &gt; 3).

The problem of guaranteeing the topological properties of manifolds given as (solutions of) algebraic constraints has been widely addressed. Methods for guaranteeing the topology of implicit plane curves are presented in [13, 14], and are based on locating the critical points and subdividing the domain such that each piece of the solution is a monotone arc. Further, a numerical method with topological guarantee for implicit planar curves is given in [15], which also detects isolated singularities and computes their degree, using the number of connected components of certain topological structure in the neighborhood of the singularity. The topology of implicit surfaces has also been extensively investigated in R 3 , within the framework of ϵ -sampling ([16] and more), in the contexts of polygonization and rendering [3, 17, 18, 19], with results for the non-regular cases as well [20, 21]. More recent results for level sets of a given implicit function in R 3 are in [11, 12], presenting algorithms for correct connectivity of the reconstructed surface by a careful case analysis and subdivision until ambiguity can be resolved, using additional information such as parameterizability and bounding gradient norm. In [22], methods for contouring one- and twomanifolds, generally in R n , are proposed, but without a topological guarantee. In [21], results from Morse theory are used to locate and classify singular points of an implicit surface in R 3 , and conclude correct topology via interactive polygonization. Other related aspects of nonlinear systems are treated in [23] on dimension reducing, in [24] on expression trees representation, in [25] on parallel computation and convergence guarantee of Newton-Raphson via the Kantorovich theorem, and more. Our focus in this paper is mainly on the subdivision stage, the topological properties of the solution set (rather than the numeric properties) in an attempt to guarantee the topology of the result, and we make frequent use of the properties of the B-spline representation.

105

110

The rest of this paper is organized as follows: Section 2 introduces the terminology and the problem setting. Section 3 provides a brief review of the subdivision paradigm for algebraic constraints solving, focusing on useful results to our problem. Section 4 provides the solution to the problem of a topologically guaranteed subdivision termination criteria for under-determined systems with bivariate solution spaces. Section 5 introduces a triangulation method for the numeric reconstruction in the topologically guaranteed sub-domains. Some examples are given in Section 6, and finally, Section 7 concludes and discusses further optional problems for this research. 115 120

## 2. Notation and Terminology

125

We now provide the problem formulation, state general assumptions, and review frequently used basic concepts. Let F be a multivariate function defined on some compact, n -dimensional box: D = [ a 1 , b 1 ] × ... × [ a n , b n ] ⊂ R n , 130

with values in R n -2 . For differentiability considerations, F can always be viewed as a function defined on some open set U ⊂ R n containing D. Unless otherwise stated, F is at least C 1 smooth, and is typically given in a tensor product B´ ezier or B-spline form (a detailed study of these representations can be found in [26]), generally represented as:

<!-- formula-not-decoded -->

The points P i 1 ...i n ∈ R n -2 are the control points of F , and the B i j ,m j 's are the B-spline basis functions of order m j of F . The zero set of F is denoted by:

<!-- formula-not-decoded -->

Our main interest is in Z ∩ D, the region of the zero set belonging to the compact domain. The concepts we now refer to, can be found in textbooks on topology and differentiability such as [27, 28]. Recall that a homeomorphism is a continuous function between topological spaces that has a continuous inverse. Homeomorphisms are the equivalence relation between topological spaces. When we later claim that the 'topology of the solution set is guaranteed', we mean that we have successfully classified the solution set 'up to homeomorphism'.

Omitting the precise and somewhat technical definition [27, 29], recall that a two dimensional sub-manifold of R n , ( n ≥ 2) is a subset M ⊂ R n , such that every p ∈ M has a neighborhood (in the topology inherited from R n ) that is homeomorphic to (an open subset of) the plane R 2 . The ability to treat the solution set as a manifold is made possible by the regularity assumption which we now describe. Denote by f i the i 'th scalar component of F. The Differential of F at a point p ∈ D is the linear map denoted by dF p , represented in the standard bases by the ( n -2) × n matrix:

<!-- formula-not-decoded -->

Definition 1. Let F : U ⊂ R n → R n -2 be a differentiable mapping of an open set U ⊂ R n . A point p ∈ U is defined to be a critical point of F if the differential dF p : R n → R n -2 is not surjective. The image F ( p ) of a critical point is called a critical value of F . A point a ∈ R n -2 that is not a critical value is called a regular value of F .

From Definition 1, it is evident that for c ∈ R n -2 to be a regular value of F, dF p is surjective for all p ∈ F -1 ( c ) . When this is the situation, F -1 ( c ) is called a regular level set . Our regularity assumption is that the solution F -1 ( ¯ 0) is a regular level set. As another interpretation, the regularity assumption is equivalent to the linear independence of the n -2 gradients of f i , for all p ∈ Z .

The term 'general position' usually refers to a situation where an arbitrarily small perturbation of non-regular input, re-attains regularity. Put differently: the configurations of the input that do not satisfy the assumption,

135

140

145

150

155

160

165

occur with zero probability. 1 Indeed, this is precisely our case, as stated in Sard's theorem [27], which roughly says that the set of critical values of a smooth function has measure zero in the range. In our setting, this is equivalent to saying that if Z = F -1 ( ¯ 0) is not a regular level set, then for any arbitrarily small ϵ &gt; 0, there's another value c ∈ R n -2 such that F -1 ( c ) is a regular level set, and ‖ c ‖ &lt; ϵ. Although rare in the pure mathematical sense, a non-surjective differential is in fact a common configuration in real life problems (i.e. tangential curves/surfaces). We briefly address this issue in Section 7. 170 175

Finally, the following theorem is what enables us to use the properties of a manifold for regular level sets:

Theorem 1 (Regular Level Set Theorem) . Every regular level set of a smooth map F is a closed, embedded, submanifold whose co-dimension is equal to the dimension of the range of F.

180

185

The proof is a consequence of the implicit function theorem (for manifolds) and can be found in [27, 29]. We summarize our setting: The solutions our problem formulation produces are two dimensional manifolds (possibly with several connected components), trimmed to a compact box D. Trimming the solution to D introduces a piecewise-smooth boundary that did not exist before (and may also have more than one component). Therefore, boundary points of Z ∩ D can originate only from the intersection with the domain's boundary, namely: 190

195

<!-- formula-not-decoded -->

For the last relation to become an equality, another general position assumption is required: the intersection of Z with ∂D is transversal, not tangential. For example: a sphere that is tangent to one of the domain's faces - is a surface without a boundary, but does have a non-empty intersection with ∂D. For the theoretic derivation we shall exclude such cases. However, in practice, a much less strict assumption is required. Tangency points and curves on the boundary that are detected by the univariate solver can be handled: isolated tangency points are ignored, and as will be revealed by the methods in Section 4 - the topological classification using the information attained from the univariate boundary solution, is not harmed. Further, if tangency with the boundary occurs along curves, one of the suggested tests introduced in Section 4 will not pass and the domain will be subdivided. A degeneracy case that is not handled, however, is the case of an entire 2D surface patch contained in the domain's boundary. This shall fail the univariate solver used for the boundaries, and must be excluded in this work (detecting such situations is a challenge for future work). 200 205 210 215

220

1 The precise definition for general position varies according to context. A more detailed exposition can be found in Ch. 1 of [30].

## 3. Subdivision Based Solvers - Background and Previous Results

We review the rather general framework of subdivision solvers that is relevant regardless of the dimension of the solution set. When referring to subdivision solvers we generally mean two steps. The first is the recursive subdivision of the domain D until either one of the following holds for some sub-domain D i :

1. No solutions can exist in D i . This is usually detected using the convex hull property [26], which enables ruling out the possibility of solutions in D i by inspecting a much simpler, typically convex, bounding volume, providing a sufficient, but not necessary condition for domain purging.
2. Some other subdivision termination criteria is satisfied. The simplest one may be the size of the domain. A more advanced requirement is some topological guarantee on the region of the solution set belonging to D i .

The second step is the numeric improvement (or refinement) step, which reconstructs the approximation of the solution in D i , preferably with the topological information attained in the previous step. The solutions of neighboring sub-domains are then united and the (binary) recursive subdivision tree is gradually resolved.

The use of subdivision solvers originated first for univariate problems such as in [31], later for fully constrained multivariate systems in [7, 9] as well as for under-constrained systems in [10, 32]. The need to guarantee the topological properties of the solution (number of roots, loops and connected components of curves, etc.) gave rise to the concept of bounding cones and pyramids, appearing in [33, 34] and generalized in [8, 10, 35]. This notion is frequently used in this paper, and reviewed herein.

When referring to the normal cone (Figure 1) of a hyper-surface (any manifold of co-dimension one) we mean any double-cone that contains all possible normal vectors of the manifold. In the implicit representation case, the i 'th hyper-surface of our problem is represented by f i ( x 1 , ..., x n ) = 0, and hence any cone containing the set {∇ f i (¯ x ) , ¯ x ∈ D } can be considered as a normal cone of f i . Such a cone is usually computed using the control points of ∇ f i , and is characterized by an axis and a span angle (namely, this approach chooses a circular cone, but this need not always be the case). This concept is widely used in the analysis of NURBS surfaces, and the smaller the span angle the 'flatter' the surface is. Finally, by considering the complementary angle and the same axis, we obtain the tangent bounding cone (Figure 1) - with the property that it contains all possible tangent directions of the implicit hyper-surface f i (¯ x ) = 0.

We give special attention to three important previous results that are essential for the termination criteria in this work. The first is the tangent cone's non-overlapping test from [9]. In [9], it is proved that for n equations with n unknowns, if the corresponding n tangent cones share only the origin as their common point, there exists at most a 225 single solution in the domain. This test is referred to as the cone test and abbreviated CT. A tractable algorithm is provided in [10] to compute the CT, using a dual hyperplane representation of the hyper-cones, enabling to test the criterion by means of solving linear systems. Next, in 230 [8] the No Loop Test (NLT) and the Single Component Test (SCT) are introduced, for cases with univariate solutions. The NLT inspects the tangential image on the unit hyper-sphere ( S n -1 ) of the (possible) solution curve in the domain, and if this image is contained in a semi235 sphere, then it is impossible for the solution curve to contain closed loops. For a curve with no loops, the SCT guarantees it has a single component in the domain by testing for exactly two boundary intersections: one entry point and one exit point from the domain. The CT, NLT 240 and SCT are generalized in this work, with several new ingredients, as bivariate solutions naturally introduce more options in the process of the topological classification.

Figure 1: The normal cone C N i in red and the tangent cone C T i in blue associated with a hyper-surface f i (¯ x ) = 0 in green, in a bounded domain, D .

<!-- image -->

245

## 4. Topologically Guaranteed Subdivision Termination for Bivariate Solutions

250

255

We can now precisely describe the problem and our proposed solution. Before going into detail, we give a more intuitive and motivational overview in Section 4.1. In Section 4.2, the Single-Loop Boundary Test (SLBT) is introduced, followed by the Injective Projection Test (IPT) in Section 4.3. Then, we summarize the algorithm in Section 4.4. In order to formulate the problem of the subdivision termination criteria with topological guarantee for bivariate solutions, the local question is considered first: the topology of Z ∩ D i for some sub-domain D i ⊂ D. The global homeomorphism of the approximated solution in D and the exact solution is considered later, and shall follow from the local treatment in D i . 280 285 290

260

265

270

275

## 4.1. Motivation and Outline of the Proposed Solution

The motivation comes from the lower dimensional cases: The classification of a zero-manifold amounts to counting the points. For the classification of one-manifolds, counting connected components is not enough, it is still required to distinguish S 1 (a closed loop) from an interval. The fact that these are the only options for one-manifolds is the content of the classification theorem for one-manifolds (refer to the appendix in [36] for a proof).

An important step is that the topology of an interval was chosen as the termination criteria in [8]. Viewing the interval as the 'one dimensional disc' we are lead to define:

Definition 2. Let D i ⊂ D be a sub-domain such that its interior has a non-empty intersection with the zero set. We say that the topology of Z ∩ D i is guaranteed if Z ∩ D i is homeomorphic to a closed disc.

Figure 2 presents some cases where the topology of Z ∩ D i cannot be guaranteed.

Figure 2: The regions (in green) of the zero set surface, Z , belonging to sub-domain D i ( Z ∩ D i ). The topology of Z ∩ D i is not guaranteed, for several reasons: it is not connected, one component is closed, and one component has a nonzero genus.

<!-- image -->

Inspired by the method in [8], the No Loop Test for curves should be generalized to a test guaranteeing no closed surfaces (i.e. surfaces that are compact with no boundary). To proceed with the analogy, the Single Component Test for curves in [8] tested that the boundary solution is the boundary of an interval (one entry point and one exit point). Hence, it should be generalized to a test checking that the boundary solution is a simple closed loop: the boundary of a closed disc. If this is correct, it should guarantee that the solution is a connected surface with a single boundary curve. But there's still topological ambiguity that did not appear in the lower dimensional case: a connected, compact (orientable) surface with a single boundary component can, topologically, be a disc, a torus minus a disc, a double torus minus a disc, etc. In other words - determining that the solution has genus zero is still required. On the other hand, such a 'Genus Zero Test' would complete the classification, as this is exactly

295

300

305

310

315

320

325

330

335

340

345

350

the content of the classification theorem for compact, oriented two-manifolds. (Ch. 5 of [37] or Ch. 12 of [28]).

It turns out that the above described scheme indeed works, but that some of the tests we propose cover a bit more than needed - enabling to skip others. Thus, the final form of our solution outline is as follows, for a sub-domain D i : 355

1. Guarantee that the solution has (at most) a single loop as its boundary curve. We call this the SingleLoop Boundary Test (SLBT). 360
2. Guarantee that there's a two dimensional plane into which there exists a one-to-one projection of the solution. We call this the Injective Projection Test (IPT).

Upon failing any of these two steps, the domain shall be subdivided. The rest of this section is devoted to proving that passing both the SLBT (Section 4.2) and the IPT (Section 4.3) implies that the solution is either empty or is a topological disc, to correctly choosing from these two alternatives, and to providing practical algorithms to execute these tests. 365 370

Remark 1 . The suggested methods guarantee the topology 'up to subdivision tolerance', in the sense that topological features that are larger than subdivision tolerance are guaranteed to be detected before the subdivision terminates. Other thresholds that effect the topological classification are the numerical tolerance that terminates the Newton-Raphson steps, the prediction step size of the curve tracing on the domain's boundary, the maximal allowed triangle edge length in the tessellation process, and the concatenation threshold - under which points are considered equal, for the connectivity of boundary curves. Typical choices (for a unit size problem) used for the examples in this work are 10 -3 for subdivision tolerance, 10 -10 for numeric iterations, 10 -4 for tracing step size, 0 . 02 for the maximal triangle edge length, and 10 -5 for the concatenation of boundary curves. 375 380

385

Remark 2 . In what follows, the linear independence of the gradients at solution points (the regularity assumption) is essential. It shall enable the theoretical derivation of the topological conclusions, as well as the termination proofs of the algorithms. However, we now shortly describe the behavior of the suggested method for non-regular input, which is demonstrated later using the implicit equation of the Steiner surface (Figure 6) -a highly non-regular equation. In such cases, sub-domains that contain singular solution points are constantly subdivided since the tests will fail. The new (smaller) sub-domains either do not contain the singularity anymore, or still contain it. In the latter case, subdivision shall terminate only upon reaching subdivision tolerance (smallest handled sub-domain size), and return as 'unknown'. Consequently, for non-regular input, regular areas are successfully resolved, and singular solution points are bounded by a set of small sub-domains. The subdivision tolerance directly bounds/governs the size of the regions holding singularities. 390 395 400

## 4.2. The Single-Loop Boundary Test

The first step towards the required homeomorphism is given by what we can deduce regarding the topology of the solution surface, from the topology of the boundary curve. We distinguish two cases: an empty boundary, and a single loop boundary. A boundary with more than one connected univariate component is considered as a failure of the test. A useful and well known property of manifolds with boundary we rely on, is that the boundary of a twomanifold must be a one-manifold with no boundary [27]. By compactness - it is a set of closed loops.

Lemma 1. Let D i ⊂ D be a sub-domain such that the boundary solution (curve) is empty. Then, exactly one of the following holds:

1. The sub-domain D i may be purged: Z ∩ D i = φ.
2. Every connected component of Z ∩ D i is a closed surface.

Proof. If the solution is non-empty it is compact, and by assumption it has no boundary, hence all of its components are closed surfaces by definition.

The importance of this seemingly trivial fact will be revealed later: once we establish the next (IPT) test, its success will enable us to decide between the two options Lemma 1 provided. This means that we shall be able to purge sub-domains due to purely topological considerations, even if they were not purged by less delicate purging tests (usually based on the control coefficients and their signs). Next, the non-empty boundary case is treated, when a single loop is detected:

Theorem 2. Let D i ⊂ D be a sub-domain such that the boundary solution (curve) is a single (connected) loop. Then, exactly one of the following holds:

1. The solution surface, Z ∩ D i is connected.
2. The solution surface, Z ∩ D i , has one component that is not a closed surface, and other components, all of which must be closed surfaces.

Proof. The two alternatives clearly are mutually exclusive. If Z ∩ D i is connected, the first option holds, and we are done. Assume Z ∩ D i is not connected. The main assumption is that the unique loop on the domain's boundary is connected, hence it is entirely contained in one of the surface components, now denoted C 1 . Therefore, C 1 is not a closed surface, as it is a (connected) surface with a nonempty boundary. In order to show that the second option of the theorem holds, it remains to justify why any component other than C 1 , denoted C 2 , must be a closed surface: Since C 1 and C 2 are disjoint, C 2 has no common points with the boundary loop. Thus, had C 2 not been a closed surface, it would have a non-empty boundary, consisting of points that are interior to D i : a contradiction the regularity assumption and Theorem 1.

405

410

415

420

425

430

435

440

445

450

455

Here as well, choosing the correct of the two alternatives Theorem 2 guarantees, will be made possible by the success of the next (IPT) test, revealed soon. The incorporation of Lemma 1 and Theorem 2 in our solver is achieved as follows: at all recursion subdivision levels, the restriction of the problem to the sub-domain's boundary is solved as part of the new sub-problem construction. For the initial domain D , each of the 2 n faces are resolved and then the intersection curves are properly concatenated, forming the initial boundary curve(s). Then, and since each domain subdivision induces a subdivision of the boundary curve(s), the only new equation solving effort engaged, is for the restriction of the problem to the splitting hyper-plane. Then, a copy of the new 'middle' solution curve(s) is concatenated with each of the two sides, and consequently the two sub-problems have their boundary problems resolved. Finally, the actual SLBT execution amounts to testing if the number of boundary components is zero, one, or more (since the connectivity is treated already, it is simply the number of closed-loop polylines). Components that are isolated points are ignored, and considered as tangency points of the solution and the sub-domain's boundary (not counted as boundary curve components). 460 465 470 475

In terms of algorithm termination, we argue that over a sequence of sub-domains approaching diameter zero, and under the regularity assumption, the number of boundary loops must eventually be zero or one. This follows from the definition of a (smooth) manifold: small enough neighborhoods must be homeomorphic to a disc. More precisely, a situation where arbitrarily small neighborhoods of a point still contain a region of the solution with more than one boundary solution is a contradiction to the existence of a positive injectivity radius [38]: a positive distance from a point p ∈ S for which the exponential map is a diffeomorphism, i.e. a disc in the tangent plane is mapped to a geodesic disc on the surface. For details on the exponential map and the existence of a positive injectivity radius, refer to Ch. 6 of [38]. 480 485 490

## 4.3. The Injective Projection Test

To proceed we need the following trivial observation:

495

Lemma 2. A planar surface M that is compact must have a non-empty boundary. In other words: there are no closed surfaces in the plane.

Proof. The continuity of the norm function, ‖ · ‖ , and the compactness of M imply that there's a point p ∈ M that has maximal distance from the origin. But since M is planar, this farthest point must be a boundary point, hence M has a non-empty boundary. 500

We use the above fact to show the underlying idea behind the IPT:

Lemma 3. Let M be a compact, two manifold with a connected boundary. If there is a homeomorphism ϕ of M into R 2 , then M is homeomorphic to a closed disc.

505

Proof. First, we argue that M is connected. Otherwise, by Theorem 2, M has at least one component that is a closed surface. By assumption, the homeomorphic image, ϕ ( M ), is planar. The restriction of ϕ to the closed surface component is a contradiction to Lemma 2, therefore M is connected. Observe that ∂ϕ ( M ) is a simple, closed, planar curve. By the Jordan curve theorem [37], ∂ϕ ( M ) separates the plane to two connected components, the bounded (interior) one, denoted I , and the unbounded (exterior) one, denoted E , having ∂ϕ ( M ) as their common boundary. Had there been any points of ϕ ( M ) in E , by compactness there would have been another boundary component - a contradiction. On the other hand, all points of I must also belong to ϕ ( M ), since missing points would again imply an additional boundary component. We have shown that ϕ ( M ) coincides with I ∪ ∂ϕ ( M ). It is a sharper version of the Jordan curve theorem (due to Schonflies [37]) that guarantees that I is indeed an open topological disc, hence ϕ ( M ) (and therefore M ) are closed discs, as required. 2

We now give an explicit candidate to take the role of this 'flattening homeomorphism', along with a tractable algorithm to test for its existence.

Theorem 3. Let F : D i ⊂ R n → R n -2 . Assume Z ∩ D i passed the SLBT, and a single boundary component is detected. For i = 1 , ..., n -2 , denote by C N i the normal cone associated with the i 'th scalar component of F. Let e j (the j 'th standard basis element of R n ), be viewed as a degenerate normal cone (with cone angle zero, see Figure 3). Assume there's some pair 1 ≤ k &lt; l ≤ n such that the corresponding n tangent cones obtained from normal cones { C N 1 , ..., C N n -2 , e k , e l } , do not overlap (intersect only at the origin). Then Z ∩ D i is homeomorphic to a closed disc, and the homeomorphism ϕ can be chosen as the projection on directions k, l .

Proof. Consider the projection π : Z ∩ D i → R 2 , mapping ( x 1 , ..., x n ) ↦→ ( x k , x l ) . First, we verify there cannot be two distinct points of Z ∩ D i mapped to the same pair ( u, v ) ∈ R 2 . This is equivalent to the uniqueness requirement of the following n constraints: n -2 of original problem, together with x k -u = 0 and x l -v = 0 . This seemingly infinite ( u, v ) parameterized family of problems can be simultaneously tested for uniqueness: recall the cone test [9] (CT) reviewed in Section 3, guaranteeing uniqueness of the solution (regardless of the question of existence, not to say of the exact solution) for fully determined systems. The tangent cones of the scalar components of F depend on the gradients only, and thus independent of u, v. In particular, the normal cones associated with the last two constraints are exactly e k and e l (the gradients of the respective constraints). The main assumption of the

2 Allowing less elementary tools to prove this simple claim, a more direct proof of Lemma 3 can be given by explicitly computing the Euler characteristic of M . This is a consequence of the Gauss-Bonnet theorem and the total curvature of a closed planar curve [39].

510

515

520

525

530

535

Figure 3: The normal cones taking part in the IPT. In R 3 , The standard basis elements e k , e l are viewed as degenerate normal cones, hence their corresponding tangent cones are the canonical planes. Together with the tangent cone C T of the original surface patch f in green, C T ∩ C T e l ∩ C T e k contains only the origin, and hence yields the injective projection required for the IPT: the injective projection of f (¯ x ) = 0 on the plane inducted by e k , e l is presented in dark green.

<!-- image -->

theorem is exactly the success of CT, which means π is one to one. The continuity of π is obvious from its definition. The continuity of π -1 follows from a well known exercise in elementary topology [28]: a continuous injective function from a compact space ( Z ∩ D i ) to a Hausdorff space ( R 2 in our case) is a homeomorphism onto its image. Therefore, π can be chosen as the 'flattening homeomorphism'.

As a last conclusion, we can finally resolve the ambiguity in the empty boundary case as well (i.e. make the correct choice between the alternatives of Lemma 1), since in the plane there are no closed surfaces (Lemma 2):

Corollary 1. Let D i ⊂ D be a sub-domain for which SLBT detected an empty boundary, and IPT returned that the topology is guaranteed. Then D i may be purged.

The actual test in our solver is executed by checking all possible coordinate pairs until either the corresponding uniqueness condition is satisfied, or, if not, subdivide further. For each choice of a possible pair, we apply the CT to the particular fully determined system: the set of cones is obtained by the cones of the original problem together with two basis elements of R n (taking the role of the degenerate normal cones). The details of the algorithm and its termination proof are described next. 550

## 4.4. The Algorithm

The general subdivision scheme is summarized in Algorithm 1, providing the top level procedure. Each of the closed solution curves obtained, is later triangulated according to the method in Section 5. Then, Algorithm 2, details the IPT, which essentially traverses the optional direction pairs and tests if the projection can be guaranteed to be injective. However, it should be noted that an orthogonal projection on any two dimensional plane 555 560

540

545

is sufficient for guaranteeing the required homeomorphism (not just the optional coordinate planes). Hence, one may choose a better candidate exploiting the geometry of the problem, such as the plane orthogonal to the n -2 gradients, when evaluated at a single point of the unknown surface, such as the center of the sub-domain. Given that the sub-domain is small enough - this option is likely to produce a successful, and possibly more stable, injective projection.

```
Algorithm 1: TwoManifoldSolver( D,F ) input : A Compact hyper-box D ⊂ R n . F = ( f 1 , ..., f n -2 ) : D → R n -2 . output : L - a set of closed solution loops, boundaries of discs in topologically guaranteed sub-domains. 1.1 if Size ( D ) < ϵ then 1.2 /* Too small to decide, terminate. */ 1.3 return NULL; 1.4 for j = 1 , ..., n -2 do 1.5 if all coefficients of f j ∈ F have the same non-zero sign then 1.6 /* No solutions exist in D . */ 1.7 return NULL; 1.8 n ← number of solution loops on ∂D ; 1.9 if n = 0 then 1.10 if IPT( D ) then 1.11 /* No solutions can exist in D . */ 1.12 return NULL; 1.13 if n = 1 then 1.14 if IPT( D ) then 1.15 /* A topological disc. */ 1.16 return Boundary loop; 1.17 /* Either n > 1 or IPT failure: */ 1.18 Subdivide D,F to D 1 , F 1 and D 2 , F 2 ; 1.19 L 1 ← TwoManifoldSolver( D 1 , F 1 ); 1.20 L 2 ← TwoManifoldSolver( D 2 , F 2 ); 1.21 return L 1 ∪ L 2 ;
```

Lemma 4. Over a sequence of subdivisions with sub-domain diameter approaching zero (and under the regularity assumption), Algorithm 2 terminates.

Proof. As the sub-domains approach a point p , by C 1 continuity, each normal cone C N j , associated with f j , approaches a single vector, denoted N j . Equivalently, C T j approach's the single hyper-plane orthogonal to f j 's respective normal. These hyper-planes are linear sub-spaces of R n (contain the origin). The failure of the CT means that there's a point other than the origin belonging to the intersection of these n hyper-planes (and consequently an entire sub-space of dimension 0 &lt; d &lt; n ). We now argue that this cannot happen for all possible selections of a pair k, l of indices. The first n -2 equations, by the regularity assumption, yield a linearly independent set of the N j 's,

565

570

575

580

585

590

```
Algorithm 2: IPT( D i , F ) input : A Compact hyper-box D i ⊂ R n . F : D i → R n -2 for which SLBT returned either zero or one boundary components. output : TRUE if the topology of Z ∩ D i , is guaranteed, FALSE otherwise. 2.1 for j = 1 , ..., n -2 do 2.2 C N j ← The normal cone associated with f j , in D i ; 2.3 C T j ← The tangent cone obtained from C N j ; 2.4 for all pairs 1 ≤ k < l ≤ n do 2.5 T k , T l ← the tangent cones obtained from the basis elements e k , e l ; 2.6 if ⋂ ( C T 1 , ..., C T n -2 , T k , T l ) = ¯ 0 then 2.7 if Boundary solution is empty then 2.8 D i can be purged. 2.9 else 2.10 /* Boundary is a single loop. */ 2.11 Z ∩ D i is homeomorphic to a disc. 2.12 return TRUE; 2.13 return FALSE;
```

595

600

605

610

615

since they are gradients of the original n -2 constraints at p . Now, since { N j } j =1 ,...,n -2 only span a sub-space of dimension n -2, there must exist a pair e k , e l such that { N j } j =1 ,...,n -2 ∪{ e k , e l } span R n . In other words, the CT must succeed and the algorithm terminates on a domain of some (finitely) small enough size. 620

Remark 3 . For the special n = 3 case, guaranteeing an injective projection can be done by finding a non-vanishing partial derivative of the single constraint [11, 12]. 625

We make a final comment to conclude this section. The main goal is that the approximated solution and the exact solution will be homeomorphic, globally in D . However, in each D i , all we know is the existence of a homeomorphism from the exact solution to a closed disc, and hence (using the method we introduce in the next section), we may approximate this disc using a polygonal mesh from the boundary curve - obtaining a local homeomorphic approximation. Uniting the local homeomorphisms to a global one is possible only if, in neighboring sub-domains, we can somehow guarantee that the local homeomorphisms agree on the common boundary. Indeed, this is possible, while we omit the full proof, and only mention that it is a consequence of two well known topological facts: the ability to extend a homeomorphism of closed loops to a homeomorphism of closed discs, agreeing on the boundary loops (known as the Alexander Lemma [37]), and the pasting lemma [28], enabling to unite the carefully constructed local homeomorphisms to a global one, now that they agree on common boundary curves. 630 635 640 645

## 5. Numeric Reconstruction of the Solution TwoManifold

To the extent of the subdivision termination, and with the regularity assumption, the topology is resolved. However, seeking a two dimensional, piecewise linear approximation of the exact solution manifold, we now encounter the problem of reconstructing a (triangular) mesh in D i , close to Z ∩ D i in some sense/metric, and homeomorphic to a disc, with the constraint of a given boundary curve. Versions of this challenge arise in the domains of implicit surface polygonization and/or rendering (reviewed in Section 1) in R 3 . However, the main issue encountered in our problem setting here is that we cannot assume the underlying implicit function is scalar valued (i.e. the codimension of the solution manifold is one). This is an essential assumption used in the previously referred methods: a notion of inside and outside the implicit surface can be assigned to the points of the embedding space. As another example, ray tracing techniques cease to be applicable since a ray shall hit a two-manifold in R n with zero probability, if n &gt; 3. We introduce a method designed to handle this issue, while taking advantage of the topological information obtained so far: we not only know that the solution set we seek is homeomorphic to a closed disc in D i , we also know the specific coordinate pair with respect to which it may be treated as the graph of an explicit function.

Let x k , x l be the coordinates with respect to which the IPT succeeded, let π denote, as before, the orthogonal projection on the k, l coordinates, and let γ denote the known, closed, boundary curve (a polyline) in R n . The problem of triangulating the closed, simple, planar polyline π ( γ ) is solved, by methods that can be found, for example in [40], as well as the refinement of such planar triangulation to any desired tolerance of the triangle edge length. The required bound on the edge length can be determined by the required tolerance of the input function values (zero set of which we seek), as will be revealed shortly.

For any fixed vertex v = ( v k , v l ) of the planar triangulation of π ( γ ), we may consider the problem of solving n -2 equations with n -2 unknowns: F ( x 1 , ..., v k , ..., v l , ..., x n ) = ¯ 0 ∈ R n -2 . From now on we refer to this problem as the the back projection problem .

As a consequence of the IPT success, we have both the existence and the uniqueness of a solution guaranteed, for each such back projection problem. We assume fully determined problems are solved to any required precision and hence solving it yields the missing n -2 coordinates of the point guaranteed to lay on the solution surface, denoted ¯ z ∈ Z.

Finally, the required triangulation is defined as follows: { ¯ z 1 , ¯ z 2 , ¯ z 3 } is a solution triangle if and only if { v 1 , v 2 , v 3 } is a triangle in the planar triangulation of the polyline π ( γ ): each ¯ z i is the elevation of v i to R n via the normal space ('back projection'). It is easily verified that the above construction gives a non degenerate triangulation with the property that γ is indeed its boundary curve,

650

655

660

665

670

675

680

and it is a manifold with a disc topology. Further, for any scalar component f i of F , one can always obtain an upper bound M on ‖ ∇ f i ‖ in D i from the control points. If L is an upper bound on the triangle edge length, by straight forward calculus estimations it follows that for any point p in the triangle: | f ( p ) |≤ √ 2 ML, giving a bound on the numeric error, for the above described triangulation method.

To conclude this section, we briefly mention two technical challenges arising in the implementation. The first is the orientation problem of the triangles: the closed loops, resulting from the subdivision step, need not be correctly oriented. Hence, once triangulated, the situation potentially introduces the solution mesh with regions of incorrect orientation. To resolve this, we require that triangle edges E 1 , E 2 together with the gradients of the original constraints, evaluated at the common vertex ¯ z ∈ R n , form a positively oriented basis for R n . The general form for this requirement is: 685

<!-- formula-not-decoded -->

For n = 3 this coincides with the familiar condition for choosing the normal line in the direction of E 1 × E 2 . Now, if the result is negative the orientation of the triangle is flipped. The second technicality is referred to as TJunctions: missing points on one copy of a common boundary curve, with respect to the second copy in its neighboring sub-domain. This occurs since two sub-domains with a common face undergo independent (and usually different) further subdivision events. Hence, new points are created in the different split locations of the common boundary curve. 690

## 6. Test Results

This section presents some results. To demonstrate the topological correctness of the termination condition we also show the boundary curves at times: these are the closed loops on the boundary of each sub-domain which succeeded to terminate subdivision (SLBT and IPT returned TRUE). Then, we also show the full triangular mesh approximation of the solution.

All examples are solved in the domain [0 , 1] n . It should be noted that the algorithm does not use any symbolic factorization or elimination methods: whenever we choose an equation that factors to a product of simpler polynomial expression - it is for instructive reasons, having a known solution, and it is used in no way during the solution process.

We start with a simple example, which is a product of a plane and a sphere (see Figure 4):

<!-- formula-not-decoded -->

as a cubic constraint, in three variables. As can be seen in Figure 4 (b), the algorithm required further subdivision steps in the areas where the two components are closer, in

695

Figure 4: The solution of Equation (3), showing the solution in (a) and the (piecewise linear) boundary curves in the topologically guaranteed domains on top of this solution, in (b).

<!-- image -->

order to guarantee the topology and distinguish the connected components.

Next, we choose a more topologically challenging example, still in R 3 , of degree 14:

<!-- formula-not-decoded -->

solution of which are seven concentric spheres of geometrically decreasing radii (Figure 5), from 0.8 to about one hundredth. With a subdivision tolerance of 0 . 001, all subdomains are correctly resolved, not reaching this subdivision tolerance.

Figure 5: The solution of Equation (4), showing seven concentric spheres of decreasing radii. The topology is correctly resolved, distinguishing the seven layers.

<!-- image -->

Another topologically challenging example is the implicit representation of the Steiner surface (scaled to a radius of 0 . 3 and translated to the domain's center):

<!-- formula-not-decoded -->

which is not a regular manifold. Indeed, the algorithm fails near the self intersections. However, in the regular regions, the topology is guaranteed. In Figure 6, the guaranteed regions are shown, at three different subdivision levels.

To demonstrate the method for n &gt; 3, we first apply it to the bisector [1] problem between two parametric surfaces: finding the locus of points for which the

700

705

710

715

Figure 6: The solution of Equation (5), showing the regions of the Steiner surface where topology could be guaranteed, with a subdivision tolerance of (a) 0.05, (b) 0.01, and (c) 0.005.

<!-- image -->

(locally) shortest distance to each of the input surfaces, S 1 ( u, v ) , S 2 ( r, s ) , is equal. One possible formulation of this requirement is given by:

<!-- formula-not-decoded -->

where n 1 ( u, v ) , n 2 ( s, t ) denote the normal vectors of the surfaces at the respective points. The first constraint requires that the two normal directions meet at a point, whereas the second constraint has the equidistant semantics (requiring an isosceles triangle is formed by the normal vectors and the line segment connecting the surface points). We have two equations and four unknowns. The solution is a two-manifold in ( u, v, r, s ) space. But each such ( u, v ) or ( r, s ) location determines, by construction, an ( x, y, z ) location on the bisector (for the precise mapping refer to [1]). In Figure 7, the bisector surface of a plane and a bi-quadratic surface is shown: it is the result of properly mapping the solution triangular mesh from R 4 to R 3 , in post-processing. A more complex example is shown in Figure 8, where the bi-quadratic surface admits hyperbolic areas (unlike the previous example where all points are elliptic), causing the normals on both sides of the surface to realize equidistant points with the plane, resulting in several disjoint components. 720

Figure 7: The solution of Equation (6), showing the bisector sheet between two surfaces. (a) presents the subdivision tree while (b) shows the final result. 735

<!-- image -->

Finally, we demonstrate how the presented method can be applied to the problem of sweeping an implicit space curve along a parameterized curve. We adopt the general notion of swept surfaces from the description on p. 472 of [41]. A section curve C ( u ) is swept along a trajectory curve T ( v ), yielding the surface defined by S ( u, v ) =

)

Figure 8: The solution of Equation (6), showing the bisector sheet(s) of two surfaces. (a) presents all components of the solution: the equidistant requirement is satisfied by points that can be found on both sides of the bi-quadratic surface. (b) shows a zoom-in on the 'interior' component between the two input surfaces, which admits self intersections near the corners.

<!-- image -->

T ( v )+ M ( v ) C ( u ) . The 3 × 3 matrix M ( v ) is used for rotating and non-uniform scaling, choice of which is affected by the required regularity of the resulting surface and is a separate challenge on its own. In the examples here we simply use the identity matrix, but the suggested method can be applied for any non-singular matrix M ( v ), given that its elements are polynomial functions of v . Now, consider the following modification: assume a parameterization of the section curve C ( u ) is not in hand (either difficult to attain or not given by a rational formula), but it is given as the simultaneous zero set of f 1 ( x, y, z ) , f 2 ( x, y, z ) . In such a case, in order to sweep the section curve along a parametric trajectory curve T ( v ) , v ∈ [0 , 1], we may consider the two constraints:

<!-- formula-not-decoded -->

Weagain have two constraints with four unknowns, ( x, y, z, v ), yielding a two-manifold solution in R 4 . However, projecting it on the first three coordinates gives the required sweep surface. In Figure 9, solutions for some choices of f i and T ( v ) are shown. The section is a curve on a torus given implicitly by f 1 , intersecting a plane 3 (Figure 9 (a)) or a sphere (Figure 9 (b)) given implicitly by f 2 . This section curve is swept along cubic parametric curves.

To get some idea on the computational costs, the running times are presented in Table 1, tested on an Intel Core i5-2300 CPU, 2.8GHz with 4GB memory. It is evident from these running times that the bottleneck varies according to the problem in hand and can be either the topological guarantee subdivision step or can be at the numeric triangulation approximation stage. Not surprising, an extreme example is the Steiner surface (Figure 6) example, where almost the entire effort is invested into the topological testing, due to the topological failures near the self intersections.

## 7. Conclusion and Future Work

In this paper, we presented an algorithm for solving under-constrained piecewise polynomial systems with two R 3 . However, the exploitation of these introduced solution abilities in the presented and other applications, to their full potential, is yet to be revealed.

3 also known as the Oval of Cassini.

<!-- image -->

## 8. Acknowledgements 765

770

Figure 9: The solution of Equations (7), with f 1 as the implicit equation of a torus, and f 2 as a plane (a) and as a sphere (b). The section curves are swept along cubic curves. 775

| Example     |   Subdivision only (Sec) |   Total time (Sec) |
|-------------|--------------------------|--------------------|
| Figure 4    |                     4.77 |              18.37 |
| Figure 5    |                    29.06 |              34.16 |
| Figure 6(a) |                     1.73 |               1.76 |
| Figure 7    |                     0.96 |              25.84 |
| Figure 8    |                    10.93 |              45.47 |
| Figure 9(a) |                    40.34 |              49.53 |
| Figure 9(b) |                    23.51 |              36.45 |

780

785

790

Table 1: Timing of the different examples presented in this work, with and without the numeric step (triangulation of boundary loops, resulting from the topologically guaranteed termination). 795

degrees of freedom. Our derivation assumed the constraints 740 are independent. Weaker regularity assumptions is a clear further direction of research: we intend to address isolated singularities first. Then, the topological classification of the solution in the presence of non-isolated critical points, is an extremely challenging problem. Another re745 lated problem is adding inequality constraints. Typically, these do not reduce the dimension of the solution, but trim it to a smaller domain. A possible approach may begin by finding the (univariate) boundary of this domain and trimming the solution according to the inequality con750 straints. Finally, optimizing cone and gradient estimates is to be further investigated. As the tangent and normal cones of hyper-surfaces are essential and frequently used tools in subdivision solvers, but at a significant computational cost, methods for constructing tighter bounds on 755 the normal/tangent fields of the input surfaces are to be studied. 800 805 810 815 820

760

Additional points for improvement include, for example, tighter bounds on the (piecewise linear) approximation of the solution manifold, and better ways to estimate the solution normal space, especially if in (projection onto) 825

This work was supported in part by the People Programme (Marie Curie Actions) of the European Union's Seventh Framework Programme FP7/2007-2013/ under REA grant agreement PIAP-GA-2011-286426, and was supported in part by the ISRAEL SCIENCE FOUNDATION (grant No.278/13).

## References

- [1] G. Elber, M.-S. Kim, The bisector surface of rational space curves, ACM Transactions on Graphics (TOG) 17 (1) (1998) 32-49.
- [2] G. Elber, Global error bounds and amelioration of sweep surfaces, Computer-Aided Design 29 (6) (1997) 441-447.
- [3] W. E. Lorensen, H. E. Cline, Marching cubes: A high resolution 3d surface construction algorithm, in: ACM Siggraph Computer Graphics, Vol. 21, ACM, 1987, pp. 163-169.
- [4] D. A. Cox, J. Little, D. O'Shea, Ideals, varieties, and algorithms: an introduction to computational algebraic geometry and commutative algebra, Vol. 10, Springer Verlag, 2007.
- [5] C. Garcia, W. I. Zangwill, Pathways to solutions, fixed points, and equilibria (1981).
- [6] N. M. Patrikalakis, T. Maekawa, Shape interrogation for computer aided design and manufacturing, Springer Verlag, 2002.
- [7] E. C. Sherbrooke, N. M. Patrikalakis, Computation of the solutions of nonlinear polynomial systems, Computer Aided Geometric Design 10 (5) (1993) 379-405.
- [8] M. Bartoˇ n, G. Elber, I. Hanniel, Topologically guaranteed univariate solutions of underconstrained polynomial systems via no-loop and single-component tests, Computer-Aided Design 43 (8) (2011) 1035-1044.
- [9] G. Elber, M.-S. Kim, Geometric constraint solver using multivariate rational spline functions, in: Proceedings of the sixth ACM symposium on Solid modeling and applications, ACM, 2001, pp. 1-10.
- [10] I. Hanniel, G. Elber, Subdivision termination criteria in subdivision multivariate solvers using dual hyperplanes representations, Computer-Aided Design 39 (5) (2007) 369-378.
- [11] S. Plantinga, G. Vegter, Isotopic approximation of implicit curves and surfaces, in: Proceedings of the 2004 Eurographics/ACM SIGGRAPH symposium on Geometry processing, ACM, 2004, pp. 245-254.
- [12] L. Lin, C. Yap, J. Yu, Non-local isotopic approximation of nonsingular surfaces, Computer-Aided Design 45 (2) (2013) 451 462.
- [13] T. A. Grandine, F. W. Klein IV, A new approach to the surface intersection problem, Computer Aided Geometric Design 14 (2) (1997) 111-134.
- [14] J. Hass, R. T. Farouki, C. Y. Han, X. Song, T. W. Sederberg, Guaranteed consistency of surface intersections and trimmed surfaces using a coupled topology resolution and domain decomposition scheme, Advances in Computational Mathematics 27 (1) (2007) 1-26.
- [15] M. Burr, S. W. Choi, B. Galehouse, C. K. Yap, Complete subdivision algorithms, ii: Isotopic meshing of singular algebraic curves, in: Proceedings of the twenty-first international symposium on Symbolic and algebraic computation, ACM, 2008, pp. 87-94.
- [16] J.-D. Boissonnat, S. Oudot, Provably good sampling and meshing of surfaces, Graphical Models 67 (5) (2005) 405-451.
- [17] J. Bloomenthal, Polygonization of implicit surfaces, Computer Aided Geometric Design 5 (4) (1988) 341-355.

- [18] L. H. de Figueiredo, J. de Miranda Gomes, D. Terzopoulos, L. Velho, Physically-based methods for polygonization of implicit surfaces, in: Proceedings of Graphics Interface, Vol. 92, 1992, pp. 250-257.
- [19] C. Bajaj, Introduction to implicit surfaces, Morgan Kaufmann, 830 1997.

835

840

845

- [20] J. Bloomenthal, K. Ferguson, Polygonization of non-manifold implicit surfaces, in: Proceedings of the 22nd annual conference on Computer graphics and interactive techniques, ACM, 1995, pp. 309-316.
- [21] B. T. Stander, J. C. Hart, Guaranteeing the topology of an implicit surface polygonization for interactive modeling, in: ACM SIGGRAPH 2005 Courses, ACM, 2005, p. 44.
- [22] J.-K. Seong, G. Elber, M.-S. Kim, Contouring 1-and 2-manifolds in arbitrary dimensions, in: Shape Modeling and Applications, 2005 International Conference, IEEE, 2005, pp. 216-225.
- [23] T. Grapsa, M. Vrahatis, Dimension reducing methods for systems of nonlinear equations and unconstrained optimization: A review, Recent Advances in Mechanics and Related Fields (2003) 215-225.
- [24] G. Elber, T. Grandine, An efficient solution to systems of multivariate polynomial using expression trees, Visualization and Computer Graphics, IEEE Transactions on 15 (4) (2009) 596604.
- [25] F. Wei, J. Feng, H. Lin, Gpu-based parallel solver via the 850 kantorovich theorem for the nonlinear bernstein polynomial systems, Computers &amp; Mathematics with Applications 62 (6) (2011) 2506-2517.

855

860

- [26] E. Cohen, R. Riesenfeld, G. Elber, Geometric modeling with splines: an introduction. 2001, AK Peters, Ltd., Massachusetts, USA.
- [27] J. M. Lee, Introduction to smooth manifolds, Vol. 218, Springer, 2012.
- [28] J. R. Munkres, Topology, Vol. 2, Prentice Hall Upper Saddle River, 2000.
- [29] M. P. Do Carmo, Riemannian geometry, Birkh¨ auser Boston, 1992.
- [30] J. Matouˇ sek, Lectures on discrete geometry, Vol. 212, Springer Verlag, 2002.
- [31] J. M. Lane, R. F. Riesenfeld, Bounds on a polynomial, BIT 865 Numerical Mathematics 21 (1) (1981) 112-117.
- [32] C. Liang, B. Mourrain, J.-P. Pavone, Subdivision methods for the topology of 2d and 3d implicit curves, in: Geometric Modeling and Algebraic Geometry, Springer, 2008, pp. 199-214.
- [33] T. W. Sederberg, A. K. Zundel, Pyramids that bound surface 870 patches, Graphical Models and Image Processing 58 (1) (1996) 75-81.

875

880

- [34] T. W. Sederberg, R. J. Meyers, Loop detection in surface patch intersections, Computer Aided Geometric Design 5 (2) (1988) 161-171.
- [35] G. Barequet, G. Elber, Optimal bounding cones of vectors in three dimensions, Information Processing Letters 93 (2) (2005) 83-89.
- [36] J. W. Milnor, Topology from the differentiable viewpoint, Princeton University Press, 1997.
- [37] D. Rolfsen, Knots and links, Vol. 7, Publish or perish Berkeley, 1976.
- [38] M. Berger, A panoramic view of Riemannian geometry, Springer Verlag, 2003.
- [39] M. P. Do Carmo, Differential geometry of curves and surfaces, 885 Vol. 2, Prentice-Hall Englewood Cliffs, 1976.
- [40] M. De Berg, M. Van Kreveld, M. Overmars, O. C. Schwarzkopf, Computational geometry, Springer, 2000.
- [41] L. Piegl, W. Tiller, The nurbs book. 1997.