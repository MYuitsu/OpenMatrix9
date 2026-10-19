# Curve Network ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 195-196
- Section title: Curve Network ￼
- Section id: m8book1-210-curve-network
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p195_img01.png, m8book1_p195_img02.png, m8book1_p195_img03.png, m8book1_p195_img04.png, m8book1_p195_img05.png, m8book1_p196_img01.png, m8book1_p196_img02.png

---
<!-- section m8book1-210-curve-network | Curve Network ￼ | m8book1 pages 195-196 -->

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

<!-- page 196 -->

of the curve’s surface will be within this value. If
the curves themselves are farther apart from each
other than the tolerance values, the best guess is
made at the surface.
Angle: If the edge curves are surface edges,
and you want the surface matching the adjacent
surfaces with tangency or curvature continuity,
this is the accuracy used to match the surface
normals.
Preview: Check to dynamically display a preview
as the options change.
Edge Matching: Determines how the edges
match the input geometry.
Surface before (left) and after (right) using Shrink
Trimmed Surface displaying Control Points and UV
Curves.
## Patch
Loose: The surface will match to the input edge
curves with less accuracy.
Position / Tangency / Curvature
See the full details in Surface menu> Blend
Surface> Continuity Options.
‘Patch’
This command creates a surface using edge
curves and/ or point objects. If a surface has been
split with a curve, leaving a hole, Patch can be
used to repair it.
## Shrink Trimmed Surface
Where is this Command:
Access this command from Surface>Patch
Note: Patch may not give the best results for every
shape of hole. It does not hug the edges as well
as the Sweep commands. Use Sweep or Curve
Network for more control of the output surface
shape, if needed.
Steps
‘ShrinkTrimmedSrf’
This command allows you to make a trimmed
surface forget it ever was trimmed, becoming an
“original” surface. As though it was created this
way from the start. In that way Smart Flow,
Control Point Editing or Create UV Curves will
work as expected.
Where is this Command:
Access this command from Surface>Shrink
Trimmed Surface
• Select Patch from the Surface menu.
• Select curves, points, point clouds and meshes to
fit surface through.
• Press Enter when done.
• Select a setting from the Patch Options dialog box.
• Click ‘OK’ to complete the command.
Steps:
• Select Shrink Trimmed Surface from the Surface
menu.
• Select trimmed surfaces to shrink.
• Press Enter when done.
Using ‘Patch’ to
create surfaces
where holes exist
in a shank.
Note: A trimmed surface that has been used in a
Boolean, Trim, or Split operation retains a memory of
the surface it was originally.
