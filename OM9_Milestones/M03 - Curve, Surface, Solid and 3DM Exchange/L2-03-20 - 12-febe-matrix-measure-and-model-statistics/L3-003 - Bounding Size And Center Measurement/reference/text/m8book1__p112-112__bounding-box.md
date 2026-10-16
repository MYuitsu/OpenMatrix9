# Bounding Box ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 112-112
- Section title: Bounding Box ￼
- Section id: m8book1-110-bounding-box
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p112_img01.png

---
<!-- section m8book1-110-bounding-box | Bounding Box ￼ | m8book1 pages 112-112 -->

<!-- page 112 -->

Layers for Make2D Objects:
## Bounding Box
Visible Lines: Select a layer name for visible
lines or type a new layer name.
‘BoundingBox’
This command draws a box around any object
or objects in the viewport to assist the user with
orienting them to other objects in the viewports.
Visible tangents: Select a layer name for visible
tangent edges or type a new layer name.
Where is this Command:
Access this command from Utilities>Bounding
Box
Steps:
Visible Clipping Planes: Select a layer name
for visible clipping plane intersections or type a
new layer name.
Hidden Lines: Select a layer name for hidden
lines or type a new layer name.
Hidden Tangents: Select a layer name for
hidden tangent edges or type a new layer name.
• Select Bounding Box from
the Utilities menu.
• Select the objects to
frame with a box.
• Press Enter.
• A Bounding Box is created
around the selected items
in the viewport.
Hidden Clipping Planes: Select a layer name
for hidden clipping plane intersections or type a
new layer name.
### Command Line Options:
Annotations: Select a layer name for dimensions
or type a new layer name.
Coordinate System C-Plane: Bases the object
creation on construction plane coordinates.
Restore Defaults: Resets layer names to default
names.
Coordinate System World: Bases the object
creation on world coordinates.
Output: For three-dimensional objects, specifies
the resulting bounding box object type.
Note: Select a new layer color for each type of
line by selecting it from the drop-down menu. The
new layer color will be entered into the text box
beside the type of line chosen. Now, when the 2-D
drawing is created, the curves may be selected
in the conventional way (right-clicking on the
corresponding color in the Layers Menu.
Solids: Creates a solid box
Meshes: Creates a mesh box.
Note: If silhouettes are nearly overlapping another
curve in the view, they may not appear on the correct
layer color.
Curves: Creates six grouped rectangles.
Tips for using Make 2D Drawings:
None: Reports the dimensions of the bounding
box at the Command Line.
Note: If an object is planar (drawn all on one
construction plane), the bounding box will also be
drawn on the plane on which the object sits. If the
object is three-dimensional, a 3-D bounding box will
be drawn around it.
Depending on the object(s) and their complexity,
it can take an hour or more when using the
Make 2D Drawing command. CPU speed is a
determining factor for the time is will take for the
command to run. Use this command at the end
of the day and return to find your complete 2D
drawing.
Make 2D Drawing does not work with mesh
objects.
