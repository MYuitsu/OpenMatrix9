# Extend Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 214-215
- Section title: Extend Surface ￼
- Section id: m8book1-232-extend-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p214_img01.png, m8book1_p214_img02.png, m8book1_p214_img03.png, m8book1_p215_img01.png, m8book1_p215_img02.png, m8book1_p215_img03.png

---
<!-- section m8book1-232-extend-surface | Extend Surface ￼ | m8book1 pages 214-215 -->

<!-- page 214 -->

## Split at Isocurve
### Command Line Options:
‘Split’ Isocurve
Divides a single surface into parts using an
isoparametric curve (isocurve) as a cutter.
Direction = U, V or Both: U: Splits a surface
with isocurves in the surface u-direction. V: Splits
a surface with isocurves in the surface v-direction.
Both: Splits a surface with isocurves in both U and
V directions.
Where is this Command:
Access this command from Surface>Split at
Isocurve
Toggle: Toggles the direction between U and V.
Steps:
Shrink (Surfaces only) = Yes or No: Yes:
Shrinks the underlying untrimmed surface close
to the trimming boundaries similar to the Shrink
Trimmed Surface command. No: Does not shrink
the underlying untrimmed surface.
• Select Split at Isocurve from the Surface menu.
• Select a single surface to split.
• Press Enter when done.
• Pick isocurves (click to set) along the surface to
split along.
• Press Enter to end the command.
## Extend Surface
Notes: The Isocurve option only appears when a
single surface is selected.
‘ExtendSrf’
Moves a surface edge to lengthen a surface.
Where is this Command:
Access this command from Surface>Extend
Surface
Steps:
Pick isocurves along the
surface to use for splitting
the surface. Click to set the
isocurve location for each.
• Select Extend Surface from the Surface menu.
• Select edge of surface to extend. Specify an
option.
• Enter a value for the extension factor in mm.
With Split at Isocurve, a
single surface is split with
two isocurves to make
three surfaces.
Selecting a
surface edge in
Extend Surface.
Preview of the
extension (the
last extension
value used) is
shown (right) and
original surface
(left).
Note: The new surface cannot be exploded
from the original.

<!-- page 215 -->

### Command Line Options:
Type:
Smooth: Extends the surface smoothly curving
from the edge.
Line: Extends the surface in a straight line from
the edge.
## Extrude Normal to Surface
‘Fin’
Extrudes a curve on a surface in the direction of
the surface normal or tangent to the surface.
Picking a point to set the height (left) and an
extrusion with multiple heights is complete (right).
Where is this Command:
Access this command from Surface>Extrude
Normal to Surface
## Unroll Surface
Steps
‘UnrollSrf’
Use this command to produce a model flat, as
pattern, to fashion into a curved surface after
production. Surfaces that curve in two directions
such as spheres or those created with curved
profiles cannot be unrolled.
Where is this Command:
Access this command from Surface>Unroll
Surface
Steps:
• Select Unroll Surface from the Surface menu.
• Select a surfaces or polysurface to unroll. Specify
the Command Line option.
• Select any curves on the surface to unroll.
• Select Extrude Normal to Surface from the
Surface menu.
• Select a curve on the surface.
• Select the surface (base). Specify a Command
Line option.
• Pick a location on the curve for a height. Drag
your cursor and click to set the height or enter
the Distance (height) in the Command Line. Enter
a positive value for an outward extrusion and a
negative value for an extrusion below the base
surface. (For a simple, uniform extrusion, just enter
a Distance amount without picking a point on the
curve, press Enter and press Enter again.)
• Pick a location to set another height, or press
Enter if the surface should be uniform in height.
Continue picking curve locations to set another
height.
Press Enter when done.
### Command Line Options:
SetBasePoint: Specify a location that serves as
the first point when picking two points that set
the extrusion distance.
Direction:
Normal: Creates the surface in the surface normal
direction.
Unroll Surface with Explode = Yes shows each single
surface unrolled flat.
Tangent: Creates the surface tangent to the
surface.
