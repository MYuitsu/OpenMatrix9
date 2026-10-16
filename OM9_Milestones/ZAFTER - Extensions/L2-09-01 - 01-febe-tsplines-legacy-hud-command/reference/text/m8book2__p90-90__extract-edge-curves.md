# Extract Edge Curves ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 90-90
- Section title: Extract Edge Curves ￼
- Section id: m8book2-109-extract-edge-curves
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-109-extract-edge-curves | Extract Edge Curves ￼ | m8book2 pages 90-90 -->

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


---

## Images

- ![m8book2_p90_img01.png](../images/m8book2_p90_img01.png) page 90 (shared page)
- ![m8book2_p90_img02.png](../images/m8book2_p90_img02.png) page 90 (shared page)
- ![m8book2_p90_img03.png](../images/m8book2_p90_img03.png) page 90 (shared page)
