## Abstract

Evaluating the intersection of two rational parametric surfaces is a recurring operation in solid modeling. However, surface intersection is not an easy problem and continues to be an active topic of research. The main reason lies in the fact that any good surface intersection technique haa to balance three conflicting goals of accuracy, robustness and efficiency. In this paper, we formulate the problems of curve and surface intersections using algebraic sets in a higher dimensional space. Using results from Elimination theory, we project the algebraic set to a lower dimensional space. The projected set can be expressed as a matrix determinant. The matrix itself, rather than its symbolic determinant, is used as the represent ation for the algebraic set in the lower dimensional space. This is a much more compact and efficient represent ation. Given such a representation, we utilize properties of straight line programs and results from linear algebra for performing geometric operations on the intersection curve. Most of the operations involve evaluating numeric determinants and computing the rank, kernel and eigenvalues of matrices. The accuracy of such operations can be improved by pivoting or other numerical techniques. We use this representat ion for inversion operation, computing the intersection of curves and surfaces and tracing the intersection curve of two surfaces in lower dimension.

Permission to copy without fee all or part of this matertial is granted provided that the copies are not made or distributed for direct commercird advantage, the ACM copyright notice and the title of the publication and its date appear, and notice is given that copying is by permission of the Association for Computing Machinery. To copy otherwise, or to xepublish, requires a fee and/or specific permission,

<!-- image -->

## A New Approach for Surface Intersection

## Dinesh Manocha and John Canny

Computer Science Division University of California Berkeley, CA 94720

## 1 Introduction

Many current geometric and solid modeling systems use parametric curves and surfaces for designing geometric objects to satisfy various interpolator, smoothness and aesthetic requirements. When it comes to dealing with intersections of these curves and surfaces, current algorithms fail to meet the minimum standards of robustness, accuracy and efficiency that these applications demand. Most of the difficulties arise due to the fact that any exact algorithm for these problems tends to be slow due to the large coefficient size of the exact result. The approximate algorithms are far from being robust and their accuracy varies with the surface degree, with the local surface geometry at the intersection curve, and the angle at which the surface intersect. hloreover, the numerical stability of these calculations is not completely understood, but appears to be poor for at least some of the necessary calculations. As a result, it is widely believed that any good surface intersection technique has to balance three conflicting goals of accuracy, robustness and efficiency [H089; PG86].

Earlier approaches to surface intersection used recurs,ve subdivision techniques based on the paradigm of 'divide and conquer' [LR80; PG86]. The main idea is to subdivide the surfaces into small pieces until each piece satisfies some flatness criterion. However the main problem with the approach lies in constructing the topology of the resulting curve from the individual intersection pieces. The algorithm is not robust and may fail in the presence of simple singularities.

Another approach for evaluating surface intersections is that of tracing the intersection curve [BHLH88; BK90; PG86]. The main idea behind tracing techniques is to compute a starting point on each component and locate all the singular points. Given the starting points, these algorithms use marching methods to trace the intersection curve and in the process use robust methods to determine all the branches at singular points.

It is possible to represent the intersection curve ss an algebraic set in the higher dimensional space spanned by the parameters of two surfaces. Given such a formulation, techniques like implicitization can be used to obtain a closed form and exact representation as an algebraic plane curve of the form ~(u, v) = O, where .f(u, v) is a bivariate polynomial [Fa86]. Such a representation is obtained by implicitizing one of the parametric surfaces and substituting the other parametrization into the implicit representation. When it comes to tracing, we may choose to trace the curve in the higher dimension or its projection in the lower dimension [H088]. However, [H088; H090] counsels against computing the projection in the lower dimensional space for the following reasons:

- Implicitizing a parametric surface entails substantial symbolic computation for degrees higher than cubic. Generally resultants are used for implicitization and it is believed that the use of resultants may introduce extraneous factors, which pose additional problems.
- Substitution of the parametric formulation into the implicit form, although conceptually simple, is numerically delicate and can lead to substantial errors. This is mainly due to catastrophic cancellation [P P88].
- By Bezout's theorem, the degree of the intersection curve is equal to the product of the degree of the intersecting surfaces. Thus, with the resulting high algebraic degrees numerical difficulties arise even when evaluating the resulting curve at some point.

As far as the problem of computing a point on each component is concerned, a recent approach of loop detection is presented in [SM88]. The basic idea is to subdivide the parametric surfaces, such that the intersection of subdivided surfaces (considered piecewise) has no closed loops and all starting points can be obtained by the intersection of boundary curves of one surface patch with the other surface. The alg~ rithm in [SM88] does not account for singular points and has been found slow in practice. Thus, no good algorithms are available for finding a point on each component or the singular points on the intersection curve.

We use results from Elimination theory to represent the implicit equation of a rational parametric surface as a matrix determinant [MC90a]. The main idea involves using the matrix itself, rather than its symbolic determinant, for representing the implicit representation and the projection of intersection curve. To evaluate such a representation we use numeric substitution and Gauss elimination. The resulting algorithm is efficient and its numeric accuracy is improved by techniques like pivoting. Furthermore, we use properties of straight line programs to evaluate the partial derivatives of the function used for representing the intersection curve. As a result, we are able to reduce the problems of finding a starting point on each component and the singular points to curve-surface intersection and computing solutions of nonlinear equations.

We use our representation in coming up with efficient and robust algorithms for curve-surface intersection and computing the inverse image of a point on the parametric surface. The techniques involved are computing the eigenvalues and eigenvectors of a matrix, singular value decomposition and determinant computation. The numerical accuracy of such opera tions is well understood and efficient implementations are available as part of standard packages like LINPACK and EISPACK [BDMS79; GBDM77]. Moreover, we use this represent at ion for tracing the intersection curve in lower dimension.

The rest of the paper is organized in the following manner. In Section 2 we present some background material on surface intersection, implicitizing parametric surfaces and how Elimination theory can be used for representing the implicit equation of a parametric surface as a matrix determinant. This formulation is used for representing the intersection curve and efficient and numerically stable algorithms are presented for evaluating the function and its partial derivatives in Section 3. In Section 4 we present two main applications of our representation: computing the inverse image of a point and curve-surface intersection. We also present results of our implement ation of these applications. Finally in Section 5, we address the problem of tracing the intersection curve and reduce the problem of finding a point on each component and the singular points to equation solving. We consider the curve represented in higher dimensional space as well as its projection in the lower dimensional space. Various approaches for equation solving are compared and some open issues are presented.

## 2 Background

The problem of surface intersection involves computing an appropriate representation of the intersection curve and designing suitable algorithms for evaluation and performing geometric operations. The exact requirements on the representation and algorithms are application dependent. In general, any representation should provide functionality for the following operations:

- Evaluate and render the intersection curve (all components and branches).
- b Decide whether a point (X, Y, Z) lies on the intersection curve.
- Sort the points lying on the intersection curve.
- e Use it as a boundary edge for a trimmed surface.

Many a times the intersection curve consists of a single component without any singularities. However, even simple cases like intersection of two cylinders can give rise to singularity. In this case the intersection curve is an algebraic space curve of degree four. For tensor product bicubic B6zier patches the intersection curve is a space curve of degree 324 and it is simple to come up with cases where the intersection curve has more than one component (as shown in Fig. I) and thereby adding to the complexity of the intersection problem.

Fig. I Intersection curve consisting of eight components

<!-- image -->

## 2.1 Implicitization

It is well known in algebraic geometry that if one of the surfaces is represented parametrically and the other one implicitly, an implicit representation of the intersection curve can be obtained by substituting the parametric formulation into the implicit representation. A B&amp;zier surface is a rational parametric surface. The fact that the set of rational parametric surfaces is a proper subset of the algebraic surfaces implies that every B6zier surface can be represented as an algebraic surface of the form ~(z, y, z, w) = O, where .f(x, y, z, w) is an irreducible homogeneous polynomial [SR85]. In other words, given two B6zier surfaces

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

Implicitize F(sJ t) into an algebraic surface of the form ~(z, y, z, w) = O. Substitute the other parametrization ~o obtain an exact representation the intersection curve as: of

<!-- formula-not-decoded -->

The problem of implicitization corresponds to formulating parametric equations like

<!-- formula-not-decoded -->

and eliminating the variables s and t. Techniques for eliminating variables have been well known in classical algebraic geometry for more than a hundred years [Sa1885; Wd50]. As far as geometric and solid modeling are concerned, these techniques were resurrected by [SAG84]. In particular, [SAG84] used resultants to implicitize tensor product surfaces into their corresponding implicit representation. Some recent approaches to the problem of implicitization include the use of Gr6bner bases [Bu85; H089]. However, most of the results were negative and in general, it is believed that any algorithm based on the implicitization approach can be inefficient and numerically unstable for surface intersection for reasons highlighted in the previous section and given in detail in [H088; H090].

The problem of implicitization has recently been analyzed in [Ch90; MC90a]. In particular, it has been shown that if a parametrization haa no base points, the resultant of the parametric equations corresponds exactly to the implicit representation. The base points of a parametrization are defined as the common rootsl of

<!-- formula-not-decoded -->

For a random choice of coefficients a parametrization has no base points. The three parametric equations are of equal degree and [MC90a] use this property along with results from Elimination theory [Di08] to show that the implicit equation can be represented as a matrix determinant. For a tensor product surface of the form (Smtn) the order of the matrix is 2rnn and for a triangular patch of degree n the matrix has order 2n2 -n [MC90a]. Each entry of the matrix is a linear polynomial of the form

1They also include the roots at infinity

<!-- formula-not-decoded -->

where aij, bij, cij and dij are rational functions of the given parametrization. If a parametrization has simple base points, it is still possible to represent the implicit represent ation as a matrix determinant. An efficient algorithm based on Vandermonde interpol~ tion has been presented in [MC90a] for computing the resultant of polynomial equations and thereby used for implicitization. In particular, it has been shown that it is possible to implicitize parametric% tions like tensor product bicubic surfaces in less than two minutes on machines like the IBM RS/6000. The implementation has been restricted to exact arithmetic. Although this algorithm is fast for practical applications, it becomes unattractive due to issues of numeric stability in the context of floating point computations and the degree of the resulting polynomials obtained after substitution.

## 3 Representation of Intersect ion Curve

We make use of results of [Di08] and [MC90a] to represent the implicit equation as an unexpanded determinant and present algorithms to perform geometric operations on the represent at ion. Given two B6zier surfaces, F(s, t) and G(u, v) as in (l), we use the corresponding formulation of resultant (depending on the fact whether F(s, t) is a tensor product surface or a triangular patch) and represent the implicit equation as a matrix determinant. The main idea is to use the matrix itself, rather than its symbolic determinant, as a representation for the implicit form and subsequently for the intersection curve. Later on we will show that this is a much more compact, efficient and numerically stable represent ation.

Given ~(z, y, z, w), as a matrix determinant

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where ~ij (%, y, z, w) is a linear polynomial of the form aij x + bij y + cij z + dij W. The coefficients aij , bij , Cij and dij can be represented as rational functions of the given parametrization. Furthermore, their computation is efficient and numerically stable.

The implicit representation of intersection curve is obtained by substituting the parametrization G(u, v) into ~(x, y, z, w). As a result, the intersect ion curve is represented as zero set of a determinant. The corresponding matrix is

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

In this case we substitute the parametrization into a linear polynomial of the form ~ij (z, y, z, w). In practice, we represent each entry of M(u, v) as 4d). As a result, there is no sigtuple of (aij, bij, Cij, ,3 nificant loss of information due to catastrophic cancellation, which is the case when the implicit representation corresponds to a polynomial of degree n [PP88]. The representation of the curve corresponding to Fig. I has been shown in Fig. II. In this case, the planar curve (which is birationally equivalent to the space curve) has eight components. In particular, the curve has two kind of components. Closed loops are shown as Cl and C2 and the open components (the ones that intersect with the boundary) are 01,..:,06.

The representation of i~tersection curve in Fig. I

<!-- image -->

## 3.1 Matrix Operations on the Representation

We make use of the representation presented in the previous section to perform geometric operations. We use the notations ikf(u, v) to denote the matrix used for representing the intersection curve and D(u, v) to represent the polynomial corresponding to Determinant(M(u, v)). For geometric operations like tracing or marching through the intersection curve, we need to effectively evaluate expressions like D(ul, WI), DU(U1, vI), D''(ul, vi), where ul and vl are real numbers and D* and DUU represent the first and second partial derivatives with respect to u.

D(ul, VI) can be efficiently and accurately evaluated in the following manner:

- . Compute the entries of ikf(ul, V1) by evaluat---ing Bernstein polynomials X(ul, V1), Y(UI, W), ~(ul, vl ) and ~(ul, w ) and taking their respective combinations. Techniques for efficient and accurate evaluation of Bernstein polynomials are well known [Fa86].
- Given JM(u1, vi), a matrix with numerical entries, use Gauss elimination to compute D(uI, WI). Furthermore, use pivoting techniques to improve the numeric stability of the resulting computation [GV89; Wi63]. It is possible to compute the condition number of the matrix and come up with a tight bound on the numerical accuracy of the result. Furthermore, efficient and well tested software for such operations is available as part of LINPACK [BDMS79].

To compute the first and higher order partials, we make use of the fact that a determinant can be represented as a straight line program. In this case, we modify the matrix structure such that entry con-( sists of a tuple Gi~(~l) ~1) = (gi~(ulj ~l),g~ al, U1 ))! where g; (ul, Vl) represents the partial derivative of gij (u, v) with respect to u and specializing u = U1 and v = V1. The resulting matrix structure is of the form

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

To compute D(ul, Vl) and D' (ul, Vl) we perform Gauss elimination. We consider the matrix formed by first entry of each tuple (equivalent to itf(ul, q)) and proceed as if we are trying to compute its determinant. As a side effect we change the entry in the second tuple. Assume we are operating on the ith and kth rows of the matrix. A typical step of Gauss elimination is of the form

<!-- formula-not-decoded -->

where gkj represents the element in the kth row and jth column of the matrix. In the new formulation this step is replaced as

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

We make a choice for the pivot element based on the first tuple (i.e. gij entry). After Gauss elimination is complete, we compute D(ul, VI) and D' (ul, Vl) in the following manner:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

This procedure can be easily extended to compute the higher order partials. Furthermore, the analysis of Gauaa elimination may be used for analyzing the numerical accuracy of partial derivatives computation. To insure the numeric stability of Gauss elimination, it is required that the intermediate c~ efficient being generated do not grow in magnitude. Since gii occurs in the denominator term in (4), the pivoting process chooses gii to be the element of maximum magnitude (with respect to the ith column or the rest of the matrix). The computation of g~j involves division by (gii)z. At the moment we have partial results from this analysis and our implementation indicating that this method should be numerical stable.

## 4 Applications of the Representation

In this section we highlight two main applications of our represent ation. They are computing the inverse coordinates for a point (zl, ~1, Zl) lying on the surface and the intersection of curves and surfaces. Both these operations are used in the algorithm used for tracing intersection curves in the next section.

## 4.1 Inversion Operation

Frequently we are given a point (xl, yl, Z1) and asked to determine if this point lies on the intersection curve of F(s, t) and G(u, v). Previous approaches to this problem take the surface parametrization and formulate the equations

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and determine whether these equations have a common solution in the range O ~ s &lt; 1 and O ~ t &lt; 1 (assuming that we are considering the intersection of B6zier surfaces). The same procedure is repeated for the other surface. However, root finding can be slow in practice and in this section, we show how to effectively use our representation for solving the inversion problem

) :

<!-- image -->

,

:

The implicit representation of the surface is the determinant of ~(x, y, z, w), as formulated in (2). The point (ZI, yl, Zl) lies on the smface if and only if the determinant of (f(xl, yl, Z1, 1)) obtained after substitution is zero. While using floating point arithmetic, it is also possible to obtain tight bound on the errors of computation [Wi63].

We used results from Elimination theory to represent the implicit equation as a matrix determinant

[MC90a]. Assume that the given surfaces are tensor product surfaces of the form (s%~). Techniques from Elimination theory take the following parametric equations:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and reduce it form to a problem in linear algebra of the

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and M is equivalent to $(z, y, z, w) in (2) and n = 2dz.

The fact that the determinant of ~(xl, yl, ZI, WI ) is zero (or very close to zero) implies that the numeric matrix is singular (or nearly singular). Let us assume that the given parametrization is faithful and (Z1, yl, Z1, Wl) does not correspond to a singular point. Consequently it has a unique preimage and ~(zl, yl, Z1, WI) is a matrix whose kernel has dimension one. We can use a technique like SVD (singular value decomposition) to accurately determine the vector in that kernel [GV89; Wi63]. Let that vector be V = (W1V2 . . . ~2d2)T. As a result, the preimage of the point (zl, yl, Z1, Wl) can be obtained by solving the equation

<!-- image -->

<!-- formula-not-decoded -->

where k is a scalar. Good implementations of SVD are available as part of LINPACK [BDMS79]. Since we are only interested in verifying whether zero is an eigenvalue of the matrix and compute the corresponding eigenvector, we may use routines for condition estimator (SGECO from LINPACK) [BDMS79].

It is more efficient than computing the SVD of the matrix.

A similar procedure can be applied to the other parametrization. Thus, the problem of inversion has been reduced to the computation of determinants and the kernel of matrices.

## 4.2 Intersecting Curves and Surfaces

Given a B6zier curve

<!-- formula-not-decoded -->

of degree d and a B6zier surface F(s, t). Let ~(z, y, z, w) be the matrix corresponding to its implicit representation. The problem of intersecting G(u) and F(s, t) can be reduced to solving three nonlinear equations in three unknowns. However, our aim is to find all the roots in the domain of interest (as shown in Fig. IV). Algebraic techniques like resultants and Gr6bner bases are too slow for practical usage and in the context of floating point computation, the accuracy of results using these methods is not completely understood. Newton's method is fast and numerically stable, however it does not guarantee all the roots in the given domain.

Fig. IV Intersecting curves and surfaces

<!-- image -->

In this section we use our formulation of the implicit representation and reduce the problem of curve- surface intersection to an eigenvalue problem. Efficient algorithms for solving eigenvalue problems are well known in numerical analysis [GLR82; GV89]. Furthermore, good implementations of these algorithms are available aa part of a package like EISPACK [GBDM77].

Given ~(z, y, z, w), we substitute the parametric equation of the curve to obtain a matrix of the form

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

The determinant of Al(u) is a univariate polynomial in u, and its roots correspond to the preimages of intersection points between the curve and the surface. Each entry of M(u) is a univariate polynomial of degree d and let us represent it as matrix polynomial

<!-- formula-not-decoded -->

where itfi's are matrices of order n with numeric entries. Let us assume that ~d is a non-singular matrix. As a result the roots of the following equations are equivalent

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Let

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

and In is an n x n identity matrix. Given ~(u), we use Theorem 1.1 [G LR82] to construct a matrix of the form

<!-- formula-not-decoded -->

such that the eigenvalues of C correspond exactly to the roots of Det(~(u)) = O. C is a numeric matrix of order dn. If Md is a singular matrix, techniques to compute the roots of Det (M(u)) = O are given in [GLR82].

Most of the currently known algorithms find all the eigenvalues of the given matrix. In our applications, we are only interested in the eigenvalues lying in the domain, say [u1, u2]. For example, when we are dealing with B6zier curves and surfaces, the domain is [0, Ii.

## 4.2.1 Implemental ion

We used EISPACK routines for computing the eigenvalues of matrices. Many special purpose algorithms are available for computing the eigenvalues of matrices, which make use of the structure of the matrix. As far as matrix C in (5) is concerned, we treat it as a general unsymmetric matrix. We use the routine RG from EISPACK for computing the eigenvalues [G BDM77]. Given a general unsymmetric matrix, it makes use of balancing techniques, reduces it to upper Hessenberg form and uses the shifted QR algorithm on the resulting matrix to compute the eigenvalues [GV89]. The current implementation of these routines compute all the eigenvalues. The performance of eigenvalue computation routines for matrices of different order are given in Table I. The timings correspond to the implementation on an IBM Rs/6ooo.

Table I

|   Order of Matrix | I Time in seconds   |
|-------------------|---------------------|
|                15 | i 8631.839844X 10-6 |
|                20 | 15717.63965 X 10-6  |
|                25 | 25753.00000 X 10-6  |
|                30 | 38763.23828 X 10-6  |
|                35 | 57124.16016 X 10-5  |
|                40 | 77398.03906 X 10-6  |
|                45 | 103343.5234 X 10-6  |
|                50 | 133956.2344 X 10-6  |
|                55 | 165395.0469 X 10-6  |
|                60 | 212041.2812 X 10-6  |
|                65 | 262103.1250 X 10-6  |

The performance of eigenvalue computation routines

While considering the intersection of a cubic B6zier curve with a bicubic patch, the order of C in (5) is 54. However, the performance of the implementation presented above can be improved by making use of the structure of C to reduce it to an upper Hessenberg form. Typically, the intersection of a B6zier curve with a bicubic patch result in two or three intersections in the domain of interest and we are only in interested in those eigenvalues. The current implementation computes all the 54 eigenvalues and many of them are complex numbers, too. However, this algorithm guarantees all the intersection points and is therefore, robust.

This technique is also directly applicable for ray tracing parametric surfaces. Every ray is a parametric curve of degree one and as a result each entry of ill(u) is a linear polynomial. It is simple to find the intersections of the ray with the control polyhedra (of the parametric surface). Those bounds can be used to define the domain of the variable u. For ray tracing bicubic patches, the order of C' is 18.

## 5 Surface Intersections

In the previous sections, we presented a represent tion of the projection of the intersection curve, as a matrix determinant, M(u, v) in (3). The determinant is denoted as D(u, v). In this section we consider the problem of intersecting two B6zier surfaces. To evaluate the intersection curve, we use the marching technique. Therefore, we need to determine a point on each component of the intersection curve and all the singular points, We consider the problem of tracing the intersection curve in lower as well as higher dimensional space.

Tracing in lower dimensions correspond to tracing an algebraic plane curve. The main advantage of this approach lies its abilities to deal with singularities. Simple singularities like cusps and loops can be easily characterized. Moreover, there is an elegant theory of resolution of singularities in algebraic geometry, which can be used for dealing with complicated singularities [Wa50]. An analogous process for surface intersection in higher dimensional space could be devised in principle, but it would be substantially more complex because it would map the intersecting surfaces simultaneously such that the singularity of the intersection would be resolved [H089].

It is widely believed that the lower dimensional approach has to cope up with a number of practical difficulties. Some of them were highlighted in Section 1 though a detailed analysis has been presented in [H088; H090]. However, we feel that our representation in terms of an unexpanded determinant and subsequent operations using techniques from linear algebra and matrix computations take care of the practical problems highlighted in [H088; H090]. As a result, it is worthwhile to reconsider the projected curve in the lower dimensional space as a practical method for evaluating surface intersections. In this section, we reduce the problem of finding a point on each component and singular points to curve-surface intersection and solving systems of non-linear equw tions. Therefore, we do not need techniques like loop detection for finding a point on each component.

## 5.1 Curve in Lower Dimension

In this case the curve has been represented as a m~ trix determinant, D(u, v). The singular points of the projected curve correspond to the roots of the equations [Wa50]

<!-- formula-not-decoded -->

and as a result, the problem of computing singular points has been reduced to equation solving.

The intersection curve has two kinds of components:

Closed loops :-They are of the form Cl and C2 in Fig. II. In other words, the component is contained in the domain and does not intersect with any of the four lines, u = O, u = 1, v = O and v = 1. In this case, at least one point on the component satisfies the equations

<!-- formula-not-decoded -->

and we can compute it by finding all the roots of these equations in the appropriate domain.

Open components :-All other components fall into this category (01, . . . . 06 in Fig. II). Points on such components can be determined by finding the intersections of the curves corresponding to G(O, v), G(l, v), G(u, O) and G(u, 1) with the surface F(s, t). Techniques for curve-surface intersection were presented in the previous section.

Thus the problem of computing a point on each component and the singular points has been reduced to curve-surface intersection and finding all roots of

<!-- formula-not-decoded -->

Given a root of the above equations, it can be substituted into the equation D' (u, v) = O to check whether it corresponds to a singular point.

## 5.2 curve in Higher Dimension

We consider the surfaces F(s, t) and G(u7 v) as defined in (l). The curve is defined in the parameter space of both the surfaces as an algebraic set of the form

<!-- formula-not-decoded -->

The components of the curve can be classified into closed loops and open components. The points on the open components can be computed by reducing the problem to curve surface intersection. As far as the problem of finding points on closed loops and singular points is concerned, we use an approach similar to the one used for the curve in the lower dimensional space. Lets consider the algebraic set defined by (8) and the extrema in the s direction is obtained by considering the total derivatives of the three equations and substituting ds = O. The resulting equation is formulated by considering

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Let J correspond to the matrix on the left hand side of the above equation and all extremal points of the curve in s direction satisfy the equation

<!-- formula-not-decoded -->

All the singular points on the curve satisfy the above equation, too. Furthermore, they also satisfy the extremal equations corresponding to t,u and v direction. As a result at least one point on each closed loop and the singular points are contained in the roots of

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

These are four equations in four unknowns and hence have a finite number of solutions (in general).

## 5.3 Equation Solving

In the previous section we reduced the problem of computing singular points and a starting point on closed loops to finding roots of nonlinear equations. In particular, the equations for curves in lower and higher dimensional space are highlighted in (7) and (9), respectively. In general, the complexity of exact algorithms for finding solutions of nonlinear equations is a function of Bezotit number of the given system. The Be.zout number corresponds to the total number of solutions that the system has in the complex projective space (counted properly) [Sa1885; WM88]. Given a system of equations, techniques for computing the Bezout number are presented in [WM8tl]. The domain is not restricted to a subset of the real space (as in our case). Lets consider the case of intersecting two bicubic B6zier surface patches. The intersection curve is of degree 324 (in general). Lets analyze the problem of finding roots of the equations for this case. For the projected curve, the monomials of highest degree in the system of equations (7) are ~54v54 and U53V54. As a result, its Bezout number is 5778. In other words, the system has 5778 non-trivial solutions in the complex projective plane. Similarly, the Bezout number of the syste,m (9) is 5346.

The high Bezout number of these system make algebraic techniques like resultants and Gr6bner bases impractical [Bu85; MC90b]. Furthermore, in the context of floating point computations, the numerical accuracy of the results obtained is poorly understood.

Another method for solving system of nonlinear polynomial equations is the homotopy method [M087]. It is also possible to use it on polynomials expressed as unexpanded determinants. In the homotopy method, we start with a known system of equations (whose solutions are known) and march along to compute the solutions of the given system. While marching the number of paths correspond to the Bezout number of the given system of equations. Therefore, this approach becomes unattractive due to the high Bezout numbers of the systems expressed in (7) and (9). Furthermore, the accuracy of the solutions for such high degree equations is not well understood.

In practice, we would expect that the system of equations, (7) and (9), to have very few solutions in the domain of interest. As a consequence, we propose to use Newton's method for equation solving. However, Newton's method can never guarantee all the solutions in the given domain. Two main techniques to improve its performance are the use of interval arithmetic and constrained optimization [PP88;

H089]. [PP88] used this method on low degree curves and it seems to work well for most cases. We are currently working on an implementation of this method for curves expressed in higher and lower dimensional space.

## 6 Conclusion

In this paper, we have presented a novel representation for the surface intersection problem. In particular, the computation of the represent at ion is efficient and numerically stable and it is being used to develop a robust strategy for evaluating the surface intersection. We used results from linear algebra and numerical analysis to come up with efficient and numerically accurate algorithms for curve-surface intersection and computing the inverse image of a point on the surface. In the process we made use of routines from LINPACK and EISPACK for their implementat ion.

As far as the problem of tracing the intersection curve is concerned, we laid stress on the comput at ion of singular points and a start point on each component. We reduced the problem to equation solving and are currently working on applying Newton's method and studying its performance. Many issues concerning the marching method, like the choice of step size and robust evaluation of all branches and components are still open.

The techniques presented in this paper are also useful for ray tracing parametric surface and represent ing offsets and blends of curves and surfaces.

## 7 Acknowledgements

Weare grateful to Prof. J. Demmel and Prof. W. Kahan for productive discussions. This research was supported in part by David and Lucile Packard Fellowship and National Science Foundation Presidential Young Investigator Award (# IRI-8958577).

## 8 References

[BDMS79] Bunch, J., Dongarra, J., Moler, C. and Stewart, G.W. (1979) 'LINPACK user's guide', SIAM, Philadelphia.

[BHLH88] Bajaj, C. L., Hoffmann, C. M., Lynch, R.E. and Hopcroft, J.E. H. (1988) 'Tracing surface intersections', Computer Aided Geometric Design, vol. 5, pp. 285-307.

[BK90] Barnhill, R. E. and Kersey, S. N. (1990) 'A marching method for parametric surface/tiurface intersection', Computer Aided Geometric Design, vol. 7, pp. 257-280.

[BIJ85] Buchberger, B. (1987) 'Gr6brwr bases: An algorithmic method in polynomial ideal theory', in Multidimensional Systems Theory, edited by N.K. Bose, pp. 184-232, D. Reidel Publishing Co.

[Ch90] Chionh, E. W. (1990) 'Base points, resultants, and the implicit representation of rational surfaces', Ph.D. thesis, Department of Computer Science, University of Waterloo, Canada.

[Di08] Dixon, A.L. (1908) 'The eliminant of three quantics in two independent variables', Proceedings of London Mathematical Society, vol. 6, pp. 49-69,473-492.

[Fa86] Farcwki, R.T. (1986) 'The characterization of parametric surface sections', Computer Vision, Graphics and Image Processing, vol. 33, pp. 209-236.

[FN90] Farouki, R.T. and Neff, C.A. (1990) '(Analytic properties of plane offset curves', Computer Aided Geometric Design, vol. 7, pp. 83-99.

[GBDM77] Garbow, B. S., Boyle, J. M., Dongarra, J. and Moler, C. B. (1977) 'Matrix eigensystem routines -EISPACK guide extension', Lecture Notes in Computer Science, vol. 51, Springer-Verlag, Berlin.

[GLR82] Gohberg, l., Lancaster, P. and Rodman, L. (1982) Matrix polynomials, Academic Press, New York.

[GV89] Golub, G.H. and Van Loan, C. F. (1989) Matrix computations, The John Hopkins Press, Baltimore, Maryland.

[H088] Hoffmann, C. (1988) 'A dimensionality paradigm for surface interrogations', Tech. report CSD-TR-837, Department of Computer Science, Purdue University.

[H089] Hoffmann, C. (1989) Geometric and solid modeling: An introduction, Morgan Kaufmann Publishers Inc.

[H090] Hoffmann, C. (1990) 'Algebraic and numeric techniques for offsets and blends', in Computation of Curves and Surfaces, eds. W. Dahmen et, al., pp. 499-529, Kluwer Academic Publishers.

[LR80] Lane, J.M. and Riesemfeld, R.F. (1980) 'A theoretical development for the computer generation and display of piecewise polynomial surfaces', IEEE Trans. Pattern Analysis and Machine Intelligence, vol. 2, no. 1.

[MC27] Morley, F, and Coble, A.B. (1927) 'New results in elimination', American Journal of Mathematics, vol. 49, pp. 463-488.

[MC90a] Manocha, D. and Canny, J. (1990a) 'Trnplicitizing rational parametric surfaces', to appear in Proceedings of IV IMA Conference on Mathematics of Surfaces, University of Bath, England. Also available as Tech. report UCB/CSD 90/592, Computer Science division, University of California, Berkeley.

[MC90b] Manocha, D. and Canny, J. (1990b) 'Multipolynomial resultant algorithms', Proceedings of International Symposium on Intelligent Robotics, pp. 348-358, Bangalore, India.

[Mr87] Morgan, A.P. (1987) Solving polynomial systems using continuation for scientific and engineering problems, Prentice-Hall, Englewood Cliffs, New Jersey.

[PK90] Ponce, J. and Kriegman, D.J. (1990) 'Computing exact aspect graphs of curved objects: parametric surfaces', Tech. report UIUCDCS-R90-1579, Department of Computer Science, University of Illinois at Urbana-Champaign.

[PP88] Prakash, P.V, and Patrikalakis, N,M, (1988) 'Surface-to-surface intersections for geometric modeling', Tech. report MITSG 88-8, MIT Sea Grant College Program, MIT,

[Pr86] Pratt, M. J. (1986) ' Surface/surface intersection problems', in The Mathematics of Surfaces, ed. by J.A. Gregory, pp. 118-142, Claredon Press, Oxford.

[Sa1885] Salmon, G. (1885) Lessons introductory to the modern higher algebra, G.E. Stechert &amp; Co., New York.

[SAG84] Sederberg, T. W., Anderson, D.C. and Goldman, R.N. (1984) 'Implicit representation of parametric curves and surfaces', Computer Vision, Graphics and Image Processing, vol. 28, pp. 72-84.

[SM88] Sederberg, T.W. and Meyers, R.J. (1988) 'Loop detection in surface patch intersections', Computer Aided Geometric Design, vol. 5, pp. 161171.

[SR85] Semple, J.G. and Roth, L. (1985) Introduction to algebraic geometry, Claredon Press, Oxford, Great Britian.

~a50] Walker, Robert J. (1950) Algebraic curves, Princeton University Press, New Jersey.

~d50] van der Waerden B. L. (1950) Modern aL gebra, (third edition) F. Ungar Publishing Co., New York.

[Wi63] Wilkinson, J.H. ( 1963) Rounding errors in algebraic processes, Prentice-Hall, Englewood Cliffs, NJ.

[WM88] Wampler, C.W. and Morgan, A.P. (1988) 'Numerical continuation methods for solving polynomial pystems arising in kinematics', Tech. report GMR-6372, Mathematics Department, General Motors Research Laboratories, Warren, Michigan.