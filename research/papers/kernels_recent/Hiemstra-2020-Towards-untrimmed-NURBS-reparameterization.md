## Oden Institute REPORT 20-06

## April 2020

## Towards untrimmed NURBS: CAD embedded reparameterization of trimmed B-rep geometry using frame-field guided global parameterization

by

René R. Hiemstra, Kendrick M. Shepherd, Michael J. Johnson, Lulin Quan, and Thomas J. R. Hughes

<!-- image -->

Oden Institute for Computational Engineering and Sciences

The University of Texas at Austin Austin, Texas 78712

Reference: René R. Hiemstra, Kendrick M. Shepherd, Michael J. Johnson, Lulin Quan, and Thomas J. R. Hughes, "Towards untrimmed NURBS: CAD embedded reparameterization of trimmed B-rep geometry using frame-field guided global parameterization," Oden Institute REPORT 20-06, Oden Institute for Computational Engineering and Sciences, The University of Texas at Austin, April 2020.

## Towards untrimmed NURBS: CAD embedded reparameterization of trimmed B-rep geometry using frame-field guided global parameterization

Ren´ e R. Hiemstra a, ∗ , Kendrick M. Shepherd b , Michael J. Johnson b , Lulin Quan b , Thomas J.R. Hughes b

a Institut f¨ ur Baumechanik und Numerische Mechanik (IBNM), Leibniz Universit¨ at Hannover b Oden Institute for Computational Engineering and Sciences, University of Texas at Austin

## Abstract

The boundary representation or B-rep is the prevalent geometry description in computer aided design (CAD). It combines aspects from explicit and implicit geometry and is incompatible with many downstream applications such as finite element analysis (FEA) and computer aided manufacturing (CAM). We develop a semi-automatic frame-field guided parameterization approach that converts trimmed B-rep geometry to conforming, watertight analysis-suitable NURBS. The resulting geometry description is simultaneously suitable for design as well as analysis and therefore avoids the need for further meshing and geometry-processing. While existing frame-field based global parameterization methods offer the high quality necessary in our application, they suffer from several robustness issues. Current challenges mainly stem from the lack of integrability of the frame-field which causes the computed parameterization to fold-over, particularly near parameterization singularities. We propose specialized constraints that incorporate properties of an analytical solution that resolve the poor behavior near singularities. Furthermore, we present a convenient and efficient solution framework that directly incorporates such constraints into a reduced basis. Improved behavior around parameterization singularities simplifies the extraction of a quadrilateral layout. This layout is subsequently fitted with C 0 NURBS to yield a conforming watertight model that is analysis suitable. The latter is illustrated by performing isogeometric shell modal analysis on several geometric models.

∗ Corresponding author

Email addresses: rene.hiemstra@ibnm.uni-hannover.de (Ren´ e R. Hiemstra), kendrick@ices.utexas.edu (Kendrick M. Shepherd), mike@ices.utexas.edu (Michael J. Johnson), quanlulin@gmail.com (Lulin Quan), hughes@ices.utexas.edu (Thomas J.R. Hughes)

## Keywords:

Isogeometric analysis, model generation, meshing, parameterization, watertight splines

5

10

15

20

25

## 1. Introduction

Originally computer aided design (CAD) was envisioned as an interactive environment to support the product development process as a whole [15, 46]. This holistic view included the conceptual design phase, in which shape and form are sought to support design objectives, all the way through to detailed design, with frequent intermediate recourse to engineering analysis. Although CAD has in many ways been very successful, it has fallen short of its original intent to serve as an interactive environment for efficient product development. A complete industry exists nowadays to support a 'leaky' design-through-analysis pipeline. According to a report of Sandia National Laboratories in 2005 [5], the time spent creating analysis suitable geometry (which includes many steps, such as de-featuring, geometry decomposition, meshing, and mesh repair ) completely dominates the time spent performing analysis. In recent years this trend has worsened because advances in computing power and efficiency of numerical solvers have cut down the analysis times while not much progress has been made to reduce the time spent on geometry pre-processing.

Isogeometric analysis was introduced by Hughes et al. [28] to improve the interoperability between computer aided design and finite element analysis by unifying the underlying technology that supports both disciplines. Its promise is to eliminate the tedious process of geometry repair, feature removal, and mesh generation, while, at the same time, maintaining a single exact representation of geometry at all stages of the design process. Although isogeometric analysis has proven its fidelity as an analysis technology, some key issues, inherited from traditional CAD, have remained.

The prevalent geometry description used in CAD is the so called boundary representation or B-rep. As the name suggests, geometry is represented by its bounding surfaces. There are two major bottlenecks that impede the design-through-analysis process:

- Apart from shell or boundary element analysis, in general, a three dimensional volumetric parameterization is required to perform finite element analysis. Hence, one challenge is to generate a volumetric description from a boundary representation.
- Boolean operations, ubiquitous in computer-aided design, use a process called trimming that leads to a non-conforming description of geometry that is largely un-editable and incompatible with downstream applications such as finite element analysis, optimization and manufactur-

Figure 1: The mathematical representation in CAD of a structural component of a car, the B-pillar, consist of 1280 trimmed NURBS surfaces that do not form a single conforming or watertight model. (Urick, B. Private communication. 2018)

<!-- image -->

ing.

- The former is commonly referred to as the volumetric problem , while the latter as the surface 30 problem .

35

40

In this work we focus on the surface problem. An example illustrating the severity of the surface problem is depicted in Figure 1. Complex CAD models are made out of thousands of NURBS patches that are plagued by 'dirty geometry', such as slivers, gaps and overlaps. Such a geometry representation is useless in downstream applications such as finite element analysis (FEA) and computer aided manufacturing (CAM) without thorough geometry repair and cleanup.

## 1.1. Limitations of the boundary representation

A B-rep is defined by a topological structure called a cell complex, a collection of NURBS surfaces and a collection of trim curves, see Figure 2. The trim curves serve to divide the surfaces into their visible and masked regions. The cell complex prescribes how the collection of NURBS surfaces are glued together to form an open or closed poly-surface.

The trim curves of the boundary representation generally do not align with the parametric directions of the surface representation and thus are only represented approximately (and disparately)

45

50

55

Figure 2: Boundary representation (B-rep) of a CAD solid model [36]

<!-- image -->

in both model and parametric space. The B-rep paradigm combines aspects of explicit parametric geometry with implicit geometry, which leads to several problems [31, 36, 43]:

1. Many surface operations are prohibited after a Boolean operation (i.e. a subtraction, intersection, or union) has been performed. The reason for this is that an edit could potentially change the topology of the model, which is not allowed by most CAD systems.
2. Major geometric features are introduced that are not aligned with the parametric directions of the underlying surface patches. This can lead to loss of shape control near trim curves for those surface operations that are permitted by the CAD system.
3. Trim curves may be represented slightly differently depending on the geometry kernel of the CAD system. This can lead to loss of accuracy, or worse, to undesirable surface artifacts, like gaps and overlaps, when converting to a common exchange format such as STEP.
4. Trim often leads to non-conforming geometric models that have undesirable gaps and overlaps that pollute downstream processes.

It turns out that trim is not only detrimental for the interoperability with downstream applications, such as finite element analysis and computer aided manufacturing, but it also constitutes

60

65

70

75

80

a major impediment for computer aided design itself! For these reasons, the Handbook of Solid Modeling [33] advises design engineers to avoid, if possible, Boolean operations. This represents a dilemma, because trimming is among the most used operations in order to model complex models of engineering interest [51]. Riesenfeld, Haimes, and Cohen [46], eminent researchers in the field of computer aided design, urge for a complete reevaluation of present-day tools and practices.

## 1.2. A new modeling paradigm

Traditionally, meshing has been regarded as a separate process that converts a design model into an analysis model. This process is in general tedious and labor intensive and leads to model inaccuracies. The motivation for this work is to embed re-parameterization techniques directly within CAD, thereby enabling the modeling of watertight conforming CAD geometry that is directly compatible with downstream applications such as FEA and CAM. In this sense we share a common philosophy with the recent work of Urick et al. [57]. Our methodology is based on recent advances in topological vector field design and processing [58], quadrilateral meshing [6] and watertight spline representations [48]. Before we explore the state of the art in existing technologies we briefly summarize the workflow, which is illustrated in Figure 3.

Assume that a given B-rep is of sufficient quality such that a linear triangulation can be obtained. Features such as B-rep vertices and boundaries are extracted directly from the B-rep and serve as input for the computation of a frame-field (see Figure 3d). Additional information, such as the principal curvature directions obtained directly from the B-rep, can also be included. The framefield is then used as a guide to obtain a high quality quadrilateral parameterization. We are interested in the coarse quad-layout structure, which is obtained by tracing the separatrices, that is, the parameterization iso-lines that emanate from singular points. The coarse quad-layout is used as a template to build a spline space. Each face in the layout corresponds to a conforming NURBS patch. If desired, these can be smoothly connected using technology such as T-splines [49] with improved treatment of extraordinary points [56]. Finally the spline surface is fitted to the original boundary representation to obtain a conforming and editable representation of the geometry.

<!-- image -->

ization

Figure 3: Proposed workflow of obtaining a watertight conforming representation from a B-rep. The B-rep consisting out of trimmed NURBS patches in (b) is triangulated (c) and feature points and lines are extracted. Subsequently a frame-field (d) is computed that follows the prescribed features. Guided by the frame-field a quadrilateral parameterization is computed (e). Finally, the coarse layout structure of the model parameterization is determined and a new, conforming and watertight NURBS model (f) is constructed that fits the original B-rep within a specified tolerance.

## 1.3. Prior work 85

Quadrilateral surface meshing has been a topic of intense research for many decades. Recently, the field has seen a rejuvenation with many new and original contributions from the graphics community. For an overview we refer the reader to [7].

90

95

100

105

Meshing algorithms can be classified into local and global methods. Local meshing algorithms (e.g. [4, 39]) are generally efficient, robust and are able to fit local features accurately. However, they do not provide much control over mesh singularities, which can lead to highly irregular unstructured meshes. Furthermore, even the most robust methods are generally 'quad-dominant,' meaning they require a small number of triangles. In contrast, global algorithms [6, 11, 20, 23, 30, 34, 35] are capable of generating highly regular parameterizations containing few mesh singularities. Many are able to accurately capture both local and global geometric and topological features [22, 27, 37]. Furthermore, some methods provide interactive control over placement of singularities [16] and control over global patch layout [12, 53]. In this work we focus solely on global parameterization methods because these methods provide the type of regular quadrilateral patch layouts that can be used as a basis for building analysis suitable splines. We do note that global parameterization is still an active area of research and obtaining robust and efficient implementations is challenging.

## 1.3.1. Frame-field guided re-parameterization

Let M = ( V, E, F ) denote an orientable open or closed linear surface triangulation . The sets V , E and F refer to its vertices v i , edges e ij and faces f ijk , respectively. A cut-graph G of M is a subset of the edges in E such that M \ G is a simply connected mesh. We consider a special cut-graph of M that also extends to a prescribed set of topologically admissible cone-singularities p ∈ M [45], derived from a frame-field F on M . The fundamental domain M ′ = ( V ′ , E ′ , F ′ ) is a topological disk consisting of all faces in F ( F ′ ≡ F ) and can be straightforwardly obtained by cutting M along G [29].

A quadrilateral surface parameterization of M is defined by two parameterization functions, denoted by

<!-- formula-not-decoded -->

110

115

120

125

130

When carefully prescribed these functions define a quadrilateral grid on M when sampled at a suitably chosen set of iso-parameter lines. Particularly, if their values at cone-singularities are integers and transitions across the edges in G are seamless and of integer value, then these functions define an integer grid map [6, 7, 8, 9, 18, 40]. Although there is progress [13] existence of arbitrary parameterizations is yet an unsolved problem.

Typically, these methods proceed by minimizing an energy

<!-- formula-not-decoded -->

subject to transition constraints along G and integer constraints, Ψ i ( p ) ∈ Z , at cone-singularities p ∈ M . Here { F i ∈ F , i = 0 , 1 } denote the linear independent components of a frame-field that have been computed a-priori. There are several difficulties with this approach:

1. The vector fields F i , i = 0 , 1 may be far from being integrable (curl-free), in which case the computed parameterization functions typically exhibit undesirable behavior such as folds and large deviations from the original, smooth fields.
2. Mixed integer programming problems are NP-complete, and thus difficult and expensive to solve.
3. The two coordinates of the parameterization may overlap near parameterization singularities.

A complete, robust implementation of frame-field guided global parameterization will have to deal with the above issues. Previous work in [18, 45, 59] has focused on curl-free frame-fields that improve the first item. Other methods, for example [38], eliminate the use of integer constraints. [21] presents a post-processing strategy to deal with ill-behavior around parameterization singularities and other undesirable artifacts.

## 1.4. Contributions and outline

The contributions of this paper are summarized as follows:

1. We present a global quadrilateral surface meshing pipeline to re-parameterize B-rep geometry to conforming watertight NURBS geometry. Our approach builds on many aspects such as frame-fields (Section 2), global parameterization (Section 3), layout extraction, and NURBS

135

140

145

150

155

fitting (Section 5). Several examples show how re-parameterized NURBS models can be used directly within structural shell analysis codes (Section 6.2).

2. We introduce specialized constraints in Section 3.2.4 that lead to well defined behavior near parameterization singularities. The computed parameterization is guaranteed to be inversionfree in the one-ring around a singularity by locally incorporating properties of an analytical solution. This means that parameterization separatrices can be traced effectively, without the need for additional post-processing, leading to simplified layout extraction (Section 5.1).
3. In Section 4 we present a convenient and efficient framework for handling linear constraints in general optimization problems. Constraints are built into a reduced basis using a special nullspace method. The basis satisfies the prescribed linear constraints exactly and is maximally sparse, thereby preserving the sparsity of the original problem. The final linear system of equations is symmetric, positive definite and can be efficiently solved using Cholesky or the conjugate gradient method.

The first contribution is aimed towards the development of computer aided design techniques that yield analysis suitable geometric models, by construction, throughout the process of design. Success of the proposed technology depends on the solution of several open challenges discussed in the previous sub-section. Our second and third contribution completely resolve challenge (3) of those listed in Section 1.3.1. Future work relating to the remaining challenges and other interesting avenues for research are discussed in the conclusion, see Section 7.

## 2. Computation of frame-fields

In this section we briefly review the computation of a guiding frame-field on surface triangulations, feature alignment and extraction of frame-field topology by a process that is called framefield matching. The approach is based on previous advances in topological vector field design and processing [58] and, in particular, the works in [32, 40].

## 2.1. Preliminaries

Input of the proposed methodology is a valid B-rep computer aided design model. It is assumed that the model is of good quality such that a valid conforming triangulation can be extracted with

160

165

170

175

180

185

relative ease. In practice this means that it has a valid cell-complex, manifold structure, and does not have sliver surfaces nor undesirable gaps that exceed the matching tolerance of the CAD system. A B-rep model that does not satisfy these conditions requires manual/semi-automatic clean-up using embedded CAD-tools or dedicated software such as CADfix 1 . An example of a valid B-rep is shown in Figure 4. B-rep features, such as G 0 boundary curves, G 1 feature lines and sharp corners are automatically extracted. Subsequently, a conforming triangulation M = ( V, E, F ) is computed that accurately captures the main features of the B-rep geometry. The discrete spaces used to represent frame-fields as well as the parameterization functions are based on linear interpolation of vertex-data on a triangulation. All computed fields are ultimately pulled back to the actual B-rep surface geometry, and hence, even a fairly coarse linear approximation of parameterization fields does not negatively effect the geometrical error made in the re-parameterization process.

## 2.1.1. Implementation

This work follows the approach proposed in [40] in which a frame-field is represented by a cross-field in a particular metric. First, a smooth metric is computed by minimizing a standard discrete Dirichlet energy over the space of symmetric positive definite tensor fields. In contrast to [40] we use a vertex-based discretization for the unknown tensor fields, instead of a face-based discretization. Our implementation of cross-fields follows that of [32]. We refer to these works for additional details regarding theory and implementation and highlight here several details important for our application.

Remark 2.1 . The main advantage of a vertex-based representation over a face-based representation is that feature, boundary and corner constraints can be strongly applied with ease. One of the main advantages of the face-based approach, compared with vertex-based fields, is that consistent field tracing is easily implemented, see e.g. [38].

## 2.2. Feature constraints

Figure 6 illustrates the type of constraints that are implemented at feature vertices. Locally, the behavior of the frame-field is controlled via a so-called feature angle, denoted by θ feature . Given

[1 See https://www.iti-global.com/cadfix](https://www.iti-global.com/cadfix)

116.00°

G

0

boundary curves

feature vertex

Figure 4: Features are automatically derived from the Brep. These include angles at feature vertices, G 0 feature curves such as boundaries and G 1 feature curves between B-rep faces.

<!-- image -->

Figure 5: The frame-field is constrained to satisfy G 0 and G 1 feature curve alignment and alignment at corners. Note that angles close to 90 deg are treated differently than angles close to 180 deg.

a feature angle, the frame-field vectors are defined, up to a possible scaling. If the scaling is unknown we choose the vectors to have unit-length. This uniquely defines the metric tensor and cross-field vector at these vertices and these values are used as boundary alignment constraints in the optimization problem.

## 2.3. Frame-field matching 190

195

The individual components of a frame-field are not necessarily ordered consistently. The process that recovers the 4 individual components of a crossor frame-field is called matching and is illustrated in Figure 7. Figure (a) shows a frame-field with singularities of index -1 / 4. The first, unordered, component is shown in Figure (b). The first linearly independent component of the frame-field, recovered after performing matching, is depicted in Figure (c). The process consists of

G

1

90.00°

feature curves

154.00°

200

205

Figure 6: Alignment constraints close to feature vertices in green. The feature angle θ feature , set to 45 deg, provides precise control over the behavior at a feature vertex.

<!-- image -->

the following steps

1. Identify faces that contain a frame-field singularity.
2. Compute the fundamental domain M ′ obtained by slicing M along edges of a cut-graph G from all interior boundary components, including the faces with singularity, to the exterior boundary in such a way that M ′ is homeomorphic to a disk. 2
3. Compute a vertex-spanning-tree: a tree data-structure that connects a start vertex to every other vertex in the fundamental domain M ′ along a single unique path.
4. Compute matching difference between frame-field vectors located at adjacent vertices. This yields an integer corresponding to every edge e ij that measures the number of π/ 4 rotations necessary to rotate the vector at vertex v i to smoothly align with the vector at vertex v j .
5. The cumulative sums of the matching differences along the paths of the vertex-spanning-tree

2 Here we assumed that we are working with open surfaces of genus 0 with k boundary components. For general surfaces one has to compute a basis for the 1st cohomology group of the surface and use these to slice the domain to obtain disk-topology.

210

yield the rotation of every frame-field vector to align smoothly with the frame-field vector at the start vertex.

For further reading on direction field matching we refer to [58]. For general references on computational geometry and mesh processing algorithms, including topics such as fundamental domain , cut-graphs , shortest path algorithms , and vertex spanning-trees , we refer to [10, 17, 19, 29].

Figure 7: Frame-field matching. (b) The four individual vectors at every vertex are not ordered consistently. To obtain the individual components of a frame-field we have to consistently match them across the simplicial surface. The matched first component are shown in Figure (c). The second component can be obtained similarly. Note that the individual vector fields are continuous everywhere except across the edges of the cut-graph.

<!-- image -->

## 3. Configuration space of admissible parameterization functions

The two linearly independent components of the frame-field , { F i ∈ F , i = 0 , 1 } , provide topological and alignment constraints for the computation of a global, conforming parameterization. In particular, after the matching process we know 215

- position and index of the parameterization singularities
- which component is tangent along the boundary of M

220

225

- the change in component across edges of the cut-graph G .

In this section we use this information to construct a discrete configuration space of admissible parameterization functions that has the appropriate topological and alignment constraints built-in to the space. Subsequently, we use this space to compute a parameterization that minimizes the energy functional E ( · ) in (2).

## 3.1. Preliminaries

Consider the following discrete space

<!-- formula-not-decoded -->

Here { φ j ( x ) , j = 1 , . . . , | V ′ | , x ∈ M } denote the classical linear hat functions defined on the fundamental domain M ′ . Note that the degrees of freedom are defined as complex numbers. Suppose Ψ h ∈ S h . Its real and imaginary components will be referred to as Ψ h 0 := Re ( Ψ h ) and Ψ h 1 := Im ( Ψ h ) .

The configuration space of admissible parameterization functions is a subspace of S h , denoted and defined by

<!-- formula-not-decoded -->

Here C ( · ) denotes a set of linear constraints. In the coming sections we shall discuss each of the following constraints in more detail

<!-- formula-not-decoded -->

We consider parameterization as a flow problem, similar to the setting of [30]. The configuration space S h c , with additional integer constraints, can also be identified as the space of integer grid maps [6, 7, 8, 9, 18, 40]. In contrast to these works, we do not require that the value of 230

235

240

the parameterization at cone singularities are integral, alleviating the need for a mixed-integer optimization. Instead, we allow user-prescribed path constraints that align iso-parameter values between multiple parameterization singularities. A disadvantage of this approach is that mesh extraction is more involved because now tracing iso-parameter lines involves tracing of fl oating-point , rather than integral , values.

## 3.2. Parameterization constraints

In this sub-section we discuss each of the constraints in (4b) in more detail. Subsequently, we show how to build the constraints into a reduced (sparse) basis such that they are satisfied by construction. This enables fast solvers such as Cholesky or the conjugate gradient method to be used to solve the resulting matrix problem.

## 3.2.1. Constraints that remove the kernel

The energy in (2) is not definite on the space of functions S h . Elements of S h are defined up to a constant. In order to obtain a unique solution it is important that this constant is set a priori, thereby removing the non-trivial kernel from the resulting set of matrix equations. A simple way to achieve this is by specifying a constraint on the average

<!-- formula-not-decoded -->

## 3.2.2. Continuity constraints across cuts

The two linearly independent vector fields { F i ∈ F , i = 0 , 1 } , which are obtained from the frame-field after the matching process, are continuous everywhere except across edges of the cutgraph, see Figure 7. It is here where the components switch direction. Consequently, special continuity constraints along edges of the cut-graph are necessary to couple the two components of the parameterization, { Ψ h i , i = 0 , 1 } , such that their tangent component is continuous across the cuts

<!-- formula-not-decoded -->

Here k match ∈ { 0 , 1 , 2 , 3 } is an integer that prescribes the change in coordinate direction across an edge of the cut-graph. We refer to [58] for additional details on this subject.

## 3.2.3. Tangential boundary constraints 245

Recall that under the metric computed in Section 2, components of the frame-field were aligned to various surface features, including the boundary. Let ∂M tangential i denote the set of feature curves and boundaries that F i , i = 0 , 1 was forced to align with. To align the parameterization iso-lines with these feature curves and boundaries we have to add additional constraints that express this tangency relation

<!-- formula-not-decoded -->

These constraints can be applied strongly for every edge on the feature curve or boundary.

## 3.2.4. Template constraints at singularities

The solution of the frame-field is always under-resolved in the vicinity of frame-field singularities, no matter the mesh-size (see, e.g. [50]). With mesh refinement this unresolved area shrinks. However, the character of the solution is independent of the mesh-size and therefore behavior around singularities remains problematic. Minimization of the energy in (2), without a proper treatment around singularities, generally leads to local fold-overs in the Ψ h 0 -and Ψ h 1 - coordinates of the parameterization. In [21] this problem is addressed in a post-processing step. We address this issue directly within the solver by incorporating properties of an analytical solution near singularities.

## Singularity template

Consider a singularity with valence 3 ν . We wish to construct a conformal mapping, G ν : C ↦→ C , that is rotationally symmetric, with angle θ ν = 2 π/ν , about the singularity located at origin 0 ∈ C . The mapping has a branch-cut aligned with the real axis. Let coordinates y ∈ C be parameterized in terms of polar coordinates such that y = r cos( θ ) + i · r sin( θ ), with angle θ ∈ [0 , 2 π ) and radius r ∈ R + . The domain of the mapping can be divided into ν sections. Point y ∈ C is located in section k ( y ) = ⌊ θ ( y ) /θ ν ⌋ . With these definitions, the conformal mapping G ν

3 Recall that valence and singularity index of a frame-field are related by: index = 1 -valence / 4.

250

255

Figure 8: Conformal mappings around singularities of different types. Note that the mapping is continuous everywhere except at the branch cut .

<!-- image -->

can be constructed as

<!-- formula-not-decoded -->

The conformal mapping G ν ( · ) is fixed for a given valence ν and is constructed only once. For this reason we call this a singularity template . When considered in the form

<!-- formula-not-decoded -->

we can distinguish between the two separate coordinates of the mapping. Figure 8 shows several examples of G ν ( y ) corresponding to different mesh valence numbers. The blue iso-lines correspond to the u -coordinate while the red iso-lines correspond to the v -coordinate. As can be observed

260

in the figures, the mappings are smooth everywhere except across the branch-cut. In practical applications we only consider valence 3 and 5; these are the type of singularities that emanate from the frame-field computation.

## Application of template constraints

To improve the behavior of the numerical solution we constrain it in the 1-ring neighborhood N 1-ring around each face containing a singularity to interpolate an analytical solution of a valence three or five conformal mapping, see Figure 9. To achieve this, the 1-ring neighborhood is first flattened using triangle flattening [16]. Let x ∈ C denote coordinates of N 1-ring with origin o , chosen as the barycentric mean of the face containing the singularity, and consider the two-parameter family of mappings g ν [ R,S ] : C ↦→ C , defined by

<!-- formula-not-decoded -->

The two complex degrees of freedom R,S ∈ C or, equivalently, the 4 real degrees of freedom a, b, s, t ∈ R such that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

are not known a priori and should follow from the solution to the global parameterization problem.

Consider the set of m global vertex coordinates x ι k ∈ N 1-ring , k = 1 , ..., m and consider the local coordinates y k = x ι k -o, k = 1 , ..., m . The constraints on the global parameterization functions Ψ i , i = 0 , 1 can then be stated as

<!-- formula-not-decoded -->

265

270

275

Here Ψ i ( x ι k ) , i = 0 , 1 denote the two parameterization functions evaluated at vertex position x ι k . The matrix in (13), denoted by ˆ B ∈ R m × 4 , is designed such that the parameterization functions Ψ i ( x ) , i = 0 , 1 interpolate, by construction, the chosen conformal mapping at a selection of vertex positions x ι k , k = 1 , ..., m near the singularity

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The matrix ˆ B is fixed for a singularity of valence ν and set of local vertex coordinates y k , k = 1 , ..., m and is determined in a pre-processing step to the global parameterization.

It can be checked that ˆ B is of full rank. Hence, there exists a matrix ˆ C ∈ R m -4 × m such that

<!-- formula-not-decoded -->

Matrix ˆ C can be interpreted as the linear constraints acting on the vertex values Ψ i ( x ι k ) , i = 0 , 1 , k = 1 , ..., m such that they interpolate the chosen conformal mapping in the one-ring around the singularity. Matrix ˆ C is computed using the null-space machinery presented in the next section.

## 3.2.5. Path constraints

An iso-line is a curve parameterized by one of the coordinate functions, while keeping the other one constant, at least when not traversing a cut of the cut-graph. Suppose γ : [ a, b ] ↦→ M is a path on M that does not traverse an edge of the associated cut-graph G . By the fundamental theorem of calculus we have

<!-- formula-not-decoded -->

Hence, if we wish that the parameterization coordinate Ψ h i has the same value at γ ( b ) as it has at γ ( a ) then the flux over γ should cancel. If curve γ crosses an edge of G then a similar result holds, but we need to take into account the switch in component direction.

Figure 10 shows an example of a parameterization before and after the path constraints have been added between parameterization singularities. In this work we allow the user to specify path constraints using a graphical user interface embedded within the CAD system. First, the user selects two singularities that need to be matched, as well as a topological path connecting the two.

Figure 9: Interpolation on the cut-mesh of the Ψ h 0 -coordinate of a conformal mapping with a valence 3 and 5 singularity. The Ψ h 1 -coordinate is similarly interpolated. Note that the cut-path to the face that contains the singularity coincides with the branch-cut of the conformal mapping. These constraints are applied within the parameterization such that locally, around the singularity, the parameterization is well defined and inversion-free.

<!-- image -->

280

A default path could be the shortest path on M given by Dijkstra's algorithm. Then, the most likely component of the parameterization is chosen automatically based on the minimum of the computed fluxes, ∫ γ dΨ h i , i = 0 , 1, see Figure 11. Finally, the new path constraints are used in the next parameterization solve leading to the result shown in Figure 10.

Figure 10: Path constraints can be added by the user leading to improved layouts.

<!-- image -->

## 4. Sparse null-space solution methodology

Minimization of the Dirichlet energy in (2) within the configuration space of admissible parameterization functions S h c leads to a saddle point problem of the following form

<!-- formula-not-decoded -->

285

290

Figure 11: The total flux of both components of the parameterization is computed along a path connecting two singularities. Component Ψ h 0 has a smaller flux across the path than Ψ h 1 . It thus makes sense to place a zero-flux condition on component Ψ h 0 of the parameterization.

<!-- image -->

Here A ∈ R N × N is a symmetric positive semi-definite matrix 4 associated with the quadratic functional. Matrix C ∈ R M × N , assumed to be of full rank 5 , collects the constraints. x ∈ R N denotes the vector with degrees of freedom corresponding to the parameterization functions and p ∈ R M are Lagrange multipliers. By formulating the problem as a saddle point problem we have lost the positive (semi)-definiteness of the unconstrained problem. This is disadvantageous because it limits the use of fast direct and iterative solvers such as Cholesky factorization and the conjugate gradient method, respectively. Furthermore, instead of reducing the effective number of degrees of freedom with M , due to the presence of M linear constraints, we have actually increased their total by M Lagrange multipliers.

By constructing an explicit basis for the null-space of the constraint matrix C we can, instead,

4 N equals 2 | V ′ | .

5 This assumption simplifies the discussion. The solution strategy we propose does not need C to be of full rank. In practice, we allow constraints that are linearly dependent.

295

300

solve a direct problem that involves inverting a symmetric positive definite matrix. The key ingredient of a null-space method [3] is the availability of a matrix Z ∈ R N × N -M , of full rank, that satisfies

<!-- formula-not-decoded -->

In other words, the columns of Z span the null-space of C . The saddle-point problem can then be solved for x by first solving

<!-- formula-not-decoded -->

in terms of an auxiliary variable y ∈ R N -M . Assuming the constraints in C properly remove the null space of matrix A , it follows that, Z T AZ is symmetric positive definite and can be pre-factored with Cholesky or solved iteratively using the conjugate gradient method. The solution x can then simply be recovered by x = Z y .

The different types of null-space methods depend specifically on the specific properties of Z and the way it is computed. For example, a null space basis can be computed using QR-factorization. The resulting matrix Z might, however, be quite densely populated. For an overview we refer to the work by Benzi, Golub and Liesen [3]. In our setting it is important that Z is sparse in order to preserve the sparsity of the original unconstrained problem. In the following we propose a strategy that leads to maximally sparse null-space bases. The approach is similar to those proposed in [24, 54, 55], in which non-uniform degree univariate B-splines and Tchebycheffian splines were constructed. An alternative approach that has features in common with ours is presented in [9].

## 4.1. Sparse basis for the null space of a single vector

First consider the case of a single constraint, c T x = 0, with c ∈ R n . Geometrically, we can consider the solution x ∈ R n to be restricted to an n -1-dimensional subspace of R n . Therefore, it is possible to transform the original constrained problem to an unconstrained one with n -1 degrees of freedom. Let P : R n ↦→ R n denote the permutation matrix such that ˜ c = P c has the following sparsity structure

<!-- formula-not-decoded -->

305

310

315

320

We seek a matrix ˜ Z ∈ R n × n -1 such that range( ˜ Z ) = ker(˜ c T ). Furthermore, we require ˜ Z to be maximally sparse and have columns with unit 1-norm, which improves conditioning. In general, this is an l 1 optimization problem. In this simple case, however, we can determine ˜ Z explicitly. Firstly, assume ˜ Z has the following sparsity structure

<!-- formula-not-decoded -->

The non-zero values are uniquely determined by requiring that ˜ c k ˜ Z k,k +˜ c k +1 ˜ Z k +1 ,k = 0 and | ˜ Z k,k | + | ˜ Z k +1 ,k | = 1 for k = K,.. . , n -1. Algorithm 1 encodes the discussed logic. It can be verified that ˜ Z is of full rank, is maximally sparse and is a basis for the null space of ˜ c . Finally, note that we require the inverse permutation of ˜ Z , that is Z = P -1 ˜ Z .

## 4.2. Sparse basis for the null space of a matrix

Using induction, the described approach can be extended to the setting with more than one constraint. This logic is encoded in Algorithm 2. The numerical stability of the described approach remains to be studied. We do note that we have not witnessed any unstable behavior in practical computations.

## 5. Quadrilateral layout and NURBS fitting

The template constraints at singularities lead to well-posed behavior around singularities. The computed parameterization is guaranteed to be inversion-free within the one-ring elements that are constrained. Consequently, all ν separatrices - iso-lines of the parameterization emanating from parameterization singularities - corresponding to a valence ν singularity are uniquely defined and can be successfully traced away from the singularity, across the triangulation M . If the parameterization is inversion-free then its separatrices, assumed of finite length, uniquely divide the surface

## Algorithm 1 Computation of a basis for the null space of a single vector

```
1: function nullspace ( c ∈ R n ) 2: c ← P c ▷ Order the values of c according to their absolute magnitude 3: if abs( c n ) < ϵ then ▷ If c is trivial return identity matrix 4: Z ← identity matrix (size: n × n ) 5: else 6: Z ← zero matrix (size: n × ( n -1)) 7: k ← 1 8: while k < n & abs( c k ) < ϵ do ▷ Treat zero values 9: Z k,k ← 1 10: k ← k +1 11: end while 12: while k < n do ▷ Treat non-zero values 13: v ← c k /c k +1 14: Z k,k ← 1 / (1 + abs( v )) 15: Z k +1 ,k ←-Z k,k ∗ v ▷ abs( Z k,k ) + abs( Z k +1 ,k ) = 1 16: k ← k +1 17: end while 18: Z ← P -1 Z ▷ Apply inverse permutation 19: end if 20: return Z 21: end function
```

up into a set of quadrilateral patches [21]. This so called quadrilateral layout, depicted in Figure 12, provides the main structure that is used to build a conforming, watertight NURBS parameterization. To every face in the layout we associate a quadrilateral NURBS patch. Using NURBS, the continuity between patches will typically be limited to C 0 . Unstructured spline technologies, such as T-splines, can additionally provide higher order continuity between patches. 325

## Algorithm 2 Computation of a null space basis

- 1: function nullspace basis ( C ∈ R M × N )

- 2: Z ← identity matrix (size: N × N )

▷ Initialize null-space basis

- 3: for k = 1 : M do

- ▷ Loop over constraints

- 4: ˜ Z ← nullspace ( C k : )

▷ Compute nullspace of k -th row of C

- 5: Z ← Z ∗ ˜ Z

▷ update Z

- 6: C ← C ∗ ˜ Z

- ▷ update C

- 7: end for

- 8: return Z

- 9: end function

Figure 12: The coarse quadrilateral layout forms the main topological structure used to reconstruct a multi-patch watertight NURBS parameterization.

<!-- image -->

## 5.1. Fitting problem

The fitting stage can be separated into an in-plane (intrinsic) and out-of plane (extrinsic) problem. The in-plane problem can be solved within the parameter space of the initial B-rep and

330

335

340

345

350

355

only involves approximation near the trimming curves. The out of plane difference between the initial B-rep and the new conforming spline surface is where most of the approximation enters. Refine-ability of spline basis functions imply that the error due to approximation can be made arbitrarily small. Several projection methods may be useful here such as B´ ezier projection [52] and Coon's patches [36, 42]. Other references of interest are [48, 57].

Although spline-fitting is a well studied subject, there is remarkably little prior work done in the context of the considered application. We have identified several important criteria

1. Flat faces should remain flat.

2. Convex shapes should remain convex.

3. Conic shapes should be represented exactly.

4. Boundary curves and other feature curves, such as creases or those bordering fillets, should be represented, if possible, exactly. This does not include Boolean intersection curves, because these cannot in general be represented with low to medium order polynomials [47].

5. Free-form surfaces should be fitted up to a user-defined tolerance.

Note that some of these requirements may be at odds with one-another, e.g. convexity and optimal approximation do not usually go hand-in-hand. Consequently, the fitting stage represents a nontrivial step within the re-parameterization pipe-line.

## 5.2. Implementation

In this paper we use cubic rational B´ ezier patches to rebuild a watertight representation. First, each edge in the quadrilateral layout is approximated by a cubic rational B´ ezier curve. For the inplane problem, we use a Coon's patch to reconstruct a rational surface from the boundary data, see [36, 42]. Out-of-plane fitting can be performed by solving a Hermite interpolation problem. Consequently, the resulting watertight and conforming B-rep is G 1 continuous between patch interfaces. A more advanced approach, using e.g. T-splines, is left for future work.

## 6. Results

Traditionally, meshing has been considered a separate, intermediate process in between design and analysis. Instead, we apply re-parameterization techniques directly within CAD as part of

360

365

370

375

380

the design process. This results in computer aided design models that are directly suitable for downstream applications such as finite element analysis. In this section we discuss implementational aspects of the methodology and apply it to several computer aided design models of engineering interest. The framework yields computer aided design models that are, by construction, analysis suitable. In this chapter this is illustrated by shell modal analyses applied to several parts of the unclassified generic hull vehicle model, provided by the U.S. Army Combat Capability Development Center (CCDC) Ground Vehicle Systems Center (USACCDCGVSC) 6 .

## 6.1. Implementation in Rhino/Grasshopper

A proof of concept of the proposed methodology is implemented in Rhino 6 for Windows using the RhinoCommon C# SDK 7 . This includes support for sparse and dense linear algebra, triangulation of B-rep CAD geometry, computation of cross- and frame-fields, topological operations such as cut-graphs, computation of a quadrilateral parameterization, extraction of quad-meshes, and finally, native implementations of watertight and conforming NURBS geometry. Rhino ships with Grasshopper 8 which provides a visual programming interface for Rhino and has access to Rhino geometry objects. We use it to combine the different stages in the pipeline: (1) B-rep triangulation; (2) frame-field computation; (3) computation of the fundamental domain and extraction of individual frame-field components using matching; (4) global parameterization; (5) layout extraction. Grasshopper also provides an interface that allows selection of path-constraints in the global parameterization step.

## 6.2. Isogeometric shell analysis using LS-Dyna

Figures 15 and 16 show two re-parameterized B-rep models from the generic hull vehicle. This includes the model that has been used throughout this paper to illustrate several aspects of the methodology and a more complicated model. The figures illustrate several of the details discussed, such as alignment of the parameterization with main features and global coarse patch structure. We recall that the workflow discussed automatically places singularities of a certain type in desirable

6 The CAD files are published as supplementary materials to this paper.

7 https://www.rhino3d.com/download/Rhino-SDK/6.0/release

8 https://www.rhino3d.com/6/new/grasshopper

Figure 13: Trimmed representation of the outer hull plating of the DEVCOM Generic Hull vehicle model.

<!-- image -->

385

390

Figure 14: Conforming parameterization of mid-surfaces of the outer hull plating of the DEVCOM Generic Hull vehicle model.

<!-- image -->

locations. The global patch layout is subsequently optimized by the user by prescribing a small set of path constraints connecting singularities. Finally, the B-rep is rebuilt according to the obtained patch layout using cubic rational B´ ezier patches. Both models exactly describe the circular holes and fit exactly the prescribed features of the initial B-rep. Note that symmetry across the main axis has been used to create the watertight NURBS models.

We perform isogeometric shell modal analysis on the two models using LS-DYNA of Livermore Software Technology Corporation 9 , an industrial, general purpose finite element code. LS-DYNA features a shell NURBS element and has several aspects of isogeometric analysis implemented, including explicit / implicit dynamics, eigenvalue problems and large deformation analyses involving contact [1, 2], combined with a large class of constitutive models. The analysis models, depicted in Figure 15d and 16d are obtained by uniform h-refinement using global knot insertion [41].

9 https://www.lstc.com/

Figure 15: A B-rep is re-parameterized into a conforming, watertight model consisting of 78 cubic rational B´ ezier patches. Figure (b) shows the NURBS control net and Figure (c) and (d) the B´ ezier mesh corresponding to the coarse and fine discretization. The refined model, which is used in modal analysis, consists of 10 × 10 uniformly refined cubic NURBS patches that are C 2 within the patch and C 0 in between patches.

<!-- image -->

Figure 16: A B-rep is re-parameterized into a conforming, watertight model consisting of 146 cubic rational B´ ezier patches. Figure (b) shows the NURBS control net and Figure (c) and (d) the B´ ezier mesh corresponding to the coarse and fine discretization. The refined model, which is used in modal analysis, consists of 10 × 10 uniformly refined cubic NURBS patches that are C 2 within the patch and C 0 in between patches.

<!-- image -->

395

400

405

410

We have used the following setup

- Reissner-Mindlin shell formulation.
- Linear isotropic material with density ρ = 7800 kg/m 3 , Young's modulus E = 200 GPa and Poisson's ratio ν = 0 . 3 (aluminum).
- Plate thickness h = 12 mm .
- Full Gaussian quadrature.
- Consistent mass.

The Reissner-Mindlin shell simulates shear deformable shells and features rotational degrees of freedom. Consequently, we require only C 0 -continuity in between elements and patches. We refer to Benson et al. [2] for further details.

Figures 17 and 18 show a selection of the low modes obtained from the discussed linear modal analysis. The color plots show the maximum displacement of the mode under consideration.

## 7. Conclusions and recommendations

Although computer aided design has in many ways been very successful it has fallen short of its original intent to serve as an interactive environment for efficient product development. Many of the problems trace back to a particular geometry description developed in the early 70's, called the B-rep or boundary representation . The B-rep combines aspects of explicit parametric geometry with an implicit representation that arises during Boolean operations using a process called trimming. The description is non-conforming, often fraught with inconsistencies such as gaps and overlaps, and therefore incompatible with downstream applications such as finite element analysis, optimization and manufacturing [36]. This paper has focused on a combination of techniques that, ultimately, should alleviate some of the complications arising from trim. The methodology is based on framefield guided global parameterization and watertight spline representations.

Figure 17: Several mode shapes of the bracket corresponding to the refined watertight NURBS model shown in Figure 15d. The color highlights maximum displacements.

<!-- image -->

Figure 18: Several mode shapes of the bracket corresponding to the refined watertight NURBS model shown in Figure 16d. The color highlights maximum displacements.

<!-- image -->

415

420

425

430

435

440

## 7.1. Contributions

Frame-field guided parameterization methods offer the kind of high quality meshes necessary within the context of computer aided design. These methods, however, suffer from several theoretical and practical problems leading to lack of robustness. Typically, these methods involve integer constraints, which are difficult and expensive to solve. A more pressing issue is that the computed parameterization may have folds and other undesirable artifacts due to an input frame-field that is far from being locally integrable. Undesirable folds are especially common near parameterization singularities.

We proposed specialized constraints that led to well defined behavior near parameterization singularities by locally incorporating properties of an analytical solution. The computed parameterization is guaranteed to be inversion-free within the constrained neighborhood of the singularity. Consequently, all separatrices are uniquely defined and can be successfully traced away from the singularity. This significantly simplifies the quad-layout extraction process, alleviating the need for complicated post-processing as proposed in [21].

These and other linear constraints were build into a reduced basis that spans a configuration space of feasible parameterization functions. The enabling technique used is a new null-space method that, besides satisfying the specified constraints exactly, is optimally sparse. Consequently, the sparsity of the original problem is preserved, thereby enabling fast solution techniques such as Cholesky and the Conjugate Gradient method. The proposed algorithm is simpler to implement and leads to sparser null-spaces bases than the approach developed in [9].

Several criteria were discussed to reconstruct spline parameterizations on coarse quadrilateral patch layouts obtained from the global parameterization method. We used rational cubic B´ ezier patches combined with Coon's patch interpolation to rebuild watertight and conforming NURBS parameterizations of the trimmed B-reps. Finally, we used LS-Dyna to perform isogeometric shell modal analysis applied to several of the re-parameterized NURBS models in order to illustrate their analysis suitability.

## 7.2. Directions for future work

The following list addresses limitations of the proposed technology and provides recommendations for future work:

445

450

455

460

465

- (i) The proposed methodology assumes that a valid triangulation can be obtained directly from the B-rep geometry. We found that current CAD systems do not have all the necessary tools to generate valid triangulations of B-rep geometry for complex models. Specifically, in the setting where B-rep models have defects that exceed the tolerance of the CAD system it is generally not possible to obtain a topologically valid triangulation without significant manual intervention. Recent work on robust triangulation and tetrahedral meshing based on the winding number could prove useful here [25, 26].
- (ii) The minimization problem in (2) does not lead to robust results when the frame-field is far from being curl-free or locally integrable . There exists current research into methods that produce integrable frame-fields, see [18, 44, 59]. Another direction would be to use only the directional information of the frame-field, not its magnitude, see e.g. [38].
- (iii) In this work we used manual constraints to line up singularities. While this avoided the use of integer constraints, it would be good to have a means of automatically selecting such constraints. The dual strip weaving method proposed in [11, 12] could provide a means of achieving this without the need for integer variables.
- (iv) Models with different levels of detail require areas with local refinement. Hence, instead of a quadrilateral mesh or layout the objective could be a T-mesh. A few works provide a first approach in this direction [14, 38].
- (v) Our approach to spline fitting used C 0 NURBS patches to rebuild a B-rep. It is cumbersome to maintain geometric continuity between patches unless continuity is build into the basis. Technologies such as T-splines remedy these issues. Future work includes a combination of item (iv) above and technologies such as T-splines, together with more advanced fitting techniques.

## Acknowledgments

T.J.R. Hughes and R.R. Hiemstra were partially supported by the National Science Foundation Industry/University Cooperative Research Center (IUCRC) for Efficient Vehicles and Sustainable

- Transportation Systems (EV-STS), and the United States Army CCDC Ground Vehicle Systems 470 Center (TARDEC/NSF Project # 1650483 AMD 2). Kendrick Shepherd was partially supported by the National Science Foundation Graduate Research Fellowship Program under Grant No. DGE1610403. Any opinions, findings, and conclusions or recommendations expressed in this material are those of the authors and do not necessarily reflect the views of the National Science Foundation.

## References 475

- [1] Benson, D., Bazilevs, Y., Hsu, M.-C., and Hughes, T. A large deformation, rotationfree, isogeometric shell. Computer Methods in Applied Mechanics and Engineering 200 , 13-16 (2011), 1367-1378.

480

485

490

- [2] Benson, D. J., Bazilevs, Y., Hsu, M. C., and Hughes, T. J. R. Isogeometric shell analysis: the Reissner-Mindlin shell. Computer Methods in Applied Mechanics and Engineering 199 , 5-8 (2010), 276-289.
- [3] Benzi, M., Golub, G. H., and Liesen, J. Numerical solution of saddle point problems. Acta numerica 14 (2005), 1-137.
- [4] Blacker, T. D., and Stephenson, M. B. Paving: A new approach to automated quadrilateral mesh generation. Int J Numer Methods Eng 32 , 4 (1991), 811-847.
- [5] Boggs, P. T., Althsuler, A. E. E., Larzelere, A. R. E. E., Walsh, E. J., Clay, R. L., and Hardwick, M. F. S. N. L. DART system analysis. Tech. rep., Sandia National Laboratories, 2005.
- [6] Bommes, D., Campen, M., Ebke, H.-C., Alliez, P., and Kobbelt, L. Integer-grid maps for reliable quad meshing. ACM Transactions on Graphics (TOG) 32 , 4 (2013), 98.
- [7] Bommes, D., L´ evy, B., Pietroni, N., Puppo, E., Silva, C., Tarini, M., and Zorin, D. Quad-mesh generation and processing: A survey. In Computer Graphics Forum (2013), vol. 32, Wiley Online Library, pp. 51-76.

495

500

- [8] Bommes, D., Zimmer, H., and Kobbelt, L. Mixed-integer quadrangulation. ACM Transactions On Graphics (TOG) 28 , 3 (2009), 1-10.
- [9] Bommes, D., Zimmer, H., and Kobbelt, L. Practical mixed-integer optimization for geometry processing. In International Conference on Curves and Surfaces (2010), Springer, pp. 193-206.
- [10] Botsch, M., Kobbelt, L., Pauly, M., Alliez, P., and L´ evy, B. Polygon mesh processing . AK Peters/CRC Press, 2010.
- [11] Campen, M., Bommes, D., and Kobbelt, L. Dual loops meshing: quality quad layouts on manifolds. ACM Transactions on Graphics (TOG) 31 , 4 (2012), 110.
- [12] Campen, M., and Kobbelt, L. Dual strip weaving: interactive design of quad layouts using elastica strips. ACM Transactions on Graphics (TOG) 33 , 6 (2014), 183.
- [13] Campen, M., Shen, H., Zhou, J., and Zorin, D. Seamless parametrization with arbitrary 505 cones for arbitrary genus. ACM Trans. Graph. 39 , 1 (Dec. 2019).
- [14] Campen, M., and Zorin, D. Similarity maps and field-guided t-splines: a perfect couple. ACM Transactions on Graphics (TOG) 36 , 4 (2017), 1-16.

510

- [15] Cohen, E., Lyche, T., and Riesenfeld, R. F. MCAD: Key historical developments. Computer Methods in Applied Mechanics and Engineering 199 , 5-8 (2010), 224-228.
- [16] Crane, K., Desbrun, M., and Schr¨ oder, P. Trivial connections on discrete surfaces. In Computer Graphics Forum (2010), vol. 29, Wiley Online Library, pp. 1525-1533.
- [17] De Berg, M., Van Kreveld, M., Overmars, M., and Schwarzkopf, O. Computational geometry. In Computational geometry . Springer, 1997, pp. 1-17.
- [18] Diamanti, O., Vaxman, A., Panozzo, D., and Sorkine-Hornung, O. Integrable 515 polyvector fields. ACM Transactions on Graphics (TOG) 34 , 4 (2015), 38.
- [19] Dijkstra, E. W. A note on two problems in connexion with graphs. Numer. Math. 1 , 1 (Dec. 1959), 269-271.

520

525

530

- [20] Dong, S., Bremer, P.-T., Garland, M., Pascucci, V., and Hart, J. C. Spectral surface quadrangulation. In ACM SIGGRAPH 2006 Papers (New York, NY, USA, 2006), SIGGRAPH '06, Association for Computing Machinery, p. 1057-1066.
- [21] Ebke, H.-C., Bommes, D., Campen, M., and Kobbelt, L. Qex: robust quad mesh extraction. ACM Transactions on Graphics (TOG) 32 , 6 (2013), 168.
- [22] Fang, X., Bao, H., Tong, Y., Desbrun, M., and Huang, J. Quadrangulation through morse-parameterization hybridization. ACM Trans. Graph. 37 , 4 (July 2018).
- [23] Gu, X., and Yau, S.-T. Global Conformal Surface Parameterization. In Eurographics Symposium on Geometry Processing (2003), L. Kobbelt, P. Schroeder, and H. Hoppe, Eds., The Eurographics Association.
- [24] Hiemstra, R. R., Hughes, T., Manni, C., Speleers, H., and Toshniwal, D. A Tchebycheffian extension of multi-degree B-splines: Algorithmic computation and properties. SIAM Journal on Numerical Analysis (Submitted) (2019).
- [25] Hu, Y., Schneider, T., Gao, X., Zhou, Q., Jacobson, A., Zorin, D., and Panozzo, D. Triwild: robust triangulation with curve constraints. ACM Transactions on Graphics (TOG) 38 , 4 (2019), 52.
- [26] Hu, Y., Zhou, Q., Gao, X., Jacobson, A., Zorin, D., and Panozzo, D. Tetrahedral 535 meshing in the wild. ACM Trans. Graph. 37 , 4 (July 2018), 60:1-60:14.
- [27] Huang, J., Zhang, M., Ma, J., Liu, X., Kobbelt, L., and Bao, H. Spectral quadrangulation with orientation and alignment control. In ACM SIGGRAPH Asia 2008 Papers (New York, NY, USA, 2008), SIGGRAPH Asia '08, Association for Computing Machinery.
- [28] Hughes, T., Cottrell, J., and Bazilevs, Y. Isogeometric analysis: CAD, finite elements, 540 NURBS, exact geometry and mesh refinement. Computer methods in applied mechanics and engineering 194 , 39-41 (2005), 4135-4195.
- [29] Jin, M., Gu, X., He, Y., and Wang, Y. Conformal Geometry: Computational Algorithms and Engineering Applications . Springer, 2018.

- [30] K¨ alberer, F., Nieser, M., and Polthier, K. Quadcover-surface parameterization us545 ing branched coverings. In Computer graphics forum (2007), vol. 26, Wiley Online Library, pp. 375-384.
- [31] Kasik, D. J., Buxton, W., and Ferguson, D. R. Ten CAD challenges. IEEE Computer Graphics and Applications 25 (2005), 81-92.
- [32] Kn¨ oppel, F., Crane, K., Pinkall, U., and Schr¨ oder, P. Globally optimal direction 550 fields. ACM Transactions on Graphics (ToG) 32 , 4 (2013), 59.
- [33] LaCourse, D. E. Handbook of solid modeling . McGraw-Hill, Inc., 1995, pp. 6.17-6.18, 6.24.

555

560

565

- [34] Lei, N., Zheng, X., Jiang, J., Lin, Y.-Y., and Gu, D. X. Quadrilateral and hexahedral mesh generation based on surface foliation theory. Computer Methods in Applied Mechanics and Engineering 316 (2017), 758-781.
- [35] Lei, N., Zheng, X., Luo, Z., and Gu, D. X. Quadrilateral and hexahedral mesh generation based on surface foliation theory II. Computer Methods in Applied Mechanics and Engineering 321 (2017), 406-426.
- [36] Marussig, B., and Hughes, T. J. A review of trimming in isogeometric analysis: challenges, data exchange and simulation aspects. Archives of computational methods in engineering 25 , 4 (2018), 1059-1127.
- [37] Myles, A., Pietroni, N., Kovacs, D., and Zorin, D. Feature-aligned t-meshes. ACM Trans. Graph. 29 , 4 (July 2010).
- [38] Myles, A., Pietroni, N., and Zorin, D. Robust field-aligned global parametrization. ACM Transactions on Graphics (TOG) 33 , 4 (2014), 135.
- [39] Owen, S. J., Staten, M. L., Canann, S. A., and Saigal, S. Q-Morph: an indirect approach to advancing front quad meshing. Int J Numer Methods Eng 44 , 9 (1999), 1317-1340.

570

575

- [40] Panozzo, D., Puppo, E., Tarini, M., and Sorkine-Hornung, O. Frame fields: Anisotropic and non-orthogonal cross fields. ACM Transactions on Graphics (TOG) 33 , 4 (2014), 134.
- [41] Piegl, L., and Tiller, W. The NURBS book . Springer Verlag, 1997.
- [42] Piegl, L., and Tiller, W. The NURBS book . Springer Science &amp;amp; Business Media, 2012.
- [43] Piegl, L. A. Ten challenges in computer-aided design. Computer-aided design 37 , 4 (2005), 461-470.
- [44] Ray, N., Li, W. C., L´ evy, B., Sheffer, A., and Alliez, P. Periodic global parameterization. ACM Trans. Graph. 25 , 4 (October 2006), 1460-1485.
- [45] Ray, N., Vallet, B., Li, W. C., and L´ evy, B. N-symmetry direction field design. ACM Transactions on Graphics (TOG) 27 , 2 (2008), 10.
- [46] Riesenfeld, R. F., Haimes, R., and Cohen, E. Initiating a CAD renaissance: Multidis580 ciplinary analysis driven design: Framework for a new generation of advanced computational design, engineering and manufacturing environments. Computer Methods in Applied Mechanics and Engineering 284 (2015), 1054-1072.
- [47] Sederberg, T. W., Anderson, D. C., and Goldman, R. N. Implicit representation of parametric curves and surfaces. Computer Vision, Graphics, and Image Processing 28 , 1 585 (1984), 72 - 84.
- [48] Sederberg, T. W., Finnigan, G. T., Li, X., Lin, H., and Ipson, H. Watertight trimmed nurbs. ACM Trans. Graph. 27 , 3 (Aug. 2008), 79:1-79:8.

590

- [49] Sederberg, T. W., Zheng, J., Bakenov, A., and Nasri, A. T-splines and t-nurccs. ACM Trans. Graph. 22 , 3 (July 2003), 477-484.
- [50] Soliman, Y., Slepˇ cev, D., and Crane, K. Optimal cone singularities for conformal flattening. ACM Trans. Graph. 37 , 4 (July 2018).

- [51] Stroud, I. Boundary representation modelling techniques . Springer Science &amp; Business Media, 2006.
- [52] Thomas, D. C., Scott, M. A., Evans, J. A., Tew, K., and Evans, E. J. B´ ezier 595 projection: a unified approach for local projection and quadrature-free refinement and coarsening of nurbs and t-splines with particular application to isogeometric design and analysis. Computer Methods in Applied Mechanics and Engineering 284 (2015), 55-105.
- [53] Tong, Y., Alliez, P., Cohen-Steiner, D., and Desbrun, M. Designing quadrangulations with discrete harmonic forms. In Proceedings of the Fourth Eurographics Symposium on 600 Geometry Processing (2006), SGP '06, Eurographics Association, pp. 201-210.
- [54] Toshniwal, D., Speleers, H., Hiemstra, R. R., and Hughes, T. J. Multi-degree smooth polar splines: A framework for geometric modeling and isogeometric analysis. Computer Methods in Applied Mechanics and Engineering 316 (2017), 1005-1061.
- [55] Toshniwal, D., Speleers, H., Hiemstra, R. R., Manni, C., and Hughes, T. J. 605 Multi-degree b-splines: Algorithmic computation and properties. Computer Aided Geometric Design 76 (2020), 101792.

610

- [56] Toshniwal, D., Speleers, H., and Hughes, T. J. Smooth cubic spline spaces on unstructured quadrilateral meshes with particular emphasis on extraordinary points: Geometric design and isogeometric analysis considerations. Computer Methods in Applied Mechanics and Engineering 327 (2017), 411-458.
- [57] Urick, B., Marussig, B., Cohen, E., Crawford, R. H., Hughes, T. J., and Riesenfeld, R. F. Watertight boolean operations: A framework for creating cad-compatible gap-free editable solid models. Computer-Aided Design 115 (2019), 147 - 160.
- [58] Vaxman, A., Campen, M., Diamanti, O., Panozzo, D., Bommes, D., Hildebrandt, 615 K., and Ben-Chen, M. Directional field synthesis, design, and processing. In Computer Graphics Forum (2016), vol. 35, Wiley Online Library, pp. 545-572.

- [59] Zhang, M., Huang, J., Liu, X., and Bao, H. A wave-based anisotropic quadrangulation method. ACM Trans. Graph. 29 , 4 (July 2010), 118:1-118:8.