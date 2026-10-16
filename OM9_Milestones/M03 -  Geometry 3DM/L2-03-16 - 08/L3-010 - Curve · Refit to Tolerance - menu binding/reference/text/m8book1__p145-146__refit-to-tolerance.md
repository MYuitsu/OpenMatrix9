# Refit to Tolerance

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 145-146
- Section title: Refit to Tolerance
- Section id: m8book1-151-refit-to-tolerance
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p145_img01.png, m8book1_p145_img02.png, m8book1_p145_img03.png, m8book1_p146_img01.png, m8book1_p146_img02.png

---
<!-- section m8book1-151-refit-to-tolerance | Refit to Tolerance | m8book1 pages 145-146 -->

<!-- page 145 -->

## Refit to Tolerance
Degree: Specifies the degree of the curve (or
surface.) When drawing a high-degree curve, the
output will not be the degree you request unless
there is at least one more control point than the
degree.
DeleteInput: Deletes the original geometry from
the file.
‘FitCrv’
Reconstructs the curve using the lowest number
of control points possible, while maintaining
the shape of the curve. This command results
in a non-rational NURBS curve of a specified
degree that matches the input curve to within the
specified tolerance. The resulting curve may have
fewer control points than the input, or it may have
more points if the input is simple. Generally, use
this command to fit very dense or complex curves
to a simpler structure.
Create new object on current layer: Creates
the new objects on the current layer. Clear this
checkbox to place the new objects on the layer of
the original curves.
Where is this Command:
Access this command from Curve>FitCrv
Steps:
Maximum deviation: Reports the maximum
deviation from the original curve in the feedback
line (above the Command Line) when Preview is
clicked. A Point object is placed on the curves
indicating the area on the curve with the largest
deviation.
• Select Refit to Tolerance from the Curve menu.
• Select curves.
• Type a new tolerance value in the Command Line
or press enter.
Preview: Click to display a preview of the output.
If you change the settings, click the Preview
button again to refresh the display.
The pink line indicates the line that will be the result
of the rebuild. This particular curve went from 17
points to seven, resulting in a deviation of around .41
mm. Your particular circumstances will dictate the
amount of deviation that is acceptable.
Notes:
• When the input to the FitCrv command is a
polyline, the FitCrv command treats the polyline
vertices as a list of points, and it tries to compute
a curve that goes near the points but has a
reasonable number of control points. The FitCrv
command is meant for polylines with many closely
spaced points.
• When the input to the FitCrv command is a control
points wiggly curve with many control points, the
FitCrv command tries to compute a curve that
has the same general shape but fewer control
points.
Notes:
•  Rebuild can be an important tool if you are having
difficulty with your sweeps or gem on curve. This
is often an issue of having a high point count
which can be resolved with the Rebuild or Refit
command.
• If you use fewer edit points, the shape will relax.
Use as few edit points as possible while still
keeping the shape of the curve.
• Rebuild Curve results in a single curve which
cannot be exploded.
•  Curves in Matrix use a Degree value of 3. Degree 1
curves is a straight line.

<!-- page 146 -->

### Command Line Options:
### Refit to Tolerance Command Line
### Options:
DeleteInput: Deletes the original geometry from
the file.
Perpendicular: Select this option before
choosing either curve to blend a curve to a
surface edge perpendicular to the surface with
continuity.
Degree: Specifies the degree of the curve (or
surface.) When drawing a high-degree curve, the
output will not be the degree you request unless
there is at least one more control point than the
degree.
OutputLayer: Specifies the layer for the results
of the command.
Current: Places the results on the current layer.
Input: Places the results on the same layer as the
input curve.
Angle Tolerance: Controls when Matrix
considers two curves or surfaces to be tangent.
This option allows you to change the Angle
Tolerance for the selected object from the model
tolerances in the program. Model tolerances can
be adjusted in Rhino options.
AtAngle: Orients the direction of the blend
curve at an angle other than perpendicular to the
edge. Choosing this option prompts for a point
or location on the edge for the start of the blend
and the direction at the start of the blend, which
sets the angle of the blend. Hold the Shift key, to
restrict the angle direction by ortho but keeping it
tangent to the edge curve.
## Quick Curve Blend
### Continuity Options:
‘Blend’
Creates a curve that blends between two curves
maintaining continuity with the input curves.
Where is this Command:
Access this command from Curve>Quick Curve
Blend
Position (G0): Position (G0 continuity)
measures location only. If the end points of each
curve are in the same location in space, the curves
are position continuous (G0) at the ends. In other
words, the two curves in question touch each
other at their end points.  Essentially the blend
curve is a straight line in between the two curves
Steps:
• Select Quick Curve Blend from the Curve menu.
• Select the first curve near the end where you want
the blend to start.
• Select the second curve near the blend to end.
Note: Blend is a history enabled command. If you
move or rotate the input curves the blended curve
will follow.
Note: The resulting curve is a separate curve.  Join
the curves if needed.
