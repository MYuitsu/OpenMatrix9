# Quick Curve Blend

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 146-146
- Section title: Quick Curve Blend
- Section id: m8book1-152-quick-curve-blend
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p146_img01.png, m8book1_p146_img02.png

---
<!-- section m8book1-152-quick-curve-blend | Quick Curve Blend | m8book1 pages 146-146 -->

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
