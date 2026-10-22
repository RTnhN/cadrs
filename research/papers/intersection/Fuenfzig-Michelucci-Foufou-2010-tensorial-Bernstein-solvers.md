International Journal of Shape Modeling c

© World Scientific Publishing Company

## Optimizations for Tensorial Bernstein-Based Solvers by using Polyhedral Bounds

## Christoph Fünfzig

Lab Electronics, Imaging and Informatics (UMR CNRS 5158), Université de Bourgogne, Dijon, France christoph.fuenfzig@u-bourgogne.fr

## Dominique Michelucci

Lab Electronics, Imaging and Informatics (UMR CNRS 5158), Université de Bourgogne, Dijon, France dmichel@u-bourgogne.fr

## Sebti Foufou

Lab Electronics, Imaging and Informatics (UMR CNRS 5158), Université de Bourgogne, Dijon, France Computer Science and Engineering, Qatar University, Doha, Qatar sfoufou@u-bourgogne.fr, sfoufou@qu.edu.qa

Received (Day Month Year) Revised (Day Month Year) Accepted (Day Month Year)

Communicated by (xxxxxxxxxx)

The tensorial Bernstein basis for multivariate polynomials in n variables has a number 3 n of functions for degree 2. Consequently, computing the representation of a multivariate polynomial in the tensorial Bernstein basis is an exponential time algorithm, which makes tensorial Bernstein-based solvers impractical for systems with more than n = 6 or 7 variables. This article describes a polytope ( Bernstein polytope ) with a number O ( b ( n , 2 )) of faces, which allows to bound a sparse, multivariate polynomial expressed in the canonical basis by solving several linear programming problems. We compare the performance of a subdivision solver using domain reductions by linear programming with a solver using a change to the tensorial Bernstein basis for domain reduction. The performance is similar for n = 2 variables but only the solver using linear programming on the Bernstein polytope can cope with a large number of variables. We demonstrate this difference with two formulations of the forward kinematics problem of a Gough-Stewart parallel robot: a direct Cartesian formulation and a coordinate-free formulation using Cayley-Menger determinants, followed by a computation of Cartesian coordinates. Furthermore, we present an optimization of the Bernstein polytope-based solver for systems containing only the monomials xi and x 2 i . For these, it is possible to obtain even better domain bounds at no cost using the quadratic curve ( xi , x 2 i ) directly.

Keywords : Bernstein polynomials, algebraic systems, subdivision solver, linear programming, simplex algorithm

1991 Mathematics Subject Classification: 14Q15, 14Q20, 65G40

## 2 C. Fünfzig, D. Michelucci, S. Foufou

## 1. Introduction

Bernstein bases and Bernstein solvers are used in computer graphics and CAD-CAM for computing intersection points between algebraic non-linear curves or surfaces. These solvers reduce domain boxes while retaining the solutions of a given polynomial system, and detect when it does not have solutions. Subdivision of the current domain box can separate different solutions and allows to converge to single solutions. Usually, a simple bisection is used for simplicity. In the overview, the solving process is depicted in the pseudo code of Figure 1.

```
Start with domain D on candidate list C While (the candidate list C is not empty) { Take box b from C and calculate ls=max longest side of b do { perform domain reduction on b if (b does not contain any solution) { exit and continue with next candidate b } else { calculate ls=max longest side of b } } while (b has been sufficiently reduced) if (ls < delta) { b contains potentially a solution } else { bisect b along longest side into b1 and b2, and add them to candidate list C } }
```

Fig. 1. Subdivision solver

For simplicity in this paper, we assume that the system has total degree at most 2. Each algebraic system can be rewritten as a system of total degree 2 by introducing additional variables and equations. In terms of performance, the domain reduction and the test for a solution are of major importance. A classic way for domain reduction is based on the representation of the multivariate polynomial in n variables in the tensorial Bernstein basis (TBB). However, the representation of a multivariate polynomial in the tensorial Bernstein basis (TBB) has exponential length: the basis for degree d has ( d + 1 ) n basis functions. It makes TBB-based solvers impractical for large systems with more than n = 6 or 7 variables, which arise for example in geometric constraint solving.

An alternative for domain reduction uses a polytope enclosing the nonlinear momomials on the domain. This polytope can be described by a polynomial number of halfspaces in the number n of variables. By using linear programming, we can compute a range bound for each multivariate polynomial and a domain bound of all system solutions.

We give a short survey of the representation of polynomials in the tensorial Bernstein basis in Section 2 and introduce our Bernstein polytope in the following Section 3. We collect data of numerical experiments calculating the intersection of 2D algebraic curves in Section 7. Therein, we compare with the domain reduction method proposed in ref. 1 using the tensorial Bernstein basis. This method was selected because it has the best domain reduction performance currently available. In Section 6, we give a possible optimization of the subdivision solver for squared variables x 2 i . It is possible to derive bounds for xi directly from the curve ( xi , x 2 i ) , which are tighter than the ones obtained by the Bernstein polytope alone.

Concerning industrial applications, Section 8 considers the kinematics of a GoughStewart parallel robot in two formulations: a direct Cartesian formulation and a coordinatefree formulation using Cayley-Menger determinants, which results in a system with much less variables. We present the systems and their solution statistics with both solvers. Finally, we give conclusions in Section 9.

## 2. Multivariate Polynomials in the Tensorial Bernstein Basis

The d + 1 Bernstein polynomials B ( d ) i of degree d , also written Bi for fixed d , are a basis for polynomials of degree ≤ d

<!-- formula-not-decoded -->

The conversion from and to the canonical basis ( x 0 , x 1 , . . . x d ) is a linear mapping 2

<!-- formula-not-decoded -->

Every Bernstein polynomial B ( d ) i ( x ) is non-negative for x ∈ [ 0 , 1 ] , and the sum of all Bernstein polynomials equals 1.

These two properties imply that p ( x ) = ∑ piBi ( x ) for x ∈ [ 0 , 1 ] is a linear convex combination of the coefficients pi . With coefficient components pi ∈ R , p ( x ) , x ∈ [ 0 , 1 ] is contained in [ min pi , max pi ] , and this enclosure is tight. For control points pi in 2D or 3D, p ( x ) describes a Bézier curve, and the curve p ( x ) , x ∈ [ 0 , 1 ] lies in the convex hull of its control points pi . For example, since x = 0 d B 0 ( x ) + 1 d B 1 ( x ) + . . . + d d Bd ( x ) , the polynomial curve ( x , y = p ( x )) for x ∈ [ 0 , 1 ] lies in the convex hull of its control points ( i / d , pi ) .

For multivariate polynomials, a Bernstein basis can be constructed using the tensorial product of univariate Bernstein basis functions

In contrast to the canonical basis ( 1 , x , x 2 , . . . x d ) , the control points in the TBB depend on the domain interval. The classical de Casteljau algorithm provides the control points for the curve section p ( x ) , x ∈ [ 0 , t ] , and for the curve section p ( x ) , x ∈ [ t , 1 ] with t ∈ ( 0 , 1 ) .

<!-- formula-not-decoded -->

Then, a multivariate polynomial p of degree ( d 1 , . . . , dn ) is represented as

<!-- formula-not-decoded -->

The de Casteljau method extends to the TBB, and it can be used to subdivide the patch.

## 4 C. Fünfzig, D. Michelucci, S. Foufou

̸

Subdivision solvers for algebraic equations, Bézier curves and surfaces exist in some variations. They all express a multivariate polynomial in the TBB since they require tight enclosures of the polynomial's range 3 , 4 or of the polynomial's roots in the domain 5 , 1 . In the projected polyhedron algorithm 5 , a domain reduction of variable xj is performed by intersecting the convex hull of projected control points ( i j / dj , pi 1 ,..., i j ,..., i n ) i j = 0 ,..., d j with the axis x j . The method in ref. 1 improves the domain reduction by computing the first and last intersection of the two univariate, degreedj Bézier curves defined by control points ( i j / dj , min i k , k = j pi 1 ,..., i j ,..., i n ) i j = 0 ,..., d j and ( i j / dj , max i k , k = j pi 1 ,..., i j ,..., i n ) i j = 0 ,..., d j with the axis x j . The reductions using the range-bounding curves are more effective than the ones using simply the convex hull. For the details of Mourrain/Pavone's method see Section 7.

̸

Note that the TBB contains a number ( 1 + d 1 )( 1 + d 2 ) . . . ( 1 + dn ) of basis functions, where di is the maximum degree of variable xi . Even for linear systems, this number 2 n has exponential growth. The canonical basis in n variables has the same number of functions but polynomial systems given in the canonical basis are sparse. They are not sparse anymore after the change to the TBB. For an illustration, the monomial 1 expressed in the TBB has a length, which is exponential in terms of n : 1 =( B ( d 1 ) 1 ( x 1 ) + . . . + B ( d 1 ) d 1 ( x 1 )) . . . ( B ( dn ) 1 ( xn ) + . . . + B ( dn ) dn ( xn )) . Clearly, the existing Bernstein-based solvers become impractical for a large number n of variables due to the exponential growth of the number of basis coefficients in terms of n . In geometric constraint solving, especially in 3D, systems with a large number of variables occur frequently. For an example, the vertex coordinates of a dodecahedron can be computed by means of geometric insight, i.e., that all its vertices lie on its circumscribed sphere. Alternatively, in a descriptive geometry system the dodecahedron might be described by its simplicial elements: the lengths of its 30 edges connecting the 20 vertices, and the coplanarities of its 12 faces.

## 3. Bernstein Polytope

For univariate polynomials, the inequalities Bi ( x ) ≥ 0 are used to define the halfspaces of a convex polyhedron in R d , which encloses the curve ( x , x 2 , . . . x d ) , x ∈ [ 0 , 1 ] . We call it the Bernstein polytope . For d = 2, the Bernstein polytope is a triangle, see Figure 2. Its delimiting halfspaces are

<!-- formula-not-decoded -->

For d = 3, the Bernstein polytope is a tetrahedron, see Figure 3. Its vertices are v 0 = ( 0 , 0 , 0 ) , v 1 = ( 1 / 3 , 0 , 0 ) , v 2 = ( 2 / 3 , 1 / 3 , 0 ) and v 3 = ( 1 , 1 , 1 ) . For example, v 0 lies on

<!-- image -->

≥

1

≥

Fig. 2. The Bernstein polytope, a triangle, enclosing the curve ( x , y = x 2 ) for x ∈ [ 0 , 1 ] .

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The extension to higher degrees is possible but illustrations are not easily possible anymore.

So far, we have considered curves with a single variable only. Representing a quadratic multivariate polynomial with them requires a separation of variables occurring in the mixed monomials of the form xy . This is possible, for example, using xy = 1 / 2 (( x + y ) 2 -x 2 -y 2 ) .

But we prefer an extension of the Bernstein polytope for the mixed monomial xy as follows. Figure 4 shows the Bernstein polytope, enclosing the surface patch ( x , y , z = xy ) , a tetrahedron. Its halfspaces are defined by

<!-- formula-not-decoded -->

This tetrahedron is optimal: it is the convex hull of the patch.

In summary, the inequalities for multivariate polynomials are obtained as the product of the inequalities for univariate polynomials.

## 6 C. Fünfzig, D. Michelucci, S. Foufou

Fig. 3. The Bernstein polytope, a tetrahedron, enclosing the curve ( x , y = x 2 , z = x 3 ) with x ∈ [ 0 , 1 ] .

<!-- image -->

z

<!-- image -->

≥

Fig. 4. The Bernstein polytope enclosing the surface patch ( x , y , z = xy ) .

## 4. Subdivision Solver

With the polytope of Section 3, linear programming (LP) 6 can be used for the computation of a bound of a polynomial's range and for the reduction of a domain box while preserving roots. In the following sections, we use the simplex algorithm in tableau form for the exposition of the method although in practice the revised simplex algorithm is recommended. We refer the reader to the exposition in ref. 6 for the efficient implementation of the simplex algorithm. In our experiments in Sections 7 and 8, we used the primal-dual revised simplex code SoPlex 1.4.2 7 on a Windows XP 32Bit system (2GB RAM) with an Intel T7200 Core2 Duo processor (2.2GHz).

## 4.1. Range Bounding

To compute a lower and an upper bound of the polynomial p ( x ) = 4 x 2 + x -3 for x ∈ [ 0 , 1 ] , substitute a new variable for the nonlinear monomial: y = x 2 . Then minimize and maximize the linear objective function 4 y + x -3 on the Bernstein polytope enclosing the curve ( x , y = x 2 ) , x ∈ [ 0 , 1 ] (the triangle in Figure 2). After replacing the monomial x 2 with the LP variable y , it is an LP problem.

<!-- formula-not-decoded -->

The simplex algorithm 6 provides the enclosure [ -3 , 2 ] .

<!-- formula-not-decoded -->

Note that the variables on the right side ("not in basis") have value 0.

## 4.2. Domain Reduction

Solving an equation 4 x 2 + x -3 = 0, x ∈ [ 0 , 1 ] is equivalent to finding the intersection points between the line 4 y + x -3 = 0, and the curve ( x , y = x 2 ) . This curve is enclosed in its Bernstein polytope, the triangle of Figure 2. Intersecting the line and the triangle, i.e., finding the minimum and maximum value of x , reduces the domain of variable x . With the same polytope as above, we minimize and maximize the objective function x . The solutions are

## 8 C. Fünfzig, D. Michelucci, S. Foufou

<!-- formula-not-decoded -->

Thus the domain interval [ 0 , 1 ] for variable x has been reduced to [ 3 / 5 , 7 / 9 ] .

For reducing the domain box while preserving roots, 2 n LP problems are solved, a minimization and a maximization for each variable Xk , k = 1 , . . . , n . If the LP problem is not feasible then the domain box contains no root. Note that these 2 n LP problems are independent and can be solved in parallel.

## 4.3. Scaling

After reduction, the domain box is not [ 0 , 1 ] n anymore. There are two possibilities to handle arbitrary domain boxes [ u , v ] ⊂ R n . Either, we scale each variable xi ∈ [ ui , vi ] , ui ≤ vi , to Xi ∈ [ 0 , 1 ] , i = 1 , . . . , n , or we formulate the Bernstein polytope of Section 3 for an arbitrary domain box [ u , v ] ⊂ R n .

The variable scaling from the box [ u , v ] to the unit hypercube [ 0 , 1 ] n is given by

<!-- formula-not-decoded -->

The scaling is a linear map of the LP variables ( Xi , Xii , Xij ) , i , j = 1 , . . . , n , i &lt; j , which has to be performed on the system of polynomial equations.

With a formulation of the Bernstein polytope for an arbitrary domain box, the system of polynomial equations remains unchanged. For an arbitrary domain box [ u , v ] ⊂ R n , the Bernstein polytope is given by

<!-- formula-not-decoded -->

## 5. Floating Point Errors

The Bernstein polytope encloses very tightly the underlying algebraic manifold:

<!-- formula-not-decoded -->

thus with a naive floating-point implementation, some roots are omitted because of rounding errors. For example, when solving x 2 -x = 0 with x ∈ [ 0 , 1 ] , the line y -x = 0 is considered (see Figure 2). If this line becomes y -x = ε due to error ε &gt; 0, the two roots are missed.

There are several strategies to cope with floating point inaccuracy:

- Rational Arithmetic throughout: Use exact computations with rational numbers for scaling and for the LP solver. Then the program becomes slower by several orders of magnitude due to an increasing representation size.
- Rational Arithmetic for LP solver: Use exact computations with rational numbers for the LP solver but work with a fixed precision at the interface to the LP solver. For example, round the box [ 20001 / 10000 , 29999 / 10000 ] to the box [ 200 / 100 , 300 / 100 ] . The rounding ensures rational numbers of bounded representation size at least at the interface of the LP solver, i.e., in the polytope specification.
- Rational Arithmetic for Linear Solver: Compute an approximate solution with an LP solver in floating-point arithmetic. Then start from the approximate LP solution basis with an LP solver in rational arithmetic to compute the exact solution. 8
- Polytope Inflation by the Error Value: During pivoting inside the simplex algorithm, the errors of floating point operations performed in each row are collected and stored with the row's constant. Finally, the hyperplane is pushed outward by this amount. It can happen that the exact feasible set is empty but after the inflation it is no more empty. This approach is conservative, i.e., it guarantees that no root is missed.

## 6. Nonlinear Reduction for Squared Variables

We introduce an important optimization of reduction for squared variables x 2 i , which is not based on the polytope for x 2 i but uses the nonlinear curve ( xi , x 2 i ) directly. Note that most of the complexity of multivariate polynomials comes from the occurrence of mixed monomials though.

It is possible to derive bounds from the nonlinear function ( xi , x 2 i ) directly instead of from its Bernstein polytope, see Figure 5. If min x 2 i and max x 2 i is known, then min xi ≥ √ min x 2 i and max xi ≤ √ max x 2 i . The values min x 2 i and max x 2 i can be obtained by solving two additional LP problems with objective functions min Xii and max Xii respectively. In the special case, where after computing min Xi and max Xi one of the inequalities B 0 ( Xi ) ≥ 0, B 2 ( Xi ) ≥ 0, or B 1 ( Xi ) ≥ 0, occur in the optimal bases of the LP, the range of Xii can be obtained directly from the LP solution vectors X min = ( . . . , X min i , X min ii , . . . ) and X max = ( . . . , X max i , X max ii , . . . ) as:

min Xii = min ( X min ii , X max ii ) and max Xii = max ( X min ii , X max ii ) .

Proof. Let X min i be the minimum value and X max i be the maximum value of variable Xi from the two LP solutions. Then the intersection of the polytope { ( Xi , Xii ) : Xii -2 Xi + 1 ≥ 0 , Xi -Xii ≥ 0 , Xii ≥ 0 } with Xi = X min i or Xi = X max i is either the interval { Xii : Xii -2 Xi + 1 ≥ 0 , Xi -Xii ≥ 0 } , or the interval { Xii : Xii ≥ 0 , Xi -Xii ≥ 0 } , see Figure 5. Due to convexity of the feasible set { Xi , Xii : X is feasible } , the intersection of the two Xii -intervals gives the possible interval for Xii . □

Fig. 5. Bounds obtained from the nonlinear function ( x , y = x 2 ) directly.

<!-- image -->

Consequently for systems containing only linear and squared monomials, the optimization is possible without additional LP calls. If also mixed monomials xi x j , i &lt; j occur, the nonlinear reduction is not possible without the explicit computation of min Xii and max Xii as we demonstrate with examples in Section 7.2. These 2 n supplementary LP problems can be solved in parallel.

## 7. Comparison with a Tensorial Bernstein Based Solver

In this section, we compare the range bounds and the solution domain bounds obtained by our solver based on the Bernstein polytope and the TBB solver in ref. 1 for total degree 2.

The TBB solver computes the coefficients in the tensorial Bernstein basis for a domain box [ u , v ]

<!-- formula-not-decoded -->

The coefficients bi 1 ... i n with respect to the TBB can be computed as described in refs. 1, 9 and 10. The polynomial's range p (( v -u ) x -u ) , x ∈ [ 0 , 1 ] n is then bounded by [ min pi 1 ... i n , max pi 1 ... i n ] . See 1 for a proof of this statement.

In ref. 1, an even stricter bound is derived by projection for each dimension j = 1 , . . . , n

<!-- formula-not-decoded -->

̸

̸

The bounds are quadratic Bézier curves, from which domain bounds for the variables xj can be obtained: Simply intersect the bounding Bézier curves with the axis xj and combine the results from the minimum and the maximum curve correctly. Assume the quadratic system P ( x ) = 0 has an invertible Jacobian matrix P ′ ( m ) at the center m of the domain D . Then it is possible to make the system jacobian the unit matrix at the center m . This preconditioned system P ( x ) P ′ ( m ) -1 = 0 has the same roots as the given system P ( x ) = 0, and it ensures a quadratic convergence 1 for simple roots. In the comparison, we use this preconditioning step with the TBB solver.

Table 1. Comparison of the solvers' performance for the systems in Figure 6. Entries "n.a." denote systems, for which the nonlinear reduction of Section 6 can not be applied without solving additional LP problems.

|                                                                       | Nb. Reduction                                                         | Nb. Bisection                                                         | Avg. Range-Factor                                                       | Avg. Domain-Factor                                                          |
|-----------------------------------------------------------------------|-----------------------------------------------------------------------|-----------------------------------------------------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| Mourrain/Pavone                                                       | Mourrain/Pavone                                                       | Mourrain/Pavone                                                       | Mourrain/Pavone                                                         | Mourrain/Pavone                                                             |
| system (a) system (b) system (c) system (d)                           | 15 53 8 10                                                            | 5 15 1 3                                                              | 3.3896 3.3730 1024.0 2.31344                                            | (4.19672, 1.79867) (1.79106, 1.96632) (1374.09, 95.9520) (1.11638, 2.31344) |
| Our LP method (reduction of all dimensions once)                      | Our LP method (reduction of all dimensions once)                      | Our LP method (reduction of all dimensions once)                      | Our LP method (reduction of all dimensions once)                        | Our LP method (reduction of all dimensions once)                            |
| system (a) system (b) system (c) system (d)                           | 18 26 28 10                                                           | 5 7 13 3                                                              | 13.1280 6.5523 1.9291 12.6969                                           | (1.9967,11.4821) (2.17404, 5.2301) (7.3423e+11, 1.3846) (1.3142, 2.5067)    |
| Our LP method with preconditioning (reduction of all dimensions once) | Our LP method with preconditioning (reduction of all dimensions once) | Our LP method with preconditioning (reduction of all dimensions once) | Our LP method with preconditioning (reduction of all dimensions once)   | Our LP method with preconditioning (reduction of all dimensions once)       |
| system (a) system (b) system (c) system (d)                           | 18 26 28 10                                                           | 5 7 13 3                                                              | 13.1130 6.5248 1.9291 12.2724                                           | (1.99673,11.4821) (2.17404, 5.2301) (7.3423e+11, 1.3846) (1.3142, 2.5067)   |
| Our LP method with square root opt (reduction of all dimensions once) | Our LP method with square root opt (reduction of all dimensions once) | Our LP method with square root opt (reduction of all dimensions once) | Our LP method with square root opt (reduction of all dimensions once)   | Our LP method with square root opt (reduction of all dimensions once)       |
| system (a) system (b) system (c) system (d)                           | 12 n.a. 2 n.a.                                                        | 3 n.a. 0 n.a.                                                         | 11.4926 n.a. 7.2149e+011 n.a. as reduction 17.7660 7.1166 1.9178 8.5097 | (2.3002, 9.1139) n.a. (7.3423e+011, 2.0) n.a.                               |
| Our LP method (reduction of a dimension as long factor >2.0           | Our LP method (reduction of a dimension as long factor >2.0           | Our LP method (reduction of a dimension as long factor >2.0           | Our LP method (reduction of a dimension as long factor >2.0             | Our LP method (reduction of a dimension as long factor >2.0                 |
| system (a) system (b) system (c) system (d)                           | 19 50 25 6                                                            | 1 7 11 1                                                              |                                                                         | (9.2743,13.3139) (2.6270, 4.1077) (3.3640e+12, 1.3939) (1.8468, 1.6842)     |

For our experiments, we use the two-dimensional algebraic curves as shown in Figure 6. Firstly, we measure the reduction factors of range values during the execution of both methods (Section 7.1). Then we perform reduction steps once in each dimension and compute the reduction factors of the interval widths. These empirically determine the convergence behavior of the methods. Section 7.2 gives the comparison.

## 7.1. Range Bounds

Range bounds on system equations are not necessary for solving the system but can be determined with both methods. For linear programming with the Bernstein polytope, it is

## 12 C. Fünfzig, D. Michelucci, S. Foufou

Fig. 6. Algebraic 2D curves: (a) parabola x 1 -x 2 0 = 0 and circle, (b) hyperbola x 0 x 1 = 1 and circle, (c) two touching circles, and (d) hyperbola x 0 x 1 = 4 and circle.

<!-- image -->

described in Section 4.1. With the TBB, range bounds are obtained from the minimum and the maximum TBB coefficient, as described in Section 7. For two variables, there is only a small difference between the ranges computed with the TBB coefficients and the ones computed with linear programming on the same domain interval. Differences in the range reduction factor are caused mainly by different domain intervals.

Table 1 contains the number of reduction steps, the number of bisections, the average reduction factor | range ( D ) | | range ( D ′ ) | of range width | range ( D ) | (column 3), and the average reduction factor | Di | | D ′ i | of domain widths | Di | . Therein, D is the domain and D ′ is the domain after a single reduction step of all variables. Note that both methods were run by doing a bisection immediately after a single reduction step until all domain side lengths are smaller than δ = 10 -3 . The process started with initial domain box [ -10 , 10 ] 2 . System (a) has two single roots, (b) has four single roots, (c) has a double root, and (d) has no root. From the measurements, it is remarkable that the TBB solver generates roughly the same range reductions (factor 3.3) in normal situations, and achieves a factor 1024.0 for the tangential contact of system (c). For linear programming on the Bernstein polytope, the number of reductions and bisections are similar, whereas the range reduction factors are more varied. An exception is system (c) with tangential contact, where our method requires some more reductions and bisections.

For a comparison, we applied the same preconditioning step, described in Section 7, to the system solved by the simplex algorithm with the Bernstein polytope. The effect of preconditioning on the revised simplex algorithm, which uses linear systems solving internally, is very small as shown in Table 1. The revised simplex algorithm is in this respect more similar to linear systems solving and its techniques to improve accuracy (pivoting, equilibration 11 ).

Table 1 also gives the statistics for the simplex algorithm with the Bernstein polytope using the nonlinear reductions of Section 6, where applicable without solving additional LP problems, i.e., for systems (a) and (c). The nonlinear reductions are very effective and largely reduce the domain widths.

## 7.2. Convergence

For the domain reduction steps, Table 1 contains the domain reduction factors in column 4. It can be seen that linear programming with the Bernstein polytope for variable xi ′ profits from reductions of variable xi , i &lt; i ′ . Usually, the reduction factor of the second variable x 2 is considerably greater than the one for the first variable x 1. The TBB solver can not take advantage similarly as the conversion to the tensorial Bernstein basis is done once before reducing all the variables.

In the paper 1, the quadratic convergence of the method in the case of a single root was proved. For the double root of system (c), the reduction is very good due to the preconditioning step. The revised simplex algorithm with the Bernstein polytope achieves similar reduction factors as the TBB solver in cases with single roots as can be seen in Table 1, systems (a), (b), (d). Also in multiple root cases, it has an average reduction factor f &gt; 1, and is thus at least linearly convergent.

## 8. Kinematics of Gough-Stewart Platform

Gough-Stewart platforms are used as parallel robots. We consider here the structure made of two triangles connected by jacks (translational joints) into an octahedron 12 , 13 . See Figure 8 for an illustration.

Fig. 7. (a) Several solutions for the given jack lengths above the base platform. Among them are two solutions that have a side change of the jack direction relative to the upper platform. (b) Solution without a side change of the jack directions.

<!-- image -->

The lower triangle serves as the base, and the upper triangle moves as the work platform. Edges of the platform and of the base are rigid, i.e., their lengths are fixed once (brown triangles in Figure 8): p 2 p 3, p 3 p 1, p 1 p 2, p 6 p 4, p 4 p 5, p 5 p 6. The lengths of the re- maining edges are computer-controlled (gray lines in Figure 8): p 1 p 4, p 1 p 6, p 2 p 4, p 2 p 5, p 3 p 5, p 3 p 6.

In the following section, we consider in detail the forward kinematics problem , where the six side lengths d 23, d 31, d 12, d 64, d 45, d 56, and the six lengths of the jacks d 14, d 16, d 24, d 25, d 35, d 36 are given, and the coordinates of the upper platform's points (three points of the octahedron) are to be computed. As usual for a parallel structure, the inverse kinematics problem is much easier, as the lengths of the six jacks can be computed directly from the points' coordinates.

## 8.1. Forward Kinematics

As shown in ref. 14, the forward kinematics problem can have up to 16 solutions.

The problem can be formulated directly in Cartesian coordinates if we fix the coordinates of the lower platform's points. Here is an example.

fixed lower triangle

<!-- formula-not-decoded -->

given distances of the upper triangle's points

<!-- formula-not-decoded -->

Interesting possibilities exist for selecting solutions. For example, due to mechanical restrictions the jacks can not change side with respect to the work platform. Let n work = ( p 5 -p 4 ) × ( p 6 -p 4 ) be the normal of the work platform, where v 1 × v 2 denotes the cross product of two vectors. Then the scalar products n · ( p 1 -p 6 ) ≤ 0, n · ( p 1 -p 5 ) ≤ 0, n · ( p 2 -p 6 ) ≤ 0, n · ( p 2 -p 4 ) ≤ 0, n · ( p 3 -p 5 ) ≤ 0, n · ( p 3 -p 4 ) ≤ 0 formulate this condition. We have to add three variables n 1, n 2, n 3 to the system for the normal direction. See Figure 7 for an example solution using this condition.

Other conditions can be formulated, i.e., the work platform should be upright with respect to the base platform ( n base · n work ≥ 0). Note that this condition is not already implied by the condition that the jacks do not change side with respect to the work platform.

Fig. 8. (a) Gough-Stewart platform. (b) Gough-Stewart platform with two distances (red lines) used in the CayleyMenger formulation.

<!-- image -->

Fig. 9. Solving Cayley-Menger equations for D 14 and D 25 . Small black boxes mark the solution points ( D 14 , D 25 ) : (a) 2 solutions, (b) 3 solutions.

<!-- image -->

With Cayley-Menger determinants, it is possible to give a coordinate-free formulation. The Cayley-Menger determinant gives a relation of all the squared distances between five points in 3-space. 15 For points p 1, p 2, p 3, p 4, p 5, the Cayley-Menger determinant contains the squared distances D 12 = d 2 12 , D 13 = d 2 13 , D 14 = d 2 14 , D 15 = d 2 15 , D 23 = d 2 23 , D 24 = d 2 24 , D 25 = d 2 25 , D 34 = d 2 34 , D 35 = d 2 35 , D 45 = d 2 45 . All these squared distances are known, except for D 14 and D 25 (red lines in Figure 8), and the equation has degree 4 in the 2 unknowns.

Table 2. Comparison of the solvers' performance for the Gough-Stewart forward kinematics. Entries "?" denote computations that did not terminate within one minute.

|                                | Nb. Solutions   | Nb. Reduction   | Nb. Bisection   | Time    |
|--------------------------------|-----------------|-----------------|-----------------|---------|
| Mourrain/Pavone                |                 |                 |                 |         |
| Cartesian n=9                  | ?               | ?               | ?               | >60.0s  |
| Cartesian n=12, no side change | ?               | ?               | ?               | >60.0s  |
| Cayley-Menger n=3              | 2               | 1153            | 103             | 0.0157s |
| Cayley-Menger n=3              | 3               | 4235            | 397             | 0.0549s |
| Our LP method                  |                 |                 |                 |         |
| Cartesian n=9,                 | 3               | 393             | 47              | 16.555s |
| Cartesian n=12, no side change | 1               | 365             | 43              | 17.946s |
| Cayley-Menger n=3              | 2               | 143             | 33              | 0.0680s |
| Cayley-Menger n=3              | 3               | 397             | 92              | 0.1783s |

<!-- formula-not-decoded -->

A second, independent equation can be generated for the points p 1, p 2, p 4, p 5, p 6. We can reformulate this system of degree 4 into a system of degree 2 by introducing an auxiliary variable D 14 , 25 for the product D 14 , 25 = D 14 D 25. Then the monomials D 2 14 D 2 25 , D 2 14 D 25, and D 14 D 2 25 become degree 2: D 2 14 , 25 , D 14 D 14 , 25, and D 14 , 25 D 25.

If the lengths d 14 and d 25 of two diagonals are known, we can compute the Cartesian coordinates of the three points on the upper triangle. For each point p 4 and p 5, the distances to points p 1, p 2, p 3 are known so that their coordinates can be computed as the intersection of three spheres. The coordinates of point p 6 can then be computed in the same way from the distances to points p 1, p 4, p 5. As an example, the intersection of the three spheres

( p 1 , r 1 = d 14 ) , ( p 2 , r 2 = d 24 ) , ( p 3 , r 3 = d 34 ) results in two possible solutions p 4, p ∗ 4 .

<!-- formula-not-decoded -->

Here, the vector u is from point p 1 to the intersection point in the plane p 1 p 2 p 3 of the two intersection circle planes (defined by the sphere intersections of ( p 1 , r 1 ) , ( p 2 , r 2 ) and of ( p 1 , r 1 ) , ( p 3 , r 3 ) ). The vector v points from this intersection point along c , which is orthogonal to the plane p 1 p 2 p 3, to an intersection point of the three circles. As the three points p 1, p 1 + u , p 4 form a right-angled triangle with two known side lengths | p 1 -p 4 | = d 14 and | p 1 + u -p 1 | = | u | , the third length | p 1 + u -p 4 | needs to be √ d 2 14 -| u | 2 by a theorem of Pythagoras.

Note that the solutions for p 4 and p 5 have to be combined according to the given distance d 45, which can have up to two solutions for each solution p 4. Figure 10 shows a geometric configuration in 2D, where a solution for p 4 connects to the two possibilities for p 5. Each one leaves two possibilities for point p 6. In total, it can have up to 4 |{ ( d 14 , d 25 ) : d ( M ( D 12 , D 13 , D 14 , D 15 , D 23 , D 24 , D 25 , D 34 , D 35 , D 45 )) = 0 , d ( M ( D 12 , D 16 , D 14 , D 15 , D 26 , D 24 , D 25 , D 46 , D 56 , D 45 )) = 0 }| solutions, where one half of them lies above the base platform and the other half lies below it due to symmetry.

Fig. 10. A solution for point p 4 connects to up to two possibilities for point p 5 .

<!-- image -->

The runtime for example systems is summarized in Table 2. A reduction step of all variables was immediately followed by a bisection of the domain box until all domain side lengths were smaller than δ = 10 -4 . For the initial domain box, [ -5 , 5 ] resp. [ 0 , 5 ] was used for the Cartesian coordinate components, and [ 0 , 5 ] for squared distances in the CayleyMenger formulation. Only the LP reduction method can be used for the direct Cartesian formulation due to its large number of variables. For n = 9, the TBB has 3 9 = 19683 basis functions, whose coefficients have to be calculated in each reduction step. For n = 12, there are already 3 12 = 531441 coefficients. The additional equations for selecting solutions (avoiding side changes of the jacks, upright orientation of the upper platform) have no significant effect on the runtime.

The Cayley-Menger formulation requires the solution of a small system only (2 equations of degree 4 in 2 variables, 3 equations of degree 2 in 3 variables), which is coordinatefree, and three calculations of the intersection of three spheres, which is coordinatedependent but can be solved by a specialized code. The solution of the coordinate-free part is possible with both solvers. For n = 3 variables, the TBB solver is faster despite a larger number of reductions and bisections. A reduction step involves only n = 3 min/max searches over 3 3 = 27 coefficients.

## 9. Conclusion

This article has compared two subdivision solvers for quadratic polynomial systems, which use the same algorithm only differing in the domain reduction step. The first solver in ref. 1 uses the tensorial Bernstein basis (TBB) representation of a multivariate polynomial. It uses the tensorial Bernstein basis to derive a pair of range-bounding Bézier curves for each variable x j of the system. Based on these Bézier curves, the solver reduces the domain of each variable x j by univariate root finding, which is beneficial especially in cases of multiple roots. 1 In contrast, our LP solver derives bounds of the solution domain from a polytope enclosure of the nonlinear patches over the domain. They are computed as solutions of linear programming problems, i.e., using the revised simplex algorithm. The polytope is defined by a number of halfspaces, which has polynomial length in terms of the input system, i.e., the number n of variables, and fixed total degree d .

In practice, problem formulations can easily incorporate a large number n of variables. In this case, solvers using the tensorial Bernstein basis suffer from the exponential number ( d + 1 ) n of basis functions for total degree d . With the LP-based method, the polytope is defined by a number of O ( n d ) halfspaces. For the Gough-Stewart platform, an octahedron, we use it to compute all solutions or specially selected solutions for the forward kinematics problem. The forward kinematics problem in Cartesian coordinates incorporates 9-12 variables and a similar number of (in-)equalities. This system size is currently intractable for a TBB solver. It can only be solved as a decomposed system, for example, using a coordinate-free formulation with Cayley-Menger determinants, which has only 2-3 variables.

The convergence behavior of both methods has been compared empirically for systems of two variables. At double roots, Mourrain's method benefits from preconditioning, which achieves a nearly unit jacobian matrix at the center of the domain. This preconditioning of the TBB solver has only a small effect with the revised simplex algorithm. The simplex algorithm is in this respect more similar to linear systems solving and its techniques to improve accuracy. In the comparison for single roots, numerical evidence confirms the quadratic convergence of both methods.

An optimization of the LP reduction-based solver is possible for systems containing squared variables x 2 i . In this case, we have demonstrated a nonlinear reduction of the domain interval for xi based on the quadratic curve ( xi , x 2 i ) . The nonlinear reduction is directly applicable for systems containing only the monomials xi and x 2 i . For arbitrary systems, it requires the solution of two additional LP problems min Xii and max Xii corresponding to x 2 i .

## Acknowledgements

Christoph Fünfzig would like to thank the Conseil Régional de Bourgogne (Regional Council of Burgundy) for support in a postdoc scholarship.

## References

1. MOURRAIN B., AND PAVONE J.-P. Subdivision methods for solving polynomial equations. Journal of Symbolic Computation 44(3) , pp. 292-306, March 2009.
2. FARIN G. Curves and Surfaces for CAGD: A Practical Guide. Academic Press Professional, Inc , San Diego, CA, 1988.
3. GARLOFF J., AND SMITH A.P. Investigation of a subdivision based algorithm for solving systems of polynomial equations. Journal of Nonlinear Analysis : Series A Theory and Methods 47 , 1 (2001), pp. 167-178.
4. ELBER G., AND KIM M.-S. Geometric constraint solver using multivariate rational spline functions. In SMA'01: Proc. of the 6th ACM Symp. on Solid Modeling and Applications (New York, NY, USA, 2001), ACM Press, pp. 1-10.
5. SHERBROOKE E.C., AND PATRIKALAKIS N.M. Computation of the solutions of nonlinear polynomial systems. Computer Aided Geometric Design 5(10) , pp. 379-405, 1993.
6. CHVATAL, V. Linear Programming. Series of Books in the Mathematical Sciences, W.H. Freeman , September 1983.
7. WUNDERLING R. SoPlex - The sequential object-oriented simplex. Konrad-Zuse-Zentrum für Informationstechnik Berlin, Department Optimization , http://soplex.zib.de/, Version 1.4.2, 2009.
8. DHIFLAOUI M., FUNKE S., KWAPPIK C., MEHLHORN K., SEEL M., SCHÖMER E., SCHULTE R., AND WEBER D. Certifying and Repairing Solutions to Large LPs, How Good are LPsolvers? In SODA , pp. 255-256, 2003.
9. LIN Q., AND ROKNE J.G. Methods for bounding the range of a polynomial. Journal of Computational and Applied Mathematics 58 , pp. 193-199, 1995.
10. LIN Q., AND ROKNE J.G. Interval approximation of higher order to the ranges of functions. Computers &amp; Mathematics with Applications 31(7) , pp. 101-109, 1996.
11. GOLUB G.H., AND VAN LOAN C.F. Matrix Computations. Johns Hopkins Studies in Mathematical Sciences , 3rd edition, Baltimore, Maryland, 1996.
12. GOUGH V.E. Contribution to discussion of papers on research in Automobile Stability, Control and Tyre performance. Proc. Auto Div. Inst. Mech. Eng. , pp. 392-394, 1956-1957.
13. MERLET J.P., Solving the Forward Kinematics of a Gough-Type Parallel Manipulator with Interval Analysis. Int. Journal Robotic Research 23(3) , pp. 221-235, 2004.

- 20 C. Fünfzig, D. Michelucci, S. Foufou
14. CHARENTUS S., DIAZ C., AND RENAUD M. Modular serial parallel redundant robot. In IMACS , Cetraro, Italy, September 18-21, 1988.
15. MICHELUCCI D., AND FOUFOU S., Using Cayley-Menger determinants for geometric constraint solving. SM '04: Proceedings of the ninth ACM Symposium on Solid Modeling and Applications , pp. 285-290, Genoa, Italy, 2004.