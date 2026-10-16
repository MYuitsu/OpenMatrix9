# Smart Flow ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 16-18
- Section title: Smart Flow ￼
- Section id: m8book2-013-smart-flow
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p16_img01.png, m8book2_p16_img02.png, m8book2_p16_img03.png, m8book2_p16_img04.png, m8book2_p17_img01.png, m8book2_p9_img02.png, m8book2_p17_img02.png, m8book2_p17_img03.png, m8book2_p18_img01.png, m8book2_p18_img02.png, m8book2_p18_img03.png

---
<!-- section m8book2-013-smart-flow | Smart Flow ￼ | m8book2 pages 16-18 -->

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

<!-- page 17 -->

Steps (Autobase):
### Command Line Options:
• Select a single Surface for the Flow Surface.
• Select Smart Flow from the Transform menu.
• Click on the “Autobase” option in the Command
Line.
Clear: Disables the relationship between the
Base surface and Destination surface. Any further
changes to the Base surface won’t ‘flow’ to the
Destination surface.
BoundingBoxAddX: Controls the distance an
object can hang off the Base surface on the X
axis and still flow to the Destination surface.
The base surface will appear in the appear in the
viewport on the active layer color.
• Place objects on the Base surface and they will
appear on the Destination Surface.
Note: When using Autobase, the relationship
between the Flow surface and the Base surface will
already be established.
Note: Matrix History must be turned ON before
running the Smart Flow command.
BoundingBoxAddY: Controls the distance an
object can hang off the Base surface on the Y axis
and still flow to the Destination surface.
Tolerance: This represents the amount of
variance or difference allowed between an
object on the Base after it flows to the Target
surface. Lowering the value makes the object
more complex, makes it true to the original, and
raising the value allows the object to morph or be
distorted in a greater way from the original Base
surface object. Keeping the value at its default will
work for nearly everything. A good value range is
minimum 0.01 to 0.25.
Autobase places a Base surface (green) flat in
Looking Down viewport.
Around=Yes/No: When Around is set to ‘Yes’
the objects that ‘flow’ to the Destination surface
will overhang the Destination surface if the objects
on the Base surface overhang that surface. When
around is set to no Matrix tries to constrain the
flowed objects to the Destination surface.
Place objects on the Base surface (green) and they
will appear on the Destination surface (orange).
### Smart Flow Without Autobase:
Use when an existing Base surface is already
present.
Steps:
• Select Smart Flow from the Transform menu.
• Select the Base surface.
• Select the Destination surface.
• Place objects on the Base surface and they will
appear on the Destination surface.
Around=Yes on the left: Flowed objects overhang the
destination surface in the same manner they do on
the Destination surface. Around=No Right: Flowed
objects are constrained to the Destination surface.
Curves bend in order to stay within the surface.

<!-- page 18 -->

### Related Commands:
## Smart Flow Rigid
These are a list of tools that can assist you when
working with Smart Flow.
‘GvSmartFlowRigid’
This toggles a Flow object to a Rigid state. Smart
Flow Rigid finds the center of an object and flows
the object to the Flow surface based on that
point without distorting the object. Use when the
objects on the Base surface should not distort
on the Target surface. By default, gems (block
instances) flow to the Target surface in a rigid
state.
Where is this Command:
Access this command from Transform>Smart
Flow Rigid
Steps:
• Select Smart Flow Rigid from the Transform menu.
• Select objects on the Base surface (flat surface) to
change to a Rigid state.
Direction ‘Dir’: When using Smart Flow, objects
‘flow’ to the Destination surface based on the
direction of the Base and Destination surfaces. If
the relationship between UV and Surface Normal
don’t match between the Base and Destination
surface objects will flow to the  Destination
surface with unpredictable and often undesirable
results. Use the Direction command to align the
Destination surface to the Base surface. See the
Utilities section of the manual for more information
on how to use the Direction command.
Before Smart Flow Rigid the bezels on the Flow
surface are distorted and break away from the
girdles of the gems.
Detect Smart Flow: This shade mode will
display Surface Normal direction as well as UV
direction on surfaces. Surface Normals that point
down (a Backface) will display red otherwise
they display by their layer color. The U direction
is indicated by red lines on the surface and the V
direction by blue.
In this example the UV and directions are aligned
between the Base and Destination surfaces, but
the Destination’s Surface Normal is flipped. Objects
added to the top of the base surface will appear on
the under side of the destination surface.
After Smart Flow Rigid the bezels on the Flow
surface are no longer distorted and follow the Base
surface bezels.
