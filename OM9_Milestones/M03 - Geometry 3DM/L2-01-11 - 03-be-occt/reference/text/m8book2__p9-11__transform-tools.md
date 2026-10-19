# Transform Tools

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 9-11
- Section title: Transform Tools
- Section id: m8book2-002-transform-tools
- Kind: chapter_intro
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p9_img01.png, m8book2_p9_img02.png, m8book2_p9_img03.png, m8book2_p9_img04.png, m8book2_p10_img01.png, m8book2_p11_img01.png, m8book2_p11_img02.png

---
<!-- section m8book2-002-transform-tools | Transform Tools | m8book2 pages 9-11 -->

<!-- page 9 -->

# CHAPTER 22
# Transform Tools
### Manipulate, relocate, scale and deform Curves, Surfaces and Solids using the
### powerful commands within the Transform menu.
object.
Cage Edit before (left) and after (right)
Transform Tools
The various commands in this menu work to
alter models or parts of models by scaling them,
re-orienting them to a new size and location,
changing their shape, repositioning them in
relationship to a C-Plane, or placing copies of
objects in a new relationship to a surface or
curve. The important thing to remember is that
Transform commands work with existing objects
and not all commands work with all object types.
The transform commands make it so you can
model a very basic shape and then manipulate
it into a more complex design. Whether it is
laying out a design flat to Smart Flow on to a
Destination surface or bending, twisting or using
Cage Edit on primitives to be exactly the form you
need
Smart Flow
Smart Flow allows you to lay your designs out
flat and have them automatically flow on to a
Destination surface. This workflow can make the
most complicated of design significantly easier.
Layout your design flat and Matrix will automatically
flow it on to your flow surface using the Smart Flow
command.
A simple set of cylinders twisted to create rope.
Cage Edit
Adjust the shape of an object using the Cage Edit
command. This tool captures geometry with a
control object (Bounding Box, Line, or Rectangle).
Specify the point count on the control object in
the X,Y and Z axis. Then change the shape of the
geometry by moving the points on the control

<!-- page 10 -->

### Skills & Commands in this Chapter
Scale and Orient:
Scale 3D, Scale 2D, Scale 1D, Scale Non-Uniform,
Scale by Plane,
Edit Objects:
Bend, Symmetry, Solid Points On, Cage Edit,
Stretch, Taper, Shear, Twist, Maelstrom, Move UV,
Soft Move, Create Cage,
Flow:
Smart Flow, Smart Flow Rigid, Disable Smart
Flow, Smart Flow Pull Down Curves, Flow Along
Curve, Flow Along Surface, Splop
Array:
Polar Array, Array Along Curve, Array Linear,
Array Along Surface, Array Curve on Surface,
Array
Move:
Rotate, Project to C-Plane, Set Points,  Make
Down Facing, Re-Map to C-Plane

<!-- page 11 -->

### Command Line Options:
## Scale 3D
Copy=Yes/No: This specifies whether or not
the objects are copied. A plus sign appears at the
cursor when Copy mode is ON.
‘Scale’
This scales selected objects uniformly on the X, Y,
and Z axes.
## Scale2D
Where is this Command:
Access this command from Transform>Scale3D
Right Click Command: Scale3D with Copy
‘Scale2D’
Scales selected objects uniformly in two
directions.
Steps:
Where is this Command:
Access this command from Transform>Scale2D
Right Click Command: Scale2D with Copy
Steps:
• Select Scale3D from the Transform menu.
• Select object(s) to scale.
• Press Enter.
• Pick an Origin point (the point to scale from).
• Type the Scale Factor, or pick two Reference
Points. Click for first Reference Point slightly
above the object. Then, drag up or down until the
desired size is reached. The second Reference
Point click will set the object size.
• Select Scale2D from the Transform menu.
• Select Object(s) to scale.
• Press Enter.
• Pick an Origin point.
• Type the Scale Factor, or pick two Reference
Points. Click for the first Reference Point slightly
above the object. Then, drag Up or Down until
the desired size is reached. The second Reference
Point click will set the object size.
• The objects are expanded or contracted uniformly
in the two directions of the axes of the active
viewport construction plane where the Scale
Factor is specified.
Scale 3D before and after (Scale factor=2)
Note: When scaling by dragging, the scale factor
becomes the default next time the Scale2D
command is used.
Scale an object from an arbitrary size to a specific
one:
• Use the Origin Point and the first Reference Point
to establish the original size of the object.
• Type the new size for the second Reference Point.
Note: When scaling by dragging, the scale factor
becomes the default the next time the scale
command is used.
Scale 2D before and after (Scale Factor=2)
### Command Line Options:
The distance between the Origin point and the first
Reference Point will determine the rate the object
scales. Placing these locations close to one another
will scale and object more rapidly.
Copy=Yes/No: This specifies whether or not
the objects are copied. A plus sign appears at the
cursor when Copy mode is ON.
