<!-- image -->

## Computing minimum distance between two implicit algebraic surfaces

Xiao-Diao Chen, Jun-Hai Yong, Guo-Qin Zheng, Jean-Claude Paul, Jia-Guang Sun

## To cite this version:

Xiao-Diao Chen, Jun-Hai Yong, Guo-Qin Zheng, Jean-Claude Paul, Jia-Guang Sun. Computing minimum distance between two implicit algebraic surfaces. Computer-Aided Design, 2006, 38 (10), pp.1053-1061. ⟨10.1016/j.cad.2006.04.012⟩. ⟨inria-00518397⟩

## HAL Id: inria-00518397

## [https://inria.hal.science/inria-00518397v1](https://inria.hal.science/inria-00518397v1)

Submitted on 17 Sep 2010

HAL is a multi-disciplinary open access archive for the deposit and dissemination of scientific research documents, whether they are published or not. The documents may come from teaching and research institutions in France or abroad, or from public or private research centers.

L'archive ouverte pluridisciplinaire HAL , est destinée au dépôt et à la diffusion de documents scientifiques de niveau recherche, publiés ou non, émanant des établissements d'enseignement et de recherche français ou étrangers, des laboratoires publics ou privés.

<!-- image -->

## Computing minimum distance between two implicit algebraic surfaces

Xiao-Diao Chena.b,*, Jun-Hai Yonga, Guo-Qin Zhenga, Jean-Claude Paula.c.1, Jia-Guang Suna

acofivitj84 j CNRS,France

Received7October2005;accepted29April2006

## Abstract

Theminimum distancecomputationproblembetween twosurfacesisveryimportant inmanyapplicationssuchasrobotics,CAD/CAMand distanceandapairofpointswheretheminimumdistance occurs.Thenewmethodalsoworkswherethereareanimplicitalgebraicsurfaceand a parametric surface.Quadric surfaces,tori and canal surfaces are used to demonstrate ournewmethod.When the two surfaces are a general quadricsurfaceandasurfacewhichisacylinder,acone oranellipticparaboloid,thenewmethodcanproducetwobivariateequationswherethe degreesarelowerthanthose ofanyexistingmethod.

Keywords:Minimum distance;Offset;Canal surface;Implicit algebraic surface;Parametric surface

## 1.Introduction

Computationoftheminimumdistancebetweentwo objects is very importantinmany applications such as robotics[15,16,20,25,34,42],CAD/CAM[9,17,32,33,45],and computergraphics[27,39].In robot motionplanning, distance informationisneededforcomputinginteractionforceand penalty function[2o].Distance information is usefulfor interferenceavoidanceinCAD/CAMandNCverification[17]. Itis alsoveryimportantfor collisiondetection anddynamic simulation in computer graphics[15,16,27].Many papers[12, 13.16.24.36]aredevotedtothecomputationoftheminimum distancebetween twopolyhedra.When theobjectsconcerned arecurvedsurfaces,theyareconvertedintopolyhedrato compute the minimum distance.However,the number of polyhedral facesapproximating thegiven curvedsurfacesis usuallyverylargewhenhighprecision is desired.The space costwouldbecomeaconsiderableproblem,andthemethods basedonpolyhedramayfailfortherealtimerequirement.

*Corresponding author at:Tsinghua University, School of Software, TsinghuaStreet,100084Beijing,China.Tel.:+8601062795442.

Jean-Claude Paul is a senior researcher at CNRS and INRIA France).He is currentlya visitingprofessorat TsinghuaUniversity.

[E-mail address:chenxdoo@mails.tsinghua.edu.cn(X.-D.Chen).](mailto:chenxd00@mails.tsinghua.edu.cn)

Recently,somepapers[11,17,22,39,40,4]have considered methodsthatworkdirectlywithcurvedsurfaces.Whenthe twosurfacesintersect,theminimumdistancebetweenthemis zero.There aremany algorithms[2,10,14,18,19,21,23,28-30, 35,41,44]designedtotestwhethertwogivensurfacesintersect. Therearethreedominanttechniquesusedintheintersection test:thesubdivisionmethod,thelatticemethod,andtheanalytic method.For some simple cases,the analyticmethods may yieldgood results[11,45].In[11],Fu gives a necessary and sufficientconditiontojudgewhetherabicubicBezierpatch andaplaneintersectornot.In[45],WangandKimshow thattwoellipsoidsareseparatedifandonlyiftheirdegree 4characteristicpolynomialhastwodistinctpositiveroots.In practice,thesetechniquesareoftencombined.Moreover,other techniques are alsousedin theintersection test.For instance, Tanakaetal.[40]useMonteCarlosimulationbasedona stochasticdifferentialequationtosampleimplicitsurfacesfor intersection test.Some otherpapers oncollisiondetectioncan befoundin[5,26].

Thecomputationoftheminimum distancebetween two curvedsurfacesusuallyinvolvesa systemofnon-linear equations.There aremanyreferencesfor solving non-linear equation systems.Some ofthe common techniques used to findlocal solutions arethe conjugate direction method,the steepestdescentmethod,theNewton-Raphsonmethod,and the gradient flow method [4,8,38,43]. To compute all roots in a given area of interest, several global solution methods such as algebraic and hybrid methods, homotopy methods and subdivision methods [31] are useful. The resultant based methods [1,3,6,7,46,47] seem to be very promising in providing solutions for low degree algebraic equations.

Fig. 1. Offset surfaces and touching point (a) growing or (b) shrinking.

<!-- image -->

In this paper, we only consider the case where the two surfaces do not intersect. There are a number of efficient ways for computing the minimum distance between two surfaces such as planes, quadrics, tori, or canal surfaces [17,22,39]. Lennerz and Sch¨ omer [22] discuss the minimum distance computation problem between two quadric surfaces. They use the Lagrange technique to obtain two bivariate polynomial equations in Lagrange multipliers. When the two surfaces are a cone and a general quadric surface, [22] obtains two bivariate polynomial equations both of degree 4 or a univariate equation of degree 12. Sohn et al. [39] use line geometry to convert the minimum distance problem into an intersection problem between the normal congruences of the two surfaces. Each point on a surface is associated with the line passing through the point and spanned by the normal of the surface at that point. The normal congruence of a surface is defined as a set of lines associated with the points on the surface. They describe one of the two normal congruences in parametric form and the other normal congruence in implicit form, and obtain two bivariate equations. Usually one of the two surfaces in [39] is in implicit form and the other is in parametric form. To describe a normal congruence for a general implicit algebraic surface, one needs to eliminate seven unknown variables from 10 polynomial equations (or eliminate four variables from seven polynomial equations by simplification), which is a non-trivial task. When the two surfaces are a cone and a general quadric surface, [39] produces two bivariate polynomial equations of degrees 4 and 8. In [17], Kim computes the minimum distance between a canal surface and a simple surface such as a plane, a sphere, a cylinder, a cone, or a torus. The fact that the normals at the closest points between the two surfaces are collinear is exploited, and characteristic circles are introduced to convert the minimum distance problem into a problem of finding the roots of a univariate function.

The offset technique employed in this paper generalizes some of the methods mentioned above. Given two surfaces, when we offset one of the closest points in one of the two surfaces along or against its normal direction using the minimum distance, we get a point on the other surface, and the distance between these two points is exactly the minimal distance. The method presented in this paper is based on this simple but important fact. Suppose two surfaces S 1 and S 2 do not intersect. We build the dynamic offset surface of one of the two surfaces, for example S 1. As Fig. 1 shows, if S 1 contains S 2, the dynamic offset surface of S 1 is shrinking, i.e., offsetting inward, towards S 2; otherwise, the dynamic offset surface of S 1 is growing, i.e., offsetting outward, towards S 2. Eventually, a dynamic offset surface O 1 will 'touch' S 2. The touching point P could be one of the two points where the minimum distance occurs. Together with the fact that the normals of the two given surfaces at the two closest points where the minimum distance occurs are collinear, we obtain the equations for calculating one of the two closest points. From these equations we may get several candidate points, e.g., in Fig. 1(b), both P and Q are candidate points which satisfy the equations. To determine the point we want, we define the corresponding distance of a candidate point. If the normal line at a candidate point intersects with S 1, the corresponding distance of the candidate point is defined as the distance from the point to the closest intersection point of the normal line and S 1; if not, it is defined as a positive infinite real number. The point we want is the candidate point whose corresponding distance is minimal. Once this point is identified, the other point in S 2 can be calculated along or against the unit normal vector of the first point with the minimum distance.

The remaining part of this paper is organized as follows: an algorithm based on the offset method to compute the minimum distance between two surfaces is given in Section 2; in Section 3, we illustrate our method for two implicit algebraic surfaces, using quadric surfaces and tori; Section 4 illustrates the new method for an implicit algebraic surface and a canal surface; whereas Section 5 compares our method with some known existing methods. Some concluding remarks are given in the last section.

## 2. Minimum distance between two surfaces

Given two surfaces S 1 and S 2, the minimum distance computation problem is to find a non-negative real number d min such that

<!-- formula-not-decoded -->

where ‖ p -q ‖ is the Euclidean distance between the points p and q . d min is the minimum distance between the surfaces S 1 and S 2. Any pair of points ( p , q ) where the minimum distance occurs is called a 'location of the minimum distance'. If S 1 and S 2 intersect, d min is equal to zero, and any intersection point is a location. Here, we assume that S 1 and S 2 do not intersect.

In this paper, we assume that at least one of the two surfaces is an implicit algebraic surface. In this case, without loss of generality, let S 2 be an implicit algebraic surface. The other surface S 1 can be either an implicit algebraic surface or a parametric surface. Suppose that S 2 is given by

<!-- formula-not-decoded -->

where q is a point in R 3 . A point q is on the surface S 2 if and only if f 2 ( q ) = 0. We write the coordinates of q as a column vector ( x , y , z ) T . The gradient at point q ∈ S 2 is

<!-- formula-not-decoded -->

where f 2 x ( q ), f 2 y ( q ), f 2 z ( q ) are the first partial derivatives of f 2 ( x , y , z ) with respect to x , y and z .

For any real number d , we define the dynamic d -offset surface of the surface S 1 to be

<!-- formula-not-decoded -->

which depends completely on the surface S 1. In fact, the new method only uses the information of S 1 and d , and is not affected by the singularity of the d -offset surface S d 1 .

Suppose that the minimum distance between S 1 and S 2 is a positive real number d min, and point p is one point of a location of d min on the surface S 1. Let N p be the gradient of S 1 at p . If p is regular, i.e., N p is not degenerated into a zero vector, there exists d = d min or d = -d min, such that a dynamic d -offset surface S d 1 is tangent to the surface S 2 at point

<!-- formula-not-decoded -->

where µ = d ‖ N p ‖ . The relationship between the minimum distance and the dynamic offset surface is given by

<!-- formula-not-decoded -->

Since q is also a point on the surface S 2, we have

<!-- formula-not-decoded -->

Let N q be the gradient of S 2 at the point q . Then, N q and N p are two collinear vectors. From (1), N q is ( f 2 x ( q ), f 2 y ( q ), f 2 z ( q )) T . Therefore, we obtain

<!-- formula-not-decoded -->

where λ ∈ R . Substituting Eq. (2) into Eq. (4), we have

<!-- formula-not-decoded -->

The resultant method [1,3,6,7,46,47] is useful for eliminating one variable from two polynomial equations. Given two polynomial equations

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

Fa and Fb have common roots if and only if

<!-- formula-not-decoded -->

R b a is called the resultant of Fa and Fb with variable µ . Since f 2 ( x , y , z ) is an implicit algebraic function about x , y and z , the equations in equation systems (3) and (5) are all polynomial equations in λ and µ . So we are able to eliminate λ and µ from equation systems (3) and (5) with the resultant method, and thus obtain

<!-- formula-not-decoded -->

Both of the above equations have three variables which are the coordinates of the point p on the surface S 1. Therefore, one more equation is required to determine the point p .

If the first surface S 1 is an implicit surface given by f 1 ( x , y , z ) = 0, then we have the equation system

<!-- formula-not-decoded -->

If the first surface S 1 is a parametric surface given by

<!-- formula-not-decoded -->

where u , v are parameters, then we have the equation system

<!-- formula-not-decoded -->

In either case, from the equation system (6) or (7), by using the methods in [3,31], we can find a solution for the coordinates of the point p . Substituting the coordinates of p into Eq. (3), we obtain the value of d . The other point q of the location is obtained from Eq. (2).

If p is singular, i.e., N p is degenerated into a zero vector, p can be found by solving

<!-- formula-not-decoded -->

For each singular point p 1 ( x 1 , y 1 , z 1 ) T ∈ S 1, the closest point q ( x , y , z ) on S 2 can be obtained by solving

<!-- formula-not-decoded -->

The corresponding distance d is just ‖ p 1 -q ‖ . Thus, whether the closest point p ∈ S 1 is singular or not, from both the equation systems (6) or (7), (2) and (3) and the equation systems (8) and (9), we are able to find the minimum distance as well as the corresponding closest points p ∈ S 1 and q ∈ S 2.

The algorithm is as follows.

Input: Two surfaces S 1 and S 2. S 1 is either an implicit surface or a parametric surface. S 2 is an implicit algebraic surface.

Output: The minimum distance d min between S 1 and S 2, and a pair of points ( p , q ) with the properties ‖ p -q ‖ = d min, p ∈ S 1 and q ∈ S 2.

- (1) If S 1 and S 2 have any intersection point p ;

Output d min = 0 and the pair of points ( p , p ) , then go to Step 5;

- (2) Solve the equation systems (6) or (7); Substitute each solution p ∈ S 1 into Eq. (3) to obtain values of d ; Compute the corresponding point q ∈ S 2 with Eq. (2);

Add all the solutions ( p , q , d ) into the set E d ;

- (3) Solve the equation systems (8) to obtain the singular points on S 1; For each singular point p ∈ S 1, compute the closest point q ∈ S 2 with equation system (9), and calculate d = ‖ p -q ‖ ;

Add all the solutions ( p , q , d ) into the set E d ;

- (4) Find d min such that

<!-- formula-not-decoded -->

Output d min and ( p , q ) ;

- (5) End of the algorithm.

## 3. Minimum distance between a quadric surface and an implicit algebraic surface

Quadric surfaces and tori are widely used [22,39,45]. In this section, the first surface S 1 is an implicit algebraic surface defined by f 1 ( x , y , z ) = 0, and the second surface S 2 is a quadric surface such as a cylinder, cone, elliptic paraboloid, ellipsoid or torus. The singular points of S 1 can be found by solving ( f 1 x , f 1 y , f 1 z ) T = 0 , and it is easy to compute the distance between a point and S 2. Here we omit the discussion on the singular points of S 1.

For the convenience of degree analysis, we provide the following theorem.

Theorem 1. Suppose that an equation system has three polynomial equations of degree 2, m and r. After eliminating one variable, one can obtain two bivariate equations of degree min { 2 m , 2 r } and m + r -1 .

Proof. Without loss of generality, assume m ≤ r , and the first equation of degree 2 is of the form (if necessary, with coordinate transformation)

<!-- formula-not-decoded -->

where Fi ( y , z ) is a polynomial of degree i in y and z , i = 1 , 2. Substituting Eq. (10) into the other two equations of the equation system, we get two new equations

<!-- formula-not-decoded -->

where Gi ( y , z ) and ¯ Gj ( y , z ) are polynomial of degree i and j in y and z , i = m -1 , m , and j = r -1 , r .

Combining Eq. (10) with the first equation in the equation system, we have

<!-- formula-not-decoded -->

From Eq. (11), we have

<!-- formula-not-decoded -->

Eqs. (12) and (13) are of degree 2 m and m + r -1. Thus, we complete the proof. □

When the three polynomial equations in an equation system are of degrees 2, 2 and r , respectively, where r ≥ 2 is a positive integer, we can obtain two bivariate polynomial equations of degrees 4 and r + 1 by Theorem 1, or a univariate polynomial equation of degree 4 r with Levin's method [23,43,48].

That is, in general detail: given two distinct quadrics A : X T AX = 0 and B : X T BX = 0, where A , B are two distinct 4 × 4 matrices, and X = ( x , y , z , 1 ) T , the pencil generated by A and B is the set R (λ) with equations X T ( A + λ B ) X = 0, λ ∈ R . Levin [23] proves that there exists a real ruled quadric of the

form

<!-- formula-not-decoded -->

in the pencil of two distinct quadrics, where the curve b ( u ) and ruling direction d ( u ) are at most quadratic rational functions in u [43,48]. Any intersection curve of the two quadrics ( QSIC ) can be parameterized in the form

<!-- formula-not-decoded -->

where c 2 ( u ) = d ( u ) T Ad ( u ), c 1 ( u ) = b ( u ) T Ad ( u ), c 0 ( u ) = b ( u ) T Ab ( u ) , a ( u ) = c 2 ( u ) b ( u ) -c 1 ( u ) d ( u ) , s ( u ) = c 2 1 ( u ) -c 2 ( u ) c 0 ( u ) (Refer to [23,43,48] for more details). Substituting the intersection curve p ( u ) into the third equation of the equation system, and removing the radical expression, we obtain a univariate polynomial equation of degree 4 r .

3.1. Minimum distance between a cylinder and an implicit algebraic surface

Suppose the first surface S 2 is a cylinder given by

<!-- formula-not-decoded -->

The equation system (6) becomes

<!-- formula-not-decoded -->

When S 1 is a quadric surface, the equations in the equation system (14) are of degree 1, 2 and 2, and we can obtain two bivariate polynomial equations of degree 2, or a univariate polynomial of degree 4.

## 3.2. Minimum distance between a cone and an implicit algebraic surface

Suppose the first surface S 2 is a cone given by

<!-- formula-not-decoded -->

where m is a positive real number.

The equation system (6) becomes

<!-- formula-not-decoded -->

where λ = 0 means that the candidate closest points on the cone are singular. So the equation system (15) is equivalent to two new equation systems

<!-- formula-not-decoded -->

and

<!-- formula-not-decoded -->

When S 1 is a quadric surface, the equations in the equation system (16) and (17) are all of degree 2. From each new equation system, we can obtain two bivariate polynomial equations of degree 3 and 4, or a univariate polynomial of degree 8.

## 3.3. Minimum distance between an elliptic paraboloid and an implicit algebraic surface

Suppose the first surface S 2 is an elliptic paraboloid given by

<!-- formula-not-decoded -->

where p is a positive real number.

The equation system (6) becomes

<!-- formula-not-decoded -->

When S 1 is a quadric surface, the three equations in the equation system (18) are of degrees 2, 2 and 4, and we can obtain two bivariate polynomials of degrees 4 and 5, or a univariate polynomial of degree 16.

## 3.4. Minimum distance between an ellipsoid and an implicit algebraic surface

Suppose the first surface S 2 is an ellipsoid given by

<!-- formula-not-decoded -->

If a = b = c , then we obtain

<!-- formula-not-decoded -->

Otherwise, suppose that a ̸= b , the equation system (6) becomes

<!-- formula-not-decoded -->

When S 1 is a quadric surface, the three equations in the equation system (19) are of degrees 2, 3 and 6. By using the transformation ( x ′ , y ′ , z ′ ) T = ( x / f 1 x , y / f 1 y , z / f 1 z ) T , we can obtain two bivariate polynomials of degree 6, or a univariate polynomial of degree 36, which can be factorized into two polynomials of degrees 12 and 24. We show that the equation system (19) will produce the same formulas as the method in [39]. The details are as follows: the normal congruence of S 1, i.e., ( L 1 , L 2 , L 3 , L 4 , L 5 , L 6 ) T , can be described as

<!-- formula-not-decoded -->

From the equation system (19), we obtain

<!-- formula-not-decoded -->

Since fL = ( L 1 L 4 + L 2 L 5 + L 3 L 6 ) = 0, by adding -c 2 fL to the first equation of equation system (20), we obtain the same formulae as the method in [39].

## 3.5. Minimum distance between a torus and an implicit algebraic surface

Suppose the first surface S 2 is a torus given by

<!-- formula-not-decoded -->

The equation system (6) becomes

<!-- formula-not-decoded -->

When S 1 is a quadric surface, the degrees of the three above equations are 2, 2 and 4, and we can obtain two bivariate polynomials of degrees 4 and 5, or a univariate polynomial of degree 16.

## 4. Minimum distance of a canal surface and an implicit algebraic surface

Our method is able to compute the minimum distance between an implicit algebraic surface and a canal surface. This section, therefore, also considers the case between two canal surfaces as well. It appears that this new method generalizes the method in [17].

Acanal surface is a swept surface along the central trajectory curve C ( t ) = ( x ( t ), y ( t ), z ( t )) T and with the radius function r ( t ) . In this section, a canal surface is the same as that defined in [17], which is the set of all x satisfying

<!-- formula-not-decoded -->

In [17], the characteristic circle K ( t ) is the circle embedded in the moving sphere with center C k ( t ) = C ( t ) -r ( t ) r ′ ( t ) C ′ ( t ) ‖ C ′ ( t ) ‖ and radius rk ( t ) = r ( t ) √ ‖ C ′ ( t ) ‖-r ′ 2 ( t ) ‖ C ′ ( t ) ‖ . The normal of the main plane of the characteristic circle is N k ( t ) = C ′ ( t ) . K ( t ) lies on the canal surface. Let p be a point on K ( t ) . If

<!-- formula-not-decoded -->

is parallel to the normal of the canal surface at p , then Kim [17] provides an equation for solving the property of the canal surface

<!-- formula-not-decoded -->

Let S 1 be an implicit algebraic surface defined by f 1 ( x , y , z ) = 0, and the second surface S 2 a canal surface defined by Eq. (22). Suppose the pair ( p , q ) is a location of the minimum distance. The line pq intersects the central trajectory curve at a point C ( t ) and the vector V = p -C ( t ) is a normal of the canal surface at p . Thus, according to Eq. (23), we have

<!-- formula-not-decoded -->

V is also a normal of S 1 at p . We obtain

<!-- formula-not-decoded -->

In particular, when S 1 is a plane, cylinder, cone, or torus, the point p can be solved geometrically from (25). Substituting them into Eq. (24) yields the same result as in [17].

When S 1 is a general quadric surface, the first three equations in the equation system (25) are linear in the coordinates of point p (which can be rewritten in matrix form M (λ) p = C ( t ) ) and we thus obtain p = M (λ) -1 C ( t ) . Substituting it into Eq. (24) and the last equation in the equation system (25), we obtain two bivariate equations in λ and t . When C ( t ) and r ( t ) are polynomials of degree n and m , the degrees of the two equations are max { 2 n + 4 , 6 } and max { 4 n + 2 , 2 n + 2 m + 2 } , respectively. When r ( t ) is a constant function, the degrees of the two equations are max { 2 n + 4 , 6 } and 2 n + 1, respectively.

When S 1 is a canal surface, suppose that the two canal surfaces S 1 and S 2 are defined by Eq. (22) with their central trajectory curves and radius functions being C 1 ( u ) , r 1 ( u ) and C 2 (v) , r 2 (v) , respectively. Suppose p and q are the closest points on S 1 and S 2, respectively. The line pq intersects the two central trajectory curves at points C 1 ( u ) and C 2 (v) , respectively. According to Eq. (23), we obtain

<!-- formula-not-decoded -->

Suppose C 1 ( u ) , C 2 (v) , r 1 ( u ) and r 2 (v) are polynomials of degrees m 1 ≤ m 2, n 1 and n 2, respectively. Then the two equations in the equation system (26) are of degrees max { 2 m 2 + 2 m 1 -2 , 2 m 2 + 2 n 1 -2 } and max { 4 m 2 -2 , 2 m 2 + 2 n 2 -2 } , respectively. When r 1 ( u ) and r 2 (v) are constant functions, the degrees of the two equations are m 2 + m 1 -1 and 2 m 2 -1, respectively.

## 5. Comparisons

This section compares our method with the methods in [17, 22,39]. Let M 1, M 2, M 3 be the corresponding methods in [17, 22,39] and M n be the corresponding method in this paper respectively. Firstly, M n is able to produce the same formulae as M 1, and is also able to provide the solution for the case between a canal surface and an ellipsoid, or the case between two canal surfaces (see Fig. 2).

Secondly, regardless of whether a case is degenerated in M 2 or not, M n is able to deal with it uniformly. In principle, M n is able to deal with the cases between two implicit algebraic surfaces of degree more than 2.

Thirdly, M 3 presents formulae for the cases between an ellipsoid and a cylinder, a cone, a torus or a parametric surface, and M n is able to produce the same formulae as M 3. Given two surfaces, M 3 needs to describe two normal congruences for the two surfaces, one in parametric form, and the other in implicit form. Usually M 3 deals with the cases between an implicit algebraic surface and a parametric surface. To obtain a normal congruence for a general implicit surface, one needs to eliminate seven unknown variables from 10 polynomial equations (or eliminate four variables from seven polynomial equations by simplification), which is a non-trivial task. On the other hand, in M n , λ is easily eliminated by turning the equation system (5) into

<!-- formula-not-decoded -->

and one only needs to eliminate µ from the equation system (3) and the above equation system (27), which is much easier. Furthermore, the parametric expression of an ellipsoid excludes one point on the ellipsoid, which should be considered additionally. For example, given two ellipsoids

<!-- formula-not-decoded -->

Fig. 2. The minimum distance between two surfaces. The first surface is a canal surface, and the second one is (a) an ellipsoid or (b) a canal surface.

<!-- image -->

Fig. 3. The minimum distance between two implicit surfaces. The first surface is an ellipsoid, and the second one is (a) a cylinder or (b) a cone.

<!-- image -->

minimum distance is 1 . 0, and the location of the minimum distance is (( 0 , 0 , -6 ) T , ( 0 , 0 , -7 ) T ) . S 1 can be parameterized as ( 2 u 1 + u 2 + v 2 , 4 v 1 + u 2 + v 2 , -6 ( u 2 + v 2 ) 1 + u 2 + v 2 ) , which excludes the closest point ( 0 , 0 , -6 ) T .

M 2, M 3 and M n are all able to obtain two resulting bivariate equations, in the unknown variables Lagrange multipliers, parameters of one surface and the coordinates of the closest point on one surface respectively. Figs. 3 and 4 show the cases between an ellipsoid and a cylinder, a cone, an elliptic paraboloid, or a torus, and the corresponding degrees of the two resulting bivariate equations are indicated in Table 1. In Table 1, the first surface S 1 is always a general quadric surface, the second surface S 2 is a general quadric surface, an ellipsoid, a cylinder, a cone, an elliptic paraboloid, a hyperboloid, or a torus. The pair ( m , n ) of numbers denotes the degrees of the two bivariate equations, and the single number denotes the degree of the resulting univariate polynomial. As shown in Table 1, the two bivariate equations obtained by our method have lower degrees than those in other methods.

We have implemented the algorithm proposed in this paper on a 1.7 GHz Windows PC using C + + , including the case between an ellipsoid and a cylinder, a cone, an ellipsoid, or a torus. we first use the method in [39] to obtain the initial solution, and then apply a standard Newton method in two variables. The average computation time for the initial solution is about 20 µ s for the cases between an ellipsoid and a cylinder, a cone, or a torus, and 40 µ s for the case between two ellipsoids. Table 2 shows the corresponding average computation time of M 2, M 3, and M n by iterative search, and the total time should be the sum of the computation time for initial solution and for iterative search. As Table 2 shows, M n is much faster than M 2 and M 3. Since M n produces a resulting univariate polynomial equation of degree 4, we are able to directly compute all the discrete solutions in less than 5 µ s, which is much less than the computation time for the initial solution.

Table 1 Degree analysis for the case between two quadric surfaces

| S 2                    | M 1   | M 2          | M 3     | M n          |
|------------------------|-------|--------------|---------|--------------|
| A general quadric      | -     | (6, 6) or 24 | (8, 16) | (6, 6) or 24 |
| An ellipsoid           | -     | (6, 6) or 24 | (8, 16) | (6, 6) or 24 |
| A cylinder             | -     | (4, 4) or 8  | (4, 4)  | (2, 2) or 4  |
| A cone                 | -     | (4, 6) or 12 | (4, 8)  | (3, 4) or 8  |
| An elliptic paraboloid | -     | (5, 6) or 18 | -       | (4, 5) or 16 |
| A hyperboloid          | -     | (6, 6) or 24 | (8, 16) | (6, 6) or 24 |
| A torus                | -     | -            | (4, 8)  | (4, 5) or 16 |

Table 2 Average computation time for an ellipsoid and a quadric by iterative search

| Quadric   |   Cylinder ( µ s ) |   Cone ( µ s ) |   Ellipsoid ( µ s ) |   Torus ( µ s ) |
|-----------|--------------------|----------------|---------------------|-----------------|
| T 2       |               4.69 |          12.81 |               19.86 |           23.84 |
| T 3       |              10.69 |          21.53 |               28.75 |           21.32 |
| T n       |               1.87 |           6.47 |               16.25 |           11.72 |

For the case between an ellipsoid and a torus, the above iterative search methods may fall into a local minimum, and it may be necessary to compute all the discrete solutions. The algebraic and hybrid method explained in [31] is useful for computing all the discrete solutions, which turns the bivariate equation system into a univariate polynomial equation using the resultant method. The real root isolating method explained in [37] works with huge polynomials of degree 1000 and more, which is useful for solving the resulting univariate polynomial. To compute all the discrete solutions for the case between an ellipsoid and a torus, the average computation times for M 2, M 3 and M n are 3235.0, 5417.2 and 2154 . 7 µ s. Again, M n is much faster than M 2 and M 3.

Fig. 4. The minimum distance between two implicit surfaces. The first surface is an ellipsoid, and the second one is (a) a paraboloid or (b) a torus.

<!-- image -->

## 6. Conclusions

An offset method for computing the minimum distance between two implicit algebraic surfaces is presented in this paper. The offset method also works with the cases between an implicit algebraic surface and a parametric surface. Quadric surfaces, tori and canal surfaces are used to illustrate our method. Our method generalizes the method in [17]. As Table 1 shows, the two resulting bivariate equations in the new method have lower degrees than those in the methods explained in [22] and [39], which may lead to less computation time cost (see Table 2). Our method is able to produce the same (possibly simpler) formulae as the method in [39], but is much easier, for it only needs to eliminate one variable from three polynomial equations, while the method in [39] needs to eliminate four variables from seven polynomial equations.

The minimum distance computation problem usually involves non-linear equation systems. It is possible, with great difficulty, to compute all the roots for the corresponding general non-linear equation systems, but it is not necessary to do so. In the future, we will investigate an efficient method for finding a good candidate solution or for pruning redundant solutions. Another future task is to extend the offset method to the cases between two parametric surfaces.

## Acknowledgements

This research was supported by the Chinese 973 Program (2004CB719403), INRIA scholarship and the National Natural Science Foundation of China (60403047, 60533070). The second author was supported by the Foundation for the Author of National Excellent Doctoral Dissertation of PR China (200342) and the Program for New Century Excellent Talents in Universities (NCET-04-0088). The authors would like to thank the reviewers for their helpful comments. The authors also thank Professor Shiu-chun Wong and Gael Patin for their helpful comments and kind help in polishing the presentation.

## References

- [1] Andrea CD, Dickenstein A. Explicit formulas for the multivariate resultant. Journal of Pure and Applied Algebra 2001;164(1-2):59-86.
- [2] Boender E. A survey of intersection algorithm for curved surfaced. Computer &amp; Graphics 1993;15(1):109-15.
- [3] Bondyfalat D, Mourrain B, Pan VY. Controlled iterative methods for solving polynomial systems. In: Proceedings of the 1998 international symposium on symbolic and algebraic computation. 1998. p. 252-59.
- [4] Brown AA, Bartholomew-Biggs MC. Some effective methods for unconstrained optimization based on the solution of systems of ordinary differential equations. Journal of Optimization Theory and Applications 1989;62(2):211-24.
- [5] Cameron S. Collision detection by four-dimensional intersection testing. IEEE Transaction on Robotics and Automation 1990;6(3):291-304.
- [6] Chionh EW, Goldman RN, Miller JR. Using multivariate resultants to find the intersection of three quadric surfaces. ACM Transactions on Graphics 1991;10(4):378-400.
- [7] Chionh EW, Sederberg TW. On the minors of the implicitization b´ ezout matrix for a rational plane curve. Computer Aided Geometric Design 2001;18(1):21-36.
- [8] Dennis JE, Schnabel RB. Numerical methods for unconstrained optimization and nonlinear equations. Englewood Cliffs (NJ): PrenticeHall; 1983.
- [9] Dietz R, Hoschek J, J¨ uttler B. An algebraic approach to curves and surfaces on the sphere and on other quadric. Computer Aided Geometric Design 1993;10(3-4):211-29.
- [10] Dupont L, Lazard S, Lazard D, Petitjean S. Near-optimal parameterization of the intersection of quadrics. In: Proceedings of the nineteenth annual symposium on Computational geometry. 2003. p. 246-55.
- [11] Fu Q. The intersection of a bicubic b´ ezier patch and a plane. Computer Aided Geometric Design 1990;7(6):475-88.
- [12] Gilbert EG, Foo C-P. Computing the distance between general convex objects in three-dimensional space. IEEE Transaction On Robotics and Automation 1990;6(1):53-61.
- [13] Gilbert EG, Johnson DW, Keerthi SS. Fast procedure for computing the distance between complex objects in three-dimensional space. IEEE Journal of Robotics and Automation 1988;4(2):193-203.
- [14] Hoffmann CM. Geometric and solid modeling: An introduction. San Francisco: Morgan Kaufmann; 1989.
- [15] Jim´ enez P, Thomas F, Torras C. 3D collision detection: A survey. Computers &amp; Graphics 2001;25(2):269-85.
- [16] Johnson DE, Cohen E. A framework for efficient minimum distance computations. In: IEEE International Conference on Robotics and Automation. 1998. p. 3678-84.

- [17] KimK-J. Minimum distance between a canal surface and a simple surface. Computer-Aided Design 2003;35(10):871-9.
- [18] Kim K-J, Kim M-S. Torus/sphere intersection based on a configuration space approach. Graphical Models and Image processing 1998;60(1): 77-92.
- [19] Krishnan S, Manocha D. An efficient surface intersection algorithm based on the lower dimensional formulation. ACM Transactions on Graphics 1997;16(1):74-106.
- [20] Latombe JC. Robot motion planning. Boston (MA): Kluwer Academic Publishers; 1991.
- [21] Lazard S, Penaranda LM, Petitjean S. Intersecting quadrics: an efficient and exact implementation. In: Proceedings of the twentieth annual symposium on computational geometry. 2004. p. 419-28.
- [22] Lennerz C, Sch¨ omer E. Efficient distance computation for quadratic curves and surfaces. In: Proceeding of geometric modeling and processing. 2002. p. 60-9.
- [23] Levin JZ. A parametric algorithm for drawing pictures of solid objects composed of quadrics. Communications of the ACM 1976;19(10): 555-63.
- [24] Lin MC, Canny JF. A fast algorithm for incremental distance and calculation. In: IEEE international conference on robotics and automation. 1991. p. 1008-14.
- [25] Lin MC, Gottschalk S. Collision detection between geometric models: A survey. In: Proceedings of IMA conference on mathematics of surfaces, volume VIII. 1998. p. 37-56.
- [26] Lin MC, Manocha D. Collision and proximity queries. In: Goodman JE, Rourke JO editors. Handbook of discrete and computational geometry, 2nd. 2004. p. 787-807.
- [27] Mirtich B, Canny J. Impulse-based simulation of rigid bodies. In: Proceedings of ACM interactive 3D graphics. 1995.
- [28] Mourrain B, T´ ecourt JP, Teilaud M. On the computation of an arrangement of quadratics in 3d. Computational Geometry 2005;30: 145-64.
- [29] Mullenheim G. On determining start points for surface/surface intersection algorithm. Computer Aided Geometric Design 1991;8(5): 401-8.
- [30] PatriKalakis NM. Surface-to-surface intersections. IEEE Computer Graphics and Applications 1993;13(1):89-95.
- [31] PatriKalakis NM, Maekawa T. Shape interrogation for computer aided design and manufacturing. Berlin: Spring-Verlag; 2002.
- [32] Peternell M, Pottmann H. A laguerre geometric approach to rational offsets. Computer Aided Geometric Design 1998;15(3):223-49.
- [33] Peternell M, Pottmann H, Ravani B. On the computational geometry of ruled surfaces. Computer-Aided Design 1999;31(1):17-32.
- [34] Ponamgi MK, Manocha M, Lin MC. Incremental algorithms for collision detection between solid models. In: Proceedings of ACM symposium on solid modeling. 1995. p. 293-304.
- [35] Pratt M, Geisow A. Surface/surface intersection problems. In: The mathematics of surface I. 1986. p. 117-42.
- [36] Quinlan S. Efficient distance computation between non-convex objects. In: IEEE International Conference on Robotics and Automation. 1994. p. 3324-29.
- [37] Rouillier F, Zimmermann P. Efficient isolation of polynomials real roots. Journal of Computational and Applied Mathematics 2004;162:33-50.
- [38] Shacham M. Numerical solution of constrained nonlinear algebraic equations. International Journal for Numerical Methods in Engineering 1986;23(1):1455-81.
- [39] Sohn K-A, J¨ uttler B, Kim M-S, Wang W. Computing distances between surfaces using line geometry. In: Pacific conference on computer graphics and applications. 2002. p. 236-45.
- [40] Tanaka S, Fukuda Y, Yamamoto H. Stochastic algorithm for detecting intersection of implicit surfaces. Computers &amp; Graphics 2000;24:523-8.
- [41] Tu CH, Wang WP, Mourrain B. Signature sequence of intersection curve of two quadrics for exact morphological classification. Technique reports of Department of Computer Science, Hong Kong University, 2005.
- [42] Turnbull C, Cameron S. Computing distances between nurbs-defined convex objects. In: IEEE international conference on robotics and automation. 1998. p. 3686-90.
- [43] Wang S, Yang XQ, Teo KL. A unified gradient flow approach to constrained nonlinear optimization problems. Computational Optimization and Applications 2003;25(1-3):251-68.
- [44] Wang W, Goldman R, Tu CH. Enhancing levin's method for computing quadric-surface intersections. Computer Aided Geometric Design 2003; 20(7):401-22.
- [45] Wang W, Wang J, Kim M-S. An algebraic condition for the separation of two ellipsoids. Computer Aided Geometric Design 2001;18(6):531-9.
- [46] Wee CE, Goldman RN. Eliminations and resultants 1: Elimination and bivariate resultants. IEEE Computer Graphics and Applications 1995; 15(1):69-77.
- [47] Wee CE, Goldman RN. Eliminations and resultants 2: Multivariate resultants. IEEE Computer Graphics and Applications 1995;15(2):60-9.
- [48] Xu ZQ, Wang XS, Chen XD, Sun JG. A robust algorithm for finding the real intersections of three quadric surfaces. Computer Aided Geometric Design 2005;22(6):515-30.

<!-- image -->

Xiao-Diao Chen is a Ph.D. student in the Department of Computer Science and Technology at Tsinghua University, China. His research interests are computeraided design and computer graphics.

<!-- image -->

Jun-Hai Yong is a faculty member in the School of Software at Tsinghua University, China, since 2002. He received his B.S. and Ph.D. in computer science from the Tsinghua University, China, in 1996 and 2001, respectively. He held a visiting researcher position in the Department of Computer Science at Hong Kong University of Science &amp; Technology in 2000. He was a post-doctoral fellow in the Department of Computer Science at the University of Kentucky

from 2000 to 2002. His research interests include computer-aided design, computer graphics, computer animation, and software engineering.

<!-- image -->

Guo-Qin Zheng is an associate professor in the School of Software at Tsinghua University, China. He received his B.S., Master and Ph.D. in mechanical manufacturing from Harbin Institute of Technology, China. His research interests include computer-aided design and software engineering.

<!-- image -->

<!-- image -->

Jean-Claude Paul is a senior researcher at CNRS and INRIA (France), and currently visiting professor at Tsinghua University. He received his Ph.D. in Mathematics from Paris University in 1976. His research interests include Numerical Analysis, Physics-Based Modeling and Computer-Aided Design.

Jia-Guang Sun is a professor in the Department of Computer Science and Technology at Tsinghua University, China. His research interests are computer graphics, computer aided design, computer-aided manufacturing, product data management and software engineering.