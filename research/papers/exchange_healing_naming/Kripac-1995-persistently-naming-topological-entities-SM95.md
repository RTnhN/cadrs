## A MECHANISM FOR PERSISTENTLY NAMING TOPOLOGICAL ENTITIES IN HISTORY-BASED PARAMETRIC SOLID MODELS

## (TopologicalID System)

Jiri Kripac

Autodesk Inc. 111 A.4clnnis Parkway, San Rafael, CA 94903

Email: jkripac@autodesk com

+ Feature dependencies within the model. For example, the information that a hole lies in a particular face, that a fillet or a chamfer is attached to a particular edge or a vertex, etc.
+ Dimensions and annotations attached to faces, edges and vertices of the model (model-drawing associativity).
- Variational constraints associated with the model.

The fundamental problem is how to identifi the topological entities of the solid model in such a way that these entities can still be identified even atler the model has been reevaluated from the history of operations [Hof93].

Unfortunately, directly using pointers to topological entities is not good enough because pointers are transient. Whenever the parametric modeler reevaluates the model, it creates a brand new model and the original pointers become invalid.

Any simple scheme, such as sequentially enumerating all entities in the model, is also not enough because when the model is edited and automatically reevaluated, the topology of the model may change and the enumeration becomes invalid. Even if the topology of the model doesn't change and only the geometry is modified, the modeling algorithms such as Boolean operations may create or modi~ the topological entities of the model in a different order and consequently the enumeration cannot be preserved.

The Topological ID Sys(em, presented in this paper, tries to solve this problem. h systematically assigns IDs to topological entities (faces, edges, vertices) in a solid model. When the solid model is being reevaluated from its history of operations, the Topological ID System maps IDs of topological entities from the old version of the solid model to IDs of corresponding topological entities in the new version of the solid mocfell. This mapping defines the correspondence between topological entities in the old Ad new model.

1Instead of saying 'old version of the solid model' or 'new version of the solid model', we will simply say 'old model' or 'new model'.

## ABSTRACT

One of fundamental problems in history-based parametric solid modeling systems is to identi@ topological entities of solid models in such a way that the same entities can still be identified after the models have been reevaluated from the sequential his(ory of modeling operations.

The Topological ID System systematically assigns /Ds (names) to topological entities (faces, edges, vertices) in solid models, When the solid model is edited and then automatically reevaluated from the history of modeling operations, the IDs of topological entities in the old version of the model are mapped to IDs of the corresponding topological entities in the new version of the model. This mapping defines the correspondence between topological entities in both models.

## 1. INTRODUCTION

Feature-based parametric modeling systems usually define models by a sequential histo~ of operations. Whenever the user performs a modeling operation, for instance chamfers an edge, makes a slot, etc., the parametric system not only performs the operation on the current model but also records all the defining parameters of the operation into a history tile. This sequential history of operations unambiguously defines the model and the parametric system can automatically recreate the model anytime by reevaluating this stored histo~ of operations.

Feature-based modeling systems associate additional information with solid models and their topological entities such as faces, edges and vertices. These associations must be persistent, i.e. when the model is automatically reevaluated from its history of operations, the topological associations must be preserved and not lost. Typical examples of associations are:

Permission to copy without fee all or part of this material is granted provided that the copies are not made or distributed for direct commercial advantage, the ACM copyright notice and the title of the publication and its date appear, and notice is given that copyin is by permission of the Association of Computing Machinery. Yo copy otherwise, or to republish, requires a fee and/or specific permission. Solid Modelin '95, Salt Lake City, Utah USA J' CI 1995 AC 0-89791 -672-7/95/0005 ...$3.50

If the topology of the model change% this mapping may not be one-to-one, e.g. one edge in the old model may correspond to more than one edge in the new model or to no edge at all. The mapping algorithm should be able to handle these cases but should avoid finding false correspondences between unrelated topological entities in both models.

The parametric modeling system communicates with the Topological ID System through a narrow interface:

- 1) When the modeling system wants to make a topological entity persistent, i.e. identifiable after the model reevaluation, it provides a pointer to the entity to the Topological ID System and obtains a Persld (Section 6).
- 2) When the model is being reevaluated, the modeling system gives this PersId back to the Topological ID System and obtains pointers to topological entities in the new model. This second step is repeated for every reevaluation of the model and every time pointers to topological entities in the new model are obtained.

The Topological ID System consists of five major components:

- IDs to name faces, edges and vertices of solid models and steps in the history of modeling operations (Section 2). ?
- An additional data structure (FaceMGraph) associated with each model that keeps the information about how faces of the model were created, split merged and deleted (Section 3). This additional information is needed when mapping IDs from the old model to the new model. +
- A mapping algorithm for mapping IDs from the old version of the model to the new version of the model (Section 4).
- An algorithm for reassigning FaceI&amp; to faces in the new version of the model consistently with the assignment in the previous version of the model (Section 5) and for updating the FaceIdGraph when a modeling operation changes the topology of the model. For detailed information see ~i94]. +
- A data structure associated with each solid model keeping a record for each modeling step. Each record contains Persids of the topological entities which were recorded at this modeling step. When this modeling step is being reevaluated, the Topological ID System automatically remaps these PersIds to correspond to topological entities in the new model.

## 2. TYPES OF IDS

## 2.1. Step/d

Each step in the history of operations is assigned a S/epId. This StepId uniquely identifies the particular step (modeling operation) and remains unchanged when the model is reevaluated.

## 2.2. Face/d

FaceMr are the principal types of IDs. Our view is that regularized solid models are 3D volumes bounded by a set of faces. Edges and vettices are considered to be intersections of bounding fiwes and Edgel&amp; and Vertexlh are therefore defined in terms of their adjacent FaceIa!r.

Each face in the boundary of a solid model is assigned a unique FaceId. A FaceId of face f is defined by three components:

<!-- formula-not-decoded -->

where.

stepId

faceIndex

surfaceType

Stepld of the step during which face f was created.

Face index of the newly created face during the step stepId.

Type of the underlying surface of the face.

## 2.3. Edge/d

Edges are considered to be intersections of two or more faces; EdgeId of edge e is therefore defined in terms of FaceIak of faces surrounding the edge (Figure 1):

<!-- formula-not-decoded -->

where:

adjFaceIds

endFaceIdso

endFaceIcbl

edgeIntersCade

Ordered cyclic list of FaceIds of faces sharing edge e.

Unordered set of FaceIak of faces at one end of edge e.

Unordered set of FaceIds of faces at the other end of edge e.

Edge intersection code.

<!-- image -->

Figure 1: E21geld(e)= [

&amp;jFaceIds: endFaceMsO'

&lt;f!l

Jj&gt;3.

U3tJ4f~4.

endFaceI&amp;l:

U&amp;fTf/$t

edgeIntersCaak:

Mry&amp;igeIntersCade]

<!-- image -->

## Figure 2:

- a) The orientable edge e I is shared by two faces, adjFace]d,s(e~) = &lt;f@-fI&gt;. ~he end O and end 1 of the edge are defined with respect to facejo
- b) The non-orientable edge e2 is shared by a single face J which is used in both directions.

An edge which has at least one adjacent face included only once in the adjFacelds set is said to be orientab/e because we can use this face to distinguish between the end Oand the end I of the edge (Figure 2a). Edges which have all their adjacent faces included more than once in the a~Facelds set are non-orientable, i.e. we cannot distinguish between end O and end I of the edge looking only at the aajFacelds set of that edge (Figure 2b).

If an edge is orientable, endFaceLirO corresponds to the end 0 of the edge and endFacekLrl corresponds to the end 1.

The adjFaceIds amd endFaceHrO, * sets of FaceIds provide local topological information about an edge. However, in some cases, this local topological information is not enough to uniquely identi~ the edge,

The edgeIntersCode field of the EdgeId provides the additional information to disambiguate edges that are otherwise topologically the same, i.e. they have exactly the same a@FaceM and endFaceMsO ). A common example are the two edges coming from the subtraction of two cylinders (Figure 3). These edges have their adjacent faces exactly the same and there is no local topological information that could be used to distinguish between these two edges.

The EdgeIntersCode is based on the relative position of the surfaces of the adjacent faces of the edge, not on the absolute coordinates. This makes the EdgelrrfersCode invariant under rigid motion and scaling transformations.

When the surfaces intersect at more than one intersection curve, each of these intersection curves is assigned a different EdgeIntersCode. The Edgeld then receives the EdgeJntersCade of the intersection curve the edge lies on. In the above example, the EdgeIds of the two edges are distinguished by their different EdgeIntersCode values.

4{e/em, elem, ...) indicates a unordered set of elements.

<!-- image -->

endo

Figure 3: The EdgeIrr/ersCade of edges e] and e2 is based on the direction of the axis of the subtracted cylinder and distinguishes these two edges.

<!-- image -->

## 2.4. Vetiex/d

Vertices are considered to be intersections of faces; Verfexld of vertex v is therefore defined by a set of FaceIds of faces intersecting at this vertex:

Vertex[d(v) = [ at+'FaceIak, ver(exlntersCodes ]

where:

adjFaceIds

Set of FaceIds of faces sharing vertex v.

ver(exln/ersCodes

Vertex intersection code for each face in adjFaceIds set.

in the case of manifold vertices, all faces sharing the vertex form a simple ordered loop around the vertex and a@FaceMs is an ordered loop of FaceIds of faces sharing this vetlex. In the case of non-manifold vertices, adjFaceIds is art unordered set of FaceIds of all faces sharing this vertex.

The set of adjacent faces provides local topological information about a vertex. However, in some special cases, such as in the case of the two vertices on a degenerate 'lemon' or 'apple' torus, this information is not enough to uniquely define the vertex.

In order to disarnbiguate VertexMs of these special vertices, an additional VertexInfersCode attribute is provided for each face in the adjFacekJs set. This information specifies where the vertex lies with respect to the surface of the face.

## 3. EVOLUTION OF FACES

The Topological ID System associates with each solid model a data structure called a FaceIdGraph which is updated every time the topology of the model changes.

The Face/dGraph is a directed acyclic graph. Its nodes, called FaceIdNodes, represent faces in the model, both faces that currently exist in the model and faces that once existed in the model but don't exist in the current model any more because they were spli~ merged or deleted as a result of a modeling operation applied to the model.

The incoming edges to a FaceIdNode represent information about the ancestors of this face and the outgoing edges represent what happened to the face (Figure 4):

+ If there is more than one incoming edge to a FaceIa!Wode, it means that the face was the result of merging two or more faces. The faces being merged ceased to exist in the model and were replaced with the new face.
+ If there is more than one outgoing edge from a Face/dNode, it means that the face has been split into several new faces. The original face ceased to exist and was replaced with several new faces.
+ If there is no incoming edge to a FaceIdNode, it means that the face has been created directly, for instance, when faces of a primitive solid are created. This is the most common case.
+ If there is one outgoing edge from a FaceldNode to a special node named akletedFaceldNode, it means that the whole face has been deleted.
+ If there is no outgoing edge from a FaceIdNode, it means that the face still exists in the current model. Such faces are called living faces. Any other FaceIdNodes in the graph represent faces that once existed in the model but were spli~ merged or deleted.

Figure 4: In this FaceJdGraph f=sf] !5,fI,2.fl,3,f2. I ~dJ2.2 have been created, thenf12 has been split into/3./ and/3.2, /]3 and ~2 1 have been merged into J3,3 and f2 I has been deleted. Finally, j3 z and/3.3 have been merged into }4.1. OnlY fl. p ~3.I andJ4 I exist in the cturent model.

<!-- image -->

'fmn deno tes a face created during StepId m and with index n w.r.t. to the step m.

## 4. MAPPING TOPOLOGICAL IDS

The mapping algorithm maps the given old ID (Faceld, EdgeId, VertexId) of a topological entity in the old version of the solid model to a set of new IDs of the corresponding topological entities in the new version of the model.

## The main requirements are:

+ Intuitive behavior, meaning that the algorithm must find the correspondences between old and new topological entities which users would expect.
+ Handling curved surface topology, e.g. edges and vertices which are intersections of non-planar surfwes, handling special cases such as cone apices, etc.
- 4 Handling topology changes between the old and the new model bit avoiding fmd~ng false correspondences between unrelated entities in both models.

The mapping is implemented by a finction mapIdO:

mapId: [ oldId oldFaceIdGraph, newFaceIdGraph, newModel,

IdMapRequest] + [newIdSet, IdMapStatus]

## where:

| oldId                                     | ID of a topological entity in the old model which is going to be mapped to the new model.                            | ID of a topological entity in the old model which is going to be mapped to the new model.                                                                                                                                       |
|-------------------------------------------|----------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| oidFaceIdGraph                            | FaceIdGraph of the old model.                                                                                        | FaceIdGraph of the old model.                                                                                                                                                                                                   |
| newFaceIdGraph                            | FaceIdGraph of the new model.                                                                                        | FaceIdGraph of the new model.                                                                                                                                                                                                   |
| newModel                                  | Solid B-Rep data structure of the new model.                                                                         | Solid B-Rep data structure of the new model.                                                                                                                                                                                    |
| IdMapRequest                              | Required quality of mapping (Section 4.4).                                                                           | Required quality of mapping (Section 4.4).                                                                                                                                                                                      |
| newIdSet                                  | The resulting set of IDs of topological entities in the new model corresponding to the old topological entity oldld. | The resulting set of IDs of topological entities in the new model corresponding to the old topological entity oldld.                                                                                                            |
| IdMapStatus                               | Achieved 'quality' of mapping.                                                                                       | Achieved 'quality' of mapping.                                                                                                                                                                                                  |
| The achieved 'quality' of mapping may be: | The achieved 'quality' of mapping may be:                                                                            | The achieved 'quality' of mapping may be:                                                                                                                                                                                       |
| &actIdMapStatus                           | &actIdMapStatus                                                                                                      | The oldId has been mapped to exactly the same new ID. It means that the topology of the entity didn't change between the old and the new version of the model.                                                                  |
| kUniqueIdh4apStatus                       | kUniqueIdh4apStatus                                                                                                  | The oidId has been mapped to exactly one new ID. It means that the entity changed between the old and new model but the mapping algorithm was able to track the change and find a single corresponding entity in the new model. |

kSomeIdh4apStatus

kFailedIdMapStatus

The oldld has been mapped to more than one new ID. It means that the old entity has been split into several pieces in the new model.

The oldld didn't map to any new ID. It means that no reasonably similar topological entity has been found in the new model.

## 4.7. Face/d Mapping

&amp;Mthm:

Map a given Face/d from the old model to the new model.

@ff:

oldFaceId

oldFaceIdGraph, newFaceIdGraph

IdMapRequest

Q,@d:

newFaceIdSet

IdMapStatus

S@2X

## + Exstct-Equality-Test:

Try to find a face with the oldFaceld in the new model. If it exists, return the oldFaceld. Otherwise, use the next two steps.

## ● Backward-Search in the oldFaceIdGraph:

Starting from the node with the given oldFaceId, go backward in the oldFaceIdGraph, until you reach a FaceId that is also present in the newFaceldGraph6. Select all such Facela%, i.e. process all branches in parallel.

## + Forward-Search in the newFaceldGraph:

For those FaceIak found during the Backward-Search, go forward in the rsewFaceldGraph until you reach nodes that have no successors, i.e, they are living nodes corresponding to faces that currently exist in the new model. Select all such nodes.

Return Facela!s of these living nodes. ❑

Figures 5 and 6 show examples of behavior of the FaceId mapping algorithm.

6The purpose of the Backward-Search is to find FaceIak in the oldFaceIdGraph which resulted in the given o!dFaceId and which are also present in the newFaceldGraph. These common FaceIds mean that there was a common step in the history of the old model and the history of the new model during which these common FaceMr were created.

Figure 5: Two boxes have &amp;en united. In the old model USC top faces ~f I and ~2 f have been merged into fsw ~3,1. In the new model these faces' stay as two different faces.

<!-- image -->

L

When facc~3 I in USC old model is mapped to the new model, USC Bsckwsrd-Se&amp;ch finds two common faces j] 1 and J2, /. The Forward-Search returns the same faces because both f],, and J2,, are living faces in the new model

Figure 6: When faceff 1 in the old model is mapped to the new model, the Bsclmvsrd-Sesrch selects the same face J\_l,1. The Forward-Search selects stl Iivine successors of this common face f],], i.e. faws.f2, I,Y3,I sndJ3,2-

<!-- image -->

## 4.2. Ecigeki Mapping

4&amp;dtitm:

Map a given &amp;igeId from the old model to the new model.

LtuzrK

oldEdgeId

oldFaceIdGraph, newFaceIdGraph B-Rep of the new model IdMapRequest

Quf12ut

newlUgeIdSet IdMapStatus

SttW

## + Exact-Equality-Testi

Try to find an edge with the oidEdgeId in the new model. If it exists, return the oldEdgeld.

Otherwise we a sequence of tests to select the matching edges. Firs4 a set of candidateEdges is created and then additional rejection tests try to reject 'less matching' candidate edges and leave just a single best one.

## + AdjFaceIds-Test:

Select all edges in the new model which have at least two adjacent FaceIa!r matching7 adjacent FaceIa!r of the oldEdgeId. Store all the edges found in the candidateEdges set.

The goal is to obtain exactly one matching edge, if possible. If one or zero candidate edges have been found, return the EdgeId of this candidate edge or return an empty set, respectively. If more than one candidate edge has been found, continue with the rejection tests.

## + EndFaceIds-Testi

Select the edges from the candidate~ges set which have at least one FaceId at each end of the candidate edge matching a FaceId at eaeh end of the oldEdgeId. A special case is when one or both endFaceMr sets of one edge are empty, in which case the corresponding endFaceMs set of the other edge must also be empty.

If both edges are oriented, the algorithm takes into account the orientation of both EdgeIds. It distinguishes which end of each WgeId is end Oand which is end I and makes sure that the correct ends of both EdgeIris are compared.

The EndFaceIds-Test may eliminate some candidate edges from the candidateEdges set. If the reduced candidateEdges set contains exactly one edge, return EdgeId of this single edge. If no edge would be leQ use the original candidateEdges set before the EndFaceIds-Test was performed. Continue with the next test.

## + EdgeIntersCode-Test:

Select the edges from the carsdidateEdges set whose EdgeIntersCode is qual to the EdgeIntersCode of the oldEZgeId.

7Saying that an old FaceId 'matches' anew FaceId means that the old FaceId maps to a set of FaceIa!r which contains the new FaceId.

The EdgeIntersCode-Test may eliminate some edges from the candidateEdges set. If no edge would be left in the candidateEdges seL use the original candidateuges set before the EdgeIntersCode-Test was performed.

Return EdgeIds of the edges left in the candidateEdges set.O

Figures 7, 8 and 9 show examples of the behavior of the EdgeId mapping algorithm.

<!-- image -->

@@

Figure 7: Edge el of the old model has been selected. In the new model, the top box has been moved. The AdjFacAls-Test finds new edge ez because fl, 1 maps tof3. 1 mdf1,2 nmps@f3,2.

<!-- image -->

<!-- image -->

Figure 8: Edge e of the old model has been selected. In the new model, the slot has been extended so that the edge was split. The AdjFacelds-Test selects two candidate edges el, e2. Neither the EndFaceIds-Test nor the EdgelntersCode-Test selects just a single edge, therefore both e] and ez are returned.

Figure 9: Edge el of the old model has been selected. In the new model, the cylinder has been sectioned. The AdjFsceIds-Test selects el and e2 as candidate edges. Because there is snore than one candidate edge, EndFaceIds-Test is tried. Neither candidate edge satisfies the EndFacelds-TesLtherefore the EdgeIntersCodeTest is used.Ilk test acceptsedgee~ snd rejectsedgee2.

<!-- image -->

z

## 4.3. Veffex/d Mapping

AlgwWuu:

Map a given Vertexfd from the old model to the new model.

LugKf:

old Ver&amp;xId

oldFaceIdGraph, newFaceldGraph B-Rep of the new model IdMapRequest new VertexIdSet

Q@td:

IdMapStatus

## S&amp;W

## O Exact-Equality-Test:

Try to find a vertex with the o!dVertexld in the new model. If it exists, return the oldVertexId. Otherwise use the BestSimilarity-Rating-Test.

## + Best-Similarity-Rating-Test:

Traverse all vertices in the new model and select the vertex or vertices whose VertexIds have the highest similarity rating with the oldVertexld. Return VertexIds of these selected vertices.

The similarity rating is based on the similarity between adjFaceI&amp; and vertexIntersCodes of the oldVertexId and vertices in the new model. The similarity rating algorithm is analogous to the algorithm used for determining the similarity between two FaceIdLoops (see Section 5). Cl

Figure 10 shows an example of the behavior of the VertexId mapping algorithm.

Figure 10: VertexVI in the old model has been selected. in the new model, the object has been sectioned. Using the BestSimilssrity-Rathrg-Tes~the mapping algorithm selects vertex V3in the new model, whose similarity rating is k.SubsetA4atch.

<!-- image -->

## 4.4. Required Quality of Mapping

The IdMapRequest input argument specifies the required 'quality' of mapping. The possible requests are:

kExactIdMapRequest

kUniqueIdMapRequest

The given old ID is required to map to exactly the same ID in the new model.

The given old ID is required to map to exactly one new ID.

kSomeIdMapRequest

The given old ID is required to map to at least one new ID.

kAnyIdMapRequest

Any result of mapping is accepted, including the case where the iven old ID couldn't map to any new ID. !

The mapping algorithm maps the given old ID to new IDs. It then checks whether the achieved quality of the mapping (IdMapStatus) satisfies the required quality of the map ing ~ (IdA4apRequest). If yes, the resulting new IDs are returned . If no~ the system tries to meet the requested quality by performing the mapping to a previous state of the new model.

The new model is rolled frack to its previous state and the mapping is performed again. If the quality of the mapping to the rolled-back model satisfies the IdMapRequest, copies of the topological entities found are made and returned along with the mapped IDs. Otherwise, the rollback continues until the IdMapRequest is satisfied or until no more roilbaek is possible. Finally, the new model is rolled forward to its original position before any rollback started.

## EWmJ21C

The model (Figure 8, old model) was created in three steps:

Step I:

Create a base feature.

Step 2:

Create a slot.

Step 3:

Attach a dimension to edge e, record Pers/d'O of edge e into the history file.

When the length of the slot is changed, the model is reevaluated. When Step 3 is being reevahtated, the parametric system needs to find an edge or edges in the new version of the model corresponding to edge e in the old version of the model. Depending on the value of IdMapRequest, the result will be:

- If MnyldMapRequest is specified, the two edges e] and e2 are returned.
+ If kUniqueldMapRequest is specified, the edge e before it was split is returned.

In our example, the parametric system would most likely speci~ kUniqueIdMapRequest to receive exactly one edge e and use its geometry to place the dimension. The whole edge e before it was split would be dimensioned. as shown in Figure 8, new model.

In other cases, e.g., if the modeling system wanted to fillet the edge, kAnyIdMapRequest would be speeified and two edges el and e2 obtained. Both edges would then ~ filleted.

## 5. MAPPING SPLIT FACES

The split face mapping algorithm maps FaceId of a face which resulted from splitting a face in the old version of the model to a corresponding split face in the new version of the model. The correspondence between the old and the new split faces is determined based on the similarity between their surrounding faces. A FaceIdLoop class is introduced for this purpose (Figure 11).

8This is the default value.

'kAnyIdMapRequest is always satisfied and the result of the mapping is instantly returned.

10Section 6.

The FaceIdLoop of face J is a cyclic list:

FaceIdLoop@ = &lt; elem(el), elem(e$, ... Y

elem(e) = [faceIdSet, edgeIntersCode ]1 1

where:

facela%t

Set of Facekr!r of adjacent faces sharing the edge e with face~

edgeIntersCode

EdgelntersCode of edge e. This information is necessary to disarnbiguate some topologically similar cases where all the split fkmeshave the same loops of adjacent FaceIa!r (Figure 12).

When a face is bounded by exactly one loop of edges, this loop is used for the FaceIdLoop. When the face is bounded by more than one loop of edges, the goal is to use the most dominant loop for the FaceIdLoop. The most dominant loop of planar faces is their outer loop. If the face is non-planar and is bounded by more than one 100P of edges all faces of all looPs are taken and the FaceIdLoop is ;onside;ed to be an unordered '&amp;t.

Figure 11: FaceIdLoop(f?)= &lt;fl.fl,f&gt;f@ FaceIdLoop@) = &lt;f&gt; f3,f@f6&gt;

<!-- image -->

v

Figure 12: FaceIdLoops of both f3 and f4 consist of the same faces ~1, f&gt;. [n order to disarnbiguate these two FaceIdLoops, liiigeIntersCcde information is used. The EdgeIntersCode of the edge between fhces fy fl is different tlom the lkfgelnterscade of the edge between facmf4f2 because the direction of the subtracted cylinder is taken into account.

<!-- image -->

11In the examples we will simpli~ the notation and write elem(e) as just the adjacent face/t

Let us now define a measure of similarity between two F'acefdl.oops. The similarity rating must return the highest value if the two FaceIdLoops are exactly the same and should return a lower value if the loops are not the same but are similar, which happens when the topology of the model changes.

~:

Measure similarity between two FaceIdLoops.

lt.UW:

oldFaceIdLoop newFaceIdLoop

oldFaceIdGraph, newFaceIdGraph

-:

kExactMatch

kSubsetMatch

k3ElemMatch

k2ConsecElemA4atch

k2ElemMatch

k! ElemMatch

kDisjointMatch

The algorithm returns one of the following similarity values, listed in the descending order from the highest similarity rating to the lowest:

@actMatch:

The loops match exactly. This means that each elem of one loop matches an elem of the other loop.

The cyclic ordering of elerns in both loops is significant, i.e. the position of the matching e/ems in both loops as well as the direction of both loops is required to be the same. 12

kSubsetMatch:

One loop is a subset of another, meaning that all e/errrs of one loop match eiems in the other loop. The cyclic ordering of eferns in both loops is significant.

k3EfemMatch:

At least three elems in one loop match elems in the other loop. The cyclic ordering of elems in both loops is significant.

k2ConsecElemMatch:

Two elems in one loop match two elems in the other loop. These two matching elems are positioned consecutively in both loops, so there is no gap between them. The order of the two matching elems in both loops is significant.

k2ElemMatch:

Similar to the above case, but there is a gap between the two matching elems in either one or both loops. The order of the two matching elems in both loops is not significant13.

klElemMatch;

Just one elem in one loop matches an elem in the other loop.

121fboth FaceIdLoops are oriented, the cyclic ordering of both loops is considered, otherwise the loops are treated as unordered sets.

13Even if not immediately eviden~ the k2ElemMa/ch is substantially weaker than k2ConsecElemMatch because the information about the orientation of both loops is lost.

kDisjointMatch:

The two loops are completely disjoint. l

Example (Figure 13):

FaceldLoop@)

```
= -=J1. f- f~ fj~ = % fss f> f4~ = <fl. J-&f> fp f~ fs'
```

FaceIdLoopf17)

FaceIdLoopfl~

FaceIdLoopfl+ = ~fl,fy fp flo fbfl 1'

The similarity rating table between the old and the new split faces is:

|               | FaceIdLoop(jJ      | FaceIdLoop@        |
|---------------|--------------------|--------------------|
| FaceIdLoop@   | kSubsetMatchRating | k2ElemMatchRating  |
| FaceIdLoopfl+ | k2E!emMatchRating  | kSubsetMatchRating |

This means that FaceIdLoop@ is most similar to FaceIdLoop(fJ and FaceIdLoopf17) is most similar to FaceIdLoopfl . Therefore face f6 maps to face ~ and face f7 i maps to face ~

Figure 13: In tbc old model, the top face of the box has been split by a slot into two faces f6 and J7 In the new model, the vertical edges of the box have been filleted before the slot operation is reevstuated.

<!-- image -->

## 6. API

In order to record a topological entity (face, edge, vertex) as persisten~ the application records the pointer to this topological entity and obtains a PersId'4. The PersId does not change when the model is reevaluated. The application then uses this PersId to refer to the recorded topological entity in the future.

To record a topological entity entPtr and obtain its PersId

Pers Id pers Id = TopId: : record (entPtr, reqls) ;

14The Pers/d is basically an index of a record which keeps ID (Faceld, EdgeId, VertexId) of the recorded entity and other data. When the PersId is remapped, the stored ID may be remapped to none or to more than one new ID.

15The IdMapRequest req says how the entity is to be remapped in the future (Section 4.4).

When the model is being reevahsated, the PersId is automatically remapped so that it refers to topological entities in the new version of the model. To obtain all entities in the new model to which a persistent topological entity with the given Persld has been remapped, the following method is used:

```
newEnts = Top Id: :get (pers Id) ;
```

The returned newEnts set is a set of pointers to all topological entities to which the given PersId refers. Because of remapping, onc PersId may refer to any number of topological entities in the new model.

The Topological ID System associates a data structure with each solid model keeping one record for each modeling step. Each record keeps the Stepfd of the step and PersIds of all topological entities which were recorded at this modeling step. For this purpose any modeling step must be enclosed between:

```
TopId: : stepBegin (step Id) ; Perform the modeling operation; TopId: :stepEnd (oldMode116, newModel, . . . ) ;
```

This way the newly created faces are assigned FaceIds, the FaceIdGraph of the new model is updated and PersIds saved with this modeling step are remapped.

Here are typical steps the parametric history-based modeling system may use when it performs an operation for the first time and when it automatically reevaluates this operation:

## Recording an Operation:

1. Obtain a pointer to a topological entity, e.g., ask the user to pick an edge to be filleted.
2. Record this topological entity and obtain its PersId.
3. Save the Persld.
4. Use the topological entity, e.g. fillet the edge.

## Reevaluating the Operation from Stored Parameters:

1. Take the saved PersId.
2. From the Persld obtain pointers to topological entities in the new model.
3. 3, Check whether exactly one new entity has been obtained. If no~ it mears that the remapping didn't map the old entity to exactly one new entity. React appropriately to this situation.
4. Use the topological entity, e.g. fillet the edge.

## 7. CONCLUSION

The presented paper tries to come up with a more robust approach to the problem of persistently identifying topologicid entities in history-based parametric solid models. We see the superiority of the presented solution in the following areas:

+ Ability to disambiguate topologically similar entities.
+ Ability to handle curved surface geometry and edges and vertices that are intersections of curved surfaces.

l%he oldModel (actually only its Face/dGraph) is needed for the ID mapping algorithms (Section 4).

+ Ability to handle split fkces and merged faces, particularly to distinguish between the individual faces resulting from splitting smother face,
+ Object-oriented software architecture with a narrow application interface.

Finally some possible directions for falter work:

+ The presented mechanism has been designed for manifold and non-manifold solid models. A natural extension would be to make the system handle general non-manifold structures.
+ An interesting research topic might be to formalize the notions of 'similarity' and 'mapping' between topological models.

## 8. ACKNOWLEDGEMENT

I would like to thank to Ravi Krkhnaswamy for reading the manuscript and making many helpful suggestions.

## 9. REFERENCES

- pP93] Buchanan, S A and pcnnington A, 'Constraint Dctinition System: A Computer-Algebra Based Approach to Solving Geometric Constraint Problems' Cornprr:er-Aided Design Vol 25 No 12 (December 1993) pp 741-750
- [DZ93] Duarr,W, Zhou, J and Lai, K, 'FSMT7A Feature SolidModeting Tool for Feature-Based Des@ and Manufacture' CompuferAidrdDesigrr Vol 25 NO 1 (January 1993) pp 29-38
- [Gib77] Giblin, P J, Graphs, Su~aces and Hornologv Chapmtrn&amp; HallfHatsted Press, New York, NY, USA (1977)
- [GZS88] Gossard, D C, Zuffrmte, R P and Sakurai, H, ' Representing Dimensions, Tolerarms, and Features in MCAE Systems' f&amp;EE Computer Graphics &amp; Applications Vol 8 No 2 (i9g8) pp 51-59
- @40f93] Hoffinartn, C M 'Gttthe Semantics of Generative Geometry Representations' Proceedings of the 19th ASMEDesign Automation Corrr New Yo~ NY, USA (1993)pp411419
- [HS79] Hanttic~ R M and Shapiro, L G, The Consistent Labeling problem Part 1' IEEETrans. Pat~emMatching c% Mach. Irrtell.Vol PAM-1 NO 2 (1979) pp 173-184
- [ti194] Kripac, J, Topological ID System' Ph.D. 7?tesis Czech Technical University in Prague, School of Electrical Engineering, Dept. of Computer Science, Csech Republic (1994)
- ~at90] Matmgvis~ J '~ Design System for Parametric Design of Complex Products' Proc. 1990 ASMEAdmnces in Design Automation cO~ -DE-VOI32-1 (1990) pp 17-24
- [Mey88] Meyer, B, Object-Oriented Sojhwre Construction Prentice Hall hrtemational, UK (1988)
- [PFP89] Pinill% J M, Finger, S and Pri~ F B, 'Shape Feature Description turdRecognition Using art Augmented Topology Graph Grammar' Prac. NSFEngineering Design Research Conr College of Engineering University of Massachusetts at Amherst USA (11-14 June 1989) pp 285-300
- [Ro191] Roller, D, 'AnApproach to Computer-Aided Parametric Design' Comprfer-Aided Design Vol 23 No 5 (June 1991) pp 385-391
- [Ros90] Rossignac, J K 'Issuesof Feature-Based Editing and Interrogation of Solid Models' Cornprders &amp; Graphics Vol 14No 2 (1990) pp 149-172
- ~V89] Roller, D, Shonek, F and VertousL A, 'IXmension-Driven Geometry in CAD A Survey' ?7reoryand Practice of Geometric Modeling Springer-Verlag (1989) pp 509-523
- [SL93] Sheu L C and Lin J T, 'Representation Scheme for Defining and Operating Form Features' Compurer-Aided Design Vol 25 No 6 (June 1993) pp 333-347
- ~091 ] Wang, N and Ozsoy, M, 'AScheme to Represent Features, Dimensions and Tolerances in Geometric Modeling'J ManuJ Syst Vot 10No 3 (1991) pp 233-240