<!-- image -->

## Subdivision methods for solving polynomial equations

Bernard Mourrain, Jean-Pascal Pavone

## To cite this version:

Bernard Mourrain, Jean-Pascal Pavone. Subdivision methods for solving polynomial equations. Journal of Symbolic Computation, 2009, 44 (3), pp.292-306. ⟨inria-00070350⟩

## HAL Id: inria-00070350 https://inria.hal.science/inria-00070350v1

Submitted on 19 May 2006

HAL is a multi-disciplinary open access archive for the deposit and dissemination of scientific research documents, whether they are published or not. The documents may come from teaching and research institutions in France or abroad, or from public or private research centers.

L'archive ouverte pluridisciplinaire HAL , est destinée au dépôt et à la diffusion de documents scientifiques de niveau recherche, publiés ou non, émanant des établissements d'enseignement et de recherche français ou étrangers, des laboratoires publics ou privés.

<!-- image -->

<!-- image -->

INSTITUT NATIONAL DE RECHERCHE EN INFORMATIQUE ET EN AUTOMATIQUE

## Subdivision methods for solving polynomial equations

Bernard Mourrain -Jean-Pascal Pavone

N° 5658

Août 2005

Thème SYM

<!-- image -->

<!-- image -->

## Subdivision methods for solving polynomial equations

Bernard Mourrain , Jean-Pascal Pavone

ThŁme SYM   SystŁmes symboliques Projet Galaad

Rapport de recherche n 5658   Aoßt 2005   22 pages

a0

Abstract: This paper presents a new algorithm for solving a system of polynomials, in a domain of n . It can be seen as an improvement of the Interval Projected Polyhedron algorithm proposed by Sherbrooke and Patrikalakis [SP93]. It uses a powerful reduction strategy based on univariate root  nder using Bernstein basis representation and Descarte's rule . We analyse the behavior of the method, from a theoretical point of view, shows that for simple roots, it has a local quadratic convergence speed and gives new bounds for the complexity of approximating real roots in a box of n . The improvement of our approach, compared with classical subdivision methods, is illustrated on geometric modeling applications such as computing intersection points of implicit curves, self-intersection points of rational curves, and on the classical parallel robot benchmark problem.

a0

Key-words: resolution, symbolic-numeric computation, polynomial equation, subdivision, real solution, Bernstein basis, Descartes rule, complexity.

## MØthode de subdivision pour la rØsolution d'Øquations polynomiales

a0

RØsumØ : Dans ce rapport nous prØsentons un nouvel algorithme pour la rØsolution de systŁmes d'Øquations polynomiales en plusieurs variables, dans un domaine de n . Il peutŒtre vu comme une amØllioration de l'algorithme Interval Projected Polyhedron proposØ par Sherbrooke and Patrikalakis [SP93]. Il utilise une technique de rØduction de domaines efcaces, qui s'appuie sur une mØthode de rØsolution de polyn mes en une variable, utilisant la reprØsentation dans la base de Bernstein et la rŁgle de Descartes . nous analysons le comportement de la mØthode d'un point de vue thØorique et monrons que pour des racines simples, il a une vitesse de convergence locale quadratique. Ceci nous conduit   de nouvelle bornes de complexitØ pour lÆpproximation de racines dans une boite de n . L'apport de notre mØthode, comparØe avec des mØthodes classiques de subdivision, est illustrØ sur des applications en modØlisation gØomØtrique, telles que le calcul de points d'intersection de courbes implicites, de points d'auto-intersection de courbes paramØtrØes et sur un problŁme classique de robotique. rØsolution, calcul symbolique-numerique, Øquation polynomiale, subdivision,

Mots-clØs : solution rØelle, base de Bernstein, rŁgle de Descartes, complexitØ.

a0

## 1 Introduction

Solving polynomial equations is ubiquitous in geometric problems. We can identify two main families of solvers: a  rst family of solvers which exploit the algebraic properties associated to these polynomials and all their polynomial combinations. It usually leads to methods which gives global informations on the set of solutions [CLO92]; a second family of solvers which treat the polynomials as real value functions and analyse the zero-level of these functions. It usually leads to local methods, such as the famous Newton (-Raphson) method [Rhe98].

a0

The approach that we describe in this paper combines, in some way, these two characteristics. We exploit the properties of polynomial representations in the Bernstein basis, to deduce easily informations on the corresponding real functions in a domain of n . Bernstein polynomial representations are ubiquitous in geometric modeling. It is known to be numerically more stable than the monomial basis representation [FG96], [FR87]. This Bernstein representation has a direct geometric meaning, in terms of control points. It provides useful properties such that the convex hull and the variation diminishing properties. These properties in conjunction with subdivision techniques explain the large variety of algorithms proposed until today for solving univariate polynomials, starting with Lane and Riesenfeld [JR81], up to the Bezier clipping methods initiated by Nishita and al [TNK90]. They combine a global control on the domain where the roots are searched with local and e cient re nements.

The situation in the multivariate case has not been studied so extensively. Two main subfamilies coexist: A  rst family which is based on subdivision techniques such as [EK01]; a second family of solvers is based on reduction techniques, as in [SP93].

The subdivision approaches use an exclusion test, based on the convex hull property in [SP93], for checking for the existence of a solution in the search domain. The result of the test is of the form:  no solution or  maybe one . If the answer is  no solution then the domain is rejected. Otherwise the domain is subdivided, generally in a way independent of the data, and this process is repeated until the domain satis es a termination criterion. This termination criterion can be simply based on the size of the domain, but it can be more elaborated [EK01], [SM88] [GS01b] (using for instance Miranda theorem). The subdivision approach provides algorithms that produce a large number of iterations especially in the case of multiple roots, but the iteration cost is signi cantly smaller than in reduction approaches, that we mention now.

Reduction approaches use a technique to contract the domain where the roots are searched, such as the convex hull property used in [SP93]. Its power resides in its capacity to concentrate on the parts of the domain where the roots are. Reductions cannot replace completely subdivisions, because it is not always possible to reduce the domain, if we have to separate the roots, but they reduce drastically the number of iterations, and thus have a great impact on the performance of the solver.

In this paper, we analysis in detail the subdivision and reduction approaches. We propose a general scheme for comparing and evaluating them. In addition, we consider a new reduction technique and new preconditioning steps, which in uence drastically the e ciency of the solvers. This scheme allows us to compare a large variety of algorithms, including the one previously known. Reporting on extended experimentations, we show the impact of different strategies on the behavior of the solvers. This leads to a new reduction-subdivision solver, which overcome the other methods. It can be seen as an improvement of the wellknown interval projected polyhedron [SP93]. We use e cient univariate solvers to optimise the reduction steps, and show that the cost of solving such univariate polynomial equations is compensated by the speed of the reduction.

a0

The paper is organised as follows. In the next section, we recall the main properties of Bernstein basis, that we will use for isolating real roots. In section 3, we describe and analyse the behavior of a family of univariate polynomial solvers, using this representation. In section 4, we describe the multivariate subdivision solver, which uses as a main ingredient a univariate solver. The importance of the preconditioning step and the subdivision strategy are discussed. We analyse the behavior of the method, from a theoretical point of view, shows that locally it is converging quadratically to a simple root and gives new bounds for the complexity of approximating real roots in a box D ⊂ n . Experimentations on geometric problems and classical benchmarks show the performances of the implementation and conclude the papers.

## 2 Bernstein polynomial representation

Let us  rst recall the main properties of Bernstein polynomial representation, that we are going to use. For a more detailed list of properties of this representation, we refer for instance to [Far90].

Any univariate polynomial f ( x ) ∈ [ x ] , of degree d , can be represented as

<!-- formula-not-decoded -->

a0

The polynomials B i d ( x ; a, b ) := ( d i ) 1 ( b -a ) d ( x -a ) i ( b -x ) d -i form the Bernstein basis on [ a, b ] . Hereafter, we are going to consider the sequence of values b = [ b 0 , .., d d ] together with the corresponding interval [ a, b ] , as a representing our polynomial f .

A fundamental algorithm that we will use on such a representation is the de Casteljau algorithm [Far90]: b 0 i = b i , i = 0 , . . . , d , b r i = (1 -t ) b r -1 i + t b r -1 i +1 ( t ) , i = 0 , . . . , d -r . It allows us to subdivide the representation of f into the two subrepresentations on the intervals [ a, (1 -t ) a + tb ] and [(1 -t ) a + tb, b ] . It requires at most 2 d ( d + 1) arithmetic operations.

A simple but interesting property that we are going to use is the following:

Theorem 1 (Descartes rule). [Ris91], [BPR03], [MRR04] The number of real roots of f ( x ) = ∑ b i B i d ( x ; a, b ) in ] a, b [ is bounded by the number V ( b ) of sign changes of b = ( b i ) i =0 ..n , and is equal modulo 2 .

As a consequence, if V ( b ) = 0 there is no root in ] a, b [ and if V ( b ) = 1 , there is one root in ] a, b [ . Another interesting property of this representation is the following:

Theorem 2 (Convex hull). [Far90], [Ris91] Let b = ( b i ) i =0 ,...,d be the control coe cients of f ( x ) on the interval [ a, b ] and c = [( ( d -i ) a + i b d , b i ) i =0 ,...,d ] the corresponding control points. The graph { ( t, f ( t )); t ∈ [ a, b ] } is in the convex hull of the control points c .

By a direct extension to the multivariate case, any polynomial f ( x 1 , . . . , x n ) ∈ [ x 1 , . . . , x n ] of degree d i in the variable x i , can be decomposed as:

a0

<!-- formula-not-decoded -->

where ( B i 1 d 1 ( x 1 ; a 1 , b 1 ) · · · B i n d n ( x n ; a n , b n )) 0 ≤ i 1 ≤ d 1 ,..., 0 ≤ i n ≤ d n is the tensor product Bernstein basis on the domain D := [ a 1 , b 1 ] ×···× [ a n , b n ] ⊂ n and b ( f ) = ( b i 1 ,...,i n ) 0 ≤ i 1 ≤ d 1 ,..., 0 ≤ i n ≤ d n are the control coe cients of f on D . The polynomial f is represented in this basis by the n th order tensor of control coe cients b ( f ) . The control points of f are

a0

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

a0

We have the following property:

De Casteljau algorithm also applies in each of the direction x i , i = 1 , . . . , n so that we can split this representation accordingly. This can be used either to split the domain or to restrict the representation to a subdomain. For a multivariate polynomial of degree d i in x i , we check that this restriction operation costs 2 ∑ n i =1 d i ∏ n i =1 ( d i +1) = O ( d n +1 ) where d = max { d 1 , . . . , d n } . Thus as the dimension and the degree increase, a good method to isolate the roots, should consider carefully when to apply this reduction operation, in order to save the computation time.

̸

<!-- formula-not-decoded -->

Lemma 1 (Projection Lemma). For any u = ( u 1 , . . . , u n ) ∈ D , and any j = 1 , . . . , n , we have m j ( f ; u j ) ≤ f ( u ) ≤ M j ( f ; u j ) .

<!-- formula-not-decoded -->

̸

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

A similar proof applies for m j ( f ; u j ) .

̸

̸

̸

As a direct consequence, we obtain the following corollary:

Corollary 1. For any root ζ = ( ζ 1 , . . . , ζ n ) ∈ n of the equation f ( x ) = 0 in the domain D , we have µ j ≤ ζ j ≤ µ j where

a0

<!-- formula-not-decoded -->

- m j ( f ; u ) ≤ 0 ≤ M j ( f ; u ) on [ µ j , µ j ]

De nition 2. For a system of polynomials f = ( f 1 , . . . , f s ) , we de ne

<!-- formula-not-decoded -->

## 3 Univariate Root Solver

Our approach for solving multivariate systems is based on e cient methods for isolating roots of univariate polynomials. In this section, we describe the di erent univariate solvers that we use for this purpose.

Descartes rule 1 yields a simple subdivision algorithm, which isolated the roots, as described in [MVY02] or [MRR04]. The behavior of the algorithm can be analysed using to the two circles theorem (see [BPR03], [Meh] or [MRR04]), which shows that at if f ( x ) = 0 has only simple roots on [ a, b ] , an upper bound of the number of recursion steps of this algorithm is at most ⌈ log 2 ( 2 σ ) ⌉ , where σ is the minimal distance between the complex roots of f .

In order to approximate a root within a given precision ϵ &gt; 0 , after it has been isolated, a usual approach is by bisection, that is by splitting the interval into two subintervals and by choosing the interval containing the root. This splitting can be performed

- either in the Bernstein basis by de Casteljau algorithm (which requires O ( d 2 ) arithmetic operations),
- or in the monomial basis using Horner methods (which requires O ( d ) arithmetic operations).
- or by a secant-like method, which consists in intersecting the interval [ a, b ] with the line with the lowest slope which is joining the  rst control point to another point, and by using this point to split the Bernstein representation,
- by computing iteratively the  rst intersection of the convex hull of the control polygon, with the x -axis and in subdividing the polynomial representation at this point [Roc90],
- or by a Newton-like method, which consists in splitting the interval at the point where the tangent cuts the interval [ a, b ] if it exists or at the middle otherwise. These operations are performed in the monomial basis.

The two  rst schema converge linearly to the root in the interval, whereas the other have a superlinear and quadratic convergence speed [Hen77]. In the third method, each iteration requires O ( d ) arithmetic operations. This method can be improved by computing the intersection of the convex hull of the control points with the x -axis, but it requires O ( d log( d )) arithmetic operations and interval arithmetic for a numerically stable implementation [PM02]. The fourth method requires O ( d 2 ) arithmetic operations.

By choosing the adequate arithmetic rounding mode during the iterations of these methods, we guaranty that the  rst (resp. last) root of the polynomial is approximated below (resp. above) at the end of the computation.

Experimentations done on univariate polynomials with random roots or coming from ray tracing problems tracing problems shows the superiority of Horner and Newton iterations in terms of speed, compared to the other methods. Such algorithms allows us to solve more than 10 6 equations of degree 9 , within a precision ϵ = 10 -12 , on an Intel Pentium4 2.0Ghz, 512 Mo of RAM workstation. Newton iteration seems to be ahead in "simple" situations where the speed of convergence compensates the arithmetic cost of the iteration.

## 4 Multivariate root  nding

In this section, we consider a system of s polynomial equations in n variables:

a0

<!-- formula-not-decoded -->

with coe cients in , that we will also denote by f ( x ) = 0 . We are looking for an approximation of the real roots of f ( x ) = 0 in the domain D = [ a 1 , b 1 ] ×··· × [ a n , b n ] , within a precision ϵ .

## 4.1 The algorithm

The general framework of the families of algorithms that we will consider consists

- 1) in applying a preconditioning step on the equations;
- 2) in reducing the domain;
- 3) and if the reduction ratio is too small, in splitting the domain;

until the size of the domain is smaller than a given precision ϵ . We have several options for each of these steps, leading to di erent algorithms with di erent behaviors, as we will see in the last section. Indeed the solvers that we will consider are parameterised by the

- preparation strategy : a transformation of the initial system followed by a projection on the x i axes;
- reduction solver : a method derived from univariate root  nding techniques used to reduce the initial domain, according to the available projections.
- subdivision rule : a technique used to subdivide the domain, in order to simplify the forthcoming steps, for searching of the roots.

## 4.2 Preconditioner

Let us describe here the two preparation steps that we use to improve, at each iteration, the numerical quality of the system. Namely, we transform the system f = 0 into an equivalent one M f = 0 , where M is an s × s invertible matrix

As such a transformation may increase the degree of some equations, with respect to some variables, it has a cost, which might not be negligible in some cases. Moreover, if for each polynomials of the system not all the variables are involved, that is if the systems is sparse with respect to the variables, such a preconditioner may transform it into a system which is not sparse anymore. In this case, we would prefer a partial preconditioner on a subsets of the equations sharing a subset of variables.

For the sake of simplicity, we will assume hereafter that the polynomials f 1 , . . . , f s are expressed in the same Tensor product Bernstein basis, that we denote hereafter by B . We are going to consider two types of transformation.

a0

Global transformation A typical di cult situation for a method which exploits the value of the functions ( f i ) i =1 ,...,s , is when two of these functions have closed graphs on the domain D . A way to avoid such a situation is to transform these equations in order to increase the distance between these graphs. As this distance is not straightforward to compute, we replace it by the distance between the control points of the functions on the domain D : for f, g ∈ [ x ] , let dist( f, g ) 2 = || f -g || 2 with

<!-- formula-not-decoded -->

where b ( f ) is the vector of control coe cients of the function f in the Bernstein basis B . This norm on the vector space of polynomials generated by the basis B is associated to a scalar product that we denote by 〈 | 〉 . The aim of this preconditioner is to optimize the angles between the vectors, that is to produce a system which is orthogonal for 〈 | 〉 . We obtain it by eigenvector computation.

Proposition 1. Let Q = ( 〈 f i | f j 〉 ) 1 ≤ i,j ≤ s and let E be a matrix of unitary eigenvectors of Q . Then ˜ f = E f is a system of polynomials which are orthogonal for the scalar product 〈 | 〉 .

<!-- formula-not-decoded -->

Proof. Let E be the matrix of (real) unitary eigenvectors of Q and let ˜ f = E f = ( ˜ f 1 , . . . , ˜ f n ) . Then the matrix of scalar product ( ˜ f i | ˜ f j ) is

where σ 1 ≥ 0 , . . . σ n ≥ are the positive eigenvalues of Q . This shows that the system ˜ f is orthonormal for the scalar product 〈 | 〉 .

We illustrate the impact of the global preconditioner in  gure 1, on two bivariate functions which graphs are very closed to each other before the preconditioning, and which are well separated after this preconditioning step. In this case, one of the new functions is even not intersecting the zero-level (gray plane), so that we can deduce directly that there is no root in the domain D .

Fig. 1. Global preconditioner

<!-- image -->

Local straightening In this section we consider square systems, for which s = n . Since we are going to use the projection lemma 1, interesting situation for reduction steps, are when the zero-level of the functions f i are orthogonal to the x i -directions. We illustrate this remark in dimension 2 , by  gure 2: In the case (a), the reduction based on corollary 1, will be of no use (because the projection of the graphs cover the intervals), whereas in case (b), a good reduction strategy will yield a good approximation of the roots. This

Fig. 2. Local preconditioner

<!-- image -->

idea of this preconditioner is thus to transform the system f = 0 , in order to be closed to the case (b). Namely, we transform locally the system f into a system J -1 f ( u 0 ) f , where J f ( u 0 ) = ( ∂ x i f j ( u 0 ) 1 ≤ i,j ≤ s is the Jacobian matrix of f at the point u 0 ∈ D . See [GS01a] for a previous application of this idea.

A direct computation shows that locally (in a neighborhood of u 0 ), the level-set of ˜ f i ( i = 1 , . . . , n ) are orthogonal to the x i -axes.

## 4.3 Reduction strategy

We describe several reduction strategies, which have been considered. See also [Spe94].

In [SP93], a method called Interval Projected Polyhedron (or IPP) is described, in order to reduce the domain of search. It is based on the convex hull property 2.

A direct improvement consists in computing the  rst (resp. last) root of the polynomial m j ( f k ; u j ) , (resp. M j ( f k ; u j ) ), in the interval [ a j , b j ] , and keep the intervals [ µ, µ ] de ned in corollary 1. As we can see in the following example,  gure 4.3 the improvement compared with the IPP approach can be substantial. The actual implementation of this reduction steps

Fig. 3. Convex hull vs. root  nding

<!-- image -->

allows us to consider the convex hull reduction, as one iteration step of this reduction process. By increasing the number of iterations, we improve the interval containing the extreme roots. The precision required in the approximation of the roots is not an important aspect of this step, since we are more interested on the ratio of the size of the new interval by the size of the initial one. This allows us to reduce more e ciently the domain, by computing several subdomains or by rejecting it more quickly, as it is illustrated in  gure 4, This method can be even further improved by taking into account simultaneously all the projections of the polynomials f j of the system. The guarantee that the computed intervals contain the root of f , is achieved by controlling the rounding mode of the operations during the de Casteljau computation.

## 4.4 Subdivision strategy

Here some simple rules that can be used to subdivide a domain. We will show in the last section their impact on the performance of the solver

The subdivision method simply checks the sign of control coe cients of the polynomials f j on the domain D . If for one of non-zero polynomial f k , its control coe cient vectors has no sign change, than D does not contain any root and should be excluded. The domain D is then split in half in a direction j for which | b j -a j | is maximal.

Fig. 4. Convex hull vs. root  nding

<!-- image -->

This is approach used in [EK01] who argued that the reduction approaches are not so interesting because they cannot avoid anyway a lot of subdivisions. In section 6, we will analyse through experimentations the e ectiveness of this remark.

A variant of this approach consists in subdividing the domain in a direction j if | b j -a j | &gt; ϵ and if the control coe cients of M j ( f k ; u j ) are not all positive, those of m j ( f k ; u j ) not all negative. This allows us to have domains more adapted to the geometry of the roots, but still a postprocessing step for gluing together connected domains may be required.

## 5 Analysis

In this section, we analyse the behavior of the method in terms of the invariants of f . Hereafter, the || · || or || · || ∞ is the ∞ -norm, a box B ⊂ n is a product of intervals I 1 ×··· × I n , | B | is the size of the box that is the maximum of the size of the intervals I l l = 1 , . . . , n . For any polynomial f ∈ [ x 1 , . . . , x n ] and any box B ⊂ n , we denote by C ( f ; B ) the piecewise linear function B → , de ned by the control points c ( f ) of f in the Bernstein basis of B .

a0

a0

a0

For any compact domain D ⊂ n and any f ∈ [ x 1 , . . . , x n ] , we denote by K 1 ( f, D ) the Lipschitz constant of f in D , satisfying

a0

a0

a0

<!-- formula-not-decoded -->

Let K 2 ( f, D ) = max x ∈D , 1 ≤ i,j ≤ n | ∂ i ∂ j f ( x ) | and K 2 ( f , D ) = max { K 2 ( f i , B ); i = 1 , . . . , s } .

̸

a0

An important property of the Bernstein representation that we will use hereafter is the following:

For any linear map M : n → n , let m 1 ( M ) be the smallest singular value of M such that if y = Mx and det( M ) = 0 , then || x || 2 ≤ 1 m 1 ( M ) || y || 2 ≤ √ n m 1 ( M ) || y || ∞ .

a0

a0

<!-- formula-not-decoded -->

Proposition 2. [Dah86] [dB87] [PK94] Let B ⊂ n be a box and f ∈ [ x 1 , . . . , x n ] , then ∀ x ∈ B ,

## 5.1 Local quadratic convergence

We consider here the local preconditioner method described in section 4.2, and shows that locally, it is converging quadratically to a simple root.

Proposition 3. Let B be a box of n and u 0 its center such that det( J f ( u 0 )) = 0 . Then

a0

<!-- formula-not-decoded -->

̸

where ˜ f = J -1 f ( u 0 ) f , m 1 ( J f ( u 0 )) is the smallest singular value of J f ( u 0 ) and ˜ m j , ˜ M j are the corresponding bounding polynomials of the system ˜ f .

Proof. By a Taylor expansion at u 0 , we have

<!-- formula-not-decoded -->

where R ( u ) only involves monomials of degree ≥ 2 . By the mean value theorem, for any u with u 0 + u ∈ B and any i = 1 , . . . , n , we have

<!-- formula-not-decoded -->

with ν i ∈ B , so that | R i ( u ) | ≤ n 2 K 2 ( f i , B ) | B | 2 . Thus

<!-- formula-not-decoded -->

where v = J -1 f ( u 0 ) f ( u 0 ) ∈ n and | F i ( u ) | ≤ ˜ k 2 | B | 2 with k 2 = K 2 ( f , B ) and ˜ k 2 = n 5 2 k 2 m 1 ( J -1 f ( u 0 )) .

a0

By proposition 2, we have |C ( R i ) -R i ( u i ) | &lt; k 2 | B | 2 and

<!-- formula-not-decoded -->

We deduce that for i = 1 , . . . , n ,

<!-- formula-not-decoded -->

which proves the proposition.

a0

This also proves that the distance between two consecutive roots, one of m i ( ˜ f i ; u i ) and one of M i ( ˜ f i ; u i ) is less than 2 ( ˜ k 2 + ˜ l 2 ) | B | 2 .

a0

As a consequence, we immediately deduce that any reduction strategy using this preconditioner and the projection lemma 1 near a simple root, will converge quadratically.

̸

Corollary 2. Let B be a box of n and u 0 its center such that det( J f ( u 0 )) = 0 . Let ˜ B be the box obtained after one reduction step, then we have

<!-- formula-not-decoded -->

a0

## 5.2 Global convergence

We consider here a domain D = [ a 1 , b 1 ] ×··· × [ a n , b n ] ⊂ n in which we are searching the roots of f = 0 . We denote them by ζ 1 , . . . , ζ t ∈ D ⊂ n .

a0

a0

For r ∈ + , we denote by S i ( r ) = { x ∈ n ; | f i ( x ) | ≤ r } and S ( r ) = ∩ n i =1 S i ( r ) . Notice that S (0) is the set of real solutions of f 1 ( x ) = 0 , . . . , f s ( x ) = 0 .

a0

Proposition 4. Assume that 0 &lt; δ &lt; K 1 ( f, D ) K 2 ( f, D ) , then for any box B of center u 0 such that u 0 ̸∈ S (2 K 1 ( f ; D ) δ ) and | B | &lt; δ , the control coe cients of one of the polynomials f i ( i = 1 , . . . , n ) on B are of the same sign.

Proof. Let r &gt; 0 and B be a box with center u 0 ̸∈ S ( r ) with | B | ≤ δ . Assume that δ ≤ r 2 k 1 , where k 1 = K 1 ( f, D ) . Then u 0 ̸∈ S i 0 ( r ) for some i 0 in { 1 , . . . , n } . Consequently, | f i 0 ( u 0 ) | ≥ r . We deduce that ∀ x ∈ B ,

<!-- formula-not-decoded -->

As f i 0 cannot vanish on the connected set B , it has a constant sign, say positive. By proposition 2, we have

<!-- formula-not-decoded -->

where k 2 = K 2 ( f i 0 , D ) . Thus if r = 2 k 1 δ and δ &lt; k 1 k 2 , we have b i 1 ,...,i n ( f i 0 ) &gt; 0 for all coe cient indexes i 1 , . . . , i n , which proves that the control coe cients of f i 0 are of the same sign on B .

We denote by N ( δ ) the minimal number of boxes of size δ in a binary subdivision of D , which are covering S (2 k 1 δ ) .

Corollary 3. For 0 &lt; δ &lt; K 1 ( f, D ) K 2 ( f, D ) , the number of boxes of size δ kept in the subdivision algorithm is bounded by N ( δ ) .

Proof. According to the previous proposition, for the boxes of size δ not covering S (2 k 1 δ ) , one of the polynomials f i 0 has control coe cients in the box of constant sign. Thus such a box is removed by the subdivision algorithm. Consequently the number of boxes of size δ in the subdivision algorithm is bounded by N ( δ ) .

Proposition 5. N ( δ ) , δ &gt; 0 is bounded.

Proof. ∀ ϵ &gt; 0 , let V ( ϵ ) = ∪ t i =1 V ( ζ i , ϵ ) be a compact union of balls V ( ζ i , ϵ ) around the roots ζ i ( i = 1 , . . . , t ). As ∩ r&gt; 0 S ( r ) = { ζ 1 , . . . , ζ t } , we have V ( ϵ ) ⊂ ∪ r&gt; 0 S ( r ) c ∩ D . As V ( ϵ ) is compact, it is covered by a  nite union of S c ( r 1 ) , . . . , S c ( r l ) . Thus, there exists r 0 = min { r 1 , . . . , r n } &gt; 0 such that V c ( ϵ ) ∩ D ⊂ S c ( r 0 ) ∩ D or S ( r 0 ) ∩ D ⊂ V ( ϵ ) .

̸

̸

̸

Let δ 0 be such that S (2 k 1 δ 0 ) ⊂ V ( σ ) and δ 0 &lt; K 1 ( f, D ) K 2 ( f, D ) . For δ ≤ δ 0 and x ∈ S (2 k 1 δ ) ⊂ V ( σ ) , we have by the mean value theorem for j = 1 , . . . , n f j ( x ) = ( x -ζ i 0 ) · ∇ f j ( ν ) , where x, ν ∈ V ( ζ i 0 , σ ) . This implies that

Let us choose σ &gt; 0 such that B ( ζ i , σ ) ∩ B ( ζ j , σ ) = ∅ if i = j and det( J f ( u )) = 0 for u ∈ ∪ s i =1 B ( ζ i , σ ) . This is possible since the roots ζ i are simple ( J f ( ζ i ) = 0 ). We denote by m 1 ( σ ) = min x ∈ V ( σ ) { m 1 ( J f ( x )) } .

<!-- formula-not-decoded -->

Consequently, the center of the boxes of size δ &lt; δ 0 kept by the algorithm are in ∪ t i =1 V ( ζ i , 2 k 1 δ m 1 ( σ ) ) , which implies that N ( δ ) ≤ t × ( 4 k 1 m 1 ( σ ) +1) n . This proves that for δ small enough and thus for any δ &gt; 0 , N ( δ ) is bounded.

We consider now the following characteristic quantities of the system f = 0 :

- We denote by N D ( f ) = max δ&gt; 0 N δ .
- Let ρ = max { δ &gt; 0 st. S (2 K 1 ( f, D ) δ ) ⊂ V ( σ ) } .
- Let σ &gt; 0 be such that V ( ζ i , σ ) ∩ V ( ζ j , σ ) = ∅ if i = j and det( J f ( x )) = 0 for x ∈ V ( σ ) = ∪ s i =1 V ( ζ i , σ ) .

a0

Theorem 3. Assume that f is a square system, with simple roots ζ 1 , . . . , ζ t in the domain D ⊂ n . Then the roots of f = 0 can be approximated within the precision 0 &lt; ϵ &lt; 1 2 , by performing at most

<!-- formula-not-decoded -->

arithmetic operations, where

- c 0 &gt; is a constant independent of f , D and n ,
- K 2 ( f , D ) is a bound on the second derivatives of the polynomials f i on D and K 1 ( f , D ) a bound on their Lipschitz constants on D ,
- d = max 1 ≤ i,j ≤ n deg x i ( f f j ) ,

̸

̸

- N D ( f ) is related to the entropy of f = 0 in D (or the number of roots) [YC04] and K 1 ( f , D ) ,
- ρ is related to the minimal distance between the roots and their distance to the variety det( J f ( x )) = 0 and K 1 ( f , D ) .

a0

Proof. Let k 1 = K 1 ( f , D ) , k 2 = K 2 ( f, D ) , ˜ k 2 = 2 ( n 2 +1) √ nk 2 m 1 ( σ ) . Let δ 0 = min( ρ, 1 2 ˜ k 2 , k 1 k 2 ) . Let n 0 ∈ be such that 2 -n 0 |D| &lt; δ 0 , that is n 0 ≥ log |D| +max(log( ρ -1 ) , log( ˜ k 2 ) + 1 , log k 2 -log k 1 ) . We take n 0 = log ρ -1 +log ˜ k 2 + log k 2 . By applying the subdivision algorithm n 0 times, we obtain boxes B 0 of size &lt; δ 0 such that the center u 0 is in S 2 k 1 δ 0 ⊂ V ( σ ) . Therefore, we have m 1 ( J f ( u 0 )) &gt; m 1 ( σ ) .

Let us denote by B k a box obtained next by application of the reduction algorithm k th times. By corollary 3, we have

<!-- formula-not-decoded -->

Thus if 1 ˜ k 2 2 2 k &lt; ϵ , we have | B k | &lt; ϵ . Since computing the inverse of J f ( u 0 ) requires O ( n 3 ) arithmetic operations and the de Casteljau algorithm O ( d n +1 ) , we deduce that the number of arithmetic operations needed to approximate the roots within ϵ is bounded by

<!-- formula-not-decoded -->

which yields the bound of the theorem.

## 6 Experimentations

The structure of the algorithm described in the section 4 has been implemented in the C++ library synaps 1 . Our objective in these experimentations is to evaluate the impact of reduction approaches compared with subdivision techniques, with and without preconditioning. The di erent methods that we compare are the following:

- sbd stands for subdivision; it is the simplest approach which looks to the minimum and maximum control coe cients on an equation-per-equation basis. It is similar to [EK01].
- rd stands for reduction. Because it uses a more sophisticated reduction step it produces less iterations than the method in [SP93].
- sbds stands for subdivision using the global preconditioner (section 4.2) we described. It works in the same way as sbd but on a transformed system.
- rds stands for reduction using the global preconditioner (section 4.2).
- rdl stands for reduction using the local preconditioner (section 4.2).

1 http://www-sop.inria.fr/galaad/software/synaps

For each method and each example, we output the number of iterations of the solver, the number of subdivisions steps, the number of domains computed at the end and the time in milliseconds on a Intel Pentium 4 2.0Ghz with 512 Mo RAM, workstation. The interesting characteristics are the number of iterations and the size of the output. In these experiments we use a modi cation of the  rst reduction solver based on Descartes rule (section 3). It handles all the projections associated to one variable, simultaneously and is able to reject domains earlier. Simple roots are approximated using bisection. In these experimentations, we use one of the simplest univariate root- nder we have described. Notice that changing the root- nder can improve the computation time but not the number of iterations. The subdivision rule consists in a splitting along the largest variable, when the reduction gain is close to 1 .

We consider examples of implicit curve intersection problems de ned by bi-homogeneous polynomials in examples (a) and (b) . Then we consider the intersection (c) and (d) and the self-intersection (e) and (f) problems for rational curves.

<!-- image -->

<!-- image -->

## (a) Bidegree : (8 , 8) , (8 , 8)

| method   |   iter. |   subd. |   result |   time (ms) |
|----------|---------|---------|----------|-------------|
| sbd      |   84887 |   84887 |    28896 |        3820 |
| rd       |   82873 |   51100 |    20336 |        4553 |
| sbds     |    6076 |    6076 |      364 |         333 |
| rds      |    1486 |     920 |      144 |         163 |
| rdl      |    1055 |     305 |       60 |         120 |

## (b) Bidegrees : (12 , 12) , (12 , 12) .

| method sbd   |   iter. 4826 |   subd. 4826 |   result 220 |   time (ms) 217 |
|--------------|--------------|--------------|--------------|-----------------|
| rd           |         2071 |         1437 |          128 |             114 |
| sbds         |         3286 |         3286 |          152 |             180 |
| rds          |         1113 |          748 |           88 |             117 |
| rdl          |          389 |          116 |           78 |              44 |

Given two rational curves C 0 = ( x 0 ( u ) , y 0 ( u ) , z 0 ( u )) and C 1 = ( x 1 ( v ) , y 1 ( v ) , z 1 ( v )) we compute their intersection by solving the system: S = C 0 × C 1 = 0 , where × is the cross-product. For computing the self-intersection of a curve, we solve the system S = C ( u ) × C ( v ) u -v = 0 and use a domain control class that clip the domain to { u &lt; v } . In the following examples of in- tersection (c) and (d) and self-intersection (e) and (f), the solutions are computed up within the precision of 10 -6 .

| method   |   iter. |   subd. |   result |   time (ms) |
|----------|---------|---------|----------|-------------|
| sbd      |    2159 |    2159 |       21 |        40.8 |
| rd       |     185 |     100 |        8 |        7.76 |
| sbds     |     681 |     681 |        8 |       26.95 |
| rds      |     108 |      38 |        7 |       10.45 |
| rdl      |      75 |      28 |        7 |        4.72 |

| method sbd   |   iter. 2193 |   subd. 2193 |   result 91 |   time (ms) 2750.55 |
|--------------|--------------|--------------|-------------|---------------------|
| rd           |          584 |          395 |          47 |              286.62 |
| sbds         |          289 |          289 |           2 |              148.22 |
| rds          |           70 |           20 |           2 |               51.95 |
| rdl          |           49 |           15 |           2 |               28.26 |

| method   |   iter. |   subd. |   result |   time (ms) |
|----------|---------|---------|----------|-------------|
| sbd      |    3979 |    3979 |       39 |     3540.41 |
| rd       |     560 |     376 |       15 |      537.11 |
| sbds     |    1577 |    1577 |       16 |     1589.27 |
| rds      |     282 |      63 |       13 |       344.9 |
| rdl      |     126 |      36 |       13 |      134.21 |

| method   |   iter. |   subd. |   result |   time (ms) |
|----------|---------|---------|----------|-------------|
| sbd      |    4647 |    4647 |       47 |     1102.67 |
| rd       |    1497 |    1012 |       23 |       409.9 |
| sbds     |     255 |     255 |        2 |       69.47 |
| rds      |      85 |      24 |        1 |        32.7 |
| rdl      |      29 |       6 |        1 |       10.15 |

<!-- image -->

On these bivariate systems we conclude on the superiority of the reduction approaches, namely reduction with local straightening ( rdl ) which have a good convergence (quadratic) in theory as well as in practice. One the other hand in hard case ( gure 6) preconditioned reduction and subdivision methods ( sbds, rdl, rds ) outperforms classical subdivision and reduction ( rd, sbd ). Notice that the improvement provided by the combination of reduction and preconditioning is interesting in practice, not only because of the time saved, but also because the size of the result is smaller. Notice that in most of the examples only rds and rdl provide the good answer.

Since our reduction principle is based on projection, di culties may arise when we increases the number of variables in our systems. Hereafter we show some experiments for a six variable problem of degree ≤ 2 in each variable, coming from the robotics community. In this example we keep the three reduction methods: rd , rdl , rds and add combinations of them that use projections: before and after local straightening ( rd+rdl ), before and after global preconditioning ( rd+rds ), and the three techniques ( rd+rdl+rds ).

| domain = [ - 3 , 3] 6   | domain = [ - 3 , 3] 6   | domain = [ - 3 , 3] 6   | domain = [ - 3 , 3] 6   | domain = [ - 3 , 3] 6   | domain = [ - 30 , 30] 6   |   domain = [ - 30 , 30] 6 |   domain = [ - 30 , 30] 6 |   domain = [ - 30 , 30] 6 |   domain = [ - 30 , 30] 6 |
|-------------------------|-------------------------|-------------------------|-------------------------|-------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|
| method                  | iter.                   | subd.                   | result                  | time (s)                |                           |                           |                           |                           |                           |
| rd                      | 34125                   | 6059                    | 492                     | 14.5                    |                           |                           |                           |                           |                           |
| rdl                     | 33471                   | 10543                   | 4                       | 23.9                    | rd                        |                    249578 |                     54765 |                       532 |                     101.6 |
| rds                     | 9082                    | 2288                    | 8                       | 12.3                    | rd+rdl                    |                     27833 |                      6623 |                         4 |                      26.9 |
| rds+rdl                 | 3555                    | 872                     | 4                       | 6.4                     | rd+rds                    |                      9908 |                      2051 |                        18 |                      15.3 |
| rd+rdl                  | 2814                    | 655                     | 4                       | 2.7                     | rd+rdl+rds                |                      7196 |                      1483 |                         4 |                      14.3 |
| rd+rds                  | 3120                    | 670                     | 15                      | 4.8                     |                           |                           |                           |                           |                           |
| rd+rdl+rds              | 1863                    | 409                     | 4                       | 3.7                     |                           |                           |                           |                           |                           |

As one can see the combination of projections is an improvement, but the important point here is that the global preconditioner tends to be better than local straightening. If we look at the  rst table, we see that the number of iterations of rdl is close to rd , but it is not the case for rds . The reason is that rdl uses a local information (a Jacobian evaluation) while rds use a global information computed using all the coe cients in the system.

In conclusion of this section, we  rst experiment that both subdivisions and reductions methods are bad solutions if they are not preconditioned. But using the same preconditioner reduction will, most of the time, beat subdivision.

## 7 Conclusion

Our approach can be seen as an enhancement of the previous works of Sherbrooke and Patrikalakis [SP93]. They proposed a reduction method. We show how to generalizes it by univariate root solving methods. Their convergence speed is locally linear, we show that preconditioning can leads to quadratic convergence. We also show that subdivisions methods can be improved by preconditioning. We give an answer to Elber and Kim [EK01], who argued that the reduction approaches are not so interesting because they cannot avoid a lot of subdivisions. Our experimentations shows that reduction can save a lot of subdivisions. In cases of  tangents solutions , a pure subdivision approach has no chance to compete with a reduction approach. However it is better to use a preconditioned subdivision than a pure reduction approach, our experimentations illustrate this phenomenon.

## Bibliography

- [BPR03] S. Basu, R. Pollack, and M.-F. Roy. Algorithms in Real Algebraic Geometry . Springer-Verlag, Berlin, 2003. ISBN 3-540-00973-6.
- [CLO92] D. Cox, J. Little, and D. O'Shea. Ideals, Varieties, and Algorithms: An Introduction to Computational Algebraic Geometry and Commutative Algebra . Undergraduate Texts in Mathematics. Springer Verlag, New York, 1992.
- [Dah86] Wolfang Dahmen. Subdivision algorithms converge quadratically. Journal of Computational and Applied Mathematics , 2:145 158, 1986.
- [dB87] Carl de Boor. B-form basics. Geometric Modeling , 2:27 49, 1987.
- [EK01] Gershon Elber and Myung-Soo Kim. Geometric constraint solver using multivariate rational spline functions. In Proceedings of the sixth ACM Symposium on Solid Modelling and Applications , pages 1 10. ACM Press, 2001.
- [Far90] G. Farin. Curves and surfaces for computer aided geometric design : a practical guide . Comp. science and sci. computing. Acad. Press, 1990.
- [FG96] R.T. Farouki and T.N.T Goodman. On the optimal stability of the bernstein basis. Mathematics of computation , 65(216):1553 1566, October 1996.
- [FR87] R. Farouki and V. Rajan. On the numerical condition of polynomials in bernstein form. Computer Aided Geometric Design , 4(3):191 216, 1987.
- [GS01a] J rgen Garlo  and Andrew P. Smith. Investigation of a subdivision based algorithm for solving systems of polynomial equations. Nonlinear Anal. , 47(1):167 178, 2001.
- [GS01b] J rgen Garlo  and Andrew P. Smith. Solution of systems of polynomial equations by using Bernstein expansion. In Symbolic algebraic methods and veri cation methods (Dagstuhl, 1999) , pages 87 97. Springer, Vienna, 2001.
- [Hen77] P. Henrici. Applied and computational complex analysis . John Wiley &amp; Sons, New York, NY, 1977.
- [JR81] J.Lane and R.Riesenfeld. Bounds on a polynomial. BIT , 21:112 117, 1981.
- [Meh] K. Mehlhorn. A remark on the sign variation method for real root isolation. Submitted for publication, report ECG-TR-123101-01.
- [MRR04] B. Mourrain, F. Rouillier, and M.-F. Roy. Bernstein's basis and real root isolation. Technical Report 5149, INRIA Rocquencourt, 2004. Submitted for journal publication.
- [MVY02] B. Mourrain, M. Vrahatis, and J.C. Yakoubsohn. On the complexity of isolating real roots and computing with certainty the topological degree. J. of Complexity , 18(2):612 640, 2002.
- [PK94] H. Prautzsch and L. Kobbelt. Convergence of subdivision and degree elevation. Advances in Computational Mathematics , 1:143 154, 1994.
- [PM02] M. P. Patrikalakis and T. Maekawa. Shape Interrogation for Computer Aided Design and Manufacturing . Springer Verlag, 2002.

- [Rhe98] Werner C. Rheinboldt. Methods for solving systems of nonlinear equations , volume 70 of CBMS-NSF Regional Conference Series in Applied Mathematics . Society for Industrial and Applied Mathematics (SIAM), Philadelphia, PA, second edition, 1998.
- [Ris91] J.J. Risler. MØthodes mathØmatiques pour la CAO . Masson, 1991.
- [Roc90] Alyn Rockwood. Accurate display of tensor product isosurfaces. In IEEE Visualization '90 Conf. , 1990.
- [SM88] T. W. Sederberg and R. J. Meyers. Loop detection in surface patch intersections. Computer Aided Geometric Design , 5(2):161 171, 1988.
- [SP93] E. C. Sherbrooke and N. M. Patrikalakis. Computation of the solutions of nonlinear polynomial systems. Comput. Aided Geom. Design , 10(5):379 405, 1993.
- [Spe94] M. Spencer. Polynomial Real Root Finding in Bernstein Form . PhD thesis, Brigham Young University., 1994.
- [TNK90] T.W. Sederberg T. Nishita and M. Kakimoto. Ray tracing trimmed rational surface patches. Computer Graphics , 24(4 (Proc. ACM Siggraph 90)):337 345, August 1990.
- [YC04] Y. Yomdin and G. Comte. Tame geometry with applications in smooth analysis . LNM 1834. Springer-Verlag, 2004.

<!-- image -->

Unité de recherche INRIA Sophia Antipolis 2004, route des Lucioles - BP 93 - 06902 Sophia Antipolis Cedex (France)

Unité de recherche INRIA Futurs : Parc Club Orsay Université - ZAC des Vignes

4, rue Jacques Monod - 91893 ORSAY Cedex (France)

Unité de recherche INRIA Lorraine : LORIA, Technopôle de Nancy-Brabois - Campus scientifique

615, rue du Jardin Botanique - BP 101 - 54602 Villers-lès-Nancy Cedex (France)

Unité de recherche INRIA Rennes : IRISA, Campus universitaire de Beaulieu - 35042 Rennes Cedex (France)

Unité de recherche INRIA Rhône-Alpes : 655, avenue de l'Europe - 38334 Montbonnot Saint-Ismier (France)

Unité de recherche INRIA Rocquencourt : Domaine de Voluceau - Rocquencourt - BP 105 - 78153 Le Chesnay Cedex (France)

Éditeur INRIA - Domaine de Voluceau - Rocquencourt, BP 105 - 78153 Le Chesnay Cedex (France) http://www.inria.fr ISSN 0249-6399