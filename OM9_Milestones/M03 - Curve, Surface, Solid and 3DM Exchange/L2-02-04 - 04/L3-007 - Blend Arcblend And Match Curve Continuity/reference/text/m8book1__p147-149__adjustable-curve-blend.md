# Adjustable Curve Blend

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 147-149
- Section title: Adjustable Curve Blend
- Section id: m8book1-153-adjustable-curve-blend
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p147_img01.png, m8book1_p147_img02.png, m8book1_p147_img03.png, m8book1_p148_img01.png, m8book1_p148_img02.png, m8book1_p148_img03.png, m8book1_p148_img04.png, m8book1_p149_img01.png, m8book1_p149_img02.png, m8book1_p149_img03.png

---
<!-- section m8book1-153-adjustable-curve-blend | Adjustable Curve Blend | m8book1 pages 147-149 -->

<!-- page 147 -->

## Adjustable Curve Blend
Tangency (G1): Tangency (G1 continuity)
measures position and curve direction at the ends.
In other words, the two curves not only touch, but
they go the same direction at the point where
they touch.
‘BlendCrv’
Creates a blend curve between curves and/or
surface edges with control over the continuity
with the input curves.
Tangency places two control points in a row (one
on the original curve and one on the new curve),
creating a quicker transition between the blends.
Where is this Command:
Access this command from Curve>Adjustable
Blend Curve
Steps:
• Select Adjustable Curve Blend from the Curve
menu.
• Select the curves.
• Select a control point to adjust.
• Press the Shift key and select for symmetry.
• A preview of the curves displays that allows
adjustment of the curve before it is made.
Note: Blend is a history enabled command. If you
move or rotate the input curves the blended curve
will follow.
### Selection Options:
Edges: Restricts selection to surface edges.
Point: Specifies a point to blend to.
Curves: This option only appears if you have
select either Point or Edge in the Command Line.
This is the default option and restricts selection to
Curves.
### Adjust Curve Blend options
Curvature (G2): Curvature continuity (G2
continuity) between two curves measures
position, direction, and radius of curvature at
the ends. If the radius of curvature is the same
at the common end point, curves are curvature
continuous (G2). In other words, the curves not
only go the same direction when they meet, but
also have the same radius at that point. Curvature
places three control points in a row (one on the
original curve and two on the new curve).
Continuity Curve 1 / 2: Sets the continuity option
between the blend curve and the input curves or
edges.
Note: Experiment with the results choosing between
Curvature and Tangency.

<!-- page 148 -->

Position (G0): Position (G0 continuity)
measures location only. If the end points of each
curve are in the same location in space, the curves
are position continuous (G0) at the ends. In other
words, the two curves in question touch each
other at their end points.  Essentially the blend
curve is a straight line in between the two curves.
Curvature (G2): Curvature continuity (G2
continuity) between two curves measures
position, direction, and radius of curvature at
the ends. If the radius of curvature is the same
at the common end point, curves are curvature
continuous (G2). In other words, the curves not
only go the same direction when they meet, but
also have the same radius at that point. Curvature
places three control points in a row (one on the
original curve and two on the new curve).
Tangency (G1): Tangency (G1 continuity)
measures position and curve direction at the ends.
In other words, the two curves not only touch, but
they go the same direction at the point where
they touch.
G3: G3 continuity adds a third requirement: planar
acceleration. Curves that are G3 continuous touch,
go the same direction, have the same radius, and
that radius is accelerating at the same rate at a
certain point. G3 continuous curves have equal
third derivatives.
Tangency places two control points in a row (one
on the original curve and one on the new curve),
creating a quicker transition between the blends.

<!-- page 149 -->

### Keys for control:
G4: G4 continuity is very seldom used, but can
be important in certain isolated cases. G4
continuous curves have all the same requirements
as G3 curves, but their curvature acceleration is
equal in three dimensions.
ALT: By default the shape curves are
perpendicular to the surface edge. Hold the Alt
key while dragging the handles to change the
angle between the shape curve and the surface
edge.
SHIFT: The Shift key controls symmetry. By
default the shape curves are separately edited at
each end, with symmetry, point editing is mirrored
to the other end of the curve.
## Curve from 2 Views
‘Crv2View’
Creates a third curve from two curves that are
planar in different construction planes. The new
curve looks the same as each of the original
curves when viewed in each plane.
Reset: Reverts the continuity to its starting bulge
amount.
Flip 1 / 2: Reverses the direction of the specified
curve. Use this option if the wrong end of the
curve was inadvertently selected.
Trim: Trims the input curves to the resulting
curve.
Join: Joins the resulting curves.
Show Curvature: Displays the curvature graph.
Curve from 2 Views uses the Extrude and Intersect
commands behind the scenes. First use Extrude on both of
the planar curves. Next run the Intersect command.
Where is this Command:
Access this command from Curve>Curve from
2 Views
Steps:
• Select Curve from 2 Views from the Curve menu.
• Select the first curve.
• Select the second curve.
