## Robust Containment Queries over Collections of Trimmed NURBS Surfaces via Generalized Winding Numbers

JACOB SPAINHOUR, Lawrence Livermore National Laboratory, USA KENNETH WEISS, Lawrence Livermore National Laboratory, USA

Fig. 1. CAD models composed of trimmed NURBS surfaces are ubiquitous in engineering and design, but robustly defining an interior for such objects is challenging in the presence of gaps or overlaps between adjacent patches. We present a method to evaluate the field of generalized winding numbers for such surfaces, providing a containment query that is indifferent to model watertightness. Our algorithm uses Stokes' theorem to reformulate the problem as a 1D line integral over the boundary of each patch, solved with an adaptive quadrature technique that is accurate to user tolerance. When Stokes' theorem is theoretically unfounded (i.e. near the surface), we perform a line-surface intersection test to recover the computational advantages of our boundary formulation. Edge cases near surface boundaries are handled accurately and robustly by extracting a parametric disk from the surface to be processed separately.

<!-- image -->

We propose a containment query that is robust to the watertightness of regions bound by trimmed NURBS surfaces, as this property is difficult to guarantee for in-the-wild CAD models. Containment is determined through the generalized winding number (GWN), a mathematical construction that is indifferent to the arrangement of surfaces in the shape. Applying contemporary techniques for the 3D GWN to trimmed NURBS surfaces requires some form of geometric discretization, introducing computational inefficiency to the algorithm and even risking containment misclassifications near the surface. In contrast, our proposed method leverages properties of the 3D solid angle to solve the relevant surface integral using a boundary formulation with rapidly converging adaptive quadrature. Batches of queries are further accelerated by memoizing (i.e. caching and reusing) quadrature node positions and tangents as they are evaluated. We demonstrate that our GWN method is robust to complex trimming geometry in a CAD model, and is accurate up to arbitrary precision at arbitrary distances from the surface. The

Authors' Contact Information: Jacob Spainhour, Lawrence Livermore National Laboratory, Livermore, USA, jspainhour@llnl.gov; Kenneth Weiss, Lawrence Livermore National Laboratory, Livermore, USA, kweiss@llnl.gov.

derived containment query is therefore robust to model non-watertightness while respecting all curved features of the input shape.

CCS Concepts: · Computing methodologies → Shape analysis ; Parametric curve and surface models .

Additional Key Words and Phrases: Winding number, point containment query, trimmed CAD models, robust geometry processing, trimmed NURBS

## 1 Introduction

In computer graphics and adjacent fields, 3D objects are often defined in terms of a boundary representation (B-Rep) of the surface. While B-reps are very useful for flexibly designing 3D objects within a CAD system, they often fail to be watertight , in the sense that they do not define a legitimate interior volume (see Figure 2). For example, disconnected edges between component surfaces may be an intentional design decision, but are just as likely to result from human error during modeling, or differing tolerances between applications [Kasik et al. 2005; McAdams et al. 2011] (see Figure 2). The use of Boolean operations during model construction can also introduce problematic non-manifold edges. Issues of watertightness are greatly exacerbated in the context of trimmed NURBS (nonuniform rational B-spline) surfaces, where such gaps are nearly mathematically inevitable due to the complexities of projecting 3D surface-surface intersection curves into 2D parametric trimming curves [Sederberg et al. 2008; Zou et al. 2024]. Nevertheless, we are often interested in determining whether a given point is contained within the shape in a way that conforms to the provided boundaries, but is robust enough to provide reasonable decisions in the vicinity of such geometric errors.

Fig. 2. In the exaggerated example (left) we see open and non-manifold edges, highlighted in red, that prohibit a strict definition of the shape's interior volume. Analogous issues are present in the in-the-wild example (right) which are numerically significant, but otherwise unnoticeable to a human observer and therefore difficult to predict or correct.

<!-- image -->

A potential solution is to restore model watertightness directly via surface repair, e.g. closing gaps and/or clipping overlaps, but manual correction is often time-consuming and error-prone. Automated systems for surface repair in the context of CAD models with arbitrary curved and trimmed surfaces is an active area of research, and is typically applied globally during export or meshgeneration [Marchandise et al. 2012; Smith et al. 2010; Xiao et al. 2021]. However, the focus of such work is often the creation of watertight models composed of linear elements (e.g. triangles), which necessarily degrade the geometric fidelity of the model.

Nevertheless, discretization of individual trimmed surfaces into a linear mesh permits application of not only more traditional surface repair techniques, but also the growing body of work that defines containment queries for non-watertight shapes [Barill et al. 2018; Feng and Crane 2024; Jacobson et al. 2013]. However, there are many contexts in which this type of approximation through discretization would either be too costly to capture complex geometry, or has the potential to introduce unforeseen and otherwise unacceptable errors in downstream applications.

We take multiphysics simulation as an important class of examples, and point to the work of Sevilla et al. for a comprehensive review of complications that arise when piecewise linear boundaries are used to approximate curved geometry in multiphysics contexts [Sevilla et al. 2008, 2011; Sevilla and Huerta 2018]. This has led to the development of isogeometric analysis techniques that operate directly on the NURBS representation defined by CAD geometry [Hughes et al. 2005; Marussig and Hughes 2017].

Similarly, methods for immersogeometric analysis model fluidstructure interactions by placing a complicated CAD model on a simpler computational grid which records the numerical solution [Fromm et al. 2023; Kamensky et al. 2015]. In such methods, it is necessary to know whether points on this grid are inside or outside the CAD model, which can be particularly challenging for points located arbitrarily close to the B-Rep.

In this work, we seek to extend the capabilities of these and other techniques by introducing the first direct evaluation method for containment queries on non-watertight B-Reps composed of trimmed NURBS surfaces (see Figure 1). We accomplish this through the generalized winding number (GWN) [Jacobson et al. 2013], a mathematical construction that is useful not just in defining such a query, but in a variety of other geometry processing routines, including Boolean operations [Trettner et al. 2022] and surface reconstruction [Jacobson et al. 2013]. Specifically, we present in this work the following principal contributions:

- We extend the theory of generalized winding numbers to the context of trimmed NURBS patches. With this, we define a containment query for B-Reps composed of unstructured collections of such shapes that is robust to (and indeed indifferent to) watertightness.
- We describe a novel algorithmic framework for evaluating the GWN with respect to a trimmed NURBS surface, greatly improving efficiency and stability by reformulating the relevant surface integral as a line integral along the surface boundary. We evaluate these integrals with an adaptive quadrature scheme to achieve results up to arbitrary accuracy, and further improve performance with a memoization strategy that caches and reuses geometric calculations for each surface. Within our algorithm, we robustly handle trim tests (visibility queries over trimming curves in the 2D parameter space of each surface) with the 2D GWN.
- Weaccurately evaluate the 3D GWN field for points arbitrarily close and even coincident with the surface by directly identifying and compensating for the jump discontinuity in the GWN field across surfaces. This permits evaluation of the GWN both in the 'far-field' and 'near-field' of the NURBS surface without any discretization.

Practical applications of the GWN typically use a spatial index to accelerate queries (e.g. in the works of Jacobson et al. [2013] and Barill et al. [2018]), which we intend to pursue in follow-up work. Instead, our focus in this work is on exact and efficient evaluation of the GWN with respect to individual trimmed NURBS surface patches, the cost of which scales nearly linearly with the number of trimming curves. Our algorithm has been implemented in Axom, a BSD-licensed open-source library of Computer Science infrastructure for HPC applications [Capps et al. 2025].

## 2 Background and Related Work

## 2.1 Containment Queries in 3D CAD Applications

There are many strategies to evaluate containment queries for CAD applications. In the typical ray casting method, an arbitrary ray is extended out from the point of interest and the number of intersections with bounding geometry are recorded. Assuming a watertight B-Rep, each such intersection increments a crossing number whose parity determines containment.

There is considerable overlap with the adjacent task of ray tracing for the purposes of rendering, which typically considers a specific, pre-selected ray direction, and only requires calculation of the first intersection of a ray with the B-Rep. Much of the research focus in ray tracing relates to triangulated surfaces, as many hardware platforms are capable of calculating ray-triangle intersections directly, exactly, and massively in parallel [Haines and Akenine-Möller 2019, Chapter 3]. However, there is also interest in applying such techniques directly to trimmed NURBS surfaces [Schollmeyer and Fröhlich 2009, 2019; Xiong et al. 2023; Zhu et al. 2025]. Of particular note are recent methods for ray tracing bilinear patches, as, similar to triangles, they admit direct geometric formulae for computing their intersections with rays [Reshetov 2019].

On more general curved surfaces, surface-line intersection routines are more complex and iterative [Efremov et al. 2005; Marschner et al. 2021; Martin et al. 2000; Nishita et al. 1990; Shen et al. 2016]. Although the direction of the cast ray is arbitrary in principle, containment strategies for watertight shapes which utilize ray casting are highly sensitive to the specific configuration of surface and cast ray. This issue is compounded in the non-watertight case, as a cast ray passing through a numerical gap between surfaces results will result in an unexpectedly incorrect containment result.

Alternatively, winding number methods count the number of times the shape encloses the point, thereby implicitly handling configurations which are difficult for ray casting, even for watertight shapes [Carvalho and Cavalcanti 1995; Hormann and Agathos 2001]. These methods can then be generalized by summing the signed angle subtended by individual boundary components so that the resulting containment decision is indifferent to watertightness (see Figure 3).

The generalized winding number (GWN) is a real-valued scalar field that degrades smoothly, yet rapidly away from integer values near gaps and/or overlaps between boundary components. This field can be interpreted as defining containment, for example, by rounding its values to the nearest integer and applying either a non-zero or even-odd rule. Although other uses of the GWN field are possible, we are primarily concerned with its evaluation for the purposes of defining this robust containment query.

## 2.2 GWNEvaluation for Discretized Surfaces

The generalized winding number was first conceptualized to provide containment queries on messy STL geometry, doing so via a hierarchical spatial index and divide-and-conquer algorithm [Jacobson et al. 2013]. This work was later extended to entirely disconnected triangle soups by Barill et al. [2018], improving computational performance with an error-controlled approximation of the GWN with respect to groups of triangles that are far from the query point.

Evaluation methods for the GWN field defined by triangle meshes and soups are particularly computationally efficient, as the solid angle which defines the 3D GWN can be directly and exactly computed for polygonal shapes. However, if the input surfaces are curved, a naïve application of these methods must necessarily use some geometric approximation of the surface to obtain the necessary input type, which in turn causes the GWN field of the approximation to diverge from the ground truth. At the same time, even generating a tessellation of a trimmed NURBS surface is a non-trivial task, and high-resolution meshes can be prohibitively expensive to generate and store for complex models [Xiong et al. 2023]

Fig. 3. In 2D (top) and 3D (bottom), the GWN of watertight shapes (left) is integer valued (middle), while the fractional GWN of a non-watertight shape (right) is equal to the signed angle subtended by its boundary. To more easily visualize the 3D GWN as a scalar field, we consider a 2D slice of the field as it intersects with the surface.

<!-- image -->

The work of Barill et al. [2018] defined the GWN of oriented point cloud data, which could be sampled directly from a CAD-derived surface. Indeed, Balu et al. [2023] do so in the context of multiphysics simulation, describing a method which uses a point cloud randomly sampled from a CAD model to define an interior volume for the otherwise messy or complex shape. However, even if the weights for each point are accurately computed from the ground-truth surface (which may not be the case depending on implementation), such a discretization results in significant errors in the evaluated GWN field, particularly for points near the surface.

In comparing our objectives to these existing methods (or naïve adaptations of existing methods to curved geometry), we find it useful to distinguish between two types of errors in the computed GWNfield. The first is inaccuracy in the GWN calculation, where the error in the GWN field causes the derived containment query to result in misclassifications. This is almost exclusively a concern for points near the surface, particularly when the GWN is computed using a low-quality intermediate discretization of the surface. In such cases, the query point can be identified as on one side of the discretized surface while being on the opposite side of the true surface, resulting in an off-by-one error in the GWN field.

The second is numerical imprecision in the computed fractional component of the GWN field. While it is necessarily true in 2D that, for a fixed set of input queries, there exists some linear discretization of the input curve that produces the correct GWN field at each query, this is no longer the case in 3D. Even if a query point is correctly placed on the proper side of the surface, the mismatch between the boundaries of the original surface and any triangulation introduces imprecision in the computed field that is difficult to predict from the triangulation alone (see Figure 4). While these errors decrease with increasing mesh resolution, they never vanish completely.

Fig. 4. Comparing the difference (right, log scale) between the GWN field of the original curved geometry (left) and its linearly discretized approximation (middle). While the induced approximation error in 2D (top) is integervalued, the analogous distribution of errors in 3D (bottom, displayed on a 2D slice of the 3D surface shown in the inset) is non-zero almost everywhere.

<!-- image -->

As we shall see, it is common for 3D GWN evaluation methods to control for each type of error independently. In general, we find that methods which discretize the surface geometry for the purposes of evaluating the GWN field precisely for far-away query points often do so at the cost of accuracy nearer to the surface. Instead, our work introduces a GWN method that is accurate at arbitrary points relative to the input surface, and maintains precision through error-controlled numerical methods, all in service of avoiding misclassifications in the derived containment query.

## 2.3 GWNEvaluation for General Surfaces

In this section we describe contemporary methods for evaluating the GWN field of curved surfaces in 3D, and more broadly explain the fundamental differences from the 2D analog.

The 2D GWN 𝑤 𝐶 ( 𝑞 ) at a query point 𝑞 with respect to a curve 𝐶 is defined as the total signed angle subtended by the curve, translated to place 𝑞 at the origin, and normalized by a factor of 2 𝜋 . This results in the following integral of the (signed) differential angle 𝑑𝜃 ,

<!-- formula-not-decoded -->

where 𝐶 -𝑞 denotes translating the curve to make 𝑞 the origin.

Evaluating this line integral directly via numerical quadrature is difficult, as the near-singular behavior of the integrand means that for points close to the curve, conventional quadrature methods are very unstable. For linear elements with (translated) endpoints 𝑎 and 𝑏 , however, this subtended angle has a simple and direct formula,

<!-- formula-not-decoded -->

This, along with the property that closed curves have integer-valued winding numbers, has been leveraged to compute the GWN of an arbitrary curve in 2D without the use of any quadrature scheme by subtracting the GWN of the curve's closure from that of the closed shape [Spainhour et al. 2024]. Indeed, for far away points, this continues to be the most efficient way of processing any individual curve for any single query point [Liu et al. 2025].

Theoretically, this framework can be applied directly to the 3D problem. The 3D GWN 𝑤 𝑆 ( 𝑞 ) at query point 𝑞 with respect to a surface 𝑆 is defined as the total solid angle subtended by the surface, again translated by 𝑞 , and normalized by a factor of 4 𝜋 :

<!-- formula-not-decoded -->

which can be written in Cartesian coordinates as:

<!-- formula-not-decoded -->

There exist a number of quadrature techniques in the literature that could be immediately applied to Equation 4, many of which can in principle be used even in the presence of complicated trimming geometry [Chin and Sukumar 2021; Gunderman et al. 2020, 2021]. However, the integrand's fundamental near-singularity makes direct numerical evaluation difficult for query points near the surface.

The hope, then, is that this too can be avoided through an exact formula for the analogous linear element. Indeed, the GWN for a 3D triangle defined by (translated) vertices 𝑎 , 𝑏 , and 𝑐 is given by

<!-- formula-not-decoded -->

But while any 2D curve can be closed with a straight line, only a trivial subset of 3D surfaces can be exactly closed by a collection of triangles. For more general 3D surfaces, the closing surface must also be a high-order surface [Sederberg et al. 2008], and so computing the GWN of the closing surface (for the purposes of subtracting the value from an integer winding number of the closed surface) is typically just as difficult as computing the GWN of the original.

To find a more direct and consistent solution method, we utilize a known property of the 3D solid angle which states that in certain contexts, it and the derived GWN field can be defined entirely from the boundary of the surface [Binysh and Alexander 2018; Gillespie et al. 2024; Jacobson et al. 2013]. In this spirit, one could imagine adapting the method of Jacobson et al. [2013] to a 3D curved surface 𝑆 by constructing a linear approximation of the surface boundary 𝜕𝑆 ′ (with 𝜕𝑆 ′ ≈ 𝜕𝑆 ), along with a coarse triangulated closure 𝑆 ′ whose GWN field can be computed exactly. For query points far from the original surface, this approach is somewhat reasonablealthough some precision is lost in the discretization of 𝜕𝑆 ′ , the GWN field of the approximated closure is related to the GWN field of the original surface for far-away points through the simple formula 𝑤 𝑆 = -𝑤 𝑆 ≈ -𝑤 𝑆 ′ .

However, the simplicity and practicality of this boundary-focused approach necessarily breaks down for near-surface points. To reiterate, deriving 𝑤 𝑆 from 𝑤 𝑆 in general requires calculating 𝑤 𝑆 ∪ 𝑆 , i.e. the integer winding number of the query within the closed surface, which requires consideration of the original, undiscretized surface to preserve accuracy close to the surface. In this case, using 𝑆 ′ as a computational proxy for 𝑆 complicates things further, as 𝑤 𝑆 ∪ 𝑆 ′ is no longer integer valued, since, by construction, 𝑆 and 𝑆 ′ do not share the same boundary.

In this way, accurate calculation of the 3D GWN requires consideration of the entire undiscretized surface, including its boundary. To our knowledge, only the method of Sawhney and Crane [2020] handles curved input directly without discretization, solving certain classes of PDEs-including that which implicitly defines the GWN-by means of random walks and closest-point projections. In the vernacular of the previous section, this method is accurate near the surface interior by respecting the geometric fidelity of curved surfaces, but risks a noisy and inconsistent containment query near surface boundaries, and loses precision for individual queries due to its stochastic nature.

Overall, our proposed GWN method compensates for the complexities of 3D space by replacing all forms of discretization error, as visualized in Figure 4, with a rapidly converging adaptive quadrature method whose error can be tightly controlled.

## 2.4 GWNEvaluation for Trimmed NURBS Surfaces

An advantage of working with trimmed NURBS is that they provide direct access to the boundary curves of each surface as a collection of 2D trimming curves defined in the surface's parameter space. Each trimmed NURBS surface within a model can have several complicated and disconnected boundary components, and it is critical that we assume no structure between the component surfaces in the input B-Rep, as this is what makes any GWN algorithm robust to the shape's watertightness.

To explore the implications of this problem context, we consider the recent 'One-Shot' approach of Martens and Bessmeltsev [2025] which, like the idealized adaptation of Jacobson et al. [2013] to curved surfaces in the previous section, evaluates the GWN of parametric surfaces with open edges by discretizing only the open edges within connected components of the shape. These polyline edges are projected onto a unit sphere around the query point, partitioning it into a graph of spherical polygons. After mapping this collection to integer crossing numbers determined through a single ray-surface intersection test, the total GWN is evaluated as the sum of each integer, weighted by the approximate surface area of the associated polygon. The quality of the boundary discretization naturally influences the precision of the area calculation, but also the accuracy of the subsequent containment decision.

While the suggested ray-surface intersection routine does take into consideration the original (undiscretized) surface, the mapping of crossing numbers to spherical polygons may still introduce off-byone errors depending on the quality of the boundary discretization. Although careful selection of the ray direction may reduce the likelihood of misclassification, this risk represents a fundamental limitation of using unconsidered discretizations of the input shape.

Although the One-Shot approach is aimed at computing the GWN for parametric surfaces, we find its high-level approach unsuitable for our problem context. For example, a stated limitation of the One-Shot method is that it is most effective for surfaces with simple boundaries, as is typical for the preprocessed curve networks used as examples in that work. On the other hand, a trimmed NURBS

surface can have an arbitrarily complicated boundary, and so partitioning the unit sphere projection of these boundaries becomes combinatorially intractable in practical contexts. Importantly, while the cost of the line-surface intersection procedure is reduced for query points which are far from the surface (and could theoretically be eliminated with minor algorithmic adjustments), the cost of this partitioning procedure remains relatively constant across space, and even demands a higher-resolution boundary discretization as query points become further away and partitioned regions become smaller. Even so, as we demonstrate in Section 4.1, our method is more performant even when applied to simple shapes.

Additionally, working with large collections of input surfaces means that the vast majority of query points will be far from all but a few surfaces. As we will see, our approach efficiently handles this scenario by operating directly on the provided boundary curves without the need for any auxiliary data structure or intersection routine. Although both the proposed algorithm and the One-Shot method must be run on each component surface individually for a given query point, our approach effectively amortizes the increased cost of handling the few nearby surfaces.

## 3 Method

In this work, we present a generalized winding number algorithm that can be accurately evaluated at arbitrary points in space relative to the CAD model, with precision up to an arbitrary user tolerance. We assume that each component surface is an oriented trimmed tensor-product NURBS patch, with trimming curves that are explicitly defined in the parameter space of the knot spans of the patches by closed loops of 2D NURBS curves. This is the default representation for trimming curves in the STEP and IGES interchange formats and a common assumption in the literature [Hamann and Tsai 1996; Krishnamurthy et al. 2009; Piegl and Tiller 1997; Piegl and Richard 1995; Sheng and Hirsch 1992; Sloup and Havran 2021]. The orientation of the patch is determined by the direction of each parameter-space knot span. Since we use the 2D GWN method of Spainhour et al. [2024] to robustly determine containment in the parameter space of each patch, we treat the collection of trimming curves as unstructured, significantly simplifying their processing. For each query point, we evaluate our algorithm for each component surface in the model and sum the results, making the algorithm indifferent to watertightness by the linearity of the GWN.

## 3.1 Algorithm Overview

Our algorithm is based on a direct application of Stokes' theorem, which reduces the dimension of Equation 4 to an integral over the boundary of each surface. For notational simplicity, we assume that the surface 𝑆 is translated so that the query point 𝑞 is at the origin.

For each translated surface 𝑆 of the model, we evaluate the GWN at query point 𝑞 = 0 by evaluating

<!-- formula-not-decoded -->

where 𝑞 is on the positive side of 𝑆 if for intersection ( 0 , 0 , 𝑧 0 ) , 𝑧 0 &gt; 0.

The first term is the Stokes'-derived boundary formulation, which we evaluate using an error-controlled adaptive Gaussian quadrature scheme. This evaluation is accelerated across batches of queries through a memoization strategy which caches and reuses geometric data for each surface, such as surface points and tangent vectors at quadrature nodes (detailed in Section 3.4). The second term is a correction term that accounts for any jump discontinuities in the GWNfield over the surface.

The specific evaluation strategy used for a given point, as well as the associated computational costs, varies according to its relative position to the surface 𝑆 and its boundary edges 𝜕𝑆 , as shown in Figure 1. These cases can be categorized as:

- Far-field : For points sufficiently far from the surface, i.e. outside a bounding box encompassing the surface, the number of intersections is known to be zero. As such, we evaluate the GWNdirectly with our unadjusted boundary formulation.
- Near-field : For points closer to the surface, we use the same boundary formulation, but must also perform a line-surface intersection test to identify an appropriate correction term.
- Edge-cases : Some points, such as those near a boundary edge of 𝑆 require special treatment. In these cases, we split 𝑆 along new trimming curves to extract the problematic portion and reprocess the resulting surfaces.

The specific form of Equation 6 assumes that the correction term is determined by the 𝑧 -axis. However, the intersection test can be performed with an arbitrary oriented line by subsequently rotating the surface so that the positive direction of the line is parallel to the positive 𝑧 -axis. As intersections with lines that are near-tangent to the surface are the most difficult to identify and resolve, we heuristically select a line with a direction equal to an average surface normal, which we compute for each surface as preprocessing. In cases where the symmetry of the surface results in an average normal with near-zero magnitude, we instead select the line randomly.

Wepresent the full algorithm to compute the GWN of an arbitrary surface in Algorithm 1. In the following subsections, we justify our method mathematically by providing an overview of Stokes' theorem, and then describe how its application to this problem naturally delineates each of the above cases. We then discuss our strategy for evaluating the GWN field at points which are coincident to the surface as well as our accelerated adaptive quadrature.

## 3.2 Reformulation with Stokes' Theorem

The connection between the 3D solid angle and an associated line integral formulation has been independently explored in several application contexts, including physical optics [Asvestas 1985], soft matter physics and electrostatics [Binysh and Alexander 2018], and perhaps most directly in differential geometry [Borodzik et al. 2017]. For example, there are many contexts in which the solution to specific line integrals over closed 3D contours can be more efficiently evaluated by means of computing the (often approximated) solid angle of some interior surface [Binysh and Alexander 2018]. Common among these works is either the selection or presupposition of a particular line 𝐿 which specifies the particular form of the boundary formulation, as is the case in the presented work. Indeed, the work of Asvestas [1985] presents a solution strategy which most closely mirrors our own, where signed intersections between the line 𝐿 and a planar surface are distinguished according to the location of the intersection, with the numerical result further adjusted through an analytically derived correction term.

Directly applying such methods to the present graphics context is restricted by the fact that the targeted geometry is more complex and cannot be a priori discretized (see Section 2.2). However, we have the advantage of being able to select the line 𝐿 which is most computationally advantageous, and further manipulate the boundary curves of the surface to ensure our algorithm's robustness. As such, we present our own derivation of these principles in differential geometry from the ground up, as doing so leads to our proposed computational treatment of general trimmed NURBS surfaces.

In its most general form, Stokes' theorem states that if a vector field 𝐹 ( 𝑥 ) is defined with continuous first derivatives on a region in R 3 containing the surface 𝑆 , then

<!-- formula-not-decoded -->

where 𝜕𝑆 is the total boundary of the surface, which may contain multiple disconnected curves.

To apply Stokes' theorem to the integral in Equation 4, we take

<!-- formula-not-decoded -->

Recall that we have made the assumption that the query point 𝑞 is located at the origin, and so this integrand is continuous on the surface whenever fi 𝑥 ≠ 0, i.e. the query point does not coincide with the surface (we treat this case explicitly in Section 3.3).

There are many distinct vector fields 𝐹 that satisfy Equation 8, and so one could evaluate the GWN for an arbitrary surface with the following 1D line integral,

<!-- formula-not-decoded -->

assuming that the theoretical conditions of Stokes' theorem are met for the surface 𝑆 .

This reformulation of the problem statement comes with a number of computational advantages. First, evaluating such an integral via numerical quadrature along the lower-dimensional space naturally improves the efficiency of the total method. This is a well-established advantage of Stokes' theorem, and is the basis of a method proposed by Gunderman et al. [2021] for evaluating arbitrary integrals on trimmed NURBS surfaces. In this problem context, we can evaluate the 'antiderivative' 𝐹 exactly for the relevant ∇× 𝐹 , further reducing the number of quadrature nodes in a scheme of fixed order 𝑛 from 𝑂 ( 𝑛 2 ) to 𝑂 ( 𝑛 ) .

Second, such a boundary reformulation fundamentally improves the stability of the ensuing quadrature scheme, as query points which are near to the surface but not near the boundary are not subject to the same near-singular behavior in the corresponding integrand. The resulting line integrals are additionally more amenable to a geometrically adaptive quadrature method, the computational burden of which is dramatically reduced by considering subdivision of 3D space curves instead of entire trimmed surfaces.

Fig. 5. Far- and near-field points are classified according to whether the line of discontinuities for antiderivative 𝐹 does not intersect (a), can be rotated to avoid intersecting (b), or unavoidably intersects (c) the surface.

<!-- image -->

However, not every surface can be treated directly by the formulation in Equation 9. To meet the conditions of Stokes' theorem, both the original vector field and its antiderivative must be continuous-not just on its domain of integration (the boundary of the surface) but also on the original domain of integration (the surface itself). Notably, the relevant vector field 𝐹 is discontinuous whenever 𝑥 2 + 𝑦 2 = 0 on the surface, i.e. whenever the surface intersects a line extending in both 𝑧 directions from the query point. In such cases, we cannot mathematically justify the direct use of Stokes' theorem, even if the resulting line integral is itself continuous.

We note that the specific antiderivative in Equation 9, characterized by a line of discontinuities on the 𝑧 -axis, was chosen arbitrarily. Indeed, one can analogously define a family of antiderivatives 𝐹 , each characterized by a different line of discontinuities.

In Figure 5, we see examples of different configurations of the surface and the line. As shown in Figure 5 (b), it is practical to view different choices for 𝐹 not as explicit vector fields, but as different rotations of the NURBS patch via its control points so that the desired line of discontinuities is aligned to the 𝑧 -axis. In this way, applying Stokes' theorem requires choosing a line which does not intersect the surface, and rotating the surface accordingly. In practice, one cannot always identify such a line for query points which are nearer to the surface, nor does one necessarily exist (see Figure 5 (c)).

The remainder of this subsection details our three cases.

3.2.1 Far-field : Direct Application of Stokes' Theorem. If an axisaligned bounding box of the shifted surface does not contain the origin (i.e. the query point is exterior to the bounding box of the original surface), then it must be the case that at least one coordinate axis does not intersect the surface, trivially providing a line 𝐿 to apply in our Stokes' theorem boundary formulation. Indeed, if the 𝑧 -axis does not intersect the bounding box, then we can directly apply the formulation in Equation 9 to evaluate the GWN. In the event that surface bounding box does not contain the origin, but does intersect the 𝑧 -axis, then we can instead consider a form of the integrand that corresponds to the 𝑥 - or 𝑦 -axis, one of which necessarily does not intersect the surface. In these cases, we use the corresponding analogs of antiderivative 𝐹 within Equation 9:

<!-- formula-not-decoded -->

Fig. 6. We illustrate a property of the GWN for two surfaces which share a boundary, viewed from different angles. In general, any two surfaces which share a boundary generate the same GWN field, modulo some integer which depends on the orientation of the query point relative to the surface.

<!-- image -->

and

<!-- formula-not-decoded -->

While our implementation first checks an axis-aligned bounding box to identify far-field scenarios, other convex shapes containing the shifted surface can also be used to guarantee a non-intersecting line 𝐿 , again followed by the appropriate rotation of the surface. For example, our implementation also checks a precomputed oriented bounding box for each NURBS surface, allowing more points to be classified as far-field. In any case, these are the most commonly evaluated forms of the integral, as an arbitrary query point will be considered far from all but a few surfaces in a typical CAD model.

The validity of this far-field formulation can be made more intuitive by making concrete a property of the GWN introduced in Section 2.3 - namely, that the GWN field of a surface can, for far away points, be defined entirely by its boundary. More specifically, any two surfaces 𝑆 and 𝑆 ′ which share a boundary 𝜕𝑆 = 𝜕𝑆 ′ generate an identical GWN field at all points outside a volume implied by 𝑆 ∪ 𝑆 ′ . Furthermore, if one were to continuously deform 𝑆 into 𝑆 ′ , then the value of the GWN at a fixed query point 𝑞 inside this volume would remain constant up until the surface passes through 𝑞 , at which point the GWN value would jump by one. We depict this scenario explicitly in Figure 6, which further illustrates the specific form of Equation 6. Although the GWN field is determined almost entirely by boundary data (and completely so for far-away points), we must still account for the influence of the internal surface; as we will show, it affects the GWN value only by a constant.

3.2.2 Near-field : Analytically Adjusted Stokes' Theorem. If the surface bounding box does contain the origin (i.e. the query point is inside the surface bounding box), we can no longer guarantee that there exists a line 𝐿 which does not intersect the surface. Instead, we must explicitly test for signed intersection between some oriented line 𝐿 and the surface. We consider cases to be 'near-field' when all intersections between 𝐿 and the surface occur away from the surface edges and other degeneracies. When these conditions are not met, the query point is handled as an 'edge-case'.

Naturally, distinguishing the near-field and edge cases requires first identifying the location of all intersections between the surface and the line in the parameter space of the surface. As noted in Section 2, there are many numerical ray casting algorithms one could apply to find signed intersections between this line and the surface, and in principle, any can be used for the purposes of evaluating the GWN. We describe our own geometric subdivision approach in Section 5.1.1, which in contrast to many algebraic alternatives, takes advantage of the broader GWN algorithm's high tolerance for imprecision in the intersection routine.

Because any intersection between 𝐿 and the surface violates the conditions of Stokes' theorem, direct application of the boundary formulation is theoretically unfounded and, in general, evaluates to an incorrect GWN. Our proposed correction term in Equation 6 permits evaluation of the GWN using the same boundary integral formulation as Equation 9, provided that for each intersection (if any), the value 0 . 5 is added or subtracted according to the orientation of the surface and line at the point of intersection to compensate for the jump discontinuity present in the GWN field along the surface.

This is illustrated in Figure 7 (bottom), showing that the difference between the ground truth GWN and the specific boundary formulation in Equation 9 is always, perhaps unexpectedly, half-integer valued. As a result, the true GWN can always be recovered from the boundary formulation by identifying the proper half-integer value.

In some sense, the existence of an analytic fix that depends only on the orientation of the query point relative to the surface is a natural consequence of the double-layer potential which, when integrated along the surface, defines the GWN scalar field [Bang et al. 2023; Barill et al. 2018]. The use of this specific correction term is made more intuitive by means of analogy in two spatial dimensions (with a direct proof provided in Appendix A). As summarized in Section 2.3 and demonstrated in Figure 7, the GWN for an arbitrary curve in 2D (left) can always be computed as the difference between an integer winding number for the curve closed by a straight line (right) and a generalized winding number defined by that straight line closure (middle). Crucially, the GWN of the closure is uniquely defined by the two boundary points of the original curve. From the lens of our 3D GWN framework, the GWN of the closure can be thought of as a 'boundary term' which must be 'corrected' by the integer winding number of the closed shape.

To make the comparison explicit, we see that in both 2D and 3D, the value of the GWN is the sum of a fractional component which does not depend on any interior geometry, and an integer component which does. Although the fractional component in 2D reduces to a subtended angle, while the 3D equivalent is not the GWN of any particular shape, both integer components can be thought of as determined by the result of a line intersection test. Most importantly, points which are known to be far from the shape do not require any correction at all. Returning our focus to 3D, we emphasize that although our line-surface intersection test does come at additional cost, it is necessary only for the surfaces nearest to the query point.

Fig. 7. Weillustrate a property of the GWN in both 2D (top) and 3D (bottom). The difference between the GWN for a curved surface (left) and that of a particular boundary term (middle) is always a constant value. In 2D, when this boundary term is the GWN of the curve's linear closure, the difference is integer valued. In 3D, when this boundary term is the integral component of Equation 6, the difference is always half -integer valued. This suggests a method for GWN evaluation by evaluating the boundary term, and adjusting it by identifying the proper constant.

<!-- image -->

3.2.3 Edge Cases : Disk Extraction via Trimming Curves. We now consider the remaining edge cases for which the query point is inside the surface bounding box, but the correction term in Equation 6 cannot be applied. As shown in Figure 8, this occurs when the line of discontinuities intersects the surface at a boundary, at a cusp, or is tangent to the surface. We treat all edge cases by either finding a new line (in the case of tangencies) or by removing from the surface a disk of radius 𝑟 in parameter space around the point of intersection. This guarantees that the line does not intersect the surface at the otherwise problematic intersection point, and permits evaluation via Equation 6. The extracted disk is then fed back into the original algorithm for reprocessing. Since the produced disk has a much smaller bounding box than the original surface, the computational cost of the reprocessing is significantly reduced.

While the choice of disk radius 𝑟 is mathematically arbitrary, it can have important consequences on computational performance and the selection of other numerical tolerances. Through observation and numerical experiments (presented in full in Section 5.1.2), we have found that fixing the radius 𝑟 to be 1% of the bounding box diagonal offers a reasonable tradeoff between these considerations.

Furthermore, the proposed strategy for handling edge cases is such that it could be used to accurately compute the GWN of any query point (at increased computational cost). Because most edge cases can be identified by a zero normal vector at the point of intersection, we can conservatively treat any intersection with a near -zero normal vector as an edge case without significantly impacting the aggregate performance of the method.

Fig. 8. Edge-Case Evaluation: For query points within a surface bounding box, we consider a line containing the point. If the line intersects the surface near a boundary curve (a) or at a point for which a normal cannot be defined such as a cusp (b), we remove the intersection by trimming the surface along a small disk in parameter space and feed the remaining disk back into our algorithm for reprocessing. If the intersection occurs at a tangent point between the line and the surface (c), we apply a random rotation to the surface that does not change the GWN evaluation, but is more likely to result in a case that is easier to handle.

<!-- image -->

Similarly, we treat any intersection within a distance 𝑟 of the surface boundary as an edge case. Because this should also include intersections that occur outside the trimmed surface, during preprocessing, we linearly extend the parameter space of each untrimmed NURBS patch by 𝑟 [Wolters 2000]. Accounting for 'near-misses' in this way also improves the robustness of the GWN algorithm to the precision of the line-surface intersection routine.

The only edge case which is not well-handled by this strategy is a line that is tangent (or near-tangent) to the surface, as the surface remaining after the disk is removed may still intersect the line. In these cases, we instead rotate the surface (i.e. consider a different line) and feed the rotated surface into the original algorithm (see Figure 8 (c)). While this is the most computationally expensive type of edge case, it is also the least common due to our choice of the cast ray to be an approximate surface normal (See Section 5.1.1).

## 3.3 GWNfor Points Coincident with the Surface

Strictly speaking, the generalized winding number of query points coincident with the surface is undefined, as the fundamental mathematical definition of the GWN (Equation 4) is discontinuous. This is a problem unique to curved shapes, as the GWN of a point coincident with a line or triangle is exactly equal to zero. However, the GWN algorithm should still return reasonable values for points coincident to curved shapes (alongside the relevant error flag) to make the method more amenable to downstream applications.

In all cases, coincident or otherwise, it is important that the GWN algorithm produce consistent results for nearby points. This prohibits the common solution of perturbing the query points slightly to avoid coincidence, as this may move truly interior or exterior points across a boundary.

Analogous to the 2D case explored in Spainhour et al. [2024], we define the winding number of coincident points to be the average of the two values of the GWN computed across the jump discontinuity of the scalar field. Fortunately, we know by the results of Section 3.2.2 that in most cases, the correction term contributes a value of 0 . 5 if the query point is on one side of the surface, and -0 . 5 if it is on the other. This means that if the query point is on the surface, these two values effectively cancel out, and no correction term needs to be added to the boundary formulation for the intersection, as shown in Equation 6.

The only unaddressed case is when the query point is coincident with a boundary curve of the surface, i.e. a coincident edge-case. This case is uniquely difficult to solve via our Stokes' theorem reformulation, as the evaluated antiderivative is also non-trivially discontinuous along its domain. We handle this in much the same way as the other edge-cases in Section 3.2.3, where we remove a disk from the surface around the query point. The GWN calculation for the larger subdivided surface is performed as normal, but we consider the GWN of the removed disk to be zero, an assumption that only holds because the query point is coincident to the disk. To strengthen this assumption, we remove a much smaller disk than in the general case. Indeed, a similar approach is used by Liu et al. [2025] in 2D to remove a segment of the curve and treat its contribution to the GWN as zero.

## 3.4 Accelerated Adaptive Quadrature

Although the 1D line integral in Equation 6 is more numerically stable than the 2D surface integral equivalent in Equation 4, its robust evaluation requires a numerical integration scheme that is guaranteed to be precise for the near-singular integrand.

ALGORITHM 1: TrimmedSurfaceGWN Evaluate the generalized winding number for a trimmed NURBS surface.

```
Input: 𝑆 : Trimmed NURBS surface 𝑞 : Query point 𝜖 𝑞 : Tolerance for adaptive quadrature 𝜖 𝑙𝑠 : Tolerance for surface-line intersection Output: 𝑤 𝑆 : The GWN evaluated at 𝑞 𝑆 ← 𝑆 -𝑞 // Translate patch so q is at the origin 𝑤 𝑆 = 0 // Initialize the GWN 𝑟 ← 0 . 01 ∗ UVBoundingBoxDiagonal ( 𝑆 ) 𝑆 ← ExtendUntrimmedPatch ( 𝑆, 𝑟 ) // Per [Wolters 2000] if 𝑞 ∉ BoundingBox ( 𝑆 ) then /* Rotate patch to not intersect 𝑧 -axis */ 𝐿 ← Non-intersecting Line 𝑆 ← RotateToZ ( 𝑆, 𝐿 ) else /* Compute 𝑧 -axis intersections with 𝑆 (Alg. 2) */ 𝑧,𝑢, 𝑣 ← LinePatchIntersections ( 𝑆, z-axis , 𝜖 𝑙𝑠 ) foreach intersection point { ( 𝑢 0 , 𝑣 0 ) , 𝑧 0 } do fi 𝑛 ← 𝑆 𝑢 ( 𝑢 0 , 𝑣 0 ) × 𝑆 𝑣 ( 𝑢 0 , 𝑣 0 ) if 𝑆 ( 𝑢 0 , 𝑣 0 ) is an interior point of 𝑆 with normal fi 𝑛 ≠ 0 then /* Add constant according to orientation */ if fi 𝑛 𝑧 𝑧 0 > 0 then 𝑤 𝑆 = 𝑤 𝑆 -0 . 5 else if fi 𝑛 𝑧 𝑧 0 < 0 then 𝑤 𝑆 = 𝑤 𝑆 + 0 . 5 else if 𝑆 is tangent to the 𝑧 -axis at 𝑆 ( 𝑢 0 , 𝑣 0 ) then /* Rotate the surface and try again */ return TrimmedSurfaceGWN ( RandomRotation ( 𝑆 ) , 0 ) else /* Extract a parameter space disk S_disk, and update the trimming curves of S */ 𝑆, 𝑆 disk ← ExtractParameterDisk ( 𝑆, 𝑟 ) 𝑤 𝑆 = 𝑤 𝑆 + TrimmedSurfaceGWN ( 𝑆 disk , 0 ) end end foreach 1D Curve in BoundaryCurves ( 𝑆 ) do 𝑤 𝑆 = 𝑤 𝑆 + EvaluateLineIntegral ( 𝐶,𝜖 𝑞 ) // Alg. 5 end return 𝑤 𝑆
```

In our implementation, we utilize a recursive subdivision approach, applying a fixed-order Gauss-Legendre quadrature rule at each level of recursion. We stop the recursion when the difference between the evaluated integral on the entire curve and the sum of the integrals on the two halves is less than a user-defined quadrature tolerance 𝜖 𝑞 . Thus, in most cases, 𝜖 𝑞 determines how many digits of precision are achieved in the computed GWN field, although it is possible for more lenient values of 𝜖 𝑞 that the two terms will agree at an incorrect value. Using a Gaussian rule at each recursion level is particularly useful as it is capable of maintaining high precision even without specific structure in the integrand. We specifically chose this recursive subdivision strategy because it effectively concentrates quadrature nodes near unstable regions of the integrand, i.e.

when the query point is close to the 3D space curve which defines the domain of integration. Elsewhere, where the integrand is stable, we maintain the typically high precision of Gaussian quadrature without additional computational expense.

To further improve computational performance, we cache the quadrature nodes and surface tangent vectors for each trimming curve at each level of recursion, and reuse these values across multiple queries. Such a memoization strategy also compensates for the inability of Gauss-Legendre quadrature nodes to be reused across refinement levels (as would be the case for a nested quadrature rule such as Gauss-Kronrod), allowing the method to fully utilize the precision of the original rule. On the other hand, this strategy can only be used if the trimming curves of a patch remain unchanged throughout the evaluation of the GWN at a single query point. In particular, since our edge-case resolution involves modifying trimming curves within the extraction disk of radius 𝑟 , our memoization strategy cannot be applied to the modified curves in these cases. We present the full algorithm in Algorithm 5 and explore its performance more fully in Section 5.1.2, particularly as the parameter 𝜖 𝑞 relates to disk radius 𝑟 .

## 4 Numerical Experiments and Results

## 4.1 Numerical Integration of Boundary Formulation

We first justify our choice of a boundary formulation in our solution method by comparing our 1D numerical integration scheme that solves the boundary form in Equation 6 to a 2D numerical integration scheme that solves the surface integral of Equation 4. We perform this comparison on one of six biquartic patches representing a sphere [Cobb 1988] where we evaluate the GWN field along two slices using a 20-point Gaussian quadrature rule. The derived 1D method involves applying this 20-point rule to each boundary curve for a total of 4 · 20 evaluations of the NURBS surface, while the 2D scheme involves a tensor product of 20-point rules for a total of 20 2 evaluations of the NURBS surface.

We show the results of this comparison in the 'Non-Adaptive Quadrature' row of Figure 9, where errors in the 1D scheme are only present near the surface boundary, while errors in the 2D scheme are present for all points near the surface. The accuracy of this 2D scheme, and in some sense even its applicability, would be further compromised by the presence of trimming curves, which are far more naturally handled in our boundary scheme.

The error in both methods can be mitigated through the use of adaptive quadrature, describing a 'Boundary-adaptive' and 'Surfaceadaptive' scheme respectively. The performance of each method is shown in the 'Adaptive Quadrature' row of Figure 9, showing that the surface-adaptive scheme would require thousands or even tens of thousands of times as many surface evaluations to achieve the same level of accuracy as our boundary-adaptive strategy, a cost which is further exacerbated when subdividing trimmed surfaces.

In [Martens and Bessmeltsev 2025], this surface-adaptive scheme is referred to simply as 'adaptive quadrature', which largely explains their characterization that quadrature is inapplicable for complex CAD surfaces. In contrast, as can be seen in Figure 10, a boundaryadaptive strategy outperforms the reported metrics of that work on similar shapes, and is capable of doing so even without taking advantage of per-query parallelism, the planar characteristics of the input boundary curves, or any structure in the input set of query points. Figure 10 also demonstrates on practical examples how algorithm performance is influenced by the position of the query point. As expected, 'near' points close to surface interiors have negligible overhead compared to 'far' points, while only those close to a surface boundary are more expensive since they must often be treated as edge-cases.

Fig. 9. Comparing surface-based 2D tensor product Gaussian quadrature schemes (left) to boundary-based 1D Gaussian quadrature schemes (right). For each, we consider a fixed ('non-adaptive') 39 th -order rule which requires 20 2 evaluation points for the surface integral and 4 · 20 evaluation points for the boundary integral (top) and an adaptive quadrature scheme which uses recursive bisection to ensure an absolute error bound of 10 -6 (bottom). The boundary formulation produces less error when evaluated as a non-adaptive scheme, and reduces evaluation cost as an adaptive scheme. Evaluation counts and absolute errors are presented in log-scale.

<!-- image -->

## 4.2 Accuracy Evaluation

We now consider the full implementation of our algorithm, which solves Equation 6 at arbitrary points up to arbitrary accuracy with a more sophisticated, error-controlled adaptive quadrature scheme. In Figure 11, we demonstrate the versatility of our algorithm by evaluating the GWN on several models adapted from the ABC dataset [Koch et al. 2019], a collection of CAD models of varying quality used to benchmark algorithms in computer graphics and machine learning. The specific models used in this work can be found in [Axom\_data 2025]. In order to exaggerate the effects of the GWN field on open 3D shapes, we have intentionally made each of these shapes nonwatertight through the deletion of several surfaces. We present qualitative results on additional watertight and non-watertight CAD models in the supplemental materials to this manuscript.

Fig. 10. GWNfields and evaluation times along an 800 × 800 slice of grid points. The 28-patch 'Vase' surface of revolution (top) is adapted from an example in Martens and Bessmeltsev [2025], which reports a threaded processing time of 484 seconds, while our approach takes 97.7 seconds serially (approximately 0.15ms per query point). Our approach takes 829.9 seconds (approximately 1.29ms per query point) on the more complicated 192-patch filleted 'Box-Sphere' shape (bottom).

<!-- image -->

In general, a GWN-derived containment query is most useful in contexts where the objective is to faithfully represent input geometry, as opposed to when there is an unknown, ground-truth shape for which the provided boundaries are a subset. The GWN field itself is uniquely determined by the set of input surfaces, and so it cannot correct the geometric errors in a surface or its trimming curves, which are themselves assumed to be inextricably defined within the input. However, the subsequent containment query remains robust to such errors, as the containment result is not meaningfully changed outside a small region around the error.

This is demonstrated in Figure 11, where, for each example, the GWNfield neatly partitions the domain into exterior and interior regions far from open boundaries, while providing reasonable containment decisions elsewhere. We can see this directly in the rightmost column of Figure 11, which shows how rounding the scalar GWNfield accounts for small errors in the surface by defining an implicit boundary through the 0 . 5-isosurface.

To consider the accuracy of our method more closely, we introduce possible adaptations of existing GWN techniques on discretized surfaces to trimmed NURBS geometry. First, we derive from the work of Jacobson et al. [2013] a strategy which triangulates the (possibly non-watertight) surface with a given level of linear deflection (as exposed through Open Cascade's BRepMesh\_IncrementalMesh triangulator [Open Cascade SAS 2011]) and evaluates the GWN field for the triangulated result. Second, we derive from the work of Barill et al. [2018] a strategy which takes a given number of samples from the surface of the shape model, along with the unit normal at these points, and evaluates the GWN field for the resulting point cloud. For each method, we conservatively define a containment classification according to a 0.5-isosurface, where any point whose GWNvalue is greater than 0.5 is classified as interior, and any point whose GWN is less than 0.5 is classified as exterior.

Fig. 11. We demonstrate our calculation of the GWN field on non-watertight CAD models of varying size and complexity. Each field is viewed through two 2D slices through the shape, the second of which is also mapped to a containment decision through a non-zero rule.

<!-- image -->

In Figure 12 we compare our proposed algorithm to the above adaptations of conventional alternatives on subsets of a watertight 'Gear' shape, with each view considered on a structured grid of 500 × 500 sample points. In Figure 12 (top), we consider the shape when it is coarsely, yet still meaningfully, discretized according to each method to see the primary characteristics of the resulting GWN

field. For the triangulation method, we see, as expected, that curved surfaces are made linear, resulting in unpredictable errors near the surface boundary while still producing integer-valued winding numbers for this watertight model.

In contrast, while the 0.5-isosurface of the scalar field provided by the point cloud method generally captures the curved features, the GWN field in its totality diverges significantly from the integervalued ground truth. Although the dipoles correctly place nearby query points on the correct side of the curved surface, their large magnitude results in scalar GWN values outside [-0 . 5 , 1 . 5 ] . This would lead to further misclassifications if containment were determined with the conventional strategy of rounding, followed by an even-odd or non-zero rule, which might be necessary on more complex shapes with nested surfaces.

<!-- image -->

Generalized WindingNumber

Fig. 12. Comparing our direct GWN method to discretized alternatives derived from triangulations [Jacobson et al. 2013] and point clouds sampled from the surface [Barill et al. 2018] on a watertight 'Gear' CAD model. To demonstrate the type of error that occurs with each method, we consider each with a relatively low level of discretization (top). At increasingly narrow 500 × 500 slices of the shape, we directly mark in red the points which are misclassified according to a 0.5-surface (bottom). Increasing the resolution of the discretization (typically) decreases the number of misclassified points, but at increasing computational cost. In contrast, our proposed method accurately classifies each of these points directly on the CAD geometry's trimmed surfaces.

To see this error more quantitatively, we record in Figure 12 (bottom) the total number of misclassifications on increasingly narrow views of the shape using the triangulation approach (left) and the point cloud approach (right), with each misclassified point in each 500 × 500 grid marked in red. While the number of misclassifications for each discretization strategy (generally) decreases as the resolution of the discretization increases, there is no level at which no misclassifications occur. Further, although the linear deflection parameter controls how far the triangulation can differ from the original shape, in effect ensuring that no points within a fixed absolute distance of the surface will be misclassified, it can be difficult to know a priori what level of discretization is necessary for correct classification at user-specified query points. This is well-illustrated in the narrowest view, in which the increased deflection parameter from 10 -3 mto 10 -5 mdoes not necessarily increase the accuracy of the method, as all points which are misclassified are by coincidence still closer than 10 -5 mto the surface. This relationship is further obfuscated in the case of a point cloud representation, where it is similarly difficult to gauge the appropriate sampling density for a point cloud that will lead to accurate classifications. In either case, this can lead to unnecessary computational expense in the form of globally dense triangulations or sampling procedures. In contrast, operating directly on the trimmed surface allows our method to perform adaptively for each query to the requested level of accuracy and correctly classify the containment of all query points.

While the approximation accuracy with any of these discretizations may be tolerated within a particular application, the computational cost of generating and storing increasingly refined discretizations grows exponentially. For example, the Gear shape has many flat regions which are trivial to triangulate, but still has 54k triangles at deflection 0 . 1 m, increasing to 2 million at deflection 0 . 001 m, and nearly 10 million at deflection 10 -5 m. The cost of constructing and accessing the spatial index through which these shapes are efficiently queried is similarly increasingly burdensome. Indeed, applying the method of Barill et al. [2018] in Figure 12 (bottom) to a cloud of 10 7 points on our computing platform with 32 GB RAM using the libigl implementation [Jacobson et al. 2018] required reducing the expansion order for clusters of points from 2 to 1, as it was otherwise not possible to store all 27 terms of the 2nd order expansion at each level in the spatial index. In contrast, our method is able to accurately classify the entire collection of points without the need for an expensive intermediate discretization, using only information provided directly from the 11MB STEP file.

## 4.3 Performance Evaluation

We now demonstrate the performance of Algorithm 1. The following numerical experiments were conducted on an AMD Ryzen 7 5700 CPU with 32GB of RAM. These results are computed serially, although we note that the evaluation of the GWN field at many points is a highly parallelizable task.

Although a triangulated B-Rep is composed of significantly more primitive components than the original collection of surfaces, methods that operate on CAD geometry are typically more expensive than equivalent operations on STL data types. However, our use of cached per-curve quadrature nodes allows us to evaluate the GWN field at a practical rate, even without the use of more sophisticated hierarchical data structures, the implementation of which we largely consider to be future work.

In Table 1, we show the average time needed to evaluate each query point on a uniform 50 × 50 × 50 grid of points located in a bounding box of each shape, as well as a per-query average. We also report the average time needed to evaluate the GWN for each surface in the shape (overall and by case), alongside the breakdown of all point-surface GWN calculations by case. This table highlights several key features of the performance of our algorithm. Most notably, although the total time needed to evaluate the GWN field in principle scales linearly with the number of trimming curves, there are many other factors which impact runtime. For example, the overall distribution between point-surface case configurations significantly affects the overall runtime. Naturally, far-field cases are the easiest to resolve, and so shapes for which most query points can be classified as far-field for most surfaces take the least amount of time to process. This is well-demonstrated by the 'Bobbin' shape taking more time on average than the 'Spring' case, despite the latter having many more surfaces and trimming curves.

Other differences come from more general characteristics of the component NURBS patches, which are best illustrated through comparison of the 'Trailer' and 'Pipe' examples. Although the Trailer shape has more surfaces, the NURBS patches themselves are predominantly flat with simple trimming curves. In contrast, the Pipe shape is more complicated, with many of its component NURBS patches being large spheres and cylinders which are trimmed and filleted. Thus, while the times to process the individual far- and near-field cases are comparable, resolving the edge-cases of the Pipe example is more challenging, and therefore more expensive. We present similar timing tests on other watertight CAD shapes from the ABC dataset in the supplemental materials to this manuscript.

As a further point of comparison, we can consider a version of the Spring shape defined by only two NURBS surfaces. Since the two patches share a bounding box, there are few query points that can be classified as far-field. Of those that remain, the vast majority must be classified as edge cases which cannot benefit from our memoization strategy, an added cost which is further burdened by the resulting boundary curves being long and therefore poorly resolved by the quadrature at the initial refinement levels. On the other hand, the original shape permits many more points to be classified and efficiently resolved as far-field. It is worth noting that while reducing the size of each patch shifts the distribution of cases and processing times, our method recovers an identical GWN field for the two shapes due to the exact subdivision of NURBS surfaces.

## 5 Discussion

## 5.1 Additional Algorithms

In this section, we highlight the computational components and tolerance parameters supporting our implementation and numerical experiments. Although there are some specifications within these methods unique to our application, we expect that alternate approaches which meet these requirements would be suitable.

5.1.1 Line-Surface Intersections. Accurately evaluating the nearfield sum in Equation 6 requires a robust intersection test between an arbitrary line and an input trimmed NURBS surface.

We use a geometric approach to identify the isoparametric coordinates of all signed intersections between a line and an untrimmed NURBS patch in the parameter space of the patch. This involves recursively subdividing the patch geometry (either through Bézier extraction of NURBS patches or bisection of individual Bézier patches) and clipping away portions of the surface where intersections provably cannot occur. By maintaining a record of each subdivision patch's placement within the parameter space of the original surface, this method converges linearly to the set of parametric points ( 𝑢, 𝑣 ) at which the line intersects the untrimmed NURBS patch.

As an early stopping criterion for recursion, we treat patches which are 'approximately bilinear' (i.e. have control points which are nearly coincident with the surface along a uniform grid) directly using the GARP algorithm (Geometric Approach to Ray/bilinear Patch) [Reshetov 2019], which describes a closed-form solution for the intersection of a line with a bilinear patch. This allows the intersection coordinates within a sub-patch to be accurately mapped back to the parameter space of the original surface.

| Shape                          | Bobbin   | Trailer   | Pipe   | Spring   | Spring, Two-Patch   |
|--------------------------------|----------|-----------|--------|----------|---------------------|
| Number of NURBS Patches        | 18       | 228       | 133    | 64       | 2                   |
| Number of Trimming Curves      | 76       | 1092      | 563    | 256      | 8                   |
| % Far-field Cases              | 92.56 %  | 99.8 %    | 99.02% | 99.57 %  | 18.26 %             |
| % Near-field Cases             | 7.00 %   | 0.116 %   | 0.81 % | 0.42 %   | 76.15 %             |
| % Edge Cases                   | 0.42 %   | 0.014 %   | 0.15 % | 0.006 %  | 5.59 %              |
| Avg. Time per Query (ms)       | 1.11     | 1.18      | 21.1   | 0.332    | 11.07               |
| Avg. Far-field Case Time (ms)  | 0.0173   | 0.0046    | 0.0313 | 0.0043   | 0.0718              |
| Avg. Near-field Case Time (ms) | 0.0805   | 0.0585    | 0.916  | 0.1301   | 0.336               |
| Avg. Edge Case Time (ms)       | 9.38     | 3.97      | 76.7   | 5.55     | 94.3                |

Table 1. We evaluate the GWN field at a uniform grid of query points for each shape, and report the average time spent to resolve each query. We also show the number of query points that are treated by each of the three cases and the average time needed to resolve each of these cases.

<!-- image -->

When the parametric coordinates of all intersections with the untrimmed patch are recorded, we supply them to a trim test based on the 2D GWN [Spainhour et al. 2024] to filter out intersection points which occur outside the parameter space defined by the trimming curves (see Algorithms 2 and 3).

The use of this algorithm exposes the 'line-surface' tolerance parameter 𝜖 𝑙𝑠 , which determines when a sub-patch is considered to be approximately bilinear (see Algorithm 4). Naturally, a stricter 𝜖 𝑙𝑠 means more linear subdivisions are needed to capture the intersections, while a more relaxed 𝜖 𝑙𝑠 means the ( 𝑢, 𝑣 ) coordinates recorded by GARP may diverge more from the true points of intersection. This may result in the query point being placed on the wrong side of the surface, and in turn, an incorrect correction term being applied in Equation 6. However, we emphasize that this dynamic bilinear approximation is used only to identify the parameter space coordinates of the intersection point, and is subsequently discarded. All geometric quantities derived from these coordinates, such as the 3D intersection point and surface normal, are computed with respect to the original surface geometry. In particular, we always compute the GWN with respect to the true surface boundary without further reference to this or any other approximation.

Within the context of the broader GWN algorithm, we further improve the robustness of the line-surface intersection routine by selecting a line for each configuration of surface and query point that is orthogonal or nearly-orthogonal to the surface at the point of intersection. Doing so avoids some of the difficulties of identifying tangent or near-tangent points of intersection. Directly identifying such a line would require additional processing, such as a closest point query, the cost of which may exceed that of the GWN calculation. In this work, we heuristically select the line in the direction of an average surface normal [Ueda 1996], which we have found to be very effective for the majority of B-Reps.

To explore the practical tradeoffs of the value for 𝜖 𝑙𝑠 , we evaluate the GWN with respect to a challenging 'teardrop' shape in Figure 13. The bottom of this shape is spherical, but is composed of

Fig. 13. Sensitivity study for the line-surface intersection threshold 𝜖 𝑙𝑠 , evaluating the GWN on query points randomly sampled from an axisaligned bounding box until 5 misclassifications are recorded. Beyond a certain threshold, the line-surface intersection routine is accurate enough to ensure there are no misclassifications in over 10 7 sampled points, while the average run-time of the algorithm only slightly increases.

<!-- image -->

4 degenerate bicubic patches rather than the biquartic patches used in the example of Figure 9. The top shape is generated by revolving a polynomial cubic Bézier curve around the 𝑧 -axis, providing inflection points which pose additional challenge for the line-surface intersection test. Around this shape, we randomly sample points using a uniform distribution within an axis-aligned bounding box and record if rounding the GWN at that point results in a containment misclassification according to an exact analytic containment formula. In this example, we vary 𝜖 𝑙𝑠 and use a fixed adaptive quadrature tolerance ( 𝜖 𝑞 = 10 -6 ) and extraction disk radius (1% of the parameter space bounding box diagonal).

Because the probability that non-adversarially selected points are misclassified is quite low, particularly for strict values of 𝜖 𝑙𝑠 , we continuously sample from this distribution until 5 points are misclassified. We also evaluate the average runtime across a separate set of 10 5 sample points, which is sufficiently many to effectively utilize cached quadrature nodes, allowing the computational cost to be amortized over the sample set.

As expected, the number of misclassified points rapidly decreases as the tolerance is made more strict, to a point where the likelihood of failure is less than 0 . 00001%. On the other hand, the associated cost of the algorithm increases as more surface subdivisions are needed to evaluate intersection points, although this burden is marginal relative to the total cost of the algorithm. As a result of this experiment, and our other numerical examples, we consider a choice of 𝜖 𝑙𝑠 = 10 -6 to offer a reasonable tradeoff between computational efficiency and accuracy.

5.1.2 Adaptive Quadrature and Disk Radius. In our current implementation, memoized quadrature nodes cannot be reused if a patch's trimming curves change during evaluation of the GWN. As a result, there is a strong correlation between overall computational performance and the choice of the disk radius used to identify and resolve near-field and edge-case configurations. On the one hand, a smaller disk radius reduces the number of edge-cases (thereby improving reuse of cached quadrature nodes), since the disk is less likely to intersect existing trimming curves. On the other hand, a smaller disk radius allows query points to be closer to the singularity in the boundary curve integrals, necessitating a stricter tolerance for 𝜖 𝑞 to ensure consistent accuracy.

To demonstrate how both the disk radius (measured as a percent of each patch's parameter space bounding box diagonal) and 𝜖 𝑞 affects the accuracy and computational efficiency of the algorithm, werepeat the numerical experiment for 𝜖 𝑙𝑠 in Section 5.1.1 and report the results in Figure 14, varying 𝜖 𝑞 during the evaluation of the GWN for disks of various sizes. In these examples, we set 𝜖 𝑙𝑠 = 10 -6 , and further note that 𝜖 𝑙𝑠 and the disk radius have an independent effect on the method's accuracy. We see that as the radius decreases, a more strict tolerance for 𝜖 𝑞 is needed to ensure there are exceptionally few misclassifications. Furthermore, we see that across disk radii, a stricter tolerance involves a greater computational cost, but this cost is marginal relative to the overall effect of disk radius. It is for this reason that we use for our algorithm a disk radius that is 1% of the parameter space bounding box diagonal, as it is small enough to allow full utilization of the memoization scheme, while still allowing for a somewhat lenient tolerance of 𝜖 𝑞 = 10 -6 .

5.1.3 Disk Extraction. Handling edge cases efficiently requires a specific type of surface subdivision in which the surface is split according to a disk defined in parameter space around a given point (see Figure 15). We accomplish this through the insertion of extra trimming curves around the given edge-case point. To the original surface, we add an extra trimming loop around such points in parameter space, and remove all existing trimming curves within the loop. To a copy of the original surface, we remove all trimming curves exterior to the loop in parameter space. In each case, this processing requires computing the set of intersections between the initial set of trimming curves and a circle in the parameter space of the patch. However, this is greatly simplified by the use of 2D generalized winding numbers in our trim tests, as these trimming curves need not be explicitly connected to form closed loops.

## 5.2 Robustness to Defective Trimming Curves

In the context of containment queries for messy CAD geometry, there are two relevant, yet distinct conceptions of robustness.

First is robustness to errors between the (valid) trimmed NURBS surfaces in the model, which is necessary when numerical gaps and overlaps between adjacent patches make conventional containment techniques like ray casting inaccurate. By the linearity of the GWN, our algorithm is evaluated independently for each patch, ensuring our containment decision is reasonable for this kind of messy input. We achieve this form of robustness through accurate and precise calculation of the GWN field at arbitrary 3D points, one surface at a time. This property of our algorithm is exhibited on watertight and non-watertight examples throughout this paper (e.g. Figures 1, 10, 11 and 12) as well as in the supplement.

Second is robustness to errors within the representation of trimmed NURBS surfaces, which is necessary when numerical gaps and overlaps between adjacent trimming curves make the surface have an ill-defined boundary. Such errors interfere with many primitive geometric operations, e.g. tessellation [Xiong et al. 2023] and calculation of surface areas [Gunderman et al. 2021] or surface normals [Ueda 1996]. Other boundary-focused methods for the GWN specifically rely on surface boundaries forming closed loops, such the calculation of the GWN via the 'One-Shot' approach [Martens and Bessmeltsev 2025] or the adaptations of [Jacobson et al. 2013] discussed in Section 4.2. More generally, trimming curve defects are challenging to resolve in this context because they cannot be directly addressed by the GWN, as the scalar field itself becomes mathematically ill-defined.

In Algorithm 1 we make the standard assumption that the trimming curves for a given patch form simple, closed loops in parameter space, where each such loop corresponds to a single component of the surface boundary. While we consider a rigorous definition and calculation of the GWN field for malformed surfaces to be a productive path for future research, we have nevertheless structured our implementation so that it can still meaningfully process surfaces which do not meet this assumption. Perhaps most importantly, our trim tests, which determine if line-surface intersections lie in the visible region of a patch, are handled through the 2D GWN [Spainhour et al. 2024], making our visibility queries indifferent to trimming curve watertightness. Similarly, our strategy for resolving edge-cases through disk subdivision assumes no structure in the collection of trimming curves, further supported by our use of the 2D GWN. Finally, our adaptive quadrature algorithm operates independently on each component of the boundary without reference to their overall arrangement.

Figure 16 demonstrates how these decisions impact our calculation of the GWN field in the presence of a surface with poorly defined trimming curves, and in turn how this GWN field impacts the subsequent containment decision. To the original shape in Figure 16 (top left), we perturb the trimming curves of the front face while keeping adjacent curves connected. The resulting set of trimming curves, shown in Figure 16 (bottom left), is still closed and connected, but the perturbation has introduced self-intersections that form small clockwise loops. The boundary formulation itself is indifferent to trimming curve orientation, and so for query points away from these self-intersections, Algorithm 1 continues to produce a GWN that degrades smoothly around the mismatch with the new boundary. While this type of trimming curve geometry poses a challenge for our correction term (Equation 6) and edge-case strategy, resulting in a noisier GWN field, the containment decisions remain consistent with those of the original model.

Fig. 14. Sensitivity study for extracted disk radius and quadrature tolerance parameter 𝜖 𝑞 , evaluating the GWN on query points randomly sampled from an axis-aligned bounding box until 5 misclassifications are recorded. Beyond a certain threshold, regardless of radius, the adaptive quadrature routine is accurate enough to ensure there are no misclassifications in over 10 7 sampled points, although there is a tradeoff between radius size and quadrature costs.

<!-- image -->

Fig. 15. Disk extraction applied to the edge case in Figure 8 (a).

<!-- image -->

Next, we consider collections of non-watertight trimming curves. For Figure 16 (top right) we delete trimming curves at each corner in parameter space, and for Figure 16 (bottom right) we delete a middle portion from each curve. In the presence of these defective trimming curves, we observe that the computed GWN field does not degrade smoothly in the same way that it does around open, yet welldefined edges. For example, applying our boundary formulation to the same malformed surface returns slightly different GWN values at the same query point depending on the chosen form of the line integral. This is seen most prominently in the boundary between regions handled by different cases, such as between far-field and edge-case scenarios. Nevertheless, we see that these discontinuities are concentrated near geometric errors, such that rounding and applying an even-odd rule to the calculated GWN field still achieves excellent containment decisions for these examples.

## 6 Conclusions

In this work, we have introduced the first algorithm to accurately and precisely compute generalized winding numbers over unstructured collections of trimmed NURBS patches. We have used this to define a robust containment query that can be applied to general CAD models, even in the presence of non-watertightness, overlapping boundary components, and other geometric errors.

For points far from the surface, we evaluate the relevant surface integral using a novel application of Stokes' theorem, reducing it to a collection of line integrals defined by trimming curves. We apply the same reformulation for points near the surface, alongside a correction term which respects the exact geometric fidelity of the original surface, resulting in accurate containment decisions at arbitrary points in space. In both cases, the line integrals themselves are solved in an error-controlled manner via adaptive quadrature, ensuring a precise calculation of the fractional value of the GWN.

This theoretical basis for our algorithm extends readily to other parametric surface representations (e.g. Bézier triangles, Coons patches, Gregory patches, curve networks), as our near-field correction term is general to entirely arbitrary surfaces. While manipulation of a NURBS surface and its boundaries is made simpler through access to control points and parametric trimming curves respectively, the basic geometric primitives on which our algorithm relies are readily available for other surfaces. The only remaining barrier to practical usage is the treatment of edge-cases, which our implementation handles via manipulation of existing trimming curves. For other types of surfaces, we anticipate that clever usage of surface subdivision could be similarly applied to remove problematic portions of the surface.

Although the focus of this work is the theoretical underpinnings of our Stokes' theorem reformulation and its implications for the evaluation of the GWN for individual surface patches, we are interested in applying hierarchical approximation methods as in Barill et al. [2018] to improve batch performance across collections of patches, particularly in the case of far-field evaluation. Similarly, we are interested in potential performance improvements that come about from the use of more specialized integration techniques, such as singularity swapping [Klinteberg and Barnett 2020] or more general fast multipole methods [Bang et al. 2023], which have been demonstrated as more stable than generic quadrature methods for evaluating harmonic, near-singular integrals such as the 2D GWN.

We are also interested in rigorously extending the definition of the GWN to surfaces with ill-defined boundaries, such as NURBS surfaces with malformed trimming curves. While our method is designed to produce reasonable containment decisions when standard assumptions are not met (e.g. trimming curve watertightness), the computed GWN field degrades irregularly near this type of geometric error. This is in contrast to the true GWN field, which degrades smoothly and rapidly around the open edges of valid surfaces.

Fig. 16. We apply Algorithm 1 to a shape with a patch with complex, but defective trimming curves. The original collection of trimming curves (top left) is modified through perturbation of internal control points, while adjacent endpoints remain connected (bottom left), and by the removal of boundary components such that adjacent trimming curves no longer connect (less extreme on top right; more extreme on bottom right). We observe that while the GWN field for such shapes degrades non-locally in the presence of these errors, the containment decision remains consistent with expectations in all three cases.

<!-- image -->

## Acknowledgments

We would like to thank the anonymous reviewers for their thoughtful consideration and insightful feedback.

This work was performed under the auspices of the U.S. Department of Energy by Lawrence Livermore National Laboratory under Contract DE-AC52-07NA27344.

## References

- J. S. Asvestas. 1985. Line integrals and physical optics. Part I. The transformation of the solid-angle surface integral to a line integral. J. Opt. Soc. Am. A 2, 6 (Jun 1985), 891-895. doi:10.1364/JOSAA.2.000891
2. Axom\_data. 2025. Input data for Axom tests and examples. https://github.com/LLNL/ axom\_data. Lawrence Livermore National Laboratory.
- A. Balu, M. R. Rajanna, J. Khristy, F. Xu, A. Krishnamurthy, and M.-C. Hsu. 2023. Direct immersogeometric fluid flow and heat transfer analysis of objects represented by point clouds. Computer Methods in Applied Mechanics and Engineering 404 (Feb. 2023), 115742. doi:10.1016/j.cma.2022.115742
- S. Bang, K. Serkh, O. Stein, and A. Jacobson. 2023. An adaptive fast-multipole-accelerated hybrid boundary integral equation method for accurate diffusion curves. ACMTrans. Graph. 42, 6, Article 215 (Dec. 2023), 28 pages. doi:10.1145/3618374
- G. Barill, N. Dickson, R. Schmidt, D. I. W. Levin, and A. Jacobson. 2018. Fast winding numbers for soups and clouds. ACM Transactions on Graphics 37, 4, Article 43 (July 2018), 12 pages. doi:10.1145/3197517.3201337
- J. Binysh and G. P. Alexander. 2018. Maxwell's theory of solid angle and the construction of knotted fields. Journal of Physics A: Mathematical and Theoretical 51, 38 (Aug. 2018), 385202. doi:10.1088/1751-8121/aad8c6
- M. Borodzik, S. Dangskul, and A. Ranicki. 2017. Solid angles and Seifert hypersurfaces. arXiv:1706.06405 [math.GT] https://arxiv.org/abs/1706.06405
- A. Capps, R. Carson, B. Corbett, N. Elliott, J. Essman, B. Gunney, B. Han, C. Harrison, R. Hornung, M. Larsen, A. Moody, E. Pauli, R. Settgast, J. Spainhour, L. Taylor, K. Weiss, C. White, B. Whitlock, M. Yang, and G. Zagaris. 2025. Axom: CS infrastructure components for HPC applications . doi:10.11578/dc.20210915.2 https://github.com/ llnl/axom.
- P. C. P. Carvalho and P. R. Cavalcanti. 1995. II.2 - Point in polyhedron testing using spherical polygons. In Graphics Gems V , A. W. Paeth (Ed.). Academic Press, Boston, 42-49. doi:10.1016/B978-0-12-543457-7.50015-2
- E. B. Chin and N. Sukumar. 2021. Scaled boundary cubature scheme for numerical integration over planar regions with affine and curved boundaries. Computer Methods in Applied Mechanics and Engineering 380 (2021), 113796. doi:10.1016/j.cma. 2021.113796
- J. E. Cobb. 1988. Tiling the sphere with rational Bézier patches. In TR UUCS-88-009 . University of Utah USA, 1-14.

A. Efremov, V. Havran, and H.-P. Seidel. 2005. Robust and numerically stable Bézier

clipping method for ray tracing NURBS surfaces. In

Conference on Computer Graphics Proceedings of the 21st Spring

(Budmerice, Slovakia)

(SCCG '05)

. Association for

Computing Machinery, New York, NY, USA, 127-135.

[doi:10.1145/1090122.1090144](https://doi.org/10.1145/1090122.1090144)

- N. Feng and K. Crane. 2024. A heat method for generalized signed distance. ACM Trans. Graph. 43, 4, Article 92 (July 2024), 19 pages. doi:10.1145/3658220
- J. E. Fromm, N. Wunsch, R. Xiang, H. Zhao, K. Maute, J. A. Evans, and D. Kamensky. 2023. Interpolation-based immersed finite element and isogeometric analysis. Computer Methods in Applied Mechanics and Engineering 405 (2023), 115890. doi:10.1016/j.cma. 2023.115890
- M. Gillespie, D. Yang, M. Botsch, and K. Crane. 2024. Ray tracing harmonic functions. ACM Trans. Graph. 43, 4, Article 99 (July 2024), 18 pages. doi:10.1145/3658201
- D. Gunderman, K. Weiss, and J. A. Evans. 2020. Spectral mesh-free quadrature for planar regions bounded by rational parametric curves. Computer-Aided Design 130

[(2020). doi:10.1016/j.cad.2020.102944](https://doi.org/10.1016/j.cad.2020.102944)

- D. Gunderman, K. Weiss, and J. A. Evans. 2021. High-accuracy mesh-free quadrature for trimmed parametric surfaces and volumes. Computer-Aided Design 141 (2021), 103093. doi:10.1016/j.cad.2021.103093
- E. Haines and T. Akenine-Möller (Eds.). 2019. Ray tracing gems . Apress. http: //raytracinggems.com.
- B. Hamann and P.-Y. Tsai. 1996. A tessellation algorithm for the representation of trimmed NURBS surfaces with arbitrary trimming curves. Computer-Aided Design 28, 6-7 (1996), 461-472. doi:10.1016/0010-4485(95)00043-7
- K. Hormann and A. Agathos. 2001. The point in polygon problem for arbitrary polygons. Computational Geometry 20, 3 (2001), 131-144. doi:10.1016/S0925-7721(01)00012-8
- T. J. R. Hughes, J. A. Cottrell, and Y. Bazilevs. 2005. Isogeometric analysis: CAD, finite elements, NURBS, exact geometry and mesh refinement. Computer Methods in Applied Mechanics and Engineering 194, 39 (2005), 4135-4195. doi:10.1016/j.cma. 2004.10.008
- A. Jacobson, L. Kavan, and O. Sorkine. 2013. Robust inside-outside segmentation using generalized winding numbers. ACM Transactions on Graphics 32, 4 (2013), 1-12. doi:10.1145/2461912.2461916
- A. Jacobson, D. Panozzo, et al. 2018. libigl: A simple C++ geometry processing library. https://libigl.github.io/.
- D. [Johnson. 2000. Utah Teapot. https://users.cs.utah.edu/~dejohnso/models/teapot.html](https://users.cs.utah.edu/~dejohnso/models/teapot.html)
- D. Kamensky, M.-C. Hsu, D. Schillinger, J. A. Evans, A. Aggarwal, Y. Bazilevs, M. S. Sacks, and T. J. R. Hughes. 2015. An immersogeometric variational framework for fluid-structure interaction: Application to bioprosthetic heart valves. Computer Methods in Applied Mechanics and Engineering 284 (2015), 1005-1053. doi:10.1016/j. cma.2014.10.040 Isogeometric Analysis Special Issue.
10. D.J. Kasik, W. Buxton, and D.R. Ferguson. 2005. Ten CAD challenges. IEEE Computer Graphics and Applications 25, 2 (2005), 81-92. doi:10.1109/MCG.2005.48
- L. Klinteberg and A. H. Barnett. 2020. Accurate quadrature of nearly singular line integrals in two and three dimensions by singularity swapping. BIT Numerical Mathematics 61 (07 2020), 1-36. doi:10.1007/s10543-020-00820-5
- S. Koch, A. Matveev, Z. Jiang, F. Williams, A. Artemov, E. Burnaev, M. Alexa, D. Zorin, and D. Panozzo. 2019. ABC: A big CAD model dataset for geometric deep learning. In The IEEE Conference on Computer Vision and Pattern Recognition (CVPR) .
- A. Krishnamurthy, R. Khardekar, and S. McMains. 2009. Optimized GPU evaluation of arbitrary degree NURBS curves and surfaces. Computer-Aided Design 41, 12 (2009), 971-980. doi:10.1016/j.cad.2009.06.015
- S. Liu, L. Liu, and X.-M. Fu. 2025. Closed-form generalized winding numbers of rational parametric curves for robust containment queries. ACM Trans. Graph. 44, 4, Article 75 (July 2025), 9 pages. doi:10.1145/3730886
- E. Marchandise, C. Piret, and J.-F. Remacle. 2012. CAD and mesh repair with Radial Basis Functions. J. Comput. Phys. 231, 5 (2012), 2376-2387. doi:10.1016/j.jcp.2011.11.033
- Z. Marschner, P. Zhang, D. Palmer, and J. Solomon. 2021. Sum-of-squares geometry processing. ACM Transactions on Graphics 40, 6, Article 253 (Dec. 2021), 13 pages. doi:10.1145/3478513.3480551
- C. Martens and M. Bessmeltsev. 2025. One-shot method for computing generalized winding numbers. Computer Graphics Forum 44 (2025). Issue 5. doi:10.1111/cgf.70194

W. Martin, E. Cohen, R. Fish, and P. Shirley. 2000.

NURBS surface.

Journal of Graphics Tools

[10487519](https://doi.org/10.1080/10867651.2000.10487519)

- B. Marussig and T. Hughes. 2017. A review of trimming in isogeometric analysis: Challenges, data exchange and simulation aspects. Archives of Computational Methods in Engineering 25 (06 2017), 1-69. doi:10.1007/s11831-017-9220-9
- A. McAdams, Y. Zhu, A. Selle, M. Empey, R. Tamstorf, J. Teran, and E. Sifakis. 2011. Efficient elasticity for character skinning with contact and collisions. ACM Transactions on Graphics 30, 4, Article 37 (July 2011), 12 pages. doi:10.1145/2010324.1964932
- T. Nishita, T. W. Sederberg, and M. Kakimoto. 1990. Ray tracing trimmed rational surface patches. SIGGRAPH Comput. Graph. 24, 4 (Sept. 1990), 337-345. doi:10.1145/ 97880.97916
4. Open Cascade SAS. 2011. Open Cascade Technology. https://dev.opencascade.org/. Version 7.8.
- L. Piegl and W. Tiller. 1997. The NURBS book . Springer-Verlag. doi:10.1007/978-3-64259223-2
- L. A. Piegl and A. M. Richard. 1995. Tessellating trimmed NURBS surfaces. ComputerAided Design 27, 1 (1995), 16-26. doi:10.1016/0010-4485(95)90749-6
- A. Reshetov. 2019. Cool patches: A geometric approach to ray/bilinear patch intersections . Apress, Berkeley, CA, 95-109. doi:10.1007/978-1-4842-4427-2\_8
- R. Sawhney and K. Crane. 2020. Monte Carlo geometry processing: A grid-free approach to PDE-based methods on volumetric domains. ACM Transactions on Graphics 39, 4 (2020).
- A. Schollmeyer and B. Fröhlich. 2009. Direct trimming of NURBS surfaces on the GPU. ACM Transactions on Graphics 28, 3, Article 47 (July 2009), 9 pages. doi:10.1145/ 1531326.1531353
- A. Schollmeyer and B. Fröhlich. 2019. Efficient and anti-aliased trimming for rendering large NURBS models. IEEE Transactions on Visualization and Computer Graphics 25, 3 (2019), 1489-1498. doi:10.1109/TVCG.2018.2814987

Practical ray tracing of trimmed

[5 (09 2000). doi:10.1080/10867651.2000.](https://doi.org/10.1080/10867651.2000.10487519)

- T. W. Sederberg, G. T. Finnigan, X. Li, H. Lin, and H. Ipson. 2008. Watertight trimmed NURBS. In ACM SIGGRAPH 2008 Papers (Los Angeles, California) (SIGGRAPH '08) . Association for Computing Machinery, New York, NY, USA, Article 79, 8 pages. doi:10.1145/1399504.1360678
- R. Sevilla, S. Fernández-Méndez, and A. Huerta. 2008. NURBS-enhanced finite element method for Euler equations. International Journal for Numerical Methods in Fluids 57, 9 (2008), 1051-1069. doi:10.1002/fld.1711
- R. Sevilla, S. Fernández-Méndez, and A. Huerta. 2011. 3D NURBS-enhanced finite element method (NEFEM). Internat. J. Numer. Methods Engrg. 88, 2 (2011), 103-125. doi:10.1002/nme.3164
- R. Sevilla and A. Huerta. 2018. HDG-NEFEM with degree adaptivity for Stokes flows. Journal of Scientific Computing 77, 3 (Feb. 2018), 1953-1980. doi:10.1007/s10915-0180657-2
- J. Shen, L. Busé, P. Alliez, and N. Dodgson. 2016. A line/trimmed NURBS surface intersection algorithm using matrix representations. Computer Aided Geometric Design 48 (2016), 1-16. doi:10.1016/j.cagd.2016.07.002
- X. Sheng and B. E. Hirsch. 1992. Triangulation of trimmed surfaces in parametric space. Computer-Aided Design 24, 8 (1992), 437-444. doi:10.1016/0010-4485(92)90011-X
- J. Sloup and V. Havran. 2021. Optimizing ray tracing of trimmed NURBS surfaces on the GPU. In Computer Graphics Forum , Vol. 40. Wiley Online Library, 161-172. doi:10.1111/cgf.14410
- B. M. Smith, T. J. Tautges, and P. P. H. Wilson. 2010. Sealing faceted surfaces to achieve watertight CAD models. In Proceedings of the 19th International Meshing Roundtable , S. Shontz (Ed.). Springer Berlin Heidelberg, Berlin, Heidelberg, 177-194.
- J. Spainhour, D. Gunderman, and K. Weiss. 2024. Robust containment queries over collections of rational parametric curves via generalized winding numbers. ACM Transactions on Graphics 43, 4, Article 38 (July 2024), 14 pages. doi:10.1145/3658228
- P. Trettner, J. Nehring-Wirxel, and L. Kobbelt. 2022. EMBER: Exact mesh booleans via efficient &amp; robust local arrangements. ACM Transactions on Graphics 41, 4, Article 39 (July 2022), 15 pages. doi:10.1145/3528223.3530181
- K. Ueda. 1996. Mean normal vector to a surface bounded by Bézier curves. Computer Aided Geometric Design 13, 5 (1996), 441-451. doi:10.1016/0167-8396(95)00038-0
- H. J. Wolters. 2000. Extensions: Extrapolation methods for CAD. Curve and Surface Design (04 2000), 433-441.
- X. Xiao, P. Alliez, L. Busé, and L. Rineau. 2021. Delaunay meshing and repairing of NURBS models. Computer Graphics Forum 40, 5 (2021), 125-142. doi:10.1111/cgf. 14362
- R. Xiong, Y. Lu, C. Chen, J. Zhu, Y. Zeng, and L. Liu. 2023. ETER: Elastic tessellation for real-time pixel-accurate rendering of large-scale NURBS models. ACM Transactions on Graphics 42, 4, Article 133 (July 2023), 13 pages. doi:10.1145/3592419
- J. Zhu, Y. Lu, R. Xiong, C. Chen, and L. Liu. 2025. Projection-driven grid-BSP tree for real-time trimming on GPU. Computer Aided Geometric Design 119 (2025), 102451. doi:10.1016/j.cagd.2025.102451
- X. Zou, S. B. Lo, R. Sevilla, O. Hassan, and K. Morgan. 2024. The generation of 3D surface meshes for NURBS-enhanced FEM. Computer-Aided Design 168 (2024), 103653. doi:10.1016/j.cad.2023.103653

## A Analytic discontinuity fix for Near-Field GWN

In this section, we provide a formal proof of the correction term in Equation 6. Assuming the surface and query are shifted such that the query is at the origin, one can compute the GWN in the 'near-field' using the Stokes' theorem boundary formulation, alongside a correction term which adds or subtracts 0 . 5 for each signed intersection between the surface and the 𝑧 -axis.

Without loss of generality, we assume there is only a single such point of intersection, as the following argument can be repeated for additional intersections. We also assume, as in Section 3.2.2, that this intersection is not on the surface boundary, has a welldefined normal vector on the surface at the point of intersection (i.e. does not occur on cusps of the surface), and that the 𝑧 -axis is not tangent to the surface. Furthermore, because the query point is not coincident with the surface, there necessarily exists a neighborhood 𝑆 𝑓 ⊂ 𝑆 around the point of intersection such that the query is in the 'far-field' of 𝑆 𝑓 , i.e. there is some 𝜖 &gt; 0 such that ∥ 𝑥 -0 ∥ &gt; 𝜖 for all 𝑥 ∈ 𝑆 𝑓 . This means that the GWN with respect to this neighborhood depends only on its boundary 𝜕𝑆 𝑓 .

We use this property to deform the original surface within the boundary of the neighborhood 𝑆 𝑓 to form a flat disk 𝐷 which is orthogonal to the 𝑧 -axis. This 'virtual' procedure considerably simplifies the analytic evaluation of the GWN within subsequent steps of this proof, but never needs to be practically implemented to compute the GWN via Algorithm 1. Indeed, because the original surface matches the deformed surface at the boundary of the neighborhood 𝑆 𝑓 , this deformation does not change the value of the GWN evaluated over the entire surface 𝑆 .

For notational simplicity, we continue to refer to this deformed surface as 𝑆 , such that 𝐷 ⊂ 𝑆 . To evaluate 𝑤 𝑆 , we now consider the relevant surface integral to be split between the disk 𝐷 and the remaining surface 𝑆 \ 𝐷 , such that

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

This is essentially the procedure by which we resolve 'edge-cases' in the main algorithm, but with the advantage of knowing the exact geometry of the disk which is extracted from the surface.

By construction, the punctured surface 𝑆 \ 𝐷 does not intersect the 𝑧 -axis, and so we can apply Stokes' theorem to this portion of the integral:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

We can split the integral over the boundary 𝜕 ( 𝑆 \ 𝐷 ) into two components: the boundary of the original surface 𝜕𝑆 and the negatively oriented boundary of the disk 𝜕𝐷 𝑐 .

<!-- formula-not-decoded -->

Of these three terms, we evaluate the integral over 𝜕𝑆 through numerical integration, and indeed this is the only integral evaluated through numerical integration, as described in Section 3.1. For the remaining terms, we instead consider their behavior in the limit as the radius of the disk 𝐷 approaches zero.

The more direct of the two is the surface integral over the disk 𝐷 . Because this value is exactly equal to 𝑤 𝐷 , i.e. the winding number of the surface 𝐷 with respect to the origin, it necessarily goes to 0 as the disk decreases in surface area. Therefore, it remains to show that the line integral around 𝜕𝐷 𝑐 approaches ± 0 . 5 as the radius of the disk approaches 0.

By the construction of the deformed surface, we can parameterize the boundary of this disk on 𝑡 ∈ [ 0 , 1 ] straightforwardly as

<!-- formula-not-decoded -->

where 𝑧 = 𝑧 0 ≠ 0 is the center of the disk and 𝑟 is its radius. We substitute this parameterization and the curve differential 𝑑 fi Γ into the integral in question and after some simplification, we have:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Clearly, as 𝑟 → 0 + , this value approaches -0 . 5 if 𝑧 0 &gt; 0 and 0 . 5 if 𝑧 0 &lt; 0. Altogether, in the case where there is an intersection between the 𝑧 -axis and the original surface 𝑆 , we can write

<!-- formula-not-decoded -->

Along with our handling of points coincident with the surface in Section 3.3, this yields Equation 6, as desired. □

## B Pseudocode for Additional Algorithms

ALGORITHM 2: LinePatchIntersections Find all intersections between an oriented line and a trimmed Bézier or NURBS surface.

```
Input: 𝐿 : Line 𝑆 : Trimmed Bézier or NURBS surface 𝜖 𝑙𝑠 : Numerical tolerance for isApproximatelyBilinear Output: 𝑡, 𝑢, 𝑣 : Lists of the parameters of intersections if not intersects ( 𝐿, BoundingBox ( 𝑆 ) ) then return {} , {} , {} /* Find all intersections of 𝐿 with untrimmed patch 𝑆 */ if 𝑆 is a Bézier surface then 𝑡, 𝑢, 𝑣 ← LineBezierIntersectionsRecursive ( 𝐿, 𝑆 ) else if 𝑆 is a NURBS surface then 𝑡, 𝑢, 𝑣 ←{} , {} , {} foreach Bézier surface 𝐵 𝑖 in ExtractBezier ( 𝑆 ) do 𝑡 0 , 𝑢 0 , 𝑣 0 ← LineBezierIntersectionsRecursive ( 𝐿, 𝐵 𝑖 ) 𝑢 0 , 𝑣 0 ← RescaleToParameterSpace ( 𝑆,𝑢 0 , 𝑣 0 ) concatenate ( { 𝑡, 𝑡 0 } , { 𝑢,𝑢 0 } , { 𝑣, 𝑣 0 }) end end 𝑡, 𝑢, 𝑣 ← RemoveDuplicates ( 𝑡, 𝑢, 𝑣 ) /* Filter intersections using 2D GWN-based trim test */ foreach Intersection point 𝑢 𝑖 , 𝑣 𝑖 do if ( 𝑢 𝑖 , 𝑣 𝑖 ) ∉ TrimmingCurves ( 𝑆 ) then pop ( { 𝑡, 𝑡 𝑖 } , { 𝑢,𝑢 𝑖 } , { 𝑣, 𝑣 𝑖 }) end end return 𝑡, 𝑢, 𝑣
```

ALGORITHM 3: LineBezierIntersectionsRecursive Recursively find all intersections between an oriented line and a Bézier surface. For nearly bilinear surfaces, we use the GARP algorithm [Reshetov 2019] to directly find intersections.

```
Input: 𝐿 : Line 𝐵 : Untrimmed Bézier surface 𝜖 𝑙𝑠 : Numerical tolerance for isApproximatelyBilinear Output: 𝑡, 𝑢, 𝑣 : Lists of the parameters of intersections if not intersects ( 𝐿, BoundingBox ( 𝐵 ) ) then return {} , {} , {} if isApproximatelyBilinear ( 𝐵, 𝜖 𝑙𝑠 ) then return GARP ( 𝐵, 𝐿 ) 𝑡, 𝑢, 𝑣 ←{} , {} , {} foreach Bézier surface 𝐵 𝑖 in split ( 𝐵 ) do 𝑡 0 , 𝑢 0 , 𝑣 0 ← LineBezierIntersectionsRecursive ( 𝐿, 𝐵 𝑖 ) 𝑢 0 , 𝑣 0 ← RescaleToParameterSpace ( 𝐵,𝑢 0 , 𝑣 0 ) concatenate ( { 𝑡, 𝑡 0 } , { 𝑢,𝑢 0 } , { 𝑣, 𝑣 0 }) end
```

return

𝑡, 𝑢, 𝑣

ALGORITHM 4: isApproximatelyBilinear Returns true if a Bézier surface 𝐵 is approximately bilinear according to the position of its control points 𝑃 𝑖,𝑗

```
Input: 𝐵 : Order ( 𝑝,𝑞 ) Tensor product Bézier surface 𝜖 𝑙𝑠 : Numerical tolerance if 𝑝 ≤ 1 and 𝑞 ≤ 1 then return True 𝐵 ℓ ( 𝑢, 𝑣 ) ← Bilinear surface defined by 4 vertex control points for 𝑖 = 0 . . . 𝑝 do for 𝑗 = 0 . . . 𝑞 do /* Evaluate the bilinear surface at uniform points in parameter space */ e 𝑃 𝑖,𝑗 = 𝐵 𝑖 𝑝 , 𝑗 𝑞 𝑃 𝑖,𝑗 = ControlPoint ( 𝐵,𝑖, 𝑗 ) if squared_distance ( 𝑃 𝑖,𝑗 , e 𝑃 𝑖,𝑗 ) > 𝜖 𝑙𝑠 then return False end end end return True
```

ALGORITHM 5: EvaluateLineIntegral Numerically evaluate Equation 9 with an adaptive Gaussian quadrature rule of fixed order. For a fixed curve and recursive depth, the quadrature nodes are cached and reused across queries.

```
Input: 𝐶 : Trimming curve 𝑞 : Query point 𝜖 𝑞 : Numerical tolerance for absolute error 𝑤 = GaussianQuadrature ( 𝐶,𝑞 ) if 𝑞 ∉ BoundingBox(q) then return 𝑤 end 𝐶 1 , 𝐶 2 ← bisect ( 𝐶 ) 𝑤 1 = GaussianQuadrature ( 𝐶 1 , 𝑞 ) 𝑤 2 = GaussianQuadrature ( 𝐶 2 , 𝑞 ) if | 𝑤 - ( 𝑤 1 + 𝑤 2 ) | < 𝜖 𝑞 then return 𝑤 1 + 𝑤 2 else /* Repeat the quadrature on each half */ return EvaluateLineIntegral ( 𝐶 1 , 𝑞, 𝜖 𝑞 ) + EvaluateLineIntegral ( 𝐶 2 , 𝑞, 𝜖 𝑞 ) end
```

## C Supplemental Material

In this supplemental material, we evaluate our 3D GWN algorithm qualitatively and quantitatively on several additional watertight and non-watertight CAD models. While most of these additional shapes are taken directly from the ABC dataset [Koch et al. 2019], others were either constructed by hand in modeling software or sourced from other online repositories. The specific models used in this work can be found in [Axom\_data 2025].

For each shape, we provide summary statistics in the format of Table 1 from the primary manuscript: We compute the GWN over a uniform grid of 50 × 50 × 50 points within the shown bounding box and report the average time, in milliseconds, needed to evaluate the GWN for each surface in the shape, alongside the breakdown of all point-surface GWN calculations by case. We also visualize the 3D GWN scalar field along different 2D slices, and show the nearest integer value to each scalar. These integers can be straightforwardly mapped to a containment decision using either a non-zero rule, or an even-odd rule.

| Shape                          | Spring   | Two-Patch Spring   |
|--------------------------------|----------|--------------------|
| Number of NURBS Patches        | 64       | 2                  |
| Number of Trimming Curves      | 256      | 8                  |
| % Far-field Cases              | 99.57 %  | 18.26 %            |
| % Near-field Cases             | 0.42 %   | 76.15 %            |
| % Edge Cases                   | 0.006 %  | 5.58 %             |
| Avg. Time per Query (ms)       | 0.332    | 11.07              |
| Avg. Far-field Case Time (ms)  | 0.0043   | 0.0718             |
| Avg. Near-field Case Time (ms) | 0.1301   | 0.336              |
| Avg. Edge Case Time (ms)       | 5.55     | 94.3               |

Fig. 17. This 'Spring' shape is derived from the ABC dataset model with index 86. We modified this originally watertight shape by removing the caps at each end. In addition to the 64-patch model defined by untrimmed Bézier patches, we also consider a version composed of only 2 NURBS patches with multiple knot spans each. Interestingly, the 64-patch model has better overall performance due to the differing distribution of cases.

<!-- image -->

| Shape                          | Bobbin   |
|--------------------------------|----------|
| Number of NURBS Patches        | 18       |
| Number of Trimming Curves      | 76       |
| % Far-field Cases              | 92.56 %  |
| % Near-field Cases             | 7.00 %   |
| % Edge Cases                   | 0.42 %   |
| Avg. Time per Query (ms)       | 1.12     |
| Avg. Far-field Case Time (ms)  | 0.0173   |
| Avg. Near-field Case Time (ms) | 0.0805   |
| Avg. Edge Case Time (ms)       | 9.38     |

Fig. 18. This 'Bobbin' shape is derived from the ABC dataset model with index 933. We modified this originally watertight shape by removing the interior portion and some additional features to reveal holes on the top face.

<!-- image -->

NearestInteger Nearest Integer Generalized Winding Number NearestInteger

Fig. 19. This 'Trailer' shape is derived from the ABC dataset model with index 4192. We modified this originally watertight shape by removing the bottom face, as well as some interior details and the surface making up the windows. Note that this shape, even in its original watertight version, contains features which overlap with one another, and therefore result in winding numbers greater than 1.

<!-- image -->

| Shape                          | Trailer   |
|--------------------------------|-----------|
| Number of NURBS Patches        | 228       |
| Number of Trimming Curves      | 1092      |
| % Far-field Cases              | 99.8 %    |
| % Near-field Cases             | 0.116 %   |
| % Edge Cases                   | 0.014 %   |
| Avg. Time per Query (ms)       | 1.18      |
| Avg. Far-field Case Time (ms)  | 0.0046    |
| Avg. Near-field Case Time (ms) | 0.0585    |
| Avg. Edge Case Time (ms)       | 3.97      |

| Shape                          | Pipe   |
|--------------------------------|--------|
| Number of NURBS Patches        | 133    |
| Number of Trimming Curves      | 563    |
| % Far-field Cases              | 99.02% |
| % Near-field Cases             | 0.81 % |
| % Edge Cases                   | 0.15 % |
| Avg. Time per Query (ms)       | 21.1   |
| Avg. Far-field Case Time (ms)  | 0.0313 |
| Avg. Near-field Case Time (ms) | 0.916  |
| Avg. Edge Case Time (ms)       | 76.7   |

| Shape                          | Gear     |
|--------------------------------|----------|
| Number of NURBS Patches        | 1784     |
| Number of Trimming Curves      | 10256    |
| % Far-field Cases              | 99.92 %  |
| % Near-field Cases             | 0.066 %  |
| % Edge Cases                   | 0.0094 % |
| Avg. Time per Query (ms)       | 55.02    |
| Avg. Far-field Case Time (ms)  | 0.0289   |
| Avg. Near-field Case Time (ms) | 0.253    |
| Avg. Edge Case Time (ms)       | 18.4     |

Fig. 21. This watertight 'Gear' shape is derived from the ABC dataset model with index 9979.

<!-- image -->

| Shape                          | Joint   |
|--------------------------------|---------|
| Number of NURBS Patches        | 117     |
| Number of Trimming Curves      | 564     |
| % Far-field Cases              | 99.5 %  |
| % Near-field Cases             | 0.435 % |
| % Edge Cases                   | 0.046 % |
| Avg. Time per Query (ms)       | 1.86    |
| Avg. Far-field Case Time (ms)  | 0.0134  |
| Avg. Near-field Case Time (ms) | 0.118   |
| Avg. Edge Case Time (ms)       | 6.69    |

Fig. 22. This 'Joint' shape is derived from the ABC dataset model with index 13. We modified this originally watertight shape by adding trimming curves to the front face, and removing a patch on the top of the shape.

<!-- image -->

Fig. 23. We modeled this open 'Sliced-Cylinder' shape after an example from [Marussig and Hughes 2017] using Rhino3D.

<!-- image -->

| Shape                          | Cylinder   |
|--------------------------------|------------|
| Number of NURBS Patches        | 4          |
| Number of Trimming Curves      | 15         |
| % Far-field Cases              | 81.9 %     |
| % Near-field Cases             | 17.6 %     |
| % Edge Cases                   | 0.504 %    |
| Avg. Time per Query (ms)       | 0.183      |
| Avg. Far-field Case Time (ms)  | 0.00973    |
| Avg. Near-field Case Time (ms) | 0.0842     |
| Avg. Edge Case Time (ms)       | 4.59       |

| Shape                          | Box-Sphere   |
|--------------------------------|--------------|
| Number of NURBS Patches        | 192          |
| Number of Trimming Curves      | 968          |
| % Far-field Cases              | 99.43 %      |
| % Near-field Cases             | 0.554 %      |
| % Edge Cases                   | 0.012 %      |
| Avg. Time per Query (ms)       | 1.46         |
| Avg. Far-field Case Time (ms)  | 0.0064       |
| Avg. Near-field Case Time (ms) | 0.0844       |
| Avg. Edge Case Time (ms)       | 6.40         |

| Shape                          | Vase    |
|--------------------------------|---------|
| Number of NURBS Patches        | 28      |
| Number of Trimming Curves      | 112     |
| % Far-field Cases              | 98.00 % |
| % Near-field Cases             | 1.96 %  |
| % Edge Cases                   | 0.032 % |
| Avg. Time per Query (ms)       | 0.192   |
| Avg. Far-field Case Time (ms)  | 0.00313 |
| Avg. Near-field Case Time (ms) | 0.130   |
| Avg. Edge Case Time (ms)       | 3.80    |

Fig. 25. This 'Vase' shape is adapted from an example in [Martens and Bessmeltsev 2025], and was created as a surface of revolution from 7 cubic Bézier curves around the z-axis with circular cross-sections composed of 4 rational Bézier curves.

<!-- image -->

| Shape                          | Utah Teapot   |
|--------------------------------|---------------|
| Number of NURBS Patches        | 28            |
| Number of Trimming Curves      | 112           |
| % Far-field Cases              | 98.7 %        |
| % Near-field Cases             | 1.23 %        |
| % Edge Cases                   | 0.0271 %      |
| Avg. Time per Query (ms)       | 0.152         |
| Avg. Far-field Case Time (ms)  | 0.00310       |
| Avg. Near-field Case Time (ms) | 0.103         |
| Avg. Edge Case Time (ms)       | 4.27          |

<!-- image -->

Fig. 27. This original 'Utah Teapot' [Johnson 2000] is composed of Bézier patches, many of which overlap. Note that the lid to the teapot is disconnected from the body, and the shape has no bottom.

| Shape                          | Connector   |
|--------------------------------|-------------|
| Number of NURBS Patches        | 60          |
| Number of Trimming Curves      | 334         |
| % Far-field Cases              | 98.8 %      |
| % Near-field Cases             | 1.17 %      |
| % Edge Cases                   | 0.0778 %    |
| Avg. Time per Query (ms)       | 5.54        |
| Avg. Far-field Case Time (ms)  | 0.0584      |
| Avg. Near-field Case Time (ms) | 0.376       |
| Avg. Edge Case Time (ms)       | 38.9        |

| Shape                          | Lamp     |
|--------------------------------|----------|
| Number of NURBS Patches        | 78       |
| Number of Trimming Curves      | 214      |
| % Far-field Cases              | 97.4 %   |
| % Near-field Cases             | 2.56 %   |
| % Edge Cases                   | 0.0093 % |
| Avg. Time per Query (ms)       | 0.836    |
| Avg. Far-field Case Time (ms)  | 0.00862  |
| Avg. Near-field Case Time (ms) | 0.0753   |
| Avg. Edge Case Time (ms)       | 4.09     |

Fig. 28. This 'Lamp' shape is taken from the ABC dataset model with index 3800. Note that the central sphere of the shape has several overlapping watertight components, which increase the winding number by integer values.

<!-- image -->

| Shape                          | Bearings   |
|--------------------------------|------------|
| Number of NURBS Patches        | 66         |
| Number of Trimming Curves      | 248        |
| % Far-field Cases              | 96.4 %     |
| % Near-field Cases             | 3.38 %     |
| % Edge Cases                   | 0.145 %    |
| Avg. Time per Query (ms)       | 1.23       |
| Avg. Far-field Case Time (ms)  | 0.00882    |
| Avg. Near-field Case Time (ms) | 0.0628     |
| Avg. Edge Case Time (ms)       | 5.53       |

Fig. 29. This 'Bearings' shape is derived from the ABC dataset model with index 7963. Note that there is a small section of the outer ring that intersects with the inner ball bearings.

<!-- image -->

| Shape                          | Bolt    |
|--------------------------------|---------|
| Number of NURBS Patches        | 248     |
| Number of Trimming Curves      | 1161    |
| % Far-field Cases              | 99.7 %  |
| % Near-field Cases             | 0.206 % |
| % Edge Cases                   | 0.016 % |
| Avg. Time per Query (ms)       | 5.76    |
| Avg. Far-field Case Time (ms)  | 0.0152  |
| Avg. Near-field Case Time (ms) | 0.402   |
| Avg. Edge Case Time (ms)       | 45.2    |

<!-- image -->

Fig. 30. This 'Bolt' assembly is derived from the ABC dataset model with index 3450. Note that there is a small section where adjacent watertight parts within the assembly overlap, increasing the integer winding number by one.

| Shape                          | Slide    |
|--------------------------------|----------|
| Number of NURBS Patches        | 298      |
| Number of Trimming Curves      | 1776     |
| % Far-field Cases              | 99.8 %   |
| % Near-field Cases             | 0.144 %  |
| % Edge Cases                   | 0.0019 % |
| Avg. Time per Query (ms)       | 3.71     |
| Avg. Far-field Case Time (ms)  | 0.0063   |
| Avg. Near-field Case Time (ms) | 2.37     |
| Avg. Edge Case Time (ms)       | 30.6     |

<!-- image -->

Fig. 31. This 'Slide' shape is derived from the ABC dataset model with index 4237.