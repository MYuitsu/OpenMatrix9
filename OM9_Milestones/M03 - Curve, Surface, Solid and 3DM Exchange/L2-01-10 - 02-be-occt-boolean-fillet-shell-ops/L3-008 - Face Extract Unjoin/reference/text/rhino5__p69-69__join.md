# Join

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 69-69
- Section title: Join
- Section id: rhino5-044-join
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p69_img01.png

---
<!-- section rhino5-044-join | Join | rhino5 pages 69-69 -->

<!-- page 69 -->

## Edit Curves and Surfaces
The editing operations in this section break objects apart, cut holes in them, and put them back together.
Some of these commands connect curves to curves or surfaces to surfaces or polysurfaces and break a
composite curve or polysurface into its components.
The commands: Join, Explode, Trim, and Split apply to curves, surfaces, and polysurfaces.
The Rebuild, ChangeDegree, and Smooth commands alter the shape of a curve or surface by changing its
underlying control point structure.
In addition, objects have properties that are assigned to them such as color, layer, rendering material, and
other attributes depending on the object. The Properties command manages these properties.
### Join
The Join command connects curves or surfaces together into one object. For example, a polycurve can consist
of straight-line segments, arcs, polylines, and free-form curves. The Join command also connects adjacent
surfaces into a polysurface.
### Explode
The Explode command removes the connection between joined curves or surfaces. For polysurfaces, this is
useful if you want to edit each individual surface with control points.
### Trim and
### Split
The Trim and Split commands are similar. The difference is when you trim an object, you select the parts to
remove and they are deleted. When you split an object, all parts are left.
The Split command will split a surface with a curve, surface, polysurface, or its own isoparametric curves.
The Untrim command removes a surface’s trimming curve, with an option to keep the curve so you can re-
use it.
### Control-point editing
You can make subtle changes in the shape of a curve or surface by moving the location of its control points.
Rhino offers many tools for editing control points. Some commands such as Rebuild, Fair, and Smooth offer
some automated solutions for redistributing control points over a curve or surface. Other commands, such as
control point dragging and nudging, MoveUVN, and HBar let you manually control the location of individual or
groups of control points.
### Control point visibility
To edit curves and surfaces by manipulating control points, use the PointsOn command to turn the control
points on.
When you are finished with control-point editing, use the PointsOff command or press Esc to turn them off.
