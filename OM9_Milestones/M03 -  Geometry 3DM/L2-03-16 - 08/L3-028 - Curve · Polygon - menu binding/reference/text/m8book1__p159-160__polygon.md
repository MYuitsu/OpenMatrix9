# Polygon

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 159-160
- Section title: Polygon
- Section id: m8book1-169-polygon
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p159_img01.png, m8book1_p159_img02.png, m8book1_p159_img03.png, m8book1_p159_img04.png, m8book1_p159_img05.png, m8book1_p160_img01.png, m8book1_p160_img02.png, m8book1_p160_img03.png, m8book1_p160_img04.png

---
<!-- section m8book1-169-polygon | Polygon | m8book1 pages 159-160 -->

<!-- page 159 -->

## Polygon
‘Polygon’
Draws a closed polyline or Polygon with a
specified number of sides.
Where is this Command:
Access this command from Curve>Polygon
Steps:
• Select Polygon from the Curve menu.
• Pick a point for the center of an inscribed polygon.
• Pick a point for the corner of a polygon.
The resulting arc is a separate curve object when trim
is set to no.
ToPoint: Extends a curve with an arc by picking
the arc’s endpoint.
Steps:
• Pick the arc’s end point.
Type=Line: Makes a line extension tangent to
the original curve.
### Command Line Options:
NumSides: Specifies the number of sides for the
polygon
Inscribed: Draws the polygon where the
corners of the sides touch an imaginary circle of
a specified radius (therefore the base polygon is
inscribed inside the imaginary circle) .
Type=Smooth: Makes a smooth curve extension
curvature continuous with the original curve.
Steps:
• Pick the center of the polygon.
• Pick a corner of the polygon
ExtensionLength: The length the will be
extended. Use negative values to shorten the
curve.
Undo: Reverses the last action.

<!-- page 160 -->

Circumscribed: Draws a polygon where the
midpoints of the sides touch an imaginary circle of
a specified radius.
Steps:
• Pick the center of the polygon.
• Pick the midpoint of the polygon edge.
Vertical: Draws a polygon perpendicular to the
construction plane.
Steps:
• Pick the center and a radius.
AroundCurve: Draws a polygon perpendicular
to a curve.
Edge: Draws a polygon by defining one edge
Steps:
Steps
• Select a curve and pick the center on the curve
and a radius.
• Pick the start of the polygon edge.
• Pick the end of the polygon edge.
.
## Spiral
‘Spiral’
Draws a Spiral curve.
Where is this Command:
Access this command from Curve>Spiral
Steps:
Star: Draws a Star shaped polygon
• Select Spiral from the Curve menu.
• Pick the start of the spiral axis, this is an imaginary
line around which the spiral will wind.
• Pick the end of the spiral axis.
• Pick the first radius and start point for the spiral.
• Pick the second radius at the end of the spiral.
Steps
• Pick the center of the star.
• Pick a corner of the star.
• Pick a radius for the imaginary circle defining the
second star radius.
