## High-accuracy mesh-free quadrature for trimmed parametric surfaces and volumes

David Gunderman a,b, ∗ , Kenneth Weiss b , John A. Evans c

a Department of Applied Mathematics, University of Colorado Boulder, Boulder, CO 80309, USA

b Lawrence Livermore National Laboratory, 7000 East Avenue, Livermore, CA 94550, USA c Ann and H.J. Smead Department of Aerospace Engineering Sciences, University of Colorado Boulder, Boulder, CO 80309, USA

## Abstract

This work presents a high-accuracy, mesh-free, generalized Stokes theorem-based numerical quadrature scheme for integrating functions over trimmed parametric surfaces and volumes. The algorithm relies on two fundamental steps: (1) We iteratively reduce the dimensionality of integration using the generalized Stokes theorem to line integrals over trimming curves, and (2) we employ numerical antidifferentiation in the generalized Stokes theorem using high-order quadrature rules. The scheme achieves exponential convergence up to trimming curve approximation error and has applications to computation of geometric moments, immersogeometric analysis, conservative field transfer between high-order curvilinear meshes, and initialization of multi-material simulations. We compare the quadrature scheme to commonly-used quadrature schemes in the literature and show that our scheme is much more efficient in terms of number of quadrature points used. We provide an open-source implementation of the scheme in MATLAB as part of QuaHOG, a software package for Quadrature of High-Order Geometries.

Keywords: Quadrature, High-order, Trimmed, Immersed

## 1. Introduction

Quadrature over trimmed parametric surfaces and over regions bounded by trimmed parametric surfaces is important in a variety of applications. For example, in computer-aided geometric design (CAGD), three-dimensional geometric objects are typically given in terms of boundary representations (BREPs), often as tensorproduct non-uniform rational B-spline (NURBS) surfaces or Tspline surfaces. In many CAGD programs, trimmed NURBS patches or T-spline patches- formed by discarding portions of the patch - are also allowed (c.f. Figure 1) [1, 2, 3, 4, 5, 6].

Some of a trimmed patch's edge curves are formed as intersections of two parametric patches. These edge curves are called trimming curves and can be represented either as physical space curves in R 3 or as parametric space curves in the parametric spaces of either trimmed patch [7]. Efficient and accurate integration of material properties over volumes bounded by sets of trimmed patches, or over the surfaces formed as unions of trimmed patches themselves, is often an important operation in the design process, and can be used, for example, to compute an object's volume or surface area [8].

In addition to its application to geometric design, integration over regions bounded by trimmed parametric surfaces is important in the context of computer-aided engineering (CAE), where it is common to use high-order polynomials and/or rational functions as a basis for analysis in the finite element method (FEM). Integration over surfaces and/or volumes formed by trimming typical finite element shapes (such as high-order quadrilateral patches or hexahedral cells) is a fundamental operation in analysis paradigms such as moment-fitting methods [9], immersed finite element methods [10], Lagrangian remap methods [11], and multi-mesh methods [12].

For example, in high-order Lagrangian remap methods for computational fluid dynamics (CFD), a curvilinear finite element mesh is allowed to move with the fluid flow until it becomes too distorted for analysis. Then, field variables defined on the distorted source mesh are projected onto an analysis-ready target mesh. Conservation of integrated material quantities, such as total energy, is ensured only with highly accurate integration over the intersections between source mesh elements and target mesh elements [11, 13, 14].

∗ Corresponding author

Email address: david.gunderman@colorado.edu (David Gunderman)

Figure 1: An example of a region formed by trimmed surface patches. a) The region is bounded by five tensor-product parametric patches with trimming curves shown in red. b) The trimmed object is shown.

<!-- image -->

As another example, in immersed finite element methods, an object of geometric interest is immersed in a simpler nonboundary-fitted background mesh. Boundary conditions, domain restrictions, and interface conditions are then taken into account through accurate integration over (1) the volume enclosed by cut cells and (2) the surface of the cut cells [15, 16, 17]. See Figure 2 for an example of a cut cell formed by a background Cartesian mesh and a geometric object shaped like the Utah teapot.

Efficient high-order integration over typical FEM volume elements and surface elements has been studied extensively for simple cell shapes, such as hexahedral cells and quadrilateral surface patches. Highly optimized rules have been developed in these cases by taking advantage of the existence of a parametric mapping from a simple reference element such as the unit cube or unit square. These methods typically rely on mapping a high order quadrature rule designed for the parametric space into the considered physical space [18, 19]. However, developing high-accuracy quadrature rules for arbitrary cell geometries and topologies, as occur for trimmed surfaces and volumes, is more difficult and is an active area of research.

Figure 2: a) In immersed methods, a shape like the Utah teapot is immersed into a background grid of Cartesian cells on which a finite element space is defined. b) The boundary is then incorporated by considering where it intersects each cell. In this case, the boundary intersects one of the cells as pictured. Efficient integration must be performed on these types of surfaces, as well as on the volume left after cutting away everything outside the given surface.

<!-- image -->

Accurate integration over arbitrary surfaces and volumes has been approached from multiple standpoints, which can be broadly categorized into (1) domain decomposition methods, such as octree decomposition or body-fitted meshing of the trimmed geometry and (2) dimension-reduction techniques, in which d -dimensional integrals are transformed into d -1-dimensional integrals. We provide a brief review of these types of methods below. For more complete short surveys of quadrature strategies for arbitrary geometries, see [20, 21, 22].

In domain decomposition-based integration methods, the integration region is decomposed into simpler subregions and then integration is performed over each of the simpler subregions. Domain decomposition can be split into two general categories: loworder domain decomposition and high-order domain decomposition. In low-order domain decomposition integration methods, the geometry of interest is decomposed into a set of simple planarfaceted tetrahedra/triangles or hexahedra/quadrilaterals which approximate the original geometry, then standard quadrature rules for these shapes are used to integrate functions [23, 24]. Because the boundary is approximated by straight-sided geometry, these methods can only achieve low-order convergence to the correct integral with respect to total number of quadrature points. Much work has been done on optimizing these low-order methods [22, 25]. However, the fundamentally low convergence precludes the possibility of high accuracy integration.

Higher-order domain decomposition integration methods rely on a decomposition of the geometric domain into high-order curvilinear elements, such as curved tetrahedra or triangles, and can be thought of as high-order meshing. These methods can attain high-order convergence to the correct integral as the number of subelements increases. However, due to the inherent difficulty of high-order meshing, the pre-processing step of finding a highorder domain decomposition is known to be costly, difficult, and nonrobust [26, 27, 28, 29, 30, 31].

A common method for simplifying quadrature over complicated geometries avoids decomposing the interior by transforming d -dimensional integrals over a domain's interior into a sum of d -1-dimensional integrals over its boundaries, typically using the generalized Stokes theorem or the implicit function theorem. These methods have been employed extensively for linear polyhedral regions and implicitly-defined regions [20, 32, 33, 34], particularly when the integrand is symbolically antidifferentiable. To the best of our knowledge, the use of a dimension reduction technique for high-accuracy integration of arbitrary functions over regions bounded by trimmed parametric surfaces or over the surfaces themselves has not been described in the literature, though similar methods have been hinted at for untrimmed surfaces [9].

Additionally, a popular belief about Stokes theorem-based methods is that they can only be used on integrands for which symbolic antiderivatives are available, and these methods have been used in this case for decades [35, 36]. However, recently it has become more common to use Stokes theorem-based methods for arbitrary integrands [37, 38] since numerical antiderivatives can be efficiently computed to high precision using a highaccuracy quadrature rule, such as Gaussian quadrature. These methods have, to the best of our knowledge, only recently been used in 2D or surface cases [39]. In this paper, we extend them to the 3D case.

Another interesting approach which has been described for trimmed volumetric domains whose trimming surfaces can be described as implicit functions is that of the error correction termtype schemes, which initially approximates the trimming function with a linear function, then applies a Taylor series correction to obtain higher order convergence [40, 41, 21]. These methods can only be applied to parametric surfaces if they can be implicitized, which can be prohibitively expensive for even quite simple shapes. For example, the intersection of two bicubic surfaces is, in the general case, a 324 th degree implicit curve in physical space [42].

Table 1: A table of convergence rates under geometric refinement of the approximation of the trimming curves and under integration refinement of the underlying Gaussian quadrature rules using our technique. The refinement strategies are explained further in Section 3.3.

|                                             | untrimmed/exact trimming curves   | approximating trimming curves                                                          |
|---------------------------------------------|-----------------------------------|----------------------------------------------------------------------------------------|
| geometric refinement integration refinement | N/A exponential convergence       | high-order convergence to integration error exponential convergence to geometric error |

If surfaces are given as polynomial or rational parametric inputs (as is the case for NURBS objects), extra information in the form of a high-order parametrization is available without any extra approximation (e.g., by polyhedra), and this information can be fully exploited to attain the most efficient quadrature rules possible. Similarly, if trimming curves are given as high-order curves, reverting to straight-edged approximations of the trimming curves loses information, which reduces accuracy.

It is important here to mention the moment-fitting method, a common tool in numerical quadrature. Given accurate moments up to degree p for the geometry, the moment-fitting method produces a quadrature scheme which is 1) exact for polynomials up to degree p and 2) such that quadrature points are at pre-specified locations. Moreover, the quadrature weights of the momentfitting quadrature rule can be enforced to be positive. Importantly, the moment-fitting method can be combined with any integration procedure and the resulting quadrature rule's accuracy depends on the accuracy of the original integration procedure used for calculation of the moments [9, 43, 44]. Our quadrature schemes can be used to accurately calculate the moments which are required as inputs into the moment-fitting algorithm.

In [37, 39], quadrature schemes with spectral (i.e., faster than any algebraic order) or high-order convergence are developed for area integration over planar domains bounded by polynomial or rational parametric curves, respectively, using an application of Green's theorem combined with numerical antidifferentiation. In this text, we extend that work to consider two problems. First, we consider the problem of evaluating the surface integral of an arbitrary function over a trimmed parametric surface, S . Second, we consider the problem of evaluating the volume integral of an arbitrary function over a region, V , bounded by a closed set of trimmed parametric surfaces. In both of these cases, we focus on regions that may be difficult to efficiently mesh. Formally, we develop numerical algorithms for computing high-accuracy quadrature schemes for integrals of the forms

<!-- formula-not-decoded -->

We assume V is bounded by a set of ns trimmed tensor product surfaces patches { S ′ i } ns i = 1 that are subsets of untrimmed patches { Si } ns i = 1 ,

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

We assume S is simply the surface defined by a union of some subset of the trimmed patches S ′ i .

The algorithms for producing our proposed quadrature scheme for surface and volume integration rely on two fundamental components:

- (1) We iteratively reduce the dimensionality of integration using the generalized Stokes theorem to line integrals over trimming curves, and
- (2) We employ numerical antidifferentiation in the generalized Stokes theorem using exponentially-convergent quadrature rules.

We note that the presented algorithm can be easily adapted to the case of trimmed and untrimmed simplicial patches, if desired.

The algorithm produces quadrature rules which, for smooth integrands, converge to the correct integral exponentially (i.e., faster than any algebraic order) to machine precision with respect to the number of quadrature points for untrimmed patches and patches with polynomial or rational trimming curves. For trimmed patches with general (e.g. algebraic) trimming curves, the algorithm converges exponentially with respect to the number of quadrature points until the error in the geometric approximation of the trimming curvs dominates the total error. Under geometric refinement of the trimming curve approximations, the integrals converge with a high order which is determined by the trimming curve approximations and the order of the surfaces. See Table 1 for a summary of these convergence rates. In addition to its convergence properties, our algorithm are computationally efficient. The cost for producing the quadrature rules scales linearly with the number of points in each of the underlying Gaussian quadrature rules, assuming trimming curves or points along the trimming curve are given a priori , as is common in computeraided design (CAD) systems.

Here we would like to note that the convergence of the strategy is limited if the integrand is not smooth within the bounding box of the domain of integration. If the integrand is p -th order continuous, then the algorithm presented will attain p + 1-th order convergence. Since this strategy is targeted at applications such as immersed boundary methods, where the integrand is smooth within the cut cell, the scheme will converge spectrally.

Contributions. Among this paper's contributions:

- We present algorithms that compute the locations and weights of high-accuracy quadrature rules for integration of arbitrary smooth integrands over trimmed parametric surfaces and over volumes bounded by such surfaces.
- A claimed difficulty of our type of approach in the literature is that it requires the computation of antiderivatives. We show that these are available to high accuracy numerically, easy to calculate for a broad class of useful integrands, and far more efficient than symbolic antiderivatives (see Appendix B).
- We compare our proposed quadrature schemes to three other commonly-used quadrature schemes in the literature that represent the state of the art for integrating over trimmed surfaces or regions bounded by trimmed surface patches. We demonstrate that the proposed algorithms produce quadrature rules which are significantly more efficient in terms of

Figure 3: A graphical outline of our algorithm. See Section 2 for an explanation of the notation. a) The integration region is given in terms of its boundaries, which are composed of trimmed surface patches. b) In the case of volume integration, Stokes theorem transforms the integral onto each of the surface patches. In the case of surface integration, the integral is already over the surface patches. c) Each of the surface integrals are then transformed into integrals over the parametric space of the patches, where trimming curves are given (either approximately or exactly) parametrically. d) Green's theorem is applied to transform the parametric area integral onto the trimming curves. e) Integration points are scattered along each trimming curve. f) Numerical Green's theorem is used to scatter quadrature points within the parametric area of interest. g) The parametric quadrature points are mapped back onto the surface. h) Finally, in the case of volume integration, numerical Stokes theorem is used to scatter quadrature points within the original volume.

<!-- image -->

the number of quadrature points needed to achieve a given integration accuracy.

Outline. The remainder of the paper is organized as follows. In Section 2, we introduce the new algorithms for producing highaccuracy quadrature rules for trimmed parametric surfaces and regions bounded by such surfaces. In Section 3, we present numerical results for integrating two integrands over four regions and compare to three other methods used in the literature. In the supplement, we present comparisons for four additional integrands. We conclude in Section 4 with a discussion of the results and ideas for future work.

## 2. Novel Quadrature Scheme

In this section, we describe our scheme for producing quadrature rules over trimmed/untrimmed surfaces and volumes. We first show how to compute surface integrals over a set of trimmed surface patches using our scheme. Then, we show how numerical antidiffentiation can be used to produce quadrature rules for computing volume integrals over a region bounded by a closed union of trimmed/untrimmed surface patches. An example of our algorithm for producing a quadrature rule is shown in Figure 3.

## 2.1. Surface Integral Case

Formally, we consider the problem of finding an appropriate high-accuracy quadrature rule for approximating the surface integral of an arbitrary function f ( x , y , z ) over a surface S ∈ R 3 defined as the union of ns trimmed parametric surface patches, S = ∪ ns i = 1 S ′ i :

<!-- formula-not-decoded -->

Each trimmed surface patch S ′ i is a subset S ′ i ⊆ Si of a corresponding untrimmed tensor-product rational surface patch Si of bi-degree ( m , n ) :

<!-- formula-not-decoded -->

where { x i j , k , y i j , k , z i j , k } ( m , n ) ( j = 0 , k = 0 ) are the control points and { w i j , k } ( m , n ) ( j = 0 , k = 0 ) are the control weights of the i -th boundary patch. In this work, the surface patches are defined in terms of the Bernstein-B´ ezier basis,

<!-- formula-not-decoded -->

Note that another basis or triangular patches could also be used with minimal modifications, but we focus for clarity on the Bernstein-B´ ezier tensor-product patch case. Note also that NURBS surfaces can be easily decomposed into rational tensorproduct Bernstein-B´ ezier patches using B´ ezier-extraction [45], so the algorithm could also be used with minimal modifications for NURBS and related bases (T-splines [46], U-splines [47], hierarchical B-splines [48], etc.).

For each component surface patch, Si , there is a set of associated (possibly disconnected) implicitly-defined trimming curves { ci , k } mi k = 1 in the surface patch's parent parametric space, R = [ 0 , 1 ] × [ 0 , 1 ] , such that ci , k ⊂ R . These trimming curves can be formed by removing the part of Si on one side of a set of trimming surface patches { Si k } mi k = 1 , or they may be given a priori . Typically included within the set { ci , k } mi k = 1 are some sections of the original boundary of R . In the parametric space of Si , the set of trimming curves { ci , k } mi k = 1 splits R into an 'interior' (possibly disconnected) region R ′ i and another (possibly disconnected) region R \ R ′ i . The boundary of the interior region R ′ i is composed of the trimming curves { ci , k } mi k = 1 . The interior region R ′ i maps through the parametric mapping to S ′ i ,

<!-- formula-not-decoded -->

See Figure 3 for an example with notation labeled.

In many cases of interest, the trimming curves ci , k can be represented exactly as a set of parametric curves in the parametric space of Si . In particular, if ci , k is formed by intersecting a planar surface patch Si and a polynomial (resp. rational) surface patch of bi-degree ( m , n ) , then the trimming curve ci , k will be a polynomial (resp. rational) parametric curve of maximum degree mn . In the case that the curves ci , k can not be represented exactly as parametric curves in the space of Si , we assume that the trimming curves ci , k are well-approximated by some set of ni , k pre-computed degree di , k approximate polynomial or rational parametric curves { c j i , k } n i , k j = 1 given in the Bernstein-B´ ezier basis and defined in the space R such that ∪ n i , k j = 1 c j i , k ≈ ci , k :

<!-- formula-not-decoded -->

where { ( u j i , k ,ℓ , v j i , k ,ℓ } d i , k ℓ = 0 are the control points and { w j i , k ,ℓ } d i , k ℓ = 0 are the control weights of the j -th approximation curve segment. Our strategy for approximating the trimming curves is given in Appendix A and a review of strategies for forming this approximation can be found in [7]. If the ci , k curves can be represented exactly using parametric rational curves, then we simply set ni , k = 1 and use the exact description c 1 i , k = ci , k . The boundary of the interior region ¶ R ′ i is then represented, exactly or approximately, by the curves { c j i , k } n i , k j = 1 .

Our algorithm for surface integrals relies on applying numerical Green's theorem in the parametric space R ′ i of each trimmed patch to transform each surface integral over S ′ i into a sum of line integrals along each approximating boundary curve segment c j i , k .

## 2.1.1. Change of Variables to Parent Coordinates

The algorithm for surface integrals proceeds by considering the integral over each component surface patch individually:

<!-- formula-not-decoded -->

Then, we perform a change of variables to rewrite each of the surface integrals as area integrals in the parametric space of Si :

<!-- formula-not-decoded -->

where ⃗ ni ( u , v ) is the normal vector to Si at the parametric coordinates ( u , v ) . Note that this can be calculated accurately and efficiently for Bernstein-B´ ezier surface patches using de Casteljau's algorithm [49].

## 2.1.2. Application of Green's Theorem

Once we have moved the problem into the parametric space of Si , we need a highly accurate quadrature rule for integrating an arbitrary function g ( u , v ) over the region R ′ i . See [39] for a more detailed derivation of the method we use and a brief survey of other methods. For an arbitrary function g ( u , v ) , by Green's theorem, we have

<!-- formula-not-decoded -->

where we take the v antiderivative,

<!-- formula-not-decoded -->

for some appropriately chosen constant Pv . Note that the antiderivative can be taken in an arbitrary direction, but we choose the v direction for simplicity of exposition. See Section 2.3.1 for a discussion of the choice of antiderivative direction and constant Pv . The choice of antiderivative direction is also discussed briefly in [37]. From this point forward through the rest of Section 2.1, we leave off subscript i , k and superscript j for clarity of exposition. The remaining steps are applied to each of the approximating curves which contribute to the sum in Equation (3).

Since the approximated trimming curves segments are given parametrically, each of the integrals in the sum on the right hand side of Equation (3) can be evaluated in its respective parametric space:

<!-- formula-not-decoded -->

Weagain remind the reader that we have suppressed the subscripts and superscripts on the c , u ( s ) , and v ( s ) . The first term in the product can be evaluated with numerical antidifferentiation using a 1D quadrature rule, such as Gaussian quadrature:

<!-- formula-not-decoded -->

Since we take the parametric mapping ( u ( s ) , v ( s )) to be defined in terms of the Bernstein-B´ ezier basis, the second term in the product can be evaluated stably to machine precision using de Casteljau's algorithm.

## 2.1.3. Final Quadrature Rule for Single Surface Patch

Using a quadrature rule for Equation (5) yields

<!-- formula-not-decoded -->

where { s m } mq m = 1 and { wm } mq m = 1 are the abcissae and weights of a 1D quadrature rule, respectively, with locations on the interval [ 0 , 1 ] . Once these quadrature points and weights have been determined, the antiderivative can also be evaluated using a quadrature rule:

<!-- formula-not-decoded -->

where the { xh } nq h = 1 and { w h } nq h = 1 are again the abcissae and weights of a 1D quadrature rule, respectively, with the locations xh here spread between Pv and v ( s m ) , where v ( s m ) will be different for each approximate trimming curve c j i , k and for each quadrature point v ( s m ) . Note also that the number of quadrature points may be different for each of these antiderivative quadrature rules, but for clarity we use nq quadrature points for all of them. Plugging these quadrature points back in and back-propagating, the approximate form of Equation (5) becomes

<!-- formula-not-decoded -->

The quadrature rule for each component of Green's theorem in Equation (3) (again noting that we have suppressed subscripts and superscripts referring to i , j , k ) therefore has formula

<!-- formula-not-decoded -->

where the quadrature weights are w ′ m , h = wm w h du ( s m ) ds and the quadrature points are ( u ( s m ) , xh ) . In our implementation we use an equal number of quadrature points for each component quadrature rule nq = mq , but in general they need not be equal.

For each surface patch, we must then multiply the quadrature weights by the normal vector and plug in the quadrature points to the defining parametric equations to convert the area quadrature rule for into a surface quadrature rule,

<!-- formula-not-decoded -->

The quadrature points { ( u ( s m ) , xh ) } and weights { w ′ m , h } correspond to evaluating an area integral corresponding to the right hand side of Eq. (2), whereas the quadrature points { ( x m , h , y m , h , z m , h ) } and weights { w m , h } correspond to evaluating surface integrals corresponding to the left hand side of Eq. (2).

The quadrature rules for all of the approximated trimming curves are then combined to produce a rule for the trimmed surface patch. Here the explicit dependence on i , k , and j has been included again for clarity:

<!-- formula-not-decoded -->

where { ( x j i , k , m , h , y j i , k , m , h , z j i , k , m , h ) } are the quadrature points and { w j i , k , m , h } are the quadrature weights.

## 2.1.4. Final Quadrature Rule for Full Surface

Finally, in order to produce a quadrature rule for the entire surface S , each of the individual quadrature rules is combined into a full quadrature rule for the original surface:

<!-- formula-not-decoded -->

where we remind the reader that ns is the number of surface patches, mi is the number of trimming curves on patch Si , ni , k is the number of trimming curve approximations for trimming curve ci , k , and mq and nq are the number of quadrature points used for each trimming curve approximation. Here { xi , k , m , n , yi , k , m , n , zi , k , m , n } are the quadrature points and { wi , k , m , n } are the quadrature weights.

## 2.2. Volume Integral Case

With an accurate quadrature scheme for surface integrals in hand, a quadrature scheme for volume integration can be found by applying an additional iteration of the generalized Stokes theorem and numerical antidifferentiation.

Formally, in this section we consider the problem of finding an appropriate quadrature rule for approximating the volume integral of an arbitrary function f ( x , y , z ) over a region V ∈ R 3 bounded by ¶ V = S , where S is defined as the union of ns trimmed parametric surface patches, S = ∪ n i = 1 S ′ i as in the previous section. However, in this section we make the additional assumption that S satisfies the Jordan-Brouwer Separation theorem. In other words, the surface S divides R 3 into an inside region V bounded by S and an unbounded outside region R 3 \ V . We note that in applications, the separation condition is never formally satisfied, since machine precision causes non-water-tightness [7]. However, our method is robust to small gaps in models, since we transform integrals onto trimming curves on each surface patch. We wish to compute the integral

<!-- formula-not-decoded -->

for arbitrary functions f ( x , y , z ) . We first use the generalized Stokes theorem to transform this volume integral into a sum of surface integrals. For an arbitrary function f ( x , y , z ) , we have

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

for some appropriately chosen constant Pz . We discuss the choice of Pz in Section 2.3.1. Note that the antiderivative can be taken in any direction, but we choose the z -direction for simplicity of exposition. Because we evaluate the antiderivative in the z -direction, the components of ⃗ dS in the x - and y -directions are zeroed out in the dot product. Equation (6) simplifies to the integral of a scalar-valued function,

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

and the normal vector factor ⃗ ni in the change of variables to parent coordinates in Eq. (2) can be replaced with the z -coordinate of the normal vector ⃗ ni z . Each of the integrals in the sum in Equation (8) can be evaluated to high-precision using a quadrature rule generated with the procedure given in Section 2.1, with the above caveat:

<!-- formula-not-decoded -->

Here we have combined the indices k , j , m , h into one index s , where n s = mini , kmqnq . By evaluating the antiderivative numerically using a quadrature rule, we obtain a quadrature rule for each surface patch's contribution to Stokes theorem:

<!-- formula-not-decoded -->

The final quadrature rule for the volume case is simply a sum of the contributions from each surface patch:

<!-- formula-not-decoded -->

where { xi , s , yi , s , zi , s , y } are the quadrature points and { wi , s w y } are the quadrature weights in the final quadrature rule.

## 2.3. Implementation Details

Some of the steps of the algorithms above require some choices about implementation which affect the final quadrature scheme, including the choice of antidifferentiation constants. In addition, the quadrature scheme can be simplified if the surface patch of interest is untrimmed. We discuss such implementation details below.

## 2.3.1. Choice of Antidifferentiation Constants

In Sections 2.2 and 2.1.2 of the algorithm given above, the antidifferentiation constants Pz and Pv must be chosen. These antidifferentiation constants should be chosen to be close to the region of integration to ensure that the quadrature points lie close to the domain of integration. In our implementation of the algorithm we choose Pz to be equal to the smallest z -value of all of the control points of the surface patches,

<!-- formula-not-decoded -->

This ensures that all of the final quadrature points will lie in the bounding box of the object of interest.

For the constant Pv , we choose to use Pv = 0, since we know a priori that all of the trimming curves lie within the original parametric surface patch, whose domain is [ 0 , 1 ] × [ 0 , 1 ] , and this will ensure that all of the surface quadrature points will lie on the original untrimmed parametric surface patch.

## 2.3.2. Simplification for Untrimmed Surface Patches

If the surface patch of interest is untrimmed, then the algorithm in Section 2.1 can be simplified by using a tensor-product quadrature rule for the whole surface patch instead of transforming the surface integral into line integrals and using a quadrature rule on each line integral. If there are n quadrature points in the chosen 1D quadrature rule, a tensor-product quadrature has n 2 points, whereas a naive implementation of our strategy produces a quadrature with 4 n 2 quadrature points and the same accuracy, including n 2 for each of the boundary curves of the surface patch. We note that the left and right curves have geometric correction factors of du ds = 0 and the bottom quadrature rule has weights w m = 0, since the length of the interval in the antiderivative (cf. Eq. (4)) is v ( s m ) -Cv = 0. Thus, if we remove the quadrature rules for the left, right, and bottom boundary curves (which contribute nothing to the sum), then our method is equivalent to the tensor-product quadrature rule. In our implementation, we simply use a tensor-product quadrature rule on the untrimmed surface patches and which is equivalent to using our method with the above simplification.

## 3. Results

In this section, we test the algorithms presented in Section 2 to calculate surface integrals and volume integrals of some representative functions over various domains defined by trimmed or untrimmed rational or polynomial parametric surface boundaries and their interiors. We first describe the test cases, test integrands, comparison methods, and refinement strategy. Then, we report results for surface and volume integrals. Finally, we consider how our algorithm performs when the trimming curve approximation is refined.

## 3.1. Test Domains

We investigate the method on four datasets:

- (1) A closed propeller shape with no trimming curves, given by 5,136 rational bicubic Bernstein-B´ ezier surface patches (see Figure 4),
- (2) A closed bike frame shape given by 2,460 rational bicubic Bernstein-B´ ezier surface patches and 7 planar trimming patches, creating a total of 64 trimming curves (see Figure 5),
- (3) A non-closed trimmed subset of a sphere shape given as 3 rational biquintic Bernstein-B´ ezier surface patches with 4 trimming surfaces that remove a cookie-cutter pattern out of the shape, creating a total of 6 trimming curves on the original surface (see Figure 6), and
- (4) The closed Utah teapot, a classic example from the computer graphics community composed of 32 polynomial bicubic surface patches and one planar patch, creating a total of 18 trimming curves (see Figure 7).

In the case of the propeller, there are no trimming curves, so for surface integration we use the simplification given in Section 2.3.1, while for volume integrals, the primary addition is the numerical Stokes step of the volume integration algorithm given in Section 2.2. It is important to note that for this text case, the surface quadrature rule for each patch therefore reduces to classical tensor-product Gaussian quadrature, and shows equivalent errors. Since there is no geometric approximation error, we expect exponential convergence under quadrature rule refinement. We discuss our approach to quadrature rule refinement below.

In the case of the bike frame, the trimming curves can be expressed as polynomial parametric curves in the parametric space of the planar patches, and none of the higher-order surface patches are trimmed. Therefore, no trimming curve approximation is necessary. We again expect exponential convergence as there is no geometric approximation error.

In the case of the trimmed sphere shape, the trimming curves are approximated in parameter space using cubic polynomial parametric curves according to the procedure given in Appendix A, since the trimming curves can not be expressed exactly using polynomial or rational parametric curves. Only surface integrals are computed for this example, since it is non-closed.

Finally, in the case of the Utah teapot, the trimming curves are also approximated in parameter space using cubic polynomial parametric trimming curves according to the procedure given in Appendix A, but in this case the shape is closed, so volume integrals are also computed. For the Utah teapot test case, we also investigate the effect of geometric refinement of the trimming curve approximation on the accuracy of the computed integrals.

All experiments and comparisons were implemented in MATLAB and are provided as part of the QuaHOG library [50].

## 3.2. Test Integrands

For the three closed shapes, we calculate both surface integrals over the boundary and volume integrals over the interior. For the non-closed trimmed sphere, we calculate only surface integrals. As test integrands, we used three polynomials of degrees 2, 4, and 6:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and three representative nonpolynomial functions

<!-- formula-not-decoded -->

These integrands were chosen because they represent classes of functions which might typically be integrated in applications: polynomials of various degrees, algebraic functions with square roots, smooth algebraic functions, and functions with poles. Each of these integrands is smooth within the domains of interest and within the bounding boxes of the domains of interest.

In the interest of space, we only include comparison plots for p 3 ( x , y , z ) and f 3 ( x , y , z ) in the main text of this paper; results for the other four integrands with comparisons are provided in the paper's supplementary material.

## 3.3. Quadrature Rule Refinement

For refinement of our quadrature rules, we use simple, nonadaptive strategies. We use Gaussian quadrature for each of the component 1D quadrature rules and set the number of quadrature points described in Section 2 equal to each other (i.e., mq = nq = np for volumes and mq = nq for surfaces) and then increase them to refine the quadrature order, thereby increasing accuracy. We calculate the 'ground truth' value of the integral by increasing the number of quadrature points per trimming curve until the difference between subsequent integration values is smaller than machine precision and, in the case of the trimmed sphere and Utah teapot test cases, by increasing the refinement of the trimming curve approximations until the difference between subsequent integration values with sufficient quadrature points per trimming curve approximation is on the order of machine precision. We perform geometric refinement by increasing the number of cubic trimming curve approximation segments.

## 3.4. Comparison Methods

To show the relative benefit of our methods, we compare with three domain decomposition-based quadrature schemes currently used in the literature. These three strategies represent the state of the art for integrating over trimmed and untrimmed polynomial or rational parametric surfaces and volumes.

- OCTREE. Our implementation of octree integration adaptively refines a Cartesian grid wherever the boundary intersects the background grid and then uses 3 rd -order Gaussian quadrature (8 points) on each leaf of the octree. Refinement is performed by increasing the maximum levels of refinement [17]. Note that the octree integration approach is only used for volume integrals, since it is a volumetric approach.
- LINEAR MESH. We generate a non-adaptive linear simplicial mesh of the geometry using Gmsh [51] which discretizes the geometry by meshing with a specified maximum element size. We then use 3 rd -order tensor-product Gaussian quadrature (8 points for tetrahedra and 4 points for triangles) on each element. The various refinement levels represent independently-generated meshes with different maximum element size [52].
- QUADRATIC MESH. Similarly to the linear meshing case, we generate a non-adaptive quadratic polynomial simplicial mesh of the geometry using Gmsh [51] with a specified maximum element size. We then again use 3 rd -order tensorproduct Gaussian quadrature (8 points for tetrahedra and 4 points for triangles) on each element. The various refinement levels represent independently-generated meshes with different maximum element size [52].

We have not optimized any of the implementations of these algorithms nor have we optimized implementations of our own algorithms, but we expect the broad trends, particularly with respect to differences in convergence rates with respect to number of quadrature points, to hold true in any implementation. In our experiments, using higher-order Gaussian quadrature rules for the comparison strategies did not improve the error much, but yielded a much higher number of quadrature points, whereas using lowerorder rules increased error. This implies that the geometric approximation itself is the primary source of error. We also note that for the comparison methods, we generate completely new meshes and octrees for each data point, a comparison which favors these methods, since typically a complete remeshing isn't performed to refine the geometry.

## 3.5. Results for Surface Integration

For each of the domains, our method significantly outperforms the two meshing comparison methods in terms of number of quadrature points needed to attain a given error, while avoiding the complicated decomposition of the ambient space necessary in these comparison methods. Note that we do not compare to the Octree strategy, since it is a volumetric approach. The results for surface integration of functions p 3 ( x , y , z ) and f 3 ( x , y , z ) over the four test cases are shown in Figure 8 and results for the other four test integrands are included in the supplement.

For the propeller model, our method far outperforms the other three methods, because our method retains geometric exactness, allowing for spectral convergence to the correct integral. When our method attains machine precision, the two meshing methods still have errors of at least 10 -5 or higher, which is a difference in accuracy of more than 10 orders of magnitude. For the bike model, our method also far outperforms the other three methods. Our method is again able to retain geometric exactness, because the trimming curves in this case are expressed exactly in terms of polynomial parametric curves, which need not be approximated. This again allows our method to attain spectral convergence to the correct integral. In this case, our method at the best point on the plot attains accuracy improvements of 12 orders of magnitude. Our method also significantly outperforms the comparison methods at lower levels of accuracy for both of these test cases.

The trimmed sphere and Utah teapot models are more nuanced, because they contain trimming curves which must be approximated. Our trimming curve approximation strategy is described in Appendix A. When computing surface integrals over the trimmed sphere model, our method is able to outperform the meshing strategies without requiring any meshing. In this case our method quickly converges to machine precision. For the Utah teapot test case our method still far outperforms the comparison methods, and converges to the geometric approximation error as expected, which is about 10 -8 for this case.

## 3.6. Results for Volume Integration

Our method also significantly outperforms the two comparison methods in terms of number of quadrature points needed to attain a given error for volume integration. The results for volume integration of functions p 3 ( x , y , z ) and f 3 ( x , y , z ) over the three closed test cases are shown in Figure 9 and results for the other four test integrands are again included in the supplement.

As in the surface integration case, our method far outperforms all three comparison methods for the propeller and bike frame models, because no geometric approximation is necessary, allowing for spectral convergence to the correct integral. For the propeller model our method outperforms the comparison methods by nearly 10 -12 at the best point on the plot and is more accurate than the comparison methods with any number of quadrature points. For the bike frame model our method also far outperforms the other three methods at high levels of accuracy and is only outperformed by adaptive octree integration at very low accuracies. For the Utah teapot test case we observe similar behavior. Our method outperforms the other methods when high accuracy is needed.

Weexpect that our method would perform much better at lower accuracies if we implemented it adaptively. However, because we have not implemented adaptivity, the adaptive octree strategy is able to outperform it at very low numbers of quadrature points. We defer investigation of an adaptive version of algorithm to another study. Regardless, at high numbers of quadrature points, our method far outperforms the other methods on all of the test cases for high accuracies.

## 3.7. Effect of Trimming Curve Approximation

It is important to note that in the trimmed sphere and Utah teapot test cases, our method achieves spectral convergence only up to the point when geometric approximation error dominates total error. In the case of surface integration over the trimmed sphere case, the chosen trimming curve approximation gives a geometric approximation error less than machine precision, so increasing the integration order reduces the integration error to below machine precision. For surface integration over the Utah teapot test case, the convergence curve flatlines near 10 -6 . For volume integration, it flatlines near 10 -9 . This begs the question, what is the behavior of the error under more refined geometric approximations of the trimming curves?

After some experimentation, we found that the convergence of the integration error with respect to geometric refinement is very fast. When cubic trimming curve approximations are used (as is the case in this study), the convergence rate is nearly 4th order for volume integration over the Utah teapot test case with respect to total number of trimming curve segments, as shown in Figure 10. The order of convergence is, in general, ( di , k + 1 ) , where di , k is the order of the trimming curve approximations. If a fixed number of quadrature points is used for each trimming curve segment under geometric refinement, the convergence rate is also ( r + 1 ) -th order with respect to the total number of quadrature points for r -th degree polynomial trimming curve approximations. Because this is a volume integral in three-dimensional space, the order of convergence with cubic trimming curves could be considered to be 12th order with respect to the number of quadrature points per dimension. This leads us to conclude that if high-precision is needed and the accuracy attained by increasing integration order is not enough, one step of geometric refinement only on the curves (without complicated mesh-refinement) can improve error drastically.

Moreover, for any particular geometric approximation, convergence up to the geometric approximation error is spectral, as shown in Figure 10. An optimal scheme would switch between using a coarser geometric approximation when lower accuracy is required and finer geometric approximations when higher accuracy is required (i.e., use the bottom-left-most point in each convergence curve of Figure 10). For the trimmed sphere and Utah teapot test cases in Figures 8 and 9, we have simply used a preset geometric approximation and increased the integration order to refine the rule. We leave exploration of the combined geometric and integration order refinement to a future work.

The Utah teapot example shows that the method in question is robust to gaps inherent in B-rep models. Many of the patches are trimmed and these patches do not coincide exactly. However, adjacency information is irrelevant to our algorithm, so the presence of gaps between patches is effectively ignored. These gaps introduce error only in the form of the geometric approximation error already mentioned. Regardless of how large these gaps are, the algorithm itself will function robustly, unlike algorithms based on 3D meshing, octree decomposition, or in/out queries.

## 4. Discussion

In this work, we have presented a high-order, mesh-free quadrature scheme for integrating over trimmed and untrimmed parametric surfaces and volumes. We demonstrated on four shapes and six integrands (three polynomial, three nonpolynomial) that the integration scheme converges spectrally when the surfaces are untrimmed or when the trimming curves can be expressed as loworder polynomial or rational parametric curves. On trimmed surfaces without exact parametric polynomial or rational trimming curves, we showed that the integration scheme attains spectral convergence up to geometric approximation error and high-order convergence when the geometric approximation is refined.

We compared to three different integration schemes: (1) adaptive octree integration, (2) linear meshing of the geometry, and (3) quadratic meshing of the geometry, and showed that in comparison to these methods, our method achieves higher accuracy with less quadrature points. We observed the same behavior of our method on simpler shapes that may more commonly appear in applications such as fictitious domain methods, such as the shape in Figure 1.

<!-- image -->

Figure 4: A depiction of the propeller model, which has no trimming curves and 5,136 rational bicubic Bernstein-B´ ezier patches. (Left) An opaque depiction. Colors represent different surface patches of the object. (Right) A partially translucent depiction.

<!-- image -->

Figure 5: A depiction of the bike frame model, which has seven trimming curves, including 2,460 rational bicubic Bernstein-B´ ezier patches and seven bilinear BernsteinB´ ezier planar patches. (Left) An opaque depiction. Colors represent different surface patches of the object. (Right) A partially translucent depiction with trimming curves highlighted in black.

Figure 6: A depiction of the trimmed sphere model, which has six trimming curves, including three rational biquintic Bernstein-B´ ezier patches. (Left) An opaque depiction. Colors represent different surface patches of the object. (Right) A partially translucent depiction with trimming curves highlighted in black.

<!-- image -->

Figure 7: A depiction of the Utah teapot frame model, which has four trimming curves, including 32 polynomial bicubic Bernstein-B´ ezier patches and one planar patch. (Left) An opaque depiction. Colors represent different surface patches of the object. (Right) A partially translucent depiction with trimming curves highlighted in black.

<!-- image -->

Figure 8: Results of using the algorithm given in Section 2 to calculate the surface integral of various functions over each of the models defined in Section 3.1. For definitions of the test integrands, see Section 3.2. Note that all plots are on a loglog scale.

<!-- image -->

Figure 9: Results of using the algorithm given in Section 2 to calculate the volume integral of various functions over the interior of each of the closed models defined in Section 3.1. For definitions of the test integrands, see Section 3.2. Note that all plots are on a loglog scale.

<!-- image -->

Figure 10: Comparing errors due to trimming curve approximation for volume integrals over the interior of the Utah teapot model. Geometric refinement is performed by increasing the number of cubic approximation curve segments ni , k per trimming curve. In each chart, successive data points within a series (colored lines) correspond to increasing quadrature order. The set of constants ni , k , init is the initial number of trimming curve segments used to approximate each of the trimming curves ci , k . We note that ni , k , init in this plot is less than the number of trimming curves used in Figure 9. The N inside of the O notation refers to the total number of quadrature points. For definitions of the test integrands, see Section 3. Note that all plots are on a loglog scale.

<!-- image -->

We would like to briefly note that the high levels of accuracy seen here may seem unnecessary in practice. This may be true, for instance, for certain geometric processing applications such as computation of geometric moments. However, in applications where many integrations must be performed, such as in the solution of time-dependent partial differential equations or in iterative methods, small errors in integration can quickly dominate overall error. High levels of accuracy are also required in applications such as Lagrangian remap, where conservation error is directly proportional to integration error. In addition, we note that while we have focused on high-accuracy integration for our application, the above methods are generally able to achieve lower accuracy with far fewer quadrature points than comparison methods, so these methods could also be applicable in situations where only lower accuracy is necessary. We posit that one of the primary advantages of the methods presented in this paper are that no surface or volume decomposition is necessary for integration, thereby increasing the robustness and decreasing the complexity of the overall integration procedure.

The quadrature schemes presented here can improve accuracy and efficiency in a variety of analysis paradigms, including immersed boundary methods, Lagrangian remap methods, momentfitting methods, and multi-mesh methods commonly seen in multi-physics applications. Our plans for future work include three primary thrusts: 1) we plan to explore adaptivity of our quadrature scheme by informing the order of quadrature on each trimming curve based on the arc length and curvature of trimming curves, thereby exploring the geometric refinement/integration refinement trade-offs, 2) we plan to investigate spectrally-accurate meshless quadrature schemes which do not have geometric approximation errors, and 3) we also plan to apply these novel high- accuracy quadrature methods to the aforementioned applications.

## Acknowledgments

This work was performed under the auspices of the U.S. Department of Energy by Lawrence Livermore National Laboratory under Contract DE-AC52-07NA27344.

## Appendix A. Trimming curve approximation

For surfaces with trimming curves that can not be represented exactly using parametric or rational parametric curves, such as the model in Figure 1, the trimmed sphere example in Figure 6, and the Utah teapot example in Figure 7, the trimming curves must be approximated. For untrimmed surfaces or surfaces with trimming curves which can be represented exactly, such as the propeller or bike frame models in Figures 4 and 5, this step is not required. In many computer-aided design models and engineering applications, the trimming curves are approximated in a pre-processing step before the analysis is started. In these cases, those approximations can be used as the c j i , k in Section 2.1. In the case that the trimming curves are not given a priori , we must approximate them.

For each Si , we must approximate each of the mi trimming curves { ci , k } mi k = 1 using some set of ni , k polynomial parametric curve segments, { c j i , k } n i , k j = 1 . There is a wide body of literature on producing ordered point sets which lie on trimming curves determined as the intersection of two parametric surfaces and a survey can be found in [7]. Once a point set is found, a high-order curve can be fit to the ordered point set, a problem which has also been studied extensively. In this work, we have used the IRIT polynomial solver [53] to find an ordered, well-spaced point set on an algebraic curve formed as the intersection between two polynomial or rational surfaces. It uses a variety of strategies to achieve this goal, including subdivision strategies and stepping along the curve. Once the ordered point set has been found, we use cubic Lagrange interpolation, with parametric interpolation points spaced based on the physical distance between consecutive points. We also tried using quintic and septic Lagrange interpolation and achieved reasonable results, although the interpolation begins to suffer from the Runge phenomenon as the interpolation order increases, which causes oscillations in high-order polynomial fits to evenly-spaced data.

Table B.2: A comparison of the average pre-processing and evaluation time for computing the 1D antiderivative of 10,000 polynomials of random degree between 1 and 20 with random standard normal coefficients over 10 , 000 intervals with random standard normal boundaries. The strategies are described in Appendix B. Timing is performed in MATLAB.

|                           | ( 1 )                   | ( 2 )                 | ( 3 )            |
|---------------------------|-------------------------|-----------------------|------------------|
| Pre-processing Evaluation | 2 · 10 - 1 1 . 7 · 10 0 | 2 · 10 - 4 8 · 10 - 7 | 0 2 . 4 · 10 - 6 |

## Appendix B. Symbolic vs Numeric Antidifferentiation

In the case of nonpolynomial functions, it is often impossible or very difficult to find symbolic antiderivatives of functions, as evidenced for example by the function f ( x , y , z ) = √ x 2 + y 2 + z 2 . However, for some classes of functions, such as polynomials, it is relatively easy to find symbolic antiderivatives. In some applications where the integrands of interest are polynomials with known coefficients and are integrated many times, it may be more computationally efficient to simply find antiderivatives symbolically as a pre-processing step. However, numeric antidifferentiation can be preferable even if the integrand is known to be polynomial when the coefficients and/or degree of the polynomial are unknown. This is because symbolically integrating such a polynomial can be cumbersome. In fact, even if the coefficients and degree are known, the symbolic antidifferentiation can still be inefficient because of the relative inefficiency of symbolic operations.

To further explore this dichotomy, we produced 10 , 000 polynomials with standard normal distribution of random degrees between 1 and 20. We use three strategies to compute the antiderivatives of these functions: (1) symbolic antidifferentiation using MATLAB's built-in int function, (2) antidifferentiation using a simple numerical implementation of the power rule, and (3) numerical antidifferentiation using 10 th order Gaussian quadrature. A table with results for pre-processing (i.e., antidifferentiation) of the function along with evaluation (i.e., evaluating at the integration bounds) averaged over 10 , 000 random polynomials and 10 , 000 random bounds is shown in Table B.2. As can be seen, symbolic antidifferentiation is far less efficient than a numerical antidifferentiation strategy. The numerical power rule implementation is slightly faster in terms of evaluation time than numerical antidifferentiation using high-order quadrature, but it is far less efficient than Gaussian quadrature in terms of pre-processing time. Importantly, the power rule strategy can only be used in a very specific set of circumstances-on polynomials in the power basis with known coefficients and exponents.

## References

- [1] M. Majeed, F. Cirak, Isogeometric analysis using manifold-based smooth basis functions, Computer Methods in Applied Mechanics and Engineering 316 (2017) 547-567.
- [2] X. Wei, Y. J. Zhang, T. J. Hughes, Truncated hierarchical tricubic c 0 spline construction on unstructured hexahedral meshes for isogeometric analysis applications, Computers &amp; Mathematics with Applications 74 (9) (2017) 2203-2220.
- [3] X. Wei, Y. J. Zhang, D. Toshniwal, H. Speleers, X. Li, C. Manni, J. A. Evans, T. J. Hughes, Blended B-spline construction on unstructured quadrilateral and hexahedral meshes with optimal convergence rates in isogeometric analysis, Computer Methods in Applied Mechanics and Engineering 341 (2018) 609-639.
- [4] H. Casquero, X. Wei, D. Toshniwal, A. Li, T. J. Hughes, J. Kiendl, Y. J. Zhang, Seamless integration of design and Kirchhoff-Love shell analysis using analysis-suitable unstructured T-splines, Computer Methods in Applied Mechanics and Engineering 360 (2020) 112765.
- [5] X. Wei, X. Li, Y. J. Zhang, T. J. Hughes, Tuned hybrid nonuniform subdivision surfaces with optimal convergence rates, International Journal for Numerical Methods in Engineering 122 (9) (2021) 2117-2144.
- [6] X. Wei, X. Li, K. Qian, T. J. Hughes, Y. J. Zhang, H. Casquero, Analysissuitable unstructured T-splines: Multiple extraordinary points per face, arXiv preprint arXiv:2103.05726 (2021).
- [7] B. Marussig, T. J. R. Hughes, A review of trimming in isogeometric analysis: Challenges, data exchange and simulation aspects, Archives of computational methods in engineering 25 (4) (2018) 1059-1127.
- [8] C. Hafner, C. Schumacher, E. Knoop, T. Auzinger, B. Bickel, M. B¨ acher, XCAD: Optimizing CAD models with extended finite elements, ACM Trans. Graph. 38 (6) (Nov. 2019).
- [9] S. Mousavi, H. Xiao, N. Sukumar, Generalized Gaussian quadrature rules on arbitrary polygons, International Journal for Numerical Methods in Engineering 82 (1) (2010) 99-113.
- [10] E. Burman, P. Hansbo, Fictitious domain finite element methods using cut elements: I. A stabilized Lagrange multiplier method, Computer Methods in Applied Mechanics and Engineering 199 (41-44) (2010) 2680-2686.
- [11] R. W. Anderson, V. A. Dobrev, T. V. Kolev, R. N. Rieben, V. Z. Tomov, High-order multi-material ALE hydrodynamics, SIAM Journal on Scientific Computing 40 (1) (2018) B32-B58.
- [12] A. Johansson, B. Kehlet, M. G. Larson, A. Logg, Multimesh finite element methods: Solving PDEs on multiple intersecting meshes, Computer Methods in Applied Mechanics and Engineering 343 (2019) 672-689.
- [13] J. Grandy, Conservative remapping and region overlays by intersecting arbitrary polyhedra, Journal of Computational Physics 148 (2) (1999) 433-466.
- [14] L. Margolin, M. Shashkov, Second-order sign-preserving conservative interpolation (remapping) on general grids, Journal of Computational Physics 184 (1) (2003) 266-298.
- [15] C. S. Peskin, The immersed boundary method, Acta numerica 11 (2002) 479-517.
- [16] D. Kamensky, M.-C. Hsu, D. Schillinger, J. A. Evans, A. Aggarwal, Y. Bazilevs, M. S. Sacks, T. J. R. Hughes, An immersogeometric variational framework for fluid-structure interaction: Application to bioprosthetic heart valves, Computer Methods in Applied Mechanics and Engineering 284 (2015) 1005-1053.
- [17] L. Kudela, N. Zander, T. Bog, S. Kollmannsberger, E. Rank, Efficient and accurate numerical quadrature for immersed boundary methods, Advanced Modeling and Simulation in Engineering Sciences 2 (1) (2015) 10.
- [18] T. J. R. Hughes, A. Reali, G. Sangalli, Efficient quadrature for NURBSbased isogeometric analysis, Computer Methods in Applied Mechanics and Engineering 199 (5-8) (2010) 301-313.
- [19] T. J. R. Hughes, The finite element method: Linear static and dynamic finite element analysis, Courier Corporation, 2012.
- [20] Y. Sudhakar, W. A. Wall, Quadrature schemes for arbitrary convex/concave volumes and integration of weak form in enriched partition of unity methods, Computer Methods in Applied Mechanics and Engineering 258 (2013) 3954.
- [21] V. Thiagarajan, V. Shapiro, Adaptively weighted numerical integration over arbitrary domains, Computers &amp; Mathematics with Applications 67 (9) (2014) 1682-1702.
- [22] S. C. Divi, C. V. Verhoosel, F. Auricchio, A. Reali, E. H. van Brummelen, Error-estimate-based adaptive integration for immersed isogeometric analysis, Computers &amp; Mathematics with Applications (2020).
- [23] G. H. Golub, J. H. Welsch, Calculation of Gauss quadrature rules, Mathematics of computation 23 (106) (1969) 221-230.

- [24] P. Keast, Moderate-degree tetrahedral quadrature formulas, Computer methods in applied mechanics and engineering 55 (3) (1986) 339-348.
- [25] A. Krishnamurthy, S. McMains, Accurate moment computation using the GPU, in: Proceedings of the 14th ACM Symposium on Solid and Physical Modeling, ACM, 2010, pp. 81-90.
- [26] X. Roca, A. Gargallo-Peir´ o, J. Sarrate, Defining quality measures for highorder planar triangles and curved mesh generation, in: Proceedings of the 20th International Meshing Roundtable, Springer, 2011, pp. 365-383.
- [27] S. Sherwin, J. Peir´ o, Mesh generation in curvilinear domains using highorder elements, International Journal for Numerical Methods in Engineering 53 (1) (2002) 207-223.
- [28] L. Engvall, J. A. Evans, Isogeometric triangular Bernstein-B´ ezier discretizations: Automatic mesh generation and geometrically exact finite element analysis, Computer Methods in Applied Mechanics and Engineering 304 (2016) 378-407.
- [29] L. Engvall, J. A. Evans, Isogeometric unstructured tetrahedral and mixedelement Bernstein-B´ ezier discretizations, Computer Methods in Applied Mechanics and Engineering 319 (2017) 83-123.
- [30] L. Engvall, J. A. Evans, Mesh quality metrics for isogeometric BernsteinB´ ezier discretizations, Computer Methods in Applied Mechanics and Engineering 371 (2020) 113305.
- [31] P. Antolin, A. Buffa, M. Martinelli, Isogeometric analysis on V-reps: First results, Computer Methods in Applied Mechanics and Engineering 355 (2019) 976-1002.
- [32] E. B. Chin, J. B. Lasserre, N. Sukumar, Numerical integration of homogeneous functions on convex and nonconvex polygons and polyhedra, Computational Mechanics 56 (6) (2015) 967-981.
- [33] R. Saye, High-order quadrature methods for implicitly defined surfaces and volumes in hyperrectangles, SIAM Journal on Scientific Computing 37 (2) (2015) A993-A1019.
- [34] M. A. Olshanskii, D. Safin, Numerical integration over implicitly defined domains for higher order unfitted finite element methods, Lobachevskii Journal of Mathematics 37 (5) (2016) 582-596.
- [35] S. Sheynin, A. Tuzikov, Moment computation for objects with spline curve boundary, IEEE Transactions on Pattern Analysis and Machine Intelligence 25 (10) (2003) 1317-1322.
- [36] B. Li, The moment calculation of polyhedra, Pattern Recognition 26 (8) (1993) 1229-1233.
- [37] A. Sommariva, M. Vianello, Gauss-Green cubature and moment computation over arbitrary geometries, Journal of Computational and Applied Mathematics 231 (2) (2009) 886-896.
- [38] T. Jonsson, M. G. Larson, K. Larsson, Cut finite element methods for elliptic problems on multipatch parametric surfaces, Computer Methods in Applied Mechanics and Engineering 324 (2017) 366-394.
- [39] D. Gunderman, K. Weiss, J. A. Evans, Spectral mesh-free quadrature for planar regions bounded by rational parametric curves, Computer Aided Design 130 (102944) (2021).
- [40] F. Scholz, A. Mantzaflaris, B. J¨ uttler, First order error correction for trimmed quadrature in isogeometric analysis, in: Chemnitz Fine Element Symposium, Springer, 2017, pp. 297-321.
- [41] F. Scholz, B. J¨ uttler, Numerical integration on trimmed three-dimensional domains (2019).
- [42] T. W. Sederberg, D. C. Anderson, R. N. Goldman, Implicit representation of parametric curves and surfaces, Computer Vision, Graphics, and Image Processing 28 (1) (1984) 72-84.
- [43] M. Joulaian, S. Hubrich, A. D¨ uster, Numerical integration of discontinuities on arbitrary domains based on moment fitting, Computational Mechanics 57 (6) (2016) 979-999.
- [44] S. Hubrich, P. Di Stolfo, L. Kudela, S. Kollmannsberger, E. Rank, A. Schr¨ oder, A. D¨ uster, Numerical integration of discontinuous functions: Moment fitting and smart octree, Computational Mechanics 60 (5) (2017) 863-881.
- [45] M. J. Borden, M. A. Scott, J. A. Evans, T. J. R. Hughes, Isogeometric finite element data structures based on B´ ezier extraction of NURBS, International Journal for Numerical Methods in Engineering 87 (1-5) (2011) 15-47.
- [46] Y. Bazilevs, V. M. Calo, J. A. Cottrell, J. A. Evans, T. J. R. Hughes, S. Lipton, M. A. Scott, T. W. Sederberg, Isogeometric analysis using T-splines, Computer Methods in Applied Mechanics and Engineering 199 (5-8) (2010) 229-263.
- [47] D. Thomas, L. Engvall, S. Schmidt, K. Tew, M. Scott, U-splines: Splines over unstructured meshes, Preprint, https://coreform. com/usplines (2018).
- [48] C. Giannelli, B. J¨ uttler, H. Speleers, Thb-splines: The truncated basis for hierarchical splines, Computer Aided Geometric Design 29 (7) (2012) 485498.
- [49] G. Farin, J. Hoschek, M.-S. Kim, Handbook of Computer Aided Geometric Design, Elsevier, 2002.
- [50] [D. Gunderman, QuaHOG: Quadrature for High-Order Geometries (2020). URL https://github.com/davidgunderman/QuaHOG](https://github.com/davidgunderman/QuaHOG)
- [51] C. Geuzaine, J.-F. Remacle, Gmsh: A 3-d finite element mesh generator with built-in pre-and post-processing facilities, International Journal for Numerical Methods in Engineering 79 (11) (2009) 1309-1331.
- [52] D. Engwirda, Locally optimal Delaunay-refinement and optimisation-based mesh generation (2014).
- [53] G. Elber, M.-S. Kim, Geometric constraint solver using multivariate rational spline functions, in: Proceedings of the sixth ACM symposium on Solid modeling and applications, 2001, pp. 1-10.