## Performing Efficient NURBS Modeling Operations on the GPU

Figure 1: Modeling operations like sketching, ray intersection and trimming performed directly on trimmed NURBS models.

<!-- image -->

## Abstract

We present algorithms for evaluating and performing modeling operations on NURBS surfaces using the programmable fragment processor on the Graphics Processing Unit (GPU). We extend our GPU-based NURBS evaluator that evaluates NURBS surfaces to compute exact normals for either standard or rational B-spline surfaces for use in rendering and geometric modeling. We build on these calculations in our new GPU algorithms to perform standard modeling operations such as inverse evaluations, ray intersections, and surface-surface intersections on the GPU. Our modeling algorithms run in real time, enabling the user to sketch on the actual surface to create new features. In addition, the designer can edit the surface by interactively trimming it without the need for retessellation. We also present a GPU-accelerated algorithm to perform surface-surface intersection operations with NURBS surfaces that can output intersection curves in the model space as well as in the parametric spaces of both the intersecting surfaces at interactive rates.

CR Categories: I.3.3 [Computer Graphics]: Hardware Architecture-Graphics Processors; I.3.3 [Computer Graphics]: Picture/Image Generation-Display Algorithms; I.3.5 [Computer Graphics]: Computational Geometry and Object ModelingSplines.

Keywords: NURBS, GPU, Inverse Evaluation, Sketching, Interactive Trimming, Intersection Curves, Prefix Sum

∗ e-mail: { adarsh-rahul-mcmains } @me.berkeley.edu

† e-mail:khaller@solidworks.com

‡ e-mail:gershon@cs.technion.ac.il

## 1 Introduction

Industrial design of products has shifted from using boxy shapes with straight edges to incorporate curved freeform surfaces. Non Uniform Rational B-Spline (NURBS) surfaces provide a convenient and compact representation of such curved surfaces that has become the representation of choice in mechanical CAD systems. Hence, real time interaction with NURBS surfaces is essential for any CAD package. However, since evaluation of a NURBS surface is inherently a compute-intensive process, commercial CAD packages deal with it by preprocessing NURBS surfaces. They usually tessellate NURBS surfaces and use the triangulated model for display as well as certain modeling operations like selection. With the advent of programmable graphics hardware, the need for tessellating the NURBS surface in the CPU for display was obviated, since the GPU can be used for the evaluation and direct display of the surfaces [Krishnamurthy et al. 2007; Kanai 2007; Guthe et al. 2005]. However, CAD packages still perform modeling operations using the CPU with either the tessellated NURBS surfaces or analytically using NURBS definitions. This reduces the interactivity for the user when designing these free form surfaces, since operations like sketching on the NURBS surface or fast evaluation of intersection curves are not possible. Leading commercial CAD packages do not allow the designer to sketch directly on the NURBS surface; instead, they restrict the user to sketching on a tangent plane. Because of this, the designer has to wait until the operation is completed to get visual feedback.

The process of finding the surface coordinates ( x, y, z ) for given parameter values ( u, v ) is called evaluation. Inverse evaluation is the process of finding the parameter values ( u, v ) given any point on the surface. We have developed a parallel algorithm for fast inverse evaluations of NURBS surfaces on the GPU. This algorithm forms the basis of many modeling operations like selection

Copyright © 2008 by the Association for Computing Machinery, Inc.

Permission  to  make  digital  or  hard  copies  of  part  or  all  of  this  work  for  personal  or classroom use is granted without fee provided that copies are not made or distributed for  commercial advantage and that copies bear this notice and the full citation on the first page. Copyrights for components of this work owned by others than ACM must be honored. Abstracting with credit is permitted. To copy otherwise, to republish, to post on servers,  or  to  redistribute  to  lists,  requires  prior  specific  permission  and/or  a  fee. Request permissions from Permissions Dept, ACM Inc., fax +1 (212) 869-0481 or e-mail permissions@acm.org.

SPM 2008, Stony Brook, New York, June 02-04, 2008.

© 2008 ACM 978-1-60558-106-4/08/0006 $5.00

(ray-surface intersection), sketching on the surface, and interactive trimming. Moreover, since these algorithms exploit the parallelism of the GPU, these operations can now be performed at interactive speeds, making immediate visual feedback to the designer possible for the first time. We demonstrate the use of our fast inverse evaluation algorithm to directly sketch on the surface, which makes certain operations like interactive trimming intuitive to the designer.

Designers are usually trained to work with curves on surfaces, such as silhouette curves and intersection curves of free-form surfaces. Thus, they would like to see real-time changes in these curves as the underlying surfaces are edited, which requires an efficient algorithm to compute intersection curves of surfaces. Finding the intersection curve is in general a very complex operation, since two NURBS surface equations of arbitrary degree have to be solved simultaneously. Many commercial CAD packages use marching methods, where the algorithm uses a numerical root-finding technique to first find a single intersection point. The algorithm then finds another point along the intersection curve that is close to the first intersection point. This process is repeated and ultimately a complete piecewise linear approximation of the intersection curve is calculated. However, since this technique is inherently serial it cannot be parallelized for efficient evaluation on the GPU. We have developed a GPU-accelerated parallel algorithm to evaluate the intersection curves using bounds on the evaluated surface points. This algorithm is both fast and guaranteed to find the intersection curves within a user-defined tolerance.

In this paper, we present GPU-based algorithms to perform modeling operations efficiently on NURBS surfaces. Our main contributions include

- A new unified method to calculate exact derivatives and exact normals of arbitrary-degree NURBS surfaces on the GPU. Our method is designed so as to not require separate fragment programs for evaluating surfaces of different degrees.
- An efficient algorithm to perform inverse evaluation of NURBS surfaces on the GPU. This algorithm finds the parametric ( u, v ) coordinate given any ( x, y, z ) coordinate on the NURBS surface within an arbitrary user-defined tolerance.
- A novel method to interactively trim and sketch on a NURBS surface in real time. This is possible because our fast inverse evaluation algorithm enables us to sketch in the model space, not just in the parametric space, with the correspondence tracked simultaneously.
- A GPU-accelerated algorithm to perform fast and robust NURBSsurface-surface intersections. The intersection curve, like the sketch curve above, is simultaneously output in the model space as well as in the parametric spaces of the two NURBS surfaces. The GPU is used to accelerate the operation by finding points on the intersection curves and the actual intersection curves are calculated from these points on the CPU.

We summarize our approach to evaluating and rendering NURBS surfaces on the GPU in Section 2; for more details please refer to [Krishnamurthy et al. 2007]. We then discuss the evaluation of first and second derivatives of the NURBS surfaces (Section 3) and then use these to compute bounding-boxes for NURBS surfaces (Section 4). Then, we describe how these bounds are used to perform inverse evaluations (Section 5) and to compute intersection curves (Section 6). Figure 2 shows these connections between the different parts of our algorithms; each of these operations are described in detail in the sections indicated.

Figure 2: Graphic showing the links between different parts of our modeling algorithms. The results of the GPU evaluations are stored in separate textures.

<!-- image -->

## 2 Previous Work

One of the main prerequisites for performing fast modeling operations on NURBS is to have a fast NURBS evaluator. We present a short outline of our algorithm to evaluate NURBS surfaces on the GPU that was explained in detail in [Krishnamurthy et al. 2007]. The main idea of our algorithm was to use a fragment program to evaluate a NURBS surface in several passes. This was different from previous approaches like [Guthe et al. 2005] that approximated and decomposed a NURBS surface into a set of bicubic rational Bezier patches and used a vertex program to evaluate them. One advantage of our approach is that we have two corresponding representations of the NURBS surface as 4-component vectors-( x, y, z, w ) coordinates-in space as well as their corresponding parametric values-( u, v ) coordinates. We exploit this correspondence during modeling operations like inverse evaluation and evaluation of intersection curves.

In our evaluation algorithm, we first evaluate the basis function values on the GPU in parallel for all the parameter positions where we want to evaluate the surface coordinates. The parameter positions can be chosen arbitrarily by the user; we chose an equally spaced grid of points to make the implementation simpler. We parallelize the de Boor evaluation algorithm ([de Boor 1978]) so that it runs efficiently on the GPU. We use the de Boor evaluation method because B-spline basis functions of any degree can be evaluated using the same fragment program. Other NURBS evaluations on the GPU either require different fragment programs for different degrees ([Kanai 2007]) or are restricted to cubic polynomials ([Guthe et al. 2005]). We perform the basis function evaluation separately for the u and v parametric directions on the GPU and store these values as textures. We multiply these basis function values with the corresponding control points to obtain the surface point coordinates. We can restrict the multiplication to the sub-mesh of the control points that affect a particular surface point because of the local support property of NURBS (Figure 3). We perform the multiplication operation in parallel for all the surface coordinates using another fragment program. The surface point coordinates thus calculated are stored directly in a texture on the GPU using the RGBA channels. While rendering, we interpret these values stored in the texture as vertex coordinates using a Vertex Buffer Object (VBO). Wethus avoid the slow operation of reading back the computed data from the GPU to the CPU, directly rendering the NURBS surface on the screen.

Figure 3: Graphic showing our NURBS evaluation algorithm on the GPU. The control mesh of size m × n and the evaluation mesh are made of four component vectors stored as RGBA textures. The surface patch is of order k u in the u direction and k v in the v direction. The multiplication is restricted to the sub-mesh of size k u × k v .

<!-- image -->

Previous work that used GPUs to render NURBS curves or surfaces focused only on efficient evaluation of the surface coordinates and/or normals ([Guthe et al. 2005; Loop and Blinn 2005; Guthe et al. 2006; Kanai 2007]). They did not use GPUs to perform modeling operations like inverse evaluations and intersection curve evaluations. Previous work on inverse evaluation of NURBS surfaces mainly focused on ray tracing NURBS surfaces. [Toth 1985; Nishita et al. 1990; Martin et al. 2000] perform ray tracing on parametric and rational surfaces by solving for the ray-surface intersection point using numerical methods. There has also been previous work on ray tracing using the GPU, which include [Purcell et al. 2002; Purcell et al. 2003; Carr et al. 2006]. Another application of inverse evaluation of NURBS is solving for geometric constraints. [Elber and Kim 2001] gave a method to solve geometric constraints by using multivariate splines, which can be used to solve several related problems like ray traps and sweep envelopes. Inverse evaluation has also been used for haptic rendering by [Thompson and Cohen 1999] to find the parametric ( u, v ) coordinates of a given point on a NURBS surface. They use inverse evaluation to solve for the contact point of a haptic probe with trimmed NURBS surfaces in a virtual environment.

The paper by [Carr et al. 2006] on ray tracing geometry images also presented a GPU algorithm to find the indices of the rendered texels in a texture, a sub-problem for our GPU algorithm for intersection calculations. This sub-problem falls under the class of stream-reduction, the process of removing unwanted elements from a stream of values and reducing it to a smaller list containing the required output. General Purpose computing on the GPU (GPGPU) uses stream reduction to remove defunct elements from the output of a previous pass before sending it as input for the next pass. Since the positions of the output elements do not have any fixed correspondence with the positions of the input, the stream-reduction process is considered non-uniform. A parallel O ( k +log n ) algorithm, where k is the output size, for non-uniform stream reduction based on prefix sums was given by [Blelloch 1990]. However, standard graphics cards do not have the capability to perform a scatter operation (random writes to different memory locations), which was an essential step in [Blelloch 1990]'s algorithm. [Horn 2005] presented another algorithm for non-uniform streamreduction on the GPU which runs in O ( n log n ) , not as efficient due to workarounds required because of lack of scatter. [Greß et al. 2006] proposed a stream reduction algorithm specifically for 2D textures on the GPU that used the fragment processor to perform other operations while performing the scatter operation, thereby hiding the latency. Recently [Sengupta et al. 2007] proposed an O ( n ) GPUstream-reduction algorithm, also using prefix sums, that relies on the latest nVIDIA CUDA architecture for its scatter func- tionality. We propose a similar O ( n ) stream-reduction algorithm based on computing a parallel prefix sum, but implement it using the standard GPGPU framework so that it is both compatible with older hardware and not limited to a single brand of GPU.

Collision detection has always been an important problem with many applications in CAD and graphics. [Govindaraju et al. 2003] used occlusion queries on graphics hardware to detect collisions of polygonal meshes in large environments. [Kipfer et al. 2004; Kolb et al. 2004] calculate collisions between particles to simulate large scale particle systems on the GPU. Recently, [Greß et al. 2006] detect collisions between deformable parameterized surfaces using GPUs. They solve the collision detection problem by generating a bounding-box hierarchy for the surface and then detect collisions by checking overlap between the bounding-boxes.

Evaluation of intersection curves is a fundamental operation in computer aided geometric design and solid modeling ([Requicha and Rossignac 1992; Hoffmann 1989]). There have been several attempts to solve the problem, since it is hard to achieve all the desired characteristics of robustness, accuracy, and efficiency. [Patrikalakis 1993] presented a comprehensive survey of the different surface-surface intersection algorithms that have been developed. [Krishnan and Manocha 1997] presented a new algebraic algorithm for efficient surface intersection using lower dimensional formulations. They also classified the conventional methods for evaluating the intersection curves as analytical methods, lattice evaluations, subdivision methods, and marching methods. Many commercial CAD software packages use the numerical marching method outlined by [Barnhill and Kersey 1990; Kriezis et al. 1990] to evaluate intersection curves.

## 3 Derivatives of NURBS Surfaces

To perform geometric operations on NURBS surfaces, we not only require the surface point coordinates themselves but also the first and second partial derivatives with respect to the two parameter directions u and v at the surface points. As a very fast first-degree approximation, we can use the evaluated point coordinates to estimate the first derivatives using central differencing. However, this approach gives rise to artificial discontinuities at patch boundaries and at rational parts of the surface. Moreover, second derivatives estimated from these first derivatives in the same manner have larger errors associated with them. One way to overcome this issue is to evaluate the normals of the surface exactly at each surface point, similar to the evaluation of the surface coordinates. Since we already evaluate the higher order basis functions from lower order basis-functions, we can directly calculate the derivatives of the basis-functions within the same framework as our basis-function evaluation algorithm, and then use the basis function derivatives to evaluate the derivatives of the NURBS surface precisely, to within machine precision.

## 3.1 Differential Geometry for B-Spline Surfaces

In this section, we present a concise version of the equations that are required for computing derivatives of NURBS surfaces, adapted from [Piegl and Tiller 1997]. We present the exact equations for a Non-Uniform B-Spline (NUBS) surface first and then extend the derivation to include rational surfaces. For a NUBS surface, S ( u, v ) , given by Equation (1), the derivatives can be computed by multiplying the control points ( P ij s) with the derivatives of the basis functions. The N p i s and N q j s are the B-spline basis functions of degree p and q respectively, defined by Equations (2) and (3); the P ij s are the NUBS control points defined as a quadrilateral mesh; and the u i s are the knots (similarly to v i s).

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The derivative of the basis function of degree p with respect to u is given by Equation (4). To evaluate the derivative of a basis function of degree p , the basis function of degree p -1 needs to be computed. We use the indicial notation N ,u to denote the derivative with respect to u . Note that the p -1 in the numerator of Equation (4) arises due to the fact that the B-spline basis function of degree p that we are differentiating is a piecewise polynomial of degree p in u .

<!-- formula-not-decoded -->

The derivatives of the B-spline basis functions, N ,u and N ,v , are then multiplied by the control points P ij to get the derivative along the u or v parametric direction on the surface as given by Equations (5) and (6) respectively. We can then calculate the surface normal N ( u, v ) of the NUBS surface (Figure 4) by taking the cross product of the u and v partial derivatives (Equation (7)). It should be noted that N ( u, v ) is not a unit vector field but it is well defined as long as S is a regular surface.

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Figure 4: Calculation of surface normal from the u and v partial derivatives.

<!-- image -->

## 3.2 Rational Derivatives

The derivatives of NURBS surfaces are not as straightforward to evaluate as in the NUBS case ([Abi-Ezzi and Wozny 1990]). This is because the derivatives have to be evaluated using the chain rule due of the existence of the rational component. The NURBS surface coordinates are evaluated as the 4-component vector shown in Equations (8) and (9). Since we evaluate the 4-component vectors without performing the rational division on the GPU, we can effectively use this data to evaluate the surface derivatives.

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The partial derivative with respect to u , derived using the chain rule (Equation (10)), can be calculated by first evaluating the product of the derivatives of the basis functions and the corresponding control points as a 4-component vector (Equation (11)) and then performing the required rational division operations. The partial derivative of the surface with respect to v can also be evaluated in a similar manner. In this work, we assume all the weights ( w ) are positive and hence no poles can occur in S or its partial derivatives.

## 3.3 GPU Implementation

The GPU implementation of the evaluation of surface derivatives is a direct extension of the evaluation of the surface coordinates as explained in Section 2. The GPU evaluation consists of four steps as given below. The first three steps are similar to the method for evaluation of the surface coordinates. We give the steps for evaluating the surface derivatives with respect to u ; the steps for finding the derivative with respect to v are similar, exchanging u and v in step 2:

1. Locate the sub-mesh of control points that influence the evaluation point coordinates.
2. Compute the basis functions and the derivatives of the basis functions along the two-parameter directions respectively.
3. (a) Compute the non-zero basis function derivatives with respect to u .

- (b) Compute the non-zero basis functions with respect to v .
3. Multiply the non-zero basis functions and the basis function derivatives with their corresponding control points from the sub-mesh and sum the results.
4. Evaluate the rational derivatives as given by Equation (10) using the evaluated surface coordinates and surface derivatives from the previous step.

One notable feature of this algorithm is that step 1 and step 2(b) are already performed while evaluating the surface coordinates using our NURBS evaluation algorithm. Moreover, computing the u derivative in step 2(a) is different from evaluating the B-spline basis function only in the final step of the evaluation. Since we are using the de Boor evaluation algorithm, evaluating the B-spline basis function of order k as well as its derivative require the evaluation of the B-spline basis function of order k -1 . In practice, since we are already computing the B-spline basis function of order k -1 , we store this intermediate result as a texture on the GPU. We then use this as input for evaluating both the B-spline basis function of order k as well as its derivative.

We evaluate separately the derivatives of the basis functions with respect to each parameter direction and store them in separate textures on the GPU. Once the derivatives with respect to the u and v directions are calculated as 4-component vectors, the surface normals are calculated. This is performed using a separate fragment program that takes the rational surface derivatives as input and then evaluates their cross product to calculate the surface normal (Equation (7)). Thus, the process of evaluating the NURBS surfaces as well as their normals can be performed efficiently within a single framework using our method.

## 4 Bounding-Boxes for NURBS Surfaces

Our parallel algorithms to perform modeling operations with NURBS surfaces require the calculation of axis-aligned boundingboxes (AABB) for the surfaces. With the help of such boundingboxes, several queries such as ray-surface intersections and surfacesurface intersections can be efficiently answered, which then form the building blocks for more complex operations like sketching on the surface and intersection curve calculations. There are different methods to construct bounding-boxes for free-form surfaces. One method is to fit bounding-boxes that enclose the control-points that define the surface. This method however does not produce very tight bounding-boxes and makes the bounding-boxes independent of the user-defined tolerance values. Another approximate method is to construct bounding-boxes enclosing sets of four adjacent points evaluated on the surface. [Greß et al. 2006] construct bounding-boxes with sets of four adjacent points on a parameterized surface for use in collision detection, but make sure their approximation of the surface is within the given tolerance by very finely subdividing the surface. However, this method does not guarantee that the surface will be completely enclosed by the boundingbox and it can potentially miss some intersections. We overcome these problems by evaluating the NURBS surface in a regular grid and then expand the bounding-boxes based on the curvature of the surface so that they are guaranteed to enclose the surface. Another advantage of this method is that the bounding-boxes automatically become tighter when we evaluate the surface at a finer resolution.

[Filip et al. 1987] give the analytical expression for the factor that can be used to expand the bounding-boxes based on the surface curvature. They show that if a parametric C 2 surface is evaluated at ( n + 1) × ( m + 1) grid of points, the deviation of the surface from the piecewise linear approximation cannot exceed a constant K defined by Equations (12)-(15).

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

To compute the bounding-boxes for a NURBS surfaces, we first evaluate the surface S ( u, v ) in a grid of points using our NURBS evaluator on the GPU. We also evaluate the precise first derivatives of the surface, ∂S/∂u and ∂S/∂v , at these points as explained in Section 3. We approximate the second partial derivatives of the surface by central differencing (explained below in Section 4.1). We then find the value of K for the surface using Equation (12). The bounding-boxes themselves are constructed by constructing boxes that enclose sets of four adjacent surface points and then expanding this box by K , which ensures that no part of the surface penetrates out of the bounding-box (Figure 5).

Figure 5: NURBS surface with bounding-boxes evaluated at two different resolutions.

<!-- image -->

## 4.1 Curvature Evaluation

Evaluating the exact curvature of the surfaces along the two parameter directions can be performed in a similar manner to evaluating the first derivatives. However, the number of additional calculation steps (16 passes for a bi-cubic surface) required for this operation is prohibitively many and therefore cannot be completed in a real-time setting. Nevertheless, since we have exact derivatives along the two parameter directions, we can approximate the second derivatives to a reasonable accuracy (error &lt; O (1 /n 2 ) for n evaluation points) by evaluating them using central differencing.

The central differencing formula for evaluating the second derivatives is given in Equation (16). The value of h is 1 /n for the u direction and 1 /m in the v direction since the surface is evaluated on a

( n +1) × ( m +1) grid of evaluation points. Three second-derivative values have to be calculated for each surface point: the second derivatives with respect to each parameter direction ( ∂ 2 S/∂u 2 and ∂ 2 S/∂v 2 ) and one mixed second derivative ( ∂ 2 S/∂u∂v ). However, we can use our same fragment program written to perform the central differencing operation to evaluate the second derivatives with different first derivative textures as input. For example, Equation (17) shows how to calculate the second derivative with respect to u using the first derivative as input using central differencing.

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Figure 6: Algorithm to evaluate bounding-boxes of a NURBS surface on the GPU.

<!-- image -->

To perform this operation on the GPU, we first evaluate the constants M 1 , M 2 , and M 3 on the GPU (Figure 6). We calculate these constants by first evaluating the three second-derivatives as explained above for each point on the grid in parallel using a fragment program written to evaluate the second derivatives. We then find the maximum value of each derivative; unfortunately such a 'reduction' operation cannot be performed in a single pass on current GPUs but require log n passes for an n × n texture. We use these constants M 1 , M 2 , and M 3 to find the expansion factor K for the surface, which is constant for a given surface patch. Finally, we construct the bounding-boxes by first using sets of four adjacent surface points to get an AABB and then expand this box in all three directions by K . The bounding-boxes themselves are stored as two textures, one each for the two extreme corners of the AABB, on the GPU. We evaluate all the bounding-boxes for a surface in a single pass using a fragment program written to evaluate the boundingboxes, which then outputs the values to the two different textures using multiple render targets.

## 5 Inverse Evaluation of NURBS Surfaces

Given a point that lies on the NURBS surface, inverse evaluation is the process of finding the parameter values corresponding to that point. Since the B-spline basis functions are non-linear, theoretical expressions for the inverse evaluation are very complex and differ based on the degree of the surfaces. Therefore, inverse evaluations are usually performed numerically to find a solution within a desired tolerance.

The standard numerical approaches based on solving the NURBS equations for inverse evaluation are not easily parallelizable to be performed efficiently on the GPU. Therefore, we chose a method based on axis-aligned bounding-boxes. The AABBs for the NURBS surface are constructed using the method outlined in Section 4. In the case of ray-tracing applications for selection and directly drawing on the surface, the AABBs are aligned parallel to the ray direction, which is perpendicular to the viewing plane. We then check for intersection between the ray and all the AABBs simultaneously using a fragment program written to perform this intersection test. The output of this program is a two-dimensional array of binary values with the value 1 corresponding to the intersection of the ray with any AABB (Figure 7). In addition, the intersecting AABB also contains information about the minimum and maximum parameter values of the surface sub-patch enclosed by the AABB. Using this correspondence, we can efficiently find the parametric ( u, v ) value corresponding to the ray intersection point on the surface.

Figure 7: Bounding-boxes stored as min and max textures are tested with the ray using a fragment program; its output is a binary texture indicating the intersection.

<!-- image -->

Since the NURBS surfaces are usually curved, there can be many surface sub-patches intersecting the given ray. We find the addresses of all the intersecting bounding-boxes (locations with the value 1 in the binary texture) by using the GPU stream reduction operation explained in Section 5.1. We use this address to access information about the intersecting bounding-box as well as the parametric ranges of the surface sub-patch enclosed by the boundingbox. Using the bounding-box information, we get bounds on the location of the intersection point of the ray with the surface in both the model space as well as in the parametric space simultaneously. If the bounding-boxes are smaller than the required tolerance, we can take the midpoint of the bounding-box as the intersection point of the ray with the surface. Once all the ray intersection points on the surface are found, we output only the point that is closest to the view-plane by evaluating the distance of all the ray intersection points from the view-plane on the CPU and choosing the point with the smallest distance value.

## 5.1 GPU Stream Reduction

An essential operation in our inverse evaluation algorithm is to find the address of all the bounding-boxes that intersect with a particular ray so that we can use this address to access information about the intersecting bounding-box. In this operation, we find the indices (location) of the texels in the texture that have the given value (in this case, 1 ). This corresponds to a class of problems known as non-uniform stream reduction. Stream reduction is usually considered a serial operation since the number of elements in the output is not known and hence the whole input has to be operated upon to output the correct result. There has been some previous work on developing parallel algorithms based on parallel prefix sum for this operation which we summarized in Section 2. Implementing this parallel prefix sum on standard graphics hardware is not straightforward however, due to the lack of scatter functionality on standard programmable GPUs.

We first explain briefly the parallel stream reduction operation described by [Blelloch 1990]. It consists of three main steps: upsweep, down-sweep, and scatter. The up-sweep operation computes a hierarchy of log n levels where each element at a higher-level is obtained as a sum of two elements in the lower-level (Algorithm 1). An example of the up-sweep operation is shown using an 8-element 1D array (Figure 8). The last element at the end of the operation gives the total number of elements with the value 1 in the input array. After performing this operation, we obtain a binary tree with the last element as the root node and the original array as the leaf nodes; each node of this tree represents the sum of all the values in the sub-tree of that node.

Algorithm 1: The up-sweep algorithm to construct a hierarchy of the input.

<!-- image -->

Figure 8: Example of the up-sweep operation performed on an 1D array given in the first row. The inputs indicated are summed at each step.

<!-- image -->

The down sweep operation given by Algorithm 2 computes the exclusive prefix sum of the original input array. The exclusive prefix sum of an array is defined as the sum of all the values preceding a particular position in the array not including the value in the position itself. Figure 9 gives an example of the down-sweep operation performed on the output shown in Figure 8 in order to calculate the exclusive prefix sum for the original input given in Figure 8. The first step of the down-sweep operation is to replace the last element (root element) in the array obtained after the up-sweep operation with the value 0 . Then in the consecutive steps, the parent element at each array is copied to the left element of the child array and the right element of the child array is calculated as the sum of the old left element and the parent element. In effect, every element now contains the sum of all the elements to the left of itself in the tree structure.

Algorithm 2: The down-sweep algorithm to construct the inclusive prefix sum.

<!-- image -->

Figure 9: Example of the down-sweep operation performed on the original 1D array given in Figure 8. The elements corresponding to the values of 1 in the original input are highlighted in the result; these are the addresses where those values are to be scattered.

<!-- image -->

The value of the exclusive prefix sum at the positions where the value of the input array is 1 gives the address to which that particular input value has to be scattered to perform the stream reduction. The final step is the scatter operation in which this address is used to reduce the input stream such that the elements with value 1 are collected at the front of the array.

However, we cannot directly use this stream reduction algorithm on the GPU due to three main reasons. The first reason is that the original algorithm was developed for 1-dimensional arrays and hence has to be adapted to operate on a 2-dimensional texture. The second reason is that the traditional GPGPU model does not allow the scatter operation, which is the last step of the stream reduction algorithm. Finally, the original formulation by [Blelloch 1990] computed the prefix sum in situ by modifying the input array. This is not possible using the standard GPGPU framework since we cannot read and write to the same location simultaneously.

We solve the first problem by first assuming that each row of the texture is a separate array and compute the first part of the up-sweep operation until each row array is reduced to a single element. Now we again perform the up-sweep operation on the array formed by concatenating all the single elements in a column along the column direction. In the example shown in Figure 10(b), we perform the up-sweep operation on each row until we end up with the values in column 7 . Then we perform the up-sweep operation on column 7 and output the results to column 8 . As shown in the example, to overcome the restriction of reading and writing to the same memory location, we maintain a hierarchy of the input texture. This method uses only twice the storage as the original texture and a single fragment program written to perform the summation can be repeatedly used. We compute the up-sweep operation in O (log n ) passes.

Figure 10: Different steps of the GPU stream reduction algorithm.

<!-- image -->

We then perform the down-sweep operation in a similar manner but in reverse order, by first performing the operation along the columns and then extending it to the rows to obtain the exclusive prefix sum of the input. In the example shown in Figure 10(c), each bold box contains the exclusive prefix sum of the corresponding bold box in Figure 10(b).

Once we have the output from the down sweep operation we extract the address of only those texels which have the value 1 in the input texture (Figure 10(d)). We reinterpret this texture as a VBO and use a vertex program written to output the addresses of the input values with value 1 as ( x, y ) coordinates, to write to two separate channels of the output texture. The size of the output texture varies based on the number of elements with value 1 in the input texture; it is equal to the first square number larger than the number of elements with value 1 in the input. This output texture is then directly used by the inverse evaluation and the surface-surface intersection applications for further processing.

## 5.2 GPU Implementation of Inverse Evaluation

The algorithm used for performing the full inverse evaluation is given pictorially in Figure 11. The three steps in the top row of Figure 11-evaluating the surface, constructing bounding-boxes, and finding intersecting boxes-are performed on the GPU. The data corresponding to the selected bounding-box is read back from the GPU. We then check on the CPU whether the ranges in the parametric domain of the surface as well as the size of the bounding-box are within the required tolerance; for example, we can use an absolute tolerance of 10 -6 in the parametric space and a relative tolerance of 10 -3 in the model space. If the tolerance conditions are met, we output the midpoint of the parametric range as the output of the inverse evaluation. If not, we re-evaluate the NURBS surface at a finer resolution within the previously output parametric range(s). The required tolerance conditions are usually met within two or three iterations since we evaluate the surface at a high resolution ( 1024 × 1024 ) during each iteration.

## 5.3 Applications of Inverse Evaluation

We can build different modeling operations using the inverse evaluation algorithm as the basic module. These operations include ray intersections, direct sketching on NURBS surfaces, and interactive trimming. Figure 12(a) shows an example where we compute all the intersection points (two in this case, marked in red) of a particular ray with the surfaces of a toy model. By aligning the ray direction perpendicular to the view plane, we can use the same algorithm for selecting a particular surface from a given set of NURBS surfaces.

Figure 11: Algorithm for inverse evaluation of NURBS surfaces.

<!-- image -->

One of the most important advantages of a real-time algorithm to perform inverse evaluation is the ability to sketch directly on the NURBS surface. The advantage comes from the fact that the curve is simultaneously sketched both in the 3-dimensional model space as well as in the 2-dimensional parameter space. This helps in performing modeling operations like extrusions and trimming, where the parameter space sketches are typically used for defining these operations. Figure 12(b) shows a curve sketched on a NURBS model and the curve in the parametric domain is shown in the inset.

<!-- image -->

(c)

Interactive trimming:

the eyes of

the model were trimmed interactively

Figure 12: Different NURBS modeling applications using inverse evaluation.

By combining our sketching interface with the algorithm that renders trimmed NURBS surfaces in real-time, we can perform interactive trimming operations (Figure 12(c)). Using our interactive trimming application, the designer gets immediate feedback on the result of the trimming operation, unlike current commercial CAD systems.

## 6 NURBS Intersection Curve Evaluation

Calculating the intersection curve of a surface-surface intersection is a frequently encountered operation in CAD systems. It forms an essential part of important CAD operations like trimming, filleting, and b-rep generation from Boolean operations. However, since it is a slow operation, it is usually performed in the background and thus the user does not get real-time feedback except in the simplest of cases. We present a GPU-accelerated surface-surface intersection algorithm to calculate intersection curves both in the model space as well as in the parametric spaces of both the surfaces.

We now give a broad overview of our surface-surface intersection algorithm. Our algorithm makes use of bounding-box hierarchies to accelerate the intersection operation. We evaluate both intersecting surfaces using the GPU and then use the method described in Section 4 to construct the AABBs for the surfaces, using the same coordinate frame. We construct a hierarchy of bounding-boxes by combining four bounding-boxes at one level to construct a single bounding-box in the next level. To find the intersection curve, we then traverse along the hierarchy simultaneously for both the surfaces and find the intersecting bounding-boxes in the lowest level using the GPU. At the same time, we also get the ranges in the parametric domain corresponding to the intersecting surface patches. We then check if the size of the bounding-boxes as well the parametric ranges are within a user-defined tolerance. Once the tolerance conditions are met, we get a better estimate of the point on the intersection curve by intersecting the linearized surface patch within the intersecting bounding-boxes.

Figure 13: Example hierarchical bounding-box comparison in the surface-surface intersection algorithm.

<!-- image -->

We will explain the details of our surface-surface intersection algorithm with an example (Figure 13). Given two surfaces, S 1 and S 2 , we evaluate them and construct their bounding-boxes as explained in previous sections. We also construct the bounding-box hierarchies for both the surfaces and store them on the GPU as textures. Once we have the hierarchies, we use the CPU to test whether the bounding-boxes of the surfaces intersect at the top-most level, level 1. If so, we then test the bounding-boxes from the next level onwards on the GPU, using one pass per level. We perform the intersection tests for all the bounding-boxes in a level in parallel using a fragment program written to perform the bounding-box intersection test. The input to the fragment program is a texture called the address texture that contains the address of the bounding-boxes in the hierarchy (also stored as textures). For example, to test for intersection in the second level, we make use of a 4 × 4 address texture on the GPU, where we test for intersection of a bounding-box of S 1 with all the four bounding-boxes of S 2 . In Figure 13, the rows of the address texture (Level 2) corresponds to bounding-boxes from S 1 and the columns correspond to bounding-boxes from S 2 . The address texture is a 4-component texture consisting of the address corresponding to bounding-boxes of S 1 and S 2 in the bounding-box hierarchy textures ( ( u 1 , v 1 , u 2 , v 2 ) stored using RGBA channels). The intersection test is performed on the GPU using a fragment program, which uses the address information to retrieve the data for the bounding-boxes from the bounding-box hierarchy and subsequently tests them for intersection. The output of the fragment program is a binary texture with a value of 1 indicating an intersection. We use the stream reduction algorithm explained in Section 5.1 to find the address of the intersecting bounding-boxes. In the example shown, we find that bounding-box 3 of S 1 intersects with bounding-boxes 1 and 3 of S 2 at level 2.

In the next level (pass), we test for the intersection of the children of the intersecting bounding-box pairs of the previous level simultaneously on the GPU. Thus, the size of the address texture varies dynamically based on the number of intersections in the previous levels. The size of the address texture is always a multiple of 4 since we test for intersection between S 1 and S 2 in blocks of 4 × 4 intersection tests. However, we make sure that this is a square texture and its size is a power-of-2 to optimize the stream reduction algorithm. The parallelism of the GPU is exploited in checking for intersection of all the intersecting bounding-box pairs at any given level and this helps in accelerating the intersection algorithm as the address texture grows in size. Once we reach the base level of the bounding-box hierarchy, we get a list of the bounding-boxes that intersect at this level (Figure 14). This list can then be used for further processing on the CPU to get the actual intersection curve.

Figure 14: Intersecting bounding-boxes of two NURBS surfaces.

<!-- image -->

In addition, we use this list to render the points on the intersection curve of each surface to a dynamic texture in the parametric domain. We map this texture back onto each surface, providing realtime feedback to the designer about the shape of the intersection curve (Figure 15).

## 6.1 Fitting an Intersection Curve

To get a better estimate of the intersection point lying on the intersection curve of two surfaces, we intersect the surface sub-patches enclosed by the intersecting bounding-boxes on the CPU. We approximate each surface sub-patch inside the bounding-box with two triangles that share an edge. We intersect these two triangles contained inside the bounding-box of the first surface with the two other triangles contained in the bounding-box of the second surface. This gives rise to four pairs of intersection tests between the triangles of the two surfaces; each intersection test can be true or false, generating 16 different cases. We show one particular case in Figure 16, where one triangle of surface S 1 intersects with another triangle of surface S 2 . The four triangles are denoted as A 0 A 1 A 2 and A 1 A 2 A 3 for surface S 1 , and B 0 B 1 B 2 and B 1 B 2 B 3 for surface S 2 in the figure. We find the midpoint of the intersection line-segment and use this midpoint as a point on the intersection curve if it lies within the intersecting region of the bounding-boxes. The intersecting region of the bounding-boxes is denoted by ( x min , y min , z min ) and ( x max , y max , z max ) in the figure. In the case of multiple intersections, we take the centroid of the midpoints of the intersection line-segments computed for each intersecting triangle pair as a point on the intersection curve. Only if this centroid lies inside the intersecting region of the bounding-boxes do we use this point for fitting a curve.

Figure 15: Intersection curves of two NURBS surfaces plotted both in the model space as well as in their corresponding parametric spaces.

<!-- image -->

Figure 16: Intersecting triangles inside overlapping bounding-box pairs to get a better estimate of the point on the intersection curve.

<!-- image -->

We then extract the 7-tuple ( x, y, z, u 1 , v 1 , u 2 , v 2 ) for each point found on the intersection curve using the above method, where ( x, y, z ) is the point on the intersection curve in 3D space, ( u 1 , v 1 ) and ( u 2 , v 2 ) are the corresponding points in the parametric space of surface S 1 and surface S 2 respectively. The parametric points are found by computing the barycentric coordinate of the ( x, y, z ) intersection point in each of the corresponding intersecting triangles and then interpolating the parametric coordinates at the three vertices of the triangle linearly using the barycentric coordinates.

Finally, to compute the actual intersection curves themselves from the list of points, we make use of a greedy algorithm (Algorithm 3) that computes the intersection curves by successively merging polylines that are close to each other. We work in the 7-dimensional space ℜ 7 , integrating the data from both the model space as well as the two parametric spaces.

Algorithm 3: Algorithm to fit polylines to the points on the intersection curves.

<!-- image -->

Since our input list of points on the intersection curve is sufficiently dense, we fit a polyline that passes through these points. If a more compact representation is required, we can fit a NURBS curve of any required order that approximates the points on the intersection curve using standard curve fitting techniques. Since the intersection points obtained from our algorithm are enclosed within their corresponding bounding boxes both in the model space and in the parametric space, we can guarantee a required bound on the results. In addition, if the arbitrary user-defined bounds are small enough, we are guaranteed not to miss any portion of the intersection curve. Since we also give instantaneous visual feedback to the user, the user can immediately know if there are any features missing and can reduce the tolerance to obtain the correct result.

The time taken to fit a polyline depends on an efficient closest neighbor query. Currently, we perform this operation through an exhaustive search that takes O ( n 2 ) time, which could be optimized by using more efficient search techniques. For the example shown in Figure 15, the polyline fitting for over 7000 points takes about 0.1 seconds on a 2GHz PC for a tolerance value of 2 × 10 -3 .

## 6.2 Intersection Points Computation Timing

We timed our GPU-accelerated algorithm for evaluating the points on the intersection curves on a 3GHz CPU with 2GB of RAM equipped with a nVIDIA Quadro FX4500 graphics card with 512MB graphics memory. We performed a surface-surface intersection of the two NURBS surfaces shown in Figure 17. The surfaces were bi-cubic NURBS with 403 × 199 and 298 × 313 control points respectively. We compare our timings to evaluate the intersection curves to the required user-defined tolerance with those of the commercial solid modeling kernel ACIS.

Figure 17: NURBS surfaces used for timing the evaluation of intersection curves.

<!-- image -->

Figure 18: Time taken for evaluating the intersection curves of the two NURBS surfaces shown in Figure 17 with different resolutions. Note that we are evaluating many more points on the intersection curve for a given resolution (Figure 19).

<!-- image -->

Figure 18 compares the time for evaluating the points on the intersection curves by varying the tolerance values. In the case of of our algorithm, these times do not include the time to fit the polyline as described above. Our GPU-accelerated evaluation is about 40 times faster than ACIS in computing the intersection curves to the standard tolerance of 10 -3 used in ACIS. The output from ACIS is an interpolated polyline where the points on the polyline are within the user-defined tolerance value from the exact intersection curve. ACIS does not guarantee any tolerance on the piecewise linear line segments that make up the polyline [Corney and Lim 2001]. On the other hand, we evaluate dense intersection points with their spacing adjusted based on the tolerance to achieve a guaranteed tolerance on the piecewise linear segments of the polyline as well. We compute almost 50 times as many points on the intersection curve as ACIS does for the standard ACIS tolerance value of 10 -3 (Figure 19).

Figure 19: Number of points evaluated on the intersection curve for different resolutions.

<!-- image -->

Table 1 gives the breakdown of the timing of our intersection algorithm for evaluating the intersection curves shown in Figure 15 for a tolerance value of 10 -3 . The evaluation of the NURBS surfaces is a large fraction of the total time. Note that we do not require such high tolerance values for giving visual feedback; hence, it can be performed at interactive rates.

Table 1: Breakdown of the timing to perform different operations of our intersection algorithm. The values are for the example shown in Figure 15 for a tolerance value of 10 -3 .

| Operation                           |   Time(s) |
|-------------------------------------|-----------|
| Evaluate NURBS surfaces             |      0.27 |
| Perform intersection tests          |      0.05 |
| Calculate dense intersection points |      0.02 |
| Total                               |      0.34 |

## 7 Conclusions

We present fast algorithms to perform interactive modeling operations on NURBS surfaces. Our algorithms do not require the latest graphics cards and are backward compatible with any graphics card that has basic programming capabilities. This is essential for the actual adoption of our algorithms in commercial CAD systems. We expect the performance of our algorithms to only improve with the advent of new and faster graphics cards.

Both our GPU algorithm to sketch on NURBS surfaces as well as our GPU-accelerated algorithm to calculate intersection curves give real-time feedback to the designer about the shape of the curves in the parametric space. This gives a direct handle for the designer to check for inconsistency if models fail during rebuilds in a CAD system. Our interactive trimming tool helps the designer to easily interact with and edit the NURBS models. Moreover, the applications that we have outlined in our paper form only a small part of the different kinds of applications that can be developed with the help of GPU-accelerated basic modeling operations. There is large potential for developing diverse applications that use these operations as building blocks.

## Acknowledgments

We would like to thank Wei Li for helping us in the background research for parallel stream reduction. We would also like to thank nVIDIA and AMD for providing us with their hardware. This material is based upon work supported in part by SolidWorks Corporation, UC Discovery under Grant No. DIG05-10190, and the National Science Foundation under Grant No. 0547675.

## References

ABI-EZZI, S. S., AND WOZNY, M. J. 1990. Factoring a homogeneous transformation for a more efficient graphics pipeline. Computer Graphics Forum 9 , 3, 245-255.

BARNHILL, R. E., AND KERSEY, S. N. 1990. A marching method for parametric surface surface intersection. Computer Aided Geometric Design 7 , 1-4, 257-280.

BLELLOCH, G. E., Ed. 1990. Vector Models for Data-Parallel Computing . MIT Press.

CARR, N. A., HOBEROCK, J., CRANE, K., AND HART, J. C. 2006. Fast GPU ray tracing of dynamic meshes using geometry images. In GI 2006: Proceedings of Graphics Interface 2006 , Canadian Information Processing Society, 203-209.

CORNEY, J., AND LIM, T. 2001. 3D Modeling with ACIS . SaxeCoburg.

DE BOOR, C. 1978. A practical guide to splines . Springer-Verlag.

- ELBER, G., AND KIM, M.-S. 2001. Geometric constraint solver using multivariate rational spline functions. In SMA 2001: Proceedings of the sixth ACM symposium on Solid Modeling and Applications , ACM, 1-10.
- FILIP, D., MAGEDSON, R., AND MARKOT, R. 1987. Surface algorithms using bounds on derivatives. Computer Aided Geometric Design 3 , 4, 295-311.
- GOVINDARAJU, N. K., REDON, S., LIN, M. C., AND MANOCHA, D. 2003. CULLIDE: Interactive collision detection between complex models in large environments using graphics hardware. In HWWS '03: Proceedings of the ACM SIGGRAPH/EUROGRAPHICS conference on Graphics hardware , Eurographics Association, 25-32.
- GRESS, A., GUTHE, M., AND KLEIN, R. 2006. GPU-based collision detection for deformable parameterized surfaces. Computer Graphics Forum 25 , 3, 497-506.
- GUTHE, M., BAL´ AZS, A., AND KLEIN, R. 2005. GPU-based trimming and tessellation of NURBS and T-spline surfaces. ACM Transactions on Graphics 24 , 3, 1016-1023.
- GUTHE, M., BAL´ AZS, A., AND KLEIN, R. 2006. GPU-based appearance preserving trimmed NURBS rendering. Journal of WSCG 14 .
- HOFFMANN, C. M. 1989. Geometric and Solid Modeling . Morgan Kaufmann Publishers Inc.
- HORN, D. 2005. GPU Gems 2 : Programming Techniques for High-Performance Graphics and General-Purpose Computation . Addison-Wesley, ch. Stream Reduction Operations for GPGPU Applications, 573-590.
- KANAI, T. 2007. Fragment-based evaluation of Non-Uniform Bspline surfaces on GPUs. Computer-Aided Design and Applications 4 , 3, 287-294.
- KIPFER, P., SEGAL, M., AND WESTERMANN, R. 2004. UberFlow: a GPU-based particle engine. In HWWS '04: Proceedings of the ACM SIGGRAPH/EUROGRAPHICS conference on Graphics hardware , ACM, New York, NY, USA, 115-122.
- KOLB, A., LATTA, L., AND REZK-SALAMA, C. 2004. Hardwarebased simulation and collision detection for large particle systems. In HWWS2004: Proceedings of the ACM SIGGRAPH/EUROGRAPHICS conference on Graphics hardware , ACM, New York, NY, USA, 123-131.
- KRIEZIS, G. A., PRAKASH, P. V., AND PATRIKALAKIS, N. M. 1990. A method for intersecting algebraic surfaces with rational polynomial patches. Computer Aided Design 22 , 10, 645-654.
- KRISHNAMURTHY, A., KHARDEKAR, R., AND MCMAINS, S. 2007. Direct evaluation of NURBS curves and surfaces on the GPU. In SPM '07: Proceedings of the 2007 ACM symposium on Solid and Physical Modeling , 329-334.
- KRISHNAN, S., AND MANOCHA, D. 1997. An efficient surface intersection algorithm based on lower-dimensional formulation. ACM Transactions on Graphics 16 , 1, 74-106.
- LOOP, C., AND BLINN, J. 2005. Resolution independent curve rendering using programmable graphics hardware. In ACMSIGGRAPH 2005 , ACM, 1000-1009.
- MARTIN, W., COHEN, E., FISH, R., AND SHIRLEY, P. 2000. Practical ray tracing of trimmed NURBS surfaces. Journal of Graphics Tools: JGT 5 , 1, 27-52.
- NISHITA, T., SEDERBERG, T. W., AND KAKIMOTO, M. 1990. Ray tracing trimmed rational surface patches. In ACM SIGGRAPH 90 , 337-345.
- PATRIKALAKIS, N. M. 1993. Surface-to-surface intersections. IEEE Comput. Graph. Appl. 13 , 1, 89-95.
- PIEGL, L. A., AND TILLER, W. 1997. The NURBS Book, Version 1.2 , second ed. Springer.
- PURCELL, T. J., BUCK, I., MARK, W. R., AND HANRAHAN, P. 2002. Ray tracing on programmable graphics hardware. ACM Transactions on Graphics 21 , 3, 703-712.
- PURCELL, T. J., DONNER, C., CAMMARANO, M., JENSEN, H. W., AND HANRAHAN, P. 2003. Photon mapping on programmable graphics hardware. In HWWS 2003: Proceedings of the ACM SIGGRAPH/EUROGRAPHICS conference on Graphics hardware , Eurographics Association, 41-50.
- REQUICHA, A. A. G., AND ROSSIGNAC, J. R. 1992. Solid modeling and beyond. IEEE Comput. Graph. Appl. 12 , 5, 31-44.
- SENGUPTA, S., HARRIS, M., ZHANG, Y., AND OWENS, J. D. 2007. Scan primitives for GPU computing. In GH '07: Proceedings of the 22nd ACM SIGGRAPH/EUROGRAPHICS symposium on Graphics Hardware , Eurographics Association, 97106.
- THOMPSON, T., AND COHEN, E. 1999. Direct haptic rendering of complex trimmed NURBS models. In 8th Annual Symp. Haptic Interfaces for Virtual Environment and Teleoperator Systems .
- TOTH, D. L. 1985. On ray tracing parametric surfaces. In ACM SIGGRAPH 85 , 171-179.