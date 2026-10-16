# Duplicate

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 82-82
- Section title: Duplicate
- Section id: m8book1-063-duplicate
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---
<!-- section m8book1-063-duplicate | Duplicate | m8book1 pages 82-82 -->

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
