## Abstract

This  paper  describes a hidden  curve  algorithm specifically designed for  sculptured surfaces. A technique is  described to  extract the  visible curves for  a  given  scene  without  the need to  approximate the  surface by polygons.  This  algorithm  produces  higher  quality results than  polygon  based algorithms, as  most  of  the  output set has  an  exact representation. Surface coherence is  used  to  speed  up the  process. Although  designed for  sculptured surfaces, this algorithm is  also suitable for  polygonal data.

<!-- image -->

<!-- image -->

## Hidden Curve Removal for  Free  Form  Surfaces*

## Gershon Elber  and Elaine  Cohen

Computer Science  Department University of  Utah

In [16,1,8,21,26] the  idea  of quantitative invisibility and the use of critical points  was  developed. Critical  points  are  intersection points  between projected polygon primitives. The  technique  was extended  to bi-quadratic patches  in  [12].  The primitives are  subdivided  at  each  critical point  which guarantees that  the interior of  each  segment  has  homogeneous visibility. A segment's visibility isthen  tested by firing a ray  from the  eye  to  an interior point  of  the  segment.

CR  Categories  and  Subject  Descriptors: 1.3.5 [Computer  Graphics]: Computational Geometry  and Object  Modeling. 1.3.7  [Computer Graphics]:  ThreeDimensional  Graphics  and Realism.

Additional Key Words and Phrases:  hidden  curve  removal,  curve-curve intersection, visibility propagation.

## 1 Introduction

Hidden line  removal is  one of the earliest computer graphics  problems;  yet  new  algorithms  appear  every year  [16,1,8,21,4,15,17,18,20,22,23,24,26]. Usually they are  developed  for  polygonal  data,  so sculptured surfaces must be preprocessed and approximated  as  large collections of  polygons. The result displays the  polygonized  models accurately, but original model information  is  lost  [4,18,20] (see  figure 1).

*This  work was supported  in  part  by DARPA (N00014-88-K0689). All opinions, findings, conclusions or recommendations expressed in  this document are  those  of  the  authors  and do not necessarily reflect the  views  of  the  sponsoring agencies.

Permission to  copy  without fee all or  part of  this material is  granted provided that the  copies are  not  made or  distributed for direct commercial advantage, the  ACM copyright notice and  the  title of  the publication and  its date appear, and  notice is given  that copying  is by permission of  the  Association for Computing  Machinery. To copy otherwise, or  to  republish, requires a fee  and/or specific permission.

While the same basic  approach  is  used here,  it  is extended  to apply to arbitrary nonuniform rational B-spline  (NURBS)  surfaces (the  primitives). By not using  polygonal  approximations, the  algorithm  eliminates  the  vast  amount of  data  resulting from the  approximation  of  surfaces by polygons. The  algorithm presented here  has  several stages (including extracting curves  of  interest, splitting at  critical points, and visibility testing). Surface  coherence  is  used extensively to  reduce  the  number of  ray  tests one  needs  to  perform to  detect curve  visibility, in  a similar way to  that  for polygons[10,11].

The trade-off is  a reduced  number of  primitives in exchange  for  a higher  complexity of  the  operations between  them. Driving  the  output  directly from the  surface  results in high quality images (Figure  1). Only silhouette curves  need to be approximated, but they can  be  approximated  at  a much higher  resolution than with  pre-polygonized surfaces. Figure  I compares  the two,  after adaptive subdivision has  been  used  to  obtain the  polygonal  approximations.

Section  2 defines the  required elements  for  presenting the algorithm  and also  introduces the basic  hidden curve  algorithm.  Section  3 briefly explains  how the  curves  of  interest are  extracted from the  surface. Usually  four  types  of  curves  are useful:  the  surface boundary curves,  curves  along C 1 discontinuities in the  surface (if any),  iso-parametric curves, and silhouette  curves.  Section  4 addresses the 2D  curve-curve

<!-- image -->

<!-- image -->

O

Figure  1:  Left:  1930 polygons, middle:  4215 polygons, right: 2 NURBS surfaces (56  rational patches)

intersection problems  specific to  this algorithm.  Section  5 presents issues in  visibility testing arising from a surface-ray intersection algorithm.  Computation  of surface-ray intersections is  relatively expensive, especially when  polygonal  approximations to  the  surfaces have not been used,  so  methods to  speed  the  computation are  described. Section  6 discusses extensions to this  algorithm for  trimmed surfaces. A  more in  depth treatment of  the  above  issues can be found  in  [14].

All  the  images/comparison examples shown in  this paper,  were generated  using  nonuniform,  rational Bspline (NURB)  surfaces as  implemented  in  the  Alpha\_l system.

## 2 The  Basic Algorithm

Let  the view  orientation be  normalized so  that the  view point  V  is  on the  positive z-axis at  ~  and the  image is projected onto  the  plane  z =  0,  which will  be referred to  as  the projection plane or  the screen. All  other  views can easily be preprocessed into  this view.

## Definition 2.1

1. A  non-self-intersecting surface  is  called a simple  surface.
2. A  non-self-intersecting curve  is  called a simple  curve.
3. Let  P  be  the  map to  the  projection plane, P(x,y, z) =  (x,y,O).  The subscript P will  usually  denote the mapped  entity, e.g., cp = P(C).

We  consider  only  simple  surfaces whose projected curves  are  simple  curves.  Also,  intersection between surfaces are  restricted to  lie along  boundaries. If  not, the  surfaces should  be intersected using  the  appropriate  boolean  operations, and trimmed surfaces should be created. Figure  9  has  surfaces which interpenetrate, and so  boundaries between  them are  unknown  and visibility cannot  be always  correctly determined.

Suppose surface S  = S(u, v) E C l,  then N(u,  v)  = ~Sxg'S 0"-~ ~ is  normal to  the  surface S. Unless  otherwise specified, all  surfaces with  which we deal  are  assumed to be C 1 continuous, and their  parametrizations are assumed to  be such that N(u,  v)  ~  O. If  p is  a point on the  surface S, then it  is  the  image of  a point  in parameter  space, so  we write p = S(u  p,  vP).

## Definition 2.2

1. A silhouette point of  a  surface is  a  point p on the  surface whose normal  has  a  zero z-component,  i.e. Nz(u  p, vP)  \_--O.
2. A  silhouette curve of  surface  S  is  an ordered  set  of  silhouette points  Q  c  S forming a  continuous  curve such that

<!-- image -->

- V  p  E  Q,  andV  e &gt;  O, 3  q E  S  such that  Nz(u  q, v')  #  O, and lipqll &lt; e.
3. If3  D  C S  that  is  the  image of  an open disk  such that V d E D, Nz(u  ~,  v  a)  =  O, we  say that  S  has a silhouette surface and the  above  definition of  the  silhouette curve  is  simply  its  boundary.

We  will  not consider surfaces with  silhouette subsurfaces in  them. Surfaces that  do contain silhouette surfaces may be preprocessed by trimming  out  the  silhouette sub-surfaces (see  Section 6).

Lemma 2.1 A surface S cart be  decomposed into  silhouette curves  and disjoint regions whose  boundaries  are surface  boundaries  and/or  silhouette curves.

Proof: Consider  a point p  e  S, Nz(uP,  vP) ¢  O, and the  set  of  points which  can  be reached from  p by continuous paths  in  the  surface that  do not  cross  any silhouette curves. Let  that set  be Rp, and let Rq be a similar set  for  point q E  S,q ~ R  p, if such  a point  q exists. Clearly R  v  1 " 7 R  q =  $,  for  if there  exists point r  E R  p  N R  q,  then  there exists a continuous path  from p to  r  and  from  r  to  q,  and  hence  from  p to  q.  But  this contradicts the  way we picked  q.  Call  the  collection of path  connected regions T¢.

For  our  proofs we require that  the  projection operation be  bijective between  each  R E 7~  and  its projected image.

Lemma2.2 Let R E  7~  and rl,r2  E  R. Then, gz(url, vr,)Y,(ura, vr~)  &gt;  O, i.e.  they have the  same sign  at  both  rl  and r~.

Proofi Since  there  exists a continuous path  from rl  to  r2  totally in  R, Nz along  that path  can  have  no zeros so Nz has  the  same sign  at  both  rl  and r2.

We  define  invisibility as invisibility count  of  zero where:

Definition 2.3 The invisibility count of a point  p  is  the  number of  intersections of  the half-open ray  ~  from the  view  point V  to  the point  p  has with sculptured surfaces  in the scene.

We  will  concentrate our discussion on determining visibility only  (i.e. where the  count  is  zero), although one  may use  our  algorithm to  detect quantative invisibility [6].

Two  types of  curves are  dealt with:

<!-- image -->

## Definition 2.4

1. Surface  boundary  curves  and surface silhouette  curves  are defined  to be active curves.
2. The framework ~b s of  a surface S  is  the set  of  all  its active curves. Set  ¢~  = p(¢s).
3. Any curve  which is not active is  defined to be a passive curve. For example, an isoparametric curve  is  passive.

Clearly active curves  are  those curves  at  which  visibility of  all curves  can  change  when crossing them.

Lemma 2.3 Let  C  be a curve  in  the  scene. If Cp  does  not  intersect UseS,, then  all points p E C  have the  same invisibility count.

Corollary 2.1 Given a curve C,  let {C~} denote  the  collection of  sub-curves of  Cp  resulting from subdividing Cp  at  each of  its intersection points  with UseSe, and let  C  ~ be each pre-image,  i.e., C i C  C.  For each C i, all  points p  E  C i have the same  invisibility count,  except  possibly the  two endpoints.

The proof of  lemma 2.3  can  be  found  in  [14]. Corollary 2.1  suggests an  algorithmic way of  testing visibility of  curves in  a  scene.  Obviously  many projected curves will intersect with  some projected frameworks, but  the number of  times  they  do is  finite and usually small. For  each  of  the  subdivided curves, only  one point  in its open interval needs  to  be tested for  its invisibility count. As noted  in  [2,11], silhouette curves can  overlap; that  is,  a silhouette curve  can be G 1 continuous and still have  cusps  in  its projection (called a curtain fold [2]). Since  the  visibility of the  silhouette curves  may change at curtain folds, they  must be split at  these points.

Using Corollary 2.1,  we  can define the  basic  algorithm:

- . Extract all curves  of  interest from the  given  surface(s) in  the  scene.  This  results in  at  least the framework of  each surface, but can also  include passive curves such  as  isoparametric curves.
- . Split each  passive and  active curve  C at  each  point where its  projection Cp intersects with any ¢~ and where cs is  closer to  the  viewer.
- . For each curve that  results from the previous stage, fire a ray  in  the  view  direction through  an

<!-- image -->

Figure  2: Four  curve  types in  surface display.

<!-- image -->

arbitrary interior point  and find  the  invisibility count  of  that point.

4. If the  invisibility count  of  the  point is  zero,  then curve  is  shown,  otherwise it is  hidden.

Based on Corollary 2.1, the  evaluation of  the  invisibility count  picks  an arbitrary point  p in  the  interior of  curve  C,  and  determines the  visibility of  all of  C  using  the  invisibility count  of  p.  Curves  are  always  split at  tangent intersections, since  they  could  potentially change  visibility at  those points. A technique is used  in [16,1,8,21,26] of  splitting a line into segments  anytime it  intersects with  any  projected polygon  boundary  to make the  segment  visibility homogeneous.  In  the  next sections, we will  describe the  different stages for  arbitrary sculptured surfaces. This  algorithm, although conceptually simple,  does require accurate computation in  nonrobust numerical situations. We  discuss our approaches to  simplifying the  operations and  to  reducing  the number of  times  these complex  operations must be performed.

## 3 Curve  Extraction

The four  types  of  curves  which we display are  boundary  curves, iso-parametric curves, curves along  C 1  discontinuities in  the  surface, and silhouette curves  (see Figure  2).  All  but  the  last type  are  view  independent.

Extraction of  the first  hree types of  curves is  usually extremely simple  since these  curves are  isoparametric. For  example,  Figure 3  shows  the  same pawn as  in  Figure 1. We  assumed the  surfaces are  C 1 continuous, but if some are  not,  they  must be subdivided at  the appropriate isoparametric values.  This  creates a new boundary  and  two  new surfaces on  which  the  algorithm is then  correct. This  splitting stage is only  added  when tangent plane  discontinuities are  present.

<!-- image -->

/mii/ml

Figure  3:  Pawn extracted curves, in  u  - v space.

The silhouette curves  are  not  isoparametric  urves usually (Figure 3),  and in  general, there  is  no computationally feasible way to  represent them exactly. An approximation technique must  be  used  in  this case, and a piecewise linear curve  is  the  simplest choice.

One method of  extracting the  silhouette curves of  a surface is to  subdivide the  surface up to  a  given e  tolerance  and  fetch all the  surface pieces with  silhouettes on them. We  seek  methods using  subdivision and spline properties to  identify regions with  silhouettes. To solve this problem  we are  interested only  in  the  z  component of  g(u, v),

<!-- formula-not-decoded -->

Now  if the  first term  on the  right side  of  equation 1 is  positive everywhere  and the  second  negative everywhere,  Nn  is  positive everywhere.

Lemma 3.1 Given a scalar B.spline surface

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Proof:  Consider partial derivative of  o"  with  respect

<!-- image -->

Figure  4:  Adaptive  silhouette extraction, in R  3.

<!-- image -->

<!-- image -->

Figure  5:  Tangent  CCI.

<!-- image -->

to  u.

<!-- formula-not-decoded -->

Since  the  blending functions B are  nonnegative, and the  difference ri+k  - ri+l &gt;  0,  if ai+l,j -  ai,j &gt; 0,  for all i  and  j,  ~ will be positive everywhere.

Lemma  3.1, and an analogous version for  aa ~, can  be used  on the  x and y coordinate functions to  provide a simple  test  for  ruling out the  existence of  silhouette curves  in  a surface. Needing  only O(mn) first order  differences, it  cart act  as  an  additional termination condition for  the  subdivision. Figure  4 shows such  an adaptive subdivision for  a  surface with  three silhouette curves.

One must trace the  linear silhouette curve  segments resulting from the subdivision process  and connect them to  form  a piecewise linear approximations of  the real  continuous silhouettes. This solution frequently results in only  poor approximations or else  the  process  is  time consuming.  To  improve the  quality of each  point  obtained from  the  subdivision, a numerical technique is  applied to  make its  approximation more accurate. The secant  method is  used  with  good con- vergence, because  the  subsurface pieces are  relatively flat (from  the  flatness test  during  the  subdivisions). Figure  6 shows  the  pawn object with  the  resulting potential silhouette sub-surfaces from  another view  (left), and improved  (right). The result is  a piecewise linear approximation of  the  real silhouette which  is  accurate on  its vertices. One can  adaptively improve  the  middle of  each  segment  until the  distance between  the  previous and new  are  within  tolerance. By including this improvement  stage, we now  have curve  which has a high  degree  of  accuracy, both  in  its  resolution (number  of  points) and  accuracy of  each  point.

## 4 Curve-Curve Intersection

Curve-curve  intersection is  not  a problem specific to hidden curve removal algorithms.  The  solution to the  curve-curve intersection problem is  generally not available in  analytic form,  and some kind  of  subdivision  or  iterative technique is  used  to  determine those points [5,19,28]. We  do not  want to  be restricted to  a specific  urve  type  or  order, but  would like as  general a solution as  possible. While the  subdivision process guarantees convergence to  all intersections, it  is  slow, and  so  effort is invested in  finding methods  to  eliminate unnecessary subdivisions.

The algorithm used here  is  based  on combining  aspects of  methods  described in  [5,19,28]. Strips (rotated bounding  boxes)  were  used  to  bound the  curves  during the  subdivision process. However, instead of  carrying out  all the  subdivisions and then  doing  all the  numerical improvement,  the  subdivision is  alternated with  a Newton's  method at  such  point in  the  process that we can guarantee a unique  intersection using  cone tests (see  [27]).

It  is  required that we find the  intersections between curves  which are  tangent to  each  other.  Many  intersections between  iso-parametric curves and  silhouettes are  of  this kind.  For  example,  figure 5 is  an enlargement of  one such  problem region  in  figure 1. If  two curves  have the  same tangent line  at  their intersection  point, the  e  selected as  the  error measure  for  the computation  method affects the  results. If  it  is  too small, then  the  method  may miss  the  intersection; if it is  too  big,  several (approximated) intersection points may result close  to  each  other, where only  one actually exists. When  surfaces have  been  approximated by polygons  exactly the  same problem  occurs.  However, crossing of  active curves  in  the  projection plane  must be detected since the  visibility may change  at  such  intersections (Lemma  2.3).

In  this specific  ase  most of  the  tangential curve in- tersection can be considered in a simpler  framework. Consider  two parametric space curves  that  map  onto curves  in R 3 intersecting at simultaneous tangencies. In most cases  (see  Figure  5)  one of  the  curves  is  a silhouette  curve  and the other  is  isoparametric. Hence considering the intersection in parametric  space reduces  it  to  the  problem  of  intersecting a planar  curve with  a straight line.

Figure  6: Sub-surfaces, potentially with silhouette, from different view (left), and improved (right).

<!-- image -->

<!-- image -->

Another numerical  instability may  arise when  dealing  with  curves  whose projections are  identical or  have identical subcurves. If the  curves  are  exactly the  same, one can be eliminated. If  two distinct curves  project onto the same  curve in the image space, but have different orders, knot vectors, or parametric  continuity, there  is  currently no stable, computationally feasible  method for  analytically determining that  the  two curves  are  identical. Instead  a heuristic threshold on the maximum number  of valid  intersections is  used. This solution causes  some  overhead  which might be reduced  by analytical detection of  such  cases.

Given a curve-curve intersector, finding  all  the intersections among N  curves  the  straightforward way of intersecting each  curves  against all the  others is  O ( N 2 ) , which is  the  worst  case  possible. Usually  one can do much  better.  Sweep  algorithms to sort  and improve this  order  have been developed,  mainly for  straight line  segments [15,9,22,24]. They improve the average case  order  to O(NlogN), but  have  the  same worst  case behavior of  O(N2). Extending  this notion  for  an arbitrary  curve  type  environment  is  complex since  curves, unlike  lines, may  intersect more than once. In fact, two spline curves  can be continuous and have  no zeros in  their first and second derivatives, but can intersect multiple times,  as  Figure  7 demonstrates.

Figure  7: Monotone curves  have  multiple intersection

<!-- image -->

<!-- image -->

<!-- image -->

Although  the  current implementation of  this algorithm  uses  a simplified bounding  box sweep,  research in  this area  is  in  progress. More on this can  be found in  [3].

## 5 Surface-ray intersection

Finding  intersections of  rays  with  surfaces occur  frequently in  graphics and  geometry  problems, in  particulax  in  the  ray-tracing rendering technique[25,29]. Usually freeform surfaces are  first approximated into polygons  or  preprocessed into small simplified pieces. Both strategies require a  relatively large amount  of  memory. Since  only  a partial invisibility count  is  needed,  we need  not  solve for  the  exact  intersection point.

Lemma 5.1 Let  S  be a path connected  continuous  surface  with  no silhouette curves, g p(¢S) is  a simple  curve,  then  S  is  visible relative  to itself, and therefore, all  curves belonging  to  S  are  totally visible relative to it.

Proof:  Since  S has  no  silhouette curves, cs (definition 2.4)  consists only  of  S boundary  curves, and  ~  has exactly one  element. Since  p(¢S)  is a simple  curve, P is  bijective on the  interior of  its whole domain.

By making use  of  this lemma we can reduce  significantly the  number of  subdivisions required to  solve the  surface-ray intersection. When  resolving the  visibility of  a given  point p often one can  use  lemma 5.1, and decide on  p-visibility much sooner  than  using  only a subdivision technique.  This  decision can be made even  if the  exact  intersection point  of  the  ray  from  V to  p with  the  surface S is  unknown. Using  curve  visibility propagation and surface coherence, the  number of  rays that  need  to  be  fired can  be reduced.

Definition 5.1 For a curve  l?,  Touching(l?) is  the  set  @all curves,  other than  13,  that  have one endpoint  of  their projections lying  on ~p.

If  a  curve o~ is  split into  oq  and o~  at the parametric  value  where P(o 0  intersected with P(fl),  such that  cq  has a  lower  parametric range than  o~2,  then:

1. as is  the next curve oral, and will  be denoted  as  c~2  =  Next(a1).
2. oil  is  the previous curve  of  o~2,  and will be denoted  as gel  =  Prcv(c~2).
3. oq,  o~2 E Touching(fl).

To use  this coherence, the  curve  adjacencies of  definition 5.1  must  be  determined and  kept  during the  curvecurve  intersection stage. Each time  a curve  is  split, all the  information on the  splitting and split curves  must be  kept, in  addition to  the  z  level ordering relation between  them. Although,  this might  look  time  consuming,  it is  not.  Each time  a curve  is  split (a  relatively complex operation), a constant number of  adjacency pointers is  updated. By  doing  so,  one can derive  a set  of  rules  by which the visibility is propagated  so fewer  rays  need be fired.  The following corollary is only  a  subset of  such  rules and which  will be sufficient to  demonstrate its power:

Corollary 5.1 If  0  is  an  active  curve,  ¢  E Touching(q), and  7 is split  into 71  and 72  such  that ")'1,72 E Touching(rT),  71  = Prey(72) then (see Figure 8 for all reference8)

- I. If  ~ 7  is  a silhouette curve of  the  surface a, and a has only  one silhouette, then  ~7 is  visible relative ~o a. See h.
2. If  17  is  a visible silhouette curve,  at  least one of 71,  72 is  visible. See a, b and c. If  7  is  a passive  curve or  the  scene  has only closed models,  then exactly  one of 71,  "Y2 is  visible. See c.
3. If  ~ is  visible but  not a silhouette curve, then exactly one of  71  ,  72 is  visible. See i.
4. Ire is  passive curve  and 77  is  a boundary curve (but not a  silhouette), and  both have the same  z  at the touching  point, then  ¢  has the  same  visibility as 7. See f.
5. If  rl  E  Touching(~b),  both are boundary curves,  and both  ~ and ¢  have the  same z at  their touching  point,  then  ~1 and ¢ have the  same  visibility. See g.

Proof.'  All  the  above conclusions result directly from  lemma 2.3.

Each of  these cases  is simple  by  itself, but  in  combination they  form  powerful visibility propagation tools. Corollary 5.1-  2 is  unique,  in  that  one needs  more information so  the  visibility of  the  adjacent curve  can  be resolved. A  visible silhouette on closed  objects must bound two regions, exactly one of  which  is  visible, so only  a single adjacent curve  should  be tested for  visibility to  classify the  regions. For  open  objects, if a  silhouette curve  splits a boundary  curve, both  subcurves may be visible (see  a,  Figure  8).

<!-- image -->

G

Figure  8:  The curves  visibility ispropagated  -  firing 2 rays  is  enough for  all  curves.

<!-- image -->

Figure  9: A  hidden  curve  view of  the  teapot.

<!-- image -->

<!-- image -->

Table  1:  Visibility propagation comparison.

|         | Propagation   | Propagation   | No propagation   | No propagation   | No propagation   |
|---------|---------------|---------------|------------------|------------------|------------------|
|         | act. rays     | pas. rays     | tel. time        | act. rays        | pas. rays        |
| discont | 9             | 14            | 5.5              | 19               | 96               |
| saddle  | 2             | 2             | 8                | 8                | 54               |
| wiggle  | 11            | 7             | 4.5              | 21               | 95               |
| pawn    | 12            | 68            | 1.9              | 17               | ~ 152            |

We  shall use  Figure  8 to  demonstrate the  propagation ability. Assume the  framework  curves are  given  in in  the  following order:  the  silhouette curve  following by boundary  curves  1,  4,  2,  3.

The  visibility of each curve in the framework is checked  first. Using Corollary 5.1,  since only  one silhouette  curves  exists, by result 1, it  is  visible.  As this  object is  open,  no propagation to  the  boundary curves is  allowed. A ray  is  fired to  test the  visibility of boundary  curve  1,  and it is  found  to  be visible. Using resutt 5.1-5, curves  2.1  (through i,  Figure  8) and 4.1 (through 2) are  found  to  be visible. The next  framework curve  with  unknown visibility is  4  which  has  been split into two,  4.1  and  4.2.  The first half (4.1) visibility is  already known, so  only  one ray  for  the  second  half (4.2) is  fired, and which find  it  to  be visibile as  well. Using  result 5.1-5  curve  4.2  (through 3)  sets curve  3.2 to  be  visible, which  in  turn  sets 3.1  (through 4),  using result  5.1-3, to  be invisible. Since  3.1  touches  curve 2.2  (at 5),  at  its end,  2.2  is  also  set, using  result 5.1-5, to  be  invisible.

Instead of  firing 8 rays, for  the  8  active curves in  the saddle  surface (1,  2.1, 2.2,  3.1,  3.2, 4.1,  4.2  boundary curves,  and one silhouette), only  2 were fired.  The question  on  the optimal  way  to order  these  curves is  still open.  But the improvement in this  specific case  in  much greater. Using  result 5.1-4, since  all the iso-parametric curves  touch  boundary  curves  at  the same z value,  they  inherit their visibility from that of  the  boundary.  Using result 5.1-3, the  visibility of the  boundary  curves is  propagated to  the  interior isoparametric pieces, so no ray  need to  be fired for  any iso-parametric curve.

In  addition to  all the  above, if the  number of  surfaces is small, a cache  of  the  subdivided surfaces can  be  handled. If a  surface was subdivided once  for given  ray, all but  the  last few  steps will be identical for  nearby  rays. The main  problem  with  this approach  is the  amount  of memory it  may require, and so it  may be useful only if the  scene  consists of  a small  number of  surfaces.

Table  1  compares  this propagation in  terms  of  number  of  rays  fired for  active and passive curve  and rel- II  I I   I I | |

<!-- image -->

Figure  10:  Support  of  trimmed  surfaces.

<!-- image -->

ative time  to  test the  visibility of all the  curves. Note the  relation between  the  two is  not  linear mainly  because  of  the  cache  used.

## 6 Trimmed Surfaces

An  important  issue  to address  is the support  of trimmed surfaces. A  careful look  in  the  teapot  (Figure 9) uncovers  a problem with the  design  near its handle/body joint. The model was originally done by juxtaposing the  surface descriptions for  the  handle  and body  without trimming  the  surfaces, so  the  handle  and the  body  surfaces interpenetrate. Curves  which  should be hidden  at  the  joint are  visible, since  this algorithm cannot  detect such  cases.

Using  trimmed  surfaces in  the  model and in  the  al- gorithm  eliminates this  difficulty. The modifications necessary in  our  algorithm to  support trimmed  surfaces were  small and  occurred in  two  stages of  the  algorithm.

<!-- image -->

- .  The curve  extraction stage  needs  to  trim  the  extracted curves  against the  trimming  curves in  the parametric space.
- . The surface-ray intersection stage  should  test if the  point on surface S that  hides the  tested point is  in  a trimmed part  of  S. The trimming  curves should  be propagated through  in  the  subdivision process, and  a totally trimmed  out  surface can  be another termination condition for the  subdivision.

Figure  10 shows the results of  the  modified  algorithm.

## 7 Acknowledgements

Particular thanks  to  the  Alpha\_l  group for  all  their help  and support in  this research.

## References

- [1]  A.  Appel.  The Notion  of  quantitative Invisibility and the Machine Rendering  of  Solids  Proceedings ACM National Conference  1967.
- [2]  J.  Blinn. A scan line algorithm for displaying parametricaly defined surfaces. Computer Graplfics 12.3.  1977.
- [3]  13.  Bruderlin, E. Cohen, and G. Elber. A  Plane-Sweep Hidden-Surface Algorithm for  Curved  Surfaces. Technical report No. 90-006,  Computer Science, University of  Utah.
- [4]  C. Sequin  and  P.  Wensley.  Visible Feature Return  at  Object  Resolution. IEEE  Computer Graphics  and Application, May  1985,  pp 27-50.
- [5]  E.  Cohen,  T.  Lyche,  and  R.  Riesenfeld. Discrete B-splines and  subdivision Techniques in  Computer-Aided  Geometric Design  and  Computer Graphics.  Computer Graphics  and Image Processing, 14,  87-111 (1980).
- [6]  D. Griswold  and  M. Cohen. Automatic  Illustration of  3D Geometric  Models:  Lines. Proceedings of  the  1990 Symposiurn on Interactive 3D Graphics.
- [7]  V.  Fuson.  Application of  a  Hidden  Line  Algorithm to  Surface  Visualization. MS  thesis, Computer Science, University of  Utah,  1984.
- [8]  R.  Galimberti and  U.  Monta~aari.  An algorithm for  Hidden Line  Elimination CACM 12,  4,  206,  April 1969.
- [9]  G. Hamliu and C. W.  Gear. Raster-scan hidden  surface algorithm techniques Computer Graphics, Vol 11,  pp 206213,  1977.
- [10]  C.  Hormmg.  An Approach  to  a  Calculation-Minimized Hidden Line  Algorithm.  Computer &amp;  Graphics, Vol 6,  No  3, pp 121-126,  1982.
- [11]  C.  Hornung.  A  Method for  Solving the  Visibility Problem IEEE CG&amp;A July  1984,  pp.  26-33.
- [12]  C. Hornung,  W. Lellek, P.  Pehwald,  and  W.  Strasser. An Area-Oriented Analytical Visiblity Method for  Displaying Parametrically Defined  Tensor-Product Surfaces. Computer Aided  Geometric  Design, 2 (1985)  197-205.
- [13]  J. Foley  and  A.  VanDam.  Fundamental of  Interactive Computer Graphics.
- [14]  G.  Elber  and E.  Cohen. Hidden Curve Removal  for Untrimmed and Trimmed NURB Surfaces.  Technical report  No. 89-019,  Computer Science, University of  Utah.
- [15]  R. Guting.  New  Algorithm for  Special Cases  of  the  Hidden Line  Elimination Problem.  Computer Vision, Graphics, emd Image Processing 40,  188-204  (1987).
- [16]  I.  Sutherland, R. Sproull, aaxd  R. Schumacker  A  Characterization of ten  Hidden-Surface Algorithms. Computer Surveys, Vol.  6,  No. 1,  Mar.  1974,  pp.  1-55.
- [17]  T.  Kamada and  S.  Kawal.  An Enhanced  Treatment of  Hidden Lines.  ACM Transaction on Graphics, Vol 6,  No. 4, October  1987,  Pages  308-323.
- [18] J.  Kripac. Classification f  edges  and its  application in determining visibility. Computer Aided Design,  Volume 17,  Number  1,  January/February 1985.
- [19]  J.  Lane and  R. Riesenfeld. A Theoretical Development  for the  Computer  Generation and  Display of  Piecewise Polynomial  Surfaces. IEEE Transaction on pattern analysis and machine  intelligence, vol. PAMI-2, No. 1,  January 1980.
- [20]  L.  Li.  Hidden-line algorithm for curved  surfaces. Computer Aided  Design, Volume 20,  No.  8,  October  1988,  Pages  466470.
- [21]  P.  Loutrel. A Solution to  the  Hidden.line Problem  for Computer Drawn Polyhedra  IEEE Transactions on  Computers, Vol.  C-19,  No. 3,  205-213, March 1970.
- [22]  M.  Mckenna. Worst-Case Optimal  Hidden-Surface Removal. ACM Transaction on Graphics, Vol 6,  No. 1,  January  1987,  Pages  19-28.
- [23]  C.  Montani  and  M. Re. Vector  and  Raster Hidden-Surface Removal Using  Parallel Connected  Stripes.  It~EE Computer Graphics  and  Application, July  1987,  pp 14-23.
- [24]  O. Nltrrnl. A Fast  Line-Sweep  Algorithm for  Hidden  Line Elimination. BIT 25 (1985), pp 466-472.
- [25]  J.  Peterson. PRT -  A High  Quality linage Systhesis System for  B-spline Sin-faces. MS  thesis, Computer Science Dept., University of  Utah,  Dec.  1987.
- [26]  J.  Rank\_in.  A  Geometric  Hidden-Line  Processing Algorithm. Comput.  &amp; Graphics Vol.  11,  No.  1,  pp.  11-19. 1987.
- [27]  T. Sederberg  and R. Meyers. Loop Detection in  Surface Patch  Intersections. Computer Aided Geometric  Design  5, pp 161-171,  1988.
- [28]  T. Sederberg and S.  PaJwy. Comparison  of  Three Curve Intersection Algorithms. Computer Aided  Design, Volume 18,  Number  1,  January/February 1986.
- [29]  M.  Sweeney.  Ray Tracing  Free-Form  B-Spline  Surfaces. IEEE  Computer  Graphics  and Application, Februaury 1986,  pp 41-49.