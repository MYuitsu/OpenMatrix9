# Sweep 4 ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 203-203
- Section title: Sweep 4 ￼
- Section id: m8book1-219-sweep-4
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p203_img01.png, m8book1_p203_img02.png

---
<!-- section m8book1-219-sweep-4 | Sweep 4 ￼ | m8book1 pages 203-203 -->

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
