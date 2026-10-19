# Direction ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 107-108
- Section title: Direction ￼
- Section id: m8book1-101-direction
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p107_img01.png, m8book1_p107_img02.png, m8book1_p107_img03.png, m8book1_p107_img04.png, m8book1_p107_img05.png, m8book1_p108_img01.png, m8book1_p108_img02.png

---
<!-- section m8book1-101-direction | Direction ￼ | m8book1 pages 107-108 -->

<!-- page 107 -->

### Single Surface Options:
## Direction
The arrow color for the U and
V directions matches the
construction plane X- and
Y-axis colors.
UReverse/VReverse:
Reverses the surface U- or V-direction.
‘Dir’
Direction displays and edits the direction of the
normals of an object. Normals are an indicator
of which direction a curve or surface is oriented
(clockwise or counter clockwise) and which way
a surface is oriented (inside or outside). These
directions are indicated with white arrows that
only appear during the function.
SwapUV: Switches the U- and V- directions.
Flip: Reverses the direction of the surface.
Where is this Command:
Access this command from Utilities>Direction
Steps:
### Multiple Surfaces Options:
Click any individual surface to flip the normal.
FlipAll: Reverses the direction on all selected
surfaces.
NextMode: Cycles through the modes for all the
selected surfaces.
• Select Direction
from the Utilities
menu. Arrows show
the direction of the
objects normal.
• Select a curve or
surface.
• Move the cursor over the object to show a
dynamic arrow for the direction.
• Click on the mouse to flip the normal direction.
Note: The Command Line options vary depending on
the type of object select when you run the command.
Each set of options will be covered in turn.
Mode=FlipU/FlipV: Reverses the U-direction or
V-direction for all the selected surfaces.
Mode=FlipNormal: Reverses the normal
direction for all of the selected surfaces.
Mode=SwapUV: Switches the U and V
directions for all the selected surfaces.
Closed polysurfaces, surfaces, and lightweight
extrusion objects cannot have their normal direction
facing toward the interior of the object.
### Single Curve Options:
Flip: Reverses the direction
of the curve.
Note: Controlling the direction of curve normals
is important. For example, if a sweep results in
a twisted surface it is likely because the curves
that were swept are facing different directions. To
remedy the twist in the surface, align the direction of
all of the curves. For example, all arrows touching the
ring rail should face the same direction.
### Multiple Curve Options:
Click on an individual curve to change its
direction.
FlipAll: Reverses the direction of
all selected curves.

<!-- page 108 -->

## Check
‘Check’
Check reports errors in the selected object’s data
structure. Unfortunately, there is no reason we
can give in this help as to why a model develops
an error. Hopefully, most users will never see
the window appear with the occurrence of bad
geometry.
During advanced modeling techniques such as Smart
Flow and curve network controlling the direction of
surface normals as well as the orientation of U and V
direction is important. For surfaces, a user may need
to alter normals when applying a bitmap texture map
or during a Boolean operation for a surface that is
not closed. If gems placed with Gem Layout tools
appear upside-down, it is usually necessary to adjust
Directions on a surface. Sometimes, if a polysurface
is open and it doesn’t shade properly, it is necessary
to check its directions and make sure they are
consistent (all facing out).
Where is this Command:
Access this command from Utilities>Check
## Select Bad Objects
Steps:
‘SelBadObjects’
“Bad” objects are ones that violate certain NURBS
rules or have structural issues that cause invalid
geometry. The Select Bad Objects command
scans all objects in the viewport. If a model is
“bad” or invalid, an error message appears and the
object highlights.
• Select Check
from the
Utilities menu.
• Select objects
to check.
Note: If there are no problems with the model, a
message stating so appears in the feedback window.
Where is this Command:
Access this command from Utilities>Select Bad
Objects
Steps:
If there are errors in the model, a message
displays. Click the Done button to close the
window. The best fix for a failed model is to delete
and rebuild it.
## Show Edges
• Click the Select Bad Objects command from the
Utilities menu.
• If bad objects are present the ‘Model Errors’ dialog
appears.
• Click Done in the ‘Model Errors’ dialog. The bad
objects will be selected.
Note: The best way to fix bad models is to delete or
rebuild the object.
‘ShowEdges’
The Show Edges command shows the edges of
a surface or polysurface if it has open or naked
edges, or if the object is closed. This tool shows
the start/stop point and sections of a surface
edge as shown in the Split Edge section. It is
also useful to preview where an object would be
separated if the Explode command is used.
If there are no errors in any objects in the viewport,
a message in the
feedback window
appears indicating
there are no bad
objects.
Where is this Command:
Access this command from Utilities>Show
Edges
Steps:
It is a good idea to check models right after
Boolean operations, which can cause invalid
models. If it is early in the design process and a
model has failed, it is best to remake the model.
• Select a polysurface for edge display.
• Select Show Edges from the Utilities menu.
This opens the ‘Edge Analysis’ menu.
