## TiGL - An Open Source Computational Geometry Library for Parametric Aircraft Design

Martin Siggel, Jan Kleinert, Tobias Stollenwerk and Reinhold Maierl

Abstract. This paper introduces the software TiGL: TiGL is an open source high-fidelity geometry modeler that is used in the conceptual and preliminary aircraft and helicopter design phase. It creates full three-dimensional models of aircraft from their parametric CPACS description. Due to its parametric nature, it is typically used for aircraft design analysis and optimization. First, we present the use-case and architecture of TiGL. Then, we discuss it's geometry module, which is used to generate the B-spline based surfaces of the aircraft. The backbone of TiGL is its surface generator for curve network interpolation, based on Gordon surfaces. One major part of this paper explains the mathematical foundation of Gordon surfaces on B-splines and how we achieve the required curve network compatibility. Finally, TiGL's aircraft component module is introduced, which is used to create the external and internal parts of aircraft, such as wings, flaps, fuselages, engines or structural elements.

Mathematics Subject Classification (2010). 65D17, 65D05, 65D10.

Keywords. aircraft, geometry, Gordon surface, B-splines, CPACS.

## 1. Introduction

Optimizing airplane designs often requires a large consortium of engineers from many different fields to work together. Every group of engineers works with a specific set of simulation tools, but almost all tools require information about the current design's geometry. The TiGL Geometry Library (TiGL) [1] generates three-dimensional airplane geometries from a standardized parametric description. It is a piece of software developed mainly at the German Aerospace Center (DLR), in cooperation with Airbus Defense and Space and RISC Software GmbH. These geometries include the outer shape exposed to the surrounding airfield as well as the inner structure of the fuselage and wings that provides the necessary stability.

The Common Parametric Aircraft Configuration Schema (CPACS) is an exchange format for describing airplane design in form of an XML-file [2, 3]. Among other things, it contains a parametric description of the aircraft geometry that is interpreted by TiGL. TiGL offers the functionality to export these CPACS geometries to standard CAD formats such as IGES, STEP, VTK as well as functions to query points and curves on the airplanes surface. TiGL uses the OpenCASCADE CAD kernel [4] to model the geometries based on B-spline surfaces. Additional geometric modeling features are included on top of OpenCASCADE, such as specialized curve interpolation and approximation functions, surface skinning algorithms and a newly implemented algorithm to interpolate curve networks based on Gordon surfaces. The library also provides interfaces to many common programming languages such as C, C++, Python, Java and MATLAB and comes with a graphical user interface to visualize a CPACS configuration.

<!-- image -->

Structure und Aeroelastics

Fig. 1. TiGL is used as the central geometry pre-processor for many analysis tools inside and outside of the DLR.

TiGL is not the only freely available parametric geometry modeler for conceptual aircraft design. While it is not the intention of this paper to compile a complete list, a few of these tools and publications deserve to be mentioned. OpenVSP [5] is a parametric aircraft design tool for aircraft developed by NASA. Haimes and Drela published on the feasibility of conceptual aircraft geometry design for high fidelity by using a bottom-up approach [6]. GeoMACH [7] is a mutli-disciplinary analysis and optimization (MDAO) tool for geometric aircraft design, which supports a large number of design variables by providing also derivatives of the geometry with respect to the design variables. Caesiom [8] is a design framework based on MATLAB that includes a parametric geometry modeler, but also simplified physical simulation tools for aerodynamics, structures, propulsion and flight control. SUMO [9] is a surface modeler specifically designed for conceptual aircraft design that comes with a mesh generator and a post-processing tool. Finally, JPAD [10] is an analysis tool suite including JPADCAD , an OpenCASCADE-based geometry modeler for aircraft.

This paper is organized as follows. Sections 1.1 and 1.2 will introduce the idea behind CPACS and what role CPACS and TiGL play in multi-disciplinary optimization. Section 2 gives an overview of the software architecture and design. Section 3 describes TiGL's backbone, the geometry module. A special focus is given to the curve network interpolation algorithm used in TiGL to generate interpolating surfaces form a network of profile and guide curves. The CPACS description and TiGL implementation for specific aircraft components such as wings, the airplane fuselage, control surfaces etc. are discussed in Section 4. Finally, the paper closes with a summary and outlook in Section 5.

## 1.1. CPACS Parametrization

The Common Parametric Aircraft Configuration Schema (CPACS) is a data model that contains parametric desciptions of aircraft configurations, as well as missions, airports, fleets and more [2]. Its development started in 2005 at the German Aerospace Center, when there was an increasing need for a common aircraft model description that can be used in Multi-Disciplinary Optimization (MDO) applications. It is specifically designed for collaborative design in a heterogenous environment of engineers from different fields. Engineers can use it to exchange information on their models and tools. Next to the model description, process information is stored so that CPACS can be used to setup interdisciplinary workflows.

CPACS it based on a schema definition (XSD) for XML and as such has a hierarchical structure. On the highest level, the XSD description contains a header with meta-information about the CPACS file, such as a description, creation date, and CPACS version. Next to the header, there are elements for airlines, airports, flights, mission definitions, studies, tool specific information as well as vehicles.

The latter element contains descriptions of airplanes or rotorcrafts, engines, fuels, materials, as well as guide and profile curves used for the geometric modeling of components. TiGL uses the parametric description from these elements to construct the aircraft geometry. Section 4 contains details on the parametric description of specific components as well as its interpretation in TiGL.

In late July 2018, CPACS 3.0 was released [3]. The upcoming release of TiGL 3.0 is tightly coupled to some of the major changes introduced in the new CPACS version. Revised definitions for 'component segment' coordinate systems or a new simplified definition of guide curve points are two reasons, why TiGL 3.0 is designed not to be backwards-compatible. This means that TiGL 3 will not be able to read CPACS 2 files. This is less error-prone and increases robustness of the code. To make the transition from CPACS 2 to CPACS 3 easier, a converter tool called 'cpacs2to3' is now being developed by the community [11].

## 1.2. Optimization

Both CPACS and the TiGL geometry library were designed specifically for use in mutli-disciplinary optimization (MDO). The parametric description of CPACS enables users to directly control the configuration of an aircraft with just a small selection of parameters, see Fig. 2. With the help of TiGL, slight modifications of aircraft geometries can be created in an automated workflow. CPACS and TiGL are currently being developed and constantly extentended in research projects related to MDO [12-14]. Some ideas to increase the optimization capabilities in the future are

- to include automatic differentiation in TiGL's geometry module,
- to automatically generate CFD meshes by including an open source mesh library, and
- to track mesh deformations through geometry changes and thus provide shape gradients with respect to parameters to an external optimization tool.

Fig. 2. The shape of wings, fuselage and tailplane can smoothly be varied with just a few parameters.

<!-- image -->

## 2. Software Architecture

As TiGL is open source software, all its dependencies are open source as well. In that sense, it can be used in its full functionality without any commercial license by the users. TiGL is mainly based on the TiXI XML library [15] to read or write the CPACS data sets. TiGL heavily relies on the OpenCASCADE Technology CAD kernel [4], which is used for the geometric and topological modeling, for CAD data exports, to create solids via constructive solid geometry (CSG), and even for visualization.

Internally, TiGL contains multiple modules that are used for the several different aspects of the software. The geometry module includes all operations required to build the curves and surfaces that finally resemble the aircraft shape. These operations are all based on B-splines and NURBS (NonUniform Rational B-Splines). In addition, it contains an extension to OpenCASCADE's boundary representation (BREP) of shapes that adds metadata to the shapes. These metadata contain information about the shape modification history and the names of the shape and its faces. Since this information is preserved during CAD file exports, it can be used by external mesh generators, e.g. to create boundary layers around the wing surface.

Fig. 3. TiGL's system architecture (A) and a screenshot of the TiGL Viewer (B), which is used to display CPACS geometries.

<!-- image -->

The CPACS tree module is an object hierarchy of the CPACS standard. Each node in the CPACS tree is mapped to a C++ object that contains all of its attributes and sub-nodes as child objects. The code for these classes is automatically generated from the CPACS XML schema (see Section 2.1).

The component module implements the modeling of the all aircraft components and the wing and fuselage structure, including spars, ribs, frames, beams and much more (see Section 4). The export and import module are the interface to other analysis tools. TiGL can export the aircraft geometries to CAD-based as well as triangulated file formats. The former includes standard CAD exchanges formats such as IGES, STEP (ISO 10303) and OpenCASCADE's internal format BREP. They are mainly used in combination with external mesh generation software. The latter includes STL (stereolithography), VTK polydata (to be used e.g. in ParaView [16]), and COLLADA (COLLAborative Design Activity) to support 3D rendering of the geometries.

Although TiGL is written in C++, it provides bindings to C, MATLAB, Python, and JAVA (see Section 2.2). In addition to the library, the software package comes with TiGL Viewer , a viewer for CPACS and other CAD files. It uses TiGL for modeling and provides a 3D OpenGL based view of the geometries. In addition to the pure visualization, TiGL Viewer includes a scripting console that can be used for small automation tasks e.g. to create a four-sided view of the aircraft. A screenshot of TiGL Viewer is shown in Fig. 3b.

## 2.1. Automatic Code Generation

For every relevant CPACS entity, there must be a corresponding representation in TiGL. This is achieved by automatic code generation from the CPACS schema which was recently introduced into TiGL. Compared the the former approach of manual implementation, automatic code generation has many benefits, e.g.

- reduced development time,
- changes to CPACS can be adapted much faster,
- fewer errors, and
- CPACS files can now be checked strictly to their standard definition.

This generator was mainly developed by RISC Software GmbH and can be publicly accessed on Github [17]. CPACSGen is a command line tool, which uses the CPACS schema file as its input and produces a C++ class for each of the CPACS nodes. There are several ways to influence the code generation process. For example, there are many node definitions in CPACS that are not used by TiGL. Therefore, the generator allows the definition of a prune list input file that lists CPACS nodes, which are completely discarded - including their sub-trees. This is an effective mechanism to drastically reduce the amount of created code. Manual modifications to the auto-generated code can be realized by inheriting from the automatically generated classes, as the auto-generated code should not be modified by hand. Although, CPACSGen was designed primarily for TiGL, it can be used for other XML schema as well with only small adaptations.

## 2.2. Software Bindings

Even though TiGL is written in C++, it is mainly used by our users from the Python programming language. In addition, TiGL also comes with bindings for C, MATLAB and Java. Based on the public C interface of TiGL, the bindings are automatically generated by a small self-developed tool included in TiGL. The tool parses the C API and creates the bindings code for each of the programming languages. Sometimes, a C function signature can be ambiguous. For example a pointer to an integer int* could be either an integer return value or an input integer array. To overcome the ambiguity, annotations inside a function's docstring allow to define the semantics of each function argument. For each supported programming language, a code generator finally produces the bindings code. Right now, we are using the following technologies to enable the bindings:

- Python: Dynamic function calls via Python's ctypes.
- MATLAB: One compiled MATLAB-mex file combined with a *.m fi le per function.
- JAVA: Dynamic DLL loading with the JNA library [18].

In addition to the relatively simplistic language bindings via the public C API, we started to bind the entire internal C++ interface to Python. This makes it possible to use TiGL and pythonOCC [19] - the Python bindings to OpenCASCADE - in an interoperable fashion. This way, geometric objects can be passed from TiGL to OpenCASCADE and vice versa. Just like pythonOCC, these bindings are created with SWIG [20]. We experienced, that these new Python bindings encourage now also external developers with no C++ knowledge to contribute code to TiGL.

## 3. Geometry Module

## 3.1. B-spline modeling

TiGL uses the OpenCASCADE Technology CAD kernel [4] extensively for many tasks, e.g. to create solid objects, apply Boolean operations to them, and as a basis for the import and export modules. The geometric operations in OpenCASCADE however are not used in TiGL since several robustness issues were experienced in the past and the quality of the generated surfaces was not always satisfying. Therefore, most of the geometric modeling algorithms are implemented in the TiGL software itself. Just as OpenCASCADE, the geometric modeling is based on B-spline curves and surfaces. A B-spline curve is defined as

<!-- formula-not-decoded -->

where { p ⃗ i } are the n control points of the curve, τ is its knot vector, and { N d i ( u, τ ) } are the B-spline basis function of degree d . B-spline surfaces are defined as a tensor product of the B-spline basis functions, in particular

<!-- formula-not-decoded -->

All geometric shapes in TiGL, such as wings, fuselages, or engines have to be modeled as a combination of multiple B-spline surfaces. A bottom-up approach, as proposed in [6], is used for the geometric modeling: CPACS defines parametric points which are then used to build up curves, such as airfoils or fuselage sections. These curves are then connected to create the final surfaces. Several surfaces are eventually formed to solids and enriched by meta-data, which contain additional information such as face names. The single solid components are finally used to create the shape of the entire aircraft via Boolean operations, typically done in Constructive Solid Geometry. This approach is illustrated in Fig. 4.

Fig. 4. Bottom-up modeling of the geometries.

<!-- image -->

The B-spline modeling algorithms used in TiGL are standard methods from textbooks [21,22]. The most often used algorithm to create curves is B-spline interpolation, which creates as B-spline curve that passes through a set of points. Let { ⃗ c i | i = 0 . . . n -1 } be a set of points that should be interpolated at the curve parameters { u i } , then the interpolation conditions form the following linear system:

<!-- formula-not-decoded -->

This can be solved using standard linear solver methods, such as Gaussian elimination. When interpolating a closed set of points - i.e. where the first and last point is identical - with a periodic, C 2 continuous B-spline curve, this linear system can get singular for even polynomial degrees. To overcome this issue, the shifting method [23] is used. The interpolation of curves - often referred to as surface skinning [24, 25] - is similar to the curve interpolation. Surface skinning requires a set of compatible B-spline curves, where the curves differ only in their control points. The control points of the skinning surface are then computed by interpolating the curve's control points as before in equation (3).

In addition to curve and surface interpolation, TiGL also uses B-spline approximation algorithms in a few cases. These algorithms perform a least-squares fit of a B-spline to a set of points.

## 3.2. Curve Network Interpolation

At the heart of the geometric module is the curve network interpolation algorithm. It allows an accurate modeling of surfaces while keeping the number of input curves small. Compared to the simpler surface skinning method [24, 25], where a set of profile curves is interpolated by a surface, additional guide curves - sometimes also called rail curves - provide more control over how the surface interpolates the profiles. The algorithm is based on the Gordon surface method (see Section 3.2.1), which a almost never found in free or open source software. To our knowledge, only Ayam [26] and Sintef's Go Tools [27] implement the method, but without addressing the curve network compatibility problem (see Section 3.2.3).

Consider a curve network with N profile curves f i ( u ) : R → R 3 with i = 1 . . . N, u ∈ [0 , 1] and M guide curves g j ( v ) : R → R 3 with j = 1 . . . M, v ∈ [0 , 1]. The curve network should be properly closed by surrounding profile and guide curves, i.e.

Fig. 5. Curve network interpolation via profile and guide curves.

<!-- image -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

This basically means, that all profile curves must begin at the first and end at the last guide curve. And vice versa, all guide curves must begin at the first and end at the last profile curve. Such a curve network is depicted in Fig. 5. This curve network should now be interpolated by one single surface s ( u, v ) : R × R → R 3 . If it is enforced, that the input curves are iso-parametric curves of the resulting surface, the following conditions must hold:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Using these iso-parametric conditions, it is now easy to see, that for any profile curve f i and guide curve g j , the compatibility condition of the curve network follows:

<!-- formula-not-decoded -->

i.e. all profile curves must intersect a guide curve at the same curve parameter, and all guides curves must intersect a profile curve at the same parameter.

3.2.1. Gordon Surfaces. William J. Gordon published a method [28] that is able to interpolate a curve network if it fulfills the curve compatibility condition (8): For any set of spline blending functions { φ j ( u ) } and { ψ i ( v ) } satisfying the conditions

̸

<!-- formula-not-decoded -->

̸

the following blending surface interpolates the curve network { f i ( u ) } and { g j ( v ) } :

<!-- formula-not-decoded -->

Here ⃗ α i,j is the intersection point of the i-th profile curve f i ( u ) with the j-th guide curve g j ( v ), i.e.

<!-- formula-not-decoded -->

Equation (10) can now be rewritten as follows:

<!-- formula-not-decoded -->

Each of the three summands can be interpreted as an interpolation surface. The first S f ( u, v ) is a surface that interpolates the profile curves { f i ( u ) } , whereas the second term S g ( u, v ) is an interpolation surface for the guide curves { g j ( v ) } . The third term, often also called tensor product surface, interpolates the net of intersection points { ⃗ α i,j } . Fig. 6 illustrates the surface construction principle for Gordon surfaces. It can be seen as a generalization of the Coons-patch method [29] to more than two profile and guide curves.

Fig. 6. Construction of the Gordon surface via its three interpolation surfaces.

<!-- image -->

3.2.2. Gordon Surfaces with B-splines. Since TiGL relies on the B-spline based OpenCASCADE CAD library, all curves of the curve network are B-splines and the Gordon surface must also be a B-spline surface finally. Since the Gordon surface consists of two skinning surfaces and one tensor product surface, the surfaces S f ( u, v ) and S g ( u, v ) can be interpreted as the B-spline based skinning surfaces of the profile curves and guide curves. According to 'the NURBS Book' [21, p. 485], the blending functions { φ j ( u ) } and { ψ i ( v ) } can be interpreted as B-spline basis functions.

For the further derivation of the B-spline based Gordon surface method, it is required that all profile curves share

- the same degree
- and a common knot vector

in addition to the compatibility conditions of the curve network (8). The same should also apply to all guide curves. In practice, both can always be achieved by using degree elevation [30,31] and knot insertion [32,33] of the input curves.

In this case, all profile curves { f k ( u ) } and all guide curves { g l ( v ) } are of the form

<!-- formula-not-decoded -->

Here, { p ⃗ ( k ) i } are the control points of the k-th profile curve and { ⃗ q ( l ) i } are the control points of the l-th guide curve.

If the profile curves { f k ( u ) } are skinned with a B-spline surface with knot vector ξ f and degree d f and the guide curves { g l ( v ) } are skinned with a B-spline surface with knot vector ξ g and degree d g , Gordon's equation (10) for B-splines can be rewritten as follows:

<!-- formula-not-decoded -->

The three control nets { ⃗ P i,j } , { ⃗ Q i,j } and { ⃗ T i,j } must comply with the following interpolation conditions for all k = 1 . . . N and l = 1 . . . M :

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

These ensure, that the first term interpolates the profile curves, the second term interpolates the guides curves, and the third term interpolates the curve network's intersection points { ⃗ α l,k } . The interpolation conditions are linear systems, which can be solved again using e.g. Gaussian elimination. It should be emphasized that the interpolation of the intersection points (17) must use the same interpolation parameters, degrees and knot vectors as the interpolation of the profile curves (15) and the guide curves (16).

The B-spline based Gordon surface (14) still is a superposition of the three interpolation surfaces. The three surfaces differ in their degree and knot vector. To be usable for TiGL in the end, the Gordon surface has to be converted back to a single B-spline surface. Fortunately, this is easy to achieve: first the degree of the surfaces is elevated to their maximum u- and v-degree. Then, knots in u- and vdirection have to be inserted, such that all surfaces share the same knot vector. After degree elevation and knot insertion, the three surfaces are compatible and thus have the same number of control points. The final B-spline based Gordon surface is created by adding the control points of the skinning surfaces and subtracting the control points of the tensor product surface.

3.2.3. Achieving compatibility of the curve network. Until know, it was assumed that the curves of the curve network are compatible, i.e. they meet the compatibility conditions (8). In practice, however, this is almost never the case, since the curves can be parametrized arbitrarily. To meet the compatibility conditions, the curve network has to be reparametrized first.

Without loss of generality, the following derivations will be performed on the profile curves. For the guide curves, the equations then follow analogously. Let the original profile curve ˜ f k (˜ u ) intersect the original guide curve ˜ g l (˜ v ) at parameter ˜ u l,k , i.e.

<!-- formula-not-decoded -->

It is known from the analyses before, that the profile curve has to intersect all guide curves at the parameters { u l } . Using a reparametrization function σ k ( u ), the reparametrized profile curve f k ( u ) can now be defined as

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The function σ k ( u ) must satisfy This type of B-spline reparametrization is described in [21, pp. 241] as 'internal point mapping'. The choice of the reparametrization function σ k ( u ) is arbitrary but must fulfill in addition to (20) the following conditions:

1. It must be twice continuously differentiable since curvature continuous surfaces are required in the end.
2. The mapping function must be strictly increasing ( σ ′ k ( u ) &gt; 0) such that each original parameter ˜ u maps uniquely to one target parameter u (monotone + bijective).

For the sake of simplicity, a B-spline interpolation based reparametrization of degree 3 was chosen, which is

<!-- formula-not-decoded -->

where the second part ensures the mapping (20). Here, ⃗ Ψ i,k ∈ R are the control values of the interpolation B-spline of the curve σ k ( u ). It should be noted, that if original parameters { ˜ u l,k } and target parameters { u l } deviate too much, the function σ k ( u ) can lose its bijectivity due to B-spline oscillations. For now, the bijectivity is checked by the TiGL software. If it fails, the code will create an error.

After the reparametrization function σ k ( u ) has been found, the composed curve ˜ f k ( σ k ( u )) has to be transformed to B-spline form. There are two ways to achieve this:

1. Exact reparametrization as described in [21, pp. 247]. Since it is exact, it does not introduce any error to the method. The big drawback of this method is the greatly increased degree of the profile / guide curve after reparametrization. Since the degrees of the original curve and the reparametrization function are multiplied for the resulting curve, an input curve ˜ f (˜ u ) with degree 4 and a reparametrization function σ ( u ) of degree 3 results in a profile function of degree 12. As a consequence, also the degree of the final surface will be large.
2. Approximate reparametrization: The reparametrized function is approximated by sampling the curve ˜ f k ( σ k ( u )) and subsequently creating a new one from these sampled points. This way, the degree can be limited and the knot vector can be chosen more freely. The obvious drawback is the additional error introduced by the approximation.

For TiGL, the second method was chosen due to the following reasons: First, large degrees of the final surfaces should be avoided to keep the numerical complexity low. Second, the error of the approximation can be controlled and it can always be reduced by increasing the number of control points. Third, the free choice of the knot vector can be exploited such that e.g. the resulting profile curves { f k ( u ) } all have the same knot vector. This helps to keep the number of knots and control points of the final surface low.

When using the approximation technique, it is essential that the reparametrized curve still exactly passes through its intersection points of the curve network, such that Gordon's equation (10) is still valid. To achieve this, a hybrid approximation / interpolation technique of the sampled curve points is used: Let { ˆ u j } be n s sample parameters of the curve ˜ f k ( σ k ( u )) and let { u l } be the curve intersection parameters of the curve with the curve network. Then the control points { p ( k ) i } of the approximation B-spline are computed by solving the following constrained linear least squares problem:

<!-- formula-not-decoded -->

Using the Lagrange multiplier method, this problem (22) can be transformed into the constrained normal equation:

with

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

As usual, λ represents the Lagrange multipliers of the constraint problem. The linear system (23) is finally solved using Gaussian elimination. If the original curve contains kinks, these kinks are reproduced in the reparametrized curve by inserting knots with a multiplicity of the curve's degree ν into the knot vector τ f prior to the approximation.

The approximation can be performed with an arbitrarily chosen number of sample points n s . To get a unique solution, n s must be larger than the number of control points n of the reparametrized curve. The number of control points should be as small as possible to avoid unnecessary computational complexity but should be large enough to keep the approximation error small. In TiGL, we simply use roughly the same number of control points n for the reparametrized curve as for the original curve. This way, it is possible to reproduce all features of the original curve. The knot vector τ f is chosen to be uniform.

## Algorithm 1: B-spline based Gordon surface creation algorithm.

Input: Curve network of profiles ˜ f k (˜ u ) and guides ˜ g l (˜ v )

Output: The Gordon surface s ( u, v ) that interpolates the network

- 1 Compute intersections parameters ˜ u l,k and ˜ v k,l such that (18) holds.
- 2 for l = 1 . . . M do
- 3 u l ← 1 N ∑ k ˜ u l,k
- 4 for k = 1 . . . N do

<!-- formula-not-decoded -->

- 6 n ← maximum number of control points of the profile curves ˜ f k (˜ u )
- 7 m ← maximum number of control points of the guide curves ˜ g l (˜ v )
- 8 for k = 1 . . . N do
- 9 Compute f k ( u ) by reparametrizing ˜ f k (˜ u ) using n control points and original intersection parameters { ˜ u l,k } and target parameters { u l } as described in Section 3.2.3.
- 10 for l = 1 . . . M do
- 11 Compute g l ( v ) by reparametrizing ˜ g l (˜ v ) using m control points analogous to the profiles.
- 12 Compute profile skinning surface S f ( u, v ) with interpolation parameters { v k } according to (15).
- 13 Compute guide skinning surface S g ( u, v ) with interpolation parameters { u l } according to (16).
- 14 Compute tensor product surface T ( u, v ) with interpolation parameters { u l } and { v k } according to (17).
- 15 Make S f ( u, v ), S g ( u, v ) and T ( u, v ) compatible by degree elevation and knot insertion.
- 16 Create final surface s ( u, v ) by adding/subtracting the control points of the compatible interpolation surfaces.
- 17 return s ( u, v )

Fig. 7. Generated aircraft surfaces with the Gordon surface method. The helicopter case demonstrates, that the method also works for a large curve network.

<!-- image -->

3.2.4. Algorithm. The whole algorithm combines the previously described steps. To achieve a low number of control points of the final surface, we use the same number of control points n in all profile / guide curves in the reparametrization step. By using always the same number of control points, all profiles / guide curves will get the same uniform knot vector and hence also the skinning surface. If the knot vectors were different, all knot vectors would have to be merged first. This would result in a very large knot vector and therefore a large number of control points for the final surface. The pseudo code of our B-spline based Gordon method is depicted in Algorithm 1. Figure 7 shows six different example geometries that are created with this algorithm. The extreme helicopter case (see Fig. 7e) shows that the algorithm is also suitable for very large curve networks. The resulting surfaces are smooth - except for intentionally inserted kinks - and interpolate the curves as they should.

## 4. Aircraft Component Module

Many aircraft component geometries can be generated using a network of profile and guide curves. Section 4.1 will therefore describe how these curves can be defined according to the CPACS definition. Afterwards, details about the definition and modeling of wings, fuselages, control surfaces, structural elements as well as engine nacelles and pylons are presented.

## 4.1. Profile and Guide Curves

In this section, we are going to introduce the two basic building blocks for modeling wings and fuselages: profile and guide curves. Both of these entities are defined with respect to some local coordinate system as it is common to the CPACS description. For the wing profiles, TiGL implements a definition based on local points as well as a parametric description (CST). The guide curves are defined by a set of points in a local coordinate system.

4.1.1. Profiles from Point Lists. This is the most commonly used approach to creating wing and fuselage profiles with TiGL. Given a set of three-dimensional points in a local coordinate system of a section, a B-spline curve is interpolated. In a second step the curve is transformed to a global coordinate system to bring it to the position of the section and scale it accordingly.

Fig. 8. Example for a wing profile created from a point list of x -z coordinates.

<!-- image -->

A typical airfoil can be seen in Fig. 8. It is created by a list of x -z -points which are ordered in a mathematically positive sense. The list starts and ends at the trailing edge of the airfoil.

4.1.2. Profiles from Parametrized Curves (CST). An alternative to the creation of wing profiles via points is the analytic description of the airfoils by the Class Shape Transformation method (CST) [34]. Both the upper and the lower half of the profile is defined by a two-dimensional curve, which reads

<!-- formula-not-decoded -->

The exponents N 1 and N 2 of the Class function C N 1 N 2 ( ψ ) = ψ N 1 (1 -ψ ) N 2 , determine the slope at the leading and trailing edge, respectively. The Shape function

<!-- formula-not-decoded -->

is a linear combination of Bernstein basis functions B i,n ( ψ ) = ( n i ) ψ i (1 -ψ ) n -i of degree n and controls the shape of the airfoil. The size of the trailing edge is given by ζ T . With this, the CST curve is completely characterized by the exponents N 1 , N 2 , the Bernstein coefficients A = ( A i ) n i =0 and ζ T . Figure 9 shows a simple example of a CST curve for an upper wing profile.

Fig. 9. Example of an upper wing profile as a CST curve with parameters N 1 = 0 . 5, N 2 = 1, A = (2 , 3 , 2 , 1) and ζ T = 0 . 2.

<!-- image -->

4.1.3. Guide Curves from Point Lists. Guide curves connect the profiles in order to create a curve network (cf. Fig. 5 and Section 3.2). Each guide curve is created by B-spline interpolation of a set of guide curve points. Since a guide curve always starts and ends at a profile, the first and the last guide curve points are attached to these profiles. The position of the start points can be given either by a relative circumference of the profile or by pointing to the end point of a previous guide curve. In the latter case, the continuity across the profile can be set. This is described in Section 4.2.2. The position of the end point is always set by a relative circumference of the profile. The intermediate guide curve points are described by three local coordinates ( α, β, γ ).

In the following, we will describe how to construct a guide curve point in real space from the local coordinates (see Fig. 10). As a first step, we draw a straight line from the start to the end point. This line defines the first axis of the coordinate system. We move along this line from the start point ( α = 0) towards the end point ( α = 1). Hereby, α is the normalized distance between start and end point. From there, we move βc ( α ) towards a pre-defined direction. Here c ( α ) = c s (1 -α ) + c e α is the linear interpolation between the typical length of the start profile c s and the end profile c e . In the case of wing profiles, c s and c e are the cord lengths of the start and end profiles, respectively. The pre-defined direction is usually the global x -axis for wing guide curves and the global z -axis for fuselage guide curves. As a last step we move γc ( α ) in the direction perpendicular to both previous directions.

Fig. 10. Construction of a guide curve point in real space from local coordinates ( α, β, γ ) = (0 . 3 , 0 . 2 , 0 . 2) and two wing profiles.

<!-- image -->

## 4.2. Wing

4.2.1. CPACS Parametrization of the Wing. According to the CPACS definition, a wing is modeled from its (cross-)sections. For a wing, at least two sections must be present, one for the root of the wing and one for the tip.

A section is a coordinate system that is used to position airfoil curves in three-dimensional space, see Fig. 11. This coordinate system is defined using a transformation consisting of scaling in three dimensions, rotation around the z -, y - and x -axis, as well as a three-dimensional translation. In addition to the transformations, sections can be translated relative to each other using a positioning vector. It consists of a sweep and dihedral angle, as well as a length for the offset between two sections. The positioning vector of a section does not influence its rotation or scale. The total translation of the section is the sum of the positioning vector in Cartesian coordinates and the translation prescribed in the section's transformation.

Within a section, several elements can be placed, where each element references one airfoil curve. An element is again a coordinate system that is used to transform an airfoil within a section. By placing two elements in one section, it is possible to define wings, whose cross section has a discontinuous jump in span-wise direction, see Fig. 11.

A wing segment is the volumetric part of the wing that connects two elements from adjacent sections. It is possible to use guide curves within each segment to influence the segment shape. All segments must have the same number of guide curves and the guide curves of two adjacent segments must be connected. Otherwise, the guide and profile curves would not constitute a valid curve network and the Gordon surface algorithm would fail.

Fig. 11. Wing sections and elements according to the CPACS definition. Section 3 contains two elements with different rotations and scale. The wing therefore has a discontinuous shape at this section.

<!-- image -->

Finally, a component segment is a part of the wing that consists of several adjacent segments. Component segments are used to define the relative position of the internal wing structural elements and fuel tanks, control devices, and the wing fuselage attachment.

4.2.2. Geometric Modeling of the Wing. The wing profile curves described in section 4.1.1 are elements of wing sections. They are transformed through the section's transformation and positioning vector, as well as the element's transformation itself.

Guide curves can be defined for the segments through guide curve points (cf. section 4.1.3). They are constructed as curves spanning the wing from its root to the tip. Together with the wing profiles, they serve as input for the Gordon algorithm described in section 3.2.1. The connected guide curves are interpolated piecewisely, depending on the prescribed continuity condition of the guide curve. Continuity conditions are optional, and they can include 'C0' , 'C1 from previous' , 'C1 to previous' , 'C2 from previous' and 'C2 to previous' according to the CPACS schema.

A connected guide curve is broken into parts at the prescribed continuity conditions, see Fig. 12. As a default, each part is interpolated smoothly, meaning a C 2 continuity is prescribed. The parts depend on each other according to the 'from previous' or 'to previous' continuity conditions. A 'from previous' conditions means, that the tangent at the beginning of the guide curve must be the same as the end tangent of the inner neighboring guide curve. A 'to previous' condition implies, that the tangent at the beginning of the guide curve is prescribed onto the end of the inner neighboring guide curve. This implies an order in which guide curve parts must be interpolated, so that the prescribed tangents are available. TiGL uses a topological sorting algorithm based on Kahn's method [35] to achieve this. Note that TiGL only prescribes tangents at the break points and not the curvature. Therefore, only C 1 continuity is guaranteed.

If there are no guide curves, the profile curves are skinned linearly, or optionally using a B-spline of degree up to three. The resulting surface must be closed by side caps at the root and tip to make a solid, otherwise the wing geometry cannot be used in Boolean operations. The modeling of the wing tip geometry is planned for the future. Fig. 13a shows a wing created from a CPACS file with TiGL.

<!-- image -->

·

Fig. 12. A wing modeled with guide curves. Guide curves 1, 4 and 5 have no continuity conditions prescribed. The (upper) leading edge is separated into three parts, where the middle part containing guide curve 2 must be interpolated after the parts containing guide curves 1 and 3. The trailing edge is broken into two parts. A ' C 0 ' continuity condition for guide curve 6 is used to model a kink between the outer and middle segment.

## 4.3. Flaps

Flight control surfaces such as ailerons, flaps, slats, spoilers and rudders can be modeled with CPACS and TiGL. There are three categories: leading edge devices, trailing edge devices and spoilers. Fig. 13b shows extended trailing edge devices that were modeled using TiGL. The devices can have an internal structure, that is similar to the definition of the wing structure, see Section 4.5.

The outer shape of a control surface is defined by defining four points in the local ( η, ξ )-coordinates of the component segment. These points roughly describe the position and shape of the control device as well as the wing cutouts. Alternatively, the exact shape of the flap can be described using profile curves. In addition to the exact control surface shape, the shape of the wing cutout can be described more precisely by defining the cutout limits independently of the flap shape and separately for the upper and lower skin of the wing. In this case, the upper and lower cutout must be closed with a profile curve on the inner and outer side of the flap.

Alongside the description of the actuators of the control surfaces, the path along which a control surface moves when it is extended can be given. For this, an inner hinge point and an outer hinge point must be provided in order to define the connection of the flap to the actuators. All possible positions of the flap are described by giving steps along a path from a minimum deflection to a maximum deflection value. A step along this path includes a translation of the control device together with a rotation around the axis defined by the two hinge points.

Using TiGL's API, the flaps can be deflected and the resulting geometry can be exported for further processing. A console in TiGL Viewer allows the interactive deflection of the control devices for more control.

Fig. 13. A wing consisting of three segments, that was created with TiGL from a CPACS file. The extended trailing edge devices of the same wing is shown in (B).

<!-- image -->

## 4.4. Fuselage

Fuselages are created in TiGL similarly to wings: A fuselage is comprised of segments, which contain sections. Each section can contain one or more elements of fuselage profiles. The sections can be connected via guide curves. In order to create a solid fuselage, the profile curves and the connected guide curves with prescribed continuity conditions are transformed to global coordinates in TiGL. Together, they form the curve network that is used as an input for the Gordon Surface algorithm. Currently, the front and back of the fuselage are closed by side caps to create solids. The modeling of fuselage noses and rears are planned for the near future. Fig. 14 shows a fuselage that was created with TiGL.

Fig. 14. A fuselage built from eight profile curves and eight guide curves.

<!-- image -->

## 4.5. Wing and Fuselage Structure

4.5.1. Wing structure geometry. The structural definition of the wing is being developed by Airbus Defence and Space since 2012 [36] and was cantributed back to the TiGL source code in 2015. The foundation is the wing component segment, consisting mainly of the upper and lower wing shell, wing stringers, spars and ribs. Currently only the geometry generation of the ribs and spars is supported by TiGL.

The spar definition, shown in Fig. 15a, is realized with spar positions and segments. A positioning can be defined by ( η , ξ ) coordinates in the relative space of the component segment or with an unique identifier (UID) referring to a section-element and a ξ value. A spar segment has to consist of two or more spar positions.

The rib geometries are constructed as a second step, because their definition can be spar dependent. There are two different rib positioning schemes, the common and the explicit one. The common rib positioning is defined by two η values, one for the first and one for the last rib. As for the spar positions, these values can also be replaced by section-element UIDs, to place ribs directly at a section border. Additionally, the number of ribs is defined in the CPACS schema, and so the remaining ribs are placed equally distributed between the start and end rib. The chord-wise borders of the ribs can be the leading or the trailing edge of the wing or any spar that intersects the rib plane.

Fig. 15. CPACS Definition (A) and construction (B) of the wing spars.

<!-- image -->

Fig. 16. The different options for the wing ribs definition.

<!-- image -->

The main issue with this CPACS definition is, that it is not possible to define three or more ribs with a dedicated chord-wise connection. This is due to the common rib definition, which uses one span-wise position in combination with an angle. This allows to generate either an exact starting point or an exact ending point. To encounter this, the explicit rib definition with an exact chord-wise start and end position, was introduced. A major drawback of this method is, that every single rib face has to be defined and no distribution can be given. The described difference is visualized in Fig. 16a and Fig. 16b.

4.5.2. Fuselage structure geometry. The structural definition of the fuselage is also developed by Airbus Defence and Space and was contributed to TiGL in 2018. The CPACS definition of the fuselage structure differs completely from the one of the wing. Unfortunately, it is based on absolute coordinates, which leads to problems with the paradigm of parametric modeling. This was solved with an internal normalization of the absolute coordinate values in TiGL. Global x values are normalized with the overall length of the fuselage. The y and z values are normalized with a bounding box, containing a curve of the fuselage loft at a global x position. This solution enables the automatic adaption of the fuselage structure, if the fuselage loft is changed. When a CPACS export is requested, the absolute values are calculated in the reverse way and exported in absolute numbers.

The structural entities of the fuselage currently supported by TiGL, are: skin cells, fuselage frames, fuselage stringer, pressure bulkheads, fuselage doors, cross beams, cross beam struts, and long floor beams. Some of these entities are shown in Fig. 17.

Fig. 17. Fuselage segment with one-dimensional structure.

<!-- image -->

## 4.6. Engine Nacelles and Pylons

Engines are connected to the wing by pylons (see Figure 18). Engine nacelles and pylons are not yet implemented in TiGL, but will be in the near future. The definition of the pylon and engine geometry is currently undergoing changes in CPACS as well. The definition for the pylon geometry as described here is already included in the latest CPACS release, while the engine nacelle definition will be updated soon. The CPACS definition of pylons resembles that of a wing. Profile curves define the span-wise cross-section of a pylon and the curves are skinned in chord-wise direction (see Fig. 18b). In accordance with the CPACS definition, the outer geometry of the engine nacelles will be modeled from profile and guide curves (see Fig. 5). Two-dimensional profile curves define the radial sections of the engine nacelle in flow direction. The profile curves are placed around the engine's symmetry axis using cylindrical coordinates, i.e. by prescribing an angle and a radius. If only one profile is given, the resulting engine nacelle will be rotationally symmetric. Otherwise, the profiles will be connected using closed guide curves.

To guarantee that the inner geometry of the engine nacelle is perfectly rotationally symmetric, a curve for the inner shape can be defined with an offset from the engines symmetry axis. This curve is used to generate a rotation surface which is then blended with the - not necessarily rotationally symmetric - outer nacelle surface in a transition zone.

<!-- image -->

- (a) A wing with engine nacelle and pylon. Here, the engine is loaded from an STEP file as a generic geometry component .
- (b) A pylon is generated from profile curves skinned in chord-wise direction.

<!-- image -->

Fig. 18. Modeling of engine nacelles and pylons.

## 5. Summary and Outlook

This paper presented the software TiGL, which is a parametric geometry generator for aircraft-like configurations. It can be used in the aircraft design optimization process, by changing the CPACS design variables of the aircraft and regenerating the geometry using TiGL. TiGL models the major parts of an aircraft, including wings, fuselages, control surface devices, the inner aircraft structure, nacelles and pylons. It is primarily focused on parametric aircraft configurations in the CPACS format, which is getting more and more traction in the aircraft design community. Using TiGL and CPACS, it is possible to model a broad range of different configurations. Figure 19 illustrates five different example configurations, all defined in the CPACS format and modeled with TiGL.

Moreover, since TiGL is modular, its core modules can also be used completely disconnected from CPACS. For example it is possible to use only TiGL's geometry module for general modeling. One of the most important features of the geometry module is the implementation of the B-spline based Gordon surface algorithm. To the authors knowledge, this algorithm almost never occurs in freely available software. Since this algorithm allows for high precision surface modeling, TiGL is also suitable for high fidelity analysis.

Fig. 19. Different design configurations created with TiGL.

<!-- image -->

TiGL is already used by the aircraft community outside the German Aerospace Center (DLR). For example Airbus Defence and Space developed the DESCARTES analysis software [37] based on TiGL and improved TiGL during their development. A graphical CPACS editor [38] is currently being developed by CFS Engineering and is based on the TiGL Viewer. The aircraft tool suite JPAD [10] is using TiGL to integrate CPACS support into their software.

The development of TiGL will continue. In the near future, we will finish our implementation on nacelles and pylons. Afterwards, belly fairings, an improved modeling of the wing tips and winglets will follow. We are currently working on the automatic mesh generation for low- and mid-fidelity CFD. Therefore, it is planned to integrate the Salome Mesh module [39], which is based on Netgen [40], such that surface and volumetric meshes can be generated via a call to TiGL's API. To improve the support of gradient based MDO, it will be investigated whether an adjoint code or automatic differentiation of the geometry kernel is possible.

## Acknowledgement

TiGL has been developed for several years now. During this time TiGL has been developed and improved by many of our colleagues. In particular we would like to thank Markus Litz, who laid the foundation for TiGL. Special thanks go to Bernhard Gruber and Roland Landertshammer from RISC Software for their work on the software development and Merlin Pelz from DLR for the Gordon surface implementation. Finally, many thanks to our colleagues Jonas Jepsen, Philipp Kunze, Sebastian Deinert, Mark Geiger, Volker Poddey, Konstantin Rusch, and Paul Putin for their contributions to TiGL.

## References

- [1] DLR-SC, 'The TiGL geometry library to process aircraft geometries in pre-design.' https://github. com/DLR-SC/tigl , 2018. Accessed: 2018-09-27.
- [2] B. Nagel, D. B¨ ohnke, V. Gollnick, P. Schmollgruber, A. Rizzi, G. La Rocca, and J. J. Alonso, 'Communication in aircraft design: Can we establish a common language,' in 28th International Congress Of The Aeronautical Sciences, Brisbane , 2012.
- [3] DLR-SL, 'CPACS - common parametric aircraft configuration schema.' https://github.com/DLR-LY/ CPACS , 2018. Accessed: 2018-09-22.
- [4] OPENCASCADE, 'Open CASCADE Technology, 3d modeling and numerical simulation.' https://www. opencascade.com . Accessed: 2018-09-25.
- [5] A. Hahn, 'Vehicle sketch pad: a parametric geometry modeler for conceptual aircraft design,' in 48th AIAA Aerospace Sciences Meeting Including the New Horizons Forum and Aerospace Exposition , p. 657, 2010.
- [6] R. Haimes and M. Drela, 'On the construction of aircraft conceptual geometry for high-fidelity analysis and design,' in 50th AIAA Aerospace sciences meeting including the new horizons forum and aerospace exposition , p. 683, 2012.
- [7] J. Hwang and J. Martins, 'GeoMACH: geometry-centric mdao of aircraft configurations with high fidelity,' in 12th AIAA Aviation Technology, Integration, and Operations (ATIO) Conference and 14th AIAA/ISSMO Multidisciplinary Analysis and Optimization Conference , p. 5605, 2012.
- [8] M. R. Afsar, M. A. H. Banna, M. J. Uddin, and M. A. Salam, 'Ceasiom: An open source multi module conceptual aircraft design tool,' International Journal of Engineering , vol. 2, no. 7, 2013.
- [9] larosterna, 'Sumo -modeling and mesh generation.' https://www.larosterna.com/products/ open-source , 2018. Accessed: 2018-09-29.
- [10] DAFResearch Group at University Naples Federico II, 'JPAD: java program toolchain for aircraft design.' https://github.com/Aircraft-Design-UniNa/jpad , 2018. Accessed: 2018-09-28.
- [11] DLR-SC, 'cpacs2to3: A tool to convert cpacs files to version 3.' https://github.com/DLR-SC/cpacs2to3 , 2018. Accessed: 2018-09-22.
- [12] N. Kroll, M. Abu-Zurayk, D. Dimitrov, T. Franz, T. F¨ uhrer, T. Gerhold, S. G¨ ortz, R. Heinrich, C. Ilic, J. Jepsen, et al. , 'Dlr project digital-x: towards virtual aircraft design and flight testing based on highfidelity methods,' CEAS Aeronautical Journal , vol. 7, no. 1, pp. 3-27, 2016.
- [13] C. Liersch, K. Huber, A. Sch¨ utte, D. Zimper, and M. Siggel, 'Multidisciplinary design and aerodynamic assessment of an agile and highly swept aircraft configuration,' CEAS Aeronautical Journal , vol. 7, no. 4, pp. 677-694, 2016.
- [14] S. Goertz, C. Ilic, J. Jepsen, M. Leitner, M. Schulze, A. Schuster, J. Scherer, R. Becker, S. Zur, and M. Petsch, 'Multi-level MDO of a long-range transport aircraft using a distributed analysis framework,' in 18th AIAA/ISSMO Multidisciplinary Analysis and Optimization Conference , p. 4326, 2017.
- [15] DLR-SC, 'TiXI: Fast and simple xml interface library.' https://github.com/DLR-SC/tixi , 2018. Accessed: 2018-09-22.

- [16] J. Ahrens, B. Geveci, and C. Law, 'Paraview: An end-user tool for large data visualization,' The visualization handbook , vol. 717, 2005.
- [17] RISC Software GmbH, 'CPACSGen: Generates CPACS schema based classes for tigl.' https://github. com/RISCSoftware/cpacs\_tigl\_gen , 2017. Accessed: 2018-09-24.
- [18] 'JNA: Java native access.' https://github.com/java-native-access/jna . Accessed: 2018-09-24.
- [19] T. Paviot and J. Feringa, 'pythonOCC-3D CAD for python.' http://www.pythonocc.org , 2016. Accessed: 2018-09-28.
- [20] D. M. Beazley et al. , 'Swig: An easy to use tool for integrating scripting languages with c and c++.,' in Tcl/Tk Workshop , 1996.
- [21] L. Piegl and W. Tiller, The NURBS book . Springer Science &amp; Business Media, 2012.
- [22] G. Farin, Curves and Surfaces for CAGD: A Practical Guide . Morgan Kaufmann, 2014.
- [23] H. Park, 'Choosing nodes and knots in closed b-spline curve interpolation to point data,' Computer-Aided Design , vol. 33, no. 13, pp. 967 - 974, 2001. DOI 10.1016/S0010-4485(00)00133-0.
- [24] A. A. Ball, 'CONSURF. part one: introduction of the conic lofting tile,' Computer-Aided Design , vol. 6, no. 4, pp. 243-249, 1974.
- [25] A. A. Ball, 'CONSURF. part two: description of the algorithms,' Computer-Aided Design , vol. 7, no. 4, pp. 237-242, 1975.
- [26] R. Schultz, 'Ayam: A free 3d modelling environment for the renderman interface..' http://ayam. sourceforge.net/ayam.html , 2018. Accessed: 2018-09-28.
- [27] SINTEF, 'GoTools.' https://www.sintef.no/projectweb/geometry-toolkits/gotools/ . Accessed: 2018-09-28.
- [28] W. J. Gordon, 'Spline-blended surface interpolation through curve networks,' Journal of Mathematics and Mechanics , pp. 931-952, 1969.
- [29] S. COONS, 'Surface for computer aided design of space forms,' MIT Project MAC, TR-41 , 1967.
- [30] H. Prautzsch, 'Degree elevation of b-spline curves,' Computer Aided Geometric Design , vol. 1, no. 2, pp. 193-198, 1984. DOI 10.1016/0167-8396(84)90031-1.
- [31] L. Piegl and W. Tiller, 'Software-engineering approach to degree elevation of b-spline curves,' ComputerAided Design , vol. 26, no. 1, pp. 17-28, 1994. DOI 10.1016/0010-4485(94)90004-3.
- [32] W. Boehm, 'Inserting new knots into b-spline curves,' Computer-Aided Design , vol. 12, no. 4, pp. 199-201, 1980. DOI 10.1016/0010-4485(80)90154-2.
- [33] E. Cohen, T. Lyche, and R. Riesenfeld, 'Discrete b-splines and subdivision techniques in computer-aided geometric design and computer graphics,' Computer graphics and image processing , vol. 14, no. 2, pp. 87111, 1980. DOI 10.1016/0146-664X(80)90040-4.
- [34] B. M. Kulfan, 'A universal parametric geometry representation method - 'CST', jan. 2007,' in 45th AIAA Aerospace Sciences Meeting and Exhibit , p. 0062, 2007.
- [35] A. B. Kahn, 'Topological sorting of large networks,' Commun. ACM , vol. 5, pp. 558-562, Nov. 1962.
- [36] R. Maierl, ¨ O. Petersson, and F. Daoud, 'Automated creation of aeroelastic optimization models from a parameterized geometry,' in 15th International Forum on Aeroelasticity and Structural Dynamics , 2013.
- [37] F. Daoud, S. Deinert, R. Maierl, and ¨ O. Petersson, 'Integrated multidisciplinary aircraft design process supported by a decentral mdo framework,' in 16th AIAA/ISSMO Multidisciplinary Analysis and Optimization Conference , p. 3090, 2015.
- [38] CFS Engineering, 'CPACSCreator.' https://github.com/cfsengineering/CPACSCreator , 2018. Accessed: 2018-09-28.
- [39] V. Bergeaud and V. Lefebvre, 'SALOME. a software integration platform for multi-physics, pre-processing and visualisation,' in SNA+MC 2010 Conference , p. 2129, 2010.
- [40] J. Sch¨ oberl, 'NETGEN an advancing front 2d/3d-mesh generator based on abstract rules,' Computing and visualization in science , vol. 1, no. 1, pp. 41-52, 1997.

Martin Siggel and Jan Kleinert and Tobias Stollenwerk German Aerospace Center (DLR) Simulation and Software Technology Linder Hoehe Cologne, 51147 Germany e-mail: martin.siggel@dlr.de

Reinhold Maierl Airbus Defence and Space Structure Analysis and Optimization Rechliner Str. Manching, 85077 Germany e-mail: reinhold.maierl@airbus.com