# Match

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 171-172
- Section title: Match
- Section id: m8book1-193-match
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p171_img01.png, m8book1_p171_img02.png, m8book1_p171_img03.png, m8book1_p172_img01.png, m8book1_p172_img02.png

---
<!-- section m8book1-193-match | Match | m8book1 pages 171-172 -->

<!-- page 171 -->

No: Section curves will not be created for objects
that are not cut by the drawn section plane.
Yes: Section curves will be created for all selected
objects as though the drawn section plane
extended through them.
## Match
The green curves are the resulting section curve
across the surface.
‘Match’
Use this command to Match the end points of two
curves so that the two curves start and end in
the same location. This command is useful when
two curves are very close but do not touch, and
so Join cannot be run on them. Match the curves
first, and then run Join and they will join up.
### Command Line Options:
AssignLayersBy: Specifies the layer for the
contour curves and points.
Where is this Command:
Access this command from Curve>Match
Steps:
CurrentLayer: The output will be on the current
layer.
InputObject: Contour curves and points will be
on the same layer as the input objects.
• Select Match from the Curve menu.
• Select the end of an open curve.
• Select the curve to match.
• Choose the options for the Match.
JoinCurves: Specifies how contour curves
created from polysurfaces will be joined.
ByPolysurface: Curves in the same contour
plane created from a polysurface will be joined.
ByContourPlane: All curves in the same
contour plane will be joined.
Two curves before match command on the left. Right
is a preview of the curves after the match command
has run.
None: No curve joining.
### Command Line Options:
GroupObjectsByContourPlane: Specifies
how contour curves will be grouped.
SurfaceEdge: Matches the curve perpendicular
to the curve and uses the surface to determine
continuity.
No: No grouping.
### Options:
Yes: Curves on the same contour plane will be
grouped.
Sets the continuity of the match.
ExtendSection: Specifies whether the section
curves will automatically apply to all selected
objects if the section plane does not intersect
them.
Position: Ensures the points of the line match up
perfectly – meaning that the end point of the first
line is located at the same coordinates on the grid
as the start point of the next line.

<!-- page 172 -->

Tangency: Both curves share the same angle of
tangency to the arc that is drawn.
be selected for the Merge option to function.
This option takes the smooth curve created by
the Match command and rebuilds it so that the
control points are spaced out evenly over the
curve. It is also joined into one smooth curve,
so that it cannot be exploded into component
segments the way a curve merely joined by the
Join option, above, can be.
## Silhouette
Curvature: Creates
an even larger curve
joining the two lines,
and dramatically
changes one of the
lines to match the
curvature of both as
they meet.
Creates outline curves from a selected surface or
polysurface. Creates curves that duplicate surface,
polysurface isoparametric curves, and mesh
edges displayed in the wireframe view.
Where is this Command:
Access this command from Curve>Silhouette
Steps:
Preserve other End:
Maintains the shape
of each curve at its
opposite end – the side
away from the match. With curves that have fewer
than 6 control points, un-checking this option
causes the shape to change far more dramatically
with either the Tangency or the Curvature
continuity
• Select Silhouette from the Curve menu.
• Select a surface or polysurface.
None: No constraints.
Position: Location only.
When you are looking at a model, the “visual edge”
of the model is where the model stops and the
background begins. This visual edge is sometimes
called the silhouette or horizon of the model.
Tangency: Position and curve direction.
Curvature: Position, direction and radius of
curvature.
Note: The silhouette direction is based on the current
view. For example, if you look at a donut from the
top, the silhouette is two circles. If you look at a
donut from the side, the silhouette is an oval.
Perpendicular to Edge: Sets the curve
perpendicular to a surface edge,
Average Curves: Depending on which of the
three continuity options is chosen (described
above), the Average Curves option produces
a different result, spreading out the change
between the two curves (averaging them) so that
they are more even in the resulting shape (when
this option is not selected, the change occurs
mainly in the first curve that was selected during
the Match).
Join: This option creates one curve out of the
two following the Match operation. If it is not
selected, two separate curves will be created:
Merge: The option for Curvature continuity must
Silhouette of a ring.
