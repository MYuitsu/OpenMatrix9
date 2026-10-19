# Soft Edit ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 216-217
- Section title: Soft Edit ￼
- Section id: m8book1-235-soft-edit
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p216_img01.png, m8book1_p216_img02.png, m8book1_p216_img03.png, m8book1_p216_img04.png, m8book1_p217_img01.png, m8book1_p217_img02.png

---
<!-- section m8book1-235-soft-edit | Soft Edit ￼ | m8book1 pages 216-217 -->

<!-- page 216 -->

Steps:
• Select Soft Edit from the Surface menu.
• Select a surface.
• Pick a base point on the surface.
• Pick a point to move to. Drag your cursor and click
to set. Specify a Command Line Option.
• Press Enter when done.
Unroll Surface with Labels = Yes shows each surface
unrolled, and labeled showing where it originated on
the polysurface ring.
The selection
point snaps
to the nearest
surface
control point.
Corresponding
control points of
the surface are
moved.
### Command Line Options:
Explode Yes/No: Yes; creates surfaces that are
not joined. No; the resulting surfaces are joined
along the same edges that were joined in the
original polysurface.
Note: To specify where a polysurface splits on an
unroll with Explode set to No, use the Unjoin Edge
command. Select the edges on the polysurface
you want open at unfolding to be unjoined prior to
unrolling.
Pick a point to move to. Drag your cursor and click to
set the ‘move to’ location.
Labels: Matching numbered dots are placed
on the edges of the original polysurface and the
flattened surfaces.
KeepProperties: Copies the object properties
from the original surface to the new surface.
RelativeTolerance: Affects the maximum
deviation of the curves from the original surface to
the unrolled surface.
Surface after running Soft Edit surface.
## Soft Edit
### Command Line Options:
‘SoftEditSrf’
Moves the surface area surrounding the selected
point, smoothly, relative to the distance from
selected point.
U Distance/V Distance: Specify a value for
distance in the U or V direction influenced by the
move. (‘1’ is the default.)
Where is this Command:
Access this command from Surface>Soft Edit
Copy=Yes/No: Creates a copy of the original
surface after Soft Edit is complete.

<!-- page 217 -->

FixEdges: When set to ‘Yes’ it keeps the edge
locations fixed in the original position. This
is useful keeping the corners of the surface
stationery when the UV distance set is greater
than the size of the surface in either given
direction.
Two cutting planes
were placed on
a cube with the
Cutting Plane
command.
### DirConstraint Options:
Constrains the direction of the move.
None: Allows free movement.
### Command Line Options:
SrfNormal: Constrains movement normal to the
surface.
3Point: Draws the rectangle using two adjacent
corner locations and a location on the opposite
side.
UTangnet: Constrains the surface tangent to the
surface U Direction
3Point steps:
VTangent: Constrains the surface tangent to the
surface V Direction
• Pick the start of an edge.
• Pick the end of the edge.
• Pick or type the width.
## Cutting Plane
## Adjust Closed Surface Seam
‘CutPlane’
Creates planar surfaces through objects at
designated locations.
‘SrfSeam’
Use this command to change the location of the
seam of any closed, unjoined surface for better
results in commands like Smart Flow.
Where is this Command:
Access this command from Surface>Cutting
Plane
Where is this Command:
Access this command from Surface>Adjust
Closed Surface Seam
Steps:
Steps:
• Select Adjust Closed Surface Seam from the
Surface menu.
• Select an unjoined, closed surface for seam
adjustment. (The surface that loops back to back
and touches itself is closed.)
• The seam of the surface is indicated by a curve.
• Adjust the surface seam. Drag your cursor and
click to set the new location for the seam point.
• Select Cutting Plane from the Surface menu.
• Select the objects that the cutting planes will pass
through.
• Press Enter when done.
• Pick the start of a line that defines the cutting
plane.
• Pick the end of a line that defines the cutting
plane.
• The prompts repeat to make several cut planes.
• Press Enter to stop making cut planes.
Note: This command creates planes perpendicular to
the current Construction planes. These planes pass
through the selected objects and are large enough to
intersect all of the objects.
Drag your cursor and
click to place the new
location for the closed
surface seam.
