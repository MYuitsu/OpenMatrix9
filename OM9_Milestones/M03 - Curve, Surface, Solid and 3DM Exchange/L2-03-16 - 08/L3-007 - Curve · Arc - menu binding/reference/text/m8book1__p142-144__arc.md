# Arc

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 142-144
- Section title: Arc
- Section id: m8book1-148-arc
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p142_img01.png, m8book1_p142_img02.png, m8book1_p142_img03.png, m8book1_p143_img01.png, m8book1_p143_img02.png, m8book1_p143_img03.png, m8book1_p144_img01.png, m8book1_p144_img02.png

---
<!-- section m8book1-148-arc | Arc | m8book1 pages 142-144 -->

<!-- page 142 -->

## Arc
StartPoint: Draws an arc starting from the arc’s
first end point.
StartPoint Steps:
‘Arc’
Draws an Arc curve using a center location, one
end point and a second end point or an angle.
• Pick the arc’s start point.
• Pick the arc’s end point.
• Click a location the arc passes through.
Where is this Command:
Access this command from Curve>Arc
Right Click Command: Arc Extension
Steps:
• Select Arc from the Curve menu.
• Pick the center of the arc’s radius.
• Pick the arc’s first endpoint This point is also the
arc’s start point.
• Pick the arc’s second end point or type an angle.
### StartPoint options:
Tilted: Draws an arc whose plane is tilted from
the current construction plane. In a different
viewport or with elevator mode, pick the start of
the tilted arc, and specify the arc’s orientation.
Direction: Specifies the direction of a line
tangent to the arc’s first end point.
Direction Steps
Length: Type a positive or negative number or
pick two points to specify the arc curve’s length.
• Pick a start point.
• Pick a tangent direction
• Pick an end point.
### Command Line Options:
Deformable: Draws a NURBS approximation of
the arc.
ThroughPoint: Specifies a location the arc curve
must pass through.
ThroughPoint Steps
• Pick a start point.
• Pick a location that the arc must pass through..
• Pick an end point.
Deformable (degree 3) (left);  Not Deformable
(degree 2) (right).
Degree: Specifies the degree of the curve.
Note: When drawing a high-degree curve, the output
curve will not be the degree you request unless there
is at least one more control point than the degree.
The red point is the start point. The green point is the
through point.
PointCount: Specifies the number of control
points.

<!-- page 143 -->

Center: Specifies a center point for the arc radius.
Center Steps
• Pick a center point.
Radius: Specifies a radius. If a tangent point
exists on the second curve that meets the radius
requirement  the tangent constraint will appear at
that point as you drag the circle along the second
curve. This option is only available when you are
defining the second tangent point.
Extension: Extends a curve with an arc by
picking the arc’s endpoint.
Extension Steps
1. Pick the curve to extend.
2. Pick the arc’s end point.
Tangent: Draws an arc tangent to two curves
with an option to specify a radius.
Tangent Steps:
• Select a tangent curve.
• Select the second tangent curve, or type a
radius to restrict the arc to the specified radius.
If a tangent point exists on the second curve
that meets the radius requirement, the tangent
constraint will appear at that point as you drag
the arc around.
• Select a third tangent curve, or press Enter to draw
a circle.
• Click to choose the arc direction from the options
displayed when the mouse is dragged.
### Extension Options
Center: Extends a curve with an arc by picking
the arc’s center and endpoint.
Steps
• Pick the arc’s center point.
• Pick the arc’s end point.
### Tangent Options:
Point: Releases the command form the tangent
constraint. Pick a point not on the tangent curve
for the arc to pass through.
Notes:
• The arc extension is not joined to the original
curve.
• To draw an arc extension and join it in one step,
use the Extend command.
• The center point is constrained to a plane
perpendicular to the curve end. If the extension is
drawn in a view perpendicular to the constraint
plane, the input will be automatically locked
parallel to the construction plane.
• In other views Matrix tries to make sure one of
the ortho directions is parallel to the construction
plane. Press the Shift key when making the pick to
maintain the ortho direction
FromFirstPoint: Forces the arc to go through
the first picked point on the curve instead of
allowing the point to slide along the curve. Option
only available when the first curve the are is
tangent to is selected .

<!-- page 144 -->

Center: Specifies a center point for the arc radius.
## Arc Direction
Center Steps
• Pick a center point.
‘Arc_StartPoint’
Draws an arc starting from the arc’s first end point
as opposed to starting from the center of the arc.
Arc Direction is the Arc’s Command Line option
StartPoint so the option for the two commands
are the same.
Where is this Command:
Access this command from Curve>Arc
Direction
Steps:
## Rebuild
• Select Arc Direction from the Curve menu.
• Pick the start point of the arc
• Pick the end point of the arc
• Pick a third point to control the arcs direction and
amount.
‘Rebuild’
Reconstructs selected curves to a specified
degree and control point number. The points are
evenly redistributed evenly along the rebuilt curve.
### Command Line Options:
Where is this Command:
Access this command from Curve>Rebuild
Steps:
Direction: Specifies the direction of a line
tangent to the arc’s first end point.
Direction Steps
• Pick a start point.
• Pick a tangent direction
• Pick an end point.
ThroughPoint: Specifies a location the arc curve
must pass through.
• Select Rebuild from the Curve menu.
• Select the curves you wish to rebuild.
• The current Point Count and Degree are shown in
parenthesis.
• Enter a new value for the Point Count.
• Click Preview to see what the rebuilt curves will
look like.
• When you are satisfied with the results, click OK.
ThroughPoint Steps:
• Pick a start point.
• Pick a location.
• Pick an end point.
### Options:
PointCount: Specifies the number of control
points.
The red point is the start point. The green point is the
through point.
