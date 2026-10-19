# Variable Chamfer Surfaces ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 206-207
- Section title: Variable Chamfer Surfaces ￼
- Section id: m8book1-223-variable-chamfer-surfaces
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p206_img01.png, m8book1_p206_img02.png, m8book1_p206_img03.png, m8book1_p206_img04.png, m8book1_p207_img01.png

---
<!-- section m8book1-223-variable-chamfer-surfaces | Variable Chamfer Surfaces ￼ | m8book1 pages 206-207 -->

<!-- page 206 -->

### Using Handle Grips:
surfaces. If a corner is narrower than the ball
radius, the ball cannot negotiate the turn which
can cause the command to fail.
### Command Line Options:
Distances: The distance from the intersection of
the surfaces to the edge of the chamfer. Enter a
value for the first and second chamfer distances
in the Command Line before picking the first
surface.
The edge point controls the position (left). The inner
point controls the Chamfer distance (right).
Extend: Extends the Chamfer surface as far as it
can along a surface.
Trim = Yes/No/Split: Yes; Trims the original
surfaces to the intersections with the resulting
surface. No; does not trim. Split: splits the original
surfaces at the resulting surface edges.
The default handles have one grip that can be
dragged to change the radius. Any added or
copied handles have two grips. Use the grip on
the edge to move the handle along the edge.
Use the grip in the center to change the radius at
the handle location.
## Variable Chamfer Surfaces
### Command Line Options:
Radius/Distance Options: Radius and
Distance options appear on the Command
Line when the inner point on the handle grip is
selected or when a handle is added.
‘VariableChamferSrf’
Creates a Chamfer surface between surface
edges with varying distance values, trims the
original surfaces faces, and joins the resulting
surfaces.
FromCurve: Select a curve. The distance of the
curve at the picked location will be used.
Where is this Command:
Access this command from Surface> Variable
Chamfer Surfaces
FromTwoPoints: Pick two points to show the
distance.
Steps:
Adding a handle in Variable Chamfer Surfaces (left)
and completing the command (right).
• Select Variable Chamfer Surfaces from the
Surface menu.
• Select a first surface for Variable radius Chamfer.
(Specify a Chamfer distance first in the Command
Line)
• Select a second surface for Variable radius
Chamfer.
• Select the Chamfer Handle that appears to edit
the position or Radius. Click on the edge point to
move the handle position. Click and drag the inner
point to change or set the Chamfer distance.
• Specify a Command Line option, or select a handle
to edit.
• Press Enter when done.

<!-- page 207 -->

### Handle Options:
AddHandle: Adds a handle along the edges.
Drag the cursor and click to set the location. Press
Enter when done.
CopyHandle: Adds a new handle using the
distance from the selected handle. Drag to move
the position, click to set it, and press Enter when
done.
A split surface is offset with Offset Surface to create
a custom channel bottom.
### Command Line Options:
RemoveHandle: Visible only when at least one
handle has been added. Click on a handle to
remove it.
Distance: Enter a distance amount for the offset.
Type a new value and press Enter when done.
SetAll: Sets the distance or radius for all handles
and all handles will be updated.
Corner = Round/ Sharp: Round creates a fillet
at sharp corners on the original surface. Sharp
maintains the sharp corner when the original
surface has a sharp corner.
Note: Only added handles can be removed. The
default handles at the ends of each open edge
segment cannot be moved or deleted. This is the
minimum information the command needs in order
to work. The handle at the end of a single closed
edge can be moved but not deleted.
Solid = Yes/ No: Makes a closed solid from the
input and offset surfaces by lofting a ruled surface
between all of the matching edges.
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces when set to ‘Yes’.
Loose = Yes/ No: For surfaces only, the
resulting surface point structure is identical to the
original surface when set to ‘Yes’.
Preview Yes/No: Yes; displays a dynamic
preview. When set to ‘No’, no preview will display.
There is no opportunity to change the options.
## Offset Surface
Tolerance: Sets the tolerance for the offset
surface. Type ‘0’ to use the default tolerance.
Both Sides: Creates an offset on both sides of
the original.
‘OffsetSrf’
Copies a surface or polysurface so that locations
on the copied surface are the same specified
distance from the original surface.
FlipAll: Changes the offset direction on all the
input objects.
Where is this Command:
Access this command from Surface>Offset
Surface
## Variable Offset Surface
Steps:
‘VariableOffsetSrf’
Use this tool to deform the surface while it offsets.
Offset different parts of a surface to different
varying distances away from the original surface
Where is this Command:
Access this command from Surface> Variable
Offset Surface
• Select Offset Surface from the Surface menu.
• Select a surfaces or polysurfaces to offset.
• Press Enter when done.
• Select object to flip direction (if needed). Preview
arrows will display the direction of the offset. Click
on the object to change the offset (flip) direction.
Enter an offset distance amount in the Command
Line when selecting ‘Distance’.
• Press Enter when done.
