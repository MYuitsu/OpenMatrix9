# Splop ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 30-31
- Section title: Splop ￼
- Section id: m8book2-031-splop
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p30_img01.png, m8book2_p31_img01.png, m8book2_p31_img02.png, m8book2_p31_img03.png, m8book2_p31_img04.png

---
<!-- section m8book2-031-splop | Splop ￼ | m8book2 pages 30-31 -->

<!-- page 30 -->

Line: Draw a line to be used as the Base curve.
Local:
Yes: Specify circles to define a ‘Tube’ of influence
around the Base curve. Everything inside the tube
is flowed. Everything outside the tube is fixed.
Inside the tube wall the flow dies off. This enables
part of the objects flowed to be deformed and
part is left alone.
No: All of the space is deformed (default).
Stretch:
No: The length of the objects flowed does not
change to match the Target curve. (Use when
Base curve and Target curves are different
lengths. The objects flowed remain the same
length.)
Yes: Objects are stretched or compressed so that
the relationship between the Target curve and the
Base curve is the same.
Objects placed on a Base surface are flowed to a
Target Surface.
## Flow Along Surface
### Command Line Options:
This command flows objects from a Base
surface to a Target surface.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
Where is this Command:
Access this command from Transform>Flow
Along Surface
Steps:
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed.
Plane: Draw a plane instead of using an existing
surface Base surface. See the Rectangle
command for option descriptions.
• Select Flow Along Surface from the Transform
menu.
• Select the Object(s) to Flow along a surface.
• Press Enter when done.
• Select the Base surface near a corner.
• Select the Target surface near a matching corner.
## Splop
‘Splop’
Copies, rotates, scales, and wraps objects on a
surface using a reference sphere
Where is this Command:
Access this command from Transform>Splop
Steps:
• Select Splop from the Transform menu.
• Select object(s) to Splop.
Creating a Base Surface: In advance, select
the Target surface and use “ShrinkTrimmedSrf’ (if
the surface has been previously trimmed) and use
‘Create UV Curves” to create curves for making
the Base. Then, use Sweep or ‘Planar Curves’ to
build a Base surface.
History: Flow Along Surface is also a Rhino
History enabled command. So, have History
recording and Update turned ON prior to running
the command. Then, after objects on flowed onto
the Target surface they can be scaled, moved or
transformed and will update accordingly.

<!-- page 31 -->

• Press Enter when done.
• Draw a Sphere around the
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed.
Flip: Reverses the direction of the object used.
## Set Points
input object(s) selecting
either the center of the
Sphere or another Sphere
option. (See Solid Menu
-Sphere)
• Continue drawing the Sphere by entering a Radius
or choose another option. Or, just drag and click
to complete the Sphere.
• Select a surface to Splop on.
‘SetPt’
This command is often used to accurately change
the position of points and control points. Objects
are moved to a specified location in the X, Y and/
or Z directions. Or, use to verify that a set of planar
curves are all in the same plane.
• Pick a Splop drop point
to place your object(s)
on the surface. (Drag out
a reference Sphere that
determines the object’s size.
Move the mouse to rotate
the object into a desired
orientation.)
• Continue placing objects with Copy ON.
• Press Enter when done.
Where is this Command:
Access this command from Transform> Set
Points
Steps:
• Select Set Points from the Transform menu.
• Select objects.
• In the Set Points dialog, click the check-boxes for
the coordinates.
• Pick a location to match. The selected objects
will line up with the set point in the direction or
directions you choose.
Notes: To move in only one direction, clear the check-
boxes for the other directions. To set one check and
clear the others, right-click the checkbox.
Objects are scaled, rotated and copied onto a
surface using Splop.
### Command Line Options:
### Command Line Options:
Set X/Set Y/Set Z: Align objects in the X, Y, and
Z directions.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
Align to World/Align to C-Plane: Aligns
using World or Construction plane coordinates.
The ‘Align to World’ option determines the
location along the axis on which to set points.
Determine this location by eye, or enter a value
into the Command Line using world coordinates.
The ‘Align to C-Plane” command sets the points
flat against the C-Plane in whatever axis you
choose.
