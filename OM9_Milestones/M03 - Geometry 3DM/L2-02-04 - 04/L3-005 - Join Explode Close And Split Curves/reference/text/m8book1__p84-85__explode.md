# Explode

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 84-85
- Section title: Explode
- Section id: m8book1-069-explode
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p84_img01.png, m8book1_p84_img02.png, m8book1_p85_img01.png, m8book1_p85_img02.png

---
<!-- section m8book1-069-explode | Explode | m8book1 pages 84-85 -->

<!-- page 84 -->

### Command Line Options:
## Move
Vertical: Moves the object vertical to the current
Construction plane.
## Rotate
‘Rotate’
The Rotate command rotates objects in a
360-degree radius around a user-selected point
in space.
Where is this Command:
Access this command from Top 11
Commands>Rotate
‘Move’
The Move command prompts the user to move
an object from one location to another in the
viewports. This is different from simply clicking on
an object or its control points and moving them
with the cursor. Because the Move command
prompts the user for a “Point to move from” and
a “Point to move to”, the O-Snaps may be used to
select an object exactly at the point from which
it needs to be moved and move it precisely to a
new location where it is needed. This precision is
especially useful when the moved object will be
joined to another after the move.
Steps:
Where is this Command:
Access this command from Top 11
Commands>Move
• Select Rotate from the Top 11 menu.
• Select the Objects to rotate.
• Pick the center of rotation.
• Type the rotation angle in the Command Line, or
pick two Reference points.
Steps:
• Select Move from the Top 11 menu.
• Select the Object(s) to Move.
• Press Enter when done.
• Pick a Point to move from.
• Pick a Point to move to.
The head and gem are being rotated together
from their shared center.
### Command Line Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
## Explode
Move objects precisely from one point to another.
‘Explode’
This command separates a polysurface (an object
made up of many separate surfaces, joined) or a
polyline curve (made up of many curves joined
together) into its component objects. There won’t
be any visible change in the model, but each
component item is now able to be selected. To re-
join the objects, select them all and click “Join”.

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
