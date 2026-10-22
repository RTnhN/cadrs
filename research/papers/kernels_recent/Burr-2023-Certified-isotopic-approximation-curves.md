## Certified simultaneous isotopic approximation of curves via subdivision ⋆

Michael Burr a , Michael Byrd a

a Clemson University, 220 Parkway Drive, 29634, Clemson, SC, USA

## Abstract

We present a certified algorithm based on subdivision for computing an isotopic approximation to any number of curves in the plane. Our algorithm is based on the certified curve approximation algorithm of Plantinga and Vegter. The main challenge in this algorithm is to correctly and efficiently identify and isolate all intersections between the curves. To overcome this challenge, we introduce a new and simple test that guarantees the global correctness of our output. A main step in our algorithm for approximating any number of curves is to correctly approximate a pair of curves. In addition to developing the details of this special case, we provide complexity analyses for both the number of steps and the bit-complexity of this algorithm using both worst-case bounds as well as those based on continuous amortization.

Keywords: Curve approximation, Plantinga and Vegter approximation algorithm, topological correctness, certified algorithms, interval arithmetic, symbolic-numeric algorithms, bit-complexity, worst-case bounds, continuous amortization

2020 MSC: 14Q65, 68W30, 68Q25

## 1. Introduction

In Plantinga and Vegter (2004, 2007), the authors introduced a pair of algorithms to construct piecewiselinear approximations to smooth and bounded real curves and surfaces in two and three dimensions, respectively. They prove that the outputs of their algorithms are both topologically correct and can be refined to be as close to the underlying curve as desired. Their algorithms are particularly interesting as they are symbolic-numeric algorithms based on subdivision whose predicates are simple, efficient, and easy to implement. On singular input, however, the Plantinga and Vegter algorithms do not terminate as both of their predicates fail on regions containing singular points. The current paper presents an algorithm in the spirit of the original Plantinga and Vegter algorithm for correctly approximating a union of smooth curves in the plane with transverse crossings.

We begin by posing the main problem addressed in this paper. Suppose that f 1 , . . . , f n ∈ Z [ x, y ] define n smooth curves V ( f 1 ) , . . . , V ( f n ) in the real plane. In addition, we assume that, pairwise, these curves intersect transversely and that no three of these curves intersect simultaneously. We construct n piecewiselinear approximations A ( f 1 ) , . . . , A ( f n ) such that each A ( f i ) approximates V ( f i ) and ( A ( f 1 ) , . . . , A ( f n )) is topologically equivalent to ( V ( f 1 ) , . . . , V ( f n )) . In our setting, topologically correct means that there is an ambient isotopy that deforms space while simultaneously taking each V ( f i ) to A ( f i ). In particular, the intersection points of the curves are carried to the crossings of the approximations. As in the original algorithm of Plantinga and Vegter, the approximations and curves can be made to be as close as desired in Hausdorff distance.

The main challenge in extending the Plantinga and Vegter algorithm is that while the original algorithm can compute topologically correct approximations to any single curve, there is no guarantee that the corresponding ambient isotopies are compatible when applied to multiple curves, see Figure 1. For example, it is possible for A ( f i ) ∩ A ( f j ) to have extraneous or missing intersections when compared to V ( f i ) ∩ V ( f j ). Even if all of the intersections between individual pairs of curves are computed correctly, the order in which the intersections A ( f j ) ∩A ( f i ) and A ( f l ) ∩A ( f i ) occur along A ( f i ) might differ from the order in which the intersections V ( f j ) ∩ V ( f i ) and V ( f l ) ∩ V ( f i ) occur along V ( f i ).

⋆ This work was supported by NSF grant DMS-1913119 and Simons Foundation collaboration grant #964285 awarded to Burr.

Email addresses: burr2@clemson.edu (Michael Burr), mbyrd6@clemson.edu (Michael Byrd)

Figure 1: Two approximations (thick lines) of a pair of curves (thin lines). The approximations and curves are paired by color and line style. A naive approach (a) misses a pair of intersections due to excursions while our approach (b) correctly approximates the curves and their intersections.

<!-- image -->

## 1.1. Previous work

An extension of the Plantinga and Vegter algorithm was introduced in Burr et al. (2012) to handle unbounded and singular input. This approach can solve the problem currently under consideration by computing an approximation to the variety V ( fg ), but this approach relies on separation bounds between singular points. These separation bounds are typically so pessimistic that it is questionable whether this algorithm is practical. On the other hand, the algorithm introduced in Lien et al. (2014) also studies the problem considered here, but their algorithm uses more restrictive tests than what we propose, and they may require a significant number of subdivisions to characterize the local behavior of curves within a region. For instance, their algorithm has more topological requirements on boxes that contain intersections of curves than what is required in our approach. The algorithm presented in Cheng et al. (2023) solves the related problem of isolating the points of V ( f 1 ) ∩V ( f 2 ) without approximating the curves themselves. Their approach is based on studying the local behavior of the curves of V ( f 1 ) and V ( f 2 ) and also uses more restrictive tests than what we propose. Our correctness statements are slightly weaker than the corresponding correctness statements in Lien et al. (2014) and Cheng et al. (2023), as we do not identify which boxes contain crossings of V ( f 1 ) and V ( f 2 ). Our correctness statements, however, are still quite strong and more in-line with the statements appearing in the original work of Plantinga and Vegter.

A preliminary version of the work in this paper appeared in Burr and Byrd (2023). In that paper, we provided the details and correctness of an algorithm to correctly approximate a pair of smooth curves in the plane, see Section 3. In the current paper, we add several complexity analyses of that algorithm, including both counting the number of regions produced by the algorithm as well as the bit-complexity of the algorithm. Our complexity analyses use both nonamortized bounds as well as amortized bounds based on continuous amortization. In addition, we extend our algorithm to correctly approximate any number of curves in the plane.

## 1.2. Outline

In Section 2, we recall the details of the Plantinga and Vegter algorithm for curve approximation. In Section 3, we provide the details of the subdivision step for our algorithm to approximate a pair of curves in the plane. This special case of approximating a pair of curves is a key step of our main algorithm. In Section

4, we analyze the complexity of Algorithm 3 for approximating a pair of curves in the plane. In Section 5, we apply the algorithm from Section 3 to develop our main algorithm, which correctly approximates any number of curves. Finally, we present a gallery of examples in Section 6.

## 2. Background

The Plantinga and Vegter algorithm for curve approximation is an adaptive subdivision-based algorithm based on the marching cube algorithm (Lorensen and Cline, 1987). The input to the algorithm is a bivariate polynomial f ∈ Z [ x, y ] and an input region R = [ a, b ] × [ c, d ] in R 2 such that a, b, c, d ∈ Z . The output is a piecewise linear approximation A ( f ), which is guaranteed to be ambient isotopic to the real variety V ( f ). The algorithm consists of three main steps: (1) the subdivision step, (2) the balancing step, and (3) the approximation step.

In the subdivision step, square regions are considered by the algorithm and either accepted and further processed or rejected and split into four equal-sized subboxes for further consideration. The acceptance and rejection is based on two predicates called C 0 and C 1 . These predicates take, as input, a square region B ⊆ R and produce either True or False . A square is accepted when either of the predicates returns True and is rejected when both predicates return False . The predicates have the following properties: When C 0 ( B ) = True , we conclude that V ( f ) does not intersect B . On the other hand, when C 1 ( B ) = True , there do not exist any pair of points (( x 1 , y 1 ) , ( x 2 , y 2 )) ∈ B × B such that ∇ f ( x 1 , y 1 ) and ∇ f ( x 2 , y 2 ) are perpendicular.

The resulting subdivision consists of a union of boxes, see Figure 1, and we use the word side to denote one of the four sides of a box (as a quadrilateral) in the subdivision. We define an edge of the subdivision to be a side of some box that is not composed of a union of sides of smaller neighboring boxes.

In the balancing step, additional subdivisions are performed until the side length of neighboring boxes differ in length by at most a factor of two. Therefore, if B is a box of the final subdivision, then the edges of B are either sides of B or half-sides of B . The edges of B are half-sides when B 's neighbor in that direction is smaller than B . Finally, in the approximation step f is evaluated on each vertex of every box B which satisfies C 1 ( B ) but not C 0 ( B ). For each edge in the subdivision where f changes signs, the algorithm adds a vertex on this edge. Finally, in each box, the vertices are connected in such a way so that the segments do not intersect, and, if there are four vertices on the sides of a box, the two vertices on the same side of the box are not connected.

## 2.1. Correctness and complexity

In the original presentation (Plantinga and Vegter, 2004, 2007), the correctness of the curve approximation algorithm was only proved for smooth and bounded curves. In Burr et al. (2012), by slightly weakening the correctness statement, the algorithm was extended to unbounded curves. In addition, in Burr et al. (2012), the algorithm and correctness statement were extended to nonsingular curves, but the practicality of this approach remains in question. In Lin and Yap (2011), the authors extend the algorithm to non-square regions within the subdivision.

In each of these algorithms, an important feature of their correctness statements is that they guarantee global correctness, not local correctness. For instance, the correctness statement does not guarantee that the approximation and the variety are isotopic when restricted to a box of the subdivision, only that they are isotopic in the input region R (or a set containing the input region). The main difference between the variety and its approximation are excursions . An excursion occurs when the variety briefly enters a neighboring box, but this behavior does not appear in the approximation, see Figure 1 and Definition 1. Instead, the ambient isotopy stretches space so that the approximation is moved into the neighboring box.

This global correctness without local correctness is a key feature of this family of algorithms. It often leads to many fewer boxes since these algorithms do not need to resolve the behavior of small excursions. This key feature makes the problem of approximating a pair of curves given by f 1 , f 2 ∈ Z [ x, y ] more challenging, however, since an excursion may involve an intersection between the varieties V ( f 1 ) and V ( f 2 ), but the ambient isotopies separate the curves and remove the intersection from the approximations A ( f 1 ) and A ( f 2 ), see Figure 1(a).

The complexity of the Plantinga and Vegter algorithm was first studied in Burr et al. (2020) using continuous amortization, see, for example, Burr and Krahmer (2012) and Burr (2016). The authors found both adaptive and worst-case complexity bounds for the number of regions formed by subdivision as well as the bit-complexity of the algorithm. In addition, they found examples which exhibited the worst-case exponential complexity. In Cucker et al. (2022), a smoothed-analysis-based approach was used to show that the average complexity of the algorithm is polynomial. In Tonelli-Cueto and Tsigaridas (2020), a conditionnumber-based approach also showed that the average complexity of the algorithm is polynomial for other classes of random polynomials including some sparse families.

## 2.2. Predicate details

The two predicates in the Plantinga and Vegter algorithm are typically implemented using interval arithmetic, see, for example, (Moore et al., 2009) for more details. Interval arithmetic extends the standard arithmetic operations to intervals. For instance,

<!-- formula-not-decoded -->

These interval operations can be extended to the evaluation of functions, and we use the symbol □ to denote any such extension. In particular, for a polynomial f ∈ Z [ x, y ] and a region B , □ f ( B ) is an interval containing the image f ( B ) := { f ( x, y ) : ( x, y ) ∈ B } . The interval □ f ( B ) is often larger than f ( B ), but may be significantly easier to compute.

The C 0 test is implemented as C 0 ( B ) = True if and only if 0 ̸∈ □ f ( B ). Since □ f ( B ) is an overapproximation to f ( B ), the condition 0 ̸∈ □ f ( B ) implies that 0 ̸∈ f ( B ), so the variety V ( f ) cannot intersect B . The C 1 test is slightly more complicated, as C 1 ( B ) = True if and only if 0 ̸∈ □ ⟨∇ f, ∇ f ⟩ ( B × B ), where ⟨· , ·⟩ denotes the standard inner product. In this formulation, each of the factors of B × B is the argument to one ∇ f . If 0 ̸∈ □ ⟨∇ f, ∇ f ⟩ ( B × B ), then there cannot be a pair of points (( x 1 , y 1 ) , ( x 2 , y 2 )) ∈ B × B such that ⟨∇ f ( x 1 , y 1 ) , ∇ f ( x 2 , y 2 ) ⟩ = 0, that is, the gradient vectors cannot be perpendicular.

We call these algorithms symbolic-numeric algorithms for two reasons: The predicates perform exact computations using the coefficients of f , that is, not merely treating f as a function. The computations themselves are performed using arbitrary-precision floating point computations on dyadic points. In other words, the evaluations are exact, but leverage the speed of floating point calculations.

## 2.3. Topological details

We collect some key facts from Plantinga and Vegter (2004, 2007) and provide a description of the ambient isotopy in the Plantinga and Vegter algorithm. These facts are used throughout our correctness proofs.

Definition 1. Let B be a box of a subdivision and f ∈ Z [ x, y ] . An excursion of V ( f ) is a component of V ( f ) ∩ B whose two endpoints are on the same edge of the subdivision, see Figure 1(a).

We note that excursions do not appear in the piecewise-linear approximation A ( f ) as they are deformed into neighboring boxes.

In Plantinga and Vegter (2004, 2007), the authors use several topological lemmas to show that the predicates C 0 and C 1 exert control over the behavior of V ( f ) within a box, see Figure 2(a).

Lemma 2 (Plantinga and Vegter (2004, 2007)) . Suppose that B is a box of a subdivision, and suppose that there are two segments s 1 and s 2 in B such that

1. the lines formed from extending s 1 and s 2 are perpendicular,
2. the value of f on both endpoints of s 1 is the same, and
3. the value of f on both endpoints of s 2 is the same.

Figure 2: Illustrations of (a) Lemma 2 and (b) Lemma 9. In (a), the existence of two perpendicular segments meeting the curve twice implies that C 1 fails in this box. In (b), the existence of two parallel segments each meeting a curve twice implies that C × 1 fails in this box.

<!-- image -->

Then, C 1 ( B ) = False .

This lemma follows from applying the mean value theorem on segments s 1 and s 2 to show that each segment contains a point where the gradient at that point is perpendicular to the segment (see also Lemma 9). Lemma 2 leads to several corollaries, three of which we list here:

Corollary 3 (Plantinga and Vegter (2004, 2007)) . Suppose that B is a box of a subdivision such that C 1 ( B ) = True . In addition, let γ f be a component of V ( f ) ∩ B which is an excursion on edge e of B . Then, γ f is contained entirely within the semicircle in B whose diameter is e .

Corollary 4 (Plantinga and Vegter (2004, 2007)) . Suppose that B is a box of a subdivision. If V ( f ) intersects two adjacent sides of B and, on each of these sides, intersects twice, then C 1 ( B ) = False .

Corollary 5 (Plantinga and Vegter (2004, 2007)) . Suppose that B is a box of a subdivision such that C 1 ( B ) = True . There is at most one component of V ( f ) ∩ B that extends between opposite sides of B .

Finally, we briefly describe how the ambient isotopy deforms the variety V ( f ) to A ( f ) as a two-step procedure. The first step of the ambient isotopy is to remove all excursions by deforming space so that the excursions are moved into neighboring boxes. Briefly, we let ˜ V ( f ) be the result of applying the first step of the ambient isotopy to V ( f ). For every box B of the subdivision, ˜ V ( f ) ∩ B and A ( f ) ∩ B are ambient isotopic within B . In particular, this means that they have the same number of components within B and the components intersect the same edges of B . The second step of the ambient isotopy simultaneously deforms ˜ V ( f ) ∩ B to A ( f ) ∩ B within each box B by sliding the points on the boundary of B to their appropriate places and straightening the curves within each box B . By careful consideration of these steps, we observe that the ambient isotopies derived from the Plantinga and Vegter algorithm move points at most one box away as the only points that move between boxes are those near excursions, but, by Corollary 3, these points are never further than one box away.

Definition 6. Let S be a union of boxes from a subdivision and γ f : [0 , 1] → V ( f ) ∩ S a curve in the variety of f . The extension of γ f without excursions is denoted by γ f , and it is the component of V ( f ) ∩ S containing γ f . The extension of γ f with excursions is the curve ˜ γ f , which is formed by following γ f forward and backwards until either (1) the curve becomes a closed loop or (2) the curve reaches the first and last intersections of the curve with the boundary of S before passing through a box not in S . See Figure 3 for details.

The extension ˜ γ f can be constructed by iteratively adding excursions and curve components at the ends of the path until the curve leaves S and passes through other boxes of the subdivision. The ambient isotopy described above directly leads to the following correctness statements:

Figure 3: An illustration of the two types of extensions of a curve γ , denoted by the thickened curve, with a region S . The first extension is γ , which is denoted by the thin curve. γ extends γ to the first time that the curve containing γ leaves the region of interest. The second extension is ˜ γ , which is denoted by the dashes. ˜ γ includes all excursions until the curve containing γ leaves the region of interest and does not immediately return. The part of the curve denoted by dots is beyond the extensions γ and ˜ γ .

<!-- image -->

Lemma 7 (Plantinga and Vegter (2004, 2007)) . Let B be a box from the output of the Plantinga and Vegter algorithm and suppose that V ( f ) intersects an edge of B twice. Then there is a path in V ( f ) connecting these two points which is a sequence of excursions.

Lemma 8 (Plantinga and Vegter (2004, 2007)) . Let S be a union of boxes from the output of the Plantinga and Vegter algorithm and γ f : [0 , 1] → V ( f ) ∩ S a curve in the variety of f . Suppose that γ f deforms to α f ⊆ A ( f ) within S . Let ˜ γ f be the extension of this path with excursions. Let α f be the component of A ( f ) ∩ S containing α f . Either ˜ γ f and α f are topological circles within S or the endpoints of ˜ γ f and α f are on the same edges of the subdivision.

## 3. Approximating a pair of curves

In our algorithm in Section 5 for approximating any number of curves, a main step in the procedure is to compute an approximation of a pair of curves. In this section, we provide an algorithm for this special case. Of the three main steps of the Plantinga and Vegter approximation algorithm, the most significant changes occur in the subdivision step, and we focus our attention on this step. Much of the material in this section was initially presented in Burr and Byrd (2023).

Given f 1 , f 2 ∈ Z [ x, y ], a first attempt to approximate ( V ( f 1 ) , V ( f 2 )) may be to simultaneously run the standard Plantinga and Vegter algorithm on f 1 and f 2 using a common refinement of the region R . This approach leads to three different types of potential errors in the approximations:

1. Missing intersections: intersections of V ( f 1 ) and V ( f 2 ) which do not correspond to intersections of A ( f 1 ) and A ( f 2 ), see Figure 1.
3. Shared edges: some of the edges of the approximations A ( f 1 ) and A ( f 2 ) may be shared between the two approximations, see Figure 4(b).
2. Extra intersections: intersections of A ( f 1 ) and A ( f 2 ) which do not correspond to intersections of V ( f 1 ) and V ( f 2 ), see Figure 4(a).

Our main tool to avoid all three of these errors is a new predicate, which we call C × 1 . On a square B , if C × 1 ( B ) = True , then there do not exist any pair of points (( x 1 , y 1 ) , ( x 2 , y 2 )) ∈ B × B such that ∇ f 1 ( x 1 , y 1 ) and ∇ f 2 ( x 2 , y 2 ) are parallel. In the plane, we may implement this test using the cross product, that is, C × 1 ( B ) = True if and only if 0 ̸∈ □ ( ∇ f 1 ×∇ f 2 )( B × B ), where each of the factors of B × B is an argument to one of the gradients. As an initial illustration of the utility of this new C × 1 predicate, we provide the following motivating result, see Figure 2(b).

Lemma 9. Let B be a rectangle where s 1 and s 2 are parallel line segments in B . Suppose that f 1 attains the same value on both endpoints of s 1 and f 2 attains the same value on both endpoints of s 2 . Then C × 1 ( B ) = False .

(a)

Figure 4: Two approximations (thick lines) of a pair of curves (thin lines). The approximations and curves are paired by color and line style. The naive approach to approximating the pairs of curves include (a) an extra intersection or (b) several shared edges of the approximation.

<!-- image -->

Proof. Suppose that ( x 1 , y 1 ) , ( x 2 , y 2 ) ∈ s 1 such that f 1 ( x 1 , y 1 ) = f 1 ( x 2 , y 2 ). By applying Rolle's theorem to f 1 ( tx 1 +(1 -t ) x 2 , ty 1 +(1 -t ) y 2 ), there is some t 1 ∈ (0 , 1) such that

This equation, however, can be rewritten as the following dot product:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

In other words, there is some point in B where ∇ f 1 is perpendicular to s 1 . Repeating this argument for f 2 gives that there is some point in B where ∇ f 2 is perpendicular to s 2 . Since s 1 and s 2 are parallel, the gradients of f 1 and f 2 are parallel for some pair of points in the box B , and C × 1 ( B ) = False .

This lemma leads to the following special case when s 1 = s 2 .

Corollary 10. Let B be a rectangle and assume that V ( f 1 ) and V ( f 2 ) intersect more than once in B . Then C × 1 ( B ) = False .

Remark 11. The use of the C × 1 test explains some of the conditions that we require on our curves. In particular, we note that if an intersection of V ( f 1 ) and V ( f 2 ) was not a transverse crossing, then the gradients would agree at the intersection and C × 1 would be False .

We now present the algorithm for the subdivision step of the pairwise curve approximation algorithm. For simplicity, we focus on the case where the input region R is a square and leave the details for the general rectangular case to Section 3.4. Since there are two polynomials f 1 , f 2 ∈ Z [ x, y ], we write C f 1 0 and C f 1 1 for the standard tests from the Plantinga and Vegter algorithm for the function f 1 . Similarly, we define C f 2 0 and C f 2 1 for f 2 . In addition, we need the notion of a neighborhood of a box:

Definition 12. Let R = [ a, b ] × [ c, d ] be a square region and S a partition of R into squares. For any square B ∈ S , the neighborhood of B in S is denoted by N ( B ) and consists of B along with all of the other squares in S that have a positive-length intersection with B , that is, squares that only meet B at its corners are not in N ( B ) . More generally, we define N 1 ( B ) := N ( B ) and N i ( B ) to be the union of all the neighborhoods of boxes in N i -1 ( B ) .

In other words, N i ( B ) consists of all the boxes which are at most i boxes away from B . We write C × 1 ( N i ( B )) = True to denote that C × 1 holds in the smallest rectangle containing N i ( B ).

## Algorithm 1 Subdivision step

Input : polynomials f 1 , f 2 ∈ Z [ x, y ], with V ( f 1 ) and V ( f 2 ) smooth curves such that V ( f 1 ) and V ( f 2 ) intersect transversely, and a square region R with integral corners such that V ( f 1 ) and V ( f 2 ) do not intersect on the boundary of R .

Output : a partition of R for further processing

- 1: initialize queue Q to contain R 2: while Q is not empty do 3: pop square B from Q 4: accept B if any of the following holds: · C f 1 0 ( B ) = C f 2 0 ( B ) = True · C f 1 0 ( B ) = C f 2 1 ( B ) = True · C f 1 1 ( B ) = C f 2 0 ( B ) = True · C f 1 1 ( B ) = C f 2 1 ( B ) = C × 1 ( N 2 ( B ) ∩ 13 B ) = True 5: reject B if none of the previous hold: 6: subdivide B into four equal-sized boxes B 1 , . . . , B 4 7: push B 1 , . . . , B 4 into Q 8: end while 9: return accepted boxes

We incorporate our new test into Algorithm 1, which, in turn, replaces the subdivision step of the standard Plantinga and Vegter algorithm. In particular, the new subdivision step is similar to naively running the Plantinga and Vegter algorithm simultaneously on f 1 and f 2 , except that when C f 1 1 ( B ) and C f 2 1 ( B ) both hold, we add the condition that C × 1 ( N 2 ( B ) ∩ 13 B ) holds, where 13 B denotes the box which is 13-times larger than B , but with the same center.

Remark 13. After the balancing step of the Plantinga and Vegter algorithm, the neighbors of B are at most twice the size of B and their neighbors are at most four times the size of B . This means that after balancing, N 2 ( B ) ⊆ 13 B . To allow for the case where boxes are not balanced during the subdivision step, we use N 2 ( B ) ∩ 13 B in Step 4 of Algorithm 1 as this region contains N 2 ( B ) of the balanced subdivision.

Suppose that after this new subdivision step, the balancing and approximation steps of the standard Plantinga and Vegter algorithm are performed, resulting in approximations A ( f 1 ) and A ( f 2 ). In the remainder of this section, we present the theoretical properties of the crossings of these approximations as well as the correctness statement that these approximations have the same topology as the underlying pair of curves.

## 3.1. Transversal crossing of approximations

For the approximations A ( f 1 ) and A ( f 2 ), we call a crossing transversal if the approximations cross within the interior of a box. For a box whose neighbors are not further subdivided, this only happens when one approximation has an edge from the north side of a box to the south side, while the other approximation extends from the east side of the box to the west side. We show that every transversal crossing of A ( f 1 ) and A ( f 2 ) corresponds to a unique crossing of the varieties V ( f 1 ) and V ( f 2 ).

Proposition 14. Let f 1 , f 2 ∈ Z [ x, y ] , and suppose that B is a box such that C f 1 1 ( B ) = C f 2 1 ( B ) = C × 1 ( N ( B )) = True . If A ( f 1 ) and A ( f 2 ) intersect transversely in B , then V ( f 1 ) and V ( f 2 ) intersect exactly once and transversely in N ( B ) .

Proof. By Corollary 10, the number of intersections of V ( f 1 ) and V ( f 2 ) is at most one in N ( B ). Moreover, any such intersection must be transversal by assumption.

We let α f 1 and α f 2 be the two components of A ( f 1 ) ∩ B and A ( f 2 ) ∩ B which intersect in B . Then, we let γ f 1 be any component of V ( f 1 ) ∩ B which deforms to a subset of α f 1 under the isotopy from the Plantinga and Vegter algorithm. In addition, we let ˜ γ f 1 be the extension of γ f 1 with excursions in B . We define γ f 2 and γ f 2 similarly.

Figure 5: The neighborhood of the box B with two crossing components. The curves intersect in the neighborhood of B while the approximations (not shown) intersect in B .

<!-- image -->

˜ We now restrict our attention to f 1 since the case for f 2 is similar. By Lemma 8, the endpoints of α f 1 and ˜ γ f 1 lie on the same edges of the subdivision, which we denote by e 1 and e 2 . Since the Plantinga and Vegter algorithm places a vertex on each of these edges, the number of intersections, counted with multiplicity, between V ( f 1 ) and each e i is odd. Moreover, by Lemma 7, all intersections between V ( f 1 ) and each e i are included in ˜ γ f 1 . Hence, γ f 1 ,D \ ˜ γ f 1 consists of two curves contained in the semicircles attached to e 1 and e 2 and extending from the diameter to the circular boundary of the semicircles.

˜ Let B D be the subset of N ( B ) which consists of the union of B with external semicircles on each edge of B . Moreover, we let γ f 1 ,D be the component of V ( f 1 ) ∩ B D which contains γ f 1 . By Corollary 3, it follows that γ f 1 ,D ⊇ γ f 1 . We define γ f 2 ,D similarly. See Figure 5 for additional details.

Hence, γ f 1 ,D is a curve in V ( f 1 ) ∩ B D with endpoints on the two semicircles attached to e 1 and e 2 . By applying the same argument to γ f 2 ,D , we see that γ f 2 ,D is a curve in V ( f 2 ) ∩ B D with endpoints on two other semicircles of B D . Since α f 1 separates the endpoints of α f 2 on the boundary of B , we conclude that the endpoints of γ f 1 ,D separate the endpoints of γ f 2 ,D on the boundary of B D . This implies the existence of a crossing in B D ⊆ N ( B ).

By investigating the proof of Proposition 14 in detail, we find that each intersection of A ( f 1 ) and A ( f 2 ) corresponds to an intersection between γ f 1 ,D and γ f 2 ,D within B D . By construction, γ f 1 ,D and γ f 2 ,D are components of V which cross B , perhaps with excursions. We claim that these two crossing components uniquely determine B . In particular, since A ( f 1 ) and A ( f 2 ) intersect in the interior of B , the ends of γ f 1 ,D and γ f 2 ,D are in different boxes of the subdivision from each other. Hence, there is no other box of the subdivision that contains a portion of both γ f 1 ,D and γ f 2 ,D as crossing components. Therefore, no two crossings of A ( f 1 ) and A ( f 2 ) can correspond to the same intersection of V ( f 1 ) and V ( f 2 ) because at least one of γ f 1 ,D and γ f 2 ,D changes when considering a different crossing. Therefore, there is an injective map between transversal intersections of A ( f 1 ) and A ( f 2 ) and transversal intersections of V ( f 1 ) and V ( f 2 ).

## 3.2. Missing intersections

We begin by noting that excursions are the only reason that the approximations A ( f 1 ) and A ( f 2 ) can miss an intersection of V ( f 1 ) and V ( f 2 ).

Lemma 15. Suppose that B is a box such that C f 1 1 ( B ) = C f 2 1 ( B ) = C × 1 ( B ) = True . Suppose, in addition, that there are no excursions either entering or exiting B . If the approximations A ( f 1 ) and A ( f 2 ) do not intersect in B , including on the boundary of B , then V ( f 1 ) and V ( f 2 ) do not intersect in B .

Proof. By the properties of the first step of the Plantinga and Vegter algorithm, see Section 2.3, since there are no excursions, the approximations A ( f 1 ) ∩ B and A ( f 2 ) ∩ B are each ambient isotopic to V ( f 1 ) ∩ B and V ( f 2 ) ∩ B within the box B , respectively. In particular, this implies that the number of components of A ( f 1 ) and V ( f 1 ) agree within B , and, similarly for A ( f 2 ) and V ( f 2 ). We recall, however, that these isotopies are not necessarily the same.

Suppose that V ( f 1 ) and V ( f 2 ) intersect in B , but A ( f 1 ) and A ( f 2 ) do not intersect. Let γ f 1 be a component of V ( f 1 ) ∩ B that intersects V ( f 2 ) ∩ B . Let α f 1 be the corresponding subset of A ( f 1 ) to which γ f 1 deforms under the ambient isotopy. Let e 1 and e 2 be the edges of the subdivision that contain the endpoints of α f 1 . By Lemma 8, γ f 1 must also begin and end on these edges. On the other hand, since A ( f 2 ) does not have vertices on these edges, it follows that on each edge e i , the value of f 2 is the same at both endpoints of e i . Since there are no excursions, it follows that V ( f 2 ) does not intersect this edge as V ( f 2 ) would need to intersect this edge twice to maintain the sign properties of the endpoints and this would be an excursion by Lemma 7.

Since A ( f 1 ) and A ( f 2 ) do not intersect, the signs of f 2 at both endpoints of α f 1 must be the same. Since we showed that the sign of f 2 is constant on the edges containing the endpoints of f 1 , the signs of f 2 on both endpoints of γ f 1 must be the same. This implies V ( f 2 ) must intersect γ f 1 an even number of times as each intersection changes the sign of the restriction f 2 | γ f 1 . Since V ( f 2 ) and γ f 1 intersect at least once, they must intersect at least twice, but this is impossible by Corollary 10.

Lemma 15 implies that missing intersections must involve at least one excursion. Our plan is to show that any missing intersection must induce a pair of intersections in the neighborhood N 2 ( B ), which is not possible since C × 1 ( N 2 ( B )) = True .

Proposition 16. Suppose that B is a box such that C f 1 1 ( B ) = C f 2 1 ( B ) = C × 1 ( N 2 ( B )) = True . Suppose that A ( f 1 ) and A ( f 2 ) do not intersect in N ( B ) , including on the boundary of N ( B ) , then V ( f 1 ) and V ( f 2 ) do not intersect in B .

Proof. Suppose that V ( f 1 ) and V ( f 2 ) intersect in B . Let γ f 1 be the component of V ( f 1 ) in V ( f 1 ) ∩ B that includes this intersection. Let ˜ γ f 1 be the extension of γ f 1 into N ( B ) including all excursions into N 2 ( B ), see Figure 6.

˜ Consider the restriction f 2 | ˜ γ f 1 . The signs of this function at the two endpoints must be opposite because each intersection between V ( f 2 ) and ˜ γ f 1 changes the sign of f 2 | ˜ γ f 1 . Having the same sign at both endpoints would require two intersections for the two sign changes, but this is impossible by Corollary 10 and that C × 1 ( N 2 ( B )) = True .

Let α f 1 be the component of A ( f 1 ) ∩N ( B ) which contains the image of γ f 1 under the ambient isotopy. We note that the image of γ f 1 is in N ( B ) since the Plantinga and Vegter algorithm does not deform the curve further than one box away. Let e 1 and e 2 be the edges of the subdivision that contain the endpoints of α f 1 . By Lemma 8, the endpoints of ˜ γ f 1 are also on the edges e 1 and e 2 . Let p 1 be the endpoint of ˜ γ f 1 on e 1 and p 2 be the endpoint of γ f 1 on e 2 .

Now, we prove that the sign of f 2 at p 1 agrees with its signs at the endpoints of e 1 . Since A ( f 2 ) does not intersect A ( f 1 ) and A ( f 1 ) has a vertex on e 1 , this implies that the signs of f 2 on the endpoints of e 1 are the same. Hence, any intersection of V ( f 2 ) with e 1 must be an excursion. Suppose, for contradiction, that the sign of f 2 at p 1 does not match the sign of f 2 at the endpoints of e 1 . Then, there are an odd number of intersections from V ( f 2 ) ∩ e 1 on either side of p 1 . Therefore, there is at least one pair of points of V ( f 2 ) ∩ e 1 on either side of p 1 which are connected by an excursion.

We show that this excursion implies that V ( f 1 ) and V ( f 2 ) intersect at least once more in N 2 ( B ), but such an intersection is not possible since C × 1 ( N 2 ( B )) = True and Corollary 10. Since V ( f 2 ) has an excursion on edge e 1 , V ( f 1 ) cannot have an excursion on this edge by Lemma 9.

Figure 6: An illustration of the setup described in the proof of Proposition 16. The two intersections between ˜ γ f 1 and excursions of ˜ γ f 2 are the impossible cases from the proof. The thick dashed lines on the boundary of N ( B ) illustrate the two components of ∂ N ( B ) split by the endpoints of ˜ γ f 1 .

<!-- image -->

The excursion of V ( f 2 ) can be internal or external to N ( B ), but the proofs are symmetric. Therefore, we focus on the external case. Since V ( f 1 ) does not have an excursion on e 1 , there is a subset γ f 1 ,D of V ( f 1 ) which is a curve contained within the external semicircle and connects p 1 to the curved edge of the semicircle. By Lemma 7, the excursion of V ( f 2 ) separates the endpoints of γ f 1 ,D . Since V ( f 1 ) cannot have an excursion on e 1 and the excursion of V ( f 2 ) must remain within the semicircle on e 1 by Corollary 3, it must be that γ f 1 ,D intersects V ( f 2 ), implying that V ( f 1 ) and V ( f 2 ) intersect an additional time in N 2 ( B ), which is not possible. For the argument in the internal case, we replace the external semicircle with the internal semicircle.

We have shown that the sign of f 2 at p 1 agrees with the sign of f 2 at the endpoints of e 1 . Similarly, the sign of p 2 agrees with the sign of f 2 at the endpoints of e 2 . Moreover, since the signs of f 2 at p 1 and p 2 differ, the sign of f 2 at the endpoints of e 1 differ from the sign of f 2 at the endpoints of e 2 . We also observe that removing edges e 1 and e 2 from the boundary ∂ N ( B ) splits the boundary into two components. Since the signs of f 2 are different at the endpoints of the two components, there must be an odd number of vertices of A ( f 2 ) on each of these two components. However, since A ( f 2 ) forms a perfect matching on its vertices in N ( B ), one of the edges of A ( f 2 ) must intersect α f 1 , but this is not possible.

Therefore, by Proposition 14, every transversal intersection of the approximations A ( f 1 ) and A ( f 2 ) corresponds to a unique intersection of the varieties V ( f 1 ) and V ( f 2 ). On the other hand, Proposition 16 implies that every intersection of the varieties V ( f 1 ) and V ( f 2 ) corresponds to an intersection of the approximations A ( f 1 ) and A ( f 2 ). However, this intersection does not need to be transversal, see Figure 7.

## 3.3. Shared edges in approximations

For the approximations A ( f 1 ) and A ( f 2 ), we call an intersection non-transversal if the approximations meet on the boundary of a box or the approximations coincide along shared segments.

Definition 17. A contiguous sequence of boxes S with shared segments or vertices of A ( f 1 ) and A ( f 2 ) is called a snake provided that the union of boxes in S is topologically a disk, see Figure 8. The boxes where the approximations separate are called the heads of the snake. We call the non-head boxes the interior of the snake. A neighborhood N ( S ) of a snake is the union of all the neighborhoods of boxes in the snake along with the neighborhoods of the heads of the snake. The neighborhood N i ( S ) is defined as N i ( S ) := N ( N i -1 ( S )) .

Figure 7: Approximations from the standard Plantinga and Vegter algorithm which share a segment while the curves intersect in two places. The approximations and the curves are paired by color and line style.

<!-- image -->

Figure 8: Example of a snake where the approximations share segments. The shared segments are illustrated by a thicker line in purple.

<!-- image -->

The definition of a snake and the properties of boxes accepted by the approximation algorithm lead to many strong properties about the topological behavior of snakes.

Lemma 18. Suppose that S is a snake and for every box B ∈ S , C × 1 ( B ) and C f 1 1 ( B ) hold. Then, every box in the interior of the snake has exactly one component of the approximation.

Proof. Suppose that B is an interior box of the snake with two components of the approximation. The two components cannot intersect the same side of B because this would satisfy the conditions of Lemma 9, which is not possible. Therefore, all four sides of the box must have a point of the approximation. This leads to an alternating sign pattern on the vertices of the box, which violates C f 1 1 as shown in Plantinga and Vegter (2004, 2007).

Lemma 19. Suppose that S is a snake and for every box B ∈ S , C × 1 ( N ( B )) holds. Let B 1 and B 2 be in the interior of S such that B 1 and B 2 share an edge. If B 1 and B 2 are the same size, then the shared approximation passes through that shared edge.

Proof. We first observe that if two sides of B 1 and B 2 are collinear, then the approximation cannot pass through both sides, as this would satisfy the conditions of Lemma 9, which is not possible. Wlog, we consider the case where B 1 is above B 2 . Up to symmetry, there are two cases to consider. If the curve approximation passes from the west to east sides of B 2 , see Figure 9(a), then there are not enough sides of B 1 for the curve approximation to pass through. So, the approximation of the snake does not pass through B 1 , so B 1 is not part of the snake, which is a contradiction.

Figure 9: Case-by-case analysis for Lemmas 19 (a,b) and 20 (c,d). In every case, condition C × 1 would be violated, so every one of these examples is not possible.

<!-- image -->

Suppose now that the approximation passes through the south side of B 2 , and, up to symmetry, the west side of B 2 , see Figure 9(b). This leaves two sides of B 1 for which the approximation can pass through without violating the observation above: the north and east sides of B 1 . From the curves of the approximation in these two boxes, we can infer the sign pattern of f 1 and f 2 at the corners of the boxes. For instance, if f 1 has a positive value at the southwest corner of B 2 , then it has a negative value at the northwest corner of B 1 and the southeast corner of B 2 . In addition, it has a positive value at the northeast corner of B 1 . This sign pattern is not allowed since, by the intermediate value theorem, there are a pair of points on the east and west sides of B 1 ∪ B 2 where f 1 has the same value. By a similar argument, the same result holds for f 2 . These conditions satisfy the hypotheses of Lemma 9, and so this configuration is impossible.

Lemma 20. Suppose that S is a snake and for every box B ∈ S , C × 1 ( N ( B )) holds. Let B 1 and B 2 be in the interior of S such that B 1 and B 2 share an edge. If B 1 is half the size of B 2 , then the shared approximation passes through that shared edge or the shared edge of the common neighbor of B 1 and B 2 .

Proof. Up to symmetry, suppose that B 1 is above B 2 and that the west sides of B 1 and B 2 are aligned. Let B ′ 1 be the complementary box, which is the same size as B 1 and is along the same edge of B 2 . We note that by the observation at the beginning of Lemma 19, the approximations cannot pass through the same side of a box twice when the neighbor has been subdivided.

Up to symmetry, there are two cases to consider. If the curve approximation passes from the west to east sides of B 2 , then the curve must pass from the north to east sides of B 1 , but then, there are not enough edges in B ′ 1 to continue the approximation by the observation above.

Now, assume that the approximation passes through the south side of B 2 , and, up to symmetry, the west side of B 2 . This leaves two sides for B 1 to continue the approximation, its north and east sides. By looking at the sign patterns that these intersections imply, we are in a nearly identical case as in the second part of Lemma 19, and this is not possible by Lemma 9.

Together, Lemmas 19 and 20 imply that three of the sides of a head of a snake have no adjacent snake boxes. Since a snake must be topologically a disk, if there were another neighbor to the head of the snake, then all of the boxes inside the loop would need to contain the approximation. However, an interior box B of the snake cannot be completely surrounded by other interior boxes of the snake since Lemmas 19 and 20 would imply that there are two components in B , but this violates Lemma 18. Therefore, the only possibility would be for the snake to curl around the head. This, however, leads to two crossings on the collinear sides, which is also not possible, see Figure 10. We collect this result in the following lemma.

Lemma 21. Suppose that S is a snake and for every box B ∈ S , C × 1 ( N ( B )) holds. Three of the sides of each head of the snake are not adjacent to any other boxes of the snake.

Figure 10: It is impossible for an interior box of a snake to be adjacent to the head of a snake. The box labeled H is the head of the snake and the vertical line to the right of the head of the snake contains two intersections with the approximations, which is not possible.

<!-- image -->

Proposition 22. Suppose that S is a snake and for every box B ∈ S , C × 1 ( N ( B )) holds. There is at most one crossing in N ( S ) corresponding to the snake.

Proof. Let α f 1 = α f 2 be the two subsets of the approximation in S which share vertices or segments. Let γ f 1 be the subset of V ( f 1 ) ∩N ( S ) that deforms to α f 1 under the ambient isotopy. Let γ f 1 be the extension γ f 1 in N 2 ( S ), but without excursions. We define γ f 2 and γ f 2 analogously, see Figure 11.

Suppose that γ f 1 and γ f 2 intersect multiple times in N ( S ). We show that this violates C × 1 ( N ( B )) for some B ∈ S , see Figure 11 for details. Walking along γ f 1 , we observe that the sign of f 2 | γ f 1 changes precisely when the curve γ f 1 crosses γ f 2 . In particular, the type of crossing, that is, from negative to positive or from positive to negative, alternate. Therefore, since there are at least two crossings, there are two crossings where the sign of ∇ f 1 ×∇ f 2 differs.

We consider both γ f 1 and γ f 2 as paths and fix an orientation for each path. Walking along the path γ f 1 , the sign of ∇ f 1 ×∇ f 2 at intersections is completely determined by whether γ f 1 is passing from the negative side of γ f 2 to the positive side or vice versa.

Finally, we apply the function ∇ f 1 ×∇ f 2 to the path γ f 1 . In other words, we look at ∇ f ( γ f 1 ) ×∇ g ( γ f 1 ). We know that the value of this continuous function has different signs along the path. Hence, by the intermediate value theorem, there is some point on this curve where this function vanishes, that is, the gradients of f 1 and f 2 are parallel, but this is not possible as it would contradict C × 1 ( N ( B )) = True for all B ∈ S .

We have shown that every snake corresponds to at most one intersection of V ( f 1 ) and V ( f 2 ). It remains to discuss how to decide if a snake corresponds to an intersection. We begin by defining the orientation of points with respect to the snake. Let S be a snake and let B be a head of the snake. Suppose that p and q are two points on the external boundary ∂B of B , that is, the three sides of B that do not include the snake. We say that q is clockwise from p with respect to the snake if walking around ∂B in a clockwise direction starting at the snake reaches p before q . In this case, p is counterclockwise from q with respect to the snake, see Figures 8 and 11.

Lemma 23. Let S be a snake and let α f 1 and α f 2 be the two components of A ( f 1 ) ∩ S and A ( f 2 ) ∩ S , respectively, that include the shared edges of the snake. Let B 1 and B 2 be the two heads of S . Let p 1 and q 1 be the ends of α f 1 and α f 2 , respectively, in B 1 , and define p 2 and q 2 similarly. The snake corresponds to an intersection if and only if the orientations from p 1 to q 1 and p 2 to q 2 are the same.

Proof. Let γ f 1 be the preimage of α f 1 under the Plantinga and Vegter isotopy. In the Plantinga and Vegter isotopy, there is some flexibility in how the isotopy is done. In particular, when removing excursions on an edge e , there is at least one point of V ( f 1 ) ∩ e which can remain fixed throughout the isotopy, for instance, one of the extremal points of V ( f 1 ) ∩ e . By appealing to this flexibility, we may assume that γ f 1 ends on the same edges of the subdivision as p 1 and p 2 . Note that γ f 1 is contained in N ( S ), but might not be a component of either V ( f 1 ) ∩ N ( S ) or V ( f 1 ) ∩ S since there is no guarantee that the endpoints of γ f 1 correspond to either the first or last excursion.

Figure 11: An illustration of the details of the proof of Proposition 22. At two consecutive intersections, the turn directions of the gradients differ. Therefore, the gradients of f 1 and f 2 must be parallel somewhere along the path between the intersections.

<!-- image -->

Let ˜ γ f 1 be the extension of γ f 1 into N ( S ) with excursions. We define γ f 2 and ˜ γ f 2 similarly. Then the endpoints of ˜ γ f 1 and ˜ γ f 2 are on the same edges of the subdivision as α f 1 and α f 2 . Since the endpoints of α f 1 and α f 2 are on different edges of the subdivision, the endpoints of α f 1 and α f 2 have the same clockwise or counterclockwise relationship as the endpoints of γ f 1 and γ f 2 .

˜ We observe that external excursions from non-head boxes of S cannot include intersections as follows: Suppose, for contradiction, that an external excursion contains an intersection. If both V ( f 1 ) and V ( f 2 ) had an excursion on the same external edge, then this would violate Lemma 9. Therefore, if V ( f 1 ) has an excursion on edge e , then f 2 | e does not change signs. Therefore, by restricting f 2 to the excursion of V ( f 1 ), there must be an even number of intersections because each intersection corresponds to a sign change of f 2 . This contradicts Proposition 22, so the number of intersections is 0. Since there are no intersections in S D \ S except perhaps between the curves of the heads of the snake, we may slightly shrink S D without changing the topology of S D , if necessary, so (1) that the endpoints of γ f 1 ,D and γ f 2 ,D are on the boundary of S D and (2) each region of S D \ S contains excursions from a single box of S .

˜ ˜ Let S D be the subset of N ( S ) consisting of the union of S with external semicircles on the external edges of S . If necessary, we slightly shrink S D in the following argument so that S and S D have the same topology. Let γ f 1 ,D be the extension of ˜ γ f 1 into S D until the curve reaches the circular boundaries of the semicircles attached to the edges containing p 1 and p 2 . By Corollary 3, γ f 1 ,D ⊇ γ f 1 . We define γ f 2 ,D similarly.

By the condition in Definition 17, S and S D are topologically disks and their boundaries are topological circles. Moreover, γ f 1 ,D and γ f 2 ,D form curves across these disks. The paths γ f 1 ,D and γ f 2 ,D intersect if and only if their endpoints interweave along the boundary of the circle. Interweaving is equivalent to having the same clockwise or counterclockwise order of their endpoints around the circle. Since their endpoints are on the curved semicircular regions attached to the edges of S , the endpoints of α f 1 and α f 2 are ordered in the same way as the endpoints of γ f 1 ,D and γ f 2 ,D .

## 3.4. Topological correctness

We provide a complete algorithm for approximating a pair of curves and prove that the output of this algorithm is topologically correct.

Theorem 24. Suppose that f 1 , f 2 ∈ Z [ x, y ] , R = [ a, b ] × [ c, d ] ⊆ R 2 a rectangle such that a, b, c, d ∈ Z . Suppose also that V ( f 1 ) and V ( f 2 ) are nonsingular within R , do not have a common intersection on the boundary of R , and intersect transversely within R . Let A ( f 1 ) and A ( f 2 ) be the output of Algorithm 2.

## Algorithm 2 Simultaneous approximation algorithm for two curves

```
Input : polynomials f 1 , f 2 ∈ Z [ x, y ], with V ( f 1 ) and V ( f 2 ) smooth curves such that V ( f 1 ) and V ( f 2 ) intersect transversely, and a square region R with integral corners such that V ( f 1 ) and V ( f 2 ) do not intersect on the boundary of R . Output : approximations A ( f 1 ) and A ( f 2 ) such that ( A ( f 1 ) , A ( f 2 )) approximates ( V ( f 1 ) , V ( f 2 )). 1: Subdivide R using the new subdivision step, Algorithm 1. 2: Further subdivide boxes B intersecting ∂R until either C f 1 0 ( B ) = True or C f 2 0 ( B ) = True . 3: Further subdivide boxes until the side lengths of neighboring boxes differ by at most a factor of two. 4: Compute the Plantinga and Vegter curve approximation. 5: Subdivide boxes which have shared approximations, but fail the condition of Definition 17. 6: Recompute the Plantinga and Vegter curve approximations, if necessary. 7: For any snakes, apply Lemma 23. 8: if there is no crossing then 9: slightly separate the edges of the snake so that the common edges do not overlap. 10: else 11: slightly separate the ends of the snake and add an explicit crossing in the middle of the snake. 12: end if 13: return the approximations
```

There exists a set U containing R and an ambient isotopy defined on U that simultaneously takes V ( f 1 ) to A ( f 1 ) and V ( f 2 ) to A ( f 2 ) , respectively.

Proof. Through a rescaling, we reduce to the case where R is a square. Second, we use the techniques for unbounded curves from Burr et al. (2012) for the boundary boxes, as long as, in each boundary box, either C f 1 0 ( B ) = True or C f 2 0 ( B ) = True . Therefore, we focus on the topological correctness in the interior of R . By Proposition 16, every intersection between V ( f 1 ) and V ( f 2 ) corresponds to either a transversal crossing of A ( f 1 ) and A ( f 2 ) or a snake. By investigating the proofs of Propositions 14 and 22, we see that these two features correspond to different types of crossings and cannot identify the same crossing twice. Thus, by Lemma 23, we identify exactly when a crossing occurs. This gives a bijection between crossings in the approximation and crossings in the varieties.

We begin the isotopy by deforming space so that each excursion is moved into a neighboring box. This can be done as no intersection can involve two excursions as follows: Suppose, to the contrary, that there is a box B such that V ( f 1 ) and V ( f 2 ) have excursions in B which intersect. Let the curves of these excursions be γ f 1 and γ f 2 , respectively. If these excursions are based on different edges of the subdivision, then γ f 1 along with the segment between its endpoints forms a closed loop. If the path γ f 2 enters the region bounded by this closed loop, then it must also exit this region as the endpoints of γ f 2 are outside of this region. By assumption, no intersection can be on the segment between the endpoints of γ f 1 . Hence, the second intersection must also be with the curve γ f 1 This accounts for two intersections within B , which is not possible by Lemma 10.

On the other hand, if these excursions are on the same edge of the subdivision, then the endpoints of γ f 1 and γ f 2 satisfy the conditions of Lemma 9, which is also not possible. Therefore, it is possible to apply this isotopy without introducing or removing any intersections between the curves.

On the other hand, suppose that no two components have endpoints on the same edge of the subdivision. In addition, suppose that γ f 1 and γ f 2 intersect. By Corollary 10, these two curves cannot intersect more than once. Therefore, the endpoints of γ f 1 separate the endpoints of γ f 2 . Moreover, since the edges containing the endpoints are different, we find, by the intermediate value theorem, that any path in B between the edges

Let ˜ V ( f 1 ) and ˜ V ( f 2 ) be the result of applying the isotopy from the paragraph above. By the correctness of the Plantinga and Vegter algorithm, within each box B , ˜ V ( f 1 ) ∩ B is ambient isotopic to A ( f 1 ) ∩ B . Similarly, ˜ V ( f 2 ) ∩ B is ambient isotopic to A ( f 2 ) ∩ B . Let γ f 1 be a component of ˜ V ( f 1 ) and γ f 2 a component of ˜ V ( f 2 ). If an endpoint of γ f 1 and γ f 2 are on the same edge of the subdivision, then the straightening step in the Plantinga and Vegter isotopy moves the endpoints to the same point, that is, it creates a snake.

containing the endpoints of γ f 1 must intersect any path in B between the edges containing the endpoints of γ f 2 . In particular, this implies that the approximations intersect in B .

Finally, for any components of ˜ V ( f 1 ) ∩ B and ˜ V ( f 2 ) ∩ B which do not intersect other curve components and whose endpoints lie on edges which are not shared by other curves, we observe that these components can be straightened without intersecting other components. In particular, as a curve is straightened its endpoints do not leave the edges in which they start and the interior of the curve does not intersect the boundary of the box. Hence, during the straightening step of the Plantinga and Vegter isotopy, each curve can be straightened individually while the rest of the curves are pushed off to the side. This straightens all of the curve components within the box without introducing intersections.

We note that small Hausdorff distance between the approximation and the variety can be achieved by making sure that the boxes containing the approximations are sufficiently small and that any snakes are also small. The output of this algorithm on the polynomials generating Figures 4(a) and 4(b) appears in Figure 12. These new versions correctly approximate the topology of the curves.

Figure 12: Topologically correct versions of (a) Figure 4(a) and (b) Figure 4(b) using Algorithm 2.

<!-- image -->

## 4. Complexity Analysis

We provide the complexity analysis of the approximation algorithm for two curves. In this analysis, we compute the number of boxes that are used in the subdivision as well as the bit-complexity of the algorithm. We provide both adaptive bounds based on continuous amortization (Burr et al., 2020; Burr, 2016; Burr and Krahmer, 2012) and non-adaptive worst-case bounds. Our analysis relies heavily on the analysis in Burr et al. (2020). For our bit-complexity analysis, we assume that Step 5 of Algorithm 2 is not a dominating term of the complexity.

Our analysis assumes that all predicates are implemented using the centered form , see, for example Ratschek and Rokne (1984). In addition, we observe that all of our predicates are for tests for the form 0 ̸∈ F ( B ) for some appropriate function F and axis-aligned square or four-dimensional cube region B . Using the centered form, these tests simplify to

<!-- formula-not-decoded -->

In our complexity analysis, we use a slightly weaker version of Algorithm 1. Instead of testing C × 1 ( N 2 ( B ) ∩ 13 B ), we only test C × 1 (13 B ). If C × 1 (13 B ) = True , then C × 1 ( N 2 ( B ) ∩ 13 B ) = True . This change can only increase the number of boxes found in the subdivision.

where m is the midpoint of B , that is, the average of the vertices, and w is the width of B , that is, the length of a side, see (Burr et al., 2020, Corollary 14) for additional details.

## 4.1. Adaptive bounds

We begin by recalling the basic facts of continuous amortization as developed in Burr et al. (2020), Burr (2016), and Burr and Krahmer (2012).

Definition 25 (cf. Burr et al. (2020, Definition 4)) . Let R = [ a, b ] × [ c, d ] ⊆ R 2 be a rectangle and C a predicate. A local size bound for C is a function F : R → R ≥ 0 with the property that

<!-- formula-not-decoded -->

In other words, F ( x ) is a lower bound on the area of any square containing x which fails the predicate. This local size bound provides a way to calculate the complexity of a subdivision-based algorithm.

Proposition 26 (cf. Burr et al. (2020, Theorem 5)) . Let R = [ a, b ] × [ c, d ] ⊆ R 2 be a square, C a predicate, and F a local size bound for C . Let h : R ≥ 0 → R be a non-increasing function. Let P be the final partition formed by iteratively splitting R into four smaller squares until C holds on each square. Then

<!-- formula-not-decoded -->

If h ≡ 1 , then the integral bounds the number of regions formed by the subdivision. If the subdivision does not terminate, then the integral is infinite.

In the current paper, there are five predicates to consider C f 1 0 , C f 2 0 , C f 1 1 , C f 2 1 , and C × 1 . Of these, there are local size bounds provided in Burr et al. (2020) for the first four. In particular, Burr et al. (2020, Corollary 16) gives a local size bound for C f i 0 :

<!-- formula-not-decoded -->

where dist C ( x, f i ) denotes the distance from x ∈ R 2 to the complex variety V C ( f i ). Similarly, Burr et al. (2020, Corollary 19) gives a local size bound for C f i 1 :

<!-- formula-not-decoded -->

It remains for us to compute a local size bound for C × 1 . Define h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) × ∇ f 2 ( y 1 , y 2 ) , where we omit the selection of the third (and only nonzero) entry in the cross product. We observe that the C × 1 test is based on whether □ h ( B,B ) contains 0. The construction in Burr et al. (2020) shows how to develop a local size bound for any function of this form. We apply these results to find a local size bound for C × 1 .

where g i ( x 1 , x 2 , y 1 , y 2 ) = ⟨∇ f 1 ( x 1 , x 2 ) , ∇ f 2 ( y 1 , y 2 ) ⟩ , and dist C (( x, x ) , g i ) denotes the distance from the point ( x, x ) in the diagonal of R 2 × R 2 ⊆ C 4 to the complex variety V C ( g i ).

Lemma 27. Let f 1 , f 2 ∈ R [ x, y ] and define h ∈ R [ x 1 , x 2 , y 1 , y 2 ] as h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) × ∇ f 2 ( y 1 , y 2 ) . Let B ⊆ R 2 , and suppose that there is a point x ∈ B such that

where d = max { deg( f 1 ) , deg( f 2 ) } . Then C × 1 (13 B ) = True . Moreover, a local size bound for this test is

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Proof. By inspection, the bounds arrived at in Burr et al. (2020, Corollaries 19 and 20) depend only on the degrees and numbers of variables of the polynomials. We observe that the degree of h is bounded by 2( d -1) and that f 1 and f 2 are bivariate polynomials. These are the exact same estimates that are used to develop the bounds in Burr et al. (2020, Corollaries 19 and 20). Therefore, the bounds in Burr et al. (2020, Corollaries 19 and 20) apply in this case, and the bounds above follow directly.

Since the predicates in Algorithm 1 are compound predicates, we combine the local size bounds from above to accommodate these predicates. We denote the predicates as C 00 , C 01 , C 10 , and C 11 , where the subscripts indicate whether C f i 0 or C f i 1 is used. For example, C 01 ( B ) is True when both C f 1 0 ( B ) and C f 2 1 ( B ) are True . Hence, C 01 ( B ) is guaranteed to be True if Area( B ) ≤ G f 1 0 ( x ) and Area( B ) ≤ G f 2 1 ( x ) for some x ∈ B . Therefore, the minimum of the two local size bounds is a local size bound for C 01 . More precisely, the local size bounds for the predicates in Algorithm 1 follow.

<!-- formula-not-decoded -->

A direct application of Proposition 26 gives the following result on the complexity of the approximation algorithm for two curves.

Theorem 28. Suppose that f 1 , f 2 ∈ Z [ x, y ] . Define g i ( x 1 , x 2 , y 1 , y 2 ) = ⟨∇ f i ( x 1 , x 2 ) , ∇ f i ( y 1 , y 2 ) ⟩ and h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) ×∇ f 2 ( y 1 , y 2 ) . Suppose that R = [ a, b ] × [ c, d ] is a square subset of R 2 such that a, b, c, d ∈ Z . The number of regions after the subdivision step performed by Algorithm 1 is bounded above by the maximum of 1 and

<!-- formula-not-decoded -->

Moreover, the algorithm does not terminate if and only if the integral diverges.

## 4.2. Nonadaptive bounds

We begin by recalling the structure for complexity estimates for nonadaptive bounds from Burr et al. (2020).

Definition 29 (cf. Burr et al. (2020, Definition 2)) . Let R = [ a, b ] × [ c, d ] ⊆ R 2 a rectangle and C a predicate. C is a diameter distance test if there is a closed set V and positive constant K such that for all squares B , we have

<!-- formula-not-decoded -->

where d ( x, V ) = min v ∈ V d ( x, v ) .

Suppose that there is a collection of predicates { C i } , each with a corresponding closed set V i , and K a uniform constant for all tests. Moreover, define the separation bound δ as 0 &lt; δ ≤ min x ∈ R max i d ( x, V i ) . The product Kδ is a uniform bound on the diameter of a square which is guaranteed to be terminal. Then Burr et al. (2020, Corollary 21) implies that when R is square, the number of boxes produced by Algorithm 1 is at most

<!-- formula-not-decoded -->

From our local size bounds, we see that a uniform bound on K for our tests is

<!-- formula-not-decoded -->

This bound is immediate from our local size bounds from Section 4.1, cf. Burr et al. (2020, Corollaries 17 and 20).

Since the tests in Algorithm 1 are compound tests, the required closed sets are built out of multiple varieties. For instance, for the C 01 test and a square B , if Diam( B ) &lt; K max x B d C ( x, V ( f 1 )) and Diam( B ) &lt;

∈ K max x ∈ B d C ( x, V ( g 2 )), then C 01 ( B ) is True . Therefore, we must find a separation bound δ so that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

By exchanging the maximum and the minimums, we find that Inequality (3) can be rewritten as

<!-- formula-not-decoded -->

For the remainder of this section, we heavily rely on the proofs in Burr et al. (2020). In particular, we omit some details and cite results at the potential cost of some clarity.

We use a direct application of Burr et al. (2020, Theorem 24) to compute lower bounds on both max { dist C ( x, f 1 ) , dist C (( x, x ) , g 1 ) } and max { dist C ( x, f 2 ) , dist C (( x, x ) , g 2 ) } . In particular, the theorem uses only the degrees and the number of variables to reach its result. These same degree bounds and number of variables apply in this case. Specializing the result to the bivariate case results in a bound from below of 2 -O ( d 25 ( d +lg H )) , where H is the maximum absolute value of the coefficients of f 1 , the coefficients of f 2 , and the coordinates of the corners of the initial region R , and d = max { deg( f 1 ) , deg( f 2 ) } .

We now focus on a lower bound for max { dist C ( x, f 1 ) , dist C ( x, f 2 ) , dist C (( x, x ) , h ) } . In order to bound this, we appeal to the computations in Burr et al. (2020). As a first step, we make the ambient spaces of all three varieties the same by defining V ( f ∆ 1 ) and V ( f ∆ 2 ) to be the images of V ( f 1 ) and V ( f 2 ) in the diagonal of C 4 , that is V ( f ∆ i ) = { ( x 1 , x 2 , x 1 , x 2 ) : ( x 1 , x 2 ) ∈ V ( f i ) } . Since f 1 and f 2 define smooth real curves which intersect transversely in R , f 1 ( x ), f 2 ( x ), and h ( x, x ) cannot all vanish simultaneously for x ∈ R . This implies that V ( f ∆ 1 ), V ( f ∆ 2 ), and V ( h ) cannot all intersect simultaneously at points ( x, x ) ∈ R × R . An application of Burr et al. (2020, Proposition 23) results in a bound for when these three varieties do not intersect.

Proposition 30. Let f 1 , f 2 ∈ Z [ x 1 , x 2 ] be of degree at most d defining smooth real curves which intersect transversely. Define h ∈ R [ x 1 , x 2 , y 1 , y 2 ] as h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) ×∇ f 2 ( y 1 , y 2 ) . Suppose that R is an axis-aligned square whose corners have integral coordinates. Let H be the maximum absolute value of the coefficients of f 1 , the coefficients of f 2 , and the coordinates of the corners of H . Let f ∆ i = { f i , x 1 -y 1 , x 2 -y 2 } be the polynomial system corresponding to the image of V C ( f i ) in the diagonal of C 4 and R ∆ = { ( x, x ) : x ∈ R } is the image of R in the diagonal of C 4 . Let ( x, x ) ∈ V C ( f ∆ 1 , f ∆ 2 , h ) , then the distance between ( x, x ) and R ∆ is at least 2 -O ( d 8 ( d +lg H )) .

Proof. The proof is nearly identical to the proof of Burr et al. (2020, Proposition 23). Namely, the only difference in the proof is in the number of inequalities and equalities needed to achieve the bound. As in the original proof, 20 inequalities are used to define the region, 6 equalities each are used to define f 1 and f 2 , and 2 equalities are used to define h . These equalities and inequalities have the same bit-sizes as those appearing in Burr et al. (2020, Proposition 23). This results in 6 more equalities than in the original proof, but the resulting bound is unchanged when specialized to the bivariate case, that is, 2 -O ( d 8 ( d +lg H ))

We may now follow the proofs of Burr et al. (2020, Proposition 22 and Theorem 24) to find a separation bound between the real parts of V ( f ∆ 1 , f ∆ 2 ) and V ( h ).

Proposition 31. Let f 1 , f 2 ∈ Z [ x 1 , x 2 ] be of degree at most d defining smooth real curves which intersect transversely. Define h ∈ R [ x 1 , x 2 , y 1 , y 2 ] as h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) ×∇ f 2 ( y 1 , y 2 ) . Suppose that R is an axis-aligned square whose corners have integral coordinates. Let H be the maximum absolute value of the coefficients of f 1 , the coefficients of f 2 , and the coordinates of the corners of H . Let f ∆ i = { f i , x 1 -y 1 , x 2 -y 2 } be the polynomial system corresponding to the image of V C ( f i ) in the diagonal of C 4 and R ∆ = { ( x, x ) : x ∈ R } is the image of R in the diagonal of C 4 . The distance between V C ( f ∆ 1 , f ∆ 2 ) and V C ( h ) is 2 -O ( d 25 ( d +lg H )) .

Proof. The proof is nearly identical to Burr et al. (2020, Proposition 22), using 20 inequalities to define the region, 6 equalities for each f i and 2 equalities for h . This corresponds to 6 extra equalities, but does not change the overall complexity of the result. The scaled bit-sizes are also unchanged as was used in Burr et al. (2020) (cf. the proof of Lemma 27). After substituting the value of k = O ( d 8 ( d +lg H )), as in Burr et al. (2020, Proposition 24), we arrive at the desired bound.

By the triangle inequality we conclude that for any x ∈ R , either dist C (( x, x ) , h ) = 2 -O ( d 25 ( d +lg H )) or dist C (( x, x ) , ( f ∆ 1 , f ∆ 2 )) = 2 -O ( d 25 ( d +lg H )) . We now bound the distance max { dist C ( x, f 1 ) , dist C ( x, f 2 ) } . To do this, we follow Burr et al. (2020, Proposition 22). It is enough to focus on the distance dist C ( f 1 , f 2 ) and not dist C ( f ∆ 1 , f ∆ 2 ) since these two distances are related by a factor of √ 2.

Lemma 32. Let f 1 , f 2 ∈ Z [ x 1 , x 2 ] be of degree at most an even number d and suppose that B ⊆ R 2 is a square whose corners are rational numbers. Let H be the maximum absolute value of the coefficients of f 1 and f 2 as well as the numerator of the corners of B . Suppose that the denominators of the corners of B are powers of 2 bounded by 2 O ( d 8 ( d +lg H )) . Let k be a positive integer so that for any x ∈ V ( f 1 , f 2 ) , the distance between x and B is more than √ 2 2 k -1 . Then,

<!-- formula-not-decoded -->

Proof. Let ε = 1 2 k and let B ε be a thickening of B by ε . More precisely, B ε is the Minkowski sum of B with ([ -ε, ε ] × [ -iε, iε ]) 2 . By assumption, V C ( f 1 , f 2 ) ∩ B ε is empty. We now apply a homothety centered at the origin of a factor of 2 k + O ( d 8 ( d +lg H )) , which makes the coefficients of the image of B integers. The resulting region has the following bounding inequalities

<!-- formula-not-decoded -->

Moreover, after clearing fractions, the homothety changes the absolute value of the coefficients of f 1 and f 2 to 2 dk + O ( d 9 ( d +lg H )) H . Identifying C 2 with R 4 , we see that the box is defined by 8 inequalities whose coefficients are bounded by 2 k + O ( d 8 ( d +lg H )) H . Moreover, the polynomials f 1 and f 2 are split into their real and complex parts. By splitting in this way, the coefficients may grow by a binomial coefficient which is trivially bounded by 2 d . Therefore, we have four equalities whose coefficients are bounded by 2 d + dk + O ( d 9 ( d +lg H )) H .

Applying Jeronimo et al. (2013, Theorem 1.2) to the inequalities for B ε with f 1 when compared to the inequalities for B ε with f 2 , we get that the separation bound within the scaled copy of B ε is

<!-- formula-not-decoded -->

Reducing this bound by the homothety results in the desired bound.

Finally, to compute max { dist C ( x, f 1 ) , dist C ( x, f 2 ) , dist C (( x, x ) , h ) } , we observe that by Proposition 31 and the triangle inequality either dist C (( x, x ) , h ) = 2 -O ( d 25 ( d +lg H )) or dist C ( x, ( f ∆ 1 , f ∆ 2 )) = 2 -O ( d 25 ( d +lg H )) . In the latter case, we substitute k = O ( d 25 ( d + lg H )) into Lemma 32 to reach a separation bound of 2 -O ( d 34 ( d +lg H )) . Collecting these results, we have the following theorem.

Theorem 33. Let f 1 , f 2 ∈ Z [ x 1 , x 2 ] be smooth and of degree at most d . Define g 1 , g 2 ∈ R [ x 1 , x 2 , y 1 , y 2 ] as g i ( x 1 , x 2 , y 1 , y 2 ) = ⟨∇ f i ( x 1 , x 2 ) , ∇ f i ( y 1 , y 2 ) ⟩ and h ∈ R [ x 1 , x 2 , y 1 , y 2 ] as h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) × ∇ f 2 ( y 1 , y 2 ) . Suppose that R is an axis-aligned square whose corners have integral coordinates. Let H be the maximum absolute value of the coefficients of f 1 , f 2 , and the coordinates of the corners of H . The number of terminal regions produced by Algorithm 1 is 2 O ( d 34 ( d +lg H )) .

.

Proof. By Equation 1, the number of boxes produced by the algorithm is 2 O (2 lg(Diam( R )) -2 lg K -2 lg δ ) . Since the diameter of R is O ( H ), K = O (1 /d ) by Equation (2), and δ = 2 -O ( d 34 ( d +lg H )) , the result follows.

## 4.3. Bit-Complexity

As above, the bit-complexity calculation is very similar to the bit-complexity calculation from Burr et al. (2020). In fact, since h , g 1 , and g 2 all have the same degree bounds and same bounds on the sizes of their coefficients, the cost to compute all of the predicates needed for Algorithm 1 is no more than three times the cost of the predicates for the Plantinga and Vegter algorithm as discussed in Burr et al. (2020). Using the upper bound on the number of boxes from Theorem 33, and the corresponding lower bound on the minimum width of boxes, the bit-complexity analysis from Burr et al. (2020) carries over to this case.

Theorem 34 (cf. Burr et al. (2020, Corollary 28)) . Let f 1 , f 2 ∈ Z [ x 1 , x 2 ] be smooth and of degree at most d . Define g 1 , g 2 ∈ R [ x 1 , x 2 , y 1 , y 2 ] as g i = ⟨∇ f i , ∇ f i ⟩ and h ∈ R [ x 1 , x 2 , y 1 , y 2 ] as h ( x 1 , x 2 , y 1 , y 2 ) = ∇ f 1 ( x 1 , x 2 ) ×∇ f 2 ( x 1 , x 2 ) . Suppose that R is an axis-aligned square whose corners have integral coordinates. Let τ = lg H be the maximum bit-size of the coefficients of f 1 , f 2 , and the corners of R . The overall bit-complexity of Algorithm 1 is

2 O ( d 34 ( d + τ )) ˜ O B ( d 37 ( d + τ )) , where the ∼ indicates that logarithmic factors have been suppressed and the B indicates that this is measuring a bit-complexity.

Remark 35. We expect that the complexity of Algorithm 1 dominates the complexity of Algorithm 2, but we do not address the additional subdivisions required for boxes on the boundary and to ensure that snakes are disks.

There are also adaptive bounds for the bit-complexity of Algorithm 1. Once again, since the predicates needed for Algorithm 1 have the same data as the tests in the Plantinga and Vegter algorithm, the complexity bounds from Burr et al. (2020) apply directly. In particular the cost functions for computing C f 1 0 and C f 2 0 is

<!-- formula-not-decoded -->

On the other hand, the cost function for computing C f 1 1 , C f 2 1 , and C × 1 is

<!-- formula-not-decoded -->

In these expressions k 0 and k 1 are the maximum values of the suppressed terms of ˜ O B over the region R , see Burr et al. (2020, Section 5.3) for additional details.

Since, on every step of the subdivision algorithm, all of C f 1 0 , C f 2 0 , C f 1 1 , C f 2 1 , and C × 1 are applied, we let h = 2 h 0 + 3 h 1 . Then, the computations in Burr et al. (2020) lead directly to an overall adaptive bit-complexity bound.

Theorem 36. Let f 1 , f 2 ∈ Z [ x, y ] be both smooth and R ⊆ R 2 be an axis-aligned square whose corners have integral coordinates. Let τ be the maximum bit-size of the coefficients of f 1 , the coefficients of f 2 , and the corners of R. The overall bit-complexity of Algorithm 1 is the maximum of h ( w ( R ) n ) and

<!-- formula-not-decoded -->

where h ( x ) = 2 h 0 ( x ) + 3 h 1 ( x ) .

Remark 37. We expect that the analyses for the Plantinga and Vegter algorithm in Cucker et al. (2022) and Tonelli-Cueto and Tsigaridas (2020) can be adapted to provide the complexity for the average case, but leave this step to future work.

## 5. Approximating more than two curves

We now consider the main problem of the paper, approximating any number of curves in the plane. In particular, suppose that f 1 , . . . , f n ∈ Z [ x, y ] define n smooth curves in the plane such that these curves intersect transversely and no three of these curves intersect simultaneously. A first attempt to simultaneously approximate these curves might be first to run the subdivision step, Algorithm 1, on each pair of curves. Second, perform the remaining steps of Algorithm 2 on the common refinement of these subdivisions. Unfortunately, we cannot guarantee that the output of this algorithm is topologically correct since the isotopies guaranteed by Theorem 24 might not be compatible.

The main challenge with approximating more than two curves is that the ambient isotopies are not guaranteed to preserve the order of intersections between multiple curves. In order to avoid this, we require that the intersections are far enough apart so that two intersections cannot interfere with each other through the homotopy. By Propositions 14 and 22, all intersections of the varieties are within the 1-neighborhoods of a box containing an intersection of the approximations or a snake containing an intersection of the approximations. The final step to make sure that the topology is correct is to ensure that these neighborhoods are all disjoint so that the ambient isotopies constructed by Theorem 24 do not allow the intersections of the varieties to interact.

## Algorithm 3 Simultaneous approximation algorithm for any number of curves

̸

- Input : polynomials f 1 , f 2 , . . . , f n ∈ Z [ x, y ], with V ( f i ) a smooth curve for all i such that V ( f i ) and V ( f j ) intersect transversely when i = j , and no three of V ( f i ), V ( f j ), and V ( f k ) simultaneously intersect when i , j , and k are all distinct. A square region R with integral corners such that V ( f i ) and V ( f j ) do not intersect on the boundary of R for i = j . Output : approximations A ( f 1 ) , . . . , A ( f n ) such that ( A ( f 1 ) , . . . , A ( f n )) approximates ( V ( f 1 ) , . . . , V ( f n )). 1: For each pair ( f i , f j ), with i = j , subdivide R using Algorithm 1. 2: Further subdivide boxes B intersecting ∂R until they satisfy C f i 0 ( B ) = True for at least n -1 curves. 3: Further subdivide boxes until the side lengths of neighboring boxes differ by at most a factor of two. 4: Compute the Plantinga and Vegter curve approximations. 5: Subdivide boxes which have shared approximations, but fail the condition of Definition 17. 6: Subdivide boxes or snakes with intersections of the approximations whose 1-neighborhoods intersect. 7: Recompute the Plantinga and Vegter curve approximations, if necessary. 8: For any snakes, apply Lemma 23. 9: if there is no crossing then 10: slightly separate the edges of the snake so that the common edges do not overlap. 11: else 12: slightly separate the ends of the snake and add an explicit crossing in the middle of the snake. 13: end if 14: return the approximations

̸

̸

Theorem 38. Suppose that f 1 , . . . , f n ∈ Z [ x, y ] , R = [ a, b ] × [ c, d ] ⊆ R 2 a rectangle such that a, b, c, d ∈ Z . Suppose also that V ( f i ) is nonsingular within R for all i , there does not exist a pair i = j such that V ( f i , f j ) includes a point on the boundary of R , for every pair i = j , V ( f i ) and V ( f j ) intersect transversely within R , and there does not exist a triple i = j , i = k , and j = k such that V ( f i , f j , f k ) includes a point in R . Let A ( f 1 ) , . . . , A ( f n ) be the output of Algorithm 3. There exists a set U containing R and an ambient isotopy defined on U that simultaneously takes V ( f i ) to A ( f i ) for all i .

̸

̸

̸

Proof. We begin with the same simplification of Theorem 24. This reduces the problem to studying the case where R is a square and allows us to focus on the topological correctness in the interior of R .

We begin the isotopy by deforming space so that each excursion is moved into a neighboring box. This can be done as no intersection can involve two excursions, as discussed in Theorem 24. Therefore, it is possible to apply this isotopy without introducing or removing any intersections between the curves.

̸

̸

̸

̸

By the isotopy argument in Theorem 24, if there is a pair of i = j as identified above, these curves will deform to an intersection or a snake. On the other hand, since the remaining curves do not intersect and are on different edges of the subdivision, the proof shows that they can be deformed, without introducing new intersections into the straight edges of the approximation.

After applying this first isotopy, let ˜ V ( f i ) be the result. By the correctness of the Plantinga and Vegter algorithm each ˜ V ( f i ) ∩ B is ambient isotopic to A ( f i ) ∩ B within each box B . We must now show that these isotopies are compatible. By the assumption on the 1-neighborhoods of crossings of approximations and snakes and appealing to the proof of Theorem 24, we see that in each box B , there is at most one of the following two situations: (1) a pair of i = j such that ˜ V ( f i ) ∩ B and ˜ V ( f j ) ∩ B cross or (2) a pair of i = j with endpoints of ˜ V ( f i ) ∩ B and ˜ V ( f j ) ∩ B which are on the same edge of B . These two cases correspond to a crossing of the approximations and a snake, respectively.

## 6. Examples

We end with a gallery of examples, illustrating the output of our algorithm.

Example 39. Consider the pair of curves given by f 1 = x 2 + y 2 -1 and f 2 = 2000 y 8 + 8000 x 2 y 6 + 12000 x 4 y 4 +8000 x 6 y 2 +2000 x 8 -3000 y 6 +9000 x 2 y 4 -21000 x 4 y 2 -1000 x 6 +1 , as seen in Figure 13(a). Our algorithm correctly finds the twelve intersections between the two curves. In addition, in Table 1, we compare the number and types of boxes produced by our algorithm and what would be produced by applying the Plantinga and Vegter algorithm twice, that is, without invoking C × 1 . We observe that the new predicates only introduce a modest number of new boxes in this example.

|                      |   C f 1 0 ,C f 2 0 |   C f 1 0 ,C f 2 1 |   C f 1 1 ,C f 2 0 |   C f 1 1 ,C f 2 1 ,C 1 |   C f 1 1 ,C f 2 1 |
|----------------------|--------------------|--------------------|--------------------|-------------------------|--------------------|
| Algorithm 2          |               1087 |                489 |                168 |                      24 |                    |
| Plantinga and Vegter |                852 |                340 |                 60 |                         |                 36 |

Table 1: The number of boxes produced by Algorithm 2 and the Plantinga and Vegter algorithm on the curves in Example 39.

Figure 13: (a) Simultaneous approximation of f 1 = x 2 + y 2 -1 (drawn in red) and f 2 = 2000 y 8 +8000 x 2 y 6 +12000 x 4 y 4 + 8000 x 6 y 2 +2000 x 8 -3000 y 6 +9000 x 2 y 4 -21000 x 4 y 2 -1000 x 6 +1 (drawn in blue). This example is inspired from a similar example in (Cheng et al., 2023). (b) Simultaneous approximation of f 1 = 10 xy 3 -x 2 y 2 -6 xy 2 + x 2 y + 5 x 3 + x 2 + 3 y and f 2 = 4 xy 3 -x 2 y 2 -8 x 4 +2 y 3 -8 x 3 +4 xy -2 x 2 -7 x +2. The predicate C × 1 induces intersections because the two curves are nearly parallel.

<!-- image -->

̸

Example 40. Consider the pair of curves f 1 = 2 y 4 +3 x 2 y 2 +8 x 3 y -12 y 2 -18 xy -12 x 2 +18 and f 2 = y 4 +4 xy 3 +3 x 2 y 2 +4 x 3 y + x 4 -12 y 2 -18 xy -12 x 2 +18 , as seen in Figure 14. There is a triple intersection of the two curves along each nearly parallel portion of the curves. Our algorithm correctly finds and approximates all of these intersections. In addition, in Table 2, we compare the number and types of boxes produced by our algorithm and what would be produced by applying the Plantinga and Vegter algorithm twice, that is, without invoking C × 1 . We observe that the new predicates produce significantly more boxes due to the work needed to process the nearly parallel curves.

|                      |   C f 1 0 ,C f 2 0 |   C f 1 0 ,C f 2 1 |   C f 1 1 ,C f 2 0 |   C f 1 1 ,C f 2 1 ,C 1 |   C f 1 1 ,C f 2 1 |
|----------------------|--------------------|--------------------|--------------------|-------------------------|--------------------|
| Algorithm 2          |               1252 |                590 |                584 |                     140 |                    |
| Plantinga and Vegter |                202 |                 86 |                 58 |                         |                 72 |

Table 2: The number of boxes produced by Algorithm 2 and the Plantinga and Vegter algorithm on the curves in Example 40.

Figure 14: (a) Simultaneous approximation of f 1 = 2 y 4 + 3 x 2 y 2 + 8 x 3 y -12 y 2 -18 xy -12 x 2 + 18 and f 2 = y 4 + 4 xy 3 + 3 x 2 y 2 +4 x 3 y + x 4 -12 y 2 -18 xy -12 x 2 +18. This example has several close curves, each with three intersections. (b) An enlarged image of a portion of (a), illustrating the three intersections along one of the nearly parallel portions of the curves.

<!-- image -->

Example 41. Consider the curves f 1 = 10 xy 3 -x 2 y 2 -6 xy 2 + x 2 y +5 x 3 + x 2 +3 y and f 2 = 4 xy 3 -x 2 y 2 -8 x 4 +2 y 3 -8 x 3 +4 xy -2 x 2 -7 x +2 , as seen in Figure 13(b). The two curves share an asymptote, and our algorithm correctly separates the two curves as they become nearly parallel. In addition, in Table 3, we compare the number and types of boxes produced by our algorithm and what would be produced by applying the Plantinga and Vegter algorithm twice, that is, without invoking C × 1 . We observe that the new predicates produce significantly more boxes due to the work needed to separate the curves as they become nearly parallel.

|                      |   C f 1 0 ,C f 2 0 |   C f 1 0 ,C f 2 1 |   C f 1 1 ,C f 2 0 |   C f 1 1 ,C f 2 1 ,C 1 |   C f 1 1 ,C f 2 1 |
|----------------------|--------------------|--------------------|--------------------|-------------------------|--------------------|
| Algorithm 2          |                310 |                186 |                171 |                       6 |                    |
| Plantinga and Vegter |                188 |                 71 |                 69 |                         |                 39 |

Table 3: The number of boxes produced by Algorithm 2 and the Plantinga and Vegter algorithm on the curves in Example 41.

Example 42. Consider the curves f 1 = 16 y 2 +16 x 2 -24 y -16 x -3 , f 2 = 16 y 2 +16 x 2 -24 y +16 x -3 and f 3 = 100 y 2 +100 x 2 +180 y -19 . The output of Algorithm 3 for these three curves appears in Figure 15. Even though there are three nearby intersections, our algorithm correctly approximates both their locations and correctly computes the orders in which the curves intersect each other.

Figure 15: Simultaneous approximation of f 1 = 16 y 2 + 16 x 2 -24 y -16 x -3, f 2 = 16 y 2 + 16 x 2 -24 y + 16 x -3 and f 3 = 100 y 2 +100 x 2 +180 y -19. For approximating more than two curves, we see in this example that the intersections from distinct pairs are in separate neighborhoods.

<!-- image -->

## References

- Michael Burr and Michael Byrd. Certified simultaneous isotopic approximation of pairs of curves via subdivision. In Proceedings of the 2023 International Symposium on Symbolic and Algebraic Computation , page 116-125, New York, NY, USA, 2023. Association for Computing Machinery.
- Michael Burr, Sung Woo Choi, Ben Galehouse, and Chee K. Yap. Complete subdivision algorithms, II: Isotopic meshing of singular algebraic curves. Journal of Symbolic Computation , 47(2):131-152, 2012.
- Michael Burr, Shuhong Gao, and Elias Tsigaridas. The complexity of subdivision for diameter-distance tests. Journal of Symbolic Computation , 101:1-27, 2020.
- Michael A. Burr. Continuous amortization and extensions: with applications to bisection-based root isolation. Journal of Symbolic Computation , 77:78-126, 2016.
- Michael A. Burr and Felix Krahmer. SqFreeEVAL : an (almost) optimal real-root isolation algorithm. Journal of Symbolic Computation , 47(2):153-166, 2012.
- Jin-San Cheng, Junyi Wen, and Bingwei Zhang. Certified numerical real root isolation for bivariate nonlinear systems. Journal of Symbolic Computation , 114:149-171, 2023.
- Felipe Cucker, Alperen A. Erg¨ ur, and Josu´ e Tonelli-Cueto. On the complexity of the Plantinga-Vegter algorithm. Discrete &amp; Computational Geometry. An International Journal of Mathematics and Computer Science , 68(3):664-708, 2022.
- Gabriela Jeronimo, Daniel Perrucci, and Elias Tsigaridas. On the minimum of a polynomial function on a basic closed semialgebraic set and applications. SIAM Journal on Optimization , 23(1):241-255, 2013.
- Jyh-Ming Lien, Vikram Sharma, Gert Vegter, and Chee Yap. Isotopic arrangement of simple curves: an exact numerical approach based on subdivision. In Mathematical software-ICMS 2014 , volume 8592 of Lecture Notes in Computer Science , pages 277-282. Springer, Heidelberg, 2014.
- Long Lin and Chee Yap. Adaptive isotopic approximation of nonsingular curves: the parameterizability and nonlocal isotopy approach. Discrete &amp; Computational Geometry. An International Journal of Mathematics and Computer Science , 45(4): 760-795, 2011.
- William E. Lorensen and Harvey E. Cline. Marching cubes: A high resolution 3d surface construction algorithm. In Proceedings of the 14th Annual Conference on Computer Graphics and Interactive Techniques , SIGGRAPH '87, pages 163-169, 1987.
- Ramon E. Moore, R. Baker Kearfott, and Michael J. Cloud. Introduction to interval analysis . Society for Industrial and Applied Mathematics (SIAM), Philadelphia, PA, 2009.
- Simon Plantinga and Gert Vegter. Isotopic implicit surface meshing. In Proceeding Symposium on Geometry Processing , pages 251-260, 2004.
- Simon Plantinga and Gert Vegter. Isotopic meshing of implicit surfaces. The Visual Computer , 23:45-58, 2007.
- Helmut Ratschek and Jon Rokne. Computer Methods for the Range of Functions. Ellis Horwood Series: Mathematics and Its Applications. Halsted Press, Australia, 1984.
- Josu´ e Tonelli-Cueto and Elias Tsigaridas. Condition numbers for the cube. I: Univariate polynomials and hypersurfaces. In Proceedings of the 45th International Symposium on Symbolic and Algebraic Computation , pages 434-441, 2020.