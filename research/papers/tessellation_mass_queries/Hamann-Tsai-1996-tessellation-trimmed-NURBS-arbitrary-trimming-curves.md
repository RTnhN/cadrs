## UC Davis

## IDAV Publications

## Title

A tessellation algorithm for the representation of trimmed NURBS surfaces with arbitrary trimming curves

## Permalink

https://escholarship.org/uc/item/04n8h0b9

## Journal

Computer-Aided Design, 28

## Authors

Hamann, Bernd Tsai, P.-Y.

## Publication Date

1996

Peer reviewed

<!-- image -->

## A tessellationalgorithmforthe representationoftrimmedNuRBS surfaceswitharbitrarytrimming curves

Bernd Hamann and Po-YuTsai*

In most CAD systems, a trimmed parametric surface is defined by two things the control informution of the surface itself and the control information of its trimming curves,which are usually defined as parametric curves in the parameter space of the surface.Ofien,trimmed parametric surfaces cause problems in the context of data exchange between CAD syslems,surface evaluation/rendering.and grid/mesh genera tion.This paper describes a mew approach for representing a trimmed surfacc.The approach is based on the idea of decomposing the valid part of the parameter space of a trimmed surface by a set of planar,ruled surfaces,which are defined in parameter space and whose union defines the entire valid part. A generalized Voronoi diagram s constructed for the set of trimming curves in parameter spacc. This diagram defines tiles around each trimming curve, which are further tessellated by utilizing a scan line technique for identifying and filling the interior of polygons. Here, the scan line technique is used to extract non-horizontal segments of ihe given trimming curves.Pairs of these segments are then linearly interpolated in horizontal parameter direction, thus defining a set of planar,ruled surfaces.The set of all these planat,ruled surfaces defines the entire valid part ofa trimmed surface exactly. Published by Elsevier Science Ltd

Keywords:bisector,aBs,scan-line algrithm,trimmed surface,Voronoi diagram

## INTRODUCTION

This paper is concerned with the conversion of parametric surfaces containing trimming curves to a set of

Department of Computer Science, University of Califomia,Davi, CA 95616-8562,USA

Paper meceied: 28 February 1995. Recised 30 April 1995

NSF Engineering Research Center for Computational Field Simula tion,Mississippi State University,PO Bo 6176, Missisippi Suste,MS 39762,USA

approximating surfaces that do not contain any trimming curves.Trimmed surfaces arise frequently in real-world applications. Typically,they are the result of surface-surface intersection (SSD.Complex geometries are defined in terms of thousands of parametric surfaces which might intersect each other. The intersection curves are usually defined in the parameter space of the surfaces,e.g.by a set of planar Bezier, B-spline, or NuRBs (non-uniform rational B-spline) curves.

This paper introduces a new approach for the representation of trimmed surfaces.The complement of the part that is'cut out' by the trimming curves is defined by mcans of decomposing the valid part of the parameter space into a set of four-sided regions.In the following. only tensor product surfaces will be considered. They are denoted by

Many CAD systems cannot represent a trimmed parametric surface implicity,ic.as a paramctric surface with the trimming curves defined in its parameter space. This causes a problem when exchanging trimmed surface data between CAD systems.A solution to this problem is the representation of the valid part (i.e. the part that remains when disregarding all holes implied by the trimming curves) by a set of basic parametric surfaces,ie.parametric surfaces without trimming curves. Furthermore,the representation of a trimmed surface by a set of basic surfaces simplifies grid/mesh generation and surface rendering.

<!-- formula-not-decoded -->

where d=（x）and）and（o）can be Bernstein-Bezier polynomials,B-spline basis functions, or even rational B-spline basis functions.It is assumed that s is C° continuous.

The 2D closed trimming curves in parameter space are denoted by

<!-- formula-not-decoded -->

where d²=（u,u）is one of the（n+1）control points of trimming curve C,and e（0)-e（1）.It is assumed that the rotation number ofall trimming curves is +1. which implies that they have the same orientation. Each trimming curve must be at least Ccontinuous but can contain tangent discontinuities.A trimming curve must not interscct another trimming curve and must not self-intersect.In most practical applications, there is one trimming curve cnclosing the region that contains all the other trimming curves,which is assumed to be c.If this enclosing trimming curve is not cxplicitly defined,the boundary of the parameter space is chosen to be cie.cis the piecewise linear curve given by the four line scgments u-0,-1,-1,and =O).Figure / shows the trimming curves ofa trimmed surface in physical and parameter space.

The approach described in this paper is similar to the construction of planar Voronoi and power diagrams in the sense that a tessellation of the valid part of the parameter space of a trimmed surface is computed. These diagrams are described in detail in References 3-6.This paper presents a new method for the construction of the curved boundaries (bisecting curves) defining the tiles associated with the trimming curves. A computationally efficient technique is used for gencrating a finite sct of points on cach bisecting curve. The technique is based on shortest distance computations for a finite set of points on a rectilinear grid in parameter space.

Much work has been done regarding the generalization of Voronoi diagrams.Most of the generalizations in 2D deal with the construction of Voronoi diagrams for sets of points,line segments,polygons,circles,and more general planar curves.The construction of Voronoi diagrams for such sets is discussed in Refercnces 7-14. Somc of these papers address the relntion between Voronoi diagrams and the medial axis transform (MAT). The efficient construction of a generalized Voronoi diagram for aset of trimming curves method described in this paper.

An algorithm for rendering trimmed surfaces by us- ing quadrilateral and triangular clements is described in Reference 15.In Reference 16,a 2D mesh generation algorithm is described that automatically discrctizes 2D regions containing trimming curves based on the identification of certain geometrical features of the trimming curves,e.g.slope discontinuities.A technique utilizing a combined'triangulation-quadrangulation' strategy of the valid part of the parameter space ofa trimmed surface is presented in Reference 17.A method for representing a trimmed Nuras (non-uniform rational B-spline) surface by a set of Bczier patches is discussed in Reference I8 the problem of data exchange is addressed: trimmed rational surfaces are approximated by non-rational surfaces.

Fgure 1Trimming curves in pbysical let and parameter (rigt） space

<!-- image -->

The algorithm discussed in Reference 19 must consider errors introduced by the piecewise linear approximation of a given trimmed surface,while the algorithm presented in this papet exactly represents the valid part ofa trimmed surface asa union ofso-calledparameter surfaces,Le.analytical,planar NURBs surfaces（u(，）, （.））. These parameter surfaces can be viewed as an additional"layerused for the indirect represcntation ofthe valid part ofa trimmed NuRes surface.Both algorithms utilize a scan line algorithm for the generation of data inside the valid part of a trimmed surface and the generation of the boundary curves of parameter surfaces,respectively.

Recently. an clegant algorithm for the polygonal approximation of trimming curves and the approximation of the valid part of a trimmed surface by a set of triangles has been introduced in Reference 19.The algorithm determines the set of triangles-defininga piccewise linear surface approximation-based on a user-spccified tolerance e, which cnsures that the approximation,in physical space,does not deviate by more than e from a given trimmed NuRas surface.

The underlying concepts of parametric curve and surface design used in this paper are covered in Refercnces 1,2 and 20.Various solutions to the SSI problem rithms is provided in Reference 22. Recent developments in surface and volume grid generation are surveyed in References 23 and 24,which also address the problems that arise when having to generate grids for trimmed surfaces.

## PROBLEMSTATEMENTAND DEFINITIONS

The trimming curves define a simply connected region in parameter space.The goal is to represent this region by a set of planar,four-sided surfaces whose union is the valid part of the parameter space. In the following. such a surface will be referred to as a porameter srfece.It is denoted by

<!-- formula-not-decoded -->

Figure 2Concept of parameter surfaces

<!-- image -->

where d=（）Thusthe partof sufce s that is implied by the paramcter surface u, is given by

<!-- formula-not-decoded -->

Figure 2 shows a trimmed surface with one hole represented by four parameter surfaces that are shaded in different grey scales.

The main problem to be solved is the generation of the boundary curves of the parameter surfaces. This problcm can be solved using a gencralization of the Voronot diagram ofa point set.When dealing with trimmed surfaces,the trimming curves define the set for which a tessellation,a generalized Voronoi diagram must be computed. The tile boundaries in a planar Voronoi diagram implied bya point set are obtained from the\_pcrpendicular biscctors of all possible point pairsGeneralizations of thisstandardVoronoi diagram are obtained when the clements for which a tessellation is to be constructed are points,line segments,circles,polygons,and more general curves. Fig ure3 showsVoronoi diagrams for a set of points and a setof circles5

Voronoi diagrams introduce riles around cach clement (points,line segments,circles etc.）according to some distance measure.A tile is defined as the region that contains all the points being closer to a particular element than any other clement.The tile boundary can be used to subdivide a tile into a set of four-sided planar surfaces whose union represents the area between the clement and the clement's tile boundary.

Figre 3Vorooi diagrsms for set of poinis snd set of circles

<!-- image -->

Figure 4Four-sided parametersurfaces in tiles around several trimming curves

<!-- image -->

Each four-sided surface can be constructed by subdividing the tileboundary curve into segments and generating additional curves connecting end points of the tile boundary segmcnts and points on the clement. This principle is shown to Figure 4.

The following definitions are needed when using the Voronoi diagram for the representation of trimmed parametric surfaces.

## Definition 1

Given a set of planar, closed, pairwise non-intersecting curveswith rotation number+1 such that no curve lies in the interior of any other curve,the locus of points that have a smaller shortest (Euclidean)distance of curve c,than to any other curve is called the tile T of . denoted by T（c）.

Ifall curves chave a continuous tangent the smallest shortest distance is equal to the pependiclar distance.Assuming that there are K such curves,T（e） is obtained as the intersection of K-1 half-spaces,ie.

<!-- formula-not-decoded -->

where H(cc,）is the half-space containing all points that have a smaller shortest distance to c than to c.

## Definition 2

Given K curves as in Definition 1,the K tiles T（e） define the genenalized Voronoi diagrum The curve separating the two half-spaces H（e.c，）and H（e,c）is called the bisector of c,and e,，The intersection points ofbisectors are called Voronoi certices.

Once the generalized Voronoi diagram (referred to as Voronol diagram in the following) has been computed for a set of trimming curves all tiles must be The boundary curves of the parameter surfaces are line segments, segments of the bisectors,and segments of the trimming curves.It is also possible to use segments ofthe curves defining the medialaris （or skeleton）of

Figure 5 shows the generalized Voronoi diagram for five arbitrary curves in the plane.

Figure 5  Generalized Voronoi diagram for five curves

<!-- image -->

the tiles as boundary curves of the parameter surfaces25,26. It is pointed out later why this can cause problems for the general case. The MAT has been used in computer vision and pattern recognition in the context of thinning algorithm25-28.

## Definition 3

Let R be a simply connected 2D region and B be its boundary.A point p inR's interior belongs to the medialaxisorskeletonofRifthereare(atleast)two points on the boundary B having the same distance from p and being boundary points with shortest distance from p.

The medial axis of a tile can be used for the construction of parameter surfaces. Certain segments of the medial axis can be used to define the boundary curves of parameter surfaces that start on a bisector and end on a trimming curve. Once the boundary curves of each parameter surface are generated its control points d, (see Equation 3) will be obtained by interpolating its four boundary curves. The chosen interpolation method must ensure that the parameter surface lies completely inside the region enclosed by its four boundary curves. Figure 6 shows the Voronoi diagram for a set of arbitrary trimming curves and the medial axes inside each tile.

## COMPUTINGTHEVORONOIDIAGRAM FOR A SET OF TRIMMING CURVES

An efficient algorithm is needed for the generation of the tile boundaries around each trimming curve in parameter space. First, tiles are constructed around the trimming curves c1, C2, C3...., and ck. The final tiles are obtained by intersecting the tile boundary curves in the Voronoi diagram for c1, C2, C3...., and ck with the enclosing trimming curve co. This general configuration of closed trimming curves is characteristic for all real-world cases of practical relevance.

Initialy, the trimming curve co is not considered in the construction of the Voronoi diagram. The generationof theVoronoi diagram isbased onthecomputationof theintersectionsofbisectorswiththeedgesina triangulation of the parameter space [0, 1] × [0, 1]. The vertices in this triangulation are labelled according to the index of the closest trimming curve. The labels are usedtodeterminewhetherthereisanintersection between bisectors and edges in the triangulation.The intersections between the edges and the bisectors are computed and properly connected,thus defining an initial approximation of the Voronoi diagram.

Figure 6 Voronoi diagram (light) for set of trimming curves and medial axes (dark) in each tile

<!-- image -->

Multiple bisectors can intersect the same edge in the triangulation,and multiple bisectors can intersect in the interior ofa triangle. These cases are addressed by the algorithm described bclow.The algorithm does not consider the case of one bisector intersecting the same edge multiple times.One does not need to consider this case due to the fact that only a discrete point （line scgment) approximation of the bisectors of the Voronoi diagram is to be computed.The approximation of the Voronoi diagram requires thcse steps:

- Construction ofa triangulation of the parameter spacc[0,1]×[0,1].
- (ii） Labelling each vertex in the triangulation with the index of the closest trimming curve,
- (i)Extraction of all triangles whose three vertices all lie in the valid part of the paramcter space,
- (iv) Computation of intersections between biscctors and cdges in the triangulation using a recursive subdivision strategy.
- (vi） Computation of intersection between bisectors and curveC.
- (v) Computation of intersections of bisectors,
- (vi） Generation of piccewise linear and cubic B-spline approximations of all tile boundaries in the Voronoi diagram.

These steps are now described in greater detail.Denotingthe minimal distance of all possible pairs of trimmingcurves by d,the initial triangulation of the parameter space only contains cdges that are shorter than d/2.This is accomplished by subdividing the parameter square [0,1]x[0,1] into squares whose diagonal is shorter than dm/2 and splitting each square into two triangles,Only triangles whose thrce vertices all lie in the valid part of the parameter space are considered for the followingcomputations.

The labels at cach vertex in the triangulation are one bisector.It is possible that the three labels at a triangle's vertices are the same,that there are two different labels,or that they are all different.In the first case,it is assumed that no bisector intersects the triangle.In the second casc,it is assumcd that there are bisectors intersecting the two edges whose cnd points have different labels.In the third case,it is

Each vertex in the triangulation is labelled according to the closest trimming curve.The square of the dis tance dbetween a vertex with coordinate vector （x.y) anda trimming curve c（r）is given by d²（r）-（xu（r）²+（y-u（r））²,∈[0,1]Thecritical pointsof d²（t） are identified,and the associated distances are computed.In addition,one computes the distances to those points on e,where slope discontinuities occur. The index of the trimming curve that has minimal distance to （x,y）is used as the label for this vertexIt turns out that the case of multiple trimming curves all having minimal distance to (x,y) docs not require a special case treatmcnt.

assumed that there are biscctors intersecting all three cdges.

Points lying on bisectors in the Voronoi diagram are computed based on Algorithm 1 for the recursive subdivision of the parameter space triangulation:

<!-- image -->

Figure 7 shows the results of Algorithm 1 when curves,

<!-- image -->

A piecewise linear approximation of the Voronol diagram is obtained by connecting the points resulting from Algorithm L. lf exactly two edges ofa triangle each contain one point on a bisctor,denoted by P and P,then Pand p.are connected.If all three cdges ofa triangle each contain one point on a biscctor, denoted by PPand p.then P.P.and pare assumed to be lying on three different bisectors.In the second case,each of the three points is connected with the point q that is the intersection of three bisectors. An iterative method is used to approximate the coordinates ofq-

The centroid of P.P,and p,dentoed by q.is used as the initial approximation of q Subsequent approximations q' are obtained in the following way:Given an approximation q',one computes its closest point x,on C，closest point xon C,and closest point x，on assuming that the curves C,,and c, are the ones closest to p.P，and p.respectively.One computes the critical points of thc squared Euclidean distance between q'and the three trimming curves (using Newton'smethod) and identifies the three closest points x x,and x. Considering a single trimming curve,there might be multiple points on it having minimal distance toq'.It is not important which point is chosen.lIf the centre of the circle passing through x，,andx,lies in the interior (or on one of the cdges)of the triangle （p,P.P),the centre is chosen asthe value of qIf the centre of this circle lies outside the triangle (P P

Fgure 7 Recursive subdivision of parameter space triangulations for generation of points on bisectors in Voeooi dinram

<!-- image -->

P),the unique point on the triangle's cdges having minimal distance to the circle's centre is chosen as the value of q+1.Due to the fact that there must be an intersection point of three bisectors in the triangle,the method will converge.The method terminates when the Euclidean distance between q'and q+is smaller than e.

Based on the piecewise linear approximation of the Voronoi diagram and the curve scgments of c,a cubic B-spline approximation is constructed for all tile boundary curves.In the following, the tile boundary curve associated with the trimming curve e,is denoted bye,The cubic B-spline representation of e,is based on a chord length parametrization defined by the lengths of the line segments in the piecewise linear approximation.Figure 9 shows the cubic B-spline curves approximating the tile boundaries for the example shown in Figure 8.

Whenever Algorithm 1 leads to a triangle whose longest edge is shorter than e (one of the termination criteria of the algorithm) the centroid of such a triangle is considered to be the intersection ofbisectors.Eventually,one obtains a piecewise linear approximation of all bisectors in the Voronoi diagram. Figure 8 shows the piccewise linear approximation of the Voronoi diagram for a set of trimming curves.This cxample has an cnclosing trimming curve e.The piecewise linear approximation of the Voronoi diagram is intersected with the cnclosing trimming curve C. and the resulting curve segments on care used for the definition of thc tile boundary curves around c,.,y....,and g

Remarkl.In practical applications,onc is often concerned with interior trimming curves that are not closed and intersect the outer trimming curve e.lf this is the case,the intersections between those trimming curves and care computed,and the resulting curve segments on ein combination with the trimming curves that are not closeddefine a new enclosing trimming curve cOnce this new curve c,has been computed,the construction of the Voronoi diagram follows the same principle as discussed above.Figure 10 shows a configuration in which the trimming curve e,is intersected by four trimming curves.Trimmed periodic surfaces (ie.surfaces for which s0,u）-（s（1,o)and/or su,0）-s（u,1）） do not require an additional case distinction.

Flgure 8Piecewise linear approcimation of Voronoi diagram for cireufar, non-coeves, and slope-discoetimuous trimming curves

<!-- image -->

Figure 9 Cubic B-spline approximation of Voronoi diagram for ciroular, non-convex, and slope-discontinuous trimming curves

<!-- image -->

Once the Voronoi diagram is known,cach tile can independently be decomposed into parameter surfaces. Thus,all tiles can be processed in parallel and only two curves （c,and e,）have to be considered for the construction of the parameter surfaces inside a particular tile.

## COMPUTINGTHEBOUNDARY CURVES FORPARAMETERSURFACES

The next step is the decomposition of the tiles in the Voronoi diagram bounded by the outer trimming curve c）into parameter surfaces.The union of all parameter surfaces defines the valid part of the trimmed surface.The computation ofan approximation of the medial axis of a tile can be extremely expensive.Furthermore,cven if one knows the medial axis of a tile,it is not quite clear how to cxtract segments from it to be used for the construction of the boundary curves of parameter surfaces and how to interpolate the boundary s Therefore,the algorithm used for the construction of the parameter surfaces is based on decomposing a tile intoa set of surfaces that have two horizontal boundary curves.The algorithm is similar to thc scan line （or scan conversion)algorithm used for filling the interior of2D polygonsand to the construction of the Reeb graph representing the topology'skeleton'of manifolds with holes Assuming that the trimming curves are C continuous,the generation of the boundary curves of the parameter surfaces inside the tile associated with trimming curve c (having tile boundary curve )requires these steps:

Figure 1o Trimming cures interecting enclosing trimming curve

<!-- image -->

The basic idea for the construction of the parameter surfaces inside the tile associated with a particular trimming curve c is as follows:The valid part of the parameter space,le.the region between the trimming curve c and the tile boundary curve E,is represented by as sct of ruled panuneter surfaces.They are obtained by identifying local extrema in u-direction (and horizontal line segments) on c and ,computing the interscctions between horizontal lines passing through these local extrema (and the horizontal line segments) and c and and constructing ruled parameter surfaces from the resulting horizontal line segmcnts inside the tile and non-horizontal curve segments on c and .

- Determining extrema in o-direction and end points of horizontal line segments on e and
- ( Computing all points p=（n,o）on c and  representing local extrema in u-direction.
- (i） Computingallend points q=（u,）andr-（,u), u&lt;,of all horizontal line segments qr on c and

- Computing intersection points between horizontal semi-infinite lines (passing through the local extrema on c and  and e and E;computing intersection points between horizontal semi-infinite lines (defined by the horizontal line segments on c and ) ande and
- (ii） Computing all intersection points between the two scmi-infinitelinesL（r）-p+（--0）and R（r）=p+（r,0）,r∈[0,x].andeand.
- (iv) Computing all intersection points between the two semi-infinite linesL(t）=g+（-r,0）and R（r）=r +（t,0）,r∈[0,=].andcand.
- Selecting those intersection points to be used as corner vertices of the ruled parameter surfaces
- (v)Determining the intersection point x on L(r) closcst to p and the intersection point y on R(r) closest to p (x andy being different from all points P.q and R),provided that both L(） and R（r） start at a local extremum:discarding all other intersection points x on L(r)and y on R（r).
- (vi) Determining those points x(y) for which the horizontal line segments xp and xq (py and ry,respectively)lie in the interior (including the boundary curves) of the tile associated with e; discarding all other intersection points x and y.
- (vi Determining the intersection point x on L(r) closcst toq and the intersection point y on R(r) closest to r(x andy being different from all points P.q.and r), provided that L(t） starts at the left and R（r) starts at the right end point ofa horizontal line segment;discarding all other interscction points x on L(r)andy on R（r）.
- Constructing a graph (vertices and cdges) for the set of local extrema,end points of horizontal line segments, and the selected intersection points,assuming there are Lscan lines and N points (considering local extrema,end points of horizontal line segments,and selected intersection points) on the kth scan line:
- （viD Ordering the points p,q,r,xand y resulting from Steps (i）-（vii） acoording to their u-and u-coordinates (ie.ordering thcm according to the scan lines they lie on and from left to right on each scan line)by constructing the ordered vertex where the point  precedes the point ,provided that vs v-coordinate is smaller than v,,s -coordinate or provided that v,su-coordinate is equal to v,,s v-coordinate but v,'s n-coordinate is smaller than ,s -coordinate,e. V:=（o&lt;D）V（-DAu&lt;H）
- （ix)Constructing a graph Gwith vertices Yand horizontal edese+-.y+1..k-1..,, j-1...,N-1, where only those edges are stored that lie in the interior of the tile associated with e (the boundary curves are considered part of the interior).
- （x）Ordering the points ，on the curve c-c（r） （and on the curve -c（r)）with respect to increasing parameter value (i,respectively)and—according to this order-addinganedge ,to G for cach pair of successive points and,on c（and on ),where and,,must not be end points of a horizontal line segment on c （or ).
- Ordering the horizontal edgesof Gfrom left to right on each scan line and generating ruled parameter surfaces by applying linear interpolation to pairs of associated non-horizontal curve scgments on c and c:
- (xi） Considering all edges ek&gt;k，k=1... L-1,with end points all lying on onc scan line and end points v,all lying on another scan line,ordering edges inu-direction &lt;uAu，s≤g.).
- E being the even number of non-horizontal boundary curves of ruled parameter surfaces having one end point on the kth scan line and the other end point on the sth scan line,generating and storing the B-spline control information for each ruled parameter surface defined by the two horizontal line segments V boundary curves,which are the exactly extracted -1 betweenVand..E/2 curve segments between V y-iandv F-j and

The localextrcma in the c-direction on c and c,computed in Step Gi),are not always characterized by a horizontal curve tangent.They can coincide with points where tangent discontinuities occur.Step (i) actually considers all'nearly horizontal' line segments on e and ,ie.line segments whose absolute slope is smaller than somc tolerance e.Figure l/ shows the localextrema in p-direction (solid points)and the end points of horizontal line segments (circles) on two simple curves c and e defininga Voronoi tile.

When computing the intersections between semi-infinite horizontal lines and c and E (Steps (i and (iv)), one must consider the existence of'nearly horizontal curve scgments on c and E that lie inside an e-strip to both sides of one of the semi-infinite horizontal lines. If such nearly horizontalcurve segments are found, they are not considered for the computation of intersections.The intersection points determined in Steps These points lie on the boundary curves of ruled parameter surfaces. Figure /2 shows the intersection points (stars) remaining after Step (vi) (same example as shown in Figure I/).

Flgure I1Local extrema in s-direction and end points of horizontal line scgments on c and

<!-- image -->

Figure 12Selected interection points betwcen semi-iefinite horisontal lines and c and

<!-- image -->

Steps （vi）-(x） define a set of （non-intersecting) edges/segments connecting local cxtremn,cnd points of horizontal curve segments,and selected intersection points. These edges/segments are horizontal line segments inside the tile,horizontal line segments on c and c.and non-horizontal curve segments on e and E.By ordering the non-horizontal edges e in n-direction (Step （ri） one generates the topological information necessary to define the four boundary curves ofeach ruled parameter surface.The four boundary curves of a ments. given by successive horizontal line segments on two different horizontal lines and two non-horizontal curve scgments on c and EFigure 3 indicates the single horizontal line segments (solid lines)and non- horizontal curve segments (dashed curves) for the example shown in Figure I1.

Flgure 13 Horizotal line segments and non-horizontal curve segments defining boundary curves afruled parameter surfaces

<!-- image -->

The final boundary curves for each parameter surface are assembled in Step (xi）. Since the two nonhorizontal boundary curves of each ruled parameter surface are curve segments on c and cone must represent these curve segments as single curves,e.g.as NURas curves.If one chooses to use a NURBs representation,it is necessary to represent the two non-horizontal boundary curves using the same degree and the same knots.This is achieved by clevating the degree of the lower-degree boundary curve to the degree of the knots of the two curves.Eventually, the ruled parameter surface u,(f.n）is given by

<!-- formula-not-decoded -->

whered²=（u）.d²=（a²)）ande（n）and R.（m）are the analytical representations (e.g.the NURBs representations)of its two non-horizontal boundary curves（seeEquation3）.Thecurves（） and ,（） have the same degree and knots.

Figure l4 illustrates some of the curvilinear grids obtained when evaluating parameter surfaces u,uniformly (same cxample as shown in Figure I/).

Remark 2. There are two main reasons why ruled surfaces are being used for the definition of the parameter surfaces u（,）.First,the scan line algorithm used to define thc upper and lower boundary curves of each parameter surface leads,in a natural way, to a ruled surface defined by the left and right boundary curves. Higher-order interpolation schemes would only lead to redundant control information. Second,thc construction of the left and night boundary curves of each parameter surface leads to curves that do not contain any local extrema in u-direction nor any horizontal line segments;thercfore,performing linear interpolation in u-direction leads to a parameter surface that is a oneto-one mapping from （.n）to （u,u）-space.This property is important in order to more easily generate grids that do not contain intersecting grid lines Remark 3.The union of all paramcter surfaces inside a particular tile locally represents the valid part of the parameter space without error. This is due to the fact that the trimming curves are given as NuRBs curves-defined in parameter space-and that the left and right boundary curves ofeach parameter surface are obtained by cxtracting segments from the given trimming curves.Using knot insertion and degree elevation algorithms allows the cxact representation of these segments.This,in turn,implies that the valid part of the parameter space is represented exactly.

Figure I4Curviineat grids of parameter surfaces resulting from upitorm evaluation

<!-- image -->

Flgure 15 Trinmed surace with oc tinming curve

<!-- image -->

Rerark4.Figure 14 clearly illustrates that a uniform cvaluation of the parameter surfaces in their respective parameter spaces （（f.v）spaces)does not lead to a uniform grid point distribution in (u,）-space throughout n tile nor to grids whose constituting grid lines match in a continuous fashion along the shared boundary curves of two parameter surfaces.This is nor the goal of the described merhod.The method presented here is purely used to define the valid part of a trimmed surface in an analytical,indirect manner.The connectivity information ofall parameter surfaces and socalled point distribution functions are uscd in a later grid generation step to obtain a certain point distribution and continuous grid lines,Grid generation methods have to take care of those parameter surfaces having cdges that degenerate to points.

## EXAMPLES

The described method has been tested for various cases,including various real-world configurations. The method is robust and is completely automatic.Figures 15-I8show four examples of trimmed surfaces （left: shaded version of trimmed surfaces:right:parameter space configuration). The trimming curves, the tile boundary curves of the generalized Voronoi diagrams, and the resulting curvilincar grids in paramcter space are shown.The parameter spaces shown in Figures 17 and 18 （on the right side in cach figure) correspond to the fuselage.and the trimming curves indicate the fuselage-wing interscction(s).

Figure 16Trimmed surfsce with mtiple trimming curves

<!-- image -->

Figure 17Trimmed surface (fuselage) with one trimming curve

<!-- image -->

Figures 15 and 16 show the same cubic B-spline defined by 4x4 control points.The two trimming curves in Figure 75.one of them being an outer enclosing trimming curve,are two cubic B-spline curves defined by 11 (inner trimming curve）and 9 control points (outer trimming curve).The three trimming curves in Figure 16 are cubic B-spline curves defined by 9,9,and 11control points.Figures 17 and 18 show the same cubic B-spline surface defined by 133x445 control points (fuselage).The one trimming curve in Figure 17 is a cubic B-spline curve defined by 63l control points. The three trimming curves in Figure18 are cubic B-spline curves defined by 631, 9, and 9 control points.

Remark5.The method has been implemented in Con an SGI Indigo2 Extreme graphics workstation.The examples shown in Figures 15-18require approximately 3s3.5s1s,and5s,respectively,for the generation of the control information of all parameter surfaces. Due to the fact that the cxample shown in Figre 15 contains an outer enclosing trimming curve, the times required for the cxamples shown in Figures 15and 76 are nearly the same.For grid generation purposes,the additional layer provided by the parameter surfacesis stored and used later for the generation of grids for the valid part of a trimmed surface.In general, this tessellation method is not meant to be used for interactive applications but meant to be used purely for the conversion ofgiven trimmed parametric surfaces to a set of surfaces without trimming curves by means of the additional parameter surface layer.

Remark 6.For each ruled parameter surface,one must store only one knot vector, which is the knot vector shared by the left and right boundary curve,and the NURns control points of the left and right boundary curve. Provided that all trimming curves are originally defined as cubic B-spline curves,which is usually the case,one does not need to perform degree clevation Assuming that the original knot vectors of the left and right boundary curves （=segments of cubic B-spline curves) ofeach parameter surface are entirely different,the number ofcontrol points required to represent the left and right boundary curves of all parameter surfaces is approximately twice the number of control points defining the original trimming curves.This fact is independent of the number of paramcter surfaces needed to represent the valid part of a trimmed surface.

Figure 18Trimmed surface (fuselage)with multiple trimming curves

<!-- image -->

## CONCLUSIONS

Amethod for representing the region between the trimming curves in the paramctcr space of a trimmed surface has been presented.A generalized Voronoi diagram is computed for the set of trimming curves, and the resulting tiles associated with each trimming curve are then furthcr subdivided into ruled parameter surfaces whose boundary curves are defined by horizontal line segments and non-horizontal curve segments of the trimming curves and tile boundary curves,

The strategy used for representing a single tile by a set of ruled paramcter surfaces can also be viewed as a strategy for generating the block boundaries (vertices and edges)ofa multi-block decomposition ofa 2D region.It is planned to extend this strategy for generating the block boundaries （vertices,edges,and faces) ofa multi-block decomposition ofa3D region.Most likely,many of the blocks resulting from the strategy will lead to unacceptable grids.Therefore,one must subdivide the blocks further.This aspect will be investigated in the future.

Altermatively,one might want to interpolate betwecn segments on the tile boundary curves and segments on the trimming curves once the generalized Voronoi diagram has been computed. lf one decides to do this,one must ensure that the chosen interpolation method guarantces a one-to-one mapping between the （.n） space （the parameter space of a parameter surface) and the （u,）-space （the parameter space of a trimmed surface).It will be investigated in which cases a tile can directly and easily be decomposed into parameter surfacesusing transfinite intcrpolation'or conformal mappings".

## ACKNOWLEDGEMENTS

This work was supported by the National Grid Project consortium and the National Science Foundation under contract EEC-8907070 to Mississippi State University. Special thanks go to the members of the research and development team of the National Grid Projcct. which was performed at the NSF Engineering Research Center for Computational Field Simulation,Mississippl State University. The reviewers comments have improved the paper significantly.

## REFERENCES

- Farin.G Cwrves and Swrfocer for Cooyutr Aided Geomeic Desigw 3rd Ed.) Aeademic Press, San Diego, CA (1993)
- 3Aurenhmmer,FPower dirms:algorithms,and applications SLAMJ.Compng Vol 16（1987)pp78-96
- Hoschck」and Laer,D Fdme of Coper Aied Gemc Desgn A K Peters,Wellesley.MA (1993)
- EdelsbrunnerHAlgorimu  CombintrialGeomety Springer-Verlag.New York,NY (1987)
- 6 Preparata,F P and Shumos,M L Computario Geomy (3rd Ed Springer-Verlag.New York. NY（i990)
- Facello,M AIplementation of a randomized algoeithm for Delaunay and regalar triangulations in three dmensions'Copel Aidnf Geom. De (1995) (to appear)
- 7Farouki,R T and Johnstone, J KThe bsector af = point and a plane parametric curve' Compc Aided Geom. Des. Vol 11 No 2 (194)pp 117-151
- Leven,D and Shara,MIntersection and proximity problems and Vonoi diams Schrtz,]nd Yap,CK （Es) Adnce n Roboti Vol I Lawence ErhaumLondon,UK （1986）pp 187-228
- Lee,DTand Drysdale,R LGeneralization of Voronoi digrams in the planeSLAM J Comug Val 10 No 1 pp 7J-87
- 10 Sharir, M "Intersection and cosest-pair problems for a set of discr²SLAM J. CompuingVol 14 No 2（1985）pp 448-468
- 12 Srinivasan,V.Nackman,LR.TanJ M.and Meshkat,S N "Automatic mesh generation using the symmetric axis transformation of polygonal domains²Proc IEEE Val s0 No9 (1992) pp 1485-1501
- 11 Srinivasan,V and Nackman,LRVoronol dingram for muipy-connected pohygonal domains I:AlgorithmIBM J. Ret. Develop. Vol 31 No3 (1987)pp 361-372
- 3 Yap,C KAn O (n log n] lgorithm for the Voronni diagram of a set of simple care segments' Di Compur Geom Vol 2 (1987) pp365-393
- 14 RockwoodA P,Heaton,Kand Dvis,TReal time rendering of trimmed surfces4CM Trn Gnph.Vol 23 (1989)pp 107-116
- 16 BahmannP L,Winchen,S L.ShehardM S,Grice,K Rand Yerry,M ARbustgeometrically bsedauatic o-dimnsional mesh generationIn.Num.Meh Eng Vol 24 （1987）pP 1043-1078
- 15 Klein,R,Mehlhorn,K and Meiser,S Rndonized increment construction ofabtract Voronoi diagramsComput.Geom. Theor.4ppl Vol 3 (1993) pp 157-184
- 17 Vries-Baayems, A E and Seebregts,C H "Exact coenion of a trimmed nonrational Bezier surface into composite or hasic norational Bzier surfaces in Hagen H (Ed) 7opicr n Sface Modeling SIAM,Philadelphin,PA (1992） pp 115-143
- 19 Piegl.L A and Ricbard.A M Tessellating trimmed URBs suraces'CoutidedDes. Vol 27 No 1 (1995)pp 16-26
- 18 Hoschek,Jand Schneider,F J'Spline coeversion for trimmed rational Bezier and B-spline sarfaces' Coput-Aided Des Vol 22 (19900pp 580-590
- 20 Bochm,W and Prautzsch,H Numericl Mhad A K Peters, Wellesley.MA (1994)
- 22 Patrikalakis.N MSurface-to-surface interectionFEEE Com pat Graph Appl Vol 13 No 1 (1993) pp 89-95
- 21 BamhilL R E Gey Proeng for Degnn Mfacring SLAM,Philadelphia,PA (1992)
- 23 Thopson,J F and Weatherill N PAspects ofnumcrical grid geseratioe current science nd art ProcIh ALA4 Applied Aeonamic Conferee Monterey,CAAugus (1993)
- 24 WeatherillN P,Hassan,O,Marum,D L nd Marchant,M J Grid gcncration by the Delaunay triangulation'von Karman Institute for Fluid Dynamics19931994 Lecture Series,January (1994)
- 26 Man,D Representing visual informationAL Mmo 4I5,Artifcial Intelligence Laboratory,Massachusets Institute of Technology.Cambridge,MA (Mny 1977)
- 25 Blum, H Biological shape and visual science (part I  Theor. Biof. Vol 38 (1973) pp 205-287
- 27 Pavidis, T Sowcneu Pamem Recogiion Springer-Verlag. New (2Z61)ANA

- 28Rosenfeld A nd Kak,AC D Picre Proceing Academic Pres.New York,NY (1976)
- C Kuni T Land Shinagan,YVisualistion:new concepts and NM（E Sfic Vinn gfPcalPhemea
- Foley,JD,n DamA,Feir,S Knd Huges,JF Cor Grapu (2nd Ed) Addion Wesley,Reading.MA (1990)
- 31Pieg L AOn NuRs a surveyEEE Com GphApVol 11 No 1 （1991)pp55-71
- 33 Thompsn,J F,WarsiZ U Aand Mastin,CWNumericaf Gd Generatiow North-Holland,New York,NY (1985)
- 32Pieg. L A 'Rational B-spine curves and surfaces for CAD and gaphisin Rogers,D Fand Eashw,RAEd） Se ofe pp225-269
- 34 George,P L Automaic Mesh Generain Wiley,New York, NY (1991)
- 351 Knupp, P M and Steinberg. S L Fidamemb of Grif Gemer now CRC Press, Bocs Raton, FL (1993)

<!-- image -->

ndan MS in compuer science from he Techica Uei af Brachweig Gmar He rceiedhir FhD in compter sciece fm Arion Se Unrsi. US4, in F991.

Bemd Hnn  an ascie profesor in eDepn oCo Science at the Unicerty of Calfoia Daps an adunt prfer in he Deparmt af Computer Sctence andn NSF Engineing Resrarch Center for ComputanonalFild Simulonon Misppi Se Ury.US4. H cumet rescarch and teaching interests ariefie ualocomper aphicand compter-ded geori egHereeeBS comper scienct,a BS nmhemics Po-Y T   PhD sde  c tational ergnerrg  Maipp Ste Lincrin US4 His cur reh inerest are computer-aided geometrie design and scieufie iation. He receied BS ineconoms an managemer infotion es fm FFew Caolic Unier Tann Reb lChina He ee an MS in cowputer acience from Miuripp State U.UA,n J992

<!-- image -->