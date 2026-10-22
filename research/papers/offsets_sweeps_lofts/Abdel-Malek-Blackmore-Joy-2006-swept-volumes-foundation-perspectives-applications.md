Abdel-Malek, K., Blackmore, D., and Joy, K., (submitted), ' Swept Volumes: Foundations, Perspectives, and Applications ', International Journal of Shape Modeling.

## Swept Volumes: Foundations, Perspectives, and Applications

## Karim Abdel-Malek

Department of Mechanical Engineering and Center for Computer Aided Design University of Iowa Iowa City, IA 52242

Tel: 319-335-5676

Amalek@engineering.uiowa.edu

## Abstract

Several fundamental developments in the past decade have led to a better understanding of swept volumes. While  the  underlying  formulation  for  characterizing  the  volume  generated  as  a  result  of  the  motion  of  a geometric entity in space has appeared in various fields under different names, this review seeks to unify the terminology  and  demonstrate  the  applicability  to different fields. This paper  reviews the various formulations  that  have  appeared,  outlines  the  basic  research  involved,  and  highlights  the  implications  on research  in  engineering,  mathematics,  and  computer  science.    The  applicability  of  this  seemingly  simple formulation  to  the  fields  of  solid  modeling,  robotics,  manufacturing  automation,  and  visualization  is demonstrated through results reported by the authors, each in their own field.

Keywords: Swept volumes, workspace analysis, NC verification, solid modeling.

## 1.0 Historical Remarks

Attempts were made to obtain the exact swept volume of the motion of a rigid body in space  in the  early 1960's using a combination of calculus and graphical techniques. .  We define a swept volume as the volume generated  by  the  motion  of  an  arbitrary  object  along  an  arbitrary  path  (or  even  a  surface)  possibly  with arbitrary  rotations.  The  problem  of  swept  volume  determination  is  concerned  with  the  calculation  of properties associated with the resulting sweep, including the delineation of the boundary for visualization and computer graphics, formulations for modeling the sweep procedure, and the calculation of the volume's mass properties  for  solid  modeling.    Perhaps  the  most  challenging  problem  facing  this  field  has  been  the identification  and  visualization  of  the  boundary,  especially  for  those  volumes  that  are  based  on  freeform geometric entities.  The concept of determining the so-called necessary boundary at several steps during the sweep  have  been  used  in  early  works  (although  still  used  today  in  some  implementations),  where

## Denis Blackmore

Department of Mathematical Sciences New Jersey Institute of Technology Newark, New Jersey 07102 Tel: 201-596-3495

Deblac@chaos.njit.edu

## Kenneth Joy

Computer Science Department University of California Davis Davis, California 95616

Tel: 1-530-752-1077

Joy@cs.ucdavis.edu

intermediate  frames  are  computed,  their  necessary  boundary  identified,  and  finally  all  boundaries  are connected (see Fig. 1).

Fig. 1 Sweeping of a geometric entity

<!-- image -->

While the mathematics used in treating sweep problems have become more sophisticated, the idea remains the same.  The goal is to determine, mathematically or not, the outer boundary of the object (also called the generator) during its motion from an initial to a final position and orientation.  The main difficulty has been (and remains to some extent) the identification of this boundary, obtaining an accurate representation, and the development of fundamental formulations implementable in computer code.

Although some researchers have  investigated  what  seemed  to  be  efficient  methods,  they  were  faced  with problems of visualization because of the limited computational power available at that time.  In fact, the field of  descriptive  geometry  has  addressed  similar  issues  (Wellman  1957).    Until  recently,  the  problem  of computing the swept volume was thought to be hindered by seemingly complex mathematics and the selfintersection of the generated volume. Many researchers presented solutions varying from implicit approaches (Klok 1986) to the numerical implicit solutions (Schroeder, et al. 1994 and Ganter, et al. 1993).

Indeed, many have addressed specific problems pertaining to swept volumes.  For example, Kim and Moon (1990)  presented  an  algebraic  algorithm  for  generating  the  purely  rotational  sweeping  volumes  of  planar objects bounded by algebraic curves. The sweep volume boundary is related to convolutions consisting of the planar object  boundaries  at  its  start  and  final  angles  and  the  circular  sweep  arcs  of  radial  extreme  points. Parida  and  Mudur  (1994)  identified  a  sufficiently  general  class  of  swept  objects  and  have  classified  the sweep rules.  Although some believe that swept volume methods employ the similar mathematics to that of offsets and Minkowski Sums (Elber and Kim 1999, Maekawa 1999, and Ahn, et al. 1997), we believe that formalisms for addressing swept volumes are very different that those for offsets. A different approach using a  surface  fitting  algorithm  for  sweep  surface  reconstruction  from  three-dimensional  measured  data  was presented by Ueng, et al. (1998), but limited to translational sweeping in which the generators traverse about the directors to form the desired sweep surface.

A  formulation  for  developing  a  geometric  representation  of  swept  volumes  for  compact n -manifolds undergoing general sweeps in R n was  presented  by  Weld  and  Leu  (1990)  as an  extension  of  the  work  by Wang and Wang (1986). It was shown that the swept volume of a compact n -manifold in R n is equal to the union of the swept volume of its boundary with one location of the compact n -manifold in the sweep. Based on this formulation, the swept volumes of polyhedral objects were generated from the swept volumes of their polygonal  faces.   The  work  by  Wang  and  Wang  was  continued  by  a  colleague  of  the  first  author  and  in collaboration with a mathematician at NJIT (second author of this paper).

In 1990, Blackmore and Leu introduced a new type of formulation, whereby every smooth Euclidean motion, or  sweep,  can  be  identified  with  a  first-order,  linear,  ordinary  differential  equation.  The  so-called  Sweep Differential Equation (SDE) method provided useful insights into the topological and geometrical nature of the swept volume of an object and will be further explored below.

Singularity  theory  (also  referred  to  as  Manifold  stratification  or  Jacobian  rank  deficiency  method  [AbdelMalek and Yeh 1997a and Abdel-Malek and Othman 1999]) was shown to address the determination of the boundary volume  obtained as  a result  of  multiple  sweeping  (multiple  sweeping  is  a  term  that  denotes  the generation  of  a  volume  in  space  whereby  an  entity  is  arbitrarily  swept  several  times  adding  a  number  of sweep parameters to the sweep equation).  It was shown that by an appropriate  manipulation of the sweep Jacobian, it is possible to stratify the Jacobian to obtain exact boundary surface patches in closed form (also called strata).  The method was expanded in recent years and was shown to address outstanding problems in robotics, manufacturing automation, and ergonomics.

In the following section, we will first introduce the fundamental concepts associated with sweeping using a simple  examples,  will  introduce  the  formulation  therein,  and  will  explore  both  specialized  and  common problems associated with sweeping, implementation, and applications.

## 2.0 Modeling

The fundamental theory of swept volumes is rooted in the Implicit Function Theorem. We first illustrate a simple  example  to  motivate  the  discussion.  We  will  then  address  two  formulations  that  have  recently appeared that treat the problem of swept volumes using (1) Manifold Stratification and (2) Sweep Envelope Differential Equation (SEDE) method.

## 2.1 Simple Example:

Consider a given curve specified by the parabola

<!-- formula-not-decoded -->

shown in Fig. 2, or in parametric form as c45

<!-- image -->

c136

c136

Fig. 2 A parabola defined on an interval 2 1 x

This curve is to be swept in the x -direction along the path given by c45

c136

c136

<!-- formula-not-decoded -->

The sweep equation is defined as

c61 c43 c45 c45 c136 c136 c136 c136 This equation represents a sweep in the xy -plane. In  order  to  define  the  boundary  of  this  sweep,  we  substitute  the  limits  for  each  parameter.    For  example, substituting the limit s c61 c45 2 into c120 ( , ) s t yields the equation of a line denoted by c108 1 defined by [ ] t T c45 2 0 on the interval t c179 [ , ] 0 5 .  Substituting the limit s c61 1 yields the line c108 2 defined as [ ] t T c43 1 3 on the same interval.  Similarly, substituting the limit for t c61 0 yields the equation of the  original curve  denoted by c108 3 and defined by [ ] s s T 4 2 c45 . Substituting t c61 5 yields the parabola after it has translated along the x -axis by five units as c108 4 defined by [ ] s s T c43 c45 5 4 2 also on the interval s c179 c45 [ , ] 2 1 .  The plots of c108 c108 1 4 c45 are shown in Fig. 3.

Two remarks:

- (1) Some curves that must be on the boundary are still missing (in particular, the line segment joining the tip of each curve).
- (2) The curves that have been determined thus far may or may not be on the boundary.

To determine the missing curve, one must find the global/local maximum point on the curve in the direction of  sweep  (i.e,  the  tip  of  the  parabola).    In  this  case,  the  local  and  global  maximum  for  the  parabola  is  at s c61 0 obtained by differentiating the curve with respect to s and setting the result to zero.  Substituting s c61 0 into the sweep equation yields the curve denoted by 5 and shown in Fig. 4.

<!-- image -->

Once all  curves  are  determined,  it  is  now  necessary  to  identify  those  curves  that  are  on  the  boundary  by eliminating  all  curve  segments  inside  the  swept  region.    This  problem  has  been  denoted  by  various designations  such  as  trimming  (Blackmore, et  al. 1999),  boundary  identification  (Abdel-Malek  and  Yeh 1997a), and clipping (Madrigal and Joy 1999). For this example, removal of internal curves (those that are

<!-- formula-not-decoded -->

<!-- image -->

not on the boundary) is performed and the result is indeed the swept region shown in Fig. 5 (note that the term swept volume is typically used even if the region is two-dimensional).

4

<!-- image -->

Note that if the curve above is swept along the y -axis instead, only its endpoints would yield curves and a maximum  cannot  be  found.      This  simple  idea  can  be  extended  into  a  generalized  formulation  by investigating the Taylor Series expansion of a vector function of several variables denoted by c120 ( ) q , where

q = [ ... ] q q n T 1 , where q i are the sweep parameters (i.e., q c61 s t T ) in the previous example.

c120 c120 c120 ( ) ( ) ( ) ( ) . ( ) ( )( ) q q q q q q q H q q q c61 c43 c182 c45 c43 c45 c45 c43 o o T o o o o R 05 (5) where H is the Hessian of c120 and R represents higher order terms.  A change in the function represented by its gradient is given as c182 c61 c182 c45 c43 c45 c45 c43 c120 c120 ( ) ( ) . ( ) ( )( ) q q q q q H q q q o T o o o o R 05 (6) If  we  assume a local  minimum then c182 c120 must  be  nonnegative, i.e., c182 c152 c120 0.   Concentrating  only  the  firstorder term in Eq. (6), we observe (as before) that c182 c120 can be nonnegative for all possible  ( ) q q -o when c182 c61 c120 ( ) q 0 o (7) In the case of a vector function c120 ( ) q , the equivalent of Eq. (b) is = -+ c120 J q q q ( ) ( ) o T o R (8) where J q q c61 c155 c155 c61 c120 c120 and setting the gradient of a function of several variables to zero is equivalent to setting the determinant of its square Jacobian matrix J to zero (i.e., finding a singular point).  Therefore, for a  3D  vector  ( c120 )  with  three  variables,  the  Jacobian  is  a  ( ) 3 3 · matrix,  whose  determinant  is  an  analytic function.  The complexity in analysis arises  when  many variables are used to define the sweep (i.e., more than three variables), resulting in a non-square Jacobian.

In order to have a well-posed formulation, constraints that are used to model the geometry of this problem should  be  independent,  except  at  certain  critical  surfaces  in  the  manifold  ( Implicit  Function  Theorem ) where the Jacobian becomes singular. However, for sweeps with more than 3 parameters, the sweep Jacobian is not square. It is important, therefore, that there not be open sets in the space of the generalized parameters in which the constraints are redundant.

## 2.2  Sweep Equations

Consider  a  surface  parameterized  in  terms  of  two  variables  as  a  ( ) 3 1 · vector  given  by c71 ( ) u ,  where u c61 u u T 1 2 .  This surface will be swept along a specified path given by the  ( ) 3 1 · vector c89 ( ) v ,  where the orientation of the surface is defined by a  ( ) 3 3 c153 rotation  matrix denoted by R ( v ).    The sweep equation generated by the motion of ( ) u on ( ) v is  defined as c120 c71 c89 ( , , ) ( ) ( , ) ( ) u u v v u u v 1 2 1 2 c61 R (9) where c120 characterizes the set of all points inside and on the boundary of the swept volume.

c71

c89

c43

To define stationary points (in this case it may be sets of constant parameters or sets of parameters defined as a function of other variables), consider the derivative of the accessible set (Eq. 9) with respect to time as

where the dot denotes a derivative with respect to time and the matrix

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

is called the Jacobian of the sweep (or sweep Jacobian) and has dimension  3 3 c153 .  The left-hand side of Eq. (10) is the velocity c38 c38 c38 x y z T of a point moving in the swept volume having a parameter velocity vector c38 q .  The physical significance of the Jacobian was thoroughly explained and used in the study of the motion of robot manipulators (Abdel-Malek, et al. 1999a).  In order to determine stationary points, the Jacobian is studied for rank deficiency.  Because the Jacobian is square, its determinant is equated to zero and solved for constant values (or expressions) that make the Jacobian singular.  The term singularity (singular set) is given to those values or expressions.

This  Jacobian  determinant  singularity  has  been  the  fundamental  underlying  theory  of  the  so-called  sweep envelope theory ,  which  has  been  consistently  limited  to  3  parameter  sweep  until  recent  work  by  AbdelMalek and Yeh (1997a) and Abdel-Malek and Othman (1999).  Early use of envelope theory is attributed to Dahlberg and Johansson (1987), Flaquer, et al. (1992), and Ganter and Uicker (1986) where it was used by the latter for detecting collisions and Wang and Wang (1986) where it was demonstrated for simulating the NC  machining  process  (see  the  theory  by  Boltianskii  1964).    Envelope  theory  was  used  by  Martin  and Stephenson (1989; 1990)  to  depict  the  swept  volume  of  simple  solids  and  by  Hu  and  Ling  (1994;  1995; 1996) to generate characteristic curves that are the point sets on the boundary of the  generator at different times.  The characteristic curves of the natural quadric surfaces (planes, circular cylinders, circular cones and spheres) were derived from the envelope theorem, which is also termed the instantaneous screw axis theorem in the field of kinematics, and it was used to describe the sweep motion of the generators. The faceted model of a swept volume is then established by warping the corresponding characteristic curves together with the partial boundaries of the generator at the initial and final positions.

Numerical methods to generate swept surfaces and volumes using implicit  modeling  techniques  were  also reported  (Schroeder,  et  al.  1994).  The  algorithm  reportedly  treats  degenerate  trajectories  such  as  selfintersections and surface singularity. Applications of this algorithm to maintainability design and robot path planning were demonstrated.  More recently, still, an expansion of the Jacobian ( ) 3 3 c153 singularity, Madrigal and Joy (1999) swept a Bezier surface to generate a solid of three variables (called a Trivariate Bezier Solid) and showed that characteristic curves can be computed even for the complex Bezier surfaces (this work will be explored in more detail in the Solid Modeling section).  Recent works that have demonstrated  methods for computing swept volumes are (Ling and Chase 1996 and Sourin and Pasko 1996). Other works that are related to swept volume formulations include ray representation methods (Hartquist, et al. 1999), methods for handling specific types of sweeping ( Jüttler and Wagner  1999), and methods for improving the approximation of swept volumes (Elber 1997).

To  summarize,  all  of  the  above  methods  for  formulating  the  sweep  equation  or  the  sweep  differential equation  have  been  able  to  address  very important  problems,  pertaining  to  boundary  identification, visualization, and mathematical representation.  However, these methods although they have the potential for expansion to multiple parameters, have been limited to three parameters.

## 2.3  Multi-parameter Sweeps (e.g., multivariate solids)

Using the formalism of shape feature languages, Jakubowski (1993) developed a multiple sweeping method for  product  modeling.  A  geometric  entity  with  more  than  two  parameters  (a  surface,  a  volume,  or  a n -dimensional entity) parametrized in terms of one or more variables as a  ( ) 3 1 · vector given by ( ) u , where

<!-- formula-not-decoded -->

c71 u = [ ... ] u u n T 1 , that is swept in space has also been considered (Abdel-Malek and Othman 1999) using the  a  systematic  method  for  consecutive  sweep  representation  adopted  from  kinematics  (Denavit  and Hartenberg 1955).  Multiple arbitrary sweeps are performed to generate a complex solid, the result of which has been called a multivariate solid (Abdel-Malek and Yang 2000).  Let the path also be defined by c89 1 1 ( ) v and the orientation defined by R 1 1 ( ) v such that the sweep equation is N q q q q R u 1 1 1 1 1 ( ) ( ) ( ) ( ) ( ) ( ) ( ) = = + x y z v v T c71 c89 (12) where q is  the  vector  of generalized  coordinates defined  by q = = [ ... ] ... q q u u v n T n T 1 1 1 and R 1 1 ( ) v is  the ( ) 3 3 · rotation  matrix  defining  the  orientation  of  the  swept  entity.    In  fact, ) ( 1 q N characterizes  the  set  of  all  points  that  belong  to  the  volume.    Another  sweep  motion  with  an  orientation R 2 2 ( ) v and along the path 2 yields an expanded swept volume in the form of

c89 c71 c89 c89 where now q = [ ... ] u u v v n T 1 1 2 .  Consecutive sweeps yield a modified set defined by N R R R R R R 3 3 2 1 3 2 1 3 2 3 = + + + (14) The generalized case yields the vector function c120 = c120 c120 c120 ( ) ( ) ( ) x y z T , such that

c89

<!-- formula-not-decoded -->

where q u v = T T T , u = [ ... ] u u n T 1 , and v = [ ... ] v v m T 1 .  This is a general sweep of a n -

dimensional geometric entity.  Indeed, a similar sweep equation can be defined as

<!-- formula-not-decoded -->

c43 where in this case not only c71 has many parameters but the path could also be an m -dimensional entity.  The significance and need for this formulation is evidenced in problems arising in robotics, solid modeling, and computer graphics.  For example, consider the robot shown in Fig. 6. The kinematic skeleton of the arm is shown in Fig. 6b where joints are characterized by q i i ; ,..., c61 1 4. The vector describing the workspace of a point P on the end-effector of the manipulator is given by c120 ( ) q , where this point has been repeatedly swept in space and the volume generated by every point touched by the end-effector is called the swept volume or 'workspace' in the field of robotics. In this case, the point P is swept first using the angle q 4 , the results of which is swept again using the angle q 3 , and subsequently using q 2 and q 1 . The same can be done for any number of DOF.

Fig. 6 (a) a robot manipulator (b) the robot's kinematic skeleton

<!-- image -->

## 2.4  Deformable Sweeps

Stretching, tapering, and twisting, for example, of an object while it is swept is an important subject and that has become more crucial in recent years for modeling membranes and deformable objects, whether in virtual reality  applications  or  for  educations  purposes  (e.g.,  modeling  of  tissues  for  medical  aplications).  For example, Sealy and Wyvill (1997) reported a method for generating arbitrary sweep objects, where the object being swept may be a 2D contour or a 3D object. Blackmore et al . (1994) extended the SDE approach from rigid sweeps to objects experiencing general smooth deformations. To accomplish this, they observed that an object that is deforming as it is swept along can be modeled by the action of a sweep of the form

<!-- formula-not-decoded -->

d dt d dt d dt d dt P W x R R x R c61 c43 c45 c45 c43 c45 c89 c89 ( ) ( ) ( ) 1 (18) where P and W are nonlinear functions associated to the nonlinear portion of the deformation experienced by the swept object. Once the appropriate SDE (Blackmore and Leu 1990 and Blackmore, et al. 1994) was derived, Blackmore and  Leu  and  their  collaborators  were  able  to  simply  replicate  the  procedures  for  rigid  sweeps  in  order  to obtain algorithms for the approximate computation of the deforming swept volumes. Very recently Wang et al . (2000) extended the sweep-envelope differential equation (SEDE) method, developed by Blackmore and Leu and their associates as an improvement for the SDE, to general deformed swept volumes.

c43 c43 where N ( x , t )  represents  a  general  nonlinear  deformation. Here it is useful to  note that when the  sweep  is rigid, R ( t ) x + N takes the simple form R ( t ) x , where R ( t ) is an orthogonal matrix of determinant = 1. Just as in  the  case  of  a  rigid  sweep,  the  above  equation  for  a  deforming  sweep  leads  to  a  corresponding  sweep differential equation (SDE) of the form

## 3.0 Formalisms

## 3.1 Manifold Stratification Method (Jacobian Rank Deficiency)

In differential geometry, a Manifold with singularities is defined as a manifold with singular or boundary parts of lower dimensions (e.g., 3, 2, and 1).  Because the sweep Jacobian defined by Eq. (11) characterizes the mapping from 3-space to n -dimensions it is natural to employ stratification (reduction) of the manifold to many pieces of lower dimensions.  A Stratified space is defined as a toplogical space which is built up from lower  dimensional  pieces  that  are  boundaryless  manifolds.    Based  on  singularity  theory  (Spivak  1968, Guillemin and Pollack 1974, and Lu, 1976), Abdel-Malek, et al. (1998) and based on earlier reports (AbdelMalek and Yeh 1997a; 1997d), were able to show that it is possible to generate the exact boundary envelope of  a  swept  volume  using  successive  stratification  of  the  manifold.  Implementation  of  Jacobian  rank deficiency  conditions  yields  a  subset  of  a  Euclidean  space  defined  by  zeros  of  a  finite  number  of differentiable functions (called varieties).  As a result, Strata (plural of the Latin word stratum-layer or level) are defined  and  used  to  delineate  boundary  sub-varieties.    Some  other  aspects  of  swept  volume  geometry were also recently analyzed using singularity theory in Blackmore et al . (2000).

## 3.2 Sweep Differential Equation Method and Extensions

An approach to swept volumes that fully exploits the intrinsic geometric and group theoretical structure of Euclidean  motions  was  first  presented  by  Blackmore  and  Leu  (1990)  and  later  developed  in  detail  in Blackmore and Leu (1992a). The key element of this approach is the sweep differential equation (SDE) that is  derived  from  the  underlying  Lie  group  structure,  which  generates  the  trajectories  that  determines  the geometry of the swept volumes.

More precisely, the boundary of the swept volume of an object can be shown to be subset of the union of (1) the grazing points on the boundary of the object during the entire sweep at which the vector field of the SDE neither points into or out of the object interior, (2) the ingress points at the beginning of the sweep, and (3)

the  egress  points  on  the  object  boundary.  A  variant  of  the  characterization  for  grazing  curves  for  the generation of swept volumes was recently developed in Chio and Lee (1999).  The grazing points at each time are computed from a tangency function describing the relationship  between  the  sweep  vector  ant  the outward normals for points on the object boundary.

One computationally expensive aspect of the SDE method is that, except for the classes of sweeps defined as autonomous  by  Blackmore  and  Leu,  the  grazing  point  curves  must  be  computed  at  every  time  along  the sweep. In order to overcome this computational difficulty, Blackmore et al . (1997b) developed an extension of the SDE method that they called the sweep-envelope differential equation (SEDE) approach and presented examples  that  illustrate  successful  integration  of  a  prototype  SEDE  program  with  commercial  NC verification software. The authors reported that the  major  advantages  of  an  algorithm  based  on  the  SEDE method are:

- (1) The grazing points set need only be computed at the initial position of the object - the remaining points are generated by the SEDE induced flow of the initial grazing points - so the computational complexity is drastically reduced; and
- (2) It  provides automatic connectivity for computed boundary points (along trajectories of the SEDE) that facilitates  integration  with  standard  algorithms  and  CAD  software  for  visual  realization  and  Boolean operations.

Algorithms based on the SEDE method have a computational cost that is a full order of magnitude less than those  based  on  the  SDE,  making  them  suitable  for  real-time  application  in  NC  machining  verification problems  and  related  problems  in  manufacturing  automation.  The  SEDE  algorithm,  which  was  originally developed only for smooth objects, was used together with some  novel smooth approximation formulas in order to calculate the swept volume generated by a general 7-parameter APT tool for a large class of sweeps that  includes  all  possible  motions  in  5-axis  NC  milling  processes  in  Leu et  al .  (1997).  Blackmore et  al . (1997a) later generalized the SEDE method to piecewise smooth objects and Wang, et al. (2000) have just extended it to deformed swept volumes.

## 3.3  Parametric, Implicit, and Free Form

Although  parametric  representations  are  more  widely  used  and  are  generally  much  simpler  to  treat  than implicit  representations,  swept  volumes  are  widely  used  in  many  applications  and  it  is  common  in  solid modeling, for example, to represent solids in implicit forms (simpler to use and to store).  In such cases, a CAD program must have the ability to perform sweep operations on the implicitly represented solid without converting the equation to parametric form.

In  joint  work  by  the  first  two  authors  and  associates,  a  new  method  for  exact  determination  of  sweeps  of implicit  surfaces  was  recently  presented  (Abdel-Malek,  et  al.  2000a;  Abdel-Malek  and  Yang  2000).    The method  is  briefly  summarized  here  for  it  provides  a  new  approach  to  the  sweep  of  implicit  surfaces. Consider the sweep equation

<!-- formula-not-decoded -->

c43 where c71 c61 [ ,..., ] x x M T 1 and w x = [ ] T T t ,  and where c89 and R are, respectively,  smooth (= C ¥ )  vector  and matrix  valued  functions  with c89 ( 0) c61 0 , R I ( 0) c61 is  the  identity  matrix,  and R ( ) t is  a  real ( ) 3 3 · orthonormal  matrix  (in  the  case  of  the M =  3)  for  every t ˛ [ , ] 0 1 .    For  the  purpose  of  developing  the formulation, it is required only that M be defined as the set of points satisfying finitely many equalities and inequalities for differentiable functions.

In  order  to  develop  a  method  that  is  consistent  and  general  for  any  implicit  surface,  we  will  represent  the surface M by a number of inequalities, with no restriction on the number of inequalities nor on the type of variables.  Let M be represented by

c136 c136 c136 c136 c71 c71 c71 c108 c108 c108 c108 where f = [ ,..., ] f f N T 1 and f i ( ) c71 denotes an expression representing the surface, c108 i is  a  limit,  the  upper subscript L indicates the lower limit, U denotes the upper limit, N is the number of expressions, and k is the number of variables.  It should be noted that this is a rather general case, but a surface may be represented by any number of expressions and any number of variables that may or may not have limits.

<!-- formula-not-decoded -->

As a result of this transformation, it is now possible to define the sweep by

c45 c43 c45 c45 c45 c45 where z = = + + + + x x t z z N N M T N M T 1 1 1 1 2 2 ,..., , , ,..., ,..., l l or using partitioned coordinates z w = [ , , ] c108 c108 1 2 T where w = [ ,..., , ] x x t N T 1 , c108 1 1 = [ ,..., ] l l N T , and c108 2 1 1 = + + + [ ,..., ] l l N N M T .    Note that the parameter t has also been included in ( ) z .

<!-- formula-not-decoded -->

c70 Rank  deficiency  conditions  first  developed  by  Abdel-Malek  and  Yeh  (1997a)  can  now  be  applied  to determine singular sets of the Jacobian z z z ( ) = ¶ ¶ defined by

c70 c108 c108 where c120 c120 w w c61 c155 c155 ; f f w w = ¶ ¶ ; f f c108 c108 1 1 = ¶ ¶ ;  and w w c108 c108 2 2 = ¶ ¶ .    The  Implicit  Function  Theorem mandates that there not be open sets in the space of z in  which the constraints are redundant.  Redundancy occurs  when  the  Jacobian c70 z ,  is  rank-deficient  which  will  subsequently  define singular  surfaces in  the manifold. Singularities as a result of the Jacobian rank deficiency condition are denoted by s i .  The example shown in Fig. 7 was treated using both the SEDE method (Blackmore, et  al. 1999) and the Jacobian rank deficiency method (Abdel-Malek, et al. 2000), where a solid cylinder implicitly defined is swept (with selfintersection).

<!-- formula-not-decoded -->

Fig. 7 Two views of the swept volume (same example treated in Blackmore, et al. using SEDE trimming methods)

<!-- image -->

## 3.4  Identifying the Boundary and Clipping Internal Entities (also called trimming)

To illustrate trimming, consider the simple problem of determining the swept volume of a point on the endeffector of a 2DOF robot manipulator as shown in Fig. 8. Analysis of the equations characterizing the totality of points touched by point P yields several curves as shown in Fig. 2b.  In order to only depict the boundary (i.e., the envelope) to this workspace, it is necessary to perform clipping (also called trimming) of the internal branches.  While this is a simple planar example, this task may prove difficult for complex shapes.  Indeed, those curve segments that are to be clipped must first be identified. The result of clipping for this example is shown in Fig. 8c.

<!-- image -->

## Fig. 8 (a) A 2-DOF robot arm (b) Internal curves as a result of the analysis (c) Trimming curves to delineate the envelope of the swept area (i.e., the workspace of the robot)

A number of methods have appeared that provide solutions to the trimming problem:

- (a) Surface-surface intersection. It has been shown that the problem of trimming is best treated using mature surface-surface intersection methods (Bajaj et al . 1988, Barnhill and Kersey 1989, Barnhill, et al. 1987, Abdel-Malek  and  Yeh  1996;  1997b),  followed  by  an  elimination  procedure  such  as  the  perturbation method (Abdel-Malek and Yeh 1997a).  Indeed, Blackmore et al . (1999) implemented efficient surfacesurface methods for both local and global trimming of swept volumes that proved computationally cost effective when combined with approximate calculation and graphical rendering of swept volumes.
- (b) Z-buffer  (see  e.g.  Wang  and  Wang  (1986)  is  a  well-known  efficient  technique  used  in  computer graphics).
- (c) Ray-casting (see e.g. Menon and Voelcker 1992 and Menon and Robinson 1993) is a procedure whereby a ray is cast from a point inside the swept volume and intersected with all surfaces. Points of intersection are numbered and their distances computed. Only surfaces with associated points of farthest distance are kept as they must belong to the boundary of the swept volume.

Blackmore et  al .  (1999)  developed  a  particularly  efficient  algorithm  for  computing  swept  volumes  that included  trimming  self-intersections.  They  first  computed  a  candidate  set  for  the  boundary  of  the  swept volume using the SEDE approach and then used the following method to trim the superfluous points: The object  was characterized by a piecewise smooth function f that  is  negative  in  the  interior,  positive  in  the exterior and zero on the boundary of the object. They then  defined the function j x ( t ):= f ( h ( x , t )),  where h represents  the  reversed  flow  induced  by  the  SDE.  Global  trimming  was  performed  for  candidate  points x such that j x ( t )&lt;0 for some t belonging to the sequence of time steps after local trimming criteria involving first  and  second  time  derivatives  of j x had  been  tested.  The  computational  complexity  of  their  algorithm, including trimming, was shown to be O ( m 2 log( log m )), where m -1 is a  lower bound for the time steps and the mesh diameter of a triangulation of the boundary of the object.

## 3.5  Volume and Surface Area Computations

The  sweep  generation  of  solid  models  is  a  widely  accepted  technique  in  dealing  with  complex  shapes occurring in mechanical components such as turbine blades and bevel gears. A key issue in the  geometric and mechanical analyses of such  elements is the accurate and economical computation  of their volumetric properties,  namely,  their  volume,  centroid  coordinates  and  inertia  tensor.    Al-Daccak  and  Angeles  (1989) presented a method for the computation of mass properties for sweep-generated solids based only on their 2D generating contour and their sweeping parameters. The direct 3D calculations of the volumetric properties of swept volumes were reduced to simple 2D calculations.  The calculation of geometric or mass properties in CAD systems has been reported in many works (Hoffmann 1989).

Most  useful  reported  techniques  for  computing  the  swept  volume  involve  integral  identities  that  convert volume to surface integrals (see a review by Akin 1990).  Other methods involve filling the solid with known entities (such as infinitesimal boxes or spheres) to compute the volume.  The most widely known technique (Olfe 1995) is computed based on a ray casting procedure whereby the total number of cells of each crosssection  is  computed  using  a  meshing  procedure.    The  volume  is  calculated  by  summing  the  individual material  volumes  contained  in  the  cells.    An  interesting  approach  to  computing  the  volume  of  a  solid enclosed by a recursive subdivision surface (Peters and Nasri 1997).

## 4.0  Applications

## 4.1 Numerically Controlled Machining Verification

Numerically controlled  machining  verification (known as  NC  verification)  of  parts  prior  to  manufacturing leads to  higher quality parts and significantly reduced  lead times.  NC verification is a very  cost-effective tool.    For  NC  verification,  the  method  for  representing  the  swept  volume  removed  by  a  tool  as  it  is manipulated over the workpiece is sometimes called the cutter-swept surface (CSS).  Seng and Joshi (1998) proposed a method whereby generated machining volumes are classified using face adjacency relationships of the bounding faces.  For example, Sheltami, et al. (1998) proposed a technique that is based on identifying generating curves along the path and connecting them into a solid model of the swept volume.

Parametric  representations  for  the  surfaces  generated  by  common  NC  milling  cutters  during  five-axis motions were extracted using the theory of envelopes by Sambandan and Wang (1989). Also for 5-axis NC verification, surfaces swept by a  general  tool  motion,  between  the  initial  and  terminal  tool  positions  were represented  by  several  characteristic  profiles  and  parametric  definitions  of  the  tool  path  and  orientation vectors (Narvekar, et al. 1990 and Narvekar and Oliver 1990).

Some of the works that have addressed NC verification without direct use of swept volume methods include the  early  work  of  Voelker  and  Hunt  (1985),  Menon  and  Voelcker  (1992),  Oliver  and  Goodman  (1990), Takata,et al. (1992), Jerard and Drysdale (1988), Koren and Lin (1995), Menon and Robinson (1993), Oliver and Goodman (1990) and Oliver (1990), Ge (1996), Liang, et al. (1997), Liu and Esterling (1997), and Wong and Wong (1998).  Liu, et al. (1996) presented an exact geometric model of the volume swept by a tool along a given path.  A swept volume-based milling process simulation system was developed for 3-axis milling of complex  parts  (El  Mounayri,  et  al.  1998).    For  applications  to  a  toroidal  surface,  Chung,  et  al.  (1998) presented  a  procedure  for  characterizing  the  cutter-swept  surface  (CSS)  of  a  generalized  tool  in  a  single valued form, z = f (x, y).

A  representation  of  swept  volumes  based  on  the  topological  properties  of n -dimensional  manifolds  was presented  by  Boussac  and  Crosnier  (1996).  The  algorithm  developed  for  the  evaluation  of  the  boundary exploits these properties in such a way that the n -dimensional object  (swept volume) is generated only from the ( n -1)-dimensional components resulting from a family of deformations.

More recently, Abdel-Malek, et al. (2000b) modeled the  motion of a cutter tool as a surface undergoing a sweep operation along another geometric entity.  A topological space describing the swept volume is built as a stratified space with corners.  It was shown that varieties appearing  inside the  manifold representing the removed  material  are  due  to  a  lower  degree  strata  of  the  Jacobian.    Some  of  the  varieties  addressed  are complicated but can be shown to be reducible because of their parametrization.  Figure 9 illustrates material removal.  Electric  discharge  machining  has  become  one  of  the  most  accurate  machining  used  for  the production of complex manufactured parts.

Fig. 9 (a) Tool and workpiece         (b) Material removed (swept volume)

<!-- image -->

The SDE approach was also applied to 5-axis milling in Leu et  al .  (1995).    Blackmore et  al .  (1994)  later generalized  the  SDE  method  to  deforming  swept  volumes  and  applied  this  to  the  analysis  of  deforming cutting tools in 3-axis NC machining in Leu et al . (1998) and the 5-axis machining of sculptured surfaces in Leu et  al .  (1999).  In  both  of  these  applications  the  deformed  swept  volume  analysis  accounts  for  cutter deflections and how they affect the resulting objects created by NC machining of simple workpieces.

## 4.2  Robot Analysis

Swept volumes have been shown to effectively depict the workspace (sometimes called the accessible output set)  of  serial  robot  manipulators.    These  studies  are  important  for  understanding  robot  functionality, dexterity, and their location with respect to targets in a working environment, more importantly their design. Figure  10b,  for  example,  is  the  workspace  of  the  4  degree  of  freedom  (4  parameters)  shown  in  Fig.  10a. Only a cross section  of the  workspace  for  this  4DOF  manipulator  was  determined  by  Haug, et  al. (1996) using a numerical approach (Fig. 10c).  It is evident that the swept volume formulation provides a broadly applicable  solution  to  robotic  analysis  and  allows  for  the  determination  of  the  exact  boundary  and visualization of the volume.

(a) A 4DOF RPRP manipulator (b) The Workspace  (c) A cross-sectional plane Fig. 10 Application of swept volumes to robot analysis

<!-- image -->

Similarly, real-time motion planning for robots in flexible manufacturing systems (FMS) was addressed by Li and Ma (1996) and Li (1995).  Swept volumes were also used in combination with a temporal interval search technique, to compute a series of viewpoints for monitoring a simulated robot operation (Abrams and Allen 1993; 1995).  Early works using numerical techniques were presented by Kumar and Waldron (1981) and  Kumar  (1985)  via  tracing  boundary  surfaces  of  a  robot  workspace.    Tsai  and  Soni  (1981)  studied accessible regions of planar manipulators, while Gupta and Roth (1982) and Gupta (1986) studied the effect of hand size on workspace analysis.  Other studies on the subject of manipulator workspaces were reported by  Sugimoto  and  Duffy  (1982),  and  Davidson  and  Hunt  (1987),  Agrawal  (1990),  Gosselin  and  Angeles (1990),    Emiris  (1993),  and  Yeh  (1996).    Zhang,  et  al.  (1996)  presented  a  graphical  representation  of kinematic workspaces. While all these cited references have addressed the sweep of the manipulator's endeffector, they have been limited to a small number of DOF and are apparently unable to obtain exact surface patches on the boundary. Pennock and Kassner (1993) presented a numerical algorithm for the  study  of  a planar three degree-of-freedom manipulator.

Blackmore et al . (1992b; 1992c) also used the SDE approach to classify swept volumes of robot links in two and three dimensions. Equations for general sweeps are derived with the use of  homogeneous  matrices for representing  position  and  orientation  of  an  object.  The  corresponding  sweep  differential  equations  are obtained  from  the  general  sweeps.  The  tangency  condition  is  used  to  classify  the  swept  volume  of  a  link element as one of type I or type II.

Numerical  criteria  were  formulated  to  find  the  workspace  (called  the  accessible  output  set)  of  a  general multi-degree-of-freedom system using continuation methods to trace boundary curves suitable for the study of both open- and closed-loop manipulators (Haug et al. 1996 and  Luh et al. 1996).  The initial criteria for this computational method were presented by Haug et al. (1992) and Wang and Wu (1993). A comparison between the exact method of swept volume computation presented by Abdel-Malek and Yeh (1997a) and the numerical technique used in the above works was jointly presented in a comparative paper by Abdel-Malek, et al. (1997).

Singular manifolds (i.e., geometric entities inside the swept volume) in joint space were addressed by Pai and Leu (1992), Burdick (1991; 1992), and Chevallereau and Daya (1994) and Chevallereau (1996).  Concepts of crossable  and  noncrossable surfaces  inside  a  manipulator's  workspace  were  first  addressed  by  Oblak  and Kohli (1988).

Exact  determination  and  visualization  of  the  workspace  using  Jacobian  rank  deficiency  methods  was presented  by  Abdel-Malek,  et  al.  (1997a).    Because  of  the  ability  to  parameterize  surface  patches  on  the boundary, it was shown that path trajectory verification could be performed (Abdel-Malek and Yeh 1997c) and placement of the manipulator in its environment to obtain maximum dexterity (Abdel-Malek 1996). The workspace of the manipulator shown in Fig. 11 also has voids (regions inside the workspace that cannot be reached).  More recently, control difficulties of the manipulator's end-effector were also addressed (AbdelMalek and Yeh 2000).

Fig. 11 (a) A four DOF robot  (b) Singular surfaces of the workspace (swept volume)

<!-- image -->

Cecarelli (1995) used an algebraic formulation  of a  workspace  boundary  to  formulate  design  equations  of three-revolute (3R) jointed manipulators and 4R manipulators (Cecarelli and Vinciquerra 1995) and was also treated by Abdel-Malek, et al. (1999) using the Jacobian rank deficiency method (shown in Fig. 12).  All the specified constant angles for this example are p 3 . Only the cross-section was obtained by the first  authors using a geometric method for toroidal sweeping.

Fig. 12 (a) Workspace of the general 4-DOF manipulator (b) Cross-section at z = 0.8

<!-- image -->

## 4.3  Solid Modeling and the Trivariate Freeform Solid

The representation of complex solids using sweep methods has been limited by the difficult mathematics and computational  complexity  in  high-fidelity  realistic  algorithms.  Although  some  commercial  CAD  systems offer very limited sweep operations, this type of solid modeling  is far from being used  on a realistic basis because of the delicate mathematics associated with it. Figure 13 shows a self-intersecting model of a sphere moving along a figure 8 surface depicted using the Jacobian rank deficiency method (Abdel-Malek and Yeh 1997a).

Fig. 13 Solid modeling obtained by sweeping a spherical surface along an 8-shaped surface

<!-- image -->

The trivariate tensor-product B-spline solid (Casale and Stanton, 1985, Farouki and Hinds, 1985, and Lasser, 1985) is a direct extension of the B-spline patch and has been shown to be useful in the creation and visualization of free-form geometric solids.  The trivariate B-spline solid is defined by a set of ( ) ( ) ( ) 1 1 1 3 2 1 + · + · + n n n control points } ,0 ,0 :0 { 3 3 2 2 1 1 3 , , 2 1 n i n i n i i i i £ £ £ £ £ £ p , and three sets of knots

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

or ], , [ 1 1 1 1 --˛ n m u u u ], , [ 1 1 2 2 --˛ n m v v v and ] , [ 1 1 3 --˛ e n m w w w .    The  products 3 3 2 2 1 1 , , , i m i m i m N N N are  the trivariate  tensor-product  B-spline  normalized  blending  functions  defined  by  the  knot  sequences,  and , 1 m , 2 m 3 m are  the  orders  of  the  spline  in  each  of  the  parametric  variables.    A  simple  B-spline  solid  is shown in Fig. 14.

Fig. 14: A cubic trivariate Bézier solid.  The solid is defined by 64 control points, shown in blue.  The boundary faces are the images of the boundaries of the domain interval.

<!-- image -->

The trivariate B-spline solid can be considered to be a swept solid, where the generator is a bivariate B-spline patch that continuously changes when being swept along a curve.    Here,  the  B-spline  patch  is  defined  by taking any two of the three parameter values of the trivariate solid, and the curve can be defined by the third coordinate.

The B-spline solids have two important features regarding their computation.  First, each solid can be refined or subdivided by a trivariate adaptation of the B-spline subdivision algorithm (Boehm, 1980).  In the case of the  B-spline  solid,  it  can  be  subdivided  in  any  parameter  direction,  creating  two  trivariate  B-spline  solids whose  union  is  the  original  one.  This  property  allows  the  researcher  to  develop  'divide-and-conquer' algorithms to be used for computation on the solids. For example, a B-spline solid may be refined/subdivided into a set of Bézier solids (B-splines with simplified knot sequences) whose union is the original  solid  (see  Farin,  1993).    Bounds  on  the  partial  derivatives  can  be  obtained  using  the  cone approximations  of  Sederberg  and  Parry  (1988), Sederberg  and  Meyers  (1988),  Kim  and  Moon  (1990),  or Kim, et al. (1993).

Visualization of trivariate B-spline solids was first demonstrated by Joy (1991), Reus, et al. (1992), and Joy and Duchaineau (1999).   Visualizing these solid objects requires the determination of the boundary surface of  the  solid,  which  is  a  combination  of  parametric  and  implicit  surfaces.      The  parametric  portion  of  the boundary is defined as a direct image of the boundary of the domain space, and the implicit portion of the boundary is defined  when the Jacobian determinant  of the  defining function is zero.   Joy  and  Duchaineau (1999)  have  utilized  the  Sederberg-Meyers  cone  approximations  to  the  partial  derivatives  of  the  trivariate spline function to establish an interval bound on the Jacobian determinant for a given rectangular cell of the domain space.  If zero is not in the interval bound for a given cell, the implicit boundary does not exist in the cell  and  the  trivariate  solid  can  be  accurately  represented  by  the  images  of  the  bounding  faces  of  the  cell. Alternatively, if zero is in the interval bound, the cell is subdivided.  Isosurface  methods (Lorensen, 1987, Wyvill and McPheeters, 1986, and Shu, 1995) are used to create the  surface  in  a  cell  where  the  Jacobian determinant vanishes.

Fig. 15: A trivariate solid approximating a sweep.  The picture on the left shows the model with only the boundary face patches, where the picture on the right shows the complete solid.

<!-- image -->

Joy (1992) and Joy and Madrigal (1999) have utilized two methods to approximate the envelope surface of a swept trivariate B-spline solid. Joy and Madrigal  (1999)  utilize  envelope  theory  to  approximate  the  swept surface.  At a set of points along the path, a characteristic curve was calculated on the envelope, 'zippering' these  curves  to  create  a  polygonal  representation  of  the  swept  solid.    Figure  16  (a)  illustrates  a  complex trivariate B-spline generator, and Figs. 16 (b) and (c)  illustrate the results of sweeping this generator along two different paths.

Fig. 16  (a) A trivariate B-spline solid (b) The swept solid defined by sweeping the generator along a spiral curve (c) Sweeping the generator along a spiral curve (with rotations)

<!-- image -->

## 4.4 Human Factors (Ergonomics)

Significant attention in recent years has been given to obtaining a better understanding of human joint ranges, measurement,  and  functionality,  especially  in  conjunction  with  commands  issued  by  the  central  nervous system, this research plays a useful role in ergonomic  design processes. It has been reported that impaired arms exhibit  well-defined  workspace  deficits  as  measured  by  the  assisted  rehabilitation  and  measurement (ARM) Guide (Reinkensmeyer, et al. 1999).  Researchers have used  rigorous  kinematic  formulations  and swept  volume  technology  to  model  human  limbs,  understand  their  workspace,  delineate  barriers  therein where  a  trajectory  becomes  difficult  to  control,  and  help  visualize  these  barriers.    For  example,  consider modeling the motion of the human arm kinematically as shown in Fig. 17a.  The resulting workspace (given appropriate ranges of motion) is shown in Fig. 17b imposed  on the human arm showing the torso (AbdelMalek, et al. 2000c).  The workspace of the index finger, for example, shown in Fig. 18 below is used in the layout design of buttons, levers, and hand tools.

Fig. 17 (a) Joints of the shoulder and elbow are considered (b) Workspace of the hand with respect to the shoulder

<!-- image -->

1

<!-- image -->

1

Fig. 18  The workspace of the index finger used in the design of hand tools

## Other Applications

In many applications, the motion of objects must be designed such that collision does not occur.  Techniques for  interference  detection  that  are  based  on  2-D  projections  of  swept  volumes  were  reported  in  the  early 1990's (Kieffer and Litvin 1990; 1991).  Algorithms for the determination of part-removal paths, creation of swept volumes, and representation of large models for interactive visualization were reported by Law, et al. (1998) and were integrated into one system called Galileo.  The system is used for maintainability analysis. Xavier (1997) presented an efficient method for computing the exact distance and interference detection for translationally-swept  objects.  The  method  was  subsequently  adapted  for  rotationally  swept  objects  and reported accurate collision detection.   Swept volume technology was also used in Penstroke modeling (Kim, et al. 1993).

## Challenges

While the fundamental mathematical foundation for treating swept volumes seems to converge onto rigorous mathematics adapted from Singularity Theory, it is evident that the level of complexity is high. As a result, the main stream of researchers addressing problems pertaining to swept volumes, even in the diverse fields discussed  above,  has  been  limited  by  the  difficulty  in  implementing  such  algorithms  in  computer  code. Although some commercial mechanical computer aided design codes have limited capability for sweeping, usually  in  the  form  of  extrude  or  revolve,  it  will  be  some  time  before  a  general  sweep  operation  will  be available.  By general sweep, we mean the ability to repeatedly sweep an object in space to define a complex swept solid.  Moreover, while concrete steps have been taken towards the visualization of multivariate solids, substantial effort must be focused on simplifying the  mathematical complexity so as to facilitate computer implementation.

## 5.0  Conclusions

The state-of-the art in the analysis of swept volumes has been presented with emphasis on applications and a review of previous work. Modeling and analysis methods were explored to some extent but were limited to the theoretical foundation rather than the implementation scheme.  Challenging applications in diverse fields of manufacturing automation, robotics, solid modeling, and human motion simulation were addressed.  It is evident that the topic of swept volumes remains a challenging subject from the standpoint of obtaining faster implementations, better visualization, and  more accurate approximations of their boundaries. Nevertheless, there  are  now  solid  foundations  based  upon  rigorous  mathematics  that  have  and  will  continue  to  foster expansion of the breadth of swept volume analysis.

## 6.0  References

- 1 Abdel-Malek, K. and Yeh, H.J., 1996 'Determining Intersection Curves Between Surfaces of Two Solids,' Computer-Aided Design , Vol. 28, No. 6-7, pp. 539-549.
- 2 Abdel-Malek, K. and Yeh, H. J., 1997a, "On the Determination of Starting Points for Parametric Surface Intersections," Computer-Aided Design , Vol. 29, No. 1, pp.21-35.
- 3 Abdel-Malek,  K. and  Yeh,  H.J., 1997b,  "Geometric  Representation  of  the  Swept  Volume  Using  Jacobian  Rank Deficiency Conditions', Computer Aided Design , Vol. 29, No. 6, pp. 457-468.
- 4 Abdel-Malek, K. and Yeh, H.J., 1997c, "Path Trajectory Verification for Robot Manipulators in a Manufacturing Environment" IMechE Journal of Engineering Manufacture , Vol. 211, pp. 547-556.
- 5 Abdel-Malek, K. and Yeh, H.J., 1997d, 'Analytical Boundary of the Workspace for General 3-DOF mechanisms,' International Journal of Robotics Research . Vol. 16, No. 2, pp. 1-12.
- 6 Abdel-Malek, K., Adkins, F., Yeh, H.J., and Haug, E.J 1997, "On the Determination of Boundaries to Manipulator Workspaces," Robotics and Computer-Integrated Manufacturing , Vol. 13, No. 1, pp.63-72.
- 7 Abdel-Malek,  K.,  Yeh,  H.J.,  and  Othman,  S.,  1998,  "  Swept  Volumes:  Void  and  Boundary  Identification," Computer Aided Design , Vol. 30, No. 13, pp. 1009-1018.
- 8 Abdel-Malek,  K.,  Yeh,  H.J.,  and  Khairallah,  N.,  1999a,  "Workspace,  Void,  and  Volume  Determination  of  the General 5DOF Manipulator, Mechanics of Structures and Machines , Vol. 27, No. 1, 91-117.
- 9 Abdel-Malek,  K.,  Seaman,  W.,  and  Yeh,  H.J.,  1999b,  'An  exact  method  for  NC  verification  of  up  to  5-axis machining', Proceedings of the 25 th ASME Design Automation Conference , Las Vegas, Nevada.
- 10 Abdel-Malek, K. and Othman, S. 1999, "Multiple sweeping using the Denavit-Hartenberg Representation Method", Computer-Aided Design , Vol. 31, pp. 567-583.
- 11 Abdel-Malek,  K.,  Yang,  J.,  and  Blackmore,  D.,  2000a,  'Closed-form  swept  volume  of  implicit  surfaces'', Proceedings of the 26 th ASME Design Automation Conference , Baltimore, MD.
- 12 Abdel-Malek,  K.,  Seaman,  W.,  and  Yeh,  H.J.,  (2000b-in  press)  'NC  Verification  of  up  to  5  Axis  Machining Processes Using Manifold Stratification' ASME Journal of Manufacturing Science and Engineering .
- 13 Abdel-Malek, K., Yang, J., Brand, R., Vannier, M., and Tanbour, E. (2000c-submitted), 'Towards Understanding the Workspace of Human Limbs', International Journal of Ergonomics .
- 14 Abdel-Malek, K. and Yang, J., 2000, 'Method and Code for the Visualization of Multivariate Solids', Proceedings of the 26 th ASME Design Automation Conference , Baltimore, MD.

- 15 Abrams, S.; Allen, P.K., 1993, 'Dynamic sensor planning', Proceedings of the IEEE International Conference on Robotics and Automation , May 2-6, v 2, Atlanta, GA, pp. 605-610.
- 16 Abrams,  S.;  Allen,  P.K.,  1995,  'Swept  volumes  and  their  use  in  viewpoint  computation  in  robot  work-cells', Proceedings of the IEEE International Symposium on Assembly and Task Planning , Pittsburgh, PA, pp. 188-193.
- 17 Agrawal, S.K. 1990, 'Workspace Boundaries of In-parallel Manipulator Systems,' Int. J. of Robotics and Automation , Vol. 7, No. 2, pp. 94-99.
- 18 Ahn,  JC,  Kim,  MS,  Lim,  SB,  1997,  "Approximate  general  sweep  boundary  of  2D  curved  object", CVGIP: Computer Vision Graphics and Image Processing , Vol. 55, pp. 98-128.
- 19 Akin, J. E., 1990, Computer-Assisted Mechanical Design , Prentice Hall, Englewod Cliffs, NJ.
- 20 Al-Daccak,  M.J.;  Angeles,  J.,  1989,  'Reduced  calculation  of  volumetric  properties  of  sweep-generated  solids', ASME (Publication) DE Computer-Aided and Computational Design  Advances in Design Automation - v 19 pt 1, pp. 157-162.
- 21 Bajaj, C.L., Hoffman, C.M., Lynch, R.E., and Hopcroft, J.E.H., 1988, 'Tracing Surface Intersections,' Computer Aided Geometric Design , Vol. 5, pp. 285-307.
- 22 Barnhill,  R.E.  and  Kersey, S.N.,  1989,  'A  Marching  Method  for  Parametric  Surface/Surface  Intersection,' Computer Aided Geometric Design , Vol. 7, pp. 257-280.
- 23 Barnhill,  R.E.,  Farin,  G.,  Jordan,  M.,  and  Piper,  B.R.,  1987,  'Surface/surface  Intersection,' Computer  Aided Geometric Design , Vol. 4, pp. 3-16.
- 24 Bartels, R., Beatty, J., and Barsky, B., 1987, An Introduction to Splines for Use in Computer Graphics and Geometric Modeling , Morgan Kaufmann Publishers.
- 25 Blackmore, D.; Leu, M. C., 1990, 'A differential equation approach to swept volumes', Proceedings of Rensselaer's 2nd International Conference on Computer Integrated Manufacturing, pp. 143-149.
- 26 Blackmore,  D.;  Leu,  M.C.,  1992a,  'Analysis  of  swept  volume  via  Lie  groups  and  differential  equations', International Journal of Robotics Research , v 11 n 6, pp. 516-537.
- 27 Blackmore, D.; Leu,  M.C.;  Wang,  K.K.,  1992b,  'Application  of  flows  and  envelopes  to  NC  machining', CIRP Annals , v 41 n 1, pp. 493-496.
- 28 Blackmore, D.; Leu, M.C.; Wang, W., 1992c, 'Classification and analysis of robot swept volumes', Proceedings of the Japan - USA Symposium on Flexible Automation Part 1 (of 2), San Francisco, CA, pp. 69-75.
- 29 Blackmore, D.; Leu, M.C.; Shih, F., 1994, 'Analysis and modeling of deformed swept volumes', Computer Aided Design , v 26 n 4, pp. 315-326.
- 30 Blackmore,  D.,  Leu,  M.C.,  Wang,  L.P.  and  Jiang,  H.,  1997a,  'Swept  volumes:  a  retrospective  and  prospective view', Neural, Parallel &amp; Scientific Computations , Vol. 5, 81-102.
- 31 Blackmore, D.; Samulyak, R.; Leu, M.C., 1999, 'Trimming swept volumes', Computer Aided Design , v 31 n 3, pp. 215-223.
- 32 Blackmore, D.; Leu, M.C.; Wang, L.P., 1997b, 'Sweep-envelope differential equation algorithm and its application to NC machining verification', Computer Aided Design , v 29 n 9, pp. 629-637.
- 33 Blackmore, D., Samulyak, R. and Leu, M.C., 2000, 'Singularity theory approach to swept volumes', Int. J. Shape Modeling, (in press).
- 34 Boehm, W., "Inserting New Knots Into B-Spline Curves", Computer-Aided Design, Vol. 12, July 1980, 199-201.
- 35 Boltianskii, V.G., Envelopes , Macmillan Publishing, New York, New York, 1964.
- 36 Boussac,  Stephane;  Crosnier,  A.,  1996,  'Swept  volumes  generated  from  deformable  objects  application  to  NC verification', Proceedings of the 1996 13th IEEE International Conference on Robotics and Automation . Part 2 (of 4) v2, Minneapolis, MN, pp. 1813-1818.
- 37 Burdick, JW, 1991, 'A Classification of 3R regional manipulator singularities and geometries' Proceedings of the IEEE Int. Conference on Robotics and Automation , Sacramento, CA, pp. 2670-2675.
- 38 Burdick, JW, 1992, 'A recursive method for finding revolute-jointed manipulator singularities', Proceedings of the IEEE International Conference on Robotics and Automation , Nice, France, pp. 448-453.
- 39 Casale, M.S., and Stanton, E.L., 1985, "An Overview of Analytic Solid Modeling", IEEE Computer Graphics and Applications , Vol 5, No. 2, pp. 45-56.
- 40 Ceccarelli, M. and Vinciguerra, A., 1995, 'On the Workspace of General 4R Manipulators,' International Journal of Robotics Research , Vol. 14, No. 2, pp. 152-160.
- 41 Ceccarelli, M., 1995, 'A Synthesis Algorithm for Three-Revolute Manipulators by Using an Algebraic Formulation of Workspace Boundary,' ASME J. of Mechanical Design , 117(2A), pp. 298-302.
- 42 Chevallereau, C. and Daya, B., 1994, 'A new method for robot control in singular configurations with motion in any Cartesian direction', Proceedings of the IEEE International Conference on Robotics and Automation ,  Vol. 4, San Diego, CA, pp. 2692-2697.

- 43 Chevallereau, C., 1996, 'Feasible trajectories for a non redundant robot at a singularity' Proc. of IEEE International Conference on Robotics and Automation, Minneapolis, MN, pp. 1871-1876.
- 44 Chou, C.J., and lee, Y.S., 1999, 'A shape generating approach for multi-axis machining G-buffer models,' Computer Aided Design, Vol. 31, pp. 761-776.
- 45 Chung, Y.C.; Park, J.W.; Shin, H.; Choi, B.K., 1998, 'Modeling the surface swept by a generalized cutter for NC verification', Computer Aided Design , Vol. 30, No.8, pp. 587-94.
- 46 Dahlberg, B., and Johansson, B., 1987, "Envelope curves and surfaces", The Mathematics of Surfaces II , University Press, 419-426.
- 47 Denavit, J., and Hartenberg, R.S., 1955, 'A Kinematic Notation for Lower-pair Mechanisms Based on Matrices,' Journal of Applied Mechanics, ASME , Vol. 22, pp. 215-221.
- 48 Davidson, JK and Hunt, 1987, 'Robot Workspace of a Tool Plane: Part I - a ruled surface and other geometry', Journal of Mechanisms, Transmissions, and Automation in Design ,  Vol. 109, pp. 50-60.
- 49 El  Mounayri,  H.;  Spence,  A.D.;  Elbestawi,  M.A.,  1998,  'Milling  process  simulation  -  a  generic  solid  modeller based paradigm', Journal of Manufacturing Science and Engineering , Transactions of the ASME, v 120 n 2, pp. 213-221.
- 50 Elber,  G,  1997,  'Global  error  bounds  and  amelioration  of  sweep  surfaces,' Computer  Aided  Design ,  Vol.  29, pp.441-447.
- 51 Elber, G. and Kim, M.S., 1999, 'Offsets, sweeps, and Minkowski sums ', Computer-aided Design , Vol. 31 (3),  pp. 163 .
- 52 Emiris, D.M., 1993, 'Workspace Analysis of Realistic Elbow and Dual-elbow Robot,' Mechanisms and Machine Theory , Vol. 28, No. 3, pp. 375-396.
- 53 Farin, G., Curves and Surfaces for Computer Aided Geometric Design , Academic Press, 1997.
- 54 Flaquer, J., Gárate, G., and Parggada, M., "Envelopes of Moving Quadric Surfaces", Computer Aided Geometric Design , Vol. 9, No. 4, 299-312.
- 55 Ganter, M A, Storti, D W, and Ensz, M T, 1993, 'On Algebraic Methods for Implicit Swept Solids with Finite Extent' ASME DE Vol 65(2), pp. 389-396.
- 56 Ganter,  M.,  Uiker,  J.,  1986,  'Dynamic  Collision  Detection  using  Swept  Solids,' ASME Journal of  Mechanism, Transmission, and Automation in Design , Vol. 108, pp. 549-555.
- 57 Ge, Q.J.,  1996,  'Kinematics-driven  geometric  modeling: a  framework for simultaneous NC  tool-path  generation and  sculptured  surface  design', Proceedings  of  the  1996  13th  IEEE  International  Conference  on  Robotics  and Automation . Part 2(2) Minneapolis, MN, pp. 1819-1824.
- 58 Gosselin, C.  and  Angeles,  J.,  1990,  'Singularity  Analysis  of  Closed  Loop  Kinematic  Chains,' IEEE  Trans.  on Robotics and Automation , Vol. 6, No. 3, pp. 281-290.
- 59 Guillemin, V. and Pollack, A., 1974, Differential Topology , Prentice-Hall.
- 60 Gupta, K.C, 1986, 'On the Nature of Robot Workspace,' Int. J. Rob. Res., 5(2) 112-121 (1986)
- 61 Gupta,  K.G.  and  Roth,  B.  1982,  'Design  Considerations  for  Manipulator  Workspace,'  ASME Journal  of Mechanical Design , Vol. 104, No. 4, pp. 704-711.
- 62 Hartquist, E.E. Menon, J.P. Suresh, K. Voelcker, H.B. and Zagajac, J., 1999, 'A computing strategy for applications involving offsets, sweeps, and Minkowski operations', Computer-aided Design , Vol. 31(3), pp. 175183.
- 63 Haug,  E.J.,  Luh,  C.M.,  Adkins,  F.A.,  and  Wang,  J.Y.,  1996,  'Numerical  algorithms  for  mapping  boundaries  of manipulator workspaces,' ASME J. Mech. Design , 118, pp. 228-234.
- 64 Haug, E.J., Wang, J.Y., and Wu, J.K., 1992, 'Dextrous Workspaces of Manipulators: I. Analytical Criteria,' Mech. Struct. and Mach., (E.J. Haug, ed.), Vol. 20, No. 3, pp. 321-361.
- 65 Hoffman, C M, 1989, Geometric and Solid Modeling, An Introduction , Morgan Kaufman, Ic., San Mateo, CA, 1989.
- 66 Hu, Z.; Ling, Z.K., 1995, 'Geometric modeling of a moving object with self-intersection', Proceedings of the 1995 ASME Design Engineering Technical Conferences, v 82 n 1, Boston, MA, pp. 141-148.
- 67 Hu, Z.J.; Ling, Z.K., 1994, 'Generating swept volumes with instantaneous screw axes', Proceedings of the 1994 ASME Design Technical Conferences . Part 1 (of 3), v 70 n pt 1, Minneapolis, MN, pp. 7-14.
- 68 Hu, Z.J.; Ling, Z.K., 1996, 'Swept volumes generated by the natural quadric surfaces', Computers &amp; Graphics, v20 n2, pp. 263-274.
- 69 Jakubowski,  R.,  1993,  'Multiple  sweeping  in  product  modeling  and  analysis', Proceedings  of  the  1993  ASME Winter Annual Meeting , v 66, New Orleans, LA, pp. 125-133.
- 70 Jerard,  R  and  Drysdale,  R,  1988  'Geometric  Simulation  of  Numerical  Control  Machinery' ASME  Computers Engineer Vol 2, pp129-136.

- 71 Joy, K.I., 1991, "Utilizing parametric hyperpatch methods for modeling and display of free-form solids", International Journal Computational Geometry and Applications , Vol. 1, No. 4, 1991, 455-471.
- 72 Joy, K.I., 1992, "Visualization of Swept Hyperpatch Solids", in Visual Computing: Integrating Computer Graphics with Computer Vision (Proceedings of Computer Graphics International '92), T. L. Kunii, ed., Springer-Verlag, Tokyo, pp. 567-582.
- 73 Joy, K.I. and Duchaineau, M.A., 1999, "Boundary Determination for Trivariate Solids", Proceedings of the 1999 Pacific Graphics Confernece, Seoul, Korea.
- 74 Jüttler, B., and Wagner M.G., 1999, 'Rational motion-based surface generation', Computer-Aided Design , Vol. 31 (3),  pp. 203-213.
- 75 Kieffer,  J.;  Litvin,  F.L., 1991,  'Swept  volume  determination  and  interference  detection  for  moving  3-D  solids', Journal of Mechanical Design , v 113 n 4, pp. 456-463.
- 76 Kieffer,  J.;  Litvin,  F.L., 1990,  'Swept  volume  determination  and  interference  detection  for  moving  3-D  solids', ASME (Publication) DE, v 24, pp. 309-318.
- 77 Kim, D.-S., 1990, Cones on Bézier Curves and Surfaces , Ph.D. Thesis, Department of Industrial and Operations Engineering, University of Michigan, Ann Arbor, Michigan.
- 78 Kim, D.-S., Papalambros, P.Y. and Woo, T.C., 1995, "Tangent, Normal, and Visibility Cones for Bézier Surfaces", Computer Aided Geometric Design , Vol. 12, pp. 305-320.
- 79 Kim, M-S, Ahn, J-W, and Lim, S-B, 1993, 'An Algebraic Algorithm to Compute the Exact General Sweep Boundary of a 2D Curved Object', Information Processing Letters , 47, pp. 221-229.
- 80 Kim, M.S.; Moon, S.R., 1990, 'Rotational sweep volumes of objects bounded by algebraic curves', Proceedings of the 1990 IEEE International Conference on Robotics and Automation , pp. 311-316.
- 81 Klok, F, 1986, 'Two Moving Coordinate Frames for Sweeping Along a 3D Trajectory', Computer-Aided Geometric Design , Vol 3, pp. 217-229.
- 82 Koren, Y. and Lin, R.S., 1995, 'Five-axis surface interpolators', Annals of CIRP , 44(1), 379-382.
- 83 Kumar, A. and Waldron, K.J., 1981, 'The Workspace of a Mechanical Manipulator,' ASME J. Mech. Design , Vol. 103, pp. 665-672.
- 84 Kumar,  V.,  1985, Robot  Manipulators-Workspaces  and  Geometric  Dexterity ,  Masters  Thesis,  The  Ohio  State University.
- 85 Lasser, D., 1985, "Bernstein-Bézier Representation of Volumes", Computer Aided Geometric Design, Vol. 2, pp. 145-149 .
- 86 Law,  C.C.;  Avila,  Sobierajski;L.,  Schroeder,  W.  1998,  'Application  of  path  planning  and  visualization  for industrial-design and maintainability-analysis', Proceedings of the 1998 Reliability and Maintainability Symposium ,  Anaheim, CA, pp. 126-131.
- 87 Leu, M.C., Blackmore, D. and Maiteh, B., 1999, 'Deformed swept volume analysis of NC machining with cutter deflection', Machining Impossible Shapes , B.K. Choi and R. Jerard, eds., Kluwer, Boston, 158-166.
- 88 Leu, M.C.; Blackmore, D.; Wang, L.P.; Pak, K.G., 1995, 'Implementation of SDE method to represent cutter swept volumes  in  5-axis  NC  milling', Proceedings  of  SPIE  -  The  International  Society  for  Optical  Engineering  Int. Conference on Intelligent manufacturing , v 2620 Wuhan, China, pp. 354-364.
- 89 Leu, M C.; Lu, F; Blackmore, D, 1998, 'Simulation of NC machining with cutter deflection by modelling deformed swept volumes', CIRP Annals - Manufacturing Technology , v 47 n 1, pp. 441-446.
- 90 Leu, M C.;  Wang,  L;  Blackmore,  D,  'Verification  program  for  5-axis  NC  machining  with  general  APT  tools', CIRP Annals - Manufacturing Technology, v 46 n 1 1997, pp. 419-424.
- 91 Li, W., 'Real-time motion planning for robot application in flexible manufacturing systems', Proceedings of SPIE - The International Society for Optical Engineering Int. Conference on Intelligent Manufacturing ,  Jun  10  1995 v 2620, Wuhan, China, pp. 620-625.
- 92 Li, W.; Ma, C., 1996, 'Robot motion planning in flexible manufacturing systems', High Technology Letters , v2(2), pp. 1-4.
- 93 Liang, X, Xiao, T, Han, X, Ruan, JX, 1997, Simulation Software GNCV of NC Verification, Author Affiliation: ICIPS Proceedings of the 1997 IEEE International Conference on Intelligent Processing Systems , Part 2 Oct 28-31, Vol. 2, Beijing, China pp. 1852-1856.
- 94 Ling, ZK and Chase, T 1996, 'Generating the swept area of a body undergoing planar motion,' ASME J. Mech. Design , Vol 118, pp221-233.
- 95 Liu, C, and Esterling, D, 1997, 'Solid Modeling of 4-Axis Wire EDM Cut Geometry', Computer Aided Design Vol. 29, No. 12, pp. 803-810
- 96 Liu, C., Esterling, D.M., Fontdecaba, J., Mosel, E, 1996, 'Dimensional verification of NC machining profiles using extended quadtrees', Computer Aided Design Vol. 28, No. 11, pp. 845-852.

- 97 Lorensen, W.E., and Cline, H.E., 1987, "Marching Cubes: {A} High Resolution {3D} Surface Construction Algorithm", Computer Graphics, Vol. 21, No. 4, 163-169.
- 98 Lu, Y.C., 1976, 'Singularity Theory and an Introduction to Catastrophe Theory', Springer-Verlag, New York.
- 99 Luh, C. M. Adkins, F. A. Haug, E. J. and Qiu, C.C., 1996. Working Capability of Stewart Platforms,' ASME J. Mech. Design , Vol. 118, pp. 220-227.
- 100 Madrigal, C. and Joy, KI, 1999, "Generating the Envelope of a Swept Trivariate Solid," Proceedings of the IASTED International Conference on Computer Graphics and Imaging , Palm Springs, California, October 25-27.
- 101 Maekawa, T., 1999, 'An overview of offset curves and surfaces', Computer-aided Design , Vol. 31 (3), pp. 165173.
- 102 Martin, R.R.; Stephenson, P.C., 1990, 'Sweeping of three-dimensional objects', Computer Aided Design , v 22 n 4, pp. 223-234.
- 103 Martin,  R.R.;  Stephenson,  P.C.,  1989,  'Swept  Volumes  in  Solid  Modellers', Proceedings  of  the  Third  IMA Conference on Mathematics of Surfaces , Oxford, p. 391.
- 104 Menon,  J.P.  and  Robinson,  D.M.  1993,  'Advanced  NC  verification  via  massively  parallel  raycasting,  ' ASME manufacturing Review , Vol. 6, 141-154.
- 105 Menon, J.P., Voelcker, H.B. 1992, 'Toward a comprehensive formulation of NC verification as a mathematical and computational problem', Proceedings of the 1992 Winter Annual Meeting of ASME ,  Vol.  59,  Anaheim,  CA,  pp. 147-164.
- 106 Narvekar, A.P.; Oliver, J.H., 1990, 'Intersection of vectors with general five-axis NC swept volumes', Proceedings of Manufacturing International '90. Part 4: Advances in Materials and Automation , Atlanta, GA, pp. 99-104.
- 107 Narvekar, AP, Huang, Y, and Oliver, J, 1992, 'Intersection of rays with parametric envelope surfaces representing five-axis  NC  milling  tool  swept  volumes', Proceedings  of  the  1992  18th  Annual  ASME  Design  Automation Conference , Vol. 44, Scottsdale, AZ, pp. 223-230.
- 108 Oblak, D. and Kohli, D., 1988, 'Boundary surfaces, limit surfaces, crossable and noncrossable surfaces in workspace of mechanical manipulators', ASME Journal of Mechanisms, Transmissions, and Automation in Design , Vol. 110, pp. 389-396.
- 109 Olfe, D. B., 1995, Computer Graphics for Design, From Algorithms to Autocad , Prentice Hall, Englewood Cliffs, NJ, 1995.
- 110 Oliver, J and Goodman, E, 1990, 'Direct Dimensional NC Verification' Computer Aided Design Vol. 22, pp. 3-9.
- 111 Oliver, J H, 1990, 'Efficient Intersection of Surface Normals with Milling Tool Swept Volumes for Discrete Threeaxis NC Verification', ASME DE Vol 23(1), pp. 159-164.
- 112 Pai, D. K. and Leu, M. C. 1992, 'Generecity and singularities of robot manipulators. IEEE Transactions on Robotics and Automation , Vol. RA-8, pp. 545-559.
- 113 Parida, L.; Mudur, S.P., 1994, 'Computational methods for evaluating swept object boundaries', Visual Computer , v 10 n 5, pp. 266-276.
- 114 Pennock, G.R. and Kassner, D.J., 1993, 'The Workspace of a General Planar Three-Degree-of-Freedom PlatformType Manipulator,' Journal of Mechanical Design , Vol. 115, pp. 269-276.
- 115 Peters, J. and Nasri, A., 1997, Computing Volumes of Solids Enclosed by Recursive Subdivision. Computer Graphics Forum , Vol. 16(3), pp. 89-94.
- 116 Reinkensmeyer,  DJ,  Dewald,  JP,  and  Rymer,  WZ,  1999,  'Guidance-based  quantification  of  arm  impairment following brain injury: a pilot study', IEEE Trans Rehabil Eng , 7(1):1-11.
- 117 Reus, J.F., Mish, K.D. and Joy, K.I., 1992, "Mechanical Deformations of Hyperpatch Solids", Proceedings of Compugraphics '92, 147-158.
- 118 Sambandan, K.; Wang, K.K., 1989, 'Five-axis swept volumes for graphic NC simulation and verification', ASME (Publication)  DE  Computer-Aided  and  Computational  Design  Advances  in  Design  Automation -  v  19  npt  1, Montreal, Que, Can, pp. 143-150.
- 119 Schroeder,  W.J.;  Lorensen,  W.E.;  Linthicum,  S.,  1994,  'Implicit  modeling  of  swept  surfaces  and  volumes', Proceedings of the 1994 IEEE Visualization Conference, Washington, DC, pp. 40-45.
- 120 Sealy, G.; Wyvill, G., 1997, 'Representing and rendering sweep objects using volume models', Proceedings of the 1997 International Conference on Computer Graphics , Hasselt, Belgium, pp. 22-27.
- 121 Sederberg, T.W. and Meyers, R., 1988, "Loop detection in surface patch intersections", Computer Aided Geometric Design , Vol. 5, No. 2, 161-171.
- 122 Sederberg, T.W. and Parry, S.R., 1986, "Free-Form Deformation of Solid Geometric Models", Computer Graphics (SIGGRAPH '86 Proceedings), Vol. 20, No. 3, pp. 151-160.
- 123 Seng, Y.-J.; Joshi, S.B.,  1998,  'Recognition  of  interacting rotational and  prismatic machining  features  from 3-D mill-turn parts', International Journal of Production Research , v 36 n 11, pp. 3147-3165.

- 124 Sheltami, K.; Bedi, S.; Ismail, F., 1998, 'Swept volumes of toroidal cutters using generating curves', International Journal of Machine Tools &amp; Manufacture , v 38 n 7, pp. 855-870.
- 125 Shu, R., Zhou, C., and Kankanhalli, M.S., 1995, "Adaptive marching cubes", The Visual Computer, Vol. 11, No. 4, pp. 202-217.
- 126 Sourin, A and Pasko, A, 1996, 'A function representation for sweeping by a moving solid,' IEEE Transactions on visualization and Computer Graphics , Vol 2, pp. 11-18.
- 127 Spivak, M. 1968, Calculus on Manifolds , Benjamin/Cummeings.
- 128 Sugimoto, K. and Duffy, J., 1982, 'Determination of Extreme Distances of a Robot Hand. Part 2: Robot Arms with Sppecial Geometry,' ASME J. of Mechanical Design , Vol. 104, pp. 704-712.
- 129 Takata,  S,  Tsai,  MD,  and  Inui,  M,  '1992,  'A  cutting  simulation  system  for  machinability  evaluation  using  a workpiece model,' Annals of CIRP 38:539-542.
- 130 Tsai, Y.C. and Soni, A.H. 1981, 'Accessible Region and Synthesis of Robot Arm,' ASME J. of Mech. Design , Vol. 103, pp. 803-811.
- 131 Ueng,  W.-D.;  Lai,  J.-Y.;  Doong,  J.-L.,  1998,  'Sweep-surface  reconstruction  from  three-dimensional  measured data', Computer Aided Design , v 30 n 10, pp. 791-805.
- 132 Voelker,  H  B  and  Hunt, W  A,  1985,  'The  Role  of  Solid  Modeling  in  Machining  Process  Modeling  and  NC Verification' SAE Tech. Paper #810195.
- 133 Wang, G., Sun, J. and Hua, X., 2000, 'The sweep-envelope differential equation algorithm for general deformed swept volumes', Computer Aided Geometric Design , Vol. 17, pp. 399-418.
- 134 Wang,  J.Y.  and  Wu,  J.K.  1993,  'Dexterous  Workspaces  of  Manipulators,  Part  2:  Computational  Methods,' Mechanics of Structures and Machines , Vol. 21, No. 4, pp. 471-506.
- 135 Wang, L.; Leu, M.C.; Blackmore, D., 2000, 'Generating swept solids for NC verification using the SEDE method', Fourth Symposium on Solid Modeling and Applications , pp. 364-75.
- 136 Wang, W.P. and Wang, K.K., 1986, 'Real-time verification of multi-axis NC programs with raster graphics', Proc. IEEE Int. Conf. on Robotics and Automation , pp. 166-171.
- 137 Weld, J.D.; Leu, M.C., 1990, 'Geometric representation of swept volumes with application to polyhedral objects', International Journal of Robotics Research , Vol. 9 n 5, pp. 105-117.
- 138 Wellman, B.L., 1957, Technical Descriptive Geometry , McGraw-Hill Book Company, New York.
- 139 Wong, T.N.; Wong, K.W., 1998, 'Feature-based design by volumetric machining features', International Journal of Production Research, v 36 n 10, pp. 2839-2862.
- 140 Wyvill, B., McPheeters, C., and Wyvill, G., 1986, "Animating Soft Objects", The Visual Computer, Vol. 2, No. 4, 1986, 235-242.
- 141 Xavier, P.G., 1997, 'Fast swept-volume distance for robust collision detection', Proceedings of the 1997 IEEE International Conference on Robotics and Automation, ICRA. Part 2 (of 4) v 2 Albuquerque, NM, pp. 1162-1169.
- 142 Yeh,  HJ,  1996,  Singularity  and  Workspace  Analyses  of  Serial  Robot  Manipulators,  Ph.D.  Dissertation,  The University of Iowa, Iowa City, IA.
- 143 Zhang, S.J., Sanger, D.J., and Howard, D., 1996, 'Workspaces of a Walking Machine and their Graphical Representation. Part I: kinematic workspaces,' Robotica, 14, pp. 71-79.
- 144 Zhou, Y., Chen, B., and Kaufman, A., 1997, "Multiresolution Tetrahedral Framework forRegular Volume Data", Proceedings of IEEE Visualization '97, pp. 135-142.