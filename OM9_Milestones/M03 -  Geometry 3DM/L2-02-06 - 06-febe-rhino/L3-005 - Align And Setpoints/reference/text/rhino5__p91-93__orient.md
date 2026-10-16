# Orient

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 91-93
- Section title: Orient
- Section id: rhino5-055-orient
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p93_img01.png, rhino5_p93_img02.png

---
<!-- section rhino5-055-orient | Orient | rhino5 pages 91-93 -->

<!-- page 91 -->

Transforms - Move, Copy, Rotate, Scale
### Array
The Array commands copy objects into evenly spaced rows and columns or around a circle.
### Orient
The Orient commands combine move or copy, scale, and rotate operations to help you position and size
objects in one command.

<!-- page 92 -->

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
