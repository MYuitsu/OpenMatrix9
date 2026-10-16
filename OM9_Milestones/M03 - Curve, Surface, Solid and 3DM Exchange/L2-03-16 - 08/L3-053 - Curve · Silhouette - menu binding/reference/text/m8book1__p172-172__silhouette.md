# Silhouette

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 172-172
- Section title: Silhouette
- Section id: m8book1-194-silhouette
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p172_img01.png, m8book1_p172_img02.png

---
<!-- section m8book1-194-silhouette | Silhouette | m8book1 pages 172-172 -->

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
