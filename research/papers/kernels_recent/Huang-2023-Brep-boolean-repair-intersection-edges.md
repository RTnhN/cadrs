## B-REP BOOLEAN RESULTING MODEL REPAIR BY CORRECTING INTERSECTION EDGES BASED ON INFERENCE PROCEDURE

## Haomian Huang*, Li Chen, Enya Shen, Jianmin Wang

School of Software Tsinghua University Beijing, China

## ABSTRACT

As the most essential part of CAD modeling operations, boolean operations on B-rep CAD models often suffer from errors. Errors caused by geometric precision or numerical uncertainty are hard to eliminate. They will reduce the reliability of boolean operations and damage the integrity of the resulting models. And it is difficult to repair false boolean resulting models damaged by errors. In practice, we find that the illegal boolean resulting models stem from the false intersection edges caused by errors. Therefore, this paper proposes an automatic method based on set reasoning to repair flawed structures of the boolean resulting models by correcting their topological intersection edges. We provide a local adaptive tolerance estimation method for each intersection edge based on its geometric features as well as its origin. Then, we propose a set of inference mechanisms based on set operations to infer whether a repair is needed based on the tolerance value and how to correct the inaccurate intersection edge. Our inference strategies are strictly proven, ensuring the reliability and robustness of the repair process. The inference process will transform the problem into a geometric equivalent form less susceptible to errors to get a more accurate intersection edge. Since our inference procedure focuses on topological features, our method can repair the flawed boolean resulting models, no matter what source of errors causes the problem.

K eywords B-rep, Boolean operations, Topological repair, Inference procedure

## 1 Introduction

Model problems caused by errors are common in the process of boolean operations in CAD kernels. Based on their sources, there are four main kinds of errors[1]: inaccurate input data, approximation errors, floating-point arithmetic errors, and inherent accuracy limitations of the geometric intersection algorithms. If the CAD kernels cannot correctly deal with these errors, then the topology of the resulting models may be corrupted. It will generate some flawed structures that users do not expect. Furthermore, models with topology errors may cause the corruption of the program when users do some subsequent operations on them. Therefore, without a solution to error problems, CAD software cannot be applied in practice.

To tackle these error problems, many researchers have done a lot of work. One solution is to implement a more accurate geometric intersection algorithm. There are many numerical methods to solve polynomial systems that arise in geometric intersections. Newton's method[2] is simple

to implement, but it cannot find all solutions and always gets stuck in local optimums. There are some methods for solving the global optimization problem in polynomial systems[3], but these methods lack accuracy and robustness. To overcome these disadvantages, researchers proposed subdivision methods to properly divide the intersection areas into multiple regions and then use different strategies to find the solutions based on each region's conditions[4]. However, all these improved intersection algorithms only reduce the latter two kinds of errors. Besides, they do not enhance CAD kernels' ability to deal with the accumulating errors, which will break models after a sequence of operations. Another way to deal with error problems is tolerant modeling, which enables CAD software to accept less precise data as input. It allows every vertex and edge to have its tolerance value. These local tolerance values can heal the flawed structures caused by errors like leaky and disconnection while maintaining the accuracy and integrity of the geometric shapes. But setting appropriate tolerance values can be a difficult task. And im- proper local tolerance values will cause inconsistencies at touch areas between two different models when assembling or self-intersection inside the topological entity, corrupting the whole model. What is worse, the tolerance values will bring uncertainty and inconsistency to the boolean operations, which generate flawed structures in the resulting models.

With the development of artificial intelligence, neural networks have been used to process CAD models. However, due to the complexity and ambiguity of B-rep models, neural networks nowadays can only handle some elementary tasks like segmentation and classification, and even then, there is still a high chance of misjudgment, which is unacceptable in practice. CAD kernels require that the algorithms must be highly accurate and stable. Otherwise, a small error will lead to unexpected wrong results.

Consequently, the solution to error problems must ensure the correctness of the resulting models regardless of the source of errors. Moreover, the method must be highly accurate and stable. Only then can this method be embedded into the CAD kernel and actually used in practice. According to our hundreds of thousands of tests on the common shapes of industrial parts, we find that the main reason why boolean operations fail is the intersection inconsistency caused by errors. Therefore, in this paper, we focus on repairing boolean resulting models with flawed structures by correcting the inaccurate intersection edges. Based on the topology level of the intersection inconsistency, we classify it into three types: face-face, edge-face, and edge-edge. For each type of scenario, a well-designed inference procedure is proposed to detect and repair this kind of inconsistency. All inference procedures are strictly proven and will help us avoid misjudgment, which will significantly enhance the accuracy and stability of the repair method. Our method first calculates appropriate tolerance values for every intersection edge. Then, we combine the intersection edges into the structure of the intersection graph, introduced in section 4.2 . Then, we use the tolerance values of each edge and the intersection graph to direct the inference procedures, which will only correct inaccurate intersection edges that will cause flawed structures in the resulting models. This method has the following contributions:

·Proposing a framework for repairing boolean resulting models by correcting intersection edges. In previous work, some researchers have noticed the intersection inconsistency in polyhedral models[5], but to the best of our knowledge, we are the first to improve the boolean results of parametric B-rep models based on their intersection edges.

·Organizing intersection edges into an intersection graph, which carries much geometric and topological information about intersection edges and the resulting model, to help make a reasonable repair. Compared with extracting information from resulting models, information from intersection edges is more intuitive, and the process is more efficient due to the simple topological structure of intersection graphs. With the help of intersection graphs, we can preserve the tiny features of the resulting models very well in the repair process.

·Valid inference procedures which use topological features to detect and correct inaccurate intersection edges. Our inference process will locate the problem intersection edges and fix them with more accurate results. This process is directed by the feature-based tolerance values of intersection edges, which are calculated automatically. Therefore, it is very friendly for users. Besides, Our inference strategies are proved by set reasoning, which promises the soundness of our algorithms.

## 2 Related Work

Robust boolean operations on plane-based models A considerable part of boolean operation errors come from the process of intersection calculation. In plane-based models like mesh, there are several methods to reduce the errors. An implicit plane-based boundary representation[6] is proposed to solve this problem. In this method, each point is implicitly represented as the intersection of three planes, and each straight line is implicitly represented as the intersection of two planes. Besides, a technology called filtered floating point predicate[7] is introduced to reduce float errors. However, this method will hurt the efficiency of the whole process. Therefore, homogeneous coordinates[8] are used to totally avoid float errors. By this method, all float point operations are converted into integer operations, but using homogeneous coordinates will make some algorithms difficult to implement, such as the triangulation of mesh grids. That is why this method cannot be widely used. And in previous work, researchers had noticed that the intersection inconsistency between two models was the main reason why boolean operations failed. Therefore, they proposed robust boolean operations on polyhedral solids based on cross-section graphs and implicit representation[5]. A cross-section graph comprises the intersection of a solid with a face. Edges in the graph are well-oriented to partition the areas of the face as being inside, on, or outside the solid. Besides, they perform robust incidence tests to help classify the areas. By using the implicit representation of points and lines, an incidence test can be transformed into multiple point-on-plane problems, which will enhance the accuracy. However, these above methods can only be applied on straight lines and planes. In industrial design, B-rep models based on parametric curves and surfaces are more widely used than polyhedral models. Unlike mesh or polyhedral models, whose errors mainly come from float errors, parametric B-rep models' error composition is more complex. Complex curves and surfaces need to be fitted with splines, which bring approximation errors after the formation of B-rep models. Moreover, the intersection of curves and surfaces is more complicated than that of straight lines and planes in mesh models. The intersection algorithms have calculation errors except for float errors, which will continue accumulating during the modeling process. Therefore, tolerant modeling technology[9] is proposed to solve the problem of model illegality caused by errors.

Tolerant modeling Generally, precise models are best for users. But for the above reasons, it is hard to get accurate data in some situations. Tolerant modeling allows the program to accept less accurate data while keeping the validity of topology structures[10]. Mainstream CAD kernels have multiple kinds of global tolerance variables. For example, the positional tolerance value is the smallest distance between two different topological vertices, and the fitting tolerance value is used to guide the accuracy of fitting algorithms for complex curves and surfaces. The global tolerance method is efficient, and it can handle most situations. But more is needed to solve some exceptional cases. And it will be difficult to transfer models' data from one CAD kernel to another with different tolerance settings. To avoid these disadvantages, local tolerant modeling is proposed[9]. It allows basic topology structures such as edges and vertices to have their own tolerance values, which does not impact the robustness and reliability of the precise modeling while strongly enhancing the ability to accept less accurate data to the model.

Tolerant analysis Determining suitable tolerance values for topology structures and whether present tolerance values can satisfy following operations become a crucial topic in tolerant modeling[11]. Tolerance-map(T-map)[12] is a hypothetical Euclidean-Space point set representing all probabilities of a feature in its tolerance zone. When assembling two components with the same features, the tolerance propagates by calculating the Minkowski sum of two T-maps. After this method was proposed, T-maps were developed for other features quickly[13]. However, the T-maps of some kinds of features are high dimensional, which makes the calculation of the Minkowski sum inefficient[14]. The matrix model[15] uses a displacement matrix to describe the small displacements of a feature in its tolerance zone[16]. Therefore, all points of this feature can be bounded by the inequality resulting from the combination of displacement matrix and tolerance values. The calculation can be very efficient, but different choices of the point may lead to different results[17], which will cause some ambiguities when tolerance propagates between different models.

Repair directly on resulting models Repairing flawed structures caused by errors in resulting models is another way to handle error problems. In the exact B-rep repairing method, models are checked and corrected by analyzing their topological and geometric elements[18]. And in the faceted B-rep repairing method, original models are approximated as polyhedrons, making the check and repair process more efficient[19]. However, errors during the previous process may make the resulting models totally collapse, which means these methods can only correct limited kinds of errors. Therefore, a repair method based on design history is proposed[20]. In some CAD kernels, model data includes not only its topological and geometric elements but also its design history, which tells the users how this model is constructed. Consequently, even if the resulting model collapses due to some errors, the program can still reconstruct the model partly and detect where the errors occur. However, this method depends on the definitions of error types, which may falsely modify the flawless models. Besides, analyzing design history data is time-consuming, and some CAD systems do not have such information for each model.

Deep learning Recently, neural networks have been used to process CAD B-rep models. Compared with mesh and point cloud data, B-rep models formed by parametric curves and surfaces are much more complex, and few neural networks can consume such B-rep data directly. To overcome the difficulty, UV-Net[21] extracts geometric features from the parameter domain as regular grids(UVgrids) and uses a face-adjacency graph representing the topological connection to form the structure of the graph neural network. Then, image convolutions on the curve and surface UV-grids are performed to get their hidden features. The hidden features, which are the output of the CNNs, are then treated as input edge and node features to the graph neural network. BRepNet[22], however, uses convolution networks only. The input of the convolution networks is the feature vectors that encode the geometric information of the faces, edges, and coedges. Compared with UV-grids, feature vectors are much simpler and only include some basic information like the geometric types, the area of a face, and the length of an edge. Therefore, these features can be extracted very efficiently. And the convolutional kernels are well-designed based on the index array of topological entities and their topological connections, which enables the hidden features to transfer from one entity to its neighbor entity. The output of the above networks can achieve satisfactory results in several downstream applications like segmentation and classification. However, due to the complexity and ambiguity of B-rep models, no neural network currently can detect the flawed structures of boolean resulting models, let alone repair them.

In our method, we detect errors before forming resulting models in boolean operations by analyzing the intersection edges, and we only use the topological and geometric information of the models that every CAD kernel will provide for users. Besides, our method does not need to improve the geometric intersection algorithm, which will make the algorithm easy to apply in other CAD kernels. Moreover, the strict inference process ensures our method does not modify any flawless structures of models.

## 3 Background Knowledge

## 3.1 Boolean operations

As the most essential part of the CAD kernel, boolean operations greatly suffer from error problems. Boolean operations are used to construct complex models based on basic models like cubes, cones, and torus. Each boolean operation of two models can be divided into four stages:

① Intersection. Calculate the face-face intersection of each face of the two models separately, and get all the intersection edges.

- ② Imprint. Imprint all these intersection edges onto both models and split each face of the two bodies. After this step, all faces of the two models fit the following propertythe face of a model either lies totally inside the body of the other model, or totally outside, or totally on the boundary of the other model.
- ③ Classification. Determine whether each face should be kept or discarded based on the input boolean operator.

④ Merge. Merge all the remaining faces into one body as the final result of the boolean operations, and correctly maintain its topology tree to ensure the resulting model's topology is legal.

## 3.2 Geometric Intersection and Topological Intersection

In the boundary representation(B-rep) model, the shape of an entity is decided by its geometric element, and its boundary is composed of lower-dimension entities. For example, faces are bounded by edges, and edges are bounded by vertices. Faces, edges, and vertices are direct compositions of a model, and we call them topological structures because they will connect with other topological structures to form a model. And the geometric element of a face, an edge, or a vertex, we call it geometric structure. A geometric structure will not connect with any other geometric structure. It is a mathematical description of a shape. To distinguish them, we call the geometric structure of a face a surface, the geometric structure of an edge a curve, and the geometric structure of a vertex a point(position). Therefore, there are two kinds of intersection ways: geometric and topological.

Geometric intersections will not consider any boundary information of its corresponding topological entity. It is the computation that happens between geometry structures.

Topological intersections, as the name indicates, will consider the boundary of the topological entity. It will filter the results of geometric intersections and leave the valid parts inside the boundary of the corresponding topological entities. The process of topological intersection runs following the order from high dimensions to low dimensions. Generally, a face-face intersection process will run multiple times edge-face intersection processes to get boundary information of each face. So will the edge-face intersection process run multiple times edge-edge intersection. Because high-dimension topological intersections will use the information returned from lower-dimension topological intersections, in this process, errors will accumulate. Therefore, the accuracy of higher dimension topological intersection will be lower.

## 3.3 Topology Set Computation and Inference Method

We introduce the inference method based on the already known topological information to determine whether intersection results need to be fixed. In order to describe the inference process better, we specify the following topological set notation:

Set containing the topological entity

a

- a itself and all its sub topological structu -res
- a ∩ b Set containing the topological intersect -ion result of entity a and entity b
- a ∈ b Entity a is a sub topological structure of entity b

For example, face f represents not only the face itself but contains all sub-topological structures, including all its boundary edges and all vertices of these edges. e ∩ f is the topological entity set containing the topological intersection result of edge e and face f . If e is a boundary edge of f , then e ∈ f .

Obviously, topology set computation satisfies commutative and associative laws, that is:

<!-- formula-not-decoded -->

Besides, there are two extra inference rules based on the feature of topological entities and topological intersections:

<!-- formula-not-decoded -->

That is, if edge e is a boundary edge of face f , then the intersection result of e and f is e itself.

Rule 3.2. v ∈ e ∩ f ⇒ v = e ∩ f , in a sufficiently small neighborhood of v .

That is, if vertex v is one of the intersection resulting entities of e and f , then v is the only intersection result of e and f within a sufficiently small neighborhood of v .

Using these two rules, we can have the following lemma:

Lemma 3.1. e 1 ∈ f 1 , e 2 ∈ f 2 , e 1 ∩ e 2 = v 0 ⇒ f 1 ∩ e 2 = v 0 , e 1 ∩ f 2 = v 0 , in a sufficiently small neighborhood of v 0

The proof process is shown as follows:

Proof 3.1. Using Rule 3.1 , we know that

<!-- formula-not-decoded -->

Therefore,

<!-- formula-not-decoded -->

Using Rule 3.2 , in a sufficiently small neighborhood of v 0 , we know that

<!-- formula-not-decoded -->

Similarly, we can get v 0 = f 1 ∩ e 2 in a sufficiently small neighborhood of v 0 .

This lemma will help us to simplify problems in some situations.

## 4 Method

## 4.1 Feature-based Tolerance Value for Intersection Edges

To use Lemma 3.1 , we should run our inference procedure in a local area. Therefore, we should calculate a proper tolerance value for each intersection edge to direct the inference process. Global tolerance can handle many situations. However, in our experiments, we find that using a global tolerance value will ignore fixing some flawed cases, especially when the shape of an intersection edge is complex. Consequently, we need to examine the features of each intersection edge and calculate an appropriate local tolerance value for it.

Intersection edges with different geometry have different accuracy. For instance, the error of a straight line is always much smaller than that of a high-degree spline curve. Therefore, the higher the degree of the mathematical expression of the geometry curve of an intersection edge, the larger its tolerance value should be.

Besides an intersection edge's geometry, the shapes of two faces that generate the intersection edge are also important factors. A straight line generated by two planes is more precise than two spline surfaces. Consequently, intersection edges coming from two faces with higher-order surfaces should have larger tolerance values.

Because different geometric intersection algorithms have different accuracy when dealing with different situations, it will take much work to determine an accurate tolerance value for an intersection edge in different situations. In practice, we find the tolerance values that can be used to direct inference procedure are positively correlated with several factors, so we have the following simple empirical formula to calculate a feature-based tolerance value t for an intersection edge e 0 generated by the intersection of face f 1 and f 2 :

<!-- formula-not-decoded -->

In the above formula, d means the maximum degree of the mathematical expression of the edge or face's geometry. t 0 means the default global positional tolerance value. Since errors may accumulate during the calculation process, we connect these coefficients using multiplication signs. Therefore, in the most straightforward situation, two intersecting planes making a straight line, the default tolerance value is significant enough to direct the inference process when k equals 1. And the coefficient k describes the level of the model's geometry deviating from the actual position. In some situations, the geometry of the faces or edges may be represented by fitting spline. And the fitting spline has its inherent numerical inaccuracy. Therefore, k is defined by the following formula:

<!-- formula-not-decoded -->

In the above formula, t f 1 , t f 2 and t e 0 are separately the fitting errors of f 1 , f 2 and e 0 . Because fitting errors are always much larger than the global positional tolerance value, their ratios can reach a value of more than 1000. Consequently, considering only the largest ratio is enough for us to achieve a large enough k value.

## 4.2 Intersection Graph

The intersection graph is a topological structure consisting of intersection edges and their vertices, which describes the intersection result of two models. The intersection edges with vertices in the same position will be connected, and these vertices in the same position will be merged into one.

Figure 1: (a) two edges are connected head to tail, e 1 is e 2 's successor and e 2 is e 1 's predecessor. (b) two edges are connected tail to tail, e 1 is e 2 's predecessor and e 2 is e 1 's predecessor. (c)two edges are connected head to head, e 1 is e 2 's successor and e 2 is e 1 's successor.

<!-- image -->

In the intersection graph, each edge has a predecessor and a successor based on its own direction, pointing from its start vertex to its end vertex. However, two neighbor edges in the graph do not need to be connected head to tail, which means their relative orientation can be arbitrary. Shown in Fig.1, the predecessor and successor of an edge must follow the rule that the predecessor is one of the neighbor edges that is connected to its start vertex, and the successor is one of the neighbor edges that connected to its end vertex. This design allows the intersection graph to have branches, shown in Fig.2. When branching at a vertex, all edges sharing this vertex will be organized like a circular linked list, and the order of the edges in this list is not unique.

Figure 2: Four edges share one vertex and make a branch at this vertex. Based on their directions, there is a legal connection ways that e 1 's successor is e 2 , e 2 's successor is e 3 , e 3 's successor is e 4 , and e 4 's predecessor is e 1 . The legal ways to connect these edges are not unique. There is another way that e 1 's successor is e 3 , e 3 's successor is e 4 , e 4 's predecessor is e 2 , and e 2 's successor is e 1 .

<!-- image -->

Specially, the predecessor of an edge will be itself when its start vertex does not connect to other edges. And similarly, if the end vertex of an edge does not connect to other edges, the successor will be itself. This feature will make adding or deleting an edge in the intersection graph more convenient since we do not need to distinguish whether this edge connects to other edges.

Although intersection graphs' topological structures are quite simple, they carry much geometric and topological information about the resulting models. Compared with extracting features from the resulting models, the information from intersection graphs is more intuitive and accurate since the extracting steps may bring ambiguity or ignore some small but important features. With the information given by intersection graphs, our inference procedures can be more robust and valid.

## 4.3 Inference Algorithms

In the repair process, we must ensure that we will not modify any accurate edges while calculating a more precise result for the flawed intersection edges. And our inference algorithms can satisfy this by using the following three strategies in different situations:

① Transform a high-dimension intersection problem into a low-dimension intersection problem to get a more accurate intersection result since high-dimension topological intersections are more affected by accumulating errors than low-dimension topological intersections.

② Use the intersection graph to get more information from other parts of the model, which can help avoid misjudging. Intersection graphs can help us identify some small features that are easily mistaken for flawed structures generated by errors.

③ Use the information we already know to substitute the intermediate result, which can help get a more accurate final result. Because of the accumulation of errors, using the intermediate result in computation will further reduce accuracy.

As mentioned earlier in this paper, the main reason why boolean operations fail is intersection inconsistencies caused by errors. And there are three types of intersection inconsistency classified by their topology level: face-face, edge-face, and edge-edge. In the following part, we will separately discuss three topological manifestations of the three types of inconsistency and how our inference algorithms detect and repair them.

## 4.3.1 Disconnection of two Intersection Edges that Should Be Connected

When computing the intersection result of two models face by face, the geometric intersection algorithm does not use the information of neighbor faces, which may cause some disconnections on the boundary of faces, shown in Fig.3. Because of the accumulating errors, two intersection edges on two separate faces, which should be connected, break up on the common boundary of the two faces. e 01 is the intersection edge of face f 0 (not showing in this map) and face f 1 , while e 02 is the intersection edge of face f 0 and face f 2 . e 12 is the common edge of f 1 and f 2 . v 1 is the intersection vertex of e 1 and e 2 , while v 1 is the intersection vertex of e 01 and e 2 . As f 0 crosses through e 12 , v 1 and v 2 should be at the same position in theory. However, because the intersection algorithm calculates the intersections of f 0 and f 1 , f 0 and f 2 separately, the results of the two intersections can be inconsistent due to the accumulation of multiple kinds of errors. This inconsistency is manifested by the dislocation of v 1 and v 2 , which makes two intersection edges on the same face f 0 disconnected. Usually, the inaccuracy of the shape of intersection edges will not make the following boolean steps fail. However, in this situation, not only will this disconnection make the resulting models inaccurate, but more seriously, it will make the area that should be bounded by intersection edges open, which will result in an illegal topological structure and make the following boolean operations crash. Therefore, this disconnection must be fixed to ensure the resulting model is legal and right.

There are three criteria we used to judge whether this situation occurs on two intersection edges e 01 and e 02 , and these criteria should be satisfied simultaneously:

- ① e 01 and e 02 have a pair of vertices v 1 and v 2 belonging separately to these two intersection edges that are very closed judged by the feature-based tolerance values of the edges.
- ② e 01 and e 02 have a common face f 0 which generates them from the intersection between itself and other two separate faces f 1 and f 2 .
- ③ f 1 and f 2 share common edges, and v 1 and v 2 are both on one of the common edges e 12 .

<!-- image -->

e12

Figure 3: Sketch map of the disconnection of two intersection edges.

The proof of why the three criteria satisfy this situation is shown as follows:

## Proof 4.1. from criterion ② , we know that:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

from criterion ① and criterion ③ , we know that:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

from criterion ③ , we know that:

<!-- formula-not-decoded -->

Therefore, from (3), (1), and (5), we know that:

v

1

=

e

= (

01

f

=

0

f

0

∩

∩

∩

e

12

f

(

=

f

0

∩

Similarly, we can get that

<!-- formula-not-decoded -->

From criterion ① , we know that v 1 and v 2 are very close. Therefore, from (6) and (7), we know that v 1 and v 2 should be at the same position, and the position is the position of the intersection vertex of f 0 and e 12

1

)

f

e

1

12

∩

∩

(

f

f

2

1

)

∩

f

2

)

```
(2) (3) (4) (5) Algorithm 1: Judge Whether the Disconnection of Two Intersection Edges Occurs input : e 1 , e 2 output : Whether two intersection edges meet all three criteria v 11 , v 12 ← getStartAndEndVertex ( e 1 ) ; v 21 , v 22 ← getStartAndEndVertex ( e 1 ) ; t ← getFeatureBasedTolerance ( e 1 ) + getFeatureBasedTolerance ( e 2 ) ; v 1 , v 2 ← findNearestPair ( { v 11 , v 12 } , { v 21 , v 22 } ) ; if distanceBetween ( v 1 , v 2 ) > t then return false ; end f 11 , f 12 ← getSourceFaces ( e 1 ) ; f 21 , f 22 ← getSourceFaces ( e 2 ) ; if haveCommonFace ( { f 11 , f 12 } , { f 21 , f 22 } ) is false then return false ; end f 0 ← getCommonFace ( { f 11 , f 12 } , { f 21 , f 22 } ) ; f 1 , f 2 ← getDifferentFace ( { f 11 , f 12 } , { f 21 , f 22 } ) ; e _ array ← getSharedEdges ( f 1 , f 2 ) ; flag ← false ; pos 1 ← getPos ( v 1 ) ; pos 2 ← getPos ( v 2 ) ; for edge ∈ e _ array do if posOnEdge ( pos 1 , edge ) and posOnEdge ( pos 2 , edge ) then flag ← true ; e 0 ← edge ; break ; end end if flag is false then return false ; end return true ;
```

Therefore, when all three above conditions meet, it means this disconnection situation exists. v 1 and v 2 should be at the same position. Another crucial thing from the proof is that a proper position to merge these two misplaced vertices is the position of the intersection vertex generated by the intersection of the common edge e 12 and face f 0 . The algorithm that detects this situation is shown as Algorithm 1 .

(6)

Since the dimension of the edge-face intersection is lower than that of the face-face intersection, which means lower errors, the new position will be more precise than the old one. The repair process is shown as Algorithm 2 .

Algorithm 2: Fix the Disconnection of two Intersection Edges

```
input : e 1 , e 2 v _ array ← edgeFaceIntersection ( e 0 , f 0 ) ; min _ dist ← maximum ; for v ∈ v _ array do dist ← distanceBetween ( v, v 1 ) + distanceBetween ( v, v 2 ) ; if dist < min _ dist then pos ← getPos ( v ) ; min _ dist ← dist ; end end setPos ( v 1 , pos ) ; setPos ( v 2 , pos ) ; return ;
```

## 4.3.2 Pretty Short Edges which Should Not Exist

It is not rare that the intersection result of two models has some very short edges. Their length is pretty small compared with the size of the models, and we should deal with them very carefully. Sometimes a pretty short edge is the remnant of a normal-length curve cut by faces' borders. But in some situations, this short edge comes from the intersection result of two slightly contacting faces, like two faces sharing a common vertex, or two tangent faces. And when these situations occur, the intersection edge should collapse into an intersection vertex based on the edge-face topological intersection results. However, the errors may disturb the process of topological edge-face intersection and make a small part of the intersection edge left.

Figure 4: Sketch map of the pretty short edge.

<!-- image -->

When two faces only intersect at certain positions, their overlapping areas are so small that the influence of accumulating errors will be obvious. As shown in Fig.4, e 1 is a boundary edge of face f 1 , while e 2 is a boundary edge of face f 2 . e 0 is the valid part of intersection curve of f 1 and f 2 (The invalid parts which are outside f 1 or f 2 are shown by dashed lines). This intersection curve should pass to the v 0 , the intersection vertex of e 1 and e 2 . But due to accumulating errors, the intersection curve intersects e 1 at v 1 and intersects e 2 at v 2 , which causes a pretty short intersection edge e 0 . And the pretty short edges will bind a sliver face and falsely break up the boundary edge of faces, which will cause topological problems in the resulting model.

We have four criteria to judge whether a pretty short edge e 0 in an intersection graph g should be an intersection vertex and need to be fixed, and these criteria should be satisfied in full:

① e 0 is an intersection edge of face f 1 and f 2 while the length of e 0 is shorter than its feature based tolerance.

② e 0 in g does not connect with any other edges or only connects to other edges in one end.

③ e 1 is a boundary edge of f 1 . e 2 is a boundary edge of f 2 . e 0 intersects with e 1 at the position of v 1 . e 0 intersects with e 2 at the position of v 2 .

④ e 1 intersects with e 2 at the position of v 0 . Both the distances between v 0 and v 1 , v 0 , and v 2 are smaller than the sum of feature-based tolerance of e 1 and e 2 .

The proof of why the four criteria satisfy this situation is shown as follows:

Proof 4.2. from criterion ① , we know that:

<!-- formula-not-decoded -->

from criterion ③ , we know that:

<!-- formula-not-decoded -->

Therefore, based on Rule 3.1 , we know that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

from criterion ④ and (2), we know that

<!-- formula-not-decoded -->

Therefore, from (4), we know that

<!-- formula-not-decoded -->

Based on (5) and Lemma 3.1 , in a sufficiently small neighborhood of v 0 , we can get that

<!-- formula-not-decoded -->

from criterion ③ , we know that

<!-- formula-not-decoded -->

Then, from (7), (1), and (2), we know that:

<!-- formula-not-decoded -->

Therefore, from (8), we know that

<!-- formula-not-decoded -->

Based on (8) and Lemma 3.1 , in a sufficiently small neighborhood of v 2 , we can get that

<!-- formula-not-decoded -->

From criterion ④ , we know that v 0 and v 2 are very close. Therefore, based on (7) and (10), we can get that

<!-- formula-not-decoded -->

Similarly, we can get that

<!-- formula-not-decoded -->

From criterion ② , we know that removing e 0 from the intersection graph will not break the graph apart. From (10), (11) and the information that e 0 is very short, we know that the intersection edge should collapse into a vertex at the same position of v 0 .

Length is an important criterion, but we need more than just considering this factor to identify flawed edges. In our practice, we find some edges very short but valid. These edges have such a feature that they are at a crucial position in the intersection graph. If we remove them, the intersection graph will be split up, and the areas that intersection edges should bound will be open, which will cause an incomplete topological structure in the resulting model. Therefore, the second criterion can avoid this mistake. Besides, if this situation happens totally inside a face, it will not affect any existing topological structure. It will add an isolated loop inside the face, which will be easily detected and removed in the final resulting model, so we do not need to do extra work here to fix it. That is why we need the third criterion. And the fourth condition restricts all steps to a small area. We repair the short edge only when the intersection edge meets all four requirements. The judge algorithm is shown as Algorithm 3 .

Algorithm 3: Judge Whether the Short Edge Should Exist

```
input : intersection edge e 0 , intersection graph g output : Whether this edge needs to be repaired. If so, return v 0 , whose position is that the pretty short edges should collapse into. t ← getFeatureBasedTolerance ( e 0 ) ; if getLength ( e 0 ) > t then return false ; end if getNext ( g, e 0 ) is exist and getPrevious ( g, e 0 ) is exist then return false ; end v 1 , v 2 ← getStartAndEndVertex ( e 0 ) ; f 1 , f 2 ← getSourceFaces ( e 0 ) ; pos 1 ← getPos ( v 1 ) ; pos 2 ← getPos ( v 2 ) ; if posOnBoundary ( pos 1 , f 1 ) is true then e 1 ← GetposOnFaceBoundary ( pos 1 , f 1 ) ; end if posOnBoundary ( pos 1 , f 2 ) is true then e 2 ← GetposOnFaceBoundary ( pos 1 , f 2 ) ; end if posOnBoundary ( pos 2 , f 1 ) is true then e 1 ← GetposOnFaceBoundary ( pos 2 , f 1 ) ; end if posOnBoundary ( pos 2 , f 2 ) is true then e 2 ← GetposOnFaceBoundary ( pos 2 , f 2 ) ; end if e 1 is not exist or e 2 is not exist then return false ; end v _ array ← edgeEdgeIntersection ( e 1 , e 2 ) ; sum ← getFeatureBasedTolerance ( e 1 ) + getFeatureBasedTolerance ( e 2 ) ; for v ∈ v _ array do dist 1 ← distanceBetween ( v 1 , v ) ; dist 2 ← distanceBetween ( v 2 , v ) ; if dist 1 < sum and dist 2 < sum then v 0 ← v ; return true , v 0 ; end end return false ;
```

To fix this wrong edge, we should make these pretty short edges collapse into an isolated vertex, so it will no longer harm the existing topological structure. Moreover, we should find a proper position for this isolated vertex. In Algorithm 3 , we have found the position should be the position of v 0 . Therefore, we should change the position of v 1 and v 2 into the position of v 0 . Then we should merge v 1 and v 2 into one single vertex and remove the geometric element of e 0 since e 0 has already collapsed into a vertex. The process is shown as Algorithm 4 .

Algorithm 4: Fix the Pretty Short Edge by Collapsing It into Vertex

```
input : intersection edge e 0 , intersection vertex v 0 v 1 , v 2 ← getStartAndEndVertex ( e 0 ) ; pos 0 ← getPos ( v 0 ) ; setPos ( v 1 , pos 0 ) ; setPos ( v 2 , pos 0 ) ; mergeVertex ( v 1 , v 2 ) ; setCurve ( e 0 , null ) ;
```

## 4.3.3 Deviation of Intersection Vertices

In the process of face-face intersection, the initial intersection edge of these two faces will be constructed first based on the geometrical intersection curve. Then this edge will intersect with each boundary edge of the two faces to find valid parts inside both faces. However, as the process of edge-edge intersection runs edge by edge separately, some inconsistency between two neighbor edges may exist.

Figure 5: Sketch map of the deviation of intersection vertices

<!-- image -->

Shown in Fig.5, e 0 is the initial intersection edge of f 0 and f 1 (not showing in this figure), e 1 and e 2 are two boundary edges of f 0 , and they meet at a common vertex v 0 . In theory, e 0 should have intersected with e 1 and e 2 at v 0 . However, due to the calculation errors, when we intersect e 0 with e 1 and e 2 separately, we find that e 0 intersects with e 1 at v 1 , and intersects with e 2 at v 2 . This slight difference will cut e 0 and generate a short edge bounded by v 1 and v 2 , which may damage the topological integrity.

When dealing with e 0 intersects with e 1 , the geometric intersection algorithm gives the intersection position of v 01 on e 1 . Use the position of v 01 to find the corresponding parameter inversely on e 0 . However, when applying the parameter on e 0 , we get the position of v 1 . The position of v 1 and v 01 is slightly different due to errors. The topological algorithm uses the parameter to get the position of the intersection vertex. Therefore, v 1 is the topological intersection result. And similar things happen when deal- ing with e 0 intersects with e 2 , which generates v 2 and v 02 . And v 2 is the topological intersection result. This situation is like the disconnection of two intersection edges we have discussed in section 4.3.1 . However, unlike happening at a common edge of two neighbor faces, it exists on two neighbor edges sharing a common vertex.

To detect this situation, we have four criteria to judge whether an intersection edge e 0 needs to be repaired, and these criteria should be satisfied in full:

① e 0 generated by the intersection of f 0 and f 1 intersects with at least two boundary edges of f 0 .

- ② e 1 and e 2 are boundary edges of f 0 . e 0 on f 0 intersects with e 1 at v 1 , e 2 at v 2 , while e 1 and e 2 share common vertex v 0 .
- ③ The distances between v 1 and v 0 , v 2 and v 0 are both shorter than the feature based tolerance of e 0 .

<!-- formula-not-decoded -->

The proof of why the four criteria satisfy this situation is shown as follows:

Proof 4.3. from criterion ① , we know that:

<!-- formula-not-decoded -->

from criterion ② , we know that:

<!-- formula-not-decoded -->

Therefore, based on Rule 3.1 , we know that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

from criterion ② , we know that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Therefore, from (4), (2) and (1), we know that

<!-- formula-not-decoded -->

from criterion ② , we know that

<!-- formula-not-decoded -->

from criterion ④ and (7), we know that

<!-- formula-not-decoded -->

Therefore, we know that

<!-- formula-not-decoded -->

Based on (8) and Lemma 3.1 , in a sufficiently small neighborhood of v 0 , we can get that

<!-- formula-not-decoded -->

From criterion ③ , we know that v 0 and v 1 are very close, therefore, based on (6) and (9), we can get that

<!-- formula-not-decoded -->

Similarly, we can get that

<!-- formula-not-decoded -->

Therefore, v 1 and v 2 should be removed and we use v 0 to replace them.

Because we have found that v 0 is on f 1 , and v 0 originally belongs to f 0 , we can assert that v 0 must be on the intersection edge of f 0 and f 1 . Therefore, the process to fix e 0 is simple. We just need to remove v 1 and v 2 and then use v 0 to substitute them.

Unlike other scenes, in this situation, the flawed intersection edge should be fixed during the process of face-face intersection. Otherwise, some intersection information that can help fix the inconsistency will be lost. In the face-face intersection, e 0 , the initial intersection edge of f 0 and f 1 will be used to intersect with all boundary edges of f 0 and f 1 to find its valid parts, which are both inside f 0 and f 1 . If e 0 intersects with e 1 at the position of v 1 and intersects with e 2 at the position of v 2 , while e 1 and e 2 belong to the same face, for example, f 0 . Then it has fit the criterion ① , and the algorithm shown in Algorithm 5 will be called.

## Algorithm 5: Judge and Fix the Deviation of Intersection Vertices on an Intersection Edge

```
input : intersection edge e 0 , f 0 and f 1 which generate intersection edge e 0 , vertex v 1 and v 2 generated by the intersection between e 0 and the boundary edge of f 0 . pos 1 ← getPos ( v 1 ) ; pos 2 ← getPos ( v 2 ) ; t ← getFeatureBasedTolerance ( e 0 ) ; e 1 ← GetPosOnFaceBoundary ( pos 1 , f 0 ) ; e 2 ← GetPosOnFaceBoundary ( pos 2 , f 0 ) ; if getCommonVertex ( e 1 , e 2 ) is not exist then return false ; end for v ∈ getCommonVertex ( e 1 , e 2 ) do dist 1 ← distanceBetween ( v 1 , v ) ; dist 2 ← distanceBetween ( v 2 , v ) ; if dist 1 < t and dist 2 < t then v 0 ← v ; pos 0 ← getPos ( v 0 ) ; if posOnFace ( pos, f 1 ) is true then remove ( v array , v 1 ) ; remove ( v array , v 2 ) ; add ( v array , v 0 ) ; return true ; end end end return false ;
```

## 5 Experiment

In this part, we will show our repair method in different situations. Our method can detect and fix flawed structures while keeping the flawless parts of models from being modified.

Example 5.1. As shown in Fig.6, the torus's center position is the same as the center position of the regular hexagonal prism, and the torus's axis of rotation coincides with the center axis of the regular hexagonal prism. The minor diameter of the torus, which is the diameter of the circle being rotated, is the same as the height of the prism. The major radius of the torus, which is the distance from the center of the circle to its rotation axis, is the same as the radius of the circumscribed circle of the bottom plane of the prism. Therefore, all vertices of the prism are on the face of the torus. And there should be 12 intersection edges between these two bodies.

Figure 6: (a) Relative position of the regular hexagonal prism and the torus. (b) An abnormal short edge appears in the boolean union resulting model of the prism and the torus.

<!-- image -->

However, in practice, the number of intersection edges is more than 12 because of the intersection errors. And there are some pretty short edges appearing near the position of the prism's vertices. This situation is described in section 4.3.3 . Fortunately, these short edges will not fail the following boolean operations, but they are still unexpected and will make the resulting model's structures more complex, which may cause potential problems.

Using the algorithm described in section 4.3.3 , we can easily detect the short edges and prolong its neighbor intersection edge to meet the vertex of the prism. After repairing, all tiny intersection edges are removed. And so will the short edges left in the resulting model. The boolean result will be more concise than that before.

Although there are some pretty short intersection edges, it should be noted that this case is different from the situation described in section 4.3.2 . These short edges, which are generated by the inconsistency of intersections between intersection edges and boundary edges, are connected in the intersection graph at both ends.

Example 5.2. As shown in Fig.7(a), the prism and torus change their relative position. in this situation, the prism translates a certain distance horizontally outward from the torus. Fig.7(b) shows these two bodies' intersection graph.

However, this intersection graph is split into three parts for the reason discussed in section 4.3.1 .

Figure 7: (a) Relative position of the prism and torus. (b) Intersection graph of the prism and torus. Because of the disconnection, the graph has been split into three parts. Each part in different dashed boxes is dyed different colors.

<!-- image -->

As shown in Fig.8, intersection edges that should have shared one of their vertices break apart. Theoretically, these two vertices should be combined to make the intersection graph valid. And our inference procedure can find this flawed structure and fix it.

Figure 8: A pair of disconnected vertices in the intersection graph

<!-- image -->

Without repairing, the area that intersection edges should have bounded is open now. In the following steps, after imprinting intersection edges on these two bodies, there will be some faces falsely being segmented, shown in Fig.9. There should be three faces on the surface of the torus after imprinting, shown in Fig.9 (a). However, due to the disconnection, one area which should be bounded by intersection edges is open now. Therefore, the false result only has two faces after imprinting, shown in Fig.9 (b). Consequently, this will affect the following classification of faces, making some faces deleted by mistake and generating incorrect boolean operations, as shown in Fig.10 (a).

Figure 9: (a) Right imprint result. (b) False imprint result due to the disconnection.

<!-- image -->

By using the algorithm discussed in section 4.3.1 , we can reunite the disconnecting vertex. After repairing, the split intersection graph reunites, and boolean results are right now, shown in Fig.10 (b).

Figure 10: Boolean subtraction result of the prism and torus(prism minus torus)(a) False boolean subtraction result, we can see the faces in the hole of the prism are missing due to the false classification result and the intersection edges on both bottom faces of prism disappear. (b) Right boolean subtraction result.

<!-- image -->

Example 5.3. Considering the cylinder and cone shown in Fig.11, initially, the cylinder's bottom face coincides with the bottom face of the cone, and then the cylinder is rotated by a slight angle around a diameter of the bottom face.

Since the bottom face of the cone now touches the side face of the cylinder only at the two ends of the rotation diameter axis, the intersection entities between the side face of the cylinder and the bottom face of the cone should be two vertices. However, because of the calculation errors, the results are two very short edges. One of them is shown in Fig.12.

Figure 11: Relative position of the cylinder and cone. Most part of the cone is inside the cylinder.

<!-- image -->

These two pretty short edges will not cause noticeable problems in the following boolean steps. However, they will be kept in the final boolean result. In the boolean subtraction result, shown in Fig.13, both short edges are non-manifold edges, an edge associated with more than two faces. The reason is that these short edges are so close to where the faces join that they are falsely regarded as on these faces, which causes significant topological flaws in the resulting body.

Figure 12: One pretty short intersection edge between the bottom face of the cone and the side face of the cylinder.

<!-- image -->

Because these two intersection edges are short enough and isolated in the intersection graph while both ends are on the boundary of faces, they will be detected and removed by the algorithm in section 4.3.2 . Then the intersection vertices between the cone's bottom face and the cylinder's side face will be added to the intersection graph to replace these two pretty short edges. After repairing, the nonmanifold edges in boolean subtraction resulting models are removed, and the whole model becomes 2-manifold again.

Figure 13: Boolean subtraction result of the cylinder and the cone(cylinder minus cone). Both of the short edges become the boundary edges of all dyed faces. Since the short edges simultaneously lie on four faces (One of the faces is totally blocked by the red one.), the whole model becomes non-manifold.

<!-- image -->

Example 5.4. Our method can well preserve the small features in boolean resulting models. Shown in Fig.14 (a), a straight ball valve is modeled by uniting several simple shapes together. In the dotted box shown in Fig.14 (b), the position of the green column's bottom face is slightly lower than that of the red cylinder's top face, which makes the green column shallowly embedded in the red cylinder.

Figure 14: (a) The outlook of the straight ball valve. (b) The colorful part in the dotted box contains a long green column and a red cylinder.

<!-- image -->

Therefore, in the process of manufacturing this part, there will be a shallow groove on the top of the red cylinder, which is an important feature. However, other repair algorithms which extract features directly from the resulting model or its design history may consider the shallow groove as a result of modeling errors and will remove this small feature by resetting the position of the long column on the top. Without the shallow groove, the column will only slightly touch the top face of the cylinder and cannot be fixed. Since the constraint is weak, its degrees of freedom are still high, which will cause a lot of trouble in following operations.

Figure 15: The shallow groove on the top of the cylinder.

<!-- image -->

By taking the intersection graph into consideration, our method can well preserve such small features and prevent them from being modified. Shown in Fig.16 (b), two very short intersection edges in the intersection graph represent the feature of the shallow groove in the resulting model. Although these two edges are pretty short compared with the size of the whole model, removing them will break the intersection graph, making the area bounded by intersection edges open, which leads to flawed topological structures in resulting models. Therefore, these two edges will be preserved, and so will the shallow groove in the resulting model. With the help of the intersection graph, our method can get more valid topological and geometric information about the model, which directs the inference procedure to make a more reasonable decision and better preserve tiny features in resulting models.

Figure 16: (a) The outlook of the intersection graph of the column and the cylinder. (b) Parts of two dotted boxes after zooming in.

<!-- image -->

## 6 Conclusion

This paper is motivated by the illegal topological structures in the resulting models of boolean operations. We found that the illegal topological structures resulted from the inaccurate intersection edges caused by errors. Therefore, we propose a method to detect and repair the flawed intersection edges. Our method uses feature-based tolerance values to direct the inference algorithms to locate flawed intersection edges in a small area. Our inference procedures ensure the repair process will not do unnecessary modifications while correcting the inaccurate intersection edges. And we show the repair effects in different situations in the experiment part.

For future work, we hope to find a way to calculate a more accurate tolerance value for each intersection edge. And we want to extend our inference algorithms to label a dataset consisting of models with flawed structures, which will facilitate the use of neural networks to help detect defective models.

## References

- [1] S. Fang, B. Brüderlin, Robustness in geometric modeling-tolerance-based methods, in: Workshop on Computational Geometry, Springer, 1991, pp. 85101.
- [2] J. Nocedal, S. J. Wright, Numerical optimization, Springer, 1999.
- [3] J. B. Lasserre, Global optimization with polynomials and the problem of moments, SIAM Journal on optimization 11 (3) (2001) 796-817.
- [4] W. Shao, F. Chen, X. Liu, Robust algebraic curve intersections with tolerance control, Computer-Aided Design 147 (2022) 103236.
- [5] C. M. Hoffmann, J. E. Hopcroft, M. Karasick, Robust set operations on polyhedral solids, IEEE Computer Graphics and Applications 9 (6) (1989) 50-59.
- [6] G. Bernstein, D. Fussell, Fast, exact, linear booleans, Computer Graphics Forum 28 (5) (2009) 1269-1278.
- [7] J. Richard Shewchuk, Adaptive precision floatingpoint arithmetic and fast robust geometric predicates, Discrete &amp; Computational Geometry 18 (1997) 305363.
- [8] J. Nehring-Wirxel, P. Trettner, L. Kobbelt, Fast exact booleans for iterated csg using octree-embedded bsps, Computer-Aided Design 135 (2021) 103015.
- [9] D. J. Jackson, Boundary representation modelling with local tolerances, in: Proceedings of the third ACM symposium on Solid modeling and applications, 1995, pp. 247-254.
- [10] [S. C. Dassault Systèmes, Tolerant Modeling, https://doc.spatial.com/get\_doc\_page/ articles/t/o/l/Tolerant\_Modeling\_670c. html (2023).](https://doc.spatial.com/get_doc_page/articles/t/o/l/Tolerant_Modeling_670c.html)
- [11] H. Chen, S. Jin, Z. Li, X. Lai, A comprehensive study of three dimensional tolerance analysis methods, Computer-Aided Design 53 (2014) 1-13.
- [12] J. Davidson, A. Mujezinovic, J. Shah, A new mathematical model for geometric tolerances as applied to round faces, J. Mech. Des. 124 (4) (2002) 609-622.
- [13] A. Mujezinovic, J. Davidson, J. Shah, A new mathematical model for geometric tolerances as applied to polygonal faces, J. Mech. Des. 126 (3) (2004) 504518.
- [14] V. Delos, D. Teissandier, A. Malyshev, G. Nuel, S. C. García, Polyhedral-based modeling and algorithms for tolerancing analysis, Computer-Aided Design 141 (2021) 103071.

- [15] A. Desrochers, A. Rivière, A matrix approach to the representation of tolerance zones and clearances, The International Journal of Advanced Manufacturing Technology 13 (1997) 630-636.
- [16] J. M. Hervé, Analyse structurelle des mécanismes par groupe des déplacements, Mechanism and Machine Theory 13 (4) (1978) 437-450.
- [17] D. E. Whitney, O. L. Gilbert, M. Jastrzebski, Representation of geometric variations using matrix transforms for statistical tolerance analysis in assemblies, Research in engineering design 6 (1994) 191-210.
- [18] C. M. Hoffman, R. Joan-Arinyo, Cad and the product master model, Computer-Aided Design 30 (11) (1998) 905-918.
- [19] G. Barequet, Using geometric hashing to repair cad objects, IEEE Computational Science and Engineer-

ing 4 (4) (1997) 22-28.

- [20] J. Yang, S. Han, Repairing cad model errors based on the design history, Computer-Aided Design 38 (6) (2006) 627-640.
- [21] P. K. Jayaraman, A. Sanghi, J. G. Lambourne, K. D. Willis, T. Davies, H. Shayani, N. Morris, Uv-net: Learning from boundary representations, in: Proceedings of the IEEE/CVF conference on computer vision and pattern recognition, 2021, pp. 11703-11712.
- [22] J. G. Lambourne, K. D. Willis, P. K. Jayaraman, A. Sanghi, P. Meltzer, H. Shayani, Brepnet: A topological message passing system for solid models, in: Proceedings of the IEEE/CVF conference on computer vision and pattern recognition, 2021, pp. 1277312782.