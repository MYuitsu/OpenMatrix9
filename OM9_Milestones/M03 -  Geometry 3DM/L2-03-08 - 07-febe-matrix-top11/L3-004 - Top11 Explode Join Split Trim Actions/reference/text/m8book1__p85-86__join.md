# Join

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 85-86
- Section title: Join
- Section id: m8book1-070-join
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p85_img01.png, m8book1_p85_img02.png, m8book1_p86_img01.png, m8book1_p86_img02.png

---
<!-- section m8book1-070-join | Join | m8book1 pages 85-86 -->

<!-- page 85 -->

Where is this Command:
Access this command from Top 11
Commands>Explode
is called Position Continuity. As long as objects
have position continuity, they will Join. This is
why it is SO important to use O-Snaps during
modeling. O-Snaps, especially the End O-Snap,
can ensure that two objects have position
continuity.
Note: If an object is a single curve or a single
surface already, it cannot be exploded.
Steps:
Where is this Command:
Access this command from Top 11
Commands>Join
• Select Explode from the Top 11 menu.
• Select the Object(s) to Explode.
• Press Enter when done.
Steps:
### Explode breaks down objects as
### follows:
• Select Join from the Top 11 menu.
• Select the Objects to Join.
• Press Enter when done.
Blocks: Block Instances are spared from the
Explode button. Block instances will not function
as Blocks in Matrix if exploded. (Explode must be
typed in the Command Line to truly explode a
Block Instance)
Dimensions: Curves and text.
Groups: Explodes objects contained in the
group, but leaves the objects grouped.
Before joining, on the left, the model is comprised of 3
separate surfaces. After joining the model is a polysurface
made up of 3 surfaces
### Command Line Options:
Mesh: Mesh parts and mesh faces based
on unwelded edges. If a mesh is completely
unwelded, then it will explode to its individual
faces.
Undo: Reverse the last action
Cage controls: Curves, surfaces, cages.
Polysurface: Surfaces.
Polycurve: Single segment curves.
Notes for curves:
You can join curves that are arranged sequentially.
When Joining curves, the old curves are discarded
and a new one added. The coordinates of the two
joined ends are replaced with a single coordinate
half way between the two original ends.
Text: Curves.
## Join
Notes for surfaces:
You can join surfaces and polysurfaces that touch
at naked edges. Joining does not change the
underlying surface geometry. It simply “glues”
adjacent surfaces together so meshing, Boolean
operations, and intersections can go across the
seam without gaps.
‘Join’
The Join command joins surfaces and curves
together into one object. In order to be joined, the
objects much touch in space: that is, the start or
end point of one object must line up with the start
or end point of another object so that they have
the same exact grid coordinates. This relationship
To change two adjacent surfaces into a single
surface, use Merge Surface Pay special attention

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
