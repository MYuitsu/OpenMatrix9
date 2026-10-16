---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 132-133
section_title: PolyLine
section_id: m8book1-142-polyline
---
# PolyLine

Source: matrix8_book_1.pdf, pages 132-133

<!-- section m8book1-142-polyline | PolyLine | m8book1 pages 132-133 -->

<!-- page 132 -->

## PolyLine
‘Polyline’
Polyline literally means many (poly) lines. This tool
allows the user to draw a single straight line or
many straight lines that together form one closed
or open shape.
Length: This option only appears when
Line is selected. Specify the length of the
next segment. When the length option
is selected you only specify the length
of the next segment the location of the point is
constrained to being parallel with the previous
point.
Where is this Command:
Access this command from Curve>Polyline
Steps:
Mode=Arc: Instead of drawing straight line
segments this option will create a polyline
composed of arc segments. The user can switch
back and forth between the two options within a
single polyline.
Arc Steps:
• Pick a Start point for the Arc segment
• Pick the End point for the Arc segment.
• Select Polyline from the Curve menu.
• Click a location for the start of the polyline
• Click a location for the end of the first line
segment.
• Continue until you complete the shape and then
press Enter.
Direction:
This options only appear when the Arc Command
Line option is selected. Specify the direction of a
line tangent to the arc’s end point. This effects the
radius of the arc.
The arc option will use the same direction for each
arc segment until a new direction is indicated.
Note: If the user wishes to create a closed polyline
curve, approach the start point of the polyline shape
with the cursor, which causes the Point O-Snap
Indicator (Point) to appear. Clicking once on this
indicator will cause the final line segment of the
polyline shape to end on the same point where the
polyline started, creating a closed curve. This is
called AutoClose hold the Alt key to suspend this
behavior.
Arc Direction Steps:
### Command Line Options:
• Pick a start point.
• Pick a tangent direction.
• Pick an end point.
PersistentClose: Closes the curve as soon as
there are two points placed. You can continue to
pick points. The curve updates the shape while
remaining closed.
In the illustration above the green line indicates the
direction chosen. From left to right the direction was
set a 45, 90, and 135 degrees.
Close: Closes the curve with an additional curve
segment between the first and last control points
and ends the command. This option only appears
after three points have been placed.
Center: This options only appear when the Arc
Command Line option is selected. Pick the center
point for the arc’s radius and then the length of
the arc. The distance from the first point and the
center define the radius of the arc segment.
Mode=Line: Line is the default option. Creates a
polyline constructed of 1 degree curve segments.

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
