# Blend Surface

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 189-192
- Section title: Blend Surface
- Section id: m8book1-207-blend-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p189_img01.png, m8book1_p189_img02.png, m8book1_p189_img03.png, m8book1_p189_img04.png, m8book1_p190_img01.png, m8book1_p190_img02.png, m8book1_p190_img03.png, m8book1_p190_img04.png, m8book1_p191_img01.png, m8book1_p191_img02.png, m8book1_p191_img03.png, m8book1_p191_img04.png, m8book1_p191_img05.png, m8book1_p191_img06.png, m8book1_p191_img07.png, m8book1_p191_img08.png, m8book1_p191_img09.png, m8book1_p192_img01.png, m8book1_p192_img02.png, m8book1_p192_img03.png

---
<!-- section m8book1-207-blend-surface | Blend Surface | m8book1 pages 189-192 -->

<!-- page 189 -->

Calculate: The calculation tests how far away the
new surface is at knot line intersections and half-
way between knot lines. Conducts tests at knot
line intersections and halfway between knot lines.
The display color indicates how far away the new
surface is from the original. Points are; green if the
surface is within absolute tolerance; yellow if it is
between tolerance and 10 times tolerance, and;
red if it is farther away than that. The lines are 10
times longer than the measured deviation in the
direction of the deviation.
Surface after Rebuild with point counts of U=6 and
V=6.
Note: Rebuild can also be used for curves, extrusions
(a type of surface), along with unjoined surfaces.
However, a combination of these cannot be selected
at the same time. Run the command individually for
each object type.
### Rebuild Surface Options:
Rebuild Surface preview.
Point Count: A surface has a U and V direction
(perpendicular). The input surface UV point
counts are shown in parenthesis next to the value
boxes. Enter a new U and V point count for the
rebuilt surface in the respective value boxes.
Degree: A surface degree is always set to 3.0.
So, in the case of rebuilding surfaces leave this
value set to 3.0. Curves have a different degree
value. Straight curves are 1.0 degree.
Preview: Click to display a preview of the
output. If you change the settings, click the
Preview button again to refresh the display. The
highlighted lines are the original surface. The
green lines are the Preview of the new  surface.
### Options:
## Blend Surface
‘BlendSrf’
Creates a surface that connects or blends
together two, open or closed, surface edges.
Delete Input:
Deletes the
original geometry
from the file.
Where is this Command:
Access this command from Surface>Blend
Surface
Current Layer: Creates the new surfaces on the
current layer. Clear this check box to place the
new surface on the layer of the original surface.
Steps:
Retrim: Trims the rebuilt surface with the original
trimming curves.
Maximum deviation: Reports the maximum
deviation from the
original surface.
• Select Blend Surface from the Surface menu.
• Select a segment for the first edge.
• Select the next adjacent segment for the first
edge. Continue selecting segments to complete
the first edge and press Enter when done.
• Select a segment for the second edge. Continue
selecting segments to complete the second edge
and press Enter when done.

<!-- page 190 -->

Gap Tolerance: If the gap between two
edges or curves is less than this value, the chain
selection will ignore the gap and will select the
next segment.
Angle Tolerance: When Continuity is set to
Tangency, if the angle between two edges or
curves is less than this value, the chain selection
will consider the criteria for continuity met and will
select the next segment.
• Select a seam point marker, and move it along the
closed curve. Continue to adjust the seam points
until they line with each other and the closed
curves all have the same direction. (Click on an
arrow to change the curve direction, if needed.)
• Press Enter when done.
• Select a control point to adjust. Hold ALT key and
move handle to adjust edge angle. Press SHIFT
for symmetry. Or, change the settings in the
‘Adjust Surface Blend’ dialog box.
• Click on ‘OK’ when done.
Undo: This will undo the last segment edge
selected.
Next: This allows selection of the next edge
curve segment.
All: This allows selection of all, both first and
second, edge segments.
### Adjust Seam Options:
These Command Line options
From left to right. (1) Drag the seam point to line
up the surface Blend. (2) Adjust a control point to
control the blend angle and amount. (3) Surface
Blend is complete (red surface).
### Chain Edge Options:
Auto Chain = Yes/No: When set to ‘Yes’ Auto
Chain will select all surfaces edges in a closed
loop after the first edge is selected.
are available along with the
ability to grab the handle and
move it in the viewport. After
selecting both the first and
second edges for the blend,
this option appears to adjust
the seam. The seam points
must line up and the direction
arrows must point in the same
direction. Select a seam point
marker, and drag it along the
closed curve. Continue to adjust the seam points
until they line with each other and the closed curves
all have the same direction.
Flip: Reverses the curve direction. Select the
seam to Flip. Or,  click the arrow end on the seam
and it will flip directions.
Chain Continuity: Controls the level of
continuity required between segments to
be selected with the Auto Chain option. See
‘Continuity Options’ below for a detailed
description.
Automatic: Attempts to align the seam points
and direction to match without intervention.
Natural: Moves the seam point to the way they
were at the beginning of the command.
Direction = Both/ Backward/ Forward:
Forward selects curves in the positive curve
direction. Backward selects curves in the negative
curve direction. And ‘Both’ selects curves in both
the positive and negative curve directions.

<!-- page 191 -->

### Adjust Surface Blend Options:
(edge) end, indicated by numbers 1 & 2. There are
five levels of Continuity to choose between:
Position, Tangency, Curvature. G3 and G4.
Position (G0): Position (G0 continuity)
A dialog box to Adjust Surface Blend appears
after the seam points are adjusted. Use the
available sliders and settings in the dialog box or
use the key controls as outlined below.
measures location only. If the end
points of each curve are in the
same location in space, the curves
are position continuous (G0) at the
ends. In other words, the two
curves in question touch each
other at their end points.
Tangency (G1): Tangency (G1 continuity)
Lock: Check the lock box to maintain the
relationship between the two curve ends. When
one slider is moved the other
goes with it.
measures position and curve
direction at the ends. In other
words, the two curves not only
touch, but they go the same
direction at the point where they
touch.
Sliders: Determine the
bulge (or distance of
influence) the surface has
on the edge curve. Sliders
control each surface end.
Curvature (G2)(default): Curvature continuity
### Keys for Control:
Shift: By default the shape curves (input edges)
are separately edited at each end, Hold the Shift
key to retain symmetry. With symmetry, point
editing is mirrored to the other end of the curve.
Alt: Hold the Alt key while dragging the handles
to change the shape curve and the surface edge.
(G2 continuity) between two
curves measures position,
direction, and radius of curvature
at the ends. If the radius of
curvature is the same at the
common end point, curves are
curvature continuous (G2). In other
words, the curves not only go the
same direction when they meet, but also have the
same radius at that point.
### Continuity Options:
G3: G3 continuity adds a third requirement; planar
acceleration. Curves that are G3
continuous touch, go the same
direction, have the same radius,
and that radius is accelerating at
the same rate at a certain point.
G3 continuous curves have equal
third derivatives.
G4: G4 continuity is very seldom used, but
Continuity: When edges
are used to make surfaces,
this sets the shape
curvature for each curve
can be important in certain
isolated cases. G4 continuous
curves have all the same
requirements as G3 curves, but
their curvature acceleration is
equal in three dimensions.

<!-- page 192 -->

### Adjust Surface Blend Options Cont’d:
### Using Handle Grips:
AddShapes: Add additional curve profile shapes
to increase control over the blend shape. This is
especially important the input shapes are complex
and more control is needed for the shape in more
locations. Click a location on each surface edge to
add another shape curve.
Planar Sections: Forces all shape curves to be
planar and parallel to the specified direction.
Same Height: If gaps between the surface vary,
this option maintains the height of the shape
curves throughout the blend.
Preview: Check to dynamically display a preview
as the options change.
The default handles have one grip that can be
dragged to change the radius. Any added or
copied handles have two grips. Use the grip on
the edge to move the handle along the edge.
Use the grip in the center to change the radius at
the handle location.
### Radius/ Distance Options:
## Variable Blend Surfaces
Radius/Distance options: Set the radius value
before the first surface is selected. And, Radius
and Distance options appear on the Command
Line when a handle grip is selected and moved.
‘VariableBlendSrf’
Creates a blend surface between surface edges
(of two intersecting surfaces) with varying radius
values, trims the original surfaces faces, and joins
the resulting surfaces.
FromCurve: Select a curve. The radius of the
curve at the picked location will be used.
Where is this Command:
Access this command from Surface>Variable
Blend Surfaces
FromTwoPoints: Pick two points to show the
Radius distance.
Note: Both surfaces must intersect.
Steps:
Dragging a handle to edit the Radius.
• Select Variable Blend Surfaces in the Surface
menu.
• Select the first surface for Variable Radius Blend
on the surface near the edge that will be blended.
(Enter a Radius value before selecting or edit the
handle later).
• Select the second surface for Variable Radius
Blend on the surface near the edge that will be
blended.
• Select a blend handle to edit. Click on the handle
and enter a different Radius value, etc. Or, press
Enter to accept the default Radius value.
• Choose from any of the Command Line options if
desired.
• Press Enter when done.
Variable Blend
Surfaces is
complete.
Surfaces are
trimmed and
joined.
