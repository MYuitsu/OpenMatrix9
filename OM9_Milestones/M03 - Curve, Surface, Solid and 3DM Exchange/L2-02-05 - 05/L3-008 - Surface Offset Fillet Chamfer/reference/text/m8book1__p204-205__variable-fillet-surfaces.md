# Variable Fillet Surfaces ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 204-205
- Section title: Variable Fillet Surfaces ￼
- Section id: m8book1-221-variable-fillet-surfaces
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p204_img01.png, m8book1_p204_img02.png, m8book1_p204_img03.png, m8book1_p205_img01.png, m8book1_p205_img02.png, m8book1_p205_img03.png

---
<!-- section m8book1-221-variable-fillet-surfaces | Variable Fillet Surfaces ￼ | m8book1 pages 204-205 -->

<!-- page 204 -->

Where is this Command:
Access this command from Surface>Variable
Fillet Surfaces
Where is this Command:
Access this command from Surface>Fillet
Surfaces
Steps:
Steps:
• Select Fillet Surfaces from the Surface menu.
• Select first surface to fillet. (Select a radius option
first in the Command Line.)
• Select the second surface to fillet.
• Select Variable Fillet Surfaces from the Surface
menu.
• Select the first surface for Variable Fillet Surfaces.
(Enter a Radii amount before selecting the
surface, or press Enter to use the default value.)
• Select the second surface for Variable Fillet
Surfaces.
• Select the Fillet Handle that appears to edit the
position or Radius. Click on the edge point to
move the handle position. Click and drag the inner
point to change or set the fillet radius.
• Specify a Command Line option, or select a handle
to edit.
• Press Enter when done.
The outside and
side surfaces are
used to create a
fillet on the right
edge with Fillet
Surface. Before
(left) and after
(right).
### Command Line Options:
Radius: Specify the fillet radius.
Extend: When one input surface is longer than
the other, the fillet surface is extended to the input
surface edges.
Trim=Yes/No: Yes; Trims the original surfaces
to the intersections with the resulting surface. No;
does not trim the original surfaces..
To edit a Fillet handle, click on the
edge point (1) to move the position. Click and drag
the inner point (2) to change the fillet radius or enter
a radius value in the Command Line.
Split: Splits the original surfaces at the resulting
surface edges.
### Radius Options:
Fillet Surface creates a
tangent, curved surface
between two surfaces
Radius/Distance Options: Radius and
Distance options appear on the Command
Line when the inner point on the handle grip is
selected or when a handle is added.
FromCurve: Select a curve. The radius of the
curve at the picked location will be used.
## Variable Fillet Surfaces
FromTwoPoints: Pick two points to show the
radius distance.
‘VariableFilletSrf’
Creates a curved, tangent surface between two
surface edges with varying radius values.

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
