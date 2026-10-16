# Slide ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 88-89
- Section title: Slide ￼
- Section id: m8book2-106-slide
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-106-slide | Slide ￼ | m8book2 pages 88-89 -->

<!-- page 88 -->

Where is this Command:
Access this command from T-Spline>Bevel
Steps:
Roundness: This option appears when
KeepOnFace is set to ‘No’ only. Also, it works only
when two or more Segments are used.
It determines how round or flat the Bevel is for
multiple Segments. Enter a value between 0.0
and 1.0. If an Edge on a corner is being beveled,
then a roundness of 0 will create a flat Bevel, and
a roundness of 1 will create a rounded Bevel.
• Select Bevel from the T-Spline menu.
• Select Edges to Bevel. (Select a loop of edges for
the best results.
• Set a Bevel percentage. Enter an amount between
0.0 - 1.0.
• Press Enter when done.
Note: Results are only visible in Box Mode.
### Command Line Options:
Bevel created
with three
Segments and
Roundness set
to 0.5.
Segments: The number of rows of faces created.
RetopoSnap=Yes/No: Turns On and Off
RetopoSnap.
RetopoOffset: When RetopoSnap is ‘On”, select
a value. Set the Offset distance from the mesh
that each point will snap to. If set to zero, the
control points will snap right on the mesh.
## Slide
A Bevel is created with 3 segments on each Edge.
Object before (left) and after (right) using Bevel.
.
Positioning = Percentage (default): The
Bevel will be contained to this percentage of
the faces neighboring the selected edge. The
Percentage is entered as a fraction between 0.0
and 1.0.
‘tsSlide’
This command slides selected Edges along
neighboring Edges.
Where is this Command:
Access this command from T-Spline>Slide
Steps:
Positioning = Distance: The Bevel will be
constrained to this distance away from the
selected edge. Set the Bevel by picking a point
along the T-Spline edge.
KeepOnFace: The outcome is based on the
number of Segments selected; two or more
Segments must be used. If on, the new faces will
be created on the faces of the old model.
• Select Slide from the T-Spline menu.
• Select Edges to Slide.
• Press Enter when done.
• Set Slide distance. Enter a percentage between
0.0 - 1.0.
• Press Enter when done.
Slide is used to move
the edges of faces.
KeepOnFace = ‘No’ with 2 Segments (left) and
KeepOnFace = ‘Yes’ with 2 Segments (right).

<!-- page 89 -->

### Command Line Options:
Where is this Command:
Access this command from T-Spline> Flatten
Steps:
Positioning = Percentage: The edge will slide
this far (as a percentage between 0.0 and 1.0)
toward the neighboring face, in the direction of
the arrows. To slide the other direction, type a
negative number.
• Select Flatten command from the T-Spline menu.
• Select at least 4 control points to Flatten.
• Press Enter when done.
Note: At least four vertices must be the input for the
Flatten points command to work. The command will
work on T-Spline, Nurbs and mesh control points.
Positioning = Distance: The edge will slide this
amount, as an absolute distance in mm, in the
direction of the arrows. To slide the other direction,
type a negative number.
Object with
control
points
before
Flatten.
Hint: Press the
“Alt” key to
disable O-Snaps
while selecting a
point along the
Edge.
Roundness: This option gives the ability to slide
along an arc. Enter a value from 0.0 to 1.0,
Object’s
control
points
after
Flatten.
RetopoSnap=Yes/No: Turns On and Off
RetopoSnap.
## Flip Normals
RetopOffset: When RetopoSnap is ‘On”, select a
value. Set the Offset distance from the mesh that
each point will snap to. If set to zero, the control
points will snap right on the mesh.
Note: It is easier to see what the slide command is
doing in Box mode than in Smooth mode. Use the
Smooth/Box Toggle to toggle to Box mode.
## Flatten
‘tsFlip’
Determine the direction a surface is facing using
the Flip Normals command. Every surface within
Matrix and T-Splines has what are referred to as
surface normals. You can think of surface normals
as, which way the surface thinks is up. This
command is used to avoid surfaces from pointing
inside out.
Where is this Command:
Access this command from T-Spline>Flip
Normals
Steps:
‘tsFlattenPoints’
Similar to the Rhino Set Points command, the
Flatten Points command aligns T-Splines vertices
along a single plane in space. This command is
ideal to use prior to creating Symmetry within an
object. It ensures the item you are mirroring will be
seamless and will Weld together with its neighbor
after the mirror.
• Select Flip Normals from the T-Spline menu.
• Select a T-Spline surface to Flip Normals.


---

## Images

- ![m8book2_p88_img01.png](../images/m8book2_p88_img01.png) page 88 (shared page)
- ![m8book2_p88_img02.png](../images/m8book2_p88_img02.png) page 88 (shared page)
- ![m8book2_p88_img03.png](../images/m8book2_p88_img03.png) page 88 (shared page)
- ![m8book2_p88_img04.png](../images/m8book2_p88_img04.png) page 88 (shared page)
- ![m8book2_p88_img05.png](../images/m8book2_p88_img05.png) page 88 (shared page)
- ![m8book2_p88_img06.png](../images/m8book2_p88_img06.png) page 88 (shared page)
- ![m8book2_p89_img01.png](../images/m8book2_p89_img01.png) page 89 (shared page)
- ![m8book2_p89_img02.png](../images/m8book2_p89_img02.png) page 89 (shared page)
- ![m8book2_p89_img03.png](../images/m8book2_p89_img03.png) page 89 (shared page)
