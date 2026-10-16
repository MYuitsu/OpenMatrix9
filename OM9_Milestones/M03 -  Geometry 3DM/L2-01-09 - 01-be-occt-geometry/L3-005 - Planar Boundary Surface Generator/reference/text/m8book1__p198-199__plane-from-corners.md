# Plane from Corners ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 198-199
- Section title: Plane from Corners ￼
- Section id: m8book1-215-plane-from-corners
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p198_img01.png, m8book1_p198_img02.png, m8book1_p199_img01.png, m8book1_p199_img02.png, m8book1_p199_img03.png

---
<!-- section m8book1-215-plane-from-corners | Plane from Corners ￼ | m8book1 pages 198-199 -->

<!-- page 198 -->

## Edge Curves
‘EdgeSrf’
Create a surface using 2, 3, or 4 unjoined, (non-
planar or planar) curves or edges. This command
works similarly to Planar Curves but the curves do
not need to be planar.
Where is this Command:
Access this command from Surface>Edge
Curves
Steps:
• Select Edge Curves from the Surface menu.
• Select 2, 3 or 4 open curves.
Vertical, 3Point, Center, and Square planes created
with Plane from Corners.
### Rectangle Options:
3Point: Draws the rectangle using two adjacent
corner locations and a location on the opposite
side.
Select four non-
planar edges to
create a surface
using ‘Edge Curves’.
3Point steps:
• Pick the start of an edge.
• Pick the end of the edge.
• Pick the width. Type a value or press Enter to use
the length value.
Vertical: Draws the rectangle perpendicular to
the Construction plane.
Note:
• Curves must be open and not joined.
• Joined curves or polylines will not create a surface.
• Curves and edges may be non-planar.
Vertical steps
## Plane from Corners
• Pick the start of the edge.
• Pick the end of the edge.
• Pick the height. Type a value or press Enter to use
the width value.
‘Plane’
Creates a rectangular, planar surface.
Center: Draws the rectangle around a center
point.
Center steps
Where is this Command:
Access this command from Surface>Plane
From Corners
Right-Click Command: Plane From Center
• Pick the center of the plane.
• Pick the other corner or type a length.
Steps
### Plane Options:
• Select Plane from Corners from the Surface menu.
• Pick the first corner of plane.
• Pick the other corner or type a length.
Deformable: Sets the degree in the U and V
directions and the point count in the U and V
directions.
Note: Hold the Shift key to draw a square.

<!-- page 199 -->

the trimming objects are deleted. This also deletes
any geometry attached to the edges of the
boundary.
A Deformable
plane is created
with 6- U and V
degrees with 10-
U and V points.
AllSimilar: Removes all trimming curves on
the edge of a trimmed surface, or if a hole edge
is selected, all holes on the same face will be
deleted.
UDegree/VDegree: Sets the Degree of the
surface in the U and V directions.
## Surface Extrude All
Use planar or non-planar, curves or surfaces to
extrude into capped or uncapped polysurfaces, to
a point, straight, tapered or along a curve.
Degree: Specifies the degree of the curve (or
surface). When drawing a high-degree curve, the
output curve will not be the degree you request
unless there is at least one more control point
than the degree.
UPointCount/VPointCount: The number of
control points in the U and V directions.
Where is this Command:
Access this command from Surface>Surface
Extrude All
Steps:
## Untrim
‘Untrim’
Removes a trimming boundary from a surface,
returning it to its untrimmed state.
• Select Surface Extrude All from the Surface menu.
• Select or get objects to Extrude.
• Select the Extrusion Type.
• Press Enter when done.
• Pick an Extrude Option in the Command Line.
Note:
Where is this Command:
Access this command from Surface>Untrim
Steps:
• Non-planar curves will not cap into a solid
polysurface.
• Planar curves will cap into a solid polysurface.
• Surfaces and curves can be extruded separately
but not together.
• Non-planar curves do not always give the best
results when choosing the tapered option.
• Select Untrim from the Surface menu.
• Select the edge to Untrim.
• Select another edge to Untrim or press Enter to
end the command.
### Straight Extrude Options (Default):
Curves or surfaces
are extruded straight,
perpendicular to the input
objects in Surface
Extrude All - Straight
(Solid=Yes).
Untrim is used to return the trimmed surfaces (left)
to their original state (right).
Direction: Two points establish the direction
angle.
### Command Line Options:
Steps
KeepTrimObjects Yes/No: Yes; the original
trimming objects are separated and retained. No;
• Pick a base point.
• Pick a second point that establishes the direction
angle.
