## A Parallel Feature-preserving Mesh Variable Offsetting Approach Based On Dynamic Programming ∗

HONGYI CAO, GANG XU *, RENSHU GU, JINLAN XU, Hangzhou Dianzi University, China XIAOYU ZHANG, Beijing Institute of Spacecraft System Engineering, China TIMON RABCZUK, Institute of Structural Mechanics, Germany

<!-- image -->

Mesh offsetting plays an important role in discrete geometric processing. In this paper, we propose a parallel feature-preserving mesh offsetting framework with variable distance. Different from the traditional method based on distance and normal vector, a new calculation of offset position is proposed by using dynamic programming and quadratic programming, and the sharp feature can be preserved after offsetting. Instead of distance implicit field, a spatial coverage region represented by polyhedral for computing offsets is proposed. Our method can generate an offsetting model with smaller mesh size, and also can achieve high quality without gaps, holes, and selfintersections. Moreover, several acceleration techniques are proposed for the efficient mesh offsetting, such as the parallel computing with grid, AABB tree and rays computing. In order to show the efficiency and robustness of the

∗ Corresponding author: gxu@hdu.edu.cn (Gang XU)

Authors' addresses: Hongyi Cao, Gang Xu *, Renshu Gu, Jinlan Xu, Hangzhou Dianzi University, 2# Street 1158, Hangzhou, Zhejiang, 310018, China; Xiaoyu Zhang, Beijing Institute of Spacecraft System Engineering, Beijing, China; Timon Rabczuk, Institute of Structural Mechanics, Bauhaus-Universität Weimar, Germany.

© 2023 Association for Computing Machinery.

This is the author's version of the work. It is posted here for your personal use. Not for redistribution. The definitive Version of Record was published in , https://doi.org/10. 1145/nnnnnnn.nnnnnnn.

proposed framework, we have tested our method on the quadmesh dataset, which is available at [https://www.quadmesh.cloud]. The source code of the proposed algorithm is available on GitHub at [https://github.com/iGameLab/PFPOffset].

Additional Key Words and Phrases: Geometry processing, Mesh offsetting, Feature-preserving, Variable distance, Dynamic programming

## ACMReference Format:

Hongyi Cao, Gang Xu *, Renshu Gu, Jinlan Xu, Xiaoyu Zhang, and Timon Rabczuk. 2023. A Parallel Feature-preserving Mesh Variable Offsetting Approach Based On Dynamic Programming. 1, 1 (October 2023), 17 pages. https://doi.org/10.1145/nnnnnnn.nnnnnnn

## 1 INTRODUCTION

Mesh offsetting is an important research topic in the field of digital geometric processing and has many applications, including but not limited to computer-aided design (CAD), collision detection, path planning, boundary layer mesh generation, and architectural design [Zint et al. 2023]. Surface mesh offsetting, for instance, offers a versatile tool for post-production adjustments to meshes, allowing sections to be enlarged or reduced based on user requirements or outcomes from physical simulations. Among them, offsetting

, Vol. 1, No. 1, Article . Publication date: October 2023.

with variable distance has many applications in CAD models of aviation devices. Further, this technique can morph a triangular mesh into a hollow object. The original triangular mesh can be viewed as the outer surface. By offsetting it inwardly, an inner surface emerges. Together, these surfaces depict a solid entity with thickness, commonly termed a "shell-like solid". This structure is notably leveraged in additive manufacturing [Xie and Chen 2017].

Fig. 1. An example to show the shortcoming of direct offsetting method in 2D. The inner surface(black) and the outer surface (blue) are shown with the outward offsetting operation. The offsetting positions of vertex 𝐷 and vertex 𝐸 are 𝐷 ′ and 𝐸 ′ respectively. This case produces self-intersection, and some part of the edge (red) cannot be identified by the searching algorithm.

<!-- image -->

Fig. 2. In the depicted figure, we present scenarios where the dual contouring mesh generation algorithm encounters challenges. Commencing on the top left, that is the initial mesh. Adjacently in the center-top, that is the resultant mesh derived from specific offsets. On the top right, the sampling grid is illustrated. Expanding upon this, the bottom visualization magnifies a cell from the aforementioned sampling grid, highlighting a particular cell that the dual contouring algorithm fails to accurately discern.

<!-- image -->

However, currently it is still a challenging problem to create an offset model robustly from the boundary polygon representation. The direct offsetting methods often cause self-intersections as shown in Fig. 1, and it is difficult to judge whether to generate after intersection for complex CAD models. The distance-field based methods can accurately define the surface generated after offsetting, but it often causes problems due to the sampling of implicit expressions as shown in Fig. 2 and Fig. 3. Moreover, there is few research work on the feature-preserving and mesh variable offsetting method with high efficiency and robustness.

Fig. 3. An example of the problem caused by sampling when meeting short distance offset. The first mesh from left to right represents the initial mesh, the second represents the result of the sampling method using 256*256*256 outward offset, and the third represents the result of our method's outward offset , the fourth shows the cross-section of our method and put together with the initial mesh.

<!-- image -->

In this paper, we will focus on the mesh offsetting problem with variable distance. We propose a parallel feature-preserving mesh variable offsetting framework to create an offset model without gaps, holes, and self-intersections. The input is a triangular mesh denoted as 𝑀𝑒𝑠ℎ 0 and the desired offset distance for each face on 𝑀𝑒𝑠ℎ 0; The output is the offsetting triangular mesh denoted as 𝑀𝑒𝑠ℎ 1. Different from the traditional method based on distance and normal vector, a new calculation of offset position is proposed by dynamic programming and quadratic programming. Instead of distance implicit field, a spatial coverage region represented by polyhedral for computing offsets is proposed.

Our offsetting approach has the following several advantages which have not been provided by existing approaches.

- Variable offsetting : We provide a practical method to generate offsetting mesh with different distance, which plays an important role for shape optimization results. Arbitrary offset distance can be handled to generate both grown and shrunk models.
- Feature-preserving : Distance-field based methods often lack sharp edges or corners indicated in the original models. In contrast, our offsetting results can preserve all the sharp feature of the original model.

- Efficient : several acceleration techniques are proposed for the efficient mesh offsetting, such as the parallel computing with grid, AABB tree and rays computing.
- Light-weight : Compared with previous method, our framework can generate a offsetting surface with smaller mesh size, which is similar with the original mesh.
- Simple : Our framework is relatively easy to implement, which can be considered as a combination of direct offsetting method and distance-field based approach. The code of our implementation has been open sourced on Github.

## 2 RELATED WORK

In this section, we discuss various research efforts related to the topic of this paper, spanning methodologies such as direct offsetting, distance-field-based methods, the Minkowski sum, skeleton-based techniques, and ray-based approaches.

## Direct Offsetting Methods

A prominent method proposed by [Jung et al. 2004] offsets based on vertex normal direction. Although efficient, it struggles with holes in complex CAD models, leading to offset meshes that can be defective, particularly in scenarios involving self-intersections. The precision issues arising from floating-point operations in selfintersections have been addressed using infinite precision operations [Campen and Kobbelt 2010], but results occasionally produce twisted meshes. Offsetting surfaces with polynomials or B-splines is explored in [Maekawa 1999]. However, subsequent intersection operations are intricate. Another approach calculates offset positions based on distance and uniform distribution, followed by point cloud reconstruction [Meng et al. 2018].

## Distance-Field-Based Methods

These methods are popular in modern 3D printing [Brunton and Rmaileh 2021]. Generally, they require resampling to generate the final mesh, which can compromise geometric features, especially in detailed meshes [Wang and Chen 2013]. Challenges also arise from the grid density, preservation of sharp features, and computational efficiency. Some attempts, such as [Kobbelt et al. 2001], have tackled the ambiguity of the Dual Contouring and Marching Cube, but they tend to excel mostly with CAD models. In the study by [Liu and Wang 2010], there are some improvements in computational efficiency. The method introduced in [Pavić and Kobbelt 2008] tries to preserve sharp features, but it involves high grid density and computational expenses. Only a few works, like [Chen et al. 2019a], delve into variable-thickness offsetting for these methods.

During the generation process, the fixed edge length grid in the dual contouring algorithm is not suitable for all local reconstructions. An improved method based on octree was proposed in the study by [Zint et al. 2023].

## Minkowski Sum Method

The Minkowski sum offers a solution for mesh offsetting by calculating the sum of mesh and sphere polygons [Rossignac and Requicha 1986]. An advanced method in [Hachenberger 2009] computes the exact 3D Minkowski sum of non-convex polyhedra by decomposing them into convex parts. Despite its robustness, the method is slow for complex CAD models and struggles with variable thickness offsets and sharp feature preservation. Its implementation in CGAL [Project 2023] is robust.

## Skeleton-Based Methods

Skeletal meshes are prevalent in geometry processing [Tagliasacchi et al. 2016]. The mesh model can be represented by medial axes and spheres [Amenta et al. 2001; Sun et al. 2015]. There are techniques to generate skeleton meshes [Lam et al. 1992; Li et al. 2015]. However, they don't always suit all mesh types, and some mesh models have intricate skeletal structures. There have been efforts to simplify medial axes [Sun et al. 2013]. Offsetting can be achieved by adjusting the radius of the balls on the skeleton. Still, these methods often fall short with CAD models with sharp features.

## Ray-Based Methods

This approach [Chen et al. 2019b; Wang and Manocha 2013] is rooted in the dexel buffer structure [Van Hook 1986]. A recent algorithm in [Chen et al. 2019b] offers an efficient parallelized method using rays and voxels to compute mesh offsets, bypassing the need for distance field computation. Nonetheless, due to the grid's voxellike structure, a high density is essential for accurate shape representation, leading to increased computational costs and dense mesh outputs. The approach also mainly considers a consistent offset, and the results are usually voxel representations.

## 3 METHOD

We initiated our exploration from two foundational methodologies. The first involves the direct offsetting method, where every mesh vertex is offset according to its normal vector, either inwards or outwards. This method yields results characterized by arc-shaped features, as depicted in Fig. 5(b). Next, we considered the method rooted in distance fields. As illustrated in Fig. 6(a), relying on distance fields for offset calculation would still manifest arc-shaped or spherical features. Despite its challenges in preserving sharp features, it uniquely delineates a region within a three-dimensional space. This defined region corresponds to the spatial coverage resulting from the offset operation. Robustness in various complex scenarios is achieved by resampling the boundaries of this region.

Both aforementioned methods fall short in retaining sharp features. However, upon closer inspection, the direct offsetting method presented opportunities for refinement. Adjustments in the offset's magnitude and direction can potentially aid in the preservation of sharp features. To capitalize on the robustness inherent to the distance field method, our proposed approach represents the offsetinduced spatial coverage through the union of multiple polyhedra. This integrated approach has the dual advantages of sharp feature preservation and robustness in intricate models.

For every face, 𝑓 𝑎𝑐𝑒 𝑖 , of a triangular mesh, 𝑀𝑒𝑠ℎ 0, the offset results in a polyhedron. This polyhedron represents the spatial coverage of 𝑓 𝑎𝑐𝑒 𝑖 , and is denoted as 𝐹 𝑖 .

Given that 𝑀𝑒𝑠ℎ 0 consists of 𝑛 triangular faces, the cumulative spatial coverage post-offset is represented by Σ 𝑛 𝑖 = 1 𝐹 𝑖 . This coverage manifests as a shell-like structure with an outer surface and an inner surface corresponding to a hollow body. In the context of an outward offset, 𝑀𝑒𝑠ℎ 0 is defined by the inner surface, while for an inward offset, 𝑀𝑒𝑠ℎ 0 corresponds to the outer surface. 𝑀𝑒𝑠ℎ 0 is determined based on the offset direction, our subsequent endeavors concentrate on extracting the opposing surface to form 𝑀𝑒𝑠ℎ 1.

Table 1. Compared with the existing methods, the more the number of " ★ " in each cell, the better the performance.

|                                        | Support Variable   | Efficiency with short distance   | Efficiency with long distance   | Sharp feature preserving   | Ambiguity   | Robustness   | mesh/voxels   |
|----------------------------------------|--------------------|----------------------------------|---------------------------------|----------------------------|-------------|--------------|---------------|
| Direct offset method[Jung et al. 2004] | ★★★★★              | ★★★★★                            | ★★★★                            | ★★★                        | ★★★★★       | ★★           | mesh          |
| Distance field[Wang and Chen 2013]     | ★                  | ★★★                              | ★★★★★                           | ★★                         | ★★★         | ★★★          | mesh          |
| Distance field[Chen et al. 2019a]      | ★★★★★              | ★★                               | ★★★★                            | ★★                         | ★★★         | ★★★          | mesh          |
| Minkowski sum[Hachenberger 2009]       | ★                  | ★★★★                             | ★★★★                            | ★★★                        | ★★★★★       | ★★★★★        | mesh          |
| Ray based method[Chen et al. 2019b]    | ★                  | ★★★★                             | ★★★★★                           | ★★★                        | ★★★★★       | ★★★★★        | voxels        |
| Ours                                   | ★★★★★              | ★★★                              | ★★                              | ★★★★★                      | ★★★★★       | ★★★★★        | mesh          |

Fig. 4. This figure shows the flow of our method, which consists of 6 parts arranged by 5 pink unidirectional arrows. The first part is the input mesh. The second part is the result of Step 𝑆 1 , showing the path formed by each vertex offset. The third part represents the calculated cumulative spatial coverage. The fourth part represents the established grid. The fifth part represents the calculation of intersection. The sixth part is post-processing.

<!-- image -->

Essentially, this challenge can be rephrased as the task of merging all 𝐹 𝑖 into a hollow polyhedron. Initial efforts involved leveraging CGAL's polygonal Boolean operations, But we encountered program anomalies after performing a significant number of 𝐹 𝑖 unions. Subsequent endeavors utilizing a bespoke Boolean operation proved insufficient in terms of robustness and efficiency.

Consequently, we adopted an approach that momentarily disregards topological relationships, focusing on intersection, subdivision, followed by ray probing techniques. This method produced a Triangle Soup constituting the offset surface. Subsequently, we employed TetWild to transform this into tetrahedra, extracting the external surface to realize the offset surface. Although this methodology may cede some efficiency, it boasts commendable robustness. Even if minor errors (less than 0.01%) arise in the Triangle Soup due to discrepancies in previous steps, the approach remains effective in delivering robust results.

The subsequent subsections will describe details of our methodology. Our approach is articulated through five steps, designated as 𝑆 1 ∼ 𝑆 5. Detailed descriptions of these steps are provided from subsection 3.1 to subsection 3.5, respectively.

Fig. 5. (a) is origin surface mesh. (b) is the result surface mesh through each vertex offsetting outward with normal direction. (c) represents our desired mesh offset outcome while preserving sharp features. Therefore, just offsetting based on the normal vectors of each facet does not align with our objective.

<!-- image -->

## 3.1 Dynamic Programming and Quadratic Programming

This step is a pivotal one in our approach. It incorporates both dynamic programming and quadratic programming to determine the optimal offset position for each vertex in 𝑀𝑒𝑠ℎ 0. Initially, our offsetting procedure originates from individual vertices, contemplating a singular offset position for each vertex.

The preservation of features can be independently addressed by considering local meshes. A vertex 𝑉 from 𝑀𝑒𝑠ℎ 0, in conjunction with several faces from its 1-ring neighborhood, constitutes a local mesh. Given 𝑘 faces within this local mesh, we denote each face as 𝑂 𝑖 (0 ≤ 𝑖 ≤ 𝑘 -1). The desired offset distance for 𝑂 𝑖 is 𝐿 𝑖 , and the plane created by 𝑂 𝑖 is represented as 𝐴 𝑖 .

To maintain sharp features while performing a variable offset on the mesh, it becomes evident that distance cannot serve as a metric of feature-preserving due to the varied offset distances across facets. Instead, employing angles as a metric is a viable solution. Extensive testing indicated that for the offset vertex 𝑉 ′ derived from 𝑉 , in order to preserve the angular attributes of the local mesh, the distance from 𝑉 ′ to the plane 𝐴 𝑖 should be equivalent to 𝐿 𝑖 .

Fig. 6. In the figure, the rectangle formed by points 𝐴 , 𝐵 , 𝐶 , and 𝐷 represents the initial mesh. The offsetting distance of lines 𝐴𝐵 and 𝐴𝐷 is 2 , the offsetting distance of lines 𝐵𝐶 and 𝐶𝐷 is 1 . (a) represents the offset result based on the distance field. (b) represents the desired offset result we aim for, preserving sharp features while performing variable offsetting. Both of which are schematic diagrams of rectangle offsetting in the 2-dimensional case. However, it can be observed that the distance field based approach does not fulfill our goal of preserving features.

<!-- image -->

To circumvent numerical precision issues that might arise during subsequent solver computations, we introduce two tolerance coefficients, 𝑡𝑜𝑙𝑒 𝑙 and 𝑡𝑜𝑙𝑒 𝑟 . Hence, the offset distance condition can be relaxed, ensuring the offset distance falls within the range 𝑡𝑜𝑙𝑒 𝑙 𝐿 𝑖 ∼ 𝑡𝑜𝑙𝑒 𝑟 𝐿 𝑖 . The two tolerance coefficients can be adjusted by the user. By default, our software sets 𝑡𝑜𝑙𝑒 𝑙 = 1 . 0 and 𝑡𝑜𝑙𝑒 𝑟 = 1 . 3.

In instances where there are infinite points that can serve as 𝑉 ′ satisfying the aforementioned distance constraints, for example, when all facets in the 1-ring neighborhood of 𝑉 lie on the same plane. Our experiments reveal that selecting the point closest to 𝑉 as 𝑉 ′ within this infinite solution space achieves the best feature fidelity. So, the optimization objective strives to minimize the distance between 𝑉 ′ and 𝑉 . Considering the quadratic nature of point-to-point distances juxtaposed with a set of linear constraints defined by the point-to-plane distances, our problem conforms to a quadratic programming framework.

The formulation of this quadratic programming equation proceeds as follows:

The distance from 𝑉 ′ to 𝑉 is 𝐷 . The coordinate of 𝑉 ′ is ( 𝑥,𝑦, 𝑧 ) 𝑇 abbreviated as fi 𝑥 . The coordinate of 𝑉 is ( 𝑥 𝑝 , 𝑦 𝑝 , 𝑧 𝑝 ) .

<!-- formula-not-decoded -->

, Vol. 1, No. 1, Article . Publication date: October 2023.

The formula can be deduced as

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The random point on plane 𝐴 𝑖 is ( 𝑥 𝑖 , 𝑦 𝑖 , 𝑧 𝑖 ) 𝑇 . The normal vector of plane 𝐴 𝑖 is ( 𝑛𝑥 𝑖 , 𝑛𝑦 𝑖 , 𝑛𝑧 𝑖 ) 𝑇 abbreviated as fi 𝑛 𝑖 . The number of faces in 1-ring neighborhood of 𝑉 is 𝑘 . Constraints are established according to the distance 𝐿 𝑖 from 𝑉 ′ to plane 𝐴 𝑖 :

<!-- formula-not-decoded -->

Constraints can be deduced as

<!-- formula-not-decoded -->

In optimizing goal 𝐷 , 𝑒 2 has no effect, so 𝑒 2 is discarded. Then the final planning goal is:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

This can be resolved using a quadratic programming solver, such as OSQP [Stellato et al. 2020]. However, certain scenarios render the above equations unsolvable, for instance, mesh vertices in slender dihedral angles or vertices shared by regions with opposing normal vectors. To address it, we propose a mechanism where a individual mesh vertex can yield multiple offset vertices. This technique necessitates the use of state compression dynamic programming, which will be elaborated upon in subsequent portion of this subsection.

There exist scenarios where quadratic programming lacks feasible solutions. In such cases, our approach, as depicted in the Fig. 7, involves displacing certain vertices from their incumbent positions to two distinct positions, resulting in a pair of offset points. This technique enables the derivation of two or more such offset points and is facilitated by a state-based dynamic programming approach.

We have refined the aforementioned approach by first categorizing each 𝑂 𝑖 into groups, totaling 𝑚 distinct groups. Each group comprises a certain number of triangular facets, and each 𝑂 𝑖 exclusively appears in one of these groups. The 𝑟 𝑡ℎ group contains 𝑛 triangular facets, denoted as 𝑘 𝑟 1, 𝑘 𝑟 2, and so on. Using the planes generated by 𝑘 𝑟 1 , 𝑘 𝑟 2 , . . . 𝑘 𝑟𝑛 and applying the aforementioned quadratic programming method, an offset vertex of the 𝑟 𝑡ℎ group , 𝑉 ′ 𝑟 , is derived.

Given that the local mesh is categorized into 𝑚 groups, our optimization objective is set as 𝐷 𝑁𝐸𝑊 = ˝ 𝑚 𝑟 = 1 | | 𝑉 -𝑉 ′ 𝑟 | | 2 . This choice is based on extensive experimentation, as we have observed that minimizing 𝐷 𝑁𝐸𝑊 results in better preservation of the mesh's features.

Fig. 7. In our method, some vertices when subjected to a mere positional displacement from one locale to another, each of these vertices engenders infeasibilities within quadratic programming. Our solution illustrated in this figure is manifested by displacing each of these vertices from its incumbent position to two distinct positions, yielding a pair of offset vertices. To facilitate the derivation of two or more such offset points, this study harnesses a state-based dynamic programming approach.

<!-- image -->

<!-- image -->

A pivotal question arises: How can these facets be optimally grouped into 𝑚 categories to minimize the target 𝐷 𝑁𝐸𝑊 ? Further, how is this 𝑚 determined? However, it is worth noting that implementing a method based solely on grouping facets by their normal vectors may lack robustness. To tackle this, we adopt a state-based dynamic programming approach.

We employ a state-based dynamic programming approach for implementation. In this context, there are 𝑘 facets, with each facet having two potential states: "selected" and "unselected". As a result, there are a total of 2 𝑘 possible combination states. We utilize a 𝑑𝑝 array with a capacity of 2 𝑘 to store the 𝐷 𝑁𝐸𝑊 values corresponding to each combination of facet selections. The binary representation 0 𝑏 010101 is used to denote the number whose binary value is 010101. For 𝑛 = 6, the binary 0 𝑏 010101 signifies the selection of the first, third, and fifth facets, whereas 0 𝑏 100100 indicates the selection of the third and sixth facets.

Consider a current state 𝑠𝑡𝑎𝑡𝑒 0. If the combination's facets, upon inspection, yield an unsolvable quadratic programming problem, they can be decomposed into two sets: 𝑠𝑡𝑎𝑡𝑒 1 and 𝑠𝑡𝑎𝑡𝑒 2, satisfying 𝑠𝑡𝑎𝑡𝑒 1 ∪ 𝑠𝑡𝑎𝑡𝑒 2 = 𝑠𝑡𝑎𝑡𝑒 0 and 𝑠𝑡𝑎𝑡𝑒 1 ∩ 𝑠𝑡𝑎𝑡𝑒 2 = ∅ . The value

𝑑𝑝 [ 0 𝑏 111111 ] represents the best solution from all divisions. Our primary aim isn't minimizing the number of splits, but ensuring that the cumulative quadratic programming objective 𝐷 across all sub-combinations, 𝐷 𝑁𝐸𝑊 , is minimized. When 𝐷 𝑁𝐸𝑊 reaches its minimum, the number of split groups corresponds to 𝑚 . Arecursive approach is used for solution, illustrated in the Algorithm 1.

The computational complexity of this dynamic programming strategy is 𝑂 ( 3 𝑘 + 2 -2 × 2 𝑘 ) . When 𝑘 = 1 , 2 , 3 , 4 , 5 , 6 , 7 , ... , the resulting sequence is 1 , 3 , 13 , 51 , 181 , 603 , 1933 , ... , which is identified as sequence A101052 on the OEIS website [https://oeis.org/]. Mathematical derivations pertaining to such issues can be found in [Giraudo 2015].

For 𝑘 ≤ 16, the computational iterations are fewer than 42915651. Consequently, when input mesh regions exhibit a vertex which 1-ring neighbor facet counts of 17 or greater, it's beneficial to implement a triangle mesh remeshing algorithm prior to our method.

Fig. 8. In the figure, a mesh is depicted, with each yellow point representing an individual vertex of this mesh. The purple points denote the newly generated vertices resulting from the offset applied to these original vertices.

<!-- image -->

After each vertex in 𝑀𝑒𝑠ℎ 0 has undergone the aforementioned dynamic programming process, the result will resemble what is depicted in Fig. 8. Additional, in the above approach, further optimization can be achieved by recursively decomposing the current state into smaller substates, even when quadratic programming at the current state has feasible solutions. A comparison is then made between the offset distance of the feasible solution at the current state and the results obtained from the recursive decomposition to determine which one results in a shorter total offset distance.

## 3.2 Reassign by Octree

In subsection 3.1, the generation of each vertex primarily takes into account the local mesh characteristics in isolation, without considering inter-vertex relationships. Consequently, in certain specialized scenarios, a proliferation of closely situated vertices can manifest.

To ameliorate mesh quality and reduce subsequent computational complexity, we opt to merge closely located vertices into a singular vertex within our approach. Specifically, when the Euclidean distance between two vertices derived from offset calculations is less than 𝜀 , these two vertices are consolidated into a unified entity.

```
Input: Vertex V, 1-ring neighbor facet's list Output: The minimum value of D_NEW 𝑘 ← The size of 1-ring neighbor facet's list; Initialize vectors 𝑑𝑝 of size 2 𝑘 ; Fill 𝑑𝑝 with -1; Initialize empty vectors 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 ; Function get_sub_state(state,pos,change) : if 𝑝𝑜𝑠 = -1 𝑎𝑛𝑑 𝑐ℎ𝑎𝑛𝑔𝑒 = 𝑇𝑟𝑢𝑒 then Put 𝑠𝑡𝑎𝑡𝑒 at the end of 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 ; return end for 𝑖 ← 𝑝𝑜𝑠 𝑡𝑜 0 do if 𝑠𝑡𝑎𝑡𝑒 & ( 2 𝑖 ) ≠ 0 then get_sub_state( 𝑠𝑡𝑎𝑡𝑒, 𝑖 -1 , 𝑐ℎ𝑎𝑛𝑔𝑒 ) ; get_sub_state( 𝑠𝑡𝑎𝑡𝑒 ⊕ ( 2 𝑖 ) , 𝑖 -1 , 𝑇 𝑟𝑢𝑒 ) ; break end else if 𝑖 = 0 then get_sub_state( 𝑠𝑡𝑎𝑡𝑒, 𝑖 -1 , 𝑐ℎ𝑎𝑛𝑔𝑒 ) ; break end end end end Function DFS(state) : if 𝑑𝑝 [ 𝑠𝑡𝑎𝑡𝑒 ] ≠ -1 then return 𝑑𝑝 [ 𝑠𝑡𝑎𝑡𝑒 ] end 𝑠𝑢𝑐𝑐𝑒𝑠𝑠,𝑉 𝑝 ← do_quadratic_programming ( 𝑠𝑡𝑎𝑡𝑒 ) ; if 𝑠𝑢𝑐𝑐𝑒𝑠𝑠 = 𝑇𝑟𝑢𝑒 then 𝑑𝑝 [ 𝑠𝑡𝑎𝑡𝑒 ] ← 𝑠𝑞𝑟𝑡 (|| 𝑉 𝑝 -𝑉 | | 2 ) ; return 𝑑𝑝 [ 𝑠𝑡𝑎𝑡𝑒 ] end clear the vector 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 ; get_sub_state( 𝑠𝑡𝑎𝑡𝑒, 𝑘, 𝐹𝑎𝑙𝑠𝑒 ) ; 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 _ 𝑡𝑚𝑝 ← 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 ; 𝑚𝑖𝑛𝑥 ←∞ ; for 𝑛𝑒𝑥𝑡 _ 𝑠𝑡𝑎𝑡𝑒 ∈ 𝑠𝑢𝑏 _ 𝑠𝑡𝑎𝑡𝑒 _ 𝑡𝑚𝑝 do 𝑠𝑢𝑏 _ 𝑎𝑛𝑠 ← DFS( 𝑛𝑒𝑥𝑡 _ 𝑠𝑡𝑎𝑡𝑒 ) + DFS( 𝑠𝑡𝑎𝑡𝑒 -𝑛𝑒𝑥𝑡 _ 𝑠𝑡𝑎𝑡𝑒 ) ; if 𝑠𝑢𝑏 _ 𝑎𝑛𝑠 < 𝑚𝑖𝑛𝑥 then 𝑚𝑖𝑛𝑥 ← 𝑠𝑢𝑏 _ 𝑎𝑛𝑠 ; end end 𝑑𝑝 [ 𝑠𝑡𝑎𝑡𝑒 ] ← 𝑚𝑖𝑛𝑥 ; return 𝑚𝑖𝑛𝑥 end DFS( ( 2 𝑘 ) -1 )
```

Algorithm 1: State-based Dynamic Programming

Nonetheless, this process may engender a cascade effect of mergers, as exemplified by the merger of 𝑉 1 ′ with 𝑉 2 ′ , 𝑉 2 ′ with 𝑉 3 ′ , and 𝑉 3 ′ with 𝑉 4 ′ , resulting in the eventual merger of 𝑉 1 ′ and 𝑉 4 ′ due to transitive relationships.

In instances where the Euclidean distance between 𝑉 1 ′ and 𝑉 4 ′ is relatively large, their merger can induce localized distortions within the result mesh. To address this challenge, we have devised an approach that entails an initial merging phase followed by a subsequent splitting phase.

Fig. 9. The figure is bifurcated into two distinct sections, each illustrating the cumulative spatial coverage constructed during Step 𝑆 3 . The left section represents the scenario where Step 𝑆 3 is executed directly without Step 𝑆 2 , while the right section depicts the sequence in which Step 𝑆 2 is executed prior to Step 𝑆 3 .

<!-- image -->

This entails employing an octree structure to reassign each merged vertex collection into multiple discrete points (see Algorithm 2 for further details). The impact of this step on the subsequent step (Step 𝑆 3) is illustrated in Fig. 9.

## 3.3 Construct the Spatial Coverage of Each Face in 𝑀𝑒𝑠ℎ 0

Once all three vertices of a triangular facet have undergone the aforementioned computation, we can then delineate its corresponding spatial coverage. This process entails conducting a Delaunay tetrahedralization using the triangle's three vertices combined with their corresponding offset vertices.

Next, the exterior surface of the Delaunay tetrahedralization result is computed. The volume enclosed by this exterior surface denotes the spatial coverage of the corresponding facet. For a given facet 𝑓 𝑎𝑐𝑒 𝑖 , its computed spatial coverage is denoted by 𝐹 𝑖 . Evidently, as depicted in Fig. 10, 𝐹 𝑖 constitutes a polyhedron. The volume enclosed by the exterior surface of 𝐹 𝑖 represents the spatial coverage of the associated facet.

Delaunay tetrahedralization can be robustly executed using the CGAL library [Project 2023].

When juxtaposed with the direct offset method, our approach inherently multiplies the number of facets by several magnitudes. This surge in facet count poses computational challenges, especially during intersection calculations.

Given this, our subsequent discourse seeks to streamline the process of extracting the characteristics of 𝑀𝑒𝑠ℎ 1 from the cumulative spatial coverage. The cumulative spatial coverage is an aggregation of spatial coverage from all facets in 𝑀𝑒𝑠ℎ 0, as illustrated in Fig. 11.

Upon meticulous examination, we discern that numerous facets can be quickly and definitively judged as non-contributors to the

## Algorithm 2: Reassign Programming By Octree

```
Output: The result of reassign 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑟𝑜𝑜𝑡 ] ← The set of vertices generated by merging; Function reassign(node) : 𝑚𝑎𝑥 _ 𝑑 ← 0; 𝑙 ← 𝑠𝑖𝑧𝑒 ( 𝑣 _ 𝑙𝑖𝑠𝑡 ( 𝑛𝑜𝑑𝑒 )) ; for 𝑖 ← 0 𝑡𝑜 𝑙 -1 do for 𝑗 ← 0 𝑡𝑜 𝑙 -1 do 𝑑 ← 𝑑𝑖𝑠𝑡𝑎𝑛𝑐𝑒 ( 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] [ 𝑖 ] , 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] [ 𝑗 ]) 𝑚𝑎𝑥 _ 𝑑 = max ( 𝑚𝑎𝑥 _ 𝑑, 𝑑 ) ; end end if 𝑚𝑎𝑥 _ 𝑑 < 𝑙𝑖𝑚𝑖𝑡 then 𝑝 ← The average coordinate of 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] ; for 𝑖 ← 0 𝑡𝑜 𝑙 -1 do 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] [ 𝑖 ] ← 𝑝 ; end return end for 𝑖 ← 0 𝑡𝑜 𝑙 -1 do 𝑐ℎ𝑖𝑙𝑑 𝑥 ← The child node where 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] [ 𝑖 ] located; append 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑛𝑜𝑑𝑒 ] [ 𝑖 ] to 𝑣 _ 𝑙𝑖𝑠𝑡 [ 𝑐ℎ𝑖𝑙𝑑 𝑥 ] ; end for 𝑖 ← 0 𝑡𝑜 7 do reassign( 𝑐ℎ𝑖𝑙𝑑 𝑖 ) ; end end
```

Input: A set of vertices generated by merging

reassign(root) ;

Fig. 10. Within the displayed figure, the polyhedra in the first row represent the spatial coverage derived when all vertices are offset to an alternative position. Conversely, the second row illustrates the spatial coverage formulated when certain vertices offset to two or more distinct position.

<!-- image -->

formation of 𝑀𝑒𝑠ℎ 1. This judgment procedure is termed "facet exclusion based on 1-ring neighbor spatial coverage." In what follows, we will utilize the spatial coverage of 𝑓 𝑎𝑐𝑒 𝑖 as an illustrative example.

This spatial coverage encompasses six vertices: 𝑉 1, 𝑉 2, 𝑉 3, 𝑉 ′ 1 , 𝑉 ′ 2 , and 𝑉 ′ 3 , as shown in Fig. 12. Naturally, this foundational logic remains consistent, even for spatial coverage with more vertices. Since the facet 𝑉 1 𝑉 2 𝑉 3 belongs to the 𝑀𝑒𝑠ℎ 0, obviously, it cannot

;

Fig. 11. The top row illustrates the initial mesh( 𝑀𝑒𝑠ℎ 0 ). The middle row showcases the cumulative spatial coverage, while the bottom row presents a cross-sectional view of the said cumulative spatial coverage.

<!-- image -->

Fig. 12. In this figure, there are three vertices of the 𝑖 -th facet in 𝑀𝑒𝑠ℎ 0 are 𝑉 1 , 𝑉 2 and 𝑉 3 which calculated offset vertices are 𝑉 ′ 1 , 𝑉 ′ 2 and 𝑉 ′ 3 . The polyhedron representing the spatial coverage is composed of vertices 𝑉 1 , 𝑉 2 , 𝑉 3 , 𝑉 ′ 1 , 𝑉 ′ 2 , and 𝑉 ′ 3 .

<!-- image -->

serve as a constituent of 𝑀𝑒𝑠ℎ 1. Consider other facets such as 𝑉 1 𝑉 2 𝑉 ′ 2 and 𝑉 ′ 1 𝑉 ′ 2 𝑉 ′ 3 . They may encounter a situation in which they are located on the boundary or within the spatial coverage of 𝑓 𝑎𝑐𝑒 𝑗 , provided that 𝑓 𝑎𝑐𝑒 𝑗 is adjacent to 𝑓 𝑎𝑐𝑒 𝑖 . If such a situation does occur, they are conclusively deemed impossible as components for 𝑀𝑒𝑠ℎ 1.

To further elucidate our specific judgment method as shown in Fig. 13, we continue to use the facet 𝑉 1 𝑉 2 𝑉 ′ 2 as a representative example. Assuming that 𝑓 𝑎𝑐𝑒 𝑢 , 𝑓 𝑎𝑐𝑒 𝑣 , and 𝑓 𝑎𝑐𝑒 𝑤 in 𝑀𝑒𝑠ℎ 0 are adjacent to 𝑓 𝑎𝑐𝑒 𝑖 , the next course of action involves calculating non-edge intersection segments between the facet 𝑉 1 𝑉 2 𝑉 ′ 2 and facets from 𝐹 𝑢 , 𝐹 𝑣 , and 𝐹 𝑤 . Utilizing these intersections, facet 𝑉 1 𝑉 2 𝑉 ′ 2 undergoes subdivision into smaller triangular facets via Constrained Delaunay triangulation on a 2D plane.

In scenarios where all subdivided facets are situated within 𝐹 𝑢 , 𝐹 𝑣 , or 𝐹 𝑤 or lie on their boundaries, the facet 𝑉 1 𝑉 2 𝑉 ′ 2 is confirmed as impossible to be a component of 𝑀𝑒𝑠ℎ 1.

Leveraging this property can significantly reduce computational complexity. Because some triangular facets that are non-contributory can be excluded from the AABB tree before executing the computation of intersections in the Step 𝑆 4.

Note: Edge intersection means when two triangles intersect, the intersection line coincides with the edge of each triangle.

Fig. 13. This figure illustrates the process of facet exclusion based on 1ring neighbor spatial coverage. In the bottom-left image, the red triangle represents 𝑓 𝑎𝑐𝑒 𝑖 , while the blue triangles represent 𝑓 𝑎𝑐𝑒 𝑗 . The image in the bottom center depicts 𝐹 𝑖 , represented as a red polyhedron. The bottom-right image showcases 𝐹 𝑗 , represented as a blue polyhedron. After eliminating the facets of the red polyhedron that are covered by the blue polyhedron, it becomes evident that only one facet of the red polyhedron remains not be excluded.

<!-- image -->

## 3.4 Build Grid and Parallel Computation of Intersections

In pursuit of heightened efficiency and improved parallel processing, we employ a spatial grid to partition the computational domain into discrete cells, which are geometrically rendered as cubes. This structured decomposition of space facilitates parallel execution by ensuring each cell can be processed independently.

Each cube, characterized by its six faces, has faces that align parallel to the standard three-dimensional planes: 𝑥𝑦 , 𝑦𝑧 , and 𝑧𝑥 . We designate the vertex with the smallest coordinates in a cell as 𝐶 0, and the one with the largest as 𝐶 1. The cell that has the smallest coordinate value for its 𝐶 0 vertex is termed 𝑆 𝑐𝑒𝑙𝑙 . The 𝐶 0 vertex coordinates at 𝑆 𝑐𝑒𝑙𝑙 represented as 𝑆𝐶 0 = ( 𝑆𝐶 0 𝑥 , 𝑆𝐶 0 𝑦 , 𝑆𝐶 0 𝑧 ) .

Fig. 14. From prior steps, the cumulative spatial coverage was derived from 𝑀𝑒𝑠ℎ 0 . Subsequent procedures aim to extract 𝑀𝑒𝑠ℎ 1 from the cumulative spatial coverage. When a mesh that necessitates an inward offset, the figure presented provides a comprehensive visual representation of the relationships among the grid built in this step, 𝑀𝑒𝑠ℎ 0 , and 𝑀𝑒𝑠ℎ 1 . The figure is divided into four distinct sections. Top-left: Showcases the grid in conjunction with 𝑀𝑒𝑠ℎ 0 . Bottom-left: Showcases the grid in conjunction with 𝑀𝑒𝑠ℎ 1 . Top-right: Features the grid in conjunction with a cross-sectional view, which is the intermediate space between 𝑀𝑒𝑠ℎ 0 and 𝑀𝑒𝑠ℎ 1 . Bottomright: Combines the cross-sectional view with 𝑀𝑒𝑠ℎ 1 .

<!-- image -->

The coordinates of 𝑆𝐶 0 are determined by the cumulative spatial coverage. To elaborate, 𝑆𝐶 0 𝑥 is the minimum x-coordinate across all vertices in the cumulative spatial coverage, while 𝑆𝐶 0 𝑦 and 𝑆𝐶 0 𝑧 are determined similarly.

Given a computational environment with a server equipped with 𝑐 cores, our goal is to employ a grid-based spatial decomposition such that the resultant cell count is proximity to 𝑛 × 𝑐 , where 2 ≤ 𝑛 ≤ 40. This proposed cell count serves as an aspirational benchmark indicative of optimal performance. To ensure a more evenly distributed computational load on each core, we assign multiple cells to each core. Therefore, our target is not merely 𝑐 , but rather 𝑛 × 𝑐 .

The edge length of each cell, is denoted as 𝐿 𝑔 . Leveraging 𝑆 𝑐𝑒𝑙𝑙 as the reference, we position new cells in ascending coordinate direction until the entirety of all spatial coverage is enclosed by the grid as shown in Fig. 14. Any given cell can be denoted by ( 𝑛 𝑥 , 𝑛 𝑦 , 𝑛 𝑧 ) , yielding the vertices 𝐶 0 and 𝐶 1:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Each cell, denoted generically as 𝑐𝑒𝑙𝑙 𝑥 , includes an associated array 𝐿𝑖𝑠𝑡 𝑥 that stores handles of certain facets of 𝑀𝑒𝑠ℎ 0. The spatial coverage of the facets in 𝐿𝑖𝑠𝑡 𝑥 overlap with the space designated by the cell. it's noteworthy that not all cells require instantiation, only those with a non-zero size for 𝐿𝑖𝑠𝑡 𝑥 are deemed necessary.

So, we had decomposed the computational domain into several cells. Each cell houses an AABB tree, denoted as 𝐴𝐴𝐵𝐵 𝑥 for the cell 𝑐𝑒𝑙𝑙 𝑥 .

Every facet of each spatial coverage polyhedron, excluding those eliminated in Step 𝑆 2, must be inserted into the AABB tree of its respective cell. If a facet spans multiple cells, it is individually inserted into the AABB trees of these cells.

The subsequent part of this step is the parallel computation of intersections among all facets. Distinct threads execute this for every cell. Within each cell, facet intersections are expedited via its inherent AABB tree. All intersections, both points and lines, serve as constraints. These constraints assist in the Delaunay triangulation of the facets, further subdividing them into smaller facets.

Once every facet undergoes this procedure, all spatial coverage facets are transformed into these subdivided facets. Relying on these subdivided facets, we reconstruct each spatial coverage. For a given spatial coverage 𝐹 𝑖 , its 𝑗 𝑡ℎ facet is labeled 𝑆 𝑖 𝑗 . Clearly, intersections between 𝑆 𝑖 𝑗 and other subdivided facets will not occur within the facet itself. Namely, when two facets intersect, the line of intersection invariably lies on the boundaries of both facets.

## 3.5 Extraction of 𝑀𝑒𝑠ℎ 1 from Cumulative Spatial Coverage

In this subsection, our goal is to extract 𝑀𝑒𝑠ℎ 1 from the cumulative spatial coverage as shown in Fig. 15. This procedure, which is inherently parallelizable, operates independently for each cell and is divided into two critical stages.

## Stage One: Ray-Based detection

Within the spatial domain of each cell, numerous instances of spatial coverage are found. For each instance, its facets, one of which is denoted as 𝑆 𝑖 𝑗 , are inserted into the cell's axis-aligned bounding box (AABB) tree. The AABB tree corresponding to the 𝑥 𝑡ℎ cell is represented as 𝐴𝐴𝐵𝐵 2 𝑥 . Distinct from the AABB tree utilized in Step 𝑆 4, referenced in subsection 3.4, 𝑆 𝑖 𝑗 is inserted into the AABB tree even if not contained within the spatial domain of the cell, as long as 𝐹 𝑖 traverses through or contained by the cell's space.

𝑆 𝑖 𝑗 is derived by the subdivision of a specific facet in Step 𝑆 4, as referenced in subsection 3.4, it follows that if the specific facet is excluded during the process delineated in Step 𝑆 2 (subsection 3.2), then 𝑆 𝑖 𝑗 can be definitively ruled out as a potential constituent of 𝑀𝑒𝑠ℎ 1.

For the facet 𝑆 𝑖 𝑗 that have not been excluded, a subsequent evaluation is required:

If the facet is situated within a single instance of spatial coverage, or if it aligns with the boundaries of two instances of distinct spatial coverage, with each coverage distinctly positioned on opposite sides of 𝑆 𝑖 𝑗 , then such a facet cannot be a component of 𝑀𝑒𝑠ℎ 1.

To streamline and expedite this evaluative process, we introduce a ray-casting methodology. For facets 𝑆 𝑖 𝑗 located in the spatial domain of 𝑐𝑒𝑙𝑙 𝑥 , a ray is emanated from the geometric centroid of 𝑆 𝑖 𝑗 , cast in a non-prescriptive direction to do fast intersection check with all facets saved in 𝐴𝐴𝐵𝐵 2 𝑥 . Then, Based on the intersections computed above, we can determine whether 𝑆 𝑖 𝑗 serves as a component of 𝑀𝑒𝑠ℎ 1. If 𝑆 𝑖 𝑗 is present in multiple cells, the intersections derived from these cells must be collectively considered to make a determination.

Fig. 15. The presented figure is divided into four distinct sections, proceeding from top to bottom. The transition from the first to the second section encapsulates the process delineated as the exclusion of facets via 1-ring neighbor spatial coverage, as detailed in subsection 3.3. The progression from the second to the third section underscores the primary utility of this step. Within the graphic, the green triangular facet denote those that couldn't be excluded in subsection 3.3. However, through the employment of an AABB tree and ray detection, these facets are effectively excluded during this step. The transition from the third to the fourth section illustrates the consolidation of all localized facets, culminating in the resultant offset surface represented as an unordered collection of triangular facets, or Triangle Soup.

<!-- image -->

Stage Two: check each 𝑆 𝑖 𝑗 internal/external in the 𝑀𝑒𝑠ℎ 0

Assuming an inward offsetting approach is taken. During extensive offsetting operations, certain facets might emerge outside of 𝑀𝑒𝑠ℎ 0. Clearly, these facets are not constituent elements of 𝑀𝑒𝑠ℎ 1 and thus necessitate exclusion. For 𝑀𝑒𝑠ℎ 0 that is both closed and manifold, the CGAL [Project 2023] library offers a robust mechanism to facilitate this determination. If not, using winding number[Jacobson et al. 2013] to realize internal and external judgment.

If we do the outward offset, just change the check if internal to check if external.

Post processing Upon completion of the aforementioned stages, a Triangle Soup is generated. Finally, show as Fig. 16 and Fig. 17, TetWild [Hu et al. 2018] are used to build mesh.

If the quality of the generated mesh are dissatisfied, it is recommended to carry out a final remeshing procedure. Several remeshing strategies are available, as highlighted in [Nivoliers et al. 2015], [Kobbelt 2004], [Hu et al. 2016], [Alliez et al. 2003], [Wang et al. 2018], [Botsch and Kobbelt 2004], and [Dunyach et al. 2013]. For instance, the methods presented in [Botsch and Kobbelt 2004] and [Dunyach et al. 2013] have been robustly implemented in the pmplibrary [Sieger and Botsch 2019].

Fig. 16. The figure elucidates the results of an inward offset. The top row delineates the initial mesh( 𝑀𝑒𝑠ℎ 0 ), the middle row showcases the resultant Triangle Soup, and the bottom row conveys the final offset mesh( 𝑀𝑒𝑠ℎ 1 ), as generated by TetWild.

<!-- image -->

## 4 EXPERIMENTAL RESULTS

We use the C++ programming language with GNU C++ compiler to implement the presented algorithm. The test device has 64GB memory, with the CPU of Intel i9-10900K, the OS of it is Ubuntu 20.04.

Using CGAL's Exact\_predicates\_exact\_constructions\_kernel to perform the intersection operation to avoid the precision problem of floating-point arithmetic.

The result data is presented in Table 2. This data is the execution of a variable offset operation where the offset distance ranges between 𝐷 𝑚𝑖𝑛 and 𝐷 𝑚𝑎𝑥 . The offset distance for each face is determined by the x-axis coordinate of its geometric center. Let the x-axis coordinates on both sides of the bounding box of 𝑀𝑒𝑠ℎ 0 be 𝑋 𝑚𝑖𝑛 and 𝑋 𝑚𝑎𝑥 respectively. If the x-axis coordinate of a face is denoted as 𝑋 , then its offset distance can be calculated as: 𝐷 𝑚𝑖𝑛 + 𝑋 -𝑋 𝑚𝑖𝑛 𝑋 𝑚𝑎𝑥 -𝑋 𝑚𝑖𝑛 ( 𝐷 𝑚𝑎𝑥 -𝐷 𝑚𝑖𝑛 ) .

Fig. 18 illustrates the mesh quality, while Fig. 19 displays the runtime of invariable mesh offset with varying offset distances. Fig.

12 · Hongyi Cao, Gang Xu *, Renshu Gu, Jinlan Xu, Xiaoyu Zhang, and Timon Rabczuk

|                           |        |             |                 |                 | Offset outward   | Offset outward   | Offset outward   | Offset inward   | Offset inward   | Offset inward   |
|---------------------------|--------|-------------|-----------------|-----------------|------------------|------------------|------------------|-----------------|-----------------|-----------------|
| model name                | vertex | face        | 𝐷 𝑚𝑖𝑛           | 𝐷 𝑚𝑎𝑥           | vertex           | face             | time             | vertex          | face            | time            |
| BendTube                  | 2843   | 5690        | 5.27888         | 10.55780        | 2001             | 4006             | 11.0             | 1854            | 3704            | 22.4            |
| CoverSlipCleaningHolder2b | 12653  | 25506       | 0.65915         | 1.31830         | 4081             | 8362             | 566.2            | 5397            | 10794           | 3079.3          |
| Hub                       | 6450   | 12980       | 1.65474         | 3.30948         | 5150             | 10380            | 108.9            | 2590            | 5188            | 78.7            |
| LowerControlArm           | 9317   | 18642       | 1.35551         | 2.71103         | 4172             | 8352             | 30.4             | 4309            | 8624            | 226.7           |
| anti_backlash_nut         | 6388   | 12784       | 0.36751         | 0.73501         | 2253             | 4514             | 38.7             | 2671            | 5350            | 83.1            |
| asm007                    | 10019  | 20042       | 0.00241         | 0.00482         | 2978             | 5960             | 82.2             | 3527            | 7058            | 550.0           |
| bamboo_pen                | 17832  | 35692       | 0.30289         | 0.60577         | 3549             | 7126             | 74.6             | 3720            | 7468            | 231.8           |
| bearing_plate             | 4907   | 9830        | 0.53945         | 1.07889         | 1435             | 2886             | 22.3             | 2092            | 4200            | 101.4           |
| beveled_shoulder_1        | 3836   | 7668        | 0.57826         | 1.15651         | 415              | 826              | 14.9             | 356             | 708             | 223.6           |
| beveled_shoulder_2        | 4106   | 8212        | 0.59610         | 1.19220         | 461              | 922              | 24.6             | 459             | 918             | 232.7           |
| blade                     | 14532  | 29060       | 0.00511         | 0.01023         | 3227             | 6450             | 48.0             | 4111            | 8218            | 42.8            |
| bladefem                  | 3621   | 7238        | 0.35489         | 0.70977         | 1235             | 2466             | 26.0             | 1039            | 2074            | 42.5            |
| bolt                      | 4752   | 9504        | 0.11209         | 0.22418         | 1370             | 2740             | 16.6             | 1720            | 3440            | 16.8            |
| bone                      | 4666   | 9328        | 0.00643         | 0.01287         | 1925             | 3846             | 16.4             | 1934            | 3864            | 15.4            |
| bone1                     | 1566   | 3128        | 0.01205         | 0.02409         | 1249             | 2494             | 6.1              | 1428            | 2852            | 6.0             |
| bozbezbozzel50K           | 24992  | 50000       | 1.39455         | 2.78910         | 11206            | 22418            | 1529.5           | 12755           | 25506           | 591.8           |
| buddha                    | 63264  | 126524      | 0.00444         | 0.00887         | 14247            | 28490            | 1010.4           | 15919           | 31834           | 220.7           |
| bunny                     | 11120  | 22236       | 0.00521         | 0.01042         | 4485             | 8966             | 56.9             | 4498            | 8992            | 36.1            |
| casting                   | 9484   | 19000       | 0.00839         | 0.01677         | 4485             | 9002             | 56.1             | 6192            | 12466           | 157.4           |
| casting2                  | 9513   | 19058       | 0.00837         | 0.01674         | 4251             | 8534             | 78.6             | 6452            | 12992           | 157.0           |
| cow2                      | 4315   | 8626        | 0.01350         | 0.02701         | 3769             | 7534             | 65.8             | 3696            | 7388            | 121.5           |
| cube_minus_sphere         | 2070   | 4136        | 0.01555         | 0.03111         | 1017             | 2030             | 6.0              | 1064            | 2124            | 24.9            |
| cup-tri                   | 5668   | 11340       | 0.38868         | 0.77737         | 3148             | 6300             | 33.0 17.7        | 3551            | 7106            | 26.7            |
| deckel                    | 5024   | 10060       | 0.38905         | 0.77809         | 2315             | 4642             |                  | 2079            | 4174            | 23.1            |
| delta_arm2_remesh         | 48668  | 97348       | 0.20622         | 0.41244         | 1545             | 3102             | 208.4            | 4439            | 8890            | 291.8           |
| des6                      | 3316   | 6632        | 0.87638         | 1.75275         | 1421             | 2842             | 28.3             | 1258            | 2516            | 56.7            |
| des7                      | 11061  | 22126       | 0.26276         | 0.52551         | 1571             | 3148             | 55.7             | 2186            | 4414            | 186.6           |
| eros100K                  | 50002  | 100000      | 0.62686         | 1.25373         | 11079            | 22154            | 227.5            | 10830           | 21656           | 189.7           |
| fandisk                   | 7250   | 14496       | 0.04936         | 0.09872         | 923              | 1842             | 28.3             | 1044            | 2084            | 81.1            |
| greek_sculpture           | 24994  | 50000       | 0.76476         | 1.52952         | 11683            | 23378            | 7021.3           | 10852           | 21716           | 3779.0          |
| gyroidpuzzle              | 13121  | 26298       | 0.35096         | 0.70191         | 8052             | 16160            | 38.1             | 8052            | 16160           | 170.3           |
| halved_oblique_scarf_2    | 5566   | 11128       | 0.45402         | 0.90805         | 565              | 1126             | 24.6             | 462             | 920             | 104.3           |
| halved_oblique_scarf_3    | 6164   | 12324       | 0.42987         | 0.85973         | 584              | 1164             | 34.8 30.7        | 631             | 1258            | 120.4           |
| hinge                     | 9544   | 19104       | 0.36614 0.01834 | 0.73228 0.03668 | 2341 2795        | 4698 5590        |                  | 2485            | 4986 7050       | 55.3 590.0      |
| impeller                  | 13063  | 26126 15078 |                 |                 | 759              |                  | 55.7             | 3525            |                 |                 |
| inlay_dovetail_3          | 7541   |             | 0.62424         | 1.24847         |                  | 1514             | 30.6             | 1367            | 2804            | 226.9           |
| lego                      | 10602  | 21200       | 0.56211         | 1.12422         | 2983             | 5962             | 51.2             | 2038            | 4072            | 267.3           |
| lion                      | 17198  | 34392       | 0.00093         | 0.00187         | 7340             | 14676            | 49.9             | 6838            | 13672           | 52.7            |
| lock_Lp                   | 5788   | 11584       | 1.04538         | 2.09076         | 2092             | 4192             | 24.2             | 3429            | 6882            | 252.1           |
| magalie_hand100K          | 50002  | 100000      | 0.48509         | 0.97018         | 7600             | 15204            | 208.7            | 7589            | 15174           | 168.4           |
| mech_piece                | 2089   | 4174        | 0.55146         | 1.10292         | 1062             | 2120             | 6.8              | 1017            | 2030            | 66.1            |
|                           | 8868   | 17752       |                 |                 |                  |                  |                  | 1970            |                 |                 |
| mid2Fem                   | 7567   | 15182       | 0.77463         | 1.54925         | 2001             | 4018             | 24.6             | 3096            | 3956            | 94.0            |
| motor_tail nasty_cheese   | 30816  | 62160       | 0.51505 0.44832 | 1.03010 0.89665 | 2761 23541       | 5570 47614       | 30.9 142.0       | 23749           | 6216 47950      | 85.6 679.0      |
| nugear                    | 17331  | 34658       | 0.21120         | 0.42239         | 2161             | 4318             | 78.5             | 2266            | 4528            | 130.3           |
| octa_flower_Lp            | 15143  | 30282       | 0.14846         | 0.29692         | 1712             | 3420             | 66.1             | 2038            | 4072            | 632.3 105.3     |
| part2                     | 5341   | 10710       | 4984.49         | 9968.99         | 2968             | 5964             | 38.1             | 3319 2765       | 6670            | 284.5           |
| part20k                   | 13055  | 26118       | 0.02221         | 0.04442         | 1866             | 3740             | 114.5            | 2336            | 5538            | 105.3           |
| pinion                    | 21439  | 42878       | 0.00925         | 0.01851         | 1613             | 3226             | 54.4             |                 | 4672            |                 |

Table 2. The test data was obtained from the website [https://www.quadmesh.cloud][Pietroni et al. 2021]. We conducted a variable offset operation where the offset distance uniformly ranged between 𝐷 𝑚𝑖𝑛 and 𝐷 𝑚𝑎𝑥 . The time presented in the table signifies the computational expense, and it is measured in seconds.

Fig. 17. The figure elucidates the results of an outward offset. The top row delineates the initial mesh( 𝑀𝑒𝑠ℎ 0 ), the middle row showcases the resultant Triangle Soup, and the bottom row conveys the final offset mesh( 𝑀𝑒𝑠ℎ 1 ), as generated by TetWild.

<!-- image -->

20, Fig. 21, Fig. 22, Fig. 23, and Fig. 24 provide visual representations corresponding to results from our experiments.

From Fig. 20, it can be observed that our method, when applying a variable outward offset to the mesh, retains sharp features. Furthermore, when executing short-distance offsets, the mesh remains intact without fragmentation.

Similarly, Fig. 21 indicates that when our method applies a variable inward offset to the mesh, it can preserve sharp features without causing any breakage in the mesh.

Currently, our demonstration software does not incorporate procedures for handling instances where 𝑀𝑒𝑠ℎ 0 is not closed or not manifold. Consequently, for such meshes, it is imperative to first apply corrective algorithms before executing our code. Potential mesh-repair methodologies can be found in the following studies: [Attene 2010], [Attene et al. 2013], [Cherchi et al. 2020], and [Zhou et al. 2016].

## 5 CONCLUSION AND FUTURE WORK

In this paper, a parallel feature-preserving mesh offsetting framework with variable distance is proposed. Our method can efficiently generate an offsetting mesh with smaller mesh size, and also can achieve high quality without gaps, holes, and self-intersections. The proposed method has been tested on the quadmesh dataset to show the robustness and efficiency.

Fig. 18. The figure is structured as follows: The topmost row portrays the initial mesh, denoted as 𝑀𝑒𝑠ℎ 0 . Subsequently, the intermediate row presents the concluding offset mesh, represented as 𝑀𝑒𝑠ℎ 1 . The final row serves as an illustrative representation of mesh quality. Facets shaded in a blue hue signify superior quality, indicating a favorable aspect ratio, while those leaning towards a red coloration represent quality with a less desirable aspect ratio. It is noteworthy that a significant proportion of the triangular facets generated via TetWild demonstrate high-quality characteristics.

<!-- image -->

Fig. 19. For simplicity, we used an invariable mesh offset as an example. For each model, we conducted multiple tests with varying offset distances to measure the time required. The horizontal axis denotes the offset distance, while the vertical axis represents time, measured in seconds. It is discernible that as the offset distance incrementally increases, the efficiency of our algorithm markedly diminishes. This decrease in performance can be fundamentally attributed to the intricate triangular facet intersections that arise with extended offset distances.

<!-- image -->

There are several future research directions in this topic. For input meshes with extremely poor quality requiring inward offsets, our method can still produce offset surfaces. However, in some regions, feature reconstruction should be improved. This challenge will be addressed by increasing the mesh density locally in the future. Furthermore, we also would like to to generate the offset mesh directly from spatial coverage without relying on TetWild.

Fig. 20. The variable offset outward, there are six groups in total, and the first mesh in each group is the green mesh, representing the initial triangular mesh. The second is a cross-sectional view of the resulting shell-like structure, and the third shows the outer surface.

<!-- image -->

## REFERENCES

Pierre Alliez, Eric Colin De Verdire, Olivier Devillers, and Martin Isenburg. 2003. Isotropic surface remeshing. In 2003 Shape Modeling International. IEEE, 49-58.

Nina Amenta, Sunghee Choi, and Ravi Krishna Kolluri. 2001. The power crust, unions of balls, and the medial axis transform. Computational Geometry 19, 2-3 (2001), 127-153.

Marco Attene. 2010. A lightweight approach to repairing digitized polygon meshes. The visual computer 26, 11 (2010), 1393-1406.

Marco Attene, Marcel Campen, and Leif Kobbelt. 2013. Polygon mesh repairing: An application perspective. ACM Computing Surveys (CSUR) 45, 2 (2013), 1-33.

Mario Botsch and Leif Kobbelt. 2004. A remeshing approach to multiresolution modeling. In Proceedings of the 2004 Eurographics/ACM SIGGRAPH symposium on Geometry processing . 185-192.

Alan Brunton and Lubna Abu Rmaileh. 2021. Displaced signed distance fields for additive manufacturing. ACM Transactions on Graphics (TOG) 40, 4 (2021), 1-13.

Marcel Campen and Leif Kobbelt. 2010. Polygonal boundary evaluation of minkowski sums and swept volumes. 29, 5 (2010), 1613-1622.

Lufeng Chen, Man-Fai Chung, Yaobin Tian, Ajay Joneja, and Kai Tang. 2019a. Variabledepth curved layer fused deposition modeling of thin-shells. Robotics and ComputerIntegrated Manufacturing 57 (2019), 422-434.

Zhen Chen, Daniele Panozzo, and Jeremie Dumas. 2019b. Half-space power diagrams and discrete surface offsets. IEEE Transactions on Visualization and Computer Graphics 26, 10 (2019), 2970-2981.

Gianmarco Cherchi, Marco Livesu, Riccardo Scateni, and Marco Attene. 2020. Fast and robust mesh arrangements using floating-point arithmetic. ACM Transactions on Graphics (TOG) 39, 6 (2020), 1-16.

Marion Dunyach, David Vanderhaeghe, Loïc Barthe, and Mario Botsch. 2013. Adaptive remeshing for real-time mesh deformation. In Eurographics 2013 . The Eurographics Association.

Samuele Giraudo. 2015. Combinatorial operads from monoids. Journal of Algebraic Combinatorics 41 (2015), 493-538.

Peter Hachenberger. 2009. Exact Minkowksi sums of polyhedra and exact and efficient decomposition of polyhedra into convex pieces. Algorithmica 55, 2 (2009), 329-345.

Kaimo Hu, Dong-Ming Yan, David Bommes, Pierre Alliez, and Bedrich Benes. 2016. Error-bounded and feature preserving surface remeshing with minimal angle improvement. IEEE transactions on visualization and computer graphics 23, 12 (2016), 2560-2573.

Yixin Hu, Qingnan Zhou, Xifeng Gao, Alec Jacobson, Denis Zorin, and Daniele Panozzo. 2018. Tetrahedral meshing in the wild. ACM Trans. Graph. 37, 4 (2018), 60-1.

Alec Jacobson, Ladislav Kavan, and Olga Sorkine-Hornung. 2013. Robust inside-outside segmentation using generalized winding numbers. ACM Transactions on Graphics (TOG) 32, 4 (2013), 1-12.

Wonhyung Jung, Hayong Shin, and Byoung K Choi. 2004. Self-intersection removal in triangular mesh offsetting. Computer-Aided Design and Applications 1, 1-4 (2004), 477-484.

Leif P Kobbelt, Mario Botsch, Ulrich Schwanecke, and Hans-Peter Seidel. 2001. Feature sensitive surface extraction from volume data. In Proceedings of the 28th annual conference on Computer graphics and interactive techniques . 57-66.

Mario Botsch Leif Kobbelt. 2004. A remeshing approach to multiresolution modeling. (2004), 185-92.

Louisa Lam, Seong-Whan Lee, and Ching Y Suen. 1992. Thinning methodologies-a comprehensive survey. IEEE Transactions on Pattern Analysis &amp; Machine Intelligence 14, 09 (1992), 869-885.

Pan Li, Bin Wang, Feng Sun, Xiaohu Guo, Caiming Zhang, and Wenping Wang. 2015. Q-mat: Computing medial axis transform by quadratic error minimization. ACM Transactions on Graphics (TOG) 35, 1 (2015), 1-16.

Shengjun Liu and Charlie CL Wang. 2010. Fast intersection-free offset surface generation from freeform models with triangular meshes. IEEE Transactions on Automation Science and Engineering 8, 2 (2010), 347-360.

Takashi Maekawa. 1999. An overview of offset curves and surfaces. Computer-Aided Design 31, 3 (1999), 165-173.

Wenlong Meng, Shuangmin Chen, Zhenyu Shu, Shi-Qing Xin, Hongbo Fu, and Changhe Tu. 2018. Efficiently computing feature-aligned and high-quality polygonal offset surfaces. Computers &amp; Graphics 70 (2018), 62-70.

Vincent Nivoliers, Bruno Lévy, and Christophe Geuzaine. 2015. Anisotropic and feature sensitive triangular remeshing using normal lifting. J. Comput. Appl. Math. 289 (2015), 225-240.

Darko Pavić and Leif Kobbelt. 2008. High-resolution volumetric computation of offset surfaces with feature preservation. In Computer Graphics Forum , Vol. 27. Wiley Online Library, 165-174.

Nico Pietroni, Stefano Nuvoli, Thomas Alderighi, Paolo Cignoni, and Marco Tarini. 2021. Reliable feature-line driven quad-remeshing. 40, 4 (2021), 1-17.

The CGAL Project. 2023. CGAL User and Reference Manual (5.6 ed.). CGAL Editorial Board. https://doc.cgal.org/5.6/Manual/packages.html

Jaroslaw R Rossignac and Aristides AG Requicha. 1986. Offsetting operations in solid modelling. Computer Aided Geometric Design 3, 2 (1986), 129-148.

Daniel Sieger and Mario Botsch. 2019. The Polygon Mesh Processing Library. http://www.pmp-library.org.

B. Stellato, G. Banjac, P. Goulart, A. Bemporad, and S. Boyd. 2020. OSQP: an operator splitting solver for quadratic programs. Mathematical Programming Computation 12, 4 (2020), 637-672. https://doi.org/10.1007/s12532-020-00179-2

Feng Sun, Yi-King Choi, Yizhou Yu, and Wenping Wang. 2013. Medial meshes for volume approximation. arXiv preprint arXiv:1308.3917 (2013).

Feng Sun, Yi-King Choi, Yizhou Yu, and Wenping Wang. 2015. Medial meshes-a compact and accurate representation of medial axis transform. IEEE transactions on visualization and computer graphics 22, 3 (2015), 1278-1290.

Andrea Tagliasacchi, Thomas Delame, Michela Spagnuolo, Nina Amenta, and Alexandru Telea. 2016. 3d skeletons: A state-of-the-art report. 35, 2 (2016), 573-597.

Tim Van Hook. 1986. Real-time shaded NC milling display. ACM SIGGRAPH Computer Graphics 20, 4 (1986), 15-20.

Charlie CL Wang and Yong Chen. 2013. Thickening freeform surfaces for solid fabrication. Rapid Prototyping Journal (2013).

Charlie CL Wang and Dinesh Manocha. 2013. GPU-based offset surface computation using point samples. Computer-Aided Design 45, 2 (2013), 321-330.

Yiqun Wang, Dong-Ming Yan, Xiaohan Liu, Chengcheng Tang, Jianwei Guo, Xiaopeng Zhang, and Peter Wonka. 2018. Isotropic surface remeshing without large and small angles. IEEE transactions on visualization and computer graphics 25, 7 (2018), 2430-2442.

Yue Xie and Xiang Chen. 2017. Support-free interior carving for 3D printing. Visual Informatics 1, 1 (2017), 9-15.

Qingnan Zhou, Eitan Grinspun, Denis Zorin, and Alec Jacobson. 2016. Mesh arrangements for solid geometry. ACM Transactions on Graphics (TOG) 35, 4 (2016), 1-15. Daniel Zint, Nissim Maruani, Mael Rouxel-Labbe, and Pierre Alliez. 2023. FeaturePreserving Offset Mesh Generation from Topology-Adapted Octrees. (2023), 12.

Fig. 21. The variable offset inward, there are six groups in total, and the first mesh in each group is the green mesh, representing the initial triangular mesh. The second is a cross-sectional view of the resulting shell-like structure, and the third shows the inner surface and the cross-sectional view together.

<!-- image -->

Fig. 22. The middle one is the initial mesh, and the two pictures on the left are the results of the long-distance inward offset and the short-distance inward offset obtained by our algorithm respectively. The two figures on the right show the results of short-distance outward offset and long-distance outward offset obtained by our algorithm respectively.

<!-- image -->

## 16 · Hongyi Cao, Gang Xu *, Renshu Gu, Jinlan Xu, Xiaoyu Zhang, and Timon Rabczuk

Fig. 23. The first column is the initial mesh, the second column is the outward offset based on the distance field, the third column is the result generated based on rays and dexels, and the fourth column is our method., we can find that the method based on sampling will be distorted due to ambiguity.

<!-- image -->

A Parallel Feature-preserving Mesh Variable Offsetting Approach Based On Dynamic Programming · 17

Fig. 24. The meaning of four column is the same as the figure23. From the figure, a notable distinction between the result obtained using our method and those derived from existing techniques is the preservation of sharp features. Our approach adeptly replicates right-angle edges and vertices.

<!-- image -->