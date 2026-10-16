# Surfaces

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 26-29
- Section title: Surfaces
- Section id: rhino5-016-surfaces
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p26_img01.png, rhino5_p26_img02.png, rhino5_p27_img01.png, rhino5_p27_img02.png, rhino5_p27_img03.png, rhino5_p28_img01.png, rhino5_p28_img02.png, rhino5_p28_img03.png, rhino5_p29_img01.png, rhino5_p29_img02.png, rhino5_p29_img03.png

---
<!-- section rhino5-016-surfaces | Surfaces | rhino5 pages 26-29 -->

<!-- page 26 -->

Rhino Objects
### Curves
A Rhino curve is similar to a piece of wire. It can be straight or wiggled, and can be open or closed.
A polycurve is several curve segments joined together end to end.
Rhino provides many tools for drawing curves. You can draw straight lines, polylines that consist of connected
line segments, arcs, circles, polygons, ellipses, helices, and spirals.
You can also draw curves using curve control points and draw curves that pass through selected points.
Curves in Rhino include lines, arcs, circles, free-form curves, and combinations of these. Curves can be open or
closed, planar or non-planar.
### Surfaces
A surface is like a rectangular stretchy rubber sheet. The NURBS form can represent simple shapes, such as
planes and cylinders, as well as free-form, sculptured surfaces.
All surface creation commands in Rhino result in the same object: a NURBS surface. Rhino has many tools for
constructing surfaces directly or from existing curves.
All NURBS surfaces have an inherently rectangular organization.

<!-- page 27 -->

Rhino Objects
Even a closed surface such as a cylinder is like a rectangular piece of paper that has been rolled up so two
opposite edges are touching. The place where the edges come together is called the seam. If a surface does not
have a rectangular shape, either it has been trimmed or the control points on the edges have been moved.
### Closed and open surfaces
A surface can be open or closed. An open cylinder is closed in one direction.
A torus (donut shape) is closed in two directions.
### Trimmed and untrimmed surfaces
Surfaces can be trimmed or untrimmed. A trimmed surface has two parts: a surface that underlies everything
and defines the geometric shape, and trimming curves that mark sections of the underlying surface that are
trimmed away.
Trimmed surfaces are created with commands that trim or split surfaces with curves and other surfaces. Some
commands create trimmed surfaces directly.

<!-- page 28 -->

Rhino Objects
The shape of a surface is still defined by a set of control points arranged in a rectangular pattern.
Since it can be important for you to know if a surface is trimmed, the Properties command lists the trimmed
or untrimmed state of the surface. Some Rhino commands work only with untrimmed surfaces and some
software does not import trimmed NURBS surfaces.
Trimming curves lie on the underlying surface. This surface may be larger than the trim curves, but you will
not see the underlying surface because Rhino does not draw the part of the surface that is outside the trim
curves. Every trimmed surface retains information about its underlying surface geometry. You can remove the
trimming curve boundaries to make the surface untrimmed with the Untrim command.
If you have a trim curve that runs across a surface, the trim curve itself does not have any real relationship to
the control point structure of the surface. You can see this if you select such a trimmed surface and turn its
control points on. You will see the control points for the whole underlying surface.

<!-- page 29 -->

Rhino Objects
If you create a surface from a planar curve, it can be a trimmed surface. The illustrated surface was created
from a circle. The control points display shows the rectangular structure of the surface.
The Untrim command removes the trimming curve from the surface to get back to the underlying untrimmed
rectangular surface.
### Surface isoparametric and edge curves
In wireframe view, surfaces look like a set of crossing curves. These curves are called isoparametric curves or
isocurve. These curves help you visualize the shape of the surface. Isoparametric curves do not define the
surface the way the polygons do in a polygon mesh. They are merely a visual aid that allows you to see the
surface on the screen. When a surface is selected, all of its isoparametric curves highlight.
Edge curves bound the surface. Surface edge curves can be used as input to other commands.
Isocurves (1), Edge curves (2).
