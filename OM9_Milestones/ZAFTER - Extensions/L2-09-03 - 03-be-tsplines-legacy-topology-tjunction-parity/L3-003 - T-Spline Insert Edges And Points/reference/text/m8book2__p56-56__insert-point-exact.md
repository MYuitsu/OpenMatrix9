# Insert Point Exact ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 56-56
- Section title: Insert Point Exact ￼
- Section id: m8book2-069-insert-point-exact
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-069-insert-point-exact | Insert Point Exact ￼ | m8book2 pages 56-56 -->

<!-- page 56 -->

## Insert Point Exact
Where is this Command:
Access this command from T-Spline>Thicken
Steps:
‘tsInsertPointExact’
Exact insertion will not change the surface shape.
This can only be performed in Smooth mode, and
it may add more control points than you desire,
as well as shift existing control points to keep the
surface from changing.
Where is this Command:
Access this command from T-Spline>Insert
Point Exact
• Begin with a T-Spline surface. This object can be
a two-dimensional or three-dimensional T-Spline
surface or object.
• Click the T-Splines Thicken command-and select
the T-Splines object.
• Press Enter when done.
• Move your cursor above or below the point that
appears on screen to determine the thickness of
the T-Spline object.
Note: The Insert Point command can destroy
the surface of the model. Avoid creating star
points, triangles and N-sided faces. If the model
automatically toggles into Box Mode and cannot
be smoothed, this is an indication that the insertion
resulted in a bad surface.
Steps:
Thickness is added to a plane. Specify the Thickness
by moving the cursor up or down.
• Select the Insert Point Exact command from the
T-Spline menu.
• Select an edge to insert a point.
• Click to set control point locations.
• Press Enter when done.
### Command Line Options:
Note: It is possible to create bad, overlapping
geometry when using the Thicken command. This
will happen if the minimum curvature of the model
is smaller than the thickness being applied. It is
recommended to examine the resulting surface for
self intersections. Keep in mind, if you know the
exact thickness for the finished surface, simply type
this value into the command and press Enter. The
resulting surface thickness is as close as possible to
the value entered, but the thickness may slightly vary
in certain areas of the design.
Insertion Type: Simple/Exact: Inserting a
point will try to keep the surface of the model as
simple as possible but the model surface will be
affected. Insert Point Exact will add a point while
maintaining the surface but the model will likely
become more complicated.
### Command Line Options:
RetopoSnap: Turns on and off RetopoSnap.
RetopoOffset: The offset distance from the
mesh that each point will snap to. If 0, the control
points will snap right on the mesh.
Direction Type:
PerVertexNormal: “PerVertex Normal” option
locates every vertex on the surface and thickens
the surface perpendicular or “normal” to each
vertex, creating a shell of the object.
## Thicken
Normal: Averages the direction of the normals
and settles on one direction to thicken towards.
‘tsThicken’
This powerful command adds thickness to a
T-Spline object, allowing you to easily create
three-dimensional objects and even hollow
T-Spline surfaces.
Crease Edges = Yes/No: The “CreaseEdges”
option is a Yes/No toggle that either creases the
edges of the thickened surface or leaves the
surface edges curved and soft.


---

## Images

- ![m8book2_p56_img01.png](../images/m8book2_p56_img01.png) page 56 (shared page)
