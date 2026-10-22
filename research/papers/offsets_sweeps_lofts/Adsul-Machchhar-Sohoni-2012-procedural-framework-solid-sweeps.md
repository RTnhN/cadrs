## Aprocedural framework and mathematical analysis for solid sweeps

Bharat Adsul, Jinesh Machchhar, Milind Sohoni

Department of Computer Science and Engineering, Indian Institute of Technology Bombay, Mumbai-400076, India

## Abstract

Sweeping is a powerful and versatile method of designing objects. Boundary of volumes (henceforth envelope) obtained by sweeping solids have been extensively investigated in the past, though, obtaining an accurate parametrization of the envelope remained computationally hard. The present work reports our approach to this problem as well as the important problem of identifying self-intersections within the envelope. Parametrization of the envelope is, of course, necessary for its use in most current CAD systems. We take the more interesting case when the solid is composed of several faces meeting smoothly. We show that the face structure of the envelope mimics locally that of the solid. We adopt the procedural approach at defining the geometry in this work which has the advantage of being accurate as well as computationally efficient. The problem of detecting local self-intersections is central to a robust implementation of the solid sweep. This has been addressed by computing a subtle mathematical invariant which detects self-intersections, and which is computationally benign and requires only point queries.

Key words: Sweeping, swept surface, self-intersections, procedural surfaces

## 1. Introduction

In this paper we focus on the problem of computing an accurate parametrization of the boundary of the volume obtained by sweeping a solid in R 3 along a trajectory and that of detecting local self-intersections in the envelope. Sweeping is an operation of fundamental importance in geometric design. It has applications like numerically controlled machining verification [3,4,17] and robot motion planning [23,24]. There have been several approaches to computing the boundary of swept volumes in the past. The works [6,21] formulate the problem using the rank deficiency of the Jacobian, [3-5] compute the envelope by solving sweep differential equations, [20] uses inverse-trajectories for deriving a point membership test for a point to belong to the envelope. In [22] the authors give a close approximation of the envelope by restricting the trajectories to piecewise screw motions.

Despite the extensive research done in the past in this area, computing an accurate parametrization of the envelope has remained an unsolved problem due to known mathematical and computational difficulties [15]. In this work we attempt to arrive at an accurate parametrization of the envelope through the procedural approach, which is an abstract way of defining surfaces and curves when closed form formulae are not available. The procedural paradigm exploits the fact that from the users point of view, a para- metric surface is just a map from R 2 to R 3 and hence can be represented in a computer by a procedure which takes as input ( u, v ) ∈ R 2 and returns ( x, y, z ) ∈ R 3 . Higher order derivatives of the surface can be returned similarly. The definition of splines through the De Casteljau's algorithm is an example of procedural parametrization. The authors in [14] compute the intersection curve of two parametric surfaces by procedural approach.

The second problem that we tackle in this paper is that of detecting local self-intersections. Self-intersections cause anomalies in the envelope. There have been rather few attempts at solving this problem in the past. The paper [2] proposes an efficient and robust method of detecting global and local self-intersections by checking whether the inversetrajectory of a point intersects the solid. In the paper [7] global and local self-intersections are detected by computing intersection of curves of contact at discrete time steps. This has the disadvantage of being computationally expensive. In the paper [18] self-intersections are accurately quantified and detected but their method is limited to sweeping tools for NC machining verification. The method employed by [25] for detecting local self-intersections is based on point set data and could be computationally expensive. In [9] the author solves the problem of detecting local selfintersections for sweeping planar profiles. In this paper we propose a novel test for detecting local self-intersections which is based on a subtle mathematical invariant of the envelope. It has the advantage of being computationally efficient and requires only point queries.

The paper is organized as follows. In Section 2 we describe the input to the sweeping algorithm, in Section 3 we discuss the overall framework for the computation of the envelope, in Sections 4 and 5 we study the mathematical structure of the envelope and quantify local selfintersections, giving a test for detecting them, in Section 6 we analyse the case when the envelope is free from selfintersections. In Section 7 we describe the algorithm for computing the procedural parametrization of the envelope. We conclude the paper in Section 8.

## 2. Preliminaries

This section outlines the basic representational structures associated with the problem. Subsection 2.1 describes the boundary representation of a solid which is typical to many CAD systems and subsequent sections, the basic inputs and outputs of the sweep algorithm. In Subsection 2.2 we define the trajectory in R 3 along which the solid is swept. Next, in subsection 2.3 we define the various mathematical sub-entities which make up the envelope.

## 2.1. Boundary representation of a solid

Boundary representation, also known as Brep, is a popular and standard method of representing a 'closed' solid M by its boundary ∂M . The boundary ∂M separates the interior of M from the exterior of M . ∂M is represented using a set of faces , edges and vertices . See figure 1 for a Brep of a solid where different faces are coloured differently. Faces meet in edges and edges meet in vertices. The Brep of a solid consists of two interconnected pieces of information, viz. the geometric and the topological.

̸

Geometric information: This consists of geometric entities, namely, vertices, edges and faces. A vertex is simply a point in R 3 . An edge is obtained by restricting the underlying parametric curve by a pair of vertices. A parametric curve in R 3 is a continuous map γ : R → R 3 . The curve γ is called regular at s 0 ∈ R if γ is differentiable and dγ ds | s 0 = ¯ 0. Here s is the parameter of the curve. An edge is derived from the underlying curve by suitably restricting the parameter s to an interval [ a, b ]. Further, it is required that the edge (more precisely, the underlying curve) is regular at all points in the interval [ a, b ] and devoid of self-intersections.

Similarly, a face is obtained by restricting the underlying parametric surface by a set of edges. A parametric surface is a continuous map S : R 2 → R 3 . The surface S is said to be regular at ( u 0 , v 0 ) ∈ R 2 if S is differentiable and ∂S ∂u | ( u 0 ,v 0 ) ∈ R 3 and ∂S ∂v | ( u 0 ,v 0 ) ∈ R 3 are linearly independent. Here u and v are the parameters of S . A face is derived from a surface by suitably restricting the parameters u and v inside a 'domain'. As expected, it is required that the face is regular at all points in the domain and devoid of self-intersection.

Fig. 1. A Brep of a solid

<!-- image -->

Topological information: Thetopological/combinatorial information consists of spatial relationships between different geometric entities, i.e., the adjacency between faces, the incidence relationships between faces and edges and so on. In figure 1, for example, the orange and the green face are adjacent. Another important component is the orientation for each face, that is, a consistent choice of outward-normal for that face. The orientation of a regular face is a choice of a unit normal from amongst S u × S v ‖ S u × S v ‖ and -S u × S v ‖ S u × S v ‖ where S is the underlying parametric surface. All the faces bounding the solid are oriented so that the unit normal at each point on each face is pointing towards the exterior of the solid.

Conceptually, a Brep through its 'global' topological information glues the 'local' geometric entities which come equipped with associated mathematical parametrizations. Note that, the regularity assumptions on the geometric entities guarantee that the tangent space at every point on an edge or a face is of the right dimension. Typically, one also imposes higher-order 'parametric' continuity requirements which are denoted by C k where k refers to the order of continuity. For the sake of simplicity, throughout this paper, we will assume that the edges and faces bounding the solid are regular of class C k for some k ≥ 2, i.e. the underlying parametrizations are twice differentiable with continuous second order derivatives. Note that, however, these do not rule out, e.g., adjacent faces meeting along sharp edges.

## 2.2. A trajectory in R 3

A trajectory in R 3 is a 1-parameter family of rigid motions in R 3 defined as follows.

Definition 2.1 A trajectory in R 3 is specified by a map h : [0 , 1] → ( SO (3) , R 3 ) , h ( t ) = ( A ( t ) , b ( t )) where A ( t ) ∈ SO (3) 1 , b ( t ) ∈ R 3 , A (0) = I, b (0) = 0 . The parameter t in this definition represents time.

For technical convenience, we assume that h is of class C k , for some k ≥ 2.

1 SO (3) = { X is a 3 × 3 real matrix | X t X = I, det ( X ) = 1 } is the special orthogonal group, i.e. the group of rotational transforms.

## 2.3. Boundary of the swept volume

Webegin by giving an intuitive description of the boundary of the swept volume. We will formalize these notions in Section 4. Let M be a solid being swept along a given trajectory h . By abuse of notation, a point in M will mean a point in the interior of M or on the boundary ∂M of M . We denote by M t the position of M at time t ∈ [0 , 1], i.e. M t = { A ( t ) x + b ( t ) | x ∈ M } , and by ∂M t , the boundary of M t . Then ⋃ t ∈ [0 , 1] M t is the volume swept by M during

this operation. Our goal is to compute the boundary of this swept volume as a Brep, which we will refer to as the envelope . For a fixed point x ∈ M , consider the trajectory of x as the map y : [0 , 1] → R 3 given by y ( t ) = A ( t ) x + b ( t ). The trajectory of x describes the motion x in R 3 under the given trajectory h . Clearly, if x is in the interior of M , no point in the image of the trajectory of x can be on the envelope. Further, at a particular time instant t 0 , only a subset of points on ∂M t 0 will lie on the envelope. The union of such points for all t 0 ∈ [0 , 1] gives the final envelope.

It is clear that, at a given time instant t 0 , only a part of ∂M t 0 is in 'contact' with the envelope. To make this more clear, fix a point x ∈ ∂M and the trajectory y of x . The derivative of the trajectory of x at a given time instant t 0 , that is, dy dt | t 0 gives the velocity of x at t 0 . It is easy to show that (cf, Section 4) x (more precisely, y ( t 0 )) is in contact with the envelope at time t 0 only if the velocity of x at t 0 is in the 'tangent-space' of ∂M t 0 at y ( t 0 ). In a generic situation, the set of points of ∂M t 0 which are in contact with the envelope will be the curve-of-contact. The union of these curves-of-contact is called the contact-set or the running envelope . Clearly, the total envelope and the contact-set are closely related. If all goes well, the envelope is obtained from the contact-set by 'capping' it by appropriate parts of M 0 and M 1 , the object at times t = 0 and t = 1. But all may not go well. The detection of anomalies is central to the use of the algorithm in industrial situations and is an important objective of this paper.

## 3. Our Approach/Framework

In this section we briefly describe the overall framework for computing the sweep surface i.e. the envelope as a Brep. We continue using the notation from the previous section where M denotes the Brep/solid and h denotes the trajectory along which M is swept.

The naive approach to computing the envelope would be to discretize time, i.e, to construct a sequence T = { 0 = t 1 , . . . , t k = 1 } and constructing the approximate envelope as E ′ = ∪ i M t i , the union of the translates. The next step would be to construct a smooth version E ′′ of E ′ above, by some fitting operation. However, this approach has several issues-(i) computation of E ′ leads to unstable booleans of two very close-by objects, leading to sliver-faces, and (ii) the fit of E ′′ to the actual E depends on a dense enough choice of T which compounds problem (i) above. There are other options, but problems remain.

In this work, we propose a novel approach based on the procedural paradigm (cf [11,14]) which has gained ascendance in many numerical kernels, e.g., ACIS (cf [12]).

Wenowdescribe the basic architecture for our algorithm. For this, we use a running example referred to in figure 1 and figure 2. The object to be swept is M as in figure 1, and the output contact-set is C as shown in figure 2. The trajectory is roughly helical with a compounded rotation.

- (i) A natural correspondence between the entities of M and the entities of C .

Every point p of the envelope comes from a curve of contact on M t , for some t , and therefore belongs to some entity of M , i.e., a vertex, edge or face. This sets up the correspondence between entities of M and those of C . The procedural approach attaches a common evaluation method to each such entity. Fig2 illustrates this correspondence. Faces of C which are generated by a particular face of M are shown in same colour. Curve-of-contact at time t = 0 is shown imprinted on the solid in red.

Along with the geometric definition of each entity, we must also construct the topological data to go with it. This data is constructed by observing that there is a local homeomorphism between a point on C and a suitable point on M .

- (ii) Accurate parametrizations of the geometric entities of C with 'time' as one of the central parameters.

This is achieved through the procedural paradigm in which all key attributes/features of the geometric entities are made available through a set of associated procedures (cf [11,14]). In our case, these procedures are based on Newton-Raphson solvers. This is the focus of Section 7.

- (iii) Topological and regularity analysis of C .

It is quite common to have a sweeping operation in which the resulting envelope/contact-set C selfintersects. These self-intersections can be broadly classified into global and local self-intersections (see figure 6). Once an accurate 'local' parametrization (as in step 2) of the faces of C is obtained, in principle, global self-intersections can be detected and dealt with by well-known (cf [14]) surface-surface intersection solvers. A more subtle mathematical issue is that of detecting singularities and local selfintersections. This is addressed in Sections 4 and 5.

We are now in a position to define the scope of this paper. In this work we describe in detail tasks (ii) and (iii) described above, namely, detecting local self-intersections and obtaining a procedural parameterization of faces. The focus of Section 6 is task (i) in the interesting case when M is composed of faces meeting smoothly. Other architectural aspects and solid-modelling implementation will be addressed in a later work.

Fig. 2. A solid is swept along a helical trajectory

<!-- image -->

## 4. Mathematical structure of the contact-set

In this section we will study in detail the mathematical structure of the boundary of the volume obtained by sweeping the solid M along the trajectory. For simplicity, we work with a single parametric surface patch S and analyse the sweep of S under the trajectory h . As explained before, we assume that both S and h are regular of class C k for k ≥ 2, and are devoid of self-intersections. In section 6, we lift the results of this section to the interesting case when M is composed of several faces/surfaces meeting smoothly. For later use, we introduce the following notation: the tanget space to a manifold X at a point p ∈ X will be denoted by T X ( p ).

We begin with the formal definition of the sweep map. Definition 4.1 Given S and h , the sweep is defined as a map σ : R 2 × [0 , 1] → R 3 given by σ ( u, v, t ) = A ( t ) S ( u, v )+ b ( t ) .

Here u, v are the parameters of S . The position of a point S ( u 0 , v 0 ) on surface S at time t 0 will be given by σ ( u 0 , v 0 , t 0 ) = A ( t 0 ) S ( u 0 , v 0 )+ b ( t 0 ) and the velocity of the point S ( u 0 , v 0 ) at time t 0 will be given by V ( u 0 , v 0 , t 0 ) = ∂σ ∂t | ( u 0 ,v 0 ,t 0 ) = A ′ ( t 0 ) S ( u 0 , v 0 ) + b ′ ( t 0 ), where ′ denotes derivative with respect to t . If N ( u 0 , v 0 ) is the unit (outward) normal to S at ( u 0 , v 0 ), then the unit normal to S t 0 at ( u 0 , v 0 ) is given by ˆ N = A ( t 0 ) N ( u 0 , v 0 ) where, S t 0 = { A ( t 0 ) S ( u, v ) + b ( t 0 ) | ( u, v ) ∈ R 2 } is the position of the surface at time instant t 0 . In order to formally define the contact-set, we look at the extended sweep in R 4 in which the fourth dimension is time [2].

Definition 4.2 Given S and h , the extended sweep is defined as a map ˆ σ : R 2 × [0 , 1] → R 4 given by ˆ σ ( u, v, t ) =

( σ ( u, v, t ) , t ) .

Thus, the sweep σ is clearly the extended sweep ˆ σ composed with the projection map along the t -dimension. Denoting partial derivatives using a subscript, we note that ˆ σ u , ˆ σ v and ˆ σ t are linearly independent for all ( u, v, t ) ∈ R 2 × [0 , 1] and ˆ σ is injective. Hence the image of ˆ σ is a 3dimensional manifold. We now define the contact-set.

Definition 4.3 Given S and h , the contact-set is the set of points σ ( u 0 , v 0 , t 0 ) such that the line { ( x 0 , y 0 , z 0 , t ) ∈ R 4 | σ ( u 0 , v 0 , t 0 ) = ( x 0 , y 0 , z 0 ) , t ∈ [0 , 1] } is tangent to (the image of) ˆ σ at ˆ σ ( u 0 , v 0 , t 0 ) = ( x 0 , y 0 , z 0 , t 0 ) . We will denote the contact-set by C .

Wewill refer to the domain of the map σ as the parameter space and the co-domain as the object space . Consider now the following function f : R 2 × [0 , 1] → R given by

<!-- formula-not-decoded -->

Recalling that V ( u, v, t ) is the velocity of the point S ( u, v ) at time t and ˆ N ( u, v, t ) is the normal to S t at ( u, v ), we look at the zero-set of this function in the parameter space.

Definition 4.4 The funnel F is defined as the zero-set of the function f specified in Eq. 1, i.e., F = { ( u, v, t ) ∈ R 2 × [0 , 1] | f ( u, v, t ) = 0 } .

In other words, if a point p = ( u 0 , v 0 , t 0 ) ∈ F , then the velocity at the point σ ( p ) lies in the tangent space of S t 0 at σ ( p ). The following lemma shows that the contact-set is precisely the image of the funnel through the sweep map. Lemma 4.1 σ ( F ) = C

<!-- formula-not-decoded -->

Hence S ( u, v ) 'contributes' a point (namely, A ( t ) S ( u, v )+ b ( t )) to the contact-set at time t iff the triplet ( u, v, t ) satisfies the following condition.

<!-- image -->

sis can be easily extended to situations where these further assumptions do not hold.

An important consequence of the assumption 4 is that F is a 2-dimensional manifold and, hence, T F ( p ) is 2dimensional at all points p ∈ F . Observe that F is also orientable as ∇ f provides a continuous non-vanishing normal.

-§ -C-≀-\-⊔-⊣-⌋-⊔-↖-∫-⌉-⊔ Thus, F is topologically nice and regular, However, quite often, C has 'anomalies' which arise due to selfintersections. One of the main contributions of this paper is a subtle, efficiently computable mathematical function on F which allows to identify points on F which give rise to anomalies in C .

Fig. 3. Funnel and contact-set

<!-- formula-not-decoded -->

For a fixed t , Eq. 2 is a system of one equation in two variables u and v , hence, in a generic situation, the solution will be a curve.

Eq. 2 can also be looked upon as the rank deficiency condition [6] of the Jacobian J σ of the map σ defined in 4.1. To make this precise, let

<!-- formula-not-decoded -->

where σ u | ( u,v,t ) = A ( t ) ∂S ∂u ( u, v ) and σ v | ( u,v,t ) = A ( t ) ∂S ∂v ( u, v ) and σ t | ( u,v,t ) = V ( u, v, t ). Observe that regularity of S ensures that J σ has rank at least 2. Further, it is easy to show that f ( u, v, t ) is a non-zero scalar multiple of the determinant of J σ . Therefore, Eq. 2 is precisely the rank deficiency condition of the Jacobian of σ .

Note that, for a point ( u 0 , v 0 , t 0 ) ∈ R 3 , the Jacobian J σ | ( u 0 ,v 0 ,t 0 ) is a map from the tangent space to the ambient parameter-space at ( u 0 , v 0 , t 0 ) to the tangent space to the ambient object space at σ ( u 0 , v 0 , t 0 ). As already noted, if ( u 0 , v 0 , t 0 ) ∈ F , then J σ | ( u 0 ,v 0 ,t 0 ) is rank-deficient and maps the 3-dimensional ambient tangent space at ( u 0 , v 0 , t 0 ), (surjectively) onto, a 2-dimensional subspace of the ambient tangent space at σ ( u 0 , v 0 , t 0 ).

The subset of F for a fixed value of t will in general be a curve and will be referred to as the pcurve-of-contact at time t and its image through σ will be a subset of C which will be referred to as the curve-of-contact since it is essentially the set of points on the surface S where S makes tangential contact with C at time t . The union of such curvesof-contact for all t gives the contact-set C . The curve-ofcontact at t will be denoted by C t and the pcurve-of-contact will be denoted by c t . Fig. 3 schematically illustrates the funnel and the contact-set.

Before proceeding further, we make the following the non-degeneracy assumption 2 that:

̸

<!-- formula-not-decoded -->

̸

Further, for ease of discussion, we assume that (i) F is connected, and (ii) ∀ ( u, v, t ) ∈ F , ( f u , f v ) = (0 , 0) Our analy- The key to our analysis ahead, is the restriction of the sweep map σ to F . We will abuse the notation, denote this restriction, again by σ . So, σ : F → C . Now, fix a point p = ( u, v, t ) ∈ F and let q = σ ( p ) ∈ C . Since det ( J σ ( p )) = 0, { σ u ( p ) , σ v ( p ) , σ t ( p ) } are linearly dependent. As S is regular, the set { σ u ( p ) , σ v ( p ) } forms a basis for the tangent space to S t . Therefore, we must have σ t = lσ u + mσ v where l and m are well-defined (unique) and are themselves continuous functions of u, v and t .

2 Examples where this does not hold are (i) a cylinder being swept along its axis (ii) a planar face being swept in a direction orthogonal to its normal. Such cases can be separately and easily handled.

Clearly, ( σ u ( p ) , σ v ( p )) is a natural (ordered) 2-frame in the object space at point q (recall that, q = σ ( p )). Further, let X ( p ) be any ordered continuous 2-frame (basis) of the tangent space T F ( p ). Note that, this 2-frame is in the parameter space and is associated to the point p . Now, through σ , more precisely, J σ ( p ), the frame X ( p ) can be transported to another natural 2-frame σ ( X ( p )) in the object space at the point q . The determinant of the linear transformation connecting these two natural frames at q , namely (i) ( σ u ( p ) , σ v ( p )) and (ii) σ ( X ( p )) is the key to the subsequent analysis. As we show later, this determinant is a positive scalar multiple of the continuous function θ : F → R defined as follows.

<!-- formula-not-decoded -->

Here p = ( u, v, t ) and f u , f v and f t denote partial derivatives of the function f w.r.t. u, v and t respectively at p , and l and m are as defined before. Note that θ is easily and robustly computed.

We state an important result which we will prove in the coming sections:

Theorem 4.1 The function θ is such that (i) θ ( p ) &lt; 0 indicates that p is a point of local self intersection as defined by most authors, (see [2,7]) and (ii) θ ( p ) = 0 is where the rank of J σ ( T F ( p )) &lt; 2 , and finally (iii) excision of the region { p | θ ( p ) ≤ 0 } from the funnel F simplifies the construction of the envelope.

## 4.1. A particular frame for T F

̸

Let p = ( u, v, t ) ∈ F . In this section, we compute a natural 2-frame X ( p ) in T F ( p ). Note that, F being the zero level-set of the function f defined in Eq. 1, ∇ f | p ⊥T F ( p ). We set β = ( -f v , f u , 0) = 0 and note that β ⊥∇ f . It is easy to see that β is tangent to the pcurve-of-contact c t . Let α = ∇ f × β = ( -f u f t , -f v f t , f 2 u + f 2 v ). Here × is the cross-product in R 3 . Clearly, the set { α, β } forms a basis of T F ( p ).

Fig. 4. A sphere rotating about an axis which is tangent to itself. There is type-1 L.S.I. but no type-2 L.S.I.

<!-- image -->

Figure 3 illustrates the basis { α, β } schematically. Observe that β is tangent to the pcurve-of-contact at time t and α points towards the 'next' pcurve-of-contact.

## 4.2. The determinant connecting the two frames

Wecontinue with the notation developed earlier. We have α = ( -f t f u , -f t f v , f 2 + f 2 ) and β = ( -f v , f u , 0). Hence,

<!-- formula-not-decoded -->

u v 2 2

So, { J σ α, J σ β } can be expressed in terms of { σ u , σ v } as follows

<!-- formula-not-decoded -->

Note that,

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

## 4.3. Singularities of C

In this subsection, we propose an efficient test for detecting singularities on the contact-set. See Fig. 4 for an example. Clearly, the detection of singularities is important in practice.

We start with the following definition.

Definition 4.5 We say that the sweep causes a singularity if the composite map F σ →C ↪ → R 3 fails to be an immersion. In other words, the sweep causes a singularity if ∃ p ∈ F such that the rank of J σ ( p )( T F ( p )) is less than 2. Following the standard usage [8], in this case we say that the point p is a critical point.

Lemma 4.2 A point r ∈ F is a critical point iff θ ( r ) = 0 iff the rank of J σ ( r )( T F ( r )) is less than 2.

Fig. 6. Global and local self-intersection

<!-- image -->

Proof. By equation 7, we have det( D ( r )) = 0 iff θ ( r ) = 0. Recall that, as shown earlier, { α ( r ) , β ( r ) } is a basis of T F ( r ), and { σ u ( r ) , σ v ( r ) } is also a basis of the 2-frame associated at σ ( r ). As D ( r ) is the matrix expressing X = { J σ ( r )( α ( r )) , J σ ( r )( α ( r )) in terms of { σ u ( r ) , σ v ( r ) } , det( D ( r )) = 0 iff rank of J σ ( r )( T F ( r )) is less than 2. Thus, r is a critical point iff θ ( r ) = 0 iff the rank of J σ ( r )( T F ( r )) is less than 2. □

Note that the above lemma proves part (ii) of theorem 4.1.

Lemma 4.3 A sweep causes a singularity if there exists points p and q on F such that θ ( p ) ≤ 0 and θ ( q ) ≥ 0 .

Proof. As θ is a continuous function on F , the existence of p and q on F with the required properties implies existence of another point r ∈ F such that θ ( r ) = 0. By the previous lemma, this implies that the sweep causes a singularity. □

Theabove lemma leads to a computationally efficient test for detecting singularities: namely, evaluating θ at sampled points on F and checking if it changes sign on F .

The analysis done so far helps us detect singularities on the contact-set. In the next section we will perform a detailed analysis of local self-intersections which is topological in nature. Towards this, note that all points in the noncritical set may not lead to points on the envelope of the swept volume. For some p = ( u 0 , v 0 , t 0 ) ∈ F , σ ( p ) may lie in the interior of the solid M t of which the surface patch S t is a part of, for some t in neighbourhood of t 0 . In that case σ ( p ) will not be on the envelope. Fig. 5 shows two sweeping examples with self-intersections. Curves-of-contact at a few time instances are shown. In the next section we focus on identifying such points.

## 5. Topological and regularity analysis of C

Quite often, the anomalies on C arise due to selfintersections. If C has self-intersections, it needs to be trimmed to obtain the envelope of the swept volume [2]. Self-intersections can be broadly be classified into global and local. Fig. 6 illustrates the difference between global and local self-intersections schematically.

If there are only global self-intersections occurring on C , the composite map F σ → C ↪ → R 3 fails to be an injection. However, it is an immersion (see [8]), i.e. ∀ p ∈ F , the rank of J σ ( p )( T F ( p )) is 2. In principle, global self-intersections can be detected by surface-surface intersection. (see [14])

Fig. 5. Examples of local self-intersection showing curves-of-contact at few time instances: (a) A cylinder with blended edges undergoing translation and rotation about x -axis (b) An ellipsoid undergoing translation along a curvilinear path

<!-- image -->

The case of local self-intersection is more subtle as it leads to singularities on C . Clearly, the detection of local self-intersections is also central to a robust implementation of the solid sweep in CAD systems.

In literature [2,7], local self-intersections have been quantified by looking at points in the contact-set which lie in the interior of the solid M t for some time instant t . Clearly, such a point cannot be on the envelope of the swept volume. This approach was used in [2] for detecting local and global selfintersections, where the authors used implicit representation of the surface bounding the solid which is being swept. We adapt this concept to when the surface of the solid is represented parametrically. We will refer to this type of local self-intersection as type-2 L.S.I. . It turns out that type2 L.S.I. is intimately related to the analysis carried out in the previous section. To make this connection precise, we first introduce another type of local self-intersection. For lack of a better name, this is called as type-1 L.S.I.

## 5.1. Type-1 local self-intersections

Definition 5.1 A type-1 L.S.I. is said to occur at a point p ∈ F if θ ( p ) ≤ 0 .

Thus, Type-1 L.S.I is our classification of a local self intersection. We will see in subsection 5.3 that a for a Type1 L.S.I point p , the image σ ( p ) does not lie on the envelope of the swept volume.

## 5.2. Type-2 local self-intersection

In order to define type-2 L.S.I. we first describe the inverse trajectory corresponding to a given trajectory [2,20].

Given a trajectory as in definition 2.1 and a fixed point x in object-space, we would like to compute the set of points in the object-space which get mapped to x at some time instant. This set can be computed through the inverse trajectory defined as follows.

Definition 5.2 Given a trajectory h , the inverse trajectory ¯ h is defined as the map ¯ h : [0 , 1] → ( SO (3) , R 3 ) given by ¯ h ( t ) = ( A t ( t ) , -A t ( t ) b ( t )) .

Thus, for a fixed point x ∈ R 3 , the inverse trajectory of x is the map ¯ y : [0 , 1] → R 3 given by ¯ y ( t ) = A t ( t )( x -b ( t )). The range of ¯ y is { A t ( t ) x -A t ( t ) b ( t ) | t ∈ [0 , 1] } = { z ∈ R 3 |∃ t ∈ [0 , 1] , A ( t ) z + b ( t ) = x } . We will denote the trajectory of x by y : [0 , 1] → R 3 , y ( t ) = A ( t ) x + b ( t ). We now note a few useful facts about inverse trajectory of x . We assume without loss of generality that A ( t 0 ) = I and b ( t 0 ) = 0. Denoting the derivative with respect to t by˙, we have

<!-- formula-not-decoded -->

Since A ∈ SO (3) we have,

<!-- formula-not-decoded -->

Differentiating Eq. 9 w.r.t. t we get

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Differentiating Eq. 10 w.r.t. t we get

<!-- formula-not-decoded -->

Using Eq. 8 and Eq. 11 we get

<!-- formula-not-decoded -->

Differentiating Eq. 8 w.r.t. time we get

<!-- formula-not-decoded -->

Using Equations 14, 11 and 12 we get

<!-- formula-not-decoded -->

We now define type-2 L.S.I. The surface S is the boundary of the solid M being swept. We will refer to the interior of M by Int ( M ) and exterior of M by Ext ( M ).

Definition 5.3 A type-2 L.S.I is said to occur at a point ( u 0 , v 0 , t 0 ) if the inverse trajectory of the point σ ( u 0 , v 0 , t 0 ) intersects Int ( M t 0 ) .(see [2])

Fig. 7. Type-2 local self-intersection

<!-- image -->

Fig. 7 illustrates type-2 L.S.I. schematically where ¯ y is the inverse trajectory of the point x ∈ S t 0 and π is the projection of ¯ y on S t 0 . Suppose the point x = σ ( u 0 , v 0 , t 0 ) ∈ C . Let λ ( t ) be the signed distance of ¯ y ( t ) from surface S t 0 . If the point ¯ y ( t ) is in Int ( M t 0 ), Ext ( M t 0 ) or on the surface S t 0 , then λ ( t ) is negative, positive or zero respectively. Then we have ¯ y ( t ) -π ( t ) = λ ( t ) N ( t ), where π ( t ) is the projection of ¯ y ( t ) on S t 0 along the unit outward pointing normal N ( t ) to S t 0 at π ( t ). Then, the following relation holds for λ .

<!-- formula-not-decoded -->

Wenowgive a necessary and sufficient condition for type2 L.S.I. to occur.

Lemma 5.1 Type-2 L.S.I. occurs at a point p = ( u 0 , v 0 , t 0 ) ∈ F if and only if either of the following conditions hold

- (i) ¨ λ ( t 0 ) = 〈 -¨ σ +2 ˙ AV,N 〉 + κv 2 &lt; 0 where κ is the normal curvature of S t 0 at ( u 0 , v 0 ) along velocity V ( p ) , N is the unit length outward pointing normal to S t 0 at ( u 0 , v 0 ) and v 2 = 〈 V ( p ) , V ( p ) 〉 .
- (ii) λ ( t ) is negative for some t in some nbd of t 0 .

Remark : The statement of the above lemma (except for the insightful expression of ¨ λ ( t 0 )) is similar in spirit to the key Theorem-2 in [2] in the context of implicitly defined solids.

Proof. Differentiating Eq. 16 with respect to time and denoting derivative w.r.t. t by˙, we get

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

At t = t 0 , ¯ y ( t 0 ) = π ( t 0 ). Since ˙ y ( t 0 ) = V ( p ) ⊥ N ( p ), it follows from Eq. 13 that ˙ ¯ y ( t 0 ) ⊥ N ( p ). It is easy to verify that ˙ π ( t 0 ) = ˙ ¯ y ( t 0 ). Hence,

<!-- formula-not-decoded -->

From Eq. 18 and Eq. 15 it follows that

<!-- formula-not-decoded -->

Since π ( t ) ∈ S t 0 ∀ t in some neighbourhood U of t 0 , we have that 〈 ˙ π ( t ) , N ( t ) 〉 = 0 , ∀ t ∈ U . Hence 〈 ¨ π ( t ) , N ( t ) 〉 + 〈 ˙ π ( t ) , ˙ N ( t ) 〉 = 0 , ∀ t ∈ U . Hence -〈 ¨ π ( t 0 ) , N ( t 0 ) 〉 =

〈 ˙ π ( t 0 ) , ˙ N ( t 0 ) 〉 = 〈 ˙ π ( t 0 ) , G ∗ ( ˙ π ( t 0 )) 〉 = 〈 ˙ y ( t 0 ) , G ∗ ( ˙ y ( t 0 )) 〉 = 〈 V ( p ) , G ∗ ( V ( p )) 〉 = κv 2 . Here G ∗ ( ˙ y ) is the differential of the Gauss map, i.e. the curvature tensor of S t 0 at point x . Using this in Eq. 20 and the fact that ˙ y ( t 0 ) = ˙ σ ( p ), ¨ y ( t 0 ) = ¨ σ ( p ) we get

<!-- formula-not-decoded -->

From Eq. 19 and Eq. 21 we conclude that if ¨ λ ( t 0 ) &lt; 0 the point x = ¯ y ( t 0 ) = σ ( u 0 , v 0 , t 0 ) is a local maxima of the function λ and the inverse trajectory of x intersects with interior of the solid M t 0 causing type-2 L.S.I. Similarly, if ¨ λ ( t 0 ) &gt; 0 we conclude that x is a local minima of λ and the inverse trajectory of x does not intersect with the interior of M t 0 and there is no L.S.I. occurring at x .

However, if ¨ λ ( t 0 ) is zero, one needs to inspect a small neighbourhood of t 0 to see if ∃ t such that λ ( t ) &lt; 0 in order to check for type-2 L.S.I. □

If ¨ λ = 0 at a point, the structure of the contact-set C is unknown at that point. We will see in the next subsection that at such a point, C has singularity.

## 5.3. Relation between type-1 L.S.I. and type-2 L.S.I.

In this subsection we will see that type-2 L.S.I. implies type-1 L.S.I. at any point p = ( u 0 , v 0 , t 0 ) ∈ F Lemma 5.2 θ ( p ) = ¨ λ ( t 0 ) .

Proof. Recalling definition of θ ( p ) from Eq. 5

<!-- formula-not-decoded -->

Here ˆ N u = G ∗ ( σ u ) and ˆ N v = G ∗ ( σ v ) where G ∗ is the shape operator (differential of the Gauss map) of S t 0 at ( u 0 , v 0 ). Also, V u = A t S u and V v = A t S v . Assume without loss of generality that A ( t 0 ) = I and b ( t 0 ) = 0. Using Eq.11 and the fact that σ t = lσ u + mσ v we get

<!-- formula-not-decoded -->

From Eqs. 21 and 22 and the fact that ∂σ ∂t 2 = V t we get θ ( p ) = lf u + mf v -f t = ¨ λ □

Let p be such that θ ( p ) &lt; 0. By lemma 5.2, ¨ λ ( t 0 ) &lt; 0. Further, by lemma 5.1 a type-2 L.S.I. occurs at p . This proves parts (i) and (iii), and hence completes the proof of theorem 4.1.

Fig. 8 schematically illustrates the region on the funnel where local self-intersection occurs and the corresponding region on the contact-set. A curve-of-contact and the corresponding pcurve-of-contact is shown in red colour at a time instant t 0 when a local self-intersection occurs. The shaded region there corresponds to θ &lt; 0. Thus sign ( θ ) changes(from -ve to +ve) as one moves from the interior to the exterior of the shaded region. Of course θ = ¨ λ = 0 on the boundary of the shaded region where C has a singularity.

-⌊-≀-⊓-\-⌈-⊣-∇-†-←-≀-{-←-L-S-I-←-∇-⌉-}-〉-≀-\

## 6. Mathematical structure of the smooth case

-〉-\-⊔-⌉-∇-〉-≀-∇-←-≀-{-←-L-S-I-←-∇-⌉-}-〉-≀-\

-‡ -§ -C-≀-\-⊔-⊣-⌋-⊔-↖-∫-⌉-⊔ In this section, we consider the smooth case where the solid M is composed of faces meeting smoothly. As usual, each face (or the associated surface patch) is smooth (of class C k for k ≥ 2). Further, adjacent faces meet smoothly at the common edge. This is referred to as G 1 continuity [13] which formally means that the unit outward normals to the adjacent faces match on the common edge. Similarly, at a vertex, all the unit outward normals to faces incident on this vertex are identical. The solid shown in figure 1 is such a solid.

Fig. 8. Region of local self-intersection

<!-- image -->

Fig. 4 shows an example of sweeping which clearly demonstrates the subtle difference between type-1 L.S.I. and type-2 L.S.I. In Fig. 4 a sphere is rotated about an axis which is tangent to itself. The contact-set is shown in green and the curve-of-contacts on the contact-set at initial position ( t = 0) and final position ( t = 1) are also shown in blue. In Fig. 4 there is type-1 L.S.I. but no type-2 L.S.I. since no point of the contact-set intersects with the interior of solid at any time.

Wenowpresent two examples of sweeping which result in local self-intersections. We apply the test for type-1 L.S.I. on these examples to demonstrate its effectiveness.

Example 5.1 Consider a solid cylinder along the y -axis with height 2 . 5 and radius 2 , parameterized as S ( u, v ) = (2 cos v, u, -2 sin v ) , u ∈ [ -1 . 25 , 1 . 25] , v ∈ [ -π, π ] being swept along the trajectory given by h ( t ) = ( A ( t ) , b ( t ))

<!-- formula-not-decoded -->

cal self-intersections as illustrated in Fig. 5(a) which shows the cylinder with blended edges and curves-of-contact at few time instances. Type-1 L.S.I is detected at time t = 0 . 1 by the test given in lemma 5.1 at point p = ( u = 0 . 18 , v = 1 . 53 , t = 0 . 1) . θ ( p ) = -2 . 378 .

Example 5.2 Consider an ellipsoid with axes lengths 3 , 1 , and 1 parameterized as S ( u, v ) = ( -3 cos( u ) cos( v ) , sin( u ) , cos( u ) sin( v )) , u ∈ [ -π 2 , π 2 ] , v ∈ [ -π, π ] being swept along the curvilinear trajectory given by h ( t ) = ( A ( t ) , b ( t ))

<!-- formula-not-decoded -->

The curves-of-contact are shown in Fig. 5(b). Type-1 L.S.I. is detected at time t = 0 . 8 . θ ( p ) = -11 . 864 at ( -0 . 791 , -0 . 157 , 0 . 8) .

Consider a sweep of M along a trajectory h which causes no self-intersections/anomalies on the contact-set C . As described in Section 3, every point p on C comes from a curve of contact on M and therefore is associated to a point q of M . Let π : C → M ( p ↦→ π ( p ) = q ) denote this natural map. For every p ∈ C , π ( p ) belongs to some geometric entity of M , i.e., a vertex, edge or face. This sets up the natural correspondence between geometric entities of C and that of M . For a face F of M , let C F denote the part of C which corresponds to the face F under this correspondence. For example, in figure 2, the green face on the solid M corresponds to multiple green faces of C . Clearly, the map π restricts naturally from C F → F .

There are situations in which, for example, an edge (or a part of it) on M remains on the boundary for a while and thus, 'sweeps' a face on C . For simplicity, we assume that such situations are ruled out. In other words, no lower dimensional geometric entity of M gives rise to a higher dimensional geometric entity on C . This is the case for the sweep operation illustrated in figure 2.

## 6.1. Local similarity within a face

Firstly, recall that each face F of M is derived from an underlying surface S F by restricting the parameters of S F to a suitable domain D F . Now, by applying the 'local' analysis of Section 4 to the surface S F , we have

Lemma 6.1 The set C F has no self-intersections and is a smooth manifold. Further, let p ∈ C F correspond to q ∈ F at time t . Then, the unit normal N ( p ) to C F at p is simply A ( t ) N ( q ) where N ( q ) is the unit normal to F at q .

Further, we would like to show that 'locally', C F has the same topology as that of F . More precisely, the natural map π : C F → F is a local homeomorphism onto its image. Thanks to the 'local' nature, we may analyse this via the underlying surface S F . For ease of notation, we sometimes omit the reference to F and freely use notations from Section 4. As the sweep σ is free of self-intersections, the key map σ : F → C F is a bijective immersion (recall that, in Section 4, F is the funnel defined via the function f = 0). Thus, by the inverse function theorem, it is invertible via a continuous inverse. Therefore, in order to show that π : C F → F is a local homeomorphism onto its image it suffices to prove the following lemma.

Lemma 6.2 The natural map π ′ : F → F defined as: for p = ( u, v, t ) ∈ F , π ′ ( p ) = S ( u, v ) , is a local homeomorphism.

̸

Remark The map π ′ is simply the composition of σ and π . Proof. Let p = ( u 0 , v 0 , t 0 ) ∈ F . Here, the assumption that f t = 0 at p is very crucial. Firstly, by the implicit function theorem, there exists a neighbourhood O of ( u 0 , v 0 ) and a continuous function t = g ( u, v ) defined on O such that ∀ ( u, v ) ∈ O , f ( u, v, g ( u, v )) = 0. Further, the set O ( p ) = { ( u, v, g ( u, v )) | ( u, v ) ∈ O } is a neighbourhood of p in F . On this neighbourhood of p , the function π ′ is a bijection and invertible. This easily follows from the fact that, for every ( u, v ) ∈ O , there is a unique time t , namely, t = g ( u, v ), such that ( u, v, g ( u, v )) ∈ O ( p ). □

## 6.2. Local similarity across faces

We begin by studying the variation of the unit normal across faces of C . Let C t denote the curve of contact of C at time t . Let p ∈ C t be such that p is common to (only) C F 1 and C F 2 where F 1 and F 2 are two distinct faces of M .

Lemma 6.3 The faces F 1 and F 2 are adjacent in M . Further, the normal to C F 1 at p is identical to the normal to C F 2 at p .

Proof. Suppose p corresponds to q . Clearly, q is common to (only) F 1 and F 2 . Thus F 1 and F 2 are adjacent in M . Further, by G 1 continuity, the normal to F 1 at q is identical to the normal to F 2 at q . By lemma 6.1, it is clear that, the normals to C F 1 , and to C F 2 , at p are identical. □

Thus, the adjacencies on C are the 'same' as the adjacencies on M . See figure 2, which effectively illustrates this through colours. Further, the adjacent entities of C meet smoothly across common lower-dimensional entities. In other words, C is also of class G 1 . Recall that the overall envelope may be obtained from the contact-set C by simply capping the appropriate parts of the solid M at the initial and final position. Therefore, the topological structure of the contact-set and hence, that of the envelope, mimics that of the solid.

The following theorem summarizes the analysis so far. Theorem 6.1 The map π from the contact-set/envelope to the solid is an adjacency-respecting local homeomorphism onto its image.

## 6.3. Curvature of C

̸

̸

In the special case when the trajectory h consists only of translations, i.e. A ( t ) = I ∀ t , the Gaussian curvature of C can be expressed in terms of the Gaussian curvature of S and the curvature of h . Since A ( t ) = I ∀ t , σ u = S u and σ v = S v . Also, V u = V v = 0. Consider a point p = ( u 0 , v 0 , t 0 ) ∈ F . By definition of F , we have that f ( u 0 , v 0 , t 0 ) = 0. Given that ∇ f | p = 0 suppose without loss of generality that f t | p = 0. Then by the implicit function theorem there exists a neighbourhood N of q = ( u 0 , v 0 ) such that ∀ ( u, v ) ∈ N , f ( u, v, t ( u, v )) = 0. Hence, t u = -f u f t and t v = -f v f t and we get a local parameterization of C in N by ψ ( u, v ) = σ ( u, v, t ( u, v )). T C ( p ) is spanned by ψ u = σ u + σ t t u and ψ v = σ v + σ t t v . Since p ∈ F by lemma 4.1, σ t is in the space spanned by σ u and σ v . Let σ t = lσ u + mσ v . Hence we express basis for T C ( p ) in terms of basis of T S ( q ) as follows

<!-- formula-not-decoded -->

where M =   1 + lt u lt v mt u 1 + mt v   . The unit normal to C is

given by ˆ N ( u, v ) = A ( t ( u, v )) N ( u, v ) = N ( u, v ) where N is the unit normal to S t . Hence, ˆ N u = N u and ˆ N v = N v . Further,

<!-- formula-not-decoded -->

where W is the Weingarten matrix of S at point q whose determinant gives the Gaussian curvature of S at q (see [1]). From Eq. 23 and Eq. 24 we have

<!-- formula-not-decoded -->

So the Weingarten matrix of C with respect to parameterization ψ is given by M -1 W and the Gaussian curvature is given by det ( M -1 W ). From Eq. 2 and the fact that N t = V u = V v = 0 we note that f = 〈 N,V 〉 , f u = 〈 N u , V 〉 , f v = 〈 N v , V 〉 and f t = 〈 N,V t 〉 . So,

<!-- formula-not-decoded -->

The Gaussian curvature of C is computed as

<!-- formula-not-decoded -->

where, 〈 N,V t 〉 is the curvature of the trajectory scaled by ‖ V t ‖ , 〈 W ( V ) , V 〉 is the normal curvature of S at p in direction V scaled by ‖ V ‖ 2 and det ( W ) is the Gaussian curvature of S at p .

## 7. Envelope computation

In this section we describe the construction of the envelope E assuming that it is free from self-intersections and hence regular. We obtain a procedural parametrization of E . The procedural paradigm is an abstract way of defining curves and surfaces. It relies on the fact that from the user's point of view, a parametric surface(curve) in R 3 is a map from R 2 ( R ) to R 3 and hence is merely a set of programs which allow the user to query the key attributes of the surface(curve), e.g. its domain and to evaluate the surface(curve) and its derivatives at the given parameter value. The procedural approach to defining geometry is especially useful when closed form formulae are not available for the parametrization map and one must resort to iterative numerical methods. We use the Newton-Raphson(NR) method for this purpose. As an example, the parametrization of the intersection curve of two surfaces is computed procedurally in [14]. As we will see, this approach has the advantage of being computationally efficient as well as accurate. For a detailed discussion on the procedural framework, see [11].

The computational framework is as follows. For the input parametric surface S and trajectory h , an approximate envelope is first computed, which we will refer to as the seed surface. Now, when the user wishes to evaluate the actual envelope or its derivative at some parameter value, a NR method will be started with seed obtained from the seed surface. The NR method will converge, upto the required tolerance, to the required point on the envelope, or to its derivative, as required. Here, the precision of the evaluation is only restricted by the finite precision of the computer and hence is accurate. It has the advantage that if a tighter degree of tolerance is required while evaluation of the surface or its derivative, the seed surface does not need to be recomputed. Thus, for the procedural definition of the envelope we need the following:

- (i) a NR formulation for computing points on E and its derivatives, which we describe in subsection 7.1
- (ii) Seed surface for seeding the NR procedure, which we describe in subsection 7.2

Recall that by the non-degeneracy assumption, E is the union of C t , ∀ t . This suggests a natural parametrization of E in which one of the surface parameters is time t . We will call the other parameter p and denote the seed surface by γ which is a map from the parameter space of E to the parameter space of σ , i.e. γ ( p, t ) = (¯ u ( p, t ) , ¯ v ( p, t ) , t ) and while the point σ ( γ ( p, t )) may not belong to E , it is close to E . In other words, γ ( p, t ) is close to F . We call the image of the seed surface through the sweep map σ as the approximate envelope and denote it by ¯ E , i.e. ¯ E ( p, t ) = σ ( γ ( p, t )). We make the following assumption about ¯ E .

Assumption 1 At every point on the iso-t curve of ¯ E , the normal plane to the iso-t curve intersects the iso-t curve of E in exactly one point.

Note that this is not a very strong assumption and holds true in practice even with rather sparse sampling of points for the seed surface. We now describe the Newton-Raphson formulation for evaluating points on E and its derivatives at a given parameter value.

## 7.1. NR formulation for faces of E

Recall that the points on E were characterized by the tangency condition given in Eq. 2. Introducing the parameters ( p, t ) of E , we rewrite Eq. 2 ∀ ( p 0 , t 0 ):

<!-- formula-not-decoded -->

So, given ( p 0 , t 0 ), we have one equation in two unknowns, viz. u ( p 0 , t 0 ) and v ( p 0 , t 0 ). E ( p 0 , t 0 ) is defined as the intersection of the plane normal to the isot (for t = t 0 ) curve of ¯ E at ¯ E ( p 0 , t 0 ) with the isot (for t = t 0 ) curve of E which is nothing but C t 0 . Recall that C t 0 is given by σ ( u ( p, t 0 ) , v ( p, t 0 ) , t 0 ) where u, v, t obey Eq. 27. Henceforth, we will suppress the notation that u, v, ¯ u and ¯ v are functions of p and t . Also, all the evaluations will be understood to be done at parameter values ( p 0 , t 0 ). The tangent to isot curve of ¯ E at ( p 0 , t 0 ) is given by

<!-- formula-not-decoded -->

Hence, E ( p 0 , t 0 ) is the solution of simultaneous system of equations 27 and 29

<!-- formula-not-decoded -->

Eq. 27 and Eq. 29 give us a system of two equations in two unknowns, u and v and hence can be put into NR framework by computing their first order derivatives w.r.t u and v . For any given parameter value ( p 0 , t 0 ), we seed the NR method with the point (¯ u ( p 0 , t 0 ) , ¯ v ( p 0 , t 0 )) and solve Eq. 27 and Eq. 29 for ( u ( p 0 , t 0 ) , v ( p 0 , t 0 )) and compute E ( p 0 , t 0 ).

Having computed E ( p, t ) we now compute first order derivatives of E assuming that they exist. In order to compute ∂ E ∂p , we differentiate Eq. 27 and Eq. 29 w.r.t. p to obtain

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Eq. 30 and Eq. 31 give a system of two equations in two unknowns, viz., ∂u ∂p and ∂v ∂p and can be put into NR framework by computing first order derivatives w.r.t. ∂u ∂p and ∂v ∂p . Note that Eq. 30 and Eq. 31 also involve u and v whose computation we have already described. After computing ∂u ∂p and ∂v ∂p , ∂ E ∂p can be computed as ∂σ ∂u ∂u ∂p + ∂σ ∂v ∂v ∂p . ∂ E ∂t can similarly be computed by differentiating Eq. 27 and Eq. 29 w.r.t. t .

Higher order derivatives can be computed in a similar manner.

## 7.2. Computation of seed surface

The seed surface is constructed by sampling a few points on the envelope and fitting a tensor product B-spline surface through these points. For this, we first sample a few time instants, say, T = { t 1 , t 2 , . . . , t n } from the time interval of the sweep. For each t i ∈ T , we sample a few points on the curve-of-contact C t i . For this, we begin with one point p on C t i and compute the tangent to C t i at p , call it T C i ( p ).

p + T C i ( p ) is used as a seed in Newton-Raphson method to obtain the next point on C t i and this process is repeated.

While we do not know of any structured way of choosing the number of sampled points, in practice even a small number of points suffice to ensure that the Assumption 1 is valid.

## 7.3. NR formulation for edges and vertices of E

An NR formulation for edges and vertices of E can be obtained in a manner similar to that for faces of E which we described in subsection 7.1. In order to obtain a procedural parametrization for edges of E , again seed curves need to be computed.

## 8. Discussion

In this work, we have proposed a novel computationally efficient test for detecting anomalies on the envelope. This has been achieved through a delicate mathematical analysis of an 'invariant'. We have provided a rich procedural framework for computing the Brep of the envelope along with its accurate parametrization. Another contribution is a natural correspondence between the geometric/topological entities of the Brep envelope and that of the Brep solid. This framework has been implemented using the ACIS kernel [12] and has been used to produce the running examples of this paper.

Ongoing work includes, for example, extending the proposed procedural framework to handle (i) deeper topological information of the Brep envelope, (ii) swept edges (an edge on the solid sweeping a face on the envelope) and so on (iii) faces meeting with G 0 continuity (i.e. sharp edges). We also to plan to extend the detection of anomalies to above settings and further, trim the appropriate part to obtain the final envelope.

Another exciting future direction would be to analyse sweeps in which some numerical invariant associated with a a curve of contact, varies over time. For example, one may imagine sweeping a torus along a trajectory where the number of components of the curve of contact changes over time. In such a case, one would like to efficiently compute deeper topological invariants, say genus, of the envelope. Our mathematical analysis coupled with a Morse-theoretic analysis appears promising.

## References

- [1] Pressley A. N. Elementary Differential Geometry. Springer, 2000
- [2] Blackmore D, Samulyak R, Leu MC. Trimming swept volumes. Computer-Aided Design 1999;31(3):215-223.
- [3] Blackmore D, Leu MC, Wang L. Sweep-envelope differential equation algorithm and its application to NC machining verification. Computer-Aided Design 1997;29(9):629-637.
- [4] Blackmore D, Leu MC. A differential equation approach to swept volumes. Proc. of rensselaers 2nd international conference on computer integrated manufacturing 1990:143-149
- [5] Wang L, Leu MC, Blackmore D. Generating swept solids for NC verification using the SEDE method. ACM symposium on solid modeling and applications, proceedings of the fourth ACM symposium on solid modeling and applications 1997;364-375
- [6] Abdel-Malek K, Yeh HJ. Geometric representation of the swept volume using Jacobian rank-deficiency conditions. ComputerAided Design 1997;29(6):457-468.
- [7] Xu Z-Q, Ye X-Z, Chen Z-Y, Zhang Y, Zhang S-Y. Trimming self-intersections in swept volume solid modelling. Journal of Zhejiang University Science A 2008;9(4):470-480.
- [8] Guillemin V, Pollack A. Differential Topology. Prentice-Hall, 1974.
- [9] Elber G. Global error bounds and amelioration of sweep surfaces. Computer-Aided Design 1997;29(6):441-447.
- [10] Martin R, Stephenson P. Sweeping of three-dimensional objects. Computer-Aided Design 1990;22(4):223-234.
- [11] Milind Sohoni. Computer aided geometric design course notes. www.cse.iitb.ac.in/~sohoni/336/main.ps
- [12] Spatial Corp. ACIS 3D modeling kernel www.spatial.com/products/3d-acis-modeling
- [13] Gerald Farin. Curves and surfaces for CAGD: A practical guide. Morgan Kaufmann Publishers. 2002
- [14] Markot R, Magedson R. Procedural method for evaluating the intersection curves of two parametric surfaces. Computer-Aided Design 1990;23(6);395-404
- [15] Abdel-Malek K, Harn-Jou Yeh, Saeb Othman. Swept volumes: void and boundary identification. Computer-Aided Design 1998;30(13);1009-1018
- [16] Huseyin Erdim, Horea T. Ilies. Detecting and quantifying envelope singularities in the plane. Computer-Aided Design 2007;39(10);829-840
- [17] Seok Won Lee, Andreas Nestler. Complete swept volume generation, Part I: Swept volume of a piecewise C1-continuous cutter at five-axis milling via Gauss map. Computer-Aided Design 2011;43(4);427-441
- [18] Seok Won Lee, Andreas Nestler. Complete swept volume generation, Part II: NC simulation of self-penetration via comprehensive analysis of envelope profiles. Computer-Aided Design 2011;43(4);427-441
- [19] Horea T. Ilies, Vadim Shapiro, The dual of sweep. ComputerAided Design 1999;31(3);179-182
- [20] Huseyin Erdim, Horea T. Ilies. Classifying points for sweeping solids. Computer-Aided Design 2008;40(9);987-998
- [21] Wang W.P., Wang K.K. Geometric Modeling for Swept Volume of Moving Solids . Computer Graphics and Applications, IEEE 1986;6(12);8-17
- [22] Rossignac J, Kim J.J, Song S. C, Sub K.C, Joung C.B. Boundary of the volume swept by a free-form solid in screw motion. Computer-Aided Design 2007;39(9);745-755
- [23] Abdel-Malek K, Yeh, H.J. Analytical Boundary of the Workspace for General 3-DOF mechanisms. International Journal of Robotics Research 1997;16(2);1-12
- [24] Abdel-Malek K, Adkins F., Yeh, H.J., Haug E.J. On the Determination of Boundaries to Manipulator Workspaces. Robotics and Computer-Integrated Manufacturing 1997;13(1);115
- [25] Peternell M, Pottmann H, Steiner T, Zhao H. Swept volumes. Computer-Aided Design and Applications 2005;2;599-608