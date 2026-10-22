- [17] J. R. Rossignac and A. A. G. Requicha. Constant Radius Blending in Solid Modelling. Computers in Mechanical Engineering, pp 65-73, July 1984.
- [18] K. Vida, R. R. Martin, and T. Varady. A Survey of Blending Methods that use Parametric Surfaces. Computer Aided Design, Vol. 26, No. 5, pp 341-365, May 1994.
- [19] J. Warren. Blending Algebraic Surfaces. ACM Transactions on Graphics, Vol. 8, No. 4, pp 263-278, August 1989.

## References

- [1] R. H. Bartels and I. Hardtke. Speed Adjustment for Key-Frame Interpolation. Graphics Interface 89 Proceedings, pp 14-19, 1989.
- [2] C. L. Bajaj and I. Ihm. Algebraic Surface Design with Hermite Interpolation. ACM Transactions on graphics, Vol 11, No 1, January 1992, pages 61-91.
- [3] A. P. Bien and F. Cheng. A Blending Model for Parametrically Dened Geometric Objects. Symposium on Solid Modeling Foundation and CAD/CAM Applications, 1991, pp. 339347.
- [4] B. K. Choi and S. Y. Ju. Constant Radius Blending in Surface Modeling. Computer Aided Design, Vol. 21, No. 4, pp 213-220, May 1989.
- [5] M. P. DoCarmo. Dierential Geometry of Curves and Surfaces. Prentice-Hall 1976.
- [6] G. Elber. Free Form Surface Analysis using a Hybrid of Symbolic and Numeric Computation. Ph.D. thesis, University of Utah, Computer Science Department, 1992.
- [7] G. Elber and E. Cohen. Error Bounded Variable Distance Oset Operator for Free Form Curves and Surfaces. International Journal of Computational Geometry and Applications, Vol. 1., No. 1, pp 67-78, March 1991.
- [8] G. Farin. Curves and Surfaces for Computer Aided Geometric Design. Academic Press, Inc. Second Edition 1990.
- [9] R. T. Farouki and V. T. Rajan. Algorithms for Polynomials in Bernstein Form. Computer Aided Geometric Design 5, pp 1-26, 1988.
- [10] D. J. Fillip. Blending Parametric Surfaces. ACM Transaction on Graphics, Vol. 8, No. 3, pp 165-173, July 1989.
- [11] J. Hoschek and D. Lasser. Fundamentals of Computer Aided geometric Design. A K Peters, Wellesley, Massachusetts, 1993.
- [12] K. Kim. Blending Parametric Surfaces. M.Sc. thesis, University of Utah, Computer Science Department, 1992.
- [13] R. K. Klass and B. Kuhn. Fillet and Surface Intersections Dened by Rolling Balls. Computer Aided Geometric Design 9, pp 185-193, 1992.
- [14] P. Koparkar. Parametric Blending using Fanout Surfaces. Symposium on Solid Modeling Foundation and CAD/CAM Applications, 1991, pp. 317-327.
- [15] M. Marsden and I. J. Schoenberg. On Variation Diminishing Spline Approximation Methods. Mathematica, Vol. 8(31), No. 1, pp 61-82, 1966.
- [16] K. Morken. Some Identities for Products and Degree Raising of Splines. Constructive Approximation, Vol 7, pp 195-208, 1991.

Figure 8: Blending using two dierent magnitudes of T 2 i ( t ) =   dC ( t ) dt -n ( u ( t ) ;; v ( t ))  .

<!-- image -->

Figure 9: Spout connection to the Utah teapot rounding using T 2 i ( t ) =   dC ( t ) dt -n ( u ( t ) ;; v ( t ))  .

<!-- image -->

Figure 7: Blending using T 5 i ( t ) =  ( S u ( u ( t ) ;; v ( t )) r ( t ) + S v ( u ( t ) ;; v ( t )) s ( t )).

<!-- image -->

method, which is guaranteed to be perpendicular to the rail curves in Euclidean space. Figure 9 shows the original teapot (a), the lleted teapot (b), and the lleted region enhanced (c).

## 4 Conclusions

The symbolic computation method proved to be a powerful tool in creating blend surfaces within machine accuracy and with a geometrically meaningful control. With the ability to symbolically compute and normalize cross tangent curves, the symbolic approach for the construction of blend surfaces can be automated. This approach is unique in that it provides an accurate tangent plane continuity for the entire boundary of the blend surface, as opposed to accurate tangent plane continuity at only a nite set of locations along this domain. Furthermore, once the tangent curves are normalized, results can be interacted with in a geometrically meaningful way.

It is our opinion that the symbolic blend surface construction, as a promising way toward a robust and fast computation of blend surfaces for freeform based models, will nd its way into current solid modeling systems.

## 5 Acknowledgment

The authors are grateful to Elaine Cohen, Mark Bloomenthal, and Beth Cobb for their valuable remarks on the various drafts of this paper.

Figure 5: Blending using T 2 i ( t ) =   dC ( t ) dt -n ( u ( t ) ;; v ( t ))  .

<!-- image -->

<!-- image -->

Figure 6: Blending using T 4 i ( t ) = K i ( t ) ; h K i ( t ) ;;n i ( u ( t ) ;;v ( t )) i h n i ( u ( t ) ;;v ( t )) ;;n i ( u ( t ) ;;v ( t )) i n i ( u ( t ) ;; v ( t )).

<!-- image -->

Figure 4: Blending using T 2 i ( t ) =   dC ( t ) dt -n ( u ( t ) ;; v ( t ))  .

<!-- image -->

## 3 Examples

Figure 4 provides a simple example for T 2 i ( t ) cross boundary tangent curve selection. Figure 5 is another, more complex, example of the use of T 2 i ( t ). The user, once selecting the T 2 i ( t ) method, can control the magnitude of the cross boundary tangent curves, by specifying a scaling curve k ( t ) as discussed Section 2.3. In Figure 5, the cross boundary tangent magnitude varies along the blend surface, larger at the wide blend surface region and smaller at the narrow blend surface region.

By specifying two guiding curves, K i ( t ), and projecting then onto the tangent planes of the surfaces along the two rail curves, the vector eld curves T 3 i ( t ) are dened. By dening two independent guiding curves, K i ( t ), one for each rail curve, the user is provided with a large degree of freedom which may be helpful when warped blending is required (see Figure 6).

In some cases, the approach using T 2 i ( t ) may have undesired eects because it provides the user with no control on the cross boundary tangent directions. In Figure 4, the blend surface bends out of the primary surfaces since its interpolated boundaries are perpendicular to its rail curves where they all meet. Using the T 5 i ( t ) method, one can specify the cross boundary tangent curves as linear combinations of S u ( u;; v ) and S v ( u;; v ). Although not automatic, this method does provide a large degree of freedom to control the cross boundary tangent curves, T i ( t ). Figure 7 shows an example of the use of T 5 i ( t ), which overcomes the rounding eects created by using T 2 i ( t ). The two scalar curves, r ( t ) and t ( t ), of d ( t ) = ( r ( t ) ;; s ( t )), are used to provide the coecients of the linear combination of the partial derivative surfaces (see equation (7)). Each is specied as the dierence of two curves, d 1 ( t ) and d 2 ( t ), in the parametric space of the surface. That is d ( t ) = ( r ( t ) ;; s ( t )) = d 1 ( t ) ; d 2 ( t ) = ( r 1 ( t ) ; r 2 ( t ) ;; s 1 ( t ) ; s 2 ( t )). Figure 8 shows the eect of scaling the cross boundary tangent magnitudes when using the T 2 i ( t ) method to compute a blend surface between two identical spheres.

Finally, the spout connection to the body of the Utah teapot was rounded using the T 2 i ( t )

| Curve (Figure 3)      |   Maximum error |
|-----------------------|-----------------|
| Original T ( t ) (a)  |           2.025 |
| 3 Inserted knots (b)  |            0.25 |
| 6 Inserted knots (c)  |         0.08333 |
| 12 Inserted knots (d) |         0.02571 |

<!-- image -->

t

Figure 3: k ^ T ( t ) k converges to unit length by scaling T i ( t ) with M i ( t ), an approximation to 1 = k T i ( t ) k . Three renement steps are shown with T ( t ) dashed, M i ( t ) dotted-dashed, and ^ T i ( t ) solid. In (a) the original curve, T ( t ), is shown, while (b) to (d) show T ( t ) rened with 3, 6, and 12 equally spaced interior knots, respectively.

exact unit length cross boundary tangent. Figure 3 shows an example of this approach and demonstrates the eect of the renement on the convergence. M i ( t ) was computed from the original quadratic B-spline curve, T ( t ), which has two interior knots in Figure 3(a). In Figure 3(b) to (d), 3, 6, and 12 knots were inserted, respectively, equally spaced in the parametric domain of T ( t ). The convergence rate of k ^ T i ( t ) k to the unit length is shown in Table 2.

Adaptive renement control could apply renement only to those portions of the domain of T ( t ) that have errors in k T ( t ) k larger than an allowed tolerance. An iterative algorithm that converges as closely as needed to the exact unit length of the cross boundary tangent curve, can easily be derived in a similar way to the one used in [7] for computation of oset approximations of freeform curves and surfaces with error bounds.

The operation of vector eld normalization approximation can be applied to various other applications. The normalization of a normal eld of a curve, can open the way for a simple oset approximation of a curve. Normalized vector elds can also be employed in animation for motion control.

One can extend the specication method for T 1 i ( t ) and allow an arbitrary linear combination of S u and S v for another type of cross boundary derivatives, as

<!-- formula-not-decoded -->

where d ( t ) = ( r ( t ) ;; s ( t )) is an arbitrary two dimensional curve that is independent of c ( t ) = ( u ( t ) ;; v ( t )).

T 1 i ( t ) to T 5 i ( t ) are ve dierent methods to dene the cross boundary tangents along the rail curves, C i ( t ). These ve methods are all symbolically computable and exactly representable in the (piecewise) polynomial or rational domains since in these cross tangent denitions only the derivative, sum, dierence, product, and composition operators are used.

We conclude this section with a remark on the degree of the expected tangent elds, T 1 to T 5 . Assume S is a surface of degree n by n . Assume c ( t ) = ( u ( t ) ;; v ( t )) is of degree m . Then, the composition of S ( u ( t ) ;; v ( t )) is of degree 2 mn . The composition of c ( t ) into the unnormalized normal vector eld, n ( u ( t ) ;; v ( t )) yields a vector eld with an even higher degree of 2 m (2 n ; 1). For a biquadratic surface S and a quadratic curve c ( t ), S ( u ( t ) ;; v ( t )) becomes degree 8 and n ( u ( t ) ;; v ( t )) degree 12. For m = n = 3, the cubic cases, S ( u ( t ) ;; v ( t )) becomes degree 18 and n ( u ( t ) ;; v ( t )) degree 30. These high degree vector elds immediately aects the degree of the resulting Hermite blend surface (Equation (4)) and should be taken into account in cases it might impose diculties. Finally, one should recall that low degree c ( t ) curves can greatly alleviate the high degrees in the composition. For an arbitrary straight line in the parametric space of S , m = 1.

## 2.3 Cross Boundary Tangent Normalization

The resulting cross boundary tangent curves T i ( t ), discussed in Section 2.2, are not normalized to a unit length. Computed symbolically, the magnitude of these vector eld curves can signicantly vary, on the order of several magnitudes. Since this has an immediate eect on the shape of the computed blend it is desirable to normalize them close to a xed magnitude. Some variation on the magnitude of T i ( t ) can probably be allowed if this variation is within a user specied tolerance. Once normalized to a unit length within this bound, scaling T i ( t ) with a constant k , can be symbolically computed as kT i ( t ), whereas variable scaling is achievable by a scalar function of t , k ( t ), as the product k ( t ) T i ( t ).

Let m ( t ) = h T ( t ) ;;T ( t ) i = k T ( t ) k 2 be the scalar curve equal to the magnitude squared of the cross boundary tangent curve T ( t ). Let M ( t ) be a scalar curve dened using m ( t ), with the same order and continuity (knot vector), by substituting each coecient k in the control polygon of m ( t ) as 1 p k in the control polygon of M ( t ).

The error between a C 2 continuous function and its Schoenberg variation diminishing spline approximation [15] over a knot vector f t i g is O ( jf t i gj 2 ), where jf t i gj = max i f t i +1 ; t i g . By using a sequence f T i ( t ) g , of rened representations of T ( t ), based on the same sequence of knot vectors, and a sequence f m i ( t ) g , where m i ( t ) = h T i ( t ) ;;T i ( t ) i , using the Schoenberg variation diminishing spline approximations to T ( t ), the sequence f M i ( t ) g converges to M ( t ) = 1 k T ( t ) k . Let ^ T i ( t ) = T i ( t ) M i ( t ). By suitable renement, ^ T i ( t ) can converge as closely as needed to the This method uses the unnormalized normal to c ( t ) in the parametric space of S , N ( t ). However, T 1 i ( t ) is not, in general, normal to C ( t ) in Euclidean space. Figure 2 shows one such example.

Figure 2: A normal to c ( t ) in parametric space (a) is not, in general, normal to C ( t ) in Euclidean space (b).

<!-- image -->

Denote by n ( u;; v ) = ( S u ( u;; v ) -S v ( u;; v )) the vector eld of the unnormalized normal of surface S . Using n ( u;; v ), an alternative denition to T 1 i ( t ) exploits n ( u;; v ):

<!-- formula-not-decoded -->

T 2 i ( t ) is guaranteed to be orthogonal to C ( t ) [10]. The vector eld curve n ( u ( t ) ;; v ( t )) provides a normal to surface S for each point along the Euclidean curve C ( t ) = S ( u ( t ) ;; v ( t )), while dC ( t ) dt is the tangent curve of C ( t ). Equation(5) provides the directions in P , the tangent plane of S , that are also normal to C ( t ).

In [10], a dierent approach is proposed for the cross boundary tangent selection. Let K 1 ( t ) = C 2 ( t ) ; C 1 ( t ) and K 2 ( t ) = C 1 ( t ) ; C 2 ( t ) = ; K 1 ( t ) be vector eld curves. K i ( t ) are called guiding curves [14]. The projection of K i ( t ) on the tangent plane of S i ( u;; v ) can provide a cross boundary tangent direction in surface S i ( u;; v ),

<!-- formula-not-decoded -->

One can consider dening the two guiding curves, K i ( t ), independently for each of the blend rail curves, resulting in cross boundary tangent curve denitions T 4 i ( t ). T 3 i ( t ) is then a special case of T 4 i ( t ), in which K 1 ( t ) = ; K 2 ( t ).

Although n i ( u ( t ) ;; v ( t )) are not the unit length normal, equation (6) is implicitly normalized by constructing a rational expression in which the denominator is equal to the magnitude of n ( u ( t ) ;; v ( t )) squared, h n ( u ( t ) ;; v ( t )) ;;n ( u ( t ) ;; v ( t )) i = k n ( u ( t ) ;; v ( t )) k 2 . In other words, a division by a scalar curve can be represented as a rational curve without explicitly computing the division, hence T 3 i ( t ) and T 4 i ( t ) are rational expressions, even if S is a polynomial surface.

<!-- formula-not-decoded -->

Table 1: The cubic Hermite basis functions.

## 2.1.2 Hermite Interpolant

The cubic Hermite interpolant is used to dene the blend surface that is symbolically computed. Table 1 denes the four basis functions of the cubic Hermite interpolant. Two positional constraints, C 1 and C 2 , and two tangential constraints, T 1 and T 2 , are needed to dene the cubic Hermite curve (See [11] for more details on the Hermite interpolant),

<!-- formula-not-decoded -->

It is worth noting that an immediate consequence of using the cubic Hermite basis functions is the following: H (0) = C 1 , H (1) = C 2 , H 0 (0) = T 1 , and H 0 (1) = T 2 .

## 2.2 Cross Boundary Tangent Denition

To use a cubic Hermite interpolation scheme (equation (3)), one must have cross boundary tangent constraints, T i ( t ), i = 1 ;; 2, as well as the two rail curves, C 1 ( t ) and C 2 ( t ).

One way to dene the cross boundary tangential constraints is to specify cross boundary tangent curves, T 1 ( t ) and T 2 ( t ). The blend surface can then be dened using the cubic Hermite basis functions (see equation 3 and Table 1) as

<!-- formula-not-decoded -->

Dening the T i ( u ) curves is a dicult problem. Either these curves must be computed automatically or the user should be provided with geometrically intuitive tools to derive them. We explore several possibilities with dierent levels of user interactions that are required.

6

Let c ( t ) = ( u ( t ) ;; v ( t )) be a curve in the parametric space of surface S ( u;; v ), and let curve C ( t ) be the composition C ( t ) = S ( c ( t )) = S ( u ( t ) ;; v ( t )). Let S u ( u;; v ) = @S ( u;;v ) @u and S v ( u;; v ) = @S ( u;;v ) @v , be the two partial derivatives of S . N ( t ) =  ( dv ( t ) dt ;; ; du ( t ) dt ) is a vector eld [5] curve representing the unnormalized normal of c ( t ) in S 's parametric space. For a regular surface, when S u ( u ( t ) ;; v ( t )) -S v ( u ( t ) ;; v ( t )) = 0, S u ( u ( t ) ;; v ( t )) and S v ( u ( t ) ;; v ( t )) form a basis for P , the tangent plane of S . In [14], it is suggested that the two cross boundary tangents, T i , i = 1 ;; 2, could be selected using the partial derivatives of S and N ( t ), yielding,

<!-- formula-not-decoded -->

where h ;; i denotes the inner product.

Figure 1: A circle with its speed prole curve in (a) is reparametrized using composition to start slowly (b) or to both start and terminate slowly (c). The markers shows equal domains in the parametric space of the circle.

<!-- image -->

knot lines of the surface, in the surface's parametric space, should be computed. See [12] for more.

This composition can be applied to various other applications, for example for the purpose of reparameterization of a given (animation) curve to a prescribed speed [1]. Given a parametric curve C ( t ), and an allowable change of parameter t ( r ), the velocity of C with respect to r equals,

<!-- formula-not-decoded -->

Hence, one can construct an allowable change of parameter t ( r ) so that t (0) = 0 ;; t (1) = 1 and

<!-- formula-not-decoded -->

resulting in a curve C ( r ) with unit velocity at its end points, a type of curve that is useful for animation, controlling the speed of the motion. Clearly, this reparameterization can be employed to arbitrarily vary the speed of the curve. Figure 1 (a) shows a NURBs circle curve reparametrized to start slowly (Figure 1 (b)) or to both start and terminate slowly (Figure 1 (c)). Both Figure 1 (b) and Figure 1 (c) were computed using a reparameterization curve, t ( r ).

## 2.1 Background

## 2.1.1 Symbolic Operations

To be able to symbolically compute blend surfaces, one must be able to symbolically represent the derivative, sum, dierence, product, and composition of scalar B ezier and NURBs curves and surfaces. Any manipulation of B ezier or NURBs curves or surfaces using these tools should result in a scalar or vector eld represented as a B ezier or NURBs curve or surface. The resulting curve or surface is exact to within the accuracy of the numerical computation, since these operations have closed forms and are, in fact, symbol manipulators. Therefore, we refer to the use of these tools as symbolic computation. These operations in the polynomial (B ezier) and piecewise polynomial and rational (NURBs) domains are discussed in several places [6, 8, 9, 12, 16]. Here, we briey discuss only the symbolic computation of a curvesurface composition in the B ezier domain as an example of this approach.

A curve-surface composition in the polynomial B ezier domain can be computed as,

<!-- formula-not-decoded -->

where P ij are the control points of S , B n i ( u ) and B m j ( v ) are the basis functions of S and c ( t ) = ( u ( t ) ;; v ( t )) is a curve in the parametric domain of S .

Assuming one can compute and represent B l k ( w ( t )), where w ( t ) 2 [0 ;; 1] is a scalar B ezier curve, as a B ezier curve, the curve S ( u ( t ) ;; v ( t )) is also representable as a B ezier curve since it is the result of sums and products of B l k ( w ( t )) terms only. From the denition of the Bernstein polynomials,

<!-- formula-not-decoded -->

Let w ( t ) = P d p =0 w i B d p ( t ). Then,

<!-- formula-not-decoded -->

Moreover (see [6, 9]),

<!-- formula-not-decoded -->

Therefore, the composition S ( u ( t ) ;; v ( t )) can be symbolically computed and represented in the polynomial B ezier domain. The computation and representation of the composition operation in the rational domain is a simple extension to the above (see [6]). For the piecewise polynomial and rational domains (NURBs), the problem is more dicult. Either the composition is posed as an interpolation problem or the curve and the surface are subdivided into polynomial or rational regions. In both cases the intersection locations of the curve with the accurately represent nished manufactured parts. In many cases, the use of Boolean operations in modeling systems results in models with undesirable sharp seams between surfaces.

The nomenclature used in discussing these smoothing surfaces diers somewhat from author to author, with the surfaces in question being known variously as rounding, llet, or blend surfaces. We will adopt the term \blend" to denote both rounding and lleting operations when referring to these surfaces. Blend surfaces are usually constructed along the seam or intersection between two other already specied surfaces, called the primary surfaces . The boundary curves of the blend surface, which lie on the primary surfaces, are referred to as rail curves [10] or trimlines [18]. In [18], a nice survey of the dierent existing blending techniques can be found.

Constant radius blends, also known as rolling ball blends, are a common means of specifying a blend surface. A standard approach to generating these blends involves osetting both primary surfaces [4, 13, 17]. Constant radius blend surfaces are important if the geometry of the blend needs to be precisely prescribed. Frequently, when the aesthetic shape of the model is a major concern, or when sharp corners should simply be chamfered or rounded, the exact geometry constraint can be removed. In [3, 10, 12], approaches are developed which do not necessarily construct constant radius blends. These approaches are appealing for their simplicity and the elimination of the surface oset operation requirements. Blending or joining algorithms for algebraic surfaces including [2, 19] have been developed separately.

This paper presents new symbolic techniques for the specication and representation of blend surfaces from within the context of a NURBs based geometric modeling system called Alpha 1, developed at the University of Utah. Section 2 presents techniques for representing blend surfaces exactly in polynomial (B ezier) or piecewise polynomial and rational (NURBs) domains from given rail curves and cross tangent curve specications. Section 3 details some blending examples while we conclude in section 4.

## 2 Symbolic Blending

In this section we use symbolic computation to derive blend surfaces. Towards this end we use the cubic Hermite interpolation scheme with two rail curves and two cross boundary tangent curves along these rail curves.

In Section 2.1, symbolic computation is reviewed. In Section 2.2, several methods to dene the cross boundary tangent information along the rail curves are discussed. Finally, in Section 2.3, an iterative approach is suggested for the normalization of the cross boundary tangent curves. This normalization process can benet other applications such as unit normal approximation for oset computation or unit length vector elds for constant speed motion planning in animation. Herein, these normalized cross boundary tangent denitions and the cubic Hermite form one can symbolically compute a blend surface that provides plane continuity, in exact form up to machine accuracy.

## A Symbolic Approach to Freeform Parametric Surface Blends

Kwansik Kim y and Gershon Elber z Department of Computer Science University of Utah Salt Lake City, UT 84112 USA

May 25, 1999

## Abstract

This paper presents a symbolic approach to the computation of blend surfaces across piecewise polynomial and rational surfaces. Curves in the parameter space of the primary surfaces can be specied as the rail curves, also known as trimlines, of the blend. Several techniques which require various levels of user interaction are presented for dening the cross boundary tangent curves along the rail curves. The resulting blend is represented as a polynomial surface having tangent plane continuity with the primary surfaces to an accuracy bounded only by the machine precision. Also presented is a normalization method that approximates a vector eld, an approach that might benet other applications such as oset approximation and animation curve construction.

## 1 Introduction

unit Key Words: Symbolic computation, Hermite interpolation, Fillets and Rounds, Blends, Reparameterization.

Blend surfaces are ones which smoothly blend two adjacent surface boundaries. In addition to being a modeling tool, blend surfaces are used to emulate manufacturing procedures using ball end tools and are employed to alleviate stress. In most cases blend surface are used to smoothly connect two other surfaces, so the exact shape of the blend may be relatively unimportant as long as continuity is satised. Tangent plane continuity is frequently considered sucient although higher order continuity is sometimes desired as well.

As geometric surface modelers have matured, the ability to represent blend surfaces has become increasingly important. One reason for this is that models are desired that more

 This work was supported in part by DARPA (N00014-91-J-4123). All opinions, ndings, conclusions or recommendations expressed in this document are those of the authors and do not necessarily reect the views of the sponsoring agencies.

y Current address: Computer and Information Science Department, 225 Applied Sciences, University of California, Santa Cruz, CA 95060

z Current address: Computer Science Department, Technion, Haifa 32000, Israel

## 