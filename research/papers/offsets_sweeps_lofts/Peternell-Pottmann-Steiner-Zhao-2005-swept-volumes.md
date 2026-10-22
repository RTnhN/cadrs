## Swept Volumes

Martin Peternell 1 , Helmut Pottmann 1  , Tibor Steiner 1  and Hongkai Zhao 2

1 Vienna University of Technology, martin@geometrie.tuwien.ac.at pottmann@geometrie.tuwien.ac.at, tibor@geometrie.tuwien.ac.at 2 University of California, Irvine, zhao@math.uci.edu

## ABSTRACT

Given a solid 3 S R ⊂ with a piecewise smooth boundary, we compute an approximation of the boundary surface of the volume which is swept by S under a smooth one-parameter motion. Using  knowledge  from  kinematical  and  elementary  differential  geometry,  the  algorithm computes  a  set  of  points  plus  surface  normals  from  the  envelope  surface.  A  study  of  the evolution  speed  of  the  so  called  characteristic  set  along  the  envelope  is  used  to  achieve  a prescribed  sampling  density.  With  a  marching  algorithm  in  a  grid,  the  part  of  the  envelope which lies on the boundary of the swept volume is extracted. The final boundary representation of the swept volume is either a triangle mesh, a B-spline surface or a point-set surface.

Keywords: motion,  swept  volume,  envelope,  marching  algorithm,  point-set  surface,  NC verification.

## 1. INTRODUCTION

The swept volume of a solid S is  formed  by  all  points  in  space  which  are  covered  by  positions  of S during  a motion.  The  most  frequently  considered  motions  are  those  which  depend  smoothly  on  a  parameter t to  be thought of as time. The arising swept volumes have a variety of applications including NC machining verification, geometric modeling, robot workspace analysis, collision detection, motion planning, etc., see e.g. [11].

## 1.1. Previous Work

The theoretical foundation of swept volumes lies in the computation of envelope surfaces in kinematical geometry. This is classical knowledge and can be found in a number of textbooks, see e.g. [5] and [16]. Probably the first contribution  to  envelopes  dealing  with  computational  aspects  is  the  PhD  thesis  of  J.  Hoschek  [9].  There  is  a number of papers which provide explicit and even rational parameterizations of envelope surfaces (see the survey [13]). In general, however, one can only achieve a numerical approximation of the boundary surface of a swept volume.

Although  there  is  already  a  pretty  large  number  of  contributions  on  this  topic  (for  a  survey,  see  [1],  [4]),  the problem cannot be considered as sufficiently well solved and thus swept volumes are hardly included in today's CAD systems. From the most recent work we point to a paper by Kim et al. [11] which addresses swept volumes of moving  polyhedra  and  exploits  the  fact  that  in  this  case  we  have  only  ruled  or  developable  surfaces  on  the boundary of the swept volume.

Recently Yang and Abdel-Malek [18] addressed the computation of swept volumes of NURBS surfaces and the NURBS approximation of the swept volume's boundary.Their approach uses as an auxiliary step the slicing of the object by parallel planes and exploits the NURBS representation for the computation of points on the complete (untrimmed)  envelope  surface.  Trimming  relies  on  surface/surface  intersection  and  the  so-called  perturbation method.  Our  approach  differs  from  this  in  several  aspects:  We  do  not  need  a  slicing  step  or  a  specific representation of the moving object's boundary. The trimming is based on a marching algorithm on an underlying grid. The final representation is a point set plus surface normals , a triangular mesh or a NURBS approximation of the former.

## 1.2. Contribution of the article

Our paper does not make any special assumption on the moving solid S apart from the natural requirement that its boundary shall be a piecewise smooth surface Φ . As many other papers in the past, we use envelope theory to

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy compute the so-called characteristic sets at a discrete number of time instances. This immediately gives a cloud of points  plus  surface  normals  in  a  superset Γ of  the  boundary  surface Ψ of  the  swept  volume.  As  a  new component in our algorithm we use a formula for the evolution speed of the characteristic curve to get a good time step selection which yields a nice data point distribution on Ψ and Γ .

A kind of marching algorithm on an underlying grid is used to trim away the part of the envelope surface Γ which lies  in  the  interior  of  the  swept  volume.  Special  care  is  taken  to  correctly  resolve  edges  at  the  boundary  of  the swept volume. At this stage we have a sufficiently dense set of points plus surface normals - a set of surfels in Graphics terminology - at the boundary surface Ψ of the swept volume. Now, any method that can interpolate or approximate  such  data  is  appropriate  to  get  the  final  boundary  representation.  We  present  solutions  with  a triangle mesh, a point-set surface and B-spline surfaces.

The novelty of our method lies in the combination of classical results of kinematical and differential geometry with recent developments in geometry processing such as fast marching, point-set surfaces ([2], [3], [12]) and Bspline approximation using squared distance minimization [14].

## 2. GEOMETRIC BACKGROUND 2.1 The velocity vector field

Kinematical geometry provides the necessary geometric background for our problem. Thus we briefly review a few basics and refer to the literature for more information [5], [16].

Consider a rigid body moving in Euclidean three-space 3 R .  We think of two copies of 3 R :  One copy associated with the moving body and called moving space or moving system Σ , and one copy called the fixed space or fixed system 0 Σ . We use Cartesian coordinates and denote points of the moving system Σ by , , x y K and points of the fixed system by 0 0 , , x y K , and so on.

A one-parameter  motion 0 Σ Σ is  a  smooth  family  of  Euclidean  congruence  transformations  depending  on  a parameter t which can be thought of as time. A point x of Σ is, at time t , mapped to the point

<!-- formula-not-decoded -->

of 0 Σ . Each point x of Σ has a path curve or trajectory 0 ( ) x t in 0 Σ . The trajectory of the origin is 0 ( ) a t . ( ) A t describes the rotational part of the motion; we have 1 T A A -= and det( ) 1 A = .

The first derivative 0 0 ( ) ( ) ( ) x t A t x a t = ⋅ + &amp; &amp; &amp; of the path 0 ( ) x t of x is its velocity vector at time t . We write 0 ( ) v x for the vector field of vectors 0 ( ) x t &amp; attached to the points 0 ( ) x t .  The same velocity field, represented in the moving system,  is 0 0 ( ) ( ) ( ) [ ( ) ( )] T T v x A t x A t A t x a t = ⋅ = ⋅ ⋅ + &amp; &amp; &amp; .  Since A is  orthogonal, T A A ⋅ &amp; is  skew  symmetric.  This  gives immediately the well-known fact that the vector field ( ) v x is linear and has the special form ( ) v x c c x = + × , where

<!-- formula-not-decoded -->

The vector c is called vector of angular velocity .

Up  to  the  first  differentiation  order,  any  one-parameter  motion  agrees  locally  with  a uniform motion,  whose velocity vector field is constant over time. Apart from the trivial uniform motion, where nothing moves at all and all velocities are zero, there are the following three cases:

- Uniform translations have c o = , but c o ≠ , i.e., all velocity vectors equal c
- Uniform rotations with nonzero angular velocity about a fixed axis are given by 0 c c ⋅ = , but c o ≠ .
- Uniform helical motions are the superposition of a uniform rotation and a uniform translation parallel to the rotation's axis. They are characterized by 0 c c ⋅ ≠ .

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy

## 2.2. Envelope condition and characteristic curve

For our problem, we have to consider the solid S in the moving system and its boundary surface Φ . Let us first assume that Φ is smooth. Then, at any time instant t , we consider those points of Φ , whose velocity vector is tangent to Φ .  In  general,  the  set  of  these  so-called characteristic points is  a  curve,  called characteristic curve ( ) C t , see Fig.1. Note that ( ) C t may be composed of several connected components. If ( ) n x is a normal vector field of Φ , the curve ( ) C t is the zero level set of the function

<!-- formula-not-decoded -->

which is defined on Φ . For a line geometric interpretation of the envelope condition ( ) 0 G x = see [16]. The set of all positions 0 ( ) C t of characteristic curves in the fixed system is the complete envelope surface Γ of Φ under the given motion. However, only some parts of the surface Γ lie on the boundary of the swept volume. Our algorithm will at first compute Γ via  the  characteristic curves and later remove that part of Γ which is inside the swept volume.

Remark : At a time instant with c o = (instantaneous translation), the characteristic curve is the silhouette of Φ for  parallel  projection  in  direction c .  In  general,  however,  this  is  not  true  and  the  rotational  part  needs  to  be considered. We warn the reader that there is a number of papers in the literature, which consider any time instant as translational in the context of envelope computations.

Figure 1: Left: Velocity vectors in points of the characteristic curve of an object. Right:  Three positions of the moving object.

<!-- image -->

Let us briefly discuss the case where Φ is  given as a level set ( ) 0 F x = .  Since surface normals are given by the gradient field F ∇ of F , the characteristic curve also satisfies

<!-- formula-not-decoded -->

Note that the surface ( ) 0 G x = simultaneously contains the characteristic curves of all level sets ( ) . F x k const = = of F .

The positions of the characteristic curves in the fixed system can also be found as follows: Φ 's position in 0 Σ at time t is 0 0 0 0 ( ) : ( ( )) 0 T F x F A x a = ⋅ -= . From general envelope theory, the derivative of this equation with respect to t is  known to contain the characteristic curve. This derivative surface is precisely the position of the surface ( ) 0 G x = in 0 Σ .

We also get some insight into the case of a moving algebraic surface ( ) 0 F x = of degree n . Its positions 0 0 ( ) 0 F x = in 0 Σ are represented by a polynomial 0 0 ( ) F x with coefficients depending on t . The derivative with respect to t is  a  polynomial  of  degree n ≤ and  thus 0 ( ) C t is  by  Bezout's  theorem  an  algebraic  curve  of  degree 2 n ≤ .  This explains the well-known simplicity of the cases of a moving plane ( C is a straight line), a sphere ( C is a circle), a quadratic cylinder or cone ( C is a rational curve of degree 4 ≤ ), or a torus ( C is algebraic of degree 8 ≤ ; as for

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy the  sphere,  the  degree  reduction  is  caused  by  the  absolute  conic).  But  even  for  simple  freeform  surfaces,  the degree will become very high and thus algebraic methods seem to be infeasible for our purpose.

## 2.3. Non-smooth boundary surface or non-smooth motion

If the boundary surface Φ of S contains sharp edges and corners, the considerations above remain correct if we generalize the meaning of the normal vector n . In order to get a good feeling what we have to do at these places, one may consider them smoothed by a rolling ball blend of a very small radius.

At an edge point p , there are two different outside unit normals , l r n n at either side of it. The set of normals at p is then of the form (1 ) l l r r h n h n λ λ -+ with [0,1] h ∈ . The corresponding unit normals form an arc of a great circle on the unit sphere.

At  a  corner  point p ,  the  set  of  normals  is  even  two-dimensional;  it  is  defined  by  the  convex  cone  to  the  unit normals of the surface patches which meet at p . We see that the envelope condition becomes weaker at edges or corners so that those appear more likely as part of the characteristic curve.

If the motion is not smooth at some time instant 0 t ,  we may also consider a locally smoothed version. The leftsided derivative at 0 t t = yields a velocity field characterized by ( , ) l l c c , and on the right side we have ( , ) r r c c . This yields  a  one-parameter  family  of  vector  fields (1 )( , ) ( , ) l l r r h c c h c c -+ , [0,1] h ∈ .  It  has  to  be  considered  for  the computation of the characteristic set which is in general a two-dimensional subset of Φ . For an initial position of a  non-closed  motion,  we  define  all  points  with 0 G ≤ as  characteristic,  and  for  an  end  position  all  points  with 0 G ≥ . The entire surface Φ arises as characteristic set for an instant with ( , ) ( , ) c c o o = .

## 2.4 Evolution of the characteristic curves

In our algorithm we will sample the motion at discrete time instances 0 , , n t t K and compute the corresponding characteristic  sets 0 ( ) i C t in 0 Σ ,  see  Fig.2.  In  order  to  get  a  good  representation  of  the  envelope  surface,  the distances  between  adjacent  curves 0 ( ) i C t and 0 1 ( ) i C t + must  not  become  too  large.  In  other  words,  our  time sampling has to respect the evolution speed of the curves 0 ( ) C t . We will derive here an equation for the speed of this  evolution.  This  is  similar  to  results  on  the  evolution  of  apparent  contours  on  a  surface,  which  have  been studied extensively in Computer Vision [6].

Figure 2: Two views of the characteristic curves on the swept volume.

<!-- image -->

Let us first look into the moving system and consider - just for the sake of simplicity of the derivation - a local parametrization ( , ) s u t of Φ in  which  the  iso-parameter lines . t const = are  the  characteristic  curves ( ) C t and . u const = are  orthogonal  trajectories  of  those.  We  are  interested  in  the  evolution  speed,  which  is  measured

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy orthogonal to ( ) C t .  Thus, it is given by the vector / : t s t s r λ ∂ ∂ = = (with / t t r s s = ).  With ( , ) n u t as  unit normal vector field of Φ , the parametrization ( , ) s u t must satisfy condition Eqn. (3),

<!-- formula-not-decoded -->

Differentiation of this identity with respect to t yields,

<!-- formula-not-decoded -->

Let v denote an arc length parameter along the parameter line . u const = passing through the considered point ( , ) s u t . Then,

<!-- formula-not-decoded -->

The variation v n of the normal in direction r is given by

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Here, ω is the Weingarten map (shape operator) [7]. We use a right handed Cartesian system ( , , ) r h n at ( , ) s u t and solve Eqn. (6) for the evolution speed in the moving system λ ,

<!-- formula-not-decoded -->

In  the  numerator  we  have  the  component  of  the  acceleration  of s in  direction  of  the  surface  normal.  The denominator contains only first order information of the motion, namely components of the velocity ( ) v s and of the angular velocity vector c ; the directions , h r and ( ) r ω ,however, depend on second order surface information. Considering a point s which moves on Φ with the evolution velocity vector r λ , the corresponding velocity vector in the fixed system (but represented in the moving system) is the sum ( ) r v s λ + . Since we are again only interested in the component orthogonal to ( ) C t , we find for the magnitude of the evolution speed in the fixed system

<!-- formula-not-decoded -->

## 3. OUTLINE OF THE BASIC ALGORITHM

In  order  to  simplify  the  presentation,  we  first  consider  the  case  where  we  are  not  interested  in  eventually appearing holes inside the swept volume. Even if there are holes, a number of applications is not interested in them. How to handle holes will be outlined in section 7.

Our algorithm contains the following three basic steps, which will be explained in more detail in the following sections.

- At a number of time instances 0 , , n t t K ,  chosen with an appropriate time step control, we compute the respective  positions 0 ( ) i C t of  the  characteristic  set  in  the  fixed  system.  The  characteristic  set  is represented as a set of points plus surface normals. This results in a  set Q .
- Based on a marching algorithm in an underlying grid of the fixed system, we remove those points from Q which lie in the interior of the swept volume. This yields a new point set B on the boundary of the swept volume.
- We compute an approximating surface to the point set B , which respects the provided surface normals well.

For the detailed explanation we assume that the boundary surface Φ of the moving surface is given as a triangle mesh (with reliably estimated surface normals) or a parametric surface. We also assume that the motion can be evaluated at any time instant. If the motion is only given at a discrete number of positions, a motion interpolation scheme (see e.g. [10],[17]) might be required if the time steps are not satisfying our criteria.

## 4. COMPUTATION OF THE CHARACTERISTIC POINTS

In order to compute the characteristic set ( ) i C t at a given time instant i t , we set up the velocity vector field ( , ) c c in the moving system, and search for the zero level set of the function G (inner product between velocity vector

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy and surface normal) in Eqn.(3). It is sufficient to compute a sequence of points plus surface normals along ( ) i C t . The maximum distance of adjacent points along the same component of ( ) i C t shall be bounded by a prescribed value δ . Finally, the positions i q of characteristic points and associated surface normals in the fixed system 0 Σ are computed.

Of  course,  at  discontinuities  of  motion  or  surface,  we  have  to  compute  characteristic  points  according  to subsection 2.3.

Along  a  given  characteristic ( ) i C t we  estimate  the  maximum  evolution  velocity max v in  the  fixed  system  using Eqn.(10).  Then,  a  time  step 1 i i i t t t + ∆ = -is  chosen  such  that max i v t δ ∆ ≤ .  In  this  way  the  spacing  across characteristics is in general not much larger than the spacing of points along the characteristic. If one wants to avoid second order estimates arising in Eqn.(10), one can use the pairs of corresponding points 0 1 ( ) j i q C t --∈ and

0 ( ) j i q C t ∈ (such that j j q q --is orthogonal to 0 ( ) i C t ) to get an estimate of max v via

<!-- formula-not-decoded -->

If max j j q q --exceeds δ , we can reduce i t accordingly and recompute 0 ( ) i C t .

## 5. SELECTION OF POINTS ON THE BOUNDARY OF THE SWEPT VOLUME

The result of the previous step is a point set Q (plus surface normals) on the envelope surface Γ . Note that we do not care about holes inside a swept volume and thus we proceed as follows to compute the outer boundary surface of the swept volume.

The data point set Q is  placed in a rectangular grid whose cell size is larger than δ .  We compute the unsigned distance  function  for  the  grid  points  with  respect  to Q .  In  order  to  trim  parts  of Q to  obtain  the  trimmed envelope Γ we perform a fast marching method similar to that described in [11] on the grid points.

Roughly this works as follows: At the beginning all grid points are labeled as unknown and inside . We begin the loop by taking a point surely outside of Γ and initialize an active front by storing this point in it and label it as trial . We extract a point p from the active front and this point p is labeled as visited and outside . We collect all its unknown neighbors q on the grid. If the distance value of q is larger than some threshold τ , q is labeled as outside and trial and q is  added  to  the  active  front.  In  this  manner  the  front  propagates  on  the  grid  and  this procedure is performed until the active front is empty.

The active front stops at the outer boundary of the envelope Γ . We perform a nearest neighbor search for all those grid points close to Γ with  respect  to  the  point  set Q which is  actually  formed by points of the characteristic curves, see sections 2.2 and 2.4. This results in a relatively uniform sampling of Γ by a subset B of points of the set Q . We note that also the normal vectors of Γ at points of B are available.

Self intersections of the envelope Γ are smoothed out and are treated in a separate procedure (section 5.1).

## 5.1. Handling of edges and features

Sharp edges on the swept volume originate from vertices of the moving solid or are global self intersections of the envelope at its outer boundary. The first type of edges are constructed by tracking the vertices of the object during the motion and checking whether they belong to the outer boundary, see Fig. 5 and 6. The second type of edges have to be detected and are computed as intersections.

There are methods for the computation of intersections of point sets in the literature, see e.g. [2],[12]. We follow a different approach and additionally use a triangulation of the point set B of the trimmed envelope Γ .

The method for detection of sharp features on Γ is motivated by the following observations:

- The envelope Γ possesses a relatively uniform sampling by the point set B .

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy

- The distances of points of B do  not  vary  much  and  thus  the  size  of  triangles  and  the  length  of  their segments is nearly constant.

According to these observations we detect sharp features by investigating distances of adjacent points taking into account the variation of their normal vectors. Let p B ∈ an  arbitrary point and n its  (unit)  normal vector. We form the surface 6 ˆ B R ⊂ consisting of points ˆ ( , ) p p wn = . The canonical metric in 6 R induces a metric on ˆ B and this defines a feature sensitive metric on B . The value w controls the influence of the normal vectors. See [15] for more information concerning the feature sensitive metric.

Once  having  defined  the  feature  locations  on B we  form  connected  components  (on  the  triangulation)  and investigate  the  neighborhood  of  each  connected  component  (with  respect  to  the  triangulation)  of  the  sharp features. Each connected component is replaced by a spine curve which represents the shape of the sharp feature.

Then profile sections of the investigated neighborhoods, orthogonal to the spine curve are computed. For typical self intersections we can recognize different sides of the neighborhood of a sharp feature. The intersection points of the approximating lines of the profile sections are good estimates for the true self intersection. The intersection points are sorted and smoothed by a spline curve which serves as approximation of the sharp feature. After having inserted  a  regularly  sampled  sequence  of  points  of  each  of  these  spline  curves  into  the  data  set B ,  a  new triangulation is performed, see Fig. 3 and 4.

Note that there is an easy kind of sharp edges: they are (parts of) trajectories of corner points of the moving solid and do not require the present approach.

<!-- image -->

Figure 3: Right: Global self intersection of the swept volume. Left: Triangulation and the inserted points along the sharp edge of the detail.

## 6. APPROXIMATING THE BOUNDARY SURFACE OF THE SWEPT VOLUME

## 6.1. Approximating triangle mesh

Since the careful handling of features labels edges, we can perform a good triangulation. The edge points have already been assembled into polygons. So we just have to perform a triangulation of the point set which contains these edge polygons.

## 6.2. B-spline approximation

Especially  if  the  boundary  surface  of  the  swept  volume  has  a  simple  topology,  a  recently  developed  B-spline approximation scheme called squared distance minimization [14] can be used.

We do not recommend to use the full squared distance formula which also requires curvature estimates, but just the  corresponding  Gauss-Newton  iteration  that  is  based  on  surfels.  Fig.  7  displays  an  example,  where  the  self intersections are smoothed.

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy

## 6.3. Approximating point-set surfaces

The definition of the boundary of the swept volume with a set of points (surfels) allows us to apply recent work on point-set surfaces. For rendering, we need a disk radius of the surfels; it is derived from the sampling density δ as λδ with 1/ 3 λ &gt; .  If  we  did  not  recover  edges  on  the  boundary  of  the  swept  volume,  there  are  many  good methods in the literature, e.g. [2],[3],[12], to represent these data as a point-set surface (implicit surface). Since surfels are available, the use of corresponding  point-set  surfaces according  to Amenta  and  Kil  [3]  is recommended. In the presence of edges special care has to be taken, as outlined e.g. in [12].

Figure 4: Objects and their swept volume with respect to some motions.

<!-- image -->

## 7. EXTENSIONS

Our method can be extended in various ways. We list here only a few items and point to a forthcoming publication in which we will address these topics in detail.

- In order to resolve holes inside the swept volume, one can write information into the grid in 0 Σ already during the sweep through the motion. Basically, we have to mark grid cells which are at least partially covered by a position of the moving body. One way to do this is to sample Φ (can be coarser than the sampling used for the computation of characteristic points); this gives a set I . At any time instant i t , we

then compute all points x I ∈ with ( ) 0 G x &gt; and label those cells into which their positions fall.

- The main ideas of the method also work for an implicit representation of the boundary surface Φ of S . We are  currently  exploring  an  approach  which  derives  a  signed  distance  representation  of  the  swept volume boundary from a signed distance representation of Φ .
- The present approach can be extended to swept volumes generated by motions which depend on more than one parameter. For example, we may consider two-parameter motions. An important special case of

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy those,  namely  purely  translational  motions,  are  generating Minkowski  sums ,  which  form  the  basic building block of mathematical morphology [8].

Acknowledgements : Part of this research has been carried out within the Competence Center Advanced Computer Vision and has been funded by the Kplus program. This work was also supported by the Austrian Science Fund (FWF) under grant P16002-N05.

<!-- image -->

Figure 5: Object with edges and its swept volume with respect to a motion.

Figure 6: Object with edges and vertices and its swept volume with respect to a motion.

<!-- image -->

Figure 7: Left: B-spline approximation of a swept volume. Right: B-spline approximation superimposed with the triangulated boundary points of the swept volume.

<!-- image -->

Computer-Aided Design &amp; Applications, Vol. 2, Nos. 1-4, 2005, pp xxx-yyy

## 8. REFERENCES

- [1] Abdel-Malek, K., Blackmore, D. and Joy, K., Swept volumes: foundations, perspectives and applications, Intl. Journal of Shape Modeling , 2004.
- [2] Adamson, A. and Alexa, M., Approximating and Intersecting Surfaces from Points, Geometry Processing 2003 ,  Eurographics, pp 245-254, 2003.
- [3] Amenta, N. and Yong Joo Kil, Defining point-set surfaces, Proceedings of SIGGRAPH 2004, ACM Press / ACM SIGGRAPH, pp 264-270, 2004.
- [4] Blackmore,  D.,  Leu,  M.C.,  Wang,  L.P.  and  Jiang,  H.,  Swept  Volume:  A  Retrospective  and  Prospective View, Neural, Parallel and Scientific Computations , vol. 5, no 1, pp 81-102, 1997.
- [5] Bottema, O. and Roth, B., Theoretical Kinematics , Dover Publ., New York, 1990.
- [6] Cipolla, R. and Giblin, P., Visual Motion of Curves and Surfaces ,  Cambridge Univ. Press, Cambridge, UK, 2000.
- [7] do Carmo, M.P., Differential Geometry of Curves and Surfaces , Prentice-Hall, 1976.
- [8] Heijmans, H. J. A. M., Morphological Image Operators , Academic Press, Boston, 1994.
- [9] Hoschek, J., Zur Ermittlung von Hüllgebilden in der Kinematik , Dissertation, TH Darmstadt, 1964.
- [10] Jüttler,  B.  and  Wagner,  M.,  Kinematics  and  Animation, Handbook  of  Computer  Aided  Geometric Design , Farin, G., Kim, M.S., and Hoschek, J.(eds.), Elsevier, pp 723-748, 2002.
- [11] Kim,  Y.J.,  Varadhan,  G.,  Lin,  M.C.  and  Manocha,  D.,  Fast  swept  volume  approximation  of  complex polyhedral models, Computer Aided Design , 36, pp 1013-1027, 2004.
- [12] Pauly,  M.,  Keiser,  R.,  Kobbelt,  L.,  and  Gross,  M.,  Shape  modeling  with  point-sampled  geometry, Proceedings of SIGGRAPH 2003, ACM Press / ACM SIGGRAPH, pp 641-650, 2003.
- [13] Pottmann,  H.  and  Leopoldseder,  S.,  Geometries  for  CAGD, Handbook of  Computer  Aided  Geometric Des ign, Farin, G., Kim, M.S., and Hoschek, J. (eds.), Elsevier, pp 43-73, 2002.
- [14] Pottmann,  H.  and  Leopoldseder,  S.,  A  concept  for  parametric  surface  fitting  which  avoids  the parametrization problem, Computer Aided Geometric Design , 20, pp 343-362, 2003.
- [15] Pottmann,  H.,  Steiner,  T.,  Hofer,  M.,  Haider,  C.,  and  Hanbury,  A.,  The  isophotic  metric  and  its application to feature sensitive morphology on surfaces, Computer Vision - ECCV 2004, Part IV, Lecture Notes in Computer Science , vol 3024, Pajdla, T. and Matas, J., Springer (eds), pp 560-572, 2004.
- [16] Pottmann, H. and Wallner, J., Computational Line Geometry ,  Springer,  Berlin-Heidelberg-New  York, 2001.
- [17] Wallner,  J.  and  Pottmann,  H., Intrinsic  subdivision  with  smooth  limits  for  graphics  and  animation , Geometry Preprint N0. 120, TU Wien, 2004.
- [18] Yang, J. and Abdel-Malek, K., Approximate swept volumes of NURBS surfaces or solids, Computer Aided Geometric Design 22, 1-26, 2005.