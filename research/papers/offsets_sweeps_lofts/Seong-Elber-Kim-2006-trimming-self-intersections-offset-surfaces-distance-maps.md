## Trimming Local and Global Self-intersections in Offset Curves/Surfaces using Distance Maps

Joon-Kyung Seong a Gershon Elber b , ∗ Myung-Soo Kim c

a

School of Computer Science and Engineering, Seoul National University, Korea

School of Computing, University of Utah, USA b Computer Science Dept. Technion-Israel Institute of Technology, Israel c

## Abstract

Wepresent a robust and efficient algorithm for trimming both local and global self-intersections in offset curves and surfaces. Our scheme is based on the derivation of a rational distance map between the original curve or surface and its offset. By solving a bivariate polynomial equation for an offset curve or a system of three polynomial equations for an offset surface, we can detect all local and global self-intersection regions in offset curves or surfaces. The zero-set of the polynomial equation(s) corresponds to the self-intersection regions. We trim these regions by projecting the zero-set into an appropriate parameter space. The projection operation simplifies the analysis of the zero-set, which makes the proposed algorithm numerically stable and efficient. Furthermore, in a post-processing step, we employ a numerical marching method, which provides a highly precise scheme for self-intersection elimination in both offset curves and surfaces. We demonstrate the effectiveness of our approach using several experimental results.

Key words: Local self-intersection, global self-intersection, offset curve, offset surface, distance map, zero-set computation

∗ Corresponding author. Address: Computer Science Dept. Technion-Israel Institute of Technology, Haifa 32000, Israel. Tel:+972-4-829-4338; fax: +972-4-829-3900.

Email addresses:

seong@cs.utah.edu

(Joon-Kyung Seong),

gershon@cs.technion.ac.il (Gershon Elber), mskim@cse.snu.ac.kr (Myung-Soo Kim).

## 1 Introduction

Offsetting of curves and surfaces is one of the most important geometric operations in CAD/CAM due to its immediate applications in geometric modeling, NC machining, and robot navigation [10]. Exact offset curves and surfaces usually have algebraic degrees considerably higher than their original curves and surfaces. Furthermore, the offsets of rational curves or surfaces are non-rational in general. Hence, offset curves or surfaces are often approximated using rational curves or surfaces of relatively lower degree [4,9,14,15,17].

When C ( t ) is given as a rational freeform curve, the exact offset curve Od ( t ) (with respect to an offset distance d ) is defined by

<!-- formula-not-decoded -->

where N ( t ) is the unit normal of C ( t ) . Since the unit normal vector N ( t ) is nonrational in general, the exact offset curve Od ( t ) is non-rational. Thus, the offset curve is usually approximated by a rational curve, denoted as O e d ( t ) , within a tolerance e . An offset approximation within a tolerance e means that O e d ( t ) lies inbetween two nearby offset curves, Od + e ( t ) and Od -e ( t ) . More precisely, O e d ( t ) is contained in the region swept by a disk of radius e moving along the exact offset curve Od ( t ) . In this paper, we denote the offset approximation as O e d ( r ) ; namely, it is parameterized independently of the original curve C ( t ) . Given a rational surface S ( u , v ) , its offset approximation surface is also represented as O e d ( r , t ) .

Even after an offset curve or surface is approximated by a rational curve or surface, the detection and elimination of self-intersections is a difficult problem. Figure 1 shows that the offset approximation may have self-intersections even though the original curve or surface has no self-intersection - an offset curve or surface has self-intersections locally due to regions of high curvature in the original curve or surface and the global self-intersection results from two different points of the curve or surface that are offset to the same location. These self-intersections must be detected and trimmed away to obtain a proper offset.

Even for offset curves [3,14], it is a non-trivial task to detect and trim all local and global self-intersections. Lee et al. [14] applied a plane sweep algorithm to detect all self-intersections of planar offset curves. Elber and Cohen [3] detected local self-intersections of offset curves by checking whether the tangent field of C ( t ) and Od ( t ) have opposite directions. Whereas a plane sweep approach is difficult to implement, the tangent field approach of [3] is limited to detecting local selfintersections only. In a recent work, Elber [7] proposed an algorithm for trimming both local and global self-intersections of offset curves. In the current paper, we extend the result of Elber [7] to the general case of trimming local and global selfintersections of offset surfaces.

Fig. 1. A freeform curve (a) and a freeform surface (b) in gray and their offset approximations. Both local and global self-intersections occur in the offset curve and the offset surface.

<!-- image -->

The problem of trimming self-intersections in offset surfaces is considerably more difficult to solve [1,2,13,16,21,22]. Cohen and Ho [2] introduced a trimming algorithm that is based on a necessary condition for a freeform surface to have selfintersections. This approach works for general surfaces. However, it is not an optimal solution for offset surfaces since no special consideration is taken into account for the relationship between the original surface and its offset. Wang [22] proposed an algorithm to compute the intersection curve between two offset surfaces, but his algorithm can deal with global self-intersections only. Aomura and Uehara [1] presented a similar approach based on numerical integration starting from random initial points. Nevertheless, this method does not guarantee the detection of all the components of self-intersections. Maekawa et al. [16] presented a method for tracing self-intersection loops in the parameter domain. In their method, starting points are computed by solving a system of nonlinear polynomial equations; but they are solving five equations in five variables and their algorithm requires special treatment for trivial solutions. Wallner et al. [21] considered the problem of computing the maximum offset distance that guarantees no local or global self-intersections. Elber and Cohen [3] detected local self-intersections of offset surfaces by considering the normal fields of a rational surface S ( u , v ) and its offset surface Od ( u , v ) with respect to an offset distance d .

In this paper, we propose a new method that simplifies the detection and trimming of offset self-intersections. Consider a disk (of radius d ) moving along the original curve C ( t ) . If an offset curve point Od ( r ) is contained in an instance of the moving disk's interior, the point Od ( r ) belongs to a self-intersecting region (Figure 2(a)). This geometric concept holds for both local and global self-intersections and it can be formulated algebraically as a rational distance map between the original curve or surface and its offset. If there is a curve point C ( t ) within a distance d from the offset curve Od ( r ) , the offset point Od ( r ) belongs to a self-intersecting region. In Figure 2(b), an offset curve segment, which is inside the circle, belongs to a self-intersecting region since the distance between the circle center p and the offset curve segment is less than d . Thus, we need to trim away such an offset curve segment Od ( r ) . The same relation also holds for a surface point S ( u , v ) and an offset surface Od ( s , t ) .

Fig. 2. (a) The self-intersecting region in an offset curve is contained in a swept region of a circle. (b) It can be represented by using the distance between an original curve (shown in gray color) and its offset curve (shown in bold lines). An offset curve segment, which is inside the circle, belongs to a self-intersecting region since the distance between the circle center p and the offset curve segment is less than the offset distance or the radius of the circle.

<!-- image -->

Let D ( r , t ) be the distance function between the original curve C ( t ) and its offset curve Od ( r ) . Then, the solution set for the inequality condition D ( r , t ) -d &lt; 0 corresponds to the self-intersection regions in the offset curve. By projecting the solution set onto the r -axis, i.e., the parameter space of the offset curve Od ( r ) , we can detect all self-intersections in the offset curve Od ( r ) . The projected region of the solution set is a union of open intervals on the r -axis; when we add end points to these open intervals, the resulting union of closed intervals is exactly the same as the projection of the zero-set of D ( r , t ) -d = 0. Therefore, we can reduce the problem of trimming offset self-intersections to that of finding the zero-set of a single polynomial equation in an rt parameter space. A variety of geometric problems involving freeform curves or surfaces can be reduced to the single question of finding the zero-set of a system of nonlinear polynomial equations in the parameter space of the original curves or surfaces. In other work, we presented several algorithms that are based on similar schemes of problem reduction to parameter space [8,18,19]. These include computing the convex hull of freeform curves or surfaces [8,18], computing bisector curves or surfaces [5], constructing sweep envelopes, and intersecting a freeform surface with a sweep surface [19]. The trimming algorithm we employ in this paper also operates on the same premise as taken in [12].

The trimming algorithm can easily be extended to offset surfaces. Similarly to the curve case, the distance function D ( u , v , r , t ) between a freeform surface S ( u , v ) and its offset surface Od ( r , t ) can be derived as a four-variate rational function. In this case, the solution set for D ( u , v , r , t ) -d &lt; 0 corresponds to the self-intersection of an offset surface. Let R denote the projected region of the solution set onto the rt -plane. Then, the offset surface patches Od ( r , t ) corresponding to the parameter domain R are self-intersecting regions. The boundary of this region R can be determined by solving three polynomial equations: D ( u , v , r , t ) -d = 0, D u ( u , v , r , t ) = 0 and D v ( u , v , r , t ) = 0, where D u and D v are the u and v -partial derivatives of D .

The topological configuration of the zero-set may be arbitrarily complex depending on the shape of the original curve or surface and the offset distance. However, the projection operation of the trimming algorithm considerably simplifies the topological analysis of the zero-set since it reduces the dimensionality of the problem. Consider the curve case with a parameter interval I in the r -axis, which is the projection of the zero-set of D ( r , t ) -d = 0. Viewed along the t -direction, there may be multiple zero-set curve segments appearing over the interval I . Nevertheless, no matter how complex the zero-set curves are, the interval I is characterized by its end points. Moreover, the algorithm for trimming offset surfaces reduces a four-dimensional problem into that of contouring curves in the rt -plane. This step greatly simplifies the trimming procedure. Hence, it is easy to implement our algorithm in a numerically stable and efficient way. Furthermore, to get highly precise self-intersection points and curves, we apply a post-processing step where numeric marching methods are employed. A few steps of the Newton-Raphson iterations are sufficient to achieve precise self-intersections both in offset curves and surfaces. Several experimental results show the effectiveness of this approach.

The main contribution of our work can be summarized as follows:

- We simplify the problem of trimming self-intersections by using a rational distance function and a projection operation;
- We reduce the difficult problem of trimming self-intersections in offset curves and surfaces to the relatively easier problem of finding the zero-set of a single polynomial equation or a system of three polynomial equations, respectively;
- We achieve highly precise self-intersection trimming by applying a post-process based on numeric marching steps.

The rest of this paper is organized as follows. In Section 2, we discuss the trimming algorithm for the self-intersection of an offset curve, an approach that is based on a distance function computation. Section 3 presents its extension to the offset surface. In Section 4, we consider the topological issue of the proposed trimming approach based on the characteristics of the zero-set of the constraint equations. In Section 5, a method for numerical improvement is addressed. Some examples are presented in Section 6, and finally, in Section 7, we conclude this paper.

## 2 Trimming Self-Intersections in Offset Curves

In this section, we consider the process of trimming self-intersections in the offset approximation of a freeform rational curve in the plane. We briefly summarize the algorithm of Elber [7] and supplement it with some new results.

Given a rational offset approximation of a rational curve C ( t ) by a distance d , O e d ( r ) , where e &gt; 0 denotes the accuracy of the approximation, consider a squared distance function

<!-- formula-not-decoded -->

If no self-intersection occurs in O e d ( r ) , then D e d ( r , t ) ≥ ( d -e ) 2 , ∀ ( r , t ) . In contrast, if O e d ( r ) is self-intersecting, then there exist points in O e d ( r ) that are closer than d -e to C ( t ) . Therefore, any pair of points O e d ( r ) and C ( t ) such that D e d ( r , t ) &lt; ( d -e ) 2 implies that there is a self-intersection.

Let r &gt; 0 be another small positive real value and let

<!-- formula-not-decoded -->

Any point ( r 0 , t 0 ) in the zero-set of F ( r , t ) = 0 represents two points, C ( t 0 ) and O e d ( r 0 ) , that are ( d -e -r ) apart. Every such point O e d ( r 0 ) must be purged away as a self-intersecting point. Therefore, the set of offset curve points that are free from self-intersections is contained in

<!-- formula-not-decoded -->

In other words, if a point r on the r -axis is on the projection of the zero-set of F ( r , t ) = 0, then the corresponding offset curve point O e d ( r ) belongs to the selfintersecting region. We denote this trimming process (using a small positive trimming distance r below the offset approximation) a r -accurate trimming or r -trimming , for short. Below we present the algorithm that detects and eliminates the self-intersection regions:

## Algorithm 1

## Input:

C ( t ) , A rational curve;

by distance d and tolerance e ;

O e d ( r ) , A rational approximation offset of C ( t )

- r , A trimming distance for the self-intersections.

## Output:

A piecewise rational approximation offset of C ( t ) by distance d , tolerance e , and r -trimming;

## Begin

return the r domain(s) of O e d ( r ) not included in Z r ; End.

<!-- formula-not-decoded -->

Since F ( r , t ) is a piecewise rational bivariate function, the zero-set, Z , could be constructed by exploiting the convex hull and subdivision properties of NURBS, yielding a highly robust divide-and-conquer zero-set computation that is reasonably efficient [6]. The parameter value r 0, such that C ( t ) is closer to O e d ( r 0 ) than the distance ( d -e -r ) , for some t , implies that the curve offset point O e d ( r 0 ) belongs to a self-intersecting region. Hence, Z is projected onto the r -axis, as Z r . The domain of the r -axis covered by this projection prescribes the regions of O e d ( r ) that present self-intersections and thus, must be purged away.

Figure 3 presents the example from Figure 1(a) again. In Figure 4, the log of the squared distance function, D e d ( r , t ) , is presented for the curve of Figure 3. Also shown in Figure 4 is the zero-set, Z , of F ( r , t ) = D e d ( r , t ) -( d -e -r ) 2 and its projection, Z r on the r -axis. In this case, the r -axis is divided into four valid intervals, which characterize three sub-regions of the offset curve that are selfintersecting. The first and third bold intervals along the r -axis are due to the global self-intersection of the curve in Figure 3, whereas the middle large thick interval corresponds to the local self-intersection in the curve. Figure 3(b) shows the r -trimmed curve segments that are computed by extracting the four valid intervals.

The projection of the zero-set, Z , should be onto the r -axis. If we project the zero-set onto the t -axis, which is the parameter space of the original curve C ( t ) , then invalid trimming of a valid parameter interval may occur on the curve C ( t ) . Considering the trimming process as a continuous sweeping of a disk (see Figure 2(a)), we need to sweep the disk along the original curve. Moving a disk along the offset curve O e d ( r ) may cut away some original curve regions that present no self-intersections. A projection of the zero-set onto the t -axis will find all locations in C ( t ) that are at a distance ( d -e -r ) from a point O e d ( r ) . In symmetry, however, the projection onto the r -axis finds all locations in O e d ( r ) that are within the distance ( d -e -r ) from a point C ( t ) . Figure 5 shows such an example. After the offset operation, we have two valid curve segments in O e d ( r ) (Figure 5(b)). Figure 6 presents the distance function, D e d ( r , t ) in gray, and the zero-set, Z , in bold lines. The projection of Z onto the t -axis generates only a single valid interval, which means that one of the valid regions of C ( t ) is trimmed away. On the other hand, the projection of the zero-set onto the r -axis produces three valid intervals. Two of them are connected since the curve is a closed one, and thus, the valid intervals under this projection yield two curve segments as shown in Figure 5(b).

<!-- image -->

Fig. 3. (a) The simple curve and its offset, from Figure 1(a). The curve and its offset are presented in (a). (b) is the result of r -trimming the curve using D e d ( r , t ) whereas, in (c), the result is improved using numerical marching.

<!-- image -->

r

Fig. 4. The distance function, D e d ( r , t ) , of the curve in Figure 3, on a logarithmic scale. Also shown, in bold lines, are the zero-set, Z , of F ( r , t ) = D e d ( r , t ) -( d -e -r ) 2 as well as the projection of Z on the r -axis, Z r .

A point O e d ( r 0 ) is called a match to a point C ( t 0 ) if it is the point offset from C ( t 0 ) . If we use r = 0 in the definition of the function F ( r , t ) , then the trimming process becomes numerically unstable as the matched point can be a trivial solution of the equation F ( r , t ) = 0. Thus, we need to exploit a positive value of r to compute the self-intersections somewhat conservatively. For numerical stability, r should be as large as possible, reducing the chance of detecting the matched points as selfintersections. Nonetheless, and while r should always be positive, as r approaches zero, it is going to be more difficult to robustly evaluate the zero-set of F ( r , t ) = 0. While describing the details of a zero-set finding is beyond the scope of this work, it is clear that, the better the zero-set finding is, the higher the quality of the offset trimming will be. In contrast, the larger r is, the higher the likelihood that we will miss small self-intersections. In practice, r was selected to be between 1% and 5% of the offset distance d . Another crucial characteristic of r is that using the function F ( r , t ) we can detect all self-intersections at distance greater than r from the trimmed offset curve, where the function F ( r , t ) yields negative values. The self-intersecting offset curve segments are extracted by the zero-set finding routine.

The outcome of Algorithm 1 is a subset of O e d ( r ) . The latter comprises curve segments that have no point closer than ( d -e -r ) to C ( t ) . By selecting r &gt; 0, the curve segments that result from Algorithm 1 are not exactly connected but slightly cross each other. A sequence of numeric Newton-Raphson marching steps at each such pair of trimmed end points could very quickly converge to the exact selfintersection location. Only a few steps are required to converge to the highly precise self-intersection location. Figure 3(c) shows the result of the numerical improving step. In Section 6, we present several other examples that demonstrate this entire procedure, including the aforementioned numerical marching step.

Fig. 5. An example that justifies the projection of the zero-set onto the r -axis.

<!-- image -->

## 3 Trimming Self-Intersections in Offset Surfaces

We now extend the trimming algorithm for offset curves to the case of offset surfaces. We derive a squared distance function between a freeform surface S ( u , v ) and its offset approximation, O e d ( r , t ) . The function becomes a four-variate one and hence, we end up dealing with a three-dimensional zero-set in a four-dimensional parameter space. The projection of the zero-set onto the rt -space prescribes the self-intersection regions in the offset surfaces. The main difference of this extension from r -trimming of a planar curve is that we now directly extract the boundary of the projected zero-set by considering two more constraints. We can do the same thing in the curve case as well.

Fig. 6. The distance function, D e d ( r , t ) , of the curve in Figure 5, on a logarithmic scale. Also shown, in bold lines, are the zero-set, Z , of F ( r , t ) = D e d ( r , t ) -( d -e -r ) 2 = 0 as well as the projection of Z on the r -axis and the t -axis.

<!-- image -->

Consider the squared distance function:

<!-- formula-not-decoded -->

Assume that r represents a trimming distance and let

<!-- formula-not-decoded -->

An offset surface point O e d ( r 0 , t 0 ) belongs to a self-intersecting region if there exists another point S ( u 1 , v 1 ) satisfying the following inequality condition:

<!-- formula-not-decoded -->

Similarly to the curve case, the set of offset surface points that are free of selfintersections is contained in

<!-- formula-not-decoded -->

As mentioned above, the solution set of the condition F ( u , v , r , t ) ≤ 0 in the uvrt -domain is involved in the self-intersections and thus, the corresponding offset points O e d ( r , t ) should be trimmed away. In other words, if the point ( r , t ) in the parameter domain falls into the projection of this solution set, then the corresponding offset surface point O e d ( r , t ) belongs to the self-intersecting region. The boundary of the 'uncovered' region of the rt -plane (under this projection) is characterized as the projection of the uv -silhouette curves (along the uv -direction) of the zero-set of the equation F ( u , v , r , t ) = 0. This means that the u -partial derivative and the v -partial derivative must simultaneously vanish along the silhouette curve, which can be characterized as the simultaneous satisfaction of the following three constraints in the uvrt -space:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Having three Equations (2), (3), and (4) in four variables, the solution is a univariate curve that is a simultaneous solution in the uvrt -space. In this work, we exploit a spline subdivision-based method for solving the set of equations, represented as scalar spline functions [6]. The zero-set in arbitrary dimension could be computed by extending the Dual Contouring method into an arbitrary dimension [20].

The univariate solution curve can be parameterized by some variable a :

<!-- formula-not-decoded -->

Then, this set of curves forms the trimming curves for the offset surfaces that are self-intersection-free up to e and r . Algorithm 2 summarizes the overall procedure:

## Algorithm 2 Input:

S ( u , v ) , A rational freeform surface;

by distance d and tolerance e ;

O e d ( r , t ) , A rational approximation offset of S ( u , v )

r , A trimming distance for the self-intersections.

## Output:

A piecewise rational approximation offset of S ( u , v ) by distance d

tolerance , and

## Begin

, e r -trimming;

D e d ( u , v , r , t ) ⇐ &lt; S ( u , v ) -O e d ( r , t ) , S ( u , v ) -O e d ( r , t ) &gt; ; F ( u , v , r , t ) ⇐ D e d ( u , v , r , t ) -( d -e -r ) 2 ; G ( u , v , r , t ) ⇐ Fu ( u , v , r , t ) ; H ( u , v , r , t ) ⇐ Fv ( u , v , r , t ) ; (1) Z ⇐ the simultaneous zero-set of F , G , and H ; return offset approximation trimmed by Z ;

End.

Step (1) in Algorithm 2 can also employ the original trimming curves of S ( u , v ) , if to begin with, S ( u , v ) is a trimmed surface. The outcome of Algorithm 2 is a trimmed offset surface approximation O e d ( r , t ) . It comprises surface patches that have no point closer than ( d -e -r ) to S ( u , v ) .

## 4 Topology of Trimming Curves in Offset Surfaces

In Section 3, we presented an algorithm for determining a boundary of the projected zero-set of Equation (2) in the rt -plane by computing uv -silhouette curves (along the uv -direction) of the zero-set. The uv -silhouette curves in the rt -plane may intersect each other; namely, a connected component of the zero-set of Equation (2) is partially blocked (along the uv -direction) by another one in the uvrt -space in the projection along the uv -direction (see Figure 7). The intersections among uv -silhouette curves should be detected and resolved for a proper trimming. In the current problem, we need to extract only the outmost boundary of the projected region, which makes the overall procedure numerically stable.

For the clarity of presentation, we first consider the possible topological problem using a simple low-dimensional case of trimming offset curves. Figure 8 shows the same squared distance function of Figure 4; the zero-set of Equation (1) is drawn in gray. The projection of the zero-set onto the r -axis is determined by the t -silhouette points at which the t -partial derivatives of F ( r , t ) vanish. The t -silhouette points are represented in bold dots in Figure 8. Viewed along the t -direction, the two zeroset components overlap. They are located in the middle part of the rt -space. The intervals resulting from these two zero-set components correspond to local selfintersections in the offset curve. Since local self-intersections are due to regions of high curvature in C ( t ) , there always exists an offset curve point O e d ( r ) in the local self-intersection region that has more than two points of C ( t ) that are closer than ( d -e -r ) to O e d ( r ) . This means that sub-regions made from the zero-sets of Equation (1) always overlap in the local self-intersection region (see the two intervals that are the projections of the two zero-set curves in the middle part of Figure 8).

Fig. 7. A zero-set of Equation (2) in the uvrt -space and its projected uv -silhouettes onto the rt -space. The zero-set components overlap along the uv -direction and cause the intersection of its projected silhouettes.

<!-- image -->

In contrast to the local self-intersections, global self-intersections do not always introduce overlaps in the zero-set. Two zero-set components that correspond to the same global self-intersection region in O e d ( r ) come as a pair. We call them a matching pair. In Figure 8, the two zero-set components that are located in the corner parts of the rt -space correspond to the global self-intersection. The two intervals resulting from the matching pair do not overlap in this case.

In the case of offset surfaces, the situation is far more complex since we need to consider the overlapping of the zero-set components in four-dimensional space. The topological structure of the problem, however, is almost identical to the case of offset curves. The uv -silhouette curves in the rt -space, which are boundary curves of the projected region of the zero-set, are intersecting each other when they correspond to the local self-intersection region. Figure 9(a) shows a freeform rational surface and its offset approximation. Figure 9(b) shows the projection of uv -silhouette curves of the zero-set of Equations (2) onto the rt -space. In the middle part, there are two intersecting loops that correspond to the local self-intersection. For a proper trimming, we should form a union of the domains bounded by the two overlapping loops into a single loop by applying a planar Boolean union operation in the rt -parameter domain. Figure 10(a) shows the resulting trimming loop.

In summary, since our approach to trimming self-intersections is based on the dis- tance function, the topological complexity can be handled in a simpler and more flexible way. Although the structure of the zero-set in two-dimensional space (off- set curves) or four-dimensional space (offset surfaces) may be arbitrarily complex, the presented scheme only requires forming a union of the projected regions of the zero-set if they overlap. The topological problem is far more complex in offset surfaces than in offset curves. Nevertheless, we can efficiently resolve it using a Boolean union operation in the rt -parameter space. Furthermore, the main strength of our approach is that the topological complexity in both local and global self-intersections is handled in a unified procedure. The local and global self-intersections are resolved simultaneously using the same union procedure. Figure 15 shows such a case. Even in this situation, our method can deal with self-intersections properly by performing a planar Boolean operation.

Fig. 8. The same example as in Figure 4. The zero-set Z is shown in thick gray and the t -silhouette points of the zero-set curves are presented as thick dots.

<!-- image -->

Fig. 9. (a) A rational freeform surface (shown in bold lines) and its untrimmed offset surface (shown in light lines). (b) The corresponding rt -projection of the zero-set of the distance square function is shown as a set of curve segments. One can see the overlapping trimming curve segments in local self-intersections.

<!-- image -->

Fig. 10. Overlapping in local self-intersections is resolved in (a) and the resulting trimmed offset surface is shown in (b). Compare with Figure 9.

<!-- image -->

## 5 Post-process of Numerical Improvement

We computed a r -trimming of offset surfaces rather conservatively by selecting r &gt; 0. The surface patches resulting from Algorithm 2 are not exactly connected but cross each other slightly (see Figure 11(a)). Therefore, we need to apply a postprocess of numeric marching steps to get a highly precise self-intersection curve of the offset surface. A numeric marching between two offset surface patches is essentially a Surface-Surface Intersection (SSI) problem.

In our implementation, we adapt a standard numerical marching SSI technique exploiting the Newton-Raphson method. Finding a good initial solution and analyzing the topology of the intersection curves are the most difficult parts in such a numerical tracing algorithm. A r -trimming, however, resolves both tasks and leads to a highly precise self-intersection curve. We first find two matching trimming curves, which are close to each other in Euclidean space after evaluation. In the example of Figure 10, there exist three trimming curves, l 1 , l 2 , l 3 (see also Figure 11 (b)). Since l 1 and l 3 correspond to the global self-intersection, they produce a single connected component of the self-intersection curves. l 2 is matched to itself since it corresponds to the local self-intersection. Consequently, we have two connected components of the intersection curves. Starting from these matchings, a few steps of the standard Newton Raphson technique produce a highly precise self-intersection curve. Figure 11(c) shows two self-intersection curves. The upper one presents selfintersection curves before the numerical improvement stage and the lower one is the result of the numerical marching step.

Fig. 11. (a) A zoom-in on Figure 10 that shows the trimmed self-intersection curves in detail. (b) shows the three trimming curves in the rt -plane. (c) The trimmed self-intersection curves (upper ones) can be improved to the highly precise intersection curves (lower ones) after a post-process of numerical marching step.

<!-- image -->

## 6 Experimental Results

Wenowpresent several examples of trimming both local and global self-intersections in offset curves and surfaces. We first show some examples for trimming offset curves. Figure 12 presents an example of a curve with several offsets in both directions. In Figure 12(a), the original curve is shown (in gray) with the untrimmed offset approximations. With the aid of the distance square function, the self-intersections are r -trimmed in Figure 12(b), and the result of applying the numerical marching step is shown in Figure 12(c).

Figures 13 and 14 present two more complex examples of offset trimming for curves. Here, (a) shows the original curve (in gray) and its untrimmed offset approximations, and (b) is the result of r -trimming the self-intersections using the function F ( r , t ) of Equation (1). In all the examples presented in this work, the trimming distance r was taken from 1% to 5% of the offset distance d , while an offset tolerance, e , was 0.1% to 1% of the offset distance. (b) and (c) in Figures 13 and 14 present the result of trimming with r at 5% and 1% of the offset distance, respectively. Small local self-intersections might escape the r -trimming step at 5%

Fig. 12. An example of a curve (in gray) and its untrimmed offsets are shown in (a). (b) is the result of r -trimming the offset curves using F ( r , t ) and (c) is the result of applying the post-process of numerical marching steps.

<!-- image -->

Fig. 13. (a) presents the original curve (in gray) and its untrimmed offsets, while (b) is the result of r -trimming the self-intersections with the aid of the function F ( r , t ) . (b) and (c) are the results of r -trimming of the offset with two different r -trimming percentages of 5% and 1%, respectively. (d) is the result after the post-process of numerical marching steps.

<!-- image -->

and are trimmed better at 1%. (d) in Figures 13 and 14 show the result after the post-process of numerical marching steps. The computation time for these results are about the same, all within one or two seconds on a 2GHz Pentium IV machine.

These small local self-intersections could clearly appear at any percentage of the r -trimming distance, when r &gt; 0. In many applications, such as robotics and CNC machining, local and arbitrarily small self-intersections will induce large accelerations along the derived path, and hence are highly undesirable. If both S ( u , v ) and its offset surface Od ( u , v ) are parametrized so that Od ( u 0 , v 0 ) is along the normal of S at S ( u 0 , v 0 ) , a typical result in many offset approximation schemes, one could employ the local self-intersection test presented in [3] as another filtering step that could completely resolve these small local self intersections.

Fig. 14. (a) presents the original curve (in gray) and its offsets, while (b) is the result of r -trimming the self-intersections using the function F ( r , t ) of Equation (1). (b) and (c) are the results of r -trimming of the offset with two different r -trimming percentages of 5% and 1%, respectively. (d) is the result after the post-process of numerical marching steps.

<!-- image -->

Continuing to examples of trimming offset surfaces, Figure 15 shows the same example as in Figure 10 but with a different offset distance. The local self-intersection and the global self-intersection now merge to form a single component. Even in this case, our approach works well and Figure 15(b) presents the result of the trimming. Figure 15(c) shows the trimming curve in the rt -parameter space.

Figures 16 and 17 present two more complex examples. Here, (a) is the original surface and its untrimmed e -offset approximation, and (b) is the result of r -trimming of the self-intersections using the function F ( u , v , r , t ) and the numerical marching step. The trimming curves in the parameter space are shown in (c) of Figures 16 and 17. In all the examples presented in this work, the trimming distance r was taken from 1% to 5% of the offset distance. Since the offset surface becomes quite complex after a rational approximation, it takes about four to five minutes for r -trimming the offset surface approximations on a 2GHz Pentium IV machine. The original surfaces presented in these experimental examples are represented by bicu- bic NURBS having about 50 to 70 control points. Their rational bicubic NURBS offset approximation with a tolerance of e = 0 . 02 (original objects' dimensions span about a unit length) turned out to have about 5000 to 6000 control points.

Fig. 15. The topology of trimming curves can change as an offset distance changes. Here, the global and local self-intersections of Figure 10 are merged to form a single component.

<!-- image -->

Fig. 16. (a) presents a sweep surface with scale change of cross-sections (bold lines) and its untrimmed e -offset approximation (light lines). (b) shows the result of r -trimming of the e -offset surface. The trimming curves are shown in (c), in the rt -parameter domain.

<!-- image -->

Figure 18 shows one interesting result of the r -trimming. The offset surface of a simple concave surface is shown in Figure 18(a). Figure 18(b) presents the simultaneous zero-set of Equations (2)-(4) in the rt -domain. There exist self-intersections in two loops, with one curve segment completely contained in the other one. According to the topological structure of the zero-set components as in Section 4, we need to resolve the self-intersections in the trimming curves. Furthermore, the inner loop should be removed since its corresponding zero-set component is totally blocked by the other one. The result of r -trimming by using the resolved trimming curves in Figure 18(c) is shown in Figure 18(d). Although we can accomplish the r -trimming robustly, a numerical improvement step in this kind of singular example is still difficult.

Fig. 17. (a) presents a sweep of a circular section (bold lines) following a helix trajectory and its untrimmed e -offset approximation (light lines) and (b) shows the result of r -trimming of the e -offset surface. The trimming curves are shown in (c), in the rt -parameter domain.

<!-- image -->

## 7 Conclusions

We have presented a robust and efficient scheme for trimming both local and global self-intersections in offset curves and surfaces. The presented approach is based on the derivation of a rational distance map between the original curve or surface and its offset. By simultaneously solving one polynomial equation for an offset curve or three polynomial equations for an offset surface in the parameter space, we can detect all the local and global self-intersection regions in offset curves or surfaces. The zero-set of the polynomial equation(s) prescribes the self-intersection regions and we trim these regions by projecting the zero-set into an appropriate parameter space. The projection operation simplifies the topological complexity of the zero-set and makes our algorithm numerically stable and efficient. Furthermore, numerical marching post-processing steps provide highly precise self-intersection elimination in both offset curves and surfaces.

A limitation of the current r -trimming approach is that some tiny self-intersection loops may not be detected in the trimming procedure. Better ways of preventing all small self-intersections in the offset curves and surfaces should be sought. By reparameterizing of the offset curves and surfaces so that they match the original curves and surfaces, we can possibly apply the local self-intersection scheme of Elber and Cohen [3], in cases where the offset is differently parameterized. Self-distance maps and discontinuities in the derivative of the maps near the offset curves and surfaces can be also useful in detecting tiny loops and global self-intersections.

Fig. 18. A simple concave surface and its untrimmed e -offset surface approximation is presented in (a) and the simultaneous zero-set of Equations (2) - (4) in the rt -domain is shown in (b). After considering the topological structure of the zero-set, self-intersections are resolved in (c). Also, the inner trimming curve has been removed since its corresponding zero-set component is totally blocked by the outer component. (d) shows the result of r -trimming of the e -offset surface. The numeric marching technique of SSI encounters some difficulties in such a highly singular case.

<!-- image -->

## ACKNOWLEDGMENTS

Wewould like to thank the anonymous reviewers for their invaluable comments. All the algorithms and figures presented in this paper were implemented and generated using the IRIT solid modeling system [11] developed at the Technion, Israel. This work was supported in part by European FP6 NoE grant 506766 (AIM@SHAPE), in part by the Israel Science Foundation (grant No. 857/04), in part by the Korean Ministry of Information and Communication (MIC) under the Program of IT Research Center on CGVR and in part by grant No. R01-2002-000-00512-0 from the Basic Research Program of the Korea Science and Engineering Foundation (KOSEF).

## References

- [1] S. Aomura and T. Uehara. Self-Intersection of an Offset Surface. Computer-Aided Design , Vol 22, No 7, pp 417-421, 1990.
- [2] E. Cohen and C. C. Ho. Surface Self-Intersection. Mathematical Methods for Curves and Surfaces , T. Lyche and L. L. Schumaker (eds.), pp 183-194, Oslo, 2000.
- [3] G. Elber and E. Cohen. Error Bounded Variable Distance Offset Operator for Freeform Curves and Surfaces. International Journal of Computational Geometry &amp;Applications , Vol 1, No 1, pp 67-78, March 1991.
- [4] G. Elber, I. K. Lee, and M. S. Kim. Comparing Offset Curve Approximation Methods. IEEE CG&amp;A , Vol 17, No 3, pp 62-71, May-June 1997.
- [5] G. Elber and M. S. Kim. Computing Rational Bisectors. IEEE CG&amp;A , Vol. 19, No. 6, pp 76-81, 1999.
- [6] G. Elber and M. S. Kim. Geometric Constraint Solver Using Multivariate Rational Spline Functions. Proc. of ACM Symposium on Solid Modeling and Applications , Ann Arbor, MI, June 4-8, 2001.
- [7] G. Elber. Trimming Local and Global Self-intersections in Offset Curves Using Distance Maps. Proc. of the 10th IMA Conference on the Mathematics of Surfaces , Leeds, UK, pp 213-222, September 2003.
- [8] G. Elber, M. S. Kim and H. S. Heo. The Convex Hull of Rational Plane Curves. Graphical Models , Vol 63, pp 151-162, 2001.
- [9] R. T. Farouki, Y. F. Tsai and G. F. Yuan. Contour Machining of Freeform Surfaces with Real-time PH Curve CNC Interpolators. Computer Aided Geometric Design , No 1, Vol 16, pp 61-76, 1999.
- [10] J. Hoschek and D. Lasser. Fundamentals of Computer Aided Geometric Design . AK Peters, Wellesley, MA, 1993.

- [11] IRIT 9.0 User's Manual, October 2000, Technion. http://www.cs.technion.ac.il/ ∼ irit.
- [12] M. S. Kim and G. Elber. Problem Reduction to Parameter Space. The Mathematics of Surface IX (Proc. of the Ninth IMA Conference) , London, pp 82-98, 2000.
- [13] C. V. V. Ravi Kumar, K. G. Shastry, and B. G. Prakash. Computing Non-SelfIntersecting Offsets of NURBS Surfaces. Computer-Aided Design , Vol 34, No 3, pp 209-228, 2002.
- [14] I. K. Lee, M. S. Kim, and G. Elber. Planar Curve Offset Based on Circle Approximation. Computer-Aided Design , Vol 28, No 8, pp 617-630, August 1996.
- [15] Y. M. Li and V. Y. Hsu. Curve Offsetting Based on Legendre Series. Computer Aided Geometric Design , No 7, Vol 15, pp 711-720, 1998.
- [16] T. Maekawa, W. Cho, and N. M. Patrikalakis. Computation of Self-Intersections of Offsets of Bezier Surface Patches. Journal of Mechanical Design: ASME Transactions , Vol 119, No 2, pp 275-283, 1997.
- [17] M. Peternell and H. Pottmann. A Laguerre Geometric Approach to Rational Offsets. Computer Aided Geometric Design , No 3, Vol 15, pp 223-249, 1998.
- [18] J. K. Seong, G. Elber, J. K. Johnstone, and M. S. Kim. The Convex Hull of Freeform Surfaces. Computing , Vol. 72, No. 1, pp 171-183, 2004.
- [19] J. K. Seong, K. J. Kim, M. S. Kim G. Elber, and R. Martin. Intersecting a Freeform Surface with a General Swept Surface. Computer-Aided Design , Vol. 37, No. 5, pp 473-483, 2005.
- [20] J. K. Seong, G. Elber, and M. S. Kim. Contouring 1- and 2-Manifolds in Arbitrary Dimensions. International Conference on Shape Modeling and Applications , pp 216225, MIT, USA, June 15-17, 2005.
- [21] J. Wallner, T. Sakkalis, T. Maekawa, H. Pottmann, and G. Yu. Self-Intersections of Offset Curves and Surfaces. International Journal of Shape Modeling , Vol 7, No 1, pp 1-21, June 2001.
- [22] Y. Wang. Intersection of Offsets of Parametric Surfaces. Computer Aided Geometric Design , Vol 13, Issue 5, pp 453-465, July 1996.