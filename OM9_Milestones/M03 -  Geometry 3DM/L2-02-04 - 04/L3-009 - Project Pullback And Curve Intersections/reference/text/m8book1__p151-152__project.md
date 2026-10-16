# Project

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 151-152
- Section title: Project
- Section id: m8book1-157-project
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p151_img01.png, m8book1_p151_img02.png, m8book1_p152_img01.png, m8book1_p152_img02.png

---
<!-- section m8book1-157-project | Project | m8book1 pages 151-152 -->

<!-- page 151 -->

Steps:
TargetObject: Places the results on the same
layer as the target surface.
• Select Pullback from the Curve menu.
• Select curves.
• Select surface to pull the curves back to.
## Project
Notes: Pullback can be used in any viewport. For
best results place the curves to be pulled as close
to the surface as possible to minimize the amount
of distortion that may occur as the curve is pulled.
The curve pulls towards a point at the center of the
surface which is pulling. When exiting the command,
the newly created curves will be always be selected.
When drawing the curves, use the fewest control
points possible. This guarantees the smoothest
possible curve pullback to the target surface.
‘Project’
Creates curves and points on a given surface, by
finding the intersection of a surface and curve
projected towards the construction plane. This
command is viewport specific. For best results,
run the Project in the viewport where the curve
appears in front of the surface. This command
works on surfaces, polysurfaces and meshes.
Where is this Command:
Access this command from Curve>Project
Steps:
• Select Project from the Curve menu.
• Select curves.
• Select surface to project the curves on to.
The input curve get pulled back to the selected
surface.
### Command Line Options:
Loose: Pulls edit points back to the surface. If any
edit point misses the surface, the curve will not be
created.
Unlike Pullback project finds the intersection of all
surfaces
DeleteInput: Deletes the original curve from the
file.
OutputLayer: Specifies the layer for the results
of the command.
Notes: When exiting the command, the newly
created curves will be always be selected.
When drawing the curves, use the fewest control
points possible. This guarantees the smoothest
possible curve pullback to the target surface.
Input: Places the results on the same layer as the
input curve.
Current: Places the results on the current layer.

<!-- page 152 -->

### Command Line Options:
Notes: Intersecting curves with other curves or
surfaces results in curves or point objects.
Loose: Pulls edit points back to the surface. If any
edit point misses the surface, the curve will not be
created.
Intersecting surfaces or polysurfaces with surfaces
or polysurfaces results in curves. To create surface
and solid intersections, use the BooleanIntersection
command.
DeleteInput: Deletes the original curve from the
file.
## Offset Curve
OutputLayer: Specifies the layer for the results
of the command.
Input: Places the results on the same layer as the
input curve
‘Offset’
Copies a curve a specified distance from the
original curve. The new curve may be larger (offset
outside) or smaller (offset inside) the original
curve.
Current: Places the results on the current layer.
TargetObject: Places the results on the same
layer as the target surface.
Where is this Command:
Access this command from Curve>Offset
Curve
Right Click Command: Offset Through Point
Steps:
## Intersection
• Select Offset Curve from the Curve menu.
• Select a curve or edge to offset.
• Click on one side of the curve to set the direction
for the offset.
‘Intersect’
Creates section curves at the intersection of two
objects on a surface, polysurface, or mesh.
Note: For best results, use proportionately small
offset distances and smooth curves; otherwise, you
may get kinks and doubled-back curves.
Where is this Command:
Access this command from Curve>Intersection
Steps:
• Select Intersection from the Curve menu.
• Select objects.
• Press Enter when done.
• This creates a curve or point at the intersection of
two objects.
Offsetting inside and outside ring rail inward in order
to create a cutter.
### Command Line Options:
Distance: Sets the offset distance.
### Corner Options:
Specifies how the offset corner continuity is
handled.
