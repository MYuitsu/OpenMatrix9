# Adjust Closed Surface Seam ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 217-217
- Section title: Adjust Closed Surface Seam ￼
- Section id: m8book1-237-adjust-closed-surface-seam
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p217_img01.png, m8book1_p217_img02.png

---
<!-- section m8book1-237-adjust-closed-surface-seam | Adjust Closed Surface Seam ￼ | m8book1 pages 217-217 -->

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
