# Extract Wire Frame

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 173-173
- Section title: Extract Wire Frame
- Section id: m8book1-195-extract-wire-frame
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p173_img01.png, m8book1_p173_img02.png

---
<!-- section m8book1-195-extract-wire-frame | Extract Wire Frame | m8book1 pages 173-173 -->

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
