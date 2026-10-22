<!-- image -->

## [Faculty Publications](https://scholarsarchive.byu.edu/facpub)

1995-07-22

## Surface Intersection Loop Destruction

Thomas W. Sederberg tom@cs.byu.edu

Alan K. Zundel

[Follow this and additional works at: https://scholarsarchive.byu.edu/facpub](https://scholarsarchive.byu.edu/facpub?utm_source=scholarsarchive.byu.edu%2Ffacpub%2F1150&utm_medium=PDF&utm_campaign=PDFCoverPages)

<!-- image -->

[Part of the Computer Sciences Commons](https://network.bepress.com/hgg/discipline/142?utm_source=scholarsarchive.byu.edu%2Ffacpub%2F1150&utm_medium=PDF&utm_campaign=PDFCoverPages)

## Original Publication Citation

A. K. Zundel and T. W. Sederberg, (aka surface intersection loop destruction)"Loop destruction using Bézier clipping," The Mathematics of Surfaces VII, R. R. Martin, ed., Oxford University Press, 1996, pp. 463-478.

## BYU ScholarsArchive Citation

Sederberg, Thomas W. and Zundel, Alan K., "Surface Intersection Loop Destruction" (1995). Faculty Publications. 1150.

[https://scholarsarchive.byu.edu/facpub/1150](https://scholarsarchive.byu.edu/facpub/1150?utm_source=scholarsarchive.byu.edu%2Ffacpub%2F1150&utm_medium=PDF&utm_campaign=PDFCoverPages)

This Peer-Reviewed Article is brought to you for free and open access by BYU ScholarsArchive. It has been accepted for inclusion in Faculty Publications by an authorized administrator of BYU ScholarsArchive. For more information, please contact ellen\_amatangelo@byu.edu.

## [Brigham Young University BYU ScholarsArchive](https://scholarsarchive.byu.edu/)

## Surface Intersection Loop Destruction

## Alan K. Zundel and Thomas W. Sederberg Brigham Young University

22 July 1996

## 1 Introduction

The intersection curve between two surface patches consists of one or more connected components or branches . Each component can be classified as either an open branch, with endpoints on at least one patch boundary, or as a closed loop (see Figure 1).

Figure 1: Open Branch and Closed Loop

<!-- image -->

It is known that the intersection curve formed by two rational surface patches is generally of very high degree (324 in the case of two bicubic patches) and genus (433 in the case of two bicubic patches) [KS88]. The large genus means it is impossible to exactly express the intersection curve in the form of a parametric equation. Hence, algorithms for 'computing' an intersection curve usually are designed to compute a piecewise approximation of the curve to within a tolerance.

Most such algorithms require that an initial point be identified on each branch of the intersection, and then a curve-following scheme is invoked to compute the approximation [Pra86, BHHL88, CO88, BK90, SN91]. Whereas starting points on open branches can be found by simply intersecting each surface with the boundary curves of the opposing surface, starting points on loops can be more elusive.

This paper introduces an algorithm for 'loop destruction' - intelligently subdividing a pair of intersecting B´ ezier surface patches so as to slice through the closed loops, thereby converting them into open branches. The algorithm is based on a technique called B´ ezier clipping -avariation of interval arithmetic that takes advantage of the convex-hull property of Bernstein polynomials. Loops are typically split in one or two tries if their diameters are at least, say, one-eighth the width of the patch. For infinitesimally small loops, the method converges quadratically.

The algorithms presented here can be applied to any surface type for which the tangent directions can be computed. Our development focuses on B´ ezier tensor product surface patches.

Various methods have been proposed for detecting closed loops. A simple but fallible method is to extract a set of isoparametric curves from one surface and intersect them with the opposing surface [Tim77]. This algorithm detects big loops multiple times, but may miss small loops. More robust loop detection techniques have been developed based on various strategies such as tesselation and subdivision techniques (see, for example [Arn87, LR80, Dok85, HAG83, HEFS85]); surface, tangent and normal cones [SM88, Hoh91, PP90]; parallel and collinear normals [SKW85, SKC89]; surface implicitization [PP87, PP90, SAG84]; and vector fields [Che88, Kri90].

The loop destruction algorithm requires a method for detecting the possible existence of a loop which in turn makes use of surface-bounding pyramids as reviewed in Section 2. Section 3 then discusses how these pyramids are used to determine the non-existence of closed loops. Section 4 reviews B´ ezier clipping and applies it to the loop destruction problem.

## 2 Bounding Pyramids

The loop destruction algorithm makes use of pyramids that bound patches and pyramids that bound tangent directions in a manner discussed below. Several variations on the idea of bounding pyramids have appeared in the literature [SM88, Kim90, Hoh91, Kri90, KP91]. This section reviews, without proof, the bounding pyramids presented in [SZ96].

## 2.1 Tangent Bounding Pyramids

Given a rational B´ ezier surface patch Q ( u, v ) with positive weights, a tangent bounding pyramid P u ( Q ) (or P v ( Q )) is any pyramid that contains all tangent directions Q u ( u, v ) (or Q v ( u, v )) 0 ≤ u, v ≤ 1. This means that any tangent vector whose tail is translated to the appropriate pyramid's vertex will lie within that pyramid.

Denote by P u + ( Q ) the nappe of P u ( Q ) that encloses all the positive tangent directions and denote the other nappe of P u ( Q ) by P u -( Q ) (see Figure 2.a).

<!-- formula-not-decoded -->

Nappes P v + ( Q ) and P v -( Q ) are defined similarly (see Figure 2.b).

<!-- image -->

U

U

Figure 2: B´ ezier Surface Patch with U- and V-Tangent Bounding pyramids

For a polynomial tensor product B´ ezier surface patch

<!-- formula-not-decoded -->

the partial derivative is itself a polynomial B´ ezier patch given by

<!-- formula-not-decoded -->

Because of the convex hull property of B´ ezier surfaces, the convex hull of the vectors ( Q i +1 ,j -Q i,j ) will contain all partial derivative directions Q u ( u, v ). In other words, the convex hull of the ( Q i +1 ,j -Q i,j ) vectors can serve as P u + ( Q ). The negative nappe P u -( Q ) is similarly the convex hull of the ( Q i,j -Q i +1 ,j ) vectors. P v ( Q ) can be constructed in like manner.

For the case of a rational B´ ezier surface patch, the computation of the hodograph is much more expensive, resulting in a degree 2 m × 2 n rational patch. A tangent bounding pyramid can be computed as having its vertex at the origin and enclosing all (2 m +1) × (2 n +1) hodograph control points. However, [SWS95] presents a much more efficient way of computing the pyramid for a rational patch that only involves vectors defined by control points of the given patch, not of the hodograph.

A useful property of tangent bounding pyramids is that if pyramid P u ( Q ) is translated so that its vertex lies at Q ( σ, τ ), then the isoparameter curve segment Q ( u, τ ), u ∈ [ σ, 1], lies within P u + ( Q ) and Q ( u, τ ), u ∈ [0 , σ ], lies within P u -( Q ) (see Figure 2.a).

In practice, we recommend the use of a pyramid with a rectangular directrix (see [SZ96]).

## 2.2 Surface Bounding Pyramids

A surface bounding pyramid has the property that when its vertex is translated to any point on the surface patch, the surface patch lies entirely outside of the pyramid. Such a pyramid can be derived from two tangent bounding pyramids P u ( Q ) and P v ( Q ) as follows.

Consider an arbitrary ray in the patch parameter space

<!-- formula-not-decoded -->

The image of one of these rays on the surface patch is a parametric curve

<!-- formula-not-decoded -->

whose first derivative (tangent) vector is:

<!-- formula-not-decoded -->

Taking the case α, β &gt; 0, we define a pyramid P ++ ( Q ) that satisfies

<!-- formula-not-decoded -->

P ++ ( Q ) can be taken as the convex hull of P u + ( Q ) and P v + ( Q ).

Figure 3: a. P ++ bounding positive quadrant b. P ( Q ) surface bounding pyramid

<!-- image -->

It is shown in [SZ96] that if P ++ ( Q ) is translated so that its vertex lies at any point on the surface patch Q ( u 0 , v 0 ), then the portion of Q ( u, v ) for which u ∈ [ u 0 , 1] and v ∈ [ v 0 , 1] will lie entirely within P ++ ( Q ).

Likewise we can define bounding volumes P + -( Q ), P --( Q ) and P -+ ( Q ), each bounding one quadrant of Q . Observe that P + -( Q ) and P -+ ( Q ) are opposite nappes of the same pyramid, and P ++ ( Q ) and P --( Q ) are opposite nappes of the same pyramid. The union of the volumes

<!-- formula-not-decoded -->

is the boolean negative of a pyramid, which we denote P ( Q ):

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

P ( Q ) bounds patch Q ( u, v ) in the sense that if the vertex of P ( Q ) is translated to any point on Q ( u, v ), the patch Q ( u, v ) will lie entirely outside of pyramid P ( Q ), as illustrated in Figure 3.b. Note that pyramid P ( Q ) always has a quadrilateral directrix.

It is shown in [SZ96] that a normal-bounding pyramid (i.e., a pyramid that bounds all surface-normal vectors) can be obtained as the complement of a surface-bounding pyramid

- the locus of all vectors that are not perpendicular to any vector inside the surfacebounding pyramid. Thus, this normal-bounding pyramid has a quadrilateral directrix, and each of its four faces is perpendicular to a face on the surface-bounding pyramid.

## 3 Loop Detection

Bounding pyramids and cones can be used in several ways to assure the non-existence of a closed loop. When we say that a pair of surfaces passes a loop test, we will mean that the test has positively determined that no loop exists. Test failure is inconclusive; the intersection may or may not involve a closed loop.

Loop Test 0 [SKC89]. If either P u ( Q 1 ) or P v ( Q 1 ) lies completely within P ( Q 2 ), no closed loop exists.

Figure 4.a shows a directional curve bounding pyramid and a surface bounding pyramid that pass the test, assuring the non-existence of any closed loops. Figure 4.b illustrates a failure of the test indicating a possible loop.

Figure 4: A. Passing the Loop. B. Failing the Loop Test.

<!-- image -->

Loop Test 0 involves a pyramid that bounds all isoparameter curves of constant u or v . This test can be generalized to consider families of curves whose preimages are parallel straight lines.

<!-- formula-not-decoded -->

defines such a family of curves for a fixed α . From the chain rule, the tangent vector for such a curve is

<!-- formula-not-decoded -->

If Q ( u, v ) is a tensor product polynomial surface of degree m × n , C ( t, u 0 , v 0 , α ) is a curve of degree n + m in t . However, for fixed α and for any u 0 , v 0 ∈ [0 , 1], C t ( t, u 0 , v 0 , α ) can be computed by evaluating a degree n × n hodograph

<!-- formula-not-decoded -->

where ˆ Q u ( u, v ) denotes Q u ( u, v ) after degree elevation in u and ˆ Q v ( u, v ) denotes Q v ( u, v ) after degree elevation in v . As in Section 2.1, we can determine a pyramid P α ( Q ) that bounds the hodograph Q α ( u, v ).

Note that Loop test 0 is actually four tests: If either P u ( Q 1 ) or P v ( Q 1 ) lies completely within P ( Q 2 ) - or if either P u ( Q 2 ) or P v ( Q 2 ) lies completely within P ( Q 1 ) - no closed loop exists. Figure 5 shows a pair of intersecting surfaces which do not intersect in a closed loop yet all four test 1 cases fail. This is not unusual. However, we can modify the test so that surfaces like this are more likely to pass the test.

Figure 5: Example Surface Pair

<!-- image -->

Loop Test 1 . If any pyramid P α ( Q 1 ) lies completely within P ( Q 2 ), no closed loop exists.

This follows directly from the proof of Loop Test 0 [SKC89]. In practice, this test is performed several times using various discrete values of α . If the patches pass the test for any value of α , no loop exists.

Loop Test 2 If no vector contained in a surface normal bound is perpendicular to any vector contained in a directional bound (for a fixed parameter direction) on the other surface, no closed loop exists.

This loop test was suggested to the authors by Tim Strotman and is based on the following reasoning. If a closed loop exists, its preimage C 1 on patch 1 is also a closed loop. Any line in the parameter space of patch 1 is parallel to a line T that is tangent to C i . At the point of tangency, the image of T is perpendicular to the normal vector of patch 2 at that point. In practice, Loop Test 2 is equivalent to Loop Test 1, except it uses the normal bound rather than the surface bound.

Test 2 is performed in the following manner.

1. For each of the four corners ( d i ) of the directional bounding pyramid

For each of the four corners ( n ) of the normal bounding pyramid

j If d i · n j &gt; max set max = d i · n j . If d i · n j &lt; min set min = d i · n j .

2. If 0 . 0 ̸∈ [ min,max ] no tangential line exists.

## 4 B´ ezier Clipping

B´ ezier clipping is the name given to a series of algorithms for performing various computations on B´ ezier curves and surfaces, such as parametric curve intersection [SWZ89, SN90], curve-surface intersection [SN91], and ray-patch intersection [NSK90]. Here it is used to guide the loop destruction process.

The basic idea behind B´ ezier clipping is to represent a Bernstein-form polynomial

<!-- formula-not-decoded -->

as an explicit B´ ezier curve

<!-- formula-not-decoded -->

The crucial feature is that the control points are evenly spaced in x along the [0 , 1] interval. The fundamental operation used in B´ ezier clipping is to determine ranges of t for which

Figure 6: B´ ezier Clipping

<!-- image -->

y ( t ) = 0. Referring to Figure 6, such ranges can be determined by intersecting the convex hull of the control points of the explicit B´ ezier curve with the x-axis. Hence, in this example, the ranges t ∈ [0 , x min ] and t ∈ [ x max , 1] do not contain a root of f ( t ).

̸

We now apply this idea to loop destruction. Denote by

<!-- formula-not-decoded -->

the hodograph d Q 1 ( u,v ) du . Q 1 is a degree m × n patch, and H u ( u, v ) is degree elevated so that it is also degree m × n .

Consider next a pyramid (with quadrilateral directrix) that bounds the normal vectors of Q 2 . Denote by n 00 , n 01 , n 10 , n 11 the unit vectors along each edge of the positive nappe of the pyramid and define

<!-- formula-not-decoded -->

Then, for any vector normal to Q 2 , there exist a, b ∈ [0 , 1] such that n ( a, b ) is parallel to that normal vector.

Denote

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

From Loop Test 2, we observe that if

<!-- formula-not-decoded -->

there is no closed loop.

We can now adopt the idea of B´ ezier clipping to identify values of v for which a closed loop cannot exist. Treat D u ( a, b, u, v ) as a degree m polynomial in u (in Bernstein form) whose coefficients are polynomials (also in Bernstein form) in the variables a, b, and v :

<!-- formula-not-decoded -->

Our goal is to identify ranges of v for which D u ( a, b, u, v ) is never zero for all possible values of a, b, u ∈ [0 , 1]. This is carried out by expressing D u ( a, b, u, v ) as a degree n polynomial in Bernstein form, with interval control points:

<!-- formula-not-decoded -->

Each of the [ D ] l are determined by taking the minimum and maximum of the 4( m + 1) values

<!-- formula-not-decoded -->

Referring to Figure 7, the gray region shows the projection of D u ( a, b, u, v ) onto the v, D u

Figure 7: B´ ezier clipping to determine loop-free regions

<!-- image -->

plane. For any fixed value of v = ˜ v ∈ [0 , 1], all values of D u ( a, b, u, v ) with a, b, u ∈ [0 , 1] lie in the intersection of the gray region with the line v = ˜ v . Then, since

<!-- formula-not-decoded -->

it is clear that no loop can exist for any value of v for which

<!-- formula-not-decoded -->

This is easily done by finding where the v axis lies outside of the convex hull of the control points of [ D ] u ( v ). In Figure 7, no loop occurs for v &lt; v min or for v &gt; v max . Clearly, if all control points of [ D ] u ( v ) have the same sign, no loops exist anywhere on the patch.

̸

In similar manner, we can determine ranges of u within which no closed loop can occur. This is done by forming an interval Bernstein polynomial [ D ] u ( u ) whose control points [ D ] k are determined by taking the minimum and maximum of the 4( n +1) values

<!-- formula-not-decoded -->

Loop destruction is then attempted by subdividing the patches. If ( u max -u min ) &lt; ( v max -u min ), then the patch is split in the u direction at ( u min + u max ) / 2. Otherwise, the split is made in the v direction at ( v min + v max ) / 2. After splitting, the loop test is again invoked. If the test passes, we're done. Otherwise, further B´ ezier-clip-guided subdivisions (and subsequent loop tests) are performed.

## 4.1 Directional Curve Bounding

We have just used the u -hodograph of Q 1 to identify loop-free regions - regions of u and v on Q 1 that cannot contain any closed loops. Likewise, we can find loop-free regions using the v -hodograph of Q 1 . Clearly, the union of the loop-free regions obtained used the u - and v -hodographs is also a (possibly larger) loop-free region.

Further, from Loop Test 2, we can potentially enlarge the loop-free regions further by considering hodographs in other parameter directions. Equation 10 defines a curve C ( t, u 0 , v 0 , α ) that is the image of a straight line in parameter space, with arbitrary direction α . We now consider how to determine loop-free regions using -1 ≤ α ≤ 1 - that is, regions where no curve C ( t, u 0 , v 0 , α ) for fixed α is tangent to the SSI curve.

Let

where

This leads to the function

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

For any given α , loop free regions may be determined by performing a B´ ezier clip on the functions [ D ] α ( u ) and [ D ] α ( v ).

Since B´ ezier clip calculations are relatively cheap, it generally pays to check a few values of α in addition to 0 and 1.

## 4.2 Spatial B´ ezier Clipping

It often happens that the bounding boxes of two patches often overlap only slightly. Figure 8 shows two surfaces for which the loop detection tests fail. Since the tests are translation invariant, if the patches can be translated so as to create a closed loop, the test will fail. In this example, the intersection is transverse, but the test fails since the patches can be translated so as to create a closed loop.

Figure 8: Surface Pair with Small Overlap

<!-- image -->

An effective solution to this problem, suggested by Tomoyuki Nishita, is to perform a B´ ezier clip of each patch against the bounding box of the other patch. This operation clips away portions of one patch which do not lie within the bounding box of the opposing patch, resulting in two smaller patches that tend to more nearly occupy roughly the same region in space. Thus, regions of each patch which definitely play no roll in the intersection have no influence on the loop detection test. The results of such a clip on the surface patch pair shown in Figure 8 are shown in Figure 9. In this refined problem, the loop test passes.

Figure 9: Surface Pair after B´ ezier Clipping to Bounding boxes

<!-- image -->

## 5 Discussion

The power of B´ ezier clipping for loop destruction is dramatically illustrated in the case of a simple convex surface just tangent to a plane (Figure 10), where there exists an infinitesimally small closed loop. In this contrived case, all rows of the control grid are translations of each other, so all the rows of the u -hodograph are identical. Therefore, [ D ] u ( u ) becomes an interval function of width zero. Further, since the surface is convex, the function is monotonic with one root. This root locates the u value of the tangency point between the surface and the plane exactly. The v location can be computed in a similar manner using the v hodograph and [ D ] v ( v ).

Figure 10: Simple Convex Surface Tangent to Plane

<!-- image -->

The example in Figure 11 involves two fairly complicated surfaces which intersect in four branches, three of which are closed loops. Loop detection obviously fails, requiring

Figure 11: Surface Intersection Problem With Multiple Loops

<!-- image -->

subdivision. The problem is so complicated that B´ ezier clipping fails to identify any loopfree region, so a bisection is performed. One of the loops is split into open branches, and the remaining two loops are separated into subproblems. One of these subproblems is illustrated in Figure 12. Both of the subproblems also fail the loop detection tests. On the third iteration, B´ ezier clipping identifies loop-free regions, guiding the subdivision to sucessfully split the two remaining loops.

## References

- [Arn87] P. R. Arner. Another Look at Surface/Surface Intersection . Deptartment of computer science, University of Utah, 1987.
- [BHHL88] C. L. Bajaj, C. M. Hoffmann, J. E. Hopcroft, and R. E. Lynch. Tracing surface intersections. Computer Aided Geometric Design , 5:285-307, 1988.
- [BK90] R. E. Barnhill and S. N. Kersey. A marching method for parametric surface/surface intersections. Computer Aided Geometric Design , 7:257-280, 1990.

Figure 12: Multiple Loop Subproblem

<!-- image -->

- [Che88] K. P. Cheng. Using plane vector fields to obtain all the intersections of two general surfaces. In W. Strasser, editor, Theory and Practice of Geometric Modeling , pages 187-204. Springer-Verlag, 1988.
- [CO88] J. J. Chen and T. M. Ozsoy. Predictor-corrector type of intersection algorithm for c 2 parametric surfaces. Computer Aided Design , 20(6):347-352, 1988.
- [Dok85] T. Dokken. Finding intersections of b-spline represented geometries using recursive subdivision techniques. Computer Aided Geometric Design , 2:189-195, 1985.
- [HAG83] S. L. Hanna, J. F. Abel, and D. P. Greenberg. Intersection of parametric surfaces by means of lookup tables. IEEE Computer Graphics and Applications , 3(7):3948, 1983.
- [HEFS85] E. G. Houghton, F. F. Emnett, J. D. Factor, and C. L. Sabharwal. Implementation of a divide-and-conquer method for intersection of parametric surfaces. Computer Aided Geometric Design , 2:173-183, 1985.
- [Hoh91] Michael Hohmeyer. A surface intersection algorithm based on loop detection. International Journal of Computational Geometry and Applications , 1(4):473490, 1991.
- [Kim90] Deok-Soo Kim. Cones on B´ ezier curves and surfaces . PhD thesis, University of Michigan, 1990.
- [KP91] G. A. Kriezis and N. M. Patrikalakis. Rational polynomial surface intersections. In G. A. Gabriele, editor, Proceedings of the 17th ASME Design Automation Conference , volume II, pages 43-53, 1991.
- [Kri90] George A. Kriezis. Algorithms for Rational Spline Surface Intersections . PhD thesis, Massachusetts Institute of Technology, 1990.
- [KS88] Sheldon Katz and Thomas W. Sederberg. Genus of the intersection curve of two rational surface patches. Computer Aided Geometric Design , 5(3):253-258, September 1988.
- [LR80] J. M. Lane and R. F. Riesenfeld. A theoretical development for the computer generation and display of piecewise polynomial surfaces. IEEE Transactions on Pattern Analysis and Machine Intelligence , 2(1), 1980.

- [NSK90] T. Nishita, T. W. Sederberg, and M. Kakimoto. Ray tracing trimmed rational surface patches. Computer Graphics , 24:337-345, 1990.
- [PP87] N. M. Patrikalakis and P. V. Prakash. Computation of algebraic polynomial parametric surface intersections. Technical report, MITSG 87-19, MIT Sea Grant College Program, 1987.
- [PP90] N. M. Patrikalakis and P. V. Prakash. Surface intersections for geometric modeling. Journal of Mechanical Design, ASME Transactions , 1990.
- [Pra86] M. J. Pratt. Surface/surface intersection problems. In J. A. Gregory, editor, The Mathematics of Surfaces II . Oxfor, Claredon Press, 1986.
- [SAG84] Thomas W. Sederberg, D. C. Anderson, and Ron N. Goldman. Implicit representation of parametric curves and surfaces. Computer Vision, Graphics and Image Processing , 28:72-84, 1984.
- [SKC89] Thomas W. Sederberg, Sheldon Katz, and Henry N. Christiansen. An improved test for closed loops in surface intersections. Computer Aided Design , 21(8):505508, October 1989.
- [SKW85] P. Sinha, E. Klassen, and K. K. Wang. Exploiting topological and geometric properties for selective subdivision. In Proc. ACM Symposium on Computational Geometry , All ACM Conferences, pages 39-45, 1985.
- [SM88] Thomas W. Sederberg and Ray J. Meyers. Loop detection in surface patch intersections. Computer Aided Geometric Design , 5:161-171, 1988.
- [SN90] Thomas W. Sederberg and Tomoyuki Nishita. Curve intersection using bezier clipping. Computer-Aided Design , pages 538-549, 1990.
- [SN91] Thomas W. Sederberg and Tomoyuki Nishita. Geometric hermite approximation of surface patch intersections curves. Computer Aided Geometric Design , 8:97114, 1991.
- [SWS95] Takafumi Saito, Guo-Jin Wang, and Thomas W. Sederberg. Hodographs and normals of rational curves and surfaces. Computer Aided Geometric Design , 12:417-430, 1995.
- [SWZ89] Thomas W. Sederberg, Scott C. White, and Alan K. Zundel. Fat arcs: A bounding region with cubic convergence. Computer Aided Design , 6:205-218, 1989.
- [SZ96] Thomas W. Sederberg and Alan K. Zundel. Pyramids that bound surface patches. CVGIP: Graphical Models and Image Processing , 58:75-81, 1996.
- [Tim77] H. G. Timmer. A solution to the surface intersection problem. Technical report, McDonnel Douglas Report No. MDC J7789, 1977.