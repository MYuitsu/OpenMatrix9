# Divide Curve

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 163-164
- Section title: Divide Curve
- Section id: m8book1-177-divide-curve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p163_img01.png, m8book1_p163_img02.png, m8book1_p164_img01.png, m8book1_p164_img02.png, m8book1_p164_img03.png

---
<!-- section m8book1-177-divide-curve | Divide Curve | m8book1 pages 163-164 -->

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

<!-- page 164 -->

The region that will be retained highlights.
A planar curve is created around the chosen
region.
The pattern on the left is before curve boolean the
pattern on the right is after.
### Command Line Options:
### Command Line Options:
DeleteInput:
None: Deletes no input curves.
All Deletes all input curves.
Length: Creates point objects a specified
distance from one another on a curve starting
from the natural start of the curve. To start
measuring at the other end, click the curve to flip
the direction.
Used: Deletes only the parts of the input curves
that correspond to the newly-created curves.
Split: Splits the curve instead of placing point
objects.
CombineRegions: Appends the selected
regions to the previously edited regions.
## Tween Curves
MarkEnds: Places a point at the ends of the
curve.
GroupOutput: Groups the resulting objects.
‘TweenCurves’
Creates curves between two open or closed
curves.
## Curve Boolean
Where is this Command:
Access this command from Curve>Tween
Curves
‘CurveBoolean’
Trims, splits, and joins curves based on their
overlapping regions.
Steps:
• Select Tween Curves from the Curve menu.
• Select the first curve.
• Select the second curve.
Where is this Command:
Access this command from Curve>Curve
Boolean
Steps
Adjust curve seams if necessary.
• Select Curve Boolean from the Curve menu.
• Select two or more overlapping curves.
• Click inside the areas to keep.
• Click an area again to remove it.
