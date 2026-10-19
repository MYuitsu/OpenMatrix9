# Fillet Curves

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 155-156
- Section title: Fillet Curves
- Section id: m8book1-162-fillet-curves
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p155_img01.png, m8book1_p155_img02.png, m8book1_p156_img01.png, m8book1_p156_img02.png, m8book1_p156_img03.png

---
<!-- section m8book1-162-fillet-curves | Fillet Curves | m8book1 pages 155-156 -->

<!-- page 155 -->

Steps:
## Fillet Curves
• Select Extract Isocurve from the Curve menu.
• Select a surface.
• Pick a place to extract a curve on the selected
surface.
‘Fillet’
The Fillet command creates a smooth arc at a
sharp “joint” between two curves. The two curves
can be; joined; touching but not joined; not
touching but would form a sharp angle if they did.
Note: The cursor is constrained to the surface
selected at the beginning of the command. Use
O-Snaps to determine a precise location for the
extracted curve.
Where is this Command:
Access this command from Curve>Fillet
Steps:
The new curve is created on the current layer color.
Change the layer color to a contrasting color so the
new curve is easily discernible.
• Select Fillet from the Curve menu.
• Select the first curve near the end.
• Select the second curve near the end.
Note: The Fillet can fail of the value for the radius
is larger than the distance the Fillet can
accommodate.
Extracting isocurves on a ring’s surface.
### Command Line Options:
Direction: Specify the direction. U, V or both
directions (perpendicular to one another)..
Toggle: Toggles the direction between U and V.
ExtractAll: Extracts all isocurves in U, V, or both
directions depending on the Direction option.
The top two points have been filleted with a radius of
.125. The rest haven’t been filleted.
### Command Line Options:
Radius: Allows you to set the fillet radius.
IgnoreTrims:
No: Surface trims are taken into
consideration. When the marker is off the
visible part of the surface, the no access
cursor is shown
Join: Joins the resulting curves.
Yes: Surface trims are ignored. When the marker
misses the untrimmed surface, the no-
access cursor is shown.
Trim: Trims the input curves with the resulting
curve.

<!-- page 156 -->

## Chamfer Curves
‘Chamfer’
The Chamfer command creates a beveled edge
between two curves that meet at a point.
Where is this Command:
Access this command from Curve>Chamfer
Steps:
ExtendArcsBy: Applies when arcs are filleted or
chamfered but need to be extended to meet the
fillet curve or chamfer line.
• Select the first curve near the end for the chamfer.
• Select the second curve near the end for the
chamfer.
Arc: The arc is extended maintaining validity.
Line: The arc is extended with a line segment,
which is joined to the arc converting it to a
polycurve.
## Fillet Corners
‘FilletCorners’
Rounds all corners of polycurves or polylines with
arcs of a single specified radius.
Where is this Command:
Access this command from Curve>Fillet
Corners
The top 4 points of the star have been chamfer to a
distance of .2mm
Steps:
• Select Fillet Corners from the Curve menu.
• Select the polycurve and press enter.
• Specify the fillet radius
### Command Line Options:
Distances: The distance from the intersection of
the curves to the chamfer.
Join: Joins the resulting curves.
Trim: Trims the in put curves with the resulting
curve.
ExtendArcsBy: Applies when arcs are filleted or
chamfered but need to be extended to meet the
fillet curve or chamfer line.
Arcs: The arc is extended maintaining validity.
All corners of a polycurve are filleted at once.
