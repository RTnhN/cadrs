## 13.472J/1.128J/2.158J/16.940J COMPUTATIONAL GEOMETRY

## Lecture 13

## N. M. Patrikalakis

Massachusetts Institute of Technology

Cambridge, MA 02139-4307, USA

Copyright c © 2003 Massachusetts Institute of Technology

## Contents

| 13 Offsets of Parametric Curves and Surfaces   | 13 Offsets of Parametric Curves and Surfaces                         | 13 Offsets of Parametric Curves and Surfaces                           |   2 |
|------------------------------------------------|----------------------------------------------------------------------|------------------------------------------------------------------------|-----|
| 13.1                                           | Motivation . . . . . . . . . . . . . . . . . . . . . . . . . . . . . | . . . . .                                                              |   2 |
| 13.2                                           | Parametric offset curves . . . . . . . . . . . . . . . . .           | . . . . . . . . . .                                                    |   5 |
|                                                | 13.2.1                                                               | Differential geometry of parametric offset curves . . . . . . . . .    |   5 |
|                                                | 13.2.2                                                               | Singularities of parametric offset curves . . . . . . . . . . . . . .  |   6 |
|                                                | 13.2.3                                                               | Approximations . . . . . . . . . . . . . . . . . . . . . . . . . . . . |   9 |
| 13.3                                           | Parametric offset surfaces . .                                       | . . . . . . . . . . . . . . . . . . . . . . . .                        |  10 |
|                                                | 13.3.1                                                               | Differential geometry of parametric offset surfaces . . . . . . . .    |  10 |
|                                                | 13.3.2                                                               | Singularities of parametric offset surfaces . . . . . . . . . . . . .  |  11 |
|                                                | 13.3.3                                                               | Tracing algorithm . . . . . . . . . . . . . . . . . . . . . . . . . .  |  13 |
|                                                | 13.3.4                                                               | Self-intersections of offsets of explicit quadratic surfaces . . . . . |  14 |
|                                                | 13.3.5                                                               | Approximations . . . . . . . . . . . . . . . . . . . . . . . . . . . . |  21 |
| Bibliography                                   | Bibliography                                                         | Bibliography                                                           |  22 |

Reading in the Textbook

- Chapter 11, pp. 293 - 353

## Lecture 13

## Offsets of Parametric Curves and Surfaces

## 13.1 Motivation

Offsets are defined as the locus of points at a signed distance d along the normal of a planar curve or surface. A literature survey on offset curves and surfaces up to 1992 was carried out by Pham [24], while the overview of the literature after 1992 and those which were not cited in [24] is given by Maekawa [14]. Offset curves and surfaces are widely used in various engineering applications, such as

- Tool path generation for pocket(2.5D), 3D and 5D NC machining [9, 1]. (See Figure 13.1).

Figure 13.1: NC machining.

<!-- image -->

- Definition of tolerance regions [4, 26, 21]. (See Figure 13.2).
- Access space representations in robotics [12]. (See Figure 13.3) a0✂a0✁a0✁a0✁a0✂a0

Figure 13.2: Definition of tolerance regions.

<!-- image -->

a0✂a0✁a0✁a0✁a0✂a0

a0✁a0✁a0✁a0✂a0

Figure 13.3: Access space representations in robotics.

<!-- image -->

- Curved plate (shell) representation in solid modeling [23]. (See Figure 13.4)
- Feature recognition through construction of skeletons or medial axes of geometric models [22, 29]. (See Figure 13.5). The medial axis is made up of boundary offset intersections.

Figure 13.4: Plate representation.

<!-- image -->

Figure 13.5: Medial Axis.

<!-- image -->

The concept of offset curves generalizes to

- pipe surfaces when the progenitor is a general 3 D curve [18].
- geodesic offsets when the progenitor is curve on a surface [20] [25] [11].

## 13.2 Parametric offset curves

## 13.2.1 Differential geometry of parametric offset curves

- A planar parametric curve r ( t ) is given by

<!-- formula-not-decoded -->

where x and y are differentiable functions of a parameter t .

- The unit normal vector of a plane curve, which is orthogonal to t , is given by

<!-- formula-not-decoded -->

where e z = (0 , 0 , 1) is a unit vector perpendicular to the plane of the curve, see Figure 13.6.

- For a plane curve, the Frenet formulae reduce to

<!-- formula-not-decoded -->

where κ is the signed curvature of the curve given by

<!-- formula-not-decoded -->

where v = | ˙ r ( t ) | is the parametric speed. The curvature κ of a curve at point P is positive when the direction of n and ⃗ PC are opposite where C is the center of the curvature of the curve at point P , see Figure 13.6.

Figure 13.6: Definitions of unit tangent and normal vectors.

<!-- image -->

- A planar offset curve ˆ r ( t ) with signed offset distance d to the progenitor r ( t ) is defined by

<!-- formula-not-decoded -->

where d &gt; 0 corresponds to positive ('exterior') and d &lt; 0 corresponds to negative ('interior') offsets.

- The unit tangent vector of the offset curve (see Figure 13.7 for illustration)

<!-- formula-not-decoded -->

- The unit normal vector of the offset curve (see Figure 13.7 for illustration)

<!-- formula-not-decoded -->

- Curvature of the offset curve

<!-- formula-not-decoded -->

## 13.2.2 Singularities of parametric offset curves

There are two kinds of singularities on the offset curves, irregular points and self-intersections .

## · Irregular points

Isolated points : This point occurs when the progenitor curve with radius R is a circle and the offset is d = -R .

Cusps : This point occurs at a point t where the tangent vector vanishes.

<!-- formula-not-decoded -->

A cusp at t = t c can be further subdivided into [7]:

1. Ordinary cusps when ˙ κ ( t c ) = 0
2. Extraordinary points when ˙ κ ( t c ) = 0 and ¨ κ ( t c ) = 0.

̸

̸

Note that (1 + κd ) / | 1 + κd | in equations (13.6) and (13.7) changes abruptly from -1 to 1 when the parameter t passes through t = t c at an ordinary cusp, while at extraordinary points (1 + κd ) / | 1 + κd | does not change its value, see Figure 13.7.

Equation (13.9) for r ( t ) = { x ( t ) , y ( t ) } can be reduced to

<!-- formula-not-decoded -->

Figure 13.7: Offsets to a parabola r = [ t, t 2 ] (thick solid line) with offsets d =-0.3, -0.5, -0.8, adapted from [5]. At d = -0 . 3 the tangent and normal vectors of the offset have the same sense that of the progenitor, while at d = -0 . 8 they flip directions.

<!-- image -->

By setting τ 2 = ˙ x 2 + ˙ y 2 and if r ( t ) is a rational polynomial curve, the computation of cusps can be reduced to system of two nonlinear polynomial equations that can be solved using the methods of Chapter 10.

Examples (see Figures 13.7 and 13.8)

Given a parabola r = ( t, t 2 ), the unit tangent and principal normal vectors are given by

<!-- formula-not-decoded -->

The curvature and its derivative are given by

<!-- formula-not-decoded -->

Since ˙ κ (0) = 0, κ ( t ) reaches an extremum at t = 0 and furthermore as ¨ κ (0) &lt; 0, κ (0) is a maximum with a curvature value κ (0) = 2. Therefore when d &gt; -1 2 the offset is nondegenerate, while when d = -1 2 , t = 0 is an extraordinary point. Let us solve κ ( t ) = -1 /d for t which yields

<!-- formula-not-decoded -->

We can easily see that if d &gt; -1 / 2, there is no real root. This means that there is no singularity as long as radius of curvature is smaller than 2. If d = -1 / 2, there exists a single root t = 0, while if d &lt; -1 / 2 there exist two symmetric values t 1 , t 2 .

## · Self-intersections

Self-intersections of an offset curve (see also Figures 13.7 and 13.8) can be obtained by seeking pairs of distinct parameter values s = t such that

̸

<!-- formula-not-decoded -->

Substitution of equation (13.2) in (13.11) yields the system [17]

<!-- formula-not-decoded -->

If r ( t ) is a rational polynomial curve, this system can be converted to a nonlinear polynomial system of four equations in four variables s , t , τ and σ where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Such a system can be solved using the IPP algorithm, see also [17]. However s = t are trivial solutions, and we must exclude them from the system, otherwise a Bernstein subdivision-based algorithm would attempt to solve for an infinite number of roots. In this case we have addressed the problem by dividing out the common factor by some algebraic manipulations [17].

Figure 13.8: Self-intersection of the offset curve of a parabola. Left: Interior offsets to the parabola r ( t ) = [ t, t 2 ] with d = -0 . 8 and cutter path; Right: Trimmed interior offsets to the parabola r ( t ) = [ t, t 2 ] with d = -0 . 8 and cutter path

<!-- image -->

## 13.2.3 Approximations

- In general, an offset curve is functionally more complex than its progenitor curve because of the square root involved in the expression of the unit normal vector. L¨ u [13] for example has shown that offset of a parabola is a rational curve and its singular point at infinity was studied by Farouki and Sederberg [8]. However, this result has not been generalized to higher order curves.

Farouki and Neff [6] have shown that the two-sided offsets of planar rational polynomial curves are high-degree implicit algebraic curves f o ( x, y ) = 0 of potentially complex shape. These equations can not typically be separated into two equations describing interior and exterior offsets individually. The degree of this implicit offset curve is n o = 4 n -2 -2 m , where n is the degree of polynomial generator curve r = [ x ( t ) , y ( t )] and m is the degree of φ ( t ) = GCD ( x ′ ( t ) , y ′ ( t )). For example the degree of the two-sided offset curve of a parabola r ( t ) = ( t, t 2 ) is 6 and of a general polynomial cubic curve is 10 with φ ( t ) a constant.

- If the progenitor surface is a NURBS curve, then its offset is usually not a NURBS curve, except for straight lines and circles.
- Because of the wide application of offset surfaces and the difficulty in directly incorporating such entities in geometric modeling systems, due to their potential analytic and algebraic complexity, a number of researchers have developed approximation algorithms for these types of geometries in terms of piecewise polynomial or rational polynomial functions [27, 10].
- Summary of an Approximation Algorithm [27], see also Figure 13.9:
1. Input is a NURBS curve.
2. Offset each leg of polygon by d .
3. Intersect consecutive legs of polygon to find new vertices.
4. Check deviation of the approximate offset with the true offset using as weights (for rational function) the weights of the progenitor curve.
5. If the deviation is larger than the given tolerance subdivide the curve into two and go back to step 1. If the deviation is smaller than the given tolerance stop.

Figure 13.9: Offset curve approximation.

<!-- image -->

## 13.3 Parametric offset surfaces

## 13.3.1 Differential geometry of parametric offset surfaces

## · Definition

A parametric offset surface ˆ r ( u, v ) is a continuum of all points at a constant distance d along normal to another parametric surface r ( u, v ) and defined as

<!-- formula-not-decoded -->

where d may be a positive or negative real number and n is the unit normal vector of r ( u, v ).

## · Sign convention for normal curvature

The normal curvature is typically considered positive if its associated center of curvature is opposite to the direction of the surface normal.

## · Relation between n and ˆ n [28]

If ˆ n ( u, v ) is the unit normal vector of ˆ r ( u, v ), then the relation between n and ˆ n is given by

<!-- formula-not-decoded -->

where ˆ S = | ˆ r u × ˆ r v | and S = | r u × r v | or expanding the right hand side of equation (13.16) and using the definitions of Gaussian curvature K and mean curvature H

<!-- formula-not-decoded -->

equation (13.16) can be rewritten as follows:

<!-- formula-not-decoded -->

If we take the norm of equation (13.16), we obtain

<!-- formula-not-decoded -->

and substituting ˆ S into equation (13.16) yields

<!-- formula-not-decoded -->

From this relation n and ˆ n are collinear but may be directed in opposite directions, if dκ max &lt; -1 or dκ min &lt; -1. This occurs when the offset is taken towards the concave region of the progenitor.

- Offsetting towards concave region of a surface is equivalent to taking the offset d &gt; 0 where κ min &lt; 0 and d &lt; 0 where κ max &gt; 0, provided the above sign convention is used.

## · Gaussian and Mean Curvatures

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Given an offset distance d , the critical curvature is defined as κ crit = -1 /d then three categories arise [5]:

κ max &gt; κ min &gt; κ crit : The normal vector of the progenitor and its offset are directed in the same direction. Also the sign of Gaussian and principal curvatures of the offset are the same that of the progenitor.

κ max &gt; κ crit &gt; κ min : The normal vector of the progenitor and its offset are directed in the opposite direction. Also the sign of Gaussian and maximum principal curvatures of the offset are opposite to that of the progenitor, while the sign of the minimum principal curvature of the offset is the same to that of the progenitor.

κ min &lt; κ max &lt; κ crit : The normal vector of the progenitor and its offset are directed in the same direction, while the sign of both principal curvatures of the offset are opposite that of the progenitor and thus the sign of Gaussian curvature of an offset remains the same as that of the progenitor.

## 13.3.2 Singularities of parametric offset surfaces

Figure 13.10: Offset surface (left), region bounded by self-intersection curve (center) and trimmed offset surface (right) of elliptic paraboloid z = 1 2 (1 . 75 x 2 +2 y 2 ) with d = 0 . 6.

<!-- image -->

- In NC machining , the cutter radius must not exceed the smallest concave principal radius of curvature of the surface to avoid gouging [9].

## · Principal Curvatures

Figure 13.11: Self-intersection curves of elliptic paraboloid ( α = 2, β = 4) with d = 0 . 3. The dot dashed line in the figure is a set of points of self-intersection curve in the xy -plane mapped onto the progenitor surface. A pair of thin solid straight lines emanating from two distinct points on the surface r ( s, t ), r ( u, v ) and intersecting along the parabola are the pairs of vectors d n ( s, t ) and d n ( u, v ).

<!-- image -->

## · Critical Offset Distance :

The largest magnitude of offset distance without degeneracy is called critical offset distance d crit . When the offset is positive, in the absence of degeneracy due to global properties, the maximum absolute value of the negative minimum principal curvature on the surface determines d crit = 1 max ( | κ min | ) . When the offset is negative, in the absence of degeneracy due to global properties, the minimum absolute value of the positive maximum principal curvature on the surface determines d crit = 1 max ( κ max ) . versa.

## · Ridges :

It is apparent from equation (13.20) that offset surfaces become singular at points called ridges . They are defined as a vector-valued mapping of two implicit curves in the uv -parametric space to 3 D space via the mapping (13.15), which satisfy κ max ( u, v ) = -1 d or κ min ( u, v ) = -1 d [9].

## · Self-intersections :

Self-intersections of an offset surface are defined by finding pairs of distinct parameter values ( s, t ) = ( u, v ) such that

<!-- formula-not-decoded -->

see also Figures 13.10, 13.11.

For parametric surfaces r ( u, v ) = [ x ( u, v ) , y ( u, v ) , z ( u, v )]

<!-- formula-not-decoded -->

̸

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Since we can fix one of the four variables ( s , t , u , v ), the system of equations (13.26) to (13.28) yields three equations with three unknowns.

We can replace S ( s, t ) and S ( u, v ) by auxiliary variables σ and ω such that σ 2 = S 2 ( s, t ) and ω 2 = S 2 ( u, v ).

Consequently the system involving polynomials and square root of polynomials has been reduced to a nonlinear polynomial system consisting of five equations with five unknowns as follows:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Since s = u , t = v are trivial solutions, we must exclude them from the system, otherwise a Bernstein subdivision-based algorithm would attempt to solve for an infinite number of roots. For the self-intersections of a normal offset of a planar polynomial curve case we have addressed this problem by dividing out the common factor by some algebraic manipulations [17]. However, for the surface case we can not divide out these factors from the system directly, since terms x ( s, t ) -x ( u, v ), y ( s, t ) -y ( u, v ) and z ( s, t ) -z ( u, v ) do not necessarily exactly involve the factors s -u and t -v . See [16] for details for how to exclude trivial solutions.

## 13.3.3 Tracing algorithm

Finding the starting points for tracing the self-intersection curve is very similar to the same problem for surface-surface intersection in Section 9.8.2. By considering that the self-intersection curve is a function of another parameter τ , s = s ( τ ), t = t ( τ ), u = u ( τ ), v = v ( τ ), and by differentiating the equation for self-intersection curves of an offset with respect to τ yields

<!-- formula-not-decoded -->

where If we denote ˆ r ( s, t ) = [ˆ x ( s, t ) , ˆ y ( s, t ) , ˆ z ( s, t )] and ˆ r ( u, v ) = [ˆ x ( u, v ) , ˆ y ( u, v ) , ˆ z ( u, v )], we can solve vector equation (13.40) for ds dτ , dt dτ , du dτ and dv dτ . This is a linear system of 3 equations in 4 unknowns ˙ s , ˙ t , ˙ u , ˙ v . The solution of this underconstrained problem is given by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

∣ ∣ Here ζ is an arbitrary non-zero factor that can be chosen to provide arc-length parametrization in the parameter domain as follows:

hence

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The points of the self-intersection curves are computed successively by integrating the initial value problem for a system of nonlinear differential equations (13.41) to (13.44) using the variable step size and variable order Adams method [2]. The sign of ζ determines the direction in which the solution proceeds. See Figures 13.12 and 13.13 for illustrations.

## 13.3.4 Self-intersections of offsets of explicit quadratic surfaces

Although offset surfaces are widely used in various engineering applications, their degenerating mechanism is not well known in a quantitative manner. We know that any regular surface can be locally approximated in the neighborhood of a point p by the explicit quadratic surface of the form r ( x, y ) = [ x, y, 1 2 ( αx 2 + βy 2 )] T to the second order where -α and -β are the principal curvatures at point p . Therefore investigations of the self-intersecting mechanisms of the offsets of explicit quadratic surfaces due to differential geometry properties lead to an understanding of the self-intersecting mechanisms of offsets of regular parametric surfaces.

- Locally any surface can be expressed as a graph of a differentiable function [3]. Given a point p on the parametric surface S , we can set an orthogonal Cartesian coordinate system xyz such that xy -plane coincides with the tangent plane of S at p and z -axis is along the normal at p . It follows that in the neighborhood of p any parametric surface S can be represented in the form r ( x, y ) = [ x, y, h ( x, y )] T , where h is a differentiable function with h (0 , 0) = h x (0 , 0) = h y (0 , 0) = 0.

Figure 13.12: Self-intersection curves of the offset of bicubic patch when d =0.09. Figure ( a ) shows the pre-images of the self-intersection curves in parameter domain. The same symbols are mapped to the same points in the offset surface. Figure ( b ) shows the mapping of the self-intersection curves in the parameter domain onto the progenitor surface. Figure ( c ) shows the offset surface and the self-intersection curves.

<!-- image -->

Figure 13.13: Self-intersection curves of the offset of bisextic patch when d =-0.08. Figure ( a ) shows the pre-images of the self-intersection curves in parameter domain. The same symbols are mapped to the same points in the offset surface. Figure ( b ) shows the mapping of the self-intersection curves in the parameter domain onto the progenitor surface. Figure ( c ) shows the offset surface and the self-intersection curves.

<!-- image -->

- We can Taylor expand h ( x, y ) about (0 , 0) as follows

<!-- formula-not-decoded -->

where lim ( x,y ) → (0 , 0) R ( x 2 + y 2 ) -3 2 = 0.

- If we take into account that h (0 , 0) = h x (0 , 0) = h y (0 , 0) = 0, we can consider

<!-- formula-not-decoded -->

as the second order approximation of h ( x, y ).

- Let us denote E , F , G and L , M , N as coefficients of the first and second fundamental forms of the surface. If we assume further that x and y axes are directed along the principal directions at (0 , 0 , 0), assuming (0 , 0 , 0) is not an umbilic, then F = M = 0 [3]. It follows that h xy (0 , 0) = 0, since M = h xy / √ 1 + h 2 x + h 2 y . Although we have assumed (0 , 0) is not an umbilic, we can show that h xy (0 , 0) will also vanish when the point is an umbilic [19]. Also the principal curvatures at p can be expressed as follows [3]:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

- If we set α = h xx (0 , 0) and β = h yy (0 , 0) (thus -α = and -β = are principal curvatures) and assuming that p is a nonplanar point, the surface can be written locally as a second order approximation in the nonparametric form given by

<!-- formula-not-decoded -->

Its corresponding parametric form is

<!-- formula-not-decoded -->

In the sequel we assume that d &gt; 0, β &gt; 0 and α ≤ β without loss of generality. It follows that at (0,0,0) equation (13.50) holds and the x -axis will be the direction of maximum principal curvature and y -axis will be the direction for the minimum principal curvature. If α and β vanish at the same time, then the surface is part of a plane. Equation (13.51) or (13.52) represents explicit quadratic surfaces which can be categorized into four types according to combinations of α and β as listed in Table 13.1.

- The four types of explicit quadratic surfaces are depicted in Figure 13.14.

Theorem [15] Self-intersection curves of offsets of the explicit quadratic surfaces r ( x, y ) = [ x, y, 1 2 ( αx 2 + βy 2 )] T and their corresponding curves in the xy -plane are as follows:

̸

Table 13.1: Four types of explicit quadratic surfaces according to α and β

| Signs of α and β   | Types of Surfaces     | Types of Points at p   |
|--------------------|-----------------------|------------------------|
| αβ < 0             | Hyperbolic Paraboloid | Hyperbolic Point       |
| αβ > 0 and α = β   | Elliptic Paraboloid   | Elliptic Point         |
| α = β              | Paraboloid            | Umbilical Point        |
| α = 0 or β = 0     | Parabolic Cylinder    | Parabolic Point        |

Figure 13.14: Explicit quadratic surfaces z = 1 2 ( αx 2 + βy 2 ). (a) Top left: Hyperbolic paraboloid ( α = -3, β = 1). (b) Top right: Elliptic paraboloid ( α = 1, β = 3). (c) Bottom left: Paraboloid ( α = β = 3). (d) Bottom right: Parabolic cylinder ( α = 0, β = 3).

<!-- image -->

1. An offset of hyperbolic paraboloid ( α &lt; 0 &lt; β ) self-intersects only in the y -direction when 1 β &lt; d . The resulting self-intersection curve is a parabola given by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and its corresponding curve in the parameter space (i.e., xy -plane) is an ellipse when | α | = β or a circle when | α | = β , (see Figure 13.15 (a)) given by

̸

2. An offset of an elliptic paraboloid (0 &lt; α &lt; β ) self-intersects only in the y -direction when 1 β &lt; d &lt; 1 α and self-intersects in both x and y -directions when 1 α &lt; d . The selfintersection curve which self-intersects in the y -direction is a parabola (see Figure 13.11) given by equation (13.54) and the corresponding curve in the xy -plane is an ellipse (see Figures 13.11, 13.15 (b)) given by equation (13.54). The self-intersection curve which self-intersects in the x -direction is also a parabola given by

<!-- formula-not-decoded -->

its corresponding curve in the xy -plane is an ellipse (see Figures 13.15 (c), (d)) given by

<!-- formula-not-decoded -->

3. An offset of a paraboloid (0 &lt; α = β ) self-intersects in all directions, when 1 β = 1 α &lt; d . The self-intersection curve is a point (0 , 0 , ( βd ) 2 -1 2 β ), and its corresponding curve in the xy -plane is a circle (see Figure 13.15 (e)) given by

<!-- formula-not-decoded -->

4. An offset of a parabolic cylinder ( α = 0 &lt; β ) self-intersects only in the y -direction when 1 β &lt; d . The resulting self-intersection curve is a straight line in the xz -plane

<!-- formula-not-decoded -->

and its corresponding curves in the xy -plane (see Figure 13.15 (f)) are two straight lines given by

<!-- formula-not-decoded -->

Figure 13.15: Self-intersection and ridge curves of offsets of explicit quadratic surfaces. The solid lines correspond to self-intersection curves which degenerates in y -direction. The dashed lines correspond to κ min ( x, y ) = -1 d . The dot dashed lines correspond to self-intersection curves which degenerates in x -direction. The dot dot dashed lines correspond to κ max ( x, y ) = -1 d . Symbols × and ∗ represent the locations of generic lemon type umbilic and non-generic umbilic. (a) hyperbolic paraboloid ( α = -2, β = 2, d = 0 . 6) (b) elliptic paraboloid ( α = 1 . 75, β = 2, d = 0 . 55) (c) elliptic paraboloid ( α = 1 . 75, β = 2, d = 0 . 6) (d) elliptic paraboloid ( α = 1 . 75, β = 2, d = 0 . 65) (e) paraboloid ( α = β = 2, d = 0 . 6) (f) parabolic cylinder ( α = 0, β = 2, d = 0 . 6)

<!-- image -->

## 13.3.5 Approximations

Parametric Offset Surface Approximation Algorithm [23]. (See Figure 13.16)

1. Input: NURBS surface patch.
2. Offset each vertex of polygon by d with unit normal vector given by

<!-- formula-not-decoded -->

3. Check deviation of the approximate offset with the true offset (using the same weights for rational functions as the progenitor).
4. If the deviation is larger than the given tolerance subdivide the surface into four and go back to step 1. If the deviation is smaller than the given tolerance stop.

Figure 13.16: Offset surface approximation.

<!-- image -->

## Bibliography

- [1] Y. J. Chen and B. Ravani. Offset surface generation and contouring in computer-aided design. Journal of Mechanisms, Transmissions, and Automation in Design, Transactions of the ASME , 109(3):133-142, March 1987.
- [2] G. Dahlquist and ˚ A. Bj¨ orck. Numerical Methods . Prentice-Hall, Inc., Englewood Cliffs, NJ, 1974.
- [3] P. M. do Carmo. Differential Geometry of Curves and Surfaces . Prentice-Hall, Inc., Englewood Cliffs, NJ, 1976.
- [4] R. T. Farouki. Exact offset procedures for simple solids. Computer Aided Geometric Design , 2(4):257-279, 1985.
- [5] R. T. Farouki. The approximation of non-degenerate offset surfaces. Computer Aided Geometric Design , 3(1):15-43, May 1986.
- [6] R. T. Farouki and C. A. Neff. Algebraic properties of plane offset curves. Computer Aided Geometric Design , 7(1 - 4):101-127, 1990.
- [7] R. T. Farouki and C. A. Neff. Analytic properties of plane offset curves. Computer Aided Geometric Design , 7(1 - 4):83-99, 1990.
- [8] R. T. Farouki and T. W. Sederberg. Analysis of the offset to a parabola. Computer Aided Geometric Design , 12(6):639-645, September 1995.
- [9] I. D. Faux and M. J. Pratt. Computational Geometry for Design and Manufacture . Ellis Horwood, Chichester, England, 1981.
- [10] R. Klass. An offset spline approximation for plane cubic splines. Computer-Aided Design , 15(4):297-299, September 1983.
- [11] R. Kunze, F.-E. Wolter, and T. Rausch. Geodesic Voronoi diagrams on parametric surfaces. In Proceedings of Computer Graphics International, CGI '97, June 1997 , pages 230-237. IEEE Computer Society Press, 1997.
- [12] T. Lozano-Perez and M. A. Wesley. An algorithm for planning collision-free paths amongst polyhedral obstacles. Communications of the ACM , 25(9):560-570, October 1979.
- [13] W. L¨ u. Rational offsets by reparametrization. Technical report, Zhejiang University, December 1992.

- [14] T. Maekawa. An overview of offset curves and surfaces. Design Laboratory Memorandum 98-2, MIT, Department of Ocean Engineering, Cambridge, MA, March 1998.
- [15] T. Maekawa. Self-intersections of offsets of quadratic surfaces: Part I, explicit surfaces. Engineering with Computers , 14:1-13, 1998.
- [16] T. Maekawa, W. Cho, and N. M. Patrikalakis. Computation of self-intersections of offsets of B´ ezier surface patches. Journal of Mechanical Design, Transactions of the ASME , 119(2):275-283, June 1997.
- [17] T. Maekawa and N. M. Patrikalakis. Computation of singularities and intersections of offsets of planar curves. Computer Aided Geometric Design , 10(5):407-429, October 1993.
- [18] T. Maekawa, N. M. Patrikalakis, T. Sakkalis, and G. Yu. Analysis and applications of pipe surfaces. Computer Aided Geometric Design , 15(5):437-458, May 1998.
- [19] T. Maekawa, F.-E. Wolter, and N. M. Patrikalakis. Umbilics and lines of curvature for shape interrogation. Computer Aided Geometric Design , 13(2):133-161, March 1996.
- [20] N. M. Patrikalakis and L. Bardis. Offsets of curves on rational B-spline surfaces. Engineering with Computers , 5:39-46, 1989.
- [21] N. M. Patrikalakis and L. Bardis. Localization of rational B-spline surfaces. Engineering with Computers , 7(4):237-252, 1991.
- [22] N. M. Patrikalakis and H. N. Gursoy. Shape interrogation by medial axis transform. In B. Ravani, editor, Proceedings of the 16th ASME Design Automation Conference: Advances in Design Automation, Computer Aided and Computational Design, Vol. I , pages 77-88, Chicago, IL, September 1990. New York: ASME.
- [23] N. M. Patrikalakis and P. V. Prakash. Free-form plate modeling using offset surfaces. Journal of OMAE, Transactions of the ASME. , 110(3):287-294, 1988.
- [24] B. Pham. Offset curves and surfaces: a brief survey. Computer-Aided Design , 24(4):223229, April 1992.
- [25] T. Rausch, F.-E. Wolter, and O. Sniehotta. Computation of medial curves on surfaces. In T. Goodman and R. Martin, editors, The Mathematics of Surfaces VII , pages 43-68. Information Geometers, 1997.
- [26] J. R. Rossignac and A. G. Requicha. Offsetting operations in solid modelling. Computer Aided Geometric Design , 3(2):129-148, 1986.
- [27] W. Tiller and E. G. Hanson. Offsets of two-dimensional profiles. IEEE Computer Graphics and Applications , 4(9):36-46, September 1984.
- [28] T. J. Willmore. An Introduction to Differential Geometry . Clarendon Press, Oxford, 1959.
- [29] F.-E. Wolter. Cut locus and medial axis in global shape interrogation and representation. Memorandum 92-2, Cambridge MA: MIT Ocean Engineering Design Laboratory, January 1992.