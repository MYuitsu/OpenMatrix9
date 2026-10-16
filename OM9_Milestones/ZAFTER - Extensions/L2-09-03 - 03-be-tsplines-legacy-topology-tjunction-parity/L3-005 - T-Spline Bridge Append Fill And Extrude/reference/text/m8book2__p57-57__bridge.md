# Bridge ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 57-57
- Section title: Bridge ￼
- Section id: m8book2-071-bridge
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-071-bridge | Bridge ￼ | m8book2 pages 57-57 -->

<!-- page 57 -->

Copy=Yes/No: If the “Copy” option is toggled to
‘Yes’, it will leave the original not-thickened surface
behind after the new thickened surface has been
created. If ‘No’ , no copy will be created.
Note: If the surface is closed, the Thicken command
creates a second, separate surface.
Segments: The “Segments” command
determines how many faces will be added within
the Bridge. If you are bridging a long distance or
if there is an angle between the two items being
bridged, increasing the number of segments will
increase the complexity of the surface. This will
smooth the transition between objects.
## Bridge
ShowPreview=On/Off: Will display a preview in
box mode of the resulting bridge.
‘tsBridge’
This powerful tool connects Edges or Faces to
one another creating transitions between objects.
The T-Spline Bridge command creates a new
T-Spline surface between two T-Spline objects.
Where is this Command:
Access this command from T-Spline>Bridge
Follow Curve = No/Yes: The “FollowCurve”
Command Line option allows you to control the
direction of the bridge by confining it to a curve.
When using the “FollowCurve” command it is best
to place the curve in the center and perpendicular
to the faces or edges you would like to bridge.
Also, take note to increase the “Segment” count to
allow for any bends in the curve you are bridging
along.
Note: The Bridge command works on faces or border
edges. In either case, two groups of faces or edges
should be selected. If faces are selected, a bridge will
be built between the borders of the selected regions,
and the faces will be removed. You may switch
between face or edge mode by using the Selection
Mode command option. You need to have the same
number of edges or faces on each side of the bridge.
Steps:
Rotation: This Command Line option becomes
available when you select ‘FollowCurve=Yes’.
‘Rotation’ allows you to enter a value in degrees to
determine the orientation of the curve. If you draw
the curve in the correct orientation, you will not
need to use the “Rotation’ option.
Twist: The “Twist” command allows you to specify
the number of complete rotations the item will
have from the start to the end of the Bridge. This
creates a spiral effect and can also help to prevent
the surface from pinching as it bridges along the
curve.
• Select Bridge from the T-Splines menu.
• Select groups of faces or edges to bridge.
• Press Enter when done.
• Select the number of segments in the Command
Line.
• Set your point for alignment and orientation of
the bridge. (These should typically be the same
point facing the same direction as if you were
performing a Sweep.)
• Press Enter when done.
## Append
### Options:
Selection Mode=Faces/Edges: Allows you to
set the selection to either faces or face edges.
‘tsAppend’
The Append command adds new faces to a
T-Spline. This command closes a surface by
drawing a face, as opposed to using the Fill Hole
command. It is also possible to use this command
to manually draw a  face unconnected to any
surface to begin the layout for a new model.
Where is this Command:
Access this command from T-Spline>Append
There are three different modes for appending a
face: Simple, From Edge, and Chain.
Selecting edges while using the Bridge command.


---

## Images

- ![m8book2_p57_img01.png](../images/m8book2_p57_img01.png) page 57 (shared page)
- ![m8book2_p57_img02.png](../images/m8book2_p57_img02.png) page 57 (shared page)
