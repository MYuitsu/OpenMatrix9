# Control Points On

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 82-83
- Section title: Control Points On
- Section id: m8book1-065-control-points-on
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p83_img01.png, m8book1_p23_img01.png

---
<!-- section m8book1-065-control-points-on | Control Points On | m8book1 pages 82-83 -->

<!-- page 82 -->

No: Pick a new direction for the next copy.
## Duplicate
### Copy once with the Alt Key:
‘Copy’
Makes duplicates of the selected objects.
While dragging any object in the viewports, press
the “Alt” key. A “+” will appear next to your cursor to
show you are in Copy mode. Release your cursor
from dragging to place the copied object.
Where is this Command:
Access this command from Top 11
Commands>Duplicate
Steps:
## Edit Points On
‘EditPtOn’
Edit Points On turns on a little point object at
every point where the mouse was clicked while
drawing a curve. This allows the user to edit the
curve by dragging around the edit points to
change the shape of the curve.
• Select Duplicate from the Top 11 menu.
• Select Objects to copy.
• Press Enter when done.
• Pick a point to copy from.
• Pick a point to copy to. Continue picking points to
place multiple copies.
• If no more copies are desired, press Enter to close
the command.
### Command Line Options:
Where is this Command:
Access this command from Top 11
Commands>Edit Points On
Steps:
Vertical: The “Vertical” option changes the
direction of the command so that the copy of the
object is constrained in a vertical direction to the
Construction plane of the viewport in which the
object is planar. Therefore the object being copied
can only be placed above or below the original.
In Place: This will copy the object in the exact
same location as the original.
• Select Edit Points On from the Top 11 menu.
• Select curve(s) for Edit Point display.
• Press Enter when done.
• Press Escape key (ESC) to turn Edit Points off
when done.
### Options after the first copy:
From Last Point:
Yes: Uses the point the last object was copied to
as the base point.
No: Uses the first base point as location to copy
from.
Note:
Edit points are like Control Points except they
are always located on the curve. Moving one edit
point generally changes the shape of the entire
curve (moving one control point only changes
the shape of the curve in a sub region). Edit
Points are most useful when you need a point on
the interior of a curve to pass exactly through a
certain location. Control Point editing is preferred
when you need to change the look of a curve and
maintaining “fairness” is important.
Use Last Distance:
Yes: Places the next copy at the same distance
from the base point as the last copy.
## Control Points On
No: Pick a new distance for the next copy.
Use Last Direction:
Yes: Places the next copy at the same direction
from the base point as the last copy.
‘PointsOn’
The Control Points On command is similar to the
Edit Points On command, except that it also works
on surfaces. However, Control Points On may

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
