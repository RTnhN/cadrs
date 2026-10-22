## Polyhedral Mass Properties (Revisited)

David Eberly, Geometric Tools, Redmond WA 98052 https://www.geometrictools.com/

This work is licensed under the Creative Commons Attribution 4.0 International License. To view a copy of this license, visit http://creativecommons.org/licenses/by/4.0/ or send a letter to Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

Created: December 31, 2002

Last Modified: November 3, 2009

## Contents

|   1 | Introduction                     |   2 |
|-----|----------------------------------|-----|
|   2 | Reduction of Volume Integrals    |   2 |
|   3 | Computation of Surface Integrals |   3 |
|   4 | Comparison to Mirtich's Formulas |   5 |
|   5 | Pseudocode                       |   5 |

## 1 Introduction

This document takes a second look at the article by Brian Mirtich, Fast and accurate computation of polyhedral mass properties , Journal of Graphics Tools, vol. 1, no. 2, pp. 31-50, 1996. The article discusses how to compute the mass and inertia tensor for a solid, simple polyhedron of constant mass density. The construction uses the Divergence Theorem from calculus for reducing volume integrals to surface integrals, a reduction from three-dimensional integrals to two-dimensional integrals. The polyhedron surface is a union of planar faces, so the surface integrals are effectively integrals in various planes. Projection of these faces onto coordinate planes are used to set up yet another reduction in dimension. Green's Theorem, the twodimensional analog of the Divergence Theorem, is used to reduce the planar integrals to line integrals around the boundary of the projected faces.

Two important points emphasized in the paper are (1) the projection of the polyhedron faces onto the appropriate coordinate planes to avoid numerical problems and (2) the reduction using Green's Theorem to obtain common subexpressions (integrals of polynomials of one variable) to avoid redundant calculations. Item (2) occurs to handle polyhedron faces with four or more vertices. Item (1) is necessary in order to robustly compute what is required by item (2). When the polyhedron faces are triangles, neither items (1) nor (2) are necessary. A simpler construction is provided here when the polyhedron faces are triangles. A consequence of the formulas as derived in this document is that they require significantly less computational time than does Mirtich's formulas. I suspect that for nontriangular faces, Mirtich's formulas are reducible to simpler expressions.

## 2 Reduction of Volume Integrals

The mass, center of mass, and inertia tensor require computing volume integrals of the type

<!-- formula-not-decoded -->

where V is the volumetric region of integration and dV is an infinitesimal measure of volume. The function p ( x, y, z ) is a polynomial selected from 1, x , y , z , x 2 , y 2 , z 2 , xy , xz , and yz . We are interested in computing these integrals where V is the region bounded by a simple polyhedron. A volume integral may be converted to a surface integral via the Divergence Theorem from calculus:

<!-- formula-not-decoded -->

where S is the boundary of the polyhedron, a union of triangular faces, and where dS is an infinitesimal measure of surface area. The function F ( x, y, z ) is chosen so that ∇ · F = p . The vector N denotes outward-pointing, unit-length surface normals. The choices for F in the Mirtich paper are

<!-- formula-not-decoded -->

The computational effort is now devoted to calculating the integrals ∫ S N · F dS . The boundary S is just the union of polyhedral faces F . An outward-pointing, unit-length normal to face F is denoted by N F . The surface integral decomposes to

<!-- formula-not-decoded -->

The integrals to be computed are now reduced to

<!-- formula-not-decoded -->

## 3 Computation of Surface Integrals

We now need to compute integrals of the form

<!-- formula-not-decoded -->

where ℓ is one of ı ,  , or k and where q is one of x , x 2 , y 2 , z 2 , x 3 , y 3 , z 3 , x 2 y , y 2 z , or z 2 x . Let the triangular face be counterclockwise ordered and have vertices P i = ( x i , y i , z i ), 0 ≤ i ≤ 2. Two edges are

<!-- formula-not-decoded -->

for 1 ≤ i ≤ 2. A parameterization of the face is

<!-- formula-not-decoded -->

where u ≥ 0, v ≥ 0, and u + v ≤ 1. The infinitesimal measure of surface area is

<!-- formula-not-decoded -->

and the outer pointing unit-length face normal is

<!-- formula-not-decoded -->

The integrals in equation (6) reduces to

<!-- formula-not-decoded -->

where x ( u, v ), y ( u, v ), and z ( u, v ) are the components of the parameterization in equation (8).

The integrals on the right-hand side of equation (11) can be computed symbolically, either by hand or by a symbolic algebra package. The formulas listed later were computed using Mathematica. The instructions are

```
x = x0+u ∗ ( x1 -x0)+v ∗ ( x2 -x0 ) y = y0+u ∗ ( y1 -y0)+v ∗ ( y2 -y0 ) z = z0+u ∗ ( z1 -z0)+v ∗ ( z2 -z0 ) i n t x = Together [ Integrate [ x , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t x x = Together [ Integrate [ xˆ2 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t y y = Together [ Integrate [ yˆ2 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t z z = Together [ Integrate [ z ˆ2 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t x x x = Together [ Integrate [ xˆ3 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t y y y = Together [ Integrate [ yˆ3 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t z z z = Together [ Integrate [ z ˆ3 , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t x x y = Together [ Integrate [ xˆ2 ∗ y , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t y y z = Together [ Integrate [ yˆ2 ∗ z , { v , 0 , 1 } , { u,0,1 -v } ] ] i n t z z x = Together [ Integrate [ zˆ2 ∗ x , { v , 0 , 1 } , { u,0,1 -v } ] ]
```

Common subexpressions may be obtained by some additional factoring. Define

<!-- formula-not-decoded -->

Think of these as macros where the input argument is a textual replacement wherever w occurs in the right-hand sides. Also define the macro

<!-- formula-not-decoded -->

The specific expressions required in the surface integrals are listed below in terms of w . Each macro is expanded three times, once for each of x , y , and z .

<!-- formula-not-decoded -->

The square brackets [ · ] indicate that the subexpression is computed and saved in temporary variables for later use. The curly braces {·} indicate that the subexpression was computed earlier and can be accessed from temporary variables. The number of subexpressions that must be stored at any one time is small, so cache coherence should not be an issue when enough floating point registers are available for storing the subexpressions (see the pseudocode later in this document).

The integrals are

<!-- formula-not-decoded -->

## 4 Comparison to Mirtich's Formulas

Let us compare the formulas for Q x = ( N F · ı ) ∫ F xdS . In this document the integral is

<!-- formula-not-decoded -->

In Mirtich's formulas there are two possibilities for computing Q x . One occurs when the projection variable is γ = z . Assembling the pieces of the corresponding formulas in the paper and switching to the notation used in this document,

<!-- formula-not-decoded -->

The final formula requires much more computational time than the one derived in this document. In fact the numerator is exactly divisible by the denominator and the fraction reduces to x 0 + x 1 + x 2 , as it should to be equivalent to the Q x in equation (16). The reduction was verified using Mathematica. If the projection variable is γ = x ,

<!-- formula-not-decoded -->

The correctness of this formula was verified using Mathematica; in fact it reduces to the one in equation (16). The computational requirements for this expression are enormous compared to that of equation (16).

Comparisons between the formulas for the other integrals is possible, but you will find that the differences in computational time become even greater than in the example shown here.

## 5 Pseudocode

The pseudocode for computing the integrals is quite simple. The polyhedron vertices are passes as the array p[] . The number of triangles is tmax . The array index[] has tmax triples of integers that are indices into the vertex array. The return values are the mass, the center of mass, and the inertia tensor relative to the center of mass. The code assumes that the rigid body has constant density 1. If your rigid body has constant density D, then you need to multiply the output mass by D and the output inertia tensor by D.

```
MACRO Subexpressions (w0,w1,w2, f1 , f2 , f3 , g0 , g1 , g2 ) { temp0 = w0+w1; f 1 = temp0+w2; temp1 = w0 ∗ w0; temp2 = temp1+w1 ∗ temp0 ; f 2 = temp2+w2 ∗ f 1 ; f 3 = w0 ∗ temp1+w1 ∗ temp2+w2 ∗ f 2 ; g0 = f2+w0 ∗ ( f 1+w0 ) ; g1 = f2+w1 ∗ ( f 1+w1 ) ; g2 = f2+w2 ∗ ( f 1+w2 ) ; } void Compute ( Point p [ ] , i n t tmax , i n t i n d e x [ ] , Real& mass , Point& cm, Matrix& i n e r t i a ) { constant Real mult [ 1 0 ] = { 1/6 ,1/24 ,1/24 ,1/24 ,1/60 ,1/60 ,1/60 ,1/120 ,1/120 ,1/120 } ; Real i n t g [ 1 0 ] = { 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 , 0 } ; // order : 1 , x , y , z , x ˆ2 , y ˆ2 , z ˆ2 , xy , yz , zx
```

```
f o r ( t = 0; t < tmax ; t++) { // get v e r t i c e s of t r i a n g l e t i 0 = index [3 ∗ t ] ; i 1 = index [3 ∗ t +1]; i 2 = index [3 ∗ t +2]; x0 = p[ i0 ] . x ; y0 = p[ i0 ] . y ; z0 = p[ i0 ] . z ; x1 = p[ i1 ] . x ; y1 = p[ i1 ] . y ; z1 = p[ i1 ] . z ; x2 = p[ i2 ] . x ; y2 = p[ i2 ] . y ; z2 = p[ i2 ] . z ; // get edges and c r o s s product of edges a1 = x1 -x0 ; b1 = y1 -y0 ; c1 = z1 -z0 ; a2 = x2 -x0 ; b2 = y2 -y0 ; c2 = z2 -z0 ; d0 = b1 ∗ c2 -b2 ∗ c1 ; d1 = a2 ∗ c1 -a1 ∗ c2 ; d2 = a1 ∗ b2 -a2 ∗ b1 ; // compute i n t e g r a l terms Subexpressions (x0 , x1 , x2 , f1x , f2x , f3x , g0x , g1x , g2x ) ; Subexpressions (y0 , y1 , y2 , f1y , f2y , f3y , g0y , g1y , g2y ) ; Subexpressions (z0 , z1 , z2 , f1z , f2z , f3z , g0z , g1z , g2z ) ; // update i n t e g r a l s i n t g [ 0 ] += d0 ∗ f 1 x ; i n t g [ 1 ] += d0 ∗ f 2 x ; i n t g [ 2 ] += d1 ∗ f 2 y ; i n t g [ 3 ] += d2 ∗ f 2 z ; i n t g [ 4 ] += d0 ∗ f 3 x ; i n t g [ 5 ] += d1 ∗ f 3 y ; i n t g [ 6 ] += d2 ∗ f 3 z ; i n t g [ 7 ] += d0 ∗ ( y0 ∗ g0x+y1 ∗ g1x+y2 ∗ g2x ) ; i n t g [ 8 ] += d1 ∗ ( z0 ∗ g0y+z1 ∗ g1y+z2 ∗ g2y ) ; i n t g [ 9 ] += d2 ∗ ( x0 ∗ g0z+x1 ∗ g1z+x2 ∗ g2z ) ; } f o r ( i = 0; i < 10; i ++) i n t g [ i ] ∗ = mult [ i ] ; mass = intg [ 0 ] ; // c e n t e r of mass cm. x = intg [1]/ mass ; cm. y = intg [2]/ mass ; cm. z = intg [3]/ mass ; // i n e r t i a t e n s o r r e l a t i v e to c e n t e r of mass i n e r t i a . xx = intg [5]+ intg [6] -mass ∗ (cm. y ∗ cm. y+cm. z ∗ cm. z ) ; i n e r t i a . yy = intg [4]+ intg [6] -mass ∗ (cm. z ∗ cm. z+cm. x ∗ cm. x ) ; i n e r t i a . z z = intg [4]+ intg [5] -mass ∗ (cm. x ∗ cm. x+cm. y ∗ cm. y ) ; i n e r t i a . xy = -( i n t g [7] -mass ∗ cm. x ∗ cm. y ) ; i n e r t i a . yz = -( i n t g [8] -mass ∗ cm. y ∗ cm. z ) ; i n e r t i a . xz = -( i n t g [9] -mass ∗ cm. z ∗ cm. x ) ; }
```