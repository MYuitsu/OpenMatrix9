# Curve and surface direction

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 93-94
- Section title: Curve and surface direction
- Section id: rhino5-057-curve-and-surface-direction
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p93_img01.png, rhino5_p93_img02.png, rhino5_p94_img01.png, rhino5_p94_img02.png

---
<!-- section rhino5-057-curve-and-surface-direction | Curve and surface direction | rhino5 pages 93-94 -->

<!-- page 93 -->

## Curve and Surface Analysis
Since Rhino is a mathematically accurate NURBS modeler, tools that provide accurate information about the
objects are provided.
### Measure distance, angle, and radius
Some analysis commands provide information about location, distance, angle between lines, and radius of a
curve. For example:
●
Distance displays the distance between two points.
●
Angle displays the angle between two lines.
●
Radius displays the radius of a curve at any point along it.
●
Length displays the length of a curve.
### Curve and surface direction
Curves and surfaces have a direction. Many commands that use direction information display direction arrows
and give you the opportunity to change (flip) the direction.
The Dir command displays the direction of a curve or surface and lets you change the direction.
The illustration shows the curve direction arrows. If the direction has not been changed, it reflects the direction
the curve was originally drawn. The arrows point from the start of the curve toward the end of the curve.
The Dir command also displays surface u-, v-, and normal direction. Surface normals are represented by
arrows perpendicular to the surface, and the u- and v-directions are indicated by arrows pointing along the
surface. Closed surfaces always have the surface normals pointing to the exterior.
The Dir command can change the u-, v-, and normal-directions of a surface. This direction can be important if
you are applying textures to the surface.

<!-- page 94 -->

Curve and Surface Analysis
### Curvature
Curve analysis tools let you turn on a graph showing the direction perpendicular to the curve at a point and the
amount of curvature, display a curvature circle, test the continuity between two curves and the intervals of
overlap between the two curves.
The CurvatureGraph command displays a curvature graph on curves and surfaces. The lines on the graph
represent a direction perpendicular to the curve at that point. The length of the line indicates the curvature.
### Visual surface analysis
Visual surface analysis commands let you examine surfaces to determine smoothness as determined by its
curvature, tangency, or other surface properties. These commands use NURBS surface evaluation and
rendering techniques to help you visually analyze surface smoothness with false color or reflection maps so you
can see the curvature and breaks in the surface.
### Environment map
The EMap command displays a bitmap on the object so it looks like a scene is being reflected by a highly
polished metal. This tool helps you find surface defects and validate your design intent.
The fluorescent tube environment map simulates tube lights shining on a reflective metal surface.
