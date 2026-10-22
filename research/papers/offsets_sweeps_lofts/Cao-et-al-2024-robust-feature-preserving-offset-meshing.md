## Robust and Feature-Preserving Offset Meshing

HONGYI CAO, Hangzhou Dianzi University, China

GANG XU*, RENSHU GU, JINLAN XU, Hangzhou Dianzi University, China

XIAOYU ZHANG, Beijing Institute of Spacecraft System Engineering, China

TIMON RABCZUK, Institute of Structural Mechanics, Germany

YUZHE LUO, State Key Laboratory of CAD&amp;CG, Zhejiang University, China and LightSpeed Studios, USA XIFENG GAO*, LightSpeed Studios, USA

𝑙

Fig. 1. For a 3D mesh with arbitrarily complex geometry and topology, our approach can robustly produce its inward and outward offset surfaces, according to user desired either uniform or varying offset distances. We assign the non-uniform offset distance to each face of the input by linearly interpolating 0.001% and 0.5% of the diagonal length of the input's bounding box. The top row shows the full views of the meshes and the bottom is their corresponding cut views.

<!-- image -->

We introduce a novel offset meshing approach that can robustly handle a 3D surface mesh with an arbitrary geometry and topology configurations, while nicely capturing the sharp features on the original input for both inward and outward offsets. Compared to the existing approaches focusing on constantradius offset, to the best of our knowledge, we propose the first-ever solution for mitered offset that can well preserve sharp features. Our method is designed based on several core principals: 1) explicitly generating the offset vertices and triangles with feature-capturing energy and constraints; 2) prioritizing the generation of the offset geometry before establishing its connectivity, 3) employing exact algorithms in critical pipeline steps for robustness, balancing the use of floating-point computations for efficiency, 4)

Authors' addresses: Hongyi Cao, hyc@hdu.edu.cn, Hangzhou Dianzi University, China; Gang Xu*, Renshu Gu, Jinlan Xu, Hangzhou Dianzi University, 2# Street 1158, Hangzhou, Zhejiang, 310018, China; Xiaoyu Zhang, Beijing Institute of Spacecraft System Engineering, Beijing, China; Timon Rabczuk, Institute of Structural Mechanics, BauhausUniversität Weimar, Germany; Yuzhe Luo, yzluo@zju.edu.cn, State Key Laboratory of CAD&amp;CG, Zhejiang University, China and LightSpeed Studios, USA; Xifeng Gao*, LightSpeed Studios, Bellevue, Washington, USA, 98004.

Permission to make digital or hard copies of all or part of this work for personal or classroom use is granted without fee provided that copies are not made or distributed for profit or commercial advantage and that copies bear this notice and the full citation on the first page. Copyrights for components of this work owned by others than ACM must be honored. Abstracting with credit is permitted. To copy otherwise, or republish, to post on servers or to redistribute to lists, requires prior specific permission and/or a fee. Request permissions from permissions@acm.org.

© 2024 Association for Computing Machinery.

XXXX-XXXX/2024/12-ART $15.00

https://doi.org/10.1145/nnnnnnn.nnnnnnn applying various conservative speed up strategies including early reject noncontributing computations to the final output. Our approach further uniquely supports variable offset distances on input surface elements, offering a wider range practical applications compared to conventional methods.

We have evaluated our method on a subset of Thinkgi10K, containing models with diverse topological and geometric complexities created by practitioners in various fields. Our results demonstrate the superiority of our approach over current state-of-the-art methods in terms of element count, feature preservation, and non-uniform offset distances of the resulting offset mesh surfaces, marking a significant advancement in the field.

CCS Concepts: · Computing methodologies → Modeling and simulation .

Additional Key Words and Phrases: Offset Surface, Mesh Repair, Variable Offsets

## ACMReference Format:

Hongyi Cao, Gang Xu*, Renshu Gu, Jinlan Xu, Xiaoyu Zhang, Timon Rabczuk, Yuzhe Luo, and Xifeng Gao*. 2024. Robust and Feature-Preserving Offset Meshing. 1, 1 (December 2024), 15 pages. https://doi.org/10.1145/nnnnnnn. nnnnnnn

## 1 INTRODUCTION

Offset mesh generation, which involves creating a parallel surface with a specific distance from a given shape, holds a place of critical importance in geometric modeling and mesh processing. This technique is fundamental in a variety of applications, including but not limited to computer-aided design and engineering, real-time rendering and animation, robotics, medical imaging, architectural design [Maekawa 1999a; Pham 1992]. For example, it is instrumental for designing mechanical parts with specific thickness requirements, such as gears and casings. In the world of animation and 3D modeling, offset surfaces enable the creation of intricate and realistic characters and environments, providing the necessary depth and complexity.

Manyprior works are proposed to generate offset surfaces for input meshes, such as explicit offsetting following normal directions [Jung et al. 2004], implicit distance function based iso-surfacing [Chen

<!-- image -->

et al. 2023; Zint et al. 2023], Minkowski Sum based boolean approach [Hachenberger 2009], and offsetting by ray-reps based method [Chen et al. 2019b]. However, differently from the well-studied curve offset research in 2D [Park and Chung 2003; Tiller and Hanson 1984; Vatti 1992] that have robust solutions for both constant radius and mitered offsets (see the inset), to the best of our knowledge, none of existing approaches in 3D can generate offset meshes in the mitered manner. Moreover, the methods for even constant radius offset typically suffer one or more of the following critical issues that hinder their practical usage: 1) lack of robustness in dealing with 'dirty' data that may have open boundaries, non-manifold vertices and edges, and self-intersections, which is common in scanned or manmade models; 2) struggle to maintain the fidelity of the original shape, especially in handling complex geometries with sharp edges and intricate details, resulting in undesired artifacts in the offset surface; 3) computational inefficiency when the offset distance is small, which can often jointly pose memory issues. These issues collectively underscore the need for an innovative solution that can address the complexities of modern geometric shapes while ensuring computational efficiency and geometry fidelity.

In this work, we propose a new explicit surface offset generation method that can address the aforementioned issues. The robustness of our approach lies in that we make no assumptions of the input mesh, such as its manifoldness, watertightness, absence of self-intersections and degeneracies, etc, reformulating the offset meshing problem and algorithm design accordingly. We further employ exact algorithmic for robust geometric computations. The feature preservation is ensured through first redefining the offset distance from traditionally employed point-to-point to point-to-plane, and then generating the offset surface geometry that can capture sharp features by solving local quadratic energies and dynamic programming. While employing the rational number representation for exact computations, we strive to be efficient by designing several acceleration strategies including parallelization through spatial domain decomposition and dynamic programming for early rejection of the expensive intersection computations.

Our solution has the additional advantages over existing approaches: 1) the resulting offset mesh is similar with the input, from both the number of triangles and connectivity aspects; 2) we support variable offset distances over different surface regions, enabling broader applications than conventional offset methods.

We empirically compare our approach against state-of-the-arts on a subset of the Thingi10K dataset [Zhou and Jacobson 2016]. Our method exhibits a significant improvement in terms of feature preservation, robustness, and element count at the same time. All the data shown in the paper can be found in the attached material, including a video to show their visual quality.

## 2 RELATED WORK

We briefly review prominent methods related to offset meshing, which are based on direct offsetting , distance field , Minkowski sum , medial axis and skeleton , and ray casting .

Direct Offsetting. A prominent method proposed by [Jung et al. 2004] offsets based on vertex normal direction. Although efficient, it struggles with holes in complex CAD models, leading to offset meshes that can be defective, particularly in scenarios involving self-intersections. The precision issues arising from floating-point operations in self-intersections have been addressed using infinite precision operations [Campen and Kobbelt 2010], but results occasionally produce twisted meshes. Offsetting surfaces with polynomials or B-splines is explored in [Maekawa 1999b]. However, subsequent intersection operations are intricate. Another approach calculates offset positions based on distance and uniform distribution, followed by point cloud reconstruction [Meng et al. 2018].

Distance Field. These methods are popular in modern 3D printing [Brunton and Rmaileh 2021]. Generally, they require resampling to generate the final mesh, which can compromise geometric features, especially in detailed meshes [Wang and Chen 2013]. Challenges also arise from the grid density, preservation of sharp features, and computational efficiency. Some attempts, such as [Kobbelt et al. 2001], have tackled the ambiguity of the Dual Contouring and Marching Cube, but they tend to excel mostly with CAD models. In the study by [Liu and Wang 2010], there are some improvements in computational efficiency. The method introduced in [Pavić and Kobbelt 2008] tries to preserve sharp features, but it involves high grid density and computational expenses. An Octree-based method is proposed in the study by [Zint et al. 2023], which requires the input to be degenerate-free and may output offsets with self-intersections. While the feature-preserving approach introduced in [Chen et al. 2023] ensures nice geometry and topology properties of iso-surfaces, it cannot handle offset distances smaller than the grid size which is a common shortcoming shared all distance field based offset extraction methods.

Among the vast offset generation literature, we realize only one work [Chen et al. 2019a] tried to address the non-uniform offset problem based on dual-contouring [Ju et al. 2002]. However, the approach would require an impractical grid resolution at regions with a small offset distance for tolerable offset reconstruction. More importantly, without much details of how the interpolation is performed for different offset distances within a grid cell, we infer that it either is computationally expensive for proper in/out sign determinations, or has discontinuity issues leading to bumpy artifacts. Moreover, their generated offset meshes would often have self-intersections, even if the input is both topologically simple and geometrically high quality.

Minkowski Sum Method. The Minkowski sum offers a solution for mesh offsetting by calculating the sum of mesh and sphere polygons [Rossignac and Requicha 1986]. An advanced method in [Hachenberger 2009] computes the exact 3D Minkowski sum of non-convex polyhedra by decomposing them into convex parts. Despite its robustness [Project 2023], the method is slow for complex CAD models and struggles with variable thickness offsets and sharp feature preservation.

Skeleton-Based Methods. Skeletal meshes are prevalent in geometry processing [Tagliasacchi et al. 2016]. The mesh model can be represented by medial axes and spheres [Amenta et al. 2001; Sun et al. 2015]. There are techniques to generate skeleton meshes [Lam et al. 1992; Li et al. 2015]. However, they usually cannot handle general models with open boundaries, self-intersections, etc. There have been efforts to simplify medial axes [Sun et al. 2013]. Offsetting can be achieved by adjusting the radius of the balls on the skeleton. Still, these methods often fall short with models with sharp features.

Ray-Based Methods. This approach [Chen et al. 2019b; Wang and Manocha 2013] is rooted in the dexel buffer structure [Van Hook 1986]. A recent algorithm in [Chen et al. 2019b] offers an efficient parallelized method using rays and voxels to compute mesh offsets, bypassing the need for distance field computation. Nonetheless, due to the grid's voxel-like structure, a high density is essential for accurate shape representation, leading to increased computational costs and dense mesh outputs. The approach also mainly considers a consistent offset, further steps have to be designed for the mesh representation conversion.

## 3 METHOD

In this section, we provide our problem statement, give the pipeline overview, explain each step in detail, and introduce the performance improvement strategies.

Problem Statement: The input of our approach contains a 3D triangle mesh 𝑀 I with an arbitrary geometry and topology configuration, an integer variable 𝑠 ∈ {-1 , 1 } to indicate the offsetting direction, and an offset scalar 𝑑 𝑖 per 𝑇 𝑖 of 𝑀 I where 𝑑 𝑖 could vary for different triangles. While locally to a triangle, 𝑠 = 1 means the offset direction has a positive product with the triangle's normal and 𝑠 = -1 is for the opposite direction, globally we employ the generalized winding number [Jacobson et al. 2013] for inward and outward checks to robustly handle 𝑀 I with possibly gaps, duplicated elements, etc.

Our goal is to generate an offset mesh 𝑀 O that satisfies three qualitative requirements. First, 𝑀 O should have the offset distance to 𝑀 I satisfying user desired value. Second, 𝑀 O needs to have nice geometry and topology properties to enable the easy design of automatic downstream algorithms. Third, 𝑀 O should contain as few as possible elements such as triangles, allowing efficient computations for the various applications of offset. While hard to properly perform a quantitative measure, we further require 𝑀 O to capture the geometry feature of 𝑀 I as much as possible, especially for prominent sharp features since we aim for reproducing the mitered offset effect.

Fig. 2. Pipeline : starting from a triangle mesh 𝑀 I (left most), our approach first generates its vertices' offset points (second to the left), then builds an offset polyhedron for each of its triangle, edge and vertex (middle). After that, we convert the polyhedra set to an intersection-free triangle soup 𝑀 C by filtering out those triangles not part of 𝑀 O (second to the right), Along with some acceleration measures, this approach involves selecting only a subset of triangles from the polyhedra set for computation, which may result in the creation of some holes. Finally, by detecting the boundaries of the holes, we retrieve the triangles that were excluded in the previous step, Then we construct the connectivity of 𝑀 O (right).

<!-- image -->

Method Overview: As illustrated by Fig. 2, we tackle the surface offset problem by first offsetting each vertex of 𝑀 I to one or more corresponding points through the solving of a point-to-plane constrained quadratic energy (Section 3.1, Fig. 2 second to the left), and then constructing a polyhedron representing the local offset volume corresponding to each vertex, edge, and triangle of 𝑀 I Section 3.2, Fig. 2 middle). Using a set of acceleration strategies, we then perform boolean operations of the offset volumes to obtain a triangle soup, 𝑀 C, representing the geometry of 𝑀 O (Section 3.3, Fig. 2 second to the right), which are finally connected to complete the generation of user desired 𝑀 O (Section 3.4, Fig. 2 right).

## 3.1 Vertex Offset

This step is to generate the offset points for each vertex 𝑉 𝑖 ∈ 𝑀 I . As illustrated in the inset, 𝑉 𝑖 might correspond to one or more points depending on different local configurations, such as nonmanifoldness, saddle region etc. For easy explanation, we first introduce the simple but effective linear constrained quadratic optimization solution when 𝑉 𝑖 has only one offset point, and then describe how to tackle the general sce-

nario, i.e. multiple offset points, via

dynamic programming.

linear constrained quadratic optimization. Let 𝑂 𝑖 be 𝑉 𝑖 's unique offset, we assume the local region of 𝑉 𝑖 is formed by a triangle set T 𝑖 where ev-

A vertex (blue) of 𝑀 I can have one or more offset points (red).

<!-- image -->

ery triangle 𝑇 𝑗 ∈ T 𝑖 is equipped with an offset distance 𝑑 𝑗 . Denote 𝑁 = |T 𝑖 | . Note that, since we assume the input could have an arbitrary topology, we set T 𝑖 to be the set of triangles that 𝑉 𝑖 directly adjacent to if the boundary of this triangle set contains a simple circle topology. Otherwise, we consider T 𝑖 as the set of all triangles with an 𝜖 distance from 𝑉 𝑖 . We set 𝜖 = 10 -5 𝑙 by default, where 𝑙 is the length of the diagonal of 𝑀 I 's bounding box.

By requiring the point-to-plane distance from 𝑂 𝑖 to 𝑇 𝑗 ∈ T 𝑖 to be 𝑑 𝑗 , we can compute 𝑂 𝑖 by solving the following optimization:

<!-- formula-not-decoded -->

where P 𝑗 = [ 𝑎 𝑏 𝑐 𝑑 ] T represents the plane of triangle 𝑇 𝑗 defined by the equation 𝑎𝑥 + 𝑏𝑦 + 𝑐𝑧 + 𝑑 = 0 and 𝑎 2 + 𝑏 2 + 𝑐 2 = 1. The optimization term | | 𝑂 𝑖 -𝑉 𝑖 | | 2 is introduced to pick the position of 𝑂 𝑖 nearest to 𝑉 𝑖 when the solution space is a plane or a line. While Equation 2 may be solved using OSQP solver [Stellato et al. 2020], due to numerical precision issues, we solve it as a two step process. We first solve an unconstrained quadratic problem:

<!-- formula-not-decoded -->

The role of 𝜆 is to ensure that, in cases where there are infinitely many solutions satisfying the point-to-plane distances, an optimal solution can be selected, while minimizing its impact on other scenarios. Consequently, it is necessary to set it to a very small number. By default, we set 𝜆 to 10 -9 . Subsequently, this expression can be solved using the least squares method. We employ the QR decomposition from the Eigen library for the solution.

Wethenperform a distance check for each plane to ensure | P T 𝑗 𝑂 𝑖 -𝑑 𝑗 | ≤ 𝛼 , where 𝛼 is a parameter to control how much the user desired point to plane distance is satisfied. Due to the presence of 𝜆 , the distances between the solved point position and the planes will always be slightly off from the desired values. Therefore, 𝛼 cannot be set to zero but must be assigned a small value. We set 𝛼 = 10 -6 by default. Since there could be cases that cannot satisfy the distance check for all triangles in T 𝑖 , we then propose to address this issue by generating multiple offsets for a vertex as detailed in the next paragraph.

Dynamic Programming. Whenthere is no solution found for Equation 3, we compute multiple offset points. We propose to solve this issue by dividing the triangle set T 𝑖 into 𝐾 subsets, denoted by Q = {T 0 𝑖 , T 1 𝑖 , · · · } where |Q| = 𝐾 , and we compute an offset point 𝑂 𝑘 𝑖 for each subset T 𝑘 𝑖 . Denote R as the offset point set, i.e. { 𝑂 0 𝑖 , 𝑂 1 𝑖 , · · · } and |R| = 𝐾 . Therefore, we need to solve the following problem:

<!-- formula-not-decoded -->

The challenge arises for deciding the 𝐾 subsets while ensuring the point-to-plane distance constraint for the planes of T 𝑘 𝑖 for each 𝑂 𝑘 𝑖 . While we can simply decompose T 𝑖 based on the similarity of the normal of the triangles, solution of 𝑂 𝑘 𝑖 is still not guaranteed to exist and normal could be wrongly computed when floating point precision is involved. Instead, we propose a dynamic programming strategy to solve Q , R as detailed below, where Algorithm 1 is the pseudo-code.

Given the 𝑁 triangles in T 𝑖 , a subset could have 2 𝑁 possible configurations. We encode each subset using the binary representation with length 𝑁 . For example, 0 𝑏 010101 represents the subset of { 𝑇 0 , 𝑇 2 , 𝑇 4 } for 𝑁 = 6, while 0 𝑏 100100 is the subset of { 𝑇 2 , 𝑇 5 } . We use 𝐷 𝑜 to store the energy values of Equation 2 when solving an offset vertex 𝑂 𝑘 𝑖 for the group of triangles T 𝑘 𝑖 represented in the aforementioned binary format 𝑥 , and 𝐷 𝑐 to store the coordinates of the corresponding offset vertex 𝑂 𝑘 𝑖 . As shown in Algorithm 1, we employ a recursive scheme (lines 2-12) to find the best group decomposition, where for each level of the recursion we decompose a set 𝑥 into two subsets ˜ 𝑥 and 𝑥 -˜ 𝑥 if 𝑥 cannot be solved by Equation 2. Here, the union of the triangles represented by ˜ 𝑥 and 𝑥 -˜ 𝑥 is the same as the triangle set of 𝑥 . In this case, we use the an array of scalars, 𝐷 𝑠 , to record the best decomposition for 𝑥 , i.e. 𝐷 𝑠 [ 𝑥 ] = ˜ 𝑥 . From which, we can retrieve all the decomposed subsets of T 𝑖 , as shown by lines 15-20 of Algorithm 1. Note that, the dimension of 𝐷 𝑜 , 𝐷 𝑐 , and 𝐷 𝑠 are all 2 𝑁 -1.

Fig. 4. Nine different vertex offset scenarios computed by our approach on a mesh example. Each column corresponds to one scenario, where the top and bottom rows show two different offset direct (outward and inward). It can be observed that in some sharp cases, the points generated by the inward offset of vertices may appear outside the input model. This situation does not pose any issues. The subsequent convex hull and winding number computations will exclude these points and .

<!-- image -->

The introduced dynamic programming has a computational complexity of 𝑂 ( 3 𝑘 + 2 ( 1 -2 𝑘 )) [A10 2024; Giraudo 2015]. The computation is efficient for 𝑁 ≤ 12, e.g. ≤ 0 . 01 𝑠 without any parallelization. Since the dynamic programming operates independently in each local context, it can be fully parallelized. However, if 𝑁 becomes excessively large, the computation time may grow significantly as show in Fig. 3.

To address this problem, we first classify the the planes defined by T 𝑖 into groups where the planes within a group have the similar normal. We then

Fig. 3. The two curves respectively demonstrate the computational time required to calculate a point with and without prior classification. It can be observed that when 𝐾 is particularly large, lacking this classification makes it difficult to compute the offset points within a reasonable timeframe.

<!-- image -->

compute an averaged plane for each group and consider the averaged one as the plane for all the triangles with each group.

To summarize, we compute the offsets for each vertex of 𝑀 I by firstly merging its adjacent triangles with similar normals, and then perform Algorithm 1 to solve the offset points. Fig. 4 illustrates different offset cases on a real example.

## Algorithm 1: Dynamic Programming for Vertex Offset

```
Input: 𝑉 𝑖 , T 𝑖 , 𝑑 , 𝑇 𝑗 ∈ T 𝑖 Output: Q , R 1: Set each entry of 𝐷 𝑜 , 𝐷 𝑐 , and 𝐷 𝑠 as NULL 2: Function DFS(x) : 3: if 𝐷 𝑜 [ 𝑥 ] ≠ NULL then 4: return 𝐷 𝑜 [ 𝑥 ] 5: 𝐷 𝑜 [ 𝑥 ]←∞ 6: if Eq.2( 𝑥 ) is solvable then 7: Set Eq.2( 𝑥 )'s energy and solution to 𝐷 𝑜 [ 𝑥 ] and 𝐷 𝑐 [ 𝑥 ] 8: for each ˜ 𝑥 derived from 𝑥 do 9: 𝑜 ← DFS( ˜ 𝑥 ) + DFS( 𝑥 -˜ 𝑥 ) 10: if 𝑜 < 𝐷 𝑜 [ 𝑥 ] then 11: 𝐷 𝑜 [ 𝑥 ] ← 𝑜 , 𝐷 𝑠 [ 𝑥 ] ← ˜ 𝑥 12: return 𝐷 𝑜 [ 𝑥 ] 13: DFS( 2 𝑁 -1 ) 14: ˜ Q ← ∅ , R ← ∅ , ˜ Q .𝑝𝑢𝑠ℎ ( 2 𝑁 -1 ) 15: while ˜ Q ≠ ∅ do 16: 𝑥 ← ˜ Q .𝑓 𝑟𝑜𝑛𝑡 () , ˜ Q .𝑝𝑜𝑝 () 17: if 𝐷 𝑠 [ 𝑥 ] = NULL then 18: Q .𝑖𝑛𝑠𝑒𝑟𝑡 ( 𝑥 ) , R .𝑖𝑛𝑠𝑒𝑟𝑡 ( 𝐷 𝑐 [ 𝑥 ]) 19: else 20: ˜ Q .𝑝𝑢𝑠ℎ ( 𝐷 𝑠 [ 𝑥 ]) , ˜ Q .𝑝𝑢𝑠ℎ ( 𝑥 -𝐷 𝑠 [ 𝑥 ])
```

## 3.2 Local Offset Polyhedron

After generating the offset points of 𝑀 I , this step is to generate the local offset volumes for the elements of 𝑀 I , i.e. vertices, edges, and triangles. Ideally, if there is only one offset point for each vertex, then we only need to generate the local offset volume 𝑃 𝑖 for each triangle, 𝑇 𝑖 ∈ 𝑀 I , and 𝑃 𝑖 is simply the triangle prism formed by 𝑇 𝑖 's three vertices and their corresponding offset points as illustrated in Fig. 5 left.

Fig. 5. From left to right, the diagrams respectively illustrate the construction of offset volumes from a triangle, a vertex, and an edge.

<!-- image -->

In practice, however, one vertex may have multiple offset points, the offset volume computation for 𝑀 I would be slightly involved. We tackle the problem by computing an offset polyhedron for each vertex, edge, and triangle of 𝑀 I respectively. For each element, its offset polyhedron is computed as the convex hull of a point set associated with the element. To be specific, for a vertex 𝑉 𝑖 of 𝑀 I , the point set includes 𝑉 𝑖 and all its offset points { 𝑂 0 𝑖 , 𝑂 1 𝑖 , · · · } corresponding to the neighboring triangle set T 𝑖 (Fig. 5 middle). For an edge 𝐸 𝑖 = { 𝑉 0 , 𝑉 1 } of 𝑀 I , its polyhedron is computed as the following (Fig. 5 right). Assume 𝐸 𝑖 is adjacent to a set of triangles T 𝐸 𝑖 , the point set of 𝐸 𝑖 for the convex hull computation includes 𝑉 0 , 𝑉 1, and those from 𝑉 0's and 𝑉 1's offset points satisfying the following condition: the offset point's corresponding triangle subset solved by Algorithm 1 contains any triangle of T 𝐸 𝑖 . Similarly, for a triangle 𝑇 𝑖 = { 𝑉 0 , 𝑉 1 , 𝑉 2 } of 𝑀 I , its polyhedron is the convex hull of the point set including 𝑉 0 , 𝑉 1 , 𝑉 2, and those offset points of 𝑉 0 , 𝑉 1 , 𝑉 2 as long as the offset point's corresponding triangle subset solved by Algorithm 1 contains 𝑇 𝑖 .

This strategy leads to a set of polyhedra, denoted as P , that their union forms the offset volume of 𝑀 I .

## 3.3 Geometry Extraction

After generating the offset polyhedra, this section is to compute all the triangles that constitute the geometry of 𝑀 O in three steps.

First, we convert the all triangles from P into a conforming triangle mesh 𝑀 𝐶 by resolving all the intersections, where the intersection points and line segments are taken as constraints for a subsequent Delaunay triangulation for each face of 𝑃 𝑖 ∈ P . After this step, 𝑀 𝐶 is intersection-free and contains

Aface of 𝑀 C is with one of the illustrated five types. Left: TII (blue), TIV (gray), TV(yellow); middle: TI (green); right: TIII (red).

<!-- image -->

only triangles. As illustrated in the inset, a triangle of 𝑀 𝐶 can be classified into one of the five types, i.e. enclosed by a 𝑃 𝑖 (TI ), shared by two 𝑃 𝑖 s that at different sides of the triangle (T II ), has an opposite sign to the offset direction as evaluated by 𝑀 I (TIII ), a triangle of 𝑀 I (TIV ), and a face of 𝑀 O (TV). Since T IV faces are properly tracked from the beginning, our next two steps are to obtain the triangles with type TV by filtering out those triangles belonging to the first three types.

Second, we identify and discard triangles with T I and TII through rational number represented ray intersection checks where the rays are shot from the centers of these faces along their normal directions. Both types can be easily detected by checking if there are any intersections between the rays and other polyhedra other than the ones these faces belong to.

Third, we detect faces with T III through the generalized winding number [Jacobson et al. 2013]. Since the winding number computation is not exact, when the offset distances 𝑑 are close to the numerical precision, TIII faces may be wrongly labeled, resulting redundant faces in 𝑀 O. However, accordingly to our extensive experiments, we don't observe such an issue for 𝑑 ≥ 10 -6 for both uniform and varying offsets. Note that, if the input mesh is watertight and free of self-intersections, T III faces can be filtered out completely by employing [Project 2023].

## 3.4 Topology Construction

After obtaining 𝑀 C, i.e., the triangle soup composing the geometry of 𝑀 O, in this section, we accomplish the connectivity of 𝑀 O to make it free of degenerates and intersections under exact number representations. 𝑀 O is also watertight given that the input is watertight and self-intersection-free.

We execute three steps to convert 𝑀 C to 𝑀 O. First, we merge the vertices with the same coordinates in rational numbers so that all triangles are connected but zero-area holes may still exist. Second, we perform a hole-filling to remove the zero-area gaps. Given a boundary loop, our hole-filling is done by inserting an edge at a time with two criteria: this edge has no intersection with any other triangles and the areas of the newly generated polygons are still zero, until the loop is fully triangulated. Third, we eliminate all degenerate elements generated during the hole-filling step through edge collapse and edge split without changing the geometry of 𝑀 O.

## 3.5 Speedup Strategies

We employ speedup strategies from different aspects to improve the performance of our approach as detailed below.

WeuseOpenMPtotrivially parallelize the vertex offset generation (Section 3.1) and the triangle polyhedra construction (Section 3.2). Furthermore, we spatially partition the spatial domain, which is covered by the geometry formed by extending 𝑀 I along three axes with the largest offset distance, into cubes. Each cube is associated with the list of triangles of 𝑀 I that are either fully contained or have intersections with the cube. So the computational domain of 𝑀 O is constrained within each cube. We apply this strategy for Section 3.3 and Section 3.4.

Data-structure-wise, we employ AABB-tree data-structure as much as possible for spatial search of vertex, face, and polyhedron elements, such as the T 𝑖 collection for 𝑉 𝑖 in Section 3.1, the polyhedron neighborhood search during intersection resolving, the rayintersection check and the winding number computation during in-

valid triangle filtering Based on the performance statistics, we find that the most time-consuming part is during the intersection resolving in Section 3.3 where an extensive number of exact computations [Hachenberger et al. 2007] are involved. While all intersections among the faces of the polyhedra of P are resolved by splitting the faces into intersection-free triangles, many of the resulting triangles are not part of 𝑀 𝑂 . In another words, if the entire or in Section

3.3.

While many polygons intersect with the edge in red, two polygons in blue are enough to early reject the edge from being further decomposed into TII edges.

<!-- image -->

the most region of a face will be discarded for computing 𝑀 𝑂 , then most of the computations for resolving the intersections with this face are wasted since most of the resulting triangles will be discarded as well. Based on the principal that postponing unnecessarily intersection computations as much as possible, we propose a speedup strategy of Section 3.3 with the pseudo-code given by Algorithm 2.

Assume T is the set of triangles of all polyhedra of P and T 𝑜 is the set of all triangles of 𝑀 𝑂 . We first subdivide T into two sets T 𝑐𝑢𝑟 and T 𝑙𝑎𝑡𝑒𝑟 where they contains triangles with high and low chances to be part of 𝑀 O respectively. As shown in lines 2-10 of Algorithm 2, as an initialization, T 𝑙𝑎𝑡𝑒𝑟 contains triangles with two scenarios that are both unlikely to be part of 𝑀 O: sharing a vertex with 𝑀 I and is inside (outside) 𝑀 I for 𝑠 = 1 ( 𝑠 = -1). We then obtain T 𝑜 in an iterative way by constructing the most of it using T 𝑐𝑢𝑟 (lines 12-23) and gradually refining it using T 𝑙𝑎𝑡𝑒𝑟 (lines 11 and 24-29). By doing so, the output of Algorithm 2 is ensured to be same as the brute-force approach in Section 3.3. During the construction of T 𝑜 using T 𝑐𝑢𝑟 , we identify a scenario that for a triangle of T 𝑐𝑢𝑟 that intersects with many polyhedra, a large part of it may be covered by one or several of those polyhedra. For such faces, there would be many intersection computations to produce T I and TII sub-triangles that are eventually discarded through ray-casting test. The inset illustrates a simplified scenario in 2D. Accordingly, in lines 17-22 of Algorithm 2, we propose to subdivide 𝑇 𝑖 into triangles by a subset of all its intersecting polyhedra to avoid unnecessarily more detailed intersection computations. Specifically, given a triangle 𝑇 𝑖 that has intersections with 𝑁 𝑝 ≥ 5 polyhedra denoted by the set P 𝑖 , we first subdivde 𝑇 𝑖 into four smaller triangles, i.e. 𝑇 𝑗 𝑖 with 𝑗 = { 0 , 1 , 2 , 3 } , using the midpoint subdivision. For each 𝑇 𝑗 𝑖 , we then find the polyhedron from P 𝑖 that contains 𝑇 𝑗 𝑖 the most by creating several sampling points within 𝑇 𝑗 𝑖 and checking the number of sampling points contained by the polyhedron. After that, corresponding to 𝑇 𝑗 𝑖 , there will be four polyhedra identified, 𝑃 𝑗 𝑖 with 𝑗 = { 0 , 1 , 2 , 3 } . We then perform the intersection resolving procedure in Section 3.3 for 𝑇 𝑖 and these four polyhedra to obtain a set of subdivided triangle set of ˜ T 𝑖 . After filtering out those triangles in ˜ T 𝑖 that are fully contained in any 𝑃 𝑗 𝑖 , T 𝑐𝑢𝑟 will be updated.

By experimenting on several models randomly chosen from our testing dataset, we find that the early filtering strategy helps our pipeline achieves ∼ 30x performance gain in large offset distance e.g. more than 1%.

## 4 EXPERIMENTS

Fig. 6. This figure shows the results of our algorithm for inward offsets at different distances. From left to right, the images are the input, 0 . 05% 𝑙 , 0 . 1% 𝑙 , 0 . 5% 𝑙 , 1% 𝑙 , and 5% 𝑙 .

<!-- image -->

We run all our experiments on a desktop with a 10-cores Intel processor clocked at 5.30 Ghz and 32Gb of memory. We implement our approach in C++, with CGAL [Project 2023] for exact computations. Throughout the entire paper, our offset distance parameter 𝑑 is formulated as the ratio of 𝑙 where 𝑙 is the length of the diagonal of the input's bounding box, e.g. 𝑑 = 1% represents that the offset

## Algorithm 2: Speed up of Section 3.3

```
Input: T , P Output: T 𝑜 1: Set T 𝑐𝑢𝑟 , T 𝑙𝑎𝑡𝑒𝑟 , and T 𝑜 as ∅ 2: for each 𝑇 𝑖 ∈ T do 3: if 𝑠 = 1 and Winding_number(centroid( 𝑇 𝑖 )) > 0 . 5 then 4: T 𝑙𝑎𝑡𝑒𝑟 .push( 𝑇 𝑖 ) 5: else if 𝑠 = -1 and Winding_number(centroid( 𝑇 𝑖 )) ≤ 0 . 5 then 6: T 𝑙𝑎𝑡𝑒𝑟 .push( 𝑇 𝑖 ) 7: else if ( 𝑇 𝑖 ) have a vertex on 𝑀 𝑖 then 8: T 𝑙𝑎𝑡𝑒𝑟 .push( 𝑇 𝑖 ) 9: else 10: T 𝑐𝑢𝑟 .push( 𝑇 𝑖 ) 11: while T 𝑐𝑢𝑟 ≠ ∅ do 12: T ′ 𝑐𝑢𝑟 ←∅ 13: for 𝑇 𝑖 ∈ T 𝑐𝑢𝑟 do 14: if 𝑁 𝑝 < 5 then 15: T ′ 𝑐𝑢𝑟 .push( 𝑇 𝑖 ) 16: else 17: Subdivide 𝑇 𝑖 into 𝑇 0 𝑖 , 𝑇 1 𝑖 , 𝑇 2 𝑖 , 𝑇 3 𝑖 18: Compute 𝑃 𝑗 𝑖 containing 𝑇 𝑗 𝑖 the most, 𝑗 = { 0 , 1 , 2 , 3 } 19: ˜ T 𝑖 ← resolving intersections between 𝑇 𝑖 and 𝑃 0 𝑖 , 𝑃 1 𝑖 , 𝑃 2 𝑖 , 𝑃 3 𝑖 20: for 𝑡 𝑗 ∈ ˜ T 𝑖 do 21: if 𝑡 𝑗 is not contained in 𝑃 0 𝑖 | | 𝑃 1 𝑖 | | 𝑃 2 𝑖 | | 𝑃 3 𝑖 then 22: T ′ 𝑐𝑢𝑟 .push( 𝑡 𝑗 ) 23: T 𝑜 ← perform Section 3.3 of T ′ 𝑐𝑢𝑟 and P 24: T 𝑐𝑢𝑟 ←∅ 25: B ← Boundary of T 𝑜 26: for 𝑇 𝑖 ∈ T 𝑙𝑎𝑡𝑒𝑟 do 27: if 𝑇 𝑖 intersects with B then 28: T 𝑐𝑢𝑟 .push( 𝑇 𝑖 ) 29: T 𝑙𝑎𝑡𝑒𝑟 .delete( 𝑇 𝑖 )
```

distance is actually 1% 𝑙 . Please refer to the supplementary material for more information.

## 4.1 Dataset and Metrics

Dataset. We use 100 models chosen from Thingi10K [Zhou and Jacobson 2016] as our test dataset as shown in Fig. 8. The dataset contains models with varying topology and geometry complexities, In the dataset, there are 50 models with fewer than 1000 triangles. There are 32 models with a triangle count ranging from 1000 to 5000. Additionally, there are 13 models with a triangle count between 5000 and 8000. Lastly, there are 5 models with more than 8000 triangles. 7 models are no manifold. 6 models are with open surface. The average number of genus, disconnected components intersecting triangle pairs and holes are 2.4, 1.94, 13.4 and 0.227, respectively.

·

Fig. 7. This figure shows the results of our algorithm for outward offsets at different distances. From left to right, the images are the input, 0 . 05% 𝑙 , 0 . 1% 𝑙 , 0 . 5% 𝑙 , 1% 𝑙 , and 5% 𝑙 .

<!-- image -->

Fig. 8. A visual gallery of the testing dataset.

<!-- image -->

For arbitrary input meshes, our generated offsets are guaranteed to be self-intersection-free and degenerate-free under exact representations. If the input mesh 𝑀 I is watertight and self-intersectionfree, our generated 𝑀 O is guaranteed to be watertight.

Weevaluate the geometry accuracy of our generated offset meshes by measuring how much of the distance between the offset mesh and input is different from the offset distance specified by users.

Mesh Distances. We employ the typically used point-to-point distance (Hausdorff distance) 𝐻 𝑝𝑜𝑖𝑛𝑡 to measure the mesh distance between 𝑀 O and 𝑀 I . Since we aim to generate mitered offset instead of the traditionally constant-radius offset, we also propose to measure point-to-plane distance 𝐻 𝑝𝑙𝑎𝑛𝑒 between 𝑀 O and 𝑀 I .

Given a vertex 𝑣 and a triangle mesh ˜ 𝑀 , we denote 𝑑 𝑣 ( 𝑣, ˜ 𝑀 ) as the closest Euclidean distance from 𝑣 to ˜ 𝑀 and ˜ 𝑣 ∈ ˜ 𝑀 is the point on ˜ 𝑀 closest to 𝑣 . We further denote 𝑑 𝑡 ( 𝑣, ˜ 𝑀 ) as the Euclidean distance from 𝑣 to the plane passing through ˜ 𝑣 and tangent to ˜ 𝑀 . The computations of 𝐻 𝑝𝑜𝑖𝑛𝑡 and 𝐻 𝑝𝑙𝑎𝑛𝑒 are summarized in Equation 4.

<!-- formula-not-decoded -->

𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 O , 𝑀 I ) and 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 O , 𝑀 I ) measures the average distances from 𝑀 O to 𝑀 I , while 𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 I , 𝑀 O ) and 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) are from 𝑀 I to 𝑀 O. The sampling strategy in Metro [Cignoni et al. 1998] is used for their computation. We use 𝐷 𝑝𝑜𝑖𝑛𝑡 = | 𝐻 𝑝𝑜𝑖𝑛𝑡 -𝑑 | and 𝐷 𝑝𝑙𝑎𝑛𝑒 = | 𝐻 𝑝𝑙𝑎𝑛𝑒 -𝑑 | to indicate how close of our generated mesh aligns with the user specified offset distance, where the smaller of 𝐷 𝑝𝑜𝑖𝑛𝑡 and 𝐷 𝑝𝑙𝑎𝑛𝑒 the better.

Feature Preservation. Moreover, to quantitatively measure the effectiveness of various approaches in feature preservation, we propose to compute the difference between sharp features of the input and output meshes. Specifically, based on the simple angle thresholding strategy [Gao et al. 2019], we first detect feature lines 𝑆 𝑀 I and 𝑆 𝑀 O of 𝑀 I and 𝑀 O respectively (black lines shown in Fig. 9 and Fig. 10), then we uniformly sample 𝑆 𝑀 I and 𝑆 𝑀 O . For each sample 𝑣 ∈ 𝑆 𝑀 I , we can find a closest sample ˜ 𝑣 ∈ 𝑆 𝑀 O . We denote 𝑑 𝑎𝑛𝑔𝑙𝑒 ( 𝑣, 𝑆 𝑀 O ) as the 𝑙 2 norm of the dihedral angles of the edges where 𝑣 and ˜ 𝑣 belong to. The difference between 𝑆 𝑀 I and 𝑆 𝑀 O can be calculated by

<!-- formula-not-decoded -->

where 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 I , 𝑆 𝑀 O ) and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 O , 𝑆 𝑀 I ) measure the average distances from 𝑆 𝑀 I to 𝑆 𝑀 O and from 𝑆 𝑀 O to 𝑆 𝑀 I respectively.

## 4.2 Our Results

Our method requires only a single user-specified parameter, offset distance 𝑑 , and generates the corresponding offset surface mesh that has the following unique properties.

Mitered offset. Aiming at generating the mitered offset surface, our approach provides the unique advantage over existing methods by respecting the features of the input excellently, as shown in Fig. 6, Fig. 7, Fig. 9, Fig. 10 and Fig. 17.

Unsurprisingly, as demonstrated in Fig. 11, as the offset distance gets larger, our generated surface tends to be a shape with a rectangular shape, distinct from the constant-radius offset with the tendency of being a sphere.

Non-uniform offset distance. By varying the offset distance 𝑑 for different triangles of the input, our approach can easily generate offset meshes with varying offset distances, as illustrated in Fig. 12 while maintaining the sharp features of the original meshes.

Topology. The topology of our generated mesh is similar to the input mesh, and the density of the mesh is close to or slightly greater than the input mesh as shown in Fig. 19.

Mesh Repair. Our approach can perform mesh repairs if the input mesh contains small gaps, self-intersections and internal redundant faces, etc. Fig. 13 demonstrates the repair capability of our approach on such a case. Since our approach works nicely for small offset distances, it's particularly beneficial for repairing meshes while introducing minor changes to the original mesh.

## 4.3 Comparisons

To demonstrate the robustness and effectiveness of our approach, we compare against five state-of-the-art competing approaches, i.e. [Alpha Wrap 2022], [Chen et al. 2023], [Minkowski sum 2023], [Zint et al. 2023], and [Jung et al. 2004], by batch processing the entire test dataset with varying offset distance settings, i.e. 𝑑 = 0 . 05% 𝑙 , 𝑑 = 0 . 1% 𝑙 , 𝑑 = 0 . 5% 𝑙 , 𝑑 = 1% 𝑙 and 𝑑 = 5% 𝑙 both inwardly (s = -1) and outwardly (s = 1). For outward offset, we compute the two way distances 𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 O , 𝑀 I ) , 𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 I , 𝑀 O ) , 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 O , 𝑀 I ) , 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O )) , 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 O , 𝑆 𝑀 I ) , and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 I , 𝑆 𝑀 O ) . However, for the inward offset surface, we evaluate the single way distance only, i.e. 𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 O , 𝑀 I ) , 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 O , 𝑀 I ) , and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 O , 𝑆 𝑀 I ) . The reason is that the inward offset surface may diminish as the offset distance gets large, as shown in Fig. 17, which renders the computed distances of 𝐻 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 I , 𝑀 O ) , 𝐻 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) , and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑆 𝑀 I , 𝑆 𝑀 O ) not necessarily meaningful. We set a maximum execution time of one hour for each algorithm on each model. For processing time more than an hour, we automatically terminate the process and mark it as a failure. While we elaborate the detailed comparisons below, Fig. 9 and Fig. 14 show visual comparisons for outward offset results generated by the various methods, Fig. 10 and Fig. 15 illustrate the visual differences for inward offset meshes produced by the comparing approaches, and Table 1 demonstrates all the quantitative statistics.

[Alpha Wrap 2022] can robustly generate a watertight and orientable surface triangle mesh from an arbitrary 3D geometry input, but cannot compute inward offsets. Therefore, as shown in Table 1, we compare with it for cases only when 𝑠 = 1. It has two parameters, i.e. an alpha to determine the size of untraversable cavities when refining and carving a 3D Delaunay triangulation and an offset distance to control how far of the output mesh vertices from the input. We compare our algorithm with [Alpha Wrap 2022]'s CGAL implementation, set its offset distance using 𝑑 . Its alpha parameter plays a critical role in determining the output mesh's geometry approximation of the input and the approach's computational speed. Using the same setting in the experiment section of [Alpha Wrap 2022], we set alpha = 100 for all the test in this paper. For the point-to-point distance 𝐷 𝑝𝑜𝑖𝑛𝑡 , while [Alpha Wrap 2022] performs excellently when measuring the distance from 𝑀 I to 𝑀 O, as shown in the 𝐷 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 I , 𝑀 O ) column of Table 1, our approach has generally better accuracy when computing the distance from 𝑀 O to 𝑀 I . Similar pattern happens for the point-to-plane distance 𝐷 𝑝𝑙𝑎𝑛𝑒 that our approach consistently achieves the smallest error for 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 O , 𝑀 I ) for all the offset distances, but [Alpha Wrap 2022] has the best accuracy for 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) . While this seems unexpected since 𝐷 𝑝𝑙𝑎𝑛𝑒 is designed for measuring the mitering offset distances, our approach may remove tiny dents as show in Fig. 16 that can lead to large 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) and 𝐷 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 I , 𝑀 O ) . In fact, our approach captures the sharp feature of the input excellently, achieving the best feature preservation among all the approaches for all the tested offset distances, as shown by the 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 I , 𝑀 O ) and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 O , 𝑀 I ) columns of Table 1, and Fig. 9.

Fig. 9. This figure shows an outward offset of 1% 𝑙 . The left side is the input, and the back shows the comparison results of six methods. It can be seen that our method performs better in restoring feature lines.

<!-- image -->

Fig. 10. This figure shows an inward offset of 1% 𝑙 . The left side is the input, and the back shows the comparison results of three methods. It can be seen that our method performs better in restoring feature lines.

<!-- image -->

Comparison with [Chen et al. 2023]. We compare our algorithm with [Chen et al. 2023] by directly running their provided online executable program. While [Chen et al. 2023] can robustly handle general inputs, relying on voxel discretization of the unsigned distance function and maching-cube-like linear approximation of the iso-surface, [Chen et al. 2023] can generate results only when the discretization is coarse as shown by the 𝐷 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 O , 𝑀 I ) for

Fig. 11. Display the large offset distance. It can be observed that generating an offset with a large offset distance using the original vertices may lead to creases due to the resolution limited by the number of vertices. The offset distance from left to right is 10% 𝑙 , 20% 𝑙 , 30% 𝑙 , 40% 𝑙 , 50% 𝑙 with outer direct. The first row is our method result, the second row is a case show radius based methods.

<!-- image -->

𝑑 = 0 . 5% 𝑙, 𝑠 = -1, 𝑑 = 1% 𝑙, 𝑠 = -1 and 𝑑 = 0 . 5% 𝑙, 𝑠 = 1. However, the provided implementation would incur memory and computational issues for small offset distances, which is denoted as ' -' in Table 1 for program crashes. Although introducing the feature preserving EMC33 scheme, their results tend to produce creases, leading to noisy feature curves as illustrated in Fig. 9 and Fig. 10.

Comparison with [Zint et al. 2023]. We compare our algorithm with [Zint et al. 2023] by running directly the source code provided by the authors. Their method also generates the mesh through implicit iso-surface extraction, similar to [Chen et al. 2023]. By using octree, it can improve performance at small offset distances. The results of this algorithm are controlled by three main parameters: the minimum octree depth (d0), the maximum octree depth (d1), and the octree depth for resolving non-manifold vertices (d2). We use the default setting, i.e. d0 = 0, d1 = 10, and d2 = 12. Due to the usage of dual contouring for iso-surface extraction, this approach preserves feature lines very well, only second to our algorithm as show in the 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 I , 𝑀 O ) and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 O , 𝑀 I ) columns of Table 1 and as show in Fig. 9. It can get the best or close to the best result in 𝐷 𝑝𝑜𝑖𝑛𝑡 ( 𝑀 O , 𝑀 I ) metric. For all the tested offset distances, [Zint et al. 2023] produces results with consistently large dense meshes as compared to most of the other approaches. Especially as the offset distance gets small, the face numbers of 𝑀 O can reach to millions of triangles, e.g. over 9M for 𝑑 = 0 . 05% 𝑙 . Furthermore, their provided program never successfully processes the entire dataset for the different batch tests. Due to unknown issues, the code provided by [Zint et al. 2023] cannot handle inward offsets, therefore, we only perform the outward offset experiments.

Fig. 12. Our approach supports the generation of feature-preserving non-uniform offsets, i.e. from 0 . 01% 𝑙 to 2% 𝑙 for each model, for any given mesh inputs (green), either inwardly (cyan) or outwardly (red).

<!-- image -->

Fig. 13. For damaged input models, when the normal vectors are correct, the outward offset with 0 . 1% 𝑙 can correct the mesh's minor gaps, selfintersections, and internal redundant faces. Green triangles indicate the border on the mesh. The red box show the fix of delete the error facet, because of the limitation of resolution with figure the boundary triangle mark with green is not obviously, this triangle only have the width of the offset distance. The input mesh has 6170 points, 9991 faces, 416 disconnected components, and 17808 pairs of self-intersecting faces. 10.8 percent of the triangles are located on the boundary of the mesh. After performing offset calculations, the mesh has 20247 points, 41663 faces, and 6 disconnected components. After checking under rational numbers, 0 pairs of selfintersecting faces were found. After saving as an OBJ file with six significant digits for floating-point numbers and reopening it for checking, 947 pairs of self-intersecting faces were found, with 0.7 percent of the triangles located on the mesh boundary.

<!-- image -->

Comparison with [Minkowski sum 2023]. This method employs the computational results of the Minkowski sum of a sphere (represented as a polyhedron) and 𝑀 𝑖 . It has a corresponding function available in version 5.6 of the CGAL library, released in 2023, and exclusively supports outward offsets. Instead of related to offset distances, the computational performance of this method is unstable and heavily depends on the model's concavity and convexity, with longer computation times observed for models with more concave features. If a model strictly adheres to the definition of a convex hull, the computational efficiency is high. It does not support meshes with self-intersections, open boundaries, etc. The evaluated measurements for 𝐷 𝑝𝑜𝑖𝑛𝑡 and 𝐷 𝑝𝑙𝑎𝑛𝑒 for this approach do not exhibit particularly good results. Its output is represented by planar polygons, which, when converted into a triangular mesh, shows a lower number of faces compared to those obtained by the implicit isosurface extraction methods. This approach preserves feature lines well, as shown in Fig. 9, but does not maintain dihedral angles well, as indicated in the 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 O , 𝑀 I ) and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 I , 𝑀 O ) columns of Table 1.

Comparison with [Jung et al. 2004]. Using C++ and CGAL, we implemented the algorithm presented in [Jung et al. 2004]. This method achieves the offset surface by first directly moving vertices of 𝑀 𝑖 following their normal directions, and then calculating a convex hull of all of the moving points. However, their approach

## Robust and Feature-Preserving Offset Meshing · 11

Fig. 14. This figure shows a comparison of our method with other methods for outward offsets. It is clearly visible that our method has an advantage in restoring sharp features and achieves better detail restoration. The two value recorded in each sub-figure are generating facets number and running time.

<!-- image -->

Table 1. This table displays the distance metrics and angle metrics for all methods. To ensure fairness, if a method cannot complete the generation of half of the models in the dataset within the constrained time at a certain offset distance, we will not evaluate it. It can be observed that our method tends to satisfy the point-to-plane distance more compared to the Hausdorff distance. Moreover, methods that generate a larger number of faces have an advantage in satisfying the Hausdorff distance. The column of angle means the different dihedral angle of sampling points in feature line. It can be easy to see our method have a good performance in maintain feature. The last three columns display the number of models successfully completed by all methods (denoted as SUC), the number of faces generated (denoted as FACE), and the average time consumed (denoted as TIME). The time cost of six timeout model of ours are 3602 . 65 𝑠, 40004 . 5 𝑠, 5539 . 63 𝑠, 4476 . 96 𝑠, 13355 . 2 𝑠, 3679 . 04 𝑠 .

| d         |   s | method                  | 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀𝑜,𝑀 𝑖   | 𝐷𝑝𝑜𝑖𝑛𝑡 ( 𝑀𝑜,𝑀 𝑖   | 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀𝑜,𝑀 𝑖 )   | 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀𝑖 ,𝑀𝑜 )   | 𝐷𝑝𝑜𝑖𝑛𝑡 ( 𝑀𝑖 ,𝑀𝑜 )   | 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀𝑖 ,𝑀𝑜 )   | SUC    | FACE          | TIME       |
|-----------|-----|-------------------------|--------------------|-------------------|----------------------|----------------------|---------------------|----------------------|--------|---------------|------------|
| 0 . 05% 𝑙 |  -1 | [Jung et al. 2004]      | 0.000132           | 0.000127          | 1.54531              | N/A                  | N/A                 | 2.18058              | 95     | 1938.6        | 281.5      |
|           |     | [Chen et al. 2023]      | -                  | -                 | -                    | N/A                  | N/A                 | -                    | 0      | -             | -          |
|           |     | Ours                    | 0.000002           | 0.000006          | 0.193007             | N/A                  | N/A                 | 0.654405             | 100    | 2333.2        | 8.6        |
| 0 . 1% 𝑙  |  -1 | [Jung et al. 2004]      | 0.000265           | 0.000255          | 2.53419              | N/A                  | N/A                 | 3.4676               | 95     | 1955.3        | 203.9      |
|           |     | [Chen et al. 2023]      | -                  | -                 | -                    | N/A                  | N/A                 | -                    | 0      | -             | -          |
|           |     | Ours                    | 0.000005           | 0.000012          | 0.265377             | N/A                  | N/A                 | 0.68033              | 100    | 2344.6        | 7.4        |
| 0 . 5% 𝑙  |  -1 | [Jung et al. 2004]      | 0.001382           | 0.001318          | 7.95833              | N/A                  | N/A                 | 9.77958              | 94     | 2010.2        | 235.4      |
|           |     | [Chen et al. 2023]      | 0.000055           | 0.000011          | 2.89306              | N/A                  | N/A                 | 2.64754              | 93     | 643734        | 70.4       |
|           |     | Ours                    | 0.000052           | 0.000061          | 0.905081             | N/A                  | N/A                 | 1.51099              | 100    | 2576.2        | 15.6       |
| 1% 𝑙      |  -1 | [Jung et al. 2004]      | 0.003081           | 0.002937          | 12.3374              | N/A                  | N/A                 | 14.0914              | 88     | 1869.9        | 236.3      |
|           |     | [Chen et al. 2023]      | 0.000219           | 0.000029          | 5.84421              | N/A                  | N/A                 | 5.08535              | 100    | 172471        | 46.8       |
|           |     | Ours                    | 0.000149           | 0.000116          | 2.10864              | N/A                  | N/A                 | 3.43261              | 100    | 2756          | 30.4       |
| 5% 𝑙      |  -1 | [Jung et al. 2004]      | 0.027254           | 0.026503          | 28.4349              | N/A                  | N/A                 | 24.1626              | 58     | 1238          | 539.9      |
|           |     | [Chen et al. 2023]      | 0.005515           | 0.000722          | 13.4986              | N/A                  | N/A                 | 12.2177              | 95     | 8459          | 1.5        |
|           |     | Ours                    | 0.002249           | 0.000198          | 7.70095              | N/A                  | N/A                 | 7.84146              | 100    | 887           | 115.4      |
| 0 . 05% 𝑙 |   1 | [ Alpha Wrap 2022 ]     | 0.000131           | 0.00016           | 9.70743              | 0.000117             | 0.000101            | 9.54859              | 100    | 21239         | 6          |
|           |     | [Jung et al. 2004]      | 0.000135           | 0.00013           | 1.35883              | 0.000296             | 0.000286            | 1.58863              | 95     | 1898.6        | 320.5      |
|           |     | [Chen et al. 2023]      | -                  | -                 | -                    | -                    | -                   | -                    | 0      | -             | -          |
|           |     | [Zint et al. 2023]      | 0.000419           | 0.00043           | 6.17889              | 0.001944             | 0.001733            | 6.00585              | 14     | 9334070       | 104.277    |
|           |     | [ Minkowski sum 2023 ]  | 0.000711           | 0.000733          | 5.87534              | 0.000137             | 0.000151            | 5.83383              | 79     | 4967.2        | 425.1      |
|           |     | Ours                    | 0.000001           | 0.000006          | 0.432793             | 0.000243             | 0.000239            | 0.868208             | 100    | 2314.1        | 6.6        |
| 0 . 1% 𝑙  |   1 | [ Alpha Wrap 2022 ]     | 0.000098           | 0.000128          | 12.427               | 0.00038              | 0.000365            | 12.7267              | 100    | 17915.2       | 5.1        |
|           |     | [Jung et al. 2004]      | 0.000274           | 0.000267          | 2.19128              | 0.000616             | 0.000605            | 2.54556              | 93     | 1860.5        | 209.2      |
|           |     | [Chen et al. 2023]      | -                  | -                 | -                    | -                    | -                   | -                    | 0      | -             | -          |
|           |     | [Zint et al. 2023]      | 0.000132           | 0.000147          | 2.54262              | 0.001364             | 0.001165            | 4.16945              | 38     | 706214        | 247.291    |
|           |     | [ Minkowski sum 2023 ]  | 0.000388           | 0.000413          | 6.30141              | 0.00029              | 0.000281            | 6.44548              | 79     | 4965.8        | 404.5      |
|           |     | Ours                    | 0.000002           | 0.000007          | 0.474414             | 0.000495             | 0.000488            | 0.94634              | 100    | 2329.3        | 6.6        |
| 0 . 5% 𝑙  |   1 | [ Alpha Wrap 2022 ]     | 0.000179           | 0.000074          | 20.6634              | 0.002488             | 0.002444            | 20.8965              | 100    | 15416         | 4          |
|           |     | [Jung et al. 2004]      | 0.001429           | 0.001351          | 6.43899              | 0.003207             | 0.003171            | 6.93395              | 88     | 1734.9        | 294.8      |
|           |     | [Chen et al. 2023]      | 0.000137           | 0.00001           | 6.73193              | 0.002582             | 0.002547            | 4.12022              | 90     | 570871        | 82.6       |
|           |     | [Zint et al. 2023]      | 0.000159           | 0.00004           | 2.19956              | 0.003191             | 0.003152            | 3.75481              | 77     | 180346        | 176.7      |
|           |     | [ Minkowski sum 2023 ]  | 0.000346           | 0.00022           | 6.80817              | 0.002668             | 0.002638            | 6.97272              | 79     | 4858.7        | 422.9      |
|           |     | Ours                    | 0.000119           | 0.000054          | 1.11749              | 0.0026               | 0.002565            | 1.52496              | 100    | 2562.3        | 15.2       |
| 1% 𝑙      |   1 | [ Alpha Wrap 2022 ]     | 0.000575           | 0.000116          | 22.9292              | 0.005089             | 0.004997            | 21.6803              | 100    | 15900.4       | 3.4        |
|           |     | [Jung et al. 2004]      | 0.002953           | 0.002674          | 9.54129              | 0.006582             | 0.006485            | 9.971                | 86     | 1761          | 438.2      |
|           |     | [Chen et al. 2023]      | 0.000529           | 0.000037          | 8.53625              | 0.005337             | 0.005237            | 6.42665              | 100    | 126664        | 50.3       |
|           |     | [Zint et al. 2023]      | 0.000427           | 0.000023          | 4.7115               | 0.006542             | 0.006432            | 6.10771              | 94     | 112318        | 160.5      |
|           |     | Minkowski sum 2023 Ours | 0.000948 0.000303  | 0.000476 0.000202 | 7.85758 1.33165      | 0.005553 0.005403    | 0.005463 0.005302   | 7.55089 1.79176      | 79 100 | 4639.1 2950.9 | 403.3 42.8 |
| 5% 𝑙      |   1 | [ Alpha Wrap 2022 ]     | 0.007818           | 0.000074          | 19.9315              | 0.029172             | 0.028471            | 18.6799              | 100    | 20533.5       | 2.7        |
|           |     | [Jung et al. 2004]      | 0.017843           | 0.013372          | 18.7543              | 0.037292             | 0.036438            | 18.0737              | 60     | 1319.8        | 780        |
|           |     | [Chen et al. 2023]      | 0.008097           | 0.000626          | 14.0047              | 0.029311             | 0.028586            | 13.6277              | 100    | 4862.7        | 1.5        |
|           |     | [Zint et al. 2023]      | 0.007315           | 0.000259          | 13.8445              | 0.03167              | 0.030529            | 13.8605              | 97     | 11977.9       | 45.6       |
|           |     | [ Minkowski sum 2023 ]  | 0.009682           | 0.002429          | 14.7866              | 0.030825             | 0.029757            | 13.5377              | 79     | 3301.1        | 418.7      |
|           |     | Ours                    | 0.006067           | 0.003044          | 5.09495              | 0.030913             | 0.02971             |                      |        |               |            |
|           |     |                         |                    |                   |                      |                      |                     | 5.11666              | 94     | 3062.4        | 266.1      |

can lead to missing or erroneous faces as shown in the second column of Fig. 15. Moreover, in scenarios with complex and many self-intersections, this approach can have a significant increase in computational time as the offset distance increases, whereas our method does not show such a significant rise as shown in the Table 1 and Fig. 18 when offset distance become large like 𝑑 &gt; = 1% 𝑙 . For many cases, relying solely on the normal direction can easily produce a large number of creases, thus generating many redundant features as shown in Fig. 9 and Fig. 10. Additionally, this results in particularly poor 𝐷 𝑎𝑛𝑔𝑙𝑒 values. And its 𝐷 𝑝𝑜𝑖𝑛𝑡 and 𝐷 𝑝𝑙𝑎𝑛𝑒 values often among the worst over all algorithm as shown in Table 1.

To summarize, our method can excellently capture sharp features of the inputs and has the best 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 O , 𝑀 I ) , 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 O , 𝑀 I ) , and 𝐷 𝑎𝑛𝑔𝑙𝑒 ( 𝑀 I , 𝑀 O ) among all methods because our approach focuses more on the mitered offset effect. Compared to the competing approaches, our method runs fast at small offset distances, and due to the presence of acceleration strategies, it still performs reasonably well as the offset distance increases.

## 4.4 Timing.

As shown in the 𝑇 column of Table 1, averaged over the entire tested dataset, our approach achieves the fastest computational speed for inward offset generation when offset distance ≤ 1%. For outward offset generation, while [Portaneri et al. 2022] is consistently among the fastest approach, as the offset distance increases, ours changes from being comparable to [Portaneri et al. 2022] at 0 . 05% and 0 . 1% and is more than 30 times faster compared to [Jung et al. 2004], [Zint et al. 2023], and Minkowski sum 2023, more than 10 times faster compared to [Jung et al. 2004], [Chen et al. 2023], [Zint et al. 2023], and Minkowski sum 2023 when 𝑑 = 0 . 5%, about 10 times faster compared to [Jung et al. 2004], and Minkowski sum 2023 when 𝑑 = 1%, and only 2-3 times faster than [Jung et al. 2004], and Minkowski sum 2023 when 𝑑 = 5%. Top-left corner of Fig. 18 further compares the competing approaches with ours on five selected representative models by varying the offset distances from 𝑑 = 0 . 25% to 5%. It's clear that our approach ranks among the fastest one when offset distance is small and the computation slows down for large offset distances. The reason behind is due to the increased amount of self-intersection computations of our approach when 𝑑 increases. Bottom-left corner of Fig. 18 further verifies that our computational bottleneck lies in the intersection resolving operations for models with excessive details. This step accounts 88 . 6% of the total computing time. Right half of Fig. 18 plots the detailed timing for each model by our approach for the varying offset distances.

Fig. 15. This figure shows a comparison of our method with other methods for inward offsets. It is clearly visible that our method has an advantage in restoring sharp features and achieves better detail restoration. And [Jung et al. 2004] can cause the error facet like show in the third row. The two value recorded in each sub-figure are generating facets number and running time.

<!-- image -->

Fig. 16. From left to right, the figures represent the input, the output of our method after a 5%l offset, and the output of [Alpha Wrap 2022]. The 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) from the red point on the left model to the red point on the middle model is not as close to 5%l as the 𝐷 𝑝𝑙𝑎𝑛𝑒 ( 𝑀 I , 𝑀 O ) from the left model to the right model.

<!-- image -->

## 5 CONCLUSION

We propose a new and robust approach to generate an offset mesh for a 3D input with an arbitrary topology and geometry complexities. Our method ensures the output with several nice properties, such as feature preservation, similar number of triangles with the input, free of self-intersections and degenerate elements. Our approach also support user-specified non-uniform offset distances. We anticipate that, with all these advantages combined, our method makes a significant advancement in the geometry processing related field.

Discussion and Limitations: Several aspects worth to investigate to further improve the current approach. First, the generalized winding number may be computed wrongly for points very close to the mesh surface, which is an issue could be mitigated through the combination of ray-intersection checks. Second, to introduce as few as possible approximations to the offset mesh generation, we only consider conservative speedup strategies in the current version, where for applications in rendering and animation, we may further improve the performance by designing accelerations with a controllable tolerance to the geometric distance.

Fig. 17. show the mesh in different offset to generate a empty mesh.

<!-- image -->

Fig. 18. Top-left corner of this figure shows the different method timing cost with 5 representative models. In 0.25%l offset distance, [Zint et al. 2023] and [Chen et al. 2023] can not success generate the offset mesh. Bottom-left corner of this figure the execution time of each part of our algorithm. The right part shows input triangle number VS time cost.

<!-- image -->

## REFERENCES

2024. The OEIS Foundation Inc. https://oeis.org/search?q=A101052&amp;language= english&amp;go=Search
2. Nina Amenta, Sunghee Choi, and Ravi Krishna Kolluri. 2001. The power crust, unions of balls, and the medial axis transform. Computational Geometry 19, 2-3 (2001), 127-153.
3. Alan Brunton and Lubna Abu Rmaileh. 2021. Displaced signed distance fields for additive manufacturing. ACM Transactions on Graphics (TOG) 40, 4 (2021), 1-13.
4. Marcel Campen and Leif Kobbelt. 2010. Polygonal boundary evaluation of minkowski sums and swept volumes. 29, 5 (2010), 1613-1622.
5. Lufeng Chen, Man-Fai Chung, Yaobin Tian, Ajay Joneja, and Kai Tang. 2019a. Variabledepth curved layer fused deposition modeling of thin-shells. Robotics and ComputerIntegrated Manufacturing 57 (2019), 422-434.
6. Zhen Chen, Zherong Pan, Kui Wu, Etienne Vouga, and Xifeng Gao. 2023. Robust Low-Poly Meshing for General 3D Models. ACM Trans. Graph. 42, 4, Article 119 (jul 2023), 20 pages. https://doi.org/10.1145/3592396
7. Zhen Chen, Daniele Panozzo, and Jeremie Dumas. 2019b. Half-space power diagrams and discrete surface offsets. IEEE Transactions on Visualization and Computer Graphics 26, 10 (2019), 2970-2981.
- P. Cignoni, C. Rocchini, and R. Scopigno. 1998. Metro: Measuring Error on Simplified Surfaces. Computer Graphics Forum 17, 2 (1998), 167-174.
9. Xifeng Gao, Hanxiao Shen, and Daniele Panozzo. 2019. Feature Preserving Octree-Based Hexahedral Meshing. Computer Graphics Forum 38, 5 (2019), 135-149. https://doi.org/10.1111/cgf.13795 arXiv:https://onlinelibrary.wiley.com/doi/pdf/10.1111/cgf.13795
10. Samuele Giraudo. 2015. Combinatorial operads from monoids. Journal of Algebraic Combinatorics 41 (2015), 493-538.
11. Peter Hachenberger. 2009. Exact Minkowksi sums of polyhedra and exact and efficient decomposition of polyhedra into convex pieces. Algorithmica 55, 2 (2009), 329-345.
12. Peter Hachenberger, Lutz Kettner, and Kurt Mehlhorn. 2007. Boolean operations on 3D selective Nef complexes: Data structure, algorithms, optimized implementation and experiments. Computational Geometry 38, 1 (2007), 64-99. https://doi.org/10.1016/ j.comgeo.2006.11.009 Special Issue on CGAL.
13. Alec Jacobson, Ladislav Kavan, and Olga Sorkine-Hornung. 2013. Robust inside-outside segmentation using generalized winding numbers. ACM Transactions on Graphics (TOG) 32, 4 (2013), 1-12.
14. Tao Ju, Frank Losasso, Scott Schaefer, and Joe Warren. 2002. Dual contouring of hermite data. In Proceedings of the 29th annual conference on Computer graphics and interactive techniques . 339-346.
15. Wonhyung Jung, Hayong Shin, and Byoung K Choi. 2004. Self-intersection removal in triangular mesh offsetting. Computer-Aided Design and Applications 1, 1-4 (2004), 477-484.
16. Leif P Kobbelt, Mario Botsch, Ulrich Schwanecke, and Hans-Peter Seidel. 2001. Feature sensitive surface extraction from volume data. In Proceedings of the 28th annual conference on Computer graphics and interactive techniques . 57-66.
17. Louisa Lam, Seong-Whan Lee, and Ching Y Suen. 1992. Thinning methodologies-a comprehensive survey. IEEE Transactions on Pattern Analysis &amp; Machine Intelligence 14, 09 (1992), 869-885.
18. Pan Li, Bin Wang, Feng Sun, Xiaohu Guo, Caiming Zhang, and Wenping Wang. 2015. Q-mat: Computing medial axis transform by quadratic error minimization. ACM Transactions on Graphics (TOG) 35, 1 (2015), 1-16.

Shengjun Liu and Charlie CL Wang. 2010. Fast intersection-free offset surface generation from freeform models with triangular meshes. IEEE Transactions on Automation Science and Engineering 8, 2 (2010), 347-360.

Takashi Maekawa. 1999a. An overview of offset curves and surfaces. Computer-Aided Design 31, 3 (1999), 165-173. https://doi.org/10.1016/S0010-4485(99)00013-5

- Takashi Maekawa. 1999b. An overview of offset curves and surfaces. Computer-Aided Design 31, 3 (1999), 165-173.

Fig. 19. Triangle number of input VS output generated by our approach.

<!-- image -->

- Wenlong Meng, Shuangmin Chen, Zhenyu Shu, Shi-Qing Xin, Hongbo Fu, and Changhe Tu. 2018. Efficiently computing feature-aligned and high-quality polygonal offset surfaces. Computers &amp; Graphics 70 (2018), 62-70.
- Sang C Park and Yun C Chung. 2003. Mitered offset for profile machining. ComputerAided Design 35, 5 (2003), 501-505.
- Darko Pavić and Leif Kobbelt. 2008. High-resolution volumetric computation of offset surfaces with feature preservation. In Computer Graphics Forum , Vol. 27. Wiley Online Library, 165-174.
- B. Pham. 1992. Offset curves and surfaces: a brief survey. Computer-Aided Design 24, 4 (1992), 223-229. https://doi.org/10.1016/0010-4485(92)90059-J
- Cédric Portaneri, Mael Rouxel-Labbé, Michael Hemmer, David Cohen-Steiner, and Pierre Alliez. 2022. Alpha wrapping with an offset. ACM Transactions on Graphics (TOG) 41, 4 (2022), 1-22.
- The CGAL Project. 2023. CGAL User and Reference Manual (5.6 ed.). CGAL Editorial Board. https://doc.cgal.org/5.6/Manual/packages.html
- Jaroslaw R Rossignac and Aristides AG Requicha. 1986. Offsetting operations in solid modelling. Computer Aided Geometric Design 3, 2 (1986), 129-148.
- B. Stellato, G. Banjac, P. Goulart, A. Bemporad, and S. Boyd. 2020. OSQP: an operator splitting solver for quadratic programs. Mathematical Programming Computation 12, 4 (2020), 637-672. https://doi.org/10.1007/s12532-020-00179-2
- Feng Sun, Yi-King Choi, Yizhou Yu, and Wenping Wang. 2013. Medial meshes for volume approximation. arXiv preprint arXiv:1308.3917 (2013).
- Feng Sun, Yi-King Choi, Yizhou Yu, and Wenping Wang. 2015. Medial meshes-a compact and accurate representation of medial axis transform. IEEE transactions on visualization and computer graphics 22, 3 (2015), 1278-1290.
- Andrea Tagliasacchi, Thomas Delame, Michela Spagnuolo, Nina Amenta, and Alexandru Telea. 2016. 3d skeletons: A state-of-the-art report. 35, 2 (2016), 573-597.
- Wayne Tiller and Eric G. Hanson. 1984. Offsets of Two-Dimensional Profiles. IEEE Computer Graphics and Applications 4, 9 (1984), 36-46. https://doi.org/10.1109/MCG. 1984.275995
- Tim Van Hook. 1986. Real-time shaded NC milling display. ACM SIGGRAPH Computer Graphics 20, 4 (1986), 15-20.
- Bala R. Vatti. 1992. A Generic Solution to Polygon Clipping. Commun. ACM 35, 7 (jul 1992), 56-63.
- Charlie CL Wang and Yong Chen. 2013. Thickening freeform surfaces for solid fabrication. Rapid Prototyping Journal (2013).
- Charlie CL Wang and Dinesh Manocha. 2013. GPU-based offset surface computation using point samples. Computer-Aided Design 45, 2 (2013), 321-330.
- Qingnan Zhou and Alec Jacobson. 2016. Thingi10K: A Dataset of 10,000 3D-Printing Models. arXiv preprint arXiv:1605.04797 (2016).
- Daniel Zint, Nissim Maruani, M Rouxel-Labb, and Pierre Alliez. 2023. Feature-Preserving Offset Mesh Generation from Topology-Adapted Octrees. In Computer Graphics Forum , Vol. 42. Wiley Online Library, e14906.