# Utilities Menu

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 105-107
- Section title: Utilities Menu
- Section id: m8book1-100-utilities-menu
- Kind: chapter_intro
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p105_img01.png, m8book1_p105_img02.png, m8book1_p106_img01.png, m8book1_p107_img01.png, m8book1_p107_img02.png, m8book1_p107_img03.png, m8book1_p107_img04.png, m8book1_p107_img05.png

---
<!-- section m8book1-100-utilities-menu | Utilities Menu | m8book1 pages 105-107 -->

<!-- page 105 -->

# CHAPTER 17
# Utilities Menu
### The commands within the Utilities menu work on objects after they have been
### created. The Utilities tools validate and fix problematic geometry as well as offer
### functionality to streamline standard modeling tasks.
command creates a 3D box for 3D objects (have
height) and a 2D rectangle around 2D objects (are
flat).
2D Drawing
More closely related to the technical presentations
possible in the View menu than the rest of the
Utilities commands, this tool creates a 2D Drawing
of a 3D model. Start up this command and select
the options (which view, how many views, which
parts to draw, what
colors, and options
for visible and
hidden lines, and
annotations. Click
OK to see 2D
drawing. It will
appear in the
viewports as curve
objects drawn on
the Lights layer.
Group & UnGroup
You will want to take careful note of these tools.
Group causes objects to work like a single object,
even though they are many. It is not Joined or
Boolean Unjoined (“Join” for solid objects), as it
does not change the constitution of the models.
It simply changes the way they act. In this case,
selecting one of them selects them all, making
it easy to control large groups of objects, such
as gems and prongs: sets of objects which are
commonly grouped by Matrix builders after
they are created. But there are frequently times
when you’ll want to select and edit one of these
grouped objects by itself. The ONLY way to get it
alone is to select the group and click on UnGroup.
This separates the objects from one another so
that you may now select and use one separately
of its neighbors. The object still knows it’s part of a
Gem Line or a Prong Layout. However, it can now
be adjusted separately for your modeling needs. If
you wish to group them again, select all objects to
Group and click on Group. Just remember: They’re
NOT a single object!
Center Object & Bounding Box
The Center Object utility is a quick way to center
any object or objects in the X, Y, and/or Z planes.
You can also place a point at the center of the
object if you should need this; for example, you
can Move or Scale an object from its center point
for more precision as you model. Another tool
that assists with centering is the Bounding Box
tool. You’ll recall that the Center O-Snap can only
find the center of relatively simple objects: such
as a rectangle or a box. The Center Object Utility
actually places an imaginary Bounding Box - or, a
box as big as the largest extents of the model (H
X W X L) and centers this box, moving the object
with the box as it does so. You can re-create this
magic with the Bounding Box tool, the Center
O-Snap, the Move command, and your old friend
F4. Or save yourself the clicks and use Center
Object. However, you may find another reason
to enclose an object in a Bounding Box. This

<!-- page 106 -->

### Skills & Commands in this Chapter
Diagnostics and Repair:
Direction, Select Bad Objects, Check, Show
Edges, Split Edges, Merge Edge, Join 2 Naked
Edges, Unjoin Edge, Bounding Box, Extract Bad
Surface
Other Utilities:
Make 2D Drawing, Center Object, Group,
Ungroup, Show Z Buffer
Mesh Tools:
Fill Hole, File Holes, Align Mesh Vertices, Mesh
from Nurbs Object, Mesh Reduce, Apply Mesh
UVN, Extract Render Mesh

<!-- page 107 -->

### Single Surface Options:
## Direction
The arrow color for the U and
V directions matches the
construction plane X- and
Y-axis colors.
UReverse/VReverse:
Reverses the surface U- or V-direction.
‘Dir’
Direction displays and edits the direction of the
normals of an object. Normals are an indicator
of which direction a curve or surface is oriented
(clockwise or counter clockwise) and which way
a surface is oriented (inside or outside). These
directions are indicated with white arrows that
only appear during the function.
SwapUV: Switches the U- and V- directions.
Flip: Reverses the direction of the surface.
Where is this Command:
Access this command from Utilities>Direction
Steps:
### Multiple Surfaces Options:
Click any individual surface to flip the normal.
FlipAll: Reverses the direction on all selected
surfaces.
NextMode: Cycles through the modes for all the
selected surfaces.
• Select Direction
from the Utilities
menu. Arrows show
the direction of the
objects normal.
• Select a curve or
surface.
• Move the cursor over the object to show a
dynamic arrow for the direction.
• Click on the mouse to flip the normal direction.
Note: The Command Line options vary depending on
the type of object select when you run the command.
Each set of options will be covered in turn.
Mode=FlipU/FlipV: Reverses the U-direction or
V-direction for all the selected surfaces.
Mode=FlipNormal: Reverses the normal
direction for all of the selected surfaces.
Mode=SwapUV: Switches the U and V
directions for all the selected surfaces.
Closed polysurfaces, surfaces, and lightweight
extrusion objects cannot have their normal direction
facing toward the interior of the object.
### Single Curve Options:
Flip: Reverses the direction
of the curve.
Note: Controlling the direction of curve normals
is important. For example, if a sweep results in
a twisted surface it is likely because the curves
that were swept are facing different directions. To
remedy the twist in the surface, align the direction of
all of the curves. For example, all arrows touching the
ring rail should face the same direction.
### Multiple Curve Options:
Click on an individual curve to change its
direction.
FlipAll: Reverses the direction of
all selected curves.
