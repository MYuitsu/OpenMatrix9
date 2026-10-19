# Array Polar ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 15-16
- Section title: Array Polar ￼
- Section id: m8book2-011-array-polar
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p15_img01.png, m8book2_p15_img02.png, m8book2_p15_img03.png, m8book2_p15_img04.png, m8book2_p15_img05.png, m8book2_p16_img01.png, m8book2_p16_img02.png, m8book2_p16_img03.png, m8book2_p16_img04.png

---
<!-- section m8book2-011-array-polar | Array Polar ￼ | m8book2 pages 15-16 -->

<!-- page 15 -->

## Array Polar
Example: Start of Spine is the middle of the object.
End of Spine is the right end.
‘Array Polar’
This tool allows the user to Copy an object a
number of times around a central point. The
object may be a point object, curve, surface, or
polysurface. The copied objects will radiate out
from the central point chosen by the user in up
to a 360 degree revolution. The user may specify
how many objects can be arrayed, as well as how
many degrees, up to 360°, around the circle the
array will fill. Then, the number of objects selected
will be arranged evenly to fill the space indicated
by the user.
### Command Line Options:
Where is this Command:
Access this command from Transform>Array
Polar
Rigid Yes/ No: Yes: Individual objects are not
deformed as they are transformed. No: Individual
selected objects are deformed.
Right Click Command: Array Polar F4
Sets F4 (0,0,0) as the center of the polar array.
Steps:
LimitToSpine Yes/ No: Yes: only the ‘Spine’
region is bent. No: The mouse click point
determines the region to Bend.
Angle: Enter an angle or amount to set the Bend
amount.
Symmetric Yes/ No: Yes: start the Bend Spine
in the middle of the object to Bend symmetrically
around the center. No: Bends only one end of the
object.
• Select Array Polar from the Transform menu.
• Select the Object(s) to Polar Array.
• Press Enter.
• Pick a point for the center of the Polar Array.
• Type the Number of items and press Enter. The
number must equal 2 or more.
• Type the Angle to fill in degrees. (360 will array the
number of items in a full circle, 180 a half circle
etc.)
• A preview of the Polar Array will be displayed.
Press Enter to accept, or select an option to
change the parameters.
Example: Start of Spine is middle of object. End of
Spine is the right End. Symmetric = Yes.
PreserveStructure=Yes: This preserves the
Control Point structure of the surface. Deformation
may be less accurate if there are too few control
points on the object.
PreserveStructure=No: Objects are refit as
needed with more control points to allow accurate
deformation.
Example: Sphere is Polar Arrayed around F4, 360°
with a count of 12.

<!-- page 16 -->

### Command Line Options:
Preview=Yes: Displays a dynamic preview. You
can change the options at this point and the
preview will update.
Preview=No: No preview will display. There is no
opportunity to change the options.
Preview = Yes
Select the Profile curve in Side viewport to Project
to the Construction Plane. Before (on the left; non-
planar curve) and After (on the right; planar curve)
Project to C-Plane.
StepAngle: Enter the Angle between the
objects.
Rotate: Rotates the objects as they are Arrayed.
Objects stay oriented to the center point of the
Array.
ZOffset: Moves each arrayed object in the Z
axis direction, by the value specified. Each object
builds from the previous object.
## Smart Flow
## Project To C-Plane
‘ProjectToCplane’
Flattens object onto the Construction Plane.
‘gvSmartFlow’
This Matrix History-enabled command morphs
objects from a Base surface to a Destination
surface. This command allows you to layout
designs flat and have them ‘flow’ to a Destination
surface.
Where is this Command:
Access this command from Transform>Project
to C-Plane
Steps:
Where is this Command:
Access this command from Transform>Smart
Flow
### Smart Flow with Autobase:
• Select Project to C-Plane from the Transform
menu.
• Select Object(s) to Project. (Click in the Viewport
for the Construction Plane on which to project the
object.)
• Select any Options in the Command Line.
• Press Enter.
Note: The projected object has the same control
point structure as the original object, except that all
the control points are projected to the construction
plane. This command works on points, curves,
surfaces, polysurfaces, and meshes.
This command option is available when Smart
Flow command is started with a single surface
selected (Target or Destination surface). A Base
surface is created using Create UV Curves and
Sweep all at once . This Base surface represents
the surface size and shape of the Destination
surface. It is also the surface you place objects on
to flow to the Destination surface.
