# Loft ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 193-195
- Section title: Loft ￼
- Section id: m8book1-209-loft
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p193_img01.png, m8book1_p194_img01.png, m8book1_p194_img02.png, m8book1_p194_img03.png, m8book1_p194_img04.png, m8book1_p195_img01.png, m8book1_p195_img02.png, m8book1_p195_img03.png, m8book1_p195_img04.png, m8book1_p195_img05.png

---
<!-- section m8book1-209-loft | Loft ￼ | m8book1 pages 193-195 -->

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

<!-- page 194 -->

original curves. The surface closely follows the
original. This is a good choice when the input
curves are going around a corner.
Normal Loft with 3 input
curves.
Straight Sections: Creates straight surfaces
between the curves instead of curved surfaces.
### Command Line Option:
Developable: Creates a separate developable
surface or polysurface from each pair of curves.
This option is designed to create surfaces that can
be flattened using the ‘Unroll Surface’  command
(Surface menu> Unroll Surface) without stretching
the surface.
Point: Allows picking a location for the start or
end of the loft at a point. This tapers the loft from
the curve to the selected point. Use this option
only at the start or end of the curve series.
Exceptions: Not all sets of curves create developable
surfaces. You may get no surface or
a partial surface. Two straight lines that are not
parallel are not developable.
### Loft Options (Style):
Uniform: Makes the object knot vectors uniform.
Style: Determines the knot and control point
structure of the resulting surface.
Closed loft: Creates a closed surface, continuing
the surface past the last curve and around to the
first curve. Available when you have selected three
or more shape curves.
Match Start Tangent: If the start curve is
a surface edge, the lofted surface maintains
tangency with the adjacent surface.
Normal (Default):  The surface gradually
changes between each shape at an even rate so
that the surface touches the curve as it nears it,
taking on its shape perfectly at that point before
morphing into the appropriate shape for the next
curve. This is a good choice when the curves are
proceeding in a relatively straight path or there is a
lot of space between the curves.
Example shows a Loft surface between a surface
edge and a curve. Match Start Tangent is checked
‘ON’ (left) and ‘OFF’ (right).
Match End Tangent: If the end curve is a
surface edge, maintains tangency with the
adjacent surface.
Loose: This causes the new surface to conform
more loosely (pulls away from) to the cross-
section curves, making for a smoother and more
even surface.
Tight: This style is very close to the Normal,
but sticks even more closely to the shape of the
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined, tangent curves.

<!-- page 195 -->

### Cross-section Curve Options:
See an explanation of these settings in Surface
menu> Sweep 1.
Preview: Check to dynamically display a preview
as the options change.
## Curve Network
Curves are assigned letters to adjust settings in the
Curve Network Options box.
NetworkSrf
Creates a surface from a network of three or more
crossing curves.
Where is this Command:
Access this command from Surface>Curve
Network
Note: All curves in one direction have to cross all
curves in the other direction and cannot cross each
other.
Steps
Curve Network surface is complete.
• Select Curve Network from the Surface menu.
• Select curves in a network.
• Press Enter when done.
• Specify the settings in the Options dialog box that
appears.
• Click on ‘OK’ to complete the command.
### Curve Network Options:
### Command Line Option:
NoAutoSort: Turns off automatic sorting so the
curves can be selected manually.
Tolerances:
Edge curves:
Sets the tolerance
for the edge
curves. The edges
of the surface will
be within this value
from the edge
curves.
Three curves in a network
of curves are intersecting
at the end points. Select
the curves in this order:
blue, red and green for
the best results.
Interior curves:
Sets the tolerance
for the interior
curves. The interior
