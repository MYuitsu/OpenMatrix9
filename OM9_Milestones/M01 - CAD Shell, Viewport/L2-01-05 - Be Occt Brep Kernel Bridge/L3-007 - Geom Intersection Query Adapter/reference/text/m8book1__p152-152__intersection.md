---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 152-152
section_title: Intersection
section_id: m8book1-158-intersection
---
# Intersection

Source: matrix8_book_1.pdf, pages 152-152

<!-- section m8book1-158-intersection | Intersection | m8book1 pages 152-152 -->

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
