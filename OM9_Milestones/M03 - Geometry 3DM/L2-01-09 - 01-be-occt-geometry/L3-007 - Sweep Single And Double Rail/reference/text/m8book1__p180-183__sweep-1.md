# Sweep 1 ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 180-183
- Section title: Sweep 1 ￼
- Section id: m8book1-201-sweep-1
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p180_img01.png, m8book1_p180_img02.png, m8book1_p180_img03.png, m8book1_p180_img04.png, m8book1_p180_img05.png, m8book1_p181_img01.png, m8book1_p181_img02.png, m8book1_p181_img03.png, m8book1_p182_img01.png, m8book1_p182_img02.png, m8book1_p182_img03.png, m8book1_p182_img04.png, m8book1_p183_img01.png, m8book1_p183_img02.png, m8book1_p183_img03.png, m8book1_p183_img04.png

---
<!-- section m8book1-201-sweep-1 | Sweep 1 ￼ | m8book1 pages 180-183 -->

<!-- page 180 -->

## Sweep 1
‘Sweep1’
Creates a surface from a series of cross-section or
Profile curves which define the surface shape and
a single rail curve (often a Ring Rail) which defines
a surface edge.
• A single open curve or a single closed curve can
be used as the cross-section curve.
• Select multiple open cross-section curves near the
same ends.
• Select multiple closed cross-section curves in the
order the surface will pass through them.
• Select either all open or all closed cross-section
curves, but not a combination of both.
• Use surface edges (or edge) for the rail curve. See
Chain Edges option below.
Where is this Command:
Access this command from Surface>Sweep 1
Or, from the F6 menu when a Ring Rail or curve
is selected
Steps:
• Select Sweep 1 from the Surface menu.
• Select a rail curve (often a Ring Rail).
• Select cross-section curves (Profiles) in the order
that the surface will pass through them.
• Press Enter when done.
• Choose any Sweep 1 Option and click on “OK’ to
close the Sweep 1 Options dialog box.
Adjust the direction of the arrows to be the same by
clicking on the end of an arrow to flip directions.
Adjust seam points to all line up when using multiple
closed, cross-section curves. Before (left) and after
(right).
Adjust Seams Points Steps (closed curves only):
Sweep 1 using a single rail with one open cross-
section curve (left) and using a single rail (Ring Rail)
with two closed cross-section curves (right) with the
Closed option checked.
### Adjust Seam Options:
• Select a seam point marker, and drag it along the
closed curve.
• Continue to adjust the seam points until they line-
up with each other and the closed curves’ arrows
all have the same direction and appear in the
same location (for example, on the Ring Rail).
• Press Enter when done.
Rules:
Flip: Reverses the curve direction. Select a curve
seam to flip.
Automatic: Attempts to align the seam points
and directions without intervention.
• Closed curve Seam Points must all be on the
inside (on the rail), or all be on the outside (away
from the rail).
• The Profile (cross-section) curves’ directions must
all match. Click on each arrow to flip the direction
of a curve.
Natural: Moves the seam points to the way they
were at the beginning of the command.

<!-- page 181 -->

### Command Line Options:
Chain Edges: This allows for selection of
multiple, touching, surface edges to use as the
single rail curve.
Steps:
• Select Chain Edges in the Command Line.
• Select first segment (surface edge) for rail.
• Select next segment (surface edge) for rail.
• Continue selecting edge segments, if needed.
Then, press Enter when done.
• Select the cross-section curves (Profiles).
• Press Enter when done.
• Select any option from the Sweep 1 Options dialog
box. Press ‘OK’ when done.
Point is used at the start and end of the Sweep 1.
### Sweep 1 Rail Options:
### Style:
Select Chain Edges when using multiple surface
edges as the single rail.
Choose from one of four Sweep 1 styles using the
drop down selection menu.
Point: Creates a surface that begins or ends at
a point. Use this option only at the start or end of
the curve series.
Freeform (default): The profile curve rotates
to maintain its angle to the rail throughout the
Sweep.
Steps:
Road-like Through Finger: The profile curve
maintains its angle to the Through Finger C-Plane
throughout the Sweep.
Road-like Side View: The profile curve
maintains its angle to the Side View C-Plane
throughout the Sweep.
• Select ‘Point’ in the Command Line at the start or
end of the selection of cross-section curves.
• Pick the point.
• Pick the cross-section curve.
• Press Enter when done.
• Select any option from the Sweep 1 Options dialog
box. Press ‘OK’ when done.
Road-like Looking Down: The profile curve
maintains its angle to the Looking Down C-Plane
throughout the Sweep.

<!-- page 182 -->

### Cross-Section Curve Options:
Closed Sweep: Creates a closed surface,
continuing the surface past the last Profile curve
around to the first Profile curve. This option is only
available after two cross-section curves are used.
Note: With closed rail curves, the first cross-section
curve selected is added to the end of the list if you
choose to create a closed surface.
Global Shape Blending: The Sweep is linearly
blended from one end to the other, creating
Sweeps that tapers evenly from one cross-section
curve to the other. Otherwise, the Sweep meets
at a tangency to the curves and blends in the
middle.
Align Shapes: Allows reversing the direction
of the cross-section curves. Click the end of the
shape curve to reverse its direction and press
Enter on the keyboard.
Do Not Simplify (default): Creates the Sweep
without doing anything to the shape curves.
Global Shape
Blending was
selected in
Sweep Style
Options (left)
and without
it (right).
Untrimmed Miters: This option is available if the
rail has sharp angles or corners. The Sweep aligns
linearly, and meets the cross-section curves at the
rail angle instead of at a tangency. Turning this ON
ensures that the resulting surfaces are Untrimmed.
If it is OFF, they will be Trimmed surfaces.
Rebuild with ‘x’ control points:
Rebuilds the
shape (Profile)
curve control
points before
creating the
Sweep. Enter a
value for number
of control points in the provided box. A lower
number of points will deform the curve more. A
higher number of control points will follow the
original curve more.
Refit within ‘x’: Refits the shape curves within
a specified tolerance before creating the Sweep.
Enter a value (mm) for the tolerance in the
provided box. The lower the tolerance value, the
closer the Sweep fits the shape curves. The higher
the value creates a greater variance between the
shape curve and the Sweep surface.
Use ‘Untrimmed
Miters’ when the
rail curve has kinks
or sharp angle. This
allows the Sweep to
follow the angle of
the rail instead of
creating a tangency
to the cross-section
curves.
Simple Sweep: Simple Sweep works in cases
where the shape curves intersect the rail at rail
edit points. This option generates simpler surfaces
in cases when the curves are perfectly set up.

<!-- page 183 -->

Refit Rail: This refits the rail curve, taking the
curve’s control points and rearranging them to
smooth the curve. See ‘Refit to Tolerance” (FitCrv)
in the Curve menu.
Note: The fitting tolerance for the rail curve is
controlled by Rhino Options > Document
Properties > Units > Absolute Tolerance.
Preview: Click to display a preview of the output.
After a setting change, click the Preview button
again to refresh the display.
Edit a Profile curve. Press Enter to complete the edit
and Sweep 1 History will automatically update the
surface.
Rules:
## Sweep 1 History
• Select multiple open cross-section curves near the
same ends.
• Select multiple closed cross-section curves in the
order the surface will pass through them.
• Select either all open or all closed cross-section
curves, but not a combination of both.
‘gvSweepHistory’
This History-enabled command creates a surface
from a series of cross-section (Profile) curves and
a single rail curve (often a Ring Rail). The created
surface automatically updates after a Profile or the
rail is edited.
### Command Line Options:
Where is this Command:
Access this command from Surface>Sweep 1
History
Closed=Yes (default): Creates a closed surface,
continuing the surface past the last curve around
to the first curve. This option only appears with a
rail that is a closed curve and two or more Profiles.
Or, from the F6 menu when a Ring Rail is
selected
Steps:
Closed=No: Creates an open surface, leaving
a gap between the first and last profile selected.
This option only works when there are two or
more profiles. This option only appears with a rail
that is a closed curve and two or more Profiles.
Flip=Yes/No: When Flip equals ‘Yes’, it reverses
the direction of the Sweep (as shown by the
preview curves) and therefore the direction of the
surface created.
• Select Sweep 1 History from the Surface menu.
• Select a rail curve.
• Select Profile curves (cross-section curve), one-by-
one in the order that the surface will pass through
them following the direction of the arrows.
• Press Enter when done.
• Edit a profile, or the rail and press Enter to
complete the edit. See the surface update
automatically after each edit.
Sweep 1
History
displays
preview
curves to
indicate the
Sweep surface
direction and
the order in
which to pick
the Profile
curves (pick
order in red).
Using Flip with Sweep 1 History, before (left) and
after (right).
