## Energy Based Multiple Refitting for Skinning

Kailash Jha*

Dept, of  Mechanical Engineering and Mining Machinery Engineering Indian School of  Mines, Dhanbad-826004, Jharkhand, India

Abstract 一 The traditional method of manipulation of knots and degrees gives poor quality of surface, if compatibility of input curves is not good enough. In this work, a new algorithm of multiple refitting of curves has been developed using minimum energy based formulation to get compatible curves for skinning. The present technique first reduces the number of control points and gives smoother surface for given accuracy and the surface obtained is then skinned by compatible curves. This technique is very useful to reduce data size when a large number of data have to be handled. Energy based technique is suita 비 e for approximating the missing data. The volumetric information can also be obtained from the surface data for analysis.

Keywords: Surface approximation, Surface skinning, Curve compatibility, Energy minimization, B-Spline curve fitting

## 1.  Introduction

Surface skinning is the process of passing a smooth surface through a set of curves called sectional curves, which, in general, may not be compatible. Tims the first task for skinning is to get compatible curves. There are two approaches to get the compatible curves, namely, (i) exact method and (ii) approximate method.

In exact method, compatible curves are obtained by manipulation of degrees and knot vectors. This technique is straightforward and efficient in computation. But, this approach results in astonishing number of control points when compatibility of curves is not good enough. While manipulating a lar 응 e number of sectional curves of different degrees and defined in different knot intervals, the exact method results in a huge number of control points and the quality of skinned surface becomes poor. This problem has been overcome in the present formulation, which is based on approximation technique.

On the other hand, in approximation technique, compatibility of curves is obtained by approximation instead of interp 이 ation. In this process tremendous saving can be achieved at the cost of q 니 ality of precision. Piegl and Tiller [1] have pointed out the ^sefhhiess of approximation technique. In the first step they use exact method, which gives a large number of control points, which are reduced in the second step of algorithm by adopting approximation technique. But, this technique gives poor qualities of s 니 rf&amp;ce when curves are irregular in parameterization. Refitting

^Corresponding author:

technique has been 니 sed to get compatible curves by employing energy coefficient 너 p to second derivative to get smoother and stable elution [2]. Hofler &amp; Pottmann [3] has developed energy-based algorithm by employing energy contribution up to second derivative to get Spline on manifold. It allows the treatment of obstacles via barrier surfaces. Abbas and Nasri [4] has adopted a technique to interpolate intersecting curves by Cutmull-Clark s 니 bdivision surfeces. Planarity and specific symmetry at the j 니 nction points have been taken to ensure interp 이 ation limit of the subdivision process by a surface with an adequate degree of smoothness. Nira et al. [5] have given a butterfly subdivision scheme for surface interpolation with tension control. It generates triang 니 lation of control points and has a tension parameter that provides design flexibility.

Present technique is similar to that of Park et al. [2] but 니 ses B-Spline interpolating function and energy coefficients up to second derivatives. The effects of energy co-efficient on smoothness of curve have been described in the previous work of the present author [7], while the effect the same has been explained in Fang and Gossard [6] by using Hermit interpolating function.

The present formulation is an approximation technique, which uses refitting algorithm based on energy technique to give smoother and stable solution. Numbers of control points are reduced and the optimal member of control points for a given accuracy is obtained based on binary search. This data reduction is very helpful when a huge number of data have to be handled and the curves obtained are very smooth due to energy­ based approximation. Manipulating energy coefficient can enhance capability of data reduction. Fitting coefficient directly reduces the error and hence improves data reduction. But, at the higher accuracy level the proposed method may be inefficient in data reduction.

<!-- image -->

Another approximation technique, which approximates the cross sections using distance map, converts the multi branching problem into sin 이 e branching and final B-Spline surface is obtained by skinning [8]. Present technique energy based approximation has been used to get cross sectional curve. Points in 3D spaces are the prime inputs for present fbmmlation.

After having compatible curves, they can be skinned in longitudinal direction by two methods, either by exact interpolation or by approximation technique. In exact interpolation, compatible curves are interpolated with the same number of control points as the control points of compatible curves in v-direction (skinning direction). While in approximation technique, binary search is used to get optimal number of control points for given tolerance. This optimal number of control points should satisfy the tolerance limit for all the control points of compatible curves [9, 10, 11].

Rest of the paper has been organized as follows:

In section 2, B-Spline curve and surface 니 lation have been described. Section 3 and section 4 of the paper deals with knot selection and algorithm of multiple refitting respectively. Energy based approximation has been described in section 5. Section 6 deals with basic formulation for B-Spline skinning. Results and Discussions have been given in section 7. Section 8 con 이니 des the current work.

## 2.  B-Spline Curves and Surfaces

The parametric representation of a p-th degree BSpline curve is given by

<!-- formula-not-decoded -->

where V, fbr i = 0, 1, 2,… ， n, are the control points, u is the parametric vahae of the curve, and are the j9-th degree B-Spline basis functions defined on the knot vector as given by

<!-- formula-not-decoded -->

with m = n+p + 2 being the total number of knot values. For an open knot vector, p + 1 numbers of knot values are repeated at the start and the end of the knot vector. Basis functions are recursively defined as given below

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

for^&gt;&gt; 1. When evaluating the basis functions, 0/0 = 0

is ass 니 med.

Extension of the above curve fbrm 니 lation in two parametric directions results in a B-Spline surface. A bi-parametric B-Spline surface can be defined as tensor product ofB-Spline curves. A B-Spline surface ofp^q degrees is defined as follows

<!-- formula-not-decoded -->

where Vy, for Z = 0, 1, 2, m and j = 0, 1, 2, n, form the control net in 3D, 丿 V 罹(히) is the z-th basis function in w-direction and is defined on knot vector ("o, 肉，...， wm+p+i), and is the 丿 -th basis function in v-direction and is defined on knot vector (v0, 5,… ， vn+q+{) with p and q being the degrees of the surface in u and v directions, respectively. Basis functions are recursively defined similar to equation (2) in both parametric directions. In equation (3), m + 1 and 〃 + 1 are numbers of control points in u and v directions, respectively. Details of B-Spline formulation can be found in references [9, 10].

## 3.  Knot Selection

The prime criteria of selection of knot vector are such that every knot segment must contain at least a point. Selection of knot is based on the parametric value of points of sectional curves. There are three basic techniq 니 es of parameterization (a) 니 nifbrm method, (b) chord length parameterization, and (c) centripetal method. Please see the references [2, 9, 10] fbr parameterization. In present work, chord length parameterization has been used. There are (n + 1) and (m + 1) number of control points and total number of points in curve respectively. Common degree of the given sectional curves is p. Details of this formulation are given in references [2, 12]. There are three possible conditions for selecting knots, which are given below.

## (1)  Number of input points are same as the number of control points assumed

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where u ； is the parametric value of input points and 7] is the knot values.

## (2)  Number of points are greater than number of control points assumed: Insert formula

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where

```
int(zW) d=(m+ 1)/(0 -卩 + 1) tt =( 产 dT) Ui = parametric value of input point
```

With this formula every knot span must have at least one Ui

## (3)  Number of points are less than number of control points

In this case the parametric values are sampled at different points in terms of the given parametric values of input points. The first condition is used to get the required knot vectors.

```
既 =#林 @ = 0, 1,.......... n) Ao = % 卩 i = 山-\ + (( ,■_, ) /' ) /«(/'= 1,.......... m*n) Vu = uk+} -uk .......... m-1) (6)
```

Final knot vectors are obtained by 니 sing equation (4) in newly sample points.

## 4.  Algorithm of Multiple Refits

In this techniq 니 e, a compromise between acc 니 racy and the costs has been done. Cost deals with the storage cost as well as the computational cost. The present algorithm starts with input points, which are given fbr each sectional curve. Maximum number of input points (m+1) among the sectional curves is determined. We assume that maximum number of common control points will not be more than this number (m+l). Common degree (p) and accuracy are also inputs fbr this algorithm. Since input points are the prime inputs so it is upon the 니 ser to decide the overall degree of the curves. If curves are given then its points are derived from the different curves. Input curves may be of different degree. Current algorithm is based on binary search, which is given below

```
Process:: MultipleRefitQ { NCMax = Maximum number of control points NCNow = C 니 rrent number of control points m = maximum number of points for input sectional curves NCMin = p NCMax = m+1 Start: NCNow = (NCMax + NCMin)/2
```

Calc 니 late the knot vector for all section curves based on knot s 이 ection criteria see section 3

Take the average of all these knot values as common knot vector For all the section curves do the energy based approximation

```
{ If (error calculated violates the tolerance limit) { if((NCMax - NCNow) < 1) { . . . if (previous value of control points exist) return with the previous control points 이 se go fbr exact interpolation } else ( NCMin = NCNow go to Start } } } if(Energy based approximation for all section curves is true in w-direction) { copy current control points in previous control points if( (NCNow - NCMin) < 1 ) { return with c 니 rrent value of control points ) else { NCMax = NCNow go to Start } 1 1
```

## 5. Energy Based Approximation

In this section, an energy based interpolation technique has been described. Control points of the input sectional curves are obtained for a given number of control points and knot vectors. Common knot vectors are obtained from the knot selection technique given in section 3 with help of number of control points and common degree of freedom of curve. This approximation technique is used multiple times till an optimal number of control points is achieved. Optimal number of points depends on the accuracy needed. For higher accuracy number of control points increase. This technique help in approximating the missing data due to energy based formulation. The sectional curve obtained by this approximation is smoother [7].

Total energy for a sectional curve considered in this formulation is given by the following equation

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

where a. p and 月 are non-negative values called stretching, bending and fitting co-efficient respectively.

A new non-negative coeflficient of energy responsible for having smooth curvature is y

In this technique eq 니 ation 7 is solved.

Approximated c 니 bic B-Spline curve is C(t)

Inp 니 t points are denoted by P,

First, second and third derivations of B-Spline curve C(t) are Clt and Cm respectively.

Parametric variable is t. These energy terms have been explained in [1, 6, 12, 13]. The squared first and second terms in equation (7) give strain energy contribution due to stretching and bending of the curve. Squared third derivative part of the equation does not have any physical significance. It has some geometrical meaning. The magnitude of the third derivative is a rough estimate of the rate of change of curvature with respect to the parametric value. The fourth term of energy equation indicates the energy stored in spring due to error (see elastic beam analogy in [6]).

Effects of these coefficients have been studied in the previous work [7]. Quadratic functional minimization technique has been used in this work which results in a curve by solving the linear set of equations [2]. Equation (7) has been solved by assuming a B-Spline curve satisfying it. Set of linear equations has been obtained finally. Control points of B-Spline curve are obtained by solving linear eq 나 ations. If equation (8) is the B-Spline solution for equation (7)

<!-- formula-not-decoded -->

where N represents B-Spline basis functions and X represents Control points [9, 10]. Equation (7) can be rewritten as

<!-- formula-not-decoded -->

where

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

Nh Nlh Nm represents first, second and third derivatives of Basis Function [10]. K, K2 and K3 are 4x4 matrices [6, 13]. A and P represent co-efficient matrix and input points respectively.

<!-- formula-not-decoded -->

If the basis function assumed is uniform or in Bezier form Kl can be calculated analytically [14]. In case of non-uniform basis function numerical technique (Gaussian quadrature) can be used to get KL matrix. Constraints are given by -

<!-- formula-not-decoded -->

where k, varies from 0 to (number of constraints-1).

This eq 니 ation may contain the positions, tangents and higher derivative data.

The final matrix equations can be expressed as reference [2].

<!-- formula-not-decoded -->

where / is a vector storing Langrange Multiplier and X are the unknown control points.

K is the assembled stiffiiess matrix with size ( 〃 + 1) x («+1)-

C is zi-constraints x (/? + 1) matrix.

n-constraints indicates number of constraints given.

Above matrix equations have been solved by Gauss­ elimination to get control points.

Details of this formulation are available in [2, 6, 7].

## 6.  B-Spline Skinning

Skinning is a process of blending the sectional curves together to form a surface (see references [9, 10], Blending direction is in v-direction, which is also known as longitudinal direction. Sectional curves may be of different degree and defined over different knot vectors. If sectional curves have same degree and defined over same knot vector then the input sectional curves are compatible; otherwise multiple refitting algorithm is applied to get the compatible section curves. Compatible sectional curves can be skinned by two methods, see section 1. Exact skinning uses interpolation with the same number of control points as the number of control points of the compatible curves in skinning direction. This results in a perfect interpolation. It is suitable for skinning less number of curves. If a large number of sectional curves are given then the approximate technique saves a large n 니 mber of control points at the cost of accuracy. Approximation technique is based on binary search to get optimal number of control points. Sectional curves assumed in this work are the set of points.

B-Spline surface interpolates 〃 +l compatible curves and can be written as equation (12), which has been rewritten as follow

<!-- formula-not-decoded -->

Control points (J 勺) fbr i = 0, 1, 2,....... , m and j= 0, 1, 2, ...... , n are obtained by energy based refitting technique in //-direction. Final control nets are obtained by interpolating or approximating (compatible control points) in v-direction. All j column of Vy are taken fbr interpolation in v-direction. Algorithm of approximate skinning is given below.

## Algorithm of approximate skinning

```
Process:: AproximateSkinning() ( NCVMax = number of sectional curves in skinning direction NCVmin = degree in skinning direction Start: NCVNmv = (NCVMax + NCVMin)/2 For each set of control points in skinning direction { calculate the knot vectors by formula given in section 3 do global interpolation with degree in skinning direction direction, NCVNow and knot vector if(error calculated violates the tolerance limit) ( if (NCVNow-NCVMin)< 1) { if (previous value of control points exists) return with previo 나 s values of control points else go for exact interpolation } else { NCVMin = NCVNow go to Start }. } if(approximation is true for all set of control points in v-direction) { . copy present control points in previous control points 谊 (NCVMax-NCVNow) < 1) { return with current values of final control points for all the control points of Compatible curves } else ( NCVMax = NCVNow; go to Start } } }
```

## 7. Results and Discussions

Several data have been tested. It has been found that the current algorithm has achieved a large percentage of data saving at the cost of accuracy. Results have been displayed by using OpenGL on a PC using windows operating system. Each result has been drawn in the frame of reference, which contains a cuboid frame. Fig. 1 shows input points and control polygon of compatible curves obtained after the present energy based approximation technique. Lines and dots in the Fig. 1 show the input points and control polygons fbr the compatible curves respectively.

<!-- image -->

Fig. 1. Input points control points of sectional curves.

Fig. 2. Control net of surface skinning.

<!-- image -->

Fig. 3. Skinned surface fbr control net show in Fig 2.

<!-- image -->

Fig. 2 shows the control net of skinned surface of the result shown in Fig. 1. Skinned surface has been shown in Fig. 3 for the input points shown in Fig. 1. In this case 10 sectional curves have been given, each curve having ten inp 니 t points. Eight control points are sufficient to get the compatible curves with accuracy 1.0. The Stretch, bending, and fitting coefficients fbr this case are 1.0, 1.0 and l.Oe9 respectively. Coefficient for third derivative energy parameter has been taken as 0.004. Exact interpolation has been done in skinning direction. In this example, even skinning with nine number of control points results in an unacceptable error of 1.5. In such cases exact approximation gives better solution. This is true once less number of curves is to be skinned. Data reduction in skinning is not very effective in this case because data given is just sufficient to approximate the points. Complex geometry needs more data hence the data savin 흠 is not affective. Ranges of input points are 476 x 448 x 300 for this example. Range of data is another parameter, which affect the data reduction. If the range of data is wider, accuracy limit should be higher for same data red 니 ction. Data reduction hence depends on the complexities of geometry of input curves and their ranges in 3-D space. Another example has been shown in Fig. 4, which is an example of the close curve. In this case ten points have been given for each curve. Present formulation gives accurate and smooth sectional curve although there is no data reduction. Ten control points are needed to approximate curves in both the directions. Fig. 4 shows the input points and control polygon for sectional curves obtained by present formulation. Fig. 5 shows the control net fbr skinned surface fbr the input points shown in Fig. 4 and Fig. 6 shows the skinned surface for the same.

<!-- image -->

Fig. 4. Input points and the control points of compatible curves.

Fig. 5. Control net for example shown in fig 4 .

<!-- image -->

Fig. 6. Skinned surface for control net shown in Fig 5.

<!-- image -->

Present work gives smoother results than Park et al. [2] because energy parameters up to third derivative have been taken for curve refitting. This work is helpful in data reduction of large data. Processing of data becomes easier because of less number of control points. As the value of tolerance decreases more n 니 mber of control points are needed to get compatible curves. The data reduction is related to accuracy needed. Wlue of fitting coefficient decreases the error and hence results more affective data reduction. Present technique gives smoother and stable results due to energy coefficients considered up to third derivative.

Current work takes data file from I-DEAS Surface Modeler (a product of Structural Dynamic Research Corporation). Effect of third energy term results smoother curves because total energy calculated is more acc 니 rate. Res 니 Its have been shown in ref [7]. Similar effects have been obtained in reference [6]. Fig. 3 on page number 54 of reference [6] shows the effect of third derivative. C 냐 rvat 니 re plot in [6] for a semi-circled curve become a straight line after addition of third derivative term only in the energy equation.

An approximated quarter of circle and its curvature plots at very high resolution have been shown in Fig. 7. Black and gray circular arcs in Fig. 7 show the curves obtained by considering coefficient of the third derivative (black curve) and without considering the third derivative (gray curve). It is very hard to predict the effect of the third derivative by seeing the curve plot. Curvature plots have been shown in X-Y coordinate (Fig. 7) at very high resolution. Black and gray curves in X-Y plane show the curvature plots by considering the third derivative and without considering the third derivative. It indicates clearly that by adding third derivative the curvature plot is smoother.

Fig. 8 shown a human face having 100 curves each curve having 100 points. If hundred control points will be taken for each curve then this results an exact fit. If the tolerance set to 1,0 then 56 control points per curve are sufficient to approximate the given sectional curves. If tolerance is set to 1.5 then number of control points of the compatible curves are 31 per curve. Approximation in skinning direction is also effective. For accuracy 1.5 and 2.0 number of control points in skinning direction are 94 and 81 respectively. Range of the input points is 500 '又 500 x 500. Energy coefficients taken are same as in Fig. 1. Effects of energy coefficient on the error of the curves have been shown in my previous work [7]. Data reduction means minimizing the control points for given accuracy. Number of control points with degree of curves gives the complete information about the curve. Branching problem have not been addressed in the present formulation.

Fig .7. Effect of coefficient of third derivative.

<!-- image -->

Fig. S. Skinned surface fbr. on smoothness of the curve 100 x 100 inp 나 t points.

<!-- image -->

Curves with and without the third derivative

## 8.  Conclusions

In this work, an algorithm of multiple refitting of curves has been developed to get compatible curves for skinning. Exact method based on manipulation of degree and knot vectors results in astonishing number of control points when compatibility of sectional curves is not good enough. Energy parameters up to third derivative have been used to give smoother curves. Missing data can be accommodated well in present formulation. Present work save significant number of control points to get skinned surface. It results in a stable s 이 ution due to energy based formulation. Data reduction is not efifective if less number of sectional curves is used. Data reduction and tolerance are inversely related. Exact interpolation as well as approximation has been done for skinning. Saving in skinning direction is not effective because ranges of the control points of section curves increases, so the accuracy range should be higher which may not be desired. Data reduction is possible only when the points given are more than the minimum number fbr given accuracy.

## References

- [1]  Piegl, L. and Tiller, W. (1996), ''Algorithm for approximate NURBS Skinning", Computer Aided Design, 28(9), 699706.
- [2]  Park, H., Kim, K.and Lee, Sang-Chan (2000), 'A method for approximate NURBS curve compatibility based on multiple curve refuting", Computer Aided Design, 32(20), 237-252.
- [3]  Hofler M. and Pottmann H. (2004), "Energy-Minimization Splines in Manifblds,\ Proceeding of ACM Siggraph, 23(3), pp-284-293.
- [4]  Abba A. and Nasri A. (2004), 'Interpolating Multiple Intersecting Curves using Catmull-Clark Subdivision Surfaces^, The Journal of CAD and Application^ 1, 1-4.
- [5]  Nira D, Levin, D. and John, A. G (1989), 'A Butterefly Subdivision Scheme fbr Surface Interpolation with Tension Control", ACM Transaction on Graphics, 9(2), 160-169.
- [6]  Fang, L., Gossard, D. C. (1995), ^Multidimensional Curve Fitting to Unorganized Data Points by Non-linear Minimization", Computer Aided Design, 27(1), 45-58.
- [7]  Jha, Kailash (2004),4'Minimum Energy Based Interpolation for Interactive Incremental Well-Path Design", Engineering Computations, 21(7), 67-69.
- [8]  Jeong, J., Kim, K., Park H.,Cho, H. and Jung, M, (1991), UB-Spline Surface Approximation to Cross-Section Using Distance Maps", 15, 876-885.
- [9]  Farin, Gerald (1993), 'Curves and Surfaces for Computer Aided Geometric Design", Academic Press, USA.
- [10]  Piegl, L. and Tiller, W. (1995), 'The NURBS book. Springer1 \ New York.
- [11]  Jha, Kailash (2003), 'Energy Based Multiple Curve Refitting for Skinning", CADG-2003, October 29-31, Macau (China).
- [12]  Park, H., Kim, K. (1996), "Smooth Surface Approximation to Serial Cross-Section", Computer Aided Design^ 28(12), 995-1005.
- [13]  \^ssilev, T. I. (1996), 'Fair Interp 이 ation and Approximation of. B-Splines by Energy Minimization and Point Insertion", Computer Aided Design^ 28(9), 753-760.
- [14]  Zhaohui, H. (1994), 'Shape Representation by B-Spline Curve Modeling with Application in Invariable Curve Matching Motion Estimation and Object Tracking", Ph D Thesis, The Drexel University.

Kailash Jha is currently Faculty-in-Charge of CAD/CAM Center, Dept, of ME&amp;MME, ISM, Dhanbad, Jharkhand, India. He has done his schooling from B. D. A. H. School, Nan 이」 r. He received his Bachelor and Master degrees in Mechanical Engineering from Regional Engineering College, Durgapur in 1991 and 1993 respectively. He received his Ph D in Mechanical Engineering from Indian Institute of Science, Bangalore in 1999. His main research interrests are surface approximation, feature based design, geometrical reasoning, reverse engineering and concurrent engineering.

Kailash Jha

<!-- image -->