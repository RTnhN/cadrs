In this paper we present an algorithm for detecting and repairing defects in the boundary of a polyhedron. These defects, usually caused by problems in CAD software, consist of small gaps bounded by edges that are incident to only one polyhedron face. The algorithm uses a partial curve matching technique for matching parts of the defects, and an optimal triangulation of 3-D polygons for resolving the unmatched parts. It is also shown that nding a consistent set of partial curve matches with maximum score, a subproblem which is related to our repairing process, is NP-Hard. Experimental results on several polyhedra are presented. Keywords: CAD, polyhedra, gap lling, curve matching, geometric hashing, triangulation. Introduction The problem studied in this paper is the detection and repair of \gaps" in the bounda polyhedron. This problem usually appears in polyhedral approximations

1 ary of of CAD objects, whose boundaries are described using curved entities of higher levels (cf. [Sheng &amp; Tucholke '91], [Dolenc &amp; M akel a '91], [Bhn &amp; Wozny '92], and [M akel a &amp; Dolenc '93]). In solid modeling the original boundary may be described as the unions and intersections of spheres, cones, etc., whereas in surface modeling it may be described by B ezier surfaces, Nurbs, etc. Some of the gaps are caused by missing surfaces, incorrect handling of adjacent patches within a surface, or (most commonly) incorrect handling of trimming curves, which are dened by the intersections of adjacent surfaces. The mesh points (that is, the computed vertices of the  Work on this paper by the second author has been supported by Oce of Naval Research Grant N00014-90-J-1284, by National Science Foundation Grants CCR-89-01484 and CCR-91-22103, and by grants from the U.S.-Israeli Binational Science Foundation, the Fund for Basic Research administered by the Israeli Academy of Sciences, and the G.I.F., the German-Israeli Foundation for Scientic Research and Development. y School of Mathematical Sciences, Tel-Aviv University, Tel-Aviv 69978, Israel z School of Mathematical Sciences, Tel-Aviv University, Tel-Aviv 69978, Israel, and Courant Institute of Mathematical Sciences, New York University, New York, NY 10012, USA

<!-- image -->

Figure 1: A cracked object polyhedral approximation) along an intersection curve between two such surfaces are often computed separately along each of its two sides, thereby creating two dierent copies of the same curve, causing a gap to appear between the copies. In the simple case, dierent point sets might be produced but according to the same curve equation; in the more complicated case, dierent equations of the curve, one for each surface containing it, are used for the mesh point evaluations. The eect of these approximation errors is invariably the same: the boundary of the resulting polyhedron contains edges which are incident to only one face (whereas

in a valid representation each edge should be incident to exactly two faces), thereby creating gaps in the boundary and making the resulting representation invalid. Such gaps may make parts of the boundary of the approximating polyhedron disconnected from other parts, or may create small holes bounded by a cycle of invalid edges. Usually, each connected component is the result of the polyhedral approximation of a single surface or part of a solid in the original representation. Figure 1 shows a typical example, where thin cracks are curving between the surfaces of the cube-like model. This phenomenon does not usually disturb graphics applications, where the gaps between the surfaces are often too small to be seen, or are handled straightforwardly [Samet &amp; Webber '88]. However, it may cause severe problems in applications which rely on the continuity of the boundary, such as nite element analysis ([Yerry &amp; Shephard '83], [Ho-Le '88]), rasterization algorithms [Foley &amp; Van Dam '84], etc. This problem arises frequently in CAD applications ([Sheng &amp; Tucholke '91], [M akel a &amp; Dolenc '93]) and a signicant number of currently available commercial CAD packages produce these gaps. According to our own practical experience, such gaps arise in almost every suciently large CAD le, so their detection and elimination is indeed a rather acute practical problem. [Sheng &amp; Tucholke '91] refer to these errors as one of the most severe software problems in rapid prototyping. Many authors, such as [Dolenc &amp; M akel a '91] and [Sheng &amp; Hirsch '92], tried to avoid it already in the surface tting triangulation process.

heuristic in order to locally ll cracks in the boundary of the polyhedron. We invoke a similar procedure (which uses a minimum area heuristic) at the second phase of our algorithm (see Section 6.2). To the best of our knowledge, no eort to solve this problem while considering the global consistency of the resulting polyhedron, based only on the polyhedral description of a model, was ever made. In order to do that, we give a precise denition of the problem, and develop a method for identifying instances of this problem and then resolving them. Let us denote the collection of cycles of invalid edges on the polyhedron boundary by the term borders. The main problem that we face is to \stitch" these borders together, i.e. add new faces that close the gaps in the boundary such that the resulting polyhedron is valid. New faces are added by connecting points along the same or dierent borders. To achieve this we rst have to identify matching portions of these borders (e.g. arcs pq and p 0 q 0 in Figure 1), and then to choose the best set of matches, construct new facets (planar faces) connecting them, and ll (by triangulation) the

remaining holes. Successful solutions to all these subproblems are described in detail in this paper. For the purpose of identifying matching portions of the borders we use a partial curve matching technique, which was rst suggested in [Kalvin, Schonberg, Schwartz &amp; Sharir '86] and in [Schwartz &amp; Sharir '87]. This technique, which uses the socalled Geometric Hashing method, originally solved the curve matching problem in the plane, under the restrictive assumption that one curve is a proper subcurve of the other one, namely: Given two curves in the plane, such that one is a (slight deformation of a) proper subcurve of the other, nd the translation and rotation of the subcurve that yields the best least-squares t to the appropriate portion

of the longer curve. This technique was extended and used in computer vision for automatic identication of partially obscured objects in two or three dimensions, an important problem in robotics applications of computer vision, which has attracted much attention. The geometric hashing technique was applied in [Hong &amp; Wolfson '88], [Wolfson '90] and

2 Denition of the Problem Consider the following description of the boundary of a polyhedron, where the boundary is represented by two lists: one contains all the vertices of the polyhedron, and the other contains all the facets. A facet is a collection of one or more polygons, all

lying in the same plane in 3D. The rst polygon is the envelope (outer boundary) of the facet, and the other polygons, if any, are windows in it (when the facet is not simply connected). Each polygon is specied as a circular sequence of indices in the vertex list. There is no restriction on the length of such an index sequence, hereafter referred to as the size of the polygon. As input to our algorithm, there is no restriction on the directions of the polygons. Eventually, in order to form an oriented 2-manifold, they will have to obey a consistency rule. For example, we require that all the facet envelopes appear in the clockwise direction when viewed from outside the polyhedron, and that all the

window polygons appear in the counter-clockwise direction when viewed this way. Thus, the body of a polygon will always be on the right hand side of every directed edge which belongs to it, when viewed from the outside. Since the directions of all the input polygons are arbitrary, we have to orient them ourselves in these consistent directions. Note that each valid edge appears in exactly two facets, and that these two appearances are oppositely directed to each other. The main problem addressed in this paper is the existence of gaps between and/or within parts of the polyhedron boundary. We want to identify correctly the matching portions of the borders, and ll them with additional triangles, such that no holes remain. The output should be an orientable manifold which describes a closed volume. As the previously suggested recipes cited in the introduction, we may also allow the resulting boundary to intersect itself near its original borders. This often happens anyway in CAD approximations of curved surfaces. The borders of the approximating polyhedral surfaces are generated in very close locations (where they should really be coincident), potentially making the surfaces either totally disconnected or intersecting. We attempt to stitch together close borders, allowing the unied boundary to the existence of small self-intersections. One example is the class of scan-line rasterization algorithms, where the \inside" and the \outside" of the rasterized object must be well dened. Self-intersections are easily handled by simply counting the number of entrances to and exits from the object along a scanning line. Another example is the class of nite element analysis algorithms, which are based on processing the boundary of the object. Usually, these algorithms are not even aware that small self-intersections occur. 3 Overview of the Algorithm Our proposed algorithm consists of the following steps: 1. Data acquisition:  Identify the connected components of the polyhedron boundary, and orient all the facets in each component with consistent directions.  Identify the border edges, each incident to only one face, and group them into a collection of border polygons, each being a cycle of border edges. Each open connected component of the polyhedron boundary is bounded by one or more border polygons. 2. Matching border portions:  Discretize each border polygon into a cyclic sequence of vertices, so that the arc length between each pair of consecutive vertices is equal to some given parameter.  Vote for border matches. Each pair of distinct vertices which belong to the discretized borders and whose mutual distance is below some threshold parameter, contributes one vote. The vote is for the match between these two borders with the appropriate shift, which maps one of the vertices to the other.

 Identify the remaining holes (usually appearing at the junctions of several

matches).  Triangulate the holes, using a 3-D minimum area triangulation technique. The following three sections describe the algorithm steps in detail. 4 Data Acquisition The description of the boundary of the polyhedron is typically given in a le, output of a CAD system. Our system allows several input formats, without any restriction on the size of the polygons, and allowing facets to contain windows. Most of the le formats used by commercial CAD systems do not include adjacency information (between facets). When the input does not specify this information, our system generates it as a preprocessing step. For this purpose we sort the edges according to the id's of their endpoints, and transform every pair of two successive edges in this order, which have the same endpoints, into an adjacency relation between facets. The same information was computed in [Dolenc and M akel a '91] using an Octreelike data structure, and in [Rock and Wozny '92] using an AVL tree. The internal representation that our system actually uses in further steps is the quad-edge data structure described in [Guibas &amp; Stol '85]. The connected components of the (broken) boundary of the polyhedron are computed in a simple depth-rst search on the dual graph of the boundary of the polyhedron. This process also allows us either to orient all the facet polygons with consistent directions, or to detect that one or more components are not orientable. In the latter case we may either ignore the problematic components or halt the algorithm. Locating the borders of the connected components of the boundary is straightforward. We consider each oriented facet polygon as a formal sum of its directed edges, and add up all these facets, with the convention that ~ e + (?~ e) = 0. The resulting sum consists of all the border edges. Since each facet polygon is a directed cycle, the resulting sum is easily seen to represent a collection of pairwise edge disjoint directed

5.1 Border Discretization Each border polygon is discretized into a cyclic sequence of points. This is done by choosing some suciently small arc length parameter s, and generating equallyspaced points, at distance s apart from each other (along the polygon boundary). In analogy with the works on object recognition cited above, we may regard the resulting discretization as signatures (also called footprints) of the borders.

5.2 Voting for Border Matches Naturally, two parts of the original object boundary, which should have shared a common polygonal curve but were split apart in the approximation, must have similar sequences of footprints along their common curve (unless the approximation was very bad). This follows from our denition of the footprints as the 3-D coordinates of the points. Thus, our next goal is to search for pairs of suciently long subsequences that closely match each other. In our approach, two subsequences (pi ; : : : ; p i+`?1 ) and (q j ; : : : ; q j+`?1 ) are said to closely match each other, if, for some chosen parameter " &gt; 0, the number of indices k for which kp i+k ?q j+k k  " is suciently close to `. We perform the following voting process, where votes are given to good point-to-point matches. The borders are given as cyclic ordered sequences of vertices. We break each cycle at an arbitrary chosen vertex. Also, the direction of a border is implied by the chosen orientation of its connected component. Had it been chosen the other way, the border direction would have been reversed. A match between two border subsequences is called direct when the sequences of vertex indices of both borders are in the same (increasing or decreasing) order; a match is called inverted when one of the sequences is in an increasing order and the other is in a decreasing order. Note the following:  Adjacent components whose orientations are consistent should have an inverted All the border vertices are preprocessed for range-searching, so that, for each vertex v, we can eciently locate all the other vertices that lie in some "-neighborhood of v. We have used a simple heuristic projection method, which projects the vertices onto each of the X-, Y - and Z-axes, and sorts them along each axis. Given a query "neighborhood, we also project it onto each axis, retrieve the three subsets of vertices, each being the set of vertices whose projections fall inside the projected neighborhood on one of the axes, then choose the subset of smallest size, and nally test each of its members for actual containment in the query neighborhood. While this method may be inecient in the worst case, it works very well in practice. Orthogonal range queries can be answered more eciently by using the range tree data structure [Mehlhorn '84, p. 69], or by fractional cascading, like in [Chazelle '88]. The positions along a border sequence b, whose length is `b , are numbered from 0 to ` b ? 1. Assume that the querying vertex v is in position i of border sequence b1 . Then, each vertex retrieved by the query, which is in position j in border sequence b 2 , contributes a vote for the direct match between borders b1 and b2 with a shift equals to (j ? i) (mod `b2 ), and a vote for the match between the borders b1 (the inverted b 1 ) and b 2 , with a shift equals to (j ? (` b1 ?1?i)) (mod `b2 ). (The latter is the inverted match between the borders b1 and b2, as dened above.) As noted above, we allow only inverted matches between two portions of the same border or between borders which bound the same component; otherwise we would introduce a topological error by creating a non-orientable surface. All these cases are illustrated in Figure 2. Match (a) is direct, hence the orientation of one of the components should be inverted. The corresponding shift is (j?i). Match (b) is inverted, hence the two involved components are consistent. The corresponding shift is (j ? ` + i + 5), where the small indices are those of the inverted top border. Finally, match (c) is between a border to itself, where the shift, when inverting the right portion, is (2i ? ` + 9). Obviously, matches between long portions of borders are reected by a large number of votes for the appropriate shift between the matching borders. Since there might be small mismatches between the two portions of the matching borders, or the arc length along one portion may not exactly coincide with the arc length along the other portion, it is most likely that a real match will be manifested by a signicant input. It is rather our a priori estimation of the physical size of the gaps created between the original surfaces. Setting " to a value which is too small (or too large) may cause a degradation in the performance of the algorithm. In the rst case, close points will not be matched, thus border matches will not be found. In the second case, too many \false" votes will result in losing the correct border matches among too many incorrect matches. Note also that, due to the implementation, each pointto-point match actually contributes two votes, but this does not spoil the voting results. In almost all the cases small portions of the borders are included in more than one candidate match. This happens when several borders occur in close locations, usually at the junction of three or more cracks. We simply eliminate those portions common to more than one signicant candidate match.

<!-- image -->

(c)

(a) (b) Figure 2: Matches between borders peak of few successive shifts in the graph that plots the number of votes between two borders (possibly the same one) as a function of the mutual shift. Note that there might be several peaks in the same graph. This implies that there are several good matches with dierent alignments between the same pair of borders. Keeping track (for each peak) of the portions of the borders which voted for this alignment, we can infer the endpoints of the corresponding match (or matches) between these two borders. We extend the matches as much as possible, based on the neighborhood of the peak, allowing sporadic mismatches, insertions or deletions up to specied limits. Each match is given a score. The score may reect not only the number of votes for the appropriate shift, but also the Euclidean length of the match and its quality (measured by the closeness of the vertices on its two sides). Our settings of these parameters are described in Section 8. Note that the " parameter for the range-searching queries is not a function of the oriented in the opposite directions. In the absence of prior preferences, the score of the inverse assignment is equal to the score of the original one. We look for the assignment of border orientations whose score is maximal. Unfortunately, this problem turns out to be NP-Hard. In order to prove that, let us rephrase the problem using graph terminology: Choose-1. Consider a weighted graph G = (V;E). The vertices of V appear in pairs (vi ; v i ). The edges of E also appear in pairs: each pair either connects vi to vj and vi to vj , or connects vi to vj and vi to vj . Each pair of edges has the same weight. The problem is to choose one vertex out of each pair, such that the total weight of the edges connecting the selected vertices is maximal. The graph problem is equivalent to the border matching problem, at least in an abstract non-geometric setting. Each pair of vertices corresponds to the two possible orientations of a border. Each pair of edges represents a match: a straight link for an inverted match, and a cross link for a direct match. Selecting one vertex out of each pair stands for the choice of the orientation of the corresponding border, and the edges connecting the selected vertices correspond to the accepted border matches. The weights of the edges stand for the scores of the matches, and in both problems we search for an optimal (maximum) choice. (It is not clear whether each abstract instance of the graph problem has a geometric interpretation. Hence, our analysis will only imply that the abstract part of the border matching problem, whose input candidate matches may be arbitrary, is NP-Hard.) The following problem is known to be NP-Hard 1 [Garey &amp; Johnson '79, p. 210]: Max Cut. Given a graph G = (V; E), and a weight w(e) 2 Z + for each e 2 E, nd a partition of V into disjoint sets V 1 and V 2 such that the sum of the weights of the edges from E that have one endpoint in V1 and one endpoint in V2 is maximal. 1 Originally, it was stated as a decision problem which turned out to be NP-Complete. Similar reasoning to that in the proof given here shows that the decision version of Choose-1 is NP-Complete too.

input. It is trivial to verify that a selection of vertices in the Choose-1 problem which yields maximal weight of the corresponding selected edges implies optimal partition of the vertices in the Max Cut problem (where V1 is the set of all vertices v i for which v 1 i was selected, and V2 is the complementary set), and vice versa. 2 It is important to note that relaxing the requirement that the weights of the two edges of the same pair are equal does not make the problem any easier. Theorem 2 implies that the relaxed problem is NP-Hard too, or else the original Choose-1 problem would not have been NP-Hard. But even without Theorem 2 we could prove that the relaxed problem is NP-Hard, by a reduction from the problem of nding an Independent Set with a maximal size in a graph. In practice, though, getting an inconsistent collection of candidate matches is very rare (provided that we do not accept matches that are too short). This is because the boundary of real models is always intended to be orientable. In the rare cases where our algorithm erroneously produces a match that is wrongly directed, this usually happens when the match is very short, and then the two match orientations are suggested, thus overlap, and are therefore eliminated. It is trivial to decide whether the set of suggested matches is consistent or not. We do that in a DFS-like process on another graph, whose vertices are the connected components of the polyhedron boundary, and whose edges are the suggested matches between them. We arbitrarily choose a vertex (a component) as the start of the search, and assign to it an arbitrary orientation; each traversed edge (match) implies a consistent orientation of the newly visited component. All we have to do is to check consistency of all pairs of vertices connected by back-edges of the DFS. This step is applied for each connected component of the new graph. If the set of suggested matches is not consistent, we use the following simple heuristic. We maintain a collection of pairwise-disjoint sets of connected components of the polyhedron boundary, where the components in each set have been stitched together by already accepted matches. Initially, each connected component of the polyhedron is put in a separate singleton set. In each step we examine one match, in decreasing score order. If the match connects between components of dierent such sets, then we merge the two sets, (and if necessary, invert the orientation of all the components of one set), and accept the match. In case the match connects between two nd operations. During the traversal of the path from a component to the root of its set, we also compute the consistency state between the component and the root. Thus, if the match connects components of the same set, we can directly conclude whether the match is consistent with the previous ones or not, and accept only consistent matches. If the match connects components of dierent sets, we link the two sets, and reset the consistency bit of the component which ceases to be a root. It now points to the other root, which has just become the root of the merged set. Finally, in case one connected component of the polyhedron boundary has several borders, we must orient them with consistent directions. This can be done simply by adding articial \matches" between these borders, with scores which dominate those of real matches. Thus, their acceptance is guaranteed. We summarize below the complete procedure for pruning match suggestions: 1. Add new match suggestions for each connected component of the polyhedron boundary which has several borders. These matches should dene consistent directions to all the borders of a component, and should be scored higher than every original match suggestion (the scores are needed only if the set of match suggestions is not consistent). 2. Check whether all the match suggestions are consistent. For this purpose build a graph G m , where each connected component of the polyhedron boundary is a vertex in G m , and each match suggestion is an edge in G m . Choose arbitrarily a vertex of G m and assign an arbitrary orientation to it. Perform a depth-rst search in G m . For every edge of the search, assign an orientation to the newly visited vertex according to the match corresponding to the edge. For every back-edge of the search, check whether its corresponding match is consistent with the already assigned orientations of its two incident vertices. If all the back-edges are consistent, then accept all the match suggestions and go to step 5. Otherwise proceed to step 3. 3. Sort all the match suggestions according to their decreasing score order. Construct (makeset) a singleton set for each connected component Ci of the boundary, where i goes from 1 to the number of connected components. Maintain

6

i

Ski and Skj . Assume

now points to C 

j

as its parent.

Reset

(namely, their consistencies are

to the current vertices, u and v, in the two chains, and make a decision as to which chain should be advanced, say v advances to a new vertex w. Then we add the new triangle 4uvw to the boundary of the polyhedron (oriented consistently with the borders), and advance the current vertex (from v to w) along the appropriate chain. When we reach the last vertex of one chain, we may further advance only the other one. This process terminates when we reach the last vertices of both chains.

-  If k i 6= kj then accept the match suggestion and link without loss of generality that C  the inconsistency of C  i according to the accepted match.  If k i = kj and Ci and Cj are consistent either both true or both false), then accept the match suggestion.  Otherwise, if ki = kj and Ci and Cj are not consistent, then reject the match suggestion. 5. Orient all the connected components in a way consistent with the accepted matches. Each connected component is now labeled to indicate whether it should be inverted or not. If we reach this step from step 2, then each component is classied according to whether it is consistent with the root vertex of the DFS on G m (or with its local root, if G m is a forest). Otherwise, if we reach this step from step 4, then each component is classied according to whether it is consistent with the root of its set (again, there might be more than one set, if the accepted matches did not connect between all the components of the polyhedron boundary). Inverting a component is performed by inverting the directions of all the facets of the component. Filling the Gaps 6.1 Stitching the Matching Borders Each match consists of two directed polygonal chains, which are very close to each other in the three-dimensional space. We arbitrarily choose one end of the match, and \merge" the two chains as if they were sorted lists. In each step we have pointers

<!-- image -->

more overlapping matches, and are thus removed. Identifying the holes is done in exactly the same way the original borders were located (see Section 4). In fact, these holes are the borders of the new boundary after the stitching phase. We found out that the best way to x these holes was by a triangulation which minimizes the total area of the triangles. We, therefore, need to solve the following problem: Given a 3-dimensional closed polygonal curve P, and an objective function 2 That is, we advance so that the newly added triangle has smaller perimeter. Actually, we used the squares of the distances, with equally good results.

Figure 3: Stitching a match Several advancing rules were examined, and the following simplest one proved itself the best. Assume that the current vertices of the two chains are v 1 i and v 2 j , which are followed by v 1 i+1 and v 2 j+1 , respectively. Then, if jv 1 i v 1 i+1 j+jv 2 j v 1 i+1 j &lt; jv 1 i v 2 j+1 j+jv 2 j v 2 j+1 j, we advance the rst chain; otherwise we advance the second chain 2 . This bears close resemblance to the merging of two sorted lists, and turns out to produce reasonably looking match triangulations. Figure 3 shows such a triangulation. We may further examine the sequence of newly added triangles, and unify adjacent coplanar (or nearly coplanar) triangles into polygons with larger sizes. Alternative advancing rules were described in [Christiansen &amp; Sederberg '78] and in [Ganapathy &amp; Dennehy '82]. As an alternative to this merge simulation, we also examined match triangulation by the procedure described in Section 6.2. This method turned out to produce rather unaesthetic results, although it yielded a minimum-area triangulation. 6.2 Filling the Holes After stitching the borders, we are likely to remain with small holes at the junctions of cracks, as illustrated in Figure 5(g). These are 3-dimensional polygons, which are composed of portions of the borders which were not matched. This happens either because they do not meet the matching threshold, or because they belong to two or

1. For i = 0; 1; : : : ; n ? 2, let W i;i+1 := 0, and for i = 0; 1; : : : ; n ? 3, let W i;i+2 := F(vi; v i+1 ; v i+2 ). Put j := 2. 2. Put j := j + 1. For i = 0; 1; : : : ; n ? j ? 1 and k = i + j let Wi;k := min i&lt;m&lt;k [Wi;m + Wm;k + F(vi; vm; vk )]: Let Oi;k be the index m where the minimum is achieved. 3. If j &lt; n?1 then go to step 2; otherwise the weight of the minimal triangulation is W0;n?1 . 4. Let S := ;. Invoke the recursive function Trace with the parameters (0; n ? 1).

```
Function Trace (i; k): if i + 2 = k then S := S [ 4vivi+1vk ; else do: a. let o := O i;k ; b. if o 6= i + 1 then Trace (i; o); c. S := S [ 4vivovk ; d. if o 6= k ? 1 then Trace (o; k); od At the termination of the algorithm, S
```

contains the required triangulation of P. For our purposes, F(u; v; w) is taken to be the area of the triangle 4uvw. In practice, in order not to totally ignore the aesthetics of the triangulation, we actually added a measure of \beauty" of a triangle to the weight function F, by making it slightly depend on the lengths of the triangle edges and on the spatial relations between them. This avoided in most of the cases the creation of long skinny triangles. Figure 4 shows an example of a triangulation of a hole. As in the stitching process, we may also test edges shared by newly added triangles, and unify groups of adjacent coplanar (or nearly coplanar) triangles into polygons with larger sizes.

<!-- image -->

nature of hashing, and does not assume anything about the geometry of the input polyhedron to the algorithm. Nevertheless, it assumes a reasonable choice of the proximity parameter ", which should yield in average a constant number of output vertices for each range-searching query. Improper choice of ", say equal to the size of the whole object, will result in -(n 2 ) access operations to the hash table, but no matches will be identied in this case. We infer the matches from the voting results also in O(n) time. (We could also achieve an O(n log 2 n) deterministic running time by using fractional cascading.) Since choosing the maximal consistent set of matches is NP-Hard (at least in its

Figure 4: Minimum area triangulation of a hole 7 Complexity Analysis We measure the complexity of the algorithm as a function of two variables: k, the size of the input (say, the number of vertices of the original object), and n, the total number of vertices along the border edges after the discretization. We denote the number of components by c, and the number of match suggestions by m. Naturally, c and m are very much less than n. We also denote the number of triangulated holes by h, and the complexity of the ith hole (i = 1; : : : ; h) by ` i . We do not regard the computation of the connectivity (facets adjacency) information as part of our algorithm, since this should be part of the input. However, our preprocessing step generates this information, if needed, in O(k log k) time, which is dictated by the sorting of the input vertices. The connected components of the boundary of the polyhedron can be found in time linear in k. The time needed for nding their borders is also O(k). The discretization of the borders takes O(n) time. As in [Hong &amp; Wolfson '88], the voting step, if it uses a hash table, can be executed with expected O(n) running time. This expected running time is due to the the number of holes is linear in the number of surfaces (and hence borders), and their size is very small. The size of a hole is usually bounded by some constant (especially with a proper choice of the proximity parameters that control the voting process), so this step, although it is asymptotically inecient, does not require more than O(n) time in practice. To conclude, the whole algorithm runs on practical instances in average O(k + n) time, which is optimal. If we also count the connectivity computation in the preprocessing, the algorithm runs in expected O(k log k+n) time. In unrealistic cases, where the hole(s) are as complex as the polyhedral boundary itself, the running time may climb to as high as O(k + n 3 ) (or to O(k log k + n 3 ), when we also compute the connectivity). The following section describes our rather comprehensive experimentation with the algorithm. In all cases that we tried, the running time was indeed small, and the cost of the (theoretically expensive) hole-stitching step was invariably negligible.

8 Experimental Results We have implemented the whole algorithm on a Digital DECstation 5000/240 and on a Sun SparcStation II in C. We have experimented with dozens of CAD les whose boundaries contained gaps, and obtained excellent results in most of the cases. These CAD les were generated by various CAD systems, such as Euclid-IS (vendor: Matra Datavision), Unigraphics (McDonnell Douglas), Catia (IBM/Dassot), CADDS-4X (Computer Vision), ME (Hewlett Packard), Pro-Engineer (Parametric Technologies), and many others. We note that most of the problems occurred when stand-alone computer programs translated curved objects from neutral le format, e.g. VDAFS [VDA '87] or IGES [NIST '91], into their polyhedral approximations. Fewer problems appeared when the CAD systems performed the same task, using a built-in function which converts the internal data into an output le which contains a polyhedron description. The les described industrial models, primarily parts and subunits of the 3 (m;n) is an extremely slowly growing functional inverse of Ackermann's function. For all practical values of m and n, (m; n) does not exceed a very small constant; cf. [Tarjan '83].

P was its perimeter, and R was the ratio between the largest and the smallest of its three edges. All these parameters were user dened, but modifying them did not achieve any better results. Therefore, the program set these defaults for the parameters, except for the voting threshold ". As noted above, this should be the user estimation of the physical size of the gaps between the original surfaces. Although there was a large enough range of valid " settings, the algorithm failed more often if this parameter was chosen improperly. A too small " resulted in the loss of matches due to the

lack of votes. On the other hand, a too large " resulted in a noisy voting table. This usually caused \intuitive" matches not to have scores suciently larger than incorrect match suggestions. Even when such intuitive matches were identied, they overlapped with erroneous match suggestions and were therefore eliminated. In all cases, too many unmatched border portions were passed to the triangulation step. When the remaining holes were not local (with respect to the geometric features of the original model), their triangulations constructed new surfaces far beyond the intention of the model designer. Figure 5(a) shows a synthetic example of an object similar to a cube, whose boundary was broken into eight components, none of which is planar. Figure 5(b) shows the same object in a wire-frame representation. Figures 5(c,d) show the object after the whole repairing process. Figures 5(e,f,g,h) show the dierent steps of the algorithm. The borders are shown in (e), the matches (after stitching the borders) are shown in (f), the remaining holes are shown (with the same perspective view) in (g), and their minimal area triangulations are shown in (h). Figure 6(a) shows another synthetic example, where a sphere contains very small holes in the poles. Figures 6(b,c) show one of the poles, before and after triangulating

the hole, respectively. Real examples turned out to be simpler, since the matching borders were much longer and closer than in the synthetic examples. Many polyhedral approximations of surface-modeling objects were repaired using the algorithm described in this paper. A typical example (a hollowed trimmed box) is shown in Figure 7. The front side of the object is shown in Figure 7(a), whereas its bottom side is shown in Figure 7(b).

<!-- image -->

(e)

(f)

(g)

(h)

Figure 5: A synthetic example

<!-- image -->

Figure 6: A sphere with perforated poles

(a)

(b)

(c)

(a)

(b)

(c)

<!-- image -->

Figure 7: A real example

<!-- image -->

(a) (b) Figure 8: Three intersecting \drums" The borders are shown in Figure 7(c), and all of them are matched, leaving only six holes. Since the holes are so small, they appear as dots in this scale. Figures 7(d,e) are closeups of the area near the left intersection of the two pairs of circular borders (as seen in (c)), before and after stitching, respectively. Figures 7(f,g) are closeups of a hole before and after its minimum-area triangulation. Figures 8 and 9 show two more real cases. Figure 8(a) shows two intersecting

\drums", where a third part protrudes out of the near drum. Figure 8(b) is a closeup of two tiled matches (from a dierent point of view), in a wire-frame representation. Figure 9(a) shows a complex of ve open hollowed cylinders. One long gap separates between the inner and outer surfaces of the complex. Two portions of the single border, that completely bounds this gap, were fully matched and tiled. Figure 9(b,c) are closeups of the tiled match. Figure 10 shows three more real examples which our algorithm fully cured. We summarize the performance of our implementation on all the examples described above in Table 1. All the time measurements were taken on a Digital DECstation 5000/240. 9 Conclusion We have proposed in this paper an algorithm for solving a practical problem, namely the detection and repair of gaps in the boundary of a polyhedron. This problem arises quite frequently in practice, during the computation of polyhedral approximations of CAD models, whose boundaries are described using curved entities of higher levels. We applied several techniques in our solution. First, we used a partial curve matching technique, adapted from computer vision, for identifying matching border

(a)

<!-- image -->

Figure 9: A complex of ve open hollowed cylinders Figure 10: Three more real examples Model Synthetic Sphere Box Drums Cylinders Impeller Drawer Adapter Complexity Border Table 1: Performance of the algorithm Vertices

<!-- image -->

45

432

1,545

826

1,571

7,284

458

6,539

Edges

111

2,448

3,244

1,712

3,132

43,635

2,817

44,682

Model Facets

37

816

609

372

783

14,545

939

14,894

Borders

Stitching

8

2

32

19

1

11

1

27

Points

278

48

5,305

3,106

1,592

47

203

84

Time (Seconds)

Triangulation Matches Total

18

0

30

35

1

0

1

0

Holes

12

2

6

22

0

11

0

27

Hole

Points

61

48

47

233

0

47

0

84

|                                   |                                          | 0.20 0.06 0.60 0.20   | 0.68 0.19 10.74 6.15   |
|-----------------------------------|------------------------------------------|-----------------------|------------------------|
|                                   |                                          | 0.00                  | 3.60                   |
|                                   | 0.48 0.13 10.14 5.95 3.60 1.12 0.24 2.48 |                       |                        |
| Synthetic Sphere Box Drums        | Synthetic Sphere Box Drums               |                       |                        |
|                                   |                                          | 0.35 0.00 0.06        |                        |
| Cylinders Impeller Drawer Adapter | Cylinders Impeller Drawer Adapter        |                       | 1.47 0.24 2.54         |

The authors wish to thank Haim Wolfson for useful discussions concerning the ge-

ometric hashing technique and its applications.

thank Cubital Ltd. for contributing data les.

The rst author would also like to tidimensional searching, SIAM J. of Computing 17 (3), 427{462.

Guibas, L. and Stol, J. (1985), Primitives for the manipulation of general subdi-

visions and the computation of Voronoi diagrams, ACM Transactions on Graphics 4

Christiansen, H.N. and Sederberg, T.W. (1978), Conversion of complex contour line

(2), 74{123.

denitions into polygonal element mosaics, Computer Graphics 13, 187{192.

Ho-Le, K. (1988), Finite element mesh generation methods:

Dolenc, A. and M akel

a, I. (1991), Optimized triangulation of parametric surfaces,

cation, Computer-Aided Design 20 (1), 27{38.

Technical Report TKO-B74, Helsinki University of Technology, to appear in Mathe-

matics of Surfaces IV.

Hong, J. and Wolfson, H.J. (1988), An improved model-based matching method using

footprints, Proc. 9th Int. Conf. on Pattern Recognition, 72{78.

A review and classi-

Foley, J.D. and Van Dam, A. (1984), Fundamentals of Interactive Computer Graph-

ics, Addison Wesley.

Galler, B.A. and Fischer, M.J. (1964), An improved equivalence algorithm, Comm.

of the ACM 7, 301{303.

Ganapathy, S. and Dennehy, T.G. (1982), A new general triangulation method for

planar contours, ACM Transactions on Computer Graphics 16 (3), 69{75.

Garey, M.R. and Johnson, D.S. (1979), Computers and Intractability, A Guide to

the Theory of NP-Completeness, Freeman and Co., San Francisco.

Klincsek, G.T. (1980), Minimal triangulations of polygonal domains, Annals of Dis-

Schwartz, J.T. and Sharir, M. (1987), Identication of partially obscured objects in

crete Mathematics 9, 121{123.

two and three dimensions by matching noisy characteristic curves, Int. J. of Robotics

Research 6 (2), 29{44.

M akel

a, I. and Dolenc A. (1993), Some ecient procedures for correcting triangu-

lated models, Proc. Symp. on Solid Freeform Fabrication, Dept. of Mech. Eng., Univ.

Sheng, X. and Hirsch, B.E. (1992), Triangulation of trimmed surfaces in paramet-

of Texas at Austin.

ric space, Computer-Aided Design 24 (8), 437{444.

Mehlhorn, K. (1984), Data Structures and Algorithms 3:

Multi-Dimensional Search-

Sheng, X. and Tucholke, U. (1991), On triangulating surface model for SLA, Proc.

ing and Computational Geometry, Brauer, W., Rozenberg G. and Salomaa A., eds.,

2nd Int. Conf. on Rapid Prototyping, Dayton, OH, 236{239.

Springer-Verlag.

Snyder, W.E., Groshong, R., Hsiao, M., Boone, K.L. and Hudacko, T. (1992), Closing

Pratt, W.K. (1978), Digital Image Processing, Wiley, NY.

gaps in edges and surfaces, Image and Vision Computing 10 (8), 523{531.

Rock, S.J. and Wozny, M.J. (1992), Generating topological information from a \bucket

of facets", Proc. Symp. on Solid Freeform Fabrication, Dept. of Mech. Eng., Univ. of

Texas at Austin, 86{94.

Samet, H. and Webber, R.E. (1988), Hierarchical data structures and algorithms

for computer graphics; Part II: Applications, IEEE Computer Graphics &amp; Applica-

tion 8 (4), 59{75.

and Machine Intelligence 12 (5), 483{489.

3D Systems, Inc. (1988), Stereolithography Interface Specication.