# Rail Revolve ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 202-203
- Section title: Rail Revolve ￼
- Section id: m8book1-218-rail-revolve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p202_img01.png, m8book1_p202_img02.png, m8book1_p203_img01.png, m8book1_p203_img02.png

---
<!-- section m8book1-218-rail-revolve | Rail Revolve ￼ | m8book1 pages 202-203 -->

<!-- page 202 -->

Rail Revolve example pictured: This is a typical
set up for Rail Revolve.
surface. Specify how many points in that direction.
Deformable revolves can be deformed smoothly
with point editing.
PointCount: When Deformable is set to ‘Yes’, this
specifies the number of control points.
FullCircle: Revolves the input curve 360 degrees
as a shortcut for specifying 360 degrees as the
revolve angle. Using this option also sets the next
use of the command to 360 as the Revolve angle.
AskForStartAngle Yes/No: No; the revolve
starts from 0 (the input curve location). Yes; pick
the angle (a number of degrees away from the
current input curve location) the revolve will start.
• The path curve is a closed curve shown in yellow.
• The profile or shape curve is an open curve inside
the closed, path curve, shown in red.
• The start of the Rail Revolve axis is the bottom of
the green line (this point lines up on the same
plane as the path curve and bottom of the profile
curve).
• The end of the Rail Revolve axis is the top of the
green line where the profile meets.
• Set up the profile and path curve so they are
positioned perpendicular to each other for best
results.
• The axis curve (green) should ideally be located in
the center of the closed, rail curve. The surface is
made around this curve.
• Closed rail curves give the best results. However,
an open curve may be used.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined tangent curves.
## Rail Revolve
‘RailRevolve’
Creates a surface or polysurface by revolving a
profile, shape, curve around a path curve that runs
perpendicular to the profile.
Where is this Command:
Access this command from Surface>Rail
Revolve
Steps:
Rail Revolve curves (top) and completed surface
(bottom). The gray curves, at both ends of the red
profile curve represent tick marks used to create the
blend curve (red) used for the profile.
(See Curve menu > Quick Curve Blend or
Adjustable Curve Blend)
### Command Line Options:
• Select Rail Revolve from the Surface menu.
• Select a profile curve. (This profile curve must be
placed inside of a closed rail curve.)
• Select a rail (path) curve.
• Pick the start of the Rail Revolve axis. (Choose a
point in the center of the closed, rail curve)
• Pick the end of the Rail Revolve axis. (Pick a point
in line with the start point).
ScaleHeight: The profile curve, in addition to
being revolved, stretches along the revolve axis
using the revolve axis origin as the scale base
point. The distance from the revolve axis origin to
the path curve start point along the revolve axis
is the primary reference height, and the distances
from the revolve axis origin to path curve control
points along the revolve axis are the secondary

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
