---
source_pdf: h:\OpenMatrix9\manuals\rhino5.pdf
source_key: rhino5
page_range: 70-72
section_title: Curve and surface degree
section_id: rhino5-048-curve-and-surface-degree
---
# Curve and surface degree

Source: rhino5.pdf, pages 70-72

<!-- section rhino5-048-curve-and-surface-degree | Curve and surface degree | rhino5 pages 70-72 -->

<!-- page 70 -->

Edit Curves and Surfaces
Control points of polysurfaces cannot be turned on for editing. Editing the control points of polysurfaces could
separate the edges of the joined surfaces creating “leaks” in the polysurface.
### Change control point locations
When you move control points, the curve or surface changes, and Rhino smoothly redraws it. The curve or
surface is not drawn though the control points rather it is attracted to the new positions of the control point.
This allows the object to be smoothly deformed. When control points are on, Rhino’s transform commands can
manipulate the points. You can also rebuild surfaces to add control points and redistribute them.
### Add, delete, or redistribute control points
Adding control points to a curve gives you more control over the shape of the curve. Manipulating control
points also lets you remove kinks, make curves uniform, and add or subtract detail. The Delete key erases
curve control points. This changes the shape of the curve.
### Curve and surface degree
A polynomial is a function like y = 3x3 –2x +1. The "degree" of the polynomial is the largest power of the
variable. For example, the degree of 3x3 –2x + 1 is 3; the degree of –x5 + x2 is 5, and so on. NURBS functions
are rational polynomials and the degree of the NURBS is the degree of the polynomial. From a NURBS modeling
point of view, the (degree –1) is the maximum number of "bends" you can get in each span.
For example:
A degree-1 curve must have at least two control points.
A line has degree less than 1. It has zero bends.

<!-- page 71 -->

Edit Curves and Surfaces
A degree-2 curve must have at least three control points.
A parabola, hyperbola, arc, and circle (conic section curves) have degree less than 2. They have one bend.
A degree-3 curve must have at least four control points.
A cubic Bézier has degree less than 3 If you arrange its control points in a zig-zag shape, you can get two bends.

<!-- page 72 -->
