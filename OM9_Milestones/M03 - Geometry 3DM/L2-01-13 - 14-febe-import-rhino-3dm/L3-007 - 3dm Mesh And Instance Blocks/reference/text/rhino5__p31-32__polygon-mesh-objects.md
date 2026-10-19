# Polygon mesh objects

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 31-32
- Section title: Polygon mesh objects
- Section id: rhino5-020-polygon-mesh-objects
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p31_img01.png, rhino5_p31_img02.png, rhino5_p31_img03.png

---
<!-- section rhino5-020-polygon-mesh-objects | Polygon mesh objects | rhino5 pages 31-32 -->

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

<!-- page 32 -->
