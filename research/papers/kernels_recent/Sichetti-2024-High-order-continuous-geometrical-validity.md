## High-Order Continuous Geometrical Validity

FEDERICO SICHETTI, University of Genoa, Italy ZIZHOU HUANG, New York University, United States MARCO ATTENE, CNR-IMATI: GENOVA, Italy DENIS ZORIN, New York University, United States ENRICO PUPPO, University of Genoa, Italy DANIELE PANOZZO, New York University, United States

Fig. 1. Soft body simulations [Ferguson et al. 2023] of twisting a beam in 3D (left) and twisting the inner circle of a fat ring in 2D (right), without (top) and with (bottom) our continuous validity check. The high-order invalid elements are marked in red, and the scaled Jacobian distribution is shown on the right. Without our method, the Twist-beam example has 1 and 161 invalid elements at t= 3 s and t= 7 s respectively; the Twist-ring example has 11 and 31 invalid elements at t= 0 . 4 s and t= 1 s respectively. With our validity check and adaptive quadrature refinement scheme, the simulations do not contain any invalid elements.

<!-- image -->

We propose a conservative algorithm to test the geometrical validity of simplicial (triangles, tetrahedra), tensor product (quadrilaterals, hexahedra), and mixed (prisms) elements of arbitrary polynomial order as they deform within a time interval.

Our algorithm uses a combination of adaptive Bézier refinement and bisection search to determine if, when, and where the Jacobian determinant of an element's polynomial geometric map becomes negative in the transition from one configuration to another. In elastodynamic simulation, our algorithm guarantees that the system remains physically valid during the entire trajectory, not only at discrete time steps. Unlike previous approaches, physical validity is preserved even when our method is implemented using floating point arithmetic. Hence, our algorithm is only slightly slower than existing inexact methods while providing guarantees and while being an easy drop-in replacement for current validity tests.

To prove the practical effectiveness of our algorithm, we demonstrate its use in a high-order Incremental Potential Contact (IPC) elastodynamic simulator and experimentally show that it prevents invalid, simulationbreaking configurations that would otherwise occur using inexact methods.

## 1 INTRODUCTION

In computer graphics, mechanical engineering, and scientific computing, physical objects are often modeled using meshes composed of simple elements, such as tetrahedra, hexahedra, and prisms. Each element is typically associated with two maps: (1) a geometric map

Authors' addresses: Federico Sichetti, University of Genoa, Italy, federico.sichetti@edu. unige.it; Zizhou Huang, New York University, United States, zizhou@nyu.edu; Marco Attene, CNR-IMATI: GENOVA, Italy, marco.attene@ge.imati.cnr.it; Denis Zorin, New York University, United States, dzorin@cs.nyu.edu; Enrico Puppo, University of Genoa, Italy, enrico.puppo@unige.it; Daniele Panozzo, New York University, United States, panozzo@nyu.edu.

that defines the element's shape; and (2) a basis map that extends quantities (such as displacement or velocities) defined at the element's nodes into its interior. For rendering, linear polynomials (hat functions) are commonly used for both maps, but higher-order versions are widely employed when greater accuracy is required. When the basis map is used to interpolate a displacement , the element's geometry is derived by combining the initial geometric map with the basis map, yielding a polynomial whose order corresponds to the highest of the two. A typical example in graphics is the use of second-order elements in finite element (FE) simulations for fabrication [Panetta et al. 2015], which generates quadratic curved elements even if the initial geometric map is piecewise linear.

In the following, we will overload the term geometry map to describe the final geometry of an object, which can thus also incorporate the displacement basis map. In mathematical notation, this is defined with a polynomial function:

<!-- formula-not-decoded -->

where 𝜎 represents a reference element domain (such as a standard simplex or multi-interval), and 𝑛 denotes the dimension of the element. See Figure 2 for examples.

During animation or simulation, objects are deformed by modifying the geometry map, the basis map, or both: these modifications are usually performed by applying linear transformations to the map coefficients. Note that, for a piecewise linear geometric map, this reduces to the usual interpolation of the coordinates of the mesh vertices. Similarly, for curved meshing, it is also typical to start with a piecewise linear mesh and then curve its elements to reduce the approximation error [Toulorge et al. 2013].

Fig. 2. A reference domain 𝜎 is mapped to linear elements (Left) and quadratic elements (Right). Quadratic elements can be given either by an initial quadratic geometry or by combining an initial linear geometry with a quadratic displacement. Geometric maps on a Lagrangian basis are specified by mapping the domain points 𝑃 𝑖 to control points 𝑥 𝑗 ( 𝑃 𝑖 ) . Blue and red areas denote a positive and negative determinant of the Jacobian, respectively. In both cases, element 𝑥 2 ( 𝜎 ) is invalid. In the linear case, the whole element is inverted, while in the quadratic case, only a small portion of 𝑥 2 ( 𝜎 ) is inverted.

<!-- image -->

Since these meshes are used to represent physical objects, a basic, yet elusive, requirement is that the geometry of the object does not self-intersect at any time during deformation. This condition can be violated in two ways: (1) by a global intersection or (2) by a local lack of injectivity of the geometric map. The first problem has been studied thoroughly in the parametrization and simulation literature, at least for linear elements [Smith and Schaefer 2015; Wang et al. 2021a]. The second condition has been considered mostly in the context of mesh parametrization, and, even in that case, no robust algorithm exists (see Section 3). In this work, we focus on the latter.

Problem statement. We denote with ¯ 𝑥 a geometric map that is dynamically changing over time and we study its injectivity. This reduces to studying the sign of the determinant of its Jacobian | 𝐽 ¯ 𝑥 | to assess the continuous validity of the element, i.e., if, where and when | 𝐽 ¯ 𝑥 | becomes negative. We note that this is a subtly challenging problem even for the simple case of a linear triangle (Figure 3): an element that is valid in its initial and final configurations might become invalid during the trajectory.

Brief summary of the state of the art. A common approach among practitioners is testing the validity of elements by just computing the value of | 𝐽 𝑥 | at quadrature points at every time step, e.g., [Dey et al. 2001]. This might fail to detect invalidity even in static tests, let alone the continuous case. Smith and Schaefer [2015] study the continuous problem for the specific case of 2D linear triangles, reducing it to a quadratic root finding, which is solved numerically; this approach is hard to extend to 3D or to higher-order polynomials. Other approaches exist to resolve the static test for high-order elements, e.g., [Johnen et al. 2014], but they are neither conservative nor easily extensible to the continuous test. More detail on the state of the art is provided in Section 3.

Contribution. We introduce the first generic formulation and algorithm for the continuous validity test of elements, supporting the most common types - such as triangles, quadrilaterals, tetrahedra, prisms, and hexahedra - and extending to high-order basis and geometric maps. Our algorithm is provably conservative when implemented using floating-point arithmetic, meaning that if an element is detected to be valid, it is guaranteed to remain valid throughout the entire specified time interval. This level of robustness, crucial for algorithmic reliability, has not been achieved by any previous method.

Fig. 3. A dynamic linear element with linear trajectories that flips twice in a time interval, resulting in an element that is valid at both time steps, but invalid in the transition. Assuming point 𝐴 remains fixed, the arrows represent the velocities of points 𝐵 and 𝐶 to reach the final position. The color of each element indicates the sign of the determinant at time 𝑡 : blue if positive and red if negative. The dynamic element's Jacobian determinant is a univariate polynomial in 𝑡 of degree 2 , with two distinct roots in [ 0 , 1 ] .

<!-- image -->

If an element becomes invalid at any point, our algorithm provides a conservative estimate of the inversion time and introduces a custom quadrature rule that accurately reflects the detected inversion. Specifically, this means that the numerical integration diverges when the element inverts, a property not provided by adaptive quadrature rules commonly used in high-order finite elements.

While designed for the dynamic case, our algorithm can also be used for the static case, with minor modifications: in this setting, our algorithm is the first algorithm to provide a conservative static geometrical validity test for high-order elements.

Evaluation. Our algorithm is designed and implemented for high performance, as its use-case is within optimization loops requiring the testing of large datasets: on static checks, we demonstrate that our test is competitive in terms of runtime with current inexact methods, being slightly slower while guaranteeing a conservative answer. To quantitatively evaluate the correctness and efficiency of our approach and compare it with more specialized alternatives, we construct a dataset of 2D and 3D time-dependent queries whose ground truth is computed using (extremely expensive) symbolic root finding.

Applications. Having access to a conservative check we discovered that it is very common for high-order FE simulations to contain invalid elements in their solution; we show examples in PolyFEM in Figures 1 and 12. This seems to be a common problem with highorder FE codes: for example, FEBio [Maas et al. 2012] also uses a static check only at quadrature points. This issue is rarely mentioned in the literature [Anderson et al. 2014; Dobrev et al. 2019] and we are not aware of other papers proposing a solution. We believe that the presence of invalidity is due to the use of insufficiently accurate quadrature to capture the infinite elastic potential inside some of the most distorted elements. This is a major source of both the numerical fragility of this software and inaccuracy in the solution as physically invalid configurations are reported as the simulation result. By replacing the validity check and the quadrature in PolyFEM with our approach, we show that these issues disappear and the impact on performance is moderate.

Impact. We believe our algorithm will be an essential addition to the growing toolkit of robust geometric building blocks used in modern parametrization, meshing, and simulation algorithms. To foster its adoption, we will release an open-source reference implementation.

## 2 OVERVIEW OF THE METHOD

Given a dynamic element ¯ 𝑥 ( 𝜎 ) deforming over the time interval [ 0 , 1 ] , we study the determinant of its Jacobian | 𝐽 ¯ 𝑥 | (Sections 5 and 6). The polynomial | 𝐽 ¯ 𝑥 | can have a high order in both its spatial and temporal variables (Appendix A). To ensure a conservative answer, we employ a custom bisection root-finding method controlled by an accuracy parameter 𝛿 &gt; 0 . Let 𝑡 ∗ denote the earliest time at which ¯ 𝑥 ( 𝜎 ) becomes invalid (i.e., | 𝐽 ¯ 𝑥 | turns negative). We return a time 𝑡 ∗ &gt; 𝑡 ∗ -𝛿 and a point 𝑃 ∈ 𝜎 such that | 𝐽 ¯ 𝑥 | is positive everywhere for 𝑡 ≤ 𝑡 ∗ , and | 𝐽 ¯ 𝑥 ( 𝑃, 𝑡 )| becomes negative for some 𝑡 ≤ 𝑡 ∗ + 𝛿 . This guarantees that the element can safely deform up to time 𝑡 ∗ while the point 𝑃 is used to adaptively refine the quadrature rule, steering the simulation away from invalid configurations (Section 7). If the element remains valid throughout, we simply return 𝑡 ∗ = 1 .

The parameter domain of a dynamic element has both space and time dimensions. Our test proceeds by bisecting the time dimension: it maintains lower 𝑡 ∗ and upper 𝑡 ∗ bounds for the critical inversion time, and terminates when the difference between these bounds is smaller than 𝛿 , or when the element is confirmed to be valid throughout the interval. The algorithm employs a priority queue of sub-domains, which are created by recursively splitting the initial space-time domain.

For a given sub-domain 𝑆 , we compute a minimum inclusion function that returns an interval 𝐼 , which is guaranteed to contain the minimum value of | 𝐽 ¯ 𝑥 | ( 𝑆 ) : if 𝐼 is strictly positive, the element is valid in 𝑆 ; if 𝐼 is strictly negative, the element is invalid somewhere within 𝑆 (but it is not necessarily invalid everywhere in 𝑆 ); if 𝐼 contains zero, nothing can be said and further refinement of 𝑆 is necessary. The minimum inclusion function is a crucial component of our method; details on its definition and computation are provided in Sections 4.3 and 6.1, respectively.

Another key aspect of the algorithm is the decoupling of refinements in the spatial and temporal dimensions. While the time dimension may require refinement until the interval between 𝑡 ∗ and 𝑡 ∗ is lower than 𝛿 , the spatial dimensions usually need less refinement. In essence, bisection along the time axis is primarily driven by the need to narrow the bounds of 𝑡 ∗ , while subdivision of the spatial domain is employed to resolve indeterminate configurations. Details and pseudo-code are given in Section 5.1.

To account for numerical errors, we employ interval arithmetic. The value of parameter 𝛿 allows for a trade-off between computation time and the accuracy of the estimation; however, regardless of the parameter choice, the algorithm always provides a conservative estimation.

Example. Consider the quadratic element in Figure 4(a). It is valid at 𝑡 = 0 , becomes invalid at some intermediate time 𝑡 ∗ &lt; 0 . 5 , and then returns to a valid state at a later time, remaining valid until 𝑡 = 1 .

The parameter domain for this dynamic element can be visualized as in Figure 4(b) with an upright triangular prism, where each horizontal slice of the prism represents the spatial domain at a specific time, and time progresses along the vertical axis from 0 to 1. Notably, the element inversion occurs within a localized wedge (red volume in the figure) during the times when the element is invalid. In general, inversions can occur anywhere within the domain, including regions away from vertices.

The minimum inclusion function is evaluated first for the whole prism. The result is an interval that contains zero, thus the domain is subdivided as in Figure 4(c) along both the time and the space dimensions, and eight sub-prisms are pushed to the priority queue. While processing the four prisms corresponding to the time interval [ 0 , 0 . 5 ] , the minimum inclusion function returns a strictly positive interval for three of them, which are discarded from the queue; and it returns a strictly negative interval for the fourth one, which intersects the red wedge: this domain is bisected only in the time dimension and its two children are pushed onto the queue. Subdomains spanning earlier times are processed first.

In the subsequent refinements (Figure 4(d), side view), the minimum inclusion function will always be strictly negative, hence only bisection in the time dimension will occur: the analysis of valid intervals that do not intersect the red wedge contributes to increasing the value of 𝑡 ∗ while the analysis of intervals that do intersect the red wedge contributes to decreasing the value of 𝑡 ∗ , until convergence. Note that, if the domain were always subdivided along space and time, as in Figure 4(e), many more subdomains would be generated and the algorithm could become much slower. For the 3D order 3 armadillo dataset, with 10% target error, the naive approach has an average processing time of ∼ 260 𝜇𝑠 per element, whereas our approach takes ∼ 14 𝜇𝑠 per element on average. For higher element orders, or queries with higher precision, the maximum number of subdivisions used in the naive approach must be limited (otherwise the check will consume unreasonable amounts of memory), and the test can fail to produce an estimate within precision on some elements.

Fig. 4. (a): A 2D quadratic element with linear trajectories is valid at the start and end positions but locally inverted during the transition. (b): Its parametric domain is an upright triangular prism, where the vertical axis represents time and each horizontal slice represents the space domain at a given time. The red region denotes the portion of the domain in which the determinant | 𝐽 ¯ 𝑥 | is not positive. The dynamic element's Jacobian determinant is a trivariate polynomial in 𝜉 1 , 𝜉 2 , 𝑡 of order 2 in its space variables and order 2 in the time variable. (c): The root finding algorithm bisects the time domain and quadrisects the spatial domain to isolate a thin slice [ 𝑡 ∗ , 𝑡 ∗ + 𝛿 ] containing the critical time of inversion 𝑡 ∗ . In the first step, both space and time dimensions are refined, generating eight sub-domains, which are pushed into a priority queue. (d): For clarity we show a side view for the following steps. Only the time dimension is bisected: the analysis of sub-domains that do not intersect the invalid region increases the value of 𝑡 ∗ while the analysis of sub-domains that do intersect the invalid region decreases the value of 𝑡 ∗ until convergence. (e): If all dimensions were refined at all steps, many more sub-domains would be generated, thus killing performance.

<!-- image -->

## 3 RELATED WORK

We briefly describe polynomial bases and their use in graphics, methods for checking static and continuous validity of elements, and discuss their application in FEA and meshing. We conclude with an overview of robust predicate evaluation techniques, which we use in our algorithm.

High-Order Bases and Geometry for Finite Element Analysis. Linear basis functions are often a sub-optimal choice in many contexts. For example, Schneider et al. [2022] advocate for the use of highorder bases for elliptic PDEs; Bargteil and Cohen [2014], Mezger et al. [2009], Suwelack et al. [2013], Ushakova [2011] use high-order elements for animation; and Mandad and Campen [2020] propose to use a high-order basis for parametrization. Our algorithm provides a guaranteed-conservative check for the validity of these elements, increasing the robustness of any method using a high-order basis.

A related, but distinct, concept is the use of high-order geometry, where the geometry of an element is represented using a high-order polynomial. More commonly, 𝐶 0 [Jiang et al. 2021a] or 𝐶 𝑘 geometric maps (IGA) [Cottrell et al. 2009] are used. The latter option is popular in mechanical engineering, where IGA envisions the use of the same representation for both interpolating the physical quantities and for representing the geometry.

Despite the different uses, our algorithm applies to generic polynomials, hence it can be used as is to ensure the validity of the geometric map.

Static Element Inversion Check. The special case of checking the geometric validity of a linear triangle/tetrahedron in a static setting has been solved in a seminal paper by Shewchuk [1997], where a robust predicate called orient2d/orient3d is introduced. This paper revolutionized mesh generation and simulation, providing a reliable, yet efficient, solution to one of the basic primitives used by meshing and simulation algorithms. To the best of our knowledge, this approach has not been extended to elements with higher order; such extension is challenging because an element may flip at certain points while remaining valid at others, as in the example shown on the right side of Figure 2.

For high-order simplicial elements, a common approach to test for element inversion consists in testing only their quadrature points [Dey et al. 2001; Gargallo-Peiró et al. 2015; Maas et al. 2012; Schneider et al. 2019]: while effective at avoiding NaNs in the integration of certain diverging elastic potentials, this approach is not conservative, leading to incorrect stresses (Figure 13). An efficient but not conservative method has been introduced in [Johnen et al. 2018, 2013, 2014], where the Jacobian determinant of the element is represented in Bézier form: the inversion check then reduces to testing the positivity of the Bézier coefficients as they undergo adaptive refinement. This method can miss inversions (we provide a numerical example in Appendix E) due to the use of floating point arithmetic. We take inspiration from this check and similarly use adaptive Bézier subdivision to derive a continuous test: our algorithm solves a different problem by adding the time dimension and is designed to be conservative due to its judicious use of rational and interval arithmetic.

For the special case of hexahedral elements, which are commonly used in commercial finite element analysis software, this problem has been extensively studied in [Ushakova 2011]. Unfortunately, these tests are insufficient to guarantee validity but are used nonetheless due to their efficiency. Vavasis [2003] proposes a sufficient condition but does not provide a conservative algorithm that takes advantage of it. Johnen et al. [2017] propose an optimized version of [Johnen et al. 2014] specifically for linear hexahedral elements, however, the approach still suffers from the same floating point issues as the original test.

A radically different approach is taken in [Marschner et al. 2020], where a Sum-of-Squares (SOS) relaxation is used to compute the minimum Jacobian determinant, reducing the problem to solving a sequence of small semidefinite programming problems of increasing complexity. The method offers generality and an elegant formulation, but is relatively expensive computationally (multiple SDP solves per element), and only guarantees injectivity up to numerical precision of an iterative convex solver, which might result in invalid elements.

All the methods described above are designed to handle static checks exclusively, which is insufficient for validating deforming elements over time.

Continuous Element Inversion Check. To the best of our knowledge, the only paper explicitly addressing the problem of checking the validity of an element that deforms over time is [Smith and Schaefer 2015]. They propose an algorithm to estimate the safest step before an inversion for 2D linear elements by using the closed form of the roots of a degree 2 polynomial. Their approach is however not conservative due to floating point rounding, as we show in Section 8. Furthermore, the method does not scale to linear 3D elements - where the polynomial is of order 3 and robust root finding is not trivial - nor higher order elements - where the polynomial is multivariate and closed forms for the roots may not even be available.

The problem is also discussed in [Anderson et al. 2014] for highorder element remapping and in [Dobrev et al. 2019] for high-order meshing, however no algorithm for the validity check is proposed. Our algorithm could be used in their setting to provide a conservative validation of their resulting elements.

High-Order Meshing. High-order meshing requires a high-order validity check to ensure that elements are valid after curving. We refer to [Geuzaine et al. 2015; Jiang et al. 2021b] for an overview of the state of the art of high-order meshing. Our contribution can be used, in its reduced form for static validity, as a provably conservative check in any of these meshing algorithms to increase their robustness.

Interval Arithmetic. According to the IEEE 754 standard, the result of a floating point operation is a rounded value of the exact result. Instead of computing a single rounded value, one can compute an interval that contains the exact result by simply rounding in both directions. Replacing FP numbers with intervals slows the calculations by an order of magnitude on average but, on the other hand, enables a provably correct implementation of geometric algorithms [Snyder 1992]. Intervals are provided by many existing libraries such as CGAL [Brönnimann et al. 1998], Boost [Schling 2011], Filib and Filib++ [Lerch et al. 2006], BIAS [Knuppel 1994] and GAOL [Goualard 2005]. We point the reader to [Tang et al. 2023] for a detailed comparison of these tools. When full portability is not required, modern SIMD architectures can be exploited to speed up interval arithmetic significantly. The basic idea is to store both the bounds in a single register that can host 128 bits, which is the space required by two double precision FP numbers, and then perform each operation on the entire register simultaneously. This approach, first introduced in [Lambov 2008], is employed in the numeric kernel of the indirect predicates library [Attene 2019], which we use in our implementation.

Fig. 5. Reference elements with domain points of order 3: triangle, square, tetrahedron, hexahedron, prism.

<!-- image -->

Weuseintervals to represent inclusion functions and to guarantee that any possible rounding error is always tracked so as to provide a correct polynomial evaluation.

## 4 PRELIMINARIES AND NOTATIONS

Weaddress 𝑛 -dimensional ( 𝑛 = 2 , 3 ) high-order meshes consisting of elements of various types. The geometry of every element is defined by a polynomial map. We will refer to the order 𝑝 of a polynomial as the maximum exponent of a single variable (as opposed to the usual notion of degree). Table 1 gives a summary of symbols defined in the following and used throughout.

## 4.1 Reference Element Domains

For each type of element, we define a common reference domain (or reference element ) 𝜎 𝑛 𝑠 ⊂ [ 0 , 1 ] 𝑛 to use as the coordinate domain.

Definition 1 (Reference Domain). Let 𝑛, 𝑠 ∈ N , 1 ≤ 𝑠 ≤ 𝑛 . The 𝑛 -dimensional reference domain 𝜎 𝑛 𝑠 ⊂ [ 0 , 1 ] 𝑛 is the locus of points with coordinates ( 𝜉 1 , . . . , 𝜉 𝑛 ) that satisfy the system of inequalities:

<!-- formula-not-decoded -->

With this notation, we have a general parameter space that works for all the most commonly used FEM elements. The element 𝜎 1 1 is the unit segment; 𝜎 2 1 is the unit square; 𝜎 2 2 is the standard triangle; 𝜎 3 1 is the unit cube; 𝜎 3 2 is the unit triangular prism; 𝜎 3 3 is the standard tetrahedron. More generally, 𝜎 𝑛 𝑠 is the tensor product of a standard 𝑠 -simplex with a standard ( 𝑛 -𝑠 ) -hypercube 1 .

1 Note that, the unit segment can be seen both as a 1-simplex and as a 1-hypercube; to avoid any ambiguity, we always treat it as a simplex, so that, e.g., the unit square is regarded as the tensor product of a 1-simplex with a 1-hypercube, rather than as a 2-hypercube.

Table 1. Symbols used in the text and where they are defined.

| Symbol            | Meaning                                            | Def.   |
|-------------------|----------------------------------------------------|--------|
| 𝑛                 | dimension of element and embedding space           | 4      |
| 𝑠                 | dimension of simplicial part of element            | 4.1    |
| 𝜎,𝜎 𝑛 𝑠           | static reference element (with dimensions)         | 4.1    |
| 𝜉 1 , . . . 𝜉 𝑛   | spatial coordinates                                | 4.1    |
| 𝑝                 | order of an element / polynomial                   | 4.2    |
| 𝑥                 | geometric map of an element                        | 4.2    |
| &#124; 𝐽 𝑥 &#124; | Jacobian determinant of 𝑥                          | 4.2    |
| I                 | intervals on the real line                         | 4.3    |
| □ 𝑓               | inclusion function for 𝑓                           | 4.3    |
| □ min 𝑓           | minimum inclusion function for 𝑓                   | 4.3    |
| 𝑡                 | time coordinate                                    | 5      |
| ¯ 𝜎, ¯ 𝜎 𝑛 𝑠      | dynamic reference element (with dim.)              | 5      |
| ¯ 𝑥               | dynamic geometric map of an element                | 5      |
| 𝑡 ∗               | minimum time at which &#124; 𝐽 ¯ 𝑥 &#124; vanishes | 5      |
| 𝑡 ∗ , 𝑡 ∗         | lower and upper bounds to 𝑡 ∗                      | 5      |
| 𝛿                 | user-specified accuracy                            | 5      |
| 𝑙 max             | maximum level of recursion                         | 5      |
| 𝜓 - ,𝜓 +          | time-only subdivision maps                         | 5      |
| 𝜓 𝑞               | 𝑞 -th subdivision map                              | 5      |
| C 𝑝 𝜎             | subset of indices of corners of element 𝜎          | 6      |
| T ± B→B           | time-only subdivision matrix                       | 6      |
| T 𝑞 B→B           | 𝑞 -th subdivision matrix                           | 6      |
| Γ 𝑝 𝜎             | set of domain points of order 𝑝 on 𝜎               | A      |
| 𝛾 𝑖               | domain point                                       | A      |
| I 𝑝 𝜎             | set of indices of points of Γ 𝑝 𝜎                  | A      |
| L 𝑚               | 𝑖 -th Lagrange polynomial of order 𝑚               | A      |
| 𝑖 B 𝑚             | 𝑖 -th Bernstein polynomial of order 𝑚              | A      |
| 𝑖 𝑓 L             | vector of coefficients in Lagrange form            | A      |
| 𝑓 B               | vector of coefficients in Bézier form              | A      |
| T L→B             | transition matrix from Lagrange to Bézier          | A      |

Fig. 6. Above: A domain 𝜎 2 2 with its domain points of order 3 and the geometric map 𝑥 to the physical element 𝑥 ( 𝜎 2 2 ) with Lagrange control points. Below: likewise for an element 𝜎 3 3 of order 2.

<!-- image -->

## 4.2 High-Order Elements

A generic polynomial 𝑓 : 𝜎 -→ R can be defined with a basis of polynomials: we consider here the Lagrange basis - which is most common in FEM - and the Bernstein basis that gives the Bézier form. Both representations define 𝑓 as a linear combination of the basis functions with control coefficients associated with domain points that form a regular grid over the reference element; the number of domain points sets the order 𝑝 of the polynomial (Figure 5).

For a given order and dimension, pre-computed conversion matrices allow us to convert between these two representations in both directions. The Lagrange and Bézier forms and the conversion matrices are detailed in Appendix A.

Geometric Map. The geometric map 𝑥 : 𝜎 𝑛 𝑠 -→ R 𝑛 that maps a reference element 𝜎 𝑛 𝑠 into the physical element 𝑥 ( 𝜎 𝑛 𝑠 ) is represented by specifying its set of control points, where each control point is a 𝑛 -dimensional point (Figure 6). Each coordinate of 𝑥 is expressed with a multivariate polynomial, which is the tensor product of a 𝑠 -variate polynomial of degree 𝑝 with ( 𝑛 -𝑠 ) univariate polynomials of degree 𝑝 , each in a different variable. It follows that the map 𝑥 has order 𝑝 in all its variables (even though its total degree may be higher).

In the following, we study the determinant of the Jacobian 𝐽 𝑥 of the geometric map 𝑥 , denoted | 𝐽 𝑥 | , which is also a multivariate polynomial in the same variables as 𝑥 , but of a different order. In particular, the number of terms of | 𝐽 𝑥 | rapidly increases with the dimension and order of the element. See Appendix A for details.

## 4.3 Minimum Inclusion Function

Informally speaking, given a real function 𝑓 and a domain 𝐷 , an inclusion function for 𝑓 over 𝐷 returns an interval that bounds the range of values of 𝑓 in 𝐷 . Inclusion functions are widely used in root finding and in the evaluation of robust predicates [Snyder 1992]. In our case, we are rather interested in an interval that just contains the minimum value of 𝑓 .

Let I be the space of intervals on the real line. For 𝑎 = [ 𝑎, 𝑎 ] ∈ I , let us define 𝑤 ( 𝑎 ) = 𝑎 -𝑎 the width of interval 𝑎 . Let 𝐴 = 𝑎 1 ×· · · × 𝑎 𝑛 ∈ I 𝑛 be a 𝑛 -dimensional interval; we extend the definition of width as 𝑤 ( 𝐴 ) = max 𝑛 𝑗 = 1 𝑤 ( 𝑎 𝑗 ) . Given 𝐷 ⊆ R 𝑛 compact, we further extend the definition of width as 𝑤 ( 𝐷 ) = min 𝐴 ⊇ 𝐷 𝑤 ( 𝐴 ) .

Let Ω ⊆ R 𝑛 be a compact domain, let 𝑓 be a real function defined on Ω , and let us denote P( Ω ) the subsets of Ω . Given a function 𝑓 : Ω -→ R , an inclusion function for 𝑓 is a function □ 𝑓 : P Ω -→ I such that, for any 𝐷 ⊆ Ω we have

<!-- formula-not-decoded -->

We say □ 𝑓 to be convergent if for any 𝐷 ⊆ Ω

<!-- formula-not-decoded -->

In particular, if 𝐷 shrinks about 𝜉 , then □ 𝑓 ( 𝐷 ) shrinks about 𝑓 ( 𝜉 ) .

Definition 2 (Minimum inclusion function). A minimum inclusion function for the function 𝑓 is a function □ min 𝑓 : P( Ω ) -→ I such that, for any 𝐷 ⊆ Ω we have

<!-- formula-not-decoded -->

If the lower end of □ min 𝑓 ( 𝐷 ) is positive, we know that 𝑓 is always positive in 𝐷 ; if the upper end is negative, we know that 𝑓 has negative values, but is not necessarily negative everywhere, in 𝐷 ; otherwise, nothing can be said about the sign of 𝑓 in 𝐷 .

A convergent inclusion function can be used to find a root of a function 𝑓 by subdividing the initial domain Ω until it becomes sufficiently small. Likewise, one can use a convergent minimum inclusion function to find the portions of Ω where 𝑓 is positive, by recursively subdividing the domain. The type of subdivision used to perform refinement depends on the shape of Ω . For instance, while bisection can be used for a multi-interval domain, simplicial domains may require less trivial subdivision rules (Appendix B).

## 4.4 Interval and Rational Arithmetic

Interval arithmetic consists of a set of operations defined on the set of real intervals I such that if 𝜉 ∈ 𝐴 𝜉 ∈ I and 𝜁 ∈ 𝐴 𝜁 ∈ I , then ( 𝜉 ∗ 𝜁 ) ∈ 𝐴 𝜉 ∗ 𝐴 𝜁 , where ∗ in the right-hand side is the interval version of the operation. If the exact result of an operation on floating point numbers falls between two representable values, rounding is required. We make our computations conservative by replacing floating point numbers with singleton intervals (i.e., intervals with matching endpoints), and rounding the left end of the result down and the right end up for every subsequent interval operation. In our code, we use the implementation of Attene [2020]. More details are provided in Appendix C.

Interval arithmetic does not prevent the propagation of error; it merely keeps track of it. One way to implement exact arithmetic is via rational numbers: as long as an algorithm only involves rational operations, we can represent numbers exactly as fractions of integer values since every floating point number is also rational. The main drawback of rational arithmetic is that it can be orders of magnitude slower than interval arithmetic because the bits needed to encode each fraction increase with computations. For this reason, our use of rational arithmetic is limited to off-line computations.

## 5 CONTINUOUS GEOMETRICAL VALIDITY

In a dynamic simulation, the elements of the mesh move and deform over time. Like space, time is discretized into time steps, which are typically regular. We assume that the control points move along straight-line trajectories at each time step, and, without loss of generality, we can assume each transition occurs between time 𝑡 = 0 and time 𝑡 = 1 . Following Definition 1 we have:

Definition 3 (Dynamic reference element). Let 𝜎 𝑛 𝑠 be a reference element. The dynamic element ¯ 𝜎 𝑛 𝑠 of 𝜎 𝑛 𝑠 is the ( 𝑛 + 1 ) -dimensional reference element 𝜎 𝑛 + 1 𝑠 = 𝜎 𝑛 𝑠 × [ 0 , 1 ] .

Assuming linear trajectories, the dynamic geometric map ¯ 𝑥 : R 𝑛 + 1 → R 𝑛 of order 𝑝 for ¯ 𝜎 𝑛 𝑠 is expressed by linear interpolation of the 𝑛 -dimensional geometric maps 𝑥 0 ( 𝜉 ) and 𝑥 1 ( 𝜉 ) of the static element at the two consecutive time steps:

<!-- formula-not-decoded -->

This map is of order 𝑝 in the 𝜉 variables and linear in 𝑡 .

## 5.1 Continuous Validity Test

Assuming that the input element is valid at time 𝑡 = 0 , the continuous validity test consists of determining whether or not the Jacobian determinant | 𝐽 ¯ 𝑥 ( 𝜉 )| is everywhere greater than 0 on ¯ 𝜎 at all times in [ 0 , 1 ] . If this is not the case, the algorithm should find the earliest time 𝑡 ∗ ∈ ( 0 , 1 ] in which the element becomes invalid - hence, it is valid within [ 0 , 𝑡 ∗ ) .

Target accuracy. Since finding an exact solution is not necessary and very expensive, we instead settle for a conservative estimate 𝑡 ∗ that is close to 𝑡 ∗ up to a given user-provided threshold 𝛿 , i.e. 𝑡 ∗ ∈ [ 𝑡 ∗ , 𝑡 ∗ + 𝛿 ] . Moreover, since we assumed that 𝑡 ∗ &gt; 0 , we require that 𝑡 ∗ be strictly positive as well, regardless of the value of 𝛿 . This threshold is used to trade off accuracy and time performance.

Early termination. An intrinsic challenge of both the static and dynamic problems is that certifying the positivity of a polynomial can be arbitrarily hard. Therefore, there is no upper bound on the number of subdivisions required to assess the validity of an element.

To prevent the algorithm from taking unreasonable time, we employ a termination criterion that triggers when refinement becomes excessive. In these extreme cases, we halt and provide an estimate 𝑡 ∗ that may not be within 𝛿 of the true value 𝑡 ∗ , but is nevertheless conservative.

Let the depth of a subdomain 𝑆 be the number of subdivisions required to obtain 𝑆 from the initial domain. In our implementation we stop when the depth of a subdomain 𝑆 exceeds a threshold 𝑙 max .

Minimum inclusion function. We rely on a convergent inclusion function, which comes together with a procedure to decompose an element into sub-elements. For the sake of clarity and generality, we first describe our algorithm avoiding the details on how we define our inclusion function and domain subdivision strategy, which we detail in Section 6. We also use 𝐽 = | 𝐽 ¯ 𝑥 | as a short-hand notation for the determinant of the Jacobian of the dynamic element at hand.

Given a generic inclusion function □ 𝐽 on the domain ¯ 𝜎 𝑛 𝑠 , we define a minimum inclusion function as follows: let □ 𝐽 ( 𝐷 ) = [ 𝐽 𝐷 , 𝐽 𝐷 ] then

<!-- formula-not-decoded -->

where D 𝐷 is a small set of samples in 𝐷 . In practice, we sample 𝐽 at these points to bound the minimum of 𝐽 from above. Note that, if 𝐽 is negative at any of those samples, we know that the element becomes invalid in 𝐷 .

Subdivision Maps. Given a reference domain 𝜎 = 𝜎 𝑛 𝑠 , we define a set of 𝑄 linear maps { 𝜓 𝑞 : 𝜎 -→ 𝜎 } 𝑞 called the subdivision maps of 𝜎 , and we call 𝜓 𝑞 ( 𝜎 ) a subdomain of 𝜎 ; we require that the union of all subdomains is 𝜎 , and the intersection of any two subdomains is either empty or has dimension less than 𝑛 . We define the standard subdivision maps for 𝑛 ∈ { 2 , 3 } that we use in our implementation in Appendix B. In the following, we always assume that 𝑄 = 2 𝑛 .

For a time dependent reference domain ¯ 𝜎 = ¯ 𝜎 𝑛 𝑠 , its subdivision maps are the same as 𝜎 𝑛 + 1 𝑠 , but we also define two additional time subdivision maps , denoted 𝜓 -and 𝜓 + , such that 𝜓 - ( ¯ 𝜎 ) and 𝜓 + ( ¯ 𝜎 ) are respectively the lower and upper half of ¯ 𝜎 when bisected in the time dimension only.

## Algorithm 1 Maximum valid time step with inclusion functions

## 1: function MaxValidStep( 𝐽, 𝛿, 𝑙 max )

```
2: 𝑃 ← PriorityQueue( ≺ ) ⊲ priority queue for subdomains 3: 𝑡 ∗ ← 1 ⊲ initialize upper bound of 𝑡 ∗ 4: 𝑡 ∗ ← 0 ⊲ initialize lower bound of 𝑡 ∗ 5: Push( 𝑃, 𝜎 ) 6: 𝐹 ← false ⊲ fl ag of whether an invalidity has been found 7: 𝑙 ← 0 ⊲ maximum subdivision depth reached so far 8: while true do 9: if 𝐹 ∧ ( 𝑡 ∗ -𝑡 ∗ ≤ 𝛿 ) ∧ ( 𝑡 ∗ > 0 ) then ⊲ reached accuracy 10: return 𝑡 ∗ ⊲ conservative estimate of 𝑡 ∗ 11: if IsEmpty( 𝑃 ) then 12: return 1 13: 𝑆 ← Pop( 𝑃 ) ⊲ get the next subdomain from 𝑃 14: 𝑙 ← max { 𝑙, Depth( 𝑆 ) } ⊲ update maximum depth 15: if 𝑙 > 𝑙 max then ⊲ maximum level reached: give up 16: return 𝑡 ∗ ⊲ conservative estimate of 𝑡 ∗ 17: 𝑡 ∗ ← StartTime( 𝑆 ) ⊲ everything before this time is valid 18: 𝐼 ← □ min 𝐽 ( 𝑆 ) ⊲ check minimum inclusion 19: if High( 𝐼 ) ≤ 0 then ⊲ there is an invalidity in 𝑆 20: if EndTime( 𝑆 ) < 𝑡 ∗ then 21: 𝐹 ← true 22: 𝑡 ∗ ← EndTime( 𝑆 ) 23: Push( 𝑃,𝜓 - ( 𝑆 ) ) ⊲ bisect on the 𝑡 axis only 24: Push( 𝑃,𝜓 + ( 𝑆 ) ) ⊲ bisect on the 𝑡 axis only 25: else if ¬( Low( 𝐼 ) > 0 ) then 26: for 𝑞 ∈ { 1 , . . . , 𝑄 } do 27: Push( 𝑃,𝜓 𝑞 ( 𝑆 ) ) ⊲ subdivide on 𝜉 and bisect on 𝑡 28: function ≺ ( 𝑆 0 , 𝑆 1 ) ⊲ priority function 29: if StartTime( 𝑆 0 ) ≠ StartTime( 𝑆 1 ) then ⊲ lower time first 30: return StartTime( 𝑆 0 ) < StartTime( 𝑆 1 ) 31: else ⊲ for ties, prioritize boxes most likely to be invalid 32: return High( □ min 𝐽 ( 𝑆 0 ) ) < High( □ min 𝐽 ( 𝑆 1 ) )
```

Pseudo-Code. The pseudo-code of the algorithm is given in Algorithm 1. The algorithm takes in input a polynomial 𝐽 ( 𝜉, 𝑡 ) defined on domain ¯ 𝜎 and the thresholds 𝛿 and 𝑙 𝑚𝑎𝑥 and returns a time 𝑡 ∗ without invalid configurations, and within a time 𝛿 of an invalid configuration. The algorithm keeps internal current lower and upper bounds for 𝑡 ∗ , initializing them to 0 and 1, respectively. Given a subdomain 𝑆 ⊂ ¯ 𝜎 , pseudocode functions StartTime ( 𝑆 ) and EndTime ( 𝑆 ) return respectively the minimum and maximum values of the time coordinate for points in 𝑆 .

The algorithm uses a priority queue 𝑃 (line 2) of subdomains of 𝜎 , with a related priority function ≺ (line 28) giving higher priority to sub-domains that span intervals of time with an earlier start point (that is, a lower minimum 𝑡 ). The initial domain 𝜎 is pushed into the priority queue 𝑃 (line 5); then elements are popped from the 𝑃 one by one (line 13), and if their minimum inclusion function does not guarantee their validity, they are subdivided and their subdomains are pushed to 𝑃 . See the next paragraph for the subdivision strategy. This continues until the queue becomes empty or an early exit condition is met. By construction, the priority function ≺ guarantees that when we pop an element 𝑆 from the queue, then 𝐽 is positive at all times before StartTime ( 𝑆 ) , and 𝑡 ∗ can then be updated accordingly.

Early exits occur if the required accuracy 𝛿 is achieved (line 9), meaning that the difference between 𝑡 ∗ and 𝑡 ∗ is less than 𝛿 , or the maximum depth 𝑙 max has been reached (line 15), meaning that we pop from the queue an interval that comes from a sequence of 𝑙 max subdivisions.

Subdivision Strategy. If the interval 𝐼 returned by □ min 𝐽 ( 𝑆 ) is completely negative - meaning that 𝑆 contains negative values of 𝐽 (line 19) - then the upper bound 𝑡 ∗ is updated, and element 𝑆 is bisected along the time dimension only, with the two resulting subdomains being pushed into 𝑃 . Conversely, if interval 𝐼 contains zero, the element 𝑆 is split along all its dimensions (line 25), including time, according to the subdivision scheme of reference element ¯ 𝜎 ; again, the resulting elements are pushed into 𝑃 . Finally, no subdivision is necessary if 𝐼 only contains positive values, and the space-time region occupied by 𝑆 will not be considered again for the remainder of the algorithm.

Note that, by bisecting only the time dimension (line 19), we postpone any refinement of the spatial dimensions until we find a time interval in which 𝐽 may potentially be positive everywhere. This strategy allows us to avoid many unnecessary refinements in the space dimensions, and to decouple the subdivision on time (controlled by accuracy 𝛿 ) from the subdivision in space (which does not have an accuracy requirement).

## 6 IMPLEMENTATION

The implementation of our method requires designing a minimum inclusion function □ min 𝑓 and a corresponding subdivision strategy that uses robust computations while keeping the runtime sufficiently low to enable its use within a simulation loop.

## 6.1 Inclusion Functions for Space and Time

As observed by Snyder [1992], interval arithmetic provides a universal way to design inclusion functions. Given a polynomial 𝑓 : Ω -→ R and 𝐷 ⊆ Ω , let 𝐴 𝐷 be the smallest multi-interval containing 𝐷 . We could define

<!-- formula-not-decoded -->

where the evaluation of 𝑓 on the right side is intended with interval arithmetic, and thus returns an interval. Any strategy subdividing 𝐷 and reducing 𝐴 𝐷 (e.g., bisection along all coordinates) provides a convergent inclusion function.

We tried this approach, but the inclusion functions may be very loose about 𝑓 and require many refinement steps to converge, or even get stuck on nearly invalid elements due to the numerical error accumulating too fast for the inclusion function to keep up with. We compare the time performance for the static case only in Table 3. We instead follow the approach proposed by Johnen et al. [2014] for the static validity test and extend it to our continuous setting.

Overview of Bézier Refinement. Let 𝑓 be the order 𝑚 polynomial of which we want to find the minimum on ¯ 𝜎 (in our case, 𝑓 = | 𝐽 ¯ 𝑥 | ). Our inclusion function is based on the Bézier representation of 𝑓 and a recursive decomposition of ¯ 𝜎 . The reason why we want to represent our polynomial in the Bézier basis is the convex hull property [Farin 2001], by which the values of 𝑓 on ¯ 𝜎 are bounded by the minimum and maximum coefficients of 𝑓 when expressed in the Bézier basis.

To obtain the vector of Bézier coefficients 𝑓 B of 𝑓 , we first compute its vector of Lagrange coefficients 𝑓 L , which can be obtained by simply evaluating 𝑓 at the domain points; then we premultiply 𝑓 L with a change of basis matrix T L→B that we shall call transformation matrix , which is described in detail in Appendix A.

Let ¯ I 𝜎 𝑚 be the set of indices of the control points of 𝑓 and ¯ C 𝑚 𝜎 ⊂ ¯ I 𝜎 𝑚 be the set of indices corresponding to the corners of ¯ 𝜎 at time 1 . Since the Bézier basis is interpolating at the corners of the domain (i.e. 𝛽 𝑗 = 𝑓 ( 𝛾 𝑗 ) for all 𝑗 ∈ ¯ C 𝑚 𝜎 ), we define the minimum inclusion function as

<!-- formula-not-decoded -->

Therefore, if all entries of 𝑓 B are positive we know the element is valid everywhere, and if any of the corner entries is non-positive we know that the element is invalid at the end time. Otherwise, the interval returned by the inclusion function contains zero, and we need to refine the search by subdividing ¯ 𝜎 .

The subdivision of ¯ 𝜎 is performed via another set of change of basis matrices, dubbed subdivision matrices , T 𝑞 B→B , for 𝑞 = 1 , . . . , 𝑄 . Premultiplication of 𝑓 B by these matrices gives a Bézier representation of 𝑓 on a smaller portion of the domain, which can be used to compute tighter bounds local to each subdomain. These matrices are defined in Appendix A.

Time-only refinement. Bisection in the time dimension only is performed analogously by multiplication of 𝑓 B with two time subdivision matrices T -B→B and T + B→B , also described in Appendix A.

## 6.2 Robust Computation

Rational Precomputation of matrices. All the transformation and subdivision matrices T L→B , T 𝑞 B→B , and T ± B→B are only dependent on element type (tetrahedron, hexahedron, etc.) and order, and as such can be precomputed offline. Since we want to minimize the accumulation of error in our computations, and all entries of these matrices are rational, we construct these matrices using exact rational arithmetic for each element type and order. The outcome is a rational matrix, which we convert to intervals by rounding the two endpoints outward if the exact value cannot be represented as a floating point number. The resulting interval is guaranteed to contain the exact value of the fraction while being as tight as possible.

Intervals. The input to our check is the set of Lagrange control points for the elements of a mesh, represented in floating point. Each floating point coordinate is converted to a singleton interval (i.e. an interval with zero width) and all subsequent operations are performed in interval arithmetic with conservative rounding.

Summary. The combination of rational precomputation and interval arithmetic ensures that our algorithm is conservative while maintaining a low computational cost (Section 8): the rational precomputation is performed only once offline and does not affect runtime, while the use of interval arithmetic adds a minor ( ∼ 2 × ) overhead over a direct floating point implementation.

## 6.3 Acceleration

Global Queries. In practical applications, one is often interested in the maximum time for which all elements are valid. We refer to this as a global dynamic query and give a strategy to accelerate it.

After an invalid element has been found (with estimated valid time step 𝑡 ∗ ), it becomes unnecessary to validate the other elements at later times: 𝑡 ∗ will anyhow be the maximal allowed step. It might be tempting to terminate early as soon as an inverted element has been found. However, this is not conservative as some other elements may still have a lower 𝑡 ∗ . We thus keep track of the smallest value of 𝑡 ∗ found in previous checks and leverage the fact that 𝑡 ∗ can never decrease in Algorithm 1 to stop computation on an element as soon as its estimate for 𝑡 ∗ exceeds the running minimum.

The order in which elements are processed matters for global queries: it is beneficial to process elements that are most likely invalid first, as it will provide higher opportunities for this pruning strategy to be effective. For this reason, we first sort the polynomials according to their constant term, in ascending order. For the order 3 Armadillo mesh in Figure 9, this strategy improves the total running time by about 60% over running the queries individually; whereas the speedup is less relevant for the order 2 mesh (about 12%).

Parallelization. Validity checks for meshes are trivially parallelized by processing elements in batches. To avoid any synchronization between different threads, every batch of queries assigned to a thread 𝑖 does an independent sorting and keeps its own running minimum 𝑡 ∗ 𝑖 to use as an early termination condition, as explained in the previous paragraph.

Precomputation of Jacobian determinant. The input to the subdivision procedure is a Lagrange representation of the Jacobian determinant polynomial of the element. This only depends on the shape and order of the element, as well as its control points.

For each element type and order combination, we symbolically compute the expression of each Lagrange coefficient in terms of the control point coordinates, and remove common subexpressions with CSE [Muchnick 1997] (we use the implementation in SymPy [Meurer et al. 2017]). This approach increases the compilation time but provides dramatic runtime performance boosts: for the order 3 Armadillo dataset, we get a speedup of about 20 × .

## 7 APPLICATION TO SIMULATION

Incremental potential time-stepping [Kane et al. 2000] is becoming popular in graphics [Li et al. 2020] and biomechanics [Martin et al. 2024] due to its robustness to extreme deformation and contact[Chen et al. 2022; Fang et al. 2024, 2021; Ferguson et al. 2023, 2021; Huang et al. 2024a,b; Lan et al. 2022a, 2023, 2022b, 2021; Li et al. 2021, 2023a, 2022, 2024; Shen et al. 2024]. We briefly summarize the approach here, without contact handling, as it is relevant to motivate the need for a continuous dynamic positivity check in physical simulation: as part of this overview, we will show that the check alone is insufficient, as IPC also requires a consistent invalidity-aware quadrature rules, which we introduce in Section 7.1.

Fig. 7. Potential landscape and one descent step from the black point. The descent step to the red point is invalid since it crosses an invalid region with infinite potential.

<!-- image -->

## 7.1 Continuous Validity in Simulation

The updated displacement 𝑢 𝑡 + 1 of an object at the next time step is computed solving an unconstrained non-linear energy minimization:

<!-- formula-not-decoded -->

where 𝑢 𝑡 is the displacement at the step 𝑡 , 𝑣 𝑡 is velocity, and 𝐸 ( 𝑢,𝑢 𝑡 , 𝑣 𝑡 ) is a time-stepping Incremental Potential [Kane et al. 2000]. We refer to [Li et al. 2020] for more details.

For common non-linear material models, this potential is infinite when an element has a negative Jacobian, as the Jacobian determinant appears in the denominator of the expression. A physically valid trajectory cannot reach a state with infinite potential: however, this is a challenging condition to enforce in practice.

Line Search. The potential 𝐸 is minimized with a descent algorithm (gradient descent or Newton), which computes a local approximation of a descent direction: this approximation might, for a finite step length, cross a region with infinite potential (Figure 7). This is a typical challenge in collision detection [Wang et al. 2021b], but is rarely considered for the elastic potential - the only work we know that considers this problem, in 2D only, is [Smith and Schaefer 2015]. This challenge can be solved using a continuous inversion check within the line search, which is the focus of our work. To the best of our knowledge, state-of-the-art IPC solvers [Li et al. 2020; Schneider et al. 2019] use a static check instead of a continuous one, which cannot guarantee trajectory validity.

Quadrature. Non-linear elastic potentials cannot be integrated exactly with numerical quadrature (as they are not polynomials), leading to unbounded errors for diverging potentials. We show in Figure 8 an example of an element with an infinite potential integrated with both a standard fixed quadrature rule and the adaptive quadrature derived by our algorithm: only in the second case does the numerical integration correctly diverge. The use of a fixed quadrature leads to solver failures as the direction computed using quadrature is not a descent direction and might thus block the progress of the solver.

## 7.2 Invalidity-Aware Quadrature Rules

Our algorithm can output additional information to generate adaptive quadrature schemes: The goal is to produce a set of quadrature points for the static element 𝜎 such that at least one of the points would end up in an invalid region at time 𝑡 ∗ , correctly making the integral computed using quadrature diverge at that time.

Fig. 8. The function is infinite in the bright yellow region, and its integral over the whole triangle is also infinite. Numerical integration using a fixed quadrature rule (left) erroneously produces a finite value. Our adaptive quadrature technique (right) puts quadrature points in the infinite-valued region and correctly captures the behavior of the function.

<!-- image -->

Tracking Subdivisions. In Algorithm 1, each subdomain keeps track of the sequence of subdivisions of ¯ 𝜎 that were taken to produce it. However, since we only require a quadrature rule for 𝜎 and not ¯ 𝜎 , it is sufficient to keep track of spatial subdivisions: this means that when an element is subdivided in time only using 𝜓 + and 𝜓 -, the sequence of the children will match that of the parent; and when an element is subdivided in all dimensions using the subdivision scheme 𝜓 𝑞 for ¯ 𝜎 , pairs of subdomains that span the same region of space at different times will share the same sequence.

To produce the adaptive quadrature rule, our algorithm uses the sequence of spatial subdivisions of the element whose minimum inclusion function reduced 𝑡 ∗ for the last time, or equivalently, the element that found the earliest invalidity. Unless the early exit condition for maximum depth is triggered, such invalidity is guaranteed to be in [ 𝑡 ∗ , 𝑡 ∗ + 𝛿 ] . In the very rare cases when the algorithm fails to find an invalid point and gives up earlier, we instead return the subdivision sequence of the element with the deepest hierarchy, which is likely to be very close to an invalid region. In this case, the optimization step will still be guaranteed to be valid for its duration.

Adaptive Quadrature. This information is then used to partition the static element by recursively subdividing it using the very same sequence of subdivisions, as in Figure 8 (right). A standard quadrature rule is applied to every subelement of 𝜎 , and the integral is evaluated as the sum of integrals on all subdomains. However, in order to guarantee that the newly placed quadrature points would indeed intersect an invalid region in the full time step, it is required that the quadrature rule contain all points in the sampling set D 𝐷 used to compute the minimum inclusion function, projected onto 𝜎 ; for example, if D 𝐷 is the set of corners of ¯ 𝜎 , the selected scheme must place quadrature points at the corners of 𝜎 .

## 8 RESULTS

Our algorithm is implemented in C++, using PolyFEM [Schneider et al. 2019] for finite element (FE) system construction, IPC Toolkit [Ferguson et al. 2020] for evaluating IPC potentials and collision detection, Pardiso [Alappat et al. 2020; Bollhöfer et al. 2019, 2020]

Fig. 9. Initial and final frames of simulations from which queries are exported. The parts in cyan are used as handles to twist and compress the model while the deformation occurs in the parts in yellow.

<!-- image -->

for the large linear systems in our global Newton solves, [Attene 2019, 2020] for interval computation, GMP [Granlund and the GMP development team 2012] for rational computation, and OpenMP [Dagum and Menon 1998] for parallelization.

The simulation experiments are run on a cluster node with an Intel Cascade Lake Platinum 8268 processor limited to 16 threads and 32Gb of memory. The benchmark experiments are run singlethreaded on a laptop computer with an AMD Ryzen 7 4000 processor and 16GB of memory.

Our reference implementation, used to generate all results, and our evaluation datasets will be released as an open-source project.

## 8.1 Benchmark

Collection and Ground Truth. We collect queries of element inversion checks from elastodynamic simulation data using IPC [Li et al. 2020] and the Neo-Hookean elasticity model. We pick two 2D models and two 3D models shown in Figure 9. We bend the bar and compress the kangaroo in 2D, whereas for 3D models we twist them by 90 ◦ while compressing them by 20% .

Our benchmark contains 172000 2D queries from the Kangaroo model (orders 1 through 4), 54985 3D queries from the Armadillo model (orders 1 through 3), 19800 3D queries from the Bunny model (orders 1 through 3), 130291 queries for the Bar model of order 1, and 31879 queries for the Bar model of order 2 (Table 2).

Correctness. To validate correctness we compute the times of inversion by symbolically computing the roots of | 𝐽 𝑓 | with minimum 𝑡 using Mathematica [Wolfram Research 2023] (see Appendix D). We restrict the ground truth to order 1 and 2 for triangles, and order 1 for quads and tetrahedra, due to the limitations of symbolic solvers: for a higher-order basis, Mathematica could not return a result within 6 hours on some of the elements. To the best of our knowledge, ours is the first dataset containing conservative times of inversion. Our algorithm correctly detects all invalid elements and returns conservative answers for the inversion times.

Fig. 10. Statistics for the Kangaroo datasets. Top to bottom: elements of order 1, 2, 3, 4; Left to right: number of space subdivisions ( # ssub), time subdivisions ( # tsub), and time to test an element. Green valid elements; red invalid ( 𝑡 ∗ &lt; 1 ) elements.

<!-- image -->

Efficiency. The average per-element cost of our algorithm increases with the degree, from around 1.5 𝜇 s for order 1 (both in 2D and 3D) to around 10 𝜇 s for order 4 in 2D and 300 𝜇 s for order 3 in 3D. Invalid elements are more expensive to process on average, as they require subdivisions until precision is reached, whereas most valid elements can be resolved in a single iteration if all their Bézier coefficients are positive. See Table 2 and Figures 10 and 11 for details.

When restricted to the static case (i.e., check the validity of an element at a given time), our algorithm becomes considerably faster than the dynamic one (by a factor of about 50 × on the order 3 Armadillo), and it has a running time slightly faster ( 2 × ) than the non-conservative static baseline used in PolyFEM and FEBio, which consists of checking the sign of | 𝐽 𝑥 | obtained with (inexact) floating point computations only at quadrature points thanks to our optimizations. See Table 3 for details.

Table 2. Results of our continuous validity test on 2D and 3D datasets element types and orders. We report the number of elements (total, valid on the whole interval, invalid at some time) and processing time in microseconds per element (average, average for valid elements, average for invalid elements, median, maximum, standard deviation). On some datasets, marked with an asterisk (*), the algorithm 'gives up' on very few elements and returns a conservative answer: see details in Section 9. In all other cases, the algorithm reaches target precision on all elements.

| Dataset   | Element     |   𝑛 |   𝑠 |      𝑝 | Element count   | Element count   | Element count   | Time per element ( 𝜇 s)   | Time per element ( 𝜇 s)   | Time per element ( 𝜇 s)   | Time per element ( 𝜇 s)   | Time per element ( 𝜇 s)   | Time per element ( 𝜇 s)   |
|-----------|-------------|-----|-----|--------|-----------------|-----------------|-----------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|---------------------------|
|           |             |     |     |        | tot             | val             | inv             | avg                       | avg val                   | avg inv                   | med                       | max                       | std                       |
| Kangaroo  | Triangles   |   2 |   2 |      1 | 172800          | 146248          | 26552           | 1.60                      | 1.32                      | 3.10                      | 1.40                      | 205.40                    | 0.85                      |
|           |             |     |   2 |        | 172800          | 145602          | 27198           | 2.14                      | 1.69                      | 4.54                      | 1.89                      | 42.53                     | 1.10                      |
|           |             |     |   3 | 172800 |                 | 145623          | 27177           | 3.83                      | 3.10                      | 7.75                      | 3.28                      | 67.26                     | 1.86                      |
|           |             |     |   4 | 172800 |                 | 145613          | 27187           | 9.28                      | 7.94                      | 16.45                     | 7.96                      | 309.19                    | 4.03                      |
| Bar 2D    | Quadrangles |   2 |   1 |      1 | 112887          | 14816           | 98071           | 3.93                      | 2.95                      | 4.07                      | 3.84                      | 10203.74                  | 34.23                     |
|           |             |     |   2 |        | 83653           | 75085           | 8568            | 74.67                     | 7.89                      | 659.96                    | 2.93                      | 2046480.70                | 10296.53                  |
| Armadillo | Tetrahedra  |   3 |   3 |      1 | 54985           | 51605           | 3380            | 1.56                      | 1.44                      | 3.43                      | 1.47                      | 21.58                     | 0.53                      |
|           |             |     |   2 |        | 54985           | 48599           | 6386            | 9.90                      | 7.34                      | 29.41                     | 5.31                      | 6726.90                   | 40.37                     |
|           |             |     |   3 |        | 54985           | 47988           | 6997            | 362.79                    | 130.92                    | 1953.05                   | 44.00                     | 78411.99                  | 1593.81                   |
| Bunny     | Tetrahedra  |   3 |   3 |      1 | 19800           | 19561           | 239             | 1.47                      | 1.45                      | 3.51                      | 1.40                      | 49.38                     | 0.63                      |
|           |             |     |   2 |        | 19800           | 19058           | 742             | 7.27                      | 5.91                      | 42.22                     | 5.31                      | 1729.90                   | 24.56                     |
|           |             |     |   3 |        | 19800           | 18954           | 846             | 175.41                    | 65.97                     | 2627.34                   | 43.02                     | 60907.39                  | 1294.73                   |
| Bar 3D    | Hexahedra   |   3 |   1 |      1 | 56031           | 24918           | 31113*          | 325.33                    | 11.10                     | 576.99                    | 16.55                     | 6650064.32                | 39487.14                  |
|           | Prisms      |   3 |   2 |      1 | 82403           | 56913           | 25490*          | 246.64                    | 8.22                      | 778.96                    | 2.38                      | 2407415.47                | 16401.57                  |

## 8.2 Comparisons

We are not aware of any other algorithm that provides a continuous validity check for elements of arbitrary order, so we compare it with algorithms that solve only a subset of the problem.

Linear Continuous. For the special case of triangular elements with linear basis, [Smith and Schaefer 2015] proposes to use a symbolic solver to find the roots of | 𝐽 𝑓 | . While extremely efficient (1 𝜇 s on average), this approach can fail to produce correct results due to numerical errors: on the 26552 invalid elements in the linear Kangaroo dataset (Figure 9), their method fails to detect inversions 13324 times, producing a time step larger than the ground truth maximum. Attempting to be conservative with a "large" numerical threshold of 10 -5 still fails in 1563 tests ( ∼ 5% ).

This method is limited to linear triangles, and cannot be extended to other elements or degrees due to its reliance on closed-form expressions for the roots.

Static High Order. For the special case of static validity check for elements of arbitrary type and order, [Johnen et al. 2014] introduces a method based on adaptive subdivision. We implemented and tested their method in double floating-point precision, and while more efficient than our static approach (on the 3D Armadillo model of order 3 our method takes 3 𝜇𝑠 longer per element on average, see Table 3), it is not conservative. We provide one example of an inverted element incorrectly detected as valid by our implementation of the algorithm in [Johnen et al. 2014] in Appendix E, whose code is attached to the submission.

Table 3. Comparison of methods for static validity checks, performed at 𝑡 = 1 on the order 3 Armadillo dataset. We list the number of detected valid and invalid elements, the number of elements for which the test was undecided, and the average and median computation times per element. Sampling at quadrature points is fast, but incorrectly classifies several invalid elements as valid; bisection with a robust interval-based inclusion function is fast on "easy" elements, but struggles a lot with nearly-inverted elements and fails to classify several of them; our implementation of the Bézier refinement based inclusion check from [Johnen et al. 2014] and our conservative method give the same results; our method is guaranteed correct at a slight performance cost. Our precomputations (last two rows) decrease the computation time by at least an order of magnitude.

| Static algorithm      |   #val |   #inv | #und   |   avg 𝜇 s |   med 𝜇 s |
|-----------------------|--------|--------|--------|-----------|-----------|
| Quadrature Points     |  48214 |   6771 | -      |        16 |        16 |
| Interval Bisection    |  46281 |   6561 | 2143   |      1400 |        23 |
| FP Bézier (no optim.) |  48050 |   6935 | 0      |        71 |        78 |
| Ours (no optim.)      |  48050 |   6935 | 0      |        86 |        95 |
| FP Bézier             |  48050 |   6935 | 0      |         5 |         5 |
| Ours                  |  48050 |   6935 | 0      |         8 |         8 |

## 8.3 Elastodynamic Simulation

We integrate our check within the PolyFEM software [Schneider et al. 2019] and use it to reproduce a selection of the bundled elastodynamic simulations. We report our findings as we integrated the check, since they highlight some fundamental issues with existing high-order FE solvers and the non-linear material models commonly used in graphics and engineering.

Potential Formulation. In the simulation, we minimize the NeoHookean energy with Newton's method. The Neo-Hookean energy density has the form of

Table 4. Simulation Statistics. Columns from left to right: simulation dimension, geometric and solution bases orders, number of cells, number of time steps, peak memory usage, the average time of the simulation per time step, the average time of our check per time step, the average time of each check over the entire mesh, and average time of the simulation per time step using the quadrature point check in PolyFEM instead of ours.

|                   |     | Order   | Order   |         |         | Mem    | Timing ( sec )   | Timing ( sec )      | Timing ( sec )   | Timing ( sec )   |
|-------------------|-----|---------|---------|---------|---------|--------|------------------|---------------------|------------------|------------------|
|                   | dim | geom    | soln    | # Cells | # Steps | ( GB ) | time step        | check per time step | check avg        | baseline         |
| Beam-twist        | 3   | 1       | 2       | 1740    | 400     | 1.4    | 16               | 5.7                 | 0.067            | 7.6              |
| Ring-twist        | 2   | 1       | 2       | 1136    | 100     | 0.7    | 0.72             | 0.16                | 0.011            | 0.15             |
| Mat-twist         | 3   | 1       | 2       | 2166    | 250     | 1.3    | 10               | 2.9                 | 0.069            | 6.9              |
| Armadillo-rollers | 3   | 1       | 2       | 5978    | 230     | 3.3    | 114              | 27                  | 0.19             | 111              |
| Microstructure    | 3   | 4       | 2       | 6414    | 25      | 2.9    | 531              | 472                 | 5.6              | 52               |

Fig. 11. Statistics for the Armadillo datasets. Top to bottom: elements of order 1, 2, 3; Left to right: number of space subdivisions ( # ssub), time subdivisions ( # tsub), and time to test an element. Green valid elements; red invalid ( 𝑡 ∗ &lt; 1 ) elements.

<!-- image -->

<!-- formula-not-decoded -->

where 𝐹 is the deformation gradient matrix ( 2 × 2 or 3 × 3 ), 𝜆 and 𝜇 are Lamé parameters. Suppose the Jacobian of the deformed element and the rest element is 𝐽 𝑑 and 𝐽 𝑟 , the deformation gradient is given by

Fig. 12. Simulation examples in [Ferguson et al. 2023] with (right) and without (left) our inversion check. The high-order elements with non-positive Jacobian points are shown in red. On the left, the numbers of flipped elements are 20, 45, and 24 from top to bottom. Our method guarantees the positivity of Jacobian.

<!-- image -->

Fig. 13. Von Mises stress distribution of Ring-twist in Figure 1. The NaN is shown in red. (A) Without our method, NaN appears on elements with flipped points. (B) With our Jacobian check and adaptive quadrature, the stress is everywhere finite.

<!-- image -->

<!-- formula-not-decoded -->

Suppose the rest element is valid, i.e., det 𝐽 𝑟 &gt; 0 ; then the positivity of det 𝐹 = det 𝐽 𝑑 / det 𝐽 𝑟 depends solely on det 𝐽 𝑑 , i.e. the Jacobian determinant that we are checking. As long as det 𝐽 𝑑 &gt; 0 , Equation (5) is valid.

Baseline Simulator. Our baseline simulator is PolyFEM [Schneider et al. 2019], using the convergent IPC formulation [Li et al. 2023b] on tetrahedral meshes, quadratic Lagrangian bases and the NeoHookean material model [Ogden 2013]. PolyFEM uses only a static inversion check on the quadrature points: in Figures 1 and 12 we show that the final result contains many invalid elements leading to NaN in stress (Figure 13A). These invalid elements are not detected using the check on quadrature points alone, but are correctly identified by our conservative check (Figure 13B).

Conservative Line-Search Only. Replacing the static sampling invalidity check with our conservative and continuous check without adaptive quadrature leads to convergence issues in the solver. When the elements become close to inversion, the negative gradient direction is not a descending direction. This happens due to the error in the numerical integration of the potential, that can be solved by using our adaptive quadrature method. All the simulations in our experiments suffer from this issue: the simulation halts after several steps (twist-beam fails at time step 145, armadillo-rollers at 33, mat-twist at 141, ring-twist at 28, and the microstructure at 7).

Conservative Line-Search + Adaptive Quadrature. This solution works but dramatically increases the number of non-linear iterations needed in the Newton method. The remaining issue is that, while the Neo-Hookean energy density is infinite at the point where the Jacobian determinant is zero, it grows at a slow rate. Mathematically, when the Jacobian determinant is exactly zero at one point and positive everywhere else, the integral of the Neo-Hookean energy still may be finite: consider the integral of a 1D function 𝑓 ( 𝑥 ) = -log (| 𝑥 |) on the interval [-1 , 1 ] , although it has a singularity at 𝑥 = 0 , the integral is

<!-- formula-not-decoded -->

This is undesired since it leads to NaN in Equation (5) and stress evaluation, causing failure of the simulation. This problem can be mitigated by adding an additional barrier term 𝜇 det 𝐹 to Equation (5). While this solution formally fixes the problem (and also practically fixes it in our experiments), it makes the potential harder to minimize while changing the material model: we are not aware of any analysis of this problem, and we believe it is an exciting avenue for future work.

Observations. With these modifications, PolyFEM produces results without invalid elements. We note that given that the material models are infinite for invalid elements, the existing nonconservative approaches often create solutions that are non-physical. A more detailed study of the effect of these errors could be an interesting venue for future work.

A second observation is that the issue with the slow growth of the Neo-Hookean potential (and others, such as Mooney-Rivlin)

is likely due to the misuse of these models for deformations that are outside of the regime they are designed to handle. It would be interesting to carefully study experimentally how accurate these material models are under extreme deformations, and see if it is possible to design other material models which are numerically more suitable for interior point optimization.

High-Order IPC. Wereproduce simulation examples that use highorder finite elements in [Ferguson et al. 2023], including Mat-twist, Armadillo-rollers, and Microstructure (Figure 12). In Figure 1, we include two more examples: Beam-twist and Ring-twist. In the Beamtwist, we apply Dirichlet boundary conditions on the two sides of the beam, rotate one side, and keep the other side fixed; in the Ringtwist, we apply Dirichlet boundary conditions to rotate the inner circle of the ring with constant speed and allow the outer circle free to move. In Figure 12, the simulation results in [Ferguson et al. 2023] have flipped elements since the simulator only checks Jacobian at quadrature points in each element, while with our conservative check and adaptive quadrature, there is no flipped element. We report the statistics in Table 4. For quadratic elements, the runtime of our method is at worst comparable to the solve time and can be as fast as 23% of the solve time; for quartic geometric elements and quadratic solutions, our method is much slower than the solve time, since the solve is on quadratic elements while the Jacobian check should be performed on the quartic elements.

## 9 CONCLUSIONS

We introduced a formulation for continuous inversion test and a corresponding conservative and efficient algorithm. Our solution addresses an open problem in existing finite element solver and parametrization algorithms, increasing robustness and providing, for the first time, a guarantee for interior point solvers to stay within the space of valid elements. While the issue does appear for linear elements, invalid elements are more commonly present in existing algorithms that use high-order bases. We believe our algorithm, and its reference implementation, will be a drop-in replacement for existing non-conservative checks used in many graphics algorithms that will increase robustness for a minor performance cost.

Limitations. The cost of our algorithm increases with the polynomial degree, especially for tensor product elements. While this is compatible with many applications in graphics and engineering (up to cubic tetrahedra, linear hexahedra/prisms), it is not scalable to very high-order FEM, where elements of order 20 or more are routinely used.

Additionally, similarly to all bisection algorithms, our approach might require a lot of refinement in certain queries where it is challenging to find the roots: while we could not find such a case for simplicial elements, our dataset contains some tensor product elements (57 hexahedra and 26 prisms) for which the algorithm exceeds the available memory on the personal machine where the benchmark was run, and must be stopped early via the 𝑙 max parameter (set to a maximum subdivision depth of 7): our algorithm still returns a conservative answer. Altering the subdivision strategy mitigates this issue and allows the algorithm to converge on the whole dataset: exploring better heuristics for subdivision may be a fruitful endeavor for improvement.

Future Work. Thanks to our approach, we discovered a previously unreported numerical problem with existing material models under extreme compression, which was obfuscated by inaccurate quadrature rules. We believe evaluating these material models in extreme compression regimes would be an interesting avenue for future work. Additionally, we discovered that high-order elastodynamic FE solvers often introduce inverted elements in their solutions: their effect on simulation accuracy requires further investigation, which is now possible since our test can detect them.

To further reduce the computational cost, it would be interesting to investigate the possibility of parallelizing individual queries or adapting our approach for massively parallel graphic processing units.

## REFERENCES

- Christie Alappat, Achim Basermann, Alan R. Bishop, Holger Fehske, Georg Hager, Olaf Schenk, Jonas Thies, and Gerhard Wellein. 2020. A Recursive Algebraic Coloring Technique for Hardware-Efficient Symmetric Sparse Matrix-Vector Multiplication. ACM Transactions on Parallel Computing 7, 3, Article 19 (June 2020), 37 pages.
- R. W. Anderson, V. A. Dobrev, Tz. V. Kolev, and R. N. Rieben. 2014. Monotonicity in highorder curvilinear finite element arbitrary Lagrangian-Eulerian remap. International Journal for Numerical Methods in Fluids 77, 5 (Oct. 2014), 249-273. https://doi.org/ 10.1002/fld.3965
- M. Attene. 2019. Indirect Predicates Library. https://github.com/MarcoAttene/Indirect\_ Predicates.
- M. Attene. 2020. Indirect predicates for Geometric Constructions. Computer-Aided Design 126 (2020), 102856.
- A. W. Bargteil and E. Cohen. 2014. Animation of Deformable Bodies with Quadratic Bézier Finite Elements. ACM Trans. Graph. 33, 3 (May 2014), 1-10.
- Matthias Bollhöfer, Aryan Eftekhari, Simon Scheidegger, and Olaf Schenk. 2019. Largescale Sparse Inverse Covariance Matrix Estimation. SIAM Journal on Scientific Computing 41, 1 (2019), 380-401.
- Matthias Bollhöfer, Olaf Schenk, Radim Janalik, Steve Hamm, and Kiran Gullapalli. 2020. State-of-the-Art Sparse Direct Solvers. Parallel Algorithms in Computational Science and Engineering (2020), 3-33.
- H. Brönnimann, C. Burnikel, and S. Pion. 1998. Interval Arithmetic Yields Efficient Dynamic Filters for Computational Geometry. In Proc. 14th Symp. on Comp. Geom. (Minneapolis, Minnesota, USA). ACM, New York, NY, USA, 165-174.
- Yunuo Chen, Minchen Li, Lei Lan, Hao Su, Yin Yang, and Chenfanfu Jiang. 2022. A unified newton barrier method for multibody dynamics. ACM Trans. Graph. 41, 4, Article 66 (July 2022), 14 pages. https://doi.org/10.1145/3528223.3530076
- J. A. Cottrell, T. J. R. Hughes, and Y. Bazilevs. 2009. Isogeometric Analysis: Toward Integration of CAD and FEA . Wiley.
- Leonardo Dagum and Ramesh Menon. 1998. OpenMP: An Industry-Standard API for Shared-Memory Programming. IEEE Comput. Sci. Eng. 5, 1 (jan 1998), 46-55. https://doi.org/10.1109/99.660313
- S. Dey, R.M. O'Bara, and M.S. Shephard. 2001. Towards curvilinear meshing in 3D: the case of quadratic simplices. Computer-Aided Design 33, 3 (2001), 199-209.
- V. Dobrev, P. Knupp, T. Kolev, K. Mittal, and V. Tomov. 2019. The Target-Matrix Optimization Paradigm for High-Order Meshes. SIAM Jou. Sci. Comp. 41, 1 (2019), B50-B68.
- Yu Fang, Minchen Li, Yadi Cao, Xuan Li, Joshuah Wolper, Yin Yang, and Chenfanfu Jiang. 2024. Augmented Incremental Potential Contact for Sticky Interactions. IEEE Transactions on Visualization and Computer Graphics 30, 8 (2024), 5596-5608. https://doi.org/10.1109/TVCG.2023.3295656
- Yu Fang, Minchen Li, Chenfanfu Jiang, and Danny M. Kaufman. 2021. Guaranteed globally injective 3D deformation processing. ACM Trans. Graph. 40, 4, Article 75 (July 2021), 13 pages. https://doi.org/10.1145/3450626.3459757
- G. Farin. 2001. Curves and Surfaces for CAGD: A Practical Guide (5th ed.). Morgan Kaufmann Publishers Inc., San Francisco, CA, USA.
- Zachary Ferguson et al. 2020. IPC Toolkit. https://ipc-sim.github.io/ipc-toolkit/. https: //ipc-sim.github.io/ipc-toolkit/
- Z. Ferguson, P. Jain, D. Zorin, T. Schneider, and D. Panozzo. 2023. High-Order Incremental Potential Contact for Elastodynamic Simulation on Curved Meshes. In ACM SIGGRAPH 2023 Conference Proceedings (, Los Angeles, CA, USA,) (SIGGRAPH '23) . Association for Computing Machinery, New York, NY, USA, Article 77, 11 pages.

·

- Zachary Ferguson, Minchen Li, Teseo Schneider, Francisca Gil-Ureta, Timothy Langlois, Chenfanfu Jiang, Denis Zorin, Danny M. Kaufman, and Daniele Panozzo. 2021. Intersection-free rigid body dynamics. ACM Trans. Graph. 40, 4, Article 183 (July 2021), 16 pages. https://doi.org/10.1145/3450626.3459802
- A. Gargallo-Peiró, X. Roca, J. Peraire, and J. Sarrate. 2015. Distortion and quality measures for validating and generating high-order tetrahedral meshes. Engineering with Computers 31, 3 (2015), 423-437.
- C. Geuzaine, A. Johnen, J. Lambrechts, J. F. Remacle, and T. Toulorge. 2015. The Generation of Valid Curvilinear Meshes . Springer International Publishing, Cham, 15-39.
- F. Goualard. 2005. Gaol: NOT Just Another Interval Library. https://sourceforge.net/ projects/gaol/.
- T. Granlund and the GMP development team. 2012. GNUMP:TheGNUMultiple Precision Arithmetic Library (5.0.5 ed.). http://gmplib.org/.
- Kemeng Huang, Floyd M. Chitalu, Huancheng Lin, and Taku Komura. 2024a. GIPC: Fast and Stable Gauss-Newton Optimization of IPC Barrier Energy. ACM Trans. Graph. 43, 2, Article 23 (March 2024), 18 pages. https://doi.org/10.1145/3643028
- Zizhou Huang, Davi Colli Tozoni, Arvi Gjoka, Zachary Ferguson, Teseo Schneider, Daniele Panozzo, and Denis Zorin. 2024b. Differentiable solver for time-dependent deformation problems with contact. ACM Trans. Graph. 43, 3, Article 31 (May 2024), 30 pages. https://doi.org/10.1145/3657648
- Z. Jiang, Z. Zhang, Y. Hu, T. Schneider, D. Zorin, and D. Panozzo. 2021a. Bijective and coarse high-order tetrahedral meshes. ACM Trans. Graph. 40, 4 (2021), 1-16.
- Z. Jiang, Z. Zhang, Y. Hu, T. Schneider, D. Zorin, and D. Panozzo. 2021b. Bijective and coarse high-order tetrahedral meshes. ACM Trans. Graph. 40, 4 (2021), 1-16.
- A. Johnen, C. Geuzaine, T. Toulorge, and J.-F. Remacle. 2018. Efficient computation of the minimum of shape quality measures on curvilinear finite elements. Computer-Aided Design 103 (2018), 24-33.
- A. Johnen, J.-F. Remacle, and C. Geuzaine. 2013. Geometrical validity of curvilinear finite elements. J. Comput. Phys. 233 (2013), 359-372. https://doi.org/10.1016/j.jcp. 2012.08.051
- A. Johnen, J. F. Remacle, and C. Geuzaine. 2014. Geometrical Validity of High-Order Triangular Finite Elements. Eng. with Comput. 30, 3 (2014), 375-382.
- A. Johnen, J.-C. Weill, and J.-F. Remacle. 2017. Robust and efficient validation of the linear hexahedral element. Procedia Engineering 203 (2017), 271-283.
- C. Kane, J. E. Marsden, M. Ortiz, and M. West. 2000. Variational integrators and the Newmark algorithm for conservative and dissipative mechanical systems. Internat. J. Numer. Methods Engrg. 49, 10 (2000), 1295-1325.
- OKnuppel. 1994. PROFIL/BIAS - A fast interval library. Computing 53, 3 (1994), 277-287. https://doi.org/10.1007/BF02307379
- B. Lambov. 2008. Interval Arithmetic Using SSE-2. In Reliable Implementation of Real Number Algorithms: Theory and Practice , P. Hertling, C. M. Hoffmann, W. Luther, and N. Revol (Eds.). Springer Berlin Heidelberg, Berlin, Heidelberg, 102-113.
- Lei Lan, Danny M. Kaufman, Minchen Li, Chenfanfu Jiang, and Yin Yang. 2022a. Affine body dynamics: fast, stable and intersection-free simulation of stiff materials. ACM Trans. Graph. 41, 4, Article 67 (July 2022), 14 pages. https://doi.org/10.1145/3528223. 3530064
- Lei Lan, Minchen Li, Chenfanfu Jiang, Huamin Wang, and Yin Yang. 2023. Second-order Stencil Descent for Interior-point Hyperelasticity. ACM Trans. Graph. 42, 4, Article 108 (July 2023), 16 pages. https://doi.org/10.1145/3592104
- Lei Lan, Guanqun Ma, Yin Yang, Changxi Zheng, Minchen Li, and Chenfanfu Jiang. 2022b. Penetration-free projective dynamics on the GPU. ACM Trans. Graph. 41, 4, Article 69 (July 2022), 16 pages. https://doi.org/10.1145/3528223.3530069
- Lei Lan, Yin Yang, Danny Kaufman, Junfeng Yao, Minchen Li, and Chenfanfu Jiang. 2021. Medial IPC: accelerated incremental potential contact with medial elastics. ACM Trans. Graph. 40, 4, Article 158 (July 2021), 16 pages. https://doi.org/10.1145/ 3450626.3459753
- M. Lerch, G. Tischler, J.W.V. Gudenberg, W. Hofschuster, and W. Kramer. 2006. FILIB++, a Fast Interval Library Supporting Containment Computations. ACM Trans. Math. Softw 32, 2 (2006), 299-324.
- M. Li, Z. Ferguson, T. Schneider, T. Langlois, D. Zorin, D. Panozzo, C. Jiang, and D. M. Kaufman. 2020. Incremental Potential Contact: Intersection- and Inversion-free Large Deformation Dynamics. ACM Trans. Graph. (SIGGRAPH) 39, 4, Article 49 (2020).
- Minchen Li, Zachary Ferguson, Teseo Schneider, Timothy Langlois, Denis Zorin, Daniele Panozzo, Chenfanfu Jiang, and Danny M. Kaufman. 2023b. Convergent Incremental Potential Contact. arXiv:2307.15908 [math.NA]
- Minchen Li, Danny M. Kaufman, and Chenfanfu Jiang. 2021. Codimensional incremental potential contact. ACM Trans. Graph. 40, 4, Article 170 (July 2021), 24 pages. https: //doi.org/10.1145/3450626.3459767
- Xuan Li, Yu Fang, Lei Lan, Huamin Wang, Yin Yang, Minchen Li, and Chenfanfu Jiang. 2023a. Subspace-Preconditioned GPU Projective Dynamics with Contact for Cloth Simulation. In SIGGRAPH Asia 2023 Conference Papers (Sydney, NSW, Australia) (SA '23) . Association for Computing Machinery, New York, NY, USA, Article 1, 12 pages. https://doi.org/10.1145/3610548.3618157

- Xuan Li, Yu Fang, Minchen Li, and Chenfanfu Jiang. 2022. BFEMP: Interpenetration-free MPM-FEM coupling with barrier contact. Computer Methods in Applied Mechanics and Engineering 390 (2 2022). https://doi.org/10.1016/j.cma.2021.114350
- Xuan Li, Minchen Li, Xuchen Han, Huamin Wang, Yin Yang, and Chenfanfu Jiang. 2024. A Dynamic Duo of Finite Elements and Material Points. In ACM SIGGRAPH 2024 Conference Papers (Denver, CO, USA) (SIGGRAPH '24) . Association for Computing Machinery, New York, NY, USA, Article 97, 11 pages. https://doi.org/10.1145/ 3641519.3657449
- S. A. Maas, B. J. Ellis, G. A. Ateshian, and J. A. Weiss. 2012. FEBio: Finite Elements for Biomechanics. Jou. of Biomechanical Engineering 134, 1 (2012).
- M. Mandad and M. Campen. 2020. Efficient piecewise higher-order parametrization of discrete surfaces with local and global injectivity. Computer-Aided Design 127 (2020), 102862.
- Z. Marschner, D. Palmer, P. Zhang, and J. Solomon. 2020. Hexahedral Mesh Repair via Sum-of-Squares Relaxation. Computer Graphics Forum 39, 5 (Aug. 2020), 133-147.
- L. Martin, P. Jain, Z. Ferguson, T. Gholamalizadeh, F. Moshfeghifar, K. Erleben, D. Panozzo, S. Abramowitch, and T. Schneider. 2024. A systematic comparison between FEBio and PolyFEM for biomechanical systems. Computer Methods and Programs in Biomedicine 244 (2024), 107938.
- Aaron Meurer, Christopher P. Smith, Mateusz Paprocki, Ondřej Čertík, Sergey B. Kirpichev, Matthew Rocklin, AMiT Kumar, Sergiu Ivanov, Jason K. Moore, Sartaj Singh, Thilina Rathnayake, Sean Vig, Brian E. Granger, Richard P. Muller, Francesco Bonazzi, Harsh Gupta, Shivam Vats, Fredrik Johansson, Fabian Pedregosa, Matthew J. Curry, Andy R. Terrel, Štěpán Roučka, Ashutosh Saboo, Isuru Fernando, Sumith Kulal, Robert Cimrman, and Anthony Scopatz. 2017. SymPy: symbolic computing in Python. PeerJ Computer Science 3 (Jan. 2017), e103. https://doi.org/10.7717/peerjcs.103
- J. Mezger, B. Thomaszewski, S. Pabst, and W. Straßer. 2009. Interactive physically-based shape editing. Computer Aided Geometric Design 26, 6 (Aug. 2009), 680-694.
- Steven S Muchnick. 1997. Advanced Compiler Design and Implementation . Morgan Kaufmann, Oxford, England.
- R.W. Ogden. 2013. Non-Linear Elastic Deformations . Dover Publications. https://books. google.com/books?id=52XDAgAAQBAJ
- Julian Panetta, Qingnan Zhou, Luigi Malomo, Nico Pietroni, Paolo Cignoni, and Denis Zorin. 2015. Elastic textures for additive fabrication. ACM Transactions on Graphics 34, 4 (July 2015), 1-12. https://doi.org/10.1145/2766937
- B. Schling. 2011. The Boost C++ Libraries. XML Press (2011).
- T. Schneider, J. Dumas, X. Gao, D. Zorin, and D. Panozzo. 2019. PolyFEM. https: //polyfem.github.io/.
- T. Schneider, Y. Hu, X. Gao, J. Dumas, D. Zorin, and D. Panozzo. 2022. A Large-Scale Comparison of Tetrahedral and Hexahedral Elements for Solving Elliptic PDEs with the Finite Element Method. ACM Trans. on Graph. 41, 3 (2022), 1-14.
- Xing Shen, Runyuan Cai, Mengxiao Bi, and Tangjie Lv. 2024. Preconditioned Nonlinear Conjugate Gradient Method for Real-time Interior-point Hyperelasticity. In ACM SIGGRAPH 2024 Conference Papers (Denver, CO, USA) (SIGGRAPH '24) . Association for Computing Machinery, New York, NY, USA, Article 96, 11 pages. https://doi. org/10.1145/3641519.3657490
- J. R. Shewchuk. 1997. Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates. Discrete &amp; Computational Geometry 18, 3 (Oct. 1997), 305-363.
- J. Smith and S. Schaefer. 2015. Bijective Parameterization with Free Boundaries. ACM Trans. Graph. 34, 4, Article 70 (2015), 9 pages.
- J Snyder. 1992. Interval Analysis For Computer Graphics. ACM SIGGRAPH (1992), 121-130.
- S. Suwelack, D. Lukarski, V. Heuveline, R. Dillmann, and S. Speidel. 2013. Accurate surface embedding for higher order finite elements. In Proceedings of the 12th ACM SIGGRAPH/Eurographics Symposium on Computer Animation (SCA '13) . ACM.
- X. Tang, Z. Ferguson, T. Schneider, D. Zorin, S. Kamil, and D. Panozzo. 2023. A CrossPlatform Benchmark for Interval Computation Libraries. In Parallel Processing and Applied Mathematics , R. Wyrzykowski, J. Dongarra, E. Deelman, and K. Karczewski (Eds.). Springer International Publishing, Cham, 415-427.
- Thomas Toulorge, Christophe Geuzaine, Jean-François Remacle, and Jonathan Lambrechts. 2013. Robust untangling of curvilinear meshes. J. Comput. Phys. 254 (Dec. 2013), 8-26. https://doi.org/10.1016/j.jcp.2013.07.022
- O. V. Ushakova. 2011. Nondegeneracy tests for hexahedral cells. Computer Methods in Applied Mechanics and Engineering 200, 17-20 (2011), 1649-1658.
- S. Vavasis. 2003. A Bernstein-Bezier Sufficient Condition for Invertibility of Polynomial Mapping Functions. (2003).
- Bolun Wang, Zachary Ferguson, Teseo Schneider, Xin Jiang, Marco Attene, and Daniele Panozzo. 2021a. A Large-scale Benchmark and an Inclusion-based Algorithm for Continuous Collision Detection. ACM Trans. Graph. 40, 5, Article 188 (sep 2021), 16 pages. https://doi.org/10.1145/3460775
- B. Wang, Z. Ferguson, T. Schneider, X. Jiang, M. Attene, and D. Panozzo. 2021b. A Large-scale Benchmark and an Inclusion-based Algorithm for Continuous Collision Detection. ACM Trans. Graph. 40, 5 (2021), 1-16.
- Inc. Wolfram Research. 2023. Mathematica, Version 13.3. https://www.wolfram.com/ mathematica Champaign, IL.

## A LANGRANGE AND BÉZIER FORMS OF | 𝐽 𝑥 |

Let 𝑓 : 𝜎 𝑛 𝑠 -→ R be a polynomial of order 𝑝 defined on a standard element according to Definition 1. We detail the representation of 𝑓 in Lagrange and Bézier forms and the conversions between these two representations.

Lagrange form. Let Γ 𝑝 𝜎 = ( 1 / 𝑝 ) Z 𝑛 ∩ 𝜎 be a grid of uniformly distributed domain points of 𝜎 , and I 𝑝 𝜎 be its set of indices (Figure 5). The Lagrange basis of order 𝑝 consists of | Γ 𝑝 𝜎 | order 𝑝 polynomials such that for each point 𝛾 𝑖 ∈ Γ 𝑝 𝜎 , L 𝑝 𝑖 ( 𝛾 𝑗 ) = 𝛿 𝑖 𝑗 . A function 𝑓 is represented in the Lagrange basis as:

<!-- formula-not-decoded -->

where 𝑦 𝑖 = 𝑓 ( 𝛾 𝑖 ) for 𝑖 ∈ I 𝑝 𝜎 .

Bézier form. The same function 𝑓 can be expressed equivalently in Bézier form by using a Bernstein basis on the same set Γ 𝑝 𝜎 of domain points:

<!-- formula-not-decoded -->

where the B 𝑝 𝑖 are Bernstein polynomials of order 𝑝 and the 𝛽 𝑖 are their corresponding control coefficients. Unlike the Lagrange case, 𝛽 𝑖 equals 𝑓 ( 𝛾 𝑖 ) only at the corners of 𝜎 . However, the graph of 𝑓 is contained in the convex hull of points ( 𝛾 𝑖 , 𝛽 𝑖 ) , providing a simple way to bound 𝑓 from below and above at all points of 𝜎 [Farin 2001].

Transformation matrix. Let us denote 𝑓 L the vector consisting of all the 𝑦 𝑖 , and likewise, 𝑓 B the vector consisting of all the 𝛽 𝑖 , for 𝑖 ∈ I 𝑝 𝜎 . We can convert between the two representations through transformation matrices [Johnen et al. 2014]:

<!-- formula-not-decoded -->

Such matrices depend only on the reference element 𝜎 𝑛 𝑠 and the order 𝑝 thus can be computed once for each element type and order. Matrix T B→L is easily computed by evaluation of Bernstein polynomials on Γ 𝑝 𝜎 :

<!-- formula-not-decoded -->

and T L→B is the inverse of T B→L .

Subdivision matrices. For each 𝑞 , we first build a transformation matrix from the Bézier basis to the Lagrange basis that interpolates the domain points 𝜓 𝑞 ( Γ ( 𝑚 )) of the 𝑞 -th subdomain. This is analogous to building matrix T B→L , by sampling the Bézier basis on 𝜓 𝑞 ( Γ ( 𝑚 )) instead of Γ ( 𝑚 ) :

<!-- formula-not-decoded -->

Then we multiply it with the Lagrange-to-Bézier matrix to build

<!-- formula-not-decoded -->

which allows us to go directly from the Bézier coefficients on the domain to the Bézier coefficients on each subdomain.

Time subdivision matrices. Time subdivision matrices defined by plugging the time subdivision maps

<!-- formula-not-decoded -->

in Equation (10) and using Equation (11) as above.

Representations of the Jacobian determinant. Given a standard element 𝜎 𝑛 𝑠 as above, let us define 𝑚 = 𝑛𝑝 -𝑠 for simplicial elements and 𝑚 = 𝑛𝑝 -1 for tensor product and mixed elements. Let us denote 𝑖 0 = 𝑚 - ( ˝ 𝑠 𝑗 = 1 𝑖 𝑗 ) and 𝜉 0 = 1 - ( ˝ 𝑠 𝑗 = 1 𝜉 𝑗 ) .

The Lagrange basis polynomials used to represent the Jacobian determinant | 𝐽 𝑥 ( 𝜉 )| on reference element 𝜎 = 𝜎 𝑛 𝑠 are:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The Bézier basis polynomials used to represent the Jacobian determinant | 𝐽 𝑥 ( 𝜉 )| on reference element 𝜎 = 𝜎 𝑛 𝑠 are:

<!-- formula-not-decoded -->

where B 𝜎 𝑖 1 ,...,𝑖 𝑛 is the product of an order 𝑚 Bernstein polynomial on the 𝑠 -simplex basis, in the variables 𝜉 1 , . . . , 𝜉 𝑠 , with an order 𝑚 Bernstein polynomial on the ( 𝑛 -𝑠 ) -tensor product basis in the variables 𝜉 𝑠 + 1 , . . . , 𝜉 𝑛 , which is itself a product of ( 𝑛 -𝑠 ) univariate Bernstein polynomials of order 𝑚 . It is easy to see that

- the order of | 𝐽 𝑥 ( 𝜉 )| in { 𝜉 1 , . . . , 𝜉 𝑠 } is 𝑛𝑝 -𝑠 ;
- the order of | 𝐽 𝑥 ( 𝜉 )| in { 𝜉 𝑠 + 1 , . . . , 𝜉 𝑛 } is 𝑛𝑝 -1 .

Notice that on 𝑛 -simplices ( 𝑠 = 𝑛 ) and 𝑛 -hypercubes ( 𝑠 = 1 ), | 𝐽 𝑥 | has the same order in every variable.

Extension to Dynamic. For the dynamic case, we add a term for time to the Bézier basis construction, an order 𝑛 univariate Bernstein polynomial in 𝑡 :

<!-- formula-not-decoded -->

It is easy to see that the order of | 𝐽 ¯ 𝑥 | remains the same as the static case in its spatial variables, while it is 𝑛 in its time variable. The number of polynomials in the Bernstein basis, which is equal to the number of terms of the given polynomial, increases combinatorially with the dimension 𝑛 and order 𝑝 and is given by

<!-- formula-not-decoded -->

for the static and the dynamic case, respectively. Note that, this combinatorial growth poses intrinsic limitations to scaling the problem up in degree and dimension. Nevertheless, it remains tractable for the most common cases, as exemplified in Table 5.

| Element name            |   𝑛 |   𝑠 |   𝑝 |   𝜉 1 ...𝑠 | 𝜉 𝑠 + 1 ...𝑛   |   𝑡 |   ¯ 𝑁 |
|-------------------------|-----|-----|-----|------------|----------------|-----|-------|
| Linear triangle         |   2 |   2 |   1 |          0 | -              |   2 |     3 |
| Quadratic triangle      |   2 |   2 |   2 |          2 | -              |   2 |    18 |
| Cubic triangle          |   2 |   2 |   3 |          4 | -              |   2 |    45 |
| Quartic triangle        |   2 |   2 |   4 |          6 | -              |   2 |    84 |
| Quintic triangle        |   2 |   2 |   5 |          8 | -              |   2 |   135 |
| Bilinear quadrangle     |   2 |   1 |   1 |          1 | 1              |   2 |    12 |
| Biquadratic quadrangle  |   2 |   1 |   2 |          3 | 3              |   2 |    48 |
| Bicubic quadrangle      |   2 |   1 |   3 |          5 | 5              |   2 |   108 |
| Linear tetrahedron      |   3 |   3 |   1 |          0 | -              |   3 |     4 |
| Quadratic tetrahedron   |   3 |   3 |   2 |          3 | -              |   3 |    80 |
| Cubic tetrahedron       |   3 |   3 |   3 |          6 | -              |   3 |   256 |
| Quartic tetrahedron     |   3 |   3 |   4 |          9 | -              |   3 |   880 |
| Bilinear tri. prism     |   3 |   2 |   1 |          1 | 2              |   3 |    36 |
| Biquadratic tri. prism  |   3 |   2 |   2 |          4 | 5              |   3 |   270 |
| Trilinear hexahedron    |   3 |   1 |   1 |          2 | 2              |   3 |   108 |
| Triquadratic hexahedron |   3 |   1 |   2 |          5 | 5              |   3 |   864 |

Table 5. Order of | 𝐽 ¯ 𝑥 ( 𝜉 ) | in the spatial variables 𝜉 and the time variable 𝑡 , and the number ¯ 𝑁 of its terms for a generic order 𝑝 map on ¯ 𝜎 𝑛 𝑠 .

## B SUBDIVISION RULES

The subdivision functions 𝜓 𝑞 for the various types of elements are listed below. Their related subdomains are shown in the insets (red bullet = origin). The dynamic elements for 𝑛 = 3 require 4D hypercubes (for tensor product) and hyper-prisms (for simplicial and mixed).

Triangle.

<!-- image -->

Quad.

<!-- formula-not-decoded -->

<!-- image -->

Tetrahedron. The subdivision of the tetrahedron is non-trivial: it is split into four tetrahedra incident at the corners of the domain (corresponding to subdomains 1-4) and a central octahedron, which is further split into four tetrahedra (domains 5-8).

```
𝜓 1 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 2 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 3 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 ) 𝜓 4 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 5 ( 𝜉 ) = ( ( 1 -𝜉 2 -𝜉 3 )/ 2 , 𝜉 2 / 2 , ( 𝜉 1 + 𝜉 2 + 𝜉 3 )/ 2 ) 𝜓 6 ( 𝜉 ) = ( ( 1 -𝜉 2 )/ 2 , ( 𝜉 1 + 𝜉 2 )/ 2 , ( 𝜉 2 + 𝜉 3 )/ 2 ) 𝜓 7 ( 𝜉 ) = ( ( 𝜉 1 + 𝜉 2 )/ 2 , ( 1 -𝜉 1 )/ 2 , 𝜉 3 / 2 ) 𝜓 8 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 𝜉 3 )/ 2 , ( 1 -𝜉 1 + 𝜉 2 )/ 2 )
```

<!-- image -->

Hexa. The subdivision of a hexahedron works both for a static hexahedral domain and a dynamic quad domain. In the latter case, the third coordinate is time.

```
𝜓 1 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 2 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 3 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 ) 𝜓 4 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 ) 𝜓 5 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 6 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 7 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 8 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 )
```

<!-- image -->

Prism. The subdivision of a prism works both for a static prism domain and a dynamic triangle domain. In the latter case, the third coordinate is time.

```
𝜓 1 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 2 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 ) 𝜓 3 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 ) 𝜓 4 ( 𝜉 ) = ( ( 1 -𝜉 1 )/ 2 , ( 1 -𝜉 2 )/ 2 , 𝜉 3 / 2 ) 𝜓 5 ( 𝜉 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 6 ( 𝜉 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 7 ( 𝜉 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 ) 𝜓 8 ( 𝜉 ) = ( ( 1 -𝜉 1 )/ 2 , ( 1 -𝜉 2 )/ 2 , ( 𝜉 3 + 1 )/ 2 )
```

<!-- image -->

Hypercube. This is a 4-hypercube that works as a domain for a dynamic hexahedral element.

```
𝜓 1 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 , 𝑡 / 2 ) 𝜓 2 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 , 𝑡 / 2 ) 𝜓 3 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 4 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 5 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , 𝑡 / 2 ) 𝜓 6 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , 𝑡 / 2 ) 𝜓 7 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 , 𝑡 / 2 ) 𝜓 8 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 , 𝑡 / 2 ) 𝜓 9 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 10 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 11 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 12 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 13 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 14 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 15 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 16 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , ( 𝜉 2 + 1 )/ 2 , ( 𝜉 3 + 1 )/ 2 , ( 𝑡 + 1 )/ 2 )
```

Hyperprism 1. This is the tensor product of a tetrahedron with an interval that works as a domain for a dynamic tetrahedral element.

```
𝜓 1 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 2 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 3 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 4 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , 𝑡 / 2 ) 𝜓 5 ( 𝜉, 𝑡 ) = ( ( 1 -𝜉 2 -𝜉 3 )/ 2 , 𝜉 2 / 2 , ( 𝜉 1 + 𝜉 2 + 𝜉 3 )/ 2 , 𝑡 / 2 ) 𝜓 6 ( 𝜉, 𝑡 ) = ( ( 1 -𝜉 2 )/ 2 , ( 𝜉 1 + 𝜉 2 )/ 2 , ( 𝜉 2 + 𝜉 3 )/ 2 , 𝑡 / 2 ) 𝜓 7 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 𝜉 2 )/ 2 , ( 1 -𝜉 1 )/ 2 , 𝜉 3 / 2 , 𝑡 / 2 ) 𝜓 8 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 𝜉 3 )/ 2 , ( 1 -𝜉 1 + 𝜉 2 )/ 2 , 𝑡 / 2 ) 𝜓 9 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 10 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 1 )/ 2 , 𝜉 2 / 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 11 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 1 )/ 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 12 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , 𝜉 2 / 2 , ( 𝜉 3 + 1 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 13 ( 𝜉, 𝑡 ) = ( ( 1 -𝜉 2 -𝜉 3 )/ 2 , 𝜉 2 / 2 , ( 𝜉 1 + 𝜉 2 + 𝜉 3 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 14 ( 𝜉, 𝑡 ) = ( ( 1 -𝜉 2 )/ 2 , ( 𝜉 1 + 𝜉 2 )/ 2 , ( 𝜉 2 + 𝜉 3 )/ 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 15 ( 𝜉, 𝑡 ) = ( ( 𝜉 1 + 𝜉 2 )/ 2 , ( 1 -𝜉 1 )/ 2 , 𝜉 3 / 2 , ( 𝑡 + 1 )/ 2 ) 𝜓 16 ( 𝜉, 𝑡 ) = ( 𝜉 1 / 2 , ( 𝜉 2 + 𝜉 3 )/ 2 , ( 1 -𝜉 1 + 𝜉 2 )/ 2 , ( 𝑡 + 1 )/ 2 )
```

Hyperprism 2. This is the tensor product of a prism with an interval that works as a domain for a dynamic prism element.

<!-- formula-not-decoded -->

## C INTERVAL ARITHMETIC

The interval type we use defines the following operations and relations:

<!-- formula-not-decoded -->

The division between intervals should be avoided, as the divisor may contain 0. However, a division of an interval by an exact number is acceptable; in our code, we have only divisions by 2, which is however exact even in floating point.

Note that there is no total ordering of intervals, so for example, [ 𝑥, 𝑥 ] ≰ 0 ⇏ [ 𝑥, 𝑥 ] &gt; 0 .

In our implementation, outward rounding is achieved by internally storing the lower end of the interval with the opposite sign and changing the processor rounding mode to always rounding up. This ensures that the interval's width increases only when the actual floating-point computation is inexact.

## D GROUND TRUTH

We designed a Mathematica script that computes symbolically the roots of | 𝐽 ¯ 𝑥 | for a given element. The script receives in input the type, dimension 𝑛 , and order 𝑝 of an element together with the control points describing its geometric map at times 𝑡 = 0 and 𝑡 = 1 . This is the same input taken by our test. The script computes the symbolic expression of | 𝐽 ¯ 𝑥 | , which is a polynomial in 𝑛 + 1 variables, finds its roots inside the domain ¯ 𝜎 , and returns the minimum value of 𝑡 in which a root is found. First, we use FindInstance to check if the region where 𝐽 ≤ 0 is empty. If so, the element is valid throughout; otherwise, we use Minimize to find the minimum time 𝑡 in such a region. The scripts will be released as part of our open-source code.

·

## E FAILURE CASE FOR [Johnen et al. 2014]

We implemented the method described in [Johnen et al. 2014] for static validity with floating point arithmetic and constructed a failure case for the following invalid quadratic triangle (when tested with Mathematica). Control points v 00 , v 20 , and v 02 are the three vertices of the triangle, and v 01 , v 11 , v 10 are the edge midpoints.

```
v 00 = ( 0 . 33333333333333331 , 0 . 33333333333333331 ) v 20 = ( 1 . 3333333333333333 , 0 . 33333333333333331 ) v 02 = ( 0 . 33333333333333331 , 1 . 3333333333333333 ) v 01 = ( 0 . 83333333333333326 , 0 . 33333333333333331 ) v 11 = ( 0 . 83333333333333326 , 0 . 58333333333333315 ) v 10 = ( 0 . 33333333333333331 , 0 . 83333333333333326 )
```