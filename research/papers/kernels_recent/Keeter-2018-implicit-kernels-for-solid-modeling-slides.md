## Implicit kernels for solid modeling

Matt Keeter 2018-01-19

matt.j.keeter@gmail.com mattkeeter.com

## A Brief* History of CAD *and probably inaccurate

## Drafting tables

<!-- image -->

<!-- image -->

## Computer-aided drafting

<!-- image -->

AutoCAD 2.18      (1985)

## Into the 3rd Dimension

<!-- image -->

<!-- image -->

'While all drafting is 2D, and almost all users will spend all their time with AutoCAD working in 2D mode, 3D is important more from a marketing perception standpoint than a technical one.'

John Walker, 1983

## The path of least resistance

Boundary representations Natural extension from 2D Easy to render Fragile representation

Geometric operations are hard!

<!-- image -->

## Solid Modeling

<!-- image -->

'I have seen solids modeling, and it is the future. For years the skeptics have criticized solids as impractical, compute intensive, and inflexible. 'You can't cut chips with solids,' they'd say, or 'sure they're fun, but what can you do with the model when you're done'?'

Eric Lyons, 1986

How would you make...

## Menger Sponges

<!-- image -->

<!-- image -->

## Family of gear pairs

<!-- image -->

<!-- image -->

<!-- image -->

<!-- image -->

## Möbius strip

<!-- image -->

## Mandelbrot Vase

<!-- image -->

<!-- image -->

Fundamentals of functional representations

## Functional representations

f : R 3 → R

f(x, y, z) → Distance

Distance &lt; 0:  inside

Distance = 0:  boundary

Distance &gt; 0:  outside

2

f(x, y, z) = x

+ y

2

+ z

2

- r

2

<!-- image -->

f(x, y, z) = x 2 + y 2 + z 2 - r 2

## Upsides &amp; downsides

- ➕ CSG (union / intersection / difference) becomes trivial
- ➕ Unusual transforms become possible
- ➖ Rendering is harder (but computers are fast, and it parallelizes well)
- ➖ Features are implicit
- ➖ Hard to interface with existing b-rep ecosystem (meshes, NURBS, etc.)

<!-- image -->

Previous work

## Hyperfun (SIGGRAPH 1999)

<!-- image -->

<!-- image -->

<!-- image -->

## Kokopelli (2012)

<!-- image -->

<!-- image -->

## ImplicitCad (2012)

<!-- image -->

<!-- image -->

'ImplicitCAD is a project dedicated to using the power of math and computer science to get stupid design problems out of the way of the 3D printing revolution.'

<!-- image -->

## Symvol (2012)

<!-- image -->

<!-- image -->

## Antimony (2013)

<!-- image -->

## Ao (2016)

## github.com/mkeeter/ao-guile-repl

<!-- image -->

## libfive + Studio (2018)

<!-- image -->

<!-- image -->

Implicit kernel design

## Representation vs. Evaluation

## Shape representation

- Lightweight manipulation of math trees
- De-duplication of clauses
- Arithmetic identities + constant folding
- Balancing of commutative operations

## Deduplication

<!-- image -->

## Arithmetic identities

<!-- image -->

## Tree balancing

<!-- image -->

## Representing a clause

```
/*  This is where tree data is actually stored  */ struct Tree_ { /* *  Destructor erases this Tree * from the global Cache */ ~Tree_(); const Opcode::Opcode op; const uint8_t flags; const unsigned rank; /*  Only populated for constants  */ const float value; /*  Only populated for operations  */ const std::shared_ptr<Tree_> lhs; const std::shared_ptr<Tree_> rhs; };
```

## Representing a clause

```
/*  This is where tree data is actually stored  */ struct Tree_ { /* *  Destructor erases this Tree * from the global Cache */ ~Tree_(); const Opcode::Opcode op; const uint8_t flags; const unsigned rank; /*  Only populated for constants  */ const float value; /*  Only populated for operations  */ const std::shared_ptr<Tree_> lhs; const std::shared_ptr<Tree_> rhs; };
```

```
/*  Lightweight, passable-by-value handle */
```

```
class Tree { /*  Overload arithmetic here! */ /*  Here's the actual Tree data */ std::shared_ptr<Tree_> ptr; };
```

## Representing a clause

```
/*  Lightweight, passable-by-value handle */ class Tree { /*  Overload arithmetic here! */ /*  Here's the actual Tree data */ std::shared_ptr<Tree_> ptr; }; class Cache { /*  Functions to handle identities here! */ typedef std::tuple<Opcode::Opcode, const Tree_*, /* lhs */ const Tree_*  /* rhs */ > Key; std::map<Key, std::weak_ptr<Tree::Tree_>> ops; std::map<float, std::weak_ptr<Tree::Tree_>> constants; }; /*  This is where tree data is actually stored  */ struct Tree_ { /* *  Destructor erases this Tree * from the global Cache */ ~Tree_(); const Opcode::Opcode op; const uint8_t flags; const unsigned rank; /*  Only populated for constants  */ const float value; /*  Only populated for operations  */ const std::shared_ptr<Tree_> lhs; const std::shared_ptr<Tree_> rhs; };
```

## Representation vs. Evaluation

## General form of algorithms

- If at minimum size, perform operations on voxel
- Otherwise, evaluate interval
- If filled or empty, return early
- Subdivide and recurse
- After recursion is done, collapse branches if possible

<!-- image -->

<!-- image -->

<!-- image -->

| index   | 0   | 1   |   2 |   3 | 4   | 5   | 6   | 7   | 8   | 9   | 10   |
|---------|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|------|
| value   | X   | Y   |   1 | 0.5 |     |     |     |     |     |     | out  |

<!-- image -->

Flat tapes are faster than chasing pointers!

<!-- image -->

<!-- image -->

Flat tapes are faster than chasing pointers!

<!-- image -->

<!-- image -->

Flat tapes are faster than chasing pointers!

<!-- image -->

<!-- image -->

Flat tapes are faster than chasing pointers!

<!-- image -->

<!-- image -->

Flat tapes are faster than chasing pointers!

## Value types

Floating-point values f(1, 0, 0) = 0

## Value types

| Floating-point values                     | f(1, 0, 0) = 0                 |
|-------------------------------------------|--------------------------------|
| Arrays of floats (optimization for speed) | f({...}, {...}, {...}) = {...} |

## Value types

| Floating-point values                     | f(1, 0, 0) = 0                     |
|-------------------------------------------|------------------------------------|
| Arrays of floats (optimization for speed) | f({...}, {...}, {...}) = {...}     |
| Interval ranges                           | f([0, 1], [1, 2], [0, 0]) = [0, 4] |

## Value types

| Floating-point values                     | f(1, 0, 0) = 0                                                 |
|-------------------------------------------|----------------------------------------------------------------|
| Arrays of floats (optimization for speed) | f({...}, {...}, {...}) = {...}                                 |
| Interval ranges                           | f([0, 1], [1, 2], [0, 0]) = [0, 4]                             |
| Derivatives (automatic differentiation)   | (df/dx)(1, 0, 0) = 2 (df/dy)(1, 0, 0) = 0 (df/dz)(1, 0, 0) = 0 |

## Tree pruning with intervals

<!-- image -->

## Tree pruning with intervals

<!-- image -->

## Tree pruning with intervals

<!-- image -->

## Tree pruning with intervals

<!-- image -->

<!-- image -->

## Outputs

## Voxel output: heightmaps

<!-- image -->

Brightness = z-height Useful for 2.5D machining

## Voxel output: shaded

<!-- image -->

Normals are based on derivatives (df/dx, df/dy, df/dz)

## B-rep output: meshes

<!-- image -->

Normals are used to position vertices on sharp edges and corners.

## Real things!

<!-- image -->

Creepy Crawly Cutter Sam Calisch

Curtain rail brackets Paul Meyer

<!-- image -->

Rotary encoder Matt Keeter

<!-- image -->

## Unsolved problems

- GPU acceleration
- Feature-based design
- Interacting with meshes
- Constraint systems