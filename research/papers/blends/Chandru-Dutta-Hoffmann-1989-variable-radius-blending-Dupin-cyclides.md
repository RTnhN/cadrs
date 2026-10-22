## Purdue University

## [Purdue e-Pubs](https://docs.lib.purdue.edu/)

[Department of Computer Science Technical Reports](https://docs.lib.purdue.edu/cstech)

1989

## On Variable Radius Blending Using Dupin Cyclides

Vijaya Chandru

Debasish Dutta

Christoph M. Hoffmann Purdue University, cmh@cs.purdue.edu

## Report Number:

89-851

Chandru, Vijaya; Dutta, Debasish; and Hoffmann, Christoph M., "On Variable Radius Blending Using Dupin Cyclides" (1989). Department of Computer Science Technical Reports. Paper 725. https://docs.lib.purdue.edu/cstech/725

This document has been made available through Purdue e-Pubs, a service of the Purdue University Libraries. Please contact epubs@purdue.edu for additional information.

Department of Computer Science VARIABLE RADIUS  BLENDING USING DUPIN  CYCLIDES

Vijaya Chandrn Dcbasish  Dulla Chrisloph M. Hoffmann

CSD-TR-851 January  1990

## VARIABLE RADIUS BLENDING USING DUPIN CYCLIDES

Vijaya Chandru.J Debasish Duttaf

School of Industrial Engineering Purdue University West Lafayette, IN  47907

jfl Chiistoph  M.  Hoffmann

Department of Computer Science Purdue University Technical Report CSD-TR-851 CAPO Report CER-89-3 January, 1989

$  Chandra was supported  in part by  grants from the ONR  under University  Research  Initiative grant number N00014-86-K-0689 and from NSF grand number DMC  88-07550.

t  Dutia received support from the NSF, grant number DMC  88-07550.

*  HofEmann  was  supported  in  part  by  NSF  grants  CCR-86-19817  and  DMC-88-07550  and ONR contract N000014-86-K-0465.

## VARIABLE RADIUS BLENDING USING DUPIN  CYCLIDES

Vijaya Chandru,$ Debasish  Duttaf

School of Industrial Engineering Purdue University West Lafayette, IN  47907

C.M. Hoffmann

Computer Sciences Department Purdue University CSD-TR-851 January  1989

$  Chan  dm  was  supported  in  pan  by  grants  from  the ONR  under University  Research  Initiative grant number N00014-86-K-0689  and from NSF grant number DMC  88-07550.

t  Dutia received support from NSF grant number DMC  88-07550.

Hoffmann  was  supported  in  pait  by  NSF  grants  CCR-86-19817  and  DMC-88-07550  and ONR contract N00014-86-K-0465.

## Variable Radius Blending Using Dupin  Cyclides

## Vijaya  Chandru, Debaaiah Dutta

School  of Industrial  Engineering Purdue  University,  W. Lafayette,  IN  47907

and

## Christoph M. Hoffmann

Department  of  Computer  Science Purdue  University,  W. Lafayette, IN47907

## 1.  Introduction

Mechanical  parts  can,  in  general,  be  decomposed  into  primary and  secondary  surfaces.  While  the  primary  surfaces  define  the  part profile,  the  secondary  surfaces  are  required to  smoothly connect them.  These  secondary  surfaces  are  referred  to  aa blending  surfaces or simply blends.

It  is  difficult to  find  a mechanical  part  or  an  assembly  without any  blending  surfaces.  Fillets  and  rounds  are  the  most  common forms  of  blends.  Others  include  fairings  and  sculptured surfaces. Typically,  smoothness  of  contact  is  a  constraint  that  all  blending surfaces must satisfy. If there are no other constraints, these blending surface are  easy  to derive  algebraically [Hoffmann  and Hopcroft 85, 86].

However, various engineering applications impose geometric constraints  on  the  blending  surfaces.  The  cross-sectional  profile  of most  blends  are  required  to  be  circular.  As  such,  common  practice in  engineering  drafting  is to  specify blends by  an appropriate  radius (e.g. 0.5R ).  When  circular  cross-sections  are  required,  the  blending surfaces are mathematically more complex. Such surfaces are typically of high algebraic degree and are difficult to derive analytically in closed form. As a result, surface interrogations become computation  intensive.

The circular cross-sections of fixed radius permit a straightforward  specification.  Such  blends  have  been  approximated by Rossignac and Requicha [Rossignac and Requicha 86]. An attractive feature of their approximation method is that only standard  CSG primitives,  in particular, cylinders  and tori  are used.

When  applications  require  the  circular  cross-section  to  vary  in radius  (e.g.  in  the  design  of  molds  and  dies),  the  problem  becomes more complicated.  Not only  do the standard  CSG primitives  appear to  be  insufficient  for  an  approximation  scheme,  but  even  a  precise analytical definition of variable radius blends does not exist. Consequently,  current practice defines these surfaces  ambiguously.

In this paper, we present a mathematically unambiguous procedure for defining variable radius  blends.  We  use Voronoi surfaces -a generalization of the Voronoi diagrams in computational  geometry  - for  such  a  definition.  Next,  we  examine the complexity  of  constructing  such blending  surfaces explicitly. It is seen  that,  even  in very  simple  cases  an explicit  construction  is  quite complex.  This motivates  an approximation  strategy.

When  approximating  variable  radius  blends,  cyclide  patches seem  natural  approximants  since  their  principal  lines  of  curvature are always circles. The  rich variety of geometric properties of cyclides  has  been  reviewed  in  [Chandru  et  al.  1988]. Other  authors have considered  cyclides  for the purpose  of  free-form surface  design, e.g.  [McLean  84],  (Martin  et  al.  86], However,  it  appears  that  for free-form  surface  design  the  cyclide  is  not  sufficiently  flexible,  at least not when  patching along principal lines of curvature. In contrast,  using  cyclides  for  blends  is promising  since  all  constraints are in one dimension, rather than  two.

Later,  it  is shown  in  our  case study  that,  cyclides  can  be  used, as  a  whole,  for  specific  variable  radius  blending  applications.  This fact has also been  discovered by others, including  [Pratt 88].

When  a  cyclide  cannot  be  used  in  its  entirety  to  represent  the variable radius blending surface, it remains, however, a good candidate  for  an  approximant.  In  general,  the  approximation of variable  radius  blends  is  complex.  Technically,  approximating a fixed  radius blend  requires  interpolating  a set  of  space  curve  points and  associated  tangents  by  pieces  of  circles  and  straight  lines.  For variable radius blends, the elements to be used in the  approximation are  restricted  to  be  the  conic  spines  of  the  cyclides.  By  using  a scheme  of  biarc  approximation  combined  with  Liming's  method, moreover, we increase  flexibility.

## 2.  Existing  Methods

In recent years,  a fair  amount  of  attention  has been  devoted  to blending  surfaces by  researchers  in  CAGD,  [Hoffmann  and  Hop croft 85,  86,  86a]  [Owen  and  Rockwood  86]  [Rossignac  and  Requcha  86] [Varady  et  al.  88].  However,  variable  radius  blends  have  not  been addressed nearly as  much. Pegna's  work in  modeling variable radius  blends  by  sweeps  seems  to  be  the  only  literature  available [Pegna 87].

From  fixed  radius  blending  we  will  use the concepts of a moving sphere and a spine to  define variable radius blends. For  fixed radius  blends,  the  spine  is  well  defined  as  the  intersection  of  the offsets  of  the  primary  surfaces,  i.e.  the  two  surfaces  being  blended. More  generally,  given  a  suitable  spine,  a variable  radius  blend  can be  defined  in  principle.  It  is  the  surface  generated  by  a  variable sphere  as  its  centre  moves  on  the  spine  while  it  maintains  contact with the primary surfaces.

In  Pegna's  approach, a  user  defined reference curve  is  the initial  spine  and  one  of  the  primary  surfaces  is  used  as  a  reference surface.  Since  the  reference  curve is not  the  intersection  of  offsets  it is adjusted  to  assume the positon  of  the  actual  spine by  an  iterative algorithm-  However, Pegna's method for obtaining the correct spine has the following  deficiencies.

- The  iterative  algorithm  does  not  use  the  exact  distance  of a point on the reference curve to the reference surface.
- The iterations to obtain a point on the exact spine (i.e.  one which is  equidistant  from  both  primary surfaces)  may exhibit sideway drifts in a manner that cannot be controlled  analytically.

One  way  of  overcoming  the  above  deficiencies  is  to  consider planes  through the  two perpendiculars at each reference curve point. In Fig.  1, let p be a point  on  the  reference  curve Cr and F be a  reference surface. Let dp{p) be  the  perpendicular  distance  of p to F.

<!-- formula-not-decoded -->

Now construct  offsets of  the primary  surfaces by  the distance dp{p). Let S denote  the plane  defined  by  the perpendiculars  from p to  the primary surfaces. The point on  the exact spine curve,  corresponding to p on Cr, is given  by the intersection of  the two offset surfaces and H.

The  above  method  to  compute  the  exact  spine  curve  from  a given reference curve  is  analytically  precise.  However, if  on  the reference curve there exist points which have more than one Trti- n i r n n - m  length  perpendicular  to  the  reference surface, this  method would be insufficient.

Although  computing  the  exact  spine  by  the  above  method  is clean  and. precise, we believe it is overly  complicated  to merit  actual usage.  Moreover,  it  does  not  provide  any  mathematical  insights  on the  surfaces  being  considered  that  might  be  beneficially  exploited. Thus,  we  do  not  explore  this  method  any  further,  but  use  the concept of points equidistant from two surfaces to introduce Voronoi  surfaces next.

## 3. Voronoi  Surfaces

In  computational  geometry  Voronoi  diagrams  are  widely  used in the study  of proximity  problems. In essence,  a Voronoi  diagram is the tesselation  of  a plane by  polygons,  each  containing  a point p of a given set S. The basis for construction of the polygon W associated  with  point p is  that  W  is  the  locus  of  all  points  on  the plane  that are closer to p than  any other point of  S. Each point of S has  a unique  polygon  containing  it  and  the Voronoi  diagram  of  the set S is the  collection  of  the polygons.  (See Fig.  2).

## [Fig.  2 here]

Voronoi  digrams have  interesting  properties  and  for  a  detailed discussion  the  reader  is referred  to  [Preparata  and  Shamos  85]. It is easy  to  see  that  each  edge  of  a Voronoi  polygon  consists  of  points that  are equidistant  from points p { and pj, of  the given set S, that it separates. Voronoi  diagrams  can be  defined for  any dimensions,  in a manner analogous to the planar  case.

In  the  previous  section,  we  were  interested  in  the  spine  curve for  variable  radius  blends.  We  noted  that  all  points  of  the  spine curve  had  to  be  equidistant  from  the  two  primary  surfaces  being blended.  Thus,  a  collection  of  such  equidistant  points,  from  two given  surfaces,  can  be  conceptualized  as  a Voronoi  surface  - a  3D analogue  of  the Voronoi  diagram  on the plane.  The Voronoi  surface we are considering, is more  complicated  than simply  a 3D version  of the planar Voronoi  diagram since, the  given set of  point  S, in 3D, is now  replaced  by  the  two  primary  surfaces.  If  there  are  three  given surfaces,  the  Voronoi surfaces  would be reduced to equidistant curves that are the intersections of several Voronoi  surfaces.

Let  lis  consider  two  primary  surfaces, F and G. The  Voronoi surface V{F, G) is  then defined to be the locus of all points equidistant from F and  G. Thus  mathematically,

<!-- formula-not-decoded -->

where dp(p) and d(j{p) are  the  perpendicular  distances  of  point  p from F and G respectively.

To  compute  the Voronoi  surface V{F, G)  a  general method  can be  described  as  follows.  Consider  two  spheres Sp and SQ, each  of radius  r,  with  centres  on F and G respectively.  Letting r  be  the offset  distance,  the  oflset  surfaces  of F and G,  are  given  by  the envelopes  generated by Sp and SQ as they  move on F and  G. This is equivalent  to  displacing  the  centres  (ui,u2) u 3)  of Sp and  (u1} t&gt;2,U3) of SQ ,  as  they  move  on F and  G,  by  the  radius  of  the  spheres. Denoting  the pair  of  linearly independent  tangent  directions  to F at any point on  it by and  t2  and similarly  to  G by  t\  and  t2 w e the following ten  equations.

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Thus,  we  have  eight algebraic equations in  10  unknowns. By eliminating  the variables  [ ui,u2&gt; u 3&gt; u i&gt; w z» u 3» r   ]  w e can. obtain  the equation  of  the Voronoi  surface V{x,y,z ) =  0 associated  with F and G.

We  illustrate  the  above  procedure  by  an  example.  Let F be  a cylinder  of  unit  radius,  parallel  to  the  z-axis  and  having  the  line (a;=0,  y=2)  as  its  axis. Let G be  another  cylinder  of  unit  radius, parallel  to  the  x-axis  and  having  line (z=0, y= -2)  as  its  axis. Thus we have the equations

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Let r  be the  radius  of the  spheres Sp and SG. At points (u1}u2j u 3) ( u i&gt; u 2» u 3) on  F G respectively,  we  have  the equations

<!-- formula-not-decoded -->

The gradients VF and VG are given  by  (2u1}  2u2-4,  0] and  [0, 2u2"H,  2u3].  The two independent tangent vectors  at (u1&gt;u2) U3) on F,  are  (0,0,1)  and  (2-u2,u1,0)  and  at  (t&gt;i,t&gt;2,t&gt;3)  on  G  are,  (1,0,0) and  (0,u 3 , u 2+2). Thus,  the  final  system  of  eight  equations  are given  by

<!-- formula-not-decoded -->

After elimination, we obtain the equation of the Voronoi  surface as

<!-- formula-not-decoded -->

which is a hyperbolic  paraboloid,  as shown in Fig.  3.

<!-- formula-not-decoded -->

## 4. Variable Radius  Blends

We  can  now  give  a precise  definition  of  variable  radius  blends. Conceptually,  the  variable  radius  blending  surface  can  be  thought of  as  being  generated  by  a  moving  sphere  of  varying  radius.  The envelope of  spheres of  constant or varying  radius have been  referred to,  in  the  classical  literature,  as  canal  surfaces  [Hilbert  and  CohnVossen  52]. Thus,  the variable  radius blend  is the portion  of  a canal surface,  bounded by the curves of tangency  with the primary surfaces  that  the  variable  sphere  maintains  contact  with,  during  its motion.

Mathematically, a variable radius blend C(F,  G) smoothly connecting  surfaces F and G at  their  curve  of  intersection,  can  be stated  as

<!-- formula-not-decoded -->

where,  ? is  a  spine  curve  and  f is  a  radius  variation  function. Further, the spine curve is defined

<!-- formula-not-decoded -->

as  the  intersection  of  the  Voronoi  surface  of F and G and  a  given reference surface  J22S]. The  choice of  a reference surface will  depend upon  the particular  surfaces being blended. When  either F or G is  a plane,  [-RS]  can  be  chosen  to  be  a  plane  and  the  spine  $  will  be given  by

<!-- formula-not-decoded -->

The radius variation function  f"  defines the law by which the  radius of  the  moving  sphere  varies.  In  general,  it  can  define  the  variable moving sphere as follows.

<!-- formula-not-decoded -->

where or is the  angle subtended  by  [RS] with  the vertical  plane.  The envelope  of  the  moving  sphere  can  now  be  obtained  by  eliminating (i1,y1,2!1)  from f and its first derivative f 1 .

Note  that  in  simple  cases  the  radius  variation  function  f"  can also be specified by  a maximal  radii of  the moving  sphere.

<!-- formula-not-decoded -->

## 5.  Complexity  of Explicit Variable Radius  Blends

The  above  method  for  defining  and  generating  variable  radius blends  is  computationally  very  intensive.  The  explicit  derivations  of the  Voronoi  surface,  spine  curve  and  subsequently  the  envelope  of the  variable  sphere,  require  excessive  symbolic  computations  and were not possible  given  the computational  resources  available  to  us. While the rigour of specification is an attractive feature, the required  computations  render such  a process impractical,  except as a preprocessing  step.  This  motivates  alternative  strategies. For  the problem at hand, a three part alternative can be outlined as follows.

- Use  geometric  insights,  if  possible, to  predetermine the form of the Voronoi  surface.
- Determine  the  spine  curve  numerically  only,  by  tracing  it in higher  dimensions.'
- Approximate  the  variable  radius  blend  by  an  appropriate lower order surface.

We remark on each alternative  below.

The general method outlined for computing the Voronoi surface  is  intractible  for  practical  purposes  (i.e.  lack  of  swap  space in  our  computer).  However,  geometric  properties  can  sometimes  be exploited  to overcome the computational  difficulties. For  example,  if one of  the  primary  surfaces is a plane  and  the  other  a cylinder  or  a cone,  the  associated  Voronoi  surface is always a cone. We  conjecture the  following  Voronoi  surfaces  associated  with  pairs  of  standard CSG primitives: a hyperbolic paraboloid for cone/cone, cone/cylinder and  cylinder/cylinder; a  paraboloid  of  rotation  for sphere/plane; a degree four surface for sphere/cylinder and sphere/ cone.

The  evaluation  of  space  curves  defined  as  the  intersection  of two  surfaces (parametric or implicit)  is  of  prime  importance  in CAGD.  To  overcome  the  excessive  computations  necessary  in  order to determine  such  curves  explicitly  numerical tracing in higher dimensions  is  an  attractive  alternative  [Hoffmann  89],  Merits  and procedures  for  tracing  algebraic  curves  can  also  be  found  in  [Bajaj et  al.  88],  [Hoffmann  88].  Using  concepts  from  classical  algebraic geometry  such  numerical  procedures  have  been  made  robust to overcome difficulties  associated with curve singularities. In our problem, tracing the spine curve  numerically in 10-dimensional space serves a dual purpose. Not only does it simplify the computation,  but  also yields  the  common  distance  and footpoints  of the  perpendiculars  from  each  curve  point to  both the  primary surfaces.

Finally, the  high degree  of the  exact  blending surface, (in general  degree  16  or  higher)  and  difficult  computations  associated with  such  surfaces  make  approximations  an  attractive  alternative. In  the  case  of  fixed  radius  blends,  a method  exists  due to  Rossignac and Requicha [Rossignac and Requicha 86], Their method approximates  the  fixed  radius  blends  by  smoothly  joined  pieces  of cylinders  and  tori.  For  variable  radius  blends,  cyclides  appear  to  be a natural  choice for the  approximant.

## 6.  CASE  STUDY:  Cylinder  and Inclined  Plane

In  this  section  we will  consider  an  example  to  demonstrate  the concepts  developed  so far. We  have chosen  the example with  a view to  keep  computations  tractible  but  one  which  does  not  compromise on  the  general  problem  characteristics.  In  particular,  we  consider the  problem  of  variable  radius  blending  of  a  circular  cylinder  and an  inclined  plane.  This  problem  permits  us  the  choice  of  a simplest possible reference surface - a plane.

## 0.1 Voronoi  Surface  Computation

Let C be  the  cylinder  of  radius R  with  its  axis  coincident  with the  z-axis. Let L be  the  inclined  plane  making  an  angle a with  the z-axis  (see Fig.  4).

<!-- formula-not-decoded -->

To obtain the  Voronoi surface V(C,i?) we can apply the proposed  method.  The  cylinder  offset  surface  by  a  distance d is given  by

<!-- formula-not-decoded -->

The offeet of  the plane by  the same distance d is given  by

<!-- formula-not-decoded -->

The Voronoi surface V(C,L) being the locus of all points equidistant  from C and L is  now  obtained  by eliminating the distance  parameter d between  the  above  two  equations. Doing  so, we obtain

<!-- formula-not-decoded -->

which is a cone having for its base an ellipse on the plane  L.

## 6.2  Cyclides  and Special Reference Planes

We  consider  the  simplest  possible  reference  surface,  a  plane. So,  the  spine curve  which  is  defined  as  the  intersection of the reference surface and the Voronoi  surface, is now  a conic  and  can  be easily  computed.

Adhering  to  the morphology  of  cyclides  presented  in  [Chandru et  al.  88],  we  know  that  a central  cyclide, with  an  ellipse  and  a hyperbola as spines, has the general  equation

<!-- formula-not-decoded -->

The three parameters  associated  with  this form  of  a cyclide  are  a, / and  r. Here, a and  /  are  the  semi-major  axis  and  focal  lengths,  of the  ellipse,  respectively,  and  r is  a  constant.  When  /  &lt; r &lt; a,  the form  of the central cyclide resembles a  squashed torus and  is referred to as a ring central cyclide. Elementary geometric properties of the ring  central cyclide readily imply the following.

Observation: The  variable  radius  blend  is  a  ring  central  cyclide iff [ 55]  is orthogonal  to the  cylinder  axis.

This  observation  was  also  made  by  Pratt [Pratt  88]. When [.RS]  is orthogonal  to  the  cylinder  axis,  the  cross-sectional  profile  of the  blended joint,  on  the XZ-plane,  is as shown  in Fig.  5.  Thus,  the radius variation  function  can  be specified  as a maximal  radii  (e.g. in  Fig.  5). The  cyclide  parameters g ,  r and  /  are  then  determined as follows.

<!-- formula-not-decoded -->

The cyclide surface and  hence  the required blend, can be constructed  by  algorithms  discussed  in  [Chandru  et  al.  88].  In  our case study,  i.e. for  the special  position  of  [55],  the  circular  cylinder is  tangent  to  the  cyclide  at  its  inner  extreme  circle  on  the  plane  of the elliptic spine.  Thus, the blending  surface  is actually  a quarter  of of the elliptic  spine  and  the  circle at which  its lower tangent plane  touches the cyclide  surface  bounded by two  special latitudinal lines curvature  namely,  the  inner  extreme  circle  on  the  plane  of the  cyclide.

## 6.3  General Reference Planes

When  the reference plane  [iZ5] is in  a general  position  (i.e.  not orthogonal  to  the  cylinder  axis),  a  conic  is  still  obtained  for  the spine  curve  of  the  blending  surface.  However,  a  ring  central  cyclide can  no  longer  be  used  in  its  entirety to  blend the  cylinder  and inclined plane at their intersection. In general,  we obtain the following set of  equations describing  the blending surface.

1.  Spine:  [Voronoi  Surface]  fl  [i?S].

<!-- formula-not-decoded -->

2. Moving  sphere S centered at  (u,v,tu).

<!-- formula-not-decoded -->

3. Directional  derivative at (u,v, w).

<!-- formula-not-decoded -->

Here,  [(m, n, 1) x  (Vu, Vw)]  is  the  spine  tangent  obtained  as the  cross  product  of  the  intersecting  surface  normals. The  equation of the variable radius blending surface can be obtained by eliminating the variables u,  t;  and t u  from the above set of equations.  However,  even  when m = 0  and  specific  values  are  given for ot,  n,  p and q, extensive  computation  is required for  determining the  variable  radius  blending  surface.  For  m =  0, the  exact  blending surface is of  degree  16.

The computational difficulties motivate approximating the blending  surface.  For  fixed  radius  blends,  a  method  exists  due  to Rossignac  and  Requicha [Rossignac  and  Requicha 86]. In their method, the blending surface is approximated by piecewise  cylinders and  tori,  implying  that  the  spine  curve  is  approximated  by  circles and  straight  lines.  Using  cyclide  pieces  instead  of  cylinders  and  tori requires  that  the  spine  curve  of  the  variable  radius  blend  now  be approximated  by the elliptic spines of the  cyclides.

## 7. Approximation  with  Cyclides

Insights  into  the  global  geometry  of cyclides are  helpful  in outlining a procedure for approximating blends with cyclides [Chandru et al. 88]. Central to  this issue is a methodology  for  space curve  approximation  by  cyclide  spines.  The  following  steps  outline the procedure to join  cyclides.

1. Define the spine to be  approximated.
2. Approximate it with piecewise  cyclide spines.
3. Erect over each element of the  approximated spine a cyclide piece.

Step  1 is the  definition  of  the  spine  for  a variable  radius  blend and has been  dealt with  in Section 4.  Step 3 requires algorithms  for the  construction  of  a  cyclide  by  its  lines  of  curvature  and  has  been discussed  in detail in  [Chandru et al. 88]. Thus, it remains to  outline a  procedure  for  Step  2. However,  we  note  that  the  Steps  2  and  3 are  not  independent  of  each  other  in  that,  for  conics  to  be  cyclide spines, additional constraints are  imposed.  We  remark  on the constraints further.

Firstly,  the  tangents  to  the  curve  points  are  skewed  lines  in space.  Thus,  a  single  conic  element  cannot  interpolate  a  pair  of curve  points and the  associated  tangent  directions. To  overcome this  difficulty,  we  use biarc elements  for  the  approximation.  In essence, conic biarcs are pairs of smoothly joined conics that interpolate the given curve points and  their associated tangent directions,  by  introducing  a  set  of  points  we  refer  to  as  the join points. The  join  point  &lt; ? , · is  associated  with  the  biarc B{( U, V ).  As shown  in  Fig. 6,  biarc i?,-( U} V) interpolates space  curve points (P,,P,-+1)  and  tangents  (T,-, 2\+1). The join  point  Qi  lies  on  a  line Ri  that can  be thought  of as  a connecting rod resting  on the tangents  T,- and  Tl+1.

<!-- formula-not-decoded -->

Each  biarc  in  itself  defines two  control  triangles.  In Fig.  6,  the control triangles are AP,- O,- Q{ and AQt-0(+1P,-+1 respectively. Within  every  such  triangle,  Liming's  method  can  be  conveniently used  to  obtain  specific  conics  - ellipses  in our  case  [Faux  and  Pratt 79]. As  shown  in Fig.  7,  the  pencil  of  conics  obtained  by  Liming's method  pass  through  the  vertices A and B , of  A ABC, and  are tangent  to  the  sides AC and BC. The  following  equation  describes the pencil of  conics

<!-- formula-not-decoded -->

The choice of a shoulder point on the  median line CD, predetermines the  specific  type of conic -  ellipse,  parabola, or hyperbola - obtained within the control triangle  &amp;ABC [Dutta 89].

<!-- formula-not-decoded -->

The  collection  of  surface  normals  at  every  longitudinal  line  of curvature  on  the  cyclide  forms  a  right  circular  cone. The  elliptic spine  of  a  cyclide  is  the  locus  of  vertices  of  all  such  cones.  Thus,  to obtain  a C 1 continuity  between  the  surfaces  of  adjacent cyclide pieces,  it is necessary  for  the  cones  (of  surface  normals)  of  adjoining cyclides  to  coincide. Illustrated  in Fig. 8, this  constraint  implies  the following:

- Angles  at the vertices of the two cones must be equal.
- Perpendicular  distances of the bases of  the two  cones,  from their common vertex, must agree in magnitude and direction.

## [Fig. 8 here]

The  approximation  procedure  is  detailed in [Dutta 89]. A simplifying fact is that, since right circular cones are being  matched, appropriate  rotations  about  their  respective  axes  has  the  effect  of untwisting the  space  curve  approximation  procedure  into  one  plane. In  particular,  we  consider  the  problem  on  the  plane  containing  the elliptic  spines  of  adjacent  cyclide  pieces.  Thus,  the  aforementioned constraints  associated  with  the  common  cone,  in  3D space,  can  now be  related  to  a  pair  of  intersecting  (generating)  lines  that  represent the cone on the plane of  axial  cross-section.

Each  right  cone  that  has  its  vertex  on  the  elliptic  spine  of  a cyclide  contains  the  foci  of  the  ellipse.  Thus,  on  the  plane  of  the ellipse,  an  axial  cross-section  of  every such  cone  is given by  the pair of  intersecting  lines  joining  the  foci  to  that  point  on  the  ellipse, which  is  the  vertex  of  the  cone  being  considered.  In  view  of  the above  fact,  the  vertex  angle  constraint  for  common  cones  between adjacent  pieces,  translates  to  common  focal  lines  through  the  cone vertex,  on  the  plane  of the  ellipses.  Focal lines  Ly and  L2 are common  to  the  adjacent  ellipses U and V, in  Fig.  9.  It  is  shown  in [Dutta  89]  that  Liming's  method  can  still  be  used  to  obtain  ellipses within the control triangles, that satisfy this  constraint.

## References

- Bajaj,  C., Hoffmann, C.M., Hopcroft, J., Lynch, R.,  (1988),  "Tracing Surface Intersections",  CAGD, v5, pp  285-307.
- Chandru,  V.,  Dutta,  D.,  and  Hoffmann, C.M., (1988), "On the Geometry  of  Dupin  Cyclides",  CSD-TR-818,  Computer Science Department, Purdue University, November  1988.
- Dutta,  D.,  (1989),  'Variable  Radius  Blends  and  Dupin Cyclides", Ph.D. Dissertation, School of Industrial Engineering, Purdue University, January  1989.
- Faux,  I.D.,  and  Pratt,  M.J.,  (1979), Computational  Geometry for Design and  Manufacture , Ellis Horwood Publishers, Chichester.

## [Fig. 9 here]

Every  longitudinal  circle  of  curvature  on  the  cyclide,  intersects the  plane  of  its  elliptic  spine  in  two  diametral  end-points.  The  line joining  these  two  points  is  the  projection  of  the  circle  of  curvature of  the  cyclide,  on  the plane of  the ellipse.  Thus, on such a plane," the constraint of a  common  base  implies  that  the  line  through the diametral  end-points  is common  to both  ellipses  (see Fig.  10). This line  always  intersects  the  major  axes  of  the  adjacent  ellipses. The ratio  in  which  the  point  of  intersection divides  each  major  axis determines  the sub-form  of  the  associated  cyclide  i.e.,  horned,  ring, or spindle  [Chandru et al. 88].

[Fig.  10 here]

- Hoffmann,  C.M.,  (1989), Geometric and  Solid  Modeling ,  Morgan Kaufman Publishers, to  appear mid 1989.
- Hoffmann,  C.M.,  (1988),  "A  Dimensionality  Paradigm  For  Surface Interrogations", CSD-TR-837, Computer Science Department, December  1988.
- Hofimann C.M., and Hopcroft, J., (1985), "Automatic surface generation in computer aided design," The Visual Computer, vl,  pp  92-100.
- Hoffmann C.M., and Hopcroft, J., (1986), "Quadratic blending surfaces,"  CAD, vl8, n6, pp  301-306.
- Hoffmann  C.M.,  and  Hopcroft,  J.,  (1086a),  "The  potential  method for blending surfaces and corners," in Geometric Modeling: Algorithms and  New Trends ,  Edited  by G. Farin,  SIAM, pp  347-365.
- Hilbert D., and Cohn-Vossen S. (1952), Geometry and the Imagination, Chelsea Publishing Co., New York, reprinted  1983.
- Martin,  R.R.,  duPont,  J.J.,  and  Sharrock,  T.J., (1986), "Cyclide surfaces  for  computer-aided  design,"  in, Mathematics of Surfaces , Conference Proceedings, Institute of Mathematics and its Applications, 1984, Oxford University Press,  Oxford.
- McLean, D., (1984), "A method for generating surfaces as a composite  of  cyclide  patches,"  The  Computer Journal, v28, n4, pp  433-438.

- Pratt, M.J., "Applications of Cyclide Surfaces in Geometric Modelling", Proceedings of the IMA Conference on Mathematics of Surfaces, Oxford, September 19-21, 1988.
- Nutboume,  A.W.,  and  Martin,  R.R.,  (1988), Differential Geometry Applied To Curve And Surface Design, Volume 1: Foundations, Ellis Horwood  Publishers.
- Pegna,  J.,  (1987),  'Variable  Sweep  Geometric  Modeling"  ,  Ph.D. Dissertation, Mechanical Engineering Department, Stanford University,  1987.
- Preparata, F.P., and Shamos, M.I., (1985), Computational Geometry:  An  Introduction, Springer-Verlag.
- Rockwood, A.P.,  and Owen, J.C., (1986), "Blending surfaces in solid modeling,"  in Geometric  Modelling:  Algorithms  and New Trends, Edited by  G. Farm, SIAM, pp  367-383.
- Rossignac, J.R., and Requicha, A.A.G., (1986), "Offsetting Operations in Solid Modeling," CAGD, v3, pp  129-148.
- Varady, T., Vida, J., and Martin, R.R., (1988), "Parametric Blending  in  a Boundary  Representation  Solid  Modeller", manuscript,  1988.

<!-- image -->

<!-- image -->

FIG.3

<!-- image -->

FIG. 4

<!-- image -->

<!-- image -->

FIG. 10

<!-- image -->

FIG. 10

<!-- image -->

FIG. 10

<!-- image -->

TOININiQ  TWO

a c L l D E S

FIG. 10

<!-- image -->

FIG.  10

<!-- image -->