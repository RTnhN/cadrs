## (12) United States Patent Rockwood

## (54) SURFACE-TO-SURFACE INTERSECTIONS IN COMPUTER-AIDED GRAPHICS AND MODELING

- (71)

Applicant: Zoo, Malibu, CA (US)

- (72) Inventor:

:Alyn Rockwood, Boulder, CO (US)

- （*） Notice:

Subject to any disclaimer. the term of this patent is extended or adjusted under 35 U.S.C. 154(b) by 0 days.

- (21)

- Appl. No.: 18/662,056

- (22) Filed:

May 13, 2024

- (51) Int. Cl. 00/21 190D (2006.01)

- (52) U.S. Cl. CPC........ G06T 17/00 (2013.01); G06T 2210/21 (2013.01): G06T 2210/52 (2013.01); G06T

2210/56 (2013.01)

- (58) Field of Classification Search None Scc application filc for complete scarch history.

## (56) References Cited

## U.S. PATENT DOCUMENTS

2004/0001060 A1* 1/2004 Stollnitz .. G06T 17/20 345/423 2005/0231530 A1* 10/2005 Liang ... G06T 19/00 345/619

US012229885B1

<!-- image -->

<!-- image -->

<!-- image -->

- (10) Patent No.:

US 12,229,885 B1

- (45) Date of Patent:

Feb.18, 2025

2014/0185873 A1*

7/2014Li

G06V 20/64

382/103

2017/0177960 A1*

6/2017 Boulanger

G01B 11/24

2021/0316463 A1* 10/2021

Ku

G06T 17/00

## OTHER PUBLICATIONS

Adarsh Krishnamurthy, Parallel GPU Algorithms for Mechanical CAD, UC Berkeley Electronic Theses and Disserlations. 2010, 1-128, Lniversity of California, Berkeley, CA.

Michael Edward Hohmeyer; *Robust and Efficient Surface Intersection for Solid Modeling'. University of California at Berkeley, Feb. 10, 1992. pp. 1-121, University of California at Berkeley, Berkeley. CA.

s Operations on the GPU', SPM "08 Proceedings of the 2008 ACM symposium on Solid and physical modcling. Jun. 4, 2008. pPp. 257-268, Stony Brook, NY.

* cited by examiner

Primary Examiner — Frank S Chen (74) Attorney, Agent, or Firm — MP Patents, LLC

## (57) ABSTRACT

A computer-implemented method for computing an intersection betwcen first and sccond surfaccs of onc or more 3I) models includes receiving a description for each of the first and second surfaces, using the descriptions to compute one or more intersection curves and outputting the one or more intersection curves.

16 Claims, 13 Drawing Sheets

100

FIG. 1

<!-- image -->

FIG. 2

<!-- image -->

FIG.3

<!-- image -->

300

FIG. 4

<!-- image -->

<!-- image -->

<!-- image -->

FIG.7

<!-- image -->

<!-- image -->

FIG.9

<!-- image -->

<!-- image -->

FIG. 11

<!-- image -->

FIG.12

<!-- image -->

FIG.13

<!-- image -->

## SURFACE-TO-SURFACEINTERSECTIONS IN COMPUTER-AIDED GRAPHICS AND MODELING

mentalitics disclosed hercin. Morcover, those of ordinary skill in the art will understand that the drawings are not to scale. Wherever possible, like elements have been indicated by identical numbers.

## TECHNICAL FIELD

FlG. 1 illustrates an overview flow of an example computer-implemented method for locating an intersection.

Embodiments of the disclosure will now be described, by way of example only, with reference to the following diagrams wherein:

FlG. 2 illustrates an example set of intersecting surfaces.

The disclosure relates generally to techniques for threedimensional (3D) modeling. More specifically, it pertains to techniques for determining the intersection of two surfaces using cnhaneed Surfacc-to-Surface Interscction (SSI) com- 10 putations.

## BACKGROUND

3D surface modeling plays a pivotal role in numerous 15 industrics, cncompassing computer-aided design (CAD), computer gaming, and animation. It is often useful to identify the intersection of two surfaces in 3D modeling. Such an intersection may be suitable for use in various sion detection. However, traditional approaches to computing SSI are computalionally demanding and tend 1o be slow, particularly with intricate surfaces or extensive datasets.

## SUMMARY

25

The disclosure describes a   computer-implemented method for computing an intersection between first and second surfaces of one or more 3D models. The method includes receiving a description for each of the first and 30 convolved and mapped to an object space. second surfaces, using the descriptions to compute one or interscction curves.

FlG. 3 illustrates a flow of another example computerimplemented method for locating an intersection.

FlG. 4 illustrates the example set of intersecting surfaces of FIG. 1 with onc surfacc dcpicted as a point cloud.

FIG. 6 illustrates a flow of an example computer-implemented parallel processing method for locating an intersection.

FlG. 5 illustrates a flow of another example computerimplemenled method for locating an intersection.

FlG. 7 illustrates an example set of intersection points which may result from actions of disclosed computer-implemented methods.

FlG. 8 illustrates an example parameterization of the set of intersection points of FIG. 7.

FlG. 10 illustrates the example set of intersecting points of FIG. 7 post processing.

FlG. 9 illustrates a flow of another example computerimplemented method for locating an intersection.

FIG. 11 illustrates an example set of intersection points

FIG. 12 illustrates an example fit of one or more curves to a set of intersection points.

Further, the disclose describes a system for computing an intersection. The system includes a memory, a storage and a 35 mented methods for locating an intersection. parallel computing processor. The parallel computing proDETAILED DESCRIPTION reccivc a description for cach of the first and second surfaces, to use the descriptions to compute one or more intersection curves and to output the one or morc intersec- 40 ments of the disclosure and manners by which they can be tion curves.

FIG. 13 schematically illustrates an cxamplc systcm suitable for use in association with disclosed computer-imple-

The following detailed description illustrates cmbodiimplemented. Although the best mode of carrying out the present disclosure has been set forth herein, those of ordinary skill in the art would recognize that other embodiments for carrying out or practicing the disclosure are also pos-

It should be noted that the terms "first", "second", and the like, herein do not denote any order, quantity, or importance. but rather are used to distinguish one element from another. Further, the terms "a"" and "an" herein do not denote a - 50 limitation of quantity, but rather denote the presence of at least one of the referenced item.

Furthcr still, thc disclosure describes a computer-implcbetween first and second surfaces of one or more 3D models. The method includes receiving a description for each of the first and second surfaces, from the description of the first surfacc, deriving a set of points represcnting the first surface and forming the set of intersection points from points representing the first surface which are distanced from the second surface by less than a threshold distance.

## BRIEF DESCRIPTION OF THE FIGURES

The summary above, as well as the following detailed description of illustrative embodiments, is better understood when read in conjunction with the appended drawings. For the purpose of illustrating the disclosure, example construc- 65 tions of the disclosure are shown in the drawings. However, the disclosure is not limited to specific methods and instru-

Embodiments of the disclosure provide a computerimplemented method for computing SSI by introducing a parallel processor-accelerated algorithm that significantly

The disclosure also describes a non-transitory computer rcadable medium for computing an intcrsection. The nontransitory computer readable medium includes computer rcadable instructions which, whcn cxccuted by a proccssor, 45 sible. cause one or more computers to receive a description for each of the first and second surfaces, use the descriptions to compute one or more intersection curves and output the one or more intersection curves.

Known processes for determining surface intersections, while accurate, can be computationally intensive and slow, especially for complex surfaces or when high precision is intersection.

Consequently, there is a need for a system to calculate SSI that is not only eficient and precise but also faster, in part. 60  by leveraging parallel processing power such as embodied in the contemporary graphics processing units (GPUs). Disclosed methods and associated systems seek to blend the accuracy of traditional SSl techniques with the computational elliciency of modern GPU-based approaches.

reduces computation time while maintaining high accuracy. The system involves converting each of the two surfaces into point clouds, comparing points between these clouds within a defined tolerance or threshold, and generating a new point cloud closely approximating the intersection of the two surfaccs. Thc method is cnhanced by mapping points to their parameter space, and then further processing the points such as by, for example, curve interpolation and/or convolution. The resulting curve in parameter space can be mapped back to the object. This approach not only accelerates the process of finding intcrscctions but also provides a framcwork for preserving geometric information between points, enhancing thc utility and accuracy of 3D surface modcling.

Additional aspects, advantages, features and objects of the disclosure will be made apparent from the drawings and the detailed description of the illustrative embodiments construed in conjunction with the appended claims that follow.

10

1.5

It will be appreciated that features of the disclosure are susceptible to being combined in various combinations 620 without departing from the scope of the disclosure as defined by the appended claims.

furthcr includcs mapping the curves found at 350 from parameter space to an object space at 370 and outputting the curves at 390.

The actions described above with reference to FIGS. 1-3 are only illustrative and other alternatives can also be provided wherc one or more actions arc added, one or more actions are removed, or one or more actions are provided in a different sequence without departing from the scope of the claims herein.

When point descriptions of the first and second surfaces have not bcen provided, it may be necessary to, from the descriptions of the first and second surfaces, generate, derive or define a first set or cloud of points lying on, at and/or otherwise representing the first surface and a second sel or cloud of points lying on, at and/or otherwise representing the second surface. For example, the set of points may be generated at or on each of the first and second surfaces by simply cvaluating paramctric polynomial descriptions of the surfaces over a chosen or pre-established grid of values. Further, generating the surface point sets may include food filling, practice of Homer's method or a combination of these. FIG. 4 illustrates the example set of intersecting surfaces of FIG. 2 with surface 270 shown as a point cloud or set of points 470.

25 In an efficiency improvement, divide and conquer techniques such as those employing quad trees or oct trees may eliminate, from evaluation and/or analysis, regions where intersection of the surfaces can be ruled out. For example, if evaluation of one of the surfaces does not yield values within The method may find, determine, establish, locate, iden- 30 a particular subdivision of the space containing the surfaces, tify. describe or obtain the intersection which may exist this subdivision does not need to be considered for interbetween first and second surfaces of one or more 3I) models section of the surfaces and a different subdivision can be in a parameler space and/or in an object space. The 3D checked for values from both surfaces. Altermatively, when models may be computer-aided design (CAD) models, comboth surfaces are evaluated to a value within a subdivision, puter-aided modeling (CAM) models, computer-aided-engi- 35 5  the intersection may be within this subdivision which can neering (CAE) models and/or computer-aided animation then be further subdivided to check, with higher resolution, models. A description of an output intersection may include for the location of the intersection. a set of intersection points and/or a set of interseclion curves.

FIGS. 1 &amp; 2 illustrate actions of a computer-implemented method for computing an intersection between first and. second surfaces, in accordance with an embodiment of the disclosure. In parl, the method is depicted as a collection of actions in a logical flow diagram, which represents a sequence of actions that may be implemented in hardware, software, or a combination thereof.

In cases where the poinl clouds were not derived from parameterized first and second surfaces, the method may further include parameterizing the first set of points on the first surface and a second set of points on the second surface or otherwise mapping the first set of points on the first surface and a second sel of points on the second surface to a parameter space. Alternatively and/or additionally, the first 5 set of points on the first surface and a second set of points On the second surface may be generated, unparameterized, in the object space of the surfaces or mapped from a parameter space to an object space.

FIGS. 5, 6 &amp; 8 illustrate actions of a detailed compulerimplemented method 500 for computing an intersection between first and second surfaces in accordance with an embodiment of the disclosure. The example method 500 is depicted as a collection of actions in a logical flow diagram, mented in hardware, software, or a combination thereof.

The choice of between parameter space and object space 50 may be made with consideration for the environmenl of the surfaces and/or 3D) model(s). For example. the greater accuracy of the parameter space may be suitable for CAE, CAM or CAD scenarios while the accuracy of object space may be sufficient for computer-aided animation.

Referring to FIG. 3, in another simple example. disclosed methods may take the form of method 300 which includes 60 which represents a sequence of actions thal may be implereceiving descriptions for intersecting surfaces at 310 and, in cases where the descriptions do not provide paramaterized. A first set of points representing the first surface and a first and second surfaces, mapping the surface descriptions second set of points representing the second surface are from object space to parameler space at 330. With the received at 510. A first set of intersection points may be surfaces mapped to parameter space, one or more curves are e 65 formed from the points representing the first surface and the found at 350 which fit points of the first surface sufficiently points representing the second surface. The first set of close 1o at least one point on the second surface. Method 300 d jo ps a J nss  q  s a

Referring to FIG. 1, in a simple example, disclosed methods, may take the form of method 100 which includes 40 receiving descriptions for first and second surfaces at 110 and, at 150, finding one or more curves that fit points on the first surface which are sufficiently close to at least one point On the second surface. The first and second surfaces may be described by data such as one or more collections of 45 computer-readable and screen displayable data descriptive of the first and second surfaces in a parameter space, such as parametric descriptions. Alternatively, the descriptions s       e case, the descriptions may be mapped from the object space to a parameter space so that the first and second surfaces are source or may be proprietary with example formats including but not being limited to IGES, STEP, STL, QIF, 3D PDF, ACIS and Parasolids. FIG. 2 illustrales an example sel of intersecting surfaces in an object space including a first intersection surface 210 and a second intersection surface 270. Curves found at 150 are output at 190.

representing the reference surface when the first surface taken as a reference surface against which the second surface is measured. The fatter surface may be selected as the leference surface so that mapping between a parameter space and an object space will have a more accurate result.

may be made contemporaneously or simultaneously by processing in parallel. For example, referring to FIG. 6, with surface point sets received at an action 610 (which may be indistinguishable from action 510), the distance between a n o p s nr  h nd s points representing the second surface may be computed contemporaneously, simultaneously or in parallel at 620 before determining whether additional, unevaluated points representing the first surface exist at 630.

In another example (not shown), the distances between lepresenting the second surface may be computed contemexample, the distances between a first point representing the s surface may be compared with the threshold contemporaneously, simultaneously or in parallel. In a still further example, the distances between two or more points of the set t 20 representing the first surface and a point representing the second surface may be compared contemporaneously, simultaneously or in parallel.

This first set of intersection points will be formed from identified, collected, gathered and/or grouped points representing the reference surface which are each less than a pre-established threshold distance from and/or are closer than the pre-established threshold distance to at least one 10 point representing the second surface. It follows that the points of the intersection set also have a proximity to points representing the second surface which is greater than a threshold. In an example, the points of the intersection set are grouped from parameterized points representing the first t15 surface.

The nearness of points representing the first surface to points representing the second surface is tested by, at 520, computing or otherwise determining a distance between each point representing the first surface and each point representing the second surface and, at 530, comparing the distances with a threshold distance.

In an example, distance may be computed as:

<!-- image -->

where (x,.: y1, Z1) refects a point of the first surface and (x2, Yz. Zz) reflects a point of the second surface.

FIG. 7 illustrates, along with surfaces 210 and 270, an example first se1 of inlersection points 710 which may resul1 25 from disclosed computer-implemented methods. FIG. 8 illustrates an example parameterizalion of an example sel of intersection points of FIG. 7.

30

Referring to FIG. 9, the points received at 910 on the first surface which are each less than a threshold distance from at least one point on the second surface may be ordered at 930. The points may be ordered according to any of a variety of example sorting techniques including but not limited to selection sorting. heap sorling. quick sorting. In another example, a starting point may be found by checking which other points of the intersection set lie on. When a point is found for which all points are on one side, that point is defined as the starting point. With a starting point established, other points may be scanned for their proximity to the slarting point and the points may be ordered according to decreasing proximity to the starting point as found in "nearest neighbor"' methods for determining the starting point.

45

When d&lt;e or d?&lt;o, the point representing the first surface is stored in a first set of intersection points at 540 and it is senting the second surface at 550. If there are additional points representing the second surface, the method advances 50 points of FIG. 7 treated with convolution. FIG. 11 illustrates as described above from 570.

In practice, point representations will be consistent in not points described in both spaces. At 530, it may be threshold. In other words. it may be determined whether the distance is greater than or equal to a threshold ε such that d≥8 or whether the square of the distance is greater than or equal to some other threshold o such that d'≥o. If so, it is determined whether there are additional points representing the second surface at 550. If there are additional points representing the second surface, the method advances to the next point at 570 in order to compute or otherwise determine the distance between the point representing the first surface and the next point representing the second surface.

The points on the first surface which are each less than a threshold distance from at least one point on the second surface may be filtered or otherwise processed at 950. For example, the points of the first intersection set may be filtered by convolving and/or eliminating redundant points thereof. FIG. 10 illustrates the example set of intersecting an example set of intersection points 1110 convolved and mapped to the object space of surfaces 210 and 270.

Continuing. at 970, one or more intersection curves which fit the set of intersection points are computed. The one or e 55 more interseclion curves may be or otherwise include a piecewise curve, a Bezier curve, a b-spline, a NURBs curve and/or another type of interpolating or approximating curve.

The first set of intersection points may lie on or otherwise represent the first surface in an object space of the first and/or second surfaces or in a parameter space of the first and/or second surfaces.

When it is determined there are no additional points representing the second surface, it is determined whether there are any additional points representing the first surface at 560. When there are additional points representing the first surface, the method advances to the next point representing the first surface at 570 and computes or otherwise determines the corresponding distance for each of the points representing the second surface. When there are no additional points representing the first surface, the first set of intersection points is output at 580.

For improved processing speed, two or more distance computations and/or two or more threshold comparisons

The one or more curves are output as the intersection at 990. The one or more intersection curves may be output as f 60 descriptive data in a parameler space of the first and/or second surfaces and/or the 3D model(s). The method may further include mapping the one or more curves from the parameter space to the object space, for example, with one or more polynomials which may be rational polynomials. 65 FIG. 12 illustrates, along with first surface 210 and second surface 270, an example fit of a curve 1210 to a set of interseclion points such as points 710 or points 1110.

Discloscd mcthods may fiurther include (not illustrated to avoid redundancy) using the descriptions of the first and second surfaces to find one or more curves that fit to a set of points on or otherwise representing the second surface which are each less than a threshold distance from at least one point on the first surfacc.

may be crcatcd bctwecn the onc or more curves fit to a first intersection set and the one or more curves fit to the second intersection set.

10

In some cases, input surfaces may not have an intersection and a notification or other indication of this condition may be output to a uscr. For cxample, if no pair of points representing each of the surfaces is less than the threshold A s e   oq rou u o e s be communicated by the method. In another example, if no pair of points is less than a given threshold, the threshold may bc adjusted for grcatcr sensitivity.

152

Thus, a second set of intersection points may be formed. from the set of points representing the first surface and the set of points representing the second surface. With the second surface taken as a reference surface against which the first surfacc is mcasured, the sccond sct of interscction a hsaa sod jo ns a jo pasqns e aq e s refercnce surfacc. This subsct will he formed from identified, collected, gathered and/or grouped points representing the reference surface which are each less than a pre-established threshold distance from and/or closer than the preestablished threshold distance to at least one point of the points representing the first surface. Alternatively stated, the points of the intersection set have a proximity to points 20 old.

The nearness of points representing the second surface to points representing the first surface is tested by determining a distance between each point representing the second 25 surface and each point representing the first surface and comparing the distances with a threshold distance. The second set of intersection points may lie on or otherwise represent the second surface in an object space of the first and/or second surfaces or in a parameter space of the first and/or sccond surfaccs.

Disclosed computer-implemented methods may be suitable for use in association with any of a variety of systems. FIG. 13 is a schematic illusiration of various components of accordance with an embodiment of the disclosure. System 1300 may include, but is not limited to, a memory 1310, computing hardwarc such as a CPU 1360, Input/Output (I/O) devices 1340, a parallel processing unit 1350, a slorage 1370, a configuration of sensors (not illustrated) and a system bus 1380 that operatively couples various components including memory 1310, CPU 1360, 1/O devices 1340, parallel processing unit 1350, storage 1370 and the sensors.

System 1300 also includes a power source (not shiown) for supplying electrical power to the various components of system 1300. The power source may, for example, include a rechargeable battery.

308

As with actions described above in refercnce to FIGS. 5 6, for improved processing speed, two or more computatemporaneously or simultaneously by processing in parallel. tions and/or two or more comparisons may be made conAs above with regard to points on the first surface, the points On the second surface which are each less than a threshold distance from at least one point on the first surface may be ordered and/or filtered.

Continuing, one or more intersection curves which fit the set of intersection points are computed and may be or include one or more interpolating and/or approximating curves. The one or more intersection curves may be output ou is a jo aods ned e u eep aandsp s second surfaces and/or the 3D model(s). The one or more curves may be mapped from the parameter space to the object space, for example, with one or more polynomials.

35

40

45

Ihe actions described above with reference to FlGS. 5-12 are only illustrative and oiher alternatives can also be 50 moving. pressing and/or swiping with a finger or a touchprovided where one or more actions are added, one or more actions are removed, or one or more actions are provided in a different sequence without departing from the scope of the claims herein.

Alternatively and/or additionally, the one or more curves that fit to a set of points representing the first surface which 60 are each less than a threshold distance from at least one point representing the second surface may be filtered, blended or averaged with the one or more curves that fit to a set of points representing the second surface which are each less than a threshold distance from at least one point representing the first surface or a central tendency of the first and second curves may be computed. For example, a healing surface

The method may further include (not illustraled) filtering, 55 ( blending, averaging or computing the central tendency of the identified intersection points on the first surface with the identified intersection points on the second surface.

Moreover, the sensors may include one or more of: an accelerometer, a magnetometer, a pressure sensor, a temperature sensor or a gyroscopic sensor. The sensors may be used to measure and collect data related to intersecting surfaces, intersections of the surfaces or to the surroundings 65 of the user. In some examples, the software product may be interfaced with the sensors. When executed on processor 1360, the sofiware product is configured to resolve and

I/O devices 1340 may include a display screen for presenting graphical images to a user of the system. The curves, points which may represent intersection surfaces or intersections therebetween. In some examples, the display screen may be a touch-sensitive display screen that is operable to receive tactile inputs from the user. These tactile inputs may. for example, include clicking, tapping, pointing. sensitive object like a pen. Additionally or alternatively, 1/O devices 1340 include a mouse or a joystick that is operable to receive inputs corresponding to clicking. pointing, and/or moving a pointer object on the graphical user interface. 1/O devices 1340 may also include a keyboard that is operable to receive inputs corresponding to pushing certain buttons on the keyboard. Additionally, 1/O devices 1340 may also include a microphone for receiving an audio input from the user, and a speaker for providing an audio output to the user.

CPU 1360 is capable of processing computer-readable surface-to-surface intersection application 1313 and other applications 1311 in order to execute various actions associated with the instructions.

Parallel processing unit 1350 may be of a variely capable of processing multiple threads in parallel. Parallel processing unit 1350 may be capable of processing any number of threads contemporaneously or simultaneously and may cmploy any of a variety of classes of parallel processing including but not limited to multi-core processing, symmelric multiprocessing. distributed computing, cluster computcomputing and combinations thereof.

integratc the outputs of thc sensors into uscful information about intersecting surfaces, intersections of the surfaces or to the surroundings of the user.

the sct of points reprcscnting the first surfacc and the set of points representing second surface. The first set of intersection points may be formed from a subset of the set of points representing the first surface. In a further example, the first s n    n  n points rcprcsenting the first surface which are cach less than a threshold distance from at least one point of the points represcnting the second surfacc.

Distance lesting module 1315 or parallel processing unit -10 1350 as configured in accordance with distance testing module 1315 may computc a distancc bctwccn cach point representing the first surface and each point representing the second surface, compare the distances with a threshold distance and form a first set of intersection points from each 1.5 point of the set representing the first surface which has a distance from one or more points of the set representing the second surface which is less than the threshold distance.

20

Memory 1310( optionally includes non-removable memory, removable memory, or a combination thereof. The non-removable memory, for example, includes RandomAccess Memory (RAM), Read-Only Memory (ROM), flash memory, or a hard drive. The removable memory. for 25 example, includes lash memory cards, memory sticks, or smart cards. Memory 1310 stores an application 1313 for computing/finding surface-to-surface intersections as well as other applications 1311.

Furthermore, I/O devices 1340 may include a network interfacc which optionally allows CPU 1360 to download points, curves, surfaces, 3D models or descriptions of any of these from a server or other source, for example. via a communication network. Conversely, the network interface Optionally allows CPU 1360 to upload points, curves, surfaccs, 3D models or dcscriptions of any of these to a scrver. Moreover, the network interface optionally allows system 1300 to communicate with other data processing units.

surface-to-surface intersections may include a sampling module 1314, a distance testing module 1315, an ordering module 1316. a filtering module 1317, a curve fitting module 1318 and a mapping module 1319. Modules 1314, 1315, 1316, 1317, 1318 and 1319 may, for example. be parts of a software product associated with the intersection computing service provided by the system. Executing the software product on the CPU 1360 and/or parallel processing unit 1350, in part, results in generating and rendering a graphical user interface on the display screen. The graphical user interface is configured to facilitate user interactions with the intersection computing service. The network interface may allow CPU 1360 to access a server to update the sofiware product and/or download one or more new software products associaled with the intersection computation service.

Example constructions for testing module 1315 as coded in computer readable instructions include:

def SSI\_bspline (meshF,meshC,gridTol):

- for i in range (len (meshF)):

point\_list=[]; uv\_list=[]

for j in range (len (meshC):

- point\_list.append ((meshC [il [O], meshC [il [1], meshC [j] [2])) retum point\_list
- -[ [ um) +xx(o [!l qs-[o][ usm) J! meshC [j] [1])**2+ (meshF [i] [2]-meshC [j] [2]) **2)&lt;gridTol:

Distance testing module 1315 or parallel processing unit Application 1313 for computing or otherwise finding 30 1350 as configured in accordance with distance testing module 1315 may form a second set of intersection points from the set of points representing the first surface and the set of points representing second surface. The second set of intersection points may be formed from a subset of the set a 35 of points representing the second surface. In a further example, the second set of intersection points may be formed from a subset of points represcnting the second surface which are each less than a threshold distance from at least one point representing the first surface.

45

Distance testing module 1315 or parallel processing unit 50 as configured in accordance with distance testing module 1315 may determine, compare and form by computing, lofqo e u no soeds reeed e u suon pue aed space of the first and/or second surfaces depending on the application of the application 1313. In some example systems, mapping module 1319 may translate points or descriptions thereof between parameter and object spaces periodically, during strategically determined phases of intersection computations and output or on demand of CPU 1360, parallel processing unit 1350, application 1313 or system 1310.

Distance testing module 1315 or parallel processing unit 1350 as configured in accordance with distance testing module 1315 may compute a distance between each point representing the second surface and each point representing the first surface, compare the distances with a threshold distance and form a second set of intersection points from each point representing the second surface which has a distance from one or more points representing the first surface which is less than the threshold distance.

Parallel processing unit 1350 is configured by instructions One or more intersection curves. Sampling module 1314 or parallel processing unit 1350 as configured in accordance with sampling module 1314 may, from the description of the 60 first surface, derive a set of points or a point cloud representing the first surface and, from the description of the second surface, derive a set of points or a point cloud representing the second surface.

Application 1313 is configured to receive, from storage 1370. l/0 1340 or the network interface, a description for each of the first and second surfaces. The first and second. s   n object space. In some example systems, mapping module 1319 may translate surfaces or descriptions thereof between parameter and object spaces periodically, during strategically determined phases of intersection computations and. n sod pd ei n e pp o no pnn 1350, application 1313 or system 1310.

Distance testing module 1315 or parallel processing unit 1350 as configured in accordance with distance testing module 1315 may form a first sel of intersection points from Distance testing module 1315 or parallel processing unit 1350 as configured in accordance with distance testing module 1315 may determine the distance between each point of a set of points representing the first surface and each 65 point of a set of points representing the second surface by determining the distance between a first point of the set representing the first surface and each point of the set

In some examples, the sensors may include a GPS sensor for determining one or more absolute spatial positions of the uscr, thc system or 3D models upon a surfacc of the Earth. In some examples, the sensors may include a timer for including time-stamps with stored intersecting surfaces, interseclions of the surfaces or the surroundings of the user. Alternatively. CPU 1360 may provide system time as refcrencc for including thc time-stamps.

representing thc sccond surface in parallel with determining the distance between a second point of the set representing the first surface and each point of the set representing the second surface.

Filtering module 1317 or parallel processing unit 1350 as configured in accordance with filtering module 1317 may filter the points on the first surface which are each less than a threshold distance from at least one point on the second. surfacc. For cxample, thc points of the first interscction sct may be filtered by convolving and/or eliminating redundant points thereof.

parallel proccssing unit 1350, may storc points, curvcs, surfaces, 3D models or descriptions of any of these in storage 1370.

The system 1300 is optionally implemented by way of at Ordering module 1316 or parallel processing unit 1350 as least one of: a mobile phone, a smart telephone, an MID, a configured in accordance with ordcring module 1316 may tablet computcr, a UMPC, a phablet computer, a PDA, a wcb order points on the first surface which are each less than a pad, a PC, a handheld PC, a laptop computer, a desktop threshold distance from at least one point on the second computer, a server workstation, a main frame, a supercomsurface. Ordering module 1316 or parallel processing unit puter, an NAS device, a large-sized touch screen with an 1350 as configured in accordance with ordering module 10 embedded PC, and an interactive entertainment device, such 1316 may order the points according to any of a varicty of as a game console, a TV sct and an STB. s   nr n  s Embodiments of the disclosure provide a computer prosorting, hcap sorting. quick sorting or a combination of gram product that includes a non-transitory or non-transient these. In another example, ordering module 1316 or parallel 1.5 processing unit 1350 as configured in accordance with ecutable code for intersection computation. The code, when pe yuod hus e py e Ae 9ier anpo So executed, is configured to perform actions of the methods as determine which sides of an initially chosen point other described in conjunction with FIGS. 1, 3, 5, 6 &amp; 9. As points lie on. The starting point may be defined as a point for actions of the disclosed mcthods may be provided in differwhich all points lie only on a single side. In a "nearest 2o ent sequences, so the computer-executable code may be neighbor' technique for determining the starting point, configured to provide a service having a different sequence s o  ssd pd n i of actions from those illustrated in FIGS. 1, 3, 5, 6 &amp; 9. In configured in accordance with ordering module 1316 may some examples, the code may be downloaded from a soffte d s   d n nn sd r the points may be ordered according to decreasing proximity y25 a data processing unit. to the starting point.

30

Curve fitting module 1318 or parallel processing unit 35 1350 as configured in accordance with curve fiing module 1318 fit one or more curves to the set or sets of the FIG. 13 is merely an example, which should not unduly interseclion points and oulput the one or more interseclion limit the scope of the claims herein. It is to be understood curves. The one or more intersection curves may be or that the specific designation for system 1300 is provided as otherwisc inelude a pieccwisc curve, a Bezicr curve, a 4o an cxample and is not to be construed as limiting system b-spline and/or a NURBs curve. Curve fiting module 1318 1300, application 1313 or modules 1314-1319 to specific or parallel processing unit 1350 as configured in accordance numbers, types, or arrangements. A person having ordinary with curve fitting module 1318 may output the one or more skill in the art will recognize many variations, alternatives, intersection curves as data descriptive of the one or more and modifications of embodiments of the disclosure. intersection curves in a parameler space or an objecl space 45 Embodiments of the disclosure are susceptible to being of the first and/or second surfaces. In some example sysused for various purposes, including, though not limited to, tems, mapping module 1319 may translate curves or enabling users to compute SSI by employing a parallel descriptions thereof between parameter and object spaces processor-acceleratable algorithm that not only accelerates periodically, during strategically determined phases of interthe task of finding intersections but also provides a frameseclion computations and ouiput or on demand of CPU J 50 work for preserving geometric information between points, 1360, parallel processing unit 1350, application 1313 or enhancing the utility and accuracy of 3D) surface modeling. system 1310. Resulting intersections may be useful for object modeling. trimming, collision detection or a combination of these.

Generally. the computer program product may be represented in modules 1314-1319 which may include various routines, programs, objects, components, data structures, etc., that are stored or embodied in one or more types of computer-readable media, as described above, and perform particular tasks or opcrations or implcmcnt particular abstract data types. Computer-readable media may be understood as any available media that can store and/or embody computer executable instructions and that may be accessed by a computing system such as 1300 or computing process.

While disclosed methods have generally been described s 55 as applicable to two surfaces in 3-dimensional space, appliof dimensions.

Modifications to embodiments of the disclosure described in the foregoing are possible without departing from the , , s s sss s "incorporating", "consisting of", "have", "is" used to describe and claim the disclosure are intended to be conslrued in a non-exclusive manner, namely allowing for , 65 items, components or elements not explicitly described also to be present. Reference to the singular is also to be construed to relate to the plural.

Moreover, storage 1370 is a non-transient data storage medium. The software product, when executed on the processor 1360, is optionally coupled to storage 1370, and is configured to cause processor 1360 to receive surface descriptions from storage 1370. Further. the software product, when executed on CPU 1360 or parallel processing unit 1350. is optionally coupled to storage 1370, and is configured to substantially continuously record and update intersection points or curves in storage 1370. The software product, when executed on CPU 1360 or parallel processing unit 1350, is optionally configured to record its last updated status in storage 1370, in case of interruplions in the Operable state of modules 1314-1319, application 1313, system 1300 and/or the graphical user interface. Additionally, the sofiware product, when executed on CPU 1360 or What is claimed is:

- receiving a description for each of the first and second 5 surfaccs;

1. A computer-implemented method for computing an intersection between first and second surfaces of one or more 3D models, comprising:

- using the descriptions to compute one or more intersection curves by:
- forming a first set of intersection points from each point of the set representing the first surface which has a distance from one or more points of the set representing the second surface which is less than a 15 threshold distance; and
- determining a distance between each point of a set of points representing the first surface and each point of 10 a sct of points represcnting thc second surface;

outputting the one or more intersection curves.

by computing the distance betwcen a first point of the sct representing the first surface and each point of the set representing the second surface in parallel with computing the distance between a second point of the set representing the first surface and each point of the set representing the sccond surfacc.

9. The system as set forth in claim 6, wherein the processor is configured to output the one or more intersection curves as one or more curves which fit the first set of intersection points.

10. The systcm as sct forth in claim 6, whercin the processor is configured to form the first set of intersection points by forming a sct of interscction points which lic on the first surface in an object space of the first and/or second surfaces.

11. A non-transitory computer readable medium for computing an intersection, comprising computer readable instructions which, when cxecuted by a processor, cause one

- receive a description for each of the first and second surfaces;

2. The mcthod as set forth in claim 1, wherein using the descriptions to compute the one or more intersection curves further comprises forming a second set of intersection 20 or more computers to: points.

3. The method as set forth in claim 1, wherein the determining the distance between each point of a set of of points representing the second surface further comprises 25 determining the distance belween a first point of the sel representing the first surface and each point of the set representing the second surface in parallel with determining the distance between a second point of the set representing the first surface and each point of the set representing the 30 second surface.

5. The method as set forth in claim 1, wherein forming the first set of interscction points further comprises forming a first set of intersection points which lie on the first surface in an object space of the first and/or second surfaces.

4. The method as set forth in claim 1, wherein outputting. the one or more intersection curves further comprises oulputting one or more curves which fit the set of intersection points.

6. A system for computing an intersection, comprising: a memory;

a storage;

- receive a description for each of the first and second 45 surfaces;
- a parallel computing processor configured by instruclions stored on the memory to:
- use the descriptions to compute one or more intersection curves by

14. The computer readable medium as set forth in claim 11, wherein the computer readable instructions which cause the one or more computers to output the one or more intersection curves further cause the one or more computers 1 55 lo output one or more curves which fit the first sel of intersection points.

40 13. The computer readable medium as set forth in claim 11, wherein the computer readable instructions which cause the one or more computer to determine the distance between each point of a set of points representing the second surface further cause the one or more computers to determine the distance between a first point of the set representing the first surface and each point of the set representing the second surface in parallel with determining the distance between a second point of the set representing the first surface and each 1 50 point of the set representing the second surface.

- computing a distance between each point of a set of points representing the first surface and each point of a set of points representing the second surface; and
- forming a first set of intersection points from each point of the set representing the first surface which has a distance from one or more points of the sel representing the second surface which is less than a threshold distance; and

Output the one or more intersection curves.

15. The computer readable medium as set forth in claim 11, wherein the computer readable instructions which cause the one or more computers to form the first set of interseca first set of intersection points which lie on the first surface in an object space of the first and/or second surfaces.

7. The system as set forth in claim 6 wherein the processor is configured to use the descriptions to compute the one or 60 tion points further cause the one or more computers to form more intersection curves by forming a second set of intersection points from the set of points representing the first surface and the set of points representing second surface.

16. A computer-implemented method for determining a set of intersection points between first and second surfaces

- receiving a description for each of the first and second surfaces,

8. 'The system as set forih in claim 6, wherein the processor is configured to compute the distance between 65 of one or more 3D models, comprising: each point of a set of points representing the first surface and each point of a set of points representing the second surface

- use the descriptions to
- determine a distance between each point of a set of points representing the first surface and each point of a set of points representing the second surface: and
- form a first set of intersection points from each point of the set representing the first surface which has a distance from one or more points of the set representing the second surface which is less than a threshold distance; and

output the first set of intersection points.

12. The computer readable medium as set forih in claim 11, wherein the computer readable instructions which cause 35 the one or more computers to use the descriptions to compute the one or more intersection curves further cause the one or more computers to form a second set of intersurface and the set of points representing second surface.

- from the description of thc first surfacc, deriving a sct of points representing the first surface; and
- forming the set of intersection points from points representing the first surface which are distanced from the second surface by less than a threshold distance.

5