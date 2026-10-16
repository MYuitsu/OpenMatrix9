# Render Mesh

- Source PDF: Matrix 8 Instruction Manual Book 1 (h:\OpenMatrix9\manuals\matrix8_book_1.pdf)
- Page range: 76-77
- Section title: Render Mesh
- Section id: m8book1-060-render-mesh
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section m8book1-060-render-mesh | Render Mesh | m8book1 pages 76-77 -->

<!-- page 76 -->

Steps:
selected, click on the Gem View icon. Now, the
Construction plane with be returned to its
default location.
## Surface View
• Select the items for Shade Selected.
• Click on the Shade Selected tool to toggle the
display from Shaded to Wireframe on only the
selected items.
• When items are deselected they are no longer in
Wireframe Mode. (This also works in the reverse;
Wireframe objects become Shaded.)
Surface View reorients the Construction plane in
the planar viewports (Looking Down, Through
Finger, Side View) to a selected surface. This
works the same as Gem View to allow the user to
easily build on the selected surface.
Where is this Command:
Access this command from Display>Surface
View
Shade Selected Off
Shade Selected On
Steps:
## Gem View
• Select the Surface that you wish to reorient the
C-Plane.
• Click on the Surface View icon in the Display menu.
• The Construction plane is now reoriented around
an imaginary bounding box of the selected
surface.
Gem View reorients the Construction plane in the
planar viewports (Looking Down, Through
Finger, Side View) to a selected gem, placing the
gem’s girdle centered at 0,0,0 (F4). This enables
objects to be easily built on gems using Mirror and
other functions with an F4 start point.
Where is this Command:
Access this command from Display>Gem View
Steps:
Note:
• When in Surface View the ring icon indicating the
orientation of the viewports switched to a green
surface icon.
• Return to the default view - with nothing
selected, click on the Surface View icon. Now,
the Construction plane with be returned to its
default location.
## Render Mesh
• Select the gem to reorient the C-Plane.
• Click on the Gem View icon in the Display menu.
• The Construction plane is reoriented so the
selected gem is now located at 0,0,0 or F4.
Gem View Looking Down
Note:
• When in Gem View the ring icon
While in Shaded mode set the Render Mesh
to either Coarse, Medium, Fine or Super Fine
resolution. Render Mesh works as a “skin” over
the wireframe model to make it look shaded.
How tightly this mesh is created is determined
by its setting.  These settings also apply to the
rendered model. Choosing a Coarse Render
Mesh resolution helps move large models quicker
(models with tens or hundreds of parts and pieces
that are all Shaded). A finer setting creates better
renders.
indicating the orientation of the
viewports switches to a gem icon.
• Return to the default view - with nothing
Where is this tool:
Access this tool from Display>Render Mesh

<!-- page 77 -->

will move slower on the screen with this setting
using the max of the computer’s resources.
Note: Render Mesh can only be visible in a Shaded
or similar Display Mode in Matrix. Wireframe display
does not display the Render Mesh settings.
Steps:
## Shade Mode
Chose how the objects in the viewport are
displayed on the screen.
• Set the Display Mode to Shaded.
• Use the drop down menu to select a Render Mesh
resolution. Move items in your viewports quicker
(Coarse Mesh). Or, choose a finer setting while
rendering your design (Fine Mesh).
Where is this Command:
You can access this command from
Display>Shade Mode
Steps:
Mesh Density Settings are found in Rhino
Options>Document Properties>Mesh>Density.
See these settings change as the Render Mesh
changes from one setting to another.
• Select the desired Shade Mode from the drop
down menu.
• Click on the Shade On/Off button to toggle
between the selected Shade mode and
Wireframe.
### Wireframe:
Coarse Mesh: The Render Mesh density is 0.4.
Wireframe:
Displays the UV curves
and edges of a surface or
the vertices and edges of
the polygons of a mesh as
unshaded wires.
Coarse Render Mesh density setting
causes the most distortion of any of the
mesh modes (edges pull away and blend
instead of being crisp). This setting can be used
when shading hundreds of objects displayed on
your screen. It will use the least of your system’s
resources while shading objects permitting
movement of objects quicker in the viewports.
### Shade Modes:
Medium Mesh: The Render Mesh density is 0.6.
Medium Render Mesh density setting will
distort objects slightly. The edges are not
as clean and fine as Fine Mesh density. Use
this setting or the Coarse setting if lots of objects
are present.
Shaded:
Uses a mesh over the wires
with the Layer color to show
an opaque surface.
Fine Mesh: The Render Mesh density is 0.9. Fine
Render Mesh density displays designs
optimally without taxing your system
resources to the max. This setting also will
produce excellent VRay renders without any
noticeable distortion and can be used as your
default Render Mesh for renders.
Super Fine Mesh: The Render Mesh density is
1.0. This displays designs crisply and
accurately without any distortion. Models
Rendered:
Displays surfaces of objects
with the material that has
been applied to them either
from the VRay Material Editor
or the F6 menu.
