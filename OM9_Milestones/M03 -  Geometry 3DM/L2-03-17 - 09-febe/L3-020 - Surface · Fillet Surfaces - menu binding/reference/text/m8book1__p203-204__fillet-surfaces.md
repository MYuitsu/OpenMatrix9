# Fillet Surfaces ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 203-204
- Section title: Fillet Surfaces ￼
- Section id: m8book1-220-fillet-surfaces
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p203_img01.png, m8book1_p203_img02.png, m8book1_p204_img01.png, m8book1_p204_img02.png, m8book1_p204_img03.png

---
<!-- section m8book1-220-fillet-surfaces | Fillet Surfaces ￼ | m8book1 pages 203-204 -->

<!-- page 203 -->

reference heights. You do not need ScaleHeight
option if the rail curve is planar and perpendicular
to the revolve axis. Use the ScaleHeight option
if the rail curve is not on a plane perpendicular
to the revolve axis and you want a part of the
profile curve to only revolve around the revolve
axis, not to move along the revolve axis as it is
revolved. This is the case if you want to build rail-
revolved surfaces with smooth round ends even
if the rail curve is not planar. With the ScaleHeight
option, the revolve axis origin location matters.
The revolve axis origin is also the scale origin. The
one-dimensional scaling happens parallel to the
revolve axis. If the rail curve is closed, the seam
must be where the profile touches the rail for
good results.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined tangent curves.
editable. The bounding box connects to the four
rails. But, the profiles will not necessarily touch all
four rails, depending on their shape.
• Sweep 4 is a history enable command so you can
select any of the profiles and use MSR to edit the
profiles.
• Matrix History must be turned off before the
Sweep 4 command is run to view updated edits.
• This command is unique so that when any of
the 4 rail curves are edited, the profiles that are
connected to them update along with the output
surface. This allows you to quickly create complex
surfaces that previously would have required
knowledge of more advanced surface modeling
techniques.
• Edit the rail curves by manipulating the control
points or edit points directly. Or, if the rail curve
is a ‘child’ of another set of objects, just edit the
‘parent’ object. (See Info & Settings > Matrix
History)
## Sweep 4
Four Rail Profiles are
selected for create a
Sweep 4.
‘gvFourProfileSweep’
This History-enabled command creates a
polysurface using 4 rail curves with attached
profiles that were created with the Four Rail
Profile command in the Tools menu.
Where is this Command:
Access this command from Surface>Sweep 4
Or, in Four Rail Profile
Note: Begin by placing profiles on the rails using
Four Rail Profile command (Tools menu).
Edit  the rail’s control
points and see the
updates.
Steps:
• Select Sweep 4 from the Surface menu.
• Select the profiles attached to the rails.
• Edit the rail control points and see the Sweep
update after each edit. (Matrix History must be
turned on when the command is first run to see
each edit.)
## Fillet Surfaces
Note:
‘FilletSrf’
Adds a tangent, curved surface between two
joined or unjoined surfaces.
• Use Four Rail Profile in advance to place profiles
on four rail curves. The profiles are attached to
the rails based on a bounding box. This bounding
box can be seen when the profiles are active and

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
