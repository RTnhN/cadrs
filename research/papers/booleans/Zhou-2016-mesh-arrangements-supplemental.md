## Supplemental Material: Mesh Arrangements for Solid Geometry

Qingnan Zhou 1

Eitan Grinspun 2

Denis Zorin 1

Alec Jacobson 2

1 New York University

2 Columbia University

## 1 Thingiverse Dataset

We tested our self-union algorithm on 10,000 meshes crawled from Thingiverse.com, a popular 3D model repository. We provide information about the entire data set in 'supplementary data/input summary.csv'. The following fields are included:

- 'thing id': Id used by Thingiverse.com for index each 'thing'. Information about each individual thing can be retrieved from http://www.thingiverse.com/thing: { thing id } .
- 'file id': A unique id for each file.
- 'license': The license associated with each file.
- 'link': Direct link to download each model.
- 'no duplicated faces': If the mesh contain any duplicated faces.
- 'closed': If the mesh is combinatorially closed.
- 'edge manifold': If every edge is adjacent to exact 2 faces (or 1 face for boundary edge).
- 'no degenerated faces': If there exists a triangle with three vertices exactly collinear.
- 'vertex manifold': If local neighborhood of every vertex is a topological disc.
- 'single component': If the mesh contains just a single connected component.
- 'is pwn': If the mesh induces a piece-wise constant winding field.

## 2 Data set from [Barki et al. 2015]

Beside the Thingiverse dataset, we also conduct exhaustive tests on the data set from [Barki et al. 2015]. This data set contains 26 models widely used by the computer graphics community. We conduct pair-wise union, intersection and difference for all 351 pairs. 1404 results are generated. The union and self intersection results are shown in Figure 1. The difference results are shown in Figure 2.

In addition to pair-wise boolean tests, We also compute the consecutive union of each model with a rotated copy of itself for 10 times.

All 1404 + 26 mesh files can be found inside 'barki test.zip' in the supplemental materials.

## References

BARKI, H., GUENNEBAUD, G., AND FOUFOU, S. 2015. Exact, robust, and efficient regularized booleans on general 3d meshes. Computers and Mathematics with Applications .

Figure 1: Result matrix showing union and intersection results of all mesh pairs. Union results are shown above the diagonal of the result matrix. Intersection results are shown below the diagonal. The diagonal shows the self-union outputs. Empty spot in the intersection region indicate the input mesh does not intersect.

<!-- image -->

Figure 2: Result matrix showing P \ Q (above diagonal) and Q \ P below diagonal.

<!-- image -->