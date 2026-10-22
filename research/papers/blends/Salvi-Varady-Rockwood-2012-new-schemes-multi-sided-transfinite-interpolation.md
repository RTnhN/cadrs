## New Schemes for Multi-sided Transfinite Surface Interpolation

Péter Salvi † , Tamás Várady † , Alyn Rockwood ‡

† Budapest University of Technology and Economics

- ‡ King Abdullah University of Science and Technology

## Abstract

One of the major problems in CAGD is to create mathematical representations for complex free-form objects composed of several smoothly connected surfaces. An intuitive solution is curvenet-based design, where designers directly define feature curves of the object, which are interpolated by multi-sided surfaces. A convenient representation for these models is based on transfinite surface interpolation. This paper introduces two new schemes for transfinite surfaces: a true multi-sided generalization of the Coons patch, and another based on a natural combination of curved side interpolants. The two formulations are supported by new parameterizations that need to satisfy strict tangential constraints along the boundaries. A few examples comparing the two patches illustrate the results.

Categories and Subject Descriptors (according to ACM CCS): I.3.5 [Computer Graphics]: Computational Geometry and Object Modeling

## 1. Introduction

Curvenet-based design 9 is an intuitive approach to create complex free-form CAD models in three dimensions. The process begins with creating a curve network representing the edges and feature lines of the object. These curves may come from several sources, like traditional blueprints, 2D sketches, or directly by some 3D graphical user interface. Then surfaces are stretched over the curve loops interpolating these boundaries. Transfinite interpolation surfaces are particularly suitable for this purpose, since they are defined solely by the boundary curves and their cross-derivatives.

The Coons patch is a well-known example for four-sided (biparametric) configurations, widely used due to its natural curvature distribution. Multi-sided patches, however, cannot be defined by two parameters, and the known solutions including the patches of Gregory 1 and Kato 4 -differ from the original Boolean sum logic proposed by Coons. These may also exhibit shape artifacts in certain situations with highly curved boundaries and uneven lengths.

In this paper we introduce two new surface representations that are 'close' to Coons' original idea. The first is a true generalization of the Coons patch, consisting of side interpolants and corner correction patches; the second is a surface combining curved side interpolants by a natural application of blend functions.

In Section 2 we will examine the Coons patch and rewrite it in an easily extensible formulation. Then in Section 3 we will review the most important traditional n -sided surface formulations. The generalized Coons patch will be presented in Section 4, along with two new parameterization methods. It is followed by the definition of the curved interpolantbased surface in Section 5, and some examples in Section 6.

This paper is a condensed version of a more-detailed journal paper 6 , where all mathematical details and the related proofs are also presented. At the same time, we hope that the basic concept of these new representational forms can be understood based on the current paper, as well.

## 2. Coons patch reformulation

Let us assume that we have a four-sided surface patch S parameterized in the ( u , v ) plane ( u , v ∈ [ 0 , 1 ] ). Given positional side constraints S ( u , 0 ) , S ( u , 1 ) , S ( 0 , v ) , S ( 1 , v ) and tangential (cross-derivative) constraints Sv ( u , 0 ) , Sv ( u , 1 ) , Su ( 0 , v ) , Su ( 1 , v ) , the patch equation suggested by Coons can be written as

Figure 1: Domain of a Coons patch using side-based parameterization

<!-- image -->

<!-- formula-not-decoded -->

This is the well-known Boolean sum formulation 2 , where a 0 , a 1 , b 0 , b 1 are the cubic Hermite blending functions, needed to satisfy G 1 continuity:

<!-- formula-not-decoded -->

In order to reformulate the above definition on a per-side basis, we will use cyclic indices (with 1 coming after 4), and introduce the so-called side parameters, si = si ( u , v ) , associated with the i -th side of the domain, taking the values u , v , 1 -u and 1 -v , as appropriate (see Fig. 1). Then, denoting the positional and tangential constraints by Pi ( si ) and Ti ( si ) , respectively, as well as using Wi , i -1 for the twist vector, the Coons patch can be formally rewritten as

<!-- formula-not-decoded -->

We assume that twist vector compatibility is ensured, i.e.,

<!-- formula-not-decoded -->

For contradicting boundary constraints, this can be resolved by Gregory's rational twists (see Gregory 3 or Farin 2 ).

This formulation can be further simplified, if we replace the tangential blending with linear interpolants or ribbons :

<!-- formula-not-decoded -->

Here we introduced another local parameter, the so-called distance parameter di = di ( u , v ) , that measures some kind of distance from the i -th side. In this four-sided case di ( u , v ) = s i + 1 ( u , v ) is a simple choice. The resulting patch becomes

<!-- formula-not-decoded -->

where Qi , i -1 is a corner correction patch given by

<!-- formula-not-decoded -->

This will not be identical to the Coons patch, but it is a very similar construction that trivially satisfies all the boundary constraints. This formulation will be the basis of the generalization presented in Section 4.

## 3. Multi-sided patches

We can interpret the same problem for an arbitrary number of boundary curves. Given n three-dimensional curves Pi ( si ) , 1 ≤ i ≤ n , and cross-derivative functions Ti ( si ) along these curves, we want to construct an n -sided patch that interpolates these boundary curves and derivatives. For this, we will need a domain polygon in the parameter plane. The simplest choice is a regular polygon; for enhanced, non-regular domain alternatives, see Várady et al. 8

We will also need a parameterization scheme that transforms the ( u , v ) coordinates of the parameter plane into local parameters of the individual boundary curves. Two possible constructions are presented in Section 4.3. For a more complete overview, see Salvi 5 .

The rest of this section concentrates on how these curves and cross-derivative functions can be combined to create an interpolating patch, and what kind of blending functions can be used. As a reminder, Figure 2 presents a bird's-eye view of the steps needed to evaluate a transfinite surface.

Charrot and Gregory 1 proposed an n -sided patch based on corner interpolants , i.e., surfaces that interpolate two consecutive boundary curves, defined as

<!-- formula-not-decoded -->

It is easy to see that these surfaces interpolate the i -th and ( i -1)-th sides, and their cross-derivatives. A similar technique will be introduced in Section 5 for interpolating three

Q

Figure 2: Evaluation of a transfinite surface: (i) point in the polygonal domain (ii) mapping into the domain of each biparametric ribbon (iii) evaluating each ribbon (iv) blending the evaluated points

<!-- image -->

sides. The Gregory patch is constructed in the following way:

<!-- formula-not-decoded -->

Note, that we continue to use cyclic indices (e.g. i -1 for i = 1 is n , and conversely, i + 1 for i = n is 1). Here a more complex blending function needs to be used, that is a rational polynomial function of the distance terms di :

<!-- formula-not-decoded -->

with the notation D e i 1 ... i k = j / ∈{ i 1 ... i k } d e j . This blending function will be used in Sec. 4.2 for the generalized Coons patch. The di parameters in an n -sided polygon are generally not equal to s i + 1 (as we have seen for rectangles in the previous section), but can be independent 'distance' parameters. For details, see Section 4.3.

Kato 4 also used the linear side interpolants Ri introduced above to define a patch as

<!-- formula-not-decoded -->

There are several variations on the blending function, one popular choice 8 being

<!-- formula-not-decoded -->

Note, that the blending function is singular at the corners; fortunately, the corner points are uniquely defined by the boundary constraints.

## 4. Generalized Coons patch

The next few sections deal with the generalization of Coons patches to n sides. The proposed interpolating surface uses the same principles as the original Coons patch, i.e., combines linear side interpolants and corner correction patches, and thus can be regarded as a natural generalization, even though the surface presented here will not revert to a Coons patch for quadrilateral configurations.

In Section 4.1 a new patch construction is introduced, and in Sections 4.2 and 4.3 we show suitable functions for blending and parameterization, respectively.

## 4.1. Coons Patch Generalization

With appropriate parameterization functions si ( u , v ) and di ( u , v ) , Equation (1) can be generalized as

<!-- formula-not-decoded -->

where Bi = Bi , i -1 + Bi + 1 , i is a side blend , and Bi , i -1 is a corner blend function. Bi , i -1 is required to have the following properties:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Section 4.2 will show a possible construction. Equation (2) means that the blend function vanishes on all sides not connected to the corner. Boundary interpolation is satisfied due to Eq. (3), and finally Eq. (4) is needed for tangential interpolation.

Note that this patch, similarly to the Coons patch, combines five surfaces to evaluate a point on the boundary: three linear ribbons and two corner correction patches. This imposes several requirements for the ( si , di ) parameterization. First, for si we need

<!-- formula-not-decoded -->

and for a point on the i -th side, the parameterization has to satisfy that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Figure 3: Blending functions Bi , i -1 and Bi over a six-sided domain.

<!-- image -->

The patches reviewed in Section 3 only require (5)-(7), so we need to satisfy more strict restrictions here, but in Section 4.3 two techniques will be presented for creating eligible parameterizations.

## 4.2. Blending functions

Recall the blending function Bi , i -1 described in Charrot and Gregory 1 :

<!-- formula-not-decoded -->

Assumptions (2) and (3) are trivially true. A closer examination of the difference quotient leads to a straightforward proof of (4), thus this blend function adheres to all requirements. Fig. 3 shows a graphical representation of Bi , i -1 , as well as the side blend Bi = Bi , i -1 + Bi + 1 , i .

## 4.3. Parameterizations

In the following we will propose two parameterizations that satisfy all the requirements (5)-(10).

## 4.3.1. Interconnected Parameterizations

Take functions si ( u , v ) that give 0 for every point on the ( i -1 ) -th side and 1 for those on the ( i + 1 ) -th side; for all other points inside the convex domain, they return a value in [ 0 , 1 ] . For example, the s coordinates of the bilinear or radial line sweeps 1 , 5 are such functions. These naturally satisfy (5) and (7). Define a blending function a ( t ) ∈ [ 0 , 1 ] → [ 0 , 1 ] with a ( 0 ) = 1 and a ( 1 ) = a ′ ( 0 ) = a ′ ( 1 ) = 0. Examples are the Hermite function a 0 ( t ) from Section 2, or a variation of the rational blend function presented in the previous section: a ( t ) = ( 1 -t ) 2 t 2 +( 1 -t ) 2 . Now we can define di by means of s i -1 and s i + 1 as follows:

<!-- formula-not-decoded -->

If we are on the i -th side, s i -1 = 1 and s i + 1 = 0, so di = 0, satisfying (6).

Still on the i -th side, di -1 and its derivative are the same as si :

<!-- formula-not-decoded -->

because the derivatives of the blend function vanish. The same reasoning works for the derivative by v . Similarly

<!-- formula-not-decoded -->

so the requirements (8), (9) and (10) are all satisfied.

Fig. 4 shows constant s and d lines for this parameterization (using the central line sweep parameterization 10 as a basis). The first image is based on the right side of the polygon; the second image is based on the small side at the top-right; and the third image is based on the top side. Note that all lines of the second image start the same way (in a differential sense) as their counterparts in the first and third images.

## 4.3.2. Biquadratic Parameterization

The patch defined in Section 4.1 is using side-based linear interpolants, but also corner-based correction patches. In the following, two parameterizations will be used, one for the ribbons (side-based parameterization), and one for the correction patches (corner-based parameterization). The idea is that a linear side interpolant should have the same parameterization near the side adjacent to its base as the correction patch of the corresponding corner, enabling the correction patch to cancel out the extra terms generated by the ribbons. This will be achieved by biquadratic maps similar to the overlap patch parameterization 7 .

We place a three-by-three control net on the domain, and the resulting planar surface defines a parameterization (see Fig. 5). We need the inverse of this biquadratic map, which can be computed by numerical methods, such as the Newton-Raphson algorithm. Note that the control points on the left-hand side are the same in both the side-based and the corner-based variants, so the two parameterizations behave identically near the left boundary. In the following we will examine the construction of these biquadratic control nets.

Imagine a domain with its i -th side on the u axis. For the side-based case, the control points are defined as

<!-- formula-not-decoded -->

Figure 4: Constant parameter lines of the interconnected parameterization.

<!-- image -->

Figure 5: Control net and constant parameter lines of the biquadratic parameterization.

<!-- image -->

where p l j , p r j and p m j are the left and right endpoints and the midpoint of the j -th side, respectively; c is the center of the domain and p o j is the domain vertex opposite to the j -th side (for even-sided polygons p o j is the midpoint of the opposite side).

When the basis is the corner at p l i , the control points are

<!-- formula-not-decoded -->

where p o i , i -1 is the vertex opposite to the vertex defined as the intersection of the i -th and ( i -1 ) -th sides (for odd-sided polygons p o i , i -1 is the midpoint of the opposite side).

Note that the first two columns are the same as in the sidebased case. Also the first two rows have their counterparts in the ( i -1 ) -th side-based biquadratic as well. These relationships tell us that the parameterization based on the i , i -1 corner behaves the same way for a point on the ( i -1 ) -th side as the parameterization based on the i -th side; and for a point on the i -th side as the parameterization based on the ( i -1 ) -th side. This is important, because it means that all superfluous data that comes into the equations from the ribbon interpolation can be eliminated by corner-parameterized correction patches.

There is one special case: three-sided segments have a very intuitive, singular parameterization (depicted in Figure 6).

So in our transfinite surface scheme, side-based parameterizations are used for ribbons ( Ri ) and blends ( Bi , i -1 ), but corner-based parametrizations are used for the correction patches ( Qi , i -1 ). We will use the convention of referring to corner-based parameters by ( s ∗ i , d ∗ i ) , while retaining ( si , di ) for side-based ones.

The parametrization requirements (5)-(7) are trivially satisfied for side-based schemes. Linearity on the sides also guarantees (8), even between a side-based and a cornerbased patch. The remaining constraints do not hold for either type, but are valid in-between: the di -1 side-parameters are equal to the s ∗ i corner-parameters in derivative sense (a corner-parameter with index i is associated with the biquadratic patch based on the i -th and ( i -1 ) -th edges, using the i -th edge as constant d ∗ i = 0). Similarly all requirements are satisfied in this way.

Using this new double parameterization system, we have to alter the original definition of the surface, substituting Qi ( s ∗ i , d ∗ i ) = Qi , i -1 ( s ∗ i , 1 -d ∗ i ) for Qi , i -1 ( si , s i -1 ) :

<!-- formula-not-decoded -->

Figure 6: Control nets for triangular domains, based on the bottom side (left image), the right side (right image) and both (middle image).

<!-- image -->

Figure 7: Constituents of a curved ribbon.

<!-- image -->

## 5. Curved ribbons

The patch defined in the previous section is based on linear ribbons. These have the advantage of being very simple and can be computed efficiently. On the other hand, highly curved surfaces may deviate far from their linear interpolants, thereby decreasing stability. Unintuitive bulges, coming from contradicting cross-derivatives, pose a common problem. Recall, however, that conventional side interpolant-based transfinite interpolation surfaces do not depend on the linearity of the ribbons. We can thus define curved ribbons that follow the shape of the surface more closely.

Let Ci ( si , di ) denote the curved ribbon for the i -th side. In order to simplify the notation, we will drop the indices of s and d when talking about only a single curved ribbon, as it does not cause any ambiguity. The definition of Ci is as follows (see Fig. 7):

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and H ( t ) is a blend function, for example the Hermite blend function a 0 ( t ) .

It is interesting to note that the curved corner interpolants of the Gregory patch 1 can be defined in terms of ribbons and correction patches, as well:

<!-- formula-not-decoded -->

The curved ribbon defined above interpolates three consecutive boundary curves using blends, which makes it essentially a three-sided Coons patch. These ribbons can be used as side interpolants for conventional transfinite interpolation patches, but there is also a more natural blending scheme, as proposed in the next section.

## 5.1. Composite ribbon patch

The generalization of Coons patches, as introduced in Section 4, does not work with curved side interpolants. However, we propose a new representation that combines curved ribbons in a different way, and renders the elimination of correction patches possible, yielding a much simpler formula:

<!-- formula-not-decoded -->

where the parameterization satisfies the requirements listed in (5)-(10).

According to the characteristics of the Bi blend function, for any point on the i -th boundary all addends of the sum vanish except for Ci -1 , Ci and Ci + 1 . Since each of these ribbons also interpolates the adjacent curves, the three ribbon points are the same. Their cumulative blend is

P. Salvi, T. Várady, A. Rockwood / New Schemes for Multi-sided Transfinite Surface Interpolation

<!-- image -->

(a) Conventional linear interpolant-based patch

Figure 8: Mean map comparison of a model with four surfaces.

<!-- image -->

<!-- formula-not-decoded -->

hence the division by two in the surface equation.

It can be proven that the parameterization constraints can be loosened. If we leave out the last two requirements (9 and 10), the patch remains valid, i.e., it will have the same tangent plane for every boundary point as the respective Coons ribbon, but it won't have the exact same tangent vector. This enables the use of many other parameterizations, such as the bilinear or the central line sweep parameterizations.

In other words, we have created a new transfinite surface representation that (i) has the same computational complexity as other conventional methods, (ii) uses curved side interpolants, and (iii) employs non-singular blend functions.

## 6. Examples

The first example in Fig. 8 shows the mean map of a model containing three-, four- and five-sided surfaces. The first image was created by a conventional linear interpolant-based patch. Note the blue areas near the sides - these are artifacts of the blending function, which abruptly drops as we move away from the boundaries. The composite ribbon patch below is free of these unwanted curvature changes.

Figure 9: Mean map comparison of a surface using different patch types.

<!-- image -->

In Fig. 9 three schemes are contrasted using a single surface. The low-curvature areas at the right-hand corners are smoothed out in both of the new patches. The generalized Coons patch and the composite ribbon patch are very similar, but the latter has even more smooth curvature transitions.

In order to understand the essential difference between the two new patches, we should examine their ribbons. Figure 10 depicts them over the constant parameter lines of the patches. In the linear ribbon case, the interpolants start to deviate from the surface at a very 'early phase', thus points in the interior are actually computed as the affine combination of relatively distant positions. On the other hand, the curved interpolants on the right-hand side image are going close to the predictable surface position, and in this way, points in the interior are combinations of ribbon points that are relatively close, keeping curvature variation at a low level. The slicing maps show that both surfaces have sufficiently good quality.

<!-- image -->

(a) Generalized Coons patch

<!-- image -->

(b) Composite ribbon patch

Figure 10: Ribbons and slicing map of a five-sided boundary configuration.

Figure 11: A G 1 model with two composite ribbon patches, using isophote line visualization.

<!-- image -->

Finally, Fig. 11 shows isophote lines on two composite ribbon patches connected by a smooth edge. The ends of the lines match along the boundary, confirming G 1 continuity, and in most points they are also unbroken, exhibiting approximately G 2 behavior.

## Conclusion

Two new surface representations for transfinite surface interpolation have been presented. The first scheme can be considered as the true generalization of the Coons patch, using classical Boolean sum logic. The second is a transfinite surface combining curved side-interpolants. In order to satisfy the boundary constraints (positions, cross-derivatives), special blend functions and parameterizations have been applied. Future research work is going to be directed towards G 2 ribbons and using non-convex parametric domains.

## Acknowledgements

This work was partially supported by the scientific program 'Development of quality-oriented and harmonized R+D+I strategy and functional model at the Budapest University of Technology and Economics' (UMFT-TÁMOP-4.2.1/B09/1/KMR-2010-0002) and a grant by the Hungarian Scientific Research Fund (No. 101845). The pictures in this paper were generated by the Sketches system developed by ShapEx Ltd, Budapest; the contribution of György Karikó is highly appreciated. The authors also acknowledge support from the Geometric Modeling and Scientific Visualization Research Center of KAUST, Saudi-Arabia.

## References

1. P. Charrot, J. A. Gregory, A pentagonal surface patch for computer aided geometric design, Computer Aided Geometric Design 1 (1) (1984) 87-94.
2. G. Farin, Curves and surfaces for CAGD: a practical guide, 5th ed., Morgan Kaufmann Publishers Inc., San Francisco, CA, USA, 2002.
3. J. A. Gregory, Smooth interpolation without twist constraints, in: R. E. Barnhill, R. F. Riesenfeld (eds.), Computer Aided Geometric Design, Academic Press, Inc., 1974, pp. 71-88.
4. K. Kato, n-sided surface generation from arbitrary boundary edges, in: P.-J. Laurent, P. Sablonnière, L. L. Schumaker (eds.), Curve and Surface Design: SaintMalo 1999, Innovations in Applied Mathematics, Vanderbilt University Press, Nashville, TN, 2000, pp. 173181.
5. P. Salvi, Fair curves and surfaces, Ph.D. thesis, Eötvös Loránd University, Budapest (2012).
6. P. Salvi, T. Várady, A. Rockwood, New multi-sided transfinite surfaces, Computer Aided Geometric Design (submitted).
7. T. Várady, Overlap patches: a new scheme for interpolating curve networks with n-sided regions, Computer Aided Geometric Design 8 (1) (1991) 7-27.
8. T. Várady, A. Rockwood, P. Salvi, Transfinite surface interpolation over irregular n-sided domains, Computer Aided Design 43 (2011) 1330-1340.
9. T. Várady, P. Salvi, A. Rockwood, 3d shape design using curve networks with ribbons, in: this proceedings.
10. T. Várady, P. Salvi, A. Rockwood, Transfinite surface interpolation with interior control, Graphical Models (under revision).