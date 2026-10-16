# Variable Blend Surfaces ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 192-193
- Section title: Variable Blend Surfaces ￼
- Section id: m8book1-208-variable-blend-surfaces
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p192_img01.png, m8book1_p192_img02.png, m8book1_p192_img03.png, m8book1_p193_img01.png

---
<!-- section m8book1-208-variable-blend-surfaces | Variable Blend Surfaces ￼ | m8book1 pages 192-193 -->

<!-- page 192 -->

### Adjust Surface Blend Options Cont’d:
### Using Handle Grips:
AddShapes: Add additional curve profile shapes
to increase control over the blend shape. This is
especially important the input shapes are complex
and more control is needed for the shape in more
locations. Click a location on each surface edge to
add another shape curve.
Planar Sections: Forces all shape curves to be
planar and parallel to the specified direction.
Same Height: If gaps between the surface vary,
this option maintains the height of the shape
curves throughout the blend.
Preview: Check to dynamically display a preview
as the options change.
The default handles have one grip that can be
dragged to change the radius. Any added or
copied handles have two grips. Use the grip on
the edge to move the handle along the edge.
Use the grip in the center to change the radius at
the handle location.
### Radius/ Distance Options:
## Variable Blend Surfaces
Radius/Distance options: Set the radius value
before the first surface is selected. And, Radius
and Distance options appear on the Command
Line when a handle grip is selected and moved.
‘VariableBlendSrf’
Creates a blend surface between surface edges
(of two intersecting surfaces) with varying radius
values, trims the original surfaces faces, and joins
the resulting surfaces.
FromCurve: Select a curve. The radius of the
curve at the picked location will be used.
Where is this Command:
Access this command from Surface>Variable
Blend Surfaces
FromTwoPoints: Pick two points to show the
Radius distance.
Note: Both surfaces must intersect.
Steps:
Dragging a handle to edit the Radius.
• Select Variable Blend Surfaces in the Surface
menu.
• Select the first surface for Variable Radius Blend
on the surface near the edge that will be blended.
(Enter a Radius value before selecting or edit the
handle later).
• Select the second surface for Variable Radius
Blend on the surface near the edge that will be
blended.
• Select a blend handle to edit. Click on the handle
and enter a different Radius value, etc. Or, press
Enter to accept the default Radius value.
• Choose from any of the Command Line options if
desired.
• Press Enter when done.
Variable Blend
Surfaces is
complete.
Surfaces are
trimmed and
joined.

<!-- page 193 -->

### Handle Options:
## Loft
AddHandle: Adds a handle along the edges.
Use the grip on the edge to move the handle
along the edge. Use the grip in the center to
change the radius at the handle location.
‘Loft’
Creates a surface that stretches between curves
or surface edges. Two or more curves or surface
edges of varying shapes can be connected using
the Loft command.
CopyHandle: Adds a new handle by selecting
an existing handle or using the distance from the
selected handle.
Where is this Command:
Access this command from Surface>Loft
Steps:
RemoveHandle: Visible only when at least one
handle has been added. Select the blend handle
to remove.
SetAll: Sets the distance or radius for all handles.
LinkHandles: Editing a single handle updates all
handles the same amount.
• Select Loft from the Surface menu.
• Select curves to Loft. Choose either closed or
open curves (or edges) in the order to be lofted.
• Drag seam point to adjust (closed curves only).
Seam points must line up.
• Press Enter when done.
• Specify the settings in the Loft Options dialog box.
• Click ‘OK’ to complete the command.
Things to remember when working with handles:
Note:  Curves can either be closed or open but not
both. Also, it is very important to click in roughly the
same position on each curve. Selecting one curve
by clicking on one side of it and the next curve by
clicking on the other side will yield a twisted surface
that is very likely to be undesirable.
### Options:
• Only added handles can be removed.
• The default handles at the ends of each open
edge segment cannot be moved or deleted. This
is the minimum information the command needs
in order to work.
• The handle at the end of a single closed edge can
be moved but not deleted.
Adjust Closed Curve Seams: Select a seam
### Rail-Type Options:
DistanceFromEdge: The distance from the
edge curve determines the intersection.
point marker, and move it along
the closed curve. Continue to
adjust the seam points until they
line with each other and the
closed curves all have the same
direction, and then press Enter.
RollingBall: The radius of a rolling ball
determines the intersection.
### Adjust Seam Options:
DistanceBetweenRails: The distance between
the edge rails determines the intersection.
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces.
Flip: Reverses the curve direction. Select the
seam to Flip. Or,  click the arrow end on the seam
and it will flip directions.
Automatic: Attempts to align the seam points
and direction to match without intervention.
Preview = Yes/No: Displays a dynamic preview.
The options can be changed at this point and the
preview will update. When set to ‘No’, no preview
will display. There is no opportunity to change the
options.
Natural: Moves the seam point to the way they
were at the beginning of the command.
