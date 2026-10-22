## Computation of the Solutions of Nonlinear Polynomial Systems

by

Evan Conway Sherbrooke

S.B. Mathematics, Massachusetts Institute of Technology June 1990

Submitted to the Department of Ocean Engineering in partial fulfllment of the requirements for the degrees of

MASTER OF SCIENCE IN NAVAL ARCHITECTURE AND MARINE ENGINEERING

and

MASTER OF SCIENCE IN ELECTRICAL ENGINEERING AND COMPUTER SCIENCE

at the

## MASSACHUSETTS INSTITUTE OF TECHNOLOGY

October 1993

Massachusetts Institute of Technology 1993. All rights reserved.

Author......

Department of Ocean Engineering October 15, 1993

Certified by ...

Nicholas M. Patrikalakis Associate Professor of Ocean Engineering Thesis Supervisor

Read by............ ......u.....

John N. Tsitsiklis Professor of Electrical Engineering Thesis Reader

Accepted by .....

'A. Douglas Carmichael Chairman, Departmental Committee on Graduate Students

Department of Ocean Engineering

MASSACHUSETTS INSTITUTE Accepted by." ....

Frederic R. Morgenthaler Chairman, Departmental Committee on Graduate Students

APR 15 1994

LiBrARIES

Department of Electrical Engineering and Computer Science

## Computation of the Solutions of Nonlinear Polynomial Systems

by

## Evan Conway Sherbrooke

Submitted to the Department of Ocean Engineering on October 15, 1993, in partial fulfillment of the requirements for the degrees of

MASTER OF SCIENCE IN NAVAL ARCHITECTURE AND MARINE ENGINEERING

and

SCIENCE

## Abstract

A fundamental problem in computer aided design is the effcient computation of all roots of a system of nonlinear polynomial equations in n variables which lie within an n-dimensional box. We present two techniques designed to solve such problems, which rely on representation of polynomials in the multivariate Bernstein basis and subdivision. In order to isolate all of the roots within the given domain, each method uses a different scheme for constructing a series of bounding boxes; the first method projects control polyhedra onto a set of coordinate planes, and the second employs linear optimization. We also examine in detail the local convergence properties of the two methods, proving that the former is quadratically convergent for n = 1 and linearly convergent for n &gt; 1, while the latter is quadratically convergent for all n. Worst-case complexity analysis, as well as analysis of actual running times are performed. The techniques are applied to the problem of determining stationary points of a scalar field. Classification of these stationary points is also described. In addition, the related problem of tracing sets of non-isolated roots is treated theoretically. Finally, several examples are presented which illustrate some of the applications of our techniques as well as their performance.

Thesis Supervisor: Nicholas M. Patrikalakis

Title: Associate Professor of Ocean Engineering

Thesis Reader: John N. Tsitsiklis

Title: Professor of Electrical Engineering

## Acknowledgements

I would like to dedicate this thesis to my parents and brother, who have provided love and support throughout my life. My parents will be pleased to note that this thesis is costing them nothing. I would also like to acknowledge my father's assistance on the theory of Linear Programming, and his creative variation of the protestant work ethic which has always been an inspiration.

I must acknowledge my advisor, Professor N. M. Patrikalakis, for his guidance throughout the process, Professor J. N. Tsitsiklis for his patience in reading this thesis, and Professor A. W. Drake for making the dual degree process totally painless. Several co-workers have provided valuable assistance, including Mr. C.-Y. Hu, Dr. T. Maekawa, and Ms. J. Zhou. They have all helped to test or enhance the algorithms described herein, and their assistance is greatly appreciated.

This work was supported, in part, by the M.I.T. Sea Grant College Program; the Office of Naval Research; and the National Science Foundation under grant numbers NA90AAD-SG-424; N00014-91-J-1014; DDM-9215411, and IRI-9224640; and by an NSF Graduate Fellowship.

## Contents

| Acknowledgements                                  | Acknowledgements                                  | Acknowledgements                                  | Acknowledgements                                  |   3 |
|---------------------------------------------------|---------------------------------------------------|---------------------------------------------------|---------------------------------------------------|-----|
| 1 Introduction                                    | 1 Introduction                                    | 1 Introduction                                    | 1 Introduction                                    |   8 |
| 2                                                 |                                                   | Literature Review                                 |                                                   |  10 |
| 3 Notation and Defnitions                         | 3 Notation and Defnitions                         | 3 Notation and Defnitions                         | 3 Notation and Defnitions                         |  13 |
| 4                                                 |                                                   | Formulation                                       |                                                   |  16 |
| 5 The Projected-Polyhedron Intersection Algorithm | 5 The Projected-Polyhedron Intersection Algorithm | 5 The Projected-Polyhedron Intersection Algorithm | 5 The Projected-Polyhedron Intersection Algorithm |  20 |
|                                                   | 5.1                                               | Motivation for the Algorithm                      | Motivation for the Algorithm                      |  20 |
|                                                   | 5.2                                               | Generating the Bounding Box .                     | Generating the Bounding Box .                     |  23 |
|                                                   | 5.3                                               | Analysis .                                        | Analysis .                                        |  26 |
|                                                   |                                                   | 5.3.1                                             | The Bounding-Box Tree                             |  27 |
|                                                   |                                                   | 5.3.2                                             | Complexity Analysis                               |  28 |
|                                                   |                                                   | 5.3.3                                             | Establishing Bounds                               |  29 |
|                                                   |                                                   | 5.3.4                                             | Quadratic Convergence in One Dimension            |  33 |
|                                                   |                                                   | 5.3.5                                             | Convergence in Higher Dimensions                  |  34 |
| 6                                                 | The Linear Programming Root Isolation Algorithm   | The Linear Programming Root Isolation Algorithm   | The Linear Programming Root Isolation Algorithm   |  37 |
|                                                   | 6.1                                               | Generating the Bounding Box . .                   | Generating the Bounding Box . .                   |  37 |
|                                                   | 6.2                                               | Analysis                                          | Analysis                                          |  41 |
|                                                   |                                                   | 6.2.1                                             | Complexity Analysis                               |  41 |
|                                                   |                                                   | 6.2.2                                             | Quadratic Convergence in n Dimensions             |  42 |
|                                                   | 7 Optimization Problems                           | 7 Optimization Problems                           | 7 Optimization Problems                           |  45 |

| 7.1                         |                                                             | Classification of Isolated Stationary Points ..             |   46 |
|-----------------------------|-------------------------------------------------------------|-------------------------------------------------------------|------|
| 7.2                         | Identification of Non-isolated Stationary Points            |                                                             |   51 |
| 7.3                         |                                                             | Remarks .                                                   |   53 |
| 8 Examples and Applications | 8 Examples and Applications                                 | 8 Examples and Applications                                 |   55 |
|                             | 8.1  Numerical Examples                                     | 8.1  Numerical Examples                                     |   55 |
|                             | 8.1.1                                                       | Intersecting Two Simple Planar Curves                       |   55 |
|                             | 8.1.2                                                       | The Wilkinson Polynomial                                    |   58 |
|                             | 8.1.3                                                       | Significant Points of a Planar Algebraic Curve              |   58 |
|                             | 8.1.4                                                       | Stationary Points of a Distance Function                    |   60 |
|                             | 8.1.5                                                       | A Stationary Point Problem at a Strict Tolerance .          |   62 |
|                             | 8.1.6                                                       | A Distance Problem with Tangency                            |   62 |
| 8.2                         | Other Applications..                                        | Other Applications..                                        |   63 |
|                             | 8.2.1                                                       | Systems of Piecewise Polynomials . .                        |   63 |
|                             | 8.2.2                                                       | More Equations than Unknowns, or Vice-Versa                 |   64 |
|                             | 8.2.3                                                       | Finding Complex Roots                                       |   64 |
| 8.3                         | Chocsing an Algorithm                                       | Chocsing an Algorithm                                       |   64 |
| 9                           | Concluding Remarks                                          | Concluding Remarks                                          |   66 |
|                             | A Supplemental Algorithms                                   | A Supplemental Algorithms                                   |   68 |
|                             | A.1                                                         | Converting from the Power to the Bernstein Basis            |   68 |
|                             | A.2 2S Subdivision of a Multivariate Bernstein Polynomial . | A.2 2S Subdivision of a Multivariate Bernstein Polynomial . |   69 |

## List of Figures

| 4-1 The control polyhedron of (c,y,c² + y² - 1) 18       |
|----------------------------------------------------------|
| 6-1  A comparison between a PP and an LP bounding box 38 |
| 8-1 The curve f(u,v) = 0 of section 8.1.3 . . 60         |
| A-1Pseudocode for basis conversion 69                    |
| A-2Pseudocode for multivariate subdivision 70            |

## List of Tables

|     | 8.1Bounding boxes in the Projected-Polyhedron algorithm for a simple two vari- ablecase.   |   56 |
|-----|--------------------------------------------------------------------------------------------|------|
| 8.2 | Bounding boxes in the Linear Programming algorithm for a simple two vari- able case.       |   57 |
| 8.3 | Roots of the Wilkinson Polynomial                                                          |   59 |
| 8.4 | Critical points of an algebraic curve                                                      |   61 |
| 8.5 | Critical points of a distance function                                                     |   62 |
| 8.6 | Comparison of CPU times of PP, LP, and hybrid algorithms jn test cases                     |   65 |

## Chapter 1

## Introduction

A fundamental problem in computer-aited design and manufacturing is the efficient computation of all solutions to a system of nonlinear polynomial equations within some finite domain. This problem arises in many different applications, as varied as they are numerous. For example, in computer-aided design it is often necessary to identify all characteristic points of an intersection curve between two surfaces, in order to trace out all of the branches of the curve; if the surfaces are piecewise rational polynomial, we must solve a system of nonlinear polynomial equations in order to identify these points. In the area of feature recognition, computing the medial axis transform requires the determination of branch points, which can frequently be formulated as the solution set of a polynomial system. This problem also arises in planning robotic motions, in computing distance functions and extrema, in arranging distributive systems in complex mechanical platforms, and in many other areas of computer-aided engineering.

In this thesis, we describe two techniques for eliciting all real solutions to a system of n nonlinear polynomial equations in n unknowns over an n-dimensional rectangular domain. We note that such an algorithm can also be used to find complex roots; simply separate each equation into a real and imaginary part, effectively doubling the number of equations inthesystem.

The thesis is structured as follows. Chapter 2 briefly reviews some of the prior research related to this topic. The notation we will use is introduced in chapter 3, and chapter 4 gives the formulation of the problem. Chapter 5 describes and analyzes the Projected-Polyhedron (PP) technique. In chapter 6, we intrcduce a new technique based on Linear Programming

(LP), and show that this method exhibits quadratic convergence irrespective of n. Chapter 7 considers the application of our root-finding schemes to optimization problems. Since, as we shall see, non-isolated roots often appear in such problems, this chapter also explores strategies for dealing with non-isolated roots. Some examples comparing the performance of the PP and LP approaches follow in chapter 8, along with a brief survey of some of the applications of this technique. A few concluding remarks are given in chapter 9, along with recommendationsforfutureresearch.

An extended summary of the results of this thesis may be found in [42], with a more detailed version in [41]. A summary of this work dealing with computation of non-isolated roots and application of the algorithms to distance function computation may be found in [47]. Further applications are explored in [32].

## Chapter 2

## Literature Review

In recent CAD-related research, three classes of methods for the computation of solutions of nonlinear polynomial systems have been favored: algebraic techniques, homotopy, and subdivision. These methods may be classified as global because they are designed to compute all roots in some area of interest. There also exist a number of local numerical techniques which n methods are used in CAD applications requiring high accuracy because they are effcient such approximations are usually obtained through some sort of global search like sampling, s of robustness makes the development of efficient and stable global techniques desirable; in what follows, we briefly review three classes of global methods.

The computation of all complex roots of nonlinear polynomial systems has typically been approached with algebraic geometry techniques like elimination or Groebner basis methods [4] [3]. These methods have many advantages; they are theoretically elegant, guaranteed to find all complex roots of a system irrespective of the dimensionality of the solution set, and well-suited for implementation in symbolic mathematical systems. However, they suffer from numerical instability, making implementation in foating point arithmetic diffcult. Furthermore, they are inefficient in memory and processing time requirements, making them unattractive for systems only moderate in degree or dimensionality. There is also a philosophical problem with these techniques: they frequently give us much more information than we need. As we shall see in chapter 8, many CAD applications only require those roots which lie in some real n-dimensional box.

The second category of methods is the class of homotopy techniques [46] [12]. These methods may be used to find all complex solutions of a nonlinear polynomial system if the number of roots is finite. Unfortunately, investigation of such methods indicates that they also tend to be numerically ill-conditioned. If we try to get around this problem by implementing the algorithm in exact rational arithmetic, we end up with enormous memory requirements because we have to solve large systems of complex initial value problems. And homotopy suffers from the same philosophical problem as algebraic geometry techniques -it simply provides more information than we need.

The third class, the category of subdivision-based techniques, is the class to which both the PP and LP methods described in this thesis belong. Lane and Riesenfeld [21] investigated the application of binary subdivision and the variation diminishing property of polynomials in the Bernstein basis to eliciting the real roots and extrema of a polynomial within an interval. Boehm [2] and Cohen et al. [5] extended this idea to general nonuniform subdivision of B-splines. Geisow [14] was among the first to develop subdivision algorithms stein bases and apply them to surface intersections. Formulation of a class of geometric nomial systems expressed in terms of B-splines, was made by Dokken[8], who suggested a solution approach based on [5]. Subdivision techniques have also more recently been used oped an adaptive subdivision algorithm which can be used to intersect two planar algebraic curves expressed in the barycentric Bernstein basis. Patrikalakis, Prakash, and Kriezis [35] [33][20] investigated the use of subdivision of algebraic curves in intersecting an implicit algebraic surface with a rational polynomial surface. Their method relies on the computation of real characteristic points of an algebraic curve represented in the Bernstein basis, which typically involves intersecting two or three algebraic curves by repeated adaptive subdivision and minimization. Minimization is used to increase the precision of the root quadratically; alternatively, Newton-Raphson iteration, which is also quadratically convergent, can be used to refine the root. Nishita et al. [28] developed an adaptive subdivision technique known as Bezier clipping to intersect rays with trimmed rational polynomial sur- face patches, also recasting the problem as the intersection of two algebraic curves expressed in the Bernstein basis. Sederberg and Nishita [40] extended this method to intersect parametric curves with parametric surfaces. Vafiadou and Patrikalakis [44] employed an early surfaces. In this thesis, we will study the n-dimensional extension of the PP algorithm in detail, along with the related LP approach. It should be noted that subdivision techniques have a number of disadvantages. They are not as general as algebraic methods, since they are designed for zero-dimensional solution sets within an n-dimensional subset of Euclidean space. Furthermore, although the chances that all roots have been found increase as the resolution tolerance is lowered, there is no certainty that each root has been extracted. Lastly, subdivision techniques provide no explicit information about root multiplicities without additional computation. However, despite these drawbacks, their speed and stability make them attractive as root-finding schemes.

The subdivision methods reviewed above belong to the general class of interval-based methods, although they have a geometric character. Interval arithmetic techniques, primarily interval extension of Newton methodis coupled with bisection to ensure convergence, have also been an active research topic in recent years [18] [27] [1]. Interval arithmetic techniques with a more geometric favor may be found in [24] [22] [23] and in [17].

## Chapter 3

## Notation and Definitions

1. The lower-case letters i, j, k, I, m, and n denote nonnegative integers.
2. The lower-case letters f, g, and h denote real-valued functions.
3. Other lower-case letters, such as  and y, denote real numbers.
4. Lower-case letters in boldface, such as x and y will denote vectors of real numbers, or points. It should be clear from the context what the dimension of any given vector is. When we work with the components of the vector x, we will assume that c is the ith component of x.
5. Upper-case boldface letters such as F denote vector-valued functions. The ith component of F is denoted by Fi.
6. Upper-case letters other than those between I and N inclusive (which are used for multi-indices, defined below) denote sets or matrices.
7. To avoid confusion, some variables may have superscripts instead of or in addition to subscripts. For example, the matrix W(k) is superscripted so that we can refer to the (k)
8. [a, b] denotes set of all real c such that a ≤  ≤ b.
9. R denotes the real numbers; Rn denotes the set of points in n-dimensional Euclidean space.

Definition 3.1 The Euclidean norm jxl of a vector x is defined to be the quantity √++...+

Definition 3.2 A multi-index I is an ordered n-tuple of nonnegative integers (i1, i2, ... , in). We say that I is bounded by the multi-inder M = (m1,m2,...,mn) if 0 ≤ i1 ≤ m1,0 ≤ i2 ≤mz,...,0 ≤in ≤mn.

We will use multi-indices as a shorthand for subscripting multi-dimensional arrays. For example, if I = (2,3), then w1 is equivalent to w23, the element of the matrix {wij} at row 2, column 3.

We will use the notation ZM, where M is a bounding multi-index, to indicate that a sum is to be taken over all possible multi-indices I which are bounded by M. For example, if M = (1, 1), then ∑M wI = woo + wo1 + w10 + w11.

Definition 3.3 The ith Bernstein polynomial of degree m is defined by

<!-- formula-not-decoded -->

The Bernstein multincmial determined by the multi-indez I = (i,i2,...,in) and the bounding multi-index M = (m1,m2, ... ,mn) is defined by

<!-- formula-not-decoded -->

where u = (ui, u2,...,Un) E Rn.

Definition 3.4 Let x = (c1,x2,...,&amp;n) be an element of Rn, and let f : Rn → R be a function of x. The graph of f is the function Gf : Rn → Rn+1 defined by

<!-- formula-not-decoded -->

Definition 3.5 Let x = (x1, r2, ... ,n) be an element of Rn. Then the ith projection map Ti : Rn → R is defined by

<!-- formula-not-decoded -->

Definition 3.6 Suppose P is a set of m points of Rn. Let C(P) denote the set of all x such that

<!-- formula-not-decoded -->

where for each i, Pi E P and ci ≥ 0, and Z1 c; = 1. Then C(P) is called the convez hull of P.

Definition 3.7 Suppose fi, f2,..., fn are functions of the parameters 21, z2,..., &amp;n. Then i, column j, is equal to Bj

Definition 3.8 Suppose f(z) and g(z) are both real-valued functions of one variable. We say that f(x) = O(g(x)) if there exists a 8 &gt; 0 and a constant 入 &gt; 0 such that for [c| &lt; 6, [f(x)l ≤ 入(g(x)].

## Chapter 4

## Formulation

Suppose we have a set of n functions of n variables fi, f2,..., fn, each of which is polynomial in the independent parameters u1, u2,... , Un. Let m n(k) denote the degree in ui of m(k) m(k) ((y) describes all the degree information of fk. Furthermore, suppose there is an n-dimensional rectangular subset of R

<!-- formula-not-decoded -->

We wish to find all points u = (u1, u2,... , un) E S such that

<!-- formula-not-decoded -->

By making the affine parameter transformation [9] ur = ai +x;(bi-ai) for each i between 1 and n inclusive, we simplify the problem to one of determining all x E [0, 1] such that

<!-- formula-not-decoded -->

Since all of the fk are polynomial in each of the n independent parameters, a simple change of basis [9] allows us to express them in the multivariate Bernstein basis. In other words, ,(k) each k ∈ {1,...,n}

<!-- formula-not-decoded -->

Representation of algebraic and piecewise algebraic surfaces (i.e., for n = 3) in terms of tensor products of Bernstein polynomials or B-splines has been studied earlier by Patrikalakis and Kriezis [31]; equation (4.4) is simply an extension to n dimensions. Provided that conversion of the problem to the Bernstein basis is exact or sufficiently accurate, the use of the Bernstein basis in conjunction with subdivision is known to be numerically stable [10]. The conversion process itself may be numerically ill-conditioned [1l]; therefore, we recommend that the problem be formulated in the Bernstein basis from the very beginning. If necessary, polynomials may be converted from the multivariate power basis to the multivariate Bernstein basis using the algorithm described in the appendix; alternatively, the problem may be reformulated using a bilinear transformation described in [25].

The form of (4.3) is not sufficient to apply the two techniques we consider in this thesis. Before we can use either method, we have to restate the problem as the intersection of the graphs of the fk (each of which is a hypersurface in Rn+1) and the hyperplane Zn+1 = 0 of Rn+1. This idea is designed to impart geometrical significance to the coefficients of the polynomials and to the solution process. A similar approach has been used extensively as the foundation of convex-hull based subdivision algorithms for intersection problems (see chapter 2). Let us take the same approach in rewriting equation (4.3). Out of each fk we will build its graph F (see definition 3.4):

<!-- formula-not-decoded -->

Clearly, (4.3) is satisfied by the point x if and only if

<!-- formula-not-decoded -->

As with the fk, the Fk may be expressed in the multivariate Bernstein basis. It can be shown that ((9])

<!-- formula-not-decoded -->

in other words, the monomial u can be expressed as the weighted sum of Bernstein polynomials with coefficients evenly spaced in the interval [0, 1]. Using equation (4.7) and the summation to unity property of the Bernstein basis (5.2), we obtain an equivalent expression where for each of the c; in equation (4.5):

Figure 4-1: The control polyhedron of (c, y,c2 + y² - 1)

<!-- image -->

<!-- formula-not-decoded -->

Substituting (4.8) into (4.5) gives a more useful representation for the Fk:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The v(? v(k) are called the control points of Fk. In figure 4-1, we show the control points of the graph of f(c,y) = x² + y² - 1, whose zero set over 0 ≤α ≤ 1,0 ≤ y≤ 1 is a quarter circle in the plane. Although at first glance this seems like an unnecessarily complicated way of posing the problem, using the parametric hypersurfaces Fk instead of the real-valued fk permits use of the powerful conver-hull property of the multivariate Bernstein basis, which we will investigate below. As we shall see, the two techniques make use of this property in different ways, resulting in different convergence characteristics.

## Chapter 5

## The Projected-Polyhedron Intersection Algorithm

In this chapter, we introduce the first of our iterative root-finding algorithms for n-dimensional problems. Although occasionally slower than the second technique, it is easy to visualize and simple in that it only requires two straightforward algorithms in order to implement it: one for subdividing multivariate polynomials in Bernstein form, and one for finding the convex hull of a two-dimensional set of points. This algorithm is an extension of earlier adaptive subdivision algorithms for n = 2 used in intersecting two planar algebraic curves (reviewed in chapter 2).

## 5.1 Motivation for the Algorithm

Having rephrased the problem in the form of (4.6), we can now use properties of the Bernstein basis in order to eliminate parts of the domain [0, 1]n which do not contain any roots. The two properties we will use are familiar ones which are immediate upon inspection of equation (3.1). The first is positivity:

<!-- formula-not-decoded -->

The second is that the Bernstein polynomials form a partition of unity [9]:

<!-- formula-not-decoded -->

Since the Bernstein multinomials are merely products of Bernstein polynomials, these two properties lead immediately to corresponding properties of Bernstein multinomials:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and

These two equations in turn lead to the important conver hull property of the multivariate Bernstein basis:

C(V(k)) be the convex hull of definition 3.6. Let r E [0, 1]n. Then for each k between 1 and n inclusive,

<!-- formula-not-decoded -->

(k) with coefficients being B1,M(c)(x). By (5.3) and (5.4), these coefficients are positive and sum to 1, and therefore satisfy the requirements of the c; in (3.5). Thus, Fk(x) is an element of the convex hull C(v(k)).

Because every point on the hypersurface Fk lies within the convex hull of its control points, equation (4.6) immediately implies the following theorem:

Theorem 5.2 Let x E [0,1]n be a point satisfying (4.3), i.e. a root commor to all of the fk. Let Xn+1 be the set of points of Rn+1 which lie on the hyperplane Cn+1 = 0. Then

<!-- formula-not-decoded -->

Proof: We are given that x satisfies (4.3). Therefore, for each k between 1 and n, x satisfies Fk(x) = (x, 0). By theorem 5.1, the point (x, 0) E C(v(k), and since any point of the form (x, 0) is automatically an element of Xn+1, (5.6) follows.

An simple but important corollary is worth stating:

Corollary 5.3 (Contrapositive) Suppose (x,0) does not satisfy (5.6). Then x is not a root common to all of the fk.

With the help of this corollary, we can devise the following general root-finding approach.

1. Let Bo = [o, 1]n. This signifies that initially our region of search is the box [O, 1]. As the algorithm proceeds, Bo will keep track of which sub-box of [o,1]n we are considering.
2. Form the convex hulls of all the F and intersect them with one another and with the hyperplane n+1 = 0. Call the intersection set A. If A is sufficiently small, stop.
3. Because A is the result of an intersection with the hyperplane Cn+1 = 0, it can be expressed in the form A = A' x {o} for some set A' ∈ [0, 1]n. By corollary 5.3, any roots in [O, 1]n must lie inside A'. We will call an element of A' a feasible point.
4. Find a box B = [a1,b1] × [a2,b2], × ... x [an, bn] E [0, 1]n which encloses A'. If this box B is not significantly smaller than [O, 1], split B into some number of equally sized, smaller boxes. Continue with the next step once for each one of these sub-boxes.
5. For each k, define a new function f by

<!-- formula-not-decoded -->

This new function may be computed by applying De Casteljau subdivision according to the box B on fk. (See the appendix for an algorithm which implements this procedure.) Notice that ff maps [O, 1]n to the same set that fk maps B to.

- pue 4oo mau au4 uau [uoq'u'oo] x ·": × [20qz'on] × ['oqt'oo] = 0g H ·0g aepdn '9 bo,k are given by ao,k &lt; ao,k + (bo,k - ao,k)ak and bo,k &lt; ao,k + (bo,k -- ao,k)bk, where k ranges from 1 to n.
7. Construct the control points of the graphs Fk from the fk, and go back to step 2.

Although the above procedure is sound, we need to resolve two problems before implementation. First, forming the convex hull of a set of m points in n dimensions is generally an O(ml?]+1) procedure [36]. And second, even when these convex hulls are generated, the time-consuming process of intersecting them remains, along with a large space requirement n          m bounding box B without actually erecuting step 2. The two algorithms we introduce in this thesis take full advantage of this fact, although they use different methods to generate B. Let us now examine the way that we generate the box B in the Projected-Polyhedron algorithm.

## 5.2 Generating the Bounding Box

We saw that the major diffculty of our computational procedure was generating and intersecting convex hulls from a set of points. The Projected-Polyhedron method generates a bounding-box by considering n two-dimensional problems on each iteration (one for each dimension of [0, 1]) instead of one n + 1-dimensional problem. The philosophy behind this approach is to generate each of the n sides of the box independently by projecting control points onto different planes.

Suppose we want to find [a,,by], the jth side of the bounding box B. Define a function πj : Rn+1 → R2 by the equation

<!-- formula-not-decoded -->

where T and Tn+1 are the projection maps of definition 3.5. Using j, we will map the control points of each of the Fk into a Cartesian plane (which for the sake of simplicity we will call the (c,y) plane). We will then generate the (two-dimensional) convex hulls of these projected points and intersect them with one another and with the c-axis. Let us call the set resulting from this intersection By. Since B, is a subset of the c-axis, it can be expressed as B; × {0} for some B' E [0, 1]. As the following theorem demonstrates, the set of feasible points A' is a subset of the box B = Bl x B2 × ... × B':

points v(k), (x, y)-plane satisfying y = 0). Finally, let

<!-- formula-not-decoded -->

Erpress Bj in the form B x {O}. Then if B = B' × B2 × ... × Bn, A' C B.

Proof: Let x E A'. Then for some choice of coeffcients (c} such that each c() ≥ 0

<!-- formula-not-decoded -->

for all k between 1 and n. Taking  of both sides, we obtain

<!-- formula-not-decoded -->

Therefore (r, 0) e C(v(b) for all k. Since also (r, 0) e X2, (z,0) e B; by (5.9), and thus Cj E B'. Because this must hold for all j, we conclude that x E B. Since x was arbitrary, A' C B.O

Keep in mind that this inclusion is in general proper; that is, usually A' ≠ B. In fact, A' tends to be much smaller than B in more than one dimension, resulting in a lower order of convergence than a tighter box around A' would achieve. Nevertheless, because generating the box is fairly simple, it is a reasonably good technique for most inputs.

Generation of the box is essentially performed according to the theorem. Suppose we convex hulls with a simple algorithm related to Graham's scan [6]. We then intersect these convex polygons with the interval [0, 1], which can be accomplished simply by marching along the vertices of each polygon and recording any intersections between their joining line segments and the c-axis as we go along. Note that since these polygons are convex, the intersection will always be either empty, a point, or a closed interval on the x-axis. This computation is also the essential element of the Bézier clipping method developed for n = 2 [28]. Once we have obtained these intervals (escaping from the procedure if any of them are empty) we simply intersect them. If the intersection B' is nonempty, it can therefore be expressed as an interval [aj, bj]. Performing this procedure for each § will give us a new B which is either empty or equal to [a1,b1] x [a2, b2] x ... x [an, bn].

Figure 5-1 shows the effect of projection in the case of an intersecting circle and ellipse. Notice that the projection of their convex hulls cover the entire interval [o, 1] on the c-axis, giving us no useful information; however, parts of the y-axis may be eliminated immediately, telling us that there is no root of the system whose y-coordinate is less than 0.1875 or greater than 0.625 (see also section 8.1.1).

Now that we have a method for generating a bounding box, it is straightforward to write the algorithm PROJECTED-PoLYHEDRON based on this method of box generation. The algorithm may be summarized in the following steps:

1. Start with an initial box of search
2. (4.3), i.e. an affine parameter transformation that changes our box of search to [o, 1]n. The transformation of the n functions fk is performed using multivariate De Casteljau subdivision (see the appendix).
3. Using the method described above, find a sub-box of [O, 1]n which contains all the roots.
4. Using the reverse of the transformation we used in step 2, determine what this subbox corresponds to in Rr. If the actual box in Rn is sufficiently small, conclude that there is a root inside and return the midpoint of the box as an approximation to it.
5. If any dimensions of the sub-box in step 3 are not much smaller than 1 unit in length (i.e., the box has not decreased much in size along one or more sides), split the box at the midpoint of each dimension which is causing trouble. Continue on the next iteration with several independent sub-problems. (For example, suppose we have 4 equations in 4 unknowns, and thus a 4-dimensional box, and suppose also that two of the sides have not decreased considerably in length in step 3. Then, due to even splitting of the box at the problem areas, we will have 22 = 4 boxes on the next iteration.)

## 6. Go back to step 2, once for each new box.

Step 2 is needed in order to use the convex hull property as described above. Step 5 is typically invoked when the box contains more than one root. The presence of more than one root in the same box obviously prevents the box from decreasing in size past a certain point, so splitting is necessary in order to isolate the roots from one another. On a typical iteration, however, step 5 will not be invoked, and step 6 will only send one box back to step 2 for processing.

Figure 5-1: Effect of projecting the polyhedra of (z,y,x2 + y2 - 1) and (2,y, + 4y2 - 1)

<!-- image -->

tolerance the root is required to satisfy. In step 5, we have to decide when a side is decreasing in size too slowly, and we use a threshold which we will call Crir. Our experiments have =         s      s on each iteration, although a slightly larger factor of CRIT= 0.85 or 0.9 is better for higherdimensional problems since the boxes tend to decrease in size more slowly anyway. We will always assume that CRIT&gt; 0.5.

## 5.3Analysis

In this section, we will first analyze the working of the algorithm by introducing a bounding bor tree. Next, we will briefy analyze the time complexity of the algorithm. Finally, by analyzing performance on a local scale, we will show that the algorithm achieves quadratic convergence for n = 1 and at best linear convergence for n &gt; 1.

## 5.3.1 The Bounding-Box Tree

Let us first prove that the algorithm actually terminates. In order to do this, we first define a few terms relating to the algorithm:

## Definition 5.5

1. An iteration s is one ececution of steps 2-5.
2. The bounding box B of iteration s is the actual boc in Rn before we transform it to [0, 1]n in step 2.
3. The children of a bounding bor B are the bores we are left with at step 6, and B is said to be the parent of these children. If we determine in step 4 that we are sufficiently close to a root, then B will have no children. Typically, B will only have one child; if splitting is necessary in step 5, then B will have a number of children equal to a power of 2.

With the parent-child relationship defined among bounding boxes, we can construct a bounding-bor tree where the nodes are bounding boxes and the links represent this relationship.

Lemma 5.6 Let B1 be a given node of the bounding-bor tree. Suppose B2 is one of its children. Then each side of B2 is no larger than the corresponding side of B1 times CRIT.

Proof: By the definition of an iteration, between B1 and B2, each side of B2 must either be smaller than the corresponding side of B1 by a factor of CRIT or else splitting was performed, in which case the side of B2 is no greater than half as long as the side of B1. Since CrrT&gt; 0.5, we are done.

Corollary 5.7 Suppose we are given some number ε &gt; 0. Then there erists a number N such that after N or fewer steps, the algorithm has terminated.

Proof: Since CRIT &lt; 1, there exists some integer p &gt; 0 such that CRITP &lt; ε. Now each node has at most 2n children for an n-dimensional problem. Furthermore, the depth of the bounding-box tree is no greater than p since any box with sides less than e has no children, and by lemma 5.6 every time we proceed down a level of the tree, every side of a child's box is at least CrIr times as small as its parent. The number of nodes in this tree must therefore be bounded above by 2np. Choosing N = 2np completes the proof.

By itself, Corollary 5.7 is not particularly reassuring. We would like all of the boxes of the first type (that is, boxes under tolerance) to contain one and only one root, but the corollary gives no assurance of this by itself. However, as we shall see, for small n reasonably small ε, all of the boxes under tolerance will contain one and only one root. Of course, the actual number of iterations needed to complete execution will usually be far smaller tha.n 2np; this value is merely an upper bound useful for showing that the algorithm terminates.

## 5.3.2Complexity Analysis

It is difficult a priori to predict how many iterations will actually be needed to complete the algorithm. However, we can analyze the amount of time required to execute each iteration of the algorithm individually (see definition 5.5). In the analysis, we will assume that each equation is degree m in each variable, or of total degree mn; although it is easy to extend the analysis to the general case that all the degrees are different, the resulting formulas are quitelong.

The generation of a convex hull of (m + 1) two-dimensional points takes at most O((m + 1) log (m + 1)n) time [6]. However, by exploiting the special structure of these at most two of them can belong to the boundary of the convex hull (specifically, the points with the minimum and maximum ordinates). Therefore, we can discard all other points from consideration, leaving us with 2(m + 1) points (a minimum and maximum point for each abscissa). These points may now be traversed in counterclockwise order as in Graham's scan [6]. The entire execution time of this procedure is O((m + 1)n). The time needed to traverse this convex hull in order to intersect it with the c-axis is no greater, so we conclude that the total execution time to perform the projection in one direction is O(m?). Because there are a total of n convex hulls which need to be intersected in a given projection plane, the total cost of a projection becomes O(nm?). Now, there are n different projection planes, one for each side of the box, so the total time needed to execute step 3 is O(n?mn).

For &amp; moment, assume that no splitting is necessary. A glance at the pseudocode in the appendix will show that to subdivide one of the fk in all n directions takes O(nmn+1) time; since there are n fk which are subdivided, the total time needed to perform the transformation is O(n2mn+1). Now, we don't know a priori how many children will result from a step of the algorithm, so the time will vary from step to step. But since we are the cost of subdivision on the children. With this useful device, we get an average cost per iteration of O(n2mn + n2mn+1) = O(n2mn+1). Although at first it may seem unpleasant that the algorithm is exponential in n, keep in mind that the input itself (the coefficients of the polynomials) is itself exponential in n, so merely printing out the control points would take exponential time. Furthermore, for most geometric modeling applications, it is quite rare to see an n higher than 6. It is often more helpful to consider that for a given number of dimensions, the algorithm is polynomial in m. This analysis is incomplete because the number of iterations needed may be quite different for systems with the same n and m. However, the local convergence properties can give us a better idea of how much time is required. In order to analyze these local convergence properties, we will first examine the relationship between the level of subdivision of a multivariate polynomial and the closeness of its control points.

## 5.3.3 Establishing Bounds

Theorem 5.8 Let a = (a1,a2,...,an) and b = (bi,b2,...,bn). Let e = (e1,E2,...,En) = (b1 - a1,b2 - az,...,bn - an). Suppose f : [0,1]n → R is the function which results from subdividing another function fo : [0, 1]n → R over the boz [a1,bi] × [a2,b2] × ... x [an,bn]; that is,

<!-- formula-not-decoded -->

Suppose further that for a given bounding multi-indez M = (mi,m2,...,mn), f(x) = ∑M p1B1,M(x). Then there erists α positive constant k independent of e such that for each I bounded by M, there erists a point x E [O, 1]n for which If(x) - pil &lt; x|e|2.

Informally, this theorem simply says that if the sides of our bounding hox are on the order of lel, then the distance from f to any of its control points is on the order of lel2. In order to prove theorem 5.8, we will break the problem up into several steps. First, we take xe r e  o r  o = x hd  r  g sdxa  e

<!-- formula-not-decoded -->

Since g is linear in each coordinate direction, it can be written as the sum of Bernstein multinomials of degree 1 in each variable. Suppose we 1ow apply degree elevation [9] in each of the n directions, so that coordinate direction j has degree m, instead of 1. Then we can rewrite g as

<!-- formula-not-decoded -->

An important property of g is that because g is linear, all of its control points q1 lie on the image set of g [9]. Another important characteristic of g is that because it is a first-order approximation to f, the difference h = f - g is a second-order quantity. That is, there exists some positive constant ko independent of e such that for all x E [o, 1],

<!-- formula-not-decoded -->

Since h = f -- g, we can also write

<!-- formula-not-decoded -->

where o1 = p1 - q1. We would like now to establish bounds on the 01, which we do by first proving bounds on the derivatives of h evaluated at 0:

Lemma 5.9 Let I = (i1,i2,...,in), and let the notation a Let h be as above. Then there ecists a positive constant ki independent of e such that

Proof: There are three cases we need to consider:

= h. Since g(0) = f(0), h(0) = 0 &lt; kile|2 for any Case I: If I = (0,0,...,0), then axT K1 &gt; 0.

I are 0), then (0) = 器,(0).

Case IIl: For any I bounded by M which does not satisfy either case above, a1h x= ax! because derivatives of g of order greater than 1 are identically 0. Now recall that f is a function which results from subdividing another function fo over the box [a1, bi] × [a2,b2] × ... × [an,bn]. Therefore, we can make use of equation (5.12) and the chain rule to obtain

<!-- formula-not-decoded -->

Because I does not satisfy either of the first two cases, we must have i1+i2+...+in ≥ 2. Therefore,

<!-- formula-not-decoded -->

Furthermore, since fo is polynomial, all of its derivatives are bounded over [O, 1]n. Thus there exists some positive constant k1 such that for all x E [o, 1] and all I bounded by M,

<!-- formula-not-decoded -->

Substituting (5.19) and (5.18) into (5.17) finally gives

<!-- formula-not-decoded -->

Since these three cases cover all permissible I, we are done.

The next step is to show that each of the d1 can be expressed as a linear combination of derivatives of h evaluated at 0. In order to do this, we recall the following result from [10]:

<!-- formula-not-decoded -->

Now h can be as easily expressed in the power multinomial basis as the Bernstein multinomial basis; that is, there exist coefficients d1 such that

<!-- formula-not-decoded -->

Substituting (5.21) for each j, we obtain

<!-- formula-not-decoded -->

If we define () = 0 for n &lt; k, (5.23) becomes

<!-- formula-not-decoded -->

Now we can easily equate like terms to obtain the following relationship between d1 and d1:

<!-- formula-not-decoded -->

This equation may not appear extremely relevant at first sight, but it becomes more useful when we recall the simple relationship between the dj and the derivatives of h at 0:

<!-- formula-not-decoded -->

This relationship can be readily verified by differentiating the right-hand side of (5.22), but it is a familiar result of multivariable calculus. By plugging (5.26) into (5.25) we obtain

<!-- formula-not-decoded -->

With this relationship established, we can prove the following lemma:

Lemma 5.10 Let 81 be as before. Then there erists some positive constant k2 independent of e such that for all I bounded by M, Io1l ≤ k2lel2.

Proof: Consider the coefficient of mk ≥ jk for k E {1,2,... ,n}, and so (m) ≥ (t). Therefore, each coeffcient is a positive number less than or equal to 1, giving

<!-- formula-not-decoded -->

Using the result of lemma 5.9, we obtain

<!-- formula-not-decoded -->

Putting K2 = m1m2 .. .mnk1 completes the proof.

Now we are ready to put all the pieces together and prove theorem 5.8.

Proof of Theorem: Pick a multi-index I bounded by M. Now, q1 lies on the image set of g, i.e. there exists a point x E [o, 1]n such that

<!-- formula-not-decoded -->

From lemma 5.10 we know that

<!-- formula-not-decoded -->

But equation (5.15) gives us a bcund on the quantity If - gl. Applying the triangle inequality, we get

<!-- formula-not-decoded -->

Setting K = ko + k2 completes the proof.

## 5.3.4 Quadratic Convergence in One Dimension

Theorem 5.11 Suppose we are given an interval [a,b] which contains one and only one root Zo of a polynomial fo : [o,1] → R. Further assume that co is a simple root. Let e = b - a, and let f denote fo subdivided over [a,b], i.e. f(x) = fo(a + re). Let e' denote the size of the new interval which results after erecuting steps 2-4 of PROJECTED-POLYHEDRON with f,a, and b as the governing parameters. Then there erists a positive constant 入 independent of e and a 8 &gt; 0 such that if e ≤o, then e' ≤ Xe?.

Proof: Now co is a simple root of fo, so f(zo) ≠ 0. By continuity, there exists a 0 such that f(x) ≠ 0 for  E [co -- o, co + o]. By assumption, ε ≤o, so [a,b] C [∞o - o,xo + 8].

Let f(x) = Zo pibi,m(z). Let g be the linear approximation to f at 0, i.e. g(z) = f(0) + zf'(0) = Z=o qibi,m(x). As before, let us construct the graphs of f and g:

<!-- formula-not-decoded -->

From lemma 5.10, we know that for each i, lqi - pil ≤ k2e?. Therefore, for each i, the control points (,pi) of F and (,qi) of G are separated by a vertical distance less than or equal to k2e?. Since G is a line, all of its control points lie on the line; thus, each control point of F must lie on or between the following two lines which are respectively above and below G:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The region enclosed by these two lines and the vertical lines r = 0 and r = 1 is convex; therefore, the convex hull of the control points of F is entirely contained within this region.

F(0) therefore

<!-- formula-not-decoded -->

where we have used the fact that f'(o) = efo(a). Since the convex hull of the control points of F is completely contained in the region bounded by Li and Lz, the size of the interval computed in step 3 of PROJECTED-PoLYHEDRON must be less than or equal to △t. Remember, however, that this interval must be rescaled in step 4; after scaling, the size E' of the new interval is given by

Then (5.37) implies that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Note that since k2 and c are independent of ε, so is 入, and therefore the algorithm exhibits quadratic convergence in one dimension.

## 5.3.5 Convergence in Higher Dimensions

Unfortunately, the quadratic convergence of one dimension does not carry over into higher dimensions. Instead, we observe at best linear convergence which, as we shall see, is due to the use of projections. Projection of control points simplifies the problem, but we pay for the privilege with a lower order of convergence.

Theorem 5.12 Suppose we are given a boz B = [a1,bi] × [a2,b2] x ... × [an,bn] which contains one and only one root xo of a system of polynomials fo,k : [o,1]n → R where k ranges from 1 to n. Assume further that xo is a simple root. Suppose fk represents the subdivision of fo.k over B. Erecute steps 2-4 of PROJECTED-POLYHEDRON, and let B' = [a',b''] × [a2,b2] × ... × [an,bn] be the boz that results in step 4. Let j be an integer between 1 and n, and let e' =b' - a', the length of the jth side of the new boz. Then e' is an O(e) quantity.

(x,fk(x)) and Gk(x) = (x,gk(x)) be the graphs of fk and gk. Let the control points of Fk be p' （k) and the control points of Gk be q/). (k)

(k) (k).

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

（) and q,1 （k) are separated by a vertical distance of not more than k2e2.

Unfortunately, the projection of Gk into the (j, n+1)-plane is not a straight line if n &gt; 1; in fact, it is generally a planar region with a nonzero area. For a fixed xj, the difference between the maximum y-coordinate and the minimum y-coordinate of this region is given by

<!-- formula-not-decoded -->

This equation is obtained by taking the minimum value of (5.13) over all x E [0, 1] with rj fixed and subtracting it from the maximum value of the same quantity. Now, since this region is the projection of a hyperplanar region into two dimensions, it must be a s  d o q a  e sides by the vertical lines  = 0 and r = 1. Notice that if n = 1, △y is identically 0.

Using this value of △y, we can proceed as we did in the previous section and find two lines L1 and L2 which have slope equal to a and which are separated by a distance △y + 2k2s2. Together with the vertical lines  = 0 and  = 1, these lines form a region enclosing the control points of Fk; since this region is convex, it also encloses the convex hull of the control points.

Following the same steps as the proof of theorem 5.11, we can show that the intersection between this region and the r-axis is an interval with width

<!-- formula-not-decoded -->

both nonzero, and so △t is an O(1) quantity. Now, we still have to intersect this interval with the other n - 1 intervals obtained using different values of k; after intersection, the final interval may be considerably smaller than △t. But if, on the other hand, for each k the intervals are about the same size, their intersection may well be about as large or the same size as △t. Therefore, this intersection is an O(1) quantity. After scaling it step 4, we finally obtain e' = O(e).

It is possible to construct special cases for n &gt; 1 where one can observe quadratic or close to quadratic convergence. However, most multivariate systems of polynomial equations hidden inside the O(e) expression increases with degree and number of variables, as one might expect. Intuitively, a higher number of variables makes the projected area "thicker" and a higher degree typically makes the intersecting hypersurfaces less like hyperplanes; both of these factors will contribute to a larger constant. In the preceding analysis, we saw that the extra "thickiiess" of the intersecting convex hulls due to projection reduces the order of convergence from quadratic to linear. Since the intersection region decreases in to a quadratically convergent algorithm. As we shall see in the next chapter, we can use linecr programming to determine such a box: this technique gives rise to a quadratically convergent algorithm regardless of dimension, which performs exceptionally well for n = 2.

## Chapter 6

## The Linear Programming Root Isolation Algorithm

In this chapter, we present and analyze the second of our root-finding algorithms for systems of multivariate polynomials. Since most of the derivation of this algorithm is given in chapter 5, we will concentrate mostly on those aspects of the Linear Programming (LP) algorithm which are different from the Projected-Polyhedron technique. The principal difference between the two methods is the way that they generate bounding boxes for the set of feasible points A' (see figure 6-1). Therefore, we begin by examining the way this box is generated in the LP approach. In the following discussion we will be working with both of the two sets A and A'. Recall that A = A' × {o}; in other words, for every element of A', there is an element of A with the first n coordinates identical and an n + 1st coordinate equal to 0. Technically speaking, the linear programming subroutine will be searching for minima and maxima of a linear function over the set A, but these extrema will be used to bound A'. Distinguishing between the two sets A and A' is unimportant, since they are isomorphic.

## 6.1  Generating the Bounding Box

We start by splitting the problem of determining B = [a1,bi] × [a2,b2] × ... × [an, bn] into the 2n independent subproblems of computing each ai and each b. If we are attempting to compute ai, the question we ask is "if we look at the ith coordinates cf all points lying in A, what is the value of the smallest?" If we are looking for bi, the end of that statement would read "what is the value of the biggest?" More formally,

Figure 6-1: A comparison between a PP and an LP bounding box

<!-- image -->

<!-- formula-not-decoded -->

where i ranges from 1 to n. The proof that A' lies within the box determined by the ai and the b is trivial, so we omit any proof of this fact.

Now, as (6.1) currently stands, it would appear that we need to have the set A to work with. However, we have already stated that the intersection set A is difficult to compute explicitly. The important thing to realize is that we do not actually have to determine A in order to evaluate (6.1); instead we write down a set of constraint equations which uniquely determine A. Then we run an optimization algorithm to determine the minimum and maximum values of c; subject to these constraints. Because all of the constraints turn out to be linear, the optimization algorithm we use is a linear programming algorithm.

It turns out that the constraint equations are quite easy to write down. In order for a point x E Rn+1 to be in A, the following two conditions must hold:

1. For each k between 1 and n, x E C(v(k).

2. Cn+1 = 0.

By the definition of A, these conditions are also sufficient; that is, if the conditions hold for a given point x, then x E A.

In order to use linear programming, we need to rewrite the constraint equations using such that

<!-- formula-not-decoded -->

where the c (k)

(k) in a more useful form:

c(k) such that CI

2. for each k between 1 and n,

3. and for each k between 1 and n,

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Suppose that we wish to solve (6.1) -- that is, find the minimum value of c over all x E A for some specific i. Then, since (6.3) must hold for i = j and for all k, minimizing i is equivalent to minimizing

subject to (6.3), (6.4), and (6.5).

<!-- formula-not-decoded -->

Now, linear programming problems are usually posed as follows: given a p x q matrix C of constraint function coefficients, a g-vector u of nonnegative unknowns, a p-vector r of right-hand sides, and a q-vector c of objective function coefficients, minimize cTu over all possible u which satisfy Cu = r [13]. With a small amount of algebra, we can rewrite our minimization problem in this form. Note that if we want to find the mazimum of cTu instead, we can simply negate all of the elements of c and obtain a minimization problem.

The vector u of unknowns will simply be all of the c(). c(k). Therefore, q, the number of elements of u, will be given by

<!-- formula-not-decoded -->

The 2n constraints obtained by writing (6.4) and (6.5) for each k may be directly inserted into C. However, in order to put (6.3) into C for all j and k which satisfy m*) : （k） ≠ 0, we must eliminate the x; from the equations. When we have done this, the first of the three conditions of theorem 6.1 may be restated as follows:

Corollary 6.2 For all j between 1 and n and all k between 1 and n - 1 such that m() ; if x is in A and if there erists a k' which is the smallest integer such that k &lt; k' ≤ n and

<!-- formula-not-decoded -->

We insert these equations (there are no more than n? -- n) into C directly, resulting in a total number of constraint equations p ≤ n? + n.

Now we can build an algorithm LP-FIND which looks much like PROJECTED-POLYHEDRON based on this box generation scheme. In fact, the only difference between the two occurs in step 3, since the boxes are generated differently. Since the other steps are the same, we will not write the steps of LP-FIND explicitly; instead, we will refer to the steps of the algorithm as we did in chapter 5.

## 6.2Analysis

In this section, we will analyze the performance of the algorithm, just as we did with PROJECTED-PoLYHEDRON. We will use some of the theorems in chapter 5 to show that LP-Find exhibits quadratic convergence. Notice that we can define a bounding-box tree for LP-FIND just as we did for PROJECTED-POLYHEDRON and use it to prove that the algorithm terminates. We also define a notion of an iteration of the algorithm just as we did in chapter 5.

## 6.2.1 Complexity Analysis

As before, we will assume that each polynomial is degree m in each variable, resulting in a total degree of mn for each of the n polynomials. Again, we do not know a priori how many iterations will be required, but we can at least assess the cost of one iteration of LP-FiND.

The cost of linear programming depends on the method used; so let us assume we are using the simplex method. This method operates on the pXq matrix C to generate a solution ue m d u + au 'anrea aqissod asarel sh si d yeya anssy [e1] au (bad)o u Aensn this value if all variables appear in all equations, and this is therefore a conservative estimate. sparse systems of equations and therefore result in a much smaller p. Since q = (m + 1), the total execution time of one call to the linear programming solver is O(n*m"). Note that a total of 2n calls to the solver are necessary -- one maximum and one minimum value for each of n dimensions. The total cost of this number of calls is therefore O(n?m?). (Note that there exist methods for simultaneously optimizing a number of objective functions on the same constraint matrix which are more effcient than naively calling one routine n times [30], but we will not consider such improvements in this thesis.)

As before, we assess the cost of subdivision on the children of the current box, so the current box only has to pay for the subdivision which creates it. As we saw before, this subdivision costs O(n?mn+1). Thus the total amortized cost for an iteration of LP-FIND is O(n5m? +n2mn+1). As we shall see in the next section, LP-FIND exhibits quadratic converThe smaller number of steps often mitigates the extra n3 factor we notice in comparing first terms of the LP and PP methods for small n. Furthermore, the LP method constructs tighter bounding boxes and is therefore likely to involve many fewer splitting steps than the PP method.

## 6.2.2  Quadratic Convergence in n Dimensions

In this section, we will prove that close to an isolated root, the LP algorithm exhibits quadratic convergence regardless of the dimension of the problem, obviously an improvement over the PP technique. As before, the proof involves linearization of the intersecting functions.

Theorem 6.3 Suppose we are given a boz B = [a1,bi]×[a2,b2] × [an,bn] which contains one and only one root xo of a system of polynomials fo,k : [0, 1]n → R where k ranges from 1 to n. Let fk represent the subdivision of fo,k over B. Ececute steps 2-4 of LP-FIND once with these parameters, and let the new box at step 4 be denoted by B' = [a',b'] × [a2,b'2]× [a', bn]. Define e = b-a and e' = b'-a'. There erists a neighborhood U of xo and a X &gt; O cspending {21,22,..,Zn}

Proof:

Step 1: In the interest of clarity, we will define e to be maxz=1{ek}. Let us start by linearizing all n of the fk according to (5.13):

<!-- formula-not-decoded -->

Suppose we determine the solution point in Rn+1 to g1 = 92 = ... = gn = Cn+1 = 0. Now, one solution by Cramer's rule (the solution may not be in [o, 1]n, but we will not worry about that at the moment). Let us call this intersection point p = (p1,P2, ... ,Pn, 0).

Step 2: Pick an arbitrary point q = (q1,92,..., 9n,0) in the intersection set A defined previously. (Recall that A is the intersection of F1, F2, ... ,Fn with the hyperplane Cn+1 = 0.) Now, from (5.15) we know that

<!-- formula-not-decoded -->

for all x E [0,1]n. Substituting (q1, q2,... ,9n) for x, we see that since all fk are zero at this point, 9k must be O(e?). Or equivalently, letting rk = gk(q1,Q2, ... , qn), there exist n points rk = (q1,q2,...,9n, rk) such that

1. rk lies on the linear approximation to F and
2. rk is an O(e?) quantity.

Step 3: Because rk lies on the linear approximation to Fk, it must be true that for each k,

<!-- formula-not-decoded -->

Of course, since p is the intersection point of all of these linear approximations, it must also be true that for each k,

<!-- formula-not-decoded -->

Subtracting (6.12) from (6.11) for each k and defining ok = qk -- Pk, we obtain the system of k equations

<!-- formula-not-decoded -->

Assuming, &amp;gain, that the Jacobian matrix J = is nonsingular, Cramer's rule (x,..,n} enables us to find each ok matrix easily. According to the rule, if we let Jk be the matrix obtained by exchanging the kth column of J with the column vector (r1, r2, ... , rn)T, then

<!-- formula-not-decoded -->

= O(e), and so each Recall (as in the proof of theorem 5.11) that the chain rule gives acj column of J has an element which is O(e). But since every column of J is an O(e) quantity, det J is O(en). Jk has n - 1 columns which are O(e) and one column (rk) which is O(e2), and so det Jk is O(en+1). Thus, ok is O(e).

Step 4: Since &amp; is O(e) and q was chosen as an arbitrary element of A, we conclude that for each k, the kth coordinate of any point in A never deviates from a fixed value (i.e., pk) by more than an O(e) quantity. Therefore, by thr definition of the LP bounding box, the length of each side of the box is an O(e) quantity. But remember that the previous discussion has assumed that the input box is [0, 1] when in fact it is B. Scaling is therefore performed in step 4; sinice the scaling factor is bk - ak = O(e), we conclude that the resulting intervalsize

<!-- formula-not-decoded -->

By applying the definition of the O notation, we conclude that there is some set U containing B (and thus B' and xo) such that the constant X &gt; O hidden within the O notation depends only on U. Translating (6.15) then gives Es = b -a ≤ Xe2, which in turn leads immediately to maxk=1{e} ≤入max2=1{e²}.

## Chapter 7

## Optimization Problems

The problem of determining maxima and minima of polynomiai functions is one that appears often in Computer Aided Design. Perhaps the most common instance is the determination of minimum or maximum distance between geometric entities described by polynomial functions. (Such entities include points, curves, and surfaces.) The formulation of such problems is treated in [47], and we include an example in chapter 8 as well. In this chapter, however, we will focus on strategies for classifying stationary points of polynomial functions as maxima, minima, or saddle points.

The computation of stationary points of distance functions gives rise to another problem: that of non-isolated sets of roots. Although such degeneracies may occur in many classes of root-finding problems, they are especially prevalent in distance computation. As an example, consider two surfaces F(u1,v1) and G(u2,v2) and the function

<!-- formula-not-decoded -->

This polynomial function of four variables describes the squared distance between all pairs of points selected from F and G respectively. The local minima of this function will indicate the points on F and G where the surfaces are locally closest to one another. But suppose the two surfaces actually intersect in a curve. Then every point on that curve represents a local minimum of D, and none of them are isolated. As one can imagine, this situation poses a significant problem for our algorithms. Both the PP and LP techniques are designed to find a finite number of roots, and therefore fall short when operating on a system with an infinite number of roots. If we feed such a system into our techniques, we notice that although areas not containing roots are eliminated, a tremendous number of small boxes around the solution set remain. The computer slows down as computation becomes excessive and memory is exhausted. Some alternative method is clearly needed here, which we will consider later in section 7.2. At the moment, however, we turn our attention to the classification of stationary points.

## 7.1  Classification of Isolated Stationary Points

S      s   r s

Definition 7.1 Suppose that D : R" → R is a scalar feld on Rn. Let x be a stationary point of D, that is VD(x) = 0. Then

1. x is a local maximum if there erists a neighborhood U of x such that for all y E U, D(y) ≤ D(x).
2. x is a local minimum if there ecists a neighborhood U of x such that for all y E U, D(y) ≥ D(x).

To illustrate the application of these definitions, let us begin with a simple one-parameter function D(u) (which may arise, for example, in the point-curve distance problem). Suppose s s uisudxa sns sone e uu 'o = (n),a '! (n)a go zuod Krs e si 0n that

<!-- formula-not-decoded -->

where t E [0, 1] is some specific value depending on h. From definition 7.1, we see that uo is a local maximum if the right hand side of (7.2) is never positive for h close to 0 and a local minimum if the right hand side is nonnegative for h close to 0. Now, if D"(uo) ≠ 0, then by continuity, there is some &amp; such that for ↓hl &lt; 6, D"(uo) has the same sign as D"(uo + ht). Therefore, if D"(uo) &lt; 0, then h2 D"(uo + ht) &lt; 0 for |hl &lt; &amp; and hence uo is a local maximum. Similarly, if D"(uo) &gt; 0, then uo is a local minimum. These results are familiar from calculus.

Unfortunately, the problem becomes more complicated if D"(uo) = 0. In this case we puxa  i pu (y + 0),a go us ua aoqe ue s au series to higher derivatives. For example, if the second derivative is zero but the third derivative is nonzero, then we will have neither a maximum nor a minimum but a point of infection. Consider the function y = c3; in any neighborhood of the stationary point = 0, the function takes on both positive and negative values and thus c = 0 is neither a maximum nor a minimum. If the third derivative is also zero, we have to look at the fourth derivative, and so on. Fortunately, these exceptions are rare, but a robust point classification algorithm should be able to handle such cases.

Similar analysis can be performed with functions of more than one variable. Consider a function D(u, v) of two variables, which might arise, for example, in a curve-curve distance problem. A Taylor expansion around the stationary point (uo,vo) to second order leads to

<!-- formula-not-decoded -->

where t E [O,1]. Instead of one second order term, this time we have three to deal with, which of course makes classification more difficult. However, by simply completing the square with the second order terms, we can easily formulate conditions for minima, maxima, and 2

<!-- formula-not-decoded -->

Now h and k appear only within squared expressions; this new form enables us to ignore the signs of h and k and instead concentrate on the signs of the coeffcients a and (c - ). To on all possible values between -oo and oo, and therefore the first term of the right hand However, the signs of each term are not so easily changed; these are determined solely by the signs of a and (c- ). If both coefficients are nonzero at (uo,vo), a continuity argument Da,(uo+ht,vo+kt) do not change sign as long as (uo + ht, vo + kt) is sufficiently close to (uo, vo). Therefore, in order to determine the signs of these coefficients within a small neighborhood of (uo, vo), we can simply evaluate them at (uo, vo); if the coefficients are nonzero there, then their signs will enable us to classify the stationary point. For example, if both coefficients are positive at the stationary point, then clearly the right hand side of (7.4) is positive within some neighborhood of the stationary point, and hence (uo, vo) can be classified as a minimum. If the first coefficient is negative and the second is positive, then because either term may be made zero at different parts of any neighborhood, the right hand side of (7.4) may be positive or negative arbitrarily close to the stationary point, and thus (uo,vo) is neither a maximum nor a minimum. The other possibilities lead to the following theorem, which is proven more fully in [16]:

## Theorem 7.2

1. If Duu &lt; 0 and D2u &lt; DuuDvv at the stationary point (uo,vo), then (uo,vo) is a local macimum.
2. If Duu &gt; 0 and Dza &lt; DuuDv, then (uo, vo) is a local minimum.
4. If none of the above conditions apply, then it is necessary to ecamine higher-order derivatives.

Notice that the third condition above applies even if Duu = 0. In this case, the second order term reduces to 2bhk + ck2. As long as b ≠ 0, specific choices of h and k within any arbitrarily small neighborhood of the stationary point can make this term either positive or negative, and hence (uo, vo) is neither a maximum nor a minimum.

It is possible to complete the square for functions of more than two variables; however, in order to develop general conditions for minima and maxima of such functions, we will employ the powerful theory of quadratic forms.

Definition 7.3 A quadratic form is an expression of the form xTAx, where x = [Z1 c2 ... n]T is an n-dimensional column vector of unknowns and A is an n X n matrit. A quadratic form xT Ax is said to be

1. positive definite if xT Ax &gt; 0 for all x ≠ 0.
2. positive semidefinite if xT Ax ≥ 0 for all x.
3. negative definite (semidefinite) if -xT Ax is positive definite (semidefinite).
4. indefinite otherwise (i.e. the form takes on both positive and negative values).

To see how these quadratic forms are used, let us take a Taylor expansion of a function of n variables D(u) around the stationary point u = uo:

<!-- formula-not-decoded -->

Here H is the Hessian matrir of D; the element of the ith row, jth column, is given by

<!-- formula-not-decoded -->

Now clearly, if the quadratic form hT Hh is positive definite, then within some neighborhood of the stationary point uo, the right hand side of (7.5) is nonnegative, and therefore uo is a local minimum. Similarly, if the quadratic form is negative definite, then uo is a local maximum.

At this point, we can use a familiar theorem of linear algebra whose proof is given in [43]:

Theorem 7.4 Let xTAx be a quadratic form, let A be a real symmetric matrit, and for i = 1,2,...,n, let d; be the determinant of the upper left i x i submatriz of A (which we will denote Ai). Then the quadratic form is positive definite if and only if di &gt; O for all i.

If the &gt; condition in the above theorem is relaxed to Z, a corresponding theorem applies to positive semidefinite forms (that is, a form is positive semidefinite if and only if d; ≥ 0 for all i).

Theorem 7.4 immediately gives us a necessary and sufficient condition for negative definiteness as well. For, if we let B = --A, the quadratic form xT Ax is negative definite if and only if xT Bx is positive definite if and only if det B; &gt; 0 for all i. But det B; = (-1)i det A and so we have the following corollary:

Corollary 7.5 Let A and d; be as before. Then the quadratic form xTAx is negative definite if and only if (-1)'d; &gt; 0 for all i.

Simply applying these conditions gives us the following theorem:

Theorem 7.6 Let D be as before,and let H be its Hessian matrir, evaluated at the stationary point uo. Let d; be the determinant of the upper left i xi submatriz Hi of H.

1. If di &gt; 0 for all i, then uo is a local minimum.
2. If(-1)'d; &gt; 0 for all i, then uo is a local marimum.

If, on the other hand, the form hT Hh is indefinite, we can conclude that uo is neither a minimum nor a maximum (in two dimensions, such points are usually cailed saddle points), as the following theorem shows:

Theorem 7.7 Let H,D,uo, and d; be as before. Suppose that the di do not satisfy the conditions for definiteness or semidefiniteness. Then uo is neither a local minimum or local marimum.

Proof: Because hT Hh is indefinite, there exist vectors hi and h2 such that hT Hh1 &lt; 0 and h Hh2 &gt; 0. Pick an arbitrary neighborhood U of uo. Then for any ε &gt; 0 smaller than some positive number 6, uo + εh1 E U, uo + εh2 E U, (eh1)T H(εh1) &lt; 0, and (eh2)T H(εh2) &gt; 0.

Using (7.5), we obtain

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Clearly, for sufficiently small ε, the right hand side of (7.7) will remain negative, while the right hand side of (7.8) will remain positive. Thus we have, for sufficiently small ε,

<!-- formula-not-decoded -->

Since U was an arbitrary neighborhood of uo, there is no neighborhood of uo where D(uo) ≥ D(u) for all u E U or where D(uo) ≤ D(u) for all u E U. Therefore uo is neither a local minimum nor a local maximum.

Unfortunately, trouble can occur if the Hessian is only semidefinite and not definite. In this case we need to look athigher order derivatives or use some other method to classify the point precisely. Although this is not too difficult for one-parameter functions, it becomes progressively more involved as the number of variables increase. This subject is beyond the [15]).

## 7.2  Identification of Non-isolated Stationary Points

Because the equation VD = 0 is equivalent to a system of n equations in n unknowns, we expect that in most cases the solution set of this system will consists of a few discrete, isolated points in R?. However, it is possible for the solution set to contain curves, surfaces, or hypersurfaces as well as points. For example, suppose D(c,y) = c?y?; then the solution set of VD = 0 consists of the two lines  = 0 and y = 0.

In this section, we will examine a marching method which shows promise in tracing curves of critical points. This type of degeneracy can occur if, for example, we are trying to find the minimum of the squared distance between two surfaces which happen to intersect. The method we use to trace out such curves involves setting up a system of differential equations which can be solved numerically using a standard ordinary differential equation solver [29].

We begin our discussion by defining the concept of a degenerate critical point [34]:

Definition 7.8 A critical point uo of a function D : Rn → R (that is, a point at which VD = O) is called degenerate if the Hessian matrir of D evaluated at uo is singular.

The following well-known theorem of differential geometry relates the concepts of degeneracy and isolation.

Theorem 7.9 Suppose a critical point uo of D is nondegenerate, i.e. its Hessian is nonsingular. Then there erists a neighborhood U of uo such that for all u E U --{uo}, VD(u) ≠ 0. (A point uo having this property is called an isolated critical point.)

Proofs of this theorem are given in [34] [15]. Unfortunately, the converse is not true; all nondegenerate critical points are isolated, but some isolated critical points are degenerate.

For example, the function f(c,y) = r3 - 3ry² has an isolated critical point at the origin, but its Hessian matrix there is the zero matrix and therefore singular. Nevertheless, this theorem is practically helpful in eliminating most cases.

We will use this theorem to help us detect curve branches of nonisolated stationary points as follows:

1. Run the PP or LP algorithm on our initial box of search, with a fairly coarse level of accuracy. For example, if we start with the search box [O,1], we may run our root-finding algorithm with a tolerance of 10-2 or 10-3.
2. Check the remaining bounding boxes after this step. If we observe a number of boxes adjacent to one another, there may be a curve in the solution set.
3. Use Newton-Raphson iteration to find roots within these boxes. Check the Hessian at these points; if it is singular, it is very likely that a curve exists. Accordingly, choose one of these roots as a starting point for our tracing technique.

This approach is by no means infallible; it is theoretically possible for there to be a number of degenerate isolated roots that will fool us into thinking that there is a curve involved. It is also possible that the solution set is a surface or even a higher dimensional entity. However, in practice these exceptions rarely occur.

Let us now assume that we have found a point on this curve, which we will call f : R -→ R". We will assume that f is a function of the parameter t, which takes on a value 0 at our starting point and that f has unit speed everywhere (i.e. |f(t)| = 1 everywhere). This unit speed condition is imposed to ensure that f has a good parameterization.

Rewrite the equatior VD = 0 as a system of n equations in n unknowns:

<!-- formula-not-decoded -->

Now, in order to stay on the curve f in moving a small distance from f(t) to f(t + dt), we need to know what increments have to be added to u1,...,un. Accordingly, we take a Taylor expansion of each equation around the critical point uo to first order:

<!-- formula-not-decoded -->

Or, rewriting this as a matrix equation, we have

<!-- formula-not-decoded -->

where du = [du1 ... dun]T and H is the Hessian of D(u) evaluated at uo.

Because uo is degenerate, the rank of H should be less than n. In fact, we anticipate that it will be n - 1, since any du satisfying (7.12) must be a vector tangent to f at uo, and hence the nullspace of H should have dimension 1. (It is possible on rare occasions that there is a curve passing through uo but the rank of the Hessian is less than n - 1. This problem may occur if the first-order Taylor expansion in (7.11) contains insufficent information due to zero derivatives. Fortunately, these cases are extremely rare in practice; see [34] for more information.)

Because we need to find a tangent vector to f at uo with unit length, we need to find some vector in the nullspace of H and then normalize it to length 1. The Singular Value Decomposition method [43] may be applied to H to generate such a vector in a stable manner. Then, after making the vector unit length, we pass it to our ordinary differential equation solver as the tangent to f at uo. We can trace backwards by changing the sign of the tangent vector.

## 7.3Remarks

We have shown the derivation of a tracing algorithm for a system of equations resulting from the gradient of a scalar field, since in our experience it is the most commonly occuring case. However, a similar approach may be attempted for more general systems of equations.

The problem of tracing surfaces as well as curves remains to be addressed. Certainly we anticipate surfaces to arise even less often than curves in stationary point problems, but they do occur. (Consider the minimum distance between two concentric spheres.) We anticipate that one may generate a set of points on such a surface in a similar fashion; namely, by starting at a point on the surface, finding the basis vectors of the Hessian's nullspace at that point, and using these basis vectors to find other points nearby. This is a topic of future research.

## Chapter 8

## Examples and Applications

In this chapter, we give a few numerical examples to demonstrate how well the techniques presented in this thesis work. Then, we show how some other problems may be restated as systems of simultaneous polynomial equations and solved with one of the two algorithms. Finally, we include a running time analysis of our algorithms and provide some recommendations on choosing the most efficient algorithm.

## 8.1  Numerical Examples

The following six examples are designed to give the reader a better idea of the capabilities of the two algorithms. Implementation of both algorithms was in AT&amp;T C++ version 2.0. C++ was chosen because both rational and floating point implementations could be written with little modification to the code.

## 8.1.1 Intersecting Two Simple Planar Curves

Our first example shows how to solve a simple planar intersection problem with both techniques. Suppose we want to intersect a circular arc with an elliptical arc

<!-- formula-not-decoded -->

9 digits is (0.894427191,0.447213595). In order to solve them with our methods, we frst This results in the system

Table 8.1: Bounding boxes in the Projected-Polyhedron algorithm for a simple two variable case

| Bounding Boxes from Projected-Polyhedron   | Bounding Boxes from Projected-Polyhedron   | Bounding Boxes from Projected-Polyhedron   | Bounding Boxes from Projected-Polyhedron   | Bounding Boxes from Projected-Polyhedron   | Bounding Boxes from Projected-Polyhedron   |
|--------------------------------------------|--------------------------------------------|--------------------------------------------|--------------------------------------------|--------------------------------------------|--------------------------------------------|
| Iter                                       | a1                                         | b1                                         | a2                                         | b2                                         | Message                                    |
| 1                                          | 0                                          | 1                                          | 0                                          | 1                                          | Binary subdivision in r                    |
| 2                                          | 0                                          | 0.5                                        | 0                                          | 1                                          | No root in box                             |
| 3                                          | 0.5                                        | 1.0                                        | 0.1875                                     | 0.61718750                                 | None                                       |
| 4                                          | 0.74605306                                 | 0.98242188                                 | 0.37952697                                 | 0.48294336                                 | None                                       |
| 5                                          | 0.86764611                                 | 0.92684826                                 | 0.44013056                                 | 0.45158886                                 | None                                       |
| 6                                          | 0.89175192                                 | 0.89838481                                 | 0.44668154                                 | 0.44756546                                 | None                                       |
| 7                                          | 0.89424540                                 | 0.89469787                                 | 0.44717954                                 | 0.44723644                                 | None                                       |
| 8                                          | 0.89441574                                 | 0.89444424                                 | 0.44721146                                 | 0.44721503                                 | None                                       |
| 9                                          | 0.89442648                                 | 0.89442826                                 | 0.44721346                                 | 0.44721368                                 | None                                       |
| 10                                         | 0.89442715                                 | 0.89442726                                 | 0.44721359                                 | 0.44721360                                 | None                                       |
| 11                                         | 0.89442719                                 | 0.89442719                                 | 0.44721360                                 | 0.44721360                                 | Root Found                                 |

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

We can now directly apply either method to this system. Running the Projected-Polyhedron algorithm gives us the results of table 8.1.

This particular example was run at a tolerance of 10-8. Notice the linear convergence rate as we approach the root.

Now let us turn to the LP algorithm. On the first iteration, the LP routine will set up the constraint matrix C' as follows:

Table 8.2: Bounding boxes in the Linear Programming algorithm for a simple two variabie case

| Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   | Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   | Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   | Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   | Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   | Bounding Boxes [@1,b1] × [a2,b2] from LP-Find   |
|-------------------------------------------------|-------------------------------------------------|-------------------------------------------------|-------------------------------------------------|-------------------------------------------------|-------------------------------------------------|
| Iter                                            | a1                                              | b1                                              | @2                                              | b2                                              | Message                                         |
| 1                                               | 0                                               | 1                                               | 0                                               | 1                                               | None                                            |
| 2                                               | 0.375                                           | 1                                               | 0.1875                                          | 0.625                                           | None                                            |
| 3                                               | 0.80012077                                      | 0.94619865                                      | 0.38069053                                      | 0.48248792                                      | None                                            |
| 4                                               | 0.88925091                                      | 0.89831844                                      | 0.44364954                                      | 0.44931108                                      | None                                            |
| 5                                               | 0.89440075                                      | 0.89444401                                      | 0.44720171                                      | 0.44722200                                      | None                                            |
| 6                                               | 0.89442719                                      | 0.89442719                                      | 0.44721360                                      | 0.44721360                                      | Root Found                                      |

<!-- formula-not-decoded -->

and the vector of right-hand-sides r will be

<!-- formula-not-decoded -->

The vector of unknown convex hull coefficients u is constrained to satisfy Cu = r. Now if, for example, we want to find ai, the smallest c-coordinate of the bounding box, then we simply find the minimum value of cTu where c equals

<!-- formula-not-decoded -->

b1, a2, and b2 may be found similarly, with the same matrix C and right-hand side'r, but with different coefficient vectors c. If we let the algorithm continue, we observe the results of table 8.2.

These results were obtained with a tolerance of 10-8. Notice how this example illustrates the quadratic convergence of the linear-programming approach; the number of significant

## 8.1.2 The Wilkinson Polynomial

Our next example attempts to solve the following twentieth degree polynomial which was shown by Wilkinson [45] to be extremely ill-conditioned:

<!-- formula-not-decoded -->

In [10], it is shown that the roots of this polynomial may be extracted with much less error by considering the polynomial in the Bernstein basis before attempting solution. In accordance with this approach, we first scale the polynomial so that all 20 of the roots lie within [0, 1], i.e.

<!-- formula-not-decoded -->

We begin by converting this polynomial into the Bernstein basis, as shown in Appendix A.1. Some care is needed here, because although once the polynomial has been transformed it is fairly stable, nevertheless the conversion process itself is ill-conditioned. To eliminate this problem, we start with exact rational coefficients and perform the conversion in exact rational arithmetic; once the process has been completed, we can easily convert the rational Bernstein coeffcients into foating-point numbers with accuracy on the order of machine precision [19]. Since this is only a one-dimensional problem, both the PP and the LP techniques are quadratically convergent. Given a tolerance of 10-7, the LP technique generated theresultsoftable8.3.

As the table indicates, both the linear programming and subdivision phases were stable enough to find all roots of this extremely unstable polynomial.

## 8.1.3 Significant Points of a Planar Algebraic Curve

A well known problem in computer-aided design is the discovery and subsequent tracing of all branches of an implicit algebraic curve f(u,v) = 0. The computation of turning points = 0) is important in solving this problem [33] [38] [20] [37]. As an example, let us find the critical points of the curve f(u, v) = 0 illustrated in figure 8-1, which is defined by

Table 8.3: Roots of the Wilkinson Polynomial

| Roots of P(c)   | Roots of P(c)      | Roots of P(c)   |
|-----------------|--------------------|-----------------|
| Root number     | root               | error           |
|                 | 0.0500000002437804 | 2.4 × 10-10     |
| 2               | 0.10000000000016   | 1.6 × 10-13     |
| 3               | 0.150000000000427  | 4.3 ×10-13      |
| 4               | 0.200000000000014  | 1.4 × 10-14     |
| 5               | 0.249999999999785  | 2.2 × 10-13     |
| 6               | 0.300000000009474  | 9.5 × 10-12     |
| 7               | 0.350000000092444  | 9.2 × 10-11     |
| 8               | 0.400000000006641  | 6.6 × 10-12     |
| 9               | 0.450000024696511  | 2.5 × 10-8      |
| 10              | 0.499999980911975  | 1.9 × 10-8      |
| 11              | 0.549999999963676  | 3.6 × 10-11     |
| 12              | 0.599999984022245  | 1.6 × 10-8      |
| 13              | 0.649999993324551  | 6.7 × 10-9      |
| 14              | 0.699999993937085  | 6.1 × 10-9      |
| 15              | 0.749999999997894  | 2.1 × 10-12     |
| 16              | 0.79999999503061   | 5.0 × 10-9      |
| 17              | 0.849999968721786  | 3.1 × 10-8      |
| 18              | 0.899999999995995  | 4.0 × 10-12     |
| 19              | 0.949999989125186  | 1.1 × 10-8      |
| 20              | 1.00000000000001   | 1.0 × 10-14     |

Figure 8-1: The curve f(u, v) = 0 of section 8.1.3

<!-- image -->

<!-- formula-not-decoded -->

Taking the two partial derivatives gives us two simultaneous equations which we can use either method to solve. With a tolerance of 10-8, the LP method computes the roots in table 8.4.

## 8.1.4 Stationary Points of a Distance Function

Our next example is essentially a problem of finding extrema. Suppose we are given two implicit algebraic surfaces f(c,y, z) = 0 and g(r,y,z) = 0. Now consider all possible pairs of points p1 = (x1, y1, 21) and p2 = (c2, y2, z2), where p1 lies on f and p2 lies on g. Suppose we want to find those pairs of points which are farthest apart, or closest together. Then As an example, let

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

We will consider C1, V1, Z2, and y2 to be independent variables, while z1 depends on 21 and y1 and z2 depends on c2 and y2. Now, for (c1,y1, Z1,2,y2, z2) to be an extremum, the partial derivatives of D with respect to the independent parameters must be 0. Accordingly, consider Z1, y1, 2, and y2 to be independent, and take partial derivatives of D with respect to these four variables. We obtain

<!-- formula-not-decoded -->

and similar expressions for the other three derivatives. Implicit differentiation of (8.11) and (8.12), gives us

<!-- formula-not-decoded -->

and similar expressions for the partial derivatives

Table 8.4: Critical points of an algebraic curve

| Critical points of f to 8 digits   | Critical points of f to 8 digits   | Critical points of f to 8 digits   |
|------------------------------------|------------------------------------|------------------------------------|
| Root number                        | u                                  |                                    |
| 1                                  | 0.92143996                         | 0.50000000                         |
| 2                                  | 0.83640890                         | 0.73280861                         |
| 3                                  | 0.75000000                         | 0.75000000                         |
| 4                                  | 0.73481004                         | 0.500000C0                         |
| 5                                  | 0.55109110                         | 0.69759134                         |
| 6                                  | 0.50000000                         | 0.50000000                         |
| 7                                  | 0.83640890                         | 0.26719139                         |
| 8                                  | 0.75000000                         | 0.25000000                         |
| 6                                  | 0.55109110                         | 0.30240866                         |

clearly these points are extrema of the square of the distance metric

<!-- formula-not-decoded -->

Table 8.5: Critical points of a distance function

| Critical points of the distance function to 8 digits   | Critical points of the distance function to 8 digits   | Critical points of the distance function to 8 digits   |   Critical points of the distance function to 8 digits | Critical points of the distance function to 8 digits   | Critical points of the distance function to 8 digits   |   Critical points of the distance function to 8 digits |
|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|--------------------------------------------------------|
| Rootnumber                                             | C1                                                     | y1                                                     |                                                     21 | C2                                                     | y2                                                     |                                                     22 |
| 1                                                      | 0.2                                                    | 0.2                                                    |                                                      0 | 0.2                                                    | 0.2                                                    |                                                      1 |
|                                                        | 0.2                                                    | 0.2                                                    |                                                    0.4 | 0.2                                                    | 0.2                                                    |                                                      1 |
| 1                                                      | 0.2                                                    | 0.2                                                    |                                                      0 | 0.2                                                    | 0.2                                                    |                                                    0.6 |
| 1                                                      | 0.2                                                    | 0.2                                                    |                                                    0.4 | 0.2                                                    | 0.2                                                    |                                                    0.6 |

equations into the partial derivatives for D and simplifying gives

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

These four equations together with (8.11) and (8.12) form a system of six polynomial equations in six unknowns, which may now be converted to Bernstein form and solved with the PP technique.

The PP technique yields the four roots in table 8.5 when run at a tolerance of 10-8:

## 8.1.5 A Stationary Point Problem at a Strict Tolerance

Consider the function

<!-- formula-not-decoded -->

e r ss s a  o   s d  s p tolerance of 10-14 gives one real root at (0.72660262158698, 0.72660262158698).

## 8.1.6 A Distance Problem with Tangency

Consider the first quadrant of the unit circle x? + y? = 1 and the portion of the parabola y = 1 - c? lying above the c-axis, both expressed as parametric rational Bezier curves.

We wish to compute the stationary points of the squared distance between the two curves; accordingly, we set D(t,u) = (c1 - c2)2 + (y1 - y2)² (where (c1(t),y1(t) is the parametric equation for the parabola and (c2(u),y2(u)) is the parametric equation for the circle), take partial derivatives with respect to t and u, and set them equal to zero. Because these curves are both assumed to be rational, we end up with one equation of degree 8 in t and degree 9 in u, and one equation of degree 8 in u and degree 9 in t. In this example, because there is a tangency at (0, 1) in the (c,y)-plane and the curves are nearly parallel, the rate of convergence drops and an extensive amount of binary subdivision is performed; consequently, this example takes a fairly long time to run to a tolerance of 10-8. This example has the following pairs of minimum distance points: (0, 1) on the circle and (0, 1) on the parabola; (1, 0) and (1, 0); (%,) and (,).

## 8.2 Other Applications

In closing this chapter, we will briefly touch on some other applications of our algorithms and how they might be used or extended to solve related problems.

## 8.2.1 Systems of Piecewise Polynomials

Up to this point, we have said nothing about extending this algorithm to piecewise polynomials. However, since B-splines possess the convex hull property, such an extension should be fairly straightforward. One simple way to solve such a problem, of course, would be to split the piecewise polynomials into separate Bernstein polynomials [2] [5] in order to solve a number of different polynomial subproblems. But since both the PP and LP algorithms rely only the convex hull property to generate new bounding boxes, reformulating the problem in the Bernstein basis is unnecessary.

Problems analogous to the four we have examined above are often encountered with piecewise polynomial entities. Determining the stationary points of the distance function between two piecewise polynomial entities may seem difficult because multiplication of piecewise polynomials is required; however, recent research by Morken [26] has shown that this is a tractable problem. Whether or not this multiplication of piecewise polynomials multiplication of Bernstein polynomials remains to be studied.

## 8.2.2  More Equations than Unknowns, or Vice-Versa

Throughout this thesis, we have been assuming that the number of equations is equal to the number of unknowns, mostly because the problem is easier to think of in this form. However, there is no particular reason that the two numbers must be equal. One common =： = O, which involves three equations in two unknowns. Solving this problem requires nothing more than a few more constraints in the LP approach, or one more polyhedron to project in the PP method.

Fewer equations than unknowns should also be possible to solve. For example, such problems arise in engineering design of complex structures.

## 8.2.3 Finding Complex Roots

These techniques can easily be applied to find complex roots as well as real roots. Simply substitute u + iv for x in (4.3), where i is the imaginary unit, and separate the real and imaginary parts. In this formulation, the coefficients w(*) ，(k) may be complex. In general, for n simultaneous complex polynomial equations, one obtains 2n real equations.

## 8.3 Choosing an Algorithm

Looking at the complexity estimates per iteration of sections 5.3.2 and 6.2.1 gives a clear indication of which algorithms perform better under various circumstances. There are two cases to consider. If n? ≤ m, the second term of the LP complexity estimate is more important, and thus both the LP and PP algorithms have the same asymptotic time of .w m is the degree of each variable in each equation. The LP algorithm will therefore be better when n is small and when the number of iterations required is considerably less. Because exceptionally well when the tolerance is fairly strict. This is in fact exactly what we observe in practice. In addition, numerical experiments suggest that for small n, the LP is more efficient than the PP for multiple roots.

However, if the problem in question has a large number of roots in the domain, both algorithms are forced to perform a large number of subdivision steps in the early stages of the algorithm; in such cases, when many iterations are needed, the PP algorithm tends to perform better. Since we do not know a priori how many roots a given problem will have, a hybrid algorithm based on both methods is useful for low n. In this approach, we perform a number of fast PP iterations until the sizes of the bounding boxes are smaller than a certain coarse tolerance (for example, 10-3). After the boxes drop below this size, we employ a few LP iterations in order to take advantage of quadratic convergence. This hybrid approach typically performs well for most n = 2 and some n = 3 problems. However, for larger n, LP iterations become too expensive, and we recommend a pure PP approach for such cases.

Table 8.6: Comparison of CPU times of PP, LP, and hybrid algorithms in test cases

| CPU timings (in seconds)   | CPU timings (in seconds)   | CPU timings (in seconds)   |   CPU timings (in seconds) | CPU timings (in seconds)   | CPU timings (in seconds)   | CPU timings (in seconds)   |
|----------------------------|----------------------------|----------------------------|----------------------------|----------------------------|----------------------------|----------------------------|
| 1                          | 23                         | 45                         |                          6 |                            |                            |                            |
| PP*                        |                            | 0.10.44.21.7               |                      23000 |                            |                            |                            |
| LP*                        | 0.2 1                      | 1.12250.7                  |                       1810 |                            |                            |                            |
| 人H                         | 0.2                        | 0.4 65 0.5                 |                       1940 |                            |                            |                            |
|                            | Tol10-810-710-81           | 10-810-1410-8              |                            |                            |                            |                            |

These observations are refiected in table 8.6. The timings correspond to results obtained on a workstation running at 36 MHz. In this table, the PP, LP and hybrid (HY) algorithms are all tested in double precision foating point arithmetic. For the hybrid algorithm, the PP was run at a tolerance of 10-4 before beginning LP iterations. Under the time tests is a row indicating the tolerance at which the examples were run; the examples themselves are numbered according to their section. For instance example number 2 is the Wilkinson polynomial of section 8.1.2. An entry of * in the table means that the example executed too quickly to be measured.

## Chapter 9

## Concluding Remarks

As mentioned in the introduction, although both algorithms are formulated to determine real roots, they can be used to find complex roots as well by simply substituting u + iv for srd       s both algorithms may be extended to systems of m equations in n unknowns, where m ≠ n; a different number of constraints in the LP method, or a different number of polyhedra to project in the PP method.

We have experimented with exact rational arithmetic as well as foating point in our implementations of the PP and LP methods. The rational arithmetic implemementation was motivated by robustness considerations. However, our experience indicates that this addition, our experience suggests that both PP and LP methods are inherently stable; nevertheless, an implementation of both methods in foating-point interval arithmetic could be quite useful in achieving robustness. Recent research in this direction is reported in [24] [22] [23] and in [17].

A few modifications to the algorithms could improve performance. For example, either method might be altered to intersect polynomials expressed in terms of barycentric coordinates [9]. This has already been done for n = 2 in [39] using projections of convex hulls. Another improvement would be the ability to select, by examing the pattern of boundingbox reduction, a better place than the midpoint to split a box when convergence slows.

The relative merits of our methods and Newton-based interval techniques should also be studied. An algorithm along the lines of the proposed technique in section 7.2 needs to be implemented in order to make our algorithms capable of handling non-isolated roots. Finally, more examples need to be run on fully optimized versions of the two algorithms. The resulting comparisons between the two and between other methods would help in choosing an algorithm to fit a given class of problems.

## Appendix A

## Supplemental Algorithms

## A.1  Converting from the Power to the Bernstein Basis

In this section we present some pseudocode which demonstrates how to convert a polynomial in n variables from the power basis to the Bernstein basis. On input, we assume that we are given an array m indexed from 1 to n and an n-dimensional array a of power basis coefficients indexed in the ith dimension from 0 to m[i]. The Bernstein coefficients are stored in the identically sized n-dimensional array b.

This particular algorithm is a simple adaptation to polynomials in n variables of the algorithm for converting univariate polynomials given in [1o]. The extension is straightforward, so we omit the derivation for the sake of brevity.

In the code of figure A-1, we have glossed over a few important details of implementaticn for the sake of clarity; for example, no mainstream computer language will actually allow the expression a[ji][2] ... [in] to index an n-dimensional array unless n is fixed at compiletime. (However, this problem may be resolved by using one-dimensional arrays instead of n-dimensional arrays. For example, a three-dimensional array a with elements ranging from a[0][o][0] to a[mi][m2][m3] could be stored as a one dimensional array Ω of (m1 + 1)(m2 + 1)(m3 + 1) elements. With this scheme, a[i][i][k] would be accessed by referencing o so  g os  s s  + ( + ) + ( + )( + ) take in addressing multidimensional arrays.)

The for loop in line 2 indicates that we loop over all possible multi-indices (j1,J2,..., Ji-1,Ji+1,... ,Jn) which are bounded by the multi-index (m[1], m[2], ... , m[i 1], m[i + 1],... , m[n]).

```
POWER-TO-BERN (a, b, m, n) 1  for i ← 1 to n for (1,j2,...,Ji-1,Ji+1....,Jn) ← (0, 0,... ,0) to (m[1],m[2],... ,m[i - 1],m[i + 1],...,m[n]) 3 for k ← 1 to m[] 4 sum ←- 0 5 for l ← 1 to m[] 9 sum ← sum+ a[j1][2] ... [i-1][][i+1] .. . [n]BINOMIAL(k, l)/BINOMIAL(m[i],l) 7 b[ji][2].. . [i-i][k][i+1]... [n] ← sum
```

Figure A-1: Pseudocode for basis conversion

## A.2 Subdivision of a Multivariate Bernstein Polynomial

In this section we give pseudocode which will allow us to subdivide a polynomial fo of n variables 1,Z2,... ,n. We assume that on input we are given two arrays of real numbers α and b which define a box B = [a[1], 6[1]] × [a[2],b[2]] × .. . × [a[n], b[n] C [0, 1]n. The act of subdivision will allow us to create a new polynomial f which is defined by

<!-- formula-not-decoded -->

We assume that fo and f are n-dimensional arrays indexed in the ith dimension from 0 to m[i], where m is an array indexed from 1 to n. The method of subdivision is De Casteljau subdivision, and the algorithm below is an extension of similar algorithms for 1 and 2 dimensions given in [9]. Because of this similarity, we do not bother to give a proof that it works.

```
DC-BOx-SUBDIVIDE (fo, a, b, m, n, f) 1  for i← 1 to n 2 if b[] = 0 a0 ← 0 else a0 ← a[]/b[i] 3 b0←li] 4 r ← 2D-ARRAY (0, m[i], 0, m[i]) 5 for (j1,j2,...,Ji-1,Ji+1,jn) <- (0,0,...,0) to (m[1],m[2],... ,m[i - 1],m[i + 1],...,m[n]) 6 for k ← 1 to m[] 7 r[k][0] ← fo[][2]..  [i-i][k][i+1] ... [n] 8 for k ← 1 to m[] 9 for l ← k to m[] 10 [1 - y][]q +[1 - ][1 - ](q - 1) → [4][] 11 for k ←- 1 to m[i] 12 [][y] -→ [0][] 13 for k ← 1 to m[] 14 for I ← k to m[i] 15 r[][] ← (1 - a)r[i- 1][k - 1] + α r[][k - 1] 16 for k ←- 1 to m[i] 17 f[i][2]..  [i-i][k][i+1] ..  [in] ← r[m[][m[] - k]
```

Figure A-2: Pseudocode for multivariate subdivision

## Bibliography

- [1]  C. Bliek. Computer Methods for Design Automation. PhD thesis, Massachusetts Institute of Technology, Cambridge, MA, July 1992.
2. S   es a  s  a o 12(4):199-201, 1980.
- [3]  B. Buchberger. Grobner bases: An algorithmic method in polynomial ideal theory. In N. K. Bose, editor, Multicimensional Systems Theory: Progress, Directions and Open Problems in Multidimensional Systems, pages 184-232, 1985. Dordrecht, Holland: D. Reidel Publishing Company.
- [4]  J. Canny.  Generalized characteristic polynomials. Journal of Symbolic Computation, 9:241-250, 1990.
- [5]  E. Cohen, T. Lyche, and R. F. Riesenfeld. Discrete B-splines and subdivision techniques in computer-aided geometric design and computer graphics. Computer Graphics and Image Processing, 14:87-111, 1980.
- [6] T. H. Cormen, C. E. Leiserson, and R. L. Rivest. Introduction to Algorithms. MIT Press, Cambridge, MA, 1990.
7. NJ, 1974.
- [8]  T. Dokken. Finding intersections of B-spline represented geometries using recursive subdivision techniques. Computer Aided Geometric Design, 2:189-195, 1985.
- [9] G. Farin. Curves and Surfaces for Computer Aided Geometric Design: A Practical Guide. Academic Press, San Diego, CA, 3rd edition, 1993.

- [10] R. T. Farouki and V. T. Rajan. On the numerical condition of polynomials in Bernstein form. Computer Aided Geometric Design, 4:191-216, March 1987.
- [11] R. T. Farouki and V. T. Rajan. Algorithms for polynomials in Bernstein form. Computer Aided Geometric Design, 5:1-26, 1988.
- [12] C. B. Garcia and W. I. Zangwill. Global continuation methods for finding all solutions to polynomial systems of equations in n variables. In A. V. Fiacco and K. O. Kortanek, editors, Ectremal Methods and Systems Analysis, pages 481-497. Springer-Verlag, New York, NY, 1980.
- [13] S. I. Gass. Linear Programming: Methods and Applications. McGraw-Hill, New York, 1985.
- [14] A. Geisow. Surface Interrogations. PhD thesis, School of Computing Studies and Accountancy, University of East Anglia, Norwich NR47TJ, U. K., July 1983.
6. Verlag, New York, 1973.
- [16] F. B. Hildebrand. Advanced Calculus for Applications. Prentice-Hall, Inc., Englewood Cliffs, New Jersey, 1976.
- [17] C.-Y. Hu. Robust Algorithms for Sculptured Shape Visualization. Master's thesis, Massachusetts Institute of Technology, Cambridge, MA, July 1993.
- [18] R. B. Kearfott. Decomposition of arithmetic expressions to improve the behavior of interval iteration for nonlinear systems. Computing, 47:169-191, 1991.
- [19] D. E. Knuth. The Art of Computer Programming, Vol. 2, Seminumerical Algorithms. Addison-Wesley, Reading, Massachusetts, 1981. 2nd Edition.
- [20] G. A. Kriezis, P. V. Prakash, and N. M. Patrikalakis. A inethod for intersecting algebraic surfaces with rational polynomial patches. Computer Aided Design, 22(10):645654, December 1990.
12. Informations-Behandling, 21(1):112-117, 1981.

- [22] T. Maekawa. Robust Computational Methods for Shape Interrogation. PhD thesis, Massachusetts Institute of Technology, Cambridge, MA, June 1993.
- [23] T. Maekawa and N. M. Patrikalakis. Interrogation of differential geometry properties for design and manufacture. The Visual Computer, May 1992. Revised April 1993. To appear.

：

- [24] T. Maekawa and N. M. Patrikalakis. Computation of singularities and intersections of offsets of planar curves. Computer Aided Geometric Design, 10:407-429, 1993.
- [25] D. Manocha and J.F. Canny. A new approach for surface intersection. International Journal of Computational Geometry and Applications, 1(4):491-516, 1991.
- [26] K. Morken. Some identities for products and degree raising of splines. Constructive Approrimation, 7:195-208,1991.
- [27]  A. Neumaier. Interval Methods for Systems of Equations. Cambridge University Press, Cambridge,1990.
- [28] T. Nishita, T. W. Sederberg, and M. Kakimoto. Ray tracing trimmed rational surface patches. ACM Computer Graphics, 24(4):337-345, August 1990.
- [29] Numerical Algorithms Group, Oxford, England. NAG Fortran Library Manual, Volumes 1-8, Mark 14 edition, 1990. 1
- [30] C. H. Papadimitriou and K. Steiglitz. Combinatorial Optimization: Algorithms and Complerity. Prentice-Hall, Englewood Cliffs, NJ, 1982.
- [31] N. M. Patrikalakis and G. A. Kriezis. Representation of piecewise continuous algebraic surfaces in terms of B-splines. The Visual Computer, 5(6):360-374, 1989.
- [32] N. M. Patrikalakis, T. Maekawa, E. C. Sherbrocke, and J. Zhou. Computation of singularities for engineering design. In T. L. Kuni and Y. Shinagawa, editors, Modern Geometric Computing for Visualization, pages 167-191. Tokyo: Springer-Verlag, June 1992.
- [33]  N. M. Patrikalakis and P. V. Prakash. Surface intersections for geometric modeling. Journal of Mechanical Design, ASME Transactions, 112(1):100-107, March 1990.

- [34] T. Poston and I. Stewart. Catastrophe Theory and its Applications. Pitman, San Francisco, CA, 1978.
- [35] P. V. Prakash and N. M. Patrikalakis. Surface-to-surface intersections for geometric modeling. Technical Report MITSG 88-8, Cambridge, MA: MIT Sea Grant College Program, 1988.
- [36] F. P. Preparata and M. I. Shamos. Computational Geometry: An Introduction. Springer-Verlag, New York, 1985.
- [37]  T. Sakkalis. The topological configuration of a real algebraic curve. Bulletin of the Australian Mathematical Society, 43:37-50, 1991.
- [38]  T. Sakkalis and R. T. Farouki. Singular points of algebraic curves. Journal of Symbolic Computation, 9:405-421, 1990.
- [39] T. W. Sederberg. Algorithms for algebraic curve intersection. Computer Aided Design, 21(9):547-554, November 1989.
- [40] T. W. Sederberg and T. Nishita. Geometric Hermite approximation of surface patch intersection curves. Computer Aided Geometric Design, 8:97-114, 1991.
- [41] E. C. Sherbrooke and N. M. Patrikalakis. Computation of the real solutions of nonlinear polynomial systems. Memorandum 91-12, Cambridge MA: MIT Ocean Engineering Design Laboratory, October 1991. Revised June 1992.
- [42] E. C. Sherbrooke and N. M. Patrikalakis. Computation of the solutions of nonlinear polynomial systems. Computer Aided Geometric Design, 10:379-405, 1993.
- [43] G. Strang. Linear Algebra and its Applications. Harcourt Brace Jovanovich, San Diego, CA, 1988.
- [44]  M. E. Vafiadou and N. M. Patrikalakis. Interrogation of offsets of polynomial surface patches. In F. H. Post and W. Barth, editors, Eurographics '91, Proceedings of the 12th Annual European Association for Computer Graphics Conference and Ezhibition, pages 247-259 and 538, Vienna, Austria, September 1991. Amsterdam: North-Holland.
- [45] J. H. Wilkinson. Rounding Errors in Algebraic Processes. H. M. Stationery Office, London, England, 1963.

- [46] W. I. Zangwill and C. B. Garcia. Pathways to solutions, fired points, and equilibria. Prentice-Hall, Englewood Cliffs, NJ, 1981.
- [47]  J. Zhou, E. C. Sherbrooke, and N. M. Patrikalakis. Computation of stationary points of distance functions. Engineering with Computers, June 1992. Revised March 1993. To appear.