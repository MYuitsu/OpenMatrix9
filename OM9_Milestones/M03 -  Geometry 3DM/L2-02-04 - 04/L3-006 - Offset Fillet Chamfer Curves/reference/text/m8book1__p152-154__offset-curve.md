# Offset Curve

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 152-154
- Section title: Offset Curve
- Section id: m8book1-159-offset-curve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p152_img01.png, m8book1_p152_img02.png, m8book1_p153_img01.png, m8book1_p153_img02.png, m8book1_p153_img03.png, m8book1_p153_img04.png, m8book1_p153_img05.png, m8book1_p154_img01.png, m8book1_p154_img02.png, m8book1_p154_img03.png, m8book1_p154_img04.png

---
<!-- section m8book1-159-offset-curve | Offset Curve | m8book1 pages 152-154 -->

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

<!-- page 153 -->

Chamfer: The corners of the offset curves will be
filled with a straight line between their endpoints.
Sharp: The corners of the offset curves will be
extended to meet at sharp corners with position
(G0) continuity.
ThroughPoint: Offsets through a picked point
instead of at a set distance.
Round: The corners of the offset curves will be
filled with arc segments with tangent (G1)
continuity.
Tolerance: Sets the tolerance for the offset
curve.
BothSides: Offsets the curve on both sides of
the input curve.
Smooth: The corners of the offset curves will be
filled with blend segments with curvature (G2)
continuity.
The green curve was the original curve in this
example
InCPlane: The Curve is offset in the plane of the
original curve. This option offsets the curve in the
plane of the current construction plane instead.
### Cap:
Closes the gap at the end of the curves.
None: No closing segments are added. The
offset curve goes on the current layer.

<!-- page 154 -->

Where is this Command:
Access this command from Curve>Offset
Curve on Surface
Right Click Command: Offset Curve on Surface
ThroughPoint
Steps:
Flat: Straight line segments are added between
the curves and joined. The object goes on the
same layer as the original.
• Select Offset Curve on Surface from the Curve
menu.
• Select a curve on surface.
• Select the base surface.
• Type the offset distance.
Round: Tangent arcs are added between the
curves and joined. The offset curve goes on the
same layer as the original.
Yellow curve is an offset off the rings surface edge,
### Command Line Options:
ThroughPoint: Offsets through a picked point
instead of at a set distance. The ThroughPoint
option lets you keep picking offset locations for
the next curve on the surface at any distance.
Flip: Reverses the direction of the offset.
## Offset Curve on Surface
Number: Specifies the number of curves to be
offset. Each curve is offset the offset distance
from the previous curve.
## Extract Isocurve
‘OffsetCrvOnSrf’
Copies a curve or surface edge on a surface
so that all locations on the copied curve are a
specified distance from the original curve and lie
on the surface. The new curve on the surface may
be larger (offset outside) or smaller (offset inside)
the original curve.
‘ExtractIsocurve’
Creates curves along a surface by duplicating a
surfaces isocurves.
Where is this Command:
Access this command from Curve>Extract
Isocurve
