# Fill Hole ￼

- Source PDF: Matrix 8 Instruction Manual Book 1 (matrix8_book_1.pdf)
- Page range: 115-115
- Section title: Fill Hole ￼
- Section id: m8book1-116-fill-hole
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book1-116-fill-hole | Fill Hole ￼ | m8book1 pages 115-115 -->

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
