# Orient on Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 19-20
- Section title: Orient on Surface ￼
- Section id: m8book2-017-orient-on-surface

---

<!-- section m8book2-017-orient-on-surface | Orient on Surface ￼ | m8book2 pages 19-20 -->

<!-- page 19 -->

## Smart Flow Break
Curves pulled down from the Flow surface assist in
the design layout.
## Orient on Surface
‘gvSmartFlowBreak’
This disables Smart Flow on a Flow surface. The
command will not remove any existing geometry
from the Flow surface but the Flow surface will
not continue to update. Complex designs can
take longer to update on a Flow surface; disabling
Smart Flow temporally while laying out your
design can be more efficient.
‘OrientOnSrf’
Moves or copies and rotates objects on a
surface using the surface’s Normal direction for
orientation..
Where is this Command:
Access this command from Transform>Smart
Flow Break
Steps:
Where is this Command:
Access this command from Transform>Orient
on Surface
Steps:
• Select Smart Flow Break from the Transform
menu.
• Select the Flow surface to Break (disable Smart
Flow).
Note: To enable Smart Flow after it has been broken,
run the Smart Flow command and select the Base
surface. Then, select the Flow surface when directed.
## Smart Flow Pull Down Curves
• Select Orient on Surface from the Transform
menu.
• Select the Object(s) to Orient. (For best results,
the object should be relatively flat.)
• Press Enter.
• Pick two Reference Points. The first point is a Base
point. The second point is a Reference point for
Scaling.
‘gvSmartFlowPullDownCurves’
This pulls down curves from the Flow surface
to the Base surface. These curves can be used
as  points of reference when laying out designs.
This command currently only works with a new
Smart Flow command that is in testing. To use
the new Smart Flow command you must type
‘gvSmartFlowTest’ into the command line and
follow the Command Line prompts.
Turn on Project or Planar Snaps while choosing
these points. These points represent the center
(Base point) and “X” direction of the object
(choose a point beyond the object’s edge) to
assist during scaling and orientation. The “Z’
direction of the current Construction plane
determines the “up” direction for the object relative
to the surface. The ‘up’ direction will then follow
the Target surface Normal direction.
Where is this Command:
Access this command from Transform>Smart
Flow Pull Down Curves
Steps:
• Select Smart Flow Pull Down Curves from the
Transform menu.
• Select the Flow surface.
• Select curves or edges on the Flow surface to pull
down to the Base surface.
• Press Enter when done.

<!-- page 20 -->

Note: First Reference point selected for center (F4)
and second Reference point is made following the X+
Axis just beyond the object. (The second Reference
point can be made in any direction. However, the
direction selected will be the X+ direction the object
is placed on the surface.)
Example: Object is placed centered at F4 keeping
the bottom above the X Axis in the Through Finger
Viewport.
• Select the Surface on which to orient the object.
A dialog opens with options for Scale, Rotation or
Rigid.
Objects are placed using Orient on Surface with
Scale, Copy and Rotate turned ON.
### Orient on Surface Options:
### Scale:
Prompt: Prompts to pick a Scale for each copy.
Move your cursor (away from and towards the
object to scale it) and click to set the object’s
scale.
Uniform: Sets the same Scale for the X,Y, and Z
directions.
X/Y/Z: Sets a separate Scale for each direction.
### Rotation:
Prompt: Prompts to pick a
• Pick Target Point on the surface.
• With Copy ON continue to place the object. Press
ESC to when done.
Rotation Angle for each copy.
An Angle preview is shown
on the surface. Move the
cursor around the object and
click to set the Angle.
As the cursor is moved over the surface, a
dynamic preview image of the transformed object
is shown being reoriented by the varying normal
direction of the surface. See picture.
Angle: Specify an Angle of rotation. Rotation is
around the surface Normal at the Target point.
