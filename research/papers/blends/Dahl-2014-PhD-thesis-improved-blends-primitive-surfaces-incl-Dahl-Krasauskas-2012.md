## IMPROVED BLENDS BETWEEN PRIMITIVE SURFACES

Heidi Elisabeth Iuell Dahl

## Dissertation presented for the degree of Philosophiæ Doctor

<!-- image -->

Centre of Mathematics for Applications

Department of Informatics

University of Oslo

## Summary

This thesis studies exact rational parametrizations of blends between the natural quadrics. The first part focuses on fixed and variable radius rolling ball blends as patches on canal surfaces. Closed formulae for the blend parametrizations are provided where possible, as well as general rational parametrization algorithms. The differential geometry of a canal surface is described in terms the differential geometry of the corresponding curve in Minkowski space. In particular, G 1 or G 2 continuity of the curve is inherited by the corresponding canal surface. The above results are applied in the construction of composite rolling ball blends with internal G 1 or G 2 continuity.

The second part of the thesis presents a new class of Pythagorean normal blending surfaces as duals of rational surfaces in isotropic space. Quadratic surfaces are classified in isotropic space, their families of isotropic Möbius circles described, and the classification extended to singular isotropic cyclides.

## List of papers

This thesis is a collection of five papers:

H. E. I. Dahl and R. Krasauskas. Rational fixed radius rolling ball blends between natural quadrics. Computer-Aided Geometric Design , 29:691-706, 2012

H. E. I. Dahl. Piecewise rational parametrizations of canal surfaces. In M. Floater, T. Lyche, M.-L. Mazure, K. Mørken, and L. L. Schumaker, editors, Mathematical Methods for Curves and Surfaces , volume 8177 of Lecture Notes in Computer Science , pages 88-111. Springer Berlin Heidelberg, 2014

H. E. I. Dahl. Rational parametrizations of edge and corner blends for isogeometric analysis. In G. Muntingh and T. Dokken, editors, SAGA - Advances in ShApes, Geometry, and Algebra , volume 10 of Geometry and Computing . Springer Berlin Heidelberg, 2014

H. E. I. Dahl and R. Krasauskas. Quadrics in isotropic space and applications. 2014

H. E. I. Dahl. Isotropic Möbius geometry and i-M circles on singular isotropic cyclides. 2014

Chapter 1 summarizes the papers, providing an overview of the theoretical setting and the original results of the thesis. The papers are included in Chapters 2-6.

## Acknowledgements

This thesis would not have been completed without the support of the people around me. I would like to thank the Computer-aided Geometry Lab at Vilnius University for welcoming me to Lithuania: Severinas Zub˙ e, Kestutis Karˇ ciauskas, and in particular Rimvydas Krasauskas who has been my main scientific advisor. Thanks are also due to my colleagues at the Department of Applied Mathematics, SINTEF ICT, for the welcome back when I resumed my position at SINTEF, in particular to Tor Dokken, my main advisor and head of the Geometry research group, and Trond Runar Hagen, head of the department, for their support and encouragement.

My PhD position at Vilnius University was funded by the European FP7 Initial Training Network SAGA (FP7-PEOPLE contract PITN-GA-2008-214584) and by the Research Council of Norway. I would like to acknowledge the inspiration provided by all the participants in the project, senior scientists and fellows alike, and by the extended SAGA family participating in the yearly workshops. Thanks to Oliver Barrowclough, Franz Fuchs, Torgunn Karoline Moe, Georg Muntingh, Elisa Postinghel, and Nelly Villamizar, for all the interesting conversations, and for being part of these last years' road towards a finished thesis.

Last but definitely not least, I am forever grateful for the patience and support of my friends and family. Thank you for your visits to Vilnius, for bringing me out of the PhDbubble from time to time, and for always showing an interest in what I am working on.

Thank you!

Heidi E. I. Dahl Oslo, June 2014

## Contents

1

Introduction

|    | 1.1                                                                | Rolling ball blends between the natural quadrics . . . . . . . . . . . . . . . . . .         | .                                                                                                                                                               |   2 |
|----|--------------------------------------------------------------------|----------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|-----|
|    |                                                                    | 1.1.1                                                                                        | Rational fixed radius rolling ball blends between natural quadrics . . .                                                                                        |   4 |
|    |                                                                    | 1.1.2                                                                                        | Piecewise rational parametrizations of canal surfaces . . . . . . . . . . .                                                                                     |   6 |
|    |                                                                    | 1.1.3                                                                                        | Rational parametrizations of edge and corner blends for isogeometric analysis . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . |   9 |
|    | 1.2                                                                | PN surface blends . . . . . . . . . . . . . . . . . .                                        | . . . . . . . . . . . . . . . . . . . . .                                                                                                                       |  12 |
|    |                                                                    | 1.2.1                                                                                        | Quadrics in isotropic space and applications . . . . . . . . . . . . . . . . .                                                                                  |  14 |
|    |                                                                    | 1.2.2                                                                                        | Isotropic Möbius geometry and i-M circles on singular isotropic cyclides                                                                                        |  17 |
|    | 1.3                                                                | Future directions of research . . .                                                          | . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                                       |  19 |
|  2 | Rational fixed radius rolling ball blends between natural quadrics | Rational fixed radius rolling ball blends between natural quadrics                           | Rational fixed radius rolling ball blends between natural quadrics                                                                                              |  21 |
|    | 2.1                                                                | Introduction . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . | . . .                                                                                                                                                           |  21 |
|    | 2.2                                                                | Theoretical background and terminology . . . . . . . . . . . . . . . . .                     | . . . . . . .                                                                                                                                                   |  22 |
|    |                                                                    | 2.2.1                                                                                        | Laguerre geometry . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                           |  23 |
|    | 2.3                                                                | Rolling ball blends of plane/cone intersections . . . . . .                                  | . . . . . . . . . . . . . .                                                                                                                                     |  26 |
|    |                                                                    | 2.3.1                                                                                        | Elliptic rolling ball blend . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                           |  26 |
|    |                                                                    | 2.3.2                                                                                        | Hyperbolic and parabolic blends . . . . . . . . . . . . . . . . . . . . . . . .                                                                                 |  29 |
|    |                                                                    | 2.3.3                                                                                        | Parametrizing cone/cone blends using the families of touching curves                                                                                            |  31 |
|    | 2.4                                                                | Parametrizing rational patches on pipe surfaces . . . . . . . . . . . .                      | . . . . . . . .                                                                                                                                                 |  31 |
|    | 2.5                                                                | Classification of pairs of natural quadrics . .                                              | . . . . . . . . . . . . . . . . . . . . . .                                                                                                                     |  35 |
|    | 2.6                                                                | Blends of rationally stable pairs of natural quadrics . . . . . . . . . . . . . .            | . . . .                                                                                                                                                         |  36 |
|    |                                                                    | 2.6.1                                                                                        | Quadratic cone/cone blends . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                |  36 |
|    |                                                                    | 2.6.2                                                                                        | Quartic cylinder/cylinder blends . . . . . . . . . . . . . . . . . . . . . . . .                                                                                |  40 |
|    |                                                                    | 2.6.3                                                                                        | Quartic cone/cone blends . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                              |  41 |
|    |                                                                    | 2.6.4                                                                                        | Quartic sphere/cylinder blends - Viviani's Curve . . . . . . . . . . . . . .                                                                                    |  43 |
|    | 2.7                                                                | Conclusions . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .          | . . . . . . .                                                                                                                                                   |  44 |
|  3 | Piecewise rational parametrizations of canal surfaces              | Piecewise rational parametrizations of canal surfaces                                        | Piecewise rational parametrizations of canal surfaces                                                                                                           |  45 |
|    | 3.1                                                                | Introduction . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .     | . . . . .                                                                                                                                                       |  45 |
|    | 3.2                                                                | Rational parametrizations of canal surfaces . . . . . . . . . . . . .                        | . . . . . . . . . .                                                                                                                                             |  46 |
|    |                                                                    | 3.2.1                                                                                        | Arcs of circles on the unit sphere . . . . . . . . . . . .                                                                                                      |  47 |
|    |                                                                    | 3.2.2                                                                                        | . . . . . . . . . . . . Parametrizing variable radius rolling ball blends . . . . . . . . . . . . . .                                                           |  48 |
|    |                                                                    | 3.2.3                                                                                        | Rational blends of the natural quadrics . . . . . . . . . . . . . . . . . . . .                                                                                 |  51 |

1

| 3.3                                                                             | Differential geometry of curves in Ρ 3,1 . . . . . . . . . . . . . .            | . . . .                                                                         |   52 |
|---------------------------------------------------------------------------------|---------------------------------------------------------------------------------|---------------------------------------------------------------------------------|------|
|                                                                                 | 3.3.1                                                                           | G 1 continuity of curves in Ρ 3,1 and canal surfaces . . . . .                  |   53 |
|                                                                                 | 3.3.2                                                                           | G 2 continuity of curves in Ρ 3,1 . . . . . . . . . . . . . . . . . .           |   54 |
| 3.4                                                                             | Differential geometry of canal surfaces                                         | . . . . . . . . . . . . . . . . . .                                             |   55 |
|                                                                                 | 3.4.1                                                                           | The principal curvatures of a canal surface . . . . . . . . . .                 |   56 |
|                                                                                 | 3.4.2                                                                           | G 2 continuous canal surfaces . . . . . . . . . . . . . . . . . . .             |   57 |
|                                                                                 | 3.4.3                                                                           | G 2 continuity with the end sphere . . . . . . . . . . . . . . .                |   58 |
|                                                                                 | 3.4.4                                                                           | The osculating cyclide . . . . . . . . . . . . . . . . . . . . . . .            |   59 |
|                                                                                 | 3.4.5                                                                           | Additional properties of canal surfaces . . . . . . . . . . . . .               |   62 |
| 3.5                                                                             | Applications: Piecewise rational corner blends                                  | . . . . . . . . . . . .                                                         |   63 |
|                                                                                 | 3.5.1                                                                           | Sequential corner blends . . . . . . . . . . . . . . . . . . . . . .            |   63 |
|                                                                                 | 3.5.2                                                                           | Spherical corner blends . . . . . . . . . . . . . . . . . . . . . . .           |   64 |
| 3.6                                                                             | Conclusions . .                                                                 | . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .               |   66 |
| 4 Rational parametrizations of edge and corner blends for isogeometric analysis | 4 Rational parametrizations of edge and corner blends for isogeometric analysis | 4 Rational parametrizations of edge and corner blends for isogeometric analysis |   69 |
| 4.1                                                                             | Introduction . . . . . .                                                        | . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                       |   69 |
| 4.2                                                                             | Beyond fixed radius blends . . .                                                | . . . . . . . . . . . . . . . . . . . . . . .                                   |   70 |
| 4.3                                                                             | It's all spheres: A short introduction to Laguerre geometry . . . .             | It's all spheres: A short introduction to Laguerre geometry . . . .             |   72 |
|                                                                                 | 4.3.1                                                                           | Minkowski space . . . . . . . . . . . . . . . . . . . . . . . . . . .           |   74 |
|                                                                                 | 4.3.2                                                                           | Curves in the bisector in Ρ 3,1 . . . . . . . . . . . . . . . . . . .           |   75 |
| 4.4                                                                             | Variable radius rolling ball blends . .                                         | . . . . . . . . . . . . . . . . . . .                                           |   76 |
|                                                                                 | 4.4.1                                                                           | Edge blends . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .         |   77 |
|                                                                                 | 4.4.2                                                                           | Corner blends . . . . . . . . . . . . . . . . . . . . . . . . . . . . .         |   79 |
|                                                                                 | 4.4.3                                                                           | Constructing composite variable radius rolling ball blends                      |   80 |
| 4.5                                                                             | Blending the example corner . . . .                                             | . . . . . . . . . . . . . . . . . . . .                                         |   84 |
| 4.6                                                                             | Generalizing the approach to blends of PN surfaces . . . .                      | . . . . .                                                                       |   87 |
| 4.7                                                                             | Conclusions . . . . . . .                                                       | . . . . . . . . . . . . . . . . . . . . . . . . . . . .                         |   88 |
| 5 Quadrics in Isotropic Space and Applications                                  | 5 Quadrics in Isotropic Space and Applications                                  | 5 Quadrics in Isotropic Space and Applications                                  |   89 |
| 5.1                                                                             | Introduction . . . . . . . . . . . . . . . . . . .                              | . . . . . . . . . . . . . . . .                                                 |   89 |
| 5.2                                                                             | Canonical forms of quadrics in Ρ 3 ++ 0 .                                       | . . . . . . . . . . . . . . . . . .                                             |   89 |
|                                                                                 | 5.2.1                                                                           | Cylinders . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .       |   91 |
|                                                                                 | 5.2.2                                                                           | Cones . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .       |   93 |
|                                                                                 | 5.2.3                                                                           | Hyperboloids . . . . . . . . . . . . . . . . . . . . . . . . . . . . .          |   94 |
|                                                                                 | 5.2.4                                                                           | Paraboloids . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .         |   94 |
|                                                                                 | 5.2.5                                                                           | Ellipsoids . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .      |   95 |
| 5.3                                                                             | Families of i-M circles on quadrics in Ρ 3 ++ 0 . . . . . .                     | . . . . . . . . .                                                               |   96 |
|                                                                                 | 5.3.1                                                                           | Cylinders . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .       |   96 |
|                                                                                 | 5.3.2                                                                           | Cones . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .       |   98 |
|                                                                                 | 5.3.3                                                                           | Hyperboloids . . . . . . . . . . . . . . . . . . . . . . . . . . . . .          |   99 |
|                                                                                 | 5.3.4                                                                           | Paraboloids . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .         |  101 |
|                                                                                 | 5.3.5                                                                           | Ellipsoids . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .      |  102 |
| 5.4                                                                             | The dual surfaces in Ρ 3 of quadrics in Ρ 3 ++                                  | 0 . . . . . . . . . . . . . . .                                                 |  102 |
| 5.4.1                                                                           | Cylinders . .                                                                   | . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                     |  104 |

## CONTENTS

|                 | 5.4.2                                                                    | Cones . . . . .                                                          |   110 112 |
|-----------------|--------------------------------------------------------------------------|--------------------------------------------------------------------------|-----------|
|                 | 5.4.3                                                                    | Hyperboloids                                                             |           |
|                 | 5.4.4                                                                    | Paraboloids .                                                            |       118 |
|                 | 5.4.5                                                                    | Ellipsoids . . .                                                         |       122 |
| 5.5             | Classification of quadrics in Ρ 3 ++ 0                                   | Classification of quadrics in Ρ 3 ++ 0                                   |       124 |
| 5.6             | Application: Rational PN surface cone blends . . . . . . . . . .         | Application: Rational PN surface cone blends . . . . . . . . . .         |       126 |
| 5.7             | Conclusions .                                                            | . . . . .                                                                |       128 |
| 6               | Isotropic Möbius geometry and i-M circles on singular isotropic cyclides | Isotropic Möbius geometry and i-M circles on singular isotropic cyclides |       131 |
| 6.1             | Introduction . . . . . .                                                 | Introduction . . . . . .                                                 |       131 |
| 6.2             | Möbius geometry in Ρ 3 ++ 0                                              | Möbius geometry in Ρ 3 ++ 0                                              |       132 |
| 6.3             | Families of i-M circles on singular isotropic cyclides . . . . . . .     | Families of i-M circles on singular isotropic cyclides . . . . . . .     |       136 |
| 6.4             | Conclusions                                                              | . . . . . .                                                              |       139 |
| Bibliography    | Bibliography                                                             | Bibliography                                                             |       141 |
| List of Figures | List of Figures                                                          | List of Figures                                                          |       147 |

## 1 Introduction

One of the major bottlenecks in the traditional computer aided design-analysis-redesign cycle is the transition between design and analysis models. The tools used in Computer Aided Design (CAD) and those used in Finite Element Analysis (FEA) have been developed independently, and their model representations have been chosen based on different needs and priorities. Isogeometric Analysis (IGA) seeks to address this by using the same geometric model throughout, from which both analysis models and design models can be extracted [13]. When developing geometric models for IGA we need to reconcile the different requirements of CAD and FEA. Though shape accuracy and geometric quality is important in CAD, gaps between adjacent surface patches are allowed within fine tolerances. In FEA, however, adjacent elements are required to match exactly in order to have watertight models, and the tradeoff is less focus on geometric quality. The introduction of IGA has therefore led to a renewed interest in exact representations of curve, surface and volume elements, for high-quality models in terms of both geometry and connectivity.

In CAD, and in particular in the design of mechanical parts, complex objects are predominantly constructed from a limited set of simple primitive surfaces: planes, the natural quadrics (spheres, and right circular cones and cylinders), and blends between them. Blending surfaces are mainly constructed by approximation using spline or NURBS (non-uniform rational B-spline) surfaces. According to [55] (presented in 1986, reporting on a survey from 1976), '99 percent of mechanical parts can be modelled exactly if one combines natural quadrics with the possibility of representing fillets and blends'. While we might expect the percentage to be somewhat lower almost 30 years later, there is still a predominance of shapes built from these primitives.

The natural quadrics are Pythagorean normal (PN) surfaces, i.e., rational surfaces with rational unit normal vector fields. A PN surface has rational offsets: if each point on the surface is moved the same distance R along its normal vector, the resulting surface (its R -offset ) is still rational. One application of PN surfaces is Computer Aided Machining (CAM), where we calculate the path of a machining tool at a constant distance from the final surface. Rational offsets are also useful in IGA where we want to construct rational volume parametrizations: by extending a PN surface parametrization along the surface normal we obtain a rational volume parametrization of uniform thickness along the boundary surface normals. Furthermore, if the original surface is a patchwork of PN surfaces with internal G 1 continuity, the unit normal vectors coincide along the boundary of adjacent patches and the resulting volume elements match exactly along the common boundary surface.

Figure 1.1: Elliptic plane/cone intersection, rolling ball, spine, and touching curves.

<!-- image -->

In current CAD systems, the blending surfaces that are represented exactly are typically patches on cylinders, spheres and tori, all of which are PN surfaces. The main result of this thesis is the extension of the list of exact PN surface blends of the natural quadrics, providing closed formulae for the blending patches when their expressions are compact, and parametrization algorithms when they are not.

The thesis is a collection of five papers, included as Chapters 2-6. The first three papers focus on exact rational parametrizations of rolling ball blends between the natural quadrics, and the construction of composite rolling ball blends with internal G 1 or G 2 continuity. They are summarized in Sec. 1.1. In the remaining two papers we construct a new class of PN surface blends as duals of rational surfaces in isotropic space, the results of which are summarized in Sec. 1.2. In Sec. 1.3 we give an overview of future directions of research.

## 1.1 Rolling ball blends between the natural quadrics

The construction of a rolling ball blend is easy to visualize (Fig. 1.1): let a sphere roll along two surfaces in such a way that at any point it is tangent to both surfaces. The two curves traced on the surfaces are called touching curves , the surface between them traced by the sphere is the rolling ball blend (Fig. 1.2), and the path traced by the centre of the sphere is called its spine curve . If the radius of the sphere varies along the intersection, this is described by the radius function of the blend. The complete surface traced by the sphere (its envelope surface) is called a canal surface , so the construction of rational parametrizations of rolling ball blends is equivalent to the construction of rational patches on canal surfaces.

In general there are four possible positions of the blend along an intersection. In order to eliminate this ambiguity we assign each surface an orientation, given by the direction of its unit normal vector field. For a sphere this is specified by the sign of its radius: when the radius is positive, the orientation of the sphere is towards its interior, if the radius is negative it is towards its exterior. The precise definition of a rolling ball blend then requires that the sphere is in oriented contact with the two surfaces along the intersection, i.e., that it is tangent and that the directions of the unit normal vectors of the two surfaces coincide at the point of tangency.

Figure 1.2: Elliptic fixed radius rolling ball blend of a plane and a cone.

<!-- image -->

The rolling ball can be considered as a one-parameter family of oriented spheres with centres on the spine curve s ( t ) and radius given by the radius function r ( t ) . Writing f ( t ) = ( s ( t ) ; r ( t )) , the space of oriented spheres is identified with the four-dimensional Minkowski space Ρ 3,1 , i.e., Ρ 4 equipped with the scalar product

<!-- formula-not-decoded -->

This scalar product defines the Minkowski metric : the distance between two points p , p 0 2 Ρ 3,1 is

<!-- formula-not-decoded -->

Remark 1.1. The distance between two points in Ρ 3,1 defined by the Minkowski metric is not necessarily real. Consider two oriented spheres, one contained completely within the other. Then the distance between their centres is smaller than the difference between their radii, and for the corresponding points p , p 0 2 Ρ 3,1 we have κ p p 0 κ 2 &lt; 0 . When κ p p 0 κ 2 0 , the Minkowski metric gives the tangential distance between the two spheres: the distance measured along a plane in oriented contact with both spheres. When the distance is zero, the two spheres are in oriented contact.

As the envelope of a one-parameter family of spheres, a canal surface corresponds to a curve in Ρ 3,1 . If the curve is rational, then so is the canal surface [45] and there exists minimal bi-degree ( n , 2 ) parametrizations of both the complete canal surface and of its rational surface patches [29]. The quadratic isoparametric curves are circles, called characteristic circles , each the contribution of one of the spheres to the envelope surface. If we consider the tangent line of a curve in Ρ 3,1 , the envelope of the corresponding linear family of spheres in Ρ 3 is called the tangent cone of the canal surface, and it is tangent along a characteristic circle.

Remark 1.2. The length of the tangent vector ˙ f is not necessarily real (see Rem. 1.1). For the canal surface to be well defined (and the envelope surface real) we require that ˙ f 2 &gt; 0 .

The geometry of oriented spheres and planes in Ρ 3 is called Laguerre geometry [11]. We associate an oriented plane in Ρ 3 with a hyperplane in Ρ 3,1 , corresponding to the spheres in oriented contact with the plane. The model of Laguerre geometry described above, where oriented spheres correspond to points and oriented planes correspond to hyperplanes, is called the cyclographic model of Laguerre geometry . It has been extensively used to study canal surfaces, in particular by Krasauskas [26-29, 32-34], and Peternell and Pottmann [46, 47, 59]. Note that the two approaches are slightly different, and that we here and in the following chapters are using the approach of Krasauskas.

Figure 1.3: Variable radius rolling ball blend of a cone and a plane: The one-parameter family of spheres and the blending patch.

<!-- image -->

Remark 1.3. While we are focusing on exact rational parametrizations of rolling ball blends and canal surfaces, there are also recent developments in approximative rational parametrization techniques for canal surfaces [8-10], and generalizations to ringed surfaces [5, 7]. There is also recent research on the use of Dupin cyclides as blending surfaces, [21, 22, 24]. These approaches do not use Laguerre geometry, but work directly on the surfaces in Ρ 3 .

In Sec. 1.1.1-1.1.3 we summarize the three papers [14, 15, 17] included in Ch. 2-4.

## 1.1.1 Rational fixed radius rolling ball blends between natural quadrics

In this first paper [17], we consider fixed radius rolling ball blends of the natural quadrics. These blends are patches on pipe surfaces , i.e., canal surfaces whose radius function is constant. There are four configurations of pairs of natural quadrics where a fixed radius rolling ball blend is rational for any radius R :

- -plan/cone intersections,
- -two cones with two points of oriented contact,
- -two cones with one point of oriented contact,
- -a cone and a sphere with one point of oriented contact.

This is a consequence of the classification of pairs of natural quadrics in [27], based on the observation in [43] that quartic Steiner surfaces and rational ruled surfaces are the only surfaces where any hyperplane section is rational.

## 1.1. ROLLING BALL BLENDS BETWEEN THE NATURAL QUADRICS

Figure 1.4: Fixed radius rolling ball blends of two cones with one (left, quartic spine) and two (right, elliptic and parabolic spine) common touching point(s).

<!-- image -->

For the first configuration, blends between a cone and a plane, the pipe surface has a quadratic spine (we say that the pipe surface is quadratic ) with constant radius function R . The rational parametrization of the blend is constructed by considering the families of cones tangent to the pipe surface. There are three such families: one family corresponds to the tangent lines of the curve in Ρ 3,1 and its cones are tangent along the characteristic circles of the canal surface. The cones in the two remaining families are tangent along families of quartic touching curves, each family covering the complete pipe surface. These cones are R -offsets of cones through the quadratic spine curve. To parametrize a fixed radius blend between two cones in the same family (one or both of which may degenerate into a plane), we parametrize the quartic touching curves over the corresponding interval.

Remark 1.4. We parametrize a quadratic blend between two of the tangent cones corresponding to tangent lines in Ρ 3,1 by parametrizing the characteristic circles of the canal surface. In fact, in our parametrization algorithms described below we parametrize the blend of two cones from different families by parametrizing arcs of characteristic circles between the two quartic touching curves.

For the second and third configurations, cone/cone blends where the two cones have one or two points of oriented contact, the above method is not suitable, as the two cones will belong to different families of tangent cones. For these cases, and for the last configuration of a sphere and a cone with one touching point, we apply the parametrization algorithm described in [29]. The parametrization F ( t , u ) of the blending patch is decomposed into the motion of the sphere along the spine curve s ( t ) and the circular curve R N t ( u ) around the sphere at each instance of t :

<!-- formula-not-decoded -->

The algorithm in [29] describes how the two endpoints 0 and 1 of the arc of circle N t ( u ) on the unit sphere are lifted to the points X and Y in Χ 2 . Areal interpolation ( 1 t ) X + t Y , t 2 Ρ , of X and Y parametrizes a line which is then projected onto an arc of circle on the unit sphere using the generalized stereographic projection. In Χ 2 this line between X and Y is not unique. In order to arrive at the correct arc between 0 and 1 , the interpolation is adjusted by a lifting coefficient . In the original algorithm in [29], is given as the unique solution of a linear system of equations. We improve the algorithm for the special case of pipe surfaces in Alg. 2.13 by deriving the explicit formulation of as a function of 0 and 1 (Lem. 2.12).

In the case of plane/cone blends, the parametrizations were sufficiently compact to be stated in their final form. When we apply Alg. 2.13 to the other configurations, however, the final expressions become unwieldy. We therefore give the closed expressions of X , Y , and , from which the parametrizations N ( t , u ) and F ( t , u ) can be calculated. The resulting parametrization of the fixed radius rolling ball blend is of minimal bi-degree ( n , 2 ) , with n specified for each of the configurations.

Two examples of quadratic and quartic fixed radius rolling ball blends between two cones are shown in Fig. 1.4. A complete set of fixed radius blends of the various configurations of natural quadrics are shown in Fig. 2.6-2.12.

Remark 1.5. The papers in this thesis are included as they were originally published, except minor copy-editing and a change of layout. Since the publication of this first paper, we have realized a simplification of the construction of the Gaussian image i of the two touching curves. In the notation of Lem. 2.4, let q 2 Ρ 3,1 be the point corresponding to the rolling ball, and p ( u 0 ) 2 Ρ 3,1 the point corresponding to the sphere inscribed in the cone in oriented contact with the rolling ball. Then κ q p ( u 0 ) κ = 0 , which means that ϕ ( q p ( u 0 )) 1..3 ϕ = ϕ ( q p ( u 0 )) 4 ϕ where ϕ . ϕ is the Euclidean metric, and ( . ) 1..3 and ( . ) 4 selects respectively the first three and the fourth coordinate of a vector in Ρ 3,1 . The left hand of the equation the distance between the centres of the two spheres, and the right the difference in radius. The Gaussian image at this point is then the unit vector

<!-- formula-not-decoded -->

This unit vector has the correct orientation with respect to the two spheres.

## Main results

- -Closed formulae for minimal bi-degree ( n , 2 ) rational parametrizations of fixed radius rolling ball blends between two natural quadrics, in the configurations where such blends exist.
- -Improvement of the parametrization algorithm for patches on canal surfaces from [29], providing the closed expression for the lifting coefficient for pipe surfaces.

## 1.1.2 Piecewise rational parametrizations of canal surfaces

We now move from the fixed radius to variable radius rolling ball blends. In the first part of [14] we extend the parametrization algorithm in [17] from pipe surfaces to canal surfaces, and from fixed to variable radius rolling ball blends. The decomposition of the parametrization in (1.3) includes a variable radius function r ( t ) :

Figure 1.5: A canal surface with its tangent and principal normal cones (left), and its osculating Dupin cyclide (right).

<!-- image -->

<!-- formula-not-decoded -->

For pipe surfaces, the characteristic circles R N ( t , u ) are great circles on the family of spheres. However, the characteristic circles r ( t ) N ( t , u ) of canal surfaces in general are not. This leads to a change in the lifting coefficient (recall that selects the correct arc of circle between two endpoints on the unit sphere) for variable radius rolling ball blends in Lem. 3.6. The parametrization algorithm for rational variable radius blends is given in Alg. 3.8.

The main focus of the paper is to describe the differential geometry of canal surfaces in terms of the differential geometry of the corresponding curves in Ρ 3,1 . The differential geometry of curves in Minkowski space Ρ 3,1 is described in [62, 63], for space-like and time-like curves respectively. This terminology is inherited from the use of Ρ 3,1 to formulate Einstein's theory of special relativity. A vector v is space-like, light-like or time-like if respectively κ v κ &gt; 0 , κ v κ = 0 , or κ v κ &lt; 0 . The local type of a curve f ( t ) is inherited from the type of its velocity vector ˙ f ( t ) , see for example the three types of lines described in Sec. 4.3.1. As we are assuming that ˙ f ( t ) &gt; 0 to ensure that the canal surface is real (Rem. 1.2), the curves in Ρ 3,1 corresponding to canal surfaces are locally space-like.

Acurve in the 4 -dimensional space Ρ 3,1 has a Frenet frame of four orthogonal unit vectors: the tangent, principal normal, and the first and second bi-normal vectors, the first three defined as in Euclidean Ρ 3 but using the Minkowski scalar product. We also define the curvature χ m of the curve in Ρ 3,1 . If we consider the corresponding canal surface in Ρ 3 , a tangent cone is the envelope of a tangent line, and we can similarly consider the envelope of the axes of the Frenet frame in Ρ 3,1 as the frame of the canal surface in Ρ 3 . Note, however, that since the signature of Minkowski space is (+++ ) , only three of these envelopes will be cones. The fourth vector in the frame will be either light-like or time-like and will not have a well-defined real envelope. A canal surface with its tangent and principal normal cones is shown on the left in Fig. 1.5.

Figure 1.6: Sequential and spherical corner blend.

<!-- image -->

Two adjacent surface patches in Ρ 3 are joined with G 1 continuity, also called tangent continuity , if their tangent planes coincide along their common boundary curve. As the family of tangent planes along a characteristic circle of a canal surface is defined by the tangent cone, two segments of canal surface are joined with G 1 continuity if and only if their tangent cones coincide along the common characteristic circle, i.e., if the tangent lines of the corresponding curve segments in Ρ 3,1 coincide. Thus G 1 continuity of a canal surface is inherited from the corresponding curve in Ρ 3,1 .

To obtain G 2 continuity, also called curvature continuity , we need to compare the principal curvatures and principal curvature directions along the common boundary curve of two adjacent surface patches. These are completely defined by the shape operator S of the surface, by respectively its eigenvalues and eigenvectors. The shape operator of a canal surface is a triangular matrix, and we find the two principal curvatures ( 1 , 2 , and X are defined in Sec. 3.4.1)

<!-- formula-not-decoded -->

The curvature line associated with the principal curvature χ 2 is the characteristic circle at t . As the principal curvature directions are perpendicular, they coincide for two segments of canal surfaces joined with G 1 continuity along a characteristic circle. Using the expressions for the principal curvatures, we show that if two curve segments in Ρ 3,1 are joined with G 2 continuity, then so are the corresponding segments of canal surfaces in Ρ 3 .

We also find the conditions for G 2 continuity with the end sphere of a curve segment. When the curve segment is limited by the condition ˙ f 2 &gt; 0 , the corresponding segment of canal surface closes to a point with G 2 continuity (see for example the figure on the right in Fig. 1.11). We can also obtain G 2 continuity along a circle on the end sphere if ϕ ˙ s ϕ 2 = 0 .

In Euclidean Ρ 3 , the osculating circle at a point on a curve is the circle best approximating the curve in this point. The corresponding concept in Ρ 3,1 is the osculating pseudo-Euclidean (PE) circle . The osculating PE circle is unique, and by construction G 2 continuous with the curve. The canal surface corresponding to a PE circle is a Dupin cyclide, giving us the unique osculating Dupin cyclide of the canal surface [6, 38, 47]. A characteristic of Dupin cyclides is that they are envelopes of two one-parameter families of spheres corresponding to two distinct curves in Ρ 3,1 . For the osculating cyclide, the second curve corresponds to the family of osculating spheres of the canal surface along the characteristic circle, i.e., the spheres tangent to the canal surface whose radius is the reciprocal of the principal curvature χ 1 at the point of tangency. A canal surface and its osculating Dupin cyclide is shown on the right in Fig. 1.5.

The parametrization algorithm and the continuity results summarized above can be exploited to construct composite blends with internal G 1 or G 2 continuity. We give two examples of such constructions: sequential and spherical corner blends. In a sequential corner blend, shown on the left in Fig. 1.6, the corresponding curve in Ρ 3,1 is a sequence of curve segments connected with G 1 continuity. The blend is a patch on the resulting canal surface with G 1 continuity. In a spherical corner blend, shown on the right in Fig. 1.6, we construct transitional edge blends to connect with a common sphere at the corner, guaranteeing internal G 2 continuity by ensuring that ϕ ˙ s ϕ 2 = 0 at the end sphere.

## Main results

- -An extension of parametrization algorithm from [17] from fixed to variable radius rational rolling ball blends, by an adjustment of the lifting coefficient .
- -A description of the differential geometry of curves in Ρ 3,1 , based on [62, 63], and the corresponding structures for canal surfaces. This is used to prove that the G 1 and G 2 continuity of a curve in Ρ 3,1 is inherited by the corresponding canal surface in Ρ 3 .
- -We apply the parametrization algorithm and continuity results to two constructions of internally G 1 and G 2 corner blends.

## 1.1.3 Rational parametrizations of edge and corner blends for isogeometric analysis

The third paper [15] on rolling ball blends of the natural quadrics presents a novel approach to the construction of blends of composite corners. In traditional blend constructions you typically specify the boundary curves in the two surfaces, and then approximate a G 1 continuous surface patch between them [60]. We move the blend construction to Minkowski space Ρ 3,1 , constructing a series of curves connected with G 1 or G 2 continuity. Algorithm 3.8 from [14] can then be applied to parametrize the blending patch on the corresponding canal surface. We construct the blend by placing control spheres , specifying the blending radius at certain key points. An example of a corner with control spheres and the resulting composite blend is shown in Fig. 1.7.

Consider the spheres in oriented contact with an oriented surface in Ρ 3 . The corresponding hypersurface in Ρ 3,1 is called its isotropic hypersurface in Ρ 3,1 . The intersection of two isotropic hypersurfaces is a 2 -dimensional surface corresponding to all spheres in oriented contact with both surfaces, called the bisector surface in Ρ 3,1 . There is a one-to-one correspondence between rolling ball blends between the two surfaces and curves in the bisector surface in Ρ 3,1 . On the left in Fig. 1.8 we see the bisector surface of two cylinders with one touching point, containing a family of spine curves. This surface in Ρ 3 is the projection onto the first three coordinates of the bisector surface in Ρ 3,1 , and the spine curves are the projections of the curves in Ρ 3,1 corresponding to a family of variable radius rolling ball blends of the two cylinders.

Figure 1.7: The control spheres and the blend of a composite corner.

<!-- image -->

When the two surfaces are planar, then so is their bisector surface in Ρ 3,1 . Any rational curve corresponding to a rational rolling ball blend can then be constructed as a Bézier curve in this 2 -dimensional plane [41]. The control points of the curve correspond to control spheres of the blend. When the bisector surface in Ρ 3,1 is non-planar we construct curves by taking hyperplane sections, specifying a sufficient number of tangent cones and control spheres to obtain at least G 1 continuity between adjacent patches. We choose to use hyperplane sections in order to keep the parametrization degrees of the curves low, but alternative construction methods of curves in surfaces may be used as long as they can preserve tangent lines at the endpoints.

In addition to the two edge blend constructions, planar Bézier curves and hyperplane sections, we describe two approaches to corner blends. For a homogeneous corner where all the adjoining edges are convex/concave we use the spherical blend described in [14]. When the three adjoining edge blends corresponds to Bézier curves, we can ensure G 2 continuity between the edge blends and the corner blend by applying Rem. 3.18 and making the point corresponding to the vertex sphere a double control point.

When the corner is heterogeneous we construct a Dupin cyclide blending patch, effectively blending the edge blend of the single edge with the opposing face. Dupin cyclide blends have been extensively studied, e.g., in [2-4, 21, 22, 28, 52-54, 56, 57], but the application to corner blends is new.

Figure 1.8: Spine curves in the bisector surface of two cylinders, and a cyclide corner blend with its tangent cones.

<!-- image -->

Remark 1.6. Compared to some of the previous approaches, the description of Dupin cyclides and Dupin cyclide blends is more natural in Minkowski space. A Dupin cyclide corresponds to two PE circles at zero distance from each other. By determining the point of oriented contact between pairs of spheres corresponding to points on the two PE circles, we construct a bi-degree ( 2,2 ) parametrization of the Dupin cyclide. The isoparametric curves are the characteristic circles with respect to the two PE circles, and they coincide with the principal curvature lines of the Dupin cyclide. Any two cones corresponding to tangent lines of one of the PE circles can thus be blended using the patch of Dupin cyclide between the corresponding characteristic circles.

In the case of a single edge blend, the only restriction on our choice of control spheres is that they should be in oriented contact with both surfaces, i.e., that the corresponding points in Ρ 3,1 lie in the bisector surface. When constructing a composite blend of a network of edges and corners, we also need to consider the continuity between adjacent blending patches. In the sequential construction of a composite blend, one end sphere and tangent cone is given by the preceding blending patch. A hyperplane is then defined by the choice of a second end sphere, and if there are sufficient degrees of freedom, a final control point or tangent cone. On the right in Fig. 1.8 we see a Dupin cyclide corner blend and its tangent cones. The two corresponding tangent lines determine the tangent direction of the curves corresponding to the adjoining edge blends.

Algorithm 4.4 describes how a composite blend is constructed, ensuring at least G 1 continuity between adjoining patches. It is illustrated by an example, both in the figures of [15] and by a description of how the 11 steps of Alg. 4.4 are completed. This approach may be generalized to a wider class of primitive surfaces, such as PN surfaces.

## Main results

- -A new approach for the construction of composite corner blends by considering blends as piecewise rational curves in Ρ 3,1 . We construct these curves with G 1 or G 2 continuity

between adjacent curve segments, ensuring the same level of continuity between adjacent patches of the corresponding variable radius rolling ball blends.

- -The parametrization algorithm for rational composite corner blends is illustrated by an example.

## 1.2 PN surface blends

In our work on rational parametrizations of rolling ball blends we have exploited the simplicity of the structure of canal surfaces: when constructing parametrizations where the characteristic circles are isoparametric curves, the complexity of a canal surface is on the level of the curve in Ρ 3,1 rather than a surface in Ρ 3 . As a consequence canal surfaces are easily parametrizable and their shape is predictable, as the family of characteristic circles limits their shape.

However, in terms of versatility and flexibility the structural simplicity of rolling ball blends is a severe limitation. If we use the method of [15] and place control spheres to define a quadratic rolling ball blend of a plane and a cone, then we only need to specify the blending radius at two points to uniquely define a symmetric blend, and at three points to define an asymmetric blend. Thus the blend is constructed with at most three degrees of freedom. Furthermore, while some canal surfaces are of low implicit degree, such as Dupin cyclides of degree 3 or 4 , the implicit degree of a canal surface will in general be comparatively high [20].

In this section, and in the two papers summarized below, we study a more general class of blending surfaces that include quadratic canal surfaces: PN surface blends that are dual to quadratic surfaces in three-dimensional isotropic space.

Remark 1.7. PN surfaces are related to rational two-parameter families of spheres in Ρ 3 , i.e., rational 2 -dimensional surfaces in Ρ 3,1 [44, 48]. Though the converse is true for curves, this is no longer the case for 2 -dimensional surfaces.

Three-dimensional isotropic space , denoted Ρ 3 ++ 0 , is Ρ 3 equipped with the scalar product η , ι with signature (++ 0 ) :

<!-- formula-not-decoded -->

This gives us the isotropic metric

<!-- formula-not-decoded -->

i.e., the distance between the two points p = p x , p y , p z , p 0 = p 0 x , p 0 y , p 0 z 2 Ρ 3 ++ 0 is measured horizontally, parallel to the plane z = 0 .

Remark 1.8. In this notation Minkowski space Ρ 3,1 is written Ρ 4 +++ . However, to keep the notation consistent throughout the thesis we still refer to Minkowski space as Ρ 3,1 .

The isotropic counterpart of Möbius geometry, where isotropic Möbius (i-M) spheres are vertical rotational elliptic paraboloids and non-isotropic (non-vertical) planes, is described in [35, 46, 47, 50]. The intersection of two i-M spheres is an i-M circle, and is either a nonisotropic line, a vertical parabola, or an ellipse whose top view (i.e., the orthogonal projection onto the plane z = 0 ) is a circle.

Figure 1.9: The dual surfaces of vertical hyperboloids in Ρ 3 ++ 0 . The figure on the right is the dual canal surface of a rotational hyperboloid.

<!-- image -->

Our interest in Ρ 3 ++ 0 is due to a theorem from [46, 47]: There is a 1-1 correspondence via duality between non-developable rational offset surfaces (Pythagorean normal (PN) surfaces) in Euclidean space and rational surfaces in isotropic space (the result is stated explicitly in Thm. 10 of [35]). In the previous section we described the cyclographic model of Laguerre geometry. The dual correspondence gives us an isotropic model of Laguerre geometry [47, 50]. In this model oriented planes in Ρ 3 are dual to points in Ρ 3 ++ 0 , and oriented spheres are dual to i-M spheres. Furthermore, an oriented circular cone is dual to an i-M circle in Ρ 3 ++ 0 , thus a surface in Ρ 3 ++ 0 containing a family of i-M circles is dual to a surface in Ρ 3 tangent to a corresponding family of tangent cones. We use this to construct a new class of PN blending surfaces between two cones, and between cones and planes.

Some examples of these dual surfaces, and the curves corresponding to the families of i-M circles, are shown in Fig. 1.9-1.12. A complete set of figures of the duals of the canonical forms of quadrics in Ρ 3 ++ 0 can be found in Ch. 5.

The construction of a blend parametrization from its dual in Ρ 3 ++ 0 is straightforward. If F ( s , t ) = ( F 1 = F 0 , F 2 = F 0 , F 3 = F 0 ) is a parametrization of the surface in Ρ 3 ++ 0 , we write

<!-- formula-not-decoded -->

for the mixed product of F i j k with its partial derivatives. Then the parametrization of the dual surface is

<!-- formula-not-decoded -->

Figure 1.10: The dual surfaces of semi-vertical (left) and horizontal (right) one-sheeted hyperboloids.

<!-- image -->

In general this results in a significant increase in the bi-degree of the parametrization from ( m , n ) for the surface in Ρ 3 ++ 0 to ( 5 m 1,5 n 1 ) for its dual. We are therefore interested in classes of surfaces in Ρ 3 ++ 0 whose duals are of relatively low degree. For blending purposes we also require that the surfaces contain one or more families of i-M circles.

Remark 1.9. When the parametrization of the dual surface is calculated from a surface of bidegree ( m , n ) in the Blaschke cylinder in Π 4 , the increase in bi-degree is given by ( 3 m 2,3 n 2 ) [30]. The increase in bi-degree when we calculate from a surface in Ρ 3 ++ 0 is expected, as the inverse stereographic projection from Ρ 3 ++ 0 to the Blaschke cylinder is quadratic. However, the image of an i-M circle under the inverse stereographic projection is a quadratic curve: the intersection of the Blaschke cylinder with a two-dimensional plane in Π 4 [35, Thm. 3]. Thus curves in Ρ 3 corresponding to i-M circles in Ρ 3 ++ 0 are of degree 4 or lower, not 9 as in the original upper limit.

In Sec. 1.2.1 we summarize [18] which is a classification of families of i-M circles on quadratic surfaces in Ρ 3 ++ 0 . Section 1.2.2 summarizes [16] which extends the classification to singular isotropic cyclides in Ρ 3 ++ 0 . The two papers are included as Ch. 5-6.

## 1.2.1 Quadrics in isotropic space and applications

In Euclidean Ρ 3 , irreducible quadratic surfaces (shortened to quadrics ), are classified into cylinders (elliptic, parabolic, and hyperbolic), cones, hyperboloids (one- and two-sheeted), paraboloids (elliptic and hyperbolic), and ellipsoids. By applying rigid motions - translations and rotations around an axis - the quadrics can be placed in canonical positions at the origin. Rigid motions are transformations that preserve the shape and size of surfaces. This is generalized to other metric spaces by the concept of isometries : transformations that preserve distances. In Euclidean Ρ 3 , isometries are either rigid motions, reflections in a plane, or compositions of these transformations (all of which may be decomposed into reflections). The quadrics in canonical positions are isometrically distinct, i.e., they are representatives of isometric equivalence classes of quadrics in Euclidean Ρ 3 .

Figure 1.11: The dual surface of an ellipsoid. The figure on the right is the dual canal surface of a sphere in Ρ 3 ++ 0 .

<!-- image -->

In isotropic space the set of isometries is slightly different, as distances are measured horizontally. The linear transformations which preserve distances are compositions of

- -translations,
- -rotations around the z -axis,
- -reflections in horizontal and vertical planes,
- -vertical scalings by a strictly positive factor, and
- -vertical shears.

The number of canonical forms of the quadrics then increases from the nine in Euclidean Ρ 3 to 23 distinct canonical forms and isometric equivalence classes in Ρ 3 ++ 0 .

The total number of families of i-M circles on a surface in Ρ 3 ++ 0 is constant under isometries (for ease of notation we write 'lines' instead of 'linear i-M circles'). For each of the 23 canonical forms we determine its families of i-M circles of each type and provide parametrizations where the families are isoparametric curves.

We then consider the dual surfaces in Ρ 3 of the quadrics in Ρ 3 ++ 0 . A family of i-M circles is dual to a family of tangent cones, which we define by its implicit equation in Ρ 3 . Each cone is tangent to a curve on the dual surface corresponding to an i-M circle on the quadric.

For some of the canonical forms, the dual is a quadratic canal surface, i.e., a canal surface corresponding to a quadratic curve in Ρ 3,1 (see, e.g., the figures on the right in Fig. 1.9, Fig. 1.11, and Fig. 1.12). Quadratic canal surfaces have been classified up to pseudo-Euclidean (PE) equivalence in [34]. The classification and the other results of the paper are summarized in Table 5.2.

Whenthe dual is a canal surface and we consider the L-equivalence class(es) of the dual, we see that three of these canonical forms are L-equivalent: upright parabolic cylinders, vertical elliptic paraboloids, and vertical hyperbolic paraboloids correspond to cubic Dupin cyclides of type P [ 2 ] + 0 . These are the only canonical forms that contain a two-parameter family of parabolic i-M circles, and their dual canal surface has a two-parameter family of tangent cones. The two families of characteristic circles of the Dupin cyclide correspond to intersections with planes parallel to the two planes of symmetries of the surfaces in Ρ 3 ++ 0 .

Figure 1.12: The dual surface of a horizontal elliptic paraboloid (left) and a horizontal elliptic cylinder (right).

<!-- image -->

Cones in Ρ 3 ++ 0 are dual to classical quadratic canal surfaces, i.e., their canonical forms correspond to the four classes of canal surfaces with signature (++) [34]. These canal surfaces have a quadratic spine, and a radius function that depends linearly on the coordinates of the spine. In the most symmetric case, i.e., the vertical circular cone, the dual is a quartic Dupin cyclide. Non-vertical cylinders are dual to quadratic canal surfaces of signature (+ 0 ) . The six canonical positions of non-vertical cylinders correspond to the six classes of quadratic curves in Ρ 3,1 with signature (+ 0 ) described in [34]. The families of lines on the cones and cylinders correspond to the characteristic circles of the canal surfaces.

We also find some canal surfaces of signature (+ ) : the duals of vertical rotational ( a = 1 ) one- and two-sheeted hyperboloids, and the duals of spheres. These canonical quadrics in Ρ 3 ++ 0 are canal surfaces only in these symmetric cases, so the general case of duals of vertical hyperboloids and ellipsoids are generalizations of certain canal surfaces. For all three cases, the number of families of tangent cones (and total number of families of lines and i-M circles) is reduced by one when the duals are canal surfaces.

For the remaining (non-vertical) canonical positions of hyperboloids and paraboloids, the duals are never canal surfaces, and constitute a new class of PN blending surfaces for circular cones. In the case of semi-vertical and horizontal two-sheeted hyperboloids, the dual surfaces resemble the duals of respectively semi-vertical and upright hyperbolic cylinders. Thus these PN surfaces can be seen as generalizations of canal surfaces of L-equivalence class P + 0 and e H + 0 .

We expect that the duals of one-sheeted hyperboloids in particular will be versatile when used to construct blending surfaces between two cones: the number and variety of families of lines and i-M circles enables us to parametrize both the hyperboloid and its dual at relatively low bi-degree.

Our motivation for this classification of the quadratic surfaces in Ρ 3 ++ 0 is the application to PN blending surfaces between cones and between cones and planes. We provide an example construction which is a generalization of quadratic rolling ball blends between cylinders and planes.

Figure 1.13: Blending three cylinders: A patch on the torus (left) and a patch on the PN surface (right).

<!-- image -->

As we described in Sec. 1.1.3, a quadratic rolling ball blend of a cone or cylinder with a plane is uniquely determined by three control spheres, or by a common tangent cone and a control sphere. Equivalently, the blend is defined by two tangent cones inscribing a common sphere. If we relax this last conditions, allowing any two tangent cones (with a total of four degrees of freedom), we can construct a bi-degree ( 3,4 ) blend between the cylinder, the plane, and the two tangent cones. The dual image in Ρ 3 ++ 0 of the plane is a point, and the duals of the cones and cylinder are three arcs of i-M circles limiting a triangular patch. By applying the parametrization algorithm in [35, Thm. 1] with the dual of the plane as a double control point of a rectangular surface patch, we find a bi-degree ( 2,2 ) parametrization of the dual of the blend. Then (1.10) gives us the blend parametrization. Fig. 1.13 shows a blending patch on the torus, defined by two tangent cylinders of equal radius, and the PN blending surface obtained by halving the radius of one of the tangent cylinders.

## Main results

- -Aclassification of irreducible quadratic surfaces in Ρ 3 ++ 0 up to isometric equivalence. Each of the 23 equivalence classes is defined by a canonical form.
- -For each of the canonical forms we give the number and type of the families of i-M circles they contain, and the corresponding families of tangent cones of the dual surface in Ρ 3 .
- -When the dual is a canal surface, we determine its L-equivalence class.
- -When the dual is a non-canal PN surface, we use it to construct a new class of cone/cone and cone/plane blends.

## 1.2.2 Isotropic Möbius geometry and i-M circles on singular isotropic cyclides

In the isotropic counterpart to Möbius geometry [35, 46, 47, 50], i-M transformations map i-M circles to i-M circles. Thus the classification of families of i-M circles on quadrics in Ρ 3 ++ 0

[18] can be extended to the images of the quadrics under i-M transformations. In [16] we adapt the results of Möbius geometry in [40, Sec. 5] from Euclidean Ρ 3 to isotropic space Ρ 3 ++ 0 , and apply the results to generalize the classification summarized in Sec. 1.2.1.

All i-M transformations can be expressed in terms of the following four generating transformations: uniform scalings, translations, inversions, and vertical reflections [35, Lem. 1].

Let B be the Blaschke cylinder [35]

<!-- formula-not-decoded -->

a stereographic projection from the Blaschke cylinder to the projective space over Ρ 3 ++ 0

<!-- formula-not-decoded -->

and = 1 its inverse

<!-- formula-not-decoded -->

We define the i-M transformation diagram of an i-M transformation as

<!-- formula-not-decoded -->

The diagram commutes for a given linear isomorphisms associated with each of the four generators above.

If W 2 Π 3 is a surface, then its i-M type ( d , c ) consists of its degree d and the multiplicity c of the isotropic absolute conic in W . The i-M degree of W is the degree of its i-M model ( W ) B , and we show that if the i-M type of W is ( d , c ) then its i-M degree is 2 ( d c ) . As is linear, it follows that the i-M degree is invariant under i-M transformations.

We also determine how the i-M type ( d , c ) of a surface W changes under the inversion inv p = T p inv T 1 p in a point p 2 Ρ 3 ++ 0 . If p is of multiplicity m with respect to W , then the i-M type of inv p ( W ) is ( 2 ( d c ) m , ( d c ) m ) .

An isotropic cyclide is defined by the equation

<!-- formula-not-decoded -->

6

where L and Q are respectively linear and quadratic polynomials. When a = 0 the i-M type of the quartic isotropic cyclide is ( 4,2 ) , and when a = 0 and L = 0 its i-M type is ( 3,1 ) . Using the formula above we find that if you invert the isotropic cyclide through a singularity of multiplicity 2 , then the resulting surface is of i-M type ( 2,0 ) , i.e., it is a quadric. As the number of families of lines and i-M circles remain constant under i-M transformations, our classification of families of lines and i-M circles on the quadrics may thus be extended to singular isotropic cyclides. Figure 1.14 shows two examples of quartic isotropic cyclides, inverse images of a horizontal cone, and their three families of i-M circles.

6

Figure 1.14: Two quartic isotropic cyclides, inverse images of a horizontal cone, and their three families of i-M circles.

<!-- image -->

## Main results

- -The isotropic counterpart of the results from Euclidean Möbius geometry described in [40, Sec. 5], in particular the formula for the change in degree under i-M inversions and the preservation of the number of families of lines and i-M circles under i-M transformation.
- -By applying these results to isotropic cyclides, we find an extension of our classification of families of i-M circles on quadrics [18] to singular isotropic cyclides.

## 1.3 Future directions of research

There are several interesting directions of future research based on the work we have completed in this thesis. One area that we have not touched upon here is quality of parametrization . Our parametrizations are high-quality in the sense that they are exact and where possible of minimal bi-degree, but we have not considered, e.g., the evenness of the parametrizations. Consider the fixed radius rolling ball blend of two cones with one touching point on the left in Fig. 1.4. We see that the parameter lines are gathered on the left hand side, both on the left and on the right hand side of the smaller cone. This means that we can not improve the evenness of the parametrization by a linear re-parametrization.

A second area of interest is extension of these surface parametrizations to volume parametrizations, and their applications in Isogeometric Analysis. This would include the implementation of our parametrization algorithm in industrial CAD and analysis systems. A natural continuation would also be to investigate blends between a wider class of primitive surfaces, such as PN surfaces.

Finally, we would like to continue our investigations of surfaces in isotropic space containing families of i-M circles, and their applications in the construction of low bi-degree parametrizations of PN blending surfaces in Euclidean Ρ 3 . This includes families of i-M circles on non-singular isotropic cyclides, and a theory of webs of i-M circles or i-M circular meshes [49] as a parallel to the surfaces in Ρ 3 described in [51]. Generalizing further, we would like to develop a theory of i-M celestial surfaces, as an isotropic analogue to [40].

2

## Rational fixed radius rolling ball blends between natural quadrics

Heidi E. I. Dahl and Rimvydas Krasauskas

Published in Computer Aided Geometric Design [17]

## Abstract

By applying results on canal surfaces, we study exact rational parametrizations of fixed radius rolling ball blends of pairs of natural quadrics. We classify all configurations where this kind of rational parametrization is possible, and describe a general algorithm for parametrizing fixed radius rolling ball blends. The algorithm is then applied to parametrize the fixed radius rolling ball blends of pairs of natural quadrics.

## 2.1 Introduction

Simple primitive shapes play an important role in CAD, as building blocks of more complex shapes. According to [55], '99 percent of mechanical parts can be modelled exactly if one combines natural quadrics with the possibility of representing fillets and blends', and while we might expect the percentage to be somewhat lower 25 years later, there is still a predominance of shapes built from these primitives. In this context planes are considered natural quadrics, along with spheres and right circular cylinders and cones. Fillets and blends (in the following we write 'blends' for both) are usually generated by fixed radius rolling ball methods. Though the natural quadrics are rational, in general rolling ball blends between them are not, so in current CAD systems they are constructed by approximation in all but the simplest cases. On the other hand, if we consider the complete surface traced by a rolling ball, not just the patch giving the blend, it is self-evident that this is a canal surface : the envelope of a one-parameter family of spheres.

Canal surfaces have been studied extensively during the last 15 years by several authors (see, for example, [12, 29, 37, 39, 45]). It has been proved that a canal surface with a rational spine (the curve traced by the centres of the spheres) and rational radius function is itself rational. Constructions of canal surface parametrizations have been presented, together with their degree bounds. Unfortunately, there is still a gap in the literature in terms of applying the theoretical results on canal surfaces to practical applications in CAD. In part this is a result of the differing world views of mathematics and engineering: the non-trivial surfaces interesting to mathematicians studying canal surfaces, while having exact rational parametrization, are of too high degree to be of any interest in practical applications. But it turns out that some simple cases, which can be parametrized with reasonably low degrees, are in fact prevalent in CAD: rolling ball blends of two natural quadrics with rational offset intersections.

Figure 2.1: Elliptic plane/cone intersection, rolling ball, spine, and touching curves.

<!-- image -->

Although shape accuracy is important in current CAD, there is no requirement that adjacent surfaces match exactly, so gaps within fine tolerances are allowed. However, the introduction of Isogeometric Analysis (see e.g. [13]) changes this as in Finite Element Analysis adjacent elements are required to match exactly. Consequently, there is growing interest in employing exact shape representations when possible to minimize the challenges related to approximation. The aim of our paper is to close the gap in the literature by applying theoretical results on canal surfaces, and by doing so extend the list of exact rational rolling ball blends of natural quadrics.

We start by introducing the necessary theoretical background in Sec. 2.2. In Sec. 2.3 we construct the parametrization of the blend for the simplest configuration of natural quadrics: plane/cone intersections. In Sec. 2.4 we present an algorithm for minimal degree parametrizations of fixed radius rolling ball blends of two surfaces with rational offset intersections, and in Sec. 2.5 we classify the remaining configurations of natural quadrics whose blends can be parametrized rationally by our approach. In Sec. 2.6 we show how the blend of two cones, and a cone and a sphere are parametrized. Finally, we sum up our results in Sec. 2.7.

## 2.2 Theoretical background and terminology

Fixed radius rolling ball blends between surfaces are a common feature in CAD programs. It is an easy concept to visualize (see Fig. 2.1): let a ball of radius R roll along the intersection of the two surfaces in such a way that at any point it is tangent to both surfaces. The two curves traced on the surfaces by the ball are called touching curves , and the surface traced by the ball between them is the rolling ball blend . The radius of the blend is the radius R of the rolling ball, and the path traced by the centre of the ball is called its spine curve . The complete surface traced by the ball is a pipe surface : a canal surface with constant radius.

Figure 2.2: The orientations of the surfaces determine the placement of the rolling ball.

<!-- image -->

Two intersecting surfaces have several possible blending surfaces: in Fig. 2.1 the blend can be placed above or below the plane, and inside or outside the cone. In order to make the positioning of the blend unambiguous, we assign surfaces orientations given by the direction of their unit normal vectors (for ease of notation, simply called normals in the rest of the text). For spheres orientation is encoded in the sign of the radius: a positive radius corresponds to normals oriented towards the inside of the sphere, a negative radius corresponds to normals oriented outwards. The blend is placed where the orientation of the rolling ball coincides with the orientation of the two surfaces, i.e., the rolling ball is in oriented contact with the two surfaces. In Fig. 2.1, if the radius of the rolling ball is positive, then the orientation of the plane and the half-cone it intersects is up- and outwards respectively (note that the orientations of the two half cones are opposite). By allowing the radius of the blend to be negative, we have reduced the number of cases to the two in Fig. 2.2.

The R-offset of a surface is constructed by moving each point on the surface the same length R along its normal. The natural quadrics are offset stable in the sense that their type is preserved when offsetting: the offset of a cylinder is still a cylinder, and so on. This offset stability is advantageous in e.g. Isogeometric analysis and applications in architecture. It is also useful when we determine the spine of a rolling ball blend:

Remark 2.1. If we intersect the R -offsets of two intersecting natural quadrics, we obtain a curve that is equidistant from the two surfaces. This is the spine of the rolling ball blend of radius R .

The touching curves of the blend are found by projecting the spine onto the two surfaces. To determine the projections onto the natural quadrics it is convenient to use some elements of Laguerre geometry.

## 2.2.1 Laguerre geometry

Laguerre geometry is a geometry of spheres - instead of considering points and distances between points, we consider oriented spheres and tangential distances between spheres. An oriented sphere p is given by its centre x = ( x 1 , x 2 , x 3 ) 2 Ρ 3 and radius x 4 2 Ρ . Using the notation p = ( x ; x 4 ) = ( x 1 , x 2 , x 3 ; x 4 ) , the space of all oriented spheres is identified with the Minkowski space Ρ 3,1 , i.e., 4-dimensional space Ρ 4 equipped with the Minkowski scalar product of vectors:

Figure 2.3: Tangential distance between spheres.

<!-- image -->

<!-- formula-not-decoded -->

A point in Ρ 3 can be considered a sphere of zero radius, so in the following ( x 1 , x 2 , x 3 ) 2 Ρ 3 is identified with ( x 1 , x 2 , x 3 ; 0 ) 2 Ρ 3,1 .

The tangential distance between two oriented spheres p 0 and p 1 is defined by the formula

<!-- formula-not-decoded -->

As long as η p 1 p 0 , p 1 p 0 ι 0 , (2.2) has a geometric interpretation: it is the distance between the touching points of the given spheres with a common oriented tangent plane (see Fig. 2.3). When the tangential distance is zero, we say that the two spheres are in oriented contact .

In differential geometry a canal surface is the envelope of the family of spheres x ( t ) = ( x 1 ( t ) , x 2 ( t ) , x 3 ( t ) ; x 4 ( t )) with centres at ( x 1 ( t ) , x 2 ( t ) , x 3 ( t )) and radius x 4 ( t ) . Using the above representation of spheres as points in Minkowski space, a canal surface corresponds to a curve in Minkowski space.

Remark 2.2. Cones and cylinders are canal surfaces. Both have linear spines (their axes), and while a cylinder has a constant radius function (i.e. is a pipe surface), the radius function for a cone is linear.

It follows from Rem. 2.2 that cones and cylinders correspond to lines in Minkowski space. A cylinder can be considered as a cone with apex at infinity, so in the following we write 'cones' meaning 'cylinders and cones', and use the parametrization

<!-- formula-not-decoded -->

for the line associated with the cone Χ , where p 0 and p 1 are two spheres inscribed in Χ . The apex of the cone corresponds to the point on the line with the fourth coordinate zero, i.e., the sphere with zero radius.

Remark 2.3. Not every line in Minkowski space describes a cone: the necessary and sufficient condition for a line through two points p 0 and p 1 to define a cone is η p 1 p 0 , p 1 p 0 ι &gt; 0 , i.e., d ( p 1 , p 0 ) 2 Ρ + . This means that cones in Ρ 3 correspond to hyperbolic lines in Ρ 3,1 , and the corresponding family of spheres is a hyperbolic pencil of spheres. When η p 1 p 0 , p 1 p 0 ι = 0 (i.e. the line is parabolic ) the line describes all spheres that are in oriented contact at one point, which is a parabolic pencil of spheres . The envelope of this family of spheres is a plane, which can be considered a degenerate cone with apex at the common touching point.

Figure 2.4: Projecting onto the cone: the tangent sphere in the cone and the corresponding parabolic pencil of spheres.

<!-- image -->

With the above elements of Laguerre geometry we determine the touching point of a sphere in oriented contact with a natural quadric. Two touching spheres generate a parabolic pencil of spheres (see Fig. 2.4), and the touching point is the apex of the corresponding degenerate cone (see Rem. 2.3). Note that a sphere q in oriented contact with a cone Χ is in oriented contact at a unique point, and is therefore in oriented contact with exactly one sphere p ( u 0 ) in the family associated with the cone. We find u 0 by solving the quadratic equation

<!-- formula-not-decoded -->

As the sphere q is in oriented contact with Χ at a unique point, (2.4) has a unique solution, so the discriminant of the quadratic equation is zero.

Lemma 2.4. A sphere q 2 Ρ 3,1 in oriented contact with the cone Χ associated with the line p ( u ) = p 0 + u ( p 1 p 0 ) 2 Ρ 3,1 is in oriented contact with the sphere

<!-- formula-not-decoded -->

The touching point T of q and Χ is the apex of the parabolic pencil of spheres generated by q and p ( u 0 ) :

<!-- formula-not-decoded -->

where 4 is the projection onto the fourth coordinate.

Setting the discriminant of (2.4) equal to zero gives the implicit equation for the spheres in oriented contact with the cone:

Corollary 2.5. A sphere x 2 Ρ 3,1 is in oriented contact with the cone Χ if and only if it satisfies

<!-- formula-not-decoded -->

We call the quadratic hypersurface of Ρ 3,1 defined in Cor. 2.5 the isotropic quadric of the cone Χ .

## 2.3 Rolling ball blends of plane/cone intersections

We start with the simplest non-trivial case of configurations of natural quadrics: the intersection of a cone and a plane. In general, the spine of a cone/plane blend is a conic section: ellipse, hyperbola or parabola. These blends are inherently rational: for any given plane and cone, their R -offsets intersect in a conic section, thus we can find a rational parametrization of the blend for any fixed radius R . In this section we show the details of the blend construction for elliptic blends, the results for hyperbolic and parabolic blends can be derived using the same approach. As the parametrizations of the touching curves and blending surfaces are rational, for ease of notation they are presented in projective coordinates where the first coordinate is the homogeneous coordinate. All parametrizations are in Ρ 3 or the projective space over Ρ 3 , so we distinguish projective points from affine points by the number of coordinates, and define the projection of x = ( x 0 , x 1 , x 2 , x 3 ) 2 ΡΠ 3 from projective to affine coordinates: [ x ] = ( x 1 = x 0 , x 2 = x 0 , x 3 = x 0 ) 2 Ρ 3 .

## 2.3.1 Elliptic rolling ball blend

Consider a cone Χ and a plane Π whose R -offsets Χ R and Π R intersect in the ellipse E (see Fig. 2.1) parametrized by

<!-- formula-not-decoded -->

We can assume that the ellipse is in canonical position without any loss of generality, as we can always consider the blend in the local coordinates of its spine. If the orientation of Π is given by the normal vector n =( 0,0,1 ) T , the plane is given implicitly by z = R . A classical result by G. P. Dandelin (see e.g. Theorem 4.1 in [58] or the tangent ball theorems in [42]) states that if a plane intersects an axial natural quadric in a non-degenerate conic, there are one or two spheres inscribed in the quadric and tangent to the plane, and the tangent points are the foci of the intersection conic. The inscribed tangent spheres are called focal spheres .

In the elliptic case, there are two spheres inscribed in the cone Χ R , tangent to the plane Π R at the foci of E . One sphere is resting on the xy -plane at the focus ( c , 0, 0 ) T , where c 2 = a 2 b 2 , so if its centre is at ( c , 0, v ) T then the radius of the sphere is v . If the half cone intersecting the plane is oriented outwards, the sphere is given by ( c , 0, v ; v ) . The tangential distance between the two focal spheres is 2 a and the radius of the second sphere is equal to the z -coordinate of its centre, so solving κ ( c , 0, v ; v ) ( c , 0, z ; z ) κ = 2 a for z gives us the second focal sphere c , 0, b 2 = v ; b 2 = v . The two focal spheres in Χ R are R -offsets of two spheres in Χ : p 0 =( c , 0, v ; R v ) and p 1 = c , 0, b 2 = v ; R b 2 = v . We use p 0 and p 1 to parametrize the line corresponding to the cone Χ in Minkowski space:

Figure 2.5: Touching curves on the pipe surface of the blend.

<!-- image -->

<!-- formula-not-decoded -->

Given the plane Π and the cone Χ , we want to parametrize the rolling ball blend of radius R with spine E . We start by finding the touching curves on the plane and the cone. The touching curve on the plane is a translation of the spine ellipse E along n :

<!-- formula-not-decoded -->

The rolling ball x t =( E ( t ) ; R ) is in oriented contact with the cone, so applying Lem. 2.4 to x t we find the tangent sphere p t . Let 4 be the projection onto the last coordinate. The touching point of x t and Χ is then given in Lem. 2.4:

<!-- formula-not-decoded -->

Expanding the expressions of x t and p t we find the parametrization of the touching curve on the cone:

<!-- formula-not-decoded -->

Note that Χ R is not the only cone that intersects the plane Π in the ellipse E - in fact p 0

Figure 2.6: Elliptic rolling ball blend, ( a , b , c , R , v 0 ) = ( 5,3,4,1,0.35 ) .

<!-- image -->

and p 1 generates a cone whose R -offsets go through E for any choice of v . Each of the cones has a quartic touching curve on the pipe surface (see Fig. 2.5). The pipe surface is the union of a one parameter family of circles, each circle belonging to one of the spheres in the one parameter family of spheres defining the pipe surface. A circle is a rational quadratic curve, so its parametrization by a line in parameter space is injective. By examining (2.12), we find that the circles are the isoparametric curves for t constant. T + v ( t ) is therefore an injective map onto the pipe surface, and as such a parametrization of the pipe surface. Evaluating T + v ( t ) at v = 0 we recover the ellipse in (2.10), so both of our original touching curves are in the family of curves.

Theorem 2.6. The rolling ball blend B E of radius R of the plane Π and cone Χ whose R -offsets intersects in the ellipse E in (2.8) is parametrized by

<!-- formula-not-decoded -->

where v 0 is the radius of the focal sphere of the offset cone at the positive focal point. This parametrization is rational of bi-degree ( 4,2 ) .

Remark 2.7. We have made a choice in the orientation of the plane and cone. The parametrization of the blend for other combinations of orientation is derived by changing n and/or the orientation of the focal spheres. By considering the cones whose offset cones have the opposite orientation of the family above, we find a second family of cones and quartic touching curves on the pipe surface.

The blend is a rationally parametrized surface patch of bi-degree ( 4,2 ) , which can be given in tensor product Bezier form:

<!-- formula-not-decoded -->

Figure 2.7: Elliptic rolling ball blend with control mesh.

<!-- image -->

where B 4 i ( t ) and B 2 j ( v ) are Bernstein basis functions, and Q i , j the weight points of the patch. Weight points are the rational equivalent to control points: a weight point Q i , j = w , w q i , j corresponds to a control point q i , j with weight w . A weight point with the first coordinate zero corresponds to a control point at infinity.

Consider the inner quarter of the elliptic pipe surface parametrized by B ( t , v ) with t 2 [ 1,1 ] and v 2 Ρ + . To retrieve the weight points of this surface patch, we reparametrize B ( t , v ) by replacing t with ˜ t = ( t + 1 ) = 2 and v with ˜ v = v = ( b + v ) to send the parameter intervals to [ 0,1 ] . Achange of polynomial basis from monomial to Bernstein basis then gives us the following weight points:

<!-- formula-not-decoded -->

where e i are unit coordinate vectors and e 0 the homogeneous coordinate.

The weight points of the rolling ball blend of radius R of the plane Π and cone Χ are found by subdividing Q at ˜ v 0 = v 0 = ( v 0 + b ) . In Fig. 2.7, we see a quarter of the blend, together with its control mesh.

## 2.3.2 Hyperbolic and parabolic blends

The above construction of the elliptic rolling ball blend applied to hyperbolic and parabolic blends gives us the parametrizations for these cases. Consider the radius R blend of a cone and a plane, with hyperbolic spine

<!-- formula-not-decoded -->

By parametrizing the cone by the offsets of the focal spheres of the spine

<!-- formula-not-decoded -->

Figure 2.8: Hyperbolic and parabolic rolling ball blends, parameters respectively ( a , b , c , R , v 0 ) = ( 0.4,0.3,0.5,0.2,0.3 ) and ( a , R , v 0 ) = ( 1,1.5,0.7 ) .

<!-- image -->

we find the parametrization of the blend:

Theorem 2.8. Let B H be the plane/cone rolling ball blend of radius R with spine H . Then the blend is parametrized by

<!-- formula-not-decoded -->

t 2 Ρ , v 2 [ 0, v 0 ] , where v 0 is the radius of the focal sphere of the spine at the positive focal point.

Consider the radius R blend of a cone and a plane, with parabolic spine

<!-- formula-not-decoded -->

By parametrizing the cone by the offsets of the focal sphere of the spine, and the offset of the apex of the offset of the cone.

<!-- formula-not-decoded -->

we find the parametrization of the blend:

Theorem 2.9. Let B P be the plane/cone rolling ball blend of radius R with parabolic spine P . Then B P is parametrized by

<!-- formula-not-decoded -->

t 2 Ρ , v 2 [ 0, v 0 ] where v 0 is the radius of the focal sphere of the spine.

Remark 2.10. The pipe surfaces in the above illustrations of quadratic rolling ball blends have no self-intersections. However, if we increase the radius R of the pipe surface, this may no longer be the case. The hyperbolic blend in Fig. 2.8 is smooth, but as the radius of the blend increases, a self-intersection will appear below the apex of the cone.

## 2.3.3 Parametrizing cone/cone blends using the families of touching curves

From the above derivation of the parametrization of the plane/cone blend, it is tempting to use the same approach to parametrize the blend of two cones in the same family (see Rem. 2.7). For a given radius R it is indeed possible to blend two such cones whose R -offsets intersect in a given conic, however this is the unique radius for which the offsets of the two cones intersect in a conic section. The two offset cones circumscribe a common sphere (the condition for having a conic intersection), but when the cones are from the same family the two spheres have opposite orientations, so their offsets have no common spheres.

In this section we have considered only one family of cones through a conic. By changing the orientation of the focal sphere(s) of the intersection, we get the second family of cones, and two cones from different families do indeed have a common oriented sphere, so the intersection of the offsets of the cones will remain a conic. As the cones belong to different families, we cannot get the blend by moving seamlessly from one touching curve to the other as we did here, but we will use the touching curves and methods from canal surfaces to parametrize the blend.

## 2.4 Parametrizing rational patches on pipe surfaces

A rational pipe surface of radius R has a parametrization of the form

<!-- formula-not-decoded -->

where s ( t ) is its spine, R its radius, and N ( t , u ) the Gaussian map of F ( t , u ) (i.e. the image of the pipe surface on the Gaussian sphere). When we construct a rolling ball blend of two surfaces, the blend is the patch on the pipe surface between the two touching curves traced by the rolling ball. Likewise the touching curves can be traced on the rolling ball, giving us a patch on the Gaussian sphere S . Parametrizing the blend is thus reduced to parametrizing the patch on S limited by the Gaussian images 0 ( t ) and 1 ( t ) of the touching curves. [29] takes advantage of this to formulate an algorithm for parametrizing tensor product Bézier patches on canal surfaces. In this section we will present a simplified algorithm for the case of pipe surfaces.

The isoparametric curve F t ( u ) on the pipe surface is an arc of a circle in the intersection with the normal plane of the spine curve. Its Gaussian image N t ( u ) is the arc of the large circle between 0 ( t ) and 1 ( t ) in the intersection of the Gaussian sphere with the plane Bx T = 0 , where B = ( 0, ( t ) ˙ s 1 ( t ) , ( t ) ˙ s 2 ( t ) , ( t ) ˙ s 3 ( t )) and x = ( 1, x 1 , x 2 , x 3 ) . ˙ s ( t ) is the tangent vector of the spine, and ( t ) the common denominator of ˙ s 1 ( t ) , ˙ s 2 ( t ) , and ˙ s 3 ( t ) .

The Gaussian sphere can be parametrized using the generalized stereographic projection P S , mapping points in ΡΠ 3 onto the sphere ([19]). If we identify the four homogeneous coordinates of a point in ΡΠ 3 with a point in Χ 2 , we get a compact formulation in complex numbers:

<!-- formula-not-decoded -->

This is also known as the universal rational parametrization of the sphere (see e.g. [29]).

Remark 2.11. We can also parametrize the Gaussian sphere by the stereographic projection from the real plane Ρ 2 onto S . However, the inverse stereographic projection maps circles on S onto both circles and lines in Ρ 2 . This is impractical as we want to recover the parametrizations of circles on S from the parametrizations of their projections in the plane. On the other hand, any circle on S is the projection of a line in Χ 2 by the generalized stereographic projection ([29], Lemma 2). Furthermore, the universality of this parametrization ensures that its degree is minimal. The level of abstraction of the parametrization construction is increased as it takes place in Χ 2 before it is projected onto the Gaussian sphere. However, the end results are the compact explicit parametrizations of the rolling ball blends given in Thms. 2.16-2.22.

We define the lifting of a curve x ( t ) = ( x 0 , x 1 , x 2 , x 3 ) 2 ΡΠ [ t ] 3 on the Gaussian sphere as

<!-- formula-not-decoded -->

(see Eq. 10 in [29]). As x ( t ) lies on the Gaussian sphere it satisfies x 2 0 x 2 3 = x 2 1 + x 2 2 , i.e., ( x 0 + x 3 ) ( x 0 x 3 ) = ( x 1 + i x 2 ) ( x 1 i x 2 ) , so x 1 + i x 2 divides U 0 ( x 0 x 3 ) and L ( x ( t )) is polynomial rather than rational. As long as the projective representation of the curve is of minimal degree, i.e., gcd ( x 0 , x 1 , x 2 , x 3 ) = 1 , we also have gcd ( L ( x ( t ))) = 1 . We find that L sends a point x ( t ) to a point in its pre-image by P S , as P S ( L ( x ( t ))) = x ( t ) . Note that the lifting is well defined outside the constant curves x ( t ) = ( 1,0,0, 1 ) (e.g. the Gaussian image of the touching curve on a horizontal plane). In this case we perform a change of coordinates to move the constant curve away from these exceptional points.

Consider the lifting of N t ( u ) in Χ [ t , u ] 2 . By parametrizing the lifting and then applying P S , we recover a parametrization of N ( t , u ) . Furthermore, the universality of P S means that if we minimize the bi-degree of the lifting, the parametrization will likewise have minimal bi-degree.

Lemma 2 in [29] gives the lifting of N t ( u ) as the line between the liftings X and Y of 0 ( t ) and 1 ( t ) parametrized by ( 1 u ) 0 X + u 1 Y where the lifting coefficient 0 1 is the unique solution of a set of linear equations. In the case of pipe surfaces, the system is reduced to a single equation:

Lemma 2.12. The system of linear complex equations determining the lifting coefficient = 0 1 2 Χ [ t ] has a unique solution up to multiplication by a real number:

<!-- formula-not-decoded -->

6

if 0 ( t ) = 1 ( t ) . Otherwise i ( t ) = ( 0 , 1 , 2 , 3 ) and

<!-- formula-not-decoded -->

Here X 0 = gcd 0 0 + 0 3 , 0 1 + i 0 2 , Y 0 = gcd 1 0 + 1 3 , 1 1 i 1 2 , and B = ( 0, B 1 , B 2 , B 3 ) = ( 0,˙ s 1 ( t ) , ˙ s 2 ( t ) , ˙ s 3 ( t )) .

Proof. In Lemma 2 in [29], the lifting coefficient is determined by the linear system of complex equations

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where X =( X 0 , X 1 ) T = L 0 ( t ) and Y =( Y 0 , Y 1 ) T = L 1 ( t ) are the lifting of the touching curves on the Gaussian sphere, and B = ( 0, B 1 , B 2 , B 3 ) = ( 0,˙ s 1 ( t ) , ˙ s 2 ( t ) , ˙ s 3 ( t )) defines the plane whose intersection with the Gaussian sphere contains the isoparametric curve N t ( u ) . (2.29) is the complex conjugate of (2.28), so it can be eliminated. By construction of the lifting we have

<!-- formula-not-decoded -->

and by inserting these equations into the remaining three equations in the system above, and separating the real and imaginary part of (2.28) ((2.27) and (2.30) are purely imaginary), we arrive at a system of four real linear equations in X = Re X 0 Y 0 and Y = Im X 0 Y 0 :

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

6

Recall that 0 and 1 lie on the plane Bx T = 0 , and so B 0 = 0 and B 1 = 0 . This gives two linear equations in B i , which are linearly independent if and only if 0 = 1 . We can then express B 1 and B 2 as functions of B 3 :

<!-- formula-not-decoded -->

and by equation (2.35), we have B 3 = 2 Y . By replacing the B i s in (2.32-2.35) by their expressions as functions of Y , we arrive at three homogeneous polynomials in X and Y .

where 0 = 0 0 + 0 3 and 1 = 1 0 + 1 3 .

Closer examination reveals that the equations are in fact equal, giving us

<!-- formula-not-decoded -->

which proves (2.25). Note that X + i Y is unique up to multiplication with a real number, which is what we required for . As

<!-- formula-not-decoded -->

is a polynomial.

When i ( t ) = ( 0 , 1 , 2 , 3 ) , (2.32-2.35) are reduced to

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The determinant of the corresponding matrix is zero, so the system is linearly dependent and we may chose to solve (2.39) and (2.40) to find X and Y . After some simplifications, this gives us

<!-- formula-not-decoded -->

where is a polynomial by the same argument as before. This proves (2.26).

The procedure outlined above is summarized in the following algorithm:

Algorithm 2.13. Consider two surfaces whose R -offsets intersect in the rational curve s ( t ) , and let T i ( t ) be the rational touching curves on the two surfaces, of the pipe surface with spine s ( t ) and radius R . The rolling ball blend of radius R of these two surfaces is parametrized by executing the following steps:

1. The Gaussian images 0 ( t ) and 1 ( t ) of the touching curves

<!-- formula-not-decoded -->

2. The liftings of the Gaussian touching curves

<!-- formula-not-decoded -->

3. The lifting coefficient = 0 1 2 Χ [ t ] from Lem. 2.12. Note that is unique up to multiplication by a real scalar, so any real factors may be eliminated.
4. The lifting of the blend in Χ [ t ] 2

<!-- formula-not-decoded -->

where = 0 1 , and 0 X and 1 Y are of equal degree (or as close as possible).

## 5. The parametrization of the blend

<!-- formula-not-decoded -->

where P S is the generalized stereographic projection in (2.23) .

This parametrization is of minimal bi-degree ( n , 2 ) .

## 2.5 Classification of pairs of natural quadrics

Before we apply Alg. 2.13 to cone/cone and cone/sphere blends, we need to determine which configurations of natural quadrics can be blended rationally. A canal surface is rational if its spine and radius function are rational [45]. The pipe surface of a fixed radius rolling ball blend has a constant radius function, so two surfaces can be blended rationally at a fixed radius if their R -offsets intersect in a rational spine curve. We say that a configuration of two surfaces is rationally stable if this is true for any R . The intersection of two quadratic surfaces is a quartic curve, which is not in general rational. If the quartic is reducible, the intersection is a combination of lines, conics, and space cubics, all of which are rational. If the quartic is irreducible, it is rational if and only if it is singular. Pairs of natural quadrics have been completely classified from the point of view of Laguerre geometry in [27], which also gives rational parametrizations of their bisector surfaces in Minkowski space. By intersecting the bisector surface with the hyperplane x 4 = R we get the spine of the rolling ball blend of radius R .

Remark 2.14. The bisector surface of two natural quadrics is always rational. However, the hyperplane sections of this surface are not necessarily so. There are only two types of algebraic surfaces whose hyperplane sections are always rational: rational ruled surfaces and the quartic Steiner surface (see [43]).

There are two configurations of cones where the intersection of the hyperplane x 4 = R with the bisector surface is always rational: cones with one or two touching points (when the bisector surface is respectively the Steiner quartic or a pair of planes). A touching point is a point where the cones are in oriented contact. When the cones have two touching points, their intersection is a pair of quadratic curves. If the intersection is degenerate, i.e., contains one or more lines, the linear components may be blended by a cylinder. Cylindrical (and torical, corresponding to circular intersections) blends are already in use in CAD, so in the following we will assume that the intersection of two cones with two touching points has two non-degenerate quadratic components, which are blended separately.

When the cones have one touching point, their intersection is a quartic curve with a singularity at the touching point. Consider the two cones Π and Θ , which correspond to the lines

<!-- formula-not-decoded -->

in Minkowski space. If Π and Θ are in oriented contact, exactly one sphere q ( v 0 ) is in oriented contact with Π , i.e., q ( v 0 ) is in the isotropic quadric of Π defined in Cor. 2.5.

We find v 0 by solving η q ( v ) p 0 , p 01 ι 2 κ q ( v ) p 0 κ 2 κ p 01 κ 2 = 0 , where p 01 = p 1 p 0 and q 01 = q 1 q 0 :

<!-- formula-not-decoded -->

As v 0 is unique we find

<!-- formula-not-decoded -->

and q ( v 0 ) = q 0 + v 0 q 01 . Likewise we find the sphere p ( u 0 ) in oriented contact with Θ . These two spheres are in oriented contact, and lets us categorize the rationally stable configurations of two cones:

Theorem 2.15. Two cones Π and Θ are in oriented contact if the discriminant of (2.48) is zero. Let p ( u 0 ) and q ( v 0 ) be the spheres in the cones in oriented contact. Then the intersection of Π and Θ is rationally stable, and is categorized by comparing the radii of p ( u 0 ) and q ( v 0 ) :

1. 4 ( p ( u 0 )) = 4 ( q ( v 0 )) : the cones have two touching points, and the intersection is a pair of quadratic curves.
2. 4 ( p ( u 0 )) 4 ( q ( v 0 )) &lt; 0 : the cones intersect in an isolated point.

6

3. 4 ( p ( u 0 )) = 4 ( q ( v 0 )) and 4 ( p ( u 0 )) 4 ( q ( v 0 )) 0 : the cones have one touching point at the apex of the parabolic pencil of spheres generated by p ( u 0 ) and q ( v 0 ) , and intersect in a quartic curve with a singularity at the touching point.

Likewise there is one rationally stable configuration of a cone and a sphere: when they have one touching point. This is the case if the sphere belongs to the isotropic quadric of the cone, as determined in Cor. 2.5, and not to the cone itself.

## 2.6 Rolling ball blends of rationally stable pairs of natural quadrics

The classification in Thm. 2.15 gives us the two rationally stable configurations of cones: the intersection curve is either a pair of quadratic curves or a singular quartic. We also have a rationally stable configuration of a cone and a sphere with one touching point, intersecting in a singular quartic curve. In this section we will apply Alg. 2.13 to these three cases to parametrize their fixed radius rolling ball blends.

## 2.6.1 Quadratic cone/cone blends

Using the cone/cone blend with an elliptic spine as an example, we show how Alg. 2.13 is applied.

We start by calculating the Gaussian images of the touching curves: In Sec. 2.3 we determined the touching curves for one family of cones in oriented contact with an elliptic pipe surface, giving the touching curves T + v ( t ) in (2.12). A cone in this family is parametrized by the spheres

<!-- formula-not-decoded -->

A cone from the other family in oriented contact with the pipe surface is parametrized by the spheres

<!-- formula-not-decoded -->

The touching curve on this cone is

<!-- formula-not-decoded -->

Then the Gaussian images of T + v ( t ) and T v ( t ) are

<!-- formula-not-decoded -->

Note that v is fixed for each of the cones. Let v 0 (resp. v 1 ) be the value associated with the cone with touching curve 0 = + v 0 (resp. 1 = v 1 ).

The second step is to calculate the liftings of 0 and 1 . In the elliptic case, we find X =( X 0 , X 1 ) = L 0 :

<!-- formula-not-decoded -->

and Y =( Y 0 , Y 1 ) = L 1 :

<!-- formula-not-decoded -->

We then calculate the lifting coefficient

<!-- formula-not-decoded -->

and factor to retrieve 0 and 1

<!-- formula-not-decoded -->

Combining X , Y , 0 , and 1 , we calculate the lifting ( 1 u ) 0 X + u 1 Y of the arc of the circle N t ( u ) between 0 ( t ) and 1 ( t ) on the Gaussian sphere:

<!-- formula-not-decoded -->

By applying P S to the lifting, we get the parametrization of the patch on the Gaussian sphere. We summarize the results of these calculations in the following theorem:

Theorem 2.16. Let Π and Θ be two cones whose R -offsets intersect in the ellipse E . If the offset cones belong to opposite families, the rolling ball blend of radius R of Π and Θ has a minimal rational parametrization B E ( t , u ) given by

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

The parametrization has minimal bi-degree ( 6,2 ) .

Proof. X , Y and i are of degree 1 , so ( 1 u ) 0 X + u 1 Y is of bi-degree ( 2,1 ) , and P S (( 1 u ) 0 X + u 1 Y ) of bi-degree ( 4,2 ) . Adding the rational function E ( t ) of bi-degree ( 2,0 ) gives us a parametrization of bi-degree ( 6,2 ) .

In order to parametrize the blend in the hyperbolic case, consider a cone in the family described in Sec. 2.3.2, parametrized by the spheres

<!-- formula-not-decoded -->

A cone in the opposite family is parametrized by

<!-- formula-not-decoded -->

Applying the algorithm, we find

Theorem 2.17. Let Π and Θ be two cones whose R -offsets intersect in the hyperbola H . If the offset cones belong to opposite families, the rolling ball blend of radius R of Π and Θ has a minimal rational parametrizationgiven by

<!-- formula-not-decoded -->

Figure 2.9: Elliptic, hyperbolic and parabolic rolling ball blends of two cones. Parameters for the parabolic blends respectively ( a , R , v 0 , v 1 ) = ( 2,2,2,5 ) and ( a , R , v 0 , v 1 ) = ( 4,2,2,5 ) .

<!-- image -->

where

<!-- formula-not-decoded -->

The parametrization has minimal bi-degree ( 6,2 ) .

In the parabolic case, consider a cone in the family described in Sec. 2.3.2, parametrized by the spheres

<!-- formula-not-decoded -->

A cone in the opposite family is parametrized by

<!-- formula-not-decoded -->

Applying the algorithm, we find

Theorem 2.18. Let Π and Θ be two cones whose R -offsets intersect in the parabola P . If the offset cones belong to opposite families, the rolling ball blend of radius R of Π and Θ has a minimal rational parametrization B P ( t , u ) given by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where

The parametrization has minimal bi-degree ( 6,2 ) .

## 2.6.2 Quartic cylinder/cylinder blends

By a careful selection of the local coordinate system of the quartic spine, we can also give the general parametrization of the blend of two cylinders with one touching point (see the supplementary materials for the construction of the parametrization of the spine). Let Π and Θ be two cylinders whose R -offsets have a common touching point in the origin, such that the common tangent plane in that point is z = 0 oriented in the positive z -direction. The offset cylinders are parametrized by the spheres p 0 =( a , b , r 0 , r 0 ) and p 1 =( 0,0, r 0 , r 0 ) , and q 0 =( 1,0, r 1 , r 1 ) and q 1 =( 0,0, r 1 , r 1 ) , and without loss of generality we can assume that r 1 r 0 0 , b &gt; 0 , and a 2 + b 2 = 1 .

The intersection curve of the two offset cylinders can then be parametrized by

<!-- formula-not-decoded -->

By applying Alg. 2.13, we find the touching curves 0 and 1 on the Gaussian sphere

<!-- formula-not-decoded -->

and the parameters of the blend:

Theorem 2.19. Let Π and Θ be two cylinders whose R -offsets have a common touching point in the origin and common tangent plane z = 0 , as given above. The rolling ball blend of radius R of Π and Θ has a minimal rational parametrization

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

The parametrization has minimal bi-degree ( 12,2 ) .

Remark 2.20. Quartic cylinder blends occur e.g. in corner blends. Consider the corner in Fig.2.10, where three planes intersect at right angles. Each of the three edges of the corner are blended by cylinders of different radii. The corner itself is blended in two patches, using the smallest of the cylinder radii to blend the largest cylinder with the middle cylinder, and with the plane between the middle and the smallest cylinder. The latter is a patch on a torus, the cylinder/cylinder blend is a quartic patch as described above.

Figure 2.10: Rolling ball blend of two cylinders with one common touching point ( a , b , r 0 , r 1 , R ) = ( 0.5, π 1 0.5 2 , 0.7, 1, 0.3 ) , and corner blend with three different edge radii.

<!-- image -->

## 2.6.3 Quartic cone/cone blends

Using the same approach as above, it is possible to parametrize the intersection of two cones with one touching point (see the supplementary materials for the construction of the parametrization of the spine). However, the lack of symmetry in cone/cone configurations with only one touching point drastically increases the length of the expressions. Let Π and Θ be two cones whose R -offsets have a common touching point in the origin, such that the common tangent plane in that point is z = 0 oriented in the positive z -direction. The offset cones are parametrized by the spheres p 0 = ( 0,0, r , r ) and p 1 = ( a , b , 0, 0 ) , and q 0 = ( 0,0, 1, 1 ) and q 1 = ( A , 0, 0, 0 ) , and without loss of generality we can assume that 0 &lt; r &lt; 1 , b &gt; 0 , and A &gt; 0 . The intersection curve of the two offset cones can then be parametrized by

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

Figure 2.11: Rolling ball blend of two cones with one common touching point (parameters ( a , b , A , r , R ) = ( 3,4,5,0.75,0.25 ) ).

<!-- image -->

By applying Alg. 2.13, we find the parametrization of the fixed radius rolling ball blend of the two cones:

Theorem 2.21. Let Π and Θ be two cones whose R -offsets have a common touching point in the origin and common tangent plane z = 0 , as given above. The rolling ball blend of radius R of Π and Θ has a minimal rational parametrization

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

The parametrization has minimal bi-degree ( 12,2 ) .

Proof. Xi and Yi are of degree two in t , and since is of degree four, 0 and 1 of degree two in t . Thus the lifting ( 1 u ) 0 X + u 1 Y is of degree four in t and one in u . As P S doubles the degrees, and the spine of the blend is also of degree four in t , this gives the blend B ( t , u ) the minimal bi-degree ( 12,2 ) .

Figure 2.12: Sphere/cylinder blend (parameters ( r , R ) = ( 0.7,0.5 ) ), and its Viviani type quartic spine curve.

<!-- image -->

<!-- image -->

## 2.6.4 Quartic sphere/cylinder blends - Viviani's Curve

A classic example of a sphere/cone intersection is Viviani's Curve - the intersection of a sphere and cylinder with one touching point, where the radius of the cylinder is half of the radius of the sphere (see Fig. 2.12). Consider the sphere Σ =( 0,0, 1; R 1 ) and the cylinder parametrized by the two spheres p 0 = ( 0,0, r ; R r ) and p 1 = ( 1,0, r ; R r ) , where 0 &lt; r &lt; 1 . Then the spine curve for the rolling ball blend of fixed radius R is parametrized by

<!-- formula-not-decoded -->

using the same approach as in the parametrization of the singular quartic intersection of two cylinders. By applying Alg. 2.13, we parametrize the rolling ball blend with spine s ( t ) :

Theorem 2.22. Consider a sphere and a cylinder whose R -offsets intersect in the quartic curve s ( t ) . The rolling ball blend of radius R of the intersection has a minimal rational parametrization B ( t , u ) given by

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The parametrization has minimal bi-degree ( 8,2 ) .

Proof. X and Y are of degree 2 in t , and of degree 1 , so ( 1 u ) 0 X + u 1 Y is of bi-degree ( 3,1 ) , and P S (( 1 u ) 0 X + u 1 Y ) of bi-degree ( 6,2 ) . Adding the rational function s ( t ) of bi-degree ( 4,0 ) gives us a parametrization of bi-degree ( 10,2 ) . However, 1 + t 2 is a factor of the denominators of both s ( t ) and P S (( 1 u ) 0 X + u 1 Y ) , so the minimal bi-degree of the blend is reduced to ( 8,2 ) .

The general case of sphere/cone blends can be calculated following Alg. 2.13.

## 2.7 Conclusions

Four configurations of natural quadrics have fixed radius rolling ball blends with rational parametrizations: plane/cone intersections, two cones with one or two touching points, and cone/sphere intersections with one touching point. We have presented closed expressions for the rational parametrizations of the fixed radius rolling ball blends in these cases, as well as a general algorithm for rational parametrizations of fixed radius rolling ball blends. The blends are all of minimal bi-degree.

Further research will investigate parametrizations of variable radius rolling ball blends, as well as volumetric representations of blends in e.g. isogeometric analysis.

## Acknowledgements

This paper has been partially financed by the Marie-Curie Initial Training Network SAGA (ShApes, Geometry, Algebra), FP7-PEOPLE contract PITN-GA-2008-214584.

## Piecewise rational parametrizations of canal surfaces

Heidi E. I. Dahl

Published in Mathematical Methods for Curves and Surfaces [14]

## Abstract

Canal surfaces, as envelopes of one-parameter families of spheres, correspond to curves in Minkowski space. We show that the continuity properties of a canal surface are inherited from the continuity properties of the associated curve, i.e., two curves joined with G 1 or G 2 continuity in Minkowski space correspond to two canal surfaces joined with the same level of continuity. We also describe an algorithm for minimal bi-degree rational parametrizations of patches on canal surfaces, and show how this can be used to parametrize piecewise rational corner and edge blends.

## 3.1 Introduction

In Computer Aided Design (CAD) complex shapes are constructed from a small set of simple primitives. To a large extent, and in particular in the design of mechanical parts, these primitive shapes are planes, the natural quadrics, and rolling ball blends between them. The natural quadrics (spheres, and right circular cylinders and cones) are rational surfaces with rational offsets, however, a rolling ball blend between two natural quadrics is not necessarily rational. In current CAD systems they are therefore constructed by approximation in all but the simplest cases, e.g., where the blend is a patch on a cylinder or torus.

Although shape accuracy is important in current CAD systems there is no requirement that adjacent surfaces match exactly, so gaps within fine tolerances are allowed. However, with the introduction of Isogeometric Analysis (IGA) (see, e.g., [13]) this changes, as in Finite Element Analysis (FEA) adjacent elements are required to match exactly. As a result of this, there is a renewed interest in exact rational parametrizations of curves and surfaces. The parametrization degrees of these exact surfaces will necessarily be higher than for approximative blends, but this disadvantage is offset by the possibility of constructing watertight patchworks of rational surfaces where the limiting curves of two adjacent patches match ex- actly. For rolling ball blends, we can then construct an exact rational parametrization of the blending surface, such that its limiting curves are contained in the two original surfaces.

Rolling ball blends are patches on canal surfaces , which are defined as envelopes of oneparameter families of spheres. Such a family can be described as a curve in the 4 -dimensional Minkowski space Ρ 3,1 , where a point ( s ; r ) corresponds to a sphere with centre s and radius r (the sign of the radius determining the orientation of the sphere). The properties of a canal surface can therefore be completely defined in terms of the properties of the corresponding curve in Ρ 3,1 .

In this paper we will examine the differential geometry of canal surfaces. In particular, we will show that if two curves in Minkowski space meet with G 1 or G 2 continuity (extending the differential geometry of curves in Ρ 3 to Ρ 3,1 ), the join of the two canal surface inherits the same degree of continuity.

In a previous paper (see [17]), we have classified and parametrized rational fixed radius rolling ball blends of pairs of natural quadrics, and established a lower bound on the bidegree of their parametrizations. By a slight modification of the parametrization algorithm, it extends to variable radius rolling ball blends. Using piecewise rational rolling ball blends of edges and corners, we extend the range of configurations that can be blended rationally at a relatively low degree, and give designers an added flexibility in creating rational blends.

We start by deriving the algorithm for minimal bi-degree rational parametrizations of canal surfaces in Sec. 3.2. In Sec. 3.3 we extend the differential geometry of Ρ 3 to Ρ 3,1 , and in Sec. 3.4 we describe properties of canal surfaces in terms of the properties of the associated curves in Ρ 3,1 . Finally, in Sec. 3.5 we demonstrate two approaches to corner blends using piecewise rational rolling ball blends, applying the parametrization algorithm from Sec. 3.2 and the continuity results from the following sections.

## 3.2 Rational parametrizations of canal surfaces

A canal surface is defined as the envelope of the family of spheres

<!-- formula-not-decoded -->

The curve s ( t ) traced by the centres of the spheres is called the spine curve and r ( t ) the radius function of the canal surface. A canal surface has a parametrization on the form

<!-- formula-not-decoded -->

where the isoparametric curve F t ( u ) for a given t is a circle, known as a characteristic circle . In fact, N t ( u ) is a circle on the unit sphere. An algorithm for minimal bi-degree rational parametrizations of N ( t , u ) can be found in [29]. Thus if f ( t ) is rational, we can also construct a rational parametrization of the canal surface F ( t , u ) (this is proved in, e.g., Theorem 5.1 of [45]). Furthermore, N ( t , u ) is the unit normal vector of the canal surface, so F ( t , u ) is a rational surface with a rational unit normal vector field, i.e., it is a Pythagorean Normal (PN) surface. PN surfaces have rational offsets, so by considering canal surfaces corresponding to rational curves in Ρ 3,1 , we obtain a class of rational rolling ball blends that have some of the advantages of the natural quadrics.

Remark 3.1. There are non-rational curves in Ρ 3,1 whose canal surfaces can be parametrized rationally. However, for these rational parametrizations the rationality of the unit normal vector field is no longer automatic.

## 3.2.1 Arcs of circles on the unit sphere

The parametrization of canal surfaces has thus been reduced to the parametrization of circles on the unit sphere, and the parametrization of rolling ball blends to the parametrization of the isoparametric arcs of circles N t ( u ) on the unit sphere.

In [17], we considered this problem for pipe surfaces , i.e., canal surfaces with constant radius function. As well as closed expressions for the minimal bi-degree rational parametrizations of the fixed radius blends, in the cases where such a blend exists, we presented a general parametrization algorithm for patches on pipe surfaces. Only a slight modification of the algorithm is necessary to extend it to variable radius blends.

The parametrization algorithm is a simplification of the results presented in [29], where minimal degree parametrizations are described both for patches and for the complete canal surface. As mentioned above, the decomposition of the parametrization has reduced the problem to the parametrization of a specific arc of circle on the unit sphere. A first naive approach would be to use the inverse stereographic projection to send the arc of circle to a line segment or an arc of circle in the plane Ρ 2 and then parametrize the image. However, this would result in a parametrization of a relatively high degree, as in most cases we parametrize a curve of degree 2 in the plane before projecting it back onto the sphere. This raises the question of whether we can find a projection that sends arcs of circles on the unit sphere onto line segments, and which increases the parametrization degree as little as possible.

The answer is generalized stereographic projection P S . In [19], P S is defined as a map from ΡΠ 3 to the unit sphere, and in [29] this is reformulated, by identifying Ρ 4 with Χ 2 , as the universal rational parametrization of the unit sphere:

<!-- formula-not-decoded -->

where U = ( U 0 , U 1 ) 2 Χ 2 . This expression is homogeneous: P S ( U ) = ϕ ϕ 2 P S ( U ) , so the generalized stereographic projection can be interpreted as a map from the complex projective line ΧΠ 1 to the unit sphere in Ρ 3 (after projection into affine coordinates).

Remark 3.2. If we restrict the domain of the generalized stereographic projection to the unit sphere in Ρ 4 , P S is called the Hopf map .

We consider a line in ΧΠ 1 as the interpolation of two complex projective points using a real parameter. Then P S sends a line in ΧΠ 1 onto a circle on the unit sphere. More importantly, any circle Χ on the unit sphere can be lifted onto a line Λ in its preimage in ΧΠ 1 so that P S ( Λ ) = Χ .

Remark 3.3. One of the advantages of the generalized stereographic projection is that unlike the stereographic projection, it does not have a distinguished point on the sphere where it is not defined.

However, the choice of lifting distinguishes one point. For arcs of circles ending in this point, we need to chose a different lifting, e.g., by a circular permutation of the coordinates.

The algorithms in [17] and [29] describe how to lift the endpoints ( t ) and ( t ) of an arc of circle on the unit sphere to X ( t ) and Y ( t ) in ΧΠ 1 . The lifting of a point x ( t ) = ( x 0 ( t ) , x 1 ( t ) , x 2 ( t ) , x 3 ( t )) 2 ΡΠ 3 is defined as

<!-- formula-not-decoded -->

(see Equation 10 in [29]). They then parametrize a line segment between X ( t ) and Y ( t ) in such a way that the projection of the line segment onto the unit sphere, using the generalized stereographic projection, is the original arc. As the generalized stereographic projection is also the universal rational parametrization of the sphere, if we minimize the parametrization degree of the line in ΧΠ 1 , its projection on the unit sphere has minimal parametrization degree.

Remark 3.4. Given two points X , Y 2 ΧΠ 1 , the line between them is unique up to a complex scalar . The choice of determines which arc of circle between P S ( X ) and P S ( Y ) the line segment is projected onto. As part of the parametrization algorithm, we therefore have to determine the correct lifting coefficient .

The parametrization of arcs of circles on the unit sphere is summarized in the following algorithm:

Algorithm 3.5. An arc of circle on the unit sphere is parametrized by executing the following steps:

1. Lift the endpoints ( t ) , ( t ) 2 ΡΠ [ t ] 3 , i = 1,2 , to X ( t ) , Y ( t ) 2 ΧΠ 1 [ t ] .
2. Determine the lifting coefficient ( t ) , and factorize ( t ) = 0 ( t ) 1 ( t ) as evenly as possible in order to distribute the increase in degree across X ( t ) and Y ( t ) .
3. Parametrize the line segment ( 1 u ) 0 ( t ) X ( t )+ u 1 ( t ) Y ( t ) , u 2 [ 0,1 ] between X ( t ) and Y ( t ) .
4. Project onto the unit sphere using the generalized stereographic projection P S .

Steps 1, 3, and 4 are identical to the corresponding steps in the algorithm for the parametrization of pipe surfaces (Alg. 13, [17]). However, while the isoparametric circles on pipe surfaces correspond to large circles on the unit sphere, this is not the case for canal surfaces in general. We therefore have to adjust accordingly.

## 3.2.2 Parametrizing variable radius rolling ball blends

The lifting coefficient is defined in [29] as the unique solution of a set of linear equations. In [17] we reduced this to a single equation for the case of pipe surfaces. We can make similar simplifications for canal surfaces in general.

Figure 3.1: Rolling ball blends of two cones.

<!-- image -->

Lemma 3.6. The system of linear complex equations determining the lifting coefficient = 0 1 2 Χ [ t ] has a unique solution up to multiplication by a real number. If [ ( t )] = [ ( t )] , where [ ( t )] = ( 1 = 0 , 2 = 0 , 3 = 0 ) is the projection from ΡΠ 3 to Ρ 3 , then

6

<!-- formula-not-decoded -->

Otherwise the two points on the unit sphere are diametrically opposite, i.e., ( t ) = ( 0 , 1 , 2 , 3 ) , ( t ) = ( 0 , 1 , 2 , 3 ) , and

<!-- formula-not-decoded -->

Here X 0 = gcd ( 0 + 3 , 1 + i 2 ) , Y 0 = gcd ( 0 + 3 , 1 i 2 ) , and

<!-- formula-not-decoded -->

where ( t ) is the common denominator of ˙ r ( t ) , ˙ s 1 ( t ) , ˙ s 2 ( t ) , and ˙ s 3 ( t ) .

Remark 3.7. The arc of circle is contained in the intersection of the unit sphere with the plane B . x = 0 , x =( x 0 , x 1 , x 2 , x 3 ) .

Proof. In [29], is given as the unique solution of a set of four complex linear equations. The main challenge in deriving (3.5) is the size of the expressions, so we will only give an outline of the calculations here.

Starting from the four complex equations, we consider their real and imaginary components. We solve two of these real equations for the real and imaginary parts of . We can then show that the remaining equations are equivalent. The expression for is then simplified exploiting the fact that the endpoints [ ( t )] and [ ( t )] are contained in the intersection of the plane B . x = 0 with the unit sphere. Eliminating any real factors, we arrive at (3.5).

This completes the parametrization algorithm for arcs of circles on the unit sphere.

Figure 3.2: Linear family generated by two spheres in oriented contact (left), and the rolling ball in oriented contact with a cone (right).

<!-- image -->

The first step in constructing a parametrization of a blend, such as the two cone-cone blends in Fig. 3.1, is to parametrize the touching curves : the curves the rolling ball trace on the two natural quadrics, i.e., the limiting curves of the patch on the canal surface. Consider two natural quadrics, and a variable radius rolling ball blend between them. Let f ( t ) 2 Ρ 3,1 be the curve in Minkowski space corresponding to the blend. How we determine the touching curves in the original surfaces depends on the type of the natural quadric.

To find the touching curve on a sphere, recall that the sphere and the rolling ball is in oriented contact for any t , i.e., the two spheres are tangent, and their unit normal vectors coincide at the touching point. Equivalently, the two corresponding points p and q 2 Ρ 3,1 are at zero distance, measured using the Minkowski metric

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

is the Minkowski scalar product . We use the notation ( \_ , \_ ) and ϕ \_ ϕ for the Euclidean scalar product and metric, respectively. The Minkowski metric measures the tangential distance between two oriented spheres. If we consider the linear family of spheres generated by the sphere and the rolling ball, the touching point is the member of the family with zero radius. To find the touching curve on the sphere, it therefore suffices to solve a linear equation, and we find that if f ( t ) is rational, the degree of the touching curve on the sphere is the same as the degree of f ( t ) .

To find the touching curve on a cone or cylinder, recall that they are envelopes of linear families of spheres. Furthermore, for a given t the rolling ball is in oriented contact with exactly one sphere in the family (see Fig. 3.2, right). We find the touching curve by the same procedure as in [17], Lemma 4, (noting that r is now a function in t ), first solving a linear equation to find the tangent sphere, and then solving a second linear equation to find the touching point. From the explicit expressions in [17], we see that if f ( t ) is rational, the degree of the touching curve is double the degree of f ( t ) .

where The second step in the construction of the parametrization is to determine the Gaussian images and of the two touching curves, i.e., the curves limiting the patch corresponding to the blend on the unit sphere. The touching curves T 0 ( t ) and T 1 ( t ) are the isoparametric curves of F ( t , u ) for u = 0 and u = 1 , and accordingly their Gaussian images are respectively N 0 ( t ) and N 1 ( t ) . This gives us the expression for and :

<!-- formula-not-decoded -->

where [ ( t )] = ( 1 = 0 , 2 = 0 , 3 = 0 ) is the projection from ΡΠ 3 to Ρ 3 .

And finally, after parametrizing the arc of circle between and using Alg. 3.5, we put together the components of the parametrization using (3.2):

<!-- formula-not-decoded -->

Combining these steps, we arrive at the following algorithm for minimal bi-degree parametrizations of variable radius rolling ball blends:

Algorithm 3.8. Consider two surfaces, and a canal surface containing a rolling ball blend between them corresponding to a rational curve f ( t ) = ( s ( t ) ; r ( t )) 2 Ρ 3,1 . The rolling ball blend is parametrized by calculating:

1. The touching curves T 0 ( t ) and T 1 ( t ) of the blend.
2. Their Gaussian images ( t ) and ( t ) .
3. The liftings X ( t ) and Y ( t ) 2 ΧΠ 1 [ t ] .
4. The lifting coefficient ( t ) = 0 ( t ) 1 ( t ) .
5. The line segment ( 1 u ) 0 ( t ) X ( t ) + u 1 ( t ) Y ( t ) , u 2 [ 0,1 ] .
6. Its generalized stereographic projection onto the unit sphere

<!-- formula-not-decoded -->

7. The parametrization of the blend F ( t , u ) = s ( t ) + r ( t ) N ( t , u ) .

If the degree of the parametrization of the line in ΧΠ 1 is minimized, this parametrization of the variable radius rolling ball blend is of minimal bi-degree ( n , 2 ) .

## 3.2.3 Rational blends of the natural quadrics

In the previous section, Alg. 3.8 was constructed under the assumption that the canal surface containing the blend is known. Given two surfaces, determining which curves f ( t ) 2 Ρ 3,1 correspond to blends between them is a separate question.

Consider the cones Χ and Χ 0 Ρ 3 corresponding to a lines Λ and Λ 0 Ρ 3,1 , and a rolling ball blend F ( t , u ) between them. At any point the rolling ball is in oriented contact with both cones, so the curve f ( t ) in Minkowski space is at zero distance to the lines. This means that it is contained in the isotropic quadric of the cone Χ (see [17]), i.e., the hypersurface in Ρ 3,1 of points at zero distance to Λ , or equivalently the spheres in oriented contact with the cone. The curve corresponding to a blend between Χ and Χ 0 is therefore contained in the 2 -dimensional intersection of their isotropic quadrics: the bisector surface of Λ and Λ 0 in Minkowski space. This surface is toric, so we may apply the results of e.g. [31] to construct spline curves in the bisector surface in Ρ 3,1 corresponding to rational blends of the two cones.

Another approach is to consider the hyperplane sections of the bisector surface. This is the approach used for fixed radius blends, where the bisector surface is intersected with the hyperplane x 4 = R . In [17], our classification of the configurations of natural quadrics that admit a rational fixed radius blends is based on the fact that there is exactly two types of surfaces where all hyperplane sections are rational: rational ruled surfaces and the Steiner surface (see [43]). Thus for these configurations, any hyperplane section of the bisector surface is a rational curve that corresponds to a rational blend of the two cones.

In the rest of the paper, we will move on from this question to consider how we can join curves in Minkowski space, and canal surfaces in Ρ 3 , with G 1 and G 2 continuity. In particular, we want to determine how the curvature properties at the join of two curves in Ρ 3,1 is reflected in the curvature properties at the join of the corresponding canal surfaces in Ρ 3 . Our main interest is to construct piecewise rational canal surfaces with a given degree of continuity at the joins, but the constructions below are valid as long as the parametrization F ( t , u ) of the canal surface is respectively once and twice differentiable at the join for G 1 and G 2 continuity.

## 3.3 Differential geometry of curves in Ρ 3,1

In the previous section, we described how the parametrization F ( t , u ) of a canal surface can be reduced to the parametrization of a circle N ( t , u ) on the unit sphere, and how this circle can be parametrized rationally. For the purposes of differential geometry, however, it is convenient to choose a non-rational parametrization of N ( t , u ) , to avoid the increase in degree from the differentiation of rational expressions. To ensure that F ( t , u ) is nondegenerate, i.e., that the envelope of the family of spheres is real, we require that κ ˙ f ( t ) κ 2 &gt; 0 .

A convenient parametrization is based on the Frenet frame of the spine curve:

<!-- formula-not-decoded -->

where t , n , and b are the unit tangent, principal normal, and bi-normal vectors of the spine curve s at t , and = ϕ ˙ s ( t ) ϕ . The non-degeneracy condition ensures that &gt; ˙ r , so the parametrization is well defined as long as the Frenet frame exists.

Remark 3.9. The choice of the Frenet frame is motivated by the existence of the Frenet equations expressing the derivatives ˙ t , ˙ n , and ˙ b in terms of t , n , and b . However, there are curves that do not have well defined Frenet frames in all points. An alternative choice of frame may, e.g., be rotation minimizing frames, which have relations with its derivatives similar to the Frenet equations. We expect that the result would be similar using this frame, however this will be the subject of further study. For now, the results in the following sections are valid if the Frenet frames of the two spine curves are well defined and identical at the join.

Figure 3.3: The sphere f t with its tangent cone.

<!-- image -->

The parametrization in (3.12) can be constructed by recalling that each sphere f t contributes a characteristic circle F t ( ) to the envelope. Consider the tangent line of the curve f ( t ) 2 Ρ 3,1 , i.e., the line through f ( t ) with direction vector ˙ f ( t ) . The envelope of this linear one-parameter family of spheres is the tangent cone of the canal surface at f t , which is tangent to f t along the characteristic circle. If is the half angle of the tangent cone we find that sin = ˙ r , which gives us the above parametrization of N ( t , ) (see Fig. 3.3).

Remark 3.10. A parametrization similar to (3.12) has already been used in the investigation of the analytic and algebraic properties of canal surfaces (see, e.g., [61]), where the focus have been on canal surfaces whose spine curves are parametrized by arc length, i.e., where = ϕ ˙ s ( t ) ϕ = 1 . However, it has been shown that it is impossible to parametrize a space curve, other than a straight line, by rational functions of its arc length (see [23]), so for a canal surface with rational spine we need to consider the general case. Fortunately, this does not significantly increase the complexity of the resulting expressions.

## 3.3.1 G 1 continuity of curves in Ρ 3,1 and canal surfaces

By the definition of the envelope, the tangent planes of the sphere f t coincide with the tangent planes of the canal surface along the characteristic circle. This gives us a first result relating the continuity of the curve in Minkowski space with the continuity of the canal surface:

Theorem 3.11. If two curves f ( t ) and g ( t ) 2 Ρ 3,1 are joined with G 1 continuity, so are the corresponding canal surfaces F ( t , u ) and G ( t , u ) 2 Ρ 3 .

Proof. G 1 continuity is also called tangent continuity . The two curves f ( t ) and g ( t ) 2 Ρ 3,1 , are tangent continuous in the point f ( t 0 ) = g ( t 0 ) if the tangent lines of f and g coincide at t = t 0 . Thus the two canal surfaces F ( t , u ) and G ( t , u ) have the same tangent cone and characteristic circle at t = t 0 . They therefore have the same unit normal vector field N ( t 0 , u ) along the characteristic circle, i.e., the tangent planes of the two adjacent surfaces coincide: the two canal surfaces are joined with tangent continuity.

For rolling ball blends, G 1 continuity may be considered sufficient, as it is the level of continuity of the blend with the original surfaces. However, existing approximate rational blends have internal G 2 continuity, so we need at least to consider if this is achievable with exact piecewise rational blends. Determining the conditions for a G 2 join of canal surfaces will therefore be the main focus of the rest of this paper.

## 3.3.2 G 2 continuity of curves in Ρ 3,1

G 2 continuity is also known as curvature continuity . In Ρ 3 , two curves are joined with curvature continuity if their osculating circles coincide at the join, i.e., if their unit tangent and principal normal vectors t and n , and curvature χ coincide. This can be extended to curves in Ρ 3,1 (see [62]).

The unit tangent vector of a curve s ( t ) 2 Ρ 3 is defined as

<!-- formula-not-decoded -->

We generalize this to a curve f ( t ) in Ρ 3,1 using the Minkowski scalar product to normalize the vector

<!-- formula-not-decoded -->

Remark 3.12. In Ρ 3 , the only vector with zero length is the vector ( 0,0,0 ) . In Minkowski space this is no longer the case. We therefore have to be careful when normalizing vectors. However, the non-degeneracy condition we stated earlier requires that κ ˙ f κ &gt; 0 , so the Minkowski unit tangent vector is well defined for curves corresponding to non-degenerate canal surfaces.

To find the unit principal normal vector n of s ( t ) , we derive ˙ s = ϕ ˙ s ϕ t , using the Frenet equation ˙ t = χ ϕ ˙ s ϕ n , and substitute the expression for t in (3.13). This gives us

<!-- formula-not-decoded -->

where, taking the norm on both sides, we find the curvature

<!-- formula-not-decoded -->

Generalizing (3.15) and (3.16) to Ρ 3,1 , we define the unit principal normal vector of a curve f ( t ) in Minkowski space in terms of the Minkowski scalar product:

<!-- formula-not-decoded -->

Figure 3.4: The spine curve s ( t ) with its Frenet frame and osculating circle (left), and the canal surface F ( t , ) with its tangent and principal normal cones (right).

<!-- image -->

and its Minkowski curvature

<!-- formula-not-decoded -->

Figure 3.4 shows the spine curve s ( t ) 2 Ρ 3 with its Frenet frame and osculating circle, and the associated canal surface F ( t , ) with tangent and principal normal cones generated by its Minkowski tangent and principal normal vectors t m and n m .

Asimilar generalization can be constructed for the first and second unit bi-normal vectors (see [62]).

Remark 3.13. A point p 2 Ρ 3 can be considered a sphere of zero radius, corresponding to the point ( p ; 0 ) 2 Ρ 3,1 . Thus a curve s ( t ) 2 Ρ 3 can be identified with the curve ( s ( t ) ; 0 ) in Minkowski space. For this curve, we then find that t m =( t ; 0 ) , n m =( n ; 0 ) , and χ = χ m .

Two curves f ( t ) and g ( t ) 2 Ρ 3,1 are joined with G 2 or curvature continuity if t m , n m , and χ m coincide at the join. We now want to prove the G 2 analogue of Thm. 3.11, i.e., that if two curves in Minkowski space are joined with G 2 continuity, then so are the associated canal surfaces.

## 3.4 Differential geometry of canal surfaces

The principal curvatures χ 1 and χ 2 in a point on a surface in Ρ 3 are defined as the maximum and minimum curvatures of its normal sections. The curvature of a surface is completely defined by χ 1 , χ 2 , and its principal curvature directions, as by Euler's theorem they determine the curvature of any normal section of the surface. χ 1 and χ 2 can be calculated as the eigenvalues, and the principal curvature directions as the eigenvectors, of the shape operator of the surface.

## 3.4.1 The principal curvatures of a canal surface

The shape operator S of a surface can be expressed in terms of the coefficients of its first and second fundamental forms. In order to compare principal curvatures, we therefore start by calculating these. The calculations are straightforward but large, so we will not show them here. The Maple file containing the details of the calculations can be obtained by application to the author.

Let f ( t ) = ( s ( t ) ; r ( t )) be a curve in Ρ 3,1 . The coefficients of the first fundamental form of f ( t ) are:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The coefficients of the second fundamental form of f ( t ) are:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where r = r ( t ) , χ and are respectively the curvature and torsion of the spine curve s ( t ) , X = r cos and Y = sin , and

<!-- formula-not-decoded -->

The shape operator S of a surface can be defined in terms of these six coefficients

<!-- formula-not-decoded -->

Inserting (3.19-3.24) and simplifying, we arrive at the shape operator of a canal surface

<!-- formula-not-decoded -->

The shape operator S is triangular, so its eigenvalues are the coefficients along the diagonal. The principal curvatures of a canal surface are therefore

<!-- formula-not-decoded -->

The second principal curvature χ 2 is the curvature of the sphere f t , and the associated curvature line is the characteristic circle. Two canal surfaces joined with G 1 continuity therefore automatically have one coinciding principal curvature and coinciding directions of principal curvature. What remains to be shown, in order to have G 2 continuity, is that the remaining principal curvature χ 1 is the same for the two canal surfaces.

## 3.4.2 G 2 continuous canal surfaces

To simplify the expressions, we assume that the Frenet frames of the two spine curves coincide. We then need to solve the equation

<!-- formula-not-decoded -->

for any X = r cos , i.e., for any , as we want to ensure χ 1 = χ 0 1 at any point along the characteristic circle.

Lemma 3.14. Two canal surfaces F ( t , u ) and F 0 ( t , u ) 2 Ρ 3 are joined with G 2 continuity if they have the same 1 and 2 .

Proof. Expanding (3.29), we arrive at

<!-- formula-not-decoded -->

6

Eliminating the two coefficients of the monomials in X proves the lemma, as the nondegeneracy condition requires 1 0 1 = 0 .

By examining 1 and 2 further, we arrive at the conjectured theorem, relating the level of continuity of the canal surface to the level of continuity of the associated curve in Minkowski space

Theorem 3.15. If two curves f ( t ) and f 0 ( t ) 2 Ρ 3,1 are joined with G 2 continuity, so are the corresponding canal surfaces F ( t , u ) and F 0 ( t , u ) 2 Ρ 3 .

Proof. G 2 continuity of curves in Ρ 3,1 is defined by coinciding unit tangent t m , principal normal n m , and Minkowski curvature χ m . We want to demonstrate that this implies that 1 and 2 are identical for the two canal surfaces.

The vectors t m and n m span a 2 -dimensional plane in Ρ 3,1 , whose restriction to the first three coordinates is the plane spanned by t and n 2 Ρ 3 . Thus if the two curves f and f 0 2 Ρ 3,1 have coinciding t m and n m , their spines s and s 0 2 Ρ 3 have coinciding t and n .

If the unit tangent vectors t m coincide at the join, the two canal surfaces have the same tangent cone, thus

<!-- formula-not-decoded -->

as ˙ r = sin ' where ' is the half angle of the tangent cone. In the plane spanned by t m and n m , we find the vector ( n ; χ 2 1 2 ) . As n = n 0 , we get χ 2 1 2 = χ 0 0 2 1 0 2 , i.e.,

<!-- formula-not-decoded -->

The Minkowski curvature can be reformulated as

<!-- formula-not-decoded -->

so if χ m = χ 0 m , we get χ = χ 0 , 1 = 0 1 , and 2 = 0 2 . Applying Lem. 3.14, this proves the theorem.

Remark 3.16. Theorems 3.11 and 3.15 confirm the conjecture in Remark 2.1 of [47] for contact of order 1 and 2 , i.e., joins of G 1 and G 2 continuity.

## 3.4.3 G 2 continuity with the end sphere

In some cases, for example when constructing corner blends (see Sec. 3.5.2), we want a segment of canal surface to be G 2 continuous with its end sphere. The principal curvatures of the sphere are χ 1 = χ 2 = 1 = r , so to calculate the conditions for G 2 continuity with the sphere we need to solve χ 1 = 1 = r , i.e.,

<!-- formula-not-decoded -->

Theorem 3.17. A segment of canal surface is joined with G 2 continuity to its end sphere if either κ ˙ f κ = 0 or ϕ ˙ s ϕ = 0 .

Proof. By inserting the expressions for 1 and 2 in (3.34), we arrive at a rational expression with numerator

<!-- formula-not-decoded -->

This gives us two cases for G 2 continuity.

The first case in the theorem, κ ˙ f κ = 0 , was initially excluded to avoid degeneracies in the canal surface. Now consider what happens when we allow this at the end of the segment of canal surface. The characteristic circle parametrized in (3.12) is reduced to a single point, so we are in fact closing the canal surface, and the surface is then by construction G 2 at the endpoint.

The second case for G 2 continuity with the end sphere is = 0 . Considering the nondegeneracy condition 2 &gt; ˙ r 2 , we need to consider the limit value of ˙ r = to determine the characteristic circle at the end sphere.

Remark 3.18. Consider a rational Bézier curve f ( t ) in Ρ 3,1 , with control points φ p 0 , . . . , p n γ (corresponding to control spheres of the canal surface). The associated canal surface is closed at t = 0 if κ p 0 p 1 κ = 0 , i.e., if the two first control spheres are in oriented contact. The canal surface is joined with its end sphere at G 2 continuity if p 0 = p 1 , i.e., the first control sphere is double.

Corollary 3.19. Let f ( t ) and g ( t ) be two Bézier curves in Ρ 3,1 with control points respectively φ p m , . . . , p 0 γ and φ p 0 , . . . , p n γ . If

1. p 1 , p 0 , p 1 are collinear,
2. p 2 , p 1 , p 0 , p 1 , p 2 span a 2 -dimensional plane, and
3. p 0 is a double control point for both curves,

then the associated canal surfaces F ( t , u ) and G ( t , u ) are joined with G 2 continuity.

Proof. If p 1 , p 0 , p 1 are collinear, then f ( t ) and g ( t ) have the same tangent line, i.e., the same t m . The control points φ p 2 , p 1 , p 0 γ , and φ p 0 , p 1 , p 2 γ span the 2 -dimensional tangent/principal normal plane, so if the two planes coincide, the two curves have the same n m as well. When p 0 is a double control point, the canal surfaces are G 2 continuous with the common control sphere along the same characteristic circle, thus they are G 2 continuous with each other.

## 3.4.4 The osculating cyclide

For a curve s ( t ) 2 Ρ 3 , a geometric interpretation of its curvature χ at a point s t is that its inverse, the radius of curvature r χ = 1 χ , is the radius of the circle best approximating the curve close to s t . This circle, called the osculating circle , lies in the plane spanned by the unit tangent vector t and the unit principal normal vector n . This interpretation can be generalized to Minkowski space.

Theorem 3.20. If at a point f t on a curve f ( t ) 2 Ρ 3,1 the unit tangent and principal normal vectors t m and n m are well defined, then there exists a unique pseudo-Euclidean (PE) circle c t ( u ) in the 2 -dimensional plane spanned by t m and n m , tangent to the curve at f t , and with the same Minkowski curvature χ m .

Proof. A PE circle in Ρ 3,1 is uniquely defined by three finite points (see [32]). Equivalently, in the plane spanned by t m and n m , a PE circle is uniquely defined by the point f t , the unit tangent vector t m and the Minkowski curvature χ m . This gives us the uniqueness of the osculating PE circle. Its existence will be shown by construction later in this section.

A Dupin cyclide is the canal surface associated with a PE circle in Minkowski space. As a curve in Ρ 3,1 has a unique osculating PE circle, a canal surface has a unique osculating cyclide , see Fig. 3.5.

Theorem 3.21. Along a characteristic circle, a canal surface is G 2 continuous with its osculating cyclide.

Proof. We showed in Thm. 3.15 that if two curves in Ρ 3,1 are joined with G 2 continuity, so are the associated canal surfaces. By construction, the osculating PE circle is G 2 continuous with the curve, which proves the theorem.

Figure 3.5: A canal surface (left) and its osculating cyclide (right).

<!-- image -->

Thus we can always extend a canal surface with G 2 continuity by joining it with its osculating cyclide.

In Sec. 3.4.1 we derived the principal curvatures χ 1 and χ 2 at a given point p on a surface. The two spheres with radii 1 = χ 1 and 1 = χ 2 in oriented contact with the surface at p are called its osculating spheres , and gives us a geometric interpretation of the principal curvatures. A definition of a G 2 continuous join of two surfaces is that both tangent planes, osculating spheres, and principal curvature directions coincide along the join.

For the canal surface corresponding to the curve f ( t ) = ( s ( t ) ; r ( t )) 2 Ρ 3,1 , f t is the osculating sphere corresponding to the principal curvature χ 2 = 1 = r ( t ) . Along the characteristic circle F t ( u ) , the second osculating sphere b c t ( u ) is in oriented contact with f t . It is therefore the member with radius 1 = χ 1 of the parabolic family of spheres generated by f t and ( F t ( u ) ; 0 ) :

<!-- formula-not-decoded -->

Solving the linear equation vr ( t ) + 0 = 1 = χ 1 for v , we find that the osculating spheres along the characteristic circle F t ( u ) are parametrized by

<!-- formula-not-decoded -->

Expanding F t ( u ) and N t ( u ) according to the parametrization in (3.12), we can write

<!-- formula-not-decoded -->

Substituting the expression for χ 1 from (3.28), we find that

<!-- formula-not-decoded -->

By reparametrizing b c t ( u ) using cos = ( 1 u 2 ) = ( 1 + u 2 ) and sin = 2 u = ( 1 + u 2 ) , we see that for each t , the curve is quadratic in u . From [6], we know that this is in fact a PE

circle, whose envelope is the Dupin necklace of the canal surface. However, since we know the explicit rational parametrization of the curve, we can show this directly.

Theorem 3.22. The envelope of the family of osculating spheres b c t ( u ) is a Dupin cyclide.

Proof. Dupin cyclides are characterized as the only surfaces which are canal surfaces with respect to two distinct one-parameter families of spheres. Considering the curve b c t ( u ) 2 Ρ 3,1 , we can assume without loss of generality that the point ( s ; r ) is located at the origin. The canal surface corresponding to b c t ( u ) is a Dupin cyclide iff there exists a second curve c t ( u ) in Ρ 3,1 such that any point ( x t , x n , x b ; x r ) 2 c t ( u ) is at zero distance from b c t ( u ) (here the variables x t , x n , x b correspond to the Frenet frame of the spine and x r to a fourth unit vector for the radius dimension):

<!-- formula-not-decoded -->

This is a quartic rational expression in u , so in order to eliminate it for any u , the five monomial coefficients in the numerator have to be identically zero. Two of them gives the implicit equations of two hyperplanes:

<!-- formula-not-decoded -->

The remaining equations are equivalent, and gives us the implicit equation of the curve c t ( u ) in the 2 -dimensional intersection of the two hyperplanes:

<!-- formula-not-decoded -->

The existence of this second curve proves the theorem.

Corollary 3.23. The curve c t ( u ) is the osculating PE circle of the curve f ( t ) , and its envelope (the Dupin collar) the osculating cyclide of the canal surface.

Proof. To prove the corollary, we need to show that c t ( u ) is contained in the 2 -dimensional plane spanned by t m and n m , and that its Minkowski curvature is χ m . The curve lies in the intersection of the two hyperplanes in (3.41), which both passes through the origin. It therefore suffices to show that t m and n m are contained in their intersection.

When we consider the vector n m , we can remove the component collinear with ( t ; ˙ r = ) . What remains is proportional to ( n ; χ 2 1 2 ) = ( 0,1,0; χ 2 1 2 ) . This proves that n m is also contained in the intersection of the two hyperplanes, and as t m and n m are not collinear, they span the 2 -dimensional intersection.

The vector t m is collinear with ( t ; ˙ r = ) = ( 1,0,0; ˙ r = ) . This point satisfies both equations in (3.41), so t m is contained in the intersection of the two hyperplanes.

The Minkowski curvature of c t ( u ) is found, e.g, by parametrizing (3.42) as a general conic, and then applying (3.18). We find that its curvature at the origin is indeed χ m .

Remark 3.24. From the above proof we see that the restriction of the 2 -dimensional plane in Ρ 3,1 spanned by t m and n m to the first three coordinates is the plane in Ρ 3 spanned by t and n . And though n is not the restriction of n m , it is uniquely determined given t m and n m .

The Corollary proves the existence of the osculating PE circle, concluding the proof of Thm. 3.20.

## 3.4.5 Additional properties of canal surfaces

Knowing the expressions for the two principal curvatures of a canal surface, we can easily calculate several other properties.

For example, the Gaussian curvature K is defined as

<!-- formula-not-decoded -->

We can then easily prove the following theorem:

Theorem 3.25. The only developable regular canal surfaces are cones and cylinders.

Proof. At regular points, the Gaussian curvature of a developable surface is identically zero. Inserting the expressions for 1 and 2 into (3.43), the numerator is linear in cos giving us two expressions to eliminate: κ ˙ f κ 2 2 χ = 0 and κ ˙ f κ ( ˙ ˙ r ¨ r ) = 0 , 8 t .

We have assumed κ ˙ f κ &gt; 0 except at the end of the canal surface, and 0 gives us a canal surface with only one point as its spine. The remaining case is χ = 0 , 8 t , which means that the spine is linear and is constant, so ˙ = 0 . To eliminate the second expression, we therefore need ¨ r = 0 , i.e., r is linear. Thus the only developable regular canal surfaces have linear spines and radius functions: they are cones and cylinders.

We can also calculate the mean curvature H of the canal surface:

<!-- formula-not-decoded -->

The first fundamental form determines whether the canal surface has any local selfintersections, as local self-intersections occur when EG F 2 = 0 . For a canal surface this gives us a condition on the radius function

<!-- formula-not-decoded -->

When the radius function is constant, i.e., for pipe surfaces, this is reduced to

<!-- formula-not-decoded -->

which should be true for any . Since min 1 = ϕ χ cos ϕ = 1 = ϕ χ ϕ , the pipe surface has no local self-intersections if

<!-- formula-not-decoded -->

Theorem 3.26. As long as its radius is less than the minimal radius of curvature of the spine, a pipe surface has no local self-intersections.

Figure 3.6: Three sided corner blended with a patch on the sphere (left) and torus (right).

<!-- image -->

## 3.5 Applications: Piecewise rational corner blends

To demonstrate the applications of the conditions for G 1 and G 2 continuity of canal surfaces, we present two constructions of blends of a three sided corner. The constructions can be generalized to certain n -sided corners, and to corners whose faces are patches on natural quadrics.

## 3.5.1 Sequential corner blends

Consider a three sided corner, whose edges are blended by patches on cylinders. If the radii of the three cylinders are equal, the associated lines in Minkowski space intersect in a point. Then a patch on the sphere corresponding to the point of intersection gives us a blend of the corner, which is G 1 continuous with the edge blends (Fig. 3.6, left).

If only two of the radii are equal, we can blend the corner with a patch on a torus, again with G 1 continuity with the edge blend (Fig. 3.6, right). In current CAD systems, these corner blends are implemented exactly. When the radii of the three cylinders are all different, however, current CAD systems have to resort to approximative blends. By constructing the blend sequentially, we can apply the fixed radius blend parametrization algorithm from [17] to construct a piecewise rational blend of the corner with internal G 1 continuity.

In this sequential construction, one of the edges of the corner is distinguished, and its blend is extended into the corner blend. We start by blending the other two edges (in Fig. 3.7 we blend the edges with the largest radii). The remaining edge is then composed of three rational pieces:

1. a line segment: the intersection of the two faces,
2. an arc of circle: the intersection of the largest cylindrical edge blend with the opposing face, and

Figure 3.7: Sequential corner blend.

<!-- image -->

3. a segment of a rational quartic curve: the intersection of two cylinders tangent in a single point.

When constructing a blend with fixed radius R , we find its spine by intersecting the R -offsets of the original surfaces. And in this offset corner (Fig. 3.7, left), the spine of the blend of the remaining edge (including the corner) is similarly composed of a line segment, an arc of circle, and a segment of a rational quartic. These three curves are joined with G 1 continuity, and the radius function of the blend is constant, so as a consequence of Thm. 3.11 the resulting composite edge-corner blend is also internally G 1 continuous.

When constructing the rational parametrization of the blend, we parametrize each of the components separately according to the closed formulae provided in [17].

## 3.5.2 Spherical corner blends

The spherical corner blend is a generalization of the case where the three cylindrical edge blends have the same radius. Consider the three curves in Ρ 3,1 corresponding to the three edge blends of the corner. Only in rare cases will they intersect in a single point. In the general case, we construct transitional curves from given points on the edge blend curves, to a common point. This will give us three transitional edge blends, and if we choose the common point properly, it corresponds to a vertex sphere (Fig. 3.8, right) tangent to all three faces, on which we determine a three-sided patch closing the composite corner blend.

The spheres tangent to all three faces constitutes a linear one-parameter family of spheres, and the associated cone is inscribed in the corner. This gives us one degree of freedom in the choice of a vertex sphere.

The spine of an edge blend lies in the bisector of the adjoining faces, which is a plane through the edge. Any rational spine can therefore be parametrized as a planar rational Bézier curve. The radius of the blend is the distance of the spine to the two adjoining faces. If we, for simplicity, assume that the vertex of the corner is in the origin and the unit normal vectors of the three faces are e 1 , e 2 , and e 3 , then the radius function of the edge blend with spine s of the face with normal vector e i is s . e i . Thus any rational curve in the bisector plane will give us a rational edge blend. The construction of edge blends can thus be reduced to the construction of rational Bézier curves in the bisector plane of the adjoining faces, and ultimately to the construction of control polygons with associated weights.

Figure 3.8: Spherical corner blend of a three sided corner.

<!-- image -->

For a spherical corner blend, the construction of a transitional edge blend is therefore reduced to the construction of a transitional control polygon connecting the spine curve of the edge blend to the centre of the vertex sphere. Let p 0 be the endpoint of the edge spine curve, p n the centre of the vertex sphere, and φ p 1 , . . . , p n 1 γ the control points in between.

If we want the corner blend to be internally G 1 continuous, it is sufficient that p 1 is on the tangent line at the end of the edge spine curve, as the join with the vertex sphere is G 1 continuous by construction.

In order to achieve internal G 2 continuity, we have to apply the results from Sections 3.4.2 and 3.4.3. Recalling Remark 3.18, in order to have a G 2 join with the vertex sphere, we choose p n 1 = p n in order to have a double control point at the vertex sphere.

In Ρ 3,1 , the bisector surface of the two hyperplanes corresponding to the two adjoining faces is a 2 -dimensional plane. This constrains any curve corresponding to a blend to the plane spanned by t m and n m . If the corner blend is internally G 1 continuous, the remaining condition to ensure internal G 2 continuity is that the spines of the edge blend and the transitional edge blend must have the same curvature χ at p 0 . For a rational Bézier curve, the curvature at an endpoint is

<!-- formula-not-decoded -->

where wi are the weights, n the degree of the spine curve, a the length of the first edge of the control polygon, and h the height of the third control point above the line containing the first edge. Given the curvature of the spine curve of the edge blend at p 0 , requiring G 2 continuity therefore gives only one additional constraint in our choice of control polygon and weights.

Summarizing the requirements for a spherical corner blend with internal G 2 continuity:

Theorem 3.27. A spherical corner blend is internally G 2 continuous if the control points

Figure 3.9: The Bézier spine curve in the bisector plane (left), and a n -sided spherical corner blend (right).

<!-- image -->

φ p 0 , . . . , p n γ and weights φ w 0 , . . . , wn γ of the transitional edge blends satisfy the following conditions:

1. p 1 is on the tangent line at the end of the edge spine curve,
2. p n 1 = p n , and
3. χ = w 0 w 2 w 2 1 n 1 n h a 2 where χ is the curvature at the end of the edge spine curve, a = ϕ p 1 p 0 ϕ , and h the distance of p 2 from the line spanned by p 0 and p 1 .

The transitional edge blends of two adjacent edges meet in at least one point: the point where the vertex sphere touches their common face, i.e., the corner of the triangular patch on the vertex sphere. In order to avoid an overlap of adjacent transitional edge blends at the vertex sphere, we impose an additional condition on their control polygons. For a three sided corner, the penultimate control point p n 1 has to be in outside the triangle defined by p n , the origin (the vertex of the corner), and 2 R e i , if the vertex sphere corresponds to the point ( p n ; R ) and the edge has direction vector e i (Fig. 3.9, left). A similar condition can be formulated for n -sided corners (Fig. 3.9, right).

## 3.6 Conclusions

We have shown that, as envelopes of one-parameter families of spheres, canal surfaces inherit their geometric continuity properties from the associated curve in Minkowski space Ρ 3,1 : if two curves are joined with G 1 or G 2 continuity, so are the corresponding canal surfaces. By extending the differential geometry of curves in Ρ 3 to Ρ 3,1 , we find that if a curve has well defined unit tangent and principal normal vectors in a point, it has a unique osculating PE circle, and the corresponding canal surface has a unique osculating cyclide. The osculating PE circle is G 2 continuous with the curve, so the osculating cyclide is G 2 continuous with the canal surface.

Rational curves in Ρ 3,1 correspond to rational canal surfaces, which can be parametrized using Alg. 3.8 with a minimal bi-degree ( n , 2 ) .

Combining these two independent results we can construct piecewise rational rolling ball blends of edges and corners of patchworks of planes and natural quadrics, with internal G 1 and G 2 continuity.

## Acknowledgements

This paper has been partially financed by the Marie-Curie Initial Training Network SAGA (ShApes, Geometry, Algebra), FP7-PEOPLE contract PITN-GA-2008-214584.

## Bibliography

- [1] R. Ablamowicz and B. Fauser. Mathematics of Clifford - a Maple package for Clifford and Graßmann algebras. Advances in Applied Clifford Algebras , 15(2):157-181, 2005.
- [2] S. Allen and D. Dutta. Cyclides in pure blending I. Computer Aided Geometric Design , 14(1):51-75, 1997.
- [3] S. Allen and D. Dutta. Cyclides in pure blending II. Computer Aided Geometric Design , 14(1):77-102, 1997.
- [4] S. Allen and D. Dutta. Supercyclides and blending. Computer Aided Geometric Design , 14(7):637-651, 1997.
- [5] G. Aumann. Curvature continuous connections of cones and cylinders. ComputerAided Design , 27(4):293-301, 1995.
- [6] A. Bartoszek, R. Langevin, and P. G. Walczak. Special canal surfaces of Σ 3 . Bulletin of the Brazilian Mathematical Society , 42(2):301-320, 2010.
- [7] B. Bastl, B. Jüttler, M. Lávicka, T. Schulz, and Z. Šír. On the parameterization of rational ringed surfaces and rational canal surfaces. 2013.
- [8] M. Bizzarri and M. Láviˇ cka. A symbolic-numerical method for computing approximate parameterizations of canal surfaces. Computer-Aided Design , 44(9):846-857, 2012.
- [9] M. Bizzarri and M. Láviˇ cka. Parameterizing rational offset canal surfaces via rational contour curves. Computer-Aided Design , 45(2):342-350, 2013.
- [10] M. Bizzarri and M. Láviˇ cka. Approximation of implicit blends by canal surfaces of low parameterization degree. In M. Floater, T. Lyche, M.-L. Mazure, K. Mørken, and L. L. Schumaker, editors, Mathematical Methods for Curves and Surfaces , volume 8177 of Lecture Notes in Computer Science , pages 34-48. Springer Berlin Heidelberg, 2014.
- [11] T. E. Cecil. Lie Sphere Geometry . Springer New York, 2008.
- [12] H. C. Cho, H. I. Choi, S.-H. Kwon, D. S. Lee, and N.-S. Wee. Clifford algebra, Lorentzian geometry, and rational parametrization of canal surfaces. Computer Aided Geometric Design , 21(4):327-339, 2004.
- [13] J. A. Cottrell, T. J. R. Hughes, and Y. Bazilevs. Isogeometric analysis: Toward integration of CAD and FEA . John Wiley and Sons, 2009.

- [14] H. E. I. Dahl. Piecewise rational parametrizations of canal surfaces. In M. Floater, T. Lyche, M.-L. Mazure, K. Mørken, and L. L. Schumaker, editors, Mathematical Methods for Curves and Surfaces , volume 8177 of Lecture Notes in Computer Science , pages 88-111. Springer Berlin Heidelberg, 2014.
- [15] H. E. I. Dahl. Rational parametrizations of edge and corner blends for isogeometric analysis. In G. Muntingh and T. Dokken, editors, SAGA - Advances in ShApes, Geometry, and Algebra , volume 10 of Geometry and Computing . Springer Berlin Heidelberg, 2014.
- [16] H. E. I. Dahl. Isotropic Möbius geometry and i-M circles on singular isotropic cyclides. 2014.
- [17] H. E. I. Dahl and R. Krasauskas. Rational fixed radius rolling ball blends between natural quadrics. Computer-Aided Geometric Design , 29:691-706, 2012.
- [18] H. E. I. Dahl and R. Krasauskas. Quadrics in isotropic space and applications. 2014.
- [19] R. Dietz, J. Hoschek, and B. Jüttler. An algebraic approach to curves and surfaces on the sphere and on other quadrics. Computer Aided Geometric Design , 10(3-4):211-229, 1993.
- [20] M. Dohm and S. Zub˙ e. The implicit equation of a canal surface. Journal of Symbolic Computation , 44(2):111-130, 2009.
- [21] L. Druoton, L. Garnier, R. Langevin, H. Marcellier, and R. Besnard. Blending planes and canal surfaces using Dupin cyclides. In H. Cherifi, J. M. Zain, and E. ElQawasmeh, editors, Digital Information and Communication Technology and Its Applications (DICTAP), Part II , pages 406-420. Springer, 2011.
- [22] L. Druoton, R. Langevin, and L. Garnier. Blending canal surfaces along given circles using Dupin cyclides. International Journal of Computer Mathematics , 91(3):641-660, 2014.
- [23] R. T. Farouki and T. Sakkalis. Rational space curves are not 'unit speed'. Computer Aided Geometric Design , 24(4):238-240, 2007.
- [24] S. Foufou and L. Garnier. Dupin cyclide blends between quadric surfaces for shape modeling. Computer Graphics Forum , 23(3):321-330, 2004.
- [25] R. Hartshorne. Algebraic geometry , volume 52 of Graduate texts in Mathematics . Springer New York, 1977.
- [26] K. Karˇ ciauskas and R. Krasauskas. Rational rolling ball blending of natural quadrics. Mathematical Modelling and analysis , 5:97-107, 2000.
- [27] M. Kazakeviˇ ci¯ ut˙ e. Classification of pairs of natural quadrics from the point of view of Laguerre geometry. Translated from Lietuvos Matematikos Rinkinys , 45(1):64-83, 2005.
- [28] M. Kazakeviˇ ci¯ ut˙ e and R. Krasauskas. Blending Cylinders and Cones using Canal Surfaces. Nonlinear Analysis: Modeling and Control , 5:77-89, 2000.

- [29] R. Krasauskas. Minimal rational parametrizations of canal surfaces. Computing , 79 (2-4):281-290, 2007.
- [30] R. Krasauskas. Branching blend of natural quadrics based on surfaces with rational offsets. Computer Aided Geometric Design , 25(4-5):332-341, 2008.
- [31] R. Krasauskas and M. Kazakeviˇ ci¯ ut˙ e. Universal rational parametrizations and spline curves on toric surfaces. In T. Dokken and B. Jüttler, editors, Computational Methods for Algebraic Spline Surfaces , pages 213-231. Springer Berlin Heidelberg, 2005.
- [32] R. Krasauskas and C. Mäurer. Studying cyclides with Laguerre geometry. Computer Aided Geometric Design , 17(2):101-126, 2000.
- [33] R. Krasauskas and M. Peternell. Rational offset surfaces and their modeling applications. In I. Z. Emiris, F. Sottile, and T. Theobald, editors, Nonlinear Computational Geometry , volume 151 of The IMA Volumes in Mathematics and its Applications , pages 109-135. Springer New York, 2010.
- [34] R. Krasauskas and S. Zub˙ e. Canal surfaces defined by quadratic families of spheres. In B. Jüttler and R. Piene, editors, Geometric Modeling and Algebraic Geometry , pages 79-92. Springer Berlin Heidelberg, 2008.
- [35] R. Krasauskas, S. Zub˙ e, and S. Cacciola. Bilinear Clifford-Bézier patches on isotropic cyclides. In M. Floater, T. Lyche, M.-L. Mazure, K. Mørken, and L. L. Schumaker, editors, Mathematical Methods for Curves and Surfaces , volume 8177 of Lecture Notes in Computer Science , pages 283-303. Springer Berlin Heidelberg, 2014.
- [36] E. Kreyszig. Differential geometry . Dover Publications, 1991.
- [37] G. Landsmann, J. Schicho, and F. Winkler. The parametrization of canal surfaces and the decomposition of polynomials into a sum of two squares. Journal of Symbolic Computation , 32(1-2):119-132, 2001.
- [38] R. Langevin and P. G. Walczak. Canal foliations of Σ 3 . Journal of the Mathematical Society of Japan , 64(2):659-682, 2012.
- [39] W. Lü and H. Pottmann. Pipe surfaces with rational spine curve are rational. Computer Aided Geometric Design , 13(7):621-628, 1996.
- [40] N. Lubbes. Families of circles on surfaces. Accepted for publication in Contributions to Algebra and Geometry , 2013.
- [41] C. Mäurer. Applications of sphere geometry in canal surface design. In P.-J. Laurent, P. Sablonnikre, and L. L. Schumaker, editors, 4th International Conference on Curves and Surfaces: Curve and Surface Design, Saint-Malo 1999 , pages 267--276. Vanderbilt University Press, 2000.
- [42] J. R. Miller and R. N. Goldman. Using tangent balls to find plane sections of natural quadrics. Computer Graphics and Applications, , 12(2):68-82, 2002.

- [43] E. H. Moore. Algebraic surfaces of which every plane-section is unicursal in the light of n-dimensional geometry. American Journal of Mathematics , 10(1):17-28, 1887.
- [44] M. Peternell. Rational two-parameter families of spheres and rational offset surfaces. Journal of Symbolic Computation , 45(1):1-18, 2010.
- [45] M. Peternell and H. Pottmann. Computing rational parametrizations of canal surfaces. Journal of Symbolic Computation , 23(2-3):255-266, 1997.
- [46] M. Peternell and H. Pottmann. A Laguerre geometric approach to rational offsets. Computer Aided Geometric Design , 15(3):223-249, 1998.
- [47] M. Peternell and H. Pottmann. Applications of Laguerre geometry in CAGD. Computer Aided Geometric Design , 15(2):165-186, 1998.
- [48] M. Peternell, B. Odehnal, and M. L. Sampoli. On quadratic two-parameter families of spheres and their envelopes. Computer Aided Geometric Design , 25(4-5):342-355, 2008.
- [49] H. Pottmann and Y. Liu. Discrete surfaces in isotropic geometry. Mathematics of surfaces XII , (i):341-363, 2007.
- [50] H. Pottmann, P. Grohs, and N. J. Mitra. Laguerre minimal surfaces, isotropic geometry and linear elasticity. Advances in Computational Mathematics , 31(4):391-419, 2008.
- [51] H. Pottmann, L. Shi, and M. Skopenkov. Darboux cyclides and webs from circles. Computer Aided Geometric Design , 29(1):77-97, 2012.
- [52] M. J. Pratt. Applications of cyclide surfaces in geometric modelling. 3rd IMA Conference on the Mathematics of Surfaces , 1988.
- [53] M. J. Pratt. Cyclides in computer aided geometric design. Computer Aided Geometric Design , 7(1-4):221-242, 1990.
- [54] M. J. Pratt. Cyclides in computer aided geometric design II. Computer Aided Geometric Design , 50(May 1993):19-21, 1995.
- [55] J. R. Rossignac. Constraints in constructive solid geometry. Proceedings of the 1986 workshop on Interactive 3D graphics - SI3D '86 , pages 93-110, 1987.
- [56] C.-K. Shene. Blending with affine and projective Dupin cyclides. Neural, Parallel &amp; Scientific Computations, Special issue on Computer Aided Geometric Design , 5:121-152, 1997.
- [57] C.-K. Shene. Blending two cones with Dupin cyclides. Computer Aided Geometric Design , 15(7):643-673, 1998.
- [58] C.-K. Shene and J. K. Johnstone. On the lower degree intersections of two natural quadrics. ACM Trans. Graph. , 13(4):400-424, 1994.
- [59] M. Skopenkov, H. Pottmann, and P. Grohs. Ruled Laguerre minimal surfaces. Mathematische Zeitschrift , 2011.

- [60] J. Vida, R. R. Martin, and T. Varady. A survey of blending methods that use parametric surfaces. Computer-Aided Design , 26(5):341-365, 1994.
- [61] Z. Xu, R. Feng, and J.-g. Sun. Analytic and algebraic properties of canal surfaces. Journal of computational and applied mathematics , 195(1-2):220-228, 2006.
- [62] S. Yılmaz and M. Turgut. On the differential geometry of the curves in Minkowski space-time I. International Journal of Ccontemporary Mathematical Sciences , 3(27):13431349, 2008.
- [63] S. Yılmaz, E. Özyılmaz, and M. Turgut. On the differential geometry of the curves in Minkowski space-time II. International Journal of Computational and Mathematical Sciences , 3(2):53-55, 2009.

## List of Figures

|   1.1 | Elliptic plane/cone intersection, rolling ball, spine, and touching curves . . .                                                                                                  |   2 |
|-------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----|
|   1.2 | Elliptic fixed radius rolling ball blend of a plane and a cone . . . . . . . . . . .                                                                                              |   3 |
|   1.3 | Variable radius rolling ball blend of a cone and a plane . . . . . . . . . . . . . .                                                                                              |   4 |
|   1.4 | Fixed radius rolling ball blends of two cones . . . . . . . . . . . . . . . . . . . . .                                                                                           |   5 |
|   1.5 | The tangent and principal normal cones, and the osculating Dupin cyclide .                                                                                                        |   7 |
|   1.6 | Sequential and spherical corner blend . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                           |   8 |
|   1.7 | The control spheres and the blend of a composite corner . . . . . . . . . . . . .                                                                                                 |  10 |
|   1.8 | Spine curves in the bisector surface of two cylinders, and a cyclide corner blend with its tangent cones . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .          |  11 |
|   1.9 | The dual surfaces of vertical one-sheet hyperboloids in Ρ 3 ++ 0 . . . . . . . . . .                                                                                              |  13 |
|  1.10 | The dual surfaces of semi-vertical and horizontal one-sheeted hyperboloids .                                                                                                      |  14 |
|  1.11 | The dual surface of an ellipsoid . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                      |  15 |
|  1.12 | The dual surface of a horizontal elliptic paraboloid and a horizontal elliptic cylinder . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . |  16 |
|  1.13 | Blending three cylinders: A patch on the torus and a patch on the PN surface                                                                                                      |  17 |
|  1.14 | Two quartic isotropic cyclides and their three families of i-M circles . . . . . .                                                                                                |  19 |
|   2.1 | Elliptic plane/cone intersection, rolling ball, spine, and touching curves . . .                                                                                                  |  22 |
|   2.2 | The orientations of the surfaces determine the placement of the rolling ball                                                                                                      |  23 |
|   2.3 | Tangential distance between spheres . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                           |  24 |
|   2.4 | Projecting onto the cone . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                      |  25 |
|   2.5 | Touching curves on the pipe surface of the blend . . . . . . . . . . . . . . . . . .                                                                                              |  27 |
|   2.6 | Elliptic rolling ball blend . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                   |  28 |
|   2.7 | Elliptic rolling ball blend with control mesh . . . . . . . . . . . . . . . . . . . . .                                                                                           |  29 |
|   2.8 | Hyperbolic and parabolic rolling ball blends . . . . . . . . . . . . . . . . . . . . .                                                                                            |  30 |
|   2.9 | Elliptic, hyperbolic and parabolic rolling ball blends of two cones . . . . . . .                                                                                                 |  39 |
|  2.10 | Rolling ball blend of two cylinders with one common touching point and corner blend with three different edge radii . . . . . . . . . . . . . . . . . . . . . .                   |  41 |
|  2.11 | Rolling ball blend of two cones with one common touching point . . . . . . .                                                                                                      |  42 |
|  2.12 | Sphere/cylinder blend and its Viviani type quartic spine curve . . . . . . . . .                                                                                                  |  43 |
|   3.1 | Rolling ball blends of two cones . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                        |  49 |
|   3.2 | Linear family generated by two spheres in oriented contact and the rolling                                                                                                        |     |

ball in oriented contact with a cone . . . . . . . . . . . . . . . .

.

.

.

.

.

.

.

.

.

.

.

.

50

|   3.3 | The sphere f t with its tangent cone . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                              |   53 |
|-------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------|
|   3.4 | The spine curve with its Frenet frame and osculating circle, and the canal surface with its tangent and principal normal cones . . . . . . . . . . . . . . . .          |   55 |
|   3.5 | A canal surface and its osculating cyclide . . . . . . . . . . . . . . . . . . . . . . .                                                                                |   60 |
|   3.6 | Three sided corner blended with a patch on the sphere and torus . . . . . . .                                                                                           |   63 |
|   3.7 | Sequential corner blend . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                           |   64 |
|   3.8 | Spherical corner blend of a three sided corner . . . . . . . . . . . . . . . . . . . .                                                                                  |   65 |
|   3.9 | The spine curve in the bisector plane and a n -sided spherical corner blend .                                                                                           |   66 |
|   4.1 | Rolling ball blend of a plane and a cone with a local self-intersection . . . . . Aplane and a cone with two control spheres and the resulting variable radius .        |   71 |
|   4.2 | rolling ball blend . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                          |   72 |
|   4.3 | Example of a composite corner and its composite blend . . . . . . . . . . . . .                                                                                         |   73 |
|   4.4 | Curves in the bisector surface and a Bézier curve in the bisector plane .                                                                                               |   77 |
|   4.5 | . . . Symmetric variable radius rolling ball blend of two cylinders with one point of oriented contact, spine curves and touching curves . . . . . . . . . . . . . . .  |   78 |
|   4.6 | A cone with a parabolic pencil of spheres and the Dupin cyclide containing the blend with the plane . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . |   80 |
|   4.7 | Singular spine curve and G 1 non-singular blending surface . . . . . . . . . . . .                                                                                      |   81 |
|   4.8 | The control spheres of the composite blend . . . . . . . . . . . . . . . . . . . . .                                                                                    |   85 |
|   4.9 | Blending the intersection of the cone with the top plane . . . . . . . . . . . . .                                                                                      |   86 |
|  4.10 | Blending the left non-convex corner . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                                 |   87 |
|   5.1 | The dual surface of a horizontal elliptic cylinder . . . . . . . . . . . . . . . . . .                                                                                  |  105 |
|   5.2 | The dual surface of a horizontal parabolic cylinder . . . . . . . . . . . . . . . . .                                                                                   |  106 |
|   5.3 | The dual surface of an upright parabolic cylinder . . . . . . . . . . . . . . . . . .                                                                                   |  107 |
|   5.4 | The dual surface of semi-vertical and horizontal hyperbolic cylinders . . . . .                                                                                         |  108 |
|   5.5 | The dual surfaces of upright hyperbolic cylinders . . . . . . . . . . . . . . . . . .                                                                                   |  110 |
|   5.6 | The dual surfaces of vertical one-sheet hyperboloids . . . . . . . . . . . . . . . .                                                                                    |  113 |
|   5.7 | The dual canal surfaces of vertical rotational one-sheet hyperboloids . . . . .                                                                                         |  114 |
|   5.8 | The dual surfaces of vertical two-sheet hyperboloids . . . . . . . . . . . . . . . .                                                                                    |  115 |
|   5.9 | The dual surfaces of semi-vertical one-sheeted hyperboloids . . . . . . . . . . .                                                                                       |  116 |
|  5.10 | The dual surfaces of semi-vertical two-sheeted hyperboloids . . . . . . . . . . .                                                                                       |  117 |
|  5.11 | The dual surfaces of horizontal one- and two-sheeted hyperboloids . . . . . .                                                                                           |  118 |
|  5.12 | The dual canal surface of a vertical elliptic paraboloid . . . . . . . . . . . . . . .                                                                                  |  119 |
|  5.13 | The dual surface of a horizontal elliptic paraboloid . . . . . . . . . . . . . . . . .                                                                                  |  120 |
|  5.14 | The dual surface of a horizontal hyperbolic paraboloid . . . . . . . . . . . . . .                                                                                      |  121 |
|  5.15 | The dual surface of an ellipsoid . . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                              |  122 |
|  5.16 | . The dual canal surface of a sphere . . . . . . . . . . . . . . . . . . . . . . . . . . . .                                                                            |  123 |
|  5.17 | Blending three cylinders: A patch on the torus and its dual in Ρ 3 ++ 0 . . . . . .                                                                                     |  127 |
|  5.18 | Blending three cylinders: A patch on the PN surface and its dual in Ρ 3 ++ 0 . .                                                                                        |  128 |
|   6.1 | A cone and its three families of i-M circles . . . . . . . . . . . . . . . . . . . . . .                                                                                |  136 |
|   6.2 | Two quartic isotropic cyclides and their three families of i-M circles . . . . . .                                                                                      |  138 |
|   6.3 | Cubic isotropic cyclide and its three families of i-M circles . . . . . . . . . . . .                                                                                   |  139 |