# Split Curves ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 90-91
- Section title: Split Curves ￼
- Section id: m8book2-111-split-curves
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-111-split-curves | Split Curves ￼ | m8book2 pages 90-91 -->

<!-- page 90 -->

## Retopology Snap Options
Note: In the Display menu use the ‘Detect Backfaces’
view to display Matrix and T-Spline backfaces as red.
‘tsEditMode’ RetopoSnap
Retopology Snap will snap T-Splines control
points to the closest mesh or polysurface in the
scene for the following commands: Append,
Extrude, Insert Edge, Insert Point, Subdivide Face,
Bevel, and Slide.
Where is this Command:
Access this command from T-Spline>
Retopology Snap Options
A T-Spline surface inside out (left) and after using
Flip Normals (right) in Detect Backfaces Display
mode.
### Options:
## Extract Edge Curves
RetopologySnap=Yes/No: Toggles ‘ON’ or
‘OFF’ whether T-Splines control points will snap to
polysurfaces or meshes in the scene.
‘tsExtractEdges’
Extracts the selected edges from a T-Spline
surface. The resulting geometry is a curve. This
command works similarly to Extract Isocurve with
NURBS but on T-Spline models.
Note: The following options are only available
when Retopology Snap is set to ‘Yes’.
Snapin: Controls how the T-Spline control
points will snap to an object based on the World
coordinates or the active View.
Where is this Command:
Access this command from T-Spline>Extract
Edge Curves
Steps:
RetopoOffset: The offset distance from the
mesh that each point will snap to. If 0, the control
points will snap right on the mesh.
## Split Curves
• Select Extract Edge Curves from the T-Spline
menu.
• Select the Edge curve you wish to extract from the
T-Spline object.
• Press Enter when done.
### Options:
‘tsSplitCurves’
Splits a network of intersecting curves into curve
segments.
Offset: Pick a point along a perpendicular edge
to select an offset curve. Or, enter a percentage
between -1.0 and 1.0. Press Enter when done.
Where is this Command:
Access this command from T-Spline>Split
Curves
Steps:
BothSides: This option is available when Offset
is chosen. This will offset a curve equal distance
on both sides of the original T-Spline edge.
• Select Split Curves from the T-Splines menu.
• Select curves to Split.
• Use File Tolerance or uncheck and enter a custom
value. See below.
• Press ‘OK’ when
done.
Extract Edge Curve is used
with Offset and Both Sides to
produce two curves from one
edge.

<!-- page 91 -->

### Options:
Use File Tolerance = On/Off: The Split Curves
command has a tolerance built in to help create a
split between curves that do not actually intersect.
Split Curves uses Rhino Properties Units settings
when defaulted to ‘ON’’ . If you wish to select your
own tolerance turn Off the File Tolerance and
enter a value in the provided box.
This overrides Rhino Properties unit tolerance and
uses the value entered.
Tolerance Value: When File Tolerance is turned
‘Off” (unchecked) enter a tolerance value to be
used during this instance of the Skin command.
Set the Display to
‘Rendered’ to view the
applied texture on the
T-Splines object in the
viewport. Or, render the
T-Splines object to get
the full effect.
Split Curves before (left) and after (right). Curves are
split at every intersection.
## Edit UVs
Rendered T-Spline object with applied texture
material.
Note: It’s always a good idea to save your design
before editing. So, place your design in a job bag
before beginning the next steps.
Edit UVs has three operations:
‘tsUVEditor’
Edit the UV layout of textures applied to T-Splines
and T-Spline render meshes. The Editor gives
control over how the texture wraps across the
whole T-Spline model instead of just being
repeated like a checkerboard.
• Set Texture Seams
• Create Texture Mesh
• Texture Mesh Live Points On
Where is this Command:
Access this command from T-Spline>Edit UVs
### Set Texture Seams:
Before Using Edit UVs:
A texture must be applied first to a T-Spline object
before using the Edit UVs command.
This operation lets you select T-Spline edges to
set the texture seams. This is where the texture
mesh will be “cut” so it can lay flat on the texture.
There is a ‘FromExisting’ option. If your model
already has a nice UV layout (for instance, if
you laid out the UVs/texture seams in another
program, or if you converted from a NURBS,
Open VRay Styles (Render Menu>VRay Styles),
go to Material Editor>Metal Materials>Texture. Use
the drop down arrow under ‘Texture Presets” to
select a Texture render material. Apply this to your
T-Spline object. (See Render Menu>VRay Styles
for detailed information.)


---

## Images

- ![m8book2_p90_img01.png](../images/m8book2_p90_img01.png) page 90 (shared page)
- ![m8book2_p90_img02.png](../images/m8book2_p90_img02.png) page 90 (shared page)
- ![m8book2_p90_img03.png](../images/m8book2_p90_img03.png) page 90 (shared page)
- ![m8book2_p91_img01.png](../images/m8book2_p91_img01.png) page 91 (shared page)
- ![m8book2_p91_img02.png](../images/m8book2_p91_img02.png) page 91 (shared page)
- ![m8book2_p91_img03.png](../images/m8book2_p91_img03.png) page 91 (shared page)
- ![m8book2_p91_img04.png](../images/m8book2_p91_img04.png) page 91 (shared page)
- ![m8book2_p91_img05.png](../images/m8book2_p91_img05.png) page 91 (shared page)
- ![m8book2_p91_img06.png](../images/m8book2_p91_img06.png) page 91 (shared page)
