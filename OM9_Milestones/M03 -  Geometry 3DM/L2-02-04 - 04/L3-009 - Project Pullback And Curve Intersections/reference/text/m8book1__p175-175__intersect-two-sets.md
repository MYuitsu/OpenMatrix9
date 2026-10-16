# Intersect Two Sets

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 175-175
- Section title: Intersect Two Sets
- Section id: m8book1-199-intersect-two-sets
- Kind: command
- Images (../images/): m8book1_p175_img01.png, m8book1_p175_img02.png

---
<!-- section m8book1-199-intersect-two-sets | Intersect Two Sets | m8book1 pages 175-175 -->

<!-- page 175 -->

## Intersect Two Sets
‘IntersectTwoSets’
Finds the intersection of one set of objects with
another set of objects. Intersections among
objects within each set are ignored. When finding
intersects between curves, point objects are
placed at the intersections.
Where is this Command:
Access this command from Curve>Intersection
Two Sets
Steps:
Steps
• Select Arc Blend from the Curve menu.
• Select the first curve near the end you want to
blend.
• Select the second curve near the end you want to
blend.
• Select the arc blend points to adjust.
• Drag the point to set the arc control polygon
length ratio. Type zero or snap to ends to minimize
radius difference.
• Type zero to automatically pick a ratio that makes
the arc radii the same when possible and as close
to being the same as possible otherwise.
• Select another arc blend point to adjust, or press
Enter to end the command.
• Select Intersect Two Sets from the Curve menu.
• Select objects got the first set.
• Select objects for the second set.
Note: Use the group command or layers to make
selection easier.
Adjusting the Arc Blend handles before confirming
the arc.
### Command Line Options:
AlternateSolution: Offers the longer solution
when more than one arc fills the requirements.
RadiusDifference: Makes it possible to draw an
S-shaped blend where the difference of the two
radii is specified.
If the difference is positive the first radius will be
bigger than second, and if negative the first radius
will be smaller.
Trim: Trims the input curves to the resulting
curve.
Join: Joins the resulting curves
