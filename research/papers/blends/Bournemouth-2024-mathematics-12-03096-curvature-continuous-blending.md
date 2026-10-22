<!-- image -->

Article

## Analytical C 2 Continuous Surface Blending

Xiangyu You 1, *, Feng Tian 2 , Wen Tang 1 , Jian Chang 3 and Jianjun Zhang 3

- 1 Department of Creative Technology, Faculty of Science and Technology, Bournemouth University, Poole BH12 5BB, UK; wtang@bournemouth.ac.uk
- 2 Division of Natural and Applied Sciences, Duke Kunshan University, Kunshan 215316, China; feng.tian978@dukekunshan.edu.cn
- 3 National Centre for Computer Animation, Faculty of Media and Communications, Bournemouth University, Poole BH12 5BB, UK; jchang@bournemouth.ac.uk (J.C.); jzhang@bournemouth.ac.uk (J.Z.)
* Correspondence: lawrenceyoux@gmail.com

Abstract: Surface blending is an important topic in geometric modelling and is widely applied in computer-aided design and creative industries to create smooth transition surfaces. Among various surface blending methods, partial differential equation (PDE)-based surface blending has the advantages of effective shape control and exact satisfaction of blending boundary constraints. However, it is not easy to solve partial differential equations subjected to blending boundary constraints. In this paper, we investigate how to solve PDEs analytically and develop an analytical PDE-based method to achieve surface blending with C 2 continuity. Taking advantage of elementary functions identified from blending boundary constraints, our proposed method first changes blending boundary constraints into a linear combination of the identified elementary functions. Accordingly, the functions for blending surfaces are constructed from these elementary functions, which transform sixth-order partial differential equations for C 2 surface blending into sixth-order ordinary differential equations (ODEs). We investigate the analytical solutions of the transformed sixth-order ordinary differential equations subjected to corresponding blending boundary constraints. With the developed analytical PDE-based method, we solve C 2 continuous surface blending problems. The surface blending example presented in this paper indicates that the developed method is simple and easy to use. It can be used to effectively control the shape of blending surfaces and at the same time exactly satisfy C 2 continuous blending boundary constraints.

Keywords: surface blending; C 2 continuity; sixth-order partial differential equation; closed-form solution

MSC: 68U05; 68U07; 35C05; 35A24

## 1. Introduction

Surface blending is very important in computer-aided design and geometric modelling. It has a wide range of applications in various industrial sectors such design, manufacturing, and creative industries.

Surface blending is the creation of a surface that smoothly connects two or more separate surfaces together. The surface to be created is called a blending surface, and the separate surfaces are called primary surfaces. The interfaces between primary surfaces and the blending surface are called trimlines [1].

Depending on different continuities on trimlines, surface blending can be divided into tangent curvature, curvature continuity, and higher continuity. Among them, tangent continuity and curvature continuity are widely met in industrial applications. Tangent continuity can be achieved with C 1 continuity, and curvature continuity can be obtained with C 2 continuity. Unlike tangent and curvature continuities, which, respectively, keep the tangent and curvature of primary surfaces and the blending surface the same on the trimlines, C 1 and C 2 continuities, respectively, keep the first and second partial derivatives' cross trimlines the same on trimlines and are more stringent than tangent and curvature continuities.

<!-- image -->

Citation: You, X.; Tian, F.; Tang, W.; Chang, J.; Zhang, J. Analytical C 2 Continuous Surface Blending. Mathematics 2024 , 12 , 3096. https://doi.org/10.3390/ math12193096

Academic Editor: Luigi Rodino

Received: 21 August 2024

Revised: 27 September 2024

Accepted: 30 September 2024

Published: 3 October 2024

<!-- image -->

Copyright: © 2024 by the authors. Licensee MDPI, Basel, Switzerland. This article is an open access article distributed under the terms and conditions of the Creative Commons Attribution (CC BY) license (https:// creativecommons.org/licenses/by/ 4.0/).

<!-- image -->

Some methods [1,2] have been developed to create blending surfaces. Among them, rolling-ball blending methods [3] are the most popular. In addition, partial-differentialequation-based surface blending [4] has also attracted a lot of attention.

Partial-differential-equation-based surface blending creates a blending surface by solving a vector-valued partial differential equation subjected to blending boundary constraints. Compared with other surface blending methods, the advantages of partial-differentialequation-based surface blending are effective shape control of blending surfaces and exact satisfaction of blending boundary constraints.

However, solving partial differential equations subjected to blending boundary constraints is not an easy task. Usually, numerical or approximate analytical resolution methods must be used. Especially for C 2 continuous surface blending, exact closed-form solutions to partial differential equations have not been developed.

To tackle the above problem, this paper will propose a new method to obtain exact closed-form solutions to sixth-order partial differential equations used for C 2 continuous surface blending. By decomposing blending boundary constraints into elementary functions and constructing blending surface functions from the decomposed elementary functions, sixth-order partial differential equations are transformed into sixth-order ordinary differential equations, and blending boundary constraints are decomposed accordingly. After that, exact closed-form solutions to the sixth-order ordinary differential equations are obtained and introduced into the constructed blending surface functions to obtain exact closed-form solutions to sixth-order partial differential equations subjected to blending boundary constraints.

The remaining parts of this paper are organized as follows. The related work is reviewed in Section 2. The mathematical model for C 2 continuous surface blending is formulated in Section 3, and the exact closed-form solutions to the mathematical model are developed in Section 4. The application of the proposed method is investigated in Section 5. Finally, the conclusions are drawn in Section 6.

## 2. Related Work

In existing work, surface blending has been extensively investigated. Various surface blending methods have been developed. Comprehensive surveys of various surface blending methods are reported in [1,2].

Among the various surface blending approaches, rolling-ball methods are the most popular and have been widely used for rounding edges and corners of mechanical parts [2]. Rolling-ball methods consist of rolling a ball along two primary surfaces to create a blending surface [3]. They can be divided into constant-radius and variable-radius rolling-ball blends. In addition to rolling-ball methods, partial-differential-equation-based surface blending also becomes an active topic due to its advantages in effectively controlling the shape of blending surfaces and exactly satisfying blending boundary constraints.

Constant-radius rolling-ball blending methods were investigated in [3,5-8]. Rossignac and Requicha proposed a new method to incorporate constant-radius blends, which is achieved by rolling a sphere in contact with primary surfaces to be blended together [3]. Choi and Ju mathematically constructed rolling-ball blends through sweeping rational quadratic curves and representing corner blends where three surfaces meet with a convex combination of linear Taylor interpolants [5]. Farouki and Sverrisson investigated the numerical methods of constant-radius blends to achieve prescribed-precision approximation and guarantee the satisfaction of specified tolerance [6]. K ó s et al. discussed how to determine the radius of rolling-ball blends from point data, which have been pre-processed and segmented [7]. By applying the results of canal surfaces, Dahl and Krasauskas developed a general algorithm to parametrize fixed-radius rolling-ball blends of pairs of natural quadrics [8].

Variable-radius rolling-ball blending methods were examined in [9-14]. Chuang et al. calculated a parametric form of variable-radius spherical and circular blends by using the derived spine curve and linkage curves [9]. Chuang and Hwang tackled the problems of the radius for variable-radius blending being difficult to specify and the spine curve being hard to trace by introducing several geometric constraints to specify the variable radius and a paradigm to implement the constraints of tracing the spine curve [10]. Luk á cs et al. treated variable-radius rolling-ball blending surfaces as the envelopes of one-parameter families of varying-radius balls, which are special cases of discriminant sets [11]. Luk á cs used the theory of envelopes and discriminant sets to analyse variable-radius rolling-ball blending surfaces, determine the differential geometric invariants of the surfaces, and characterize the progressive and regressive points on the variable-radius rolling-ball blending surfaces [12]. Chuang and Lien presented two formulations to determine general blending between parametric surfaces. The first formulation represents the blend as a sweeping surface whose radii satisfy a specific one-parameter curve, and the second formulation also defines the blend as a sweeping surface but whose radii satisfy a specific two-parameter surface [13]. K ó s generalized the algorithms for reconstructing constant-radius rolling-ball blends to reconstruct variable-radius blends [14].

Rolling-ball methods are simple and intuitive. They can exactly satisfy positional and tangential continuities on trimlines. However, they are less effective at controlling the shape of blending surfaces and cannot satisfy higher continuities on trimlines. In contrast, PDE-based surface blending not only exactly satisfies blending boundary constraints of C 1 , C 2 , and C n ( n &gt; 2) continuities on trimlines but also effectively controls the shape of blending surfaces.

Bloor and Wilson pioneered partial differential equation-based surface blending [4]. The challenge for partial differential equation-based surface blending is how to solve partial differential equations in questions where they are subjected to blending boundary constraints. Due to the difficulty in solving partial differential equations, some numerical methods have been proposed. For example, the finite difference method was introduced in [15,16], and the finite element methods were investigated in [17-20]. Besides numerical methods, some approximate analytical methods have also been proposed. For example, an approximate analytical solution to a vector-valued fourth-order partial differential equation was obtained in [21] to achieve C 1 continuous surface blending, another approximate analytical solution to a vector-valued sixth-order partial differential equation was presented in [22] to create C 2 continuous blending surfaces, and a unified approach based on a time-dependent approximate analytical solution to a vector-valued sixth-order partial differential equation was proposed to tackle both time-independent and time-dependent surface blending with C 2 continuity [23].

In nature, approximate analytical methods are numerical since the errors at many collocation points are calculated and a system of linear algebra equations are solved to numerically determine the values of the unknown constants used to define blending surfaces. Since numerical methods require large computer resources and involve high computational costs, they are not ideal for surface blending. Exact closed-form solutions address this problem. In this paper, we will propose exact closed-form solutions to sixthorder partial differential equations to achieve C 2 continuous surface blending.

## 3. Mathematical Model

According to [24], the boundary constraints for C 2 continuous surface blending can be formulated as

<!-- formula-not-decoded -->

where u and v are two parametric variables; t ( u , v ) stands for the three components, x ( u , v ) , y ( u , v ) , and z ( u , v ) , of a blending surface; Bt 1 ( v ) and Bt 4 ( v ) are the position components of boundary curves at the boundaries u = 0 and u = 1, respectively; Bt 2 ( v ) and Bt 5 ( v ) are the first partial derivatives and Bt 3 ( v ) and Bt 6 ( v ) are the second partial derivatives of the blending surface X ( u , v ) = [ x ( u , v ) y ( u , v ) z ( u , v ) ] T with respect to the parametric variable u at the boundary curves, and

<!-- formula-not-decoded -->

To satisfy the six equations in the blending boundary constraints (1), the solutions to the partial differential equations for the x ( u , v ) , y ( u , v ) , and z ( u , v ) components should, respectively, involve six unknowns. Since a sixth-order partial differential equation involves six unknowns, the partial differential equations for the x ( u , v ) , y ( u , v ) , and z ( u , v ) components can be taken to be

<!-- formula-not-decoded -->

̸

where γ = 0, η = 0, λ = 0, and ρ = 0 are called shape control parameters since they can be used to control the shape of blending surfaces.

̸

̸

̸

The mathematical model of C 2 continuous surface blending is obtained by putting Equations (3) and (1) together. Its accurate analytical solution will be developed in the following section.

The six equations for each component, t = x , y , z , in the blending boundary constraints can be decomposed into constants, dt 0, n ( t = x , y , z ; n = 1, 2, · · · , 6), and elementary functions. In this paper, we investigate the surface blending problems whose elementary functions have the differential property: the second derivatives of the elementary functions are the same as the elementary functions themselves multiplied by a coefficient.

If we use the functions f tk ( v ) ( t = x , y , z ; k = 1, 2, 3, · · · , K ) to indicate the elementary functions in the blending boundary constraints (1), the above differential property can be formulated as below:

<!-- formula-not-decoded -->

where ξ tk is the coefficient obtained from the differentiation operation.

Introducing the functions f tk ( v ) ( t = x , y , z ; k = 1, 2, 3, · · · , K ) into the blending boundary constraints (1) and considering the constants dt 0, n ( n = 1, 2, · · · , 6), Equation (1) can be changed into

<!-- formula-not-decoded -->

Since the blending surface should satisfy the blending boundary constraints at the boundaries u = 0 and u = 1, the vector-valued blending surface function should (1) involve the functions f tk ( v ) and (2) have the functions of the parametric variable u only to satisfy the constraints of the constants dt 0, n ( n = 1, 2, · · · , 6) obtained from the boundaries u = 0 and u = 1. Therefore, the vector-valued blending surface function can be constructed as below:

<!-- formula-not-decoded -->

Substituting Equation (6) into Equation (3), the partial differential equations in Equation (3) are transformed into

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Considering the differential property (4), Equation (7) can be written as the following ordinary differential equation:

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

Substituting Equation (6) into the blending boundary constraints (5), the following boundary constraints are obtained:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where

After the above treatment, the mathematical model, i.e., Equations (3) and (1), is changed into Equations (8) and (10), and the original problem of solving the partial differential equations in Equation (3) subjected to the boundary constraints (1) is transformed into solving the ordinary differential equations in Equation (8) subjected to the blending boundary constraints (10). In what follows, we will develop an exact analytical approach to obtain the closed-form solutions to the partial differential equation in Equation (8) subjected to Equation (10).

## 4. Closed-Form Solutions of the Mathematical Model

The first part of Equation (8) contains three ordinary differential equations; their exact analytical solutions can be written as below:

<!-- formula-not-decoded -->

Since the above analytical solutions have exactly satisfied the ordinary differential equations defined by the first part of Equation (8), we use the blending boundary constraints (10) to determine the unknown constants in Equation (12).

Substituting Equation (12) into (10) and taking k = 0 in (10), we obtain the following equations:

<!-- formula-not-decoded -->

Solving the six linear algebra equations in (13), we obtain the six unknown constants Ctj ( j = 1, 2, · · · , 6 ) . Substituting them back into Equation (12), the exact analytical solutions for the ordinary differential equations defined by the first ODE of Equation (8) are found to be

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The second part of Equation (8) also contains three ordinary differential equations. To solve these three ordinary differential equations, we let their solutions Gtk ( u ) ( t = x , y , z ) have the form of

<!-- formula-not-decoded -->

For clarity, we first use a 0, a 1 , a 2, and a 3 to indicate at 0, a t 1 k , a t 2 k , and a t 3 k in Equation (9), respectively. Then, we substitute Equation (16) into the second part of Equation (8) and obtain the following nonlinear algebra equation:

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Equation (17) is a nonlinear algebra equation. In order to solve the equation, we write it as

<!-- formula-not-decoded -->

Comparing Equation (19) with Equation (17), the following equations, which relate b 1 , b 2, and b 3 to a 4 , a 5, and a 6, are obtained below:

<!-- formula-not-decoded -->

Using b 3 to represent b 1 and b 2 in Equation (20), the three algebraic equations in (20) are changed into one cubic equation below:

<!-- formula-not-decoded -->

where The cubic Equation (21) has three roots. Solving the equation, we obtain its three roots below:

<!-- formula-not-decoded -->

where the second subscript 1, 2, and 3 indicate the first, second, and third root, respectively; i is an imaginary number; and

<!-- formula-not-decoded -->

Substituting Equation (23) back into Equation (20), we obtain b 1, j and b 2, j , which correspond to b 3, j ( j = 1, 2, 3), respectively. Here, we take b 3,1 as an example to demonstrate how to determine b 1,1 and b 2,1, as well as the corresponding solutions to the ordinary differential equations defined by the second ODE of Equation (8).

Introducing the first part of Equation (22) into (20) and solving for b 1 and b 2, we obtain b 1 and b 2 below:

<!-- formula-not-decoded -->

Once again, for clarity, we use b 1 , b 2, and b 3 to indicate b 1,1 , b 2,1, and b 3,1 in the following mathematical derivations.

Equation (19) can be decomposed into two nonlinear algebra equations, which have the forms of

<!-- formula-not-decoded -->

Substituting b 3 = b 3,1 into the first part of Equation (25), we obtain the following two roots ( r tk ) 1,2 :

<!-- formula-not-decoded -->

̸

Since f tk ( v ) has the differential property (4), a 3 = 0 according to Equation (9), and a 6 = 0 according to Equation (18). Therefore, b 3 = 0 according to Equation (21).

̸

̸

Substituting b 1 = b 1,1 and b 2 = b 2,1 into the second part of Equation (25) and solving for r 2 tk , we obtain

<!-- formula-not-decoded -->

The above equation has four roots; they can be obtained according to the three different cases below.

<!-- formula-not-decoded -->

For this case, ( r 2 sk ) 1,2 = -b 1 /2, and there are three different situations, i.e., b 1 &lt; 0, b 1 = 0, and b 1 &gt; 0. According to b 2 1 = 4 b 2 and Equation (20), we know b 1 = 0 will lead to b 2 = 0 and a 5 = a 6 = 0. From Equations (18) and (9), we know a 5 = a 6 = 0 will lead to λ = ρ = 0, which changes the partial differential equations defined in Equation (3).

Therefore, b 1 cannot be zero. The roots for the remaining two situations, b 1 &lt; 0 and b 1 &gt; 0, can be summarized as

<!-- formula-not-decoded -->

From the two roots in (26) and the four roots in (28), we obtain the four different solutions to the ordinary differential equations defined by the second part of Equation (8) indicated below.

(1) For b 3 &lt; 0, b 2 1 = 4 b 2, and b 1 &lt; 0, we have ( r sk ) 1,2 = ± q 0 according to Equation (26) and ( r sk ) 3,4,5,6 = ± q 1 according to Equation (28). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(2) For b 3 &lt; 0, b 2 1 = 4 b 2, and b 1 &gt; 0, we have ( r sk ) 1,2 = ± q 0 according to Equation (26) and ( r sk ) 3,4,5,6 = ± iq 1 according to Equation (28). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(3) For b 3 &gt; 0, b 2 1 = 4 b 2, and b 1 &lt; 0, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26) and ( r sk ) 3,4,5,6 = ± q 1 according to Equation (28). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(4) For b 3 &gt; 0, b 2 1 = 4 b 2, and b 1 &gt; 0, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26) and ( r sk ) 3,4,5,6 = ± iq 1 according to Equation (28). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

4.2. Case 2: b 2 1 &gt; 4 b 2

For this case, there are six different situations, i.e., -b 1 ± √ b 2 1 -4 b 2 &lt; 0, -b 1 ± √ b 2 1 -4 b 2 = 0, and -b 1 ± √ b 2 1 -4 b 2 &gt; 0. It is obvious that -b 1 ± √ b 2 1 -4 b 2 cannot be zero since -b 1 ± √ b 2 1 -4 b 2 = 0 will lead to b 2 = 0 and a 6 = 0 according to Equation (20).

Therefore, we only consider

-

b

1

±

b

2

1

-

4

b

2

&lt;

0 and

-

b

1

±

b

2

1

-

4

b

2

&gt;

0.

The roots of Equation (27) for the remaining four situations -b 1 ± √ b 2 1 -4 b 2 &lt; 0 and -b 1 ± √ b 2 1 -4 b 2 &gt; 0 can be summarized as

<!-- formula-not-decoded -->

√

√

where

<!-- formula-not-decoded -->

From the two roots in (26) and the four roots in (33), we obtain the six different solutions to the ordinary differential equations defined by the second part of Equation (8) shown below.

(5) For b 3 &lt; 0, b 2 1 &gt; 4 b 2, and b 1 &lt; -√ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = ± q 2 and ( r sk ) 5,6 = ± q 3 according to Equation (33). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(6) For b 3 &lt; 0, b 2 1 &gt; 4 b 2, and -√ b 2 1 -4 b 2 &lt; b 1 &lt; √ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = ± q 2 and ( r sk ) 5,6 = ± iq 3 according to Equation (33). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(7) For b 3 &lt; 0, b 2 1 &gt; 4 b 2, and b 1 &gt; √ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = ± iq 2 and ( r sk ) 5,6 = ± iq 3 according to Equation (33). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(8) For b 3 &gt; 0, b 2 1 &gt; 4 b 2, and b 1 &lt; -√ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = ± q 2 and ( r sk ) 5,6 = ± q 3 according to Equation (33). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(9) For b 3 &gt; 0, b 2 1 &gt; 4 b 2, and -√ b 2 1 -4 b 2 &lt; b 1 &lt; √ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = ± q 2 and ( r sk ) 5,6 = ± iq 3 according to Equation (33). Thesolutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(10) For b 3 &gt; 0, b 2 1 &gt; 4 b 2, and b 1 &gt; √ b 2 1 -4 b 2, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = ± iq 2 and ( r sk ) 5,6 = ± iq 3 according to Equation (33). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

4.3. Case 3: b 2 1 &lt; 4 b 2

For this case, Equation (27) is changed into the following equation:

<!-- formula-not-decoded -->

̸

Depending on whether b 1 = 0 or b 1 = 0, different roots can be obtained from the above equation. In what follows, we investigate this.

̸

4.3.1. b 1 = 0

In order to obtain the roots ( r tk ) 3,4,5,6 for b 1 = 0 from Equation (41), we transform Equation (41) into

̸

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

̸

where

According to Equation (42), the four roots ( r tk ) 3,4,5,6 for b 1 = 0 of Case 3 can be written in the following form:

<!-- formula-not-decoded -->

If we let

<!-- formula-not-decoded -->

the four roots for b 1 &gt; 0 are

and the four roots for b 1 &lt; 0 are

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

From the two roots in (26) and the four roots in (46) and (47), we obtain the four different solutions to the ordinary differential equations defined by the second part of Equation (8) shown below.

(11) For b 3 &lt; 0, b 2 1 &lt; 4 b 2, and b 1 &gt; 0, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = q 5 ± iq 4 and ( r sk ) 5,6 = -q 5 ∓ iq 4 according to Equation (46). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(12) For b 3 &lt; 0, b 2 1 &lt; 4 b 2, and b 1 &lt; 0, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = q 4 ± iq 5 and ( r sk ) 5,6 = -q 4 ∓ iq 5 according to Equation (47). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(13) For b 3 &gt; 0, b 2 1 &lt; 4 b 2, and b 1 &gt; 0, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = q 5 ± iq 4 and ( r sk ) 5,6 = -q 5 ∓ iq 4 according to Equation (46). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(14) For b 3 &gt; 0, b 2 1 &lt; 4 b 2, and b 1 &lt; 0, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = q 4 ± iq 5 and ( r sk ) 5,6 = -q 4 ∓ iq 5 according to Equation (47). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

4.3.2. b 1 = 0

When b 1 = 0, we obtain b 3 = a 4 from the first part of Equation (20) and b 2 = a 5 from the second part of Equation (20). Substituting b 2 = a 5 and b 3 = a 4 into the third part of Equation (20) and considering Equations (18) and (9), we obtain ηλ = ργ . Thus, b 1 = 0 presents a special case of ηλ = ργ .

Substituting b 1 = 0 into Equation (41), we obtain

<!-- formula-not-decoded -->

Considering the condition b 2 1 &lt; 4 b 2, the case for b 1 = 0 and b 2 &lt; 0 does not exist. For the case of b 1 = 0 and b 2 = 0, we have a 5 = 0 from the second part of Equation (20) and λ = 0 from Equations (18) and (9), which changes the partial differential equations defined in Equation (3). Thus, we will not consider this case.

For b 1 = 0 and b 2 &gt; 0, the four roots can be obtained from the third part of Equation (52), which has the form of

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where

(15) For b 3 &lt; 0, b 2 1 &lt; 4 b 2, b 1 = 0, and b 2 &gt; 0, we have ( r sk ) 1,2 = ± q 0 according to Equation (26), and ( r sk ) 3,4 = ( 1 ± i ) q 7 and ( r sk ) 5,6 = -( 1 ± i ) q 7 according to Equation (53). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

(16) For b 3 &gt; 0, b 2 1 &lt; 4 b 2, b 1 = 0, and b 2 &gt; 0, we have ( r sk ) 1,2 = ± iq 0 according to Equation (26), and ( r sk ) 3,4 = ( 1 ± i ) q 7 and ( r sk ) 5,6 = -( 1 ± i ) q 7 according to Equation (53). The solutions to the ordinary differential equations defined by the second part of Equation (8) are

<!-- formula-not-decoded -->

Substituting each of the Equations (29)-(32), (35)-(40), (48)-(51), and (55)-(56) into Equation (10), we can determine the six unknown constants Ctjk ( j = 1, 2, · · · , 6 ) .

Introducing the obtained Gt 0 ( u ) in Equation (14) and Gtk ( u ) in one of the Equations (29)-(32), (35)-(40), (48)-(51), and (55)-(56) into Equation (6), we obtain the mathematical equations of t ( u , v ) ( t = x , y , z ) and use them to create blending surfaces. In the following section, we will give an example to demonstrate the application of the closed-form solutions developed in this section in surface blending.

## 5. Applications

In this section, we first use some of the solutions obtained in this paper to create blending surfaces between primary surfaces. After that, we investigate how different shape control parameters and their combinations affect the shape of blending surfaces.

For all the examples presented in the paper, the parametric equations for the top and bottom primary surfaces are

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Assuming that the top boundary, u = 0, of the middle blending surface is taken to be at u = u 0 of the top primary surface, letting u = u 0 in Equation (57), we obtain the following blending boundary constraints between the top primary surface and the middle blending surface at the position u = u 0:

<!-- formula-not-decoded -->

Assuming that the bottom boundary, u = 1, of the middle blending surface is taken to be at u = u 1 of the bottom primary surface, letting u = u 1 in Equation (58), we obtain the following blending boundary constraints between the bottom primary surface and the middle blending surface at the position u = u 1 :

<!-- formula-not-decoded -->

and The blending boundary constraints (59a) and (59b) indicate that the elementary functions are f x 1 ( v ) = sin 2 π v for the x component, f y 1 ( v ) = cos 2 π v for the y component, and constants for the z component. Substituting Gx 0 ( u )= Gy 0 ( u ) = 0, f x 1 ( v ) = sin 2 π v , f y 1 ( v ) = cos 2 π v , and Kx = Ky = 1 for the x and y components and Kz = 0 for the z component into Equation (6), we obtain the following constructed functions of the blending surface:

<!-- formula-not-decoded -->

With the method developed in the previous section, we obtain the unknown functions Gx 1 ( u ) , Gy 1 ( u ) , and Gz 0 ( u ) . Substituting them back into Equation (60), we obtain the functions defining the blending surface.

The geometric parameters are taken to be a = 2.6, b = 4.5, c = h 3 = 5, d = h 1 = 2, h 2 = 3, u 0 = 0.35, and u 1 = 0.3. On average, the CPU time used to determine all the unknown constants of a blending surface with the above closed-form solutions is 20 microseconds on a laptop with 2.5 GHz and 8 GB of RAM.

When users use the proposed approach to carry out surface blending tasks, they can specify a combination of the shape control parameters γ , η , λ , and ρ . If the combination satisfies the conditions of one of Equations (29)-(32), (35)-(40), (48)-(51), or (55)-(56) such as the conditions of b 3 &lt; 0, b 2 1 &lt; 4 b 2, and b 1 &lt; 0, the blending surface is created. If the combination does not satisfy the conditions of any equations, users can specify a new combination of the shape control parameters to create a blending surface. Working in this way can easily and quickly create blending surfaces. However, if users want to use a specified equation from Equations (29)-(32), (35)-(40), (48)-(51), and (55)-(56) to create a blending surface, it may be very difficult to find a combination of the shape control parameters that satisfies the conditions required by the specified equation. This is because the relationships between b 1 , b 2, and b 3 and the shape control parameters γ , η , λ , and ρ are complex and cannot be explicitly presented as shown in Equations (21)-(24), (18), and (9).

In spite of the difficulty, we have found some combinations of shape control parameters for the specified equations listed in Table 1 and show the obtained blending surfaces in Figure 1.

Table 1. Shape control parameters used to create blending surfaces in Figure 1.

| Shape Control Parameters   | Shape Control Parameters   | Shape Control Parameters   | Shape Control Parameters   | Equation Number   | Figure Number   |
|----------------------------|----------------------------|----------------------------|----------------------------|-------------------|-----------------|
| γ                          | η × ( - 4 π 2 )            | λ × ( 16 π 4 )             | ρ × ( - 64 π 6 )           | Equation Number   | Figure Number   |
| 1.0                        | - 1.2 × 10 2               | 4.5 × 10 3                 | - 5 × 10 4                 | Equation (29)     | Figure 1a       |
| 1.0                        | 9.0 × 10 1                 | 1.5 × 10 3                 | - 2.5 × 10 4               | Equation (30)     | Figure 1b       |
| 1.0                        | 8.8 × 10 1                 | - 1.79 × 10 2              | 9.0 × 10 1                 | Equation (31)     | Figure 1c       |
| 1.0                        | - 2.95 × 10 1              | - 1.0 × 10 1               | - 1.5 × 10 2               | Equation (48)     | Figure 1d       |
| 1.0                        | - 8.0 × 10 1               | 2.37 × 10 3                | - 2.34 × 10 4              | Equation (49)     | Figure 1e       |
| 1.0                        | 9.9 × 10 1                 | - 9.7 × 10 1               | 3.0 × 10 2                 | Equation (51)     | Figure 1f       |

The method proposed in this paper provides a powerful tool for quickly creating different shapes of a blending surface but still exactly satisfying given blending boundary constraints. We will demonstrate this with some examples below. For all these examples, the geometric parameters are same as above.

First, we fix the shape control parameters η = λ = ρ = 1 unchanged but set the shape control parameter γ to 1, -0.9, -1.1, -1.3, -1.5, -1.6, -1.7, -1.75, and -1.8. We obtain different shapes of the blending surface, which are depicted in Figure 2.

and the

sented shape control as shown

In

eters

in spite for parameters in Equations of the

𝛾

,

𝜂

,

(21)-(24),

difficulty,

the specified

Figure

1.

When the shape control parameter γ changes from 1 to -1.8, the shape of the blending surface changes from concave to convex. At the value γ = -1.5, the middle part of the blending surface becomes a straight cylinder with a uniform cross-section size. When the shape control parameter γ changes from -1.5 to 1, the blending surface becomes more and more concave. When the shape control parameter γ changes from -1.5 to -1.8, the blending surface becomes more and more convex with a symmetry plane at the middle of the blending surface. Although different shapes of the blending surface are obtained by different values of the shape control parameter γ , the same continuities at the two trimlines are exactly maintained. Table 1. Shape control parameters used to create blending surfaces in Figure 1. Shape Control Parameters Equation Number Figure Number 𝜸 𝜼 ൈ ሺെ𝟒𝝅 𝟐 ሻ 𝝀 ൈ ሺ𝟏𝟔𝝅 𝟒 ሻ 𝝆 ൈ ሺെ𝟔𝟒𝝅 𝟔 ሻ 1.0 െ 1.2 ൈ 10 ଶ 4.5 ൈ 10 ଷ െ 5 ൈ 10 ସ Equation (29) Figure 1a 1.0 9.0 ൈ 10 ଵ 1.5 ൈ 10 ଷ െ 2.5 ൈ 10 ସ Equation (30) Figure 1b 1.0 8.8 ൈ 10 ଵ െ 1.79 ൈ 10 ଶ 9.0 ൈ 10 ଵ Equation (31) Figure 1c 1.0 െ 2.95 ൈ 10 ଵ െ 1.0 ൈ 10 ଵ െ 1.5 ൈ 10 ଶ Equation (48) Figure 1d 1.0 െ 8.0 ൈ 10 ଵ 2.37 ൈ 10 ଷ െ 2.34 ൈ 10 ସ Equation (49) Figure 1e 1.0 9.9 ൈ 10 ଵ െ 9.7 ൈ 10 ଵ 3.0 ൈ 10 ଶ Equation (51) Figure 1f

Figure 1. Blending surfaces created with different closed-form solutions. Figure 1. Blending surfaces created with different closed-form solutions.

<!-- image -->

The method proposed in this paper provides a powerful tool for quickly creating different shapes of a blending surface but still exactly satisfying given blending boundary constraints. We will demonstrate this with some examples below. For all these examples, the geometric parameters are same as above. Second, we set the shape control parameters γ = -1.8 and λ = ρ = 1 and keep them unchanged. Then, we set the shape control parameter η to 1.2, 1.4, 1.6, 1.8, 2.2, 3, 4, 4.6, 4.9, 5, 6, 7, 8, and 9. Different shapes of the blending surface are obtained and shown in Figure 3.

First, we fix the shape control parameters 𝜂 ൌ 𝜆 ൌ 𝜌 ൌ 1 unchanged but set the shape control parameter 𝛾 to 1, -0.9, -1.1, -1.3, -1.5, -1.6, -1.7, -1.75, and -1.8. We obtain different shapes of the blending surface, which are depicted in Figure 2. According to Figure 3, the shape change of the blending surface can be divided into four parts. The first part is Figure 3a-e. In this part, the value of the shape control parameter η increases from 1.2 to 2.2, causing the blending surface to change its shape from a convex one to a straight cylinder with a uniform cross-section size. The second part is Figure 3e-g. In this part, the value of the shape control parameter η increases from 2.2 to 4, causing the blending surface to change its shape from the straight cylinder into the frustum of a cone with the lower part of the frustum becoming smaller and smaller. The third part is Figure 3g-j. In this part, the value of the shape control parameter η increases from 4 to 5, which causes the following changes: (1) the lower part continues to become smaller and smaller, and (2) the upper part changes from straight to convex with bigger cross-section sizes. The fourth part is Figure 3k-n. There is a jump in the shape change between the third part and the fourth part. In the third part, the blending surface has a large cross-section size at the upper part and a small cross-section size at the lower part. After the jump, the shapes of the blending surface in the fourth part show that the cross-section size at the upper part becomes small, but the cross-section size at the lower part becomes large. In the fourth part, the value of the shape control parameter η increases from 6 to 9, which makes the lower part of the blending surface become smaller and smaller.

we have

equations listed

𝜆

,

and

𝜌

(18), and

found

in are

(9).

some

Table combinations

1

and complex and cannot be of shape show the obtained explicitly control pre- param- blending surfaces FOR

PEER

REVIEW

14

of

Figure 2. Different shapes of a blending surface determined by the shape control parameters 𝜂 ൌ 𝜆 ൌ 𝜌 ൌ 1 and different values of the shape control parameter 𝛾 . Figure 2. Different shapes of a blending surface determined by the shape control parameters η = λ = ρ = 1 and different values of the shape control parameter γ .

<!-- image -->

When the shape control parameter 𝛾 changes from 1 to -1.8, the shape of the blending surface changes from concave to convex. At the value 𝛾 ൌ െ 1.5, the middle part of the blending surface becomes a straight cylinder with a uniform cross-section size. When the shape control parameter 𝛾 changes from -1.5 to 1, the blending surface becomes more and more concave. When the shape control parameter 𝛾 changes from -1.5 to -1.8, the blending surface becomes more and more convex with a symmetry plane at the middle of the For one given combination of the shape control parameters γ , η , and ρ , changing the shape control parameter λ can obtain different shapes of blending surfaces. For another given combination of the shape control parameters γ , η , and ρ , changing the shape control parameter λ can obtain more different shapes of blending surfaces. Here, we use three different combinations of the shape control parameters γ , η , and ρ to demonstrate this.

blending surface. Although different shapes of the blending surface are obtained by different values of the shape control parameter 𝛾 , the same continuities at the two trimlines are exactly maintained. The first combination is the shape control parameters γ = -1.8, η = 6, and ρ = 1 unchanged. We set the shape control parameter λ to 5, 4, 3, 2, and 0.6. The obtained shapes of the blending surface are shown in Figure 4.

Second, we set the shape control parameters 𝛾 ൌ െ 1.8 and 𝜆 ൌ 𝜌 ൌ 1 and keep them unchanged. Then, we set the shape control parameter 𝜂 to 1.2, 1.4, 1.6, 1.8, 2.2, 3, 4, 4.6, 4.9, 5, 6, 7, 8, and 9. Different shapes of the blending surface are obtained and shown It can be seen from Figure 4 that when the shape control parameter λ reduces its value from 5 to 0.6, the upper part of the blending surface become smaller and smaller. At λ = 0.6, the upper part of the blending surface becomes very small.

in Figure 3. The second combination is the shape control parameters γ = -1.8, η = 1, and ρ = 1 unchanged. We set the shape control parameter λ to 1.1, 1.2, 1.3, 1.4, 1.6, 1.8, 2.2, and 2.6. The obtained shapes of the blending surface are shown in Figure 5.

The images depicted in Figure 5 indicate that this combination leads to convex shapes with the biggest cross-section size at the middle of the blending surface. When the shape control parameter λ increases its value from 1.1 to 2.6, the blending surface becomes less convex, and the cross-section size at the middle of the blending surface becomes smaller and smaller.

The third combination is the shape control parameters γ = -1.8, η = 4.6, and ρ = 1 unchanged. We set the shape control parameter λ to 1.2, 1.4, 1.6, 1.8, and 2. The obtained shapes of the blending surface are shown in Figure 6.

Mathematics

2024

,

12

,

x

FOR

PEER

REVIEW

15

of

Figure 3. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜆 ൌ 𝜌 ൌ 1 , and different values of the shape control parameter 𝜂 . Figure 3. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, λ = ρ = 1, and different values of the shape control parameter η .

<!-- image -->

According

parts.

to

The

fi

𝜂

Figure

rst

increases part

3,

is

from

1.2

the shape

Figure

to change

3a-e.

In

2.2, this causing the of the blending part, the value blending surface of surface the can shape to change be divided control its into param- shape from

a

four

eter Mathematics

2024

,

12

,

x

FOR

PEER

combination of the

eter can obtain

𝜆

combinations shape more control parameters different of the

The rst shape

,

,

and

𝛾

𝜂

shapes of blending

control

combination

fi

unchanged.

shapes of We the parameters is the set the shape blending

,

changing

𝜌

surfaces.

𝛾

,

the shape

Here, we use

,

and

𝜂

shape to demonstrate

𝜌

control

control

surface are parameters parameter shown in control param- three different this.

𝛾 ൌ െ

,

1.8

𝜆

to

5,

4,

Figure

4.

Figure 4. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜂 ൌ 6, 𝜌 ൌ 1 , and different values of the shape control parameter 𝜆 . Figure 4. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, η = 6, ρ = 1, and different values of the shape control parameter λ . REVIEW 17 of 22

<!-- image -->

It can be seen from Figure

4

that when the shape control parameter

𝜆

reduces its

smaller.

At

𝜌 ൌ

and

1

2.6.

Figure 5. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜂 ൌ 1, 𝜌 ൌ 1 , and different values of the shape control parameter 𝜆 . Figure 5. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, η = 1, ρ = 1, and different values of the shape control parameter λ .

<!-- image -->

The images depicted in Figure 5 indicate that this combination leads to convex shapes with the biggest cross-section size at the middle of the blending surface. When the shape control parameter 𝜆 increases its value from 1.1 to 2.6, the blending surface becomes less convex, and the cross-section size at the middle of the blending surface becomes smaller and smaller. The images in Figure 6 indicate that for this combination and the different values of the shape control parameter λ , the upper shape of the blending surface is convex, and the lower shape of the blending surface is concave. When the value of the shape control parameter

The third

unchanged.

combination

We

shapes set the is shape the of

The blending images

shape surface in Figure control the shape control control parameter are shown

6

indicate

parameter

𝜆

,

the parameters

𝜆

in to

1.2,

𝛾 ൌ െ

1.4,

Figure

that

upper

6.

for

shape this combination of the and the blending surface

1.8

1.6,

1.8,

,

𝜂 ൌ

4.6

and

2.

,

and

The

different

is

𝜌 ൌ

1

obtained

values

convex, and of the the lower

3,

2, and

𝜂 ൌ

0.6.

,

and

6

The

𝜌 ൌ

obtained Mathematics

2024

Mathematics

2024

,

,

12

12

,

x

,

x

FOR

FOR

λ increases from 1.2 to 2, the upper part of the blending surface becomes more and more convex, but the lower part of the blending surface becomes more and more concave.

PEER

PEER

Similarly, for one given combination of the shape control parameters γ , η , and λ , changing the shape control parameter ρ can obtain different shapes of blending surfaces. For another given combination of the shape control parameters γ , η , and λ , changing the shape control parameter ρ can obtain more different shapes of blending surfaces. Here, we use two different combinations of the shape control parameters γ , η , and λ to demonstrate this. REVIEW 18 of 22 REVIEW 18 of 22

𝛾 ൌ

𝜆

<!-- image -->

changing

,

surfaces.

Figure 6. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜂 ൌ 4.6, 𝜌 ൌ 1 , and different values of the shape control parameter 𝜆 . Figure 6. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, η = 4.6, ρ = 1, and different values of the shape control parameter λ . For another given combination of the shape control parameters 𝛾 , 𝜂 , and 𝜆 , changing the shape control parameter 𝜌 can obtain more different shapes of blending surfaces. Here,

we the two shape control different parameter combinations

𝜌

the can obtain shape different control shapes parameters of blending

𝛾

Similarly, for one given combination of the shape control parameters 𝛾 , 𝜂 , and 𝜆 , changing the shape control parameter 𝜌 can obtain different shapes of blending surfaces. For another given combination of the shape control parameters 𝛾 , 𝜂 , and 𝜆 , changing the shape control parameter 𝜌 can obtain more different shapes of blending surfaces. Here, The first combination is the shape control parameters γ = -1.8 and η = λ = 1 unchanged. Then, we set the value of the shape control parameter ρ to 1.15, 1.2, 1.25, 1.3, 1.35, and 1.4. The obtained shapes of the blending surface are shown in Figure 7. strate this. The fi rst combination is the shape control parameters 𝛾 ൌ െ 1.8 and 𝜂 ൌ 𝜆 ൌ 1 unchanged. Then, we set the value of the shape control parameter 𝜌 to 1.15, 1.2, 1.25, 1.3, 1.35, and 1.4. The obtained shapes of the blending surface are shown in Figure 7.

we use use of combinations

,

𝜂

,

and

𝜆

parameters

𝛾

,

𝜂

,

to demon-

and

and to demon-

𝜆

𝜂 ൌ 𝜆 ൌ

1.15, un-

1

1.2,

1.25,

Figure

𝜂 ൌ 𝜆 ൌ

<!-- image -->

1,

Figure 7. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜂 ൌ 𝜆 ൌ 1 , and different values of the shape control parameter 𝜌 . Figure 7. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, η = λ = 1, and different values of the shape control parameter ρ .

(

d

)

𝛾 ൌ െ

1.8,

𝜂 ൌ 𝜆 ൌ

𝜌 ൌ

Figure

7.

1.3

Different

1,

shapes

(

e

)

𝛾 ൌ െ

1.8,

𝜌 ൌ

𝜂 ൌ 𝜆 ൌ

1.35

blending surface

1,

determined

(

f

)

by

𝛾 ൌ െ

1.8,

𝜌 ൌ

the

𝜂 ൌ 𝜆 ൌ

1.4

shape control

1,

parameters

7.

1.3,

two different of the shape control of

a

𝛾 ൌ

less

the convex.

blending When the surface

The second

unchanged.

value of

becomes

combination

Then,

1.3, and

2.

we set

The

The the more is the the obtained images

has

a

different

When value shapes shown shape the shape shape control concave.

shape control

of

of

in Figure

change

control the shape the blending parameter parameters

𝜌

increases

𝛾 ൌ െ

control from

1.8

,

𝜂 ൌ

parameter

surface

8

indicate

in are that comparison

parameter

𝜌

shown

the

6

1.3

,

𝜌

to

in

upper

with

increases the from part of lower

0.1

to part

2, the

0.1,

Figure and

0.3, to

1.4,

𝜆 ൌ

1

0.5,

8.

the blending

of the

upper blending part of

0.7,

surface

surface.

the blend-

It can be observed from the images depicted in Figure 7 that the shape of the blending surface is convex at ρ = 1.15, straight at ρ = 1.3, and concave at ρ = 1.4. When the value of the shape control parameter ρ increases from 1.15 to 1.3, the blending surface becomes less convex. When the value of the shape control parameter ρ increases from 1.3 to 1.4, the blending surface becomes more concave. ing surface almost maintains a similar shape, but the lower part of the blending surface reduces its cross-section size greatly until its front view shape changes from a convex curved one at 𝜌 ൌ 0.1 into a straight one at 𝜌 ൌ 2 . From the above discussions, we can conclude the following: (1) the proposed closedform solutions are correct, easy to use, efficient, and powerful in shape control of blending surfaces, (2) each of the four shape control parameters has a strong influence on the shape

The second combination is the shape control parameters γ = -1.8, η = 6, and λ = 1 unchanged. Then, we set the value of the shape control parameter ρ to 0.1, 0.3, 0.5, 0.7, 1.3, and 2. The obtained shapes of the blending surface are shown in Figure 8. of blending surfaces, (3) different combinations of the four shape control parameters can create an enormous number of shapes from the same blending surface, and (4) the four shape control parameters can be developed into a powerful shape control handles to con-

trol the shape of

a

blending surface.

Figure 8. Different shapes of a blending surface determined by the shape control parameters 𝛾 ൌ െ 1.8, 𝜂 ൌ 6, 𝜆 ൌ 1 , and different values of the shape control parameter 𝜌 . Figure 8. Different shapes of a blending surface determined by the shape control parameters γ = -1.8, η = 6, λ = 1, and different values of the shape control parameter ρ .

<!-- image -->

6.

Conclusions and Future Work

In this paper, we have developed a new method to create 𝐶 ଶ continuous blending surfaces. The method is based on three sixth-order partial differential equations, respectively, for 𝑥 , 𝑦 , and 𝑧 components subjected to 𝐶 ଶ continuous blending constraints. To solve the sixth-order partial differential equations, we have decomposed blending boundary constraints into elementary functions and constructed blending surface functions from the decomposed elementary functions. By introducing the constructed blending surface functions into the sixth-order partial differential equations and blending boundary The images shown in Figure 8 indicate that the upper part of the blending surface has a different shape change in comparison with the lower part of the blending surface. When the shape control parameter ρ increases from 0.1 to 2, the upper part of the blending surface almost maintains a similar shape, but the lower part of the blending surface reduces its cross-section size greatly until its front view shape changes from a convex curved one at ρ = 0.1 into a straight one at ρ = 2.

constraints, we have transformed the sixth-order partial differential equations into sixthFrom the above discussions, we can conclude the following: (1) the proposed closedform solutions are correct, easy to use, efficient, and powerful in shape control of blending surfaces, (2) each of the four shape control parameters has a strong influence on the shape of blending surfaces, (3) different combinations of the four shape control parameters can create an enormous number of shapes from the same blending surface, and (4) the four shape control parameters can be developed into a powerful shape control handles to control the shape of a blending surface.

## 6. Conclusions and Future Work

In this paper, we have developed a new method to create C 2 continuous blending surfaces. The method is based on three sixth-order partial differential equations, respectively, for x , y , and z components subjected to C 2 continuous blending constraints. To solve the sixth-order partial differential equations, we have decomposed blending boundary constraints into elementary functions and constructed blending surface functions from the decomposed elementary functions. By introducing the constructed blending surface functions into the sixth-order partial differential equations and blending boundary constraints, we have transformed the sixth-order partial differential equations into sixth-order ordinary differential equations and corresponding blending boundary constraints, solved the sixth-order ordinary differential equations subjected to the corresponding blending boundary constraints analytically to obtain their closed-form solutions, and substituted the obtained closed-form solutions into the constructed blending surface functions to solve C 2 continuous surface blending problems.

We have applied the developed method to create blending surfaces and investigate how different shape control parameters and their combinations affect the shape of blending surfaces. The application indicates that the proposed method is simple and easy to use, the obtained closed-form solutions are correct, and the shape control parameters and their combinations are effective at controlling the shape of blending surfaces while C 2 continuous blending boundary constraints are exactly satisfied.

With the approach proposed in this paper, blending surfaces are created from Equation (6) and one of Equations (29)-(32), (35)-(40), (48)-(51), (55), and (56). Since Equations (29)-(32), (35)-(40), (48)-(51), (55), and (56) involve complicated mathematical functions, finding the relationships between the shape of blending surfaces and the shape control parameters is a challenge. Although this paper has presented many examples to show how different shape control parameters and their combinations affect the shape of blending surfaces, the intuitiveness of the shape control parameters still requires further investigation. In our future work, we intend to address this issue by proposing an optimization method outlined below.

First, users draw one or more profile curves to define the shape of the blending surface that they want to achieve. For example, users draw one profile curve C 0 ( u ) = [ Cx 0 ( u ) Cy 0 ( u ) Cz 0 ( u ) ] T at v = 0.0 to define the shape of a blending surface. Substituting v = 0.0 into X ( u , v , γ , η , λ , ρ ) = [ x ( u , v , γ , η , λ , ρ ) y ( u , v , γ , η , λ , ρ ) z ( u , v , γ , η , λ , ρ ) ] T defined in Equation (6), we obtain the profile curve X ( u , 0.0, γ , η , λ , ρ ) = [ x ( u , 0.0, γ , η , λ , ρ ) y ( u , 0.0, γ , η , λ , ρ ) z ( u , 0.0, γ , η , λ , ρ ) ] T . Next, we calculate the squared difference between X ( u , 0.0, γ , η , λ , ρ ) and C 0 ( u ) and obtain D ( u , γ , η , λ , ρ ) = [ X ( u , 0.0, γ , η , λ , ρ ) -C 0 ( u )] 2 . After that, we minimize the difference and obtain γ , η , λ , and ρ by solving the nonlinear equations ∂ D ( u , γ , η , λ , ρ ) / ∂γ = 0, ∂ D ( u , γ , η , λ , ρ ) / ∂η = 0, ∂ D ( u , γ , η , λ , ρ ) / ∂λ = 0, and ∂ D ( u , γ , η , λ , ρ ) / ∂ρ = 0. Finally, substituting the obtained shape control parameters into Equation (6), we obtain the blending surface X ( u , v ) , whose profile curve X ( u , 0.0 ) at v = 0.0 approximates the users' drawn profile curve C 0 ( u ) .

If users draw more profile curves, the above method is still applicable. For example, if users draw four profile curves C 0 ( u ) at v = 0.0, C 0.25 ( u ) at v = 0.25, C 0.5 ( u ) at v = 0.5, and C 0.75 ( u ) at v = 0.75, the squared difference can be formulated as D ( u , γ , η , λ , ρ ) = [ X ( u , 0.0, γ , η , λ , ρ ) -C 0 ( u )] 2 + [ X ( u , 0.25, γ , η , λ , ρ ) -C 0.25 ( u )] 2 + [ X ( u , 0.5, γ , η , λ , ρ ) -C 0.5 ( u )] 2 + [ X ( u , 0.75, γ , η , λ , ρ ) -C 0.75 ( u )] 2 . Then, ∂ D ( u , γ , η , λ , ρ ) / ∂γ = 0, ∂ D ( u , γ , η , λ , ρ ) / ∂η = 0, ∂ D ( u , γ , η , λ , ρ ) / ∂λ = 0, and ∂ D ( u , γ , η , λ , ρ ) / ∂ρ = 0 can be used to obtain γ , η , λ , and ρ and the blending surface X ( u , v ) .

In this paper, we use position functions and first and second partial derivatives on the isocurves u = u 0 and u = u 1 of primary surfaces to determine boundary constraints of a blending surface. In some situations, a blending surface is required to smoothly connect two primary surfaces at non-isocurves. In our following work, we will extend our proposed approach to tackle surface blending problems in such situations. The basic idea of extending our proposed approach is as follows.

First, we determine one trimline, which is a 3D non-isocurve, on one primary surface and another trimline, which is also a 3D non-isocurve, on anther primary surface. This can be achieved by manually drawing a 3D non-isocurve on a primary surface or introducing u = f ( v ) into a primary surface S ( u , v ) to obtain a 3D non-isocurve. Then, the two trimlines can be formulated as B 0 ( v ) = [ Bx 0 ( v ) By 0 ( v ) Bz 0 ( v ) ] T and B 3 ( v ) = [ Bx 3 ( v ) By 3 ( v ) Bz 3 ( v ) ] T , respectively, where the subscript '0' indicates the trimline on the first primary surface, and the subscript '3' denotes the timeline on the second primary surface. After obtaining the trimlines B 0 ( v ) and B 3 ( v ) , we can use them and the two primary surfaces to determine the first and second partial derivatives, respec-

## References

1. Vida, J.; Martin, R.R.; Varady, T. A survey of blending methods that use parametric surfaces. Comput.-Aided Des. 1994 , 26 , 341-365. [CrossRef]
2. Hatna, A.; Grieve, R.J.; Broomhead, P. Surface blending for machining purposes: A brief survey and application for machining compound surfaces. Proc. Inst. Mech. Eng. Part B J. Eng. Manuf. 2001 , 215 , 1397-1408. [CrossRef]
3. Rossignac, J.R.; Requicha, A.A.G. Constant-radius blending in solid modeling. ASME Comput. Mech. Eng. 1984 , 3 , 65-73.
4. Bloor, M.I.G.; Wilson, M.J. Generating blend surfaces using partial differential equations. Comput.-Aided Des. 1989 , 21 , 165-171. [CrossRef]
5. Choi, B.K.; Ju, S.Y. Constant-radius blending in surface blending. Comput.-Aided Des. 1989 , 21 , 213-220. [CrossRef]
6. Farouki, R.A.M.; Sverrisson, R. Approximation of rolling-ball blends for free-form parametric surfaces. Comput.-Aided Des. 1996 , 28 , 871-878. [CrossRef]
7. K ó s, G.; Martin, R.R.; Vrady, T. Methods to recover constant radius rolling ball blends in reverse engineering. Comput. Aided Geom. Des. 2000 , 17 , 127-160. [CrossRef]
8. Dahl, H.E.I.; Krasauskas, R. Rational fixed radius rolling ball blends between natural quadrics. Comput. Aided Geom. Des. 2012 , 29 , 691-706. [CrossRef]
9. Chuang, J.H.; Lin, C.H.; Hwang, W.C. Variable-radius blending of parametric surfaces. Vis. Comput. 1995 , 11 , 513-525. [CrossRef]
10. Chuang, J.H.; Hwang, W.C. Variable-radius blending by constrained spine generation. Vis. Comput. 1997 , 13 , 316-329. [CrossRef]
11. Luk á cs, G.; Hermann, T.; V á rady, T. Continuity and self-intersections of variable radius rolling ball blend surfaces. In Product Modeling for Computer Integrated Design and Manufacture, IFIP Advances in Information and Communication Technology ; Pratt, M.J., Sriram, R.D., Wozny, M.J., Eds.; Springer: Boston, MA, USA, 1997.
12. Luk á cs, G. Differential geometry of G1 variable-radius rolling ball blend surfaces. Comput. Aided Geom. Des. 1998 , 15 , 585-613. [CrossRef]
13. Chuang, J.-H.; Lien, P.-L. One and two-parameter blending for parametric surfaces. J. Inf. Sci. Eng. 1998 , 14 , 461-477.
14. K ó s, G. Recovering variable radius rolling ball blends in reverse engineering. J. Manuf. Sci. Prod. 2011 , 3 , 151-158.
15. Bloor, M.I.G.; Wilson, M.J.; Hagen, H. The smoothing properties of variational schemes for surface design. Comput. Aided Geom. Des. 1995 , 12 , 381-394. [CrossRef]
16. Cheng, S.Y.; Bloor, M.I.G.; Saia, A.; Wilson, M.J. Blending between quadric surfaces using partial differential equations. In Advances in Design Automation, Vol 1. Computer and Computational Design ; Ravani, B., Ed.; ASME Press: New York, NY, USA, 1990; pp. 257-263.
17. Brown, J.M.; Bloor, M.I.G.; Bloor, M.S.; Wilson, M.J. The accuracy of B-spline finite element approximations to PDE surfaces. Comput. Methods Appl. Mech. Eng. 1998 , 158 , 221-234. [CrossRef]
18. Li, Z.C. Boundary penalty finite element methods for blending surfaces: I. Basic theory. J. Comput. Math. 1998 , 16 , 457-480.
19. Li, Z.C. Boundary penalty finite element methods for blending surfaces: II. Biharmonic equations. J. Comput. Math. 1999 , 110 , 55-176. [CrossRef]
20. Li, Z.C.; Chang, C.-S. Boundary penalty finite element methods for blending surfaces. III. Superconvergence and stability and examples. J. Comput. Appl. Math. 1999 , 110 , 241-270. [CrossRef]
21. You, L.H.; Zhang, J.J.; Comninos, P. Blending surface generation using a fast and accurate analytical solution of a fourth order PDE with three shape control parameters. Vis. Comput. 2004 , 20 , 199-214. [CrossRef]
22. You, L.H.; Comninos, P.; Zhang, J.J. PDE blending surfaces with C2 continuity. Comput. Graph. 2004 , 28 , 895-906. [CrossRef]

tively, crossing the trimlines B 0 ( v ) and B 3 ( v ) . The first derivative and second derivative crossing the trimline B 0 ( v ) can be formulated as B 1 ( v ) and B 2 ( v ) , and those crossing the trimline B 3 ( v ) can be formulated as B 4 ( v ) and B 5 ( v ) . Substituting B 0 ( v ) , B 1 ( v ) , . . ., B 5 ( v ) into Equation (1), we obtain the blending boundary constraints on non-isocurves of primary surfaces. With the obtained blending boundary constraints and the approach proposed in this paper, we can tackle the problems of blending primary surfaces on their non-isocurves.

Author Contributions: Conceptualization, methodology, software, and visualization, X.Y.; supervision, F.T., W.T., J.C. and J.Z.; formal analysis and investigation, X.Y., F.T., W.T., J.C. and J.Z.; writing-original draft preparation, X.Y.; writing-review and editing, F.T., W.T., J.C. and J.Z.; resources, F.T., W.T., J.C. and J.Z. All authors have read and agreed to the published version of the manuscript.

Funding: This research received no external funding.

Data Availability Statement: The original contributions presented in the study are included in the article, further inquiries can be directed to the corresponding author.

Conflicts of Interest: The authors declare no conflicts of interest.

23. You, X.Y.; Tian, F.; Tang, W. C2 continuous blending of time-dependent parametric surfaces. ASME J. Comput. Inf. Sci. Eng. 2019 , 19 , 041005. [CrossRef]
24. You, X.Y. Differential Equation-Based Shape Interpolation for Surface Blending and Facial Blendshapes. PhD thesis, Bournemouth University, Poole, UK. Available online: https://eprints.bournemouth.ac.uk/36696/1/YOU,%20Xiangyu\_Ph.D.\_2022.pdf (accessed on 12 August 2024).

Disclaimer/Publisher's Note: The statements, opinions and data contained in all publications are solely those of the individual author(s) and contributor(s) and not of MDPI and/or the editor(s). MDPI and/or the editor(s) disclaim responsibility for any injury to people or property resulting from any ideas, methods, instructions or products referred to in the content.