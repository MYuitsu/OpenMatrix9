# Orient Perpendicular to Curve ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 21-22
- Section title: Orient Perpendicular to Curve ￼
- Section id: m8book2-018-orient-perpendicular-to-curve

---

<!-- section m8book2-018-orient-perpendicular-to-curve | Orient Perpendicular to Curve ￼ | m8book2 pages 21-22 -->

<!-- page 21 -->

### Rigid:
## Orient Perpendicular to Curve
Rigid=Yes: This keeps the object in its original
shape and will not distort or morph it.
Rigid=No: The object is distorted as it is placed
on the surface.
‘OrientOnCrv’
This command orients an object so that it is
perpendicular to a curve selected by the user. It
is especially useful when placing a profile to be
perpendicular to a rail curve. The perpendicular
plane is determined by the current Construction
plane when the Base point is picked.
### Command Line Options:
Where is this Command:
Access this command from Transform>Orient
Perpendicular to Curve
Steps:
OnSurface: Sets the initial Base point on the
surface. This option is useful if the object is already
on the Target surface but must be copied or
moved. The option uses initial Base points on the
Target surface and the surface Normal rather than
the Construction plane Normal to determine the
initial orientation of the object.
• Select Orient Perpendicular to Curve from the
Transform menu.
• Select the Object(s) to Orient.
• Press Enter.
• Pick a Base point on the object to be oriented.
This is the point the object connects to the curve.
(The active construction plane viewport defines
the perpendicular plane on which the object is
oriented.)
• Select the Orientation curve
• Pick a new Base point on the curve.
The surface Normal direction is facing outwards so
the object will orient in that direction.
Copy=Yes/No: Specifies
whether or not the objects
are Copied. A plus sign
appears at the cursor when
copy mode is turned ON.
The curve object is placed planar in the Looking
Down viewport with its top to the left.
Flip: This reverses the
orientation of the object so the top is the bottom
and vice versa.
IgnoreTrims= Yes/No: A surface area is
represented by bounding box around the surface
behind the scenes. So, when Ignore Trims is set to
NO, Matrix will calculate the surface by its trimmed
edges (not the bounding box). When set to YES,
Matrix will allow objects to be placed on what is
represented by the bounding box (what appears
to be off the trimmed surface shown.)
Using Orient Perpendicular to Curve, the curve
object is placed on the Orientation curve.

<!-- page 22 -->

### Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when Copy mode is turned ON.
XFlip/YFlip: Reverses the orientation relative to
the original orientation to the Construction plane.
Rotate: Pick an angle or two Reference points to
rotate the oriented object.
## Orient on Curve
‘OrientOnCrv’
This command orients an object along a selected
curve at a chosen Base point.
Select object to Orient in the Looking Down
Viewport.
### Command Line Options:
Where is this Command:
Access this command from Transform>Orient
on Curve
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is turned ON.
Steps:
Perpendicular: Orients the object perpendicular
to the curve. The perpendicular plane is
determined by the current Construction plane
when the base point is picked.
## Symmetry
• Select Orient on Curve from the Transform menu.
• Select the Object(s) to Orient.
• Press Enter when done.
• Pick a Base point (where the object will touch the
curve) on the object to be oriented.
• Select the Orientation curve.
• Pick a new Base point on the curve.
‘Symmetry’
This command mirrors curves and surfaces, makes
the mirrored half tangent to the original. As the
original object is edited, the mirrored half updates
to match the original.
Where is this Command:
Access this command from
Transform>Symmetry
Object is oriented in the Through Finger Viewport.
Note: Rhino History recording must be enabled when
the command is run.
Steps:
• Select Symmetry from the Transform menu.
• Select curve end or surface edge.
• Pick the Start of the Symmetry plane (this is similar
to the Mirror command.)
• Pick the End of the Symmetry plane.
