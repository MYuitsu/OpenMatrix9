# Split

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 86-86
- Section title: Split
- Section id: m8book1-071-split
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p86_img01.png, m8book1_p86_img02.png

---
<!-- section m8book1-071-split | Split | m8book1 pages 86-86 -->

<!-- page 86 -->

to the setting of the Smooth option to get the
geometry you want.
own isoparametric curves. This option only
appears when a single surface is selected.
## Split
Split single surface with an isocurve.
‘Split’
The Split command will split curves, surfaces, and
polysurfaces. This command assigns one object
(or group of objects) as the object(s) to be cut
and another object (or group of objects) as the
objects doing the cutting. Multiple objects may be
selected during the Split command: for instance,
one object may be split with multiple cutters; or
multiple objects may be split with a single cutter.
Note: Use the Intersection O-Snap for snapping to
isocurve intersections.
### Direction
Where is this Command:
Access this command from Top 11
Commands>Split
U: Splits a surface with an isocurve in the surface
U direction.
Steps:
V: Splits a surface with an isocurve in the surface
V direction.
Both: Splits a surface with an isocurve in both U
and V directions.
• Select Split from the Top 11 menu.
• Select the Objects. (Multiple objects can be
selected to split with multiple objects.)
• Select Cutting objects. Deselect cutting objects
with the “Ctrl” Key.
• Press Enter to end the command.
Toggle: Toggles the direction between U and V.
Shrink (Surfaces Only):
Yes: Shrinks the underlying untrimmed surface
close to trimming boundaries similar to the Shrink
Trimmed Surface command.
Notes:
• The Isocurve option only appears when a single
surface is selected.
• Use Untrim to remove a trimming boundary from a
surface.
No: Does not shrink the underlying surface.
### Options:
## Trim
Point (Curves Only): Pick locations on the
curve. This option only appears when a curve is
selected.
Split a curve with a point. Closed curves must be split at
two or more locations.
‘Trim’
The Trim command allows the user to select a
number of objects and, at every point where they
intersect, a Split point will be made. When using
Trim, each object is used to split another, meaning
that there are no distinctions between “cutting
objects” and “objects to be cut”: each object
is split at the point where it intersects another
one. During the Trim command, the user can left
mouse-click on any part of an object that was
Isocurve (Surfaces only): Split an object by its
