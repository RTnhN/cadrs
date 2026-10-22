<!-- image -->

## Near-Optimal Parameterization of the Intersection of Quadrics: II. A Classification of Pencils

Laurent Dupont, Daniel Lazard, Sylvain Lazard, Sylvain Petitjean

## To cite this version:

Laurent Dupont, Daniel Lazard, Sylvain Lazard, Sylvain Petitjean. Near-Optimal Parameterization of the Intersection of Quadrics: II. A Classification of Pencils. Journal of Symbolic Computation, 2008, 43 (3), pp.192215. ⟨10.1016/j.jsc.2007.10.012⟩. ⟨inria-00186090⟩

## HAL Id: inria-00186090 https://inria.hal.science/inria-00186090v1

Submitted on 7 Nov 2007

HAL is a multi-disciplinary open access archive for the deposit and dissemination of scientific research documents, whether they are published or not. The documents may come from teaching and research institutions in France or abroad, or from public or private research centers.

L'archive ouverte pluridisciplinaire HAL , est destinée au dépôt et à la diffusion de documents scientifiques de niveau recherche, publiés ou non, émanant des établissements d'enseignement et de recherche français ou étrangers, des laboratoires publics ou privés.

<!-- image -->

## Near-Optimal Parameterization of the Intersection of Quadrics: II. A Classification of Pencils

## Laurent Dupont a , Daniel Lazard b , Sylvain Lazard a , Sylvain Petitjean a

a LORIA (INRIA, CNRS, Nancy Université) and VEGAS project (INRIA Nancy - Grand Est), Nancy, France b

LIP6 (Université Paris 6, CNRS) and SALSA project (INRIA Paris-Rocquencourt), Paris, France

## Abstract

We present here the first classification of pencils of quadrics based on the type of their intersection in real projective space and we show how this classification can be used to compute efficiently the type of the real intersection. This classification is at the core of the design of the algorithms, presented in Part III, for computing, in all cases of singular intersection, a near-optimal parameterization with polynomial functions, that is a parameterization in projective space whose coordinates functions are polynomial and such that the number of distinct square roots appearing in the coefficients is at most one away from the minimum.

Key words:

Intersection of surfaces, pencils of quadrics, classification, curve parameterization.

1991 MSC:

15A21, 51-04, 68U05, 68U07, 68W30

## 1. Introduction

We presented, in Part I, an algorithm for computing parameterizations of the intersection of pairs of quadrics which produces near-optimal parameterizations when the intersection is a smooth quartic (the generic case). When the intersection is singular, this algorithm usually fails to produce near-optimal parameterizations.

For computing such near-optimal parameterizations in all cases, we design, in Part III, dedicated algorithms for every type of real intersection. For this, we need to detect the type of intersection of any two quadrics. We also need to obtain structural information on the intersection curve which we use to drive the algorithm for computing near-optimal parameterizations. We need, in particular, to classify real pencils of quadrics of P 3 ( R ) .

Email address: Firstname.Lastname@loria.fr ( Laurent Dupont, Sylvain Lazard, Sylvain Petitjean), Firstname.Lastname@lip6.fr (Daniel Lazard).

URLs: www.loria.fr/~dupont/ (Laurent Dupont), www-calfor.lip6.fr/~dl/ (Daniel Lazard), www.loria.fr/~lazard/ (Sylvain Lazard), www.loria.fr/~petitjea/ (Sylvain Petitjean).

We present here the first classification of pencils of quadrics based on the type of their real projective intersection. A summary of this classification is given in Tables 1 and 2. We then show how this classification can be used to compute efficiently the type of the real intersection; in particular, we show how computations with non-rational numbers can be avoided when the input quadrics have rational coefficients.

It should be stressed that, even though the classification of pencils over the reals is presented here as an intermediate step in a more global process ( i.e. , the parameterization of the intersection), this classification has an interest on its own. It could be used for instance in a collision detection context to predict at which time stamps a collision between two moving objects will happen. It also gives the ability of drawing pictures of all possible types of intersection; such a gallery of intersections is given in Figure 1.

Related work. Classifying pencils of quadrics over the complexes, that is by considering the intersection of quadrics in projective complex space, was achieved by Segre in the nineteenth century (Segre, 1883).

In the context of the representation of the geometry of Boolean combinations of volumes bounded by quadric surfaces, Ocken et al. (1987) showed how two quadrics can be simultaneously diagonalized using a real projective transformation and used this diagonalization to parameterize the real intersection of the quadrics. The analysis is however incomplete and some intersection morphologies are overlooked, leading to possible misclassifications. In particular, the cases when the characteristic polynomial of the pencil has two double roots are missing; they include morphologies such as a cubic and a non-secant line or four non-concurrent lines.

The next result on the classification of pencil of quadrics based on the real type of the intersection was obtained by Tu et al. (2002) who classified pencils in the generic case, that is when the intersection is a smooth quartic (in complex space) whose number of connected components (in real space) is two, one, or zero. Note that Wang and Krasauskas (2004) also obtained results on the classification of pencils in the generic case when the pencil is furthermore restricted to be generated by two ellipsoids in affine space. Related results have also been obtained on the separation of two ellipsoids in affine space (Wang et al., 2001).

In September 2005, Tu, Wang, Mourrain, and Wang published a research report (Tu et al., 2005) presenting a classification of pencils very similar to ours. They use the Canonical Pair Form Theorem of F. Uhlig as basic mathematical tool and refine the classification of pencils of quadrics over the complexes in exactly the same way as we do. There are however differences between the two approaches. First, we classify pencils using the inertia of the quadrics at the multiple roots of the characteristic polynomial, except for a small number of cases where simple geometric conditions allow to discriminate. By contrast, Tu et al. classify pencils using the inertia of the quadrics between the roots of the characteristic polynomial (plus the degree of the minimal polynomial of the characteristic polynomial in some cases), and rely on Puiseux expansion to deduce some information at the (multiple) roots. Second, the classification of Tu et al. is limited to non-degenerate pencils ( i.e. , pencils whose characteristic polynomial does not vanish identically), while ours covers all possible cases.

The rest of this part is organized as follows. Section 2 recalls some notation and reviews the classical Segre classification of pencils of quadrics over the complexes. We then refine this classification over the reals: we consider regular pencils, i.e. , pencils that contain quadrics that are non-singular, in Section 3, and singular pencils in Section 4. We finally discuss algorithmic issues in Section 5, before concluding.

Table 1. Classification of pencils in the case where the characteristic polynomial D ( λ, µ ) = det( λS + µT ) does not identically vanish. ( λ 1 , µ 1 ) denotes a multiple root of D ( λ, µ ) , if any, and ( λ 2 , µ 2 ) another root (not necessarily simple). When the multiplicity of ( λ 1 , µ 1 ) is even, s denotes the sign of D ( λ, µ ) in the neighborhood of this root (which is the sign of D ( λ,µ ) ( µ 1 λ -λ 1 µ ) 2 at ( λ, µ ) = ( λ 1 , µ 1 ) when the root is double). Note that s is well defined when D ( λ, µ ) has two double roots since the sign is the same in the neighborhood of both roots. When D ( λ, µ ) has a multiple root, the additional simple roots are not indicated. Note that, in the case where D ( λ, µ ) has a quadruple root that corresponds to a quadric of inertia (1 , 1) (a pair of planes), the two cases corresponding to Segre characteristics [(31)] and [(22)] are distinguished by the fact that the singular line of the pair of planes is contained in no other quadric of the pencil in the former case and in all the quadrics of the pencils in the latter case. The Segre characteristic is mentioned for clarity but is not needed for the classification.

| Segre string      | roots of D ( λ,µ ) in C                   | rank or inertia of R ( λ 1 ,µ 1 )   | rank or inertia of R ( λ 2 ,µ 2 )   | type of ( λ 2 ,µ 2 )   | s   | type of intersection in P 3 ( R )                                                                       |
|-------------------|-------------------------------------------|-------------------------------------|-------------------------------------|------------------------|-----|---------------------------------------------------------------------------------------------------------|
| [1111]            | 4 simple roots                            |                                     |                                     |                        |     | smooth quartic or ∅ ; see Finsler (1936/1937) &Tu et al. (2002) (or also Th. I.5 &I.25)                 |
| [112] [112] [112] | 1 double root 1 double root 1 double root | (3 , 0) (2 , 1) (2 , 1)             |                                     | real real real complex | - + | point nodal quartic; isolated node nodal quartic; convex singularity nodal quartic; concave singularity |
| [112]             |                                           |                                     |                                     |                        | +   | ∅ two points two non-secant conics                                                                      |
| [11(11)] [11(11)] | 1 double root                             | rank 3                              |                                     |                        |     |                                                                                                         |
|                   | 1 double root                             | (2 , 0)                             |                                     | real                   |     |                                                                                                         |
|                   | 1 double root                             |                                     |                                     |                        | -   |                                                                                                         |
|                   |                                           | (2 , 0)                             |                                     | real                   |     |                                                                                                         |
| [11(11)]          | 1 double root                             | (1 , 1)                             | (2 , 1)                             | real                   | -   |                                                                                                         |
| [11(11)]          | 1 double root                             | (1 , 1)                             | (3 , 0)                             | real                   | -   | ∅                                                                                                       |
| [11(11)]          | 1 double root                             | (1 , 1)                             |                                     | real                   | +   | two secant conics; convex singularity                                                                   |
| [11(11)]          | 1 double root                             | rank 2                              |                                     | complex                | -   | conic                                                                                                   |
| [11(11)]          | 1 double root                             | rank 2                              |                                     | complex                | +   | two secant conics; concave singularity                                                                  |
| [13]              | triple root                               | rank 3                              |                                     |                        |     | cuspidal quartic                                                                                        |
| [1(21)] [1(21)]   | triple root triple root                   | (2 , 0) (1 , 1)                     |                                     |                        |     | double point two tangent conics                                                                         |
| [1(111)] [1(111)] | triple root triple root                   | rank 1                              | (2 , 1) (3 , 0)                     |                        |     | double conic ∅                                                                                          |
| [4]               | quadruple root                            | rank 1 rank 3                       |                                     |                        |     | cubic and tangent line                                                                                  |
| [(31)]            | quadruple root                            | (1 , 1)                             |                                     |                        | -   | conic                                                                                                   |
| [(31)]            | quadruple root                            | (1 , 1)                             |                                     |                        | +   | conic and two lines crossing on the conic                                                               |
| [(22)]            | quadruple root                            | (2 , 0)                             |                                     |                        |     | double line                                                                                             |
| [(22)]            | quadruple root                            | (1 , 1)                             |                                     |                        | +   | two simple lines &a double line                                                                         |
| [(211)]           | quadruple root                            | rank 1                              |                                     |                        | -   | point                                                                                                   |
| [(211)]           | quadruple root                            | rank 1                              |                                     |                        | +   | two secant double lines                                                                                 |
| [(1111)]          | quadruple root                            | rank 0                              |                                     |                        |     | any smooth quadric of the pencil                                                                        |
| [22]              | 2 double roots                            | rank 3                              | rank 3                              | real                   |     | cubic and secant line                                                                                   |
| [22]              | 2 double roots                            | rank 3                              | rank 3                              | complex                |     | cubic and non-secant line                                                                               |
| [2(11)]           | 2 double roots                            | (3 , 0)                             | rank 2                              | real                   |     | point                                                                                                   |
| [2(11)]           | 2 double roots                            | (2 , 1)                             | rank 2                              | real                   | +   | conic and two intersecting lines                                                                        |
| [2(11)]           | 2 double roots                            | (2 , 1)                             | rank 2                              | real                   | -   | conic and point                                                                                         |
| [(11)(11)]        | 2 double roots                            | (2 , 0)                             | (2 , 0)                             | real                   | +   | ∅                                                                                                       |
| [(11)(11)]        | 2 double roots                            | (2 , 0)                             | (1 , 1)                             | real                   | -   | two points                                                                                              |
| [(11)(11)]        | 2 double roots                            | (1 , 1)                             | (2 , 0)                             | real                   | -   | two points                                                                                              |
| [(11)(11)]        | 2 double roots                            | (1 , 1)                             | (1 , 1)                             | real                   | +   | four lines (skew quadrilateral)                                                                         |
| [(11)(11)]        | 2 double roots                            | rank 2                              | rank 2                              | complex                |     | two secant lines                                                                                        |

Table 2. Classification of pencils in the case where D ( λ, µ ) identically vanishes. In the bottom part, the quadrics of the pencil have a singular point p in common. D 3 ( λ, µ ) is the determinant of the 3 × 3 upper-left matrix of R ( λ, µ ) after a congruence transformation sending p to (0 , 0 , 0 , 1) . The conic associated with a root of D 3 ( λ, µ ) corresponds to the 3 × 3 upper-left matrix or R ( λ, µ ) . ( λ 1 , µ 1 ) denotes the multiple root of D 3 ( λ, µ ) (if any) and ( λ 2 , µ 2 ) another root. When D 3 ( λ, µ ) has a multiple root, the additional simple roots are not indicated. The Segre characteristic is mentioned for clarity but is not needed for the classification.

| Segre string            | roots of D 3 ( λ,µ ) in C                                   | rank or inertia of R ( λ 1 ,µ 1 )   | inertia of R ( λ 2 ,µ 2 )   | type of ( λ 2 ,µ 2 )   | type of intersection in P 3 ( R )                                              |
|-------------------------|-------------------------------------------------------------|-------------------------------------|-----------------------------|------------------------|--------------------------------------------------------------------------------|
| [1 { 3 } ]              | no common singular point                                    |                                     |                             |                        | conic and double line                                                          |
| [111] [111] [111] [111] | 3 simple roots 3 simple roots 3 simple roots 3 simple roots | (1 , 1) (2 , 0)                     | (1 , 1) (2 , 0)             | real real real complex | four concurrent lines meeting at p point p point p two lines intersecting at p |
| [12] [12]               | double root double root                                     | (1 , 1) (2 , 0)                     |                             |                        | 2 lines and a double line meeting at p double line                             |
| [1(11)] [1(11)]         | double root double root                                     | rank 1 rank 1                       | (1 , 1) (2 , 0)             |                        | two double lines meeting at p point p                                          |
| [3]                     | triple root                                                 | rank 2                              |                             |                        | a line and a triple line meeting at p                                          |
| [(21)]                  | triple root                                                 | rank 1                              |                             |                        | a quadruple line                                                               |
| [(111)]                 | triple root                                                 | rank 0                              |                             |                        | any non-trivial quadric of the pencil                                          |
|                         | D 3 ( λ,µ ) ≡ 0                                             |                                     |                             |                        | same as in Table 5                                                             |

## 2. Preliminaries

All quadrics considered in the paper are defined in real projective space P 3 ( R ) , i.e. , they are of the form Q S = { x ∈ P 3 ( R ) | x T S x = 0 } where S is a 4 by 4 real symmetric matrix. Recall that the inertia of a quadric Q R is the pair of the numbers of positive and negative eigenvalues of R , in a decreasing order (see Table I.1 1 ); the rank of Q R is the one of R . Recall also that given two quadrics Q S and Q T , the characteristic polynomial, D ( λ, µ ) , of the pencil { R ( λ, µ ) = λS + µT | ( λ, µ ) ∈ P 1 ( R ) } generated by S and T is the determinant of R ( λ, µ ) ; for the sake of simplicity, we sometimes refer to a member of the pencil as R ( λ ) = λS -T and to the characteristic polynomial as D ( λ ) = det R ( λ ) . Note that, since all the considered quadrics have real coefficients, the coefficients of the characteristic polynomials of pencils are real.

Recall that a point p ∈ P 3 ( C ) of a quadric Q S is said to be singular if the gradient of x T S x is zero at p , that is if p is in the kernel of S ; note that quadrics with real coefficients have only real singular points. The quadric Q S is said to be singular if it contains at least one singular point (which is equivalent to det S = 0 ). In the following, we refer to a singular line of a quadric as a line whose points are all singular points of the quadric. Similarly, a point p ∈ P 3 ( C ) of a curve C defined by the implicit equations Q S = Q T = 0 is singular if the rank of the Jacobian matrix of C (the matrix of partial derivatives of Q S and Q T ) is at most 1 when evaluated at p . A curve is singular if it contains at least a singular point (in P 3 ( C ) ). Note that all it is well-know (see below and Table 3) that the intersection of two quadrics is non-singular if and only if it is a smooth quartic in P 3 ( C ) (which can be, in P 3 ( R ) , a smooth quartic or the empty set); moreover, the intersection is non-singular if and only if the characteristic polynomial admits four simple roots in C . Finally, a pencil of quadrics { Q λS + µT | ( λ, µ ) ∈ P 1 ( R ) } is said to be regular if it contains a non-singular quadric, i.e. , the characteristic polynomial does not vanish identically; otherwise, the pencil is singular.

1 When reference is made to a section or result in another part of the paper, it is prefixed by the part number.

Fig. 1. A gallery of intersections. a. Nodal quartic. b. Nodal quartic with isolated singular point. c. Cubic and secant line. d. Cubic and tangent line. e. Two secant conics. f. Two double lines. g. Four lines forming a skew quadrilateral. h. Two lines and a double line. i. Conic and two lines not crossing on the conic, the two lines being imaginary. j. Four concurrent lines, only two of which are real. k. Two lines and a double line, the three being concurrent. l. Conic and double line. The pictures were made with the Surf visualization tool.

<!-- image -->

In the following, a quadric of rank 3 is called a (projective) cone ; it is real if its inertia is (2 , 1) ; otherwise, its inertia is (3 , 0) , it is called imaginary and has a unique real point (in P 3 ( R ) ) which is its singular point. A quadric of rank 2 is a pair of planes ; it is real if its inertia is (1 , 1) ; otherwise, its inertia is (2 , 0) , it is called imaginary and its real points (in P 3 ( R ) ) are its singular line, that is, the line of intersection of the two planes. A quadric of inertia (1 , 0) is called a double plane and is necessarily real.

We now review classical material on the classification of pencils of quadrics over the complexes. It will serve as the starting point for our classification of pencils over the reals in Sections 3 and 4. The classification of pencils of quadrics over P 3 ( C ) was originally achieved by the Italian mathematician Corrado Segre (1883); more recent and accessible accounts can be found in Bromwich (1906) or Hodge and Pedoe (1953).

This classification, summarized in Tables 3-5, establishes a one-to-one correspondence between the type of intersection, in P 3 ( C ) , of any two distinct quadrics of a pencil with the Segre characteristic, whose definition we recall below. This definition is rather involved and its meaning is not intuitive at first. We thus also use a weaker but more intuitive representation by the number of multiple roots, ( λ i , µ i ) , i ∈ { 1 , 2 } , of the characteristic polynomial D ( λ, µ ) and the ranks of the associated matrices R ( λ i , µ i ) .

We now recall the definition of Segre characteristic of a pencil R ( λ, µ ) = λS + µT and show that it directly induces the number of multiple roots of D ( λ, µ ) = det R ( λ, µ ) and the rank of the associated matrices. 2 Let ( λ i , µ i ) , i = 1 , . . . , p , be the roots of D and m i be their respective multiplicities. Let m j i be the minimum multiplicity of ( λ i , µ i ) in all the polynomials defined as subdeterminant of order n -j of R ( λ, µ ) ; if ( λ i , µ i ) is not a root of one of these polynomials, then m j i = 0 . Note that m j i ⩾ m j +1 i for all j . Let t i ⩾ 1 be the smallest integer such that m t i i = 0 and let e j i = m j -1 i -m j i with j = 1 , . . . , t i and m 0 i = m i . The factors ( λµ i -µλ i ) e j i are called the elementary divisors of D ( λ, µ ) and the exponents e j i the characteristic numbers , associated with the root ( λ i , µ i ) . Note that their study goes back to Weierstrass (1868). Segre introduced the following notation to denote the various characteristic numbers associated with the degenerate quadrics that appear in a pencil: σ n = [( e 1 1 , . . . , e t 1 1 ) , ( e 1 2 , . . . , e t 2 2 ) , . . . , ( e 1 p , . . . , e t p p )] , with the convention that the parentheses enclosing the characteristic numbers of ( λ i , µ i ) are dropped when t i = 1 . This is known as the Segre characteristic , string , or symbol of the pencil.

Now, note that the multiplicity m i of a root ( λ i , µ i ) of D is the sum e 1 i + · · · + e t j i and that the rank of R ( λ i , µ i ) is n -t i , since ( λ i , µ i ) is a root of all subdeterminants of R ( λ, µ ) of order n -t i + k, k &gt; 0 , but not of all subdeterminants of order n -t i .

## 3. Classification of regular pencils over the reals

In this section, we classify, over the reals, regular pencils, that is, pencils whose characteristic polynomial D ( λ, µ ) does not identically vanish. The case of singular pencils is treated in Section 4. A summary of this classification is given in Table 1.

2 S and T are here real symmetric matrices of size n by n .

| Segre characteristic σ 4                                                                               | roots of D ( λ,µ ) in C and rank of associated matrices                                                                                                                                                                                                                                                                                      | type of intersection in P 3 ( C )                                                                                                                                                                                                                                                                                                |
|--------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [1111] [112] [11(11)] [13] [1(21)] [1(111)] [4] [(31)] [(22)] [(211)] [(1111)] [22] [2(11)] [(11)(11)] | four simple roots one double root, rank 3 one double root, rank 2 triple root, rank 3 triple root, rank 2 triple root, rank 1 quadruple root, rank 3 quadruple root, rank 2 quadruple root, rank 2 quadruple root, rank 1 quadruple root, rank 0 two double roots, both rank 3 two double roots, ranks 3 and 2 two double roots, both rank 2 | smooth quartic nodal quartic two secant conics cuspidal quartic two tangent conics double conic cubic and tangent line conic and two lines crossing on the conic two lines and a double line two double lines smooth quadric cubic and secant line conic and two lines not crossing on the conic four lines (skew quadrilateral) |

Table 3. Classification of pencils by Segre symbol when the characteristic polynomial D ( λ, µ ) does not identically vanish. When D ( λ, µ ) has a multiple root ( λ i , µ i ) , the rank of the associated matrix R ( λ i , µ i ) is indicated; the additional simple roots are not indicated and their associated matrices have rank 3.

Table 4. Classification of pencils by Segre symbol when D ( λ, µ ) ≡ 0 . When the quadrics have a singular point p in common (bottom part), D 3 ( λ, µ ) is the determinant of R 3 ( λ, µ ) , the 3 × 3 upper-left matrix of R ( λ, µ ) after a congruence transformation sending p to (0 , 0 , 0 , 1) . The matrix associated with a root ( λ i , µ i ) of D 3 ( λ, µ ) is R 3 ( λ i , µ i ) . The matrices associated to the simple roots of D 3 ( λ, µ ) have rank 2.

| Segre characteristic σ 3                        | roots of D 3 ( λ,µ ) in C and rank of associated matrices                                                                              | type of intersection in P 3 ( C )                                                                                       |
|-------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|
| [1 { 3 } ]                                      | no common singular point                                                                                                               | conic and double line                                                                                                   |
| [111] [12] [1(11)] [3] [(21)] [(111)] [ { 3 } ] | three simple roots double root, rank 2 double root, rank 1 triple root, rank 2 triple root, rank 1 triple root, rank 0 D 3 ( λ,µ ) ≡ 0 | four concurrent lines two lines and a double line two double lines line and triple line quadruple line cone see Table 5 |

Table 5. Classification of pencils by Segre characteristic in the case where D ( λ, µ ) ≡ 0 and D 3 ( λ, µ ) ≡ 0 . When the quadrics of the pencil have (at least) two singular point p and q in common (bottom part), D 2 ( λ, µ ) is the determinant of R 2 ( λ, µ ) , the 2 × 2 upper-left matrix of R ( λ, µ ) after a congruence transformation sending p and q to (0 , 0 , 0 , 1) and (0 , 0 , 1 , 0) . The matrix associated with a root ( λ i , µ i ) of D 2 ( λ, µ ) is R 2 ( λ i , µ i ) . The matrices associated to the simple roots of D 2 ( λ, µ ) have rank 1.

| Segre characteristic σ 2   | roots of D 2 ( λ,µ ) in C and rank of associated matrices                | type of intersection in P 3 ( C )                         |
|----------------------------|--------------------------------------------------------------------------|-----------------------------------------------------------|
| [ { 3 } ]                  | no two common singular points                                            | line and plane                                            |
| [11] [2] [(11)]            | two simple roots double root, rank 1 double root, rank 0 D 2 ( λ,µ ) ≡ 0 | quadruple line plane pair of distinct planes double plane |

We consider, in turn, every Segre characteristic of Table 3 and examine different cases according to whether the roots of the characteristic polynomial are real or not. We then examine the conditions leading to the different types of intersection over the reals.

In each case, we start by computing the canonical form of the pair ( S, T ) using the Canonical Pair Form Theorem for pairs of real symmetric matrices (see Uhlig (1973, 1976) or Theorem I.10). Recall that the congruence transformation in the Canonical Pair Form Theorem preserves the roots (values and multiplicities) of the characteristic polynomial of the pencil. We then deduce from the canonical form a normal form of the pencil over the reals by rescaling and translating the roots to particularly simple values. This normal form is in a sense the 'simplest pair' of quadrics having a given real intersection type. The normal pencil is equivalent to any pencil of quadrics with the same real and complex intersection type. It should however be kept in mind that that these projective transformations sending pencils into normal forms may involve irrational numbers.

We treat the first case (nodal quartic in P 3 ( C ) ) in detail. For the other cases, we move directly to the normal form without first making the canonical form explicit.

Note that, in the case where the Segre characteristic is [1111] , which corresponds to a smooth quartic in P 3 ( C ) , the classification on the type of intersection in P 3 ( R ) follows from results by Finsler (1936/1937) and Tu et al. (2002) (see also Theorems I.5 and I.25). Also, the case [(1111)] does not necessitate any further treatment: save for the quadric corresponding to the quadruple root (which is P 3 ( R ) ), all the quadrics of the pencil are equal and the intersection is thus any of those non-trivial quadrics.

A singularity of the intersection will be called convex if the branches of the curve are on the same side of the common tangent plane to the branches at the singularity, concave otherwise. It should be stressed that there is a close connection between the type of the singularity and the notion of affine finiteness introduced by Tu et al. (2002). Recall that a point set is called affinely finite if it does not intersect some projective plane and affinely infinite otherwise. As we shall see, a convex singularity corresponds to an affinely finite intersection, while a concave one corresponds to an affinely infinite intersection. Furthermore, our classification directly yields the following theorem which provides a global property on the intersection of two quadrics from a property of the pencil; note that this theorem is similar in spirit to the theorem due to Finsler (1936/1937) (see Theorem I.5) which characterizes when two quadrics have an empty intersection.

Theorem 1. If two distinct quadrics have a pencil whose characteristic polynomial does not identically vanish, their intersection is affinely finite if and only if there exists a quadric of inertia (3 , 1) in the pencil.

Proof. Any quadric of inertia (3 , 1) is affinely finite, thus if the pencil contains such a quadric, the intersection is affinely finite. Conversely, in the case where the intersection is a smooth quartic (in P 3 ( C ) ), the property follows from Tu et al. (2002) (see also Theorem I.25). Otherwise, it follows from our classification that, when there is no quadric of inertia (3 , 1) in a pencil generated by two distinct quadrics, the intersection either contains a line (and therefore is affinely infinite) or is a nodal quartic with a concave singularity or two secant conics with a concave singularity. In the last two cases, we show below that the intersection is affinely infinite. ✷

## 3.1. Nodal quartic in P 3 ( C ) , σ 4 = [112]

From Table 3, we know that the characteristic polynomial det R ( λ ) has a double root λ 1 and two simple roots λ 2 and λ 3 . Furthermore, the three associated matrices R ( λ i ) have rank 3. Also λ 1 is necessarily real because, otherwise, its conjugate would also be a double root of det R ( λ ) . The Canonical Pair Form Theorem (see Theorem I.10) thus implies that to λ 1 corresponds one real Jordan block of size 2. We consider two cases depending on whether λ 2 and λ 3 are real.

λ 2 and λ 3 are real. R ( λ 2 ) and R ( λ 3 ) are real projective cones. The Canonical Pair Form Theorem gives that S and T are simultaneously congruent to the quadrics of equations

<!-- formula-not-decoded -->

λ 1 S -T and λ 2 S -T are thus simultaneously congruent to the quadrics of equations

<!-- formula-not-decoded -->

̸

̸

<!-- formula-not-decoded -->

Let ε = sign λ 2 -λ 3 λ 1 -λ 3 (recall that λ 1 = λ 3 and λ 2 = λ 3 ). By multiplying the above two equations by -ε 1 ∣ λ 2 -λ 3 λ 1 -λ 3 ∣ and -ε 1 , respectively, we can rewrite them as

<!-- formula-not-decoded -->

∣ ∣ We obtain that R ( λ 1 ) = λ 1 S -T and R ( λ 2 ) = λ 2 S -T are simultaneously congruent, by a real projective transformation P , to the quadrics of equations

<!-- formula-not-decoded -->

with a, b, c ∈ {-1 , 1 } . One can further assume that c = 1 by changing x in -x .

From now on we forget about the transformation P and identify R ( λ i ) with P T R ( λ i ) P , but it should be kept to mind that things happen in the local frame induced by P .

If a or b is -1 , the cone R ( λ 1 ) has inertia (2 , 1) and thus is real. Otherwise ( a = b = 1 ), the cone R ( λ 1 ) is imaginary and its real locus is reduced to its apex p = (1 , 0 , 0 , 0) . The other cone R ( λ 2 ) is always real and contains the apex p of R ( λ 1 ) . We distinguish the three following cases.

- a = b = 1 . The real part of the nodal quartic is reduced to its node, the apex p of R ( λ 1 ) .

<!-- formula-not-decoded -->

- Only one of a and b is 1 . Assume for instance that a = 1 , b = -1 (the other case is obtained by exchanging z and w ). By substituting the parameterization of the cone y 2 + z 2 -w 2 = 0 (see Table I.2)

into the equation of the other cone xy + w 2 = 0 , we get the equation s u v +( u 2 + v 2 2 ) 2 = 0 . Note that, if uv = 0 , then the only solution is ( u, v, s ) = (0 , 0 , 1) . Thus, solving this equation in s , we get that the nodal quartic is parameterized by the point (1 , 0 , 0 , 0) and

<!-- formula-not-decoded -->

such that uv = 0 . Note, however, that if uv = 0 then X ( u, v ) = (1 , 0 , 0 , 0) T . Hence the nodal quartic is entirely parameterized by (2).

̸

The nodal quartic is thus real and its node, corresponding to the parameters (1 , 0) and (0 , 1) , is at p . The plane tangent to the quadric Q R ( λ 2 ) at the quartic's node p is y = 0 . In a neighborhood of this node, x = ( u 2 + v 2 ) 2 &gt; 0 and y = -4 u 2 v 2 ⩽ 0 (recall that X ( u, v ) is projective, so its coordinates are defined up to a non-zero scalar). We conclude that the two branches lie on the same side of the tangent plane and that the singularity is convex. As can readily be seen, the plane x = 0 does not intersect the quartic, so the intersection is affinely finite.

- a = -1 , b = -1 . Similarly as before, substituting the parameterization ( s, u 2 + v 2 2 , uv, u 2 -v 2 2 ) of the cone y 2 -z 2 -w 2 = 0 into the other cone xy + w 2 = 0 , yields the equation s u 2 + v 2 2 + ( u 2 -v 2 2 ) 2 = 0 . Solving this equation in s , we get that the nodal quartic is parameterized by the point (1 , 0 , 0 , 0) (if ( u, v ) = (0 , 0) ) and

<!-- formula-not-decoded -->

Wenow argue that we can easily distinguish between these three cases. For this, we first prove the following lemma.

The nodal quartic is thus real with an isolated singular point (since point (1 , 0 , 0 , 0) is not attained by X ( u, v ) , ( u, v ) ∈ P 1 ( R ) ).

Lemma 2. Given any pencil of quadrics generated by S and T whose characteristic polynomial det( λS + µT ) = 0 has a double root ( λ 1 , µ 1 ) , the sign of det( λS + µT )) ( µ 1 λ -λ 1 µ ) 2 at ( λ 1 , µ 1 ) is invariant by a real projective transformation of the pencil and does not depend on the choice of S and T in the pencil.

̸

Proof. Suppose that D ( λ, µ ) = det( λS + µT ) has a double root ( λ 1 , µ 1 ) . The lemma claims that for any real projective transformation P and any a 1 , . . . , a 4 ∈ R such that a 1 a 4 -a 2 a 3 = 0 , D ′ ( λ ′ , µ ′ ) = det( λ ′ P T ( a 1 S + a 2 T ) P + µ ′ P T ( a 3 S + a 4 T ) P ) has a double root ( λ ′ 1 , µ ′ 1 ) such that D ( λ,µ ) ( µ 1 λ -λ 1 µ ) 2 at ( λ 1 , µ 1 ) has same sign as D ′ ( λ ′ ,µ ′ ) ( µ ′ 1 λ ′ -λ ′ 1 µ ′ ) 2 at ( λ ′ 1 , µ ′ 1 ) . We have D ′ ( λ ′ , µ ′ ) = (det P ) 2 D ( a 1 λ ′ + a 3 µ ′ , a 2 λ ′ + a 4 µ ′ ) . Thus D ′ ( λ ′ , µ ′ ) has a double root ( λ ′ 1 , µ ′ 1 ) defined by

<!-- formula-not-decoded -->

It follows that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Proposition 3. If the characteristic polynomial det( λS + µT ) = 0 has two simple real roots and one double root ( λ 1 , µ 1 ) whose associated matrix λ 1 S + µ 1 T has rank three, then the intersection of S and T in P 3 ( C ) is a nodal quartic whose node is the apex of λ 1 S + µ 1 T .

Moreover, if the inertia of λ 1 S + µ 1 T is (3 , 0) then the real part of the nodal quartic is reduced to its node. Otherwise the nodal quartic is real; furthermore, if det( λS + µT ) ( µ 1 λ -λ 1 µ ) 2 is negative for ( λ, µ ) = ( λ 1 , µ 1 ) , the node is isolated and, otherwise, the singularity is convex.

Proof. The first part of the proposition follows directly from the Segre characteristic (see Table 3 and Section 2). Now, if the inertia of λ 1 S + µ 1 T is (3 , 0) , then a = b = 1 in (1) and the result follows as discussed above. Otherwise, considering S ′ = P T ( λ 1 S -T ) P and T ′ = P T ( λ 2 S -T ) P , (1) gives that det( λS ′ + µT ′ ) = -aλ ( bλ + µ ) µ 2 / 4 . Evaluating det( λS ′ + µT ′ ) µ 2 at ( λ, µ ) = (1 , 0) , gives by Lemma 2 that -ab has same sign as det( λS + µT ) ( µ 1 λ -λ 1 µ ) 2 at ( λ 1 , µ 1 ) . The result then follows from the discussion above depending on whether a = b = -1 or ab = -1 . ✷

̸

λ 2 and λ 3 are complex conjugate. The reduction to normal pencil form is slightly more involved in this case. Let λ 2 = α +i β, λ 3 = λ 2 , β = 0 . The Canonical Pair Form Theorem gives that S and T are simultaneously congruent to the quadrics of equations

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Through this congruence transformation, S ′ = λ 1 S -T has equation

where ξ is real and positive. Through the congruence transformation and with the above transformation ( z, w ) ↦→ ( z ′ , w ′ ) , S has equation

<!-- formula-not-decoded -->

Through the above congruence transformations, the quadric T ′ = βS -2 ξ -1 ξ ( ξ + 1 ξ ) 2 ( λ 1 S -T ) has equation

<!-- formula-not-decoded -->

Finally, by making a shift on x , rescaling on the four axes, and changing the signs of x and z , we get that the two quadrics S ′ and T ′ are simultaneously congruent to the quadrics of equations

<!-- formula-not-decoded -->

As before, we now drop reference to the accumulated congruence transformation and work in the local coordinate system. By substituting the parameterization of the cone y 2 + zw = 0 (see Table I.2), ( s, uv, u 2 , -v 2 ) with ( u, v, s ) ∈ P ⋆ 2 ( R ) , into the other quadric xy + z 2 -w 2 = 0 , and solving in s , we get the parameterization of the nodal quartic

<!-- formula-not-decoded -->

The nodal quartic is thus real and its node, corresponding to the parameters (1 , 0) and (0 , 1) , is at p = (1 , 0 , 0 , 0) , the apex of S ′ . The plane tangent to the quadric xy + z 2 -w 2 = 0 at the quartic's node p is y = 0 . In a neighborhood of the quartic's node on the branch corresponding to the parameter (0 , 1) , x = v 4 -u 4 &gt; 0 and y = u 2 v 2 ⩾ 0 . On the other branch corresponding to the parameter (1 , 0) , x = v 4 -u 4 &lt; 0 and y = u 2 v 2 ⩾ 0 . Hence, the two branches of the quartic are on opposite sides of the tangent plane y = 0 in a neighborhood of the node, i.e. , the singularity is concave.

̸

Let us briefly show that the intersection is affinely infinite in this case. Consider the plane ℓ 1 x + ℓ 2 y + ℓ 3 z + ℓ 4 w = 0 , ( ℓ 1 , ℓ 2 , ℓ 3 , ℓ 4 ) ∈ P 3 ( R ) , which we intersect with the nodal quartic under consideration. This yields a quartic equation E in ( u, v ) . If ℓ 1 = 0 , E has v in factor, meaning that the point (1 , 0 , 0 , 0) of parameter (0 , 1) belongs to the plane. If ℓ 1 = 0 , the coefficients of u 4 and v 4 in E have opposite sign, implying by Descartes' Sign Rule that E has at least one real non-trivial solution. The nodal quartic is thus cut by any plane of P 3 ( R ) , implying it is affinely infinite.

To summarize, we have the following result.

Proposition 4. If the characteristic polynomial det( λS + µT ) = 0 has two simple complex conjugate roots and one double root ( λ 1 , µ 1 ) whose associated matrix λ 1 S + µ 1 T has rank three, then the intersection of S and T is a real nodal quartic with a concave singularity at its node, the apex of λ 1 S + µ 1 T .

## 3.2. Two secant conics in P 3 ( C ) , σ 4 = [11(11)]

The characteristic polynomial has a double root λ 1 and the rank of R ( λ 1 ) is 2. λ 1 is necessarily real and there are two Jordan blocks of size 1 associated with it in the canonical form. Let λ 2 and λ 3 be the other (simple) roots, associated with quadrics of rank 3. We have two cases.

λ 2 and λ 3 are real. λ 2 and λ 3 appear in real Jordan blocks of size 1. The normal forms of R ( λ 1 ) and R ( λ 2 ) are z 2 + aw 2 = 0 and x 2 + by 2 + cw 2 = 0 with a, b, c ∈ {-1 , 1 } .

- a = ± 1 , b = 1 , c = 1 : The planes are real or imaginary and the cone is imaginary. The apex of the cone is not on the planes, so intersection is empty.

The two planes of R ( λ 1 ) are real if the matrix has inertia (1 , 1) , i.e. , if a = -1 . The cone R ( λ 2 ) is real if its inertia is (2 , 1) , i.e. , if b = -1 or c = -1 . The two conics of the intersection are secant over the reals if the singular line z = w = 0 of the pair of planes meets the cone in real points, i.e. , if b = -1 . We have the following cases:

- a = 1 , b = 1 , c = -1 : The planes are imaginary and the cone is real. Their real intersection is the intersection of the singular line z = w = 0 of the pair of planes with the cone. The real intersection is thus empty.
- a = 1 , b = -1 , c = ± 1 : The planes are imaginary and the cone is real. The line z = w = 0 intersects the cone in two points of coordinates (1 , 1 , 0 , 0) and ( -1 , 1 , 0 , 0) . The intersection is reduced to these two points.
- a = -1 , b = 1 , c = -1 : The planes and the cone are real. The line z = w = 0 does not intersect the cone, so intersection consists of two non-secant conics.
- a = -1 , b = -1 , c = ± 1 : The planes and the cone are real. The line z = w = 0 intersects the conics. The intersection consists of two conics intersecting in two points p ± of coordinates ( ± 1 , 1 , 0 , 0) . All the quadrics of the pencil have the same tangent plane P ± : x ∓ y = 0 at p ± . The two conics of the intersection are on the same side of P ± , i.e. , the singularity is convex. When c = 1 , the plane y = 0 does not intersect the conics. The same goes for the plane x = 0 when c = -1 . We conclude that the intersection is affinely finite.

Computing the inertia of R ( λ 1 ) gives a . Also, in normal form, the characteristic polynomial det( λR ( λ 1 ) + µR ( λ 2 )) is equal to bµ 2 λ ( aλ + cµ ) . Thus, by Lemma 2, ab is equal to the sign of det( λS + µT ) ( µ 1 λ -λ 1 µ ) 2 at ( λ 1 , µ 1 ) . Hence we can easily compute a and b . Finally, we need to compute c but only in the case where a = -1 and b = 1 . Then c = 1 if the inertia of R ( λ 2 ) (or R ( λ 3 ) ) is (3 , 0) ; otherwise c = -1 and the inertia of R ( λ 2 ) (or R ( λ 3 ) ) is (2 , 1) .

λ 2 and λ 3 are complex conjugate. There are two complex Jordan blocks of size 2 associated with the two roots. The pencil normal form is obtained as in Section 3.1. The end result is the two quadrics zw = 0 and x 2 + ay 2 + z 2 -w 2 = 0 with a ∈ {-1 , 1 } .

- a = 1 : One conic is real, the other is imaginary.

The pair of planes R ( λ 1 ) is always real. The intersection consists of the two conics z = x 2 + ay 2 -w 2 = 0 and w = x 2 + ay 2 + z 2 = 0 . We have two cases:

- a = -1 : The two conics are real. They intersect at the points p ± of coordinates (1 , ± 1 , 0 , 0) . All the quadrics of the pencil have the same tangent plane P ± : x ∓ y = 0 at p ± . The two ± i.e.
- conics of the intersection are on opposite sides of P , , the singularity is concave. 2

̸

Let us show that the intersection is affinely infinite. Parameterizing the first conic by ( u + v 2 , u 2 -v 2 , 0 , 2 uv ) and the second by ( u 2 -v 2 , u 2 + v 2 , 2 uv, 0) , and intersecting with the plane of equation ℓ 1 x + ℓ 2 y + ℓ 3 z + ℓ 4 w = 0 , ( ℓ 1 , ℓ 2 , ℓ 3 , ℓ 4 ) ∈ P 3 ( R ) , we obtain two quadratic equations in ( u, v ) . The product of the coefficients of u 2 and v 2 is ℓ 2 1 -ℓ 2 2 in one case and ℓ 2 2 -ℓ 2 1 in the other case. Therefore, Descartes' Sign Rule implies the existence of a real non-trivial solution to at least one of the quadratic equations if ℓ 2 1 = ℓ 2 2 . If ℓ 2 1 = ℓ 2 2 , then each of the two quadratic equations has either ( u, v ) = (0 , 1) or (1 , 0) as real non-trivial solution. We conclude that the intersection is cut by any plane of P 3 ( R ) , i.e. , it is affinely infinite.

Note finally that, in normal form, the characteristic polynomial det ( λR ( λ 1 ) + µR ( λ 2 )) is equal to -aµ 2 ( µ 2 + λ 2 / 4) . Hence a is opposite to the sign of det( λS + µT ) ( µ 1 λ -λ 1 µ ) 2 at ( λ 1 , µ 1 ) (by Lemma 2).

```
3.3. Cuspidal quartic in P 3 ( C ) , σ 4 = [13]
```

The characteristic polynomial has a triple root λ 1 , which is necessarily real. To it corresponds a real Jordan block of size 3. R ( λ 1 ) has rank 3. Let λ 2 be the other root, necessarily real, and R ( λ 2 ) the associated cone. The normal forms of R ( λ 1 ) and R ( λ 2 ) are w 2 + yz = 0 and y 2 + xz = 0 . The intersection consists of a cuspidal quartic which can be parameterized (in the local coordinate system of the normal form) by X ( u, v ) = ( v 4 , u 2 v 2 , -u 4 , u 3 v ) with ( u, v ) ∈ P 1 ( R ) . The quartic has a cusp at p = (1 , 0 , 0 , 0) (the vertex of the first cone), which corresponds to ( u, v ) = (0 , 1) . The intersection of R ( λ 1 ) with the plane tangent to R ( λ 2 ) at p gives the (double) line tangent to the quartic at p , i.e. , z = w 2 = 0 .

```
3.4. Two tangent conics in P 3 ( C ) , σ 4 = [1(21)]
```

The characteristic polynomial has a triple root λ 1 and the rank of R ( λ 1 ) is 2. λ 1 is necessarily real. Attached to λ 1 are two real Jordan blocks, one of size 2, the other of size 1. Let λ 2 be the other simple real root, with R ( λ 2 ) of rank 3. The normal forms of R ( λ 1 ) and R ( λ 2 ) are x 2 + aw 2 = 0 and xy + z 2 = 0 with a ∈ {-1 , 1 } .

The pair of planes R ( λ 1 ) is real when the matrix has inertia (1 , 1) , i.e. , when a = -1 . The cone R ( λ 2 ) is real since its inertia is (2 , 1) . So we have two cases:

- a = 1 : The pair of planes is imaginary. Its real part is restricted to the line x = w = 0 , which intersects the cone in the real double point (0 , 1 , 0 , 0) . The intersection is reduced to that point.
- a = -1 : The planes are real. The intersection consists of two conics intersecting in the double point (0 , 1 , 0 , 0) and sharing a common tangent at that point.

## 3.5. Double conic in P 3 ( C ) , σ 4 = [1(111)]

The characteristic polynomial has a real triple root λ 1 and the rank of R ( λ 1 ) is 1. The Jordan normal form of S -1 T contains three blocks of size 1 for λ 1 . Let λ 2 be the other real root, with R ( λ 2 ) of rank 3. The normal forms of R ( λ 1 ) and R ( λ 2 ) are w 2 = 0 and x 2 + ay 2 + z 2 = 0 with a ∈ {-1 , 1 } . The cone R ( λ 2 ) is real if its inertia is (2 , 1) , i.e. , if a = -1 . We have two cases:

- a = 1 : The cone is imaginary. Its real apex does not lie on the plane w = 0 , so the intersection is empty.
- a = -1 : The cone is real. The intersection consists of a double conic lying in the plane w = 0 .

## 3.6. Cubic and tangent line in P 3 ( C ) , σ 4 = [4]

The characteristic polynomial has a quadruple root λ 1 and the rank of R ( λ 1 ) is 3. λ 1 is necessarily real. Associated with it is a unique real Jordan block of size 4. The normal forms of R ( λ 1 ) and S are z 2 + yw = 0 and xw + yz = 0 . The intersection contains the line z = w = 0 . The cubic is parameterized by X ( u, v ) = ( u 3 , -u 2 v, uv 2 , v 3 ) with ( u, v ) ∈ P 1 ( R ) . The cubic intersects the line in the point of coordinate (1 , 0 , 0 , 0) , corresponding to the parameter (1 , 0) . The cubic and the line are tangent at that point.

## 3.7. Conic and two lines crossing on the conic in P 3 ( C ) , σ 4 = [(31)]

The characteristic polynomial has a quadruple root λ 1 , with R ( λ 1 ) of rank 2. λ 1 is necessarily real. To it correspond two real Jordan blocks of size 3 and 1. The normal forms of R ( λ 1 ) and S are yz = 0 and y 2 + xz + aw 2 = 0 with a ∈ {-1 , 1 } .

z = 0 and y 2 + aw 2 = 0 define two real or imaginary lines that intersect at point p = (1 , 0 , 0 , 0) . y = 0 and xz + aw 2 = 0 define a real conic which contains p . We have two cases:

- a = 1 : The lines are imaginary except for their intersection point on the conic. The intersection is reduced to the conic.
- a = -1 : The lines are real. The intersection consists of a conic and two lines crossing on the conic at p .

The characteristic polynomial in normal form det ( λR ( λ 1 ) + µS ) = -aµ 4 / 4 has a quadruple root and thus is always non-negative or non-positive. In this case, it is straightforward to show, similarly as in the proof of Lemma 2, that the sign ⩾ 0 or ⩽ 0 of det ( λS + µT ) is invariant by real projective transformation and independent of the choice of S and T in the pencil. Hence a is opposite to the sign of det ( λS + µT ) for any ( λ, µ ) that is not the quadruple root.

We also have the following straightforward lemma which we use for differentiating between the two cases in which the Segre characteristic is [(31)] or [(22)] and the characteristic polynomial has a quadruple root which corresponds to a pair of planes (see Lemmas 5 and 6).

Lemma 5. If the Segre characteristic of the pencil is [(31)] , then the quadruple root of the characteristic polynomial D ( λ, µ ) corresponds to a quadric of inertia (1 , 1) (a pair of planes) whose singular line is not contained in any other quadric of the pencil.

## 3.8. Two lines and a double line in P 3 ( C ) , σ 4 = [(22)]

The characteristic polynomial has a quadruple root λ 1 , with R ( λ 1 ) of rank 2. λ 1 is necessarily real and there are two real Jordan blocks associated with it, both of size 2. The normal forms of R ( λ 1 ) and S are y 2 + aw 2 = 0 and xy + azw = 0 with a ∈ {-1 , 1 } . The intersection consists of the double line y = w = 0 and two simple lines y ± √ -aw = x ± √ -az = 0 cutting the double line in the points ( ∓ √ -a, 0 , 1 , 0) .

The pair of planes is real if its inertia is (1 , 1) , i.e. , if a = -1 . We have two cases:

- a = 1 : The two simple lines are imaginary. The intersection is the double line y = w = 0 .
- a = -1 : The intersection consists of the two simple lines y ± w = x ± z = 0 and the double line y = w = 0 .

Note that the characteristic polynomial det( λR ( λ 1 ) + µS ) is equal in normal form to a 2 µ 4 16 . Thus D ( λ, µ ) is positive for any ( λ, µ ) distinct from the quadruple root. Note also that, in the case where a = -1 , the singular line y = w = 0 of the pair of planes y 2 -w 2 = 0 is contained in all the quadrics of the pencil; we thus have the following lemma.

Lemma 6. If the Segre characteristic of the pencil is [(22)] and the quadric corresponding to the quadruple root of the characteristic polynomial D ( λ, µ ) is of inertia (1 , 1) (a pair of planes), then its singular line is contained in all the quadrics of the pencil.

## 3.9. Two double lines in P 3 ( C ) , σ 4 = [(211)]

The characteristic polynomial has a quadruple root λ 1 , with R ( λ 1 ) of rank 1. λ 1 is real and there are three real Jordan blocks associated with it, two having size 1 and the last size 2. The normal forms of R ( λ 1 ) and S are w 2 = 0 and x 2 + ay 2 + zw = 0 with a ∈ {-1 , 1 } . The intersection consists of two double lines w 2 = x 2 + ay 2 = 0 . There are two cases:

- a = 1 : The two double lines are imaginary. The intersection is reduced to their real intersection point, i.e. , (0 , 0 , 1 , 0) .
- a = -1 : The two double lines w 2 = x -y = 0 and w 2 = x + y = 0 are real so the intersection consists of these two lines, meeting at (0 , 0 , 1 , 0) .

The characteristic polynomial (in normal form) is equal to det( λR ( λ 1 ) + µS ) = -aλ 4 / 4 thus a is opposite to the sign of det( λS + µT ) for any ( λ, µ ) that is not a root.

<!-- formula-not-decoded -->

The characteristic polynomial has two double roots λ 1 and λ 2 . The associated quadrics both have rank 3. λ 1 and λ 2 are either both real or complex conjugate.

λ 1 and λ 2 are real. There is a real Jordan block of size 2 associated with each root. The normal forms of R ( λ 1 ) and R ( λ 2 ) are y 2 + zw = 0 and xy + w 2 = 0 . The intersection consists of the line y = w = 0 and a cubic. The cubic is parameterized by X ( u, v ) = ( u 3 , -uv 2 , -v 3 , u 2 v ) with ( u, v ) ∈ P 1 ( R ) . The line intersects the cubic in the two points of coordinates (1 , 0 , 0 , 0) and (0 , 0 , 1 , 0) , corresponding to the parameters (1 , 0) and (0 , 1) .

̸

λ 1 and λ 2 are complex conjugate. Let λ 1 = α +i β, λ 2 = λ 1 , β = 0 . There is complex Jordan block of size 4 associated with the two roots. The normal forms of S and R ( α ) are xw + yz = 0 and xz -yw + zw = 0 . The intersection contains the line z = w = 0 . The cubic is parameterized by X ( u, v ) = ( -u 2 v, uv 2 , u 3 + uv 2 , u 2 v + v 3 ) with ( u, v ) ∈ P 1 ( R ) . The cubic intersects the line in the points of coordinates (1 , i , 0 , 0) and (1 , -i , 0 , 0) . Thus, over the reals, the cubic and the line do not intersect.

## 3.11. Conic and two lines not crossing on the conic in P 3 ( C ) , σ 4 = [2(11)]

The characteristic polynomial has two double roots λ 1 and λ 2 , with associated ranks 3 (a projective cone) and 2 (a pair of planes) respectively. The two roots are necessarily real, otherwise the ranks of the quadrics R ( λ 1 ) and R ( λ 2 ) would be the same. Associated with λ 1 and λ 2 are respectively a unique real Jordan block of size 2 and two real Jordan blocks of size 1. The pencil normal form is given by y 2 + az 2 + bw 2 = 0 and xy = 0 with a, b ∈ {-1 , 1 } .

The plane x = 0 contains a conic which is real when a = -1 or b = -1 and imaginary otherwise. The plane y = 0 contains two lines which are real if ab &lt; 0 and imaginary otherwise. The lines cross at the point (1 , 0 , 0 , 0) , the apex of R ( λ 1 ) , which is not on the conic.

The pair of planes R ( λ 2 ) is always real. The cone R ( λ 1 ) is real when its inertia is (2 , 1) , i.e. , when a = -1 or b = -1 . We have three cases:

- a = 1 , b = 1 : The lines and the conic are imaginary. The intersection is reduced to the real point of intersection of the two lines, that is (1 , 0 , 0 , 0) .
- a = -b : The lines and the conic are real. The intersection consists of a conic and two intersecting lines, each cutting the conic in a point (at (0 , 0 , 1 , 1) and (0 , 0 , -1 , 1) ).
- a = -1 , b = -1 : The lines are imaginary, the conic is real. The intersection consists of a conic and the point (1 , 0 , 0 , 0) , intersection of the two lines.

To determine in which of the three situations we are, first compute the inertia of R ( λ 1 ) . If the inertia is (3 , 0) , this implies that a = b = 1 . Otherwise, we consider as before the characteristic polynomial in normal form det ( λR ( λ 1 ) + µR ( λ 2 )) = -abλ 2 µ 2 / 4 and it is straightforward to show that -ab is the sign of det( λS + µT ) for any ( λ, µ ) that is not a root.

## 3.12. Four lines forming a skew quadrilateral in P 3 ( C ) , σ 4 = [(11)(11)]

The characteristic polynomial has two double roots λ 1 and λ 2 , with associated quadrics of rank 2. λ 1 and λ 2 can be either both real or both complex conjugate.

λ 1 and λ 2 are real. Each root appears in two real Jordan blocks of size 1. The normal forms of R ( λ 1 ) and R ( λ 2 ) are z 2 + aw 2 = 0 and x 2 + by 2 = 0 with a, b ∈ {-1 , 1 } .

The first pair of planes is imaginary if a = 1 . The second pair of planes is imaginary if b = 1 . There are three cases:

- a = 1 , b = 1 : The four lines are imaginary and the intersection is empty.
- a = -1 , b = -1 : The four lines are real. The intersection consists of four lines forming a skew quadrilateral.
- a = -b : One pair of planes is real, the other is imaginary. If a = 1 , the intersection consists of the points of intersection of the line z = w = 0 with the pair of planes x 2 -y 2 = 0 , that is the points (1 , 1 , 0 , 0) and ( -1 , 1 , 0 , 0) . Similarly, if b = 1 the intersection is reduced to the two points (0 , 0 , 1 , 1) and (0 , 0 , -1 , 1) .

The values of a and b follow from the inertia of R ( λ 1 ) and R ( λ 2 ) . Note also that b directly follows from a because, the characteristic polynomial (in normal form) det( λR ( λ 1 ) + µR ( λ 2 )) = abλ 2 µ 2 and, as before, ab is equal to the sign of det( λS + µT ) for any ( λ, µ ) that is not a root.

̸

λ 1 and λ 2 are complex conjugate. Let λ 1 = a + i b, λ 2 = λ 1 , b = 0 . The roots appear in two complex Jordan blocks of size 2. The normal forms of S and aS -T are xy + zw = 0 and x 2 -y 2 + z 2 -w 2 = 0 . The intersection consists of two real lines of equations x ± w = y ∓ z = 0 and two imaginary lines of equations x ± i z = y ∓ i w = 0 .

## 4. Classification of singular pencils over the reals

We now examine the singular pencils of P 3 ( R ) , i.e. , those whose characteristic polynomial vanishes identically. A summary of this classification is given in Table 2. There are two cases according to whether two arbitrary quadrics of the pencil have a singular point in common.

## 4.1. Q S and Q T have no singular point in common, σ 4 = [1 { 3 } ]

We first prove the following lemma.

Lemma 7. If det R ( λ, µ ) ≡ 0 and Q S and Q T have no singular point in common, then every quadric of the pencil has a singular point p that belongs to all the other quadrics of the pencil which, furthermore, share a common tangent plane at p .

Proof. Let Q R be any quadric of the pencil. First note that R has rank at most 3, otherwise the characteristic polynomial would not identically vanish.

If R has rank 1, it is a double plane in P 3 ( C ) containing only singular points. Since there is no quadric of inertia (4 , 0) in the pencil, the intersection of the double plane with every other quadric of the pencil is not empty in P 3 ( R ) (by Theorem I.5). Hence Q R contains a singular point that belongs to all the quadrics of the pencil.

If R has rank 2, it is a pair of planes in P 3 ( C ) with a real singular line. By the Segre classification (see Table 4) we know that the intersection in P 3 ( C ) contains a conic and a double line. Furthermore, the line is necessarily real because otherwise its conjugate would also be in the intersection. This line lies in one of the two planes of Q R and thus cuts any other line in that plane and in particular the singular line of the pair of planes. Hence Q R contains a singular point that belongs to all the quadrics of the pencil.

̸

If R has rank 3, we apply a congruence transformation so that Q R has the diagonal form ax 2 + by 2 + cz 2 = 0 , with abc = 0 . We also change the generators of the pencil, replacing S by R . After these transformations, the determinant D ( λ, µ ) becomes the sum of δabcλ 3 µ and of other terms of degree at least 2 in µ , where δ is the coefficient of w 2 in the equation of Q T . The hypothesis that D ( λ, µ ) ≡ 0 thus implies that δ = 0 . Hence the singular point (0 , 0 , 0 , 1) of Q R belongs to Q T and thus to all the quadrics of the pencil.

̸

Thus, in all cases, any quadric Q R of the pencil has a singular point, p , that belongs to all the quadrics of the pencil. Furthermore, every quadric Q R ′ = Q R of the pencil is not singular at p because, otherwise, p would be a singular point of all the quadrics of the pencil, contradicting the hypotheses.

̸

̸

̸

̸

Finally, let J R and J R | p denote, respectively, the matrix of partial derivative ( ∂Q R ∂x , ∂Q R ∂y , ∂Q R ∂z , ∂Q R ∂w ) and its evaluation at p . There exists ( λ 0 , µ 0 ) ∈ P 1 ( R ) such that R = λ 0 S + µ 0 T and since Q R is singular at p , we have J R | p = λ 0 J S | p + µ 0 J T | p = 0 . Assume, for simplicity, that Q R = Q S and thus that µ 0 = 0 . For any quadric Q R ′ = Q R of the pencil, there exists ( λ, µ ) = ( λ 0 , µ 0 ) in P 1 ( R ) such that R ′ = λS + µT and thus J R ′ | p = λJ S | p + µJ T | p = λµ 0 -µλ 0 µ 0 J S | p = 0 . Thus, the tangent plane of Q R ′ at p has equation ( x, y, z, w ) · J R ′ | p = λµ 0 -µλ 0 µ 0 ( x, y, z, w ) · J S | p = 0 . It follows that all quadrics Q R ′ = Q R in the pencil have the same tangent plane at p . ✷

̸

By Lemma 7, there exist a singular point s of Q S and a singular point t of Q T that belong to all the quadrics Q λS + µT of the pencil. Quadrics Q S , Q T , and Q S + T have rank at most 3 since the characteristic polynomial identically vanishes, and they are not of inertia (3 , 0) (see Table I.1) since they contain s and t that are distinct by assumption. Hence Q S , Q T , and Q S + T are cones or pairs of (possibly complex) planes. Thus, since s and t are singular points of Q S and Q T , respectively, the line st is entirely contained in Q S and Q T , and thus is also contained in Q S + T . Moreover, s and t are not singular points of Q S + T because otherwise all the quadrics of the pencil would be singular at these points, contradicting the hypothesis. It now follows from the

̸

fact that Q S + T is a cone or a pair of planes that its tangent planes at s and t coincide. Therefore, by Lemma 7, the tangent plane of Q S at t is the same as the tangent plane of Q T at s .

Now we change the coordinate system in such a way that s and t have coordinates (0 , 0 , 0 , 1) and (0 , 0 , 1 , 0) and that the common tangent plane has equation x = 0 . Then the equations of Q S and Q T become xz + q 1 ( x, y ) = 0 and xw + q 2 ( x, y ) = 0 , where q 1 and q 2 are binary quadratic forms. The two equations can thus be expressed in the form ay 2 + x ( by + cx + z ) = 0 and a ′ y 2 + x ( b ′ y + c ′ x + w ) = 0 . By a new change of coordinate system, we get equations of the form ay 2 + xz = 0 and a ′ y 2 + xw = 0 . Replacing the second quadric by a linear combination of the two and applying the change of coordinates a ′ z -aw → w and a scaling on y , gives, as normal form for the pencil, the two quadrics xw = 0 and xz + ay 2 = 0 with a ∈ {-1 , 1 } . Furthermore, we can set a = 1 by changing z in -z .

Therefore, the intersection consists of the double line x = y 2 = 0 and the conic w = xz -y 2 = 0 . The line and the conic meet at (0 , 0 , 1 , 0) in the local coordinate system.

## 4.2. Q S and Q T have a singular point in common

Let p be the common singular point. After a rational change of coordinate system, we may suppose that p has coordinates (0 , 0 , 0 , 1) . In the new frame, the equations of the quadrics are homogeneous polynomials of degree 2 in three variables. To classify the different types of intersection, we may identify the quadrics with their upper left 3 × 3 matrices and look at the multiple roots of the cubic characteristic polynomial, which we refer to as the restricted characteristic polynomial , and the ranks of the associated matrices. We thus apply the Canonical Pair Theorem to pairs of conics.

The case [(111)] is trivial and left aside: in that situation, the restricted characteristic polynomial has a (real) triple root, the associated quadric has rank 0 and all the other quadrics of the pencil are cones. The intersection consists of any cone of the pencil, that is any quadric of the pencil distinct from P 3 ( R ) .

## 4.2.1. Four concurrent lines in P 3 ( C ) , σ 3 = [111]

The restricted characteristic polynomial has three simple roots. At least one is real: call it λ 1 . Let λ 2 be another root. To these roots correspond quadrics of rank 2.

If λ 2 is real, then the three roots are real. The normal forms of R ( λ 1 ) and R ( λ 2 ) are ay 2 + z 2 = 0 and bx 2 + z 2 = 0 with a, b ∈ {-1 , 1 } . Note that the equation of the third pair of planes of the pencil is obtained by subtracting the two equations, giving ay 2 -bx 2 = 0 . We have two cases:

- a = b = 1 or a = -b : When a = b = 1 , both R ( λ 1 ) and R ( λ 2 ) represent imaginary pairs of planes. When a = -b , the third pair of planes is imaginary, as well as one of the first two. In both cases, the intersection is reduced to the point p .
- a = b = -1 : The intersection consists of four concurrent lines y -ε 1 z = x -ε 2 z = 0 , with ε 1 , ε 2 ∈ {-1 , 1 } , meeting at p .

Both a and b are equal to -1 if and only if R ( λ 1 ) and R ( λ 2 ) have inertia (1 , 1) .

̸

If λ 2 = α +i β is complex, β = 0 , we obtain as normal form (proceeding as in Section 3.1) the two quadrics x 2 + y 2 -z 2 = 0 and yz = 0 . The intersection consists of two real lines y = x 2 -z 2 = 0 , intersecting at p , and two complex lines z = x 2 + y 2 = 0 .

## 4.2.2. Two lines and a double line in P 3 ( C ) , σ 3 = [12]

The restricted characteristic polynomial has a double root λ 1 , which is real. The Jordan normal form of S -1 T contains one real Jordan block of size 2. Let λ 2 be the other root, also real. The normal forms of R ( λ 1 ) and R ( λ 2 ) are y 2 + az 2 = 0 and xy = 0 with a ∈ {-1 , 1 } . There are two cases:

- a = -1 : The intersection consists of the double line y = z 2 = 0 and the two simple lines x = y -z = 0 and x = y + z = 0 . The three lines are concurrent at p .
- a = 1 : The two simple lines are imaginary. Their common point is on the double line, so the intersection consists of this double line y = z 2 = 0 .

Note that the value of a follows from the inertia of R ( λ 1 ) .

## 4.2.3. Two double lines in P 3 ( C ) , σ 3 = [1(11)]

The restricted characteristic polynomial has a double root λ 1 , which is real. The canonical pair form has two real Jordan blocks of size 1 associated with λ 1 . Let λ 2 be the other root, also real. The normal forms of R ( λ 1 ) and R ( λ 2 ) are z 2 = 0 and x 2 + ay 2 = 0 with a ∈ {-1 , 1 } . The pair of planes R ( λ 2 ) is real when its inertia is (1 , 1) , i.e. , when a = -1 . We have two cases:

- a = -1 : The intersection consists of the two double lines x -y = z 2 = 0 and x + y = z 2 = 0 , meeting at p .
- a = 1 : The intersection is reduced to the point p .

Note that the value of a follows from the inertia of R ( λ 2 ) .

## 4.2.4. Line and triple line in P 3 ( C ) , σ 3 = [3]

The restricted characteristic polynomial has a triple root λ 1 , which is real. The Jordan normal form of S -1 T contains one real Jordan block of size 3. The normal forms of S and R ( λ 1 ) are xz + y 2 = 0 and yz = 0 . The intersection consists of the triple line z = y 3 = 0 and the simple line x = y = 0 . The two lines cut at p , the singular point of all the quadrics of the pencil.

## 4.2.5. Quadruple line in P 3 ( C ) , σ 3 = [(21)]

The restricted characteristic polynomial has a real triple root λ 1 . The canonical pair form has two real Jordan blocks of size 2 and 1. The normal forms of R ( λ 1 ) and S are y 2 = 0 and z 2 + xy = 0 . The intersection consists of the quadruple line y 2 = z 2 = 0 .

## 4.2.6. σ 3 = [ { 3 } ] and remaining cases

In this case, the restricted characteristic polynomial identically vanishes. One can easily prove that if the two conics S and T have no two singular point in common, the pencil can be put in the normal form λxy + µxz . The intersection consists of the plane x = 0 and the line y = z = 0 , which meets the plane at p .

If the two conics have a singular point in common (call it q ), we can go from 3 × 3 matrices to 2 × 2 matrices, similarly as above, by sending q to (0 , 0 , 1 , 0) . Consider the new characteristic polynomial, a quadratic polynomial. The cases are:

- Two simple real roots: The pencil can be put in the normal form λx 2 + µy 2 . The intersection consists of the quadruple line x 2 = y 2 = 0 which goes through p and q .
- Two simple complex roots: A normal form for the pencil is λxy + µ ( x 2 -y 2 ) , giving the quadruple line x 2 = y 2 = 0 for the intersection.
- A double real root, with a real Jordan block of size 2: The normal form is λxy + µy 2 . The intersection consists of the plane y = 0 .
- Vanishing quadratic equation: The intersection consists of a double plane.

## 5. Algorithmic issues

We assume here that the two input quadrics Q S and Q T have rational coefficients and show how the type of their intersection in P 3 ( R ) can be deduced from the classification of Tables 1 and 2 with computations involving only rational numbers. This type-detection algorithm has been implemented in C++, using arbitrary-precision integer arithmetic, as part of our implementation for parameterizing the intersection of two quadrics (Lazard et al., 2006). Note that detailed pseudo-code can be found in Dupont et al. (2005). We consider separately regular and singular pencils.

## 5.1. Regular pencils: D ( λ, µ ) ̸≡ 0

First note that the coefficients of the polynomial D ( λ, µ ) = det( λS + µT ) are rational since S and T have rational coefficients. Note also that the number and multiplicity of the multiple roots of D ( λ, µ ) follow from the degree of the gcd of the partial derivatives of the characteristic polynomial ( i.e. , gcd( ∂ D ∂λ , ∂ D ∂µ ) ) and from the sign of its discriminant (when it is of degree two). All these computations only involve rational numbers.

Notice also that, if D ( λ, µ ) admits a unique multiple root, then it is necessarily real and rational because, otherwise, its (algebraic or complex) conjugate would also be a multiple root. Furthermore, such a multiple root can easily be computed from the gcd of the partial derivatives. Also, the type, real or complex, of another root (if any) of D ( λ, µ ) and the sign s of D ( λ, µ ) in the neighborhood of the multiple root can easily be computed.

Notice finally that the inertia of a quadric Q R with rational coefficients can easily be computed as follows. Let p ( ω ) = det ( R -ωI ) , where I is the identity matrix. Since R is symmetric, all its eigenvalues are real. Thus, Descartes' Sign Rule on p ( ω ) and p ( -ω ) gives the exact numbers of positive and negative eigenvalues.

We now show how the type of the intersection of Q S and Q T in P 3 ( R ) follows from Table 1. We consider different cases according to the number of multiple roots of D ( λ, µ ) and treat first the case of empty intersection.

Empty intersection. First we check, as described in Section I.4.3, whether the intersection is empty. Recall that this can be done by computing separating intervals of the roots of D ( λ, µ ) and testing whether there exists a quadric of inertia (4 , 0) among quadrics of the pencil corresponding to some rational values ( λ, µ ) in between the separating intervals of D ( λ, µ ) . We can thus assume in the following that the intersection is not empty.

No multiple root. Since the intersection is not empty, it is a smooth quartic. Note, furthermore, that the number of connected components of the quartic and whether it is affinely finite or infinite follows from the number of real roots of D ( λ, µ ) (see Tu et al. (2002) or Theorem I.25).

One multiple root. Since the intersection is not empty, the type of the intersection follows from the multiplicity of the multiple root, the inertia of the associated quadric, the type (real or complex) of another root, and s , except in the case where there is one quadruple root whose associated quadric has inertia (1 , 1) (a pair of planes) and s = + . Then, we need to determine whether the singular line of the pair of planes is contained in any other quadric of the pencil.

Computing the singular locus of a quadric amounts to computing the kernel of the associated matrix. Thus determining whether the singular line of a quadric of inertia (1 , 1) with rational coefficients lies in any given other quadrics of the pencil does not involve non-rational numbers.

Two double roots. Note first that if the two double roots are rational, we can easily compute, as before, the inertia of the corresponding quadrics and the sign s , which determine the type of the intersection.

Assume now that the double roots are not rational. Then they are (algebraic or complex) conjugate; thus, the two associated quadrics are also conjugate and they have the same rank. We describe below how to compute the rank of such quadrics without involving irrational numbers. Then, the type of the intersection follows from the rank of these quadrics, the type, real or complex, of the multiple roots, and the sign s of D ( λ, µ ) outside the roots.

<!-- formula-not-decoded -->

Computing the rank of the quadrics associated with two non-rational double roots of D ( λ, µ ) . The roots have the form ( λ 0 , µ 0 ) = ( a, b ± √ c ) , where ( a, b, c ) ∈ P 2 ( Q ) , c is not a square and c is either positive (the two roots are algebraic conjugate) or negative (the two roots are complex conjugate). The matrices associated to the roots have the form R = aS + ( b ± √ c ) T . We can furthermore assume that a is non-zero because, otherwise, the inertia of R is the inertia of T which has rational coefficients. Consider now the 8 × 8 rational matrix

Suppose that the vector k 1 + √ c k 2 is in the kernel of R (where k 1 and k 2 have rational coordinates). Then ( aS + bT ) k 1 ± cT k 2 ± √ c ( T k 1 ± ( aS + bT ) k 2 ) = 0 and thus ( aS + bT ) k 1 ± cT k 2 = T k 1 ± ( aS + bT ) k 2 = 0 . Hence, the column vector ( k T 1 , ± k T 2 ) T is in the kernel of M . Similarly, the vector ( ± c k T 2 , k T 1 ) T is also in the kernel of M . Conversely, if one of these two vectors is in the kernel of M then k 1 + √ c k 2 is in the kernel of R . It follows that the dimension of the kernel of M is twice the dimension of the kernel of R . We can thus conclude that the rank of R is 4 -1 2 dimker M .

## 5.2. Singular pencils: D ( λ, µ ) ≡ 0

We now show how the type of the intersection of Q S and Q T follows from Table 2 when D ( λ, µ ) vanishes identically. First, we determine if Q S and Q T have a singular point in common. As mentioned above, determining the singular locus of a quadric amounts to computing the kernel of the corresponding matrix. Determining the common singular locus of Q S and Q T thus amounts to intersecting the two linear spaces corresponding to the kernels of S and T . These computations do not involve irrational numbers.

If Q S and Q T do not have a common singular point, the intersection is a conic and a double line. Otherwise, if they have a singular point p in common, we send it to (0 , 0 , 0 , 1) and work with the pencil of conics living in the plane w = 0 . To actually compute the restricted pencil R 3 ( λ, µ ) , we build the matrix of a real projective transformation P obtained by putting p as the last column and completing P so that its columns form a basis of P 3 ( R ) . R 3 ( λ, µ ) is then the principal submatrix of the matrix P T ( λS + µT ) P and D 3 ( λ, µ ) = det R 3 ( λ, µ ) .

We first consider the case where D 3 ( λ, µ ) does not identically vanish. First note that any multiple root of D 3 ( λ, µ ) is rational because otherwise its (algebraic or complex) conjugate would also be a multiple root. Hence, if D 3 ( λ, µ ) admits a multiple root, all the roots are rational; they can be computed without involving irrational numbers, as well as the inertias of the corresponding quadrics which induce the type of the intersection. On the other hand, if D 3 ( λ, µ ) admits no multiple root, we proceed as follows. We apply Finsler's Theorem (see Theorem I.5) on the restricted pencil R 3 ( λ, µ ) in the same way that we tested empty intersection in the previous subsection: if the restricted pencil contains a quadric of inertia (3 , 0) , the intersection of the pencil of conics in the plane w = 0 is empty and thus the intersection of the two initial quadrics is reduced to p . Otherwise, the type, real or complex, of the roots of D 3 ( λ, µ ) determines the type of the intersection. Finally, when D 3 ( λ, µ ) vanishes identically, Table 5 gives, similarly as above, the type of the intersection (which is here the same in P 3 ( R ) and P 3 ( C ) ).

## 6. Conclusion

We have presented a classification of pencils of quadrics based on the type of their intersection in real projective space. This classification, which is derived from the Canonical Pair Form Theorem for pairs of real symmetric matrices (Uhlig, 1973, 1976), is at the core of the design of the algorithm, presented in the third part of this paper, for computing near-optimal rational parameterizations of the intersection of pairs of quadrics in all singular cases.

## References

- Bromwich, T., 1906. Quadratic Forms and Their Classification by Means of Invariant Factors. Cambridge Tracts in Mathematics and Mathematical Physics.
- Dupont, L., Lazard, D., Lazard, S., Petitjean, S., Sept. 2005. Near-optimal parameterization of the intersection of quadrics: II. A classification of pencils. Research Report n o 5668, INRIA, 37 pages.
- Finsler, P., 1936/1937. Über das Vorkommen definiter und semidefiniter Formen in Scharen quadratischer Formen. Comment. Math. Helv. 9, 188-192.
- Hodge, W., Pedoe, D., 1953. Methods of Algebraic Geometry. Cambridge University Press, volumes I and II.
- Lazard, S., Peñaranda, L. M., Petitjean, S., 2006. Intersecting quadrics: An efficient and exact implementation. Computational Geometry: Theory and Applications 35 (1-2), 74-99, special issue of the 20th ACM Symposium on Computational Geometry, 2004.
- Ocken, S., Schwartz, J. T., Sharir, M., 1987. Precise implementation of CAD primitives using rational parametrization of standard surfaces. In: Schwartz, J. T., Sharir, M., Hopcroft, J. (Eds.), Planning, Geometry, and Complexity of Robot Motion. Ablex Publishing, pp. 245-266.
- Segre, C., 1883. Studio sulle quadriche in uno spazio lineare ad un numero qualunque di dimensioni. Mem. della R. Acc. delle Scienze di Torino 36 (2), 3-86.
- Surf, 2000-2003. A Tool To Visualize Algebraic Curves and Surfaces. Stephan Endrass, Hans Huelf, Ruediger Oertel, Ralf Schmitt, Kai Schneider and Johannes Beigel, http://surf. sourceforge.net .
- Tu, C., Wang, W., Mourrain, B., Wang, J., September 2005. Signature sequence of intersection curve of two quadrics for exact morphological classification. Tech. Rep. TR-2005-09, The University of Hong-Kong.
- Tu, C., Wang, W., Wang, J., 2002. Classifying the nonsingular intersection curve of two quadric surfaces. In: Proc. of GMP'02 (Geometric Modeling and Processing). pp. 23-32.
- Uhlig, F., 1973. Simultaneous block diagonalization of two real symmetric matrices. Linear Algebra and Its Applications 7, 281-289.
- Uhlig, F., 1976. A canonical form for a pair of real symmetric matrices that generate a nonsingular pencil. Linear Algebra and Its Applications 14, 189-209.
- Wang, W., Krasauskas, R., 2004. Interference analysis of conics and quadrics. In: Goldman, R., Krasauskas, R. (Eds.), Topics in Algebraic Geometry and Geometric Modeling: Workshop on Algebraic Geometry and Geometric Modeling, July 29-August 2, 2002, Vilnius University, Lithuania. Vol. 334. AMS Contemporary Mathematics, pp. 25-36.
- Wang, W., Wang, J., Kim, M.-S., 2001. An algebraic condition for the separation of two ellipsoids. Computer Aided Geometric Design 18 (6), 531-539.
- Weierstrass, K., 1868. Zur Theorie der bilinearen und quadratischen Formen. Monatshefte Akademie der Wissenschaften, Berlin, 310-338.