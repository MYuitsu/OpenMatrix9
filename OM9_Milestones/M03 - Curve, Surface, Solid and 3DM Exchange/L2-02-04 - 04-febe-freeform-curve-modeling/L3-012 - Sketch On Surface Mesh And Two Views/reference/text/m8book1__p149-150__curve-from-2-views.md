# Curve from 2 Views

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 149-150
- Section title: Curve from 2 Views
- Section id: m8book1-154-curve-from-2-views
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p149_img01.png, m8book1_p149_img02.png, m8book1_p149_img03.png, m8book1_p150_img01.png, m8book1_p150_img02.png, m8book1_p150_img03.png, m8book1_p150_img04.png, m8book1_p150_img05.png, m8book1_p150_img06.png

---
<!-- section m8book1-154-curve-from-2-views | Curve from 2 Views | m8book1 pages 149-150 -->

<!-- page 149 -->

### Keys for control:
G4: G4 continuity is very seldom used, but can
be important in certain isolated cases. G4
continuous curves have all the same requirements
as G3 curves, but their curvature acceleration is
equal in three dimensions.
ALT: By default the shape curves are
perpendicular to the surface edge. Hold the Alt
key while dragging the handles to change the
angle between the shape curve and the surface
edge.
SHIFT: The Shift key controls symmetry. By
default the shape curves are separately edited at
each end, with symmetry, point editing is mirrored
to the other end of the curve.
## Curve from 2 Views
‘Crv2View’
Creates a third curve from two curves that are
planar in different construction planes. The new
curve looks the same as each of the original
curves when viewed in each plane.
Reset: Reverts the continuity to its starting bulge
amount.
Flip 1 / 2: Reverses the direction of the specified
curve. Use this option if the wrong end of the
curve was inadvertently selected.
Trim: Trims the input curves to the resulting
curve.
Join: Joins the resulting curves.
Show Curvature: Displays the curvature graph.
Curve from 2 Views uses the Extrude and Intersect
commands behind the scenes. First use Extrude on both of
the planar curves. Next run the Intersect command.
Where is this Command:
Access this command from Curve>Curve from
2 Views
Steps:
• Select Curve from 2 Views from the Curve menu.
• Select the first curve.
• Select the second curve.

<!-- page 150 -->

Where is this Command:
Access this command from Curve>Curve From
2 View History
Steps:
• Select Curve from 2 Views with History from the
Curve menu.
• Select The first curve then the second curve.
• Full=True or False in the Command Line and press
Enter.
### Command Line Options:
Full=True/False: When set to true Matrix will
draw a curve both on the top and bottom of the
ring rail. When set to false Matrix will only draw a
curve on the top of the finger rail (from 9 o’clock
to 3 o’clock as seen in the Through Finger View.
Full=True Matrix creates a curve on both side of the
finger rail.
The red curve have matches the shape of both the
brown curves and green curve when viewed in there
respective planar views.
### Command Line Options:
Direction: Two points establish the direction
angle.
Steps
Full=False leaves the bottom of the ring shank
unfinished. Complete the Shank by sweeping the ring
manually.
• Pick a base point.
• Pick a second point that establishes the direction
angle.
## Pullback
## Curve 2 Views with History
‘Pull’
Creates curves and points on a given surface by
pulling a curve or points onto a single surface or
mesh object.
Where is this Command:
Access this command from Curve>Pull
Creates a third curve from two curves that are
planar in different construction planes. The new
curve looks the same as each of the original
curves when viewed in each plane. This command
behaves the same as Curve from 2 views but has
an additional option.
