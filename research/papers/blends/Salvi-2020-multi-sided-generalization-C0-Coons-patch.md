## A multi-sided generalization of the C 0 Coons patch

## Péter Salvi

Budapest University of Technology and Economics

## Abstract

Most multi-sided transfinite surfaces require cross-derivatives at the boundaries. Here we show a general n-sided patch that interpolates all boundaries based on only positional information. The surface is a weighted sum of n Coons patches, using a parameterization based on Wachspress coordinates.

## 1. Introduction

Filling an n -sided hole with a multi-sided surface is an important problem in CAGD. Usually the patch should connect to the adjacent surfaces with at least G 1 continuity, but in some applications only positional ( C 0 ) continuity is needed, and normal vectors or cross derivatives at the boundary curves are not available.

For n = 4, the C 0 Coons patch 1 solves this problem; in this paper we show how to generalize it to any number of sides.

## 2. Previous work

Most transfinite surface representation in the literature assume G 1 constraints, and the patch equations make use of the fixed cross-derivatives at the boundary. This can be circumvented by generating a normal fence automatically, e.g. with a rotation minimizing frame 3 ; however, in a C 0 setting this is an overkill, simpler methods exist.

One well-known solution is the harmonic surface, which creates a 'soap film' filling the boundary loop by solving the harmonic equation on a mesh with fixed boundaries. This, however, minimizes the total area of the surface, which often has unintuitive results, see an example in Section 4.

The basic idea of the proposed method, i.e., to define the surface as the weighted sum of n Coons patches, each interpolating three consecutive sides, is the same as in the CR patch 2 .

## 3. The multi-sided C 0 Coons patch

Let Ci ( t ) : [ 0 , 1 ] → R 3 denote the i -th boundary curve. Let us also assume Ci ( 1 ) = Ci + 1 ( 0 ) for all i (with circular indexing). Then the ribbon Ri is defined as a C 0 Coons patch interpolating Ci -1 , Ci , Ci + 1 , and C opp i - a cubic curve fitted onto the initial and (negated) end derivatives of sides i + 2 and i -2, respectively (see Figure 1).

Figure 1: Construction of a four-sided Coons ribbon.

<!-- image -->

## Formally,

<!-- formula-not-decoded -->

where C opp is defined as the Bézier curve † determined by the control points

<!-- formula-not-decoded -->

<!-- formula-not-decoded -->

The surface is defined over a regular n -sided polygon. The Wachspress coordinates of a domain point p are defined as

† Except for n = 3, where C opp degenerates to the point Ci + 1 ( 1 ) .

Figure 2: Comparison with the harmonic surface on a 5-sided boundary loop.

<!-- image -->

̸

<!-- formula-not-decoded -->

̸

where Di ( p ) is the perpendicular distance of p from the i -th edge of the domain polygon. Ribbon parameterization is based on these generalized barycentric coordinates:

<!-- formula-not-decoded -->

It is easy to see that si , di ∈ [ 0 , 1 ] , and that di has the following properties:

1. di = 0 on the i -th side.

2. di = 1 on the 'far' sides (all sides except i -1, i and i + 1).

3. di -1 + di + 1 = 1 on the i -th side.

Finally, we define the patch as

<!-- formula-not-decoded -->

where Bi is the blending function

<!-- formula-not-decoded -->

The interpolation property is satisfied due to the properties of di mentioned above.

(Note: si in Eq. (5) cannot be evaluated when di = 1, but at these locations the weight Bi ( di ) also vanishes.)

## 4. Examples

Figure 2 shows a comparison with the harmonic surface, which - due to its area minimizing property - results in an unnaturally flat patch.

Figure 3 shows a model with 5 patches: two 3-sided, one 4-sided, one 5-sided, and one 6-sided. The mean curvature map and contouring both show good surface quality.

## Conclusion

We have defined a natural generalization of the C 0 Coons patch - a lightweight and efficient multi-sided surface representation, applicable when only positional data is available.

Figure 3: The 'pocket' model.

<!-- image -->

## Acknowledgements

This work was supported by the Hungarian Scientific Research Fund (OTKA, No. 124727).

## References

1. Steven Anson Coons. Surfaces for computer-aided design of space forms. Technical Report MIT/LCS/TR41, Massachusetts Institute of Technology, 1967.
2. Péter Salvi, Tamás Várady, and Alyn Rockwood. Ribbon-based transfinite surfaces. Computer Aided Geometric Design , 31(9):613-630, 2014.
3. Wenping Wang, Bert Jüttler, Dayue Zheng, and Yang Liu. Computation of rotation minimizing frames. ACM Transactions on Graphics , 27(1):2, 2008.