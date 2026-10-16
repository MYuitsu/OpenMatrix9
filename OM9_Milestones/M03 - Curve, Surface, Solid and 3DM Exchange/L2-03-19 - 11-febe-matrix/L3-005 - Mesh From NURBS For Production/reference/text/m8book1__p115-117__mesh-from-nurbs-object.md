# Mesh from NURBS Object ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 115-117
- Section title: Mesh from NURBS Object ￼
- Section id: m8book1-119-mesh-from-nurbs-object
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p116_img01.png, m8book1_p116_img02.png

---
<!-- section m8book1-119-mesh-from-nurbs-object | Mesh from NURBS Object ￼ | m8book1 pages 115-117 -->

<!-- page 115 -->

Steps:
## Fill Hole
• Start with a mesh object with a hole or open
edges.
• Select Align Mesh Vertices from the Utilities menu.
• Press Enter.
‘FillMeshHole’
When working with meshes imported into Matrix
from another program, the mesh may Import with
holes. The Fill Hole command will help repair an
imported mesh.
### Command Line Options:
Select Vertices: Select vertices to align.
Where is this Command:
Access this command from Utilities>Fill Hole
Steps:
Select Naked Edges: Select naked edges to
align all vertices on the edge.
Distance To Adjust: Set the tolerance distance.
## Mesh from NURBS Object
• Start with a mesh object with a hole or open
edges.
• Run the Fill Hole command from the Utilities menu.
• Select a mesh hole boundary.
## Fill Holes
‘Mesh”
The Mesh command converts a NURBS object(s)
into a mesh object. Mesh objects are usually used
as the final step in the creation process. An STL
(stereolithography) file is an example of a mesh
object. Mesh files are a common file format that
can be opened in both Matrix and in the programs
used by mills or rapid prototyping machines.
‘FillMeshHoles’
When working with meshes imported into Matrix
from another program, the mesh may import with
holes. This tool repairs imported meshes by filling
all holes in a polygon mesh object with triangular
faces. This helps repair mesh files for rapid
prototype printing.
Where is this Command:
Access this command from Utilities>Mesh from
NURBS Object
Where is this Command:
Access this command from Utilities>Fill Holes
Steps:
• Start with a mesh object with a hole or open
edges.
• Select Fill Holes from the Utilities menu.
## Align Mesh Vertices
Note: Do NOT convert a NURBS model to a mesh
model if additional changes to the object are
required. This is because there are many necessary
functions that cannot be used with a mesh object,
such as Trim, Split, Join, Blend, Booleans,
and many more. Matrix creates triangles and
quadrilaterals meshes for export into various file
formats. When surfaces are joined together in Matrix,
the meshes along the joined edge have coincident
vertices. If a mesh is generated from a solid, there
will be no holes in the mesh making it a seamless,
watertight closed mesh. This is valuable for export to
STL rapid prototyping files.
Steps:
‘AlignMeshVertices’
The Align Mesh Vertices command re-aligns
the edges of the polygons used to make up the
mesh, which makes it easier for the program to
fill any holes of the new, curved edge that will be
replacing the existing shape.
• Select Mesh from the Utilities menu.
• Select the surfaces or polysurfaces you wish to
mesh.
• Preview the mesh and set the mesh options.
Where is this Command:
Access this command from Utilities>Align Mesh
Vertices

<!-- page 116 -->

### Polygon Mesh Options:
the objects. It will tend to make meshes denser in
areas of high curvature and less dense in flatter
areas.
Fewer Polygons-More Polygons: The Slider
roughly controls the density and number of
mesh polygons. Set the slider and click preview. If
acceptable, click okay.
Preview: The mesh is drawn as a preview in the
viewports and the dialog box stays on-screen for
more adjustments.
Maximum Aspect Ratio: Surfaces are initially
tessellated with a regular quadrangle mesh and
then that mesh is refined. The initial quad mesh
is constructed so that on average, the maximum
aspect ratio of the quads is less than or equal to
Maximum aspect ratio. Smaller values result in
slower meshing and a higher polygon count with
more equilateral and nicely shaped polygons.
This is approximately the maximum aspect ratio
of the quads in the Minimum initial grid quads.
Setting Maximum aspect ratio to zero turns off
the option. Zero means no limit. The default value
for this option is zero and the suggested range,
when not zero, is from 1 to 100. This setting is scale
independent. Application: When shading long,
skinny objects, use 0. This allows infinite ratios.
Control the smoothness of the mesh with other
parameters.
### Menu Options:
Minimum Edge Length: If any edge is shorter
than the Minimum edge length, no further
division of the mesh faces occur. This is also,
approximately, the minimum edge length of the
quads in the minimum initial grid quads. The
default value for this option is 0.0001. 0 units
and the usable range depends on the size of the
model. Bigger values result in faster meshing,
less accurate meshes and a lower polygon
count. Setting this value to zero turns off the
minimum edge length option. This option is scale
dependent. The value is always in the current unit
system.
Density: Uses a formula to control how close the
polygon edges are to the original surface. Values
are between 0 and 1. Larger values result in a
mesh with a higher polygon count.
Maximum Edge Length: Polygons are further
divided until all polygon edges are shorter
than this value. This is also, approximately, the
maximum edge length of the quads in the
Minimum initial grid quads. Smaller values result in
slower meshing and a higher polygon count with
more equally sized polygons. Setting this value to
zero turns off the option. The default value is zero
and the usable range depends on the size of the
model. This option is scale dependent.
Maximum Angle: Sets the maximum allowable
angle between input surface normals at
neighboring mesh vertices. If the angle between
surface normals is greater than this setting, the
mesh is further refined (more vertices are inserted)
and the mesh is made denser. Two vertices are
neighbors if they are at the opposite ends of a
single facet edge. The Maximum angle setting
will influence the meshing of objects of the same
shape in the same way regardless of the size of

<!-- page 117 -->

suggested range is from 5 to 90 degrees. Setting
Maximum angle to 0 turns off the option. This
option is scale independent.
Toggling this option off results in faster meshing,
less accurate meshes, and a lower polygon count.
Clearing this check box also means untrimmed
individual surfaces and surface areas away from
trim edges and joined edges are meshed with
evenly sized quadrangles.
Maximum Distance Edge to Surface: Use
for making sure the polygons are approximately
the same size. Maximum distance, edge to
surface: Polygons divide until the distance from
a polygon edge midpoint to the NURBS surface
is smaller than this value. This distance is also the
approximate maximum distance from polygon
edge midpoints to the NURBS surface in the
Minimum initial grid quads. Smaller values result
in slower meshing, more accurate meshes, and a
higher polygon count. Setting this value to zero
turns off the option. The default value is zero
and the usable range depends on the size of the
model.
Jagged Seams: All surfaces mesh
independently and Rhino does not stitch
the edges of joined surface edges together.
Meshes for each surface in a polysurface do not
necessarily meet to form a watertight mesh. If
Jagged seams is not checked, watertight meshes
are created. This causes faster meshing, a lower
polygon count and cracks between joined
surfaces in the rendered image.
Application: Matrix does not support watertight
quadrangle meshes unless you are meshing
a single untrimmed surface. In this case, clear
Refine mesh and use Jagged seams to generate
quadrangle meshes.
Application: Use as a general polygon mesh
tolerance setting.
Minimum Initial Grid Quads:
Initial mesh grid is a quad mesh Rhino creates
on each NURBS surface in the first stage of
meshing. When the initial mesh grid is made,
trim curves are ignored. After the initial grid is
made, Rhino meshes all trim edges, connects the
initial grid to the trim edges and then refines the
mesh if the Refine mesh option is selected. The
number of quadrangles per surface in the initial
mesh grid. In practice, Matrix will use at least this
many polygons on each surface. Bigger values
result in slower meshing, more accurate meshes
and a higher polygon count with more evenly
distributed polygons. Setting this value to zero
turns off the option. The default value is 16. The
suggested range is from 0 to 10000. This option is
scale independent.
Simple Planes: All planar surfaces are meshed
by meshing the surface edges and then filling the
area bounded by the edges with triangles. This
setting causes slower meshing and a minimum
polygon count on planar surface, especially for
complex trimmed surfaces. If Simple planes is
selected, the settings, except Jagged seams, are
ignored for planar surfaces and the planar surface
is meshed with as few polygons as possible.
Application: Use to make sure that surfaces with
very subtle details are meshed with a high enough
polygon count.
Check Box Options
Pack Textures: This packs the mesh textures
of the selected polysurfaces. When polysurfaces
are meshed, the packed texture coordinates are
created. A packed texture is a partition of the
unit square into disjoint sub-rectangles so that
one bitmap can be used to apply independent
textures to each face of the polysurface.
Refine Mesh: After its initial meshing, Rhino
uses a recursive process to refine the mesh until
it meets the criteria defined by Maximum angle,
Minimum edge length, Maximum edge length,
and Maximum distance, edge to surface options.
The mesh is refined until the angle between
surface normals along a polygon edge is smaller
than this value. The default is 20 degrees and the
