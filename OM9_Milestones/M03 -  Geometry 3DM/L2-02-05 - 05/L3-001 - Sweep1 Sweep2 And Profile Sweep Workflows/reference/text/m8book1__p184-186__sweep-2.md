# Sweep 2 ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 184-186
- Section title: Sweep 2 ￼
- Section id: m8book1-203-sweep-2
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p184_img01.png, m8book1_p185_img01.png, m8book1_p185_img02.png, m8book1_p186_img01.png, m8book1_p186_img02.png, m8book1_p186_img03.png

---
<!-- section m8book1-203-sweep-2 | Sweep 2 ￼ | m8book1 pages 184-186 -->

<!-- page 184 -->

## Sweep 2
‘Sweep2’
Creates a surface from a series of cross-section or
Profile curves which define the surface shape and
two rail curves (often Ring Rails).
Where is this Command:
Access this command from Surface>Sweep 2
History
Sweep 2
with open
rails and
profiles
(left).
Sweep 2
with closed
rails and
profiles
(right).
Or, from the F6 menu when two curves are
selected
Steps:
### Adjust Seam Options:
Flip: Reverses the curve direction. Select a curve
seam to flip.
Automatic: Attempts to align the seam points
and directions without intervention.
• Select Sweep 2 from the Surface menu.
• Select the first rail (often an inside Ring Rail).
• Select the second rail (often an Outside Ring Rail).
• Select cross-section curves (Profiles) in the order
that the surface will pass through them.
• Press Enter when done.
• Choose any Sweep Option and click on “OK’ to
close the Sweep 2 Options dialog box.
Natural: Moves the seam points to the way they
were at the beginning of the command.
Adjust Seams Points Steps (closed curves only):
### Command Line Options:
Chain Edges: This allows for selection of
multiple, touching, surface edges to use as a rail
curve.
• Select a seam point marker, and drag it along the
closed curve.
• Continue to adjust the seam points until they line-
up with each other and the closed curves’ arrows
all have the same direction.
• Press Enter when done.
Steps:
Rules:
• Select Chain Edges in the Command Line.
• Select first segment (surface edge) for rail.
• Select next segment (surface edge) for rail.
• Continue selecting edge segments, if needed.
Then, press Enter when done.
• Select the cross-section curves (Profiles).
• Press Enter when done.
• Select any option from the Sweep Options dialog
box. Press ‘OK’ when done.
Point: Creates a surface that begins or ends at
a point. Use this option only at the start or end of
the curve series.
Steps:
• Select ‘Point’ in the Command Line at the start or
end of the selection of cross-section curves.
• Closed curve Seam Points must all be on the
inside (on the rail), or all be on the outside (away
from the rail).
• The Profile (cross-section) curves’ directions must
all match. Click on each arrow to flip the direction
of a curve.
• A single open curve or a single closed curve can
be used as the cross-section curve.
• Select multiple open cross-section curves near the
same ends.
• Select multiple closed cross-section curves in the
order the surface will pass through them.
• Select either all open or all closed cross-section
curves, but not a combination of both.
• Use surface edges (or edge) for a rail curve. See
Chain Edges option below.

<!-- page 185 -->

• Pick the point.
• Pick the cross-section curve.
• Press Enter when done.
• Select any option from the Sweep Options dialog
box. Press ‘OK’ when done.
### Cross-Section Curve Options:
Maintain Height is ON (left) and OFF (right).
### Rail Curve Options:
Do Not Simplify (default): Creates the Sweep
without doing anything to the shape curves.
Continuity is only enabled if the rails are surface
edges and the shape curves are non-rational, that
is, all control point weights are one. Exact arcs
and ellipse segments are rational. Most of the time
these controls are not used.
Note: Only continuity options that the curve structure
(point count and rational/non rational) supports will
be available (not grayed out).
Rebuild with ‘x’ control points: Rebuilds
the shape (Profile) curve control points before
creating the Sweep. Enter a value for number
of control points in the provided box. A lower
number of points will deform the curve more. A
higher number of control points will follow the
original curve more.
Position / Tangency / Curvature
Sets the continuity for the edge match. See Blend
Surface in the Surface menu for more details.
Refit within ‘x’: Refits the shape curves within
a specified tolerance before creating the Sweep.
Enter a value (mm) for the tolerance in the
provided box. The lower the tolerance value, the
closer the Sweep fits the shape curves. The higher
the value creates a greater variance between the
shape curve and the Sweep surface.
Position (G0): Position (G0 continuity)
measures location only. If the end points of each
curve are in the same location in space, the curves
are position continuous (G0) at the ends. In other
words, the two curves in question touch each
other at their end points.
Preserve First/ Last Shape: When edge
curves are used as rails, the surface may pull away
from the selected Profile curves. These options
force the surface shape to match the first profile
curve or last profile curve.
Tangency (G1): Tangency (G1 continuity)
measures position and curve direction at the ends.
In other words, the two curves not only touch, but
they go the same direction at the point where
they touch.
Maintain Height: When creating a surface,
Sweep 2 usually maintains a relationship between
the height and the width; where the rails are wider
apart, the surface is taller; where they are closer
together, the surface is shorter. To keep the height
consistent regardless of width, turn ON Maintain
Height.
Curvature (G2): Curvature continuity (G2
continuity) between two curves measures
position, direction, and radius of curvature at
the ends. If the radius of curvature is the same
at the common end point, curves are curvature

<!-- page 186 -->

continuous (G2). In other words, the curves not
only go the same direction when they meet, but
also have the same radius at that point.
Preview: Click to display a preview of the output.
After a setting change, click the Preview button
again to refresh the display.
## Sweep 2 History
‘gvSweep2History’
This History-enabled command creates a surface
from a series of cross-section (Profile) curves and
two rail curves (often Ring Rails). The created
surface automatically updates after profiles or rails
are edited.
Where is this Command:
Access this command from Surface>Sweep 2
History
Or, from the F6 menu when two curves are
selected
Closed Sweep: Creates a closed surface,
continuing the surface past the last Profile
curve around to the first curve. This option is
only available after two cross-section curves are
selected.
Steps:
Note: With closed rail curves, the first cross-section
curve selected is added to the end of the list if you
choose to create a closed surface.
Simple Sweep: Simple Sweep works in cases
where the shape curves intersect the rail at rail
edit points. This option generates simpler surfaces
in cases when the curves are perfectly set up.
• Select Sweep 2 History from the Surface menu.
• Select two rail curves.
• Select profile curves (cross-section) in the order
that the surface will pass through them.
• Press Enter when done.
• Edit a profile, or a rail and press Enter to complete
the edit. See the surface update automatically
after each edit.
Add Slash: Adds additional cross-section
alignments to control how the surface is created
between sections. Select first point on one rail.
Then, select a point on the other rail. Press Enter
when done.
Sweep 2 History displays preview curves to indicate
the Sweep surface direction and the order in which
to pick the Profile curves (pick order in red). Change
Closed = Yes to Closed = No in the Command Line
to create a gap between the first and last selected
profiles.
Add Slash - Additional alignment is added as if more
profiles existed. Select any point on one rail. Use
Perpendicular O-Snaps to set the second point on
the second rail.
