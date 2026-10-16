# Line

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 133-135
- Section title: Line
- Section id: m8book1-143-line
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p133_img01.png, m8book1_p133_img02.png, m8book1_p133_img03.png, m8book1_p133_img04.png, m8book1_p134_img01.png, m8book1_p134_img02.png, m8book1_p134_img03.png, m8book1_p135_img01.png, m8book1_p135_img02.png

---
<!-- section m8book1-143-line | Line | m8book1 pages 133-135 -->

<!-- page 133 -->

Center Steps:
Normal: Draws the line perpendicular or normal
to a location on a surface.
• Select the center of the arc.
• Pick the point for the end of the arc.
Normal Steps:
• Select a surface.
• Pick the start of the line on the surface.
• Pick the end of the line or type a length, and press
Enter.
The Orange dot indicates the start of the arc, the
Green dot the center of the arc and the Blue dot the
end of the arc.
### Normal Options:
Helpers=Yes/No: Turns on dynamic tracking
similar to Smart Track. Helpers indicates when the
next point on the polyline is tangent or ortho.
## Line
IgnoreTrims:
No: Surface trims are taken into consideration.
When the marker is off the visible part
of the surface, the no-access cursor is
shown.
‘Line’
Draws one Line segment
Yes: Surface trims are ignored. When the marker
misses the untrimmed surface, the no-access
cursor is shown.
Where is this Command:
Access this command from Curve>Line
Angled: Draws the line at a specified angle from
a reference line.
Right Click Command: Line Both Sides
Angled Steps:
Steps:
• Pick the start of a base (reference) line.
• Pick the end of a base (reference) line.
• Type the pivot angle, and press Enter.
• Pick the end of the line.
• Select Line from the Curve menu.
• Click a location for the start of the line
• Click a location for the end of the line.
### Command Line Options:
BothSides: Draws the object on both sides of
the start point, creating the object twice as long
as you indicate. The first mouse click places the
midpoint of the line.
Vertical: Draws the line vertical to the
construction plane.

<!-- page 134 -->

Vertical Steps:
Perpendicular: Draws the line perpendicular to
or from a curve.
Perpendicular Steps:
• Pick the start of the line.
• Pick the end of the line or type a length and press
Enter.
• Pick the start of the line on a curve.
• Pick the end of the line.
A white snap appears along the curve indicating that
the curve is perpendicular.
### Perpendicular Options:
FourPoint: Draws the line using two points to
establish direction and two points to establish
length.
FourPoint Steps:
BothSides: Draws the object on both sides of
the start point, creating the object twice as long
as you indicate. The first mouse click places the
midpoint of the line.
• Pick the start of the base line (reference location).
• Pick the end of the base line (second reference
location).
• Pick the start of the line.
• Pick the end of the line.
Point: Allows you to pick a point that is near, but
not on a curve, overriding the built-in object snap.
Bisector: Draws the line that bisects a specified
angle.
Bisector Steps:
• Pick the start of the bisector line.
• Pick the start of the angle to bisect.
• Pick the end of the angle to bisect.
• Pick the end of the line or type a length, and press
Enter.
PointOnCurve:
Pick a point along the curve to serve as the
start of your curve. This option allows you to
snap to a particular location on the curve while
the end point is also being constrained to be
perpendicular to the original curve.
FromFirstPoint: Forces the line to go through
the first picked point on the curve instead of
allowing the point to slide along the curve. This
option is only available when BothSides or
PointOnCurve is selected.
2Curves: The line will be perpendicular to two
curves.
Tangent: Draws the line tangent from an existing
curve.
Bisects a user defined angle.

<!-- page 135 -->

Tangent Steps:
Extension Steps
• Pick the start of the line on a curve.
• Pick the end of the line.
• Select a curve (line) near the end you want to
extend.
• Pick the end of the line or type a distance and
press Enter.
Extend an existing curve.
## Interp Curve
A white snap appears along the curve that the new
curve is tangent to.
### Tangent Options:
BothSides: Draws the object on both sides of
the start point, creating the object twice as long
as you indicate. The first mouse click places the
midpoint of the line.
‘InterpCrv’
Interp Curve creates a free-from curve whose
shape is dependent upon points placed by the
user. Each click of the mouse sets a point that
falls along the curve. The placement of each point
determines the shape and curvature of the line:
more points closer together make for a tighter
curve that is easier to control, while fewer points
placed further apart make for a looser, smoother
curve that is more difficult to control.
Point: Allows you to pick a point that is near, but
not on a curve, overriding the built-in object snap.
Where is this Command:
Access this command from Curve>Interp Curve
Right Click Command: Continue Interp Curve
Steps:
Point on Curve:
Pick a point along the curve to serve as the start
of your curve. This option allows you to snap to
a particular location on the curve while the end
point is also being constrained to be tangent to
the original curve.
• Select Interp Curve from the Curve menu.
• Pick the start of the curve.
• Pick the next points along the curve.
• Press Enter to end the curve.
FromFirstPoint: Forces the line to go through
the first picked point on the curve instead of
allowing the point to slide along the curve. This
option is only available when BothSides or
PointOnCurve is selected.
The curve automatically closes when the cursor
moves close to the curve’s start point regardless
of whether O-snaps are toggled to the On
position.
Steps:
2Curves: The line will be perpendicular to two
curves.
Extension: Extends a curve with a line.
• Move the cursor close to the start point of the
curve, and pick. The curve will close.
• Press the Alt key to suspend automatic closing.
