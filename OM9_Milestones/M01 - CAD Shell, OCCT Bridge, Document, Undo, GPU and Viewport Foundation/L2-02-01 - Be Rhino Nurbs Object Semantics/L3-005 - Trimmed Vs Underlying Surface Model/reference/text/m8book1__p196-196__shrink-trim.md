---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 196-196
section_title: Shrink Trimmed Surface ￼
section_id: m8book1-211-shrink-trimmed-surface
---
# Shrink Trimmed Surface ￼

Source: matrix8_book_1.pdf, pages 196-196

<!-- section m8book1-211-shrink-trimmed-surface | Shrink Trimmed Surface ￼ | m8book1 pages 196-196 -->

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
