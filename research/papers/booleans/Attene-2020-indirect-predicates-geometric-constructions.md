## Abstract

Geometric predicates are a basic ingredient to implement a vast range of algorithms in computational geometry. Modern implementations employ floating point filtering techniques to combine efficiency and robustness, and state-of-the-art predicates are guaranteed to be always exact while being only slightly slower than corresponding (inexact) floating point implementations. Unfortunately, if the input to these predicates is an intermediate construction of an algorithm, its floating point representation may be affected by an approximation error, and correctness is no longer guaranteed.

This paper introduces the concept of indirect geometric predicate: instead of taking the intermediate construction as an explicit input, an indirect predicate considers the primitive geometric elements which are combined to produce such a construction. This makes it possible to keep track of the floating point approximation, and thus to exploit efficient filters and expansion arithmetic to exactly resolve the predicate with minimal overhead with respect to a naive floating point implementation. As a representative example, we show how to extend standard predicates to the case of points of intersection of linear elements (i.e. lines and planes) and show that, on classical problems, this approach outperforms state-of-the-art solutions based on lazy exact intermediate representations.

of the predicate, independently of how cleverly it has been implemented. Unfortunately, many useful algorithms require such intermediate constructions, and for these cases state-of-the-art robust solutions mostly rely on exact or lazy exact arithmetic kernels, which are still significantly slower and more memory intensive than floating point implementations.

In this paper, we show that most of these algorithms can be robustly implemented thanks to a novel concept of indirect geometric predicate. In particular, we show that the geometric construction itself can be filtered along with the predicate, thus leading to provably robust algorithms which are comparable with naive floating point implementations in terms of both performances and memory footprint. The reference implementation and the scripts to reproduce the results in the paper are provided in the additional material and will be released as an open-source project.

## 2. Background and prior art

The majority of geometric predicates evaluate the sign of a (homogeneous) polynomial, typically a matrix determinant. As an example, the Cartesian 2D orientation of three points a , b , and c corresponds to the sign of the determinant of the 2 × 2 matrix whose rows are the vectors b -a and c -a . Instead of using exact arithmetic everywhere, which is too slow, modern implementations use arithmetic filtering. These implementations evaluate the polynomial sign using only FP arithmetic, but while keeping track of the approximation error. If this error is large enough to make the sign ambiguous (i.e. when the filter 'fails'), they switch to exact arithmetic. The idea is that the failure rate is low enough to make the impact of exact arithmetic virtually negligible. Several approaches exist to bound the accumulated error. In general, calculating a precise bound is slower but minimizes the probability of filter failures. Conversely, conservatively approximating the bound is faster but less precise.

Keywords: Geometric predicates, Exact arithmetic, Filtering 1. Introduction The study and design of computational geometry algorithms is widely based on the so-called Real Arithmetic Model. According to that model, coordinates, lengths, angles, and many other entities, are all considered real numbers, and the algorithm correctness is based on this assumption. Practical implementations, however, are typically based on floating point (FP) arithmetic, and the aforementioned entities are represented by FP numbers. Floating point operations produce approximated results and, in some cases, the mismatch with the theoretical exact result might put the algorithm in an inconsistent state, cause infinite loops, and lead to crashes [1] [2]. An easy solution to this problem is to replace FP numbers with exact number types, but the consequent slowdown is rarely acceptable for practical applications. Geometric algorithms have the peculiar property that their flow depends on the analysis of geometric entities and their configuration, and geometric predicates are the basic building blocks to perform such an analysis. Broadly speaking, a geometric predicate is a function that maps a set of geometric entities to an element of a (small) set of values. For example, the predicate collinear maps a triplet of points to a value in the set { true , false } , whereas the predicate orient2d maps a triplet of points to a value in the set { left turn , collinear , right turn } . To avoid inconsistencies, it is often essential that each predicate that determines the program flow produces exact results. arXiv:2105.09772v2  [cs.CG]  23 Jan 2025

Arithmetic filtering techniques (see Sect. 2.1) allow implementing predicates so that they are always exact, while being only slightly slower than a corresponding naive floating point implementation [3]. These techniques are appropriate when the predicate parameters are all input values and hence not computed as intermediate constructions by the algorithm. Conversely, if intermediate constructions are used as predicate parameters, an approximation during their computation may corrupt the result

## Indirect predicates for geometric constructions

Marco Attene a

a CNR-IMATI, Via De Marni 6, 16149 Genova, Italy

## 2.1. Static, semi-static and dynamic filtering

Given an arithmetic expression involving a number of variables (e.g. the polynomial of a predicate), its evaluation using FP arithmetic is subject to a roundoff error that depends on the values assumed by the variables. This error can be bounded by a constant threshold that, in turn, can be computed analytically and initialized at compilation time. At runtime, if the magnitude of the evaluated expression is larger than this threshold, its sign is guaranteed to be correct. This approach is known as static filtering [4], and the overhead with respect to a plain FP implementation is due to a simple comparison. Without any assumption on the input, the threshold is typically too large and hence not really useful in practice. Nonetheless, if all the potential input values are known at the beginning of the program, the threshold can be computed once for all and then used to filter all the calls to the predicate.

Since the actual input to a predicate is typically made of points which are close to each other, considering only these points to derive a 'local' threshold leads to tighter error bounds and, hence, to less filter failures. This approach is known as semi-static filtering , and the overhead is due to a few additional operations to compute the threshold at each predicate call [5].

Instead of computing an explicit threshold, interval arithmetic can be used to tightly bound the exact result. Basically, the result of any single operation is rounded by both excess and defect, thus leading to an interval containing the exact result. All the operations accumulate these approximations into growing intervals. If the interval representing the eventual result does not contain the zero, then its sign is defined without ambiguity. This approach is an example of dynamic filtering , and evaluating the predicate in this way takes from three to eight times longer than plain FP calculation [6].

It is often advantageous to use all these approaches in a cascaded multi-stage evaluation. Thus, the first attempt is based on plain FP calculation, and a single additional comparison allows implementing a static filter. If this filter fails, a few more comparisons (e.g. four for the orient2d predicate) allow implementing a semi-static filter. If this filter also fails, the polynomial is reevaluated using interval arithmetic, which is the last attempt. As mentioned above, the first (static) filter makes sense only if a reasonably tight bound can be computed based on the input. In general, the process may safely start from the semi static filter while keeping the overhead reasonably small.

## 2.2. Exact and lazy exact evaluation

When all the filters fail, the predicate expression must be evaluated using exact arithmetic. State-of-the-art libraries allow, for example, to represent intermediate results using rational numbers, thus without any loss of precision [7]. Also, polynomials can be exactly evaluated by using arbitrary precision floating point numbers [8] or so-called floating point expansions [9]. This latter approach is particularly efficient as it fully exploits the floating point hardware. Shewchuck's geometric predicates [10] and Levy's Geogram [11] exploit floating point expansions and are nearly as efficient as plain FP implementations on classical test beds. At the time of writing, Geogram predicates can be considered to represent the state of the art.

When intermediate constructions are used as input to a predicate, current robust solutions are based on lazy exact evaluation [12]. This is the approach adopted in CGAL 's Lazy Exact kernel: any intermediate construction is symbolically represented by the history of all the operations that combine input values to build the construction [13]. Besides keeping track of the operations, the construction is also approximately evaluated using interval arithmetic [6]. Since predicate expressions are also evaluated using interval arithmetic, their parameters can be intermediate constructions whose intervals propagate throughout the predicate's expression evaluation. If the resulting interval contains the zero, it means that the sign is ambiguous, and all the involved symbolic representations are tracked back and explicitly evaluated using exact arithmetic. The expectation is that in most of the cases interval arithmetic is sufficiently precise to reach an unambiguous conclusion, and hence exact evaluation is necessary only in a minority of the cases. Nonetheless, even in the ideal case where exact arithmetic is never called into play, interval arithmetic can be up to eight times slower than FP arithmetic [6], and keeping track of the whole history has its own cost too.

Note that all the mechanisms described so far have been designed to compensate for numerical imprecision. Nonetheless, numerical imprecision is not the only source of ambiguity in geometric computation: many algorithms assume that their input is in so-called general position and, if this is not the case, this precondition needs to be simulated [14]. This is rather orthogonal to compensating for numerical imprecision, and the two techniques should be combined when necessary.

## 3. Geometric algorithm classification

Herewith, we classify geometric algorithms based on the number of steps that require intermediate constructions. There are algorithms that have no such steps, that is, where all the predicates depend directly on input values. Then, there are algorithms having only one such step, where predicates depend either on input values or on intermediate constructions that, in their turn, depend directly on input values. And so on, up to arbitrarily complicated algorithms with cascaded sequences of predicate evaluations and intermediate constructions that mutually depend on each other. Our key observation is that the vast majority of state-of-the-art practical algorithms belong to the first two classes.

If S is a set of 2D points whose coordinates are FP numbers, calculating their Delaunay triangulation amounts to repeatedly evaluate orient2d and incircle predicates whose input are points in S . Thus, this is a typical example of algorithm of the first class. Other examples include calculating convex hulls, Voronoi diagrams, surface interpolation, and many others.

If S is a set of 2D segments that possibly intersect, calculating the constrained Delaunay triangulation of their arrangement is an example of algorithm of the second class. Indeed, the intersection of two segments is an intermediate construction used as input by the orient2d and incircle predicates. Other examples include the calculation of Minkowski sums [15], mesh booleans [16], exact mesh repairing [17], and so on.

Current techniques allow combining robustness and efficiency only for algorithms of the first class thanks to arithmetic filtering. In this paper, we propose a novel approach to move forward and include algorithms of the second class too, thus covering a much wider subset of fundamental geometric algorithms.

## 4. Indirect geometric predicates

The core intuition behind our contribution is the following: if the code to calculate the intermediate construction is included in the predicate itself, arithmetic filtering may be applied to that code as well, and the predicate can be evaluated using FP arithmetic as long as the filter does not fail.

When intermediate constructions are used as parameters for a predicate, they are passed to the predicate in an unevaluated form: in this case, we say that these parameters are implicit and the predicate is indirect . An indirect predicate must accept at least one implicit parameter, but may accept standard explicit parameters too. In the latter case, the parameter has an explicit value and can be assumed to be exact (e.g. it is an input value or the result of an operation that does not introduce approximation errors). Conversely, in the former case the parameter is defined as an unevaluated function of explicit values (e.g. the average of input values). Since most geometric predicates have points as parameters, we extend this terminology to points and say that an explicit point is represented by its explicit coordinates, whereas an implicit point is represented by a geometric construction over explicit points (e.g. by the intersection of two lines, each defined by two explicit points).

The idea is to rewrite the expression of an implicit parameter i as a fraction λ/ d where both the numerator λ and the denominator d are polynomials. Then, any standard predicate can be rewritten by substituting, in its own polynomial Λ , explicit parameters with polynomial fractions representing implicit parameters, thus reducing the whole predicate expression to another polynomial fraction Λ ′ / D ′ . Evaluating the sign of such a fraction amounts to evaluate the sign of both the numerator Λ ′ and denominator D ′ , and both can be done robustly and efficiently through arithmetic filtering.

When an implicit parameter is a point, we derive a common denominator for all the fractions representing its Cartesian coordinates. Hence, an implicit point i = ( i x , i y , i z ) is rewritten as i = ( λ ix / di , λ iy / di , λ iz / di ) or, using homogeneous coordinates, as ( λ ix , λ iy , λ iz , di ).

Note that possible singular / degenerate configurations must be detected and appropriately handled. Typically, this amounts to check whether d becomes zero for each of the implicit input values.

In the following subsections, we show how to apply this transformation to some non-trivial geometric constructions, and how to apply them to fundamental geometric predicates.

## 4.1. Line-line intersections in 2D

Let p be an implicit 2D point defined as the intersection of two lines l a and l b , each defined by two points. Specifically, e a 1 and e a 2 define l a , whereas e b 1 and e b 2 define l b (Fig. 1-left). Therefore:

<!-- formula-not-decoded -->

where:

<!-- formula-not-decoded -->

Note that p is undefined if d = 0, which happens if l a and l b are parallel or if either of the two point pairs is degenerate.

Figure 1: Implicit point examples. Left - line-line intersection: four explicit 2D points define the implicit point p . Right - line-plane intersection: five explicit 3D points define the implicit point p .

<!-- image -->

## 4.2. Line-plane intersections in 3D

Let p be an implicit 3D point defined as the intersection of a straight line L and a plane Π , let L be defined by two points q 1 , q 2 , and let Π be defined by three points r , s , t in the Euclidean space (Fig. 1-right). Therefore:

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

Note that p is undefined if d = 0, which happens if L and Π are parallel, if q 1 = q 2 , or if r , s , t are collinear or coincident.

## 4.3. orient2d

Let p 1 , p 2 , p 3 be three points on the Euclidean plane. The polynomial expression of the classical orient2d predicate is Λ = det | p 2 -p 1 ; p 3 -p 1 | = ( p 2 x -p 1 x )( p 3 y -p 1 y ) -( p 2 y -p 1 y )( p 3 x -p 1 x ). To start with, consider the simple case where just one of the points is an intermediate construction. Let it be p 1 , and let us assume that we are able to rewrite its Cartesian coordinates as polynomial fractions (e.g. as described in Sect. 4.1). Therefore:

<!-- formula-not-decoded -->

By substituting p 1 with ( λ x / d , λ y / d ), we obtain Λ = ( p 2 x -λ x / d )( p 3 y -λ y / d ) -( p 2 y -λ y / d )( p 3 x -λ x / d ) = Λ ′ / D ′ , where Λ ′ = ( dp 2 x -λ x )( dp 3 y -λ y ) -( dp 2 y -λ y )( dp 3 x -λ x ) and D ′ = d 2 . Since the denominator is a square, it can be safely dropped and the sign is unambiguously defined by Λ ′ .

Now, let us consider the most generic case where all the three points are intermediate constructions. We obtain:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Where the λ i and the di are computed as above. By substituting p 1 , p 2 , p 3 with their corresponding fractions, we obtain Λ = ( λ 2 x / d 2 -λ 1 x / d 1 )( λ 3 y / d 3 -λ 1 y / d 1 ) -( λ 2 y / d 2 -λ 1 y / d 1 )( λ 3 x / d 3 -λ 1 x / d 1 ) = Λ ′ / D ′ , where

Λ ′ = ( d 1 λ 2 x -d 2 λ 1 x )( d 1 λ 3 y -d 3 λ 1 y ) -( d 1 λ 2 y -d 2 λ 1 y )( d 1 λ 3 x -d 3 λ 1 x ) D ′ = d 2 1 d 2 d 3 .

## 4.4. incircle

Let p 1 , p 2 , p 3 , p 4 be four points on the Euclidean plane. The polynomial expression of the classical incircle predicate is the following:

<!-- formula-not-decoded -->

If each of the four points is implicitly defined, we can rewrite this expression as:

<!-- formula-not-decoded -->

The common denominator to all the nine elements is D ′ = ( d 1 d 2 d 3 d 4 ) 2 . When none of the d 's is zero, the sign of Λ corresponds to the sign of the following Λ ′ :

<!-- formula-not-decoded -->

## 4.5. orient2d3d

Let Π be an oriented plane identified by three non-aligned points r , s and t in the Euclidean 3D space. Also, let p 1 , p 2 , p 3 be three points on Π . The orient2d3d predicate generalizes the standard orient2d predicate, and states whether p 1 , p 2 , p 3 form a right turn, a left turn, or a straight line on Π . It turns out that orient2d3d ( p 1 , p 2 , p 3 , r , s , t ) = orient2d ( Γ ( p 1 ) , Γ ( p 2 ) , Γ ( p 3 )) ∗ orient2d ( Γ ( r ) , Γ ( s ) , Γ ( t )), where Γ ( p i ) indicates the 2D projection of p i on one of the three axis-aligned planes such that orient2d ( Γ ( r ) , Γ ( s ) , Γ ( t )) , 0. An effective way to select an appropriate projection plane is to consider a normal vector at Π and simply drop the coordinate with largest magnitude. Without loss of generality, from now on we assume that we are dropping the Z coordinate or, equivalently, that we are projecting on the XY plane (i.e. Γ ( &lt; px , py , pz &gt; ) = &lt; px , py &gt; ).

Now, let us suppose that p 1 is implicitly defined (e.g. as described in Sect. 4.2). Therefore:

<!-- formula-not-decoded -->

Since we are projecting on XY, we do not even need to calculate λ z , and simply re-write Λ = ( p 2 x -p 1 x )( p 3 y -p 1 y ) -( p 2 y -p 1 y )( p 3 x -p 1 x ) = ( p 2 x -λ x / d )( p 3 y -λ y / d ) -( p 2 y -λ y / d )( p 3 x -λ x / d ) = Λ ′ / D ′ , where Λ ′ = ( dp 2 x -λ x )( dp 3 y -λ y ) -( dp 2 y -λ y )( dp 3 x -λ x ) and D ′ = d 2 . Hence, orient2d ( Γ ( p 1 ) , Γ ( p 2 ) , Γ ( p 3 )) = sign ( Λ ′ ).

## 4.6. orient3d

Similar arguments hold for the orient3d predicate. When all its input points are explicit, the orient3d polynomial is:

<!-- formula-not-decoded -->

If we replace p 1 with an implicit point we obtain the following expression for an indirect orient3d predicate:

<!-- formula-not-decoded -->

which, if d , 0, has the same sign as the simpler:

<!-- formula-not-decoded -->

where the corresponding D ′ is d .

Formulas for the case where more than one point is implicit can be easily derived using a similar rewriting.

## 5. Computational models and implementation

The input to an indirect predicate is a set of points, each being either explicit or implicit. In the former case, the point is a standard array of Cartesian coordinates, each encoded as a double precision FP number; in the latter case, it is a collection of explicit points to be combined. Upon invocation, an indirect predicate determines which of its input parameters are implicit, and internally calculates λ and d for each of them. In our implementation we consider three computation models: floating point arithmetic (with semi-static filter), interval arithmetic (with dynamic filter), expansion arithmetic (exact). The predicate is evaluated with the fastest model which guarantees exactness.

## 5.1. Filtered evaluation in floating point

We begin by calculating the d 's and, for each of them, use a semi-static filter that states whether its sign is unambiguous. If the filter succeeds for all the d 's, we compute the λ 's, evaluate Λ ′ , and use another semi-static filter to guarantee that its sign is correct. If the former filter fails for one of the d 's, it means that precision is insufficient to exclude that the implicit point is undefined (e.g. intersection of parallel lines). In this case, it makes no sense to proceed with this arithmetic model. As soon as a filter fails, we stop and switch to interval arithmetic (Sect. 5.2). Upon success, the sign of Λ ′ is possibly inverted depending on the sign of D ′ . Weobserve that D ′ is just a product of the d 's, therefore we simply count how many of them have a negative sign (possible multiplicities must be considered). If they are an even number, we return the sign of Λ ′ , otherwise its inverse. Semi-static filters for the floating point model are calculated using a variation of [5] (see Appendix A for details).

## 5.2. Filtered evaluation using intervals

When starting with this arithmetic model, we first set the CPU rounding mode to towards infinity . This is required to correctly and efficiently evaluate the subsequent interval arithmetic block. Then, we calculate the d 's and check that their sign is not ambiguous (i.e. their intervals do not contain the zero). If this dynamic filter succeeds for all the d 's, we compute the λ 's, evaluate Λ ′ , and check that the resulting sign is not ambiguous. As soon as one of the filters fail we stop and switch to exact arithmetic (Sect. 5.3). If all the filters succeed, the sign of Λ ′ is possibly inverted depending on the sign of the d 's. Upon termination (both with success or failure), we reset the rounding to the standard to nearest mode required by the other computational models.

## 5.3. Exact evaluation using expansions

When the predicate is computed using an exact arithmetic model there is a minor difference. Indeed, when computing the d 's it might happen that some of them are exactly zero. If so, the predicate cannot be evaluated because at least one of its implicit parameters is undefined. Conversely, if all the d 's are nonzero, we calculate Λ ′ and check whether it is zero. If so, there is no need to check the d 's and the predicate returns zero. Otherwise, we possibly invert the sign as in the previous models.

Since Λ ′ is a polynomial, we may exactly evaluate its sign using floating point expansions. Roughly speaking, an expansion is an array of FP numbers whose sum is the exact number being represented. Under certain assumptions, FP expansions can be efficiently summed, subtracted, and multiplied, all without error and while exploiting the floating point hardware [9].

## 5.4. Caching

If the same implicit point is used as input to many predicate calls, its λ 's and d are recomputed many times. Therefore, upon the first occurrence it is worth storing their values for future reuse. For the sake of implementing filters, besides these values we must also store information to accumulate error bounds. For semi-static filters, this amounts to store an additional FP number (see Appendix A). For dynamic filters, this amounts to store the λ 's and d using intervals.

Clearly, such a caching mechanism speeds up the computation at the expense of a higher memory usage. Our experiments (see Sect. 7) show that caching λ 's and d 's for floating point and interval arithmetic models produces relevant speed ups while requiring a relatively low increase in memory footprint. Conversely, storing these values for the exact model is not advantageous: indeed, storing expansions for each point requires a significant amount of memory, and the time overhead due to the management of all this additional data is higher than what we save in the few switches which are required in typical cases (Table 3). Note that pre-calculated intervals are also used in CGAL Lazy exact kernel [18], but the use of cached FP filters makes our predicate evaluaton significantly faster (Sect. 7).

## 6. Code generation

An indirect predicate must consider all the possible configurations of its input parameters. For example, the orient2d predicate has three points as parameters, and the type of each of these points can be either implicit or explicit . The most generic implementation may assume that all the points are implicit and, if any of them is explicit, its λ 's can be simply replaced with the Cartesian coordinates and its d with 1. Though this approach is correct, such a naive implementation would lead to unnecessarily loose filters which, in turn, would require unnecessary switches to more precise models. Consequently, this implementation would be unnecessarily slow.

A much more efficient approach is based on the implementation of a specific instance of the predicate for each of the possible input configurations. If the three input points of our indirect orient2d predicate are all explicit, the predicate reduces to the standard orient2d , which is very efficient and accumulates small errors. If only the first point is implicit, we may use a dedicated implementation based on the Λ and D described earlier in Sect. 4.3. This is slightly less efficient than the previous version and produces a slightly higher error. And so on. Hence, an indirect orient2d predicate might be efficiently implemented using eight instances (EEE, EEI, EIE, EII, IEE, IEI, IIE, III, where the three characters indicate the type of each input point). Actually, many of these instances can be reduced to each other by transposing their input parameters and possibly inverting the resulting sign (e.g. orient2d (a, b, c) = orient2d (b, c, a) = -orient2d (a, c, b)). It is easy to verify that four versions (EEE, IEE, IIE, III) of orient2d are sufficient to cover all the cases.

Implementing even a single instance with proper semi-static and dynamic filtering, and with expansion-based evaluation, is a tedious and error-prone process. Fortunately, this process can be mostly automated [19]. If the predicate is naively implemented as a C function using FP arithmetic, software tools exist [5] to parse such a function and produce another C function which includes a semi-static filter. These tools analyze the polynomial and calculate an upper bound for the roundoff error. In the produced code, the magnitude of the resulting determinant is compared with such a bound and, if smaller, the predicate returns a particular value indicating that the filter failed (e.g. FPG UNCERTAIN ). Meyer and Pion's tool has been used and extended by Levy in the Predicate Construction Kit [11], where both the semi-static filtered and the exact versions of the predicate are produced.

We have further extended upon this approach, and have implemented a software that parses a text-based formula representing the polynomial. Starting from such a formula, our tool produces three C ++ functions, each implementing the formula using one of the arithmetic models described earlier. A further fourth function is generated that calls the other three in order and terminates as soon as one of them returns a guaranteed result.

Table 1: Results of our experiments 1.1 and 2.1. Elapsed time in seconds and, in parenthesis, peak memory usage in Mbytes.

|   N. Points | Exp. 1.1         | Exp. 2.1         |
|-------------|------------------|------------------|
|        1000 | 0.000691 (2.74)  | 0.000862 (2.77)  |
|       10000 | 0.008698 (6.29)  | 0.008864 (6.49)  |
|      100000 | 0.117321 (27.99) | 0.12001 (25.56)  |
|     1000000 | 1.79624 (268.53) | 1.82953 (284.55) |

## 7. Experiments

For the sake of experimentation, we have implemented a C ++ library with an API defining a genericPoint class. A genericPoint can be specialized to be either 2D or 3D, implicit or explicit. The library also provides functions implementing both standard and indirect predicates. Each such function takes a number of genericPoint objects as input, internally determines their type, and properly handles all the cases as described in Sect. 6. The library has been used as a basis to implement a simple Delaunay triangulation algorithm that can deal with genericPoint objects. Both the library and the triangulation algorithm have been compiled and tested on an MS Windows 10 OS using Visual C ++ 2019. The objectives are measuring the efficiency of our predicates, the overhead of indirect predicates when compared to their direct versions, the impact of all the possible caching levels, and the improvement with respect to the state of the art. Tests were run on a standard PC equipped with an Intel Core i7-4770 with 16 Gb RAM. Based on this setting, the following experiments were run. Results are summarized in Tables 1 and 2.

## 7.1. Experiment 1.1 - Random explicit 2D points

For this experiment, we created and triangulated a set of genericPoint objects, each specialized to represent an explicit 2D point. Their Cartesian coordinates are randomly sampled within the unit square [0 , 1] × [0 , 1]. Since all the points are explicit, the orient2d and incircle predicates used by the algorithm always switch to the standard direct version of the predicates. A parameter allows controlling the cardinality of the point set in the range [1, 1000000]. The objective of this test is to define a reference for the other tests, in order to evaluate the overhead due to the use of indirect predicates when compared with their classical direct version.

## 7.2. Experiment 1.2 - Random mixed 2D points

Herewith, our set of genericPoint objects still contains 2D points, but their type is mixed. A subset of them are explicit, whereas the others are implicitly defined as the intersection of line pairs. Explicit points are randomly scattered within the unit square as in the previous experiment. Also, line pairs are created so that their intersections are randomly scattered within the unit square. A parameter allows controlling the percentage of implicit points over a total of 1 million points constituting the entire set. In this case, both orient2d and incircle may either switch to the direct version or select the proper indirect predicate instance, depending on the type of the genericPoint s.

## 7.3. Experiment 1.3 - Regular mixed 2D points

This dataset of 1000000 genericPoint objects contains mixed 2D points as in Experiment 1.2 but, instead of being randomly scattered, they form a regular grid within the unit square. A subset of them are explicit, whereas the others are implicitly defined as the intersection of line pairs. Even in this case, a parameter allows controlling the percentage of implicit points over the total. This dataset stresses the predicates that are largely required to switch to exact arithmetic.

## 7.4. Experiment 2.1 - Random explicit 3D points

For this experiment, we created a set of genericPoint objects, each specialized to represent an explicit 3D point whose Cartesian coordinates are randomly sampled within the unit cube [0 , 1] × [0 , 1] × [0 , 1]. The Delaunay triangulation is calculated on the projection of these points on the XY plane. As for Experiment 1.1, the objective is to define a reference for the following Experiments 2.2 and 2.3.

## 7.5. Experiment 2.2 - Random mixed 3D points

Here, our set of genericPoint objects contains 3D points, both explicit and implicit, and the Delaunay triangulation is calculated on their projection on the XY plane. As in the previous experiment, the first half of the set is made of explicit points, whereas the other half are implicitly defined as the intersection of lines and planes. Explicit points are randomly scattered within the unit cube [0 , 1] × [0 , 1] × [0 , 1]. Line-plane pairs are created so that their intersections belong to the unit cube. A parameter allows controlling the percentage of implicit points over a total of 1 million points constituting the entire set.

## 7.6. Experiment 2.3 - Regular mixed 3D points

This dataset of 1000000 genericPoint objects contains mixed 3D points as in Experiment 2.2 but, instead of being randomly scattered, their projections on the XY plane form a regular grid within the unit square. Their Z coordinate is still randomnly selected in the interval [0,1]. A subset of the points are explicit, whereas the others are implicitly defined as the intersection of properly configured line-plane pairs. Even in this case, a parameter allows controlling the percentage of implicit points over the total. This dataset stresses the predicates that are largely required to switch to exact arithmetic.

## 7.7. Comparison with state of the art

CGAL [18] is probably one of the most used libraries when robustness and efficiency must be combined, in particular when important fundamental algorithms come into play (e.g. Delaunay triangulations). That is why we compare our results with CGAL . In our comparison, we always strive to exploit CGAL in its most efficient configuration while providing a guarantee of correctness. Namely, we want to guarantee that the resulting Delaunay triangulation has the correct topology.

We re-run all our tests using CGAL Delaunay triangulation algorithm. For experiments 1.1 and 2.1, where all the points are explicit, we configure CGAL to use the Exact Predicates Inexact Constructions kernel, whereas for the others we need exact constructions to represent the intersections. Implicit points are generated using CGAL::intersection() . For experiments that require projection on the XY plane, CGAL::Projection traits xy 3 was used. We assume that CGAL implements efficient state-of-the-art algorithms but, since using two different implementations may introduce a bias independently of our contribution, we have also re-implemented our own triangulation algorithm using CGAL 's exact number type CGAL::Lazy exact nt&lt;CGAL::Gmpq&gt; and its predicates CGAL::orientation() and CGAL::side of oriented circle() . However, after having verified that this re-implementation is slower than CGAL 's native algorithm, we have abandoned it. Tables 4 and 5 report the results of our experiments using CGAL .

|   %implicit | Exp. 1.2         | Exp. 1.3         | Exp. 2.2         | Exp. 2.3         |
|-------------|------------------|------------------|------------------|------------------|
|           0 | 1.79624 (268.53) | 2.63308 (268.36) | 1.82953 (284.55) | 2.64815 (284.37) |
|          10 | 2.23134 (292.84) | 5.0791 (292.99)  | 2.56663 (321.96) | 7.14617 (322.95) |
|          25 | 2.82864 (328.84) | 8.45898 (329.3)  | 3.41211 (377.43) | 16.6439 (379.75) |
|          50 | 3.62846 (389.14) | 14.0788 (390.61) | 4.24588 (469.68) | 38.3586 (472.40) |
|         100 | 3.81999 (509.93) | 27.1925 (511.29) | 4.29742 (654.52) | 75.0639 (657.06) |

Table 2: Results of our experiments 1.2, 1.3, 2.2 and 2.3. Elapsed time in seconds and, in parenthesis, peak memory usage in Mbytes.

Table 3: Results of our experiment 1.2 using different caching approaches. λ 's and d were cached only for the FP model (column FP ), for both the FP and interval arithmetic models (column Interval ), or for all the available models (column Exact ). Elapsed time in seconds and, in parenthesis, peak memory usage in Mbytes.

|   %implicit | no cache         | FP               | Interval         | Exact            |
|-------------|------------------|------------------|------------------|------------------|
|           0 | 1.79552 (268.51) | 1.78396 (268.53) | 1.79624 (268.53) | 1.82953 (268.53) |
|          10 | 2.49302 (283.07) | 2.4852 (286.20)  | 2.23134 (292.84) | 2.27749 (358.66) |
|          25 | 3.5918 (304.77)  | 3.39718 (312.93) | 2.82864 (328.84) | 2.86765 (494.35) |
|          50 | 5.25428 (341.23) | 5.00143 (357.23) | 3.62846 (389.14) | 3.79824 614.96)  |
|         100 | 7.16629 (413.65) | 6.9816 (445.84)  | 3.81999 (509.93) | 3.9589 (1167.84) |

Table 4: Results of our experiments 1.1 and 2.1 using CGAL with exact predicates and inexact constructions kernel. Elapsed time in seconds and, in parenthesis, peak memory usage in Mbytes.

|   N. Points | Exp. 1.1          | Exp. 2.1         |
|-------------|-------------------|------------------|
|        1000 | 0.000253 (3.52)   | 0.000273 (3.29)  |
|       10000 | 0.015636 (4.68)   | 0.015622 (4.72)  |
|      100000 | 0.098743 (16.64)  | 0.109367 (19.62) |
|     1000000 | 1.168691 (135.81) | 1.15617 (167.65) |

## 7.8. Results discussion

In all the experiments, the total number of predicate calls depends on the input size. For Experiment 1.1, these numbers are as follows: 1000 points ⇒ 4395 calls to orient2d and 7502 calls to incircle ; 10K points ⇒ 45K and 94K calls; 100K points ⇒ 469K and 1.14M calls; 1M points ⇒ 4.8M and 13.3M calls. These numbers do not vary significantly across different experiments with same input size.

Tables 1 and 2 show that the impact of having implicit points in the input is rather limited. The maximum increase in both time and memory usage is around 2x in 2D and 2.5x in 3D, which happens when all the points are implicit. Most importantly, the resource demand gradually increases as the relative amount of implicit points in the set grows. Hence, our approach is particularly convenient in cases where implicit points are sparse. Experiments 1.3 and 2.3 show that our predicates behave reasonably well even in extremely degenerate cases where exact arithmetic is used extensively. In particular, we observe that the peak memory usage in these cases is virtually equivalent to that of corresponding non-degenerate input sets. The impact on the elapsed time is of course more significant but, even in this case, we observe that it gracefully grows as the percentage of implicit points increases. Table 3 shows the impact of caching the λ 's and d 's: when these values are stored for the FP model only the overall gain in speed is modest, but when the Interval model is included the algorithm becomes nearly twice as fast with only 23% more memory requirements.

Our results also demonstrate that indirect predicates outperform state of the art methods with same guarantees. Even if CGAL is faster when inexact constructions are allowed (Experiments 1.1 and 2.1, see Tables 1 and 4), its lazy exact kernel becomes necessary to represent implicit points and, in this case, our implementation is from 3.86 to 11.7 times faster than CGAL , while requiring up to 3.5 times less memory (see Tables 2 and 5 and Fig. 2).

## 8. Conclusions

We have shown that, even if intermediate geometric constructions are necessary, the efficiency of floating point hardware can be exploited to quickly evaluate geometric predicates with guarantees of correctness. This partly solves a problem introduced by Meyer and Pion [5] more than ten years ago. The problem is only partly solved because our approach requires that intermediate constructions can be represented as polynomial fractions. Though this is sufficient for most of the algorithms of the second type introduced in Sect. 3, there are still a few useful algorithms that do not satisfy this requirement. One noticeable example is the constrained Delaunay triangulation in 3D, where state of the art algorithms use irrational numbers to generate Steiner points [20]. Furthermore, more numerically complex techniques exist which involve cascaded predicates and constructions, and hence do not belong to our second class of algorithms (e.g. [21]). Unfortunately, indirect predicates do not help in these cases, and attempting to generalize their paradigm is a challenging research direction: indeed, the combination of cascaded constructions would easily lead to rather complicated expressions whose filters become too large to be useful in practice. Nonetheless, indirect predicates enable much faster implementations for a wide range of useful algorithms of the second class.

We also observe that indirect predicates allow implementing algorithms with guarantees of robustness and combinatorial correctness. Nonetheless, if their results are the input to other algorithms which expect floating point coordinates, implicit points must be evaluated and their coordinates approximated. Such an approximation may lead to invalid configurations (e.g. with self intersections) even if the combinatorics is exact. This problem is known as snap rounding and, though effective algorithms exist for the 2D case, no practical guaranteed implementation exists yet for the 3D case. This problem is worth a further investigation [22].

Table 5: Results of our experiments 1.2, 1.3, 2.2 and 2.3 using CGAL with exact predicates and exact constructions kernel. Elapsed time in seconds and, in parenthesis, peak memory usage in Mbytes.

|   %implicit | Exp. 1.2          | Exp. 1.3          | Exp. 2.2          | Exp. 2.3          |
|-------------|-------------------|-------------------|-------------------|-------------------|
|           0 | 6.92141 (168.86)  | 26.4514 (362.26)  | 33.2009 (193.40)  | 68.7454 (483.43)  |
|          10 | 8.09864 (252.48)  | 27.9512 (492.32)  | 35.0445 (338.69)  | 76.5043 (716.04)  |
|          25 | 9.76497 (377.75)  | 32.0291 (698.53)  | 38.7077 (553.73)  | 88.9425 (1060.50) |
|          50 | 11.9211 (586.53)  | 40.966 (1040.29)  | 42.2784 (908.59)  | 111.112 (1634.54) |
|         100 | 15.9442 (1006.73) | 60.9706 (1682.76) | 50.1841 (1605.84) | crashed           |

Figure 2: Elapsed time for experiments 1.2, 1.3, 2.2 and 2.3 using indirect predicates (blue) and CGAL (orange).

<!-- image -->

## Acknowledgements

This work is partly supported by the EU ERC Advanced Grant CHANGE, agreement No 694515. Thanks are due to the colleagues at CNR-IMATI for helpful discussions.

## References

- [1] L. Kettner, K. Mehlhorn, S. Pion, S. Schirra, C. Yap, Classroom examples of robustness problems in geometric computations, in: S. Albers, T. Radzik (Eds.), Algorithms - ESA 2004, Springer Berlin Heidelberg, Berlin, Heidelberg, 2004, pp. 702-713.
- [2] C. Li, S. Pion, C. Yap, Recent progress in exact geometric computation, The Journal of Logic and Algebraic Programming 64 (1) (2005) 85 - 111, practical development of exact real number computation. doi:https:// doi.org/10.1016/j.jlap.2004.07.006 .
- [3] O. Devillers, F. P. Preparata, A probabilistic analysis of the power of arithmetic filters, Discrete &amp; Computational Geometry 20 (4) (1998) 523-547. doi:10.1007/PL00009400 .
- [4] S. Fortune, C. J. Van Wyk, Static analysis yields efficient exact integer arithmetic for computational geometry, ACM Trans. Graph. 15 (3) (1996) 223248. doi:10.1145/231731.231735 .
- [5] A. Meyer, S. Pion, FPG: A code generator for fast and certified geometric predicates, in: Real Numbers and Computers, 2008, pp. 47-60.
- [6] H. Br¨ onnimann, C. Burnikel, S. Pion, Interval arithmetic yields efficient dynamic filters for computational geometry, in: Proceedings of the Fourteenth Annual Symposium on Computational Geometry, SCG '98, ACM, New York, NY, USA, 1998, pp. 165-174. doi:10.1145/276884.276903 .
- [7] S. Fortune, C. J. Van Wyk, Efficient exact arithmetic for computational geometry, in: Proceedings of the Ninth Annual Symposium on Computational Geometry, SCG '93, ACM, New York, NY, USA, 1993, pp. 163-172. doi:10.1145/160985.161015 .
- [8] L. Fousse, G. Hanrot, V. Lef` evre, P. P´ elissier, P. Zimmermann, Mpfr: A multiple-precision binary floating-point library with correct rounding, ACM Trans. Math. Softw. 33 (2). doi:10.1145/1236463.1236468 .
- [9] M. Joldes, O. Marty, J.-M. Muller, V. Popescu, Arithmetic algorithms for extended precision using floating-point expansions, IEEE TRANSACTIONS ON COMPUTERS 65 (4) (2016) 1197-1210.
- [10] J. R. Shewchuk, Adaptive precision floating-point arithmetic and fast robust geometric predicates, Discrete &amp; Computational Geometry 18 (3) (1997) 305-363.
- [11] B. L´ evy, Robustness and efficiency of geometric programs, Comput. Aided Des. 72 (C) (2016) 3-12. doi:10.1016/j.cad.2015.10.004 .
- [12] S. Pion, A. Fabri, A generic lazy evaluation scheme for exact geometric computations, Science of Computer Programming 76 (4) (2011) 307 - 323, special issue on library-centric software design (LCSD 2006). doi:https: //doi.org/10.1016/j.scico.2010.09.003 .
- [13] S. Schirra, On the use of adaptive, exact decisions number types based on expression-dags in geometric computing, in: Proceedings of the 26th Canadian Conference on Computational Geometry, CCCG '14, 2014.
- [14] H. Edelsbrunner, E. P. M¨ ucke, Simulation of simplicity: A technique to cope with degenerate cases in geometric algorithms, ACM Trans. Graph. 9 (1) (1990) 66-104. doi:10.1145/77635.77639 . URL https://doi.org/10.1145/77635.77639
- [15] P. Hachenberger, Exact minkowksi sums of polyhedra and exact and efficient decomposition of polyhedra into convex pieces, Algorithmica 55 (2) (2009) 329-345. doi:10.1007/s00453-008-9219-6 .
- [16] Q. Zhou, E. Grinspun, D. Zorin, A. Jacobson, Mesh arrangements for solid geometry, ACM Trans. Graph. 35 (4) (2016) 39:1-39:15. doi:10.1145/ 2897824.2925901 .
- [17] M. Attene, As-exact-as-possible repair of unprintable stl files, Rapid Prototyping Journal 24 (5) (2018) 855-864. doi: 10.1108/RPJ-11-2016-0185 .
- [18] The CGAL Project, CGAL User and Reference Manual, 5.0 Edition, CGAL Editorial Board, 2019.

URL

[https://doc.cgal.org/5.0/Manual/packages.html](https://doc.cgal.org/5.0/Manual/packages.html)

- [19] A. Nanevski, G. Blelloch, R. Harper, Automatic generation of staged geometric predicates, Higher-Order and Symbolic Computation 16 (4) (2003) 379-400. doi:10.1023/A:1025876920522 .
- [20] H. Si, Tetgen, a delaunay-based quality tetrahedral mesh generator, ACM Trans. Math. Softw. 41 (2) (2015) 11:1-11:36. doi:10.1145/2629697 .
- [21] F. Cazals, A. Parameswaran, S. Pion, Robust construction of the threedimensional flow complex, in: Proceedings of the Twenty-Fourth Annual Symposium on Computational Geometry, SCG '08, Association for Computing Machinery, New York, NY, USA, 2008, p. 182-191. doi: 10.1145/1377676.1377705 .

[URL https://doi.org/10.1145/1377676.1377705](https://doi.org/10.1145/1377676.1377705)

- [22] O. Devillers, S. Lazard, W. J. Lenhart, 3D Snap Rounding, in: B. Speckmann, C. D. T´ oth (Eds.), 34th International Symposium on Computational Geometry (SoCG 2018), Vol. 99 of Leibniz International Proceedings in Informatics (LIPIcs), Schloss Dagstuhl-Leibniz-Zentrum fuer Informatik, Dagstuhl, Germany, 2018, pp. 30:1-30:14. doi:10.4230/LIPIcs.SoCG. 2018.30 .

## A. Semi-static filter calculation

A semi-static filter needs to (1) calculate a threshold value as a function of input parameters vi and (2) verify that | Λ ′ | is larger than this value. In our code, the threshold ϵ ( v 1 , ..., vn ) is calculated as a product δ (1) B ( v 1 , ..., vn ), where δ (1) is a constant value which depends only on the predicate's expression and is computed at compile time as described in [5]. Conversely, B ( v 1 , ..., vn ) depends on the input values vi and must be calculated at each predicate call. This dynamic part of the filter is the product of k factors b 1 b 2 . . . bk , where bj is the absolute value of either one of the input variables vi or a difference va -vb between two of them. Note that factors need not be different from each other (e.g. bi and bj may be the same for i , j ). Differently from [5], we slightly overestimate B ( v 1 , ..., vn ) for the sake of efficient caching. Specifically, we replace the product b 1 b 2 . . . bk with the power β k , where β = argmax { b 1 , . . . , bk } . The reason for this is the following.

Let P ( v 1 , . . . , vk , i 1 , . . . , i n ) be a predicate where the vi 's are explicit parameters whereas the i j 's are implicit. Any of the implicit parameters i is defined as a composition of explicit values w 1 . . . wm . Upon evaluation using the FP model, we first use the w 's to calculate the λ 's and d 's, and then use these λ 's and d 's together with the explicit parameters v 's to evaluate the whole predicate. The semi-static filter for this composed calculation must take all the variables into account. Thus, the dynamic component of our filter is β k , where β is the maximum of the bi 's deriving from explicit variables v and the bj 's deriving from the w 's. The maximum over the bj 's is computed only once for each implicit point and cached for later use. Hence, replacing the product with the power allows using a single variable to cache the bounds for any λ and d . When compared with the caching benefits, the performance loss due to our overestimation is virtually negligible. Fig. 3 shows an example of the code produced by our tool.

```
int orient2d_indirect_SEE_filtered(double l1x, double lly, double d1, double p2x,double p2y,double p3x,double p3y,double prev_maxval) double a = d1 * p2x; double c = d1 * p3y; double e = d1 * p2y; double g = d1 * p3x; double ab =a -l1x; double cd = c - 11y; double ef = e - lly; double gh = g -l1x; double abcd =ab *cd; double efgh = ef * gh; double L = abcd - efgh; double_tmp_fabs,max_var =prev_maxval; if ((_tmp_fabs = fabs(p2x)) > max_var) max_var = _tmp_fabs; if ((_tmp_fabs = fabs(p2y)) > max_var) max_var =_tmp_fabs; if ((_tmp_fabs = fabs(p3x)) > max_var) max_var = _tmp_fabs; if ((_tmp_fabs = fabs(p3y)) > max_var) max_var = _tmp_fabs; double epsilon = 1.048458195263004e-13 * max_var * max_var * max_var * max_var * max_var * max_var; if (L > epsilon) return IP_Sign::POSITIVE; if（-L>epsilon)return IP_Sign::NEGATIVE; return Filtered_Sign::UNCERTAIN;
```

Figure 3: Code produced by our tool for an indirect version of the orient2d predicate. The first of the three points is implicitly defined by the intersection of two lines that, in turn, are defined by explicit points w 1 , . . . , w 4. The values for λ x , λ y and d ( l1x , l1y , d1 ) are calculated externally and passed to this function along with the maximum β deriving from the w 's ( prev maxval ). The constant value of δ (1) ( = 1.048458195263004e-13 ) accumulates the error bounds of this function and those of the external function that calculates the λ 's and d .