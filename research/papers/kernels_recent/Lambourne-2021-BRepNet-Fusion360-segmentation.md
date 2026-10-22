## BRepNet: A topological message passing system for solid models

Joseph G. Lambourne Autodesk Research

Aditya Sanghi Autodesk Research Karl D.D. Willis Autodesk Research Peter Meltzer UCL, Computer Science

## Abstract

Boundary representation (B-rep) models are the standard way 3D shapes are described in Computer-Aided Design (CAD) applications. They combine lightweight parametric curves and surfaces with topological information which connects the geometric entities to describe manifolds. In this paper we introduce BRepNet, a neural network architecture designed to operate directly on B-rep data structures, avoiding the need to approximate the model as meshes or point clouds. BRepNet defines convolutional kernels with respect to oriented coedges in the data structure. In the neighborhood of each coedge, a small collection of faces, edges and coedges can be identified and patterns in the feature vectors from these entities detected by specific learnable parameters. In addition, to encourage further deep learning research with B-reps, we publish the Fusion 360 Gallery segmentation dataset. A collection of over 35,000 B-rep models annotated with information about the modeling operations which created each face. We demonstrate that BRepNet can segment these models with higher accuracy than methods working on meshes, and point clouds.

## 1. Introduction

Boundary representation (B-rep) models are the de facto standard for describing 3D objects in commercial Computer Aided Design (CAD) software. They consist of collections of trimmed parametric surfaces along with the adjacency relationships between them [44]. Prismatic shapes can be represented using lightweight primitive curves and surfaces while free-form objects can be defined using NURBS [33]. Although this makes the representation both compact and expressive, the complexity of the data structures and limited availability of labelled datasets has presented a high barrier to entry for researchers.

The problem of segmenting B-rep models, based on Pradeep Kumar Jayaraman Autodesk Research Hooman Shayani Autodesk Research learned patterns, is of particular interest as it allows the automation of many laborious manual tasks in CAD, Computer Aided Engineering (CAE) and Computer Aided Process Planning (CAPP) [6, 45, 1, 38]. Currently these require a user to repeatedly select groups of faces and/or edges as input for the modeling or manufacturing operation. Examples include model simplification in preparation for finite element analysis [12] and segmenting a model according to the manufacturing process or machining toolpath strategy required to make the object [1, 45].

Figure 1: BRepNet convolutional kernels are defined with respect to topological entities called coedges (dashed arrows). Feature vectors from a small collection of faces (grey), edges (black) and coedges (blue) adjacent to each coedge (red) are multiplied by the learnable parameters in the kernel. The hidden states arising from the convolution can then be pooled to perform face segmentation.

<!-- image -->

In addition, parametric feature history is often lost when models are exchanged between different CAD applications [23] and many commercial CAD systems use segmentation algorithms to recover this information [5, 13].

Although attempts were made in the 90s to apply neural networks to the task of B-rep segmentation [19, 11, 30, 14, 41, 38], the absence of machine learning frameworks and large labelled datasets caused progress to stall until very recently [20, 10]. In this paper we introduce BRepNet, a novel neural network architecture designed specifically to operate directly on the faces and edges of B-rep data structures and take full advantage of the topological relationships between them. In addition, we hope to revitalize interest in the problem of B-rep segmentation with the publication of the Fusion 360 Gallery segmentation dataset. For the first time we provide a collection of over 35,000 3D models, in multiple representations, annotated with segmentation labels revealing the modeling operations used to create them.

The BRepNet approach is motivated by the observation that in convolutional neural networks for image processing, the weights operate on pixels with known locations within the filter window. A similar arrangement can be achieved with B-reps, where a small collection of faces, edges and coedges can be identified at well defined locations relative to each coedge in the data structure (see Figure 1). Feature vectors can be extracted from these neighbouring entities and concatenated in a known order, allowing convolution to take place as a matrix/vector multiplication [18, 21]. As in image convolution, specific entities relative to each coedge map to specific learnable parameters in our convolutional kernels, allowing patterns in the input data to be easily recognized [32, 9]. The key contributions are as follows:

- We introduce BRepNet, a network architecture using a novel convolution technique which takes full advantage of the topological information the B-rep stores.
- We publish the Fusion 360 Gallery segmentation dataset that contains over 35,000 segmented 3D models in B-rep, mesh, and point cloud format.
- We provide experimental results on the Fusion 360 Gallery segmentation task, including ablation studies and comparisons to other representations and methods.

Our results demonstrate that direct use of B-rep data solves the Fusion 360 Gallery segmentation problem with higher performance and parameter efficiency than other techniques based on point cloud and mesh representations.

## 2. Related work

Historically, the task of B-rep segmentation has focused on the detection of form features (connected regions of a model with a characteristic shape or pattern with some significance [38]). Feature detection has been an active area of research since the mid 1970s [19], with a range of different heuristic approaches investigated [25, 40, 3, 22, 37, 29].

Early neural networks. Neural networks were first employed by Prabhakar et al . [34] with a number of extensions and refinements made over the years [31, 14, 41]. In these early works the B-rep structure is first converted to a face adjacency graph with node features extracted from the Brep faces and attributes for the arcs extracted from the B-rep edges. Heuristics are then used to break the graph into small connected components which are passed to the networks individually. These techniques were limited by the computer power of the time and so the networks see only a small part of the B-rep at once.

Voxels. Feature detection methods based on voxels [7, 46] offer some advantages for manufacturability analysis, however the cubic storage complexity puts severe limitations on the size of geometric features which can be detected. As CAD models often contain small but important features, the applicability of these techniques with current GPU hardware is quite limited.

Point clouds. Point cloud segmentation has shown excellent results in recent years [35, 36, 43], but typically requires a large number of points to be uniformly sampled from the (B-rep) objects' surface. Faces with small areas can easily be under-sampled and incorrectly classified.

Meshes. Triangle meshes are another important representation for 3D objects, and a number of authors have proposed convolution strategies which operate on them [42, 18, 9, 28]. MeshCNN [18] operates on the edges of a triangle mesh with convolution carried out by aggregating information from the five edges of two adjacent triangles onto the central edge. Liu et al . [28] introduce a convolution scheme which operates on directed triangle edges and use it to generate neural network conditioned subdivision surfaces. Although the data structures for triangle meshes are simple, converting B-reps to high quality manifold meshes requires special meshing procedures. By working directly on the original B-rep topology we can avoid the requirement to generate good quality meshes and operate directly on a more compact representation.

Graphs. B-rep model segmentation can also be viewed as a node classification problem on graphs. Two concurrent unpublished works have applied graph convolution approaches to B-rep segmentation [20, 10]. Jayaraman et al . [20] uses convolution layers to create input features from grids of 3D points and normal vectors, while Cao et al . [10] uses only planar faces which can be directly represented as feature vectors of length 4. In both cases the Brep data structure is translated to a face adjacency graph which causes some information about relative topological locations of nearby entities to lost.

Figure 2: A) B-rep topology comprises faces, edges, loops, coedges and vertices. B) Starting from a given coedge (red), the topology can be traversed by following a sequence of instructions which indicate which entity to move to in the next hop. The instruction sequence { mate, next, mate, face } is illustrated. C) The walks from the red coedge to some neighbouring entities are described in terms of products of the incidence matrices N , P , M , F and E .

<!-- image -->

## 3. Method

## 3.1. B-rep data structures

Industrial CAD packages have internal data structures which are similar to the partial entity structure described by Lee et al . [27]. These structures support the modeling of 2-dimensional manifolds, 3-dimensional volumes and even non-manifold complexes which can arise as intermediate states in boolean operations [44].

A B-rep comprises of faces, edges, loops, coedges and vertices (Figure 2a). A face is a connected region of the model's surface which may have internal holes [15]. An edge defines the curve where two faces meet and a vertex defines the point where edges meet. Faces have an underlying parametric surface which is divided into visible and hidden regions by a series of boundary loops.

Each loop consists of a doubly linked list of directed edges called coedges, topological entities which are used to represent the adjacency relationships in the B-rep [27]. A coedge stores pointers to the next and previous coedge in the loop, its adjacent or 'mating' coedge, its parent face and parent edge. In this work we consider only closed and manifold B-reps where each coedge has exactly one mating coedge, providing sufficient information for the edges in the structure to be traversed in the same way as in the winged edge [8] and QuadEdge [17] data structures.

## 3.2. Topological walks

By following the pointers which the coedges store, we can walk from a given coedge on the B-rep to entities in its neighborhood. The choice of which pointer to follow at each hop can be thought of as a sequence of instructions which will take us from some starting coedge to a destination coedge. From there we can optionally make one final jump to its owner edge or face. This sequence of instructions defines a topological walk.

An example of a simple topological walk for the instruction sequence: { mate, next, mate, face } is shown in Figure 2b. The starting coedge is shown in red and the coedges traversed during the walk are shown in blue.

For a set of B-rep faces f = { f 1 , f 2 , ..., f | f | } , edges e = { e 1 , e 2 , ..., e | e | } , and coedges c = { c 1 , c 2 , ..., c | c | } , geometric information can be extracted and used to build three input feature matrices X f ∈ R | f |× p , X e ∈ R | e |× q and X c ∈ R | c |× r for the face features, edge features and coedge features respectively as described in Section 3.3.

The next, previous and mating adjacency relationships between coedges can be written as three matrices:

<!-- formula-not-decoded -->

Here N ij = 1 indicates that c j is the next coedge in the loop from c i and M ij = 1 when coedge c j is the mate of coedge c i . As each coedge has exactly one next, previous and mating coedge, these matrices simply define permutations on the list of coedges in the B-rep. Also we can see that P = N -1 = N T . A matrix defining a topological walk between two coedges can then be built by multiplying N , P and M in the sequence in which the next , previous and mate instructions appear in the walk (Figure 2c).

The relationships between a coedge and its parent face and parent edge can also be represented using incidence matrices F ∈ { 0 , 1 } | c |×| f | and E ∈ { 0 , 1 } | c |×| e | . Here F ij = 1 indicates that coedge c i is in a loop around face f j and E ij = 1 indicates that coedge c i belongs to edge e j .

The transform Ψ = FX f allow us to construct a matrix Ψ ∈ R | c |× p by copying the i th row of the matrix of face features X f to the j th row of Ψ for each coedge c j with parent face f i . The matrix E works in a similar way for edges. Topological walks which terminate on faces or edges can then be represented in matrix form by multiplying the matrix for the walk over the coedges by E or F .

## 3.3. Input feature extraction

Geometric feature information from the faces, edges and coedges of the B-rep are passed into the network in the feature matrices X f , X e and X c . One approach to the extraction of geometric features from B-rep faces is given by [20], where grids of points and normal vectors are sampled from the parametric surface geometry and compressed into feature vectors using a CNN. In this work we investigate whether the Fusion 360 Gallery segmentation problem can be solved without providing the network with any coordinate information, instead using only a small amount of extremely concise information from the B-rep data structure. Using coordinate free input features has the advantage that they are invariant to translation and rotation and protects the intellectual property of CAD operators by not reveling the model geometry, while still allowing the network to perform useful tasks.

For face features, the network is given a one-hot vector encoding of the possible surface types (plane, cylinder, cone, sphere, torus). One additional value is used to indicate a rational NURBS surface [33]. In the case of non-rational B-splines all these values will be zero. We also provide the network with the area of each face. For edge features we provide a one-hot vector encoding of the possible kinds of edge geometry (line, circle, ellipse, helix, intersection curve). We encode edge convexity in three one-hot values (concave edge, convex edge, smooth edge). One additional flag indicates if an edge forms a closed loop. Finally the edge length is added. For coedges, the network is passed a single flag indicating whether the direction of the coedge is the same as the direction of the parametric curve of the edge. The input features are standardized over the training set and the same scaling applied to the validation and test sets. More detail is in the supplementary material.

## 3.4. Convolution

Convolutional kernels in BRepNet are defined relative to the coedges of the B-rep. As noted by Lui et al . [28], because coedges are directed this removes the ambiguity between the faces to the left and right of a coedge and avoids the need to aggregate features using symmetric functions as in [18]. The relative topological locations between a starting coedge and the faces, edges and coedges which will take part in a convolution are defined by topological walks. Each walk can be expressed in matrix form by multiplying the matrices N , P , M , F and E in the order in which the next, previous, mate, face or edge instructions must be executed. An example of a collection of faces, edges and coedges which can be used in a BRepNet kernel is shown in Figure 2c. The products of the matrices required to reach each of the destination entities are marked. The matrices encoding the walks to faces, edges and coedges are arranged in three lists K f , K e and K c receptively.

A forward pass through the network proceeds as follows. We start by initialising the matrices H ( 0 ) f = X f , H ( 0 ) e = X e and H ( 0 ) c = X c . These three matrices are then passed through a number of the convolution units as shown in Figure 3. Following convolution unit t , the hidden state matrices H ( t ) f , H ( t ) e and H ( t ) c are generated. The width of these hidden states is defined by a hyper-parameter s . For face classification tasks a final convolution unit generates only matrix H ( T + 1 ) f ∈ R | f |× u which are the per-face segmentation scores for each of the u classes.

Figure 3: The BRepNet network architecture. Input feature vectors from faces, edges and coedges are passed through a stack of T convolution units to generate hidden states H ( t ) f , H ( t ) e and H ( t ) c . A final convolution unit generates only the segmentation scores for faces.

<!-- image -->

Inside each convolution unit three processes take place. First we build up a matrix Ψ where

<!-- formula-not-decoded -->

This procedure populates the i th row of Ψ with the concatenated hidden state vectors of the entities defined by the kernel with starting coedge c i .

Each row of Ψ is then passed through a multi-layer perceptron (MLP) with parameters Θ ( t ) and ReLU nonlinearities. The input to the first layer of the MLP depends on the number of columns of Ψ while all other MLP layers have a size 3 times the width of the hidden states s .

Following the MLP, we generate a matrix Z ∈ R | c |× 3 s . The rows of Z are associated with coedges in the B-rep. A simple architecture would pass the single matrix Z to the subsequent convolution units, however we observe that this simple approach gives poor performance on B-rep models where faces have multiple loops (e.g. a face with a hole). In this case the edges of the B-rep do not form a connected graph and information cannot flow between the loops of multi-loop faces. The performance of the network is greatly enhanced by pooling information from the coedges onto their parent faces and edges in each convolution unit. This allows information to flow from the coedges in one loop onto the parent face, making it accessible to coedges in another loop in subsequent layers. To apply this pooling the matrix Z is first split into 3 sub-matrices of size | c | × s .

Figure 4: An overview of 3D models from the Fusion 360 Gallery segmentation dataset (left). Each 3D model is labeled according to the CAD modeling operations used to create it (right).

<!-- image -->

<!-- formula-not-decoded -->

H ( t + 1 ) c is the matrix of hidden states for the coedges in the next layer, which requires no further processing. To build the i th row of the matrix H ( t + 1 ) f we apply element wise max pooling over the rows of Z f corresponding to the coedges with parent face f i . The matrix H ( t + 1 ) e is built in a similar way by max pooling the pairs of rows of Z e corresponding to coedges with the same parent edge.

A diagram showing the matrices and operations performed in each convolution unit are shown in Figure 3.

## 3.5. Face classifications

The per-face segmentation scores for each class u i can then be calculated as follows. In the final convolution unit, the last layer of the MLP has just | u | neurons and produces only the matrix Z f ∈ R | c |×| u | . The matrix of segmentation scores, H ( T +1) f ∈ R | f |×| u | , is then built by pooling the coedge feature vectors onto their parent faces as before. A cross-entropy loss can then be used to train the network.

## 4. Fusion 360 Gallery segmentation dataset

In this section we introduce, to our knowledge for the first time, a dataset containing segmentation information for B-rep models and the corresponding triangle meshes and point clouds. The Fusion 360 Gallery segmentation dataset is produced from designs submitted by users of the CADpackage Autodesk Fusion 360 and is segmented based on the CAD modeling operations used to create each face. This modeling history information is not available in existing datasets [24, 47, 2] and goes beyond what was designed, providing insights into how people design 3D models.

The segmentation dataset contains a total of 35,858 3D models with per-face, per-triangle, and per-point segment labels provided for the B-rep, mesh and point cloud representations (Figure 4, left). For segmentation we use a small subset of the most common CAD modeling operations: extrude , chamfer , fillet , and revolve . In order to create a segmentation which contains as much information as possible about the CAD modeling operations, we subdivide extrude operations into additive (i.e. adding) and subtractive (i.e. cutting) extrusion operations, and further divide the faces created by extrude and revolve into side and end faces. This gives a set of eight labels for each face: ExtrudeSide , ExtrudeEnd , CutSide , CutEnd , Fillet , Chamfer , RevolveSide , and RevolveEnd (Figure 4, right). Further details on the dataset are provided in the supplementary material.

## 5. Experiments

In this section we perform experiments to examine the following important network capabilities. First we show that loop ordering information is useful for solving a B-rep segmentation problem. We study how performance is affected when the incidence relations in the matrices N and P are withheld from our architecture and explore a range of kernel configurations to find which one is optimal. We ana- lyze the features passed to the network, identify the key information used to generate the segmentation and provide insights on why these features are important. To demonstrate the advantages of learning based approaches we compare with a traditional rule-based feature recognition algorithm. Next, we compare BRepNet performance against an EdgeConditioned Convolution (ECC) graph network [39, 16]. This architecture is chosen as it can ingest the same input features as BRepNet, but employs no special techniques to exploit the manifold nature of the B-rep. As such this comparison shows the gains which can be made when specific kernel weights operate on specific neighbouring nodes.

Figure 5: Different BRepNet kernel configurations (left) for which the accuracy and IoU are compared (right). The accuracy and IoU for the Edge-Conditioned Convolution (ECC) graph network [39] discussed in Section 5.5 is also shown. MLP width s is adjusted to keep the total number of parameters | Θ | in the network to around 360k.

|                        | Kernel                               |   s | &#124; Θ &#124;   | Accuracy%    | IoU%         |
|------------------------|--------------------------------------|-----|-------------------|--------------|--------------|
|                        | Simple edge                          | 120 | 359k              | 91.02 ± 0.20 | 74.03 ± 0.55 |
|                        | Asymmetric                           | 120 | 359k              | 91.66 ± 0.17 | 75.01 ± 0.54 |
| Asymmetric Simple Edge | Asymmetric++ Asymmetric+ Asymmetric+ | 113 | 358k              | 91.82 ± 0.13 | 75.06 ± 0.71 |
|                        | Asymmetric++                         | 107 | 359k              | 92.05 ± 0.07 | 75.69 ± 0.38 |
|                        | Winged edge                          |  84 | 359k              | 92.52 ± 0.15 | 77.10 ± 0.54 |
|                        | Winged edge+                         |  75 | 357k              | 92.50 ± 0.15 | 76.86 ± 0.47 |
|                        | Winged edge++                        |  63 | 358k              | 92.50 ± 0.12 | 77.14 ± 0.44 |
| Winged edge            | Winged edge++ Winged edge+ ECC       | 153 | 360k              | 90.36 ± 0.23 | 72.08 ± 0.50 |

Finally, as B-rep models can be converted to meshes and point clouds we compare against networks using these representations. We investigate the advantages of working directly with the B-rep data structure and challenges of using approximations to the true geometry.

The data is divided into a 70/15/15% train/validation/test split. In each of the experiments above the networks are trained for 50 epochs and the weights with the smallest validation loss are recorded. The performance of these trained models on the test set is then evaluated. The reported values are the average over 10 runs with different random seeds and the error bars are computed as the standard deviation.

## 5.1. Evaluation metrics

We use accuracy and intersection over union (IoU) to evaluate network performance. Due to data imbalance in the Fusion 360 Gallery segmentation dataset, the IoU metric is useful for providing insight into the performance on the rarer classes. Rather than computing IoU values for individual B-rep models and then averaging as in [35], we consider the entire collection of all faces in the test set at once. This methodology is referred to as 'part IoU' in [26] and avoids the special case when a B-rep model has no faces, either predicted or in the ground truth, for a given class.

## 5.2. Choice of kernel

The BRepNet architecture provides a flexible framework for defining the relative topological locations of the entities which make up a convolutional kernel. Here we study how the choice of these entities affects network performance. Figure 5, left shows the range of different kernel configurations used in the experiments. The corresponding topological walks are included in the supplementary material. As the number of parameters in the MLP is dependent on the number of entities in the kernel, we adjust the hyper-parameter s to keep the total number of network parameters as close as possible to 360k. This decouples the effect of aggregating information from a wider region of the B-rep and the effects of increasing network capacity. For each kernel configuration we train a network with two convolutional units, each with a two layer MLP. Figure 5, right shows the accuracy and IoU for each kernel configuration along with the values of s and the corresponding number of parameters.

The ability of the network to exploit loop ordering information can now be evaluated. The 'simple edge' and 'asymmetric' kernels are carefully chosen to have the same number of faces, edges and coedges, allowing them to be compared directly without any adjustments in the MLP width. The 'simple edge' arrangement contains only an edge and its two adjacent faces and coedges, giving it information similar to a face adjacency graph, but withholding information regarding the order in which coedges are arranged around the loop. The 'asymmetric' kernel includes the next coedge in the loop in place of the mating coedge, allowing the kernel to observe patterns like contiguous smooth edges. We observe 0 . 98% improvement in IoU when moving from the 'simple edge' to 'asymmetric' kernels. While this improvement is less than 2 standard deviations, a Welch's unequal variances t-test gives a P value of 0 . 0012 for this result, indicating that the coedge ordering information is useful for the segmentation task.

The 'winged edge' kernel configuration is similar to the half-flaps described by Liu et al . [28]. It achieves an accuracy of 92 . 52% and an IoU of 77 . 10% , over 5 standard deviations above the IoU value achieved by the 'simple edge' kernel. Adding additional entities to the kernel results in only very marginal gains as shown in the table at the right of Figure 5. This can be understood intuitively as the 'winged edge' kernel includes a compact set of topological entities immediately adjacent to a given edge. When the kernel is expanded beyond this size, the locations at which the topological walks terminate become dependent on the B-rep topology in the vicinity of the edge. For example the 'winged edge++' kernel configuration includes walks like NMN and MPM which will evaluate to the same entity when walking around vertices of valance 3 but distinct entities when the vertex has valance 4 or higher. The 'winged edge' kernel lies at a sweet spot containing enough entities to allows patterns in local regions of the B-rep topology to be recognized, while being small enough not to be adversely affected by differences in the topology.

## 5.3. Ablation studies on input features

Here we identify which of the input features described in Section 3.3 play an important role in the results for the segmentation. The network is trained with groups of input features removed and the resulting IoU values are shown in Figure 6. The 'winged edge+' kernel configuration is employed in these experiments and the hyper parameters are as described in Section 5.2. We see that removing the one-hot encodings for surface type reduces IoU by 3 . 7% and removing curve type information reduces the IoU by 3 . 9% . These large reductions in performance are expected as the surface and curve type information is the primary way geometric information is fed to the network.

Wealso observe a 4 . 6% reduction in IoU when edge convexity is removed. Edge convexity is well known to be useful for the detection of form features and was used in a large number of early neural networks [34, 31, 41, 38]. Joshi et al . [22] offers an insight into how edge convexity could be useful with the observation that a face with all convex edges cannot be a part of a concave feature ( CutSide or CutEnd ).

Removing other input features have much smaller effects. Without the edge length feature the IoU only decreases by 0 . 7% and removing the face area feature causes just a 0 . 4% IoU decrease. Hence we conclude that edge convexity, curve type and surface type are the primary pieces of information used by the network in segmentation.

## 5.4. Heuristic method comparison

Many CAD modeling packages use rule-based algorithms for the detection of form features. Here we compare against the feature recognition capabilities of Autodesk Shape Manager (ASM) [4], an industry standard CAD kernel used in numerous commercial products.

As ASM cannot detect the RevolveEnd segment type, we omit this when computing the ASM average IoU result.

Figure 6: Effect on IoU of removing groups of input features from the network.

<!-- image -->

ASM does not identify any modeling feature type for 13% of the faces in the dataset and we consider these faces to be incorrectly classified.

The results for ASM feature recognition on the Fusion 360 Gallery segmentation task are shown in Table 1. While BRepNet achieves an IoU value 27% higher than ASM, this does not reflect the results qualitatively. When features are recognized by the ASM algorithm, the faces identified are always geometrically consistent with the type of feature found. The confusion is between classes where the actual modeling technique used is ambiguous. For example, a designer may choose to create a cylinder with an extruded circle or a revolved rectangle. The higher accuracy achieved on the classification task by BRepNet shows the network is capable of learning the most likely modeling technique a designer will employ rather than simply identifying one of the possible solutions.

## 5.5. Edge-conditioned convolution graph network

In this section we compare BRepNet performance with an Edge-Conditioned Convolution (ECC) graph network as described in [39]. As discussed in Section 5.3, an important indicator for the class of a face is the convexity of its surrounding edges. As this architecture allows edge attributes to affect the messages passed between faces it is well suited to Fusion 360 Gallery segmentation task. The B-rep topology is translated into a face adjacency graph with the faces represented as nodes connected by pairs of directed arcs with opposite orientations. We use the face features X f as input node features. As the directed arcs map 1:1 with the coedges in the B-rep, we create the attribute vectors for each directed arc by concatenating the corresponding coedge features with the features of its parent B-rep edge.

We match the hyper-parameters of the network to those of BRepNet as closely as possible. Two edge-conditioned convolution layers are used, with the edge specific weight matrices computed by two-layer MLPs. The width of the first MLP input is defined by the number of face features and all subsequent widths were set to 153. This gave the ECC network a total of 359 , 558 parameters which is the closest possible match to BRepNet. The accuracy and IoU of the ECC network are shown in Figure 5 and Table 1. The IoU value is more than 5% below what BRepNet can achieve using the 'winged edge' kernel. This is expected as this graph network architecture is not specifically designed for convolution on manifolds and does not map specific learnable parameters to specific entities in the convolution. We would expect to see IoU values approximately equal to what BRepNet achieves with the 'simple edge' kernel, but on the test set the ECC gives a 2% lower IoU value. We noticed that BRepNet is more stable than the ECC during training, and we believe the poor performance of the ECC on the test set may be partially due to the choice of epoch for which the trained model was recorded for use at test time. In all experiments the model with the lowest validation loss is used for evaluation on the test set.

Table 1: Accuracy, IoU and number of model parameters, | Θ | , for a variety of different networks. The BRepNet results are for the 'winged edge' kernel configuration. The per-entity accuracy and IoU columns refer to per-edge accuracy in the case of MeshCNN and per-point accuracy in the case of PointNet++.

|            | Per-face accuracy   | Per-face IoU   | Per-entity accuracy   | Per-entity IoU   | &#124; Θ &#124;   |
|------------|---------------------|----------------|-----------------------|------------------|-------------------|
| BRepNet    | 92.52 ± 0.15        | 77.10 ± 0.54   | -                     | -                | 359,100           |
| ECC        | 90.36 ± 0.23        | 72.08 ± 0.50   | -                     | -                | 359,558           |
| ASM        | 64.57               | 49.53          | -                     | -                | -                 |
| PointNet++ | 74.00 ± 0.84        | 34.78 ± 2.23   | 82.78 ± 0.30          | 36.49 ± 1.67     | 1,403,784         |
| MeshCNN    | 62.99 ± 0.37        | 20.59 ± 0.41   | 73.81 ± 0.51          | 24.07 ± 0.42     | 2,279,720         |

## 5.6. Comparison with geometry based methods

In this section we investigate the advantages of working directly with B-rep models rather than converting the geometry to meshes or point clouds. We generate closed and manifold meshes from the B-rep geometry with close to 3000 triangles edges each. Computing meshes which meet this criteria is itself a difficult task requiring specialized meshing algorithms. For 13% of B-rep models the meshing algorithm failed entirely and the B-rep representations of these models were removed from the dataset. Avoiding the requirement to generate these high quality meshes is a major advantage of working directly on B-rep data.

Point cloud data is then generated by random uniform sampling of 2048 points over the surface of each mesh. As the number of points sampled from each face is determined by area, small faces generate a low number of points. The potential to under-sample some faces is a disadvantage for point cloud techniques as small holes and grooves, which can be critical for the function of the part, may be missed.

Two well-known architectures, PointNet++ [36], and MeshCNN [18] are adapted for the face segmentation task. To compare performance between multiple representations we use per-face classification accuracy and IoU as our primary evaluation metrics. The triangles generated from each B-rep face inherit the label of that face and points inherit the labels of the triangles from which they were sampled. Triangle edges are considered to be owned by the first of the two triangles sharing the edge and edge labels are derived from the faces which generated the triangles. The per-face accuracy and IoU is then evaluated by averaging the segmentation scores for the points or edges derived from each face. This gives a prediction of the class for each face, from which the accuracy and mean IoU can be evaluated as described in Section 5.1. In addition to the per-face metrics we also report the per-point and per-edge accuracy and IoU for PointNet++ and MeshCNN respectively. As for the perface metrics, the IoU values are computed by considering the points or edges from all bodies together.

Table 1 details the accuracy and IoU results of the segmentation task along with the number of model parameters. Both BRepNet and the ECC easily outperform the geometry based methods by more than 16% accuracy and 37% in IoU with just under 1 / 4 the number of parameters. This demonstrates the advantages of working directly with the compact B-rep data rather than derived representations.

## 6. Conclusions

We have presented BRepNet, a neural network architecture which can operate directly on B-rep models. We also introduced the Fusion 360 Gallery segmentation dataset and provide benchmark results on the segmentation problem. Our results show that by using the concise surface and edge type information from the B-rep data structure the network can easily outperform existing techniques using point clouds and meshes on the Fusion 360 Gallery dataset segmentation task. In addition we demonstrate that by defining convolutional kernels relative to the coedges of the B-rep, the architecture can make use of information about the next and previous coedges in the loops around faces, giving better performance than an edge conditioned convolution network with the same number of parameters. In future work we plan to apply this convolution scheme to other problems where graphs are embedded in 2D manifolds such as polyhedral models, subdivision surfaces, super-pixel segmentations of image data, region growing algorithms on triangle meshes and for learning tasks on Voronoi diagrams.

## References

- [1] Mazin Al-wswasi, Atanas Valev Ivanov, and Harris G. Makatsoris. A survey on smart automated computeraided process planning (acapp) techniques. The International Journal of Advanced Manufacturing Technology , 97:809-832, 2018. 1
- [2] Atin Angrish, Benjamin Craver, and Binil Starly. 'FabSearch': A 3D CAD Model-Based Search Engine for Sourcing Manufacturing Services. Journal of Computing and Information Science in Engineering , 19(4), 06 2019. 041006. 5
- [3] S. Ansaldi, K. De Floriani, and B. Falcidienc. An edge-face relational scheme for boundary representations. Computer Graphics Forum , 4(4):319-332, 1985. 2
- [4] Autodesk. Autodesk shapemanager. https://en.wikipedia.org/wiki/ShapeManager. 7
- [5] Autodesk. Inventor Feature Recognition , 2012. 1
- [6] Bojan R. Babic, Nenad Nesic, and Zoran Miljkovic. A review of automated feature recognition with rulebased pattern recognition. Comput. Ind. , 59:321-337, 2008. 1
- [7] Aditya Balu, Kin Gwn Lore, Gavin Young, Adarsh Krishnamurthy, and Soumik Sarkar. A deep 3d convolutional neural network based design for manufacturability framework. CoRR , abs/1612.02141, 2016. 2
- [8] Bruce G. Baumgart. Winged edge polyhedron representation. Technical report, Stanford University, Stanford, CA, USA, 1972. 3
- [9] Giorgos Bouritsas, Sergiy Bokhnyak, Stylianos Ploumpis, Stefanos Zafeiriou, and Michael Bronstein. Neural 3d morphable models: Spiral convolutional networks for 3d shape representation learning and generation. 2019 IEEE/CVF International Conference on Computer Vision (ICCV) , Oct 2019. 2
- [10] Weijuan Cao, Trevor Robinson, Yang Hua, Flavien Boussuge, Andrew R. Colligan, and Wanbin Pan. Graph representation of 3d cad models for machining feature recognition with deep learning. In Proceedings of the ASME 2020 International Design Engineering Technical Conferences and Computers and Information in Engineering Conference , IDETC-CIE. ASME, 2020. 2
- [11] Y. H. Chen and H. M. Lee. A neural network system feature recognition for two-dimensional. International Journal of Computer Integrated Manufacturing , 11(2):111-117, 1998. 2
- [12] Florence Danglade, Jean-Philippe Pernot, and Philippe V´ eron. On the use of machine learning to defeature cad models for simulation. Computer-Aided Design and Applications , 11(3):358-368, 2014. 1
- [13] Dassault. Solidworks FeatureWorks , 2019. 1
- [14] Lian Ding and Yong Yue. Novel ann-based feature recognition incorporating design by features. Computers in Industry , 55:197-222, 10 2004. 2
- [15] Charles M Eastman and Kevin J Weiler. Geometric modeling using the euler operators. 12 p. : ill. Pittsburgh: Institute of Physical Planning, Carnegie Mellon University, February, 1979. includes bibliography , 01 1979. 3
- [16] Justin Gilmer, Samuel S. Schoenholz, Patrick F. Riley, Oriol Vinyals, and George E. Dahl. Neural message passing for quantum chemistry. In Doina Precup and Yee Whye Teh, editors, Proceedings of the 34th International Conference on Machine Learning , volume 70 of Proceedings of Machine Learning Research , pages 1263-1272, International Convention Centre, Sydney, Australia, 06-11 Aug 2017. PMLR. 6
- [17] Leonidas Guibas and Jorge Stolfi. Primitives for the manipulation of general subdivisions and the computation of voronoi. ACM Trans. Graph. , 4(2):74-123, Apr. 1985. 3
- [18] Rana Hanocka, Amir Hertz, Noa Fish, Raja Giryes, Shachar Fleishman, and Daniel Cohen-Or. Meshcnn: a network with an edge. ACMTransactions on Graphics (TOG) , 38(4):1-12, 2019. 2, 4, 8, 14
- [19] Mark Henderson, Gopal Srinath, Roger Stage, Kim Walker, and William Regli. Boundary representationbased feature identification. Manufacturing Research and Technology , 20(C):15-38, 1994. 2
- [20] Pradeep Kumar Jayaraman, Aditya Sanghi, Joseph Lambourne, Thomas Davies, Hooman Shayani, and Nigel Morris. Uv-net: Learning from curve-networks and solids. arXiv:2006.10211v1 , 2020. 2, 3
- [21] Y. Jia. Learning Semantic Image Representations at a Large Scale . PhD thesis, University of California, Berkeley, 2014. 2
- [22] S. Joshi and T.C. Chang. Graph-based heuristics for recognition of machined features from a 3d solid model. Computer-Aided Design , 20(2):58 - 66, 1988. 2, 7
- [23] Byung Chul Kim and Soonhung Han. Integration of history-based parametric translators using the automation apis. Int. J. Product Lifecycle Management , 2, 01 2007. 1
- [24] Sebastian Koch, Albert Matveev, Zhongshi Jiang, Francis Williams, Alexey Artemov, Evgeny Burnaev, Marc Alexa, Denis Zorin, and Daniele Panozzo. Abc: A big cad model dataset for geometric deep learning. In Proceedings of the IEEE Conference on Computer Vision and Pattern Recognition , pages 96019611, 2019. 5

- [25] L.K. Kyprianou. Shape Classification in Computeraided Design . University Library, 1980. 2
- [26] Eric-Tuan Le, Iasonas Kokkinos, and Niloy J. Mitra. Going deeper with lean point networks. In Proceedings of the IEEE/CVF Conference on Computer Vision and Pattern Recognition (CVPR) , June 2020. 6
- [27] Sang Hun Lee and Kunwoo Lee. Partial entity structure: A compact non-manifold boundary representation based on partial topological entities. In Proceedings of the Sixth ACM Symposium on Solid Modeling and Applications , SMA '01, page 159-170, New York, NY, USA, 2001. Association for Computing Machinery. 3
- [28] Hsueh-Ti Derek Liu, Vladimir G. Kim, Siddhartha Chaudhuri, Noam Aigerman, and Alec Jacobson. Neural subdivision. ACM Trans. Graph. , 39(4), July 2020. 2, 4, 6
- [29] Lujie Ma, Zhengdong Huang, and Yanwei Wang. Automatic discovery of common design structures in cad models. Comput. Graph. , 34:545-555, 2010. 2
- [30] M. Marquez, R. Gill, and A. White. Application of neural networks in feature recognition of mould reinforced plastic parts. Concurrent Engineering , 7(2):115-122, 1999. 2
- [31] Konstantinos Nezis and George Vosniakos. Recognizing 212d shape features using a neural network and heuristics. Computer-Aided Design , 29(7):523-539, 1997. 2, 7
- [32] Mathias Niepert, Mohamed Ahmed, and Konstantin Kutzkov. Learning convolutional neural networks for graphs, 2016. 2
- [33] Les Piegl and Wayne Tiller. The NURBS Book . Springer-Verlag, New York, NY, USA, second edition, 1996. 1, 4
- [34] S. Prabhakar and Mark Henderson. Automatic formfeature recognition using neural-network-based techniques on boundary representations of solid models. CAD Computer Aided Design , 24(7):381-393, jul 1992. 2, 7
- [35] Charles R Qi, Hao Su, Kaichun Mo, and Leonidas J Guibas. Pointnet: Deep learning on point sets for 3d classification and segmentation. In Proceedings of the IEEE conference on computer vision and pattern recognition , pages 652-660, 2017. 2, 6
- [36] Charles Ruizhongtai Qi, Li Yi, Hao Su, and Leonidas J Guibas. Pointnet++: Deep hierarchical feature learning on point sets in a metric space. In Advances in neural information processing systems , pages 50995108, 2017. 2, 8, 14
- [37] Hiroshi Sakurai and David C. Gossard. Recognizing shape features in solid models. IEEE Computer Graphics and Applications , 10:22-32, 1990. 2
- [38] Yang Shi, Zhang Yicha, Kaishu Xia, and Ramy Harik. A critical review of feature recognition techniques. Computer-Aided Design and Applications , 17:861899, 01 2020. 1, 2, 7
- [39] Martin Simonovsky and Nikos Komodakis. Dynamic edge-conditioned filters in convolutional neural networks on graphs. 2017 IEEE Conference on Computer Vision and Pattern Recognition (CVPR) , Jul 2017. 6, 7, 14
- [40] S. Staley, Mark Henderson, and D. Anderson. Using syntactic pattern recognition to extract feature information from a solid geometric database. Computers in Mechanical Engineering , 2, 09 1983. 2
- [41] V. Sunil and Sanket Pande. Automatic recognition of machining features using artificial neural networks. The International Journal of Advanced Manufacturing Technology , 41:932-947, 04 2009. 2, 7
- [42] Pengyu Wang, Yuan Gan, Panpan Shui, Fenggen Yu, Yan Zhang, Songle Chen, and Zhengxing Sun. 3d shape segmentation via shape fully convolutional networks. Computers &amp; Graphics , 70:128-139, Feb 2018. 2
- [43] Yue Wang, Yongbin Sun, Ziwei Liu, Sanjay E. Sarma, Michael M. Bronstein, and Justin M. Solomon. Dynamic graph CNN for learning on point clouds. CoRR , abs/1801.07829, 2018. 2
- [44] K.J. Weiler. Topological structures for geometric modeling . Technical report RPI, Center for Interactive Computer Graphics. University Microfilms, 1986. 1, 3
- [45] Yong Yue, Lian Ding, Kemal Ahmet, John Painter, and Mick Walters. Study of neural network techniques for computer integrated manufacture. Engineering Computations , 19:136-157, 03 2002. 1
- [46] Ziwei Zhang, Peng Cui, and Wenwu Zhu. Deep learning on graphs: A survey. CoRR , abs/1812.04202, 2018. 2
- [47] Zhibo Zhang, Prakhar Jaiswal, and Rahul Rai. Featurenet: machining feature recognition based on 3d convolution neural network. Computer-Aided Design , 101:12-22, 2018. 5

## A. Supplementary Material

## A.1. Dataset statistics

The Fusion 360 Gallery segmentation dataset contains a total of 35,858 B-rep bodies with corresponding segmentation information, high quality triangle meshes and point clouds. The train/test split used in this work is published with the dataset. It contains 5 , 399 bodies in the test set, with the remaining 30 , 459 bodies for use training and performing validation. Example designs from the dataset are shown in Figure 10, colored according to segmentation label.

The complexity of the models can be understood from the number of faces per B-rep body and the number of CAD modeling operations used in their construction. Histograms of these distributions are shown in Figure 7. Many of the bodies are relatively simple with half of them having fewer than 9 faces. There is a long tail of more complicated Breps with the most complex having 421 faces. Just over half of the bodies were constructed with more than one CAD modeling operation and 31% have two or more. Only 1% of the bodies used more than 10 operations in the construction history and the maximum number of operations used to create any B-rep in the dataset is 59.

As explained in Section 4, the dataset is modeled entirely using extrusions, revolutions, fillets and chamfers. The Autodesk Fusion 360 CAD package allows the geometry/topology created by each modeling operation to be immediately combined with the body being constructed using either a boolean union, subtraction or intersection. Extrusions are the most common way geometry is created, comprising 74% of modeling operations. To avoid a very large imbalance in the dataset, the collection of faces created by extrusions are subdivided as follows. When the extrusions were used to create new bodies or unioned with to existing bodies, the faces are placed in the Extrude class, while faces from extrusions which are subtracted from or intersected with existing bodies are placed in the Cut class. The faces generated by extrusions and revolutions are then further subdivided into side and end faces. For extrusions, side faces are created by sweeping the profile geometry while end faces are parallel to the plane on which the profile was sketched. For revolutions, sides faces are created by revolving the profile. RevolveEnd faces are only created when the extrusion is not 360 degrees as shown in Figure 9. This results in eight possible labels: ExtrudeSide , ExtrudeEnd , CutSide , CutEnd , Fillet , Chamfer , RevolveSide , and RevolveEnd .

The fraction of faces with each label type is shown in Figure 8. We see that just over half the faces are in the ExtrudeSide class, making the dataset relatively imbalanced. In particular the RevolveEnd class is very rare, accounting for just 0 . 08% of faces. The CutEnd and Chamfer classes are also rare at 2 . 35% and 2 . 94% of the faces respectively. The faces in these classes are often planar, so the correct segment type can only be identified by considering the surrounding shape. This makes the identification of these rare classes an extremely challenging task.

<!-- image -->

Figure 7: The distribution of faces per body (top) and of CAD modeling features used to generate each body (bottom).

Figure 8: The percentage of faces in each class.

<!-- image -->

Figure 9: Side and end faces for extrusions and revolutions.

<!-- image -->

Figure 10: Example designs from the Fusion 360 Gallery segmentation dataset, colored by segmentation label.

<!-- image -->

Table 2: The topological walks making up the kernels shown in Figure 5.

| Kernel        | Faces   | Edges                                                 | Coedges                                                                 |
|---------------|---------|-------------------------------------------------------|-------------------------------------------------------------------------|
| Simple edge   | F , MF  | E                                                     | I , M                                                                   |
| Asymmetric    | F , MF  | E                                                     | I , N                                                                   |
| Asymmetric+   | F , MF  | E                                                     | I , M , N                                                               |
| Asymmetric++  | F , MF  | E , NE                                                | I , M , N                                                               |
| Winged edge   | F , MF  | E , NE , PE , MNE , MPE                               | I , M , N , P , MN , MP                                                 |
| Winged edge+  | F , MF  | E , NE , PE , MNE , MPE                               | I , M , N , NM , P , PM , MN , MNM , MP , MPM                           |
| Winged edge++ | F , MF  | E , NE , PE , MNE , MPE , NMNE , PMPE , MPMPE , MNMNE | I , M , N , NM , P , PM , MN , MNM , MP , MPM , NMN , PMP , MPMP , MNMN |

## A.1.1 Data preparation

In this section we describe the processing performed on the designs from the Autodesk Online Gallery 1 to generate the Fusion 360 Gallery segmentation dataset. Scripts were used to drive the Application Programming Interface (API) of the Fusion 360 CAD package to load each Fusion document and suppress all features except extrusions, revolutions, fillets and chamfers. While this procedure modified the shapes of some models, it greatly increased the number of B-rep bodies available for the dataset.

The number of faces and edges in the resulting bodies, along with the surface area and volume of each body, was then recorded. To remove duplicate models, we first search for bodies with the same number of faces and edges. The area and volume of these candidates are then checked and bodies for which the area and volume matches to within 1% were discarded. The de-duplication procedure was verified using thumbnails of the resulting duplicate free dataset. These were ordered by surface area and images were manually inspected to verify that the area and volume tolerance was appropriate for detecting duplicate bodies, even when rigid body transforms had been applied. Models with small topological differences were not considered to be duplicates as BRepNet is designed to be sensitive to differences in the model topology.

All the B-reps in the dataset were transformed to place the center of their bounding boxes at the origin. A uniform scaling was then applied so that the largest length of the bounding box is 2 model units across. Meshes and point clouds were then extracted from these scaled models.

1 https://gallery.autodesk.com/fusion360

## A.1.2 Input feature standardization

The input features for BRepNet are standardized using the following procedure. The mean, µ and standard deviation, σ , are computed for each input feature for the entire training set. The standardized values x ′ are then computed as

<!-- formula-not-decoded -->

## A.1.3 Support for operation grouping and ordering problems

In addition to the segmentation data, we also provide more detailed information about the modeling operations used to construct the solid. For each B-rep face we provide a unique identifier for the operation which created it, allowing groups of faces created by different operations of the same type to be separated. For extrusions and revolutions we also provide a classification of the face as Start , End or Side . We provide the type of each operation and the order in which the operations were applied in the parametric model history. For the point cloud and mesh representations we provide the mapping from each point and triangle to the face from which it was sampled, allowing this extra information to be used for all representations. More details on how this data is organized is in the dataset documentation 2 .

This additional data is intended to support research into reverse engineering tasks. Grouping the points or triangles according to the operation which created them is a first step towards rebuilding the parametric history. Further subdividing each extrusion based on the Start , End and Side information allows the extrusion direction to be predicted. Slicing the mesh perpendicular to this extrusion direction can then allow the profile curves to be extracted and the extruded volumes regenerated. Finally, by predicting the ground truth operation type and order, sensible sequences for the modeling operations can be learned.

2 https://github.com/AutodeskAILab/Fusion360GalleryDataset

Figure 11: The per-face IoU values achieved by each network for each segment class.

<!-- image -->

## A.2. Kernels

In Section 5.2 the performance of BRepNet was compared with a number of different kernels. Diagrams showing the entities taking part in these kernels are shown in Figure 5. In these diagrams each topological walk in the kernel terminates on a distinct entity. It should be noted that this will not always be the case for arbitrary B-reps data. Some local topology will result in two or more topological walks terminating on the same entity. No special case handling is required when this happens. The procedure described in Equation 2 simply concatenates feature vectors from the same entity into the same row of Ψ multiple times. The network learns to recognizer these repeated patterns in the feature vectors in the same way as in the case where the entities are distinct. Table 2 gives the full lists of topological walks which make up the kernels.

## A.3. Experiments

## A.3.1 Training details

For all the experiments described in Section 5, the BRepNet network was trained using the Adam optimizer, with default parameters (learning rate of 0 . 001 and betas 0 . 9 and 0 . 999 ). The B-reps in the dataset were divided into minibatches, each containing approximately 1000 faces. Multiple B-Reps can be combined into a same batch by row-wise concatenation of the input feature matrices X f , X e and X c , and diagonal concatenation of the matrices N , P , M , E and F . Training and evaluate was performed using NVIDIA Tesla V100 GPUs. Training took an average of 45s perepoch with the network typically achieving a minimum loss value by around the 15 th epoch. Hence the BRepNet network could be trained on the Fusion 360 Gallery segmenta- tion dataset in under 12 minutes from random seed.

For experiments comparing against PointNet++ and MeshCNN we use the official implementation and retain the default hyper-parameters where possible. PointNet++ models are trained with a batch size of 32, a learning rate of 0.001 and momentum of 0.9 using the TensorFlow implementation 3 from [36]. The PyTorch implementation 4 of MeshCNN was used with a batch size of 12 , a learning rate of 0 . 0002 and momentum of 0 . 9 . The maximum number of input edges for any mesh was 3500 and the pooling resolutions were set to 2500 , 1750 and 1000 .

The ECC used the pytorch-geometric NNConv implementation 5 . The Adam optimizer was used for the training with default parameters (learning rate of 0 . 001 and betas 0 . 9 and 0 . 999 ).

## A.3.2 Comparison of IoU for different classes

In this section we show the IoU values achieved by BRepNet, the Edge-Conditioned Convolution (ECC) graph network [39], PointNet++ [36] and MeshCNN [18] for the different classes individually. Figure 11 shows the per-face IoU values each network achieved on each class and Figure 12 shows images of the face segmentation on some example models. We see that a key reason why BRepNet and the ECC network perform better than PointNet++ and MeshCNN is their ability to correctly classify faces in the rare classes.

The RevolveEnd class always consists of planar faces and accounts for just 0 . 08% of the dataset. While BRepNet finds this class challenging, achieving only 49% IoU, both PointNet++ and MeshCNN fail to identify any RevolveEnd faces. We believe this is because the RevolveEnd class can only be identified by considering a face in the context of its neighbouring faces and edges. This is something which the ECC and BRepNet approaches can do easily, as both networks are designed to leverage information from adjacent faces. PointNet++ and MeshCNN have a hierarchical design which is intended to improve the flow of information from neighbouring geometry. PointNet++ achieves an IoU of 39% of the neighbouring RevolveSide faces, while MeshCNN achieves only 17% . Neither architecture manages the extremely challenging task of using their identification of the RevolveSide faces to correctly classify the adjacent groups of planar points/edges as RevolveEnd .

3 https://github.com/charlesq34/pointnet2

4 https://github.com/ranahanocka/MeshCNN

5 https://pytorch-geometric.readthedocs.io/en/latest/modules/nn.html

Figure 12: The per-face segmentation generated by each network.

<!-- image -->

The CutEnd class is also rare, accounting for just 2 . 35% of faces in the dataset. As for RevolveEnd , these faces are always planar. The primary way they can be distinguished from the more common ExtrudeEnd faces is by observing that they are often surrounded by concave edges. PointNet++ is able to correctly identify 10 . 83% of CutEnd faces, while MeshCNN recognizes just 0 . 01% of them. While MeshCNN uses dihedral angle as an input feature, it does not distinguish between concave and convex edges. This would explain why MeshCNN struggles to distinguish the subtractive extrusion classes ( CutSide and CutEnd ) from the more common additive extrusions ( ExtrudeSide and ExtrudeEnd ).

The Fillet and Chamfer classes account for 10 . 22% and 2 . 94% of faces in the dataset. Fillets are very distinctive features with smooth edges and typically cylindrical or toroidal geometry. Both BRepNet and the ECC have high IoU scores for this class ( 97 . 57% and 97 . 32% respectively), demonstrating that these patterns in the input features are easy to spot using both architectures. Faces in the Chamfer class are much less distinctive. They are often planar or conical and their edges can be either concave or convex. Consequently BRepNet and the ECC achieve lower IoU scores of 81 . 80% and 71 . 16% . PointNet++ and MeshCNN also have higher IoU values for fillets than for chamfers. For fillets these networks achieve IoUs of 49 . 31% and 36 . 49% respectively while for chamfers PointNet++ achieves only an IoU of 21 . 79% and MeshCNN fails to detect any chamfer features.

Both Fillet and Chamfer faces tend to have relatively small areas and consequently are prone to under-sampling when fixed edge-count meshes and fixed size point clouds are generated, however the IoU values PointNet++ and MeshCNN achieve for Fillet suggests this was not a major limiting factor for the relatively small solids in the Fusion 360 Gallery segmentation dataset. Comparing the results for Fillet with the less distinctive CutSide class we see that all network are more successful at detecting fillets, despite these faces having smaller areas and accounting for a similar fraction of the dataset.

The ExtrudeSide and ExtrudeEnd classes account for 50 . 60% and 15 . 72% of the dataset respectively. All net- works do well on these classes. It should be noted that for both BRepNet and the ECC the IoU achieved for the most common ExtrudeSide class is smaller than for the more distinctive Fillet class.