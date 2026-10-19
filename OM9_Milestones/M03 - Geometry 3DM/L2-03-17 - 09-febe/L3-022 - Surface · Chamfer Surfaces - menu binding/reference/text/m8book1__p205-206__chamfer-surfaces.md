# Chamfer Surfaces ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 205-206
- Section title: Chamfer Surfaces ￼
- Section id: m8book1-222-chamfer-surfaces
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p205_img01.png, m8book1_p205_img02.png, m8book1_p205_img03.png, m8book1_p206_img01.png, m8book1_p206_img02.png, m8book1_p206_img03.png, m8book1_p206_img04.png

---
<!-- section m8book1-222-chamfer-surfaces | Chamfer Surfaces ￼ | m8book1 pages 205-206 -->

<!-- page 205 -->

### Handle Options Cont’d:
### Handle Options:
AddHandle: Adds a handle
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces when set to ‘Yes’.
along the edges. Drag the
cursor and click to set the
location. Press Enter when
done.
Preview Yes/No: Yes; displays a dynamic
preview. When set to ‘No’, no preview will display.
There is no opportunity to change the options.
## Chamfer Surfaces
CopyHandle: Adds a new
handle using the distance
from the selected handle.
Drag to move the position,
click to set it, and press Enter
when done.
‘ChamferSrf’
Creates a flat beveled edge between two input
surfaces..
RemoveHandle: Visible only when at least one
handle has been added. Click on a handle to
remove it.
Where is this Command:
Access this command from Surface>Chamfer
Surfaces
Steps:
SetAll: Sets the distance or radius for all handles
and all handles will be updated.
• Select Chamfer Surfaces from the Surface menu.
• Select first surface to chamfer. (Select a distance
option, first, in the Command Line.)
• Select the second surface to chamfer.
Note: Only added handles can be removed. The
default handles at the ends of each open edge
segment cannot be moved or deleted. This is the
minimum information the command needs in order
to work. The handle at the end of a single closed
edge can be moved but not deleted.
### Rail Type Options:
Three rail types control the intersection.
A flat, beveled
edge is added
between
two surfaces
with Chamfer
Surfaces.
DistFromEdge: The distance from the edge
curves determines the intersection.
Note:
RollingBall (Default): The radius of a rolling ball
determines the intersection.
DistBetweenRails: The distance between the
edge rails determines the intersection.
Variable Fillet
Surfaces is
complete.
• Component surfaces will be selected and
unjoined from their polysurfaces.
• The first chamfer distance is the distance from the
location where the two surfaces would intersect
to the chamfer point on the first surface. (Specify
the distance value in the Command Line before
selecting the surfaces.)
• The second chamfer distance is the distance
from the location where the two surfaces would
intersect if extended to the chamfer point on the
second surface.
• This command works on the analogy of rolling a
ball of a defined radius along the edges of the

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
