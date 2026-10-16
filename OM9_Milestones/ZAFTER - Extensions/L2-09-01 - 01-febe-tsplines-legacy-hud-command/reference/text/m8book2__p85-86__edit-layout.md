# Edit Layout ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 85-86
- Section title: Edit Layout ￼
- Section id: m8book2-102-edit-layout
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-102-edit-layout | Edit Layout ￼ | m8book2 pages 85-86 -->

<!-- page 85 -->

Note: Not all Star Points can be turned to T-Points. A
Star Point can be changed to a T-Point if the vertex’s
valence is less than 4 and if the resulting face into
which the T-Point will point is a four-sided face.
Selection Sets dialog box
### Options:
Record: Save a selection. While you have
your desired T-Splines grips selected, press the
record icon and a new selection group will be
saved. You can name this new selection.
Play: Selects the saved group.
Delete: Deletes the current selection group.
Inside the T-Splines Layout command, Star Points,
T-Points L-Points will appear as Star, T, and L icons,
respectively.
An error Star Point shows in the Layout display as a
red star. This occurs when a point is simultaneously a
T-Point and a Star Point.
Rename the group by clicking in the name box. A
number to the left of the name dictates how many
grips are included in that set.
### Command Line Options:
## Edit Layout
ShowTPoints Yes/No: When set to ‘Yes’ all
T-Points are displayed as letter ‘T’. When set to
‘No’, T-Points are not displayed.
ShowLPoints Yes/No: An L-Point is a special
type of T-Point that occurs where two edges
running perpendicularly meet, forming an ‘L’
shape. When set to ‘Yes’ all L-Points are displayed
as letter ‘L’. When set to ‘No’, L-Points are not
displayed.
‘tsLayout’
Allows you to fix errors that prevent our model
from displaying as a smooth T-Spline. This
command also allows you to optimize the
smoothness and isocurve flow of your model by
changing T-Points to Star Points and vice versa.
Understanding the basic layout properties of
a T-Spline will improve your success with the
program. See Understanding T-Points and Star
Points below to begin.
ShowStars Yes/No: When set to ‘Yes’ all Star
Points are displayed as yellow star icons. When set
to ‘No’, Star Points are not displayed.
Where is this Command:
Access this command from T-Spline>Edit
Layout
Steps:
ShowErrorStars Yes/No: When set to ‘Yes’
all Star Points with errors are displayed as red
star icons. When set to ‘No’, Star Points are not
displayed.
• Select Edit Layout from the T-Spline menu.
• Select T-Spline to edit T-Points and Star Points.
• Select a Vertex. To switch a T-Point to a Star
Point, click on it and the icon will change. Also,
select a T-Point to change which direction it
points.

<!-- page 86 -->

Error Star Point
displays as a red
star.
ShowSeashells Yes/No: Seashells are T-Points
with errors. When set to ‘Yes’ all error Seashells are
displayed as red. When set to ‘No’, Seashells are
not displayed.
non-rectangular. Star points can occur when
using commands like Extrude and Delete Faces.
It can be difficult to control to topology of the
surface near a Star Point. Avoid using Star Points
in general, but especially in areas around crease
or near the surface edge. Improper placement
of Star Points can cause the surface to switch
to Box Mode, unable to shade in Smooth Mode.
When converting to T-spline to NURBS, each
location where a Star Point is present will split into
separate surfaces. Star Points can have 3,5,6, 7,8,....
edges (Valence) and Tangent continuous (G1)
smoothness.
AutoRepair: This option automatically switches
all T-Points to Star Points necessary to display the
smooth surface. It also converts all Seashells to
Star Points.
Ordinary Control Point:
Valence (number of edges) = 4 (can be 2,3 or 4 on
the border)
Smoothness = Curvature continuous (C2)
### Understanding T-Points and Star
### Points:
T-Splines have four types of vertices (or control
points): T-Points, Star Points, Tangency Handles,
and Ordinary Control Points.
These vertex types are distinguished by their
valence (how many edges are attached to the
point) and their smoothness. Understanding basic
properties of these vertices will increase your
ability to successfully model with T-Splines.
Tangency Handle:
Tangency Handles control the tangency of the
T-Spline surface at a crease or the edge of the
surface. They always exist in the surface for all
creases and edges; however, the default visibility
is set to Display Moved and only displays those
Tangency Handles which have been purposefully
edited by the user. Tangency Handles have
a Valence of  1, 2, or 3 and the Smoothness is
Controls tangency at a crease or edge of surface.
(C0)
## T-Splines Library
‘gvTSplinesLibrary’
A wide variety of pre-built T-Spline designs are
available within the T-Splines Library. Begin
designing from one of the many Starting Point
inventory models or add to an existing design.
Where is this Command:
Access this command from T-Spline>T-Splines
Library
Steps:
T-Point: T-Points are a fork in the T-Splines road
and are unique to the T-Splines program. Unlike
NURBS surfaces where the U and V normals run
perpendicular to one another in straight lines,
T-Splines can come to an abrupt ending part way
through the design and can be started up again
with no problem. T-Splines can have a partial row
of control points that end in a T-Point. T-Points
typically occur when Edges are deleted, during
such commands as Insert Edge, Merge Edges and
Weld. T-Points will react just like regular control
points and maintain a smooth surface without
any kinking or stretching. T-Points can have 3 or 2
edges (Valence) and Curvature continuous (C2)
smoothness.
Star Point: Star Points usually occur on the
corner of objects and allow for a T-Spline to be
• Open the T-Splines Library from the T-Spline
menu.
• Click once on a thumbnail to select a model from
the browser to load into Matrix. (Be sure the
appropriate layer color is turned on in the Layers
menu.)


---

## Images

- ![m8book2_p85_img01.png](../images/m8book2_p85_img01.png) page 85 (shared page)
- ![m8book2_p85_img02.png](../images/m8book2_p85_img02.png) page 85 (shared page)
- ![m8book2_p86_img01.png](../images/m8book2_p86_img01.png) page 86 (shared page)
