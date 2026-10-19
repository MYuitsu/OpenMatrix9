# Remap to C-Plane ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 37-38
- Section title: Remap to C-Plane ￼
- Section id: m8book2-040-remap-to-c-plane
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p37_img01.png, m8book2_p37_img02.png, m8book2_p37_img03.png, m8book2_p38_img01.png, m8book2_p38_img02.png, m8book2_p38_img03.png, m8book2_p38_img04.png, m8book2_p38_img05.png

---
<!-- section m8book2-040-remap-to-c-plane | Remap to C-Plane ￼ | m8book2 pages 37-38 -->

<!-- page 37 -->

### Second Radius Options:
## Remap to C-Plane
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
‘RemapCPlane’
Reorients selected objects to a different
Construction plane with the option to copy.
Where is this Command:
Access this command from Transform>Remap
to C-Plane
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed.
Steps:
## Make Down Facing
• Select Remap to C-Plane from the Transform
menu.
• Select the object(s) to remap.
• Press Enter when done.
• Click in the viewport (C-Plane) to map to. This is
where you want the object(s) oriented.
‘gvMakeDownFacing’
Rotates an object and places a selected surface
(face) at bottom, flat on the viewport. This works
for simple, joined objects with a flat side to be
selected. This is useful for quickly rotating simple
objects used for rendering.
The objects are moved and rotated so they have the
same relationship to the new Construction plane as
they had to the original Construction plane.
Where is this Command:
Access this command from Transform>Make
Down Facing
Steps:
• Select Make Down Facing from the Transform
menu.
• Select object to move.
• Select a Face of the object to Rotate to the
bottom.
Object before Remap to C-Plane is oriented face-up
in the Looking Down viewport.
Object using Make Down Facing before and after.
The surface on the front of the ring was selected as
the ‘face’ to rotate.
Object after Remap to C-Plane is oriented face-up in
the Side viewport.

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
