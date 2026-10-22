## Multi-sided Surfaces with Curvature Continuity

Péter Salvi, Tamás Várady

Budapest University of Technology and Economics

## Abstract

The basic idea of curve network-based design is to construct a collection of smoothly connected surface patches that interpolate boundary constraints extracted solely from the curve network. While the majority of applications demands only tangent plane (G 1 ) continuity between the adjacent patches, there are applications where curvature continuous connections (G 2 ) are required. Examples include handling special curve network configurations with supplemented internal edges, 'master-slave' curvature constraints and general topology surface approximations over meshes. The first step of the surface generation process is the construction of interpolant surfaces that enforce suitable cross-derivatives for transfinite surface patches; these interpolants are often called ribbons . For G 2 interpolation we extend Gregory's multi-sided surface scheme, and focus on creating and combining special parabolic ribbons. We discuss the basic patch construction including the blending functions and a special sweepline parameterization. A proof of G 2 continuity is given in the Appendix. The application of curvature continuous multi-sided patches is demonstrated by a few simple examples.

Categories and Subject Descriptors (according to ACM CCS): I.3.5 [Computer Graphics]: Computational Geometry and Object Modeling - curvenet-based design, transfinite surfaces, Gregory patches, G 2 continuity

## 1. Introduction

In curve network-based design, surface models are directly defined by a collection of freeform curves, arranged into a single 3D network with general topology. Curves may come from (i) sketch input, (ii) feature curves extracted from orthogonal views, (iii) curves traced on triangular meshes or (iv) direct 3D editing. Once the curves are defined, all the surfaces are generated automatically. This calls for a representation based on geometric information extracted solely from the boundaries. Transfinite surface interpolation is a natural choice, as it does not require a grid of control points to define the interior shape, and all n boundaries are handled uniformly, unlike in the case of trimmed quadrilateral surfaces. The ability to interactively edit prescribed boundaries and cross-derivatives is also an advantage in contrast to recursive subdivision schemes.

The first step of surface generation is to compute crossdirectional data, such as common tangent planes and, when needed, curvatures that will be shared by the adjacent patches. Then interpolant surfaces or ribbon s are generated, that carry first or second-degree cross-derivative constraints to be eventually interpolated by the transfinite surfaces.

The majority of multi-sided transfinite surfaces are defined over convex domains, combining only linear ribbon surfaces and enabling G 1 continuity between the adjacent patches. At the same time, there are several practical design situations, where this approach is not sufficient, and higher degree continuity is required.

- (i) It often occurs that additional curves need to be inserted into the curve network to make it suitable for applying convex methods. The supplemented curves must be compatible with the already defined ribbons, and it is particularly important to produce seamless transitions along these curves. A typical example is when two curves span a concave angle at a common vertex and then a composite patch - with convex domains - is created. Another example is to generate surfaces by interpolating two disjoint loops with prescribed slopes (see Figure 7 later in Section 5).
- (ii) Another interesting situation is when a designer wants to retain one of the surfaces (the master ), but also wants to prescribe a curvature continuous connection for the adjacent patch (the slave ), see Figure 8.
- (iii) A third example is when we have a general topol-

ogy curve network defined over a mesh and want to obtain a good approximation of the interior data points (see Figure 9). Clearly, if we extract not only the normal vectors, but also curvature information from the underlying mesh, we can obtain a more accurate approximation.

In the above cases, it is possible to ensure G 2 continuity by using parabolic ribbons. In Section 2 we give an overview of related publications. In Section 3 various methods to compute linear and parabolic interpolants will be discussed, while in Section 4 we introduce an extended Gregory patch formulation with a new sweepline parameterization that makes it possible to combine parabolic ribbons and yield curvature continuous surfaces. A few examples will illustrate the surface scheme in Section 5.

## 2. Previous Work

Most papers concerning transfinite surface interpolation assume that either the ribbons, or at least the cross-derivatives are given, and very little is written about how this information can be extracted from a curve network. The reason for this apparent lack of interest may be that at the time when transfinite surface interpolation appeared with the Coons 2 and Gregory 1 patches, its main application was hole filling and vertex blending, where additional cross-directional data were readily available. This trend lived on, even after the advent of curve network-based ideas, such as the Minimum Variation Surfaces 9 .

There is a constant interest in multi-sided patches with G 2 continuity, the first efforts dating back 25 years 3 . In particular, an extension of the Gregory patch similar to the one described here was published by Hall and Mullineux 4 .

Special treatment of difficult configurations is also a recurring theme in the literature; most approaches use concave domains 7 , 8 , 6 to handle holes and extreme spatial structures, though we have very limited information concerning the practical design potential and the quality of these shapes.

## 3. Computing Ribbons

In the course of curve network-based design, users create and/or edit the boundary curves, while surfaces are generated in an automatic manner. This means that crossderivatives and curvatures are also derived solely from the curve network. Here we will deal only with generating smooth connections, though in a real system the user may want to create sharp edges, as well.

All continuity constraints are accomplished through defining 'proper' ribbons, i.e., once we set the adjacent ribbons G 1 or G 2 continuous, the respective transfinite surfaces will inherit this property.

Two adjacent ribbons are G 1 continuous if they share the same sweep of normal vectors (called the normal fence )

Figure 1: Normal fence and mean curvature map

<!-- image -->

Figure 2: Normal curvature arcs and Gauss curvature map

<!-- image -->

along the common boundary. Figure 1 shows an example, where the fence is rendered as a series of yellow lines. The necessary condition for G 1 continuity is that the first crossderivatives on both sides are perpendicular to the fence.

When we require G 2 continuity between two surfaces, we can make use of the Linkage Curve Theorem 10 , 5 :

Two surfaces tangent along a C 1 -smooth linkage curve are curvature continuous, if and only if at every point of the linkage curve, their normal curvature agrees for an arbitrary direction other than the tangent of the linkage curve.

This practically means that if we have a particular directional sweep along the common boundary, and the normal curvatures of the two ribbon surfaces in this direction are always the same, then we have G 2 continuity. Figure 2 shows an example with the common normal curvatures shown as circular arcs.

In the rest of this section, we will investigate how to determine normals and curvatures from the curve network.

## 3.1. Preliminaries

A ribbon is a four-sided side interpolant surface R ( s , d ) , where s is the side parameter , and d is the distance parameter . The side parameter is in the interval [ 0 , 1 ] and runs along the boundary curve. The distance parameter is defined in the cross direction; it is zero on the boundary and increases as we move away from it.

For a given n -sided patch, there is a loop of curves, Pi ( si ) , and we need to create the corresponding ribbons Ri ( si , di ) , i ∈ [ 1 . . . n ] . The parameters ( si , di ) can also be regarded as functions that map values from a common domain (see Section 4.2). Note also that the indexing is circular, with 1 coming after n and vice versa, and we have Pi -1 ( 1 ) = Pi ( 0 ) for all i .

## 3.2. Cross-derivatives

We assume that for each vertex of the network the crossing curves define a local tangent plane. For each boundary Pi ( si ) of a given patch, there exists a normal fence Ni ( si ) that interpolates the normals at the related corners and minimizes its rotation along the boundary. This is called a rotationminimizing frame or RMF. An exact (closed form) solution of the underlying differential equation cannot be determined, but approximations can be computed via a sequence of discrete points 14 .

The cross-derivatives of the ribbons are defined as

<!-- formula-not-decoded -->

where a ( si ) and b ( si ) are scalar functions, and Di ( si ) represents a direction vector function, that is perpendicular to Ni ( si ) everywhere. One trivial choice is the binormal

<!-- formula-not-decoded -->

but other definitions are also possible.

The scalar functions satisfy end conditions at the corner points ( si = 0 and si = 1), but there are further degrees of freedom to define these in order to optimize the shape of the patch. For example, cross-derivatives at the middle of the boundary ( si = 0 . 5) can be prescribed by users for enhanced control of the surface shape. Alternatively, these can be optimized by fairing algorithms, which is the subject of ongoing research.

## 3.3. Creating Ribbons

Wewill look at two types of side interpolants: linear ribbons, that are ruled surfaces suitable for creating a G 1 -continuous model, and parabolic ribbons, that are quadratic in the cross direction, and rely on the computed common curvature values to ensure G 2 continuity.

Figure 3: Setting the control points of a parabolic ribbon.

<!-- image -->

## 3.3.1. Linear Ribbons

Given a boundary curve Pi ( si ) and the corresponding crossderivative Ti ( si ) , ribbon construction is straightforward:

<!-- formula-not-decoded -->

## 3.3.2. Parabolic Ribbons

Parabolic ribbons are also simple, having the form

<!-- formula-not-decoded -->

where Ci ( si ) is the second cross-derivative of the ribbon.

It is a natural choice to calculate the normal curvature in the sweeping direction of the first cross-derivative, i.e., in the plane spanned by Ti ( si ) and Ni ( si ) . Let us transform, for ease of computation, the parabolic arc of Ri at a fixed ˆ si into a local coordinate system, where the first point is the origin, and the tangent of the arc is the local x -axis. Then at a given boundary point the equation of the parabola can be written as a quadratic Bézier curve

<!-- formula-not-decoded -->

yielding k = h 2 a 2 l 2 (see Figure 3). Assuming that the width of the parabolic ribbon is the same as the corresponding linear one, l is already defined. Then the prescribed curvature k can be set by means of a and h , which define the second and third control points of the parabolic arc.

The choice of a gives us a degree of freedom. It may be chosen as a constant. A better choice is to optimize a so that the parabolic ribbon should minimally deviate from the corresponding linear one. This can be formalized as

<!-- formula-not-decoded -->

which leads to the depressed cubic equation

<!-- formula-not-decoded -->

that can be solved by Cardano's method.

## 4. Surface Creation

There are various transfinite surface schemes that can be applied to interpolate given ribbons. For a comparison, see a review of the authors 13 . Here we will use Gregory patches, as it is one of the simplest and most well-known methods.

Figure 4: Parameterizations

<!-- image -->

where w is an arbitrary sweeping direction. (See also the proof of continuity given in the Appendix.)

The above property can be satisfied, if we create the sweeping lines using Hermite polynomials. Without loss of generality, let the base edge be a segment from ( 0 , 0 ) to ( 1 , 0 ) , a and b edge vectors associated with sides i -1 and i + 1, respectively, and ( u , v ) the point to be mapped, see Figure 4b. Then we can construct the equation

<!-- formula-not-decoded -->

where H ( s ) = 2 s 3 -3 s 2 + 1, and s and d are unknown. This leads to a fourth-degree equation in s :

<!-- formula-not-decoded -->

where the coefficients are

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

This does not pose any difficulty for real-time computation, as efficient algorithms exist for solving fourth-degree polynomial equations 11 , and the values for a given resolution can be cached. For the result, see Figure 4c. Note, that now the blue sweeplines of side i -1 and the red sweeplines of side i + 1 at the bottom are identical in a differential sense.

## 4.3. Correction Terms

Let us investigate the partial derivatives of two ribbons meeting at a common corner point. We need a single corner inter-

## 4.1. Ribbons

Gregory patches combine corner interpolants, so our first task is to convert our side-based (linear or parabolic) ribbons to corner-based surfaces. It is well-known from the classical theory of Boolean-sum surfaces 2 that a correction patch Qi , i -1 is needed to cancel out the unwanted terms coming from the combination of the two side interpolants (see also Section 4.3):

<!-- formula-not-decoded -->

where s j = s j ( u , v ) ( j ∈ [ 1 . . . n ] ) denote the side parameters defined over the polygonal domain. This brings us to the next topic: how to map the four-sided ribbon surfaces onto the n -sided domain polygon, or inversely, how to determine the local side parameters from a given ( u , v ) point.

## 4.2. Parameterization

The domain of a Gregory patch is an n -sided polygon in the 2D plane. Previous research 13 shows that the use of irregular polygons reflecting the spatial distribution of the boundary curves generally improves the quality of transfinite surfaces. In this paper, however, we will use regular domains for simplicity's sake. Experience shows that these behave well unless we have extreme boundary configurations with very uneven side lengths or sudden curvature changes.

Let us determine the parameter si for the i -th side. Traditionally, Gregory patches are parameterized by radial sweeping lines 1 (Figure 4a), connecting the domain point in question to the intersection of the extended polygon sides i -1 and i + 1. This is suitable for G 1 continuous surfaces, but further differential properties are required for G 2 continuity. For each point on the i -th side, it is a necessary condition that the parametric speed of the adjacent side parameters s i -1 and s i + 1 is identical, i.e.,

<!-- formula-not-decoded -->

Figure 5: Blend function with m = 2 (left) and m = 3 (right)

<!-- image -->

polant, but the partial derivatives are not necessarily identical, which may pose a problem called twist incompatibility . Let us introduce the notation t i = 1 -s i -1 . We would need

<!-- formula-not-decoded -->

Every curve network satisfies this equation for p = q = 0, as the boundaries meet at a fixed corner point. It is also natural to require that the equation holds for p = 0, q = 1 and p = 1, q = 0, constraining the first cross-derivative at the boundary to the tangent of the neighboring curve. Most networks do not go beyond this point - and even if the curves match a common surface curvature 5 , it only handles the additional cases p = 2, q = 0 and p = 0, q = 2, as well as p = q = 1.

When some partial derivatives of the ribbons are not compatible, we need to apply Gregory's rational twists. These replace the constant vectors Wp , q by rational expressions combining the two parametric variables (see below). In our current research, we assume that the boundary curves match only in position, and for all other terms rational expressions are used. This may create another layer of flexibility for shape optimization.

The correction patch is defined as

<!-- formula-not-decoded -->

where each W is a rational function of si and t i . The computation of these is a fairly straightforward generalization of the classical Gregory twists 15 . As an illustration, we show two such terms, the remaining ones are similar:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Substituting 0 for either si or t i eliminates one of the conflicting partial derivatives.

## 4.4. Blending Functions

Every transfinite surface scheme combines individual interpolants by special blending functions that ensure the required interpolation properties and gradually vanish as we move towards the center of the domain. Gregory patches consist of corner interpolants, so we need corner blend s that interpolate the corner points, gradually fade on the adjacent sides, and vanish on all other sides.

For each ( u , v ) point in the polygonal domain, we determine an n -tuple of distance values D i , computed as the perpendicular distances from the i -th side. Let Di 1 ... i k = j / ∈{ i 1 ... i k } D m j , then the corner blend is defined as

<!-- formula-not-decoded -->

This function satisfies all requirements - it yields 1 at the ( i -1 , i ) corner, ensures a 'gradual' 1 → 0 transition on sides i -1 and i as we move away from the corner, and vanishes on all the remaining sides.

The exponent m controls how rapidly the contribution of a single ribbon changes, compare the two images in Figure 5. It also ensures that the resulting surface retains the ( m -1 ) -th derivatives of the ribbons, so we can use m = 2 for linear ribbons, and m = 3 for parabolic ribbons to achieve G 1 and G 2 continuity, respectively.

## 4.5. Surface Equation

Given a curve network with side interpolants, now we can create all constituents - the corner interpolants, the domain polygon, the parameterization and the blending functions. Putting these together, we arrive at

Figure 6: Connectivity between Gregory patches

<!-- image -->

Figure 7: Improved surfacing using connection curves.

<!-- image -->

<!-- formula-not-decoded -->

## 5. Examples

Figure 6 shows two adjacent patches with linear vs. parabolic ribbons. In this example, the target curvatures were computed by averaging those on the left and right sides. It can be seen, that with parabolic ribbons the curvatures nicely match, and the isophote strips smoothly change across the shared boundary, showing G 2 continuity.

Figure 8: Vertex blend with three master surfaces

<!-- image -->

In the rest of this section, we present some applications of parabolic ribbons.

## 5.1. Difficult Curve Network Configurations

While most of the time G 1 transfinite surface interpolation produces a collection of smoothly connected convex patches, there are certain cases, where in order to handle complex configurations or avoid shape artifacts, we need to insert 'artificial' connection curves into the network:

- curve loops with a concave angle,
- avoiding distortions due to highly curved boundaries,
- connect internal loops (holes).

Examples are shown in Figure 7. In these cases, first we insert connection curves, then create G 1 patches. After taking the average of the curvatures, we compute parabolic ribbons and regenerate the patches now with curvature continuity. These composite patches remain smooth internally and the seamlines are invisible along the connection curves. (Note, that generally connection curves are automatically generated, and remain hidden from the users.)

## 5.2. Master-Slave constructions

There are various methods to compute a target curvature function along a given curve. The most straightforward solution is averaging the curvatures of two adjacent G 1 patches, as before. Another typical situation is, when the curvature of a master patch needs to be retained. Then these curvatures are propagated to the surrounding slave patches using parabolic ribbons. Such an example is shown in Figure 8, where a setback vertex blend was created, satisfying curvature continuity joining three edge blends, i.e., we have three master surfaces, and one slave in the middle.

Figure 9: Deviation from the mesh using linear (left) and parabolic (right) ribbons

<!-- image -->

## 5.3. Mesh Approximation

Creating a concise representation of a mesh is an important task in general topology surface modeling. First, a network of curves is drawn on the mesh, which also defines a multi-sided patch structure. By deducing boundary curves and cross-derivatives from the mesh, transfinite surfaces can nicely approximate the data points in the interior. Using locally estimated normal vectors we can create normal fences and linear ribbons for G 1 patches. If we estimate local curvatures along the boundaries, as well, this makes it possible to create parabolic ribbons and G 2 patches, which will produce smoother and more accurate surface models (see Figure 9).

## Conclusion

We have discussed an approach for G 2 transfinite surface interpolation combining parabolic ribbons. These ribbons match curvatures that are associated with the edges of a general topology curve network. The formulation is based on corner interpolants and a special sweepline parameterization. Continuity issues and the computation of linear and parabolic ribbons were explained in details. A few applications where G 2 ribbons are needed have also been presented.

There are several open issues in transfinite surface interpolation for future research. Ribbon creation is one of the fundamental ones, as they are constrained, but not uniquely defined. We are currently investigating ribbon optimization approaches for surface fairing and obtaining the best possible transfinite approximations over triangular meshes.

## Acknowledgements

This work was supported by the Hungarian Scientific Research Fund (OTKA, No. 101845). The pictures in this paper were generated by the Sketches system developed by ShapEx Ltd., Budapest. The contribution of György Karikó to develop this prototype system is highly appreciated.

## References

1. P. Charrot, J. A. Gregory, A pentagonal surface patch for computer aided geometric design, Computer Aided Geometric Design 1 (1) (1984) 87-94.
2. S. A. Coons, Surfaces for computer-aided design of space forms, Tech. Rep. MIT/LCS/TR-41, Massachusetts Institute of Technology (1967).
3. J. A. Gregory, J. M. Hahn, A C 2 polygonal surface patch, Computer Aided Geometric Design 6 (1) (1989) 69-75.
4. R. Hall, G. Mullineux, Continuity between gregorylike patches, Computer aided geometric design 16 (3) (1999) 197-216.
5. T. Hermann, G. Lukács, F.-E. Wolter, Geometrical criteria on the higher order smoothness of composite surfaces, Computer Aided Geometric Design 16 (9) (1999) 907-911.
6. K. Hormann, M. S. Floater, Mean value coordinates for arbitrary planar polygons, Transactions on Graphics 25 (4) (2006) 1424-1441.
7. K. Kato, Generation of n-sided surface patches with holes, Computer-Aided Design 23 (10) (1991) 676683.
8. K. Kato, N-sided surface generation from arbitrary boundary edges, in: Curve and Surface Design, Innovations in Applied Mathematics, Vanderbilt University Press, 2000, pp. 173-181.

9. H. P. Moreton, C. H. Sequin, Minimum variation curves and surfaces for computer-aided geometric design, in: N. S. Sapidis (ed.), Designing Fair Curves and Surfaces, SIAM, 1994, pp. 123-159.
10. J. Pegna, F.-E. Wolter, Geometrical criteria to guarantee curvature continuity of blend surfaces, Journal of Mechanical Design 114 (1) (1992) 201-210.
11. W. H. Press, S. A. Teukolsky, W. T. Vetterling, B. P. Flannery, Numerical recipes in C (2nd ed.): the art of scientific computing, Cambridge University Press, 1992.
12. P. Salvi, T. Várady, A. Rockwood, Ribbon-based transfinite surfaces, Computer Aided Geometric Design (submitted).
13. T. Várady, A. Rockwood, P. Salvi, Transfinite surface interpolation over irregular n-sided domains, Computer Aided Design 43 (11) (2011) 1330-1340.
14. W. Wang, B. Jüttler, D. Zheng, Y. Liu, Computation of rotation minimizing frames, Transactions on Graphics 27 (1) (2008) 2.
15. A. Worsey, A modified C2 Coons' patch, Computer Aided Geometric Design 1 (4) (1984) 357-360.

## Appendix A: Proof of Continuity

We will provide here a short proof that Gregory patches with matching parabolic ribbons are indeed G 2 continuous. We will prove a stronger statement: the modified Gregory patches described in the paper interpolate their ribbons with C 2 continuity.

To save space, arguments of functions are omitted, when this cannot cause any misunderstanding. From here onwards, we look at a point on the i th side, i.e., si -1 = 1 and si + 1 = 0. In this situation, the blending function has several important properties 12 , which we list here without proof:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

w being an arbitrary direction. From the above it also follows that

<!-- formula-not-decoded -->

## C 0 Continuity

This is very straightforward, using property (1) of the blend functions:

<!-- formula-not-decoded -->

## C 1 Continuity

As before, most of the equation vanishes, leaving

<!-- formula-not-decoded -->

Since Ri + 1 , i = Ri , i -1, we can use property (4) to eliminate the last term. After some calculation, we arrive at

<!-- formula-not-decoded -->

which is a combination of the ribbon's side- and cross-derivatives, proving G 1 continuity. Using the parameterization constraint ¶ s i + 1 ¶ w = ¶ s i -1 ¶ w , we get back the ribbon's first derivative, so we have proved C 1 continuity.

## C 2 Continuity

In a similar vein, we can eliminate a large part of the equation, after which the following remains:

<!-- formula-not-decoded -->

The last term vanishes once again, due to property (5). This leads to

<!-- formula-not-decoded -->

Applying the parameterization constraint, we get

<!-- formula-not-decoded -->

which is the second derivative of the ribbon, so we have proved C 2 continuity.