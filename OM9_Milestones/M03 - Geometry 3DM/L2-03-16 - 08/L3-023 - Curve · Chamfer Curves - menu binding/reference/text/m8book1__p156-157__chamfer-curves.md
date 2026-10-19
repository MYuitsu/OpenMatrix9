# Chamfer Curves

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 156-157
- Section title: Chamfer Curves
- Section id: m8book1-164-chamfer-curves
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p156_img01.png, m8book1_p156_img02.png, m8book1_p156_img03.png, m8book1_p157_img01.png, m8book1_p157_img02.png, m8book1_p157_img03.png

---
<!-- section m8book1-164-chamfer-curves | Chamfer Curves | m8book1 pages 156-157 -->

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

<!-- page 157 -->

Where is this Command:
Access this command from Curve>Apply UV
Curves
Line: The arc is extended with a line segment,
which is joined to the arc converting it to a
polycurve.
Steps:
## Create UV Curves
• Select planar curves on world XY plane to apply to
a surface.
• Select a surface to apply the planar curves to.
• The curves are mapped to the surface.
Corresponding 3-D curves are created on the
surface.
‘CreateUVCrv’
Projects a surface’s untrimmed boundary and
trim curves onto the world x-y plane. The results
can be used in conjunction with Apply UV Curve
or Flow Along Surface. Use the results as a 2D
pattern on which to layout design details and then
return onto the 3D surface.
Note: Use the CreateUVCrv command to generate
planar versions of the u and v curves of a surface
so you can use them as a guide to orient your text
or pattern. Then Apply UV curves to the original
surface.
Where is this Command:
Access this command from Curve>Create UV
Curves
Steps:
• Select Create UV Curves from the Curve menu.
• Select a surface.
Note: The control polygon determines the sizes of
the uv curves.
Create UV Curves works with single untrimmed
surfaces.
A pattern is created and placed on the UV curve.
Then the Apply UV Curve command is used to place
the pattern onto the rings surface.
## Through Points
‘CurveThroughPt’
Fits a Curve through selected point objects.
Where is this Command:
Access this command from Curve>Through
Points
The UV curves are created from the ring.
Steps
• Select Through Points from the Curve menu.
• Select points.
## Apply UV Curves
‘ApplyCrv’
Wrap planar curves onto a surface. Use the
Create UV Curves prior to this command to create
template on which to design then use the Apply
UV Curves command to transfer the planar curves
onto the 3D surface.
