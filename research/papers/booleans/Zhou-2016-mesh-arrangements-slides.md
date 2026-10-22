## Mesh Arrangements for Solid Geometry

Qingnan Zhou, Eitan Grinspun, Denis Zorin, Alec Jacobson

## Motivation: Fabrication

## Motivation: Fabrication

<!-- image -->

## Motivation: Fabrication

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Thingi10K Dataset

<!-- image -->

## Problem Statement

<!-- image -->

## Problem Statement

<!-- image -->

## Problem Statement

<!-- image -->

Mesh Arrangement

<!-- image -->

<!-- image -->

## Previous Works: CGAL

<!-- image -->

## Previous Works: CGAL

<!-- image -->

## Previous Works: Cork, QuickCSG

<!-- image -->

## Input and Output

Valid Input

<!-- image -->

Valid Output

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Winding Number

<!-- image -->

## Definitions

## PWN Mesh:

A mesh that induces P iecewise-constant W inding N umbers.

## Definitions

## PWN Mesh:

A mesh that induces P iecewise-constant W inding N umbers.

## Solid Mesh:

A PWN mesh that

- induces 0/1 winding number field
- without self-intersection or degeneracies

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

Multiple Components

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

Vertex Non-manifold

<!-- image -->

Edge Non-manifold

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

Vertex Non-manifold

Edge Non-manifold

<!-- image -->

Degenerated Faces

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

<!-- image -->

Vertex Non-manifold

Edge Non-manifold

<!-- image -->

Degenerated Faces

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

<!-- image -->

Vertex Non-manifold

<!-- image -->

Nested Components

Edge Non-manifold

<!-- image -->

Degenerated Faces

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

<!-- image -->

Vertex Non-manifold

<!-- image -->

Nested Components

<!-- image -->

Edge Non-manifold

Overlapping Faces

<!-- image -->

Degenerated Faces

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

<!-- image -->

Vertex Non-manifold

<!-- image -->

Nested Components

<!-- image -->

Edge Non-manifold

<!-- image -->

Overlapping Faces

<!-- image -->

Degenerated Faces

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

## PWN Mesh

PWN Mesh : A mesh that induces P iecewise-constant W inding N umbers.

<!-- image -->

<!-- image -->

Multiple Components

<!-- image -->

<!-- image -->

Vertex Non-manifold

<!-- image -->

Nested Components

<!-- image -->

Edge Non-manifold

<!-- image -->

Overlapping Faces

Degenerated Faces

<!-- image -->

Topologically Open

<!-- image -->

## Valid Input

## 10,000 Meshes from Thingiverse

<!-- image -->

## Input and Output

Input: PWN Meshes

<!-- image -->

Output: Solid Mesh

<!-- image -->

## Input and Output

<!-- image -->

Output: Solid Mesh

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Mesh Arrangement

<!-- image -->

## Algorithm

Input Meshes

<!-- image -->

## Algorithm

Input Meshes Resolving (Self-)Intersections

<!-- image -->

## Algorithm

Input Meshes Resolving (Self-)Intersections Extracting Cells

<!-- image -->

## Algorithm

Input Meshes Resolving (Self-)Intersections Extracting Cells Propagating Winding Numbers

<!-- image -->

(0, 0, 0)

(1, 0, 0)

(0, 1, 0)

(0, 0, 1)

(1, 1, 0)

(0, 1, 1)

(1, 0, 1)

(1, 1, 1)

## Algorithm

Resolving (Self-)Intersections Extracting Cells Propagating Winding Numbers

<!-- image -->

(0, 0, 0)

(1, 0, 0)

(0, 1, 0)

(0, 0, 1)

(1, 1, 0)

(0, 1, 1)

(1, 0, 1)

(1, 1, 1)

## Resolving (Self-)Intersections

<!-- image -->

<!-- image -->

## Resolving (Self-)Intersections

<!-- image -->

Pitfall: CDT is not unique, overlapping triangle needs to be consistently resolved!

<!-- image -->

## Algorithm

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Extracting Cells

<!-- image -->

<!-- image -->

## Algorithm

Resolving (Self-)Intersections Extracting Cells Propagating Winding Numbers

<!-- image -->

(0, 0, 0)

(1, 0, 0)

(0, 1, 0)

(0, 0, 1)

(1, 1, 0)

(0, 1, 1)

(1, 0, 1)

(1, 1, 1)

## Propagating Winding Numbers

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

(0, 0, 0)

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

<!-- image -->

<!-- image -->

## Propagating Winding Numbers

(0, 0, 0)

(1, 0, 0)

(0, 1, 0)

(0, 0, 1)

(1, 1, 0)

(0, 1, 1)

(1, 0, 1)

(1, 1, 1)

<!-- image -->

<!-- image -->

## Results: Self Union

## Check 1: produced an output

| our method   | 100%   |
|--------------|--------|
| CGAL         |        |
| QuickCSG     | 99.9%  |
| Cork         | 94.1%  |
| Attene       | 98.3%  |

## Results: Self Union

## Check 2: output is closed

| our method   |       | 100%   |
|--------------|-------|--------|
| CGAL         | 73.4% |        |
| Carve        |       | 99.4%  |
| QuickCSG     | 66.0% | 93.6%  |
| Cork         |       | 98.2%  |
| Attene       |       |        |

## Results: Self Union

## Check 3: output is PWN

| our method   |       | 100%   |
|--------------|-------|--------|
| CGAL         | 73.4% | 95.2%  |
| QuickCSG     | 66.0% |        |
| Cork         |       | 93.6%  |
| Attene       |       | 98.2%  |

## Results: Self Union

## Check 4: output is self-intersection free

| our method   | 100%   |
|--------------|--------|
| CGAL         | 51%    |
| Carve        | 60%    |
| QuickCSG     | 62%    |
| Cork         | 61%    |
| Attene       | 70%    |

## Results: Self Union

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

A\B B\A AUB ANB

<!-- image -->

1404 results for all possible pairs.

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al. 2015] Dataset

<!-- image -->

## Results: [Barki et al.] Dataset

<!-- image -->

## Result: Reverse Boolean

D

0

\

<!-- image -->

?

## Result: Reverse Boolean

<!-- image -->

## Results: Bunny Carve

<!-- image -->

## Results: Bunny Carve

<!-- image -->

## Results: Swept Volume

<!-- image -->

## Results: Swept Volume

<!-- image -->

## Results: CNC Milling

<!-- image -->

## Results: CNC Milling

<!-- image -->

## Conclusion

<!-- image -->

## Future Works

- Boost performance with adaptive techniques.
- Repairing non-PWN meshes

<!-- image -->

<!-- image -->

<!-- image -->

## Resources

## Source Code

<!-- image -->

[https://github.com/libigl/libigl](https://github.com/libigl/libigl)

PyMesh

[https://github.com/qnzhou/PyMesh](https://github.com/qnzhou/PyMesh)

## Dataset: Thingi10K

<!-- image -->

[https://ten-thousand-models.appspot.com/](https://ten-thousand-models.appspot.com/)