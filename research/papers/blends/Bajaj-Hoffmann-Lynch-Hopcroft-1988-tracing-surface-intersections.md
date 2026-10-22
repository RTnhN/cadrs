## Tracing surface intersections

C.L. BAJAI , C.M. HOFFMANN 2, R.E. LYNCH

Computer Sciente Department,Purdue University,Wesr Lafayette,IN 47907,USA

J.E.H. HOPCROFT 3

Computer Science Deparinient, Cornell University, Ithaca, NY 14853,USA

ReceivedSeptember1987 Revised April 1988

Abstract.We consider the problem of tracing the intersection of surfaces given either implicitly or paramelrically. We give a numerical tracing procedure in which a third-order Taylor approximant is constructed for construction relates [o local paramelenizations of the curve al singularities, and discuss our experience with the melhod. For plane curves, given implicidy, we show how desingularization lechniques can be incorporaled to trace correculy through all types of singularities. An inmplementation of this method is also discussed.

## 1. Introduction

the inlersection of two surfaces. Existing geometric modeling systems typically restrict the [Requicha et al. '83], arbitrary quadrics [Levin '79, Ocken el al. '83], or parametric patches of various types [Boehm et al. '84, Pratt "86]. With such specializations many good techniques can be developed thal take advantage of the specific restrictions.

In this paper, we consider the evaluation of surface intersections in general. The intersecting surfaces may be specified implicitly as ∫(x, y, z) = 0 and f(x, y, z) = 0, where § and f, are smooth functions, or paraimetrically as (x = G,.n(u, ), y = G2.1(μ, U), z = G3.1(u, v) and (x = G1.2(μz, uz), y=G2.2(u2, u2), z= G3,2(u2, D2)), where the G.j, i =1, 2, 3, j= 1, 2, are smooth functions. In full generality, tracing the intersection curve is a difficult problem, and one of our objectives is to explore the scope of a purely numerical approach. In [Pratt et al. '86], Pratt and Geisow review several such methods. A common problem stems from the inherent geometric complexily of high degree algebraic curves that arise as curves of intersection. In particular, such a curve may possess singular points where the curve has an abrupt change of normal direction (cusps), multiple self-intersecting branches (nodes), or self-tangent branches (lacnodes).

A numerical tracing scheme technically requires formulating and solving a linear syslem of equations. Such a system is formulaled both for determining an approximanl to the curve at a given poinl, as well as when refining an estimate of the location of a point on the curve with Newton iteration. At a curve singularity, the linear system is singular, and at points nearby the condition number of the system is so large that roundoff errors destroy all accuracy of the estimate. Purely numerical tracing schemes have great dificulties in this situation: As the singularity is approached, these programs may fail. Even if they trace through the singularity without mishap, they may identify the curve branches incorrectly.

1 Supporled in pari by NSF Grant MIP 85-21356 and ARO Contract DAAG29-85-C0018 under Comel/MSI

A strength of the method lies in its ability to consolidate the compulation necded for the Newton iteration with the compulalion determining the power series expansion. Moreover, as we show, there is a close correspondence of the computational machinery needed by the meihod with an algebraic procedure for analyzing the curve at singular poinis. Although this correspondence is nol exploited in this paper, il permits a fairly simple exlension lo cope directly with a large class of singularities.

It is not known how to rectify all these difficulties with a single numerical method. Nevertheless, it is our experience that a carefully crafted numerical tracing rouline is capable of handling many of the difficulties characterized above. We propose here such a schcme in which the intersection curve is locally approximated by a low degree Taylor polynomial interpolanl, and a new curve point estimale is derived from it by taking sleps of variable lengths. Newton iteration is then used to refine this new point estimate.

Our approach to surface intersection tracing applies directly to solid modeling operations, for example when intersecling faces are defined on implicit surfaces. Moreover, when rendering curved faces, silhouelle curves of curved faces need to bc determined, and may be defined as the intersection of two surfaces. Another advanlage of our approach is that we can construcl higher order approximants to the intersection of parametric surfaces directly. Previously, only piecewise linear lechniques have been used that are constructed either from subdivision s         s have found, there is no difficulty in solving the underconstrained system and the slep length constraint is artificial.

] algebraic curves and surface intersections. The method technically relies on the Implicit Function Theorem, seeking 1o represent a curve branch explicitly in one coordinate as function of the olher coordinate(s). The advantage of such a representation is that it allows simple slepping techniques. On the other hand, the quality of approximation is limited by a more stringent convergence criterion, and the method does not seem to have a natural extension that handles singular points.

Next, we consider the special case of lracing plane algebraic curvcs defined implicitly a5 /(x, y) = 0. Tracing plane curves which are given parametrically simply amounts 1o evalualing the paramelric equations for several distincl parameter values. So, onc could try lo oblain a rational parameterization of f. Only curves of genus zero possess a rational parametric form, however. For algorithms to test whether and how implicitly defined plane curves can be rationally parameterized, see [Abhyankar el al. '87c].

The lracing of implicitly defined plane curves arises in solid modeling in a number of ways:

- (1) When the faces of a model are paramctric patches, with a-priori known implicit equations, edges bounding these patches can be represenled as planc curves in the parameler plane of one of the faces, see [Farouki '86].
- (2) When intersecting two implicit surfaces ∫(x, y, z)=0 and f(x, y, z)=0, one of them, say ∫1, might possess a rational paramelerization. If so, the parametric form can be determined in certain cases. By substituting thereafter the parametric equations of f into the implicit equation of ∫2, a plane curve in the parameler plane is obtained that is in birational correspondence with the intersection curve of ∫ and f. For efficienl algorithms to lest whether and how an implicit quadric or cubic algebraic surface can be parameterized, see [Abhyankar et al. '87a, Abhyankar et al. '87b].

1

(3) When intersecting nonralional implicit surfaces f(x, y, z) =0 and f(x, y, ε) = 0, one can always find a rational surface f(x, y, z)= 0 containing the intersection curve of §, and /. After f has been found, it is easily parameterized, and wc can obtain a plane curve by substituting as in (2) above. For methods to find fs see [Hoffmann *88, Snyder el al. '14, p. 219]. Furthermore, there are projection techniqucs that find a birational map to some plane algebraic curve directly and which, as part of the map so construcied, delermine / [Abhyankar et al. '87d, Garrity et al. '87].

Here, birational correspondence means 1hat in each direction rational maps exist. In conscquence, a tracing procedure for plane algebraic curvcs yields a tracing procedure for algebraic space curves. Note, however, that the corresponding plane curve migh1 have more singularities than the space curvc. Moreovcr, the dcgree of the curve is the product of 1he surface degrees, so that tracing the corresponding planar curve is numerically more delicatc. If the birational map is not derived carefully, finally, the degrcc of the plane curve may be even higher. Thus, for simple singularities, the purely numerical approach remains attractive.

Wc show that for plane algebraic curves the correct branch conneclivity can be achieved by utilizing resulis from algebraic geometry. The trace of f(x, y) = O comimences al a given input point withi a desired direction. At noncritical segmcnts, we proceed numerically as beforc. When the condition number of the system becomes very large, we try to locate a ncarby curve singularity. Then, by applying quadratic transformations, the branch of ∫ we trace is birationally mapped to a branch of a transformed curve g that has no singularities. The transformed branch is traced and the points of g are mapped to corresponding points of f. The Irace of g continues until we have passed the singularity of f. In this way, correct branch connectivity is achieved.

Geisow, [Geisow '83, Pralt el al. '86], discusses a number of prior approaches to tracing plane algebraic curves and then proposes subdivision. Brielly. the curve h(x, y)=0 is conceptualized as the inlersection of z = h(x, y) and z = 0, and afler translating z = h(x, y) into Bernstein form, several subdivision schemes are proposed for evaluating the curve in snrall regions in which it is well behaved. Although the meihod can cope with many singularities, no analysis is made to identify branch connectivity or to give an analysis of the struclure of the singularity. Waggenspack [Waggenspack "87], extends this approach by approximating within a subdivided region, the curve by a conic or a rational cubic.

There are algorithms for analyzing the lopology of real algcbraic curves in the plane, e.g.. [Arnon et al. '83]. Based on cylindrical algebraic decomposition, [Collins '83], these algorithms make extensive use of symbolic computalion and rool isolation to locatc critical curve points, that is, singularities and points whose tangenis are parallel lo one of the coordinalc axes. Thercafter, the critical poinis are connected with curve segments that are simple to trace. Algorithms of this lype will never fail. However, due lo extensive computations. they have not yet made an impact on solid and geometric modeling practices. Whether specialized versions will cvenlually be competitive in space or time remains to be seen.

## 2. Notation and definitions

Partial derivatives are written by subscripting, for example, f = af/ax. J. = a²f/(axdy), and so on. Since we consider analytic curves and surfaces, we have Jxy = J etc.

Vectors and vector functions arc denoled by bold letters. The immer produci of vcctors α and b is denoted a- b. The length of the veclor a is 1lαll = √a·a.

The gradienr of f is the vector Vf = (fx, Jy, f). The Hessian of f is the symmelric matrix

<!-- formula-not-decoded -->

The inlersection of § and § is dcnoted by r(s) and is a vector function of the argumcnt s, typically the arc lengih measured from somc point on the curve. Derivatives of r(s) are denoted ', r",..., s(om).

If one of the surfaces is a plane, then a simple coordinale transformation reduces the problem to tracing a plane curve f(x, y) = 0. Assume that this curve contains thc origin and is algehraic. Then the order form is the homogeneous polynomial F(x, y) consisting of the terms of lowcst degree in f. It contains information abou1 the curve's behavior at the origin. If the order form is linear, then the curve has a simple point at the origin, i.e., thc curve is not singular at the origin. If the order form is nonlinear, thcn the origin is a singulariry. The degree of F is then called the order of the singularity. Morcover, the linear faclors of F are equations of the rangenrs of the curve at the origin.

At point p =(x, y, z) is regular on ∫ if the gradient of f al p is not null; otherwise the and if the gradients Vf, and V/ are linearly independent. That is, the surfaccs are nol singular al p and intersect transversally.

An important concept from algebraic geometry, used to study the local curve structure, is that of place, e.g. [Walker '50, p. 96]. Brienly, a place of f(x, y) = 0 is a pair of power series

<!-- formula-not-decoded -->

such tha1 f(x(s), y(s) is identically zero. The place is said to be centered al the point (x(0), y(0)) of the curve. It is always possible to choose the place such that x(s) = s*, for some k. Intuitively, a place is a local parameterization of the curve, centercd al (x(0), y(O), with a cerlain radius of convergence thal varies with the place.

The order of a place centered a1 the origin is the lowest exponenl with a nonzero coeficien1 in the power series. For example, the order of

If the center c is no1 a singular poin1, thcn the place is equivalent 1o the Taylor series about c. If the c is singular, then the curve may have more than one distinc1 placc centered al c, each corresponding lo a distinct branch of the curve.

<!-- formula-not-decoded -->

is two, whereas the order of

<!-- formula-not-decoded -->

Centered al cvery nonsingular point, the curve has exactly one linear place, i.e., a place of order one. At a singular point the curve has one or more placcs which may or may not be linear. However, if there is only one place al a singular poin1, thcn this place must bc nonlinear.

is one.

## 3. Nonsingular curve points on surface interscctions

We consider first tracing the inlersection of implicit surfaces, /(x, y, z)=0 and ∫(x, y, z) = 0, given an initial curve point and a direction. In the simplest situalion we trace the intersection in a neighborhood in which both f and f. are regular and their gradients are linearly independent. Geometrically this means that the surfaces intersect transversally and are not singular in thc vicinity. We formulalc a system of cquations from which both the local approximation as well as the Newton iteration are derivcd. Under the assumption of linearly independent gradients, we have a system of linear equations of rank 2. The choices made when solving the system correspond to parameterizing the approximant by arc length.

## 3.1.Equarion for the intersecrion

We then skeich how this approach can be direcily transferred to tracing 1he intersection of parametric surfaces, (G(u, U1), G2.(, v1), Gs.,(μ, v)) and (G1.2(μz, v2), G22(uz, 2), G3,2(u2, 2), with the G,j, i = 1, 2, 3, j = 1, 2, as srmooth functions, given an initial curve point and a desired direction. Again, higher order approximants are easily constructed and are useful for estimaling a safe step lengih. Under the assumption of linearly independent gradicnts, we now have a system of linear equations of rank 3. It is clear that the approach generalizes lo tracing the intersection of  - 1 hypersurfaces in -dimensional space.

We treal the case that the intersection r is a function, having a1 least four continuous derivatives, of a parameler s. Then

<!-- formula-not-decoded -->

where p is the cubic Taylor interpolant to r al s = 0 and e is its error, or remainder. Below we give a numerical procedure for finding values of the derivatives, given a point qo on the intersection. Since e(s) = O(s*) in a bounded interval containing s = O, a sufficiently small s makes the value p(s) of the cubic an accurate eslimate of r(s). Using P(s) as an initial cstimate, one can then obtain another poinl, 41 on the inlersecuion with a very few sleps of Newton iteralion. The process then repeats. In this way a sequence of points, 9n, = 0, 1, 2,...,on the intersection is determined.

<!-- formula-not-decoded -->

The derivalives are necessarily not unique because the parameterization of r by s is nonunique. We choose s as arc length. Then the unit tangent , the unit principle normal n, and the unit binormal b are related by the Frenet-Serret formulae [Franklin '44, p. 107]:

where k=l/o is curvalure and T=1/ is torsion. The veclors f, n, and b form an orthonormal triad with n = b X 1. With s arc length, the derivatives of r arc given by

<!-- formula-not-decoded -->

## 3.2. Implicit definition

First suppose that points on the curve are defined as solulions of f;(x, y, z) =∫(r) = 0, j = 1, 2. The Taylor expansion of f,(r(s)) in powers of s is

where Vf, is the gradient of f; and H, its Hessian, both evaluated at r(0).

<!-- formula-not-decoded -->

Since the intersection satisfies f;(r(s)) = 0, the coefficient of cach powcr of s in (4) must be zero. Given a point q = r(O) on the intersection, the m-th derivative of r thcn satisfies

<!-- formula-not-decoded -->

The quanlities b,m arc expressed in terms of the partial derivativcs of f; and lower-order derivatives of r; e.g.:

<!-- formula-not-decoded -->

for b,a, see Appendix A.1. For each nl, (5) is a pair of cquations for the 1hree components of "(")(O). Appendix A.2 details how to solve this system with numerically stable techniques.

It follows from the independence of the gradients that there is a unit vector 1 which is perpendicular to both gradienis:

<!-- formula-not-decoded -->

Except for sign, r is unique. Any vcctor can be written as a linear combination of these three; in particular

<!-- formula-not-decoded -->

Direci substitution into (5) yields

<!-- formula-not-decoded -->

There is a unique solution, Pn, Ym, of this system and, therefore,

<!-- formula-not-decoded -->

with α, arbitrary, is the general solution of the system (5).

For m = 2, the unique solution of (6) and the choice α, = 0 leads to a uniquc vector r"(0). Thcn with x the positive square root of r"(0) - r"(o), we have r"(0) = kn, where n is the unit principlc normal to the intersection.

Because b.1 = b21 = 0 makes β = Y = 0, we have r'(0)= αm1. The choice α,= 1 makes 7'(0) a unit vector tangent lo the interseclion. For very small s, the term sr'(0) in (l) determines the orientation of the intersection and we choose the sign of r so as lo maintain the orientation when s is positive. Specifically, let r'-1 denote the derivative at the (n - 1)-th point on the intcrsection. Afler r is computed for the n-th point, if s'-1 ·? &lt; 0, then we replace I with -r.

Finaly, by taking αg = -x, we have obtained the first three derivatives of  related as in (3).

## 3.3.Paranetric definition

Next suppose that points on the surfaces are given in terms of parameters (n,, v), k = 1, 2:

<!-- formula-not-decoded -->

where the G;x are given smooth functions. The interseclion is defined by G,.(u, v)= G,a(u2, vz), j = 1, 2, 3, a system of three cquations in four unknowns. Once the unknowns have been determined as functions of s, points r(s) on the intersection are obtained by direct evaluation:

<!-- formula-not-decoded -->

where k is either 1 or 2.

Let R be the vector with four components defined by R =(u: ", H2, v2), and sct

Then F;(R(s)) = 0, and the Taylor expansion of F(R(s)) is

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

so that if Q is a solution of (9), then

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where T is a unit vector orthogonal to the three gradienis, and α, is arbitrary.

If the sel of threc gradients is linearly independent, then the general solution of (1l) is R(m)=αmT+βF +YmVF+8mVF,

(12)

Comparing equations (4), (5), and (7) with (10), (11), and (12), respectively. one sees that the only difference between 1he implicit and the parametric determination of the intersection is (a) the number of cormponents of the veciors and (b) the numher of cquations. Thus, a gcneral numerical method for one also applies to the other with minor modifications. In our Fortran implementation of the implicit formulation, this is accomplished by changing the sizc of some arrays and including the evaluation of points on the intersection with (8). Moreover, it should be evident that the method generalizes [o tracing the intersection of  - I smooth hypersurfaces in n-dimensional space assuming transversal intersections.

## 3.4.Newronupproximation

Given an initial point Po near the curve, we find a point g on the curve by gencrating a sequence of points P1, P2, :-. →g. We set f;(r(s)) = 0, r(0) = Pk and sr'(0) = △ in (4), and neglec1 the terms with higher powers of s Io get Newlon's method. Thus we solve

Equation (13) is the same as equation (5). When the pair of gradicnts is linearly indepcndent, the general solution is

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Once the point q is found with acceptable accuracy, the approximation of r(s) with r(0) = q is detcrmined as described above.

and the values of β, and Yk are detcrmined uniquely. Because r is orthogonal to both surfaces, a change of px in the direchion of r changes the values of f; only negligibly, and we set α, = 0, thereby obtaining a unique solution for Ak. We then set Px++ = Pk + Ax-

## 3.5. Step length

We use the higher order derivatives of r to estimate the accuracy of the low-order lerms in the Taylor approximation. With this estimale, a step size is chosen such that the contribution of the second and third order lerms together is at mos1 1/5 of the first order lerm. That is, we require that both

<!-- formula-not-decoded -->

are smaller than Isr'(0)il/10 = Isi/10. For an example see Section 3.7 below. Sincc the stcp sizcs could become arbitrarily small, a minimum step size is specified also.

This method for choosing step lengths is not guarantced 1o give accurale results in gcncral, but is simple and works in most situations, since the small magnitude of the highcr order terms often indicates convergence of the full Taylor series. For more sophisticated ways guaranleed to give accurale resulis, see [Montaudouin et al. '86].

## 3.6.Transformations oftheequations

The intersecuion of f and f is also the intersection of

<!-- formula-not-decoded -->

system

<!-- formula-not-decoded -->

where the b;, are as before.

<!-- image -->

Fig. 3.1. Cylinder-cylinder intersection, x² + z2 + 2= = 0ny²+z²+4z=0.

<!-- image -->

Fig. 3.3. Tacnode singularity, z+ x^ + y = 0 nz + y2 =0.

Fig. 3.2, Nodal singularity, = +y²- x = 0 = + x² = 0.

<!-- image -->

Fig.3.4.Tacnode and nodal singularitics,-2.xd-y4 =0nz-3x²y+y²-2y²=0.

<!-- image -->

in which Vf, and Vf are orihonormal or some of the intrinsic curve parameters, such as curvature radius, appear explicitly on the right side. This shifts the programming work to finding proper constants. Moreover, some of these choices parallel an algebraic approach to finding a local approximation at a singular curve point, as explained in Section 4.

## 3.7. 1inpiementation

We have implemented the numerical tracing procedure in Fortran. Figs. 3.1 through 3.8 show some examples of curve traces that were produced with this program and a slandard graph utility under Berkeley Unix. The plane curves have bcen Iraced as the intersection of f(x, y)=0 with z=0, without any program modifications. As described further in the appendix, the linear system is solved using singular value decomposition [Golub et al. '85, Stewart '73]. This approach is numerically very stable and increases the reliability in near-singular cases considerably.

<!-- image -->

Fig. 3.5. Projection of Fig. 3.2. onlo the plane :=0, y²-x²-x²=0.

<!-- image -->

Fig. 3,7. Two real components touching y2 - xd - y4 = 0.

Fig. 3.6. Projection of Fig. 3.4 onlo the plane 2 =0. 2x4-3x²y+y²-2y+y4=0.

<!-- image -->

Fig. 3.8. One self-intersecting real comiponent, y² - x6 '0=-

<!-- image -->

Table 1

| Point                           |   Itcr | Next Step   |
|---------------------------------|--------|-------------|
| (+0.199682 2+0.218711-0.039873) |        | 0,27637     |
| (+0.015686 6+0.015809-0.000246) |        | -0.19340    |
| (-0.124761-0.116720 -0.015565)  |        | +0.16081    |
| (-0.245628-0.213339-0.060333)   |        | +0.15031    |
| (-0.358096-0.286903-0.128233)   |  32222 | +0.15308    |

Al certain singularities, e.g., for the nodal singularity in Fig. 3.5, the curvc orientation Vf, × V/2 reverses. This is a global property thal depends on how the curve branches intersecl at the singularity. If one were lo determine its presence in this way, a complete analysis of the singularity would be required. To avoid this, we have added a heuristic thal reverses the lracing direction whenever the oriented tangent changes by more than a maximum angle, say 90

In our experience,nodal singularities cause noproblcms as long as the tangent directions of the intersecting branches are sufficicntly scparated. Many tacnodes are also handled reliably, e.g., Fig. 3.4. However, therc are situations where branches may be confused. For exanple, both the curve C,: y2 - x4 - y4 = 0 (Fig. 3.7) and the curve Cz: y2 - x6 - y6 = 0 (Fig. 3.8) are traced as if they had two real componenis meeting tangentially a1 the origin. Whilc this is correct for C, it is not correct for C2, since C2 consisis of a single real componcnt with two computation of Section 4.1 or the singularity analysis of [Owen el al. '87] does not sufficc to distinguish the two cases.

Table 1 above shows a shorl sample trace of z+ y?- x? Mz+x². The curve is shown graphically in Fig. 3.2. The iniial poin1 estimate is (0.2,0.2, -0.1). Thc step length is determined adaptivcly as described. In addition 1o point coordinates, both thc ncxt step length and the number of Newton iterations needed to determine the point to within 10- 10 are shown. For simplicity, only 5 decimals are given. At this singularity the orientation reverses and is reflected in the changc of sign of ihe slep length. Since the third derivative r" is not necessarily perpendicular to r', the point distance docs not always correspond to the step length.

## 4. Singular curve points

Consider now the intersection curve when the surfaces are given implicitly by /1 and f2. At a singularity p, the Taylor expansion of r does not exist in the ordinary sense. Nevertheless, System (5) remains formally valid and can be used to determine approximants to r al p. This fact is less attractive than one might suspect at firsl, since the equalions no longer are linear and, thus, become more difficult lo solve. A point p is singular on r(s) for one of ihe following reasons:

- (1) The gradients Vf, and Vf, are linearly dependent.
- (2) One of the gradients, say V/, is zero, but the other is not.

## 4.1. Tangents at singular points

We consider Case 1, i.e., linearly dependent surface gradients. From Section 3.6 it follows that lhis case is in substance the same as Case 2, and we demonstrate how the familiar tangent cone construction corresponds to an elementary simplification of 1he Equation System (5).

When the gradicnts are linearly dependent, the tangent planes of §, and f are the same. Wc assume without loss of generality that the point p is the origin and Vf = (0, 0, 1). Therefore, we maywrite

<!-- formula-not-decoded -->

where the polynomials / and f consisis of terms of degree 2 or higher.

It is possible that F is divisible by z. In that case the computation must be iterated; i.e., we must derermine a f4 by subtracting from f3 a multiplc of z*§, where k is suitably chosen. Mora in [Mora '82] proves tha1 this computation lerminatcs.

Now the intersection of f and f is also the interscction of f, and § =f-μf1. We determine the curve tangent(s) from f1 and f. The terms of lowcst order in f comprise a homogeneous form F that approximates the surface /3 = 0 in the neighborhood of the origin und has degrec 2 or higher. F(x, y, z) =0 is a cone with the origin al its vertex. It intersects the plane langent to f = 0 only at the origin or in a set of lines through the origin that are tangent to the branches of s, the intersection of /, and f2.

We determine the tangents to the intersection at the origin by substituting z = O in F. This yields the homogeneous polynomial F(x, y, 0) in two variables. The roots of F(x, y, 0) are (0, 0) and (入u, Av) where not both  and v are zero and 入 ≠ 0. The rool (0, 0) is an improper solution for G and is excluded. If there are no other real roots, then the cone inlersecis the plane z = O only in the origin, a case that docs not arise when tracing a curve branch.

We demonstrate by example that this tangent compulalion is equivalently done by elementary manipulalion of the equations of System (5). The deeper rcason for this is further clarified below and rests on the correspondence of the Taylor series al regular curvc poinis with formal power series expansion of r al singularities.

For every other real root (入u, Au) we obtain a corresponding tangent vector ' = (Au, Au, 0) 10 r at the origin. Here 入 is choscn such that the vector has length 1.

Example. Consider the iniersection of the two cylinders f, = x² + z2 + 2z = 0 and § = y² + 2 + 4z = 0 which is irreducible and has a nodal singularity at the origin, as shown in Fig. 3.1. The curve is equivalently the interseclion of f with the elliptic cone / =§ - 2/ = y2 - 2x2 2. For this cone F = J. Therefore the tangents at the origin are given by the roois of y2 - 2x, i.c., they are the lines (入, V2 入,0) and (-入, V2入, 0).

Next, when we formulate the equations of System (5) for f and §2 and writc (x(s), y(s), z(s)) for r(s), we obtain a1 the origin

<!-- formula-not-decoded -->

By subtracting the third cquation [wice from the fourth and dividing by two, we obtain the cquation

<!-- formula-not-decoded -->

Note the similarity between this equalion and the tangcnt cone of f. Thus, solving System (5) for r' is equivalent to determining the langent directions from J1 and f3.

4.2.Aigebraic correspondence

<!-- formula-not-decoded -->

where (a, b, c;) is a vector, e.g., [Walker '50, Ch. IV.2, V.5]. The formal derivative of  by s is delined as

<!-- formula-not-decoded -->

The power series must satisfy identically ∫(r(s)) = 0. Substituting the series of r(s) into f, and collecting terms, we obtain a formal series

<!-- formula-not-decoded -->

This leads to a system of equations

<!-- formula-not-decoded -->

where K, is the coefficient of s" in the rcsulting series. A similar system of equations is obtained for f(r(s) = 0. Because the formal derivative above has all the familiar properties of derivatives, thcse equations are formally the same as System (5).

We consider the intersection of the surfaces f = z + y2 - x² and f2 = z + x2 with a nodal singularity at the origin, as shown in Fig. 3.2. We set

Because of this algebraic correspondence, it is possible lo approximale the curve al a singular point by formulating the system of equations as before and solving it for the unknown coefficients. In contrast to the nonsingular case, however, the system no longcr is linear and thus is more difficult to solve. Wc explain the proccdure by an example:

<!-- formula-not-decoded -->

where (α, b1, c) =r'(0), (a2, b2, C2) = r"(0)/2, and so on. The equations of System (13), or cquivalently, of System (5), are thus

<!-- formula-not-decoded -->

As before, the system is underconstrained. It is possible lo choose the independent quantities such that one of lhe series is ±s* [Walker '50, p. 109]. Hcre s need not correspond to arc length. We choose C2= -1 and c =ca= ... =0. Then the following two solutions are obrained:

and

<!-- formula-not-decoded -->

Tlhe scries correspond to the local paramelerizalions of the Iwo intersecting branches. For remarks about their convcrgence see, e.g., [van der Waerden '38, p. 52].

Because the equations are nonlinear, this approach is difficult to implement. The dcgree of the equations depends on the order of the singularity. In the simples1 cases this is Iwo. However, higher order singularities can occur that may make it difficult to solve the equations and to identify subsequently a solution that paramelerizes the Lraversed path.

## 5. Plane curves

We now consider tracing a scgment of the plane algebraic curve f(x, y) = 0, beginning a1 an initial point (xo, yo) at which tracing commences in a specified direction. For simplicity, we assume that thc initial point is not singular. With this assumption, the trace direction is simply specified as posirive, following the tangent vector ( -fy, f), or negative, tracing in the opposite direction. If the initial point is singular, a more complicated specification proccdure is required tha1 identifies the intended branch and a direction on it. Such specificalions can be worked ou1 without difficulty, based on the desingularization techniques described below. See also [Hoffmann el al. '87] for a discussion of this problem in the context of solid modeling.

## 5.1. Desingularization

Desingularization of plane curves is based on the following classical thcorem, proved by Riemann and Cayley:

Theoren.Every plane curue can be birationally rransformed into a curve devoid of singularities.

Among the different proofs of the theorem are constructive versions that derive the needed birational transformation from a sequence of simple quadratic transformations, e.g., [Abhyan-

<!-- formula-not-decoded -->

The inverse transformations are, respectively, x=x, y= x,y, and x= x2y, y = y2. The basic properties of transformation T, can be summarized as follows:

Fig. 5.1. Desingularization of a nodal singularily. (a) Curve y² - x² - x =0.(b)Curve alter applying T: y² -1- x1

<!-- image -->

- (1) All poinls (x, y) with x ≠ 0 are mapped 1-1 to the x-)i plane.
- (2) All points (0, y) are mapped to infinity.
- (3) As we approach the origin on a branch, the limit of the image points is the image of the origin on the branch. This limit depends on the direction of approach, hence the pencil of dircclions through the origin, except 1he y-axis, are mapped to finite points on the y-axis.

In particular, T, maps irreduciblc curves lo irreducible curves. The line x, = 0 is called the exceptiona/ line of T,. The propcries of T, are analogous. The exccplional line of T, is y2 = 0. In intuitive terms, ihe transformation separatc curves branches tha1 inlersecl with differenl tangen1 directions. This is plausible since the line y - mx = 0 through the origin is mapped 10 the line y -m = 0 thal interccpts the y axis a1 distance m from the origin. Moreover, branches that arc in higher ordcr coniacl, such as tacnodes, are mapped to singularities in the X1-y1 plane at which ihe contact order is reduced. Finally, the order of a nonlinear branch through the origin is also reduced. The latter two facis are nol casily seen, as they depend on slructural properlies not readily apparent from the graph of f and the elementary concepts such as tangcnt direction, curvalure, etc. Neveriheless, given a suitable measurc for the complexity of a singular point, i1 can be shown that every application of T, or T simplifies 1he struclure, each of whose leavcs corresponds to a nonsingular curve branch. For cxample, [Abhyankar '83] defines such a measure based on the slructure of the order form, [Walker '50] uses a measure relaled to the curve genus, whereas [van der Wacrden '38] uses the intersection multiplicity of the branch with ihe polar form as a measure of complexity.

Fig. 5.2. Desingularization of a cuspidal singularity. (a) Curve y² - x' = 0. (b) Curve after upplying T: y² - x) = 0.

<!-- image -->

Fig. 5.7. Topology of y2 - xb - y6 = 0 at the singularily. (a) Curve. (b) Schenalic of its crossing branches.

<!-- image -->

points (a, b), i.e., points such that f(a, b) &lt; 0, the branch orientation reverses precisely when this branch intersects an even number of other branches. Two examples, Figs. 5.6 and 5.7, show the curve in the neighborhood of the singularity as well as a schematic diagram of the topological structure of the singularity.

We now quantify the correspondence between the orientalion of / and its proper transform g and derivc a simple method for detecting orientation reversal without having to analyze the topological struclure of the singularity in detail. Lel p = (au, bo) bc a nonsingular point of f.

wherc a≠0. Lct

<!-- formula-not-decoded -->

be the place of / centered at p. The place defines a branch orientation of increasing s that need nol agree with the slandard orientation (-f,∫x). Centered at the corresponding point P =(uo, bo/ao), the transformed curve g has the place

<!-- formula-not-decoded -->

Since x(s)=x,(s), the curve and i1s Iransform are orienled the same way. Moreovcr, since (s) = y(s)/x(s), we divide 1he two power series 1o obtain

<!-- formula-not-decoded -->

and so on. Now p and p are not singular. Consequently, the Taylor series exisis, a is proportional to -f, and -gy, b, is proportional to f, and c, is proportional to gx. Thus, the sign of the proportionality factor α relates the orientation of the Taylor series with the standard orientation. Therefore, given the direction of tracing f, wc obtain the corresponding tracing direction of g from

<!-- formula-not-decoded -->

Conversely, given the tracing direction of g, we obtain the corresponding tracing direction of f in the same way.

In consequence, the following procedure is uscd to maintain a consistent tracing direction througli singularities:

<!-- image -->

<!-- image -->

<!-- image -->

<!-- image -->

Fig.5.12. 2x-3x²y+y²-2y²+y0.

<!-- image -->

Fig. 5.14.(x² +y²)²+ 3x²y- y²= 0.

Fig. 5.13. x+ x²y²-2.x²y-xy² + y²=0.

<!-- image -->

Fig. 5.15. (x² + y²) - 4.x²)² = 0.

<!-- image -->

Step 1. We traverse f in the direction u( -fy, J), where u = I or u = -1.

Step 3. If α&gt; 0, the transform g is traversed in the direction u(-gy, Sx); otherwise, it is traversed in the opposite direction.

a    r    n rd s e e   d be a point on f travcrsed before the singularity, and let p1 be the corresponding point on g. The partials of ∫ and g are evaluated at these points, and the factor α of proporlionality determined as described above.

The same traversal correlation is established when leaving the vicinity of the singularity, re-establishing the proper fraversal dircction on f from the traversal direclion on g.

## 5.5.Implementation

We have implermenled the algorithm on a Symbolics 3650 Lisp machine and traced the curves shown in Figs. 5.8 through 5.15. In our experience with the program, it is possible to Irace through complex singularilies. A problem for llie presen1 implementation is locating the singularity accurately. For example, locating the cuspidal singularity of the family of curvcs small values of e the curve has vcry high curvature in the vicinity of the origin and appears to be singular.

## Acknowledgements

We thank Professor Abhyankar for various insightful discussions in algebraic geometry.

## Refcrences

Abhyankar. S. (1983). Dcsingularization of planc curves., Proc. Symp. Pure Math. Amer. Math. Soc., Providence, RI 40 (1). 1-45.

- Abhyankar, S. and Bajaj、C. (1987a), Automatic Tational paramielcrization of curves and surfaccs 1; Conics and conicoids, Computer Aided Design 19 (1), 11-14.
- Abhyankar, S. and Bajaj, C. (1987c), Aulomatic rational paranicterization of curves and surfaccs Ill: Algebraic planc curves. Computcr Aided Gcometric Design 5, 309-321.
- Abhyankar, S. and Bajaj, C. (1987b), Automatic rational paramelerization of curves and surfaces Il: Cubics and cubicoids, Computer Aided Design 19 (9), 499-502.
- Abhyankar. S. and Bajaj. C. (1987d). Aulomalic rational parameterization of curves and surfaces IV: Algcbraic space curvcs,Tech.Rept.703,Comp.Science,Purdue Universily.
- Boehm, W., Farin, G. and Kahmann, J. (1984). A survey of curve and surface ncthods in CAGD. Computer Aidcd Geometric Design 1, 1-60.
- Arnon, D.S. and McCallum, S. (1983), A polynomial-time algorithm for the topological type of a real algebraic curve. Tech.Rep1. 454. Comp. Scicnce Depl., Purdue Universily. See also J. Symh, Comput. 3 (2). (1988).
- Cohcn,E, Lyche,T.and Riesenfeld. R.(19s0), Discrcle B-splines and subdivision lcchniqucs in computer aided gconiciric design and computer graphics, Computer Graphics and Image Processing 14, 87-111.
- Dongarra, J., Moler. C., Bunch, J. and Stewar1, G. (1979).Linpack User's Guide,SIAM.Philadelphia, PA.
- Collins, G.(1983). Quantifier elimination for real closed fields: A guide to the literature. B. Buchbcrger, G. Collins and R.Loos, eds.. Conpufer Aigebrd, Symbolic und Aigebraic Computarion. 2nd cdition, Springer, Berlin 79-82.
- Farouki, R., (1986). Trimumed surface algorithms for the evaluation and interrogalion of solid boundary rcprcsentations,IBM J. Res.Develop 31.
- Franklin,P..(1944),Methnds of Adcanced Caiches,McGraw-Hill, NewYork.
- A. Geisow (1983). Surface Interrogurious, Ph.D. Dissertalion, University of Eas1 Anglia, School of Computing Studies and Accounlancy.
- Hoffmann, C.. (198R), Algebraic curves, in: J. Rice, cd., Marhenatical Aspecis of Sciemifie Sofnware, IMA Volumes in Math. and Appl. 14 (Springer, Berlin, 1988) 101-122.
- Golub, G., and Van Loan,C..(1985). Matrix Cowmpurarions, John Hopkins University Press, Ballimore.
- Hofmann, C., and Hopcroft, J., (1987), Geometric ambiguities in boundary representations. Computer Aided Design 19 (3),. 141-147.
- Levin, J,(1979), Mathematical models for delernining the interseclions of quadric surfaccs, Computer Graphics and Inage Proccssing 11, 73-87.
- Mora, F. (1982), An Algorithm lo compuic 1he equations of tangent cones, EOROCAM '82, Leclure Nolcs in Computer Science 144, Springer. Berlin, 158-165.
- Moniaudouin, Y., Tiler, W, and Vold, H., (1986), Applications of power series in computational gcomclry, Computer Aided Design 18 (10). 514-524.
- Ocken, S., Schwariz, J. and Sharir, M. (1983), Precise inplcmcniation oI CAD primitives using ralional paramelerizations of standard surfaces,Plamming. Geometry' and Comptexity of Robor Motion,Ablex Publishing. Norwood. NJ, 245-266.
- Owen, J. and Rockwood, A. (1987), lntersection of general implicit surfaccs, in: G. Farin, ed.. Geonietric Modeling: Aigorirhns and Trends,SIAM,Philadelphia,PA,
- Mathenatics of Surfaces,Oxford University Press,117-142.
- Requicha, A., and Voelcker, H., (1983), Solid modeling: Current siatus and rcscarch direclions, IEEE Comp. Graphics Appl. 3, 25-37.
- Pratt, M., and Geisow, A., (1986), Surface/Surfacc intersection problems, in: J. Gregory. cd., The Marhemarics of Surfaces. Oxford University Press, 117--142.
- Siewarl, G.,(1973).Iutroduction to Marrix Conputations, Academic Press, New York.
- van der Wacrden,B,(1938), Einfinug in die algebraische Gcometrie,Springer,Berlin.2nd edition,1973.
- Purdue Universily.December 1987.
- Weslcy,M., Lozano-Perez, T.,Liberman, L.Lavin, M. and Grossman, D. (1980). A geomeiric modeling system for automatcd niechanical assembly,IBM J. Res.Develop.24.64-74.
- Walker, R., (1950), Aigebraic Curves. Springer, New York, 1978.

## Appendix: Computational delails

We describe in more detail the derivation of the quantities bi.s, i = 1, 2, B.m, J = 1, 2, 3 and the usc of the singular value decomposition to solve the linear syslem of Section 3.

C

## A.1. Derivation of the bi,n, and Bi,m

The expressions for b;,s arc developed from the Taylor cxpansion of ∫, and f2 of Section 3.2. For /1 wc obtain

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

and so on. Substiruting into the Taylor's series for ∫, and equaling to zero the coefficients of s", m = 1, 2, 3, we get the equations

<!-- formula-not-decoded -->

They are the equations

<!-- formula-not-decoded -->

of Section 3.1. The explicit form above is used for computing b1.2 and b,. in the program. A similar set of formulae is obtained for computing b2.2 and b2,g when f, is replaced with ∫2-

The expressions for B.m are developed, in an analogous fashion, from the Taylor cxpansion of F, F, and F, of Section 3.3.

## A.2. Singular value decomposition

Both Newlon's method for refining a point estimate and the delermination of the curve approximant entail solving a linear system

<!-- formula-not-decoded -->

For the implicit case A is a 3-by-2 matrix whose columns are the gradicnts of / and f2, and where w and z are column vectors of length 3 and 2, respectivcly. For the parametric casc A is a 4-by-3 mairix whose columns are the gradients of Ff, F, and F, and where w and z are column vectors of length 4 and 3, respectively.

When the pair of gradients is linearly independen1, then the gcneral solution of this system was written in Section3.2as

<!-- formula-not-decoded -->

and in Section 3.3 as

<!-- formula-not-decoded -->

This is no1 the general solution at a singularity where lhe pair of gradients is lincarly dependent.

To treat all cases in a uniform way with a compulationally stable process, we compute the singular value decomposition of A [Golub e1 al. '85, Stewart *73]. (Wc linked the thoroughly tested routines of Linpack [Dongarra el al. '79] to our program.) Thus, we factor A as A = UEV'r, where U∈ R3x and V∈R?x2 for 1he implicit/implicitl case are orthogonal and V∈ R3x3 are orthogonal matrices and Z∈Rx3 is diagonal. Thc system A'w=z now becomes

<!-- formula-not-decoded -->

and we write its solution as

<!-- formula-not-decoded -->

where U, denotes 1he j-th column of U. Since the gradients Vf, and Vf and the veclor r arc not generally orthonormal, and since the U, are, the quantities α', β', and y' diffcr from their counterpart in Section 3.1.

There are three cases:

(i) If the pair of gradients is linearly independent, then Z. &gt; 0, Z2.2 &gt; 0, and the first two columns of U span ihe same space as the pair of gradients. In that case,

<!-- formula-not-decoded -->

(i) If the pair of gradients is linearly dependent and at leasl one is nonzero, then Z, &gt; 0, Z22 = 0, and the first column of U spans the same space as the pair of gradients. If Vz ≠ 0, then there is no solution;otherwise

and y' is arbitrary.

<!-- formula-not-decoded -->

and β' and y' are arbitrary.

This is now used as follows.

(ii) If both gradients are zero, then so is ∑. If z ≠ 0, then there is no solution; otherwise α', β', and r' are arbitrary.

Nevton's merhod. We always choose Y' = 0. In Case (i), β' is also set to zcro. In Case (i), the initial guess is perturbed and the iteration restarted. Usually two or three itcrations suffice. If the singular value decomposition is not recomputed al each iteration, the number of iterations typically doubles.

Finding rhe Approxinanr. The solutions to the linear systems are determined using thc Frenel-Serrel formulae [Franklin '44, p. 107]:

<!-- formula-not-decoded -->

where s is arc length, r is the unit tangent, n is the principle normal, b is the binormal, x = l/o is curvature, and T= 1/r is torsion. The vectors r, n, and b form an orthonormal triad with

<!-- formula-not-decoded -->

At a point r(s) on the curve, we have

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

In Case (i) we obtain r' = yiU, using Yi = ±1. For the first point on the curve, the sign of Y' is an input parameter; for other poinis, the sign of Y is chosen to be the sign of r'(0)TU, at the previous point r'(O). To get r"(s), we use Y? = 0, so that r' and r" are orthogonal. The length of r"(s) gives the curvature x. To get r "(s), we choose Y' = -*.

In Case (ii), we return to the preceding point and double the computed step lengih.

In Case (i), we project r'(O) into the plane spanned by U, U, and then normalizc the projection to get r'(s); an input vector is given if k = 0. For r", we choose β and y to make I" and r' orthogonal; r'" is chosen as above.

1