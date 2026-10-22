## A Note on Ribbon-based Biharmonic Surface Patches

## Márton Vaitkus a

a Budapest University of Technology and Economics,

2 Magyar Tudósok Körútja, Budapest, Hungary

E-mail:

vaitkus@iit.bme.hu

Abstract: In this short note we describe a simple adaptation of biharmonic surfaces to interpolate boundary cross-derivatives given in ribbon form, and compare with the recently proposed Generalized B-spline patches.

|   Contents | Contents                                              |   Contents |
|------------|-------------------------------------------------------|------------|
|          1 | Introduction                                          |          2 |
|          2 | Biharmonic interpolation and its mixed discretization |          2 |
|          3 | Ribbon-based biharmonic patches                       |          4 |
|          4 | Discussion                                            |          4 |

## 1 Introduction

Creating multi-sided general-topology surface patches that connect to their neighbourhood with geometric continuity [1] is a widely studied problem in computer-aided geometric design (CAGD). A large variety of methods for creating multi-sided surfaces have been proposed, see e.g. [2] and [3] for overviews. While some progress has been made regarding mutlti-sided implicit surfaces [4], in practice parametric representations are generally preferred. Recently developed methods [3, 5, 6] even allow for surfaces to be defined over domains that are bounded by arbitrary curves in the plane and are possibly multiply connected. Such surfaces are generally defined using boundary ribbons , i.e. bi-parametric surfaces defining positional and cross-derivative information to be interpolated along the boundaries. For transfinite interpolation methods [7-10] ribbons can be defined as general (possibly procedurally generated) bi-parametric surfaces. Other methods are based on control points [3, 5, 6, 11-14], in which case ribbons must be piecewise-rational surfaces in Bézier or B-spline form. As an example of the state-of-the-art, Generalized B-Spline (GBS) patches [3] employ local re-parameterizations to map a general curved domain onto the domain of the B-spline ribbons, and pull back blending functions naturally associated with the ribbons onto the common domain, and then - after applying suitable corrections to enforce interpolation of boundary derivatives and partition of unity - each point of the GBS patch can be evaluated as a convex combination of the ribbon control points.

As an alternative to such direct methods, surfaces could also be defined as solutions of functional optimization problems, or partial differential equations that express some measure of fairness or quality for the shape (such as integrated curvature or curvature variation). Solving nonlinear optimization problems or PDEs is generally only possible via computationally expensive iterative methods [15-18]. A more efficient alternative is to use quadratic energies, or linear PDEs which lead to simple linear systems of equations to solve [19-21]. Generally, such linear variational surfaces [22] are defined in terms of normal derivatives along their boundary. In this short paper, we describe how to relate biharmonic surfaces as proposed by [20] to ribbon-based approaches and compare the resulting generalized Hermite interpolants to Generalized B-spline surfaces.

## 2 Biharmonic interpolation and its mixed discretization

In this chapter, we follow the derivations of [20]. Over a 2D domain Ω ⊂ R 2 , a generalization of (cubic) Hermite interpolation called biharmonic interpolation can be formulated as the constrained optimization of the Laplacian energy (closely related - but not always identical - to the so-called thin-plate energy [23]):

<!-- formula-not-decoded -->

where n ( x, y ) is the inwards normal at the boundary. We choose to discretize this problem using (mixed) finite elements, but note that alternatives such as boundary element [24] and Monte Carlo [25] methods could also be used to compute an approximate solution to the biharmonic problem.

Proceeding with a mixed formulation and introducing the auxiliary variable v = ∆ u and Lagrange multiplier λ , we get the mixed Lagrangian:

<!-- formula-not-decoded -->

Applying Green's formula we derive the following saddle point problem:

<!-- formula-not-decoded -->

Discretizing Ω with a triangulation, and approximating the unknowns as piecewiselinear functions

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and differentiating with respect to the unknown coefficients we get a linear system

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

is the well-known cotangent Laplace matrix,

<!-- formula-not-decoded -->

is the mass matrix, which is often approximated by a diagonal ("lumped mass") matrix containing the sum of each row (equal to the Voronoi area around each vertex). The right hand side of the equation is defined as b = -L Ω ,∂ Ω u 0 -Nd 0 where L Ω ,∂ Ω is the Laplacian matrix restricted to the boundary, N ij = ∫ ∂ Ω φ i φ j dxdy is the boundary "area" (length) associated to a given vertex, and u 0 , d 0 are sampled values of the boundary conditions.

The reduced form

where

<!-- formula-not-decoded -->

is similar to an alternative discretization of the biharmonic equation [19]. The vertex values can be computed as a linear combination of the boundary values and derivatives:

<!-- formula-not-decoded -->

where the columns of the matrices H 0 , H 1 can be interpeted as generalized cubic Hermite blending functions.

## 3 Ribbon-based biharmonic patches

In general, we have boundary conditions prescribed in the form of procedural or tensorproduct Bernstein-Bézier or B-spline ribbons associated with our curved domain Ω :

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

In particular, we want to use R ( s, 0) and ∂ R ∂h as our boundary conditions for the biharmonic interpolation. We locally (re)parameterize each ribbon onto the curved domain Ω , using e.g. harmonic interpolation [3, 26]

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

As a consequence, the normal derivatives must be computed by also taking into account the gradient of the local parameterizations:

<!-- formula-not-decoded -->

with e.g. ∂s i ∂ n = ∇ s i · n , where ∇ s i = [ ∂s i ∂u ∂s i ∂v ] can be approximated over the domain mesh with any of the usual methods [27]. Note that unlike other ribbon-based patches that directly use the parameterization in their definition [3, 10], for biharmonic patches only the boundary derivatives of the reparameterization are utilized, similarly to the usual sufficient conditions for geometric continuity [1].

## 4 Discussion

We compare B-spline ribbon-based biharmonic patches with Generalized B-spline surfaces of [3] in Figure 1 and Figure 2. As expected, biharmonic patches usually (although not always) have a more even curvature distribution than GBS patches, due to their energybased formulation. However, our experience is that biharmonic patches are less responsive to control point movements than more explicit methods such as GBS patches (likely due to the built-in fairness), which might make make it difficult to do fine-grained manual design. This supports earlier claims by [28, 29].

For biharmonic patches, blending functions can be pre-evaluated by substituting Kronecker deltas for the positions and normal derivatives and converting from a Hermite to a Bernstein representation of cross-derivatives. The resulting blending functions are compared with those of GBS patches on Figure 3. As can be seen, the functions are often fairly similar, although biharmonic blending functions can be negative, while GBS blending functions are guaranteed to form a convex partition of unity.

Instead of the presented control point-based approach, one could also interpret ribbons of biharmonic surfaces in a Hermite-like manner as in e.g. [9, 10]. We mention that ribbon-based triharmonic surfaces could also be derived following [20], based on the second derivatives of the reparameterization - although the accurate calculation of these over triangulated domains might be challenging.

Figure 1 : Comparison of vertex blend surfaces. Colors indicate mean curvature.

<!-- image -->

## References

- [1] P. Kiciak, Geometric Continuity of Curves and Surfaces , Synthesis Lectures on Visual Computing, Morgan &amp; Claypool Publishers (2017), 10.1007/978-3-031-02590-7.
- [2] J. Peters, Splines for meshes with irregularities , The SMAI Journal of computational mathematics 5 (2019) 161.
- [3] M. Vaitkus, T. Várady, P. Salvi and Á. Sipos, Multi-sided B-spline surfaces over curved, multi-connected domains , Computer Aided Geometric Design 89 (2021) 102019.

Figure 2 : Comparison of multiply connected surfaces. Colors indicate mean curvature.

<!-- image -->

- [4] Á. Sipos, T. Várady, P. Salvi and M. Vaitkus, Multi-sided implicit surfacing with I-patches , Computers &amp; Graphics 90 (2020) 29.
- [5] T. Várady, P. Salvi, M. Vaitkus and Á. Sipos, Multi-sided Bézier surfaces over curved, multi-connected domains , Computer Aided Geometric Design 78 (2020) 101828.
- [6] F. Martin and U. Reif, Trimmed spline surfaces with accurate boundary control , in Geometric Challenges in Isogeometric Analysis , vol. 49 of Springer INdAM Series , pp. 123-148, 2022, DOI.
- [7] M.A. Sabin, Transfinite surface interpolation , in The Mathematics of Surfaces VI , pp. 517-534, IMA, 1996.
- [8] C. Dyken and M.S. Floater, Transfinite mean value interpolation , Computer Aided Geometric Design 26 (2009) 117.
- [9] T. Várady, A. Rockwood and P. Salvi, Transfinite surface interpolation over irregular n -sided domains , Computer-Aided Design 43 (2011) 1330.

Figure 3 : Comparison of blend function level surfaces. Yellow color indicates negative values.

<!-- image -->

- [10] P. Salvi, T. Várady and A. Rockwood, Ribbon-based transfinite surfaces , Computer Aided Geometric Design 31 (2014) 613.
- [11] T. Várady, Overlap patches: a new scheme for interpolating curve networks with n -sided regions , Computer Aided Geometric Design 8 (1991) 7.
- [12] T. Várady, P. Salvi and G. Karikó, A multi-sided Bézier patch with a simple control structure , Computer Graphics Forum 35 (2016) 307.
- [13] G.J. Hettinga and J. Kosinka, Multisided generalisations of Gregory patches , Computer Aided Geometric Design 62 (2018) 166.
- [14] G.J. Hettinga and J. Kosinka, A multisided C 2 B-spline patch over extraordinary vertices in quadrilateral meshes , Computer-Aided Design 127 (2020) 102855.

- [15] H.P. Moreton and C.H. Séquin, Functional optimization for fair surface design , ACM SIGGRAPH Computer Graphics 26 (1992) 167.
- [16] R. Schneider and L. Kobbelt, Generating fair meshes with G 1 boundary conditions , in Geometric Modeling and Processing 2000 , pp. 251-261, IEEE, 2000, DOI.
- [17] H. Pan, Y. Liu, A. Sheffer, N. Vining, C. Li and W. Wang, Flow aligned surfacing of curve networks , ACM Transactions on Graphics (TOG) 34 (2015) 127.
- [18] Y. Soliman, A. Chern, O. Diamanti, F. Knöppel, U. Pinkall and P. Schröder, Constrained Willmore surfaces , ACM Transactions on Graphics (TOG) 40 (2021) 112.
- [19] M. Botsch and L. Kobbelt, An intuitive framework for real-time freeform modeling , ACM Transactions on Graphics (TOG) 23 (2004) 630.
- [20] A. Jacobson, E. Tosun, O. Sorkine and D. Zorin, Mixed finite elements for variational surface modeling , Computer Graphics Forum 29 (2010) 1565.
- [21] J. Andrews, P. Joshi and N. Carr, A linear variational system for modelling from curves , Computer Graphics Forum 30 (2011) 1850.
- [22] M. Botsch and O. Sorkine, On linear variational surface deformation methods , IEEE Transactions on Visualization and Computer Graphics 14 (2007) 213.
- [23] O. Stein, E. Grinspun, M. Wardetzky and A. Jacobson, Natural boundary conditions for smoothing in geometry processing , ACM Transactions on Graphics (TOG) 37 (2018) 1.
- [24] O. Weber, R. Poranne and C. Gotsman, Biharmonic coordinates , Computer Graphics Forum 31 (2012) 2409.
- [25] R. Sawhney and K. Crane, Monte Carlo geometry processing: a grid-free approach to PDE-based methods on volumetric domains , ACM Transactions on Graphics (TOG) 39 (2020) 123.
- [26] P. Joshi, M. Meyer, T. DeRose, B. Green and T. Sanocki, Harmonic coordinates for character articulation , ACM Transactions on Graphics (TOG) 26 (2007) 71.
- [27] C. Mancinelli, M. Livesu and E. Puppo, A comparison of methods for gradient field estimation on simplicial meshes , Computers &amp; Graphics 80 (2019) 37.
- [28] J. Peters, Smoothness, fairness and the need for better multi-sided patches , in Topics in Algebraic Geometry and Geometric Modeling , vol. 334 of Contemporary Mathematics , pp. 55-64, AMS, 2003, DOI.
- [29] J. Peters and K. Karčiauskas, An introduction to guided and polar surfacing , in 7th International Conference on Mathematical Methods for Curves and Surfaces , vol. 5862 of Lecture Notes in Computer Science , pp. 299-315, 2008, DOI.