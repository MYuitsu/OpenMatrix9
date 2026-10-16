# Spiral

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 160-161
- Section title: Spiral
- Section id: m8book1-170-spiral
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p160_img01.png, m8book1_p160_img02.png, m8book1_p160_img03.png, m8book1_p160_img04.png, m8book1_p161_img01.png, m8book1_p161_img02.png, m8book1_p161_img03.png, m8book1_p161_img04.png

---
<!-- section m8book1-170-spiral | Spiral | m8book1 pages 160-161 -->

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

<!-- page 161 -->

### Command Line Options:
Flat: Draws a planar spiral curve.
ReverseTwist: Reverses the direction of the
twist to counter-clockwise. Drag the mouse after
changing options to preview.
NumPointsPerTurn: Sets the number of control
points in each turn of the Spiral. This option is only
available when AroundCurve is selected.
Vertical: Draws a spiral curve whose axis
perpendicular to the construction plane.
The 2 spirals above were created with the same settings
except the spiral on the right has NumPointsPerTurn set to
40 and the one on the left 10. Too many control points can
make it difficult to smoothly edit a curve.
AroundCurve: Draws a spiral curve around
another curve.
## Helix
‘Helix’
Think of the Helix as a spiral with a fixed radius.
Only one radius can be defined for the helix,
meaning that it does not “grow” or “shrink” from
start to finish but is simply a uniform-sized coil.
Where is this Command:
Access this command from Curve>Helix
Steps:
### Options:
• Select Helix from the Curve menu.
• Pick the start of the helix axis.
• Pick the end of the helix axis.
• Specify the radius.
Diameter/Radius: Click the option to toggle
between picking a Radius and picking a Diameter
Mode=Turns: When using this mode the number
of turns takes precedence,the pitch adjusts
automatically.
Mode=Pitch: When using this mode the pitch
takes precedence,the number of turns adjusts
automatically.
