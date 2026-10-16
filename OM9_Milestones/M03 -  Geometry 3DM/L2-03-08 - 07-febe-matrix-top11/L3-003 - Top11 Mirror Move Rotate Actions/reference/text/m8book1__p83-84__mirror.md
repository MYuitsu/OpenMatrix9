# Mirror

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 83-84
- Section title: Mirror
- Section id: m8book1-066-mirror
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p83_img01.png, m8book1_p23_img01.png, m8book1_p84_img01.png, m8book1_p84_img02.png

---
<!-- section m8book1-066-mirror | Mirror | m8book1 pages 83-84 -->

<!-- page 83 -->

longer needed, simply delete it after the process is
complete.
Where is this Command:
Access this command from Top 11
Commands>Mirror
only be used for single surfaces, not polysurfaces.
When applied to a curve, the Control Points On
button affects the bend of the curve between the
Edit Points which are the knots (points) on the
curve. The Edit Points On function only affects
the points and allows the user to move them to a
different location
Right-Click Command: Mirror from F4
Steps:
Where is this Command:
Access this command from Top 11
Commands>Control Points On
• Select Mirror from the Top 11 menu.
• Select the Object(s) to Mirror.
• Press Enter when done.
• Pick the Start Point of the Mirror plane.
Or, by pressing F10 with an object selected.
Steps:
The Mirror plane determines the orientation of the
mirrored objects. Frequently this will be center of
the universe (0,0,0) or F4.
• Pick the End of the Mirror plane.
• Select Control Points On from the Top 11 menu.
• Select Object(s) for Control Point display.
• Press Enter when done.
• Press Escape (ESC) to turn Control Points off
when done.
Mirror a gem and bezel to the opposite side of the ring
to complete the design.
### Command Line Options:
3point: Pick three points to define a Mirror plane.
Note:
When using Control
Points to edit curves
that are smooth and
consist of several curve
segments joined together,
the curve will fuse into a
single curve that cannot
be exploded. If necessary use the Split command
with the Point option and Near O-Snap to break
the curve into segments. To ensure the seams
of polysurfaces do not accidentally crack open,
polysurface control points cannot be turned
on. To control-point edit a polysurface, Explode
the object into separate surfaces first or use
the Extract Surface command to separate the
surfaces you want to edit. While control points
are on, they can be selected and deleted. This
changes the shape of the curve or surface.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
## Mirror
X Axis: Automatically mirrors the object around
the Construction plane X axis.
Y Axis: Automatically mirrors the object around
the Construction plane Y axis.
‘Mirror’
The Mirror command makes a true mirror image
of the objects selected. The Mirror command will
mirror any object – curves, points, surfaces, or
polysurfaces. This command makes a copy of the
object being mirrored. If the original object is no

<!-- page 84 -->

### Command Line Options:
## Move
Vertical: Moves the object vertical to the current
Construction plane.
## Rotate
‘Rotate’
The Rotate command rotates objects in a
360-degree radius around a user-selected point
in space.
Where is this Command:
Access this command from Top 11
Commands>Rotate
‘Move’
The Move command prompts the user to move
an object from one location to another in the
viewports. This is different from simply clicking on
an object or its control points and moving them
with the cursor. Because the Move command
prompts the user for a “Point to move from” and
a “Point to move to”, the O-Snaps may be used to
select an object exactly at the point from which
it needs to be moved and move it precisely to a
new location where it is needed. This precision is
especially useful when the moved object will be
joined to another after the move.
Steps:
Where is this Command:
Access this command from Top 11
Commands>Move
• Select Rotate from the Top 11 menu.
• Select the Objects to rotate.
• Pick the center of rotation.
• Type the rotation angle in the Command Line, or
pick two Reference points.
Steps:
• Select Move from the Top 11 menu.
• Select the Object(s) to Move.
• Press Enter when done.
• Pick a Point to move from.
• Pick a Point to move to.
The head and gem are being rotated together
from their shared center.
### Command Line Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
## Explode
Move objects precisely from one point to another.
‘Explode’
This command separates a polysurface (an object
made up of many separate surfaces, joined) or a
polyline curve (made up of many curves joined
together) into its component objects. There won’t
be any visible change in the model, but each
component item is now able to be selected. To re-
join the objects, select them all and click “Join”.
