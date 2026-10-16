# Planar Curves ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 197-197
- Section title: Planar Curves ￼
- Section id: m8book1-213-planar-curves
- Kind: command
- Images (../images/): m8book1_p197_img01.png

---
<!-- section m8book1-213-planar-curves | Planar Curves ￼ | m8book1 pages 197-197 -->

<!-- page 197 -->

### Patch Options:
## Planar Curves
General:
Sample Point Spacing: The nominal 3D
distance between points sampled from input
curves. Minimum of 8 points per curve.
‘PlanarSrf’
Create surfaces using planar curves, polylines or
surface edges (on a single Construction plane).
A single, flat surface on the same Construction
plane is created between the curves.
Surface U Spans: This indicates the U direction
span count for the automatically generated
surface. Increase the amount for a more complex,
tight-fitting surface.
Where is this Command:
Access this command from Surface>Planar
Curves
Steps:
Surface V Spans: This indicates the V direction
span count for the automatically generated
surface. Increase the amount for a more complex,
tight-fitting surface.
• Select Planar Curves from the Surface menu.
• Select planar curves, polylines or edges to build
the surface.
• Press Enter when done.
Stiffness: This setting tells how much is allowed
for best fit plane to deform. The larger the number,
the ‘stiffer’ and more planar the resulting surface
is.
Adjust Tangency: Match to the tangent
direction of surfaces if the input curves are edges
of existing surfaces.
Automatic Trim: This setting defaults to ‘ON’. It
tries to find an outside curve and trims the surface
to it.
Surfaces created with Planar Curves
Starting Surface:
Select Starting Surface: Click to select a
starting surface where a surface has already been
created similar in shape to the surface desired with
‘Patch’.
Starting Surface Pull: This is similar to Stiffness,
but it applies to the starting surface. The larger the
pull, the closer the resulting surface shape will be
to the starting surface.
Preserve Edges: This clamps the edges of the
starting surface in place. It’s useful when you don’t
want the starting surface to move.
Note:
• Curves may be joined, polylines or curves, or
merely touching (having Position Continuity), but
they must be on the same plane.
• Two or more curves that do not touch and are not
planar will not form a surface.
• If two closed curves partially overlap (intersect),
each curve produces a separate surface.
• If a closed curve is wholly in another closed curve
then it will be treated as a hole.
• If two closed curves on two or more different
planes are selected, two separate surfaces will be
made.
Delete Input: The starting surface is deleted
after the new surface is made.
