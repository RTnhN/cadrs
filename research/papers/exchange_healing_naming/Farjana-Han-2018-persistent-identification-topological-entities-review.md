<!-- image -->

## REVIEW

## Mechanisms of Persistent Identification of Topological Entities in CAD Systems: A Review

Shahjadi Hisan Farjana a*, Soonhung Han

aSchool ofEngineering，MacquarieUniversity，Sydney，Australia

bDepartment of Mechanical Engineering,Korea Advanced Institute of Science and Technology,Daejeon,Republic of Korea

Received 5 June 2017; revised 9 January 2018; accepted 30 January 2018 Availableonline16November2018

## KEYWORDS

Feature based CAD; Persistent identification; Topology; Geometry; Topological entity

## Contents

| 1.   | Introduction...                        |   2838 |
|------|----------------------------------------|--------|
| 2.   | Parametric CAD systems ...             |   2838 |
| 3.   | Persistent identification mechanism... |   2839 |
| 3.1. | Basic naming.....                      |   2840 |
| 3.2. | Ambiguity solving....                  |   2841 |
| 3.3. | Name matching ...                      |   2842 |

* Corresponding author.

E-mail addresses:shahjadisynthy@gmail.com, shahjadi-hisan.farjana@students.mq.edu.au (S.H.Farjana).

Peer review under responsibility of Faculty of Engineering,AlexandriaUniversity.

1110-01682018Faculty of Engineering,AlexandriaUniversity.Production and hosting by Elsevier B.V.

AbstractParametric CAD modelling systems requires the successful CAD model creation and modificationintheproduct designphase.It demands persistent identification of topologicalentities,retrieving faces,edges,andvertexes are done with those entities specific unique names called persistent IDs.Those names should contain their unique properties which would be unambiguous among topological entities.In this paper, we are going to present reviews of existing persistent identification mechanisms and discuss their merits and demerits based ontopologybased approach, geometrybased approach,or a combination of both ones.This review paper tries to define persistentidentificationmechanismnotsimplybytopology andgeometrybutbytheiruniqueproperties likefeature orientationbasedpersistentidentification,faceIDGraphbased persistentidentification etc.Previouspapers classified thepersistentidentificationmechanismby onlythetopological or geometricalpropertiesofentitieswhich aretoosimpletounderstandtheinsidemeritsanddemerits of that method. This paper also identifies the advantages and limitations of the current persistent identification approaches while suggesting a new approach should be developed consisting of both topology andgeometrybased approach.This paper alsosuggeststhefuture direction of persistent identificationofparametricCADmodelsandtheirentities.

2018Faculty of Engineering,AlexandriaUniversity.Production and hostingbyElsevier B.V.Thisis an open access article under the CC BY-NC-ND license (http://creativecommons.org/licenses/by-nc-nd/4.0/).

## Alexandria University

## Alexandria Engineering Journal

www.elsevier.com/locate/aej www.sciencedirect.com

<!-- image -->

<!-- image -->

| 3.4.                                                              | Persistent identification problems of commercial CAD systems....   |   2842 |
|-------------------------------------------------------------------|--------------------------------------------------------------------|--------|
| 4. Classifications of persistent identification mechanisms......: | 4. Classifications of persistent identification mechanisms......:  |   2842 |
| 4.1.                                                              | Feature orientation based persistent identification ...            |   2842 |
| 4.2.                                                              | faceIDgraph based persistent identification . .. ... ..            |   2843 |
| 4.3.                                                              | ShellGraph based persistent identification . ...                   |   2844 |
| 4.4.                                                              | Feature ID based persistent identification ....                    |   2844 |
| 4.5.                                                              | SemanticIDbased persistent identification..                        |   2846 |
| 4.6.                                                              | B-Rep based persistent identification .......                      |   2846 |
| 4.7.                                                              | Point based persistent identification ...                          |   2846 |
| 5.                                                                | Comparison of the existing research                                |   2847 |
| 6.                                                                | Conclusions......                                                  |   2848 |
| Acknowledgments....:                                              | Acknowledgments....:                                               |   2848 |
| Conflict of interest .....                                        | Conflict of interest .....                                         |   2848 |
| References ..:                                                    | References ..:                                                     |   2848 |

## 1. Introduction

Existing CAD design systems are parametric and historybased feature modeling systems which uses a parametric model definition of the CAD object with a boundary representation (b-rep) model of the final shape. The parametric definition comprises of a variety of information, such as parameterized feature instances, constraints, geometric entities, etc. The order of creation of feature instances is defined as the design model history [1]. In the feature based CAD modeling systems, successful retrieval of the topological entities in necessary once a designer wants to translate his design model from one CAD system to another CAD system. CAD model translation can be of two types-direct translation and translation based on XML format neutral macro fle. The limitation of direct translationis,thedesignintent(productcreationandmodification history) could belost which should not be altered in successful CAD data translation. Macro-parametric approach enables the CAD designer to able to exchange the parametric information of CAD models comprising of product creation history and modification history. In this approach, a set of standard modeling commands is defined in the format of anXMLformat neutral macro file. The history based macro-parametric approach of KAIST (iCAD lab) solves these issues by using XML format neutral macro file (Figs. 3,4). Once a CAD design model is modified/new feature is created, old topological entities lost their identity/previous feature modification information [18,20].

Persistent identificationproblemshouldbesolvedbyidentification of all the referenced topological entities every time after design modification. This mechanism attaches names to topological entities after creation based on the operations of creation andretrievethosetopological entities every time after the modification occurs. Names are necessary to represent the entities referenced in the parametric specification whether they exist in the model or not.Where every single constructive gesture creates several entities,which have tobe distinguished from each other and therefore named.The names shall be defined using such a way to generate a new geometric model, the geometric entity, referenced by a name in the re- evaluated model is the same as the same as the one referenced by the same nameintheinitialmodel.Twodifferentkindsof geometric and topological entities may be distinguished [1-6,17,18,20]. There are several distinguished researches from last 20 years about persistent identification mechanism,which all have both their advantages and disadvantages. But until now, there are no standards for persistent identification mechanism and consistently using them in all parametric features based CAD systems. The focus of this review paper is to theoretically classify the persistent identification mechanism and analyzetheexistingresearchesfor their advantages andlimitations to conclude on a new approach to solve the current problems to make a standard [18,20].

## 2.Parametric CAD systems

In the feature based design, part models are created. To redesign a part, features are edited; their descriptions are automaticallymodified andrecordedby the designinterface andare compiled serially by a design compiler (CAD feature editing ally define models by a sequential history of modeling steps. Each step contains operations and parameters.A parametric modeler is a system for geometric design which preserves not only the designed object but also the set of constructive gestures used to design it.

It follows a two-fold data structure. They record the geometric shape of the currently designed product,often as a Boundary representation model as representation of current instance. They record the conceptual design of which the current instances results, its parametric specification, which consists of constraints represented as expressions that reference the current model entity which encompasses various kinds of operations.

Fig. 1 Persistent Naming type classifications.

<!-- image -->

<!-- image -->

Fig.2 2Types of topological entities.

<!-- image -->

- ·Lost of designintents(feature,constraints,parametricinformation)

Fig. 3  CAD Modeling approached using STEP.

Fig. 4 4Macro-parametrics approach of iCAD lab KAIST.

<!-- image -->

This two-fold data structure raises three different types of issues: Define a data model that provides for the simultaneous exchange of expressions, of variables involved in these expressions&amp;ofvariables involved in these expressions and ofvalues of these variables. Topological naming problem which are to assign persistent names to geometric and topological entities that may not exist or that may cut into different parts in the geometry of the current instance. And this topological naming problem can happen at the design process which is entity naming problem. Or after design reevaluation which is known as name matching problem.

In feature based design, editing operations can be classified into the following broad categories:

E1: Inserting or deleting a feature. E2: Changing feature attributes. E3: Modifying dimension values.

E5: Changing the feature shape definition. E1 to E4 requires name matching.

E4: Changing the dimension schemes.

## 3.Persistentidentificationmechanism

Inthe parametric CAD modeling systems,the structure of topological entity naming should be unique, must be applicable for same type of topological entities (applicable to generally for all entities),independent of allparametric CAD modeling systems and which will generate unambiguous names (Fig. 2). Topological entities created with different feature modeling operations should be retrieved and named using the same method.If the persistent identification mechanism is dependent on CAD systems,the mechanism may not be applicable to each type of CAD systems (topology based CAD system/geometry based CAD system)which will drive the integration between heterogeneous CAD systems with successful CADdata exchangeimpossible.And the namingrule shouldbeunambiguous toretrieve thenames of topological entities modified through topology splitting and topology merging.

And persistent identification should be done with the minimum datarequiredfor CADmodel translation;that arefeature type, attributes of a feature, parameters of a feature, local coordinates of a feature, etc. Naming can be classified as basic naming and ambiguity solving part (Fig. 1). Basic naming involves attachingnames withtopologicalentities associated with that feature (face, edge, vertices) based on their unique properties. But in different cases, topological entities could have same basic name; which is defined as ambiguity problem, becausefeaturemodificationinformationcannotbereflected in the original basic name of the entities.Ambiguity could arise from topology merging or topology splitting case, or from patternfeature operations while modifying the CAD model. It can also happen during the creation stage of CAD model. For solving these issues, name matching should be conducted between homogeneous CAD systems (name matching) or heterogeneous CAD systems (name mapping) (Fig. 1). Name matchinginvolvestwodifferentways.Localmatchingbetween the modified topological entities (1: N comparison) or global matching between topological entities (N: N comparison). Local matching compares a single entity from evaluated model with all the referenced entity is the pre-edit model; whereas entities in pre-edit model of with the subset of topological entiglobal matching tries tomatch among the subset of topological ties in post-edit model [12-14,21].

Fig. 5Two important ways of basic naming.

<!-- image -->

<!-- image -->

Fig.6Literature survey of basic naming of persistent identification.

Fig. 7Example and solution of ambiguity problem-topology merging.

<!-- image -->

Fig. 8Example and solution of ambiguity problem-topology splitting.

<!-- image -->

Fig. 9 Example and solution of ambiguity problem-pattern feature operation.

<!-- image -->

Fig. 10Literature survey of ambiguity problems solving of persistent identification.

<!-- image -->

## 3.1.Basic naming

Persistentidentificationmechanismcanbebroadlyclassified based or geometry based which is dependent on the CAD systems. In can also be a combined method comprised of topology and geometry. CATIA follows topology based basic naming, where SolidWorks follows geometry based basic naming [22, 23] (Fig. 5). Previous researches shows, the basic naming order of topologybasedmethodcan 1beeitherof faces are named first where face names are used to assign names to edges and vertices.Entities can alsobe classified as invariant or contingent entities.Significant researches are mentioned through Fig. 6.

Fig. 11 Name matching mechanism example byMun's method [13-15].

<!-- image -->

## Invariant entities:

Geometric or topological entities which can be completely andunambiguously characterizedby the structure of a constructive gesture and its input parameters,independently of involved values.

## Contingent entities:

Geometric or topological entities that results from an interaction between the pre-existing geometric model and invariant entitiesresultingfrom aparticular constructivegesture.

## 3.2.Ambiguity solving

Ambiguity occursin CADpartmodelswhen afterfeature creation operations (product design phase) or feature editing operation (design review phase), two or more topological entities have the same basic name (topology splitting, Fig. 8) or twoormore entities whosebasicnameswere differentwere added into one (topology merging, Fig. 7). Then a different not only sufficient to solve this problem.Ambiguous entities can be distinguished using geometry based method, because theirgeometric dimensions shouldbe something differentfrom the initial one (Fig. 9).

<!-- image -->

Fig. 12 Name mapping mechanism example by Mun's method [13-15].

<!-- image -->

Fig. 13 Literature Survey of Name matching and name mapping of persistent identification.

Fig. 14 Persistent identification problems of old SolidWorks [8,13,22].

<!-- image -->

Fig. 15 Persistent identification problem of freeCAD [28].

<!-- image -->

Fig. 16 Persistent identification problem of Siemens JT of ISO (topology splitting) [20,29].

<!-- image -->

Fig. 17Persistent identification problem of Siemens JT of ISO (topology merging) [20,29].

<!-- image -->

In topology based naming of iCAD lab KAIST, ambiguity problems are solved with the Object space information system (OSI)based ambiguity solving method;which enables the geometry comparison of entities and then new entity name is returned which contains original basic name with ambiguity solving information [20,13,18] (see Fig. 10).

## 3.3.Name matching

Name matching is required between topological entities of initial CAD model and corresponding topological entities of reevaluated CAD model.And after name matching if there in any modified topological entities then they are given different names from the initial model to identify them persistently. Also,name matching can be of global matching or local matching. Local matching compares identified entity with the other entities in the initial model; whereas global matching compares the subset of topological entities of new model with the subset of topological entities in old model.

Its N: N matching technique which requires much computation time and skill with greater accuracy. But In case of local matching, it's 1: N leaves lower accuracy of matching result and fastest computational ability. In previous studies, different approaches were followed.In the persistent identification mechanism of iCAD lab KAIST (topology based method), local matching between entities are applied. If one entity is modified all the corresponding entities are added to the candidate matching set (Fig. 11).

Name mapping is the matching between entities when a model from one CAD system is converted into CAD model of another CAD system and the new model requires some design review; then the corresponding topological entities need to be matched (Fig. 12).

## 3.4.Persistentidentificationproblems ofcommercial CAD systems

Figs. 13-17 are used to explain the existing problems of persistent identification in commercial CAD systmes. Fig.14 describes the problems of SolidWorks,which occurs during modeling of a block. Similarly, Fig. 15 describes the persistent identification problem of FreeCAD. Figs. 16 and 17 is used to describe the persistent identification problem of Siemens JT (topology splitting and topology merging).

## 4.Classifications of persistent identification mechanisms

Persistent identification canbe classified based on 6 different mechanisms. Previous researches try to classify persistent identification mechanism based on entity type/entity information used for classification.We have classified persistent identification system based on the mechanism it followed.

## 4.1.Feature orientation based persistent identification

For achieving product modeler independence from the recorded design history, Capoyleas mechanism substitute all design reference such as the edge to be rounded by generic persistent names.During design compilation,these generic names are first persistently mirrored to geometric references in the Brep a solid modeler. When the design is changed a new variant is constructed based on the generic descriptions and on the history of the previous variant. This describes matching generic names that may not be unambiguous to b-rep entities in the new design variant.

This mechanism relies with the assumption that the order in which the features are defined is not changed, but these restrictions canbe relaxedwith suitable adjustments tothe algorithms.The elements referenced for constituting the particular feature are always evaluated with respect to the geometry that exists at the time when the feature is added to the design.

Feature orientation: The topological entity information derived from the feature modelling procedure is called as featureorientation.Featureorientationinformationcanbe broadly entitled based on the extrusion direction or the spin of revolution.

<!-- image -->

el v2

Fig. 18 Name attaching system by Capoyleas method [1].

Localorientation:FeatureOrientationinformationwhichis derived locally from the boundary elements geometry information is calledlocalorientationwhich isbased on the direction of adjacentface which involves an edge and on orientation of faces in the solid boundary.

<!-- formula-not-decoded -->

Or in the case of vertices,

Sv = sgn ((n1xn2) n3)

Lv1 = [fl,f3, f2] Lv2 = [fl,f2, f3]

Lv3 =[f1, f2, f4] Lv4 =[f1, f4, f2]

## Distinguishing edges withfeature orientation:

Edge = [F,f, c]

f =face

Where F = feature

c = + /- according to whether the edge is used by f in the direction of feature orientation.

## Distinguishingverticeswithfeatureorientation:

Vertices = [F, f, I]

f = incident face that is not adjacent to the intersection edge of the feature F.

Where F=feature

I = records the sign of inner product.

## Naming intersection edges:

Ee = [Ce, Le, fe] Where, Ce = all the adjacent faces of the topological entity. Le = local orientation. Fe = feature orientation.

## Namingintersectionvertices:

Ev = [Cv, Lv, fv] Where, Cv = all the adjacent faces of the topological entity. Lv = local orientation. Fv = feature orientation.

Fig. 19 Name attaching system by Kripac's method [5,6].

<!-- image -->

And persistent naming starts with naming of the intersection edges, secondly with the intersection edges and nonintersecting edges(Fig. 18 [1]).

## Matching a vertex:

In Chen's research, name matching is performed based on feature orientation method followed by Capoyleas research.

In Chen's research of topological entity matching, vertices are being matched to define the constraints, to build the datum features, to be utilized by changing features. In the feature dimensioning case, vertex in the three dimensional space need to be identified. Moreover according to their method, while modifying the features they can accept multiple number of vertexes with the same name.

## Matching an edge:

Name of the edges are given by defining constraints to construct the datum features, to be utilized by chamfers and roundsfeatures.For constraints anddatumfeatures ambiguous edges are acceptable if they are collinear line segments which contains constraint orientation.

## Matching a face:

Face matching is performed to determine the limits of feature attachment operations,definingdraft operations,identifying a sketch plane or constraining a datum definition. The degree of entity matching with which a face should be matched varies with the feature modelling operation. This mechanism doesn't consider ambiguity in naming matching phase. So as a whole,they tried to name all the entities which are unnecessary. They also didn't consider the ambiguity in the naming matching phase. Implementation is also very difficult [2-4].

## 4.2.faceIDgraphbasedpersistentidentification

StepID = history of modeling steps. FaceID = [StepID, faceIndex, SurfaceType]

EdgeID = [adjFaceIDs, endFaceIDs, edgeIntersCode]

VertexID=[adjFaceIDs,vertexIntersCode]

faceIDGraph is updated every time when topology of the model changes.Incoming edges to faceIDNode reflects incoming edges which will change and outcome represents the outgoing edges, what happened to that edges(Fig. 19).

If there is more than one incoming edge in FaceIDNode then face is merged. If there is more than one outgoing edge in FaceIDNode then face is split. And if no outgoing edge fromfaceIDNode thenfaces arelivingin the same state.It proposes an intersecting structure for identifying contingent entities based on face history(creations, merges and deletion of faces) &amp; a complex name matching among them.

1. Face graph structure allowing both naming of all entities (edges and vertices are named in terms of their adjacent faces) &amp; naming matching.

Fig. 20 Name attaching system by Agbodan's method [7].

<!-- image -->

2. Table recording name of the contingent entities that are referenced together with pointers on the geometry more, some information for the matching algorithm.
3. Geometry of the designed object.

This method has some limitations like:

1. They do not explain how the initial faces are named.
3. More memory space required.
2. Limitation in naming approach.

Kripac [5,6] proposed a topological system of API (application programming interface) that introduces a faceIDGraph describing basic naming and name matching mechanism while the table is changed andupdated every time once the model is modified.Topological entity matching is based on the compare and contrast between one or more faces in two graphs. But his way of distinguishing between edges/faces with the exactly same basic name is not clear which can leads to unpredictable results.Kripac also didn't address how to calculate intersCode and basic naming of face was not considered [5,6].

## 4.3.ShellGraphbasedpersistentidentification

Naming is based on shell graph,invariant and

contingent entities. They propose to identify invariant and contingent entitiesunambiguously.

Name matching- Nodes of the shell graph are matched from the initial design of shell graph onto the current reevaluation shell graph. Other topological entities are matched by reference to the shell graph.

Shell graph is classified into different sections:

1. Hierarchical Shells: shells and subshells to be able to apprehend the geometry at various levels of granularity defined for each class if feature.
3. Overlapping Shells: A subshell can be a part of several shells. The overlapping hierarchical structure enables, during construction, the creation of any shell aggregates.
2. Connected Shells: each shell is comprised of jointed subshells. Represents the history (split, merge or deletion) of the invariant initial structure.
4. 4.Aggregation Shells: For the designer to be able to reference geometry at various levels of granularity.

Agbodan [7] presented a persistent identification mechanism based on Shell graph [7] which is similar to Kripac's faceIDGraph except shell and sub shell information. This is difficult to implement (Fig. 20 [7]).

They proposed a complete method for identifying and matching topological entities based on their underlying topology which is based on the invariant structure of each class of form features and on its topological evolution. The matching method compares the initial and thereevaluated topological histories which consist of two phases. During the local phase, any couple of entities in the reevaluated model and in the initial model.And in the global phase the final matching maximizes the topological similarity between the matched entities of both models [7,26].

They have proposed a method for persistent naming which was associated with a hierarchical architecture involving tracing the historical evolution of invariant entities in each constructive gesture. The suggested method defines a shell graph, which have a hierarchical structure of shells of an historical structure of faces (Fig. 22).

The shells characterization enables on the one hand to represent the different types of shells required in CAD during initial modelling operation, also to identify the important shells matching to the referenced shells during model modification. A shell could be connected or overlapping. The matching method is a global method of topological matching in the sense that it involves two sets of faces in order to find the best possible matching for all faces. Then, it addresses the tracability loss problems.

In another paper, Marchiex have identified five common concepts which result from most of the studies. First, the distinctionbetween invariant and contingent entities.Second,the naming mechanisms used for invariant entities resulting from constructive gestures.Third,the invariant entity names used to name contingent entities.Fourth,the capability to trace the face history either uses a dedicated structure.Fifth,the global software architecture. They also proposed criterion refers to the use of pure topology or of both geometry and topology in the naming phase. The second criterion refers to the locality or globality of the matching method during the reevaluation phase [19].

In another paper,Marchiexhave proposed some additional information about the surfaces to improve the global matching between topological entities. This method considered the shape of non-planar faces and it also considered each surface as a collection of hollow(s) and/or hump(s).Practically,a decomposition operation according to the product shape criterion was implemented [26].

## 4.4.FeatureIDbasedpersistentidentification

Wu suggest that the name of face is decided by a type of feature creation and location of face in the feature.So Wu defines naming method as types of feature, profile based feature, entity based feature,and feature based feature.

For example extrude, revolve and sweep, profile based feature is a feature made from a profile. Basic form of profile based feature is

ONside = [FeatID, FeatIDp, IDelement, FeatIDPath, IDtrajectory

FeatID means ID of feature; feature type like "Sweep", "Revolve" can be of this value. FeatIDp is defined as the ID of Profile of the feature,and FeatIDPath is theID of Path.

Fig. 21 Name attaching system by Marchiex's method [12].

<!-- image -->

Fig. 22 2Name attaching order by Wu's method [8].

<!-- image -->

Fig. 23 Features covered by name attaching and ambiguity solving methods of Mun's research [13-15].

<!-- image -->

IDelement and IDtrajectory is the ID of the corresponding element and trajectory.Startingface is -1,end face is-2 asIDelement, and other IDs become 0.

So, the definition of naming format of profile based feature is

ON (F) = [FeatID, 0, -1, 0, O], if F is the starting face = [FeatID, FeatIDp, IDelement, FeatIDp -1, 0, O], if F is a side face

- = [FeatID, 0, -2, 0, O], if F is the ending face

Forexamplechamfer,round and shell,entitybasedfeature is the feature created using reference entity in the part for special operations. Basic form of entity based feature is

ON (F)= [FeatID, FeatIDFF1,IDelement, FeatIDFF2, IDelement2]

FeatIDFF1 and FeatIDFF2 means selected adjacent faces F1, F2. If adjacent face is only one, values of FeatIDFF1 and FeatIDFF2 are same. IDelement and IDelement2 are values of corresponding part in F1 and F2.

For example Linear Pattern, Circular Pattern and Mirror, Feature based feature is the feature created using existing feature.Basic form of entity based feature is

ON (F)= [FeatID,FeatIDp, IDelement, FeatIDPath, IDtrajectory, InsNum] (Fig. 21).

PSI is a solution for ambiguity using geometry information with a topological basic name explained in a previous chapter. If a face is split into two faces which have same topological

InsNum is the number of instance belonged to face F, and other IDs are the same value formerly explained.

Fig. 24 Ambiguity solving system by Mun's method [13,14].

<!-- image -->

name, using geometry information from each face. Basic format of PSI is

<!-- formula-not-decoded -->

ON (F) is a basic name of face suggested by Wu, "Seq"" is the number of corresponding face,and Totlemeans the total number of faces which have same topological name [8] (see Fig. 23).

## Naming of a face

EN (f) = BN (f): OSI: SN (2)

- = basic name: OSI: secondary name While there is no ambiguous split entities, OSI is [O, O]. While there is no ambiguous merged face, SN is [O, 0].

## Naming of an edge

```
EN (e)= EN (f1) # EN (f2) # OSI (3) = adjacent face name f1 # adjacent face name f2 # OSI.
```

## Naming of a vertex

EN (v) = EN (f1) {# EN (fi)} # OSI, where i ≠ 1

- =name of the adjacent facef1{# name of the adjacent face fi} # OSI.

BN (f) refers to the basic name of the face of a feature. EN (f), EN (e), and EN (v) are consecutively the names of a face, an edge, with a vertex. The number of adjacent face names inside the expression "{ }" can be zero, one, or more.

Mun suggest a method named Object Space Information (OSI)based onPSI and SecondaryName(SN)whichincludes merging and split information for ambiguity solving. OSI is very similar with PSI, but big difference between two methods is that OSI uses 3D geometry space to select reference points. Basic format of OSI is

OSI = [Order, Total\_Num]

Total\_Num means the total number of faces which have same topological name, Order means order of corresponding face decided by x, y, z value. Process of numbering using OSI is same with PSI. Fig. 4 shows two split faces f1, f2. Reference points Pf1 for fl and Pf2 for f2 are selected, as you can see, coordinate values of Pf1 is smaller, so f1 is the priority.

Secondary Name is information of merging and split of faces.A basic format SN is

```
SN = [Total_Num, BN (f), {FN (f)}]
```

Total\_Num is the total number of related faces; BN (f) means basic names of related faces. Names of two faces, BN (f1\_blank) and BN (f3\_tool),are merged into BN (f1\_blank):0, 0:1; BN (f3\_tool). And the final name of face becomes BN (f1\_blank):0, 0:2; BN (f3\_tool); BN (f2\_blank) after face BN (f2\_blank) is merged (Fig. 24).

## 4.5.SemanticIDbased persistentidentification

Hepworth's paper represents the enhancements over previous researches which more efficiently will identifythe faces and edges by lazy naming as well as caching and normalizing topological entity data. The automated feature reservation method prevents multiple users from simultaneously editing of the same feature, thus avoiding conflicts. A method is presented which ensures the consistency of the copies of the model between CAD clients by the same order on all the clients.In cases of conficting operations preserved locally an efficient model consistency method is presented which provides consisare applied consistently on all models. An integrated task management system is presented to avoid conflicts related to varying user design intent. The limitations of this method are the mathematicalmodellooks at semantic conflict reduction through automated feature reservationfor models rather than feature dependencies. The integrated task management system is quite simple that describes tasks tobe done and a user it should be assigned to which is limited to a single commercial CAD system [24].

In Wangs' paper, B-Rep variance is based on geometry continuity in parametric complex Euclidean (PpC3) space is proposed where Shape and relation parameters are differentiated in PpC3.A semantic id scheme based is developed such that entities are named based on geometry to solve the problem of topology mismatch in parametric modeling to improve the interoperabilityofCADfeaturemodeling.Inthissurfacebased scheme, ids are identified by themselves by the feature modeling operations. The inclusion of geometric information and boundary association in topological ids make the geometric and topological semantics. The geometric ids have construct relations of surfaces, curves, and points. Geometry information is included in ids when multiple curves are formed by intersecting the set of surfaces, or multiple edges which are created by same curve and boundary surfaces [25].

## 4.6.B-Repbased persistent identification

Shapiro proposed the definition for Boundary Representation (BR-)as deformationfor solid models inthe same parametric family: small changes in parameter values would result in small changes which may include local collapses in the boundary representation. This formulation explains the difficulties in devising sufficient tests for BR-deformation in practice.This method is also applicable to more general cellular models of pointsets and useful in developing universal standards in parametric modeling. These parametric interfaces allow the user to design and modify solid models by means of high-level parametric definitions that are constructed with intuitive meaning to the user. The new solid modeling systems no longer can guarantee that the parametric models are valid or unambiguous, where the results of modeling operations are not always predictable [27].

A unique method has been presented by Bidarra to solve persistent naming problem in parametric feature modeling systems. Which uses a dual representation which is comprised of a parametric definition and a b-rep of the resulting shape of the model.

When the identification problem occurs because of the parametric definition of the object, entities which are non- persistent arereferenced toretrieve and attach andpositionthe extra features in the model.Previous researches to solve the persistent naming problem tried to maintain the record of all non-persistent but were not successful, and it was questionable about the feasibility.A completely unique approach was therefore proposed, which distinguishes and separately deals with the problem: (i) entity naming in parametric definitions, and (i) entity identification in the b- rep. regarding the first one, all references are made to entities in the parametric definition, instead of in the b-rep. Such entities are persistent and so, obviously, their names too. Regarding the last one, featurerelated information is deployed, both in the parametric definition and in the b-rep, to identify any of the entity which is being referred to. Bidarra and Bronsvoort proposed a unique approach, in which feature references are made to the persistent topological entities in the parametric definitions.However, they did not address complex entity naming cases, and their approach requires a user's intervention sometimes which is very complicated. In addition, this mechanism demands a change in users' conventional product data modeling procedures and in the commercial 3D CAD systems'modeling functions [9,10,11].

## 4.7.Point based persistent identification

When feature is defined, the first entity to be defined is points in most cases. Then lines and faces are defined using these points. But because many entities in current feature-based CAD systems are based on faces in their definition, there are some problems in matching entity names between CAD systems. In this research, the naming method based on point is suggested to solve these matching problems. Because point is the basic entity to define feature, the point-based naming method can be the most suitable solution.

It can be expressed in a simple form like fl = [polygon, 4, v1l, v2, v3, v4]

## Definition of line:

Normally a definition of line is ""the shortest distance between two points". But other type of line like arc can be crerequired. To define all kinds of line, "Type" of line for identification and optional point are needed. Based on these, basic form of line can be expressed by

Line = ["Type", Pointl, Point2, Op\_point]

"Type" is NULL in case of norm al line, and "Circle", "Type" is "circle", Pointll is a center of circle and radius is s e st  ,  o,   g  ! point, and Point2 is an end point, and Op\_point is a center of arc.

In this research, type of face is divided into "polygon al" and "circular". The reason why type of face is divided like this is that the persistent identification problem must be resolved using only topology in polyhedral models whereas it's necessary to use geometry information in curved models.

## Definition of face:

## Definition of polygonal face:

Like rectangle and hexagon, polygonal face consists of multiple points. Based on this, a format of definition of polygonal face can be expressed by

Face = ["Type", n, Pointl, Point2... Point N]

In this case only "Polygonal" can be value of "Type". "n" is the number of points.After that, the name of each point is input. In this case, face "F1" is a rectangle created from the extrusion of Edge "E1" and the name of"F1" must be identified to create a feature in "F1". But for the application of point-base naming method, first of all, the name of vertex must be identified.

In this case, face "F 1" has 4 vertexes (V1, V2, V3, and V4). VV1 is a start point of Edge E1 of Sketch1, V2 is a point created as a result of extrusion of V1. And V4 is an End point of Edge E1, V3 is a point created as a result of extrusion of V4. Using these, name of each vertex can be expressed by:

V1: Skettch1, E1, Start

V4:Skettch1,E1,End

V2:Skettch1,E1,Start:Extrude

V3:Skettch1,E1,End:Extrude

As these names can be input into the format of definition, definition of face"FF1" is

## Definition of split face:

Face (F11) = ["Polygonal", 4, "Skketch1, E1, Start", "Sketch1, E1, Start: Extrude ","Sketch1, E1, End: Extrude"," Sketch1, EE1, End"]

On the point of view of basic definition, split face is not different from normal polygonal face. But new operation

is added in a process of splitting of face new points are also created as a result. So, new definition is required to apply these newly created points.

```
V5 = Skketch2,E4,End V8 = Skketch2, E4, End: Cut_Extrude
```

Finally, the definition of face"F1-1" is

FACE (F1-1) = ["Polygon", 4, "Sketch1, E1, Start", "Sketch2, E4, End ", "Sketch2, E4, End: Cut\_Extrude"," Sketchl,E1, and End"]

At the same time, face "F1-2" also can be defined. V2 is a point created as a result of extrusion of V1 and V3 is a point created as a result of extrusion of V4.V6 is a start point of E4 and V8 is a point created as a result of cut- extrude of V4.So definition of face "F1-2" is FACE (F1-2)= ["Polygon",4，"Sketch1,E1,Start: Extrude","Sketch1, E1,End: Extrude ","Sketch2, E4,Start: Cut\_Extrude"," Sketch2, E44, Start" [16].

Fig. 25 Comparison tables of existing researches.

<!-- image -->

Information for the definition of circular face is type, center point and radius of face. Sometimes optional radius is required in the case of donut-shape face. Based on this,basic format of circular face is

## Definition of circular face:

FACE = ["Circle", Center\_Point, Radius, Op\_Radius]

A circular face "F1", whose center is (0, 0) and radius is 10, According to a rule, definition of circular dace "F1" is

FACE (F1) = ["Circle", (0, 0), 10] [9]

## 5.Comparison of the existingresearch

As there is no single standard method for persistent identification, every research addressed the existing problems in their own ways. Among the topology based methods, Capoyleas, Kripac, Wu and Agbodan followed the profile and face based basic naming; Agbodan didn't considered the ambiguity problem solving. Capoyleas and Kripac followed different methods for ambiguity problems. Capoyleas focus was restricted within ambiguity solving where Kripac extended upto name matching. Chen and Zheng considered invariant and contingent entity based basic naming. Among the geometry based methods,Song'smethod is a completelygeometrybased from basic naming to name mapping. It would not generate ambiguity but the problem is this mechanism is not a feature based method. So, retrieving the feature modification information's' from the topological entity name is not possible Wu's basic naming method involves the geometrybased ambiguity solving method which involves name matching also, but the major problem is CAD system dependency. These problems were addressed by Mun, whose basic naming method was followed by Wu's method.But the major difference between Mun's method and Wu's method is Mun considered OSI based ambiguity solving which is CAD system independent, followed by local matching mechanism.Among the topology based methods, Mun's ambiguity solving method is nearly complete with some minor problems such as the feature coverage of these methods shouldbe extendedfurther infuture sothat thismethod can become a standard one for persistent identification.Also,a new basic naming method can be developed which will identify basic entities along with the dimensional modification information as a combination of topology and geometry (Fig. 25).

## 6. Conclusions

As there are many commercial CAD modeling software that are available to us,different CAD system can follow different persistent identification mechanism. Like Solidworks and UG follows geometry based persistent identification mechanism, CATIA and Pro-Engineer follows topology based persistent identification. This paper aims to classify persistent identification mechanism into different branches according to their own geometry. This paper also suggests Mun's topology based basic naming(followedbyWu)and OSI based ambiguity solving method as a standard method. Moreover, basic naming schema can be extended for addressing dimensional modification problems to be identified automatically without any special/differentmethod.Also,thefeatures coveredbythis persistent identification method should be extended in future. Further research work is necessary in creating a hybrid basic naming method in combination with topology and geometry, which will be free from ambiguity problems and will not copy the same name after splitting or merging the topological entities. Until now no research work combines the basic naming method in combination with topology and geometry which will resolve most of the common persistent identification problem.

## Acknowledgments

This research was supported by the Plant Research Program [Project ID:14IFIP-B091004-01] funded by the Ministry of Land, Infrastructure and transport, and by the Industrial Core Technology Development Program [Project ID: 10063452] funded by the Ministry of Trade, Industry andEnergy of the Korean Government.

## Conflict of interest

There is no conflict of interest.

## References

- [1] V. Capoyleas, X. Chen, C. Hoffmann, Generic naming in generative, constraint-based design, Comput.-Aided Des. 28 (1) (1996) 17-26.
- [3] X. Chen,C. Hoffmann, 1995. Design compilation for feature based and constraint-based CAD, in:Proceedings of the third ACMSymposium on Solid Modeling and Applications (SMA *95),ACM, New York, NY, USA, pp.13-19.
- [2] X. Chen, C. Hoffmann, On editability of feature- based design, Comput. Aided Des. 27 (12) (1995) 905-914.
- [4] C.-M. Hoffmann, R. Juan, EREP: an editable high-level representation for geometric design and analysis, Technical Report CER-92-24,Department of Computer Sciences,Purdue University,West Lafayette,Indiana,1993.
- [5] J. Kripac, Topological ID system — a mechanism for persistently naming topological entities in history-based parametric solid models. PhD Thesis, Czech Technical University,School of Electrical Engineering,Department of Computer Science, Czech Republic, Prague, 1994.
- [7] D. Agbodan, D. Marcheix, G. Pierra, Persistent naming for parametric models, Proceedings of the eighth International Conference in Central Europe on Computer Graphics, Visualization and Interactive Digital Media *2000, University of West Bohemia, Campus Bory, Plzen - Bory, Czech Republic, 2000.
- [6] J. Kripac, A mechanism for persistently naming topological entities in history-based parametric solid models, in: C.M. Hoffmann, J.R. Rossignac (Eds.)， Proceedings of Solid Modeling "95 - Third Symposium on Solid Modeling and Applications, 17-19 May, Salt Lake City, UT, USA, ACM Press, New York, 1995, pp. 21-30. Also in: Computer-Aided Design 29(2): 113-122.
- [8] J. Wu, T. Zhang, Zhang, A face based mechanism for naming, recording and retrieving topological entities, Comput. Aided Des.33 (10) (2001) 687-698.
- [9] R. Bidarra, W.F. Bronsvoort, Semantic feature modelling. Comput. Aided-Des. 32 (3) (2000) 201-225.
- [10] R. Bidarra, W.F. Bronsvoort, Persistent naming through persistent entities, in: H. Suzuki, R. Martin (Eds.), Proceedings of Geometric Modeling and Processing 2002 - Theory and Applications, 10-12 July, Wako, Japan, IEEE Computer Society, CA,2002, pp.233-240.
- [11] R. Bidarra, W.F. Bronsvoort, A feature based solution to the persistent naming problem, Comput.-Aided Des. Appl. 2 (1-4) (2005) 517-526.
- [12] D. Marcheix, G. Pierra, 2002.A survey of the persistent naming problem, in: K. Lee, N.M. Patrikalakis (Eds.), Proceedings of Solid Modeling "02 - Seventh Symposium on Solid Modeling and Applications, 17-21 June, Saarbricken, Germany.
- [14] S.U. Cheon, D. Mun, S. Han, B.C. Kim, Name matching method using topology merging and splitting history for exchange of feature-based CAD models, J. Mech. Sci. Technol. 26 (10) (2012) 3201-3212.
- [13] D. Mun, S. Han, Identification of topological entities and naming mapping based on IGM for paramctric CAD model exchanges, Int. J. CAD/CAM 5 (1) (2005) 69-81.
- [15] D. Mun,S.Han (2006), Persistent naming method using OSI (Object SpaceInformation)and SN(SecondaryName),Korea CADCAMconference.
- [17] G.-H. Choi, D. Mun, S. Han, Exchange of CAD part models based on the macro-parametric approach, Int. J. CAD/CAM 2 (1) (2002) 13-21.
- [16] 1. Song, S. Han, Parametric CAD Data Exchange Using Geometry-Based Neutral Macro File, Cooperative Design, Visualization, and Engineering, 7th International Conference, CDVE 2010 Calvia, Mallorca, Spain, September 19-22,2010 Proceedings, 2010.
- [18] http://www.macro-parametrics.org/.
- [19] D. Agbodan, D. Marcheix, G. Pierra, Data Model Arch. Parametr. J. Geom. Graph.3 (1) (1999) 17-38.
- [20] Shahjadi Hisan Farjana, Soonhung Han, Duhwan Mun, Implementation of persistentidentification oftopological entities based on macro-parametrics approach, J. Comput. Des. Eng. (JCDE), http://doi.org/10.1016/jcde.2016.01.001.
- [21] ISO, ISO 10303-111, Industrial automation systems and integration —Product data representation and exchange — Part 11l: Integrated application resource: Elements for the procedural modelling of solid shapes. — Part 11l1: Integrated application resource: Elements for the procedural modelling of solid shapes, 2007.

- [22] Solidworks homepage, 2015. http://www.solidworks.com.
- [23] CATIA homepage,2015.www.3ds.com/products/catia.
- [24] Ammon Hepworth, Daniel Staves, Logan Hill, Kevin Tew, C. Greg Jensen, W. Edward Red, Enhancements for improved topologicalentityidentificationperformanceinmulti-user CAD, Comput.-Aided Des. Appl. 12 (5) (2015).
- [25] Y. Wang, B.O. Nnaji, Geometry-based semantic id for persistent and interoperable reference in feature-based parametric modeling, Comput. Aided Des. 37 (10) (2005) 1081-1093.
- [26] D.Agbodan, D.Marcheix, G.Pierra,C.Thabaud,A topologicalentitymatching techniqueforgeometricparametric
6. models,in: SMI '03: Proceedings of the Shape Modeling International 2003,page 235,Washington,DC,USA,IEEE Computer Society,2003.
- [27] S. Raghothama, S. Shapiro, Boundary Representation Deformation in Parametric Solid Modeling, ACM Trans. Graph.17 (4) (October 1998) 259-286.
- [29] https://www.plm.automation.siemens.com/en\_us/products/ open/jtopen/.
- [28] http://www.freecadweb.org/.