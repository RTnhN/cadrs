## Blending Surface Modelling Using Sixth Order PDEs

L. H. You and Jian J. Zhang

National Centre for Computer Animation, Bournemouth University, United Kingdom

Abstract 一 In order to model blending surfaces with curvature continuity, in this paper we apply sixth order partial differential equations (PDEs), which are solved with a composite power series based method. The proposed composite power series based approach meets boundary conditions exactly, minimises the errors of the PDEs, and creates almost as accurate blending surfaces as those from the closed form solution that is the most accurate but achievable only for some simple blending problems. Since only a few unknown constants are involved, the proposed method is comparable with the closed form solution in terms of computational efficiency. Moreover, it can be used to construct 3- or 4-sided patches through the satisfaction of continuities along all edges of the patches. Therefore, the developed method is simpler and more efficient than numerical methods, more powerful than the analytical methods, and can be implemented into an effective tool for the generation and manipulation of complex free-form surfaces.

Keywords: Curvature continuity surface blending, Sixth order partial differential equation, Composite power series solution, Weighted residual method

## 1.  Introduction

Blending surfaces are widely used in product design. In general, two kinds of blending surfaces are commonly 니 sefhl: those with tangent and curvature continuities. Although tangent continuity is sufficient for many cases, the ability to satisfy a higher degree of smoothness requirement is essential for various situations. For instance, streamlined surfaces of an automobile are aesthetically appealing and those of an aircraft with curvature continuity can reduce the risk of flow separation and turbulence.

Blending with curvature continuity has been investi­ gated by a number of researchers. Boehm proposed a method to generate curvature continuous curves and surfaces by generalizing the well-known construction of the Bezier points of a cubic spline curve or surface [3]. Jones decomposed an 77-sided region into n rectangles and indicated that the rectangular patches are biseptic for curvature continuity [15]. Farin discussed how to construct curvature continuous planar curves consisting of conic segments represented in rational Bezier form [10]. Pegna demonstrated the design of second order smooth blending surfaces by requiring that normal curvatures agree along all tangent directions at the linl&lt;age curve of two patches [21]. By sweeping a (possi 미 y variable) circular arc to represent blending surfaces, he gave a method which can g 니 arantee continuity of the unit surface normal and of the normal radi 니 s of curvature along the linkage curve [22]. Later on, he and Wolter presented a Linkage Curve theorem that is pertaining to the design of curvat 나 re continuous blending surfaces [23]. Zheng et al. investigated the curvature contin 니 ity between two adjacent rational Bezier surfaces which may be either rectangular or triangular [36]. Filkins et al. used an approximation method to blend surfaces which maintain curvature continuity to the underlying surfaces with a non-uniform rational B-spline (NURBS) surface [11]. Schichtel presented a technique for filling polygonal holes using a transfinite interpolant. This technic [니 e is to model the second-order smooth transitions at an arbitrary linkage curve between two surfaces [25]. Aumann proposed the so-called normal ringed surfaces to form curvature continuous connections of cones and/or cylinders [1]. Hartmann blended an implicit surface with a parametric surface and achieved G 흐 -continuous transitions through introduction of a simple additional condition [12]. In his later work, he introduced a method for curvature-continuous (G2) interpolation of an arbitrary sequence of points on a surface (implicit or parametric) which can be used for G1 blending of curves on s 나 rf 让 ces [13]. Recently, Hartmann proposed the implicit Gn vertex blending methods which can be applied to other surfaces such as parametrically defined s 니 rf 代 ces [14]. By defining curvature contin 니。니 s splines based on 3-sided patches, Peters discussed how to construct smooth surfaces over irregular meshes [24]. Ye developed the Gaussian and mean curvature criteria which individually guarantee the curvature continuity along the linkage curve [27]. He also introduced a method for local construction of a curvature continuous (GC2) piecewise polynomial surface through the interpolation of a given rectangular curvat 나 re contin 니。나 s quintic curve mesh [28]. Bohl and Reif discussed conditions how degenerate triangular Bezier patches guarantee curvature continuity [4]. Kim et al. demon­ strated how to extend a given surface with a piecewise smooth bo 니 ndary and indicated the extended surface is C 존continuity along the old bo 니 ndary [16].

^Corresponding author:

<!-- image -->

Blending surfaces can also be created using the sol 니 tions to PDEs, which has attracted an increasing amount of research efforts. Compared with other more mainstream approaches, this group of techniques prove very flexible in dealing with certain unus 니 al blending problems in addition to the 'ordinary' ones. For example, it can easily blend an open surface with a closed one (Fig. 6) and can also blend s 니雨 ces with creases. This would otherwise be extremely challenging fbr other techniques.

PDE surface generation is a relatively new research topic in geometric modelling, however has already shown a great deal of potential. Bloor and Wilson investigated free-form surface generation with partial differential equations [2]. Lowe et al, created blending s 니 rf 知 ces that satisfy certain given design conditions [18], Dekanski et al. applied PDE method in generation of a properller blade geometry [7]. Du and Qin developed a technique for direct manipulation and interactive sculpting based on PDE and equation of motion [8]. They also considered geometric and physical constraints in dynamic PDE-based s 니刑沃 design [9]. Mimis et al. discussed shape parameterization of a twostroke engine and optimization of scavenging properties of the engine [19]. Zhang and You applied PDE based approach in vase design [33] and surface blending [34]. Ugail and Wilson combined shape parametrisation with a standard method of numerical optimization and demonstrated the capability of setting up automatic design optimisation problems [26]. You and Zhang extended PDE surface modelling from static problems to dynamic ones [31]. Monterde and Ugail proposed a general PDE method to create Bezier surfaces from the boundary information [20]. These papers and many others employed a fourth order partial differential equation. Since the vector-valued parameter has a strong infhaence on the shape of the blending surfaces to be 잉 enerated, an improved fourth order PDE was proposed by You and Zhang [29]. In this new PDE, three vector-val 니 ed parameters were introduced aiming to provide the designer with more control and flexi­ bility on the created blending surfaces. However, these PDEs can only ensure the functional and tangential continuities at the linkage curves, i.e. the highest continuity is C1. This is because a fourth order PDE will not have enough degrees of freedom to satisfy curvature continuity requirements. In order to remedy this issue, sixth order partial differential equations were proposed by You et al. [32], Zhang and You [35], and Kubiesa et al. [17].

An important question to be answered fbr the PDE based approach, however, is how to solve the partial differential equations efficiently and accurately. From the properties of PEDs, one can 나 nderstand that closed form solutions do not exist fbr the majority of surface blending problems. Numerical methods s 니 ch as the finite element method [5] and finite difference method [6] are 나 sually the most obvious choice. However, due to the nature of discretization, these reported numerical methods cannot exactly satisfy the boundary conditions of the blending problems which is a minimum requirement for surface blending. Moreover, these methods usually involve a large number of 니 nknowns which result in the resolution of a large set of linear algebra equations. It is therefore very time-consuming. This disadvantage in many cases has significantly hampered the use of the PDE based techniques, especially in situations where interactivity is required, s 니 ch as real time graphics applications. In order to create surfaces quickly, we will propose a resol 나 tion method for solving sixth order PDEs. This solution is an extension of the work presented in [34] where only a fourth order PDE was involved. This method makes use of the composite power series expansion to approximate the blending surfeces. The basic features of o 니!' method are that it can satisfy boundary conditions exactly and minimize the error of the sixth order PDEs effectively. Since only a very small number of collocation points and unknown constants are involved, the proposed method is not only accurate, but also computationally very efficient.

## 2.  Composite power series solutions of sixth order partial differential equations

Considering the efiect of the vector-vahied parameters on the shape of the blending surfaces, we use the following sixth order partial differential equations to produce curvature continuous blending surfaces

<!-- formula-not-decoded -->

where a = [ax ay az]\ b = [bx by 妇〈 c = [cx cy cz]T,d = [dx dy are vector-valued shape control parameters, x = [匸卩 긔' represents a vector-valued position function and u, v are the parametric variables.

Next, we will define the boundary conditions. According to [23], curvature contin 나 ity across the linkage curve is achieved when the second fundamental tensors of the two connected patches are identical at all points of the linkage curve. That gives,

<!-- formula-not-decoded -->

where subscripts and "一' denote two adjacent surface patches, respectively.

Since du and dv are the differentials of two arbitrary parametric variables, Eqn (2) is equivalent to the following ones

<!-- formula-not-decoded -->

where

L = nruu N=nrm and N=nrvv (4)

<!-- formula-not-decoded -->

Since the normal n will be shared by both surfaces, Eqn (3) will definitely be satisfied, if the following conditions hold,

<!-- formula-not-decoded -->

Therefore, the boundary conditions for surface blending with up to curvature continuities will include position and tangent contin 니 ity conditions as well as Eqn (6). Thus the bo 니 ndary conditions can be written as

<!-- formula-not-decoded -->

After the above treatment, a blending surface x(w, v) with curvature contin 니 ity can be created with the solution to PDE (1) subject to boundary conditions (7).

Clearly, the closed form solution of PDE (1) under boundaiy conditions (7) does not exist fbr general cases. Here we use the same methodology which we developed in [34] to find the approximate analytical solution. In order to simplify the solving process for PDE (1), we firstly define a linearly independent basic function as those consisting of constant 1, the parametric variable v, its various elementary functions excluding polynomials, and their combinations not in a polynomial form. Then, we can write the following linearly independent basic functions from the boundary conditions (7),

<!-- formula-not-decoded -->

With the above preparation, we propose to approximate the blending surface, i.e. the solution of Eqn (1) s 니切 ect to boundary conditions (7) with the same composite power series expansion used in [34] whose form is given by

<!-- formula-not-decoded -->

where Mxh Myj, Mzk represent the numbers of the terms of the power series taken for the 无,力, z 比 components, respectively.

If a linearly independent basic function of the boundary conditions (7) is 1, say for example g;-(v) = 1, the corresponding exponent m takes values from 0 to 5, because these 6 unknown constants can be uniquely determined by the boundary conditions given in Eqn (10).

Taking the x component as an example, referring to Eqn (9), the bo 니 ndary conditions fbr 払 v) can be written as

<!-- formula-not-decoded -->

Substituting x 仞 , v) of Eqn (9) into Eqn (10), we find that 으오 = q , ( y ) and -prod 니 ce the same dudv " ' 。허

d^Xj equation, and so do -r = &lt;3/0^/ ， (v) and x, = 0 ()幻(卩). dv

泓 . 02x. Therefore, bo 니 ndary conditions ----- - and -- in Eqn dudv 而

(10) for both 이 = 0 and w 1 are redundant.

Similar to the treatment given in [34], we can now solve Eqn (10) for the unknown constants pjm (m = 0,12…,5). They are given by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Here qJ 〃 =O, 1,2,...,5) are known constants. Repla­ cing p 沥 (' ？ =° ， 1 ， 2,…,5) of Xj(u, v) in the first expre­ ssion of Eqn (9) with Eqn (11), one obtains the foll­ owing x component that satisfies boundary conditions (10) exactly

<!-- formula-not-decoded -->

Since Eqn (9) is an approximate solution of Eqn (1) under boundary conditions (7), Eqn (1) is not accurately satisfied by substituting Eqn (9) into (1). The error can be described with a residual function (error function). Still taking the x component as an example, its residual function can be formulated by

<!-- formula-not-decoded -->

where Uim and A{ can be written as

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

Assuming that the blending region is defined by Uq&lt;u&lt;U\ and vQ&lt;v&lt; choosing N collocation points in this region and substituting the values of u and v at these collocation points into Eqn (13), the residual vahies R(um v') (n= 1,2,.../V) at these collocation points can be computed by [30]

<!-- formula-not-decoded -->

where R, C and B are TV x 1 arrays consisting of the resid 니 al values, unknown constants and constant terms,

respectively, and A is a Nx'^(MXI-5') matrix consisi=o

ting of the coefficients of the unknown constants pim (i = 0,1,2,..., l;m = 6,7,...,A 為).

The sq 니 ared sum of the resid 니 al values of Eqn (16) is

<!-- formula-not-decoded -->

And the blending s 니 rf 咨 ce minimising the error of PDE (1) can be determined by

<!-- formula-not-decoded -->

which leads to the following set of linear algebraic equations

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Solving these linear algebra equations determines all the unknown constants of the x component. The y and z components can be obtained similarly.

## 3. Accuracy and computational efficiency

In this section we will examine the accuracy and computational efficiency of the above-proposed com­ posite power series method through two blending examples. The first is to blend a circular torus and an elliptic hyperboloid of one sheet and the second example is to blend an open surface and a plane. Altho 니 gh the first is straightforward, the second would be very challenging for other blending techniques. From this point of view, one can also sense the versatility and powerfulness of the PDE based blending approach.

It is well known that the closed form solution of a PDE is the most accurate and efficient among all possible solutions. For the first example, the closed form solution of PDE (1) 니 nder boundary conditions (20) is obtainable, if the vector-vahied shape control parameters are set to those presented below. We will solve PDE (1) using both the presented method and the closed form solution method, and compare the accuracy and computational efficiency of both approaches.

The boundary conditions for this blending example * are

<!-- formula-not-decoded -->

From the boundary conditions, it can be seen that the linearly independent basic functions are cosv for the x component, sinv for the y component and 1 for the z component. Therefore, the solution of Eqn (1) takes the form of

<!-- formula-not-decoded -->

Using the method introduced in the previous section, we can determine all the unknown constants of Eqn (21). Let 니 s evenly distribute 9 collocation points within the rectang 니 lar region of u and v, and take the mimbers of the power series terms of the x and y components to be MxQ = = 7. This means that there are only 2 unknown constants in the solving equation (19) to be determined. When setting the vector-valued parameters to be ax = bx = % ， = by = 1 and cx = dx = cy= % = -l, the blending surface is obtained and depicted in Fig. la.

*Undefined symbols used in the examples throughout this paper are geometric parameters of the relevant example.

Fig. 1. Blending between a circular torus and an elliptic hyper­ boloid of one sheet.

<!-- image -->

<!-- image -->

Substituting the same vector-valued shape control parameters into PDE (1), the sixth order parti 이 differential eq 니 ations for the x and y components of Eqn (1) become

<!-- formula-not-decoded -->

Under the same bo 나 ndary conditions, the closed form solution of Eqn (22) exists which has the following form

<!-- formula-not-decoded -->

It should be pointed out that the closed form solutions of Eqn (1) under the same bo 니 ndary conditions for arbitrary values of the vector-valued parameters are 니 sually not obtainable.

Introducing Eqn (23) into (22), we can obtain x andj components. The z component can be taken to be the third of Eqn (21) which satisfies the sixth order PDE of the z component. Therefore the closed form solution of Eqn (1) is

<!-- formula-not-decoded -->

where all the unknown constants can be determined by the bo 니 ndary conditions of this blending problems.

With Eqn (24), we generated the blending surface and depicted it in Fig. lb. Vis 니 ally, the images shown in both fig 니 res look identical.

Excellent agreement of the blending surfaces between the proposed composite power series method and closed form solution can be further demonstrated through a quantitative comparison. We use the Euclidean norm to measure the difference. Choosing lu and Iv points respectively in the x and y directions within the blending region, the Euclidean norm can be written as

<!-- formula-not-decoded -->

where the x, y and z components without " 〜 "stand for the closed form solutions and those with "for the composite power series solutions.

Equation (25) is a measurement of the absolute errors between two surfaces. We can also use relative errors between two surfaces to measure the difference between the proposed composite power series solution and the closed form solution. The equation of the relative errors takes the form of

<!-- formula-not-decoded -->

Uniformly choosing 99 x 99 points within the blending region, i.e., IU = JV = 99, the Euclidean norm between these two blending surfaces is only £=6.111 x IO-3 and the difference from Eqn (26) is E= 3.135 x IO-3. It s 나 ggests that with even a small number of collocation points and power series terms, the proposed composite power series method can generate blending surfaces almost as accurately as the closed form solution method. Considering how the method works, the reason is easy to comprehend: since the proposed composite power series method always satisfies the boundary conditions exactly up to the order of curvature continuity, the discrepancy at the interior region of the blending surface will have very limited effect both visually and functionally. Moreover, this discrepancy is further reduced by the least square minimisation, leaving almost no room fbr errors.

It is also worthy mentioning that the computing efficiency of the proposed method is m 니 ch higher than that of the other numerical methods such as the finite element method and the finite difference method. The finite element method, for example, uses a large number of elements or nodes to achieve reasonable accuracy, which involves with many unknowns. As a consequence, the resolution of the linear equations is inevitably time-consuming. The proposed method on the other hand, only needs to solve a small number of linear eq 나 ations. With the above-chosen number of the collocation points and power series terms, the resolution process took less than 10-6 second for the proposed composite power series and closed form resolution methods on an ordinaiy PC. This is also true even if we chose 8 x 8 = 64 collocation points and Mxq = Mg = 9 power series terms. Thus we can conclude that the proposed composite power series method can generate blending surfaces almost as accurately and fast as the 이 osed form solution method. The comp 니 tational efficiency is the same also fbr the problems whose closed form solutions do not exist, thus making many previously unsolvable problems solvable.

For the second example, the boundary conditions are given by %

<!-- formula-not-decoded -->

The vector-val 니 ed parameters in Eqn (1) were taken to be q= 1, b=3 护 , q = 3/A dj=p6 (i = x,y,z) and p = 3. The image obtained from the proposed composite power series and closed form sol 나 tions were depicted in Figs. 2a and 2b, respectively. Again both methods produced identical images.

## 4.  Influences of vector-valued shape control parameters on surface shapes

By 니 sing different values of the vector-valued shape control parameters, the sol 니 tion to PDE (1) is changed which will lead to different surface shapes. In this section, we will use two examples to highlight their effects.

The first example is to blend a sphere and an ellipsoid. The boundary conditions fbr this blending task are given below

Fig. 2. Blending between an open surface and a plane.

<!-- image -->

<!-- formula-not-decoded -->

The sol 나 tion of Eqn (1) under these boundary condi­ tions has the same form as that of Eqn (21). We will take 3x3 collocation points and 2 unknown constants. Initially, we set the vector-valued parameters to ax = bx =Cx = dx = %, = by = Cy = dy = I. The obtained blending surface is depicted in Fig. 3a. Then making cx = cv = 10 and keeping all other parameters unchanged, we obtain the blending surface in Fig. 3b.

The second example is to blend a conical frustum with an elliptic cylinder. The boundary conditions for this blending task are taken as

<!-- formula-not-decoded -->

Fig. 3・ Blending between a sphere and an ellipsoid.

<!-- image -->

<!-- image -->

<!-- formula-not-decoded -->

Using the above described proced 니 re, the solution of Eqn (1) 니 nder these boundary conditions takes the form of

<!-- formula-not-decoded -->

Using the proposed method, the unknown constants of Eqn (30) can be similarly determined. With the collocation points and the unknown constants unchanged, the blending s 니 rf 代 ce in Fig. 4a is obtained when all the vector-valued parameters are set to 1, and Fig. 4b created when parameters bx, by and bz are set to 100 and the others are kept the same.

It is evident that the vector-valued shape control parameters in Eqn (1) have a strong influence on the shape of the blending surfaces. By changing their vahies, we can generate different surface shapes.

## 5. More complex examples

Two more complex examples of surface blending will be presented to demonstrate the strength of the proposed approach. The first is to blend two inter- secting cylinders, whose PDE was solved only with the finite element method or finite difference method in existing literature, which is numerically expensive. The second example is to determine the transition surface between an open surface and a plane interpolating a specified curve.

Fig. 4. Blending between a conical frustum and an elliptic cylinder.

<!-- image -->

For the blending between two intersecting cylinders, the boundary conditions at the linkage curves are as follows

<!-- formula-not-decoded -->

According to these boundary conditions, we can obtain the linearly independent basic functions and construct the following composite power series solution of Eqn (1)

<!-- formula-not-decoded -->

On determining the unknown constants of the above equation, the blending surface generated is depicted in Fig. 5.

For the blending between an open surface and a plane at a specified curve, the parametric equation of the open s 니! 0ce are taken to be

Fig. 5. Blending between two intersecting cylinders.

<!-- image -->

<!-- formula-not-decoded -->

Taking u = 0.2 in the above equation and specifying the curve on the plane to be blended, we can obtain the bo 니 ndary conditions of this blending task

<!-- formula-not-decoded -->

The composite power series solution of Eqn (1) corresponding to the linearly independent basic functions in these boundary conditions can be taken to have the following form

<!-- formula-not-decoded -->

<!-- image -->

Fig. 6. Blending between an open s 니 rfhce and a plane at a specified curve.

<!-- image -->

Specifying the values of all the geometric parameters in the bo 니 ndary conditions and then determining all the unknown constants in Eqn (35) with the above proposed method, the blending surface obtained is given in Fig. 6.

## 6. Conclusions

By solving sixth order partial differential equations with fb 니 r vector-valued shape control parameters subject to blending bo 니 ndary conditions, we have presented a method for surface blending with up to curvat 니 re continuities. In comparison with our previous work [34], we are able to solve a higher order (sixth order) PDE as efficiently as a lower order one, and offer better smoothness.

Traditionally, PDE based methods are only appli­ cable to a limited number of applications. This is due to the fact that the closed form solution of a PDE is either extremely difficult to obtain or does not exist. Existing numerical methods are computationally expensive. This is rather regrettable, as such methods do offer many advantages over other surface blending approaches. In order to overcome this limitation and make our proposed PDE approach practicable to a large number of blendin 음 problems, we have developed an efficient and accurate resolution method using the composite power series expansion and the weighted residual technique. This method can satisfy the boundary conditions exactly and minimise the errors at the interior region of the s 니 HWce. With this method, the generated blending surface shares the exact position, tangent and curvature values with the primary surfaces at the linkage curves. Tt was fb 나 nd that this method has almost the same accuracy and computational efficiency as the closed form solutions.

The influences of the vector-valued shape control parameters on the shape of the blendin 응 surfaces have also been examined. Their variations have a strong influence on the shape of the generated blending surfaces, and thus can be potentially exploited to serve as 니 ser interface tools fbr shape manipulation.

## References

- [1]  Aumann, G (1995), "Curvature continuous connections of cones and cylinders,'' Computer Aided Geometric Design, 27(4), 293-301.
- [2]  Bloor, M. I. G and Wilson, M. J. (1990), 'Using Partial differential equations to generate free-form surfaces/1 Computer-Aided Design, 22(4), 202-212.
- [3]  Boehm, W. (1986), '"Curvature continuous curves and surfaces, Computer Aided Design, 18(2), 105-106.
- [4]  Bohl, H. and Rei£ U. (1997), ''Degenerate Bezier patches with continuo 나 s curvature," Computer Aided Geometric Design, 14(8), 749-761.
- [5]  Brown, J. M., Bloor, M. I. G, Bloor, M. S. and Wilson, M. J. (1998), "The accuracy of B-spline finite element approximations to PDE surfaces,5, Computer Methods in Applied Mechanics and Engineerings 158(3-4), 221-234.
6. [이 Cheng, S. Y., Bloor, M. I. G, Saia, A. and Wilson, M. J. (1990), "Blending between quadric surfaces using partial differential equations. In Ravani B (ed): Advances in Design Automation,Vol. 1, Computer Aided and Computational Design. ASME, 257-263.
7. [기 Dekanski, C. W., Bloor, M. I. G. and Wilson, M. J. (1995), 'The generation of propeller blade geometries using the PDE method," Journal of Ship Research, 39(2), 108-116.
- [8]  Du, H. and Qin, H. (2000), 'Direct manipulation and interactive sculpting of PDE surfaces," Computer Graphics Forum, 19(3), 261-270.
- [9] Du, H. and Qin, H. (2005), "Dynamic PDE-based surface design 나 sing geometric and physical constraints, Graphical Models, 67( 1), 43-71.
10. [1 이 Farin, G (1989), ''Curvature continuity and offsets for piecewise conics," ACM Transactions on Graphics, 8(2), 89-99.
- [11]  Filkins R C., Tuohy S. T. and Patrikalakis N. M. (1993), "Computational methods fbr blending surface approxi­ mation, Engineering with Computers, 9(1), 49-62.
- [12]  Hartmann, E. (1995), "Blending an implicit with a parametric surface, Computer Aided Geometric Design, 12(8), 825-835.
- [13]  Hartmann, E. (1996), K 了 interpolation and blending on surfaces," The Visual Computer, 12(4), 181-192.
- [14]  Hartmann, E. (2001), 'Implicit (了사) lending of vertices, Computer Aided Geometric Design^ 18(3), 267-285.
- [15]  Jones, A. K. (1988), "Nonrectangular surface patches with curvature continuity, Computer Aided Design^ 20(6), 325-335.
16. [1 이 Kim, H., Oh, S. and Yim, J.-W. (2005), "Smooth surface extension with curvat 나 re bo 니 nd," Computer Aided Geometric Design, 22(1), 27-43.
- [17]  Kubiesa, S., Ugail, H. and Wilson, M. J. (2004), 4tInteractive design using higher order PDEs," Visual Computer, 20(10), 682-693.
- [18]  Lowe, T. W., Bloor, M. I. G and Wilson, M. J. (1990), "Functionality in blend design," Computer Aided Design^ 22(10), 655-665.
- [19]  Mimis, A. P., Bloor, M. I. G and Wilson, M. J. (2001),

- "Shape parameterization and optimization of a two-stroke engine, Journal of Propulsion and Power^ 17(3), 492498.
- [20]  Monterde, J. and Ugail, H. (2006), 'A general 4th-order PDE method to generate Bezier surfaces from the boundary/5 Computer Aided Geometric Design^ 23(2), 208-225.
- [21]  Pegna, J. (1989), "Simple practical criterion to guarantee second order smoothness of blend surfaces," ComputerAided and Computational Design, 19(1), 93-105.
- [22]  Pegna, J. (1990), 'Spherical and circular blending of functional surfaces, Journal of Onshore Mechanics and Arctic Engineering, 112(2), 134-142.
- [23]  Pegna, J. and Wolter, F.-E. (1992), "Geometrical criteria to guarantee curvature continuity of blend surfaces," Journal of Mechanical Design, Transactions of the ASME, 114, 201-210.
- [24]  Peters, J. (1996), ''Curvature continuous spline surfaces over irregular meshes/5 Computer Aided Geometric Design, 13(2), 101-131.
- [25]  Schichtel, M. (1993), UG2 Blend surfaces and filling ofNsided holes," IEEE Computer Graphics &amp; Applications, 13(5), 68-73.
- [26]  Ugail, H. and Wilson, M. J. (2003), ''Efficient shape parametrisation for automatic design optimisation 니 sing a partial differential equation formulation, Computers and Structures, 81(28-29), 2601-2609.
- [27]  Ye, X. Z. (1996), "The Gaussian and mean curvature criteria fbr curvature continuity between surfaces," Computer Aided Geometric Design, 13(6), 549-567.
- [28]  Ye, X. Z. (1997), 'Curvature continuous interpolation of curve meshes, Computer Aided Geometric Design, 14(2),

Dr. L. H. y 。니 is currently a senior research fellow at the National Centre fbr Comp 나 ter Animation, Boumemoth Media School, Bournemouth University, UK. 니 is research interests are in computer graphics, computer animation and geometric modelling.

L. H. you

<!-- image -->

169-190.

- [29]  You L. H. and Zhang, J. J. (1999), "Blending surface generation with a fourth order partial differential equation, The Sixth International Conference on Computer-A ided Design and Computer Graphics, Shanghai, China, 10351039.
- [30]  You, L. H., Zhang, J. J. and Comninos, P. (2000), 'A volumetric deformable muscle model fbr computer animation using weighted residual method," Computer Methods in Applied Mechanics and Engineering, 190, 853-863.
- [31]  You, L. H. and Zhang J. J. (2003), 'Fast Generation of 3D Deformable Moving Surfaces,5, IEEE Transactions on Systems, Man and Cybernetics, Part B: Cybernetics, 33(4), 616-625.
- [32]  You, L. H., Comninos, P. and Zhang, J. J. (2004), "PDE blending surfaces with C2 continuity, Computers &amp; Graphics, 28, 895-906.
- [33]  Zhang, J. J. and You, L. H. (2002), "PDE based surface representation-Vase design," Computers &amp; Graphics, 26, 89-98.
- [34]  Zhang, J. J. and You, L. H. (2004), t4Surface blending using a power series sol 니 tion to fb 니 rth order partial differential equations, International Journal of Shape Modeling, 10(2), 155-185.
- [35]  Zhang J. J. and You, L. H. (2004), "Fast Surface Modelling Using a 6th Order PDE,' Computer Graphics Forum, 23(3), 311-320.
- [36]  Zheng, J. M., Wang, G Z. and Liang, Y. D. (1992), ''Curvature continuity between adjacent rational Bezier patches," Computer Aided Geometric Design^ 9(5), 321335.

Professor Jian J. Zhans leads the Computer Graphics Research Group at the National Centre fbr Computer Animation, and he is Head of Research at Bournemouth Media School, Bournemouth University, UK. His current research interests include computer graphics, physics­ based modeling, computer animation, geometric modelling, medical simulation and visualisation.

Jian J. Zhang

<!-- image -->