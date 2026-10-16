# Project To C-Plane ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 16-16
- Section title: Project To C-Plane ￼
- Section id: m8book2-012-project-to-c-plane
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p16_img01.png, m8book2_p16_img02.png, m8book2_p16_img03.png, m8book2_p16_img04.png

---
<!-- section m8book2-012-project-to-c-plane | Project To C-Plane ￼ | m8book2 pages 16-16 -->

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
