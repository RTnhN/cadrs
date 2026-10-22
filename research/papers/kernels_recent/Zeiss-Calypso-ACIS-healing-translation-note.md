## Non-ACIS CAD model load Process

## Step 1.  Translating a Model Into the ACIS SAT Format

The Translator takes a 3D model from a non -ACIS system and converts the model's topology and geometry into the ACIS SAT format.

## Step 2.  Model Healing

Once the Tra nslator converts the model's topology and geometry into SAT format. Healing provides the functionality to "heal" (mend) the model, improving its integrity. Why? Sometimes 3D models originating from a non -ACIS system can be inaccurate. ACIS Healing analyzes the geometry and topology inaccuracies and corrects them .

## · The Healing Process Geometry Simplification

Geometry simplification converts spline geometry into their corresponding analytic forms (e.g. arc, cone, plane, etc.), wherever possible.

## Stitching

The stitching function provides the means to stitch a set of faces together to form a single sheet or solid body. In the case of a surface model that lacks topology, such as from IGES, this phase adds topology to the model. This method is achieved in steps (or phases starting with a single small tolerance, an attempt is made to stitch all the faces. If any faces cannot be stitched, the tolerance is increased slightly. This step is repeated until a single tolerance is found that all the faces can be stitched at, or the maximum number of steps is reached.

## Geometry Repair or Building

Geometry repair (building) heals inaccuracies in the model. In this phase, a series of geometric operations are performed to improve the precision of face, edge, and vertex data.

<!-- image -->

## The IGES Translator in Calypso

The ACIS IGES Translator provides the functionality for exchanging only geometric data in IGES format, between proprietary CAD/CAM/CAE applications and ACIS -enabled applications such as Calypso. The translator primary fun ction is to covert an IGES file into an 'inaccurate' ACIS file. The IGES translator is licensed from Spatial Corporation.

## Translation nuances

During the loading of an IGES file in Calypso, translation problems will arise due to the difference between the  IGES precision and ACIS precision requirements. Typical problems that occur are:

1. Gaps appear between adjacent edges of a face.
2. The face edges may not lie accurately on the underlying surfaces.
3. Surfaces that were originally analytic (elements, surfaces, etc.) in the native CAD application are output as splines in the IGES file. While reading such IGES files into ACIS, they come in as splines.
4. IGES files often contain loose surfaces and edges without connectivity information .

The ACIS IGES Translator co ntains limited "face healing" functionality that repairs the trimmed faces to ensure that edges are connected and lie on the surface within ACIS precision. This addresses problems 1 and 2, identified above. However, the translator does not perform healing and stitching.

Solutions to problems 3 and 4 require much more sophisticated algorithms and 3D modeling capabilities, found in Healing.

## Version and Entity support

It is important to know the properties of versioning and entities of your IGES file. If y our IGES file does not comply with such properties there will be little or no translation to ACIS.

The Translator supports the following read/write and import/export format:

- The IGES Translator reads and writes files between the IGES format and the ACIS SAT format for versions 1.6 to 4.x of ACIS.
- The IGES Translator imports and exports Manifold Solid Boundary Representation Objects (MSBO) and trimmed surfaces between IGES version 5.2 and ACIS SAT.

## The Translator supports the following IGES entities:

- Points
- Curves
- Surfaces
- Trimmed surfaces
- Solids (entity type 186)

## Healing in Calypso

## What is Minimum Model Preparation?

This option invokes the Simplification call to ACIS, which is basically the conversion of splines to analytic  forms.  This  only  works for  planes,  cylinders,  circular  cones,  tori,  and  spheres.  Non -circular cones (pyramids) are not simplified. This function also allows the user to invert the vector of a 'one-sided' face. In Simplification the preset tolerance is .010 MM

## What is Autohealin g?

The  option  invokes  an  automated  analysis,  calculation,  and  fix  process  for  Simplification, Stitching, and Geometry building at preset values. For Simplification the preset tolerance is .010 MM. For stitching the preset tolerance minimum value is 1e -005 MM and the maximum value is 1 unit.  For Geometry Build the tolerance is preset at .10 MM.

## What is Step -by-Step Healing?

This  option  allows  for  user  interaction  of  the  Simplification,  Stitching,  and  Build  Geometry processes of Healing. The tolerance val ue for Simplification defaults at .010 MM and may be set to  any  value  by  the  user.    The  Stitching  tolerance  value  is  the  same  as  the  preset  value  in Autohealing. The Build Geometry tolerance defaults at .10 MM and may be changed by the user.

## What is Simplification Tolerance?

This is the tolerance at which spline surfaces get simplified to analytic surfaces. If the tolerance is tight,  only  spline  surfaces  that  are exact analytic  surfaces  get  simplified.  If  the  tolerance  is loosened, then approximate analytic fits to splines are obtained. In such cases, the gaps between surfaces may increase and healing in subsequent operations may be more difficult. Increasing the  tolerance  is  needed  whenever  analytic  surfaces  are  not  output  in  NURBS  (Non -Uniform Rational B-Splines) surfaces.

## What is Stitching Tolerance?

The minimum tolerance specifies the tolerance at which stitching commences. Therefore, this should be sufficiently low so as to stitch very small edges. The maximum tolerance specifies the point at which s titching stops and therefore needs to be sufficiently large so as to stitch faces with large interface  gaps in  the  body.  Stitching  is  performed  at  incremental  steps  in  between  the minimum and the maximum tolerance.

## What is Build Geometry Tolerance?

This  is the tolerance that 'fills in the gap' of the model after stitching. This should typically be around 3 times the maximum gap size in the model. The maximum gap size calculated during stitching is used as the geometry building tolerance in for automatic h ealing. However, the user may  need  to  increase  the  geometry  building  tolerance  if  the  healed  geometry  deviates substantially from the original geometry. (Too many holes in the model.)

## Effects to an IGES file from Calypso CAD modeler

## General

Import the file 'cad.igs' using the pull down CAD-CAD File-Load. Note that the cylindrical entity does not have a normal solid model silhouette appearance. Currently the model is an inaccurate ACIS model and Calypso cannot extract geometry at this point.

Invoke model simplification by using the pull down CAD -CAD File-Minimal Model Preparation. Click the render icon and extract a feature on the most forward surface of the cylinder. Notice that  there are four distinct entities of two planes and two cylinders. These elements are not currently stitched. This effect is also procured using the pull down CAD -CAD File-Step-by-Step Healing and invoking the Simplification option at the default value.

Invoke interactive processing by using the pull down CAD -CAD File- Step-byStep Healing and use the defaults values. Click the render icon and extract a feature on the most forward surface of the cylinder. Notice that there are only three distinct entities of two planes and one cylinder. These ele ments are simplified and stitched and geometry has been built.

<!-- image -->

<!-- image -->

<!-- image -->

## Multiple entity inaccuracies

Import the IGES file 'cad1.igs'. Sometimes  a  translated  IGES  file  may import with inaccurate information whenever  multiple  entities  are  used  to create a feature. In this case an exterior slot (which normally is created with four planes  and  two  cylinder  en tities)  was translated  as  having  an  extra  cylinder due to incomplete or inaccurate concatenation of the entities edges.

Invoke  interactive  processing  by  using the  pull  down  CAD -CAD  File-  Step-byStep Healing and use the defaults values. Click the render icon and extract a feature on the far right surface of the slot. Notice that there are now 6 distinct entities of four planes and two cylinders. These elements are simplified and stitched and geometry has been built.

## General File Size effects

| Translation       | File size increases due to translator memory allocation         |
|-------------------|-----------------------------------------------------------------|
| Simplification    | File size decreases when splines are converted to analytics     |
| Stitching         | File size deceases d ue to 'joining' of surfaces                |
| Geometry Building | File size increases due to filling in the gaps of intersections |
| Autohealing       | File size increases due to net surfaces being created           |

## A word on Tolerant Modeling

The tolerant modeling architecture in ACIS can accept less precise geometric data and create valid topology. Tolerant edge and vertex capabilities allow the software to attach tolerance values to them, so that even when edges do not intersect they're brought close enough to be useful. Tolerant mode ling increases the scope of the data that ACIS can import. It solves the problems associated with importing inaccurate data or "leaky" models while providing the framework for model healing and data translation.

<!-- image -->

<!-- image -->