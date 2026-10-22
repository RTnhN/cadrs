## Flends: Generalized Fillets via B-splines

Zachary D. Eager ∗ and John F. Dannenhoffer III † Aerospace Computational Methods Laboratory (ACML) Syracuse University, Syracuse, New York, 13244, USA

The methods for creating transition surfaces (e.g. fillets, chamfers, etc.), as implemented in many solid modelers, are restrictive and fragile. For example, the extent of a fillet on the two surfaces are linked and cannot be specified separately. Also, both fillets and chamfers yield curvature discontinuities where they meet the original surfaces, which is known to be sub-optimal aerodynamically. In this paper, we present more robust and flexible methods for creating transition surfaces using B-spline surfaces. These methods are solid modeler agnostic, however we've implemented them using the Engineering Sketch Pad (ESP).

## I. Introduction

A common desire in solid modeling is the ability to create a transition surface between two adjacent faces of a solid, as well as creating a transition surface between two faces of two disjoint solids. Currently existing approaches available in most solid modeling software include fillets, chamfers, sweeps, lofts, etc. These methods typically take minimal input from the user and are thus restrictive and can additionally be highly fragile depending on the complexity of the geometry. Our goal is to create methods that allow the creation of transition surfaces that are more flexible and robust than the traditional methods. The solid modeling software we will focus on in this paper is the Engineering Sketch Pad (ESP) [1]. We note that the methods presented here are not inherently specific to this software and could be implemented for any system that provides analogous supporting functions.

## II. Generalized Fillets

Fig. 1 Demonstration of unequal exposure on the original surfaces by a fillet.

<!-- image -->

The issue with fillets and chamfers is that they are, by definition, restrictive. The user input for these methods is typically a fixed radius, resulting in minimal control over where fillet/chamfer meets the original surfaces (see Fig. 1). Furthermore, since fillets are created by circular arcs of the input radius and chamfers are ruled, there is inherently a discontinuity in curvature where the fillet/chamfer meets the original surfaces. It can also be common for the solid modeler to fail when creating a fillet/chamfer involving complex geometries.

∗ Ph. D. Student, Mechanical and Aerospace Engineering

† Associate Professor, Mechanical and Aerospace Engineering, AIAA Associate Fellow

<!-- image -->

In this section, we develop a method for producing a generalized fillet surface where the user has control over the curvature of surface as well as more control over where the generalized fillet surface meets the original surfaces. We also demonstrate how this method can be more robust than ESP's fillet and chamfer methods. For the remainder of this manuscript, this generalized fillet surface will be referred to as a 'flend.'

## A. Blending Surface Generation

Consider the simple case where we wish to create a flend between two intersected boxes (see Fig. 2(a)). We assume at this stage that the locations where the flend meets the original surfaces has been determined, denoted by the offset edges on the two boxes (see Fig. 2(b)). For this example, the offset edges are 0.25 units from the intersection edge such that a fillet or chamfer of radius 0.25 will join the two surfaces at these edges.

Fig. 2 The original intersected boxes (a) and the intersected boxes with the offset edges and corresponding points (b).

<!-- image -->

We begin by determining a set of corresponding points for an edge group. An edge group consists of the first offset edge, the intersection edge, and second offset edge (the order of offset edges is arbitrary). Therefore, the number of edge groups is equal to the number of intersection edges (e.g. in Fig. 2(b), there are four edge groups). Assuming that each of the three edges in a group is parameterized on t 2 » a ; b … , the corresponding points would be the points on each edge for a given value of t . We will call these points P A ; P B ; and P C , respectively (see Fig. 2(b)).

Next, we compute P I , the three-plane intersection [2] of: (1) the plane that is parallel to the surface at P A , (2) the plane perpendicular to the intersection edge at P B , and (3) the plane that is parallel to the surface at P C .

Consider first the 2D plane formed by the points P A , P I , and P C (see Fig. 3). In this plane, we define a clamped, cubic B-spline (of equal knot spacing) with 5 control points ( P 0, P 1, ..., P 4), and introduce a parameter called 'dip' 2 » 0 ; 1 … for controlling the curvature. We take the beginning and end points ( P 0 and P 4) and place them at the points where the B-spline will meet the original surfaces, P A and P C , respectively. For the second point ( P 1), we take the distance between P 0 and P I (the intersection point) and multiply it by the dip level. We choose point P 1 to be this distance away from P 0 in the direction of P I (tangent to original surface at P 0), P 1 = P 0 + j P 0 P I j dip. The fourth point ( P 3) is determined analogously using P 4 and P I . We then take the distance between P 1 (computed previously) and P I and multiply it by the dip level. We choose point P to be this distance away from P 1 in the direction of P I , P = P 1 + j P 1 P I j dip. P is determined analogously using P 3 and P I . Finally, we choose P 2 to be the midpoint of P and P , P 2 = P + P 2 .

In order to create the 3D flend, we sample as many corresponding point sets as desired (as many points required to accurately represent the offset edges) for an edge group and compute the cubic B-spline control points for each corresponding point set. We then construct a B-spline surface which is degree 1 in one direction (along the intersection edge) and degree 3 in the other. The control points of the B-spline surface are the sets of the cubic B-spline control points computed previously; the control points ( P 0, P 1, ..., P 4) are the points in the degree 3 direction. Note that choosing P I as we did, and moving the inner control points ( P 1 and P 3) towards this point, ensures C 1 continuity where the B-spline surface meets the original surfaces [3]. This process is repeated for each edge group, and the collection of surfaces created from each edge group makes up the flend. For the two intersecting boxes example, the flend consists of four B-spline surfaces stitched together. Fig. 4(a)-(f) show flends for various values of dip in juxtaposition to the fillet and chamfer. We observe that zero dip recreates the chamfer and a dip value of about 0.3 strongly resembles a traditional fillet for the given configuration. Note that, for higher values of dip (see Fig. 4(f)), flends do not have the strong discontinuity in curvature that is inherent when using fillets.

Fig. 3 5 control-point B-spline curve generation schematic.

<!-- image -->

## B. Offset Curve Generation &amp; Intersection

Suppose now that we are not given the offset edges on the surfaces to be joined. Furthermore, we would like to choose each offset edge to be an arbitrary distance ( d 1 and d 2, respectively) from the intersection edge, independently of the dip.

For a given point on an intersection edge, we compute the corresponding offset point on the offset surface (the surface that will contain the offset) as follows. First, determine the outward normal on the offset surface at the given point as well as the direction vector on the intersection edge at this point. Typically, 3D surfaces are parameterized by two values, u and v . At any point on a surface we can query the derivative vectors w.r.t u and v , i.e. h dx u ; d y u ; dz u i and h dx v ; d y v ; dz v i . The cross product of these yields the outward normal h dx ; d y ; dz i at that point on the surface. Further, the cross product of the outward normal on the offset surface and the direction vector h r x ; r y ; rz i on the intersection edge results in a new vector h x ; y ; z i which represents the desired traversal direction on the offset surface. See Fig. 5 for a pictorial description.

Beginning at the given point on the offset surface, we solve the following set of equations for h du ; d v i :

<!-- formula-not-decoded -->

4

5

4

5

Since this is an over-determined system we compute the least-squares solution [4]. The resulting h du ; d v i is the direction vector in uv-space on the offset surface that most closely matches the desired direction vector h x ; y ; z i . We then take a 'small' step in this h du ; d v i direction on the offset surface. Since the directions of the unit vectors in uv-space are not constant w.r.t to xyz-space, we can not simply move the desired distance d in this direction. Instead, after taking a small step we re-compute h dx u ; d y u ; dz u i and h dx v ; d y v ; dz v i at this new ' u ; v ' location and re-solve the equations for h du ; d v i and take another small step in that direction. We repeat this until we've travelled the desired distance d along the offset surface from the given point on the intersection edge. We compute the offset points for an arbitrary number of points along the intersection edge (as many points as necessary to accurately represent the offset edge) and fit a cubic B-spline to this set of points. This B-spline curve is the offset edge on the offset surface.

Fig. 4 Flends for various values of dip as well as the fillet and chamfer for two intersecting boxes. Colors showing maximum curvature.

<!-- image -->

Performing the aforementioned steps using the other surface (adjacent to the intersection edge) as the offset surface yields the offset edge on that surface. We emphasize that the distances for the offset edges on each face can be chosen independently. Together, the intersection edge and two newly created offset edges form an edge group (described in the previous subsection). Once we have determined all of the edge groups from the intersection edges, we can apply the logic outlined in the previous subsection to create a flend.

Fig. 5 Offset direction schematic.

<!-- image -->

Fig. 6 Example where offset edges (denoted as red dashed lines) intersect.

<!-- image -->

It is often the case that offset edges will intersect. It is necessary to detect this when creating the edge groups so that the resulting flend does not intersect itself. Consider the example in Fig. 6 where we wish to create a fillet about the two intersected bodies. Clearly, the offset edges (denoted as red dashed lines) will intersect in the region shown. In order to detect this we currently apply a two-step process.

First, we take the two intersecting offset edges ( P 1 ' t ' = ' u ; v ' and P 2 ' t ' = ' u ; v ' ) and divide each into several segments using the running parameter t for each. We determine the ' u ; v ' on the offset surface at the beginning and end of each segment for each offset edge and linearize each curve in uv-space using these points for each segment. We then check all pairs of the linearized segments for intersection. If the points describing the first line segment are ' u 1 ; v 1 ' ; ' u 2 ; v 2 ' and the second are ' u 3 ; v 3 ' ; ' u 4 ; v 4 ' , we can then apply the following formula [5]:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

If 0 t 1 1 and 0 t 2 1, then the linearized segments intersect at t 1 on the first segment and t 2 on the second. Now, we perform a multivariate Newton's search [4] to find the intersection of the offset edges using the t 1 ; t 2 found previously as the initial guess. We define two functions which are both functions of two variables:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Essentially, these functions measure the distance between the u and v components of P 1 and P 2 which we require to be zero. The iterative scheme for Newton's method is

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where the Jacobian is

Once we have found t 1 ; t 2 to the desired accuracy, we trim the offset curves P 1 and P 2 to these parameterized values.

<!-- image -->

Fig. 7 Comparison of fillet and flend for box-'oval' intersection. Colors show maximum curvature.

<!-- image -->

(b) Flend (offset distances of

d

1

=

d

2

=

0

:

15

,

dip

=

0

:

3

).

Fig. 8 Comparison of fillet and flend for cylinder-'horseshoe' intersection. Colors show maximum curvature.

<!-- image -->

(a) Cylinder-NACA4412 intersection, the region of interest shown in (b) and (c) is denoted in red.

(b) Failed fillet (radius, r = 0 : 25 ) at trailing edge.

<!-- image -->

(c) Flend (offset distances of d 1 = d 2 = 0 : 25 , dip = 0 : 3 ) at trailing edge.

<!-- image -->

Fig. 9 Comparison of failed fillet and flend for cylinder-NACA4412 intersection. Colors in (b) and (c) show maximum curvature.

## C. Demonstrations &amp; Comparisons

This section contains several examples of flends, demonstrating their flexibility and robustness in comparison to fillets.

Fig. 7 demonstrates the ability to choose the offset distances (i.e. the location where the transition surface meets the original surfaces) independently. Fig. 8 further shows how the offset edges computed using our method remain a fixed distance from the intersection edge, whereas the width of the fillet fluctuates (note the inscribed curves in Fig. 8(a) that are the offsets for the flend in Fig. 8(b)). The example in Fig. 9 of the cylinder intersected with a NACA 4412 airfoil is a subtle but important demonstration of robustness. In Fig. 9(b), we observe that the fillet on the trailing edge of the airfoil does not join the fillet on the wing at the same location, whereas the offset distances for the flend (see Fig. 9(c)) are identical for all surfaces on the airfoil.

## III. Flends about disjoint bodies

Auser may wish to create a smooth transition surface between two faces on disjoint bodies (see Fig. 10), as opposed to transition surfaces between adjacent faces discussed in the previous sections. This is, however, essentially the same problem as discussed in the previous section. The only catch is that we do not have the intersection edges as a guide for generating the transition surface (i.e. the edge on which P B lies). Therefore, we we need to develop an additional method for creating this faux intersection.

Fig. 10 Adjacent and normal face definitions.

<!-- image -->

## A. Blending Surface Generation

First, the directions at the ends blending surface (where the blending surface meets the original surfaces) must to be determined. The current approach for determining the desired direction is to find the most perpendicular vector to the normal vectors on the adjacent faces. We begin by computing m normal vectors equally distributed along the edges on the adjacent faces (see Fig. 10), which can be expressed as A = hfi v 1 ; fi v 2 ; :::; fi v m i . The most perpendicular vector fi x is then any non-trivial solution to A T fi x = fi 0, which can be computed via SVD [4].

Starting from a point on the adjacent face (i.e. P A or P C ), the final direction vector ( fi d A or fi d C ) is in the direction of the adjacent face's slope (uv-space) which most closely matches the most perpendicular vector fi x (xyz-space), such that the transition surface will be slope continuous where it meets the adjacent faces.

In order to construct the transition surface. we need some sense of an 'intersection' between corresponding direction vectors found in the previous step. To do this we, we compute the points on each of the lines formed by these direction vectors that minimizes the distance between them and take the average. If we express these lines parametrically as P A + t fi d A and P C + s fi d C , the points on each line which minimize the distance between them can be computed as [6]

<!-- formula-not-decoded -->

respectively. We use the average of these points as the faux intersection point, that is P B = ' C A + C C ' 2.

Once we have this faux intersection point, we apply the methodology for creating the control meshes discussed previously. Thus, we have a method for creating transition surfaces between two disjoint bodies which allows the user to parametrically control the resulting curvature. Note that we have assumed the number of edges on each of the bodies being flended to be equivalent. Furthermore, we assume that the corresponding edges on each of the faces are known (or can be determined using additional logic).

## B. Demonstrations &amp; Comparisons

In each of the cases shown, the original bodies are those on the left and right and the blending surfaces are those in the center (joining the original bodies). Fig. 11 demonstrates this method being used on two boxes of the same dimensions where one is translated slightly in space. Fig. 12 demonstrates this method on two cylinders of the same dimensions where one is translated and rotated.

## IV. Conclusion &amp; Future Work

We have developed methods that allow the creation of transition surfaces (dubbed 'flends') that are more flexible and robust than traditional methods. Moreover, we first constructed a method for creating transition surfaces between two faces which takes as input the desired level of dip. This parameter allows the the user to control curvature of the resulting surface, namely the discontinuity in curvature where the transition surface meets the original surfaces. We further expanded on this method by describing how the locations at which the transition surface meet the original surfaces can be chosen directly and independently of one another. Lastly, we developed a supplemental method that allows the creation of transition surfaces between disjoint. Through several examples, implemented in ESP, we demonstrated the flexibility and robustness of these methods in comparison to the ESP's methods. We reiterate that these methods are not specific to any solid modeler and can be implemented for any system with the supporting functions used herein.

Fig. 12 Flend between two cylinders.

<!-- image -->

There are several improvements to these methods that we wish to explore. First, C 2 continuity where the transition surfaces meet the original surfaces would be strongly desirable for certain applications. We would like to give the user the ability to enforce this. Next, the methods considered in this manuscript are only for creating transition surfaces between two surfaces. Also, a user may wish to create a flend about three faces (e.g. the corners of a box). We are currently working on a concise supplement to these methods that allows for this. Lastly, for the method discussed in Sec. II, we require the intersection of the bodies to be flended to be computed a priori. For complex geometries, this can be computational expensive and error-prone. Thus, we would like to be able to create the flend between the two un-intersected bodies.

## References

[1] Haimes, R., and Dannenhoffer, J., 'The engineering sketch pad: A solid-modeling, feature-based, web-enabled system for building parametric geometry,' 21st AIAA Computational Fluid Dynamics Conference , 2013.

- [2] Goldman, R., 'INTERSECTION OF THREE PLANES,' Graphics Gems , edited by A. S. GLASSNER, Morgan Kaufmann, San Diego, 1990, p. 305. doi:https://doi.org/10.1016/B978-0-08-050753-8.50065-6, URL http://www.sciencedirect.com/ science/article/pii/B9780080507538500656 .
- [3] Piegl, L., and Tiller, W., The NURBS Book , Springer-Verlag, Berlin, Heidelberg, 1995.
- [4] Press, W. H., Teukolsky, S. A., Vetterling, W. T., and Flannery, B. P., Numerical Recipes in C (2nd Ed.): The Art of Scientific Computing , Cambridge University Press, New York, NY, USA, 1992.
- [5] O'Rourke, J., Computational Geometry in C (2nd Ed.) , Cambridge University Press, New York, NY, USA, 1998.
- [6] Weisstein, E. W., Skew Lines , MathWorld - A Wolfram Web Resource, 2018. URL http://mathworld.wolfram.com/ SkewLines.html .