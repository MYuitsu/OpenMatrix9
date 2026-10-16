# Merge ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 63-64
- Section title: Merge ￼
- Section id: m8book2-080-merge
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-080-merge | Merge ￼ | m8book2 pages 63-64 -->

<!-- page 63 -->

## Remove Creases
‘tsRemoveCreases’
Removes selected creases from a T-Spline model.
Where is this Command:
Access this command from T-Spline> Remove
Creases
Align to World/Align to C-Plane: Aligns
using World or Construction plane coordinates.
The Align to World option allows you to determine
the location along the axis on which to Set Points.
Determine this location by eye, or enter a value
into the Command Line using World coordinates.
The Align to C-Plane command sets the points
flat against the C-Plane in whatever axis you
choose.
Steps:
Copy = Yes/No: Yes; creates a new copy of the
edited object. No: does not create a copy.
• Select Remove Creases from the T-Spline menu.
• Select the creased edges from the T-Spline object
to remove the creases.
• Press Enter when done. The Crease is now
removed from the T-Splines object.
## Set Points
In Select Vert edit mode, select the Vertices or
control points to align with Set Points.
‘SetPt’
The Set Point command quickly and efficiently
moves and organizes points or verts along a
specific axes. Use this tool to even up edges and
create uniform Vert layouts.
## Merge
Where is this Command:
Access this command from T-Spline> Set
Points
Steps:
‘tsMerge’
The Merge command is similar to Weld but
instead of working on vertices it works on edges.
Merge can be used to connect one edge, or a
chain of edges to another edge or chain of edges.
Join a single edge into one or more edges.
Where is this Command:
Access this command from T-Spline>Merge
• Select Set Points from the T-Spline menu.
• With Select Vert’ Mode enabled, select the Verts to
move.
• In the Set Points dialog, click
the check boxes for the
coordinates.
• Pick a location to match by
dragging the vertices. The
selected objects will line
up with the set point in the
direction or directions you choose.
Note: The process for merging edges will determine
the end result. If both edges or sets of edges are
highlighted prior to running the Merge command, the
new location of both edges will be averaged and they
will meet in the middle. If, on the other hand, you
begin the command and then select an edge, the first
edge that is selected will be moved to the location of
the second edge. This is important to understand to
control the face layout of the design.
Steps:
Note: To move in only direction, clear the check
boxes for the other directions. To set one check and
clear the others, right-click the check box.
### Set Point Options:
Set X/Set Y/Set Z: Align objects in the X, Y, and
Z directions.
• Select Merge from the T-Spline menu.
• Select the first edge or edge chain to Merge. (Use
Select Edge edit mode in the T-Splines HUD.)
• Press Enter.
• Select the second edge or edge chain to Merge.
• Press Enter.
• The first edge moves to meet the second.

<!-- page 64 -->

Alternate Steps:
## Weld
• Select both set sets of edges before running the
Merge command. (Use Select Edit edit mode in
the T-Splines HUD.)
• Select Merge from the T-Splines menu.
• Both edges meet in the middle.
‘tsweld’
The Weld command is used to connect T-Splines
objects to one another, or back to themselves.
Weld connects vertices or points to each other
within a single T-Spline surface or can be used to
repair areas of a model that may not be properly
closed. The order of the selection determines the
overall result.
Note: When using Merge, no additional faces are
added between the areas being Merged. Instead the
existing edges are moved together and seamlessly
connected to one another.
Where is this Command:
Access this command from T-Spline>Weld
Note: If two points are preselected, the Weld
command will average the distances between them.
If the points are selected after the command is run,
the first point will be moved to the position of the
second point.
Steps:
• Select points (Verts) to Weld. (Use Select Vert edit
mode in the T-Splines HUD.)
• Select the Weld command in the T-Spline menu.
• Both Verts are joined and meet in the middle.
Merge with preselected points, before and after.
### Command Line Option:
Alternate Steps:
Smooth: Determines whether the surfaces are
merged smoothly or with a crease. To do so, begin
the command and use the Smooth toggle located
in the Command Line.
## Unweld
• Click on the Weld command in the T-Spline menu.
• Click on the first Vert. Press Enter.
• Click on the second Vert. Press enter.
• First Vert moves to the second.
‘tsUnweld’
The Unweld edge works backwards from Merge
edge in that it disconnects two connected edges.
However, the edges will remain on top of one
another until they are moved.
Where is this Command:
Access this command from T-Spline>Unweld
Steps:
Tip: If you Weld only one vertex, the model will
temporarily not be able to be displayed as a smooth
surface. You need to Weld at least two vertices in a
row to toggle your model to Smooth mode.
• Select edges to Unweld.
•  Select Unweld from the T-Spline menu.


---

## Images

- ![m8book2_p63_img01.png](../images/m8book2_p63_img01.png) page 63 (shared page)
- ![m8book2_p63_img02.png](../images/m8book2_p63_img02.png) page 63 (shared page)
- ![m8book2_p64_img01.png](../images/m8book2_p64_img01.png) page 64 (shared page)
- ![m8book2_p64_img02.png](../images/m8book2_p64_img02.png) page 64 (shared page)
- ![m8book2_p64_img03.png](../images/m8book2_p64_img03.png) page 64 (shared page)
