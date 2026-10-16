# Duplicate Edge

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 165-165
- Section title: Duplicate Edge
- Section id: m8book1-180-duplicate-edge
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p165_img01.png, m8book1_p165_img02.png

---
<!-- section m8book1-180-duplicate-edge | Duplicate Edge | m8book1 pages 165-165 -->

<!-- page 165 -->

## Duplicate Edge
‘DupEdge’
Creates a curve that duplicates a surface edge on
a closed or open surface..
Where is this Command:
Access this command from Curve>Duplicate
Edge
Steps:
• Select Duplicate Edge from the Curve menu.
• Select edges of a surface.
• Press enter when done.
### Command Line Options:
Number: Specifies the number of curves created
between the two input curves.
MatchMethod: Specifies the method for refining
the output curves.
None: Uses the control points of the curves for
matching. So the first control point of first curve
is matched to first control point of the second
curve.
Select edges of an object you would like to duplicate.
Refit: Refits the output curves like using the
Refit to Tolerance command. Both the input
curve and the output curve will have the same
structure. The resulting curves are usually more
complex than input unless input curves are
compatible.
### Command Line Options:
OutputLayer: Specifies the layer for the results
of the command.
Current: Places the results on the current layer.
Input: Places the results on the same layer as the
input curve.
SamplePoints: Input curves are divided to
the specified number of points on the curve,
corresponding points define new points that
output curves go through. If you are making one
tween curve, the command essentially does the
following;
## Duplicate Border
‘DupBorder’
Creates a curve or polyline that duplicates any
open edge on a surface, polysurface, hatch, or
mesh border.
Divides the two curves into an equal number
of points, finds the midpoint between the
corresponding points on the curves, and
interpolates the tween curve through those points
SampleNumber: The number of sample points
to use.
Where is this Command:
Access this command from Curve>Duplicate
Border
