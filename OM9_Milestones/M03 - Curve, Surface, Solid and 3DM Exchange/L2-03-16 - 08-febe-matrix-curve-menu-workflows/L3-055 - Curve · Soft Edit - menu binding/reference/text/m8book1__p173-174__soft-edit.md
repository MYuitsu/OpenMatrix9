# Soft Edit

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 173-174
- Section title: Soft Edit
- Section id: m8book1-196-soft-edit
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p173_img01.png, m8book1_p173_img02.png, m8book1_p174_img01.png

---
<!-- section m8book1-196-soft-edit | Soft Edit | m8book1 pages 173-174 -->

<!-- page 173 -->

## Extract Wire Frame
## Soft Edit
‘ExtractWire’
Creates curves that duplicate surface, polysurface
isoparametric curves, and mesh edges displayed
in the wireframe view.
‘Soft Edit”
Moves the curve area surrounding a selected
point smoothly relative to the distance from
selected point.
Where is this Command:
Access this command from Curve>Soft Edit
Where is this Command:
Access this command from Curve>Extract
Wireframe
Steps:
• Select Extract Wire Frame from the Curve menu.
• Select surfaces or polysurfaces.
Editing is done by moving the selected point. This
location on the curve is moved, and the move
is smoothly tapered off with increasing distance
along the curve from this point.
Use this command to fine-tune curves with many
control points.
The command takes over the moving of the
control points so that the curve stays smooth.
This is difficult on curves that are very dense. The
falloff distance is adjustable, allowing the changes
made to the curve to be more or less local.
Steps:
• Select Soft Edit from the Curve menu.
• Select a curve.
• Pick a point on the curve.
• Pick a point to move to.
• Press Enter to end the command.
Extracted wireframe
Notes: The selection point snaps to the closest edit
point. Corresponding control points of the curve/
surface are moved.
### Command Line Options:
OutputLayer: Specifies the layer for the results
of the command.
Current: Places the results on the current layer.
Input: Places the results on the same layer as the
input curve.
GroupOutput: Groups the resulting objects.
Soft move a curve from a selected point.

<!-- page 174 -->

### Soft Edit Command Line Options
• Pick a location to set another distance, or press
Enter if the offset should be a uniform distance
from the original.
### Command Line Options:
Distance: The distance, in model units, along
the curve from the editing point over which the
strength of the editing falls off smoothly. Either
enter a value or click on the curve to set the
distance.
SetBasePoint: Specifies the first point for a
distance for the offset if it should be different from
the point where the curve was selected.
Copy=Yes/No: Specifies whether or not the
objects are copied.
Flip: Reverses the direction.
FixEnds: Keeps the position of the curve ends
fixed.
Note: Positive values for the distance follow the
direction arrows.
No: If the Distance setting is larger than the
distance to one or both ends of the curve, the end
of the curve will be allowed to move.
Yes: If the Distance setting is larger than the
distance to one or both ends of the curve, the end
of the curve will not be allowed to move.
All control points are moved as they would be
according to the normal falloff except the end
control points. This can lead to an abrupt change
in the curve near the end on dense curves.
## Offset Normal to Surface
‘Offset Normal’
Copies a curve on a surface in the normal
direction of the surface so that all locations on the
copied curve are a specified distance from the
original curve.
Where is this Command:
Access this command from Curve>Offset
Normal to Surface
Offsetting curve from the surface in the direction of
the surface normal.
Steps:
## Arc Blend
• Select Offset Normal to Surface from the Curve
menu.
• Select a curve on a surface.
• Select the base surface.
• Specify the offset distance.
‘ArcBlend’
Creates a blend curve consisting of two arcs
between two curves with adjustable endpoints
and bulge.
The selected curve is pulled back to the surface
first. The offset distance is calculated from the
surface.
Where is this Command:
Access this command from Curve>Arc Blend
