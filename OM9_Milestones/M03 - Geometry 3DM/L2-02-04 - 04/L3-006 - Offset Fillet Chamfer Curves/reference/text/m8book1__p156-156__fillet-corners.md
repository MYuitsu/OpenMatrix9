# Fillet Corners

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 156-156
- Section title: Fillet Corners
- Section id: m8book1-163-fillet-corners
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p156_img01.png, m8book1_p156_img02.png, m8book1_p156_img03.png

---
<!-- section m8book1-163-fillet-corners | Fillet Corners | m8book1 pages 156-156 -->

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
