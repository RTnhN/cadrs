## Shelling and Offsetting Bodies

## Mark Forsyth

D-Cubed Ltd, 68 Castle Street, Cambridge, U.K.

## Abstract

Offsetting is a procedure where a uniform thickness is added or removed from a solid model. Shelling is a procedure closely related to offsetting which turns a solid model into a thin-walled shell of constant thickness. This 'shelled' solid may be closed or open (a hollowed out solid). These procedures have important applications in mould creation, tolerancing, N.C. tool path generation, clearance checking and so on.

operator cannot be expressed aa the same boolean combination of offsets of the primitives (see [3, 5, 6, 7]). Rossignac and Requicha [5] introduced CSGO (CSG with offsetting) to get round this, where the offsetting is added as an operator in the CSG tree. Farouki [3] gives descriptions of the offset of the boundary of solids of extrusion and of planar, convex polyhedra.

Geometry useful in offsetting, such as offset curves, offset surfaces, curves offset along surface normals and rolling ball blend surfaces are reviewed. A description of the offsetting operation on boundary representation (Brep) solid models is given. A practical algorithm for offsetting an important sub-class of Brep solids is presented. In this slgorithm, face/face re-intersection and blending are used to offset all or a subset of the faces of the solid, An algorithm for shelling is also given which uses this offsetting algorithm. If all the faces in a closed solid are included then a closed shell results. Some of the faces may be omitted, in which case an open shell results.

## 1 Introduction

Offsetting operations have many industrial applications such ss tool path generation for numerically controlled machines, mould creation, clearance checking and so on. Offsetting adds or removes a constant thickness to or from a solid model. Shelling is a procedure whereby a solid model is converted into a thin-walled shell. This shelled solid can be open or closed.

Offsetting operations on Constructive Solid Geometry (CSG) modellers has been considered by Rossignac and Requicha, [5]. The problem for CSG modeliers is that, if a solid is some boolean combination of primitives, then the offset

Permission to copy without fee all or part of this material is granted provided that the copies are not made or distributed for direct commercial advantaqe, the ACM copyright notice and the title of the publication and Its date appear, and notice is given that copyin is by permission of the Association of Computing ? Machinery. o copy otherwise, or to republish, requires a fee and/or specific permission.

Solid Modelin '95, Salt Lake City, Utah USA 4 0 1995 AC 0-89791 -672-7/95/0005.,.$3,50

The geometry of offset curves and surfaces is well documented and has been studied for over 300 years (Kreyszig [10] cites a treatise on parallel curves given by Leibniz in 1692), This paper will give a brief survey (section 2) of offset geometry, concentrating on the properties useful for the description of offset solid models.

In section 3 a general description of the offset of a Boundary Representation (Brep) solid will be given. Section 4 will introduce the shelling operators. In section 5, an algorithm for offsetting all or a subset of the faces of a solid is given. Finally, shelling algorithms are given, which use this offsetting algorithm. The offsetting algorithm presented will work for a sub-claas of Brep solids. The algorithm requires:

- If a vertex of a solid lies on a number of surfaces then the offset of these surfaces must intersect in a number of discrete points which lie on all the offset surfaces. This is guaranteed if there are three surfaces at a vertex.
- The offset solid does not self-intersect.

The algorithms presented have been implemented in the Hewlett-Packard CAD system, SolidDesigner.

## 2 Offset Geometry

First consider offsetting geometric entities. This involves creating a geometric entity (like a curve or surface) on which every point is a fixed distance from a base geometric entity. There are several possibilities and those useful in offsetting and shelling are described below.

## 2.1 Planar Offset Curves

Given a planar curve C(t) define the curve offset from this in the plane by a constant (non-zero) distance v. The offset curve, Co(t), lies in the plane and is defined as

<!-- formula-not-decoded -->

where

- N.(t) = Z x T(i) is the curve unit normal.
- Z is the unit normal to the plane
- T(t) is the curve unit tangent.

These are also known as pamllel curves and for a detailed analysis one is referred to [1] and [2].

Note that these offset curves can be very complex even for simple curves. For example, [2] shows that there are algebraic representations both C(t) + qNc(t) and C(t) -vNc(t) together when C is an polynomial or rational curve, but that one of the branches on its own is not in general an algebraic curve, even for simple curves such as parabola and ellipses.

## 2.2 Offset Surfaces

Given a surface 9(u, u) (in the rest of this paper this shall be referred to as the base surface), then the offset of S by a constant, non-zero distance, q, is defined as

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

and Su is the first partial derivative with respect to u e.t.c. In the literature these are known as pamllel aurjacea.

The normal to the offset surface, No, satisfiea the following relation,

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and kl (u, v) and kz(u, o) are the principal curvatures of the surface S(u, u). For a more detailed analysis see, for example, [4, 8, 9, 10]. Note the following :-

- Given a position P on the offset surface at parameters (s, t), (i.e. P = @(s, t)) then let,

<!-- formula-not-decoded -->

then P' lies on the surface S, and has parameters (s, t) (i.e. P' = S(s, t)). This is a useful property of our definition (2). We will refer to this as a pammeter preserving oflset.

- Ne is parallel or anti-parallel to N.
- If either kl = I/q or kz = I/q then the offset surface normal is not defined. In this situation there is a cusp or ridge on the offset surface. If both kl and kz are greater than or both kl and kz are less than l/q then Ne is parallel to N. If one is larger and one smaller then No and N are anti-parallel.

## 2.3 The Offset of a Surfiace Singularity

Let S(u, v) be a surface (defined on [UO,u1] x [UO, u1]) which haa a geometric singularity (i.e. a singularity which is not removable by a reparamet risation). Assume that S has a degenerate u boundary, i.e. S.(UO, v) = O, Vu. Obviously the analysis is similar for degenerate u boundaries.

At this boundary N(u, v) is not uniquely defined. However, a normal can be defined Vv E [VO, vi]. This can be done using, for example, L'Hbpital's rule, since both the numerator and denominator tend to Oat the singularity in the formula for N(u, u). Note, in general, N(uo, t) # N(uo, s) for s # t.

So at the surface singularity there are a range of normal values, therefore the singular point offsets into a curve on the offset surface. That is, if V = S(UO,v) (Vv), the singular vertex, then the offset of V, E3v is

<!-- formula-not-decoded -->

which is a curve on e(u, u). For example, it is easy to demonstrate (and visualise) that if S(U1 u) is rotationally symmetric (like a cone, torus or spun surface) then @Iv(t) is a circle.

Figure 1: Outward offset of elliptical cone

<!-- image -->

Consider the offset of a (half) elliptical cone. Outward (i.e. q &gt; O) and inward are shown (see figures 1 and 2). Note the apex offsets into a non-planar curve. From the

Figure 2: Inward offset of elliptical cone

<!-- image -->

definition (2),

<!-- formula-not-decoded -->

so at the singular point,

<!-- formula-not-decoded -->

or

and so

<!-- formula-not-decoded -->

Therefore C3V(~) lies on the sphere IX VI = IvI (i.e. a sphere centred at V with radius Iof). Also the sphere normal is given by

<!-- formula-not-decoded -->

thus at X = @v(t) = @(uo, t)

<!-- formula-not-decoded -->

Since N is parallel to the offset surface normal, N,P~ is paraIlel to the offset surface normal. So there is a tangent intersection between the surface @(u, u) and the sphere IXVI = Iv! and this is namely, @v(t) = V + qN(uo, t).

In figure 3 the example offset elliptical cone and it's tangent sphere are viewed from the side.

Figure 3: Offset of elliptical cone with tangent sphere

<!-- image -->

## 2.4 Singularities on the Offset Surface

As stated above, if the offset distance is equsl to one of the principal curvatures of the base surface there will be a cusp or self-intersection in the offset surface. Since, near to a singularity the curvature of the base surface is very high, then cusps and self-intersections in the offset surface are likely.

Note from (4) and (5), both yl(u, v) and 72(u, v) are equal to +1. Assume that on the base surface kl becomes infinite at a singularity, but kz remains either greater than or less than l/q such that 72 is constant. If ~ &gt;0 then

- a) If kl &lt; 0, kl ~ -00 as one approaches the singularity, then 1k] q &gt;0 and N@ is always defined.
- b) If kl &gt; 0, kl -00 as one approaches the singularity, then if at any stage kl &lt; l/q there will be a point where 1 kl q = 0.

Examples:-

<!-- formula-not-decoded -->

Right Circular Half Cone: Here kQ = O and kl E (O, -cm). Thus 72 = 1, and k, = -cos(a)/R(v), where a c (O,%/2) is the cone half angle and R(u) &gt;0 is the radius of the cone at a parameters (u, v). So there is a degeneracy when R(v) = -rjcos(a). So if rj &gt; 0 then there are no self-intersections. If ~ &lt; 0 then there is always a self-intersection.

Torus: For a torus, of major radius 4 and minor radius h, then

<!-- formula-not-decoded -->

(where C(u) is a circle radius, a, and unit vectors X(u), Y(u)) and can write kl = cos(v)/(a -bcos(v)) and kz = -I/b. From these

- I. If ~ = -b then the offset torus degenerates to a circle. If 9 &lt; -b then the offset torus will be turned inside out.
2. If q &gt; a b then the offset torus will self-intersect.
3. Ifq&lt;-a -b then the offset torus will selfintersect.

Self-intersecting surfaces are problematic for boundary representation solid modellers, thus these surfaces have to trimmed back to the self-intersection where possible.

## 2.5 Rolling Ball Surface

This is the surface which is a constant distance from a curve. It is often also called a pipe surface or a conai surface. These can be defined as the envelope of spheres centred on the curve (thus the name rolling ball surface).

This surface is commonly used in solid modelling to blend two faces. This surface is defined from a (spine) curve, C(t), and a radius value, q. It is defined as,

<!-- formula-not-decoded -->

where B(u) and N(u) are the unit binormal and normal to C(u). Note, a sphere whose centre lies on the spine curve and has the same radius as the rolling ball, R(u, v) intersects R(u, v) in a tangent circle.

Also (see [9] pp 251,252),

<!-- formula-not-decoded -->

where ~(u) is the curvature of C(u). This implies that q &lt; rnin~~(u)'] I to ensure the roiling ball does not self-intersect or kink. The normal is then,

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

Assuming the condition on q, the normal to R(u, u) can be written simply as,

<!-- formula-not-decoded -->

## 2.6 Offset of Curve on a Surface

Given a curve, C(t), on a surface, 9(u, u), the curve is offset along the surface normal by a constant distance, q.

Since the curve lies on the surface define relations u x u(t) and v * v(t) such that

<!-- formula-not-decoded -->

and at any point on the curve the surface normal is defined as

<!-- formula-not-decoded -->

This allows the following oflset curve to be defined as,

<!-- formula-not-decoded -->

These curves have the following well known properties useful in solid modelling, all of which have simple proofs.

Tangent Edge Property If two surfaces intersect tangentially along a curve, C(t), then their offset surfaces will intersect tangentially along the curve, Co(t).

Blended Edge Property Given two surfaces, S1 and S2 intersecting along a non-tangent curve, C(t), then let R be a rolling ball with spine curve C(t) and C@be the offset curve of C, along the normal of S1. Typically, in a solid model, there would be faces on the surfaces and their intersection would be an edge. Then

- Co lies on R.
- Co is a tangent intersection between R and the offset by q ofsl.

Vertex property Given two curves Cl (t) and C2(T) on a surface S(u, u) which intersect at a point V = Cl (to) = Cz(m ), with offset curves C! and C:. Then define two rolling ball surfaces RI and R2 with spine curves Cl and C2 (given by (8)), respectively. The circular isoparametric curves R1 (to,s) and Rz (ro, c) are

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

where N1 and B1 are the unit normal and blnormal to the curve Cl(t) at t = to, Similarly for N2 and Bz.

Then the offset of V is V + q N is an intersection of the offset curves, C: and C!, and is one of the intersections of the circles, El and E2.

## 3 Offsetting Solid Models

The process of offsetting adds or removes a uniform thickness, q, to or from a solid model. If q &gt; 0 then the offset operation adds material, and removes material if q &lt;0.

In Brep modelling only the boundary of the offset solid need be considered. The boundary of the offset solid is a subset of the offset of the boundary of the original solid (see, for example, [5] for more details). For example, when offsetting faces meeting at a concave edge (see figure 4) there are pointo on the faces whose offset is inside the solid and so do not form part of the boundary of the offset solid. Thus the offset of the boundary must be trimmed bock to where it self-intersects.

Throughout the rest of this paper, it will be assumed that q is less that the smallest radius of curvature of a surface supporting a face or any curve on an edge in the solid model. An exception is made, when the offset of vertices at surface singularities are discussed. It will also be assumed that the body is a manifold solid. Consider what happens to the offset of faces, edges and vertices.

<!-- image -->

\

J

Figure 4: Offset Boundaries

## 3.1 The Offset of a Face

A face is a finite region of a surface bounded by loops of edges. Each face has a sense S = +1 such that the sense times the surface normal points away from the solid's interior. So to offset a face it's surface must be offset by the signed dist ante

<!-- formula-not-decoded -->

If the face has surface S and is bounded by a sequence of n edges with curves Ci (i = 1, , . . . n) and m vertices Vj (j=l ,. ... m) then the offset face has surface @', n edges C: and m vertices V$. Where

~' is the offset surface of S(u, v) by distance q, given by (2),

C! is the offset curve of Ci by distance VSgiven by (15),

<!-- formula-not-decoded -->

It is easy to see that if Vj is a vertex on edges with curves C~ and Ci+l then V$ lies on C! and C~+l.

Notice that we have described the offsetting of the boundary of a face, and these bounding offset curves C! will not, in general, represent the offset of the face edges.

## 3.2 The Offset of an Edge

The offset of an edge (not to be confused with the offsetting of the boundary of a face) is all the points a distance q from the edge. All the points a distance q from the edge's curve is a rolling ball surface of radius q with spine curve as the edge curve. Note this rolling ball surface may not appear in the resulting offset solid.

First assume we are offsetting outwards i.e. offsetting in the positive face normal direction. If there are two faces, Y1 and %2 (with offset surfaces @I and 02, respectively.) meeting at the edge (with curve C(t), say) then the curve C(t) offsets into two curves C! and C:, as described above. N.B. C! lies on the offset of 71 and C: lies on the offset of 72. There are three cases, depending on the convexity of the edge.

Tangent Here C! = C: and the offset of the edge is a single edge. The rolling ball surfaces touches @l and @a tangentially along the curve C!.

Convex Here C! # C:. Both C! and C: lie on the rolling ball (see section 2.6). So the offset of the edge generates a face bounded by the curves C! and C: (also by where the rolling ball meets the offset of the vertices of the edge) which has the rolling ball as it's surface.

Concave Again C! # Cf. But in this case the rolling ball is always 'inside' the offset model (see figure 5) and need not be considered. The offset edge is where the two offset faces intersect.

Figure 5 shows sectioned views of edge offsets.

Figure 5: Offset of an Edge

<!-- image -->

If we are offsetting inwards then the convex/concave situations are reversed. That is, tangent offsets to tangent, convex offsets to an edge and concave offsets to rolling ball surface.

## 3.3 The Offset of a Vertex

The offset of a vertex depends on how many edges meet at the vertex and the convexity of those edges.

Geometrically, the offset of a vertex is a sphere, that is all the points, X, a distance, q, from a single position (the vertex position, V) is the sphere IX VI = Iql.

However, this sphere may never appear in the resultant offset model. Similar to the concave edge case, the sphere may be entirely inside the offset model.

## 3.3.1 Isolated Singular Vertices

If the vertex is at an isolated surface singularity then the vertex offsets into either a segment of a sphere or a point or curve of self-intersection. If the face offset distance, q~, (given by (16)) is greater than zero then there are two cases.

- If the singular vertex is 'convex' with respect to the face normal (i.e. is like a sharp point on the solid) then the offset is a segment of a sphere. This sphere meets the face's surface along the curve given by (6).
- If the simzular vertex is 'concave' with resuect to the face norm-al (i.e. is like a dimple on the ~olid) then the offset is a point or curve of self-intersection of the offset surface.

These cases are reversed if q, is less than zero.

## 3.3.2 Vertex Offsets

In general, the offset of a multiple edge vertex can be very complex indeed. The claasic example which illustrates the situation is a four sided pyramid with a rectangular base. The apex of this pyramid has four convex edges, if we offset inwards then these edges will offset into four edges (since we are offsetting convex edges inwards, we get no rolling ball contribution). However, because the base is rectangular these four offset edges will not intersect in one point. In fact the offset inwards of a rectangular pyramid is a 'wedge' shape. The vertex splits into an edge and two vertices on offsetting. Note the following:-

- The offset of a convex edge is a rolling ball surface radius Iv[, with spine curve as the edge's curve.
- The offset of a vertex is a sDhere (centred at V. radius Iql). Since the vertex lie; on the edge's curve, this sphere tangentially intersects the rolling ball of the edge. The intersection is a circle radius Iql, centre V and the normal to the circle's plane is the edge curve's tangent at V.

If we consider offsetting outwards only. Offsetting inwards, is the same if we swap all references to conuez with concave and vice-versa. Then :-

- . Every conuez edge generates a rolling ball surface.
- o Every tangent edge generates a tangent edge.
- Every concave edge generates a concave edge.
- o If there are more than two convex edges or the vertex is at a surface singularity there will be a face with a spherical surface. This will meet the rolling ball surfaces of the convex edges along tangent circular edges and singular surfaces along the curve given by (6).
- Where there are concave edges, further edges may be required. These happen when offset faces and rolling ball surfaces adjacent to the concave edge intersect. Figure 6 shows a complex vertex region with a concave edge. Here the two rolling balls (offset convex edges) have intersected creating an edge.

Algorithmic difficulties arise when there are concave edges. If the vertex has any number of convex or tangent edges then it can always be offset into a spherical face bounded by circular edges. Offsetting concave edges can require the addition of many edges between faces which did not intersect prior to the offset operation.

The offsetting of edges and vertices into rolling balls and spheres can be viewed as a blending problem. Here instead of offsetting convex edges their supporting faces are offset and intersected, these intersections can then be blended with a constant radius. Offsetting is often seen as a means to blend or fillet a solid (e.g. [5]), we invert this and use blending to generate the offset solid model.

## 4 Shelling

Only completely solid bodies (i.e. bodies which have no interior voids) make any sense for this operation. The simplest case is to shell a body, so that we change the body to have an interior void a constant distance from the original. A more complex form of shelling is to permit faces to be opened. Here, some faces are not offset creating an open shell. These operations are very useful in mould creation.

## 4.1 Closed Shelling

This is a simple application of the offset operation. The faces of a body form a closed shell. These are then copied, ofiset, negated and added to the body. Th~s results in a closed, thin-walled shell, If the original solid is M and it's offset is Af then the closed shelling operation is the solid S formed by the difference:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where -M is the complement of M.

## 4.2 Open Shelling

Again, faces of the body are offset, but the set of faces to be offset does not form a closed solid. Opened faces are not offset. If a face, ~ shares an edge with a face to be opened then it's offset must also intersect the opened face (otherwise the operation makes little sense). The effect of this is to change the opened face into a number of thin striplike faces (see, for example, figure 7). The edges on a face are formed into loops (i.e. a closed, simply-connected set of edges). If an opened face has N{ loops then this will be split into at least l@ faces by the open shell operation. The sense of the opened face may need changing depending on the convexity of the edges of the opened faces. Three cases are considered depending on the convexity of all the edges of a loop.

- All Convex The opened face's normal remains in the same direction.
- All Concave The opened face's normal is reversed.
- Differing Convexity If there are n, (n &gt; 1) changes of convexity then the opened face is split into n -1 faces. Each of these faces is supported by the same surface, but those arising from concave edges have the opposite face senses.

Figure 8 shows sectioned opened solids. Let M be the original solid, and be bounded by the set of faces {~1, . . . . =n, 01,... Om} where ~i are faces to be offset and Oj are faces to be opened. Let hf be the (partially) offset solid, assuming the set of faces {Ff, ,.. , F:, 01, . . . 0~} (where Ff are the offsets of 71) forms the boundary of a closed solid. An open shelling operation is the solid S formed by the difference:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Figure 6: Offset of a Complex Vertex Region

<!-- image -->

Figure 7: Opened Faces

<!-- image -->

mm

IL?J

<!-- image -->

If!!!ll

Figure 8: The Effect of Edge Convexity on Opened Face

## 5 Offsetting A Body

An algorithm is given to offset a number of faces from a solid. This can be used to offset a solid body by offsetting all the faces of the body.

## 5.1 Offset of a Number of Faces

The algorithm to offset of a number faces is described by the nine sequential steps as follows:

1. Generate offset surfaces of the surfaces of each face. Offset the surface of each face by the distance Sq where S is the face's sense.
2. Generate offset curves for any tangent edge, where we can (see sec. 5.1, 1). For each edge that haa not had an offset curve computed:
- Recompute geometry of the edge by surface/surface intersections.
4. o Let N' be the number of intersection curves found for the edge.
3. Generate offset positions for any vertex where we can (see sees. 5.1.3 to 5.1.6), For each vertex that has not had it's offset position computed:-
6. . Recompute geometry of the vertex by curve/surface intersections.
- Let NP be the number of intersection points found for the vertex.
4. Attach offset surfaces to faces.
5. If for an edge N' is 1 then attach that geometry to the edge.
6. If for an vertex Np is 1 then attach that geometry to the vertex.
7. If there are any edges or vertices which have not had their new geometry attached (i.e. where N= or Np &gt; 1) then the criteria given in 5.1, I are used to select unique curves and positions for the edge and vertices.
8. Check the solid does not self-intersect.
9. If 0&gt;0 then blend all convex edges with a radius, ~. If q &lt; 0 then blend all concave ~dges with a radius, -~.

## 5.1.1 Computing Offset Edges

As stated above we can compute the curve of an tangent edge by computing the offset curve (15). Often these offset curves these are not representable by a modeller but there are numerous special cases. These csaes are very common in solid modelling. For example:-

- Any isoparameter line offsets into an isoparameter line of the offset surface (see, for example, [10]).
- 0 The offset curve of a planar curve is the translation of that curve along the plane normal.
- The offset of a curve on a sphere is a scaling of the curve about the sphere centre.

and so on.

If an edge is non-tangent then it's offset is computed by re-intersection. The intersection of a pair of surfaces may consist of multiple branches. When this happens a curve for the edge is chosen by the following criteria,

1. The curve must pass through the offset positions calculated for the vertices of the edge.
2. The curve must not create a self-intersecting body.
3. Choose curves which have the same convexity (with respect to the offset faces) as the edge being offset.
4. Where the above criteria do not uniquelv identify a .-curve, the closest to the original edge is chosen.

## 5.1.2 Computing Offset Vertices

We consider four cases, where we can simply compute the offset vertex.

1. There are three faces (and three edges) meeting at the vertex. All three edges are non-tangent.
2. There are three faces meeting at the vertex, where one of the edges is tangent.
3. All faces meet at the vertex tangentially.
4. The vertex is an isolated surface singularity.

If the vertex cannot be computed by the above methods then re-intersection is used.

## 5.1.3 Three Transversal Faces

If there are three transversal faces meeting at a vertex then it should be possible to generate the offset vertex position by relaxation techniques (such as Newton's method). We have a vertex on three surfaces S1, S2 and Ss at, say, V = Sl(a, L9) = S2(7, 6) = SS(C,c). Then if the offset surfaces are @l, Elz and @a, respectively, we can solve for r,s, t,u, v,w in Ql(r, s) = 02(t, u) = Os(v, w) to give the offset vertex position. If the offset surfaces are pam meter preserving then the three positions El = @I(a, d), E2 = @z(y, 6) and Ea = @s(~, () provide good estimates for relaxation. This relaxation will fail if any surface is singular at the vertex or any pair of surfaces meet tangentially.

Note also that some of these surfaces may not be being offset. In these cases we use the original surface with the original vertex position as the estimate.

## 5.1.4 Three Faces, One Tangent Edge

If there are three faces at a vertex where two faces are tangential (i.e. generating a tangent edge) then we can generate the offset vertex position if we have been able to compute the offset tangent edge (see 5.1.1). Similar to 5.1.3 we can generate the offset vertex position by relaxing to the intersection of the offset curve and the transversal surface. Again good estimates are available from the model.

## 5.1.5 All Faces Tangent

This is the simplest case. Here there are a number of faces at the vertex, and the surface normals of every face at the vertex are the parallel. A simple calculation gives the offset vertex position as V + qN where V is the vertex position and N the normal direction.

## 5.1.6 Isolated Surface Singularities

Here the algorithm expects the offset surface also to be singular. The offset vertex is then at the singular point of the offset surface.

## 5,2 Shelling A Body

## 5,2.1 Closed Shelling

Closed shelling of a solid can be implemented in the following three steps.

1. Copy all the faces of the body. All geometric and topological entities are copied.
2. Offset all the copied faces. Recompute all geometry of edges and vertices. The algorithms described in 5.1 are used to do this.
3. Add the offset faces to the body. If the offset distance is greater than O, these will form the exterior of the shelled solid and the original faces will form an interior void, If the offset distance is less than O, the offset faces will form an interior void and the original faces will remain the exterior of the shelled solid.

Figure 9: Sectioned Shelled Bottle

<!-- image -->

## 5.2.2 Open Shelling

To shell a body with a number of open faces, the algorithm is very similar to that for a closed shell, but we must be careful with the faces being opened.

1. Copy all faces which are to be offset.
2. Offset all the copied faces. Recompute all geometry of edges and vertices. The algorithms described in 5.1 are used to do this.
3. Check for self-intersection between faces of the original solid and the offset faces.
4. If the offset distance is greater than zero the original faces of the body (except the faces being opened) must be negated. If the offset distance is less than zero the offset faces must be negated.
5. Determine all the face which share an edge with a face being opened.
6. Intersect these with the opened faces and imprint the intersections onto the opened faces and adjacent faces, splitting edges and faces where necessary.
7. Delete any faces and edges which are not in the set described in (19) and (2o).

Figure 10: Part to Be Shelled

<!-- image -->

## 6 Conclusion

This paper discussed offsetting and shelling operations on Boundary Representation (Brep) solid modellers. Offset geometry useful to offsetting solids was reviewed and a detailed description of the offset of the boundary of a solid was given.

Practical algorithms for the implementation of these operations in Brep solid modellers were presented. These algorithms have been imdernented in the Hewlett-Packard CAD . system SolidDesigner, Figures 9 to 11 give some examples of the shelling operation on non-trivial bodies.

These alg~ri~hms are restricted to a subclaas of Brep solids. More work is required to remove these restrictions and Boolean operations may provide the required techniques.

## 7 Acknowledgements

The author would like to grateful acknowledge the help and support of Dr. John Owen, D-Cubed Ltd, in preparing this paper. This work was supported by the Mechanical Design Division of Hewlett Packard.

## REFERENCES

- [1] Farouki, R.T. and Neff, C.A (1990) AnaJ@ic properties o~ plane oflset curues, Computer Aided Geometric Design 7, 83-99.
- [2] Farouki, R.T. and Ifeff, CA (1990) Algebraic properties of plane oflset curues, Computer Aided Geometric Design 7, 101-127.
- [3] Farouki, R.T. (1985) Exact Oflset F'roceduresfor Simple Solids, Computer Aided Geometric Design 2, 257-279.
- [4] Farouki, R.T. (1985) The Approximation of Non-Degenerate Ofiset Surfaces, Computer Aided Geometric Design 3, 15-43.
- [5] Rossignac, J.R. and Requicha, A.A.G (1985) Offset Operations in Solid Modelling, Computer Aided Geometric Design 2, 129-148.
- [6] Saeed, S.E.0, de Pennington, A, and Dodsworth J.R (1988) Offsetting in Geometric Modelling, ComputerAided Design 20, 67-74.
- [7] Pham, B. (1992) Oflset Curves and surfaces: A Brief Suruey, Computer-Aided Design 24, 223-229.
- [8] van Brunt, B, (1994) Weingarten Surfaces, Design and Application of Curves and Surface, Mathematics of Surfaces V, ed by R. B. Fisher, 49-87.
- [9] Guggenheimer, H,W. (1963) Differential Geometry, Dover Publications Inc.
- [10] Kreyszig, E. (1991) Differential Geometry, Dover Publications Inc.

Figure 11: Shell with Three Open Faces

<!-- image -->