<!-- image -->

<!-- image -->

<!-- image -->

The  geometric  kernel  has  been  developed  by  an experienced team of Russian mathematicians since 1995. C3D Labs as a company was founded in 2012.

Its role was to convert the 3D kernel into a soware development  kit  (SDK);  to  turn  it  into  a  standalone product; and to market it internationally. The result is the "C3D Toolkit", built on renowned Russian mathematics  and  with  nearly  thirty  years'  experience  in working with advanced CAD components.

<!-- image -->

<!-- image -->

<!-- image -->

<!-- image -->

In  addition  to  B-reps,  polygonal  representation  is also available. Polygonal models consist of approximated faces and polygons. They are convenient for analysis and visualizations.

## C3D Modeler Functionality

## Solid modeling operations:

- Boolean;
- extrude, revolve, sweep;
- thin-walled bodies;
- fillets and chamfers;
- face tapering;
- ribs.

## Wireframe modeling operations:

- curves;
- intersection of curves;
- extend;
- trim;
- unwrapping;
- equidistant curves;
- outline curves.

## Suace modeling operations:

- extrude; revolute, sweep;
- lo, suace based on a curve grid;
- equidistant suace;
- trim/extend;
- fillets;
- patch, stitch.

<!-- image -->

## Direct modeling operations:

- remove faces;
- change the fillet radius;
- replace face;
- move/rotate face;
- resize face;
- merge smoothly jointed faces;
- resize holes.

## Sheet modeling operations:

- bend/unbend;
- convert solid to sheet metal;
- stamp;
- rolled sheet metal;
- bend relief;
- louvers;
- reinforcement rib.

## C3D Modeler suppos the following geometric calculations:

- suace areas, volumes, and properties of mass inertia;
- building planar projections;
- generating suace triangulations.

C3D Modeler supports much more operations. For the full list please visit the Developers section at c3dlabs.ru

<!-- image -->

<!-- image -->

## Functionality:

- by dimension type: linear, radial, diametral, angular, curve length;
- by dimension calculation type: driving, interval and variable dimensions.

<!-- image -->

## Rapid collision/gap detection for solid models, support for both exact B-rep geometry and approximate polygonal meshes

## Integrity of Assemblies Guaranteed

All  assemblies  should  be  free  of  collisions  between their components.

With  our  Collision  Detector,  your  CAD  and  BIM applications check assemblies for collisions at early stages, prevent assembly or kinematics errors. Early detection of collisions reduces manufacturing costs.

The  distance  measurement  function  checks  userspecified gaps between parts and sub-assemblies.

## Suppo for B-Rep and Polygonal Geometry Representations

Collision Detector works accurately with B-rep geometry. If necessary, polygonal representation can be used to speed up the analysis.

<!-- image -->

Collision  Detector  can  be  configured  to  limit  the scope of detection.

For example, you may need to detect all faces where the  bodies  intersect.  or you  may  just want to  know whether  bodies  or  subassemblies  collide  without going  into  the  details.  You  can  also  exclude  some pairs from the detection and select from a range of collision detection strategoes. The detection results can be tangents or in-volume intersections.

<!-- image -->

## C3D Vision capabilities:

- render any polygonal representation;
- user interaction with the model;
- camera control to display the scene;
- adding annotation objects;
- SW/HW accelerated visualization;
- integration with C3D Modeler objects;
- support for a ranger of video adapters.

The engine《

<!-- image -->

## Designed for engineering visualization in CAD/CAM/CAE/BIM applications

The  visualization  module  is  intended  for  CAD  and similar applications. C3D Vision provides a range of tools for quick soware development:

- object search, filtering by type and primitives;
- search for model objects under the cursor;
- handles and manipulators to interact with the model;
- measurements: linear, radial and angular dimensions.

## Integrated Environment

C3D Vision is closely integrated with the C3D Modeler geometric  kernel.  To  automatically  generate  scene graphs  based  on  mathematical  models,  it  is  suicient to call just one function. There is the option to calculate polygonal models for visualization objects (based on math representation of the geometry) in synchronous or multithreading mode.

## Peormance optimization

For high peormance with scenes and large models: hardware acceleration estimating the visibility of the scene  objects  by  specifying  object  sizes  in  screen pixels  estimating  the  visibility  of  the  o-screen objects adjustable LOD (levels of detail).

Theairbo

<!-- image -->

## Suppo for Exchange Formats

C3D  Converter  can  export  and  import  geometry from/to  most  commonly  used  exchange  formats available under a common license. The choice of an exchange  format  depends  on  the  specific  model representation or end user demands. The supported formats  are  carefully  selected  for  your  better experience.

<!-- image -->

| Format                      | Read   | Write   | Additional licence required   |
|-----------------------------|--------|---------|-------------------------------|
| C3D                         | ь      | ь       |                               |
| STEP (AP203, AP214 и AP242) | ь      | ь       |                               |
| IGES 5.3                    | ь      | ь       |                               |
| ACIS (SAT)                  | ь      | ь       |                               |
| ACIS (SAB)                  | ь      |         |                               |
| Parasolid (X_T, X_B)        | ь      | ь       |                               |
| JT                          | ь      | ь       |                               |
| VRML 2.0                    | ь      | ь       |                               |
| STL                         | ь      | ь       |                               |
| 3MF                         |        | ь       |                               |
| OBJ                         | ь      |         |                               |
| NX v2017-2021               | ь      |         | ь                             |
| SolidWorks v2015-2023       | ь      |         | ь                             |
| IFC                         | ь      |         | ь                             |

<!-- image -->

## C3D PolyShaper

The Stanford bunny Computer Graphics Laboratory Stanford University

<!-- image -->

The polygonal representation uses vertices, edges, and flat facets to build faceted 3D bodies. You can use C3D Converter to read/write such models, and C3D Vision to visualize them. C3D PolyShaper is a toolbox designed to edit, analyse and heal polygonal objects.

## Diagnoics and Healing

Polygonal  meshes  may  have  geometric  and  topological issues. The list of geometric model errors can be  extensive,  including  such  issues  as  gaps  and openings, duplicated vertices, degenerate triangles, and non-manifold geometry. and so on. Such errors can cause most algorithms to fail. Diagnostics and healing are essential stages in polygonal modeling.

## C3D PolyShaper oers:

- mesh diagnostics;
- fixing degenerated triangles;
- mesh stitching;
- fixing inconsistent normals;
- checking for non-manifold geometry.

## Simplification, Refinement, and Remeshing of the Triangulation Mesh

Reducing the number of triangles improves the performance.  C3D  PolyShaper  has  two  mesh  simplification  options:  to  a  given  number  of  triangles  or a given accuracy. One can obtain a set of simplified meshes with dierent levels of detail. The simplification  algorithm  guarantees  the  preservation  of  the topological properties of the polygonal object.

The  reverse  local  mesh  refinement  tool  is  used  for computer-aided  analysis  to  enhance  the  local simulation accuracy. It preserves a smooth transition from larger to smaller triangles.

Remeshing improves the mesh quality by making the triangles  as  equilateral  as  possible.  Better  mesh contributes to the reliability of many algorithms and simulation processes.

Fertility AIM@SHAPE-VISIONAIR Shape Repository

<!-- image -->

## Editing

C3D PolyShaper oers the following polygonal mesh editing tools:

- deleting triangles;
- Boolean operations;
- plane sections;
- mesh trimming by a cube of a specified size.

## Polygonal Model Analysis

C3D PolyShaper can validate meshes generated by 3D scanning, topology optimization, and other methods.

When  it  is  known  in  advance  that  objects  should match (for example, when comparing a CAD model to a 3D scanned model), we can apply automatic or semi-automatic  object  matching  for  subsequent comparison.

Another approach is reverse engineering when a polygonal  model  is  converted  into  a  solid.  For  this, analytic  suaces  are  approximated  with  the  least squares  method.  The  suace  type  is  either  autodetected  or  selected  by  the  user,  such  as  plane, cylinder, cone, sphere, or torus. The mesh sensitivity to  noise  can  be  adjusted  to  meet  various  criteria. The shape control tool aims to prevent the creation of  suaces  that  are  close  to  degenerate,  such  as nearly flat or nearly cylindrical cones. The inputs are a set of triangles and the desired accuracy.

Another  area  involves  the  automatic  conversion  of mostly  free-form  models  into  a  shell  composed  of NURBS patches.

As a rule, what we require is not just the mesh itself, but rather the mesh with its facets logically divided into  some  groups.  The  segmentation  builds  the topology of the polygonal model similar to conventional B-Rep models. The dierence is that the faces consist  of  connected  groups  of  facets,  while  the edges are composed of sequences of mesh edges. Each  topology  segment  may  have  an  attribute. The attribute  can  be  a  number,  vector,  suace,  or arbitrary data structure.

<!-- image -->

<!-- image -->

- reverse engineering parts from 3D scanner files;
- converting models from online component/typical catalogs for manufacturing and construction sectors;
- topology optimization post-processing in CAE applications.

## Through its API, C3D B-Shaper operates in two modes:

- automatic: generates acceptable results for highquality polygon meshes, which are usually triangulations of CAD models that mostly contain elementary suaces;
- interactive: when a polygon mesh is constructed from 3D object scans and contains arbitrary suaces.

《Guita ofthe

<!-- image -->

## B-Shaper capabilities:

- control the suace recognition accuracy;
- segmentation of polygonal meshes;
- segmentation editing: merge, split, etc.;
- reconstruction of a predefined suace on a segment;
- generate B-rep models.

C3D Modeler can model top-quality curves and suaces. It works with Class F curves that meet the following requirements:

<!-- image -->

<!-- image -->

<!-- image -->

## C3D Web Vision oers:

- 3D model visualization, view management;
- interactive geometry control.

## Integrated Environment

C3D Web Vision is closely integrated with our other C3D components:

- C3D Converter: to read third-party formats;
- C3D Modeler: to generate triangulation meshes, optimize and fix geometry.

## Interactive Model Features

C3D Web Vision can:

- select geometry under the cursor or in a fence;
- build cross sections using multiple planes;
- add annotations, measurements;
- adjust transparency, visibility, color, hide/show elements.

## Benefits:

C3D Web Vision consists of a microservice and a GUI client.  Such  as  architecture  gives  a  boost  to  web developers: they can accelerate the product development, control visualization capabilities, and load large assemblies without expert skills in 3D graphics.

With  C3D Web Vision, you  can  build  BIM, AEC,  PLM, CAM,  and  EDA  web  applications  with  stunning  3D graphics. The module creates interactive 3D scenes, uses  the  3D  model  tree,  oers  interactive  scene manipulation, and more.

Asphalt roller

<!-- image -->

## Model tools:

- screenshots;
- exploded views;
- animation (if available);
- view of multiple model configurations (if available);
- dynamic cross-sections;
- geometric measurements;
- mass, moments of inertia and other physical properties.

<!-- image -->

C3D Viewer is an easy-to-use application for 3D geometry visualization. It can read models in standard CAD formats and save them in the C3D format. It uses C3D Modeler, C3D Solver, C3D Vision and C3D Converter modules of the C3D Toolkit

## Geometry Impo and Expo:

- reading C3D, JT, STEP, X\_T, X\_B, SAT, IGES, STL, VRML, OBJ formats;
- loading multiple models into one scene;
- saving models to C3D and as JPEG, TIFF, BMP, PNG images.

## Navigation Functions:

- pan;
- rotate the model;
- rotate the camera;
- orbit rotate;
- zoom;
- standard views.

Marriage Palace

<!-- image -->

## Enterprise edition

C3D Viewer is available as an embedded visualization  tool  with  a  simple  API.  Developers  can  quickly add  C3D  Viewer  to  their  products.  A  license  key  is required.

## C3D Viewer Enterprise Edition oers additional functionality:

- saving 3D models to JT, STEP, X\_T, X\_B, SAT, IGES, STL, VRML formats;
- adding annotations to 3D models;
- proprietary ActiveX-based API for integration with corporate IT systems.

## Peormance options for better experience with complicated models:

- hiding edges in the shaded view mode;
- anti-aliasing;
- pixel/frustum culling.

## C3D Viewer Standard Edition is a and-alone desktop application:

- free and unlimited;
- installation packages for Windows 32/64-bit, Linux;
- inteace languages: English, Russian.

<!-- image -->