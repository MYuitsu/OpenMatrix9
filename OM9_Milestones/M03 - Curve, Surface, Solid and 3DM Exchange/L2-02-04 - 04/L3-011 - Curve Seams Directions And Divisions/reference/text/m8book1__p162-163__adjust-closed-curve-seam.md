# Adjust Closed Curve Seam

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 162-163
- Section title: Adjust Closed Curve Seam
- Section id: m8book1-175-adjust-closed-curve-seam
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p162_img01.png, m8book1_p162_img02.png, m8book1_p163_img01.png, m8book1_p163_img02.png

---
<!-- section m8book1-175-adjust-closed-curve-seam | Adjust Closed Curve Seam | m8book1 pages 162-163 -->

<!-- page 162 -->

### Command Line Options:
## Mark Curve Start
Vertical: Draws a Helix curve whose axis is
perpendicular to the construction plane.
‘CrvStart’
Places a point at the start of a curve.
AroundCurve: Draws a Helix curve around
another curve.
Where is this Command:
Access this command from Curve>Mark Curve
Start
### Options:
Steps:
Diameter/Radius: Click the option to toggle
between picking a Radius and picking a Diameter
• Select Mark Curve Start from the Curve menu.
• Select a curve or curves.
Mode=Turns: When using this mode the number
of turns takes precedence,the pitch adjusts
automatically.
Mode=Pitch: When using this mode the pitch
takes precedence,the number of turns adjusts
automatically.
## Mark Curve End
ReverseTwist: Reverses the direction of the
twist to counter-clockwise. Drag the mouse after
changing options to preview.
‘CrvEnd’
Places a point at the end of a curve.
NumPointsPerTurn: Sets the number of control
points in each turn of the Helix. This option is only
available when AroundCurve is selected.
Where is this Command:
Access this command from Curve>Mark Curve
End
Steps
## Point
• Select Mark Curve End from the Curve menu.
• Select a curve or curves.
‘Point’
Creates a point object or point objects.
Where is this Command:
Access this command from Curve>Point
Right Click Command: Multiple Points
Steps:
• Select Point from the Curve menu.
• Pick a location for the point object.
## Adjust Closed Curve Seam
‘CrvSeam’
Changes the seam (start/end) location on closed
curves.
Where is this Command:
Access this command from Curve>Adjust
Closed Curve Seam
This gem has points applied to its prong points
(using Gem Guides) to make them easy to snap to.

<!-- page 163 -->

Steps:
Steps:
• Select Continue Interpolated Curve from the
Curve menu.
• Select an open curve near an end.
• Pick the next points.
• Press Enter to end the curve.
Note: Works best with curves that were drawn with
the InterpCrv command.
• Select Adjust Closed Curve Seam from the Curve
menu.
• Select one or more closed curves.
• Select a seam point marker, and move it along the
closed curve.
• Continue to adjust the seam points until they line
with each other and the closed curves all have the
same direction, and then press Enter.
### Command Line Options:
PersistentClose: Closes the curve as soon as
there are two points placed. You can continue to
pick points. The curve updates the shape while
remaining closed.
Note: The seam point of each closed curve is
indicated with a point. Each seam point is connected
with a tracking line. Arrows indicate the direction of
each curve.
Close: Closes the curve smoothly, creating a
periodic curve.
If a sweep has a twist in it, try to align all of the
seam points on the profiles being swept so they are
located in a similar location and facing the same
direction.
Sharp: Closes the curve with a kink, creating a
non-periodic curve.
### Command Line Options:
Undo: Reverses the last action.
Flip: Reverses the curve direction.
## Divide Curve
Automatic: Attempts to align the seam points
and directions without intervention.
Natural: Moves the seam points to the way they
were at the beginning of the command.
‘Divide’
Creates point objects along a curve or splits a
curve by the number of equal length segments or
segments of a specified length.
## Continue Interpolated Curve
Where is this Command:
Access this command from Curve>Divide
Curve
Steps:
‘ContinueinterpCrv’
Continues to draw the selected curve interpolating
through picked points.
Where is this Command:
Access this command from Curve>Continue
Interpolated Curve
• Select Divide Curve from the Curve menu.
• Select curves.
• Type the number of segments.
• The point objects this command creates are
spaced evenly along the curve, with an additional
point at each end of the curve.
