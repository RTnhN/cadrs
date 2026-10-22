## Computing Moments of Objects Enclosed by Piecewise Polynomial Surfaces

CARLOS GONZALEZ-OCHOA, SCOTT MCCAMMON and JO ¨ RG PETERS Purdue University

Combining a polynomial free-form surface representation with Gauss' divergence theorem allows efficient and exact calculation of the moments of the enclosed object. For example, for a cubic representation, volume, center of mass, and the inertia tensor can be computed in seconds even for complex objects with several thousand patches while changes due to local modification of the surface geometry can be computed in real-time as feedback for animation or design. Speed and simplicity of the approach allow solving the inverse problem of modeling to match prescribed moments.

Categories and Subject Descriptors: G.1.4 [ Numerical Analysis ]: Quadrature and Numerical Differentiationmultiple quadrature ; I.3.5 [ Computer Graphics ]: Computational Geometry and Object Modelingcurve, surface, solid, and object modeling

General Terms: Algorithms, Design

Additional Key Words and Phrases: Animation techniques, CAD/CAM, geometric modeling, interactive techniques, splines and surfaces, university education, virtual/interactive environments

## 1. INTRODUCTION

Realistic animation and geometric design must both pay close attention to the physics implied by the first moments, the volume, center of mass, and inertia frame of the objects they manipulate. A jug whose fill level is inconsistent with the amount of water put into it, or one that does not topple over when the projection of the center of mass moves outside the support, puzzles and distracts the viewer and can lead to disaster when unobserved in engineering design. Although animators and designers often have an excellent intuitive grasp of the approximate physics implied by shape, the immediate and exact feedback from a tool such as the one developed in the following can save the user both time and energy. Real-time visual feedback, visualization of moments, and the solution of

Authors' address: Department of Computer Sciences, Purdue University, W. Lafayette, IN 47907-1398; email: jorg@cs.purdue.edu.

Permission to make digital/hard copy of part or all of this work for personal or classroom use is granted without fee provided that the copies are not made or distributed for profit or commercial advantage, the copyright notice, the title of the publication, and its date appear, and notice is given that copying is by permission of the ACM, Inc. To copy otherwise, to republish, to post on servers, or to redistribute to lists, requires prior specific permission and/or a fee.

© 1998 ACM 0730-0301/98/0700-0143 $05.00

simple inverse problems help to identify critical points in the moment distribution.

The key to fast and exact computation of moments of smooth free-form surfaces is to combine the advantages of a low-degree polynomial surface representation with Gauss' divergence theorem. Section 2 reviews how the well-known divergence theorem reduces the moment computation for piecewise polynomial surfaces to integrating related polynomials. Section 3 collects criteria on the surface representation that allow efficient application of this fact. For free-form surfacing, C 1 -surface splines [Peters 1995] match the bill and are used as a concrete representation of free-form manifolds in the following sections. Section 4 discusses the details of the evaluation of the surface integrals as simple averages of the coefficients and gives an operations count. In Section 5, we measure and visualize moments and in Section 6 we model geometry to match moments. Section 7 lists extensions and more applications ahead.

## 1.1 Prior Work

The engineering literature is rich in applications of moments and algorithms for computing moments based on cellular, piecewise constant, or polyhedral piecewise linear approximations of the object ([Lee and Requicha 1982; Lien and Kajiya 1984; Cattani and Paoluzzi 1990], the massprop command in the ACIS solid modeler), of its surface [Messner and Taylor 1980; Miles and Tough 1983], of the divergence theorem [Yang et al. 1997], or the associated integrals [Timmer and Stern 1980]. For example, Kobbelt [1997] uses a subtle nonuniform extrapolation scheme that requires exact arithmetic to approximately evaluate moments.

Although appropriate for nonpolynomial or nonlinearly trimmed surface representations, approximations are inappropriate (namely, less efficient or accurate) for objects enclosed by standard polynomial patches, such as polynomial B-splines. The approximations obscure the fact that the moment computation requires no evaluation of rational integrals or approximation of surface area. This article highlights this fact by combining a simple surface representation with classical formulas and thus illustrating the direct and exact computation of the first moments of smooth (and nonsmooth) polynomial free-form surfaces of arbitrary patch layout and topological genus.

## 2. THE ELEGANT DIVERGENCE THEOREM

Applied by Gauss in 1813, the well-known divergence theorem (see, e.g., Rudin [1987, 10.51]), relates a volume integral to an integral over the surface. Given a map f ; R n R 3 , its divergence ' z f 5 ( ( › / › x i ) f i and its normal component f z N 5 ( f iNi , the theorem states that

<!-- formula-not-decoded -->

ACM Transactions on Graphics, Vol. 17, No. 3, July, 1998.

·

that is, the integral of the divergence of f over the volume V equals the integral of the normal component over the surface S of V .

To apply the theorem to moment calculations denote the parameterization of the surface S by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and the domain of x by U ; that is, ( u , v ) [ U . The surface normal is the vector N 5 n / u n u , where

<!-- formula-not-decoded -->

The crucial observation is that by change of variable

<!-- formula-not-decoded -->

That is, the area element u n u is the inverse of the normalization factor of the normal direction so that

<!-- formula-not-decoded -->

If both f and x are polynomial the integrand is polynomial; and if U is a simple domain, say a triangle or square, then it is clear, and made precise in Section 4, that the integral can be determined efficiently, explicitly, and exactly. This remains true if the parameterization x of S consists of patches x i and U is the union of the patch domains U i :

<!-- formula-not-decoded -->

We now consider specific choices of f that allow us to determine the first

146 · C. Gonzalez-Ochoa et al.

three moments of a solid with uniform mass distribution. It is efficient to choose f 5 [0, 0, f 3] so that we need to compute only

<!-- formula-not-decoded -->

The zeroth moment, } 0 5 * V dV , measures the volume enclosed by S . With f 3 5 z ,

<!-- formula-not-decoded -->

The first moments } i 5 * VxidV represent the components of the center of mass. Choosing f 3 as xz , yz , and z 2 / 2, respectively, yields

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The second moments yield the components of the inertia tensor

<!-- formula-not-decoded -->

We list } ij 5 * Vxi x jdV .

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

ACM Transactions on Graphics, Vol. 17, No. 3, July, 1998.

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The preceding formulas assume that the volume has homogeneous density. If this is not the case but the antiderivative of the moment-weighted mass distribution r ( x , y , z ) is known, f 3 can be chosen so that similar formulas hold.

## 3. SELECTING A SURFACE REPRESENTATION

The derivation in the previous section did not depend on a particular representation of the surface S . However, it implied that for efficient and exact computability of the moments these properties of the surface patches are desirable:

- (1) polynomial patches,
- (2) low degree patches,
- (3) few patches, and
- (4) simple domains.

These properties rule out several otherwise excellent surface representations. For example, S-patches [DeRose and Loop 1988] and the recent construction by Grimm and Hughes [1995] are rational and have nonsimple domains so that integrals are best evaluated by numerical approximation. Simplex splines [Fong and Seidel 1993] use a large number of pieces and generalized subdivision surfaces (e.g., Doo and Sabin [1978]; Catmull and Clark [1978]; and Zorin et al. [1996]), and lack a global analytic definition for application of the divergence theorem (but see Peters and Nasri [1997]). Although nonlinearly trimmed NURBS patches violate the simple domain requirement, standard polynomial tensor-product Bsplines as well as other methods [Shirman and Se ´quin 1987; Reif 1998; Prautzsch 1997] match the requirements but are of higher degree, and hence more expensive to evaluate than C 1 -surface splines [Peters 1995]. In the following sections, we use piecewise cubic surface splines for the experiments, not least because they allow interactive modeling of smooth

·

and nonsmooth surfaces of arbitrary patch layout and topological genus. The Appendix has a synopsis of surface spline properties and the corresponding surfacing algorithm.

## 4. COMPUTING SURFACE INTEGRALS

To determine the moments, integrals of type

<!-- formula-not-decoded -->

are evaluated. For example, for cubic patches, the integrands of the q th moment are scalar-valued polynomial pieces in two variables and of total degree

<!-- formula-not-decoded -->

that is, degree 7, 10, and 13, respectively, for the first three moments. The exact integral of a piece is simply the average of its Bernstein-Be ´zier control points (see, e.g., de Boor [1987]); that is, integration consists of summing k ; 5 ( d 1 2)( d 1 1)/ 2 coefficients and dividing by k .

To verify that the moment calculation fits into an interactive environment, we compute the number of operations. The polynomial n 3 is the difference of products of derivatives. Algorithms for differentiation and multiplication of polynomials in the Bernstein-Be ´zier form are standard and can be found in many textbooks, for example, Farin [1993] and in the short article by de Boor [1987]. For the count we recall that a polynomial of degree d in two variables has

<!-- formula-not-decoded -->

coefficients.

- -Differentiation requires m ( d 2 1) differences.
- -Integration requires m ( d ) additions and one division.
- -Multiplication of two scalar-valued polynomials of degree d 1 and d 2 , respectively, requires m ( d 1 ) m ( d 2 ) multiplications and m ( d 1 ) m ( d 2 ) 2 m ( d 1 1 d 2 ) additions provided we prescale and postscale the coefficients with precomputed binomial multipliers ( i dj ) at a cost of m ( d 1 ) 1 m ( d 2 ) 1 m ( d 1 1 d 2 ) multiplications.

Figure 1 depicts the computation of the volume contribution of a cubic patch together with the corresponding operation count: there are four differentiations at 6 Q each, two multiplications at (36 1 6 1 6 1 15) R and (36 2 15) Q each, the addition of two degree 4 scalar-valued polynomials at m (4) Q 5 15 Q , and the final multiplication of a degree 4 with a degree 3 polynomial. The additional work for each moment is tabulated in the

·

Fig. 1. Computing schema for the volume contribution of a cubic Be ´zier patch.

<!-- image -->

following. The work includes pre- and postscaling which is not necessary when the polynomials are reused in further calculations since the prescaling of the next undoes the postscaling. Figure 2 shows the same process for a bicubic patch.

| Moment   |   d | 1               | z       |
|----------|-----|-----------------|---------|
| n 3      |   4 | 81              | 126     |
| volume   |   7 | 114 1 36        | 211     |
| centroid |  10 | (294 1 66) z 3  | 436 z 3 |
| inertia  |  13 | (660 1 105) z 6 | 775 z 6 |

Hence, we can compute the volume contribution of one patch in roughly 570 operations and all components of the first three moments of a patch in less than 12K operations.

## 5. MEASURING MOMENTS

The principal inertia directions are axes of symmetry with respect to mass distribution and are computed as eigenvectors of the inertia tensor ( . The eigenvalues of the inertia tensor represent resistance to torque. Both sets of information are displayed in the inertia frame, a stencil attached to the center of mass whose legs are aligned with the principal inertia directions and stretched according to the corresponding eigenvalues. The smoothed brick shape in Figure 3 is displayed to provide a simple example of an inertia frame. Figure 4 shows three snapshots of a real-time design sequence that modifies the geometry of the brick. More complicated objects are shown in Figure 5. In Figure 5 ( top ), the geometry of the object is changed by moving one arm of the robot gripper, the result of lifting the surface spline control net. In Figure 5 ( bottom ), the geometry of the object is changed by smoothing, the result of changing the surface spline cut ratios from zero to 0.5. The shadow is generated as an identical object with all z -components collapsed to zero.

## 6. MODELING TO MATCH MOMENTS

The ability to measure moments efficiently as demonstrated in the previous section is the key to solving the inverse problem of designing geometry to match a prescribed (set of) moments. In general this problem is underconstrained since the geometry offers many degrees of freedom. A standard approach to regularizing the problem is to add variational constraints. Rather than grazing on this fertile field, we look at some practically relevant problems where the geometry is restricted to change only in one parameter.

Fig. 2. Computing schema of the volume contribution of a bicubic Be ´zier patch.

<!-- image -->

Fig. 3. The inertia frame of a smoothed brick shape. The inertia frame is formed by the principal directions of the inertia tensor attached to the center of mass.

<!-- image -->

## 6.1 Volume Matching

Consider putting water into a jug held in fixed position. Until overflow, the geometry of the 'water object' is constrained by the geometry of the container: the only degree of freedom is the height of the water level. A well-posed problem is therefore to ask for the fill height h # of the container x given a fixed water volume V 0 . The problem is nontrivial, since the top surface of the water at rest is a plane that cuts the container patches in

ACM Transactions on Graphics, Vol. 17, No. 3, July, 1998.

·

Fig. 4. Change of the inertia frame under distortion of the transparently rendered brick.

<!-- image -->

algebraic curves resulting in challenging boundary integrals when an exact solution is sought. Rotating the container if necessary so that the negative z -direction agrees with the direction of gravity, the problem can be formally written as

<!-- formula-not-decoded -->

The problem has the nice property that h # is a monotone function of the volume so that bisection will not only succeed if the fluid volume fits into the container but also yield a sequence of upper and lower bounds in the process. Newton's method guarantees no such bound and has to cope with the fact that h # is generally not a differentiable function of V 0 : consider the rate of change at the branch point when filling a T-shape.

Each bisection step requires computing a volume bounded by the intersection of the free-form surface with a plane. To avoid the exact intersection computation, we subdivide the patches in the vicinity of the intersection such that we overestimate or underestimate the volume already enclosed. Using surface splines, the computation is fast enough to allow the real-time animation shown in Figure 6. Here the volume is kept constant and the fill height is recomputed, yielding design feedback while the containing shape is interactively deformed.

## 6.2 Mass Matching

As a second example, we determine the point of instability of a gripper placed on a tabletop. Here the free parameter is the opening angle a and the condition is that the projection of the center of mass m ( x ) comes to lie on the table edge E ; that is,

find a such that m ~ x ~ a , u !! , E .

Fig. 5. Change of the inertia frame under deformation and smoothing.

<!-- image -->

·

Fig. 6. Deformation of the container object alters the fill height for a constant fill volume.

<!-- image -->

Bisection with a call to the mass computing module rather than the volume module solves the problem. The sequence of computations is animated in Figure 7. Note that as a varies, the blend between finger and body changes. Therefore the standard engineering approach of decomposing the gripper into a fixed and a moving part and adding their respective centers of mass to compute the center of mass of the whole object yields a less accurate result.

## 154 · C. Gonzalez-Ochoa et al.

Fig. 7. Determining the point of instability while increasing the opening angle.

<!-- image -->

ACM Transactions on Graphics, Vol. 17, No. 3, July, 1998.

·

Fig. 8. Using the inertia frame to match an object to a cloud of sample points.

<!-- image -->

## 6.3 Inertia Matching

The inertia tensor ( is a positive definite, symmetric matrix. As pointed out in Section 5, its eigenvectors and eigenvalues can be used, together with the center of mass, to define a frame that is unique if the eigenvalues are distinct. Matching inertia frames can thus be used as a quick first approximation to determine an unknown rigid motion by which an object is displaced with respect to a reference position (see, e.g., Prokop and Reeves [1992]). For example, Figure 8 shows the input data of a successful object registration, where a cloud of measured points from a physical object and a surface model of this object are to be brought into agreement by a rigid motion, computed as an affine transformation relating the inertia frames of the two entities. In our limited experience, this method worked very well even when the sampled points were not exactly on the given model and was vastly faster than sampling the surface model and attempting a leastsquares fit of the two point clouds.

## 7. CONCLUSION

Moment-based tools will give animators and designers a much better feel for the objects they are working with and undoubtedly inspire innovative uses. A likely application, hinted at in Figure 7, is physics-based animation. Mechanical engineers for their part may appreciate exact moment information for complex free-form objects. One of the referees suggested preserving moments during multiresolution modeling.

·

Fig. 9. Input polyhedron and planar-cut polyhedron.

<!-- image -->

As mentioned in Section 6, automatic shape modification based on matching moments is an intriguing, though underconstrained problem. For a low-degree representation such as surface splines, it is actually possible to relate a change of the moments explicitly to a change of the spline parameters, so that variational criteria more complex than generic leastsquares minimization may be well defined and computationally tractable.

## APPENDIX. SURFACE SPLINE SYNOPSIS

C 1 -surface splines [Peters 1995] define a piecewise cubic manifold capable of modeling smooth and nonsmooth surfaces of arbitrary patch layout and topological genus. The embedding of the smooth manifold into R 3 is locally governed by so-called blend ratios. These numbers allow for local distribution of curvature and fine-tuning of shape; where they are set to zero, sharp edges and vertices result so that also purely polyhedral objects can be modeled in this framework. Surface splines come in three flavors, with an underlying representation of exclusively 4-sided, exclusively 3-sided, and a mixture of 3- and 4-sided patches, respectively. Exclusively 4-sided patches have recently been used for reconstruction [Eck and Hoppe 1995]. However, the exclusively 3-sided and the mixed representation have better shape properties, satisfy the strong local convex hull property, and have lower algebraic and parametric degree. Moreover, they too can be represented as standard linearly trimmed NURBS quilts and hence stored, transmitted, and rendered in standard form [Peters 1997].

Surfacing with surface splines consists of two stages. In the first, illustrated in Figure 9, an arbitrarily connected input polyhedron is transformed into a planar-cut polyhedron by (edge and) corner cutting (cf Peters [1995, pp. 649-650]). A planar-cut polyhedron is a polyhedron with every interior vertex surrounded by four facets. The first and third facet are 4-sided, the other two must be the affine image of an n -gon if they have more than four edges. Zero depth cuts, corresponding to a nonsmooth embedding of the C 1 manifold into R 3 , are permitted so that also purely polyhedral approximations can be modeled and measured in this framework. In the second step, the Be ´zier coefficients of the cubic patches are computed as simple convex combinations of the planar-cut polyhedron (cf Peters [1995, p. 652]).

·

- CATMULL, E. AND CLARK, J. 1978. Recursively generated B-spline surfaces on arbitrary topological meshes. Comput. Aided Des. 10 , 350-355.
- CATTANI, C. AND PAOLUZZI, A. 1990. Boundary integration over linear polyhedra. Comput. Aided Des. 22 , 2 (March), 130-135.
- DE BOOR, C. 1987. B-form basics. In Geometric Modeling: Algorithms and New Trends , G. Farin, Ed., SIAM, Philadelphia, 131-148.
- DEROSE, T. D. AND LOOP, C. T. 1988. S-patches: A class of representations for multi-sided surface patches. Tech. Rep., University of Washington, Department of Computer Science, Seattle.
- DOO, D. AND SABIN, M. 1978. Behaviour of recursive subdivision surfaces near extraordinary points. Comput. Aided Des. 10 , 6 (Nov.), 356-360.
- ECK, M. AND HOPPE, H. 1995. Automatic reconstruction of B-spline surfaces of arbitrary topological type. Proceedings of SIGGRAPH 95 (Los Angeles, CA, Aug. 6-11).
- FARIN, G. 1993. Curves and Surfaces for Computer Aided Geometric Design . Third ed., Academic, London.
- FONG, P. AND SEIDEL, H.-P. 1993. An implementation of triangular B-spline surface over arbitrary triangulations. Comput. Aided Geom. Des. 10 , 267-275.
- GRIMM, C. M. AND HUGHES, J. F. 1995. Modeling surfaces of arbitrary topology using manifolds. Proceedings of SIGGRAPH 95 (Los Angeles, CA, Aug. 6-11).
- KOBBELT, L. 1997. Robust and efficient evaluation of functional on parametric surfaces. In Proceedings of the Thirteenth Annual Symposium on Computational Geometry (June 4-6), ACM, New York, 376-381.
- LEE, Y. T. AND REQUICHA, A. A. G. 1982. Algorithms for computing the volume and other integral properties of solids. II. A family of algorithms based on representation conversion and cellular approximation. Commun. ACM 25 (Sept.), 642-650.
- LEIN, S. AND KAJIYA, J. 1984. A symbolic method for calculating the integral properties of arbitrary nonconvex polyhedra. IEEE Comput. Graph. Appl. 4 , 9 (Oct.), 35-41.
- MESSNER, A. AND TAYLOR, G. 1980. Algorithm 550: Solid polyhedron measures. ACM Trans. Math. Softw. 6 , 1 (March), 121-130.
- MILES, R. G. AND TOUGH, J. G. 1983. A method for computation of inertia properties for general areas. Comput. Aided Des. 15 , 196-200.
- PETERS, J. 1995. C 1 -surface splines. SIAM J. Numer. Anal. 32 , 2, 645-666.
- PETERS, J. 1997. Pcp2nurb: Smooth free-form surfacing with linearly trimmed bicubic B-splines. ACM Trans. Math. Softw. (to appear); see also netlib software.
- PETERS, J. AND NASRI, A. 1997. Computing volumes of solids enclosed by recursive subdivision surfaces. Comput. Graph. Forum 16 , 3 (Sept.), C-89-C-94.
- PRAUTZSCH, H. 1997. Free-form splines. Comput. Aided Geom. Des. 14 , 421-447.
- PROKOP, R. J. AND REEVES, A. P. 1992. A survey of moment-based techniques for unoccluded object representation and recognition. CVGIP: Graph. Models Image Process. 54 , 5 (Sept.), 438-460.
- REIF, U. 1998. Turbs-topologically unrestricted B-splines. Constr. Approx. (to appear).
- RUDIN, W. 1987. Real and complex analysis. Series in Higher Mathematics, McGraw-Hill, New York.
- SHIRMAN, L. AND SE ´ QUIN, C. 1987. Local surface interpolation with Be ´zier patches. J. CAGD 6 , 2, 167-172.
- TIMMER, H. AND STERN, J. 1980. Computation of global geometric properties of solid objects. Comput. Aided Des. 12 , 6, 301-304.
- YANG, L., ALBREGTSEN, F., AND TAXT, T. 1997. Fast computation of three-dimensional geometric moments using a discrete divergence theorem and a generalization to higher dimensions. Graph. Models Image Process. 59 , 2 (March), 97-108.
- ZORIN, D., SCHRO ¨ DER, P., SWELDENS, W. 1996. Interpolating subdivision for meshes with arbitrary topology. Proceedings of SIGGRAPH 96 (New Orleans, LA, Aug. 4-9), 189-192.

Received March 1997; revised February 1998; accepted March 1998