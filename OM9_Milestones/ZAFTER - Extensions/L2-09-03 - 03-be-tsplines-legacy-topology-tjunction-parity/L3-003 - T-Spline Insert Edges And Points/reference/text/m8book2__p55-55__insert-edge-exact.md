# Insert Edge Exact ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 55-55
- Section title: Insert Edge Exact ￼
- Section id: m8book2-067-insert-edge-exact
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-067-insert-edge-exact | Insert Edge Exact ￼ | m8book2 pages 55-55 -->

<!-- page 55 -->

## Insert Point Simple
Percentage: The edge will be inserted this far
(enter a number between 0 and 1 as a percentage)
toward the neighboring face.
Distance: The edge will be inserted this far away
as an absolute distance from the selected edge.
To insert in the other direction, type a negative
number. (Note: the absolute distance is calculated
in Box mode, not Smooth mode).
‘tsInsertPointSimple’
Allows you to insert points on edges. It also allows
you to extend existing control points. To use the
command, click on the edges where you would
like to add control points. Simple insertion can
be used both in Smooth and Box mode. Adding
a point with simple insertion will not move the
location of any other points; however, the surface
shape will likely change.
RetopoSnap: Turns on and off RetopoSnap.
## Insert Edge Exact
Where is this Command:
Access this command from T-Spline>Insert
Point Simple
Note: The Insert Point command can destroy
the surface of the model. Avoid creating star
points, triangles and N-sided faces. If the model
automatically toggles into Box Mode and cannot
be smoothed, this is an indication that the insertion
resulted in a bad surface.
‘tsInsertEdgeExact’
Insert Edge Exact will not change the surface
shape. This can only be performed in Smooth
mode, and it may add more control points than
you desire, as well as shift existing control points
to keep the surface from changing. Using the
“Exact” mode on Insert Edge also standardizes the
surface.
Steps:
Where is this Command:
Access this command from T-Spline>Insert
Edge Exact
Steps:
• Select the Insert Point Simple command from the
T-Spline menu..
• Select an edge to insert a point.
• Press Enter.
• Click to set control point locations.
• Press Enter when done.
• Select the Insert Edge Exact command from the
T-Spline menu..
• Select a edge loop or edge ring to be split.
• Pick a point along the edge to insert a new edge.
### Command Line Options:
Using Insert Point
Simple to insert
multiple points.
### Command Line Options:
Insertion Type: Simple/Exact: Inserting an
edge will try to keep the surface of the model as
simple as possible but the model surface will be
affected. Insert Edge Exact will add and edge
while maintaining the surface but the model will
likely become more complicated.
Insertion Type: Simple/Exact: Inserting a
point will try to keep the surface of the model as
simple as possible but the model surface will be
affected. Insert Point Exact will add a point while
maintaining the surface but the model will likely
become more complicated.
Both Sides: When this option is selected a edge
will be added to both sides of the original edge
This allows you to insert edges on both sides of an
existing edge an equal distance apart, helping you
to maintain symmetry.
RetopoSnap: Turns on and off RetopoSnap.


---

## Images

- ![m8book2_p55_img01.png](../images/m8book2_p55_img01.png) page 55 (shared page)
