# Soft Move ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 38-39
- Section title: Soft Move ￼
- Section id: m8book2-042-soft-move
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p38_img01.png, m8book2_p38_img02.png, m8book2_p38_img03.png, m8book2_p38_img04.png, m8book2_p38_img05.png, m8book2_p39_img01.png, m8book2_p39_img02.png

---
<!-- section m8book2-042-soft-move | Soft Move ￼ | m8book2 pages 38-39 -->

<!-- page 38 -->

### Options:
### Command Line Options:
U/V/N: Moves a control
point in the U, V, or
Normal direction.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
CPlane: Type the name of the Construction
plane to map to.
Scale: Sets the scale
factor of the sliders in
Millimeters.
View: Type the name of view to map to.
### UV Move Mode Options:
## MoveUVN
Along Tangent: The U and V sliders move the
points in a plane that is roughly tangent to the
surface.
‘MoveUVN’
Moves selected surface control points along the
surface in U, V and Normal directions.
Along Control Polygon:
The U and V sliders move
the points along control
polygon chords.
Where is this Command:
Access this command from Transform>Move
UVN
Along Extensions: The control point moves
along the imaginary extensions of the control
polygon.
U-Symmetry/V-Symmetry: Makes it possible
to edit two sides of a symmetrical surface at the
same time. Select all points to edit.
Smoothing: Use smoothing before ‘MoveUVN’
Steps::
• Select Move UVN from the Transform menu.
• Select Control Points On from the Top 11 menu with
a selected surface.
• Select point(s) on the surface to move. Hold ‘Shift’
to select multiple points to move.
• Use the sliders in the ‘Move UVN’ options box to
move the control point.
to make it possible to
combine the actions.
Smoothing tends to
even out the control
point grid,
Select
points to
move on a
surface.
## Soft Move
‘SoftMove’
This tool is especially effective for moving Control
Points around for an object (surface or cage) in
which they are very dense. However, it works on all
objects. To work on all points in a curve or surface
that fall within a given distance from the edit point
use the ‘SoftEditCrv’ and ‘SoftEditSrf’ commands.
Where is this Command:
Access this command from Transform>Soft
Move
Points are moved in the N (Normal) direction using
the Move UVN - ‘N’ control slider in Along Tangent
mode.

<!-- page 39 -->

Steps:
• Select Soft Move from the Transform menu.
• Select objects to Soft Move.
• Press Enter when done.
• Pick the point to Move from or click an object type
option.
• Pick the Radius point of influence.
• Pick an Offset point.
• Select a Control Point to Move, or press Enter to
End the command.
Surfaces: Select surfaces to radiate the falloff
curve and influence the Move. The reference
surface forms the tip of the Falloff dome and the
Falloff curve is active across the edges of the
surface over the set Radius.
Interactive Manipulation:
1.Drag the inner circle to the falloff distance.
2.Drag the outer circle to change the radius
of influence (fall-off/inner circle will drag
proportionally).
3.Drag the point at the top of the dome to change
the offset amount.
4.Drag the point at the base of the dome to
change the base point.
### Command Line Options
Vertical: Moves objects vertical to the
Construction plane.
Selecting an Offset point after choosing the Radius
in Soft Move.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
LinkAll: Ties the Falloff, Radius, Offset, and
Anchor to each other so changing one, changes
the others.
## Create Cage
‘Cage’
Creates a box-shaped Cage object to be used
with the Cage Edit command to deform other
objects.
Soft Move of points completed.
### Object Type Options:
Note: Cages with a small number of control points
can modify more complex objects. This ensures that
transitions are smoother than could be achieved
using standard control point editing. After creating
the Cage object, use the CageEdit command to
capture objects with the Cage. Use the Cage’s
control points to deform the captive objects.
Points: Select point objects as basis for the
Move. The Falloff curve is based on each selected
point and acts through a Sphere of Radius set by
the user.
Where is this Command:
Access this command from Transform>Create
Cage
Steps:
• Select Create Cage from the Transform menu.
• Pick the first corner of Cage base.
• Pick the second corner of Cage base.
Curves: Select curves as basis for the Falloff
curve. The Falloff is applied radially along the
curve creating a tube-shaped volume where the
Move is active.
