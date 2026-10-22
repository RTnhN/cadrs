## An algorithmic approach to direct spline products: procedures and computational aspects

Francesco Patrizi a, ∗ , Alessandra Sestini b

a Department of Mathematics, University of Rome Tor Vergata, Via della Ricerca Scientifica 1, Rome, 00133, Italy

b Department of Mathematics and Computer Science "Ulisse Dini", University of Florence, Viale Giovanni Battista Morgagni 67/a, Florence, 50134, Italy

## Abstract

We introduce an efficient algorithmic procedure for implementing the direct formula that represents the product of splines in the B-spline basis. We first demonstrate the relevance of this direct approach through numerical evidence showing that implicit methods, such as collocation, may fail in some instances due to severe ill-conditioning of the associated system matrices, whereas the direct formula remains robust. We then recast the direct formula into an algorithmic framework based on the Oslo Algorithm and subsequently enhance it, through a factorization of the terms to be computed, to dramatically improve computational efficiency. Extensive numerical experiments illustrate the substantial reduction in computational cost achieved by the proposed method. Implementation aspects are also discussed to ensure numerical stability and applicability.

Keywords: spline product, knot insertion, collocation, Oslo Algorithm

## 1. Introduction

In this work we present a simple yet effective algorithmic procedure for the direct multiplication of splines. Specifically, we develop an efficient implementation of the direct formula introduced by Mørken (1991) for computing the B-spline coefficients of the product of two splines. Despite its theoretical relevance, the formula of Mørken (1991) has seen limited practical use, largely due to its cumbersome combinatorial structure and the associated computational costs. As a result, research and applications have often favored alternative approaches, either direct, such as those based on blossoming techniques, as in Chen et al. (2009); Lee (1994); Ueda (1994), or implicit, relying on interpolation or basis transformations, as in Piegl and Tiller (1997).

Our contribution revives and significantly enhances the direct approach of Mørken (1991). We recast the original formula into a clear and accessible algorithmic procedure whose core relies on the Oslo Algorithm, developed in Cohen et al. (1980); Lyche and Mørken (1986); Lyche (1988). Building on this formulation, we introduce an improved procedure that dramatically reduces the computational complexity of the direct method. The computation of each B-spline coefficient is reduced to a sequence of matrix-vector products involving rectangular bidiagonal matrices of decreasing size. This structure makes the method transparent, easy to implement, and readily integrable into existing spline-based codes, in contrast to more involved blossoming-based techniques. Compared to indirect approaches, such as interpolation or change-of-basis methods, the proposed direct procedure is also considerably more numerically stable. In particular, for higher spline degrees, indirect methods often suffer from severe ill-conditioning, leading to substantial loss of accuracy or even numerical failure. Our numerical experiments demonstrate that the direct approach, when implemented with the proposed algorithm, remains robust and yields significantly more accurate results across a wide range of settings.

∗ corresponding author email: patrizi@mat.uniroma2.it

Beyond its intrinsic theoretical interest, spline multiplication plays a central role in many applications. It is a fundamental building block in the construction of Gram (mass) matrices for L 2 projections onto spline spaces and in the assembly of system matrices for Galerkin discretizations, particularly within the framework of isogeometric analysis (IgA, Hughes et al. (2005)) and spline-based boundary element methods (IgA-BEM). In IgA, while many applications employ moderate spline degrees, higher-degree regimes are also relevant due to p - and k -refinement strategies, where increasing the degree provides faster convergence than mesh refinement (see, e.g., the estimates in Sande et al. (2019); Manni et al. (2023)) and is routinely used to achieve high accuracy with few degrees of freedom, as in Cottrell et al. (2007); Beirão da Veiga et al. (2011); Sangalli and Tani (2018). Furthermore, under p -refinement, the collocation approach already loses 2 -3 digits of accuracy for moderate degrees ( p ≤ 10 ), independently of mesh fineness. This loss can be significant in high-accuracy simulations and may be further amplified by subsequent ill-conditioned steps, such as the solution of the resulting Galerkin systems. In this context, the proposed direct approach enhances quadrature schemes and assembly pipelines. Moreover, once the product spline is represented directly in the B-spline basis, integration reduces to the exact integration of B-splines, for which closed-form formulas are available, see, e.g., Lyche et al. (2018, Theorem 5). This completely avoids element-wise integration and the use of specialized Gaussian quadrature rules, yielding global (support-wise), exact, and robust integration procedures within spline-based discretization frameworks. Spline products are also essential in operations involving rational B-splines (NURBS), such as addition, as well as in various geometric and graphical computations, including distance evaluation between spline curves. Moreover, classical spline procedures such as degree elevation can be interpreted as special cases of spline products.

The presentation of the paper is limited to the univariate setting only for clarity. However, the proposed approach extends straightforwardly to multivariate tensor-product spline spaces, thanks to the locality of the algorithm and the separable structure of tensor-product bases. Extensions to multivariate NURBSare likewise immediate, since spline products constitute the core operation in rational arithmetic.

The remainder of the paper is organized as follows. In Section 2, we illustrate the importance of direct spline multiplication in practical applications by comparing the direct approach of Mørken (1991), implemented with our algorithm, to implicit interpolation-based methods. The results reveal accuracy gaps of several orders of magnitude between direct and implicit approaches, with implicit approaches often becoming numerically unreliable. In Section 3, we recall the Oslo Algorithm and the direct formula of Mørken (1991), and reformulate the latter into an algorithmic framework. We analyze the computational challenges of a naive implementation and introduce an improved procedure based on a suitable factorization of the terms, together with additional implementation details to ensure numerical stability. Section 4 is devoted to numerical experiments, where we compare the naive and improved implementations of the direct formula and demonstrate the dramatic gains in computational efficiency. Finally, Section 5 concludes the paper and outlines possible directions for future work.

## 2. Motivation: a numerical study of stability and correctness

In this section we motivate, through numerical experiments, the use of a direct formula for spline products. We show that the standard collocation (interpolation) approach can be unreliable in relevant cases. The issue is the large condition number of the associated matrix, which leads to instability and loss of accuracy. We also show that the improved direct formula of Mørken (1991), described in the next section, provides significantly more accurate results.

We assume familiarity with B-splines and their basic properties. For a brief introduction, see Lyche et al. (2018); Manni and Speleers (2016). More comprehensive treatments can be found in de Boor (1978); Schumaker (2007).

Let p 1 , p 2 be two degrees. Let τ τ 1 = ( τ 1 j ) n 1 + p 1 +1 j =1 and τ τ 2 = ( τ 2 j ) n 2 + p 2 +1 j =1 be two knot vectors, assumed open with respect to p 1 and p 2 (that is, with first and last knots of multiplicity p j + 1 , for j = 1 , 2 , respectively). Denote by S p 1 τ 1 and S p 2 τ 2 the corresponding spline spaces. Then dim S p j τ j = n j for j = 1 , 2 .

Let f ∈ S p 1 τ 1 and g ∈ S p 2 τ 2 . Their product h := f · g belongs to S p t , where p = p 1 + p 2 and t = ( t j ) m + p +1 j =1 is open with respect to p . The breakpoints of t are the distinct values in τ τ 1 ∪ τ τ 2 . The knot multiplicities are chosen to preserve the lowest smoothness of the factors:

<!-- formula-not-decoded -->

where µ j ( t i ) is the multiplicity of t i in τ τ j .

The goal is to determine the coefficients b := ( b i ) m i =1 such that

<!-- formula-not-decoded -->

A widespread approach is collocation. This is an implicit method, since it requires solving a linear system. It is often used because of its simplicity. The idea is to interpolate the values of f · g . Since dim S p t = m , we impose

<!-- formula-not-decoded -->

at m distinct nodes { x i } m i =1 . This yields the linear system C b = h , where C is the B-spline collocation matrix

<!-- formula-not-decoded -->

Such a matrix is nonsingular if and only if the nesting conditions of Schoenberg and Whitney (1953) hold, i.e., B i, t ( x i ) &gt; 0 for all i . In that case C is in fact totally positive. A common choice to verify the conditions is to take the Greville abscissae as nodes:

<!-- formula-not-decoded -->

With a standard ordering, C is banded with bandwidth p +1 . The cost of LU factorization is therefore 2 3 mp 2 .

Alternative implicit approaches use Bézier extraction, see Piegl and Tiller (1997). These involve knot insertion, Bernstein multiplication, and knot removal. However, conversions to Bernstein form are ill-conditioned processes, see Farouki (2012, Section 6.5). Direct approaches based on blossoming have also been proposed, see Ueda (1994); Lee (1994). These are equivalent to the formula of Mørken (1991), but are more involved.

However, in the remainder of this section we show, through numerical tests, that direct formulas are sometimes necessary. We compare the accuracy achieved with our enhanced version of the direct approach of Mørken (1991) and collocation. In the tests, splines are defined on open knot vectors with simple internal knots, unless stated otherwise (as in, e.g., Section 2.2). We use uniform knots. Experiments with non-uniform knots, including highly clustered ones, showed no qualitative differences and are omitted. Errors are measured using a discrete relative L ∞ norm, defined as follows. Let X := { x j } N j =1 be a set of uniform points. For the direct method (denoted by M ) and collocation (denoted by C ), we define

Figure 1: Error comparison and condition number of C for the product of a fixed cubic B-spline and a polynomial of increasing degree. (a) max X e M and max X e C vs. polynomial degree. (b) Condition number of C .

<!-- image -->

<!-- formula-not-decoded -->

In all tests we fix N = 201 . For each test, we present the errors max X e M and max X e C under varying degrees or mesh sizes, and the estimated condition number of the matrix C , using the command condest of Matlab. Across all cases, e M remains at or near machine precision (between 1e-16 and 1e-14 ), even for high polynomial degrees. In contrast, e C deteriorates, often rapidly, with increasing degree, in line with the growth of its condition number, and may lead to numerical breakdown.

## 2.1. Product of a fixed B-spline, or spline, and polynomials of increasing degrees

In this test we consider the product of a fixed spline with polynomials of increasing degree, a setting arising, e.g., in B-spline weighted quadrature in IgA-BEM, where spline basis functions are multiplied by polynomial approximations of boundary kernels (see Bracco et al. (2026)). In such contexts, p -refinement on the polynomial factor improves accuracy, and spline-polynomial products are required prior to integration when assembling the discrete system.

We fix f as a B-spline of degree p 1 = 3 on [0 , 1] and let g be a polynomial on [0 , 1] with random coefficients in [ -1 , 1] and degree p 2 ∈ { 1 , . . . , 50 } . Results are shown in Figure 1 (a)-(b). We also consider f as a generic spline of degree p 1 = 3 on [0 , 1] and random coefficients in [ -1 , 1] , with g as above. The results are consistent with the single B-spline case.

## 2.2. The Galerkin isogeometric case with p - and k -refinements

In this test we consider the product of two B-splines defined on the same uniform knot vector τ τ on [0 , 1] , with equal degree p 1 = p 2 ∈ { 3 , . . . , 50 } . As the degree increases, knot multiplicities are adjusted to preserve global C 2 smoothness, as in standard p -refinement procedures for Galerkin discretizations (see Hughes et al. (2005); Cottrell et al. (2007)).

Figure 2: Error comparison and collocation condition number for B i, τ · B j, τ on a common knot vector τ with fixed smoothness ( C 2 ) and increasing degree. For fixed i , mean of max X e j M and max X e C j vs. polynomial degree (a) and mean condition number of C j (b), over all j such that B i and B j have overlapping supports.

<!-- image -->

̸

For each degree, we fix f = B i, τ and consider all B-splines B j, τ with overlapping support, i.e., supp B i, τ ∩ supp B j, τ = ∅ . The reported error is the average of the relative maximal errors over all such pairs. Results are shown in Figure 2 (a)-(b). We also examine the k -refinement regime, where smoothness increases to C p j -1 with the degree. This represents the most challenging case for collocation. Results are reported in Figure 3 (a)-(b).

## 2.3. Product of splines on same knot vector and same degree

As a further test, we consider the product of two splines defined on the same knot vector τ τ on [0 , 1] , with equal degree p 1 = p 2 ∈ { 1 , . . . , 50 } . This generalizes the previous setting (Galerkin case with k -refinement) by replacing B-splines with generic splines. We take τ τ with 5 uniform breakpoints, and define f and g as splines with random coefficients in [ -1 , 1] . Results are reported in Figure 4 (a)-(b).

## 2.4. Product of a fixed spline with splines on increasingly finer meshes

In this test we consider the product of a fixed spline with splines defined on successive refinements of its knot vector. This setting arises, e.g., in IgA-BEM quadrature schemes Degli Esposti et al. (2023), where a spline is multiplied by a spline quasi-interpolant defined on a refined mesh.

We fix f as a spline of degree p 1 = 3 on a knot vector τ τ 1 with 5 uniform breakpoints in [0 , 1] , with random coefficients in [ -1 , 1] . The second factor g is a spline of fixed degree p 2 with random coefficients in [ -1 , 1] , defined on knot vectors τ τ 2 obtained by dyadic refinement of τ τ 1 , with 2 n + 3 breakpoints, n = 1 , . . . , 10 .

For low degrees (e.g., p 2 = 3 ), both approaches yield errors of machine precision across all meshes. For higher degrees, the behavior changes: Figure 5 (a)-(b) shows the case p 2 = 30 , where collocation exhibits non-negligible errors on coarse meshes before aligning with the direct approach under refinement. However, the refinement level at which accuracy is recovered is not known a priori , and can only be inferred indirectly, e.g., from the condition number of C .

Figure 3: Error comparison and collocation condition number for B i, τ · B j, τ with maximal smoothness and increasing degree. For fixed i , mean of max X e j M and max X e C j vs. polynomial degree (a) and mean condition number of C j (b), over all j such that B i and B j have overlapping supports.

<!-- image -->

Figure 4: Error comparison and condition number of C for the product of splines on a common knot vector τ with increasing degree. (a) max X e M and max X e C vs. polynomial degree. (b) Estimated condition number of C .

<!-- image -->

Figure 5: Error comparison and condition number of C for the product of a cubic spline and a spline of degree 30 . The first spline is defined on a fixed knot vector τ 1 with 5 uniform breakpoints, the second on τ 2 with 2 n + 3 breakpoints, n = 1 , . . . , 10 . (a) max X e M and max X e C vs. n . (b) Estimated condition number of C .

<!-- image -->

## 3. Improving the direct formula

In this section we review the direct formula of Mørken (1991) and propose a refined variant. This new formulation reduces the computational effort and makes the approach practical.

The original presentation is given as a single formula. We instead express it as a simple procedure based on the Oslo Algorithm. This algorithm is central both to the original construction and to our improvement. We therefore begin with a brief review of the Oslo Algorithm. We then describe the method of Mørken (1991) from an algorithmic viewpoint and introduce our enhanced procedure.

## 3.1. The Oslo Algorithm

Let p ≥ 1 . Let τ τ = ( τ j ) n + p +1 j =1 and t = ( t i ) m + p +1 i =1 be two open knot vectors of degree p , defined on the same interval I ⊆ R . Assume τ τ ⊆ t , counting multiplicities. Then S p τ ⊆ S p t .

Let q ∈ S p τ and let c = ( c j ) n j =1 be its B-spline coefficients. The Oslo Algorithm computes the coefficients b = ( b i ) m i =1 of q in the B-spline basis of S p t . This corresponds to performing multiple knot insertions at once.

We adopt the matrix formulation of Lyche and Mørken (2018, Section 4.2.3). For d = 1 , . . . , p , k ∈ { p +1 , . . . , n } , and t ∈ I , define the bidiagonal d × ( d +1) matrices R k d ( t ) by

<!-- formula-not-decoded -->

with the convention that zero denominators yield zero entries.

The algorithm is summarized in Algorithm 1. For each coefficient b i , select the index k such that t i ∈ [ τ k , τ k +1 ) . This determines the local coefficient vector c k := ( c j ) k j = k -p and the local knot vector

τ τ k := ( τ j ) k + p j = k +1 -p . Let also t i := ( t j ) i + p j = i +1 . Using these data, construct the matrices R k d ( t i + d ) for d = 1 , . . . , p . The algorithm then applies successive matrix-vector products. The matrix sizes decrease from p × ( p + 1) to 1 × 2 . Accordingly, the vector c k is reduced step by step, from length p + 1 to a scalar. This scalar is the coefficient b i .

## Algorithm 1 Oslo Algorithm b = oslo\_algorithm ( p, τ τ , c , t )

## Input:

p : degree

τ τ : ( τ j ) n + p +1 j =1 open knot vector on I

c :

B-spline coefficients in S p τ

t : ( t j ) m + p +1 j =1 open knot vector on I and such that τ τ ⊆ t

## Output:

b :

( b i ) m i =1 B-spline coefficients in S p t

for i = 1 , . . . , m 1 Find k such that t i ∈ [ τ k , τ k +1 ) ; 2 Set c k := ( c k -p , . . . , c k ) T ⊆ c ; 3 Assign b i := R k 1 ( t i +1 ) R k 2 ( t i +2 ) · · · R k p ( t i + p ) c k ;

Thanks to the bidiagonal structure of R k d , explicit assembly of these matrices is not required. A matrix-free implementation is possible via a de Boor-type procedure, see Lyche and Mørken (2018, Section 2.5), as schematized in Algorithm 2. In practice, the matrix multiplications in the last step of Algorithm 1 are replaced by b i = deboor ( p, τ τ k , c k , t i ) .

We conclude with remarks on implementation and computational aspects.

Remark 3.1. The Oslo Algorithm requires τ τ and t to be open with respect to p . This is not restrictive. If τ τ is not open, we extend it to an open knot vector ̂ τ τ by repeating the first and last knots. The additional B-spline coefficients are set to zero. The same applies to t , yielding ̂ t . After computing b on ̂ t , we retain only the coefficients corresponding to the original knot vector t .

Remark 3.2. The cost of multiplying a d × ( d +1) bidiagonal matrix by a vector of length d +1 is 3 d flops, as each row requires two multiplications and one addition. For each coefficient b i , the algorithm performs p such products, with decreasing sizes. The total cost per coefficient is

<!-- formula-not-decoded -->

where . = denotes equality up to lower-order terms. This estimate excludes matrix assembly (which is consistent with the estimate for the collocation approach, where we consider only the LU factorization cost).

The total cost of the Oslo Algorithm is therefore 3 2 mp 2 . However, each coefficient b i is computed independently. The method is thus well suited for parallelization.

## Input:

p : degree

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

t i : ( t j ) i + p j = i +1 ⊆ t knot sub-vector needed to compute b i

## Output:

b i : i th B-spline coefficient in S p t

Remark: all vector products and divisions are meant element-wise.

<!-- formula-not-decoded -->

Remark 3.3. The computation of each b i is local. It depends only on the subvector t i of t and on the local data c k and τ τ k . As a consequence, t need not be a global refinement of τ τ . It is sufficient that t locally refines τ τ . In particular, [ t 1 , t m + p +1 ] may be a subinterval of [ τ 1 , τ n + p +1 ] . In this case, the Oslo Algorithm yields the coefficients of the restriction of q ∈ S p τ to [ t 1 , t m + p +1 ] in S p t .

Remark 3.4. Algorithm 2 generalizes the de Boor algorithm for spline evaluation: if t i is replaced by ( t, . . . , t ) , the output b i is the value of the spline at t . The Oslo Algorithm also generalizes Bézier extraction. If t has the same breakpoints as τ τ , each with multiplicity p + 1 , the algorithm yields the Bernstein representation.

Remark 3.5. The product R k 1 ( t i +1 ) · · · R k p ( t i + p ) =: α α T defines a row vector of length p +1 . Its entries are called discrete B-splines of order p + 1 , typically denoted as α j,p +1 , τ , t ( i ) . The coefficient b i can be written as b i = α α T c k . Precomputing α α may be advantageous when expressing many splines of S p τ in S p t . However, this is not the case for the direct formula of Mørken (1991). We therefore avoid forming α α . Instead, we update c k after each multiplication. These operations are performed in a matrix-free fashion via a de Boor-type procedure. This avoids both matrix assembly and storage. The original formulation of Mørken (1991) is instead written in terms of these discrete B-splines and presented as a single formula.

## 3.2. The Mørken formula

The coefficient b i in (2) for the product h = f · g is given by Mørken (1991, Theorem 3.1) as

<!-- formula-not-decoded -->

Here P = ( π 1 , . . . , π p 1 ) ranges over set C of all combinations of p 1 indices from I := { 1 , . . . , p } , with p = p 1 + p 2 . The complementary set is Q := I \ P . The number of terms in the sum is therefore ( p p 1 ) = ( p p 2 ) .

For each P , define the subvectors t P := ( t i π 1 , . . . , t i π p 1 ) and t Q from t i ⊆ t . Let k 1 and k 2 be such that t i ∈ [ τ ℓ k ℓ , τ ℓ k ℓ +1 ) for ℓ = 1 , 2 . Then define the local knot vectors τ τ k 1 , τ τ k 2 and the local coefficient vectors c k 1 , c k 2 as in the Oslo Algorithm. These data determine the discrete B-splines α α P and α α Q in (3).

An algorithmic implementation for computing b = ( b i ) m i =1 is given in Algorithm 3. The Oslo Algo-

Algorithm 3 Mørken Algorithm -[ p, t , b ] = mørken ( p 1 , p 2 , τ 1 , τ 2 , c 1 , c 2 )

## Input:

p 1 , p 2 :

degrees of the spline factors

τ τ , τ τ :

1 2

knot vectors of the spline factors

c 1 , c 2 : coefficient vectors of the spline factors

## Output:

p : degree of the spline product

t : knot vector of the spline product

b : coefficient vector of the spline product

```
1 Set p = p 1 + p 2 ; 2 Define t = ( t j ) m + p +1 j =1 from τ τ 1 , τ τ 2 as described in (1); 3 Initialize b =0 0 ∈ R m ; 4 Let C be the set of combinations of p 1 indices from I := { 1 , . . . , p } for i = 1 , . . . , m 5 Let t i := ( t j ) i + p j = i +1 ; 6 If t i ∈ [ τ ℓ k ℓ , τ ℓ k ℓ +1 ) , let τ τ k ℓ := ( τ ℓ j ) k ℓ + p ℓ j = k ℓ +1 -p ℓ ⊆ τ τ ℓ , for ℓ = 1 , 2 ; 7 Let c k ℓ := ( c ℓ j ) k ℓ j = k ℓ -p ℓ ⊆ c ℓ , for ℓ = 1 , 2 ; for all P := ( π 1 , . . . , π p 1 ) ∈ C 8 Let t P := ( t i + π k ) p 1 k =1 ⊆ t i ; 9 Let t Q := t i \ t P ; 10 Set b P i = deboor ( p 1 , τ τ k 1 , c k 1 , t P ) ; 11 Set b Q i = deboor ( p 2 , τ τ k 2 , c k 2 , t Q ) ; 12 Update b i ← b i + b P i · b Q i ; Update b ← b / ( p ) ;
```

- 13 p 1

rithm appears explicitly in lines 5-7 and 10-11. Each de Boor call has cost 3 2 p 2 ℓ , ℓ = 1 , 2 . Hence, each update of b i costs about 3 2 ( p 2 1 + p 2 2 ) . The total cost is

<!-- formula-not-decoded -->

However, the computation of the coefficients b i can be parallelized. The outer loop over i is independent and can be removed. The inner loop over combinations can also be parallelized, since all terms are independent. This only affects the order of summation.

- ;

Despite these parallelizations, the number of terms ( p p 1 ) grows rapidly. The method is therefore practical only when p 2 is small relative to p 1 . This limitation explains why the formula is rarely used in practice, where implicit methods such as collocation are preferred. In the next section we present an improved procedure that reduces the number of terms in the sum.

Remark 3.6. In the original formulation of Mørken (1991), the knot vectors t P and t Q are global and refine τ τ 1 and τ τ 2 . Here we use local knot vectors. These may be non-open and span smaller intervals. They only need to refine τ τ 1 and τ τ 2 locally on their knot spans. This is valid due to the locality of the Oslo Algorithm and because openness is not required (see Remarks 3.1 and 3.3).

## 3.3. The improved procedure

The main drawback of Algorithm 3 is the inner loop over all combinations of p 1 indices from a set of size p . This introduces a factor ( p p 1 ) in the cost, which grows rapidly with p . Moreover, the algorithm does not exploit repeated knots in t i . As a result, identical subvectors t P and t Q are recomputed multiple times.

To address this issue, we replace the loop over C with a loop over a set T i of distinct combinations derived from t i . This set depends on i , unlike C , which is global. We also introduce multiplicities. For each element of T i , we store the number of occurrences in the full set C . These values form a set M i , paired with T i .

This leads to a factorization of the sum in (3). Identical terms are grouped together. Each distinct term corresponds to an element of T i and is weighted by its multiplicity in M i . We denote by knot\_combinations the routine that computes T i and M i from t i and p 1 . Its description is given in the next subsection.

The resulting procedure is shown in Algorithm 4. It computes b = ( b i ) m i =1 more efficiently than Algorithm 3. The improvement is confirmed numerically in Section 4. The inner loop now iterates over T i , not over index combinations. We keep the notation t P for a generic element, to emphasize the link with Algorithm 3. Here, however, P is only a label and does not range over C . Let

<!-- formula-not-decoded -->

be the total number and mean number, respectively, of distinct combinations used to compute b . The total cost is then estimated as

<!-- formula-not-decoded -->

This matches the cost of Algorithm 3, with ( p p 1 ) replaced by ¯ ν . In Section 4 we compare these quantities and show the gain in efficiency.

The algorithm admits as well parallelization. The coefficients b i can be computed independently. For fixed i , each contribution associated with t P ∈ T i is also independent. Both loops can therefore be parallelized.

Remark 3.7. Both Algorithm 3 and Algorithm 4 require binomial coefficients. Their direct computation may overflow for large p . Two standard strategies can be used (see Press et al. (2007)).

A first approach uses

<!-- formula-not-decoded -->

## Input:

p 1 , p 2 :

degrees of the spline factors

τ τ 1 , τ τ 2 :

knot vectors of the spline factors

c 1 , c 2 :

coefficient vectors of the spline factors

## Output:

p : degree of the spline product

t : knot vector of the spline product

b : coefficient vector of the spline product

```
1 Set p = p 1 + p 2 ; 2 Define t = ( t j ) m + p +1 j =1 from τ τ 1 , τ τ 2 as in (1); 3 Initialize b =0 0 ∈ R m ; for i = 1 , . . . , m 4 Let t i := ( t j ) i + p j = i +1 ; 5 If t i ∈ [ τ ℓ k ℓ , τ ℓ k ℓ +1 ) , let τ τ k ℓ := ( τ ℓ j ) k ℓ + p ℓ j = k ℓ +1 -p ℓ ⊆ τ τ ℓ , for ℓ = 1 , 2 ; 6 Let c k ℓ := ( c ℓ j ) k ℓ j = k ℓ -p ℓ ⊆ c ℓ , for ℓ = 1 , 2 ; 7 Set [ T i , M i ] = knot_combinations ( t i , p 1 ) ; for all ( t P , m P ) in ( T i , M i ) 8 Let t Q := t i \ t P ; 9 Set b P i = deboor ( p 1 , τ τ k 1 , c k 1 , t P ) ; 10 Set b Q i = deboor ( p 2 , τ τ k 2 , c k 2 , t Q ) ; 11 Update b i ← b i + m P · b P i · b Q i ; Update b ← b / ( p ) ;
```

12 p 1

where we assumed k = min( k, n -k ) without loss of generality. This form is stable when evaluated incrementally. A second approach uses the Gamma function:

<!-- formula-not-decoded -->

In practice, one evaluates

<!-- formula-not-decoded -->

This avoids overflow and replaces products by sums. It is the approach used in the experiments of Section 2.

Remark 3.8. A similar factorization idea appears in Chen et al. (2009), applied to the formula of Ueda (1994). The two approaches differ in representation. The formula of Ueda (1994) uses blossoms, while that of Mørken (1991) uses discrete B-splines.

The method of Chen et al. (2009) introduces a sliding window algorithm to evaluate blossoms efficiently. However, blossoming remains complex in practice (see Chen et al. (2009, Sections VII-VIII)).

Our approach proceeds differently. First, we express the formula of Mørken (1991) in algorithmic form using a matrix-free procedure based on the Oslo Algorithm. Second, we factorize the expression to reduce the cost. This leads to a simpler and more practical method, easier to integrate into spline and isogeometric analysis codes.

## 3.4. Identification of distinct combinations and number of repetitions

For convenience, we write t i in terms of distinct breakpoints and multiplicities:

<!-- formula-not-decoded -->

where θ j is the j th distinct knot and m j its multiplicity. Since t i has length p ,

<!-- formula-not-decoded -->

We construct all distinct combinations of p 1 knots from t i via a recursive procedure. The recursion is based on the number of breakpoints in Θ i . We start from the minimal case of at most two breakpoints. For longer sequences, we reduce the problem to a shorter sequence by removing one breakpoint.

Assume first that Θ i has at most two breakpoints:

<!-- formula-not-decoded -->

with m 2 = 0 allowed. The multiplicities µ 1 , µ 2 of a combination ( θ µ 1 1 , θ µ 2 2 ) of length p 1 satisfy

<!-- formula-not-decoded -->

The admissible range of µ 1 is therefore

<!-- formula-not-decoded -->

All distinct combinations are obtained by iterating over these values:

<!-- formula-not-decoded -->

If Θ i has more than two breakpoints, we proceed recursively. The multiplicity µ 1 of θ 1 satisfies

<!-- formula-not-decoded -->

For each admissible µ 1 , we concatenate θ µ 1 1 with all combinations of length p 1 -µ 1 from

<!-- formula-not-decoded -->

The same procedure is applied recursively to ̂ Θ i . The recursion terminates when only two breakpoints remain.

We now determine multiplicities. A distinct combination

<!-- formula-not-decoded -->

is defined by the multiplicities µ P j . For each breakpoint θ j , the number of ways to select µ P j occurrences among m j identical knots is ( m j µ P j ) . Since selections are independent, the total multiplicity is

<!-- formula-not-decoded -->

This value counts how many times t P appears in the full set of combinations. It provides the weight used in Algorithm 4 to group identical terms in (3). The full procedure is summarized in Algorithm 5.

Algorithm 5 Distinct combinations and their multiplicities -[ T i , M i ] = knot\_combinations ( t i , p 1 )

## Input:

<!-- formula-not-decoded -->

p 1 :

spline factor degree

## Output:

T i : collection of distinct combinations

M i : multiplicities of the distinct combinations

- 1 Let Θ i := ( θ m 1 1 , . . . , θ m s s ) be the breakpoints in t i of multiplicities m 1 , . . . , m s with m 1 &gt; 0 and m ℓ ≥ 0 for ℓ = 2 , . . . , s ; 2 Initialize T i ← ∅ and M i ← ∅ ; 3 Set m max := min { p 1 , m 1 } ; 4 Set m min := p 1 -min { p 1 , ∑ s ℓ =2 m ℓ } ; if Θ i has at most two breakpoints, i.e., s ≤ 2 then for µ 1 = m max , m max -1 , . . . , m min 5 Set µ 2 = p 1 -µ 1 ; 6 Insert ( θ µ 1 1 , θ µ 2 2 ) in T i ; 7 Insert ( m 1 µ 1 )( m 2 µ 2 ) in M i ; else for µ 1 = m max , m max -1 , . . . , m min 8 Let ̂ t i be t i with all the m 1 occurrences of θ 1 removed; 9 Let [ ̂ T i , ̂ M i ] = knot\_combinations ( ̂ t i , p 1 -µ 1 ) ; 10 Concatenate θ µ 1 1 to each element of ̂ T i ; 11 Insert the resulting knot vectors in T i ; 12 Insert in M i the rescaled repetitions ( m 1 µ 1 ) · ̂ M i ;

Figure 6: Comparison of ¯ ν and ( p p 1 ) to compute one coefficient for the product of a fixed cubic B-spline and a polynomial of increasing degree. (a) comparison vs. polynomial degree. (b) ¯ ν vs. polynomial degree.

<!-- image -->

## 4. Computational aspects and comparisons on the considered numerical tests

In this section we compare the number of terms required by the direct formula (3) when implemented with Algorithm 3 and with the improved Algorithm 4.

For Algorithm 3, the number of terms is fixed and equal to ( p p 1 ) . For Algorithm 4, it depends on the index i . We therefore consider the average number of terms, denoted by ¯ ν , over all indices i . The comparison is performed for the test cases of Section 2.

The results show that Algorithm 4 always requires fewer terms. In some cases the difference is dramatic. The binomial coefficient may reach values of order 1e29 , while ¯ ν remains moderate. Even when the gap is smaller, the gain is still significant. This cost is incurred for each coefficient b i . Hence, the advantage increases with the number of coefficients m .

The experiments also show that the type of factors (single B-splines or full splines) does not affect the number of terms per coefficient. It only changes the dimension of the product space S p t , i.e., the number of coefficients m to compute. The comparison is therefore essentially unchanged.

For each test we report two figures: one comparing ( p p 1 ) with ¯ ν and one showing only the growth of ¯ ν .

## 4.1. Product of a fixed B-spline, or spline, and polynomials of increasing degrees

We first consider the case of Section 2.1. A fixed cubic B-spline is multiplied by a polynomial of degree p 2 ∈ { 1 , . . . , 50 } . The results are shown in Figure 6 (a)-(b). We also tested a general spline of degree p 1 = 3 with the same polynomial factor. The behavior is unchanged.

## 4.2. The Galerkin isogeometric case with p - and k -refinements

We next consider the product of two B-splines on the same knot vector τ τ , with equal degree p 1 = p 2 ∈ { 3 , . . . , 50 } . The knot multiplicities are adjusted to preserve global C 2 smoothness, as in p -refinement.

̸

For each degree, we fix f = B i, τ and consider all B-splines B j, τ with overlapping support, supp B i, τ ∩ supp B j, τ = ∅ . The average ¯ ν is computed by averaging first over all such j , and then over i = 1 , . . . , m . The results are shown in Figure 7 (a)-(b). The rapid growth of ( p p 1 ) follows from the asymptotic behaviour of the central binomial coefficient, i.e., when p 1 = ⌈ p/ 2 ⌉ . By Stirling's approximation, it behaves like 2 p / √ p and grows exponentially with p . We also consider the k -refinement case, where smoothness is maximal. The results are shown in Figure 8 (a)-(b).

Figure 7: Comparison of ¯ ν (averaged over i and j ) and ( p p 1 ) to compute one coefficient of B i, τ · B j, τ with fixed smoothness ( C 2 ) and increasing degree. (a) comparison vs. polynomial degree. (b) ¯ ν vs. polynomial degree.

<!-- image -->

## 4.3. Product of splines on same knot vector and same degree

We consider the product of two splines on the same knot vector τ τ , with degree p 1 = p 2 ∈ { 1 , . . . , 50 } . The knot vector has five breakpoints in [0 , 1] . The coefficients of both splines are random in [ -1 , 1] . The comparison is shown in Figure 9 (a)-(b).

## 4.4. Product of a fixed spline with splines on increasingly finer meshes

Finally, we consider the product of a fixed spline of degree p 1 = 3 on a knot vector τ τ 1 with splines defined on successive refinements of τ τ 1 . Even for p 2 = 3 , the improved procedure already yields a significant reduction in the number of terms, see Figure 10 (a)-(b). The advantage increases with the degree. For instance, for p 2 = 30 , the gain is much more pronounced, see Figure 11 (a)-(b).

## 5. Conclusion

In this work we have revisited the direct formula of Mørken (1991) for representing spline products in the B-spline basis and shown that, when reformulated algorithmically, it yields a robust, accurate, and computationally efficient alternative to the implicit interpolation approach. By relying on the Oslo Algorithm, the computation is reduced to successive matrix-vector products with bidiagonal matrices. Then, by factoring out repeated contributions in the combinatorial sum, we achieve a substantial reduction in computational cost while preserving numerical stability.

Extensive numerical experiments confirm that the proposed algorithm remains efficient even for high polynomial degrees, where naive direct implementations of the direct formula become impractical and implicit methods suffer from severe ill-conditioning. In representative test cases, the procedure is often superior in accuracy by several orders of magnitude compared to the interpolation approach.

Figure 8: Comparison of ¯ ν (averaged over i and j ) and ( p p 1 ) to compute one coefficient of B i, τ · B j, τ with maximal smoothness and increasing degree. (a) comparison vs. polynomial degree. (b) ¯ ν vs. polynomial degree.

<!-- image -->

Figure 9: Comparison of ¯ ν and ( p p 1 ) to compute one coefficient for the product of splines on a common knot vector τ with increasing degree. (a) comparison vs. polynomial degree. (b) ¯ ν vs. polynomial degree.

<!-- image -->

Figure 10: Comparison of ¯ ν and ( p p 1 ) to compute one coefficient for the product of cubic splines on nested meshes. The first spline is defined on a fixed knot vector τ 1 with 5 uniform breakpoints, the second on τ 2 with 2 n + 3 breakpoints, n = 1 , . . . , 10 . (a) comparison vs. n . (b) ¯ ν vs. n .

<!-- image -->

Figure 11: Comparison of ¯ ν and ( p p 1 ) to compute one coefficient for the product of a cubic spline and a spline of degree 30 on nested meshes. The first spline is defined on a fixed knot vector τ 1 with 5 uniform breakpoints, the second on τ 2 with 2 n +3 breakpoints, n = 1 , . . . , 10 . (a) comparison vs n . (b) ¯ ν vs. n .

<!-- image -->

Several directions remain open for future research. First, while the algorithm exhibits an intrinsic parallel structure, both across spline coefficients and across distinct knot combinations, a systematic investigation of parallel and high-performance implementations, including shared-memory architectures, is of clear interest for scalability. Second, a deeper theoretical analysis of the computational complexity of the improved procedure, including sharp bounds and average-case estimates, would further clarify its advantages over blossoming-based and implicit approaches. Third, it would be of interest to establish direct formulations for other fundamental spline operations, such as the composition of splines, and extend the direct-versus-implicit comparison carried out in this work. Finally, adaptive variants of the algorithm, in which negligible contributions to the computation of the product B-spline coefficients are automatically dismissed, represent a possible direction for further reducing computational effort in large-scale applications.

Overall, the results of this work suggest that direct spline products, long regarded as theoretically elegant but practically cumbersome, can in fact be implemented in a simple, efficient, and robust manner, making them a viable and attractive tool for a wide range of spline-based numerical methods.

## Acknowledgments

This work has been supported by the MUR Excellence Department Project MatMod@TOV (CUP E83C23000330006) awarded to the Department of Mathematics of the University of Rome Tor Vergata. The authors are members of Gruppo Nazionale per il Calcolo Scientifico of the Italian Istituto Nazionale di Alta Matematica 'Francesco Severi' (GNCS-INdAM). The INdAM support through GNCS-2026 annual project (CUP E53C25002010001) is gratefully acknowledged.

## References

- de Boor, C., 1978. A practical guide to splines. volume 27 of Applied Mathematical Sciences . SpringerVerlag, New York-Berlin.
- Bracco, C., Patrizi, F., Sestini, A., 2026. A smoothly varying quadrature approach for 3D IgA-BEM discretizations: application to Stokes flow simulations. Comput. Methods Appl. Mech. Engrg. 452, Paper No. 118773, 22. URL: https://doi.org/10.1016/j.cma.2026.118773 , doi:10.1016/j.cma.2026.118773.
- Chen, X., Riesenfeld, R.F., Cohen, E., 2009. An algorithm for direct multiplication of B-splines. IEEE transactions on automation science and engineering 6, 433-442.
- Cohen, E., Lyche, T., Riesenfeld, R., 1980. Discrete B-splines and subdivision techniques in computeraided geometric design and computer graphics. Computer graphics and image processing 14, 87-111.
- Cottrell, J.A., Hughes, T.J.R., Reali, A., 2007. Studies of refinement and continuity in isogeometric structural analysis. Comput. Methods Appl. Mech. Engrg. 196, 4160-4183. doi:10.1016/j.cma.2007.04.007.
- Degli Esposti, B., Falini, A., Kanduč, T., Sampoli, M.L., Sestini, A., 2023. IgA-BEM for 3D Helmholtz problems using conforming and non-conforming multi-patch discretizations and B-spline tailored numerical integration. Comput. Math. Appl. 147, 164-184. URL: https://doi.org/10.1016/j.camwa.2023.07.012 , doi:10.1016/j.camwa.2023.07.012.
- Farouki, R.T., 2012. The Bernstein polynomial basis: a centennial retrospective. Comput. Aided Geom. Design 29, 379-419. URL: https://doi.org/10.1016/j.cagd.2012.03.001 , doi:10.1016/j.cagd.2012.03.001.

- Hughes, T.J.R., Cottrell, J.A., Bazilevs, Y., 2005. Isogeometric analysis: CAD, finite elements, NURBS, exact geometry and mesh refinement. Comput. Methods Appl. Mech. Engrg. 194, 4135-4195. URL: https://doi.org/10.1016/j.cma.2004.10.008 , doi:10.1016/j.cma.2004.10.008.
- Lee, E.T.Y., 1994. Computing a chain of blossoms, with application to products of splines. Comput. Aided Geom. Design 11, 597-620. URL: https://doi.org/10.1016/0167-8396(94)90054-X , doi:10.1016/0167-8396(94)90054-X.
- Lyche, T., 1988. Note on the Oslo algorithm. Computer-aided design 20, 353-355.
- Lyche, T., Manni, C., Speleers, H., 2018. Foundations of spline theory: B-splines, spline approximation, and hierarchical refinement, in: Splines and PDEs: from approximation theory to numerical linear algebra. Springer, Cham. volume 2219 of Lecture Notes in Math. , pp. 1-76.
- Lyche, T., Mørken, K., 1986. Making the Oslo algorithm more efficient. SIAM J. Numer. Anal. 23, 663-675. URL: https://doi.org/10.1137/0723042 , doi:10.1137/0723042.
- Lyche, T., Mørken, K., 2018. Spline Methods Draft.
- Manni, C., Sande, E., Speleers, H., 2023. Outlier-free spline spaces for isogeometric discretizations of biharmonic and polyharmonic eigenvalue problems. Comput. Methods Appl. Mech. Engrg. 417, Paper No. 116314, 33. URL: https://doi.org/10.1016/j.cma.2023.116314 , doi:10.1016/j.cma.2023.116314.
- Manni, C., Speleers, H., 2016. Standard and non-standard CAGD tools for isogeometric analysis: a tutorial, in: Isogeometric analysis: a new paradigm in the numerical approximation of PDEs. Springer, [Cham]. volume 2161 of Lecture Notes in Math. , pp. 1-69.
- Mørken, K., 1991. Some identities for products and degree raising of splines. Constr. Approx. 7, 195-208. URL: https://doi.org/10.1007/BF01888153 , doi:10.1007/BF01888153.
- Piegl, L., Tiller, W., 1997. Symbolic operators for NURBS. Computer-Aided Design 29, 361-368.
- Press, W.H., Teukolsky, S.A., Vetterling, W.T., Flannery, B.P., 2007. Numerical Recipes: The Art of Scientific Computing. 3rd ed., Cambridge University Press, Cambridge, UK.
- Sande, E., Manni, C., Speleers, H., 2019. Sharp error estimates for spline approximation: explicit constants, n -widths, and eigenfunction convergence. Math. Models Methods Appl. Sci. 29, 11751205. URL: https://doi.org/10.1142/S0218202519500192 , doi:10.1142/S0218202519500192.
- Sangalli, G., Tani, M., 2018. Matrix-free weighted quadrature for a computationally efficient isogeometric k -method. Comput. Methods Appl. Mech. Engrg. 338, 117-133. URL: https://doi.org/10.1016/j.cma.2018.04.029 , doi:10.1016/j.cma.2018.04.029.
- Schoenberg, I.J., Whitney, A., 1953. On Pólya frequence functions. III. The positivity of translation determinants with an application to the interpolation problem by spline curves. Trans. Amer. Math. Soc. 74, 246-259. URL: https://doi.org/10.2307/1990881 , doi:10.2307/1990881.
- Schumaker, L.L., 2007. Spline functions: basic theory. Cambridge Mathematical Library. third ed., Cambridge University Press, Cambridge. URL: https://doi.org/10.1017/CBO9780511618994 , doi:10.1017/CBO9780511618994.

- Ueda, K., 1994. Multiplication as a general operation for splines, in: Curves and surfaces in geometric design (Chamonix-Mont-Blanc, 1993). A K Peters, Wellesley, MA, pp. 475-482.
- Beirão da Veiga, L., Buffa, A., Rivas, J., Sangalli, G., 2011. Some estimates for h -p -k -refinement in isogeometric analysis. Numer. Math. 118, 271-305. URL: https://doi.org/10.1007/s00211-010-0338-z , doi:10.1007/s00211-010-0338-z.