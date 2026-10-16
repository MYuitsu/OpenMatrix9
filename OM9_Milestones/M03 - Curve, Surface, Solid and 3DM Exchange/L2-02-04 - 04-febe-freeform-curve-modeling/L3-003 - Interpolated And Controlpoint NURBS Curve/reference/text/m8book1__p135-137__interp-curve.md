# Interp Curve

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 135-137
- Section title: Interp Curve
- Section id: m8book1-144-interp-curve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p135_img01.png, m8book1_p135_img02.png, m8book1_p136_img01.png, m8book1_p136_img02.png, m8book1_p136_img03.png, m8book1_p136_img04.png, m8book1_p137_img01.png, m8book1_p137_img02.png, m8book1_p137_img03.png

---
<!-- section m8book1-144-interp-curve | Interp Curve | m8book1 pages 135-137 -->

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

<!-- page 136 -->

### Command Line Options:
confusing to the user, so it is always a good idea
to leave the Knots feature at its default setting.
PersistentClose: Closes the curve as soon as
there are two points placed. You can continue to
pick points. The curve updates the shape while
remaining closed.
Degree: Changing this number affects the “pull”
on the curve, making it bend at different levels
of strength (1 – 11) between the points that are
placed. Setting this number higher (its upper limit
is 11) makes the bend between each point far
more dramatically.
1 Degree Curve
3 Degree Curve
Start Tangent: When starting up the Interp
Curve command, the user has the option to start
the curve so that its starting point is tangent to
another curve or tangent to a direction shown by
picking the start and end direction.
### Knots:
“Knots” is Rhino’s term for the points that are
placed to make the curve. Typing “K” into the
Command Line and pressing Enter will reveal
that the three settings for this option are Uniform,
Chord, and SqrtChord (square root chord).
Uniform: Knots are always placed approximately
one millimeter away from one another, meaning
that the curve will bend at 1 mm increments
(as though the points on the curve, or, the little
people holding the “plank”, are spaced 1 mm away
from each other all over the grid) regardless of the
points placed by the user. It’s not a good idea to
turn this feature on, since it can make using this
tool pretty confusing.
The green curve added for illustration indicates the
user defined tangent for the start of the curve.
Chord: Is the default setting for this option, and
it means that the knot spacing corresponds to the
mouse-clicks of the user.
End Tangent: Draws the end of the curve
tangent to another curve or tangent to a direction
shown by picking a start and end direction.
SqrtChrd: (square root chord) places the knots
at a point equal to the square root of the distance
between the two points picked by the user.
This, too, makes using the Interp Curve rather
Close: Will cause the start and end point of the
curve to join smoothly creating a periodic curve.
Type ‘C’ and press enter to close the curve.

<!-- page 137 -->

Vertical: Draws the rectangle perpendicular to
the construction plane.
Sharp: Close the curve with a kink, creating a non
periodic curve. Type ’S’ and press enter to close
the curve with a sharp junction.
Vertical steps
• Pick the start of the edge.
• Pick the end of the edge.
• Pick or type a width.
Each curve has the same knots the curve on the right
has Sharp set to No and the one on the Left is set to
Yes.
Undo: Reverses the last action.
## Rectangle
Center: Draws the rectangle around a center
point.
‘Rectangle’
The Rectangle tool draws rectangles or squares.
Center steps
• Pick the center.
• Pick the other corner or type a length.
Rounded: Rounds the corners of the rectangle.
Where is this Command:
Access this command from Curve>Rectangle
Right Click Command: Rectangle Rounded
Corners.
Rounded steps
Steps:
• Draw the rectangle, then pick the radius of the
rounded corner.
• Select Rectangle from the Curve menu.
• Pick the first corner of the rectangle.
• Pick the other corner. Or type a value and press
Enter to specify the length and then pick the
other corner. Hold shift to create a square.
### Command Line Options:
3Point: Draws the rectangle using two adjacent
corner locations and a location on the opposite
side. Use this command to draw a rectangle on a
slant.
3Point steps
• Pick the start of an edge.
• Pick the end of the edge.
• Pick or type the width.
