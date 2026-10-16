# Shear ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 35-36
- Section title: Shear ￼
- Section id: m8book2-036-shear
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p35_img01.png, m8book2_p35_img02.png, m8book2_p36_img01.png, m8book2_p36_img02.png, m8book2_p36_img03.png

---
<!-- section m8book2-036-shear | Shear ￼ | m8book2 pages 35-36 -->

<!-- page 35 -->

• Pick the End of the Taper axis.
• Enter a Taper Start distance.
• Enter a Taper End distance.
Perpendicular: Draws the line perpendicular to
or from a curve.
Infinite:
Yes: The deformation happens throughout the
object, even if the axis is shorter.
No: The deformation takes place only the length
of the axis. The length of the axis is important. If
the axis is shorter than the object, the taper only
applies to that part of the object.
PreserveStructure:
Yes: Preserves the control point structure of the
surface. Deformation may be less accurate if there
are too few control points in the object.
No: Objects are refit as needed with more control
points to allow accurate deformation.
Perpendicular Options: ‘Point’ allows the
selection of a point that is near, but not on
a curve, overriding the built-in object snap.
‘PointOnCurve” forces the line to go through
the first picked point on the curve instead of
allowing the point to slide along the curve. And,
‘2Curves” places the Taper axis line
perpendicular to two curves.
## Shear
Tangent: Draws the line tangent from an existing
curve.
‘Shear’
Objects are skewed parallel to one axis at a
specified angle. A sheared rectangle, for example,
is transformed into a parallelogram. The sides of
the parallelogram will progressively lengthen as
deformation proceeds but the top and bottom
lines maintain their original length.
Tangent Options: ‘Point’ allows selection of a
point that is near, but not on a curve, overriding
the built-in object snap. ‘PointOnCurve’ forces
the line to go through the first picked point on the
curve instead of allowing the point to slide along
the curve. ‘2Curves’ makes the line perpendicular
to two curves.
Where is this Command:
Access this command from Transform>Shear
Extension: Extends a curve with a line.
Steps:
Extension Steps
• Select Shear from the Transform menu.
• Select object(s) to Shear.
• Press Enter when done.
• Pick the Origin point.
• Select a curve (line) near the End you want to
extend.
• Pick the End of the line or type a Distance.
• Press Enter.
The Origin point does not move with the shear.
### Command Line Options:
• Pick the Reference point that
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
defines the first point of the
Shear Angle.
• Pick the Shear Angle.
Rigid=Yes/No: Yes: Individual objects are not
deformed as they are transformed. No: Individual
selected objects are deformed.
Flat=Yes/No: Yes: Individual selected objects
are deformed. No: Creates a one-directional, one-
dimensional taper.
Origin point selected at center of bottom. First
Reference point made at center of top. Shear Angle
selected to the right of the top.

<!-- page 36 -->

### Command Line Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
object. The deformation takes place only the
length of the axis. When Infinite is set to no the
axis length is important. If the axis is shorter than
the object, the Twist applies only to part of the
object. In addition, the Twist blends in and out at
th ends of the axis.
## Twist
## Maelstrom
‘Twist’
Deforms objects by rotating them along an axis.
Where is this Command:
Access this command from Transform>Twist
Steps:
‘Maelstrom’
Deform an object by twisting it while using two
Reference circles; one representing the part of the
object to leave alone and the other representing
the part of the object to twist.
Where is this Command:
Access this command from
Transform>Maelstrom
Steps:
• Select Twist from the Transform menu.
• Select the Objects to Twist.
• Press Enter when done.
• Pick the Start of the Twist axis.
• Pick the End of the Twist axis.
• Type the Twist Angle, or pick two Reference points
to define the Rotation Angle.
• Select Maelstrom for the Transform menu.
• Select objects to deform and press Enter.
• Pick the Center and Radius of the first Reference
circle; the part to remain the same (no
deformation).
• Pick the second
Object before and after using Twist.
radius of the
second Reference
circle representing
the area to be
deformed.
• Move the mouse to
set the Coil Angle
or Twist in the
object.
### Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed.
Object before and after using Maelstrom.
Note: See Curve menu - Circle for Circle options -
Vertical, 2Point, 3Point, Tangent, etc
Infinite:
Yes: The deformation is constant throughout the
object, even if the axis is shorter than the object.
No: The twisting is not constant throughout the
