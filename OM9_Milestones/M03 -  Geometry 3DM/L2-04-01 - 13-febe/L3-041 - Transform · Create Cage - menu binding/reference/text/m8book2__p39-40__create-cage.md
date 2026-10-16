# Create Cage ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 39-40
- Section title: Create Cage ￼
- Section id: m8book2-043-create-cage
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p39_img01.png, m8book2_p39_img02.png, m8book2_p40_img01.png

---
<!-- section m8book2-043-create-cage | Create Cage ￼ | m8book2 pages 39-40 -->

<!-- page 39 -->

Steps:
• Select Soft Move from the Transform menu.
• Select objects to Soft Move.
• Press Enter when done.
• Pick the point to Move from or click an object type
option.
• Pick the Radius point of influence.
• Pick an Offset point.
• Select a Control Point to Move, or press Enter to
End the command.
Surfaces: Select surfaces to radiate the falloff
curve and influence the Move. The reference
surface forms the tip of the Falloff dome and the
Falloff curve is active across the edges of the
surface over the set Radius.
Interactive Manipulation:
1.Drag the inner circle to the falloff distance.
2.Drag the outer circle to change the radius
of influence (fall-off/inner circle will drag
proportionally).
3.Drag the point at the top of the dome to change
the offset amount.
4.Drag the point at the base of the dome to
change the base point.
### Command Line Options
Vertical: Moves objects vertical to the
Construction plane.
Selecting an Offset point after choosing the Radius
in Soft Move.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
LinkAll: Ties the Falloff, Radius, Offset, and
Anchor to each other so changing one, changes
the others.
## Create Cage
‘Cage’
Creates a box-shaped Cage object to be used
with the Cage Edit command to deform other
objects.
Soft Move of points completed.
### Object Type Options:
Note: Cages with a small number of control points
can modify more complex objects. This ensures that
transitions are smoother than could be achieved
using standard control point editing. After creating
the Cage object, use the CageEdit command to
capture objects with the Cage. Use the Cage’s
control points to deform the captive objects.
Points: Select point objects as basis for the
Move. The Falloff curve is based on each selected
point and acts through a Sphere of Radius set by
the user.
Where is this Command:
Access this command from Transform>Create
Cage
Steps:
• Select Create Cage from the Transform menu.
• Pick the first corner of Cage base.
• Pick the second corner of Cage base.
Curves: Select curves as basis for the Falloff
curve. The Falloff is applied radially along the
curve creating a tube-shaped volume where the
Move is active.

<!-- page 40 -->

• Pick the height of the Cage.
• Enter the number of Cage points.
• Press Enter.
XPointCount/YPointCount/ZPointCount:
Number of control points in the X, Y, and Z
directions
XDegree/YDegree/ZDegree:
Degree in the X, Y,  and Z directions.
Create Cage places a Cage in the viewport to be
used with editing other objects.
### Command Line Options:
BoundingBox: Uses the captive objects
bounding box as the control object.
Bounding Box Options:
CoordinateSystem: Sets the coordinate system
for the bounding box.
CPlane: Uses Construction plane coordinates.
World: Uses World coordinates
3Point: Pick three points to establish a
coordinate system
Diagonal: Draws the base rectangle from two
diagonal corners. No option for side length is
offered.
Diagonal Steps:
• Pick two opposite corners
• Cube option
• Pick a height that also defines the Cube’s
orientation.
Cage points: Once the box shape is defined,
define the number of control points and degree of
the Cage in each of three directions.
