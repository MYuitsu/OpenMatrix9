# Continue Control Point Curve

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 166-167
- Section title: Continue Control Point Curve
- Section id: m8book1-183-continue-control-point-curve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p166_img01.png, m8book1_p166_img02.png, m8book1_p167_img01.png, m8book1_p167_img02.png

---
<!-- section m8book1-183-continue-control-point-curve | Continue Control Point Curve | m8book1 pages 166-167 -->

<!-- page 166 -->

Steps:
The curve automatically closes when the cursor
moves close to the curve’s start point regardless
of whether O-snaps are toggled to the On
position.
• Select Duplicate Border from the Curve menu.
• Select objects. The border curves or polylines of
the selected objects are created as separate
objects.
Steps:
• Move the cursor close to the start point of the
curve, and pick. The curve will close.
• Press the Alt key to suspend automatic closing.
### Curve Command Line Options
Degree: Specifies the degree of the curve (or
surface).
Extracts a curve on any open edge.
When drawing a high-degree curve, the output
curve will not be the degree you request unless
there is at least one more control point than the
degree.
### Command Line Options:
PersistentClose: Closes the curve as soon as
there are two points placed. You can continue to
pick points. The curve updates the shape while
remaining closed.
OutputLayer: Specifies the layer for the results
of the command.
Current: Places the results on the current layer.
Close: Closes the curve smoothly, creating a
periodic curve.
Input: Places the results on the same layer as the
input curve.
Sharp: Closes the curve with a kink, creating a
non-periodic curve.
## Curve
Undo: Reverses the last action.
‘Curve’
Draws a curve from control point locations.
## Continue Control Point Curve
Where is this Command:
Access this command from Curve>Curve
Right Click Command: Continue Curve
‘ContinueCurve’
Continues to draw the selected curve using
control point curve.
Steps:
Where is this Command:
Access this command from Curve>Continue
Curve
• Select Curve from the Curve menu.
• Pick the start of the curve.
• Pick the next points.
• Press Enter to end the curve.

<!-- page 167 -->

Steps:
Note: The cursor is constrained to the surface
selected at the beginning of the command. Use
O-Snaps to precisely locate the extracted curve.
• Select Continue Control Point Curve from the
Curve menu.
• Select an open curve near an end.
• Pick the next points.
• Press Enter to end the curve.
Extract a curve on a surface
at a specified.
### Command Line Options:
### Command Line Options:
Angle: Set the angle of the curve that will be
extracted on the surface.
PersistentClose:  Closes the curve after
pressing Enter. The curve updates the shape when
closed.
## Cross Section Profiles
Close: Closes the curve smoothly, creating a
periodic curve.
Sharp: Closes the curve with a kink, creating a
non-periodic curve.
‘CSec’
Create cross-section curves through several
different (parallel) curves known as “Profiles” in this
command.
Undo: Reverses the last action.
Where is this Command:
Access this command from Curve>Cross
Section Profiles
## Extract Angle Isocurve
Steps:
‘gvExtractIsocurve’
Extracts a curve on a surface at a specified angle.
Where is this Command:
Access this command from Curve>Extract
Angle Isocurve
Steps:
• Select Extract Angle Isocurve from the Curve
menu.
• Select a surface.
• Specify an angle in the Command Line. (Use
hotkeys A & D to modify the angle visually).
• Pick a place to extract curve on the selected
surface.
• Using any curve creation command, draw “profile
curves” that roughly define the shape of your
object.
• Select the “profiles” in either clockwise or
counterclockwise order..
• Pick the start of a line that is perpendicular to the
current “profile curves”.
• Pick the end of the line (perpendicular to the
current “profile curves”.
• In the plane defined at the cross-section line, a
curve will be created that passes through the
profile curves, and press Enter. Once you have
made the cross-section curves, you can use the
Loft or other command to create a surface
through them.
