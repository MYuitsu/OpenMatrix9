---
source_pdf: h:\OpenMatrix9\manuals\rhino5.pdf
source_key: rhino5
page_range: 31-31
section_title: Light-weight extrusion objects
section_id: rhino5-019-light-weight-extrusion-objects
---
# Light-weight extrusion objects

Source: rhino5.pdf, pages 31-31

<!-- section rhino5-019-light-weight-extrusion-objects | Light-weight extrusion objects | rhino5 pages 31-31 -->

<!-- page 31 -->

Rhino Objects
### Light-weight extrusion objects
Light-weight extrusion objects use only a profile curve and a length as input instead of the network of
isocurves normally needed for NURBS objects. The Box, Cylinder, Tube, and ExtrudeCrv commands create
extrusion objects. Extrusion objects can be closed with a planar cap or open. These objects will be converted to
polysurfaces by some commands if necessary to add additional information for editing.
### Polygon mesh objects
Because there are many modelers that use polygon meshes to represent geometry for rendering, animation,
stereolithography, visualization, and finite element analysis, the Mesh command translates NURBS geometry
into polygonal meshes for export. In addition, the Mesh creation commands MeshSphere, MeshBox,
MeshCylinder, etc., draw mesh objects.
### Note:
There is no easy way to convert a mesh model into a NURBS model. The information
that defines the objects is completely different.
However, Rhino has a few commands for drawing curves on meshes and extracting
vertex points and other information from mesh objects to assist in using mesh
information to create NURBS models.
