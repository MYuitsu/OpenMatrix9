# Shade Mode

- Source PDF: Matrix 8 Instruction Manual Book 1 (h:\OpenMatrix9\manuals\matrix8_book_1.pdf)
- Page range: 77-80
- Section title: Shade Mode
- Section id: m8book1-061-shade-mode
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section m8book1-061-shade-mode | Shade Mode | m8book1 pages 77-80 -->

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

<!-- page 78 -->

Art Color:
Displays a simplified surface
with a heavy outline on a
white background. Curves and
the grid are not shown.
Ghosted:
Displays a surface that is
semi-transparent while being
able to see through it as in
Wireframe.
Art Color Wires:
This is identical to Art Color
but the curves and the grid
are displayed.
X-Ray:
This is similar to Ghosted
and shows a less transparent
surface. X-Ray is slightly less
transparent than Ghosted.
Chalk:
Displays the model as drawn
on a chalk board and features
shading lines while showing
curves.
Technical:
Shows opaque, solid colors
like Shaded but on a white
background with heavy black
wire outlines.
Artistic:
Displays the design as a
sketch, drawn on a textured
paper.
Detect All:
This shade mode incorporates
all of the diagnostic shade
modes named “Detect......”.
Pen:
This mode is comparable to
Artistic, except with a lighter
background and less shading.
Detect Backface:
Shows surfaces with normals
that face out as green and as
red if the normals are inverted,
facing inwards.

<!-- page 79 -->

Detect Naked Edges:
Highlights any surface edges
that are unjoined or naked.
Legacy:
Displays model as a schematic
illustration - an outline sketch
with shade lines on a light
background.
Detect Smart Flow:
Indicates Smart Flow surfaces
in the color red.
Machine:
Shows the model as a solid
color without wires, curves or
any outlines.
Matrix Blue:
Displays model as a schematic
illustration like Legacy except
using a blue outline with white
on a blue background.
Detect UV Normals:
Displays the direction of
surface normals: Red for
normals pointed outwards and
green for those inwards.
Floorplan:
Displays the model as a
blueprint with a white, outline
sketch on a blue background.
Matrix White:
Displays as a schematic
illustration like Legacy but
using a black outline with
white on a black background.
Ice:
Shows the model as semi-
transparent in light gray tones
with a gray background and
no curves.
Pastel:
Displays the design with a
black outline showing Layer
colors in pastel hues on a light,
textured background.

<!-- page 80 -->

Vivid:
Displays opaque surfaces with
dark edge curves on a white
grid. Curves are also visible.
Plastic:
Shows shiny, solid Layer colors
with dark edge curves. This is
good for displaying meshes
with no wires.
Presentation:
Displays surfaces of objects
with the Render materials
applied and no visible curves
or grid.
Wire Render:
This is comparable to
Rendered but displays the
objects’ Layer colors in outlines
showing  isocurves, edge and
construction curves.
Shiny Plastic:
This mode is similar to Plastic
but has a high gloss finish. It
also shows backfaces in red.
Working Render:
This is comparable to Wire
Render without displaying
isocurves.
Working Shade:
This is a high gloss version
of Shiny Plastic. It displays
backfaces as gray.
Simple Shade:
This mode bypasses the
Layer colors and displays the
objects as opaque, gray tones
on a white background with
no curves.
TsShiny:
This mode is akin to Shaded
but slightly less shiny.
Tech White:
This mode is identical to
Simple Shade but also
displays the isocurves and
edge curves.
