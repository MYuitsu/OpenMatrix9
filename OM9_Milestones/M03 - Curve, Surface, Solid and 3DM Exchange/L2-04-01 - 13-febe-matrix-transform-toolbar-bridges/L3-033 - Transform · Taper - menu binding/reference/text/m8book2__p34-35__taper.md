# Taper ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 34-35
- Section title: Taper ￼
- Section id: m8book2-035-taper
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p34_img01.png, m8book2_p35_img01.png, m8book2_p35_img02.png

---
<!-- section m8book2-035-taper | Taper ￼ | m8book2 pages 34-35 -->

<!-- page 34 -->

### Second Radius Options:
### Start of Taper Axis Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
BothSides: Draws the object on both sides of
the start point, creating the object twice as long
as you indicate.
Normal: Draws the Taper axis line normal to a
location on a surface.
Rigid=Yes/No: Yes: individual objects are not
deformed as they are transformed. No: individual
selected objects are deformed.
Normal Steps:
Length: Specify the new overall length of the
stretch axis.
• Pick the Start of the Taper axis on the object’s
surface.
• Pick the End of the Taper axis line or enter a length.
• Enter a Taper Start distance.
• Enter a Taper End distance.
## Taper
Angled: Draws the line at a specified angle from
a reference line.
‘Taper’
Deforms objects toward or away from a specified
axis.
Angled Steps:
Where is this Command:
Access this command from Transform>Taper
Right Click command: Taper Flat Infinite
Steps:
• Pick the Start of a base (Reference) line.
• Pick the End of a base (Reference) line.
• Type an amount for Pivot Angle,.
• Pick the End of the Taper axis line or enter a length.
• Enter a Taper Start distance.
• Enter a Taper End distance.
Vertical: Draws the Taper axis line vertical to the
Construction plane.
FourPoint: Draws the line using two points to
establish direction and two points to establish
length.
• Select Taper from the Transform menu.
• Select object(s) to Taper.
• Press Enter when done.
• Select the Start of the Taper axis.
• Select the End of the Taper axis.
• Choose Command Line options.
• Select a Start distance.
• Select an End distance.
FourPoint Steps:
• Pick the Start of the base line (Reference location).
• Pick the End of the base line (second Reference
location).
• Pick the Start of the Taper axis.
• Pick the End of the Taper axis.
• Enter a Taper Start distance.
• Enter a Taper End distance.
Object before and after using Taper.
Bisector: Draws the line that bisects a specified
angle.
Bisector Steps:
• Pick the Start of the Bisector line.
• Pick the Start of the Angle to bisect.
• Pick the End of the Angle to bisect.

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
