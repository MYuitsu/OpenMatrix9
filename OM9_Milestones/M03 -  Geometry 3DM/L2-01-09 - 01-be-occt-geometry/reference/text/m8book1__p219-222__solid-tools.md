# Solid Tools

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 219-222
- Section title: Solid Tools
- Section id: m8book1-239-solid-tools
- Kind: chapter_intro
- Images (../images/): m8book1_p219_img01.png, m8book1_p219_img02.png, m8book1_p219_img03.png, m8book1_p219_img04.png, m8book1_p219_img05.png, m8book1_p219_img06.png, m8book1_p220_img01.png, m8book1_p220_img02.png, m8book1_p220_img03.png, m8book1_p220_img04.png, m8book1_p221_img01.png, m8book1_p221_img02.png, m8book1_p221_img03.png, m8book1_p221_img04.png, m8book1_p221_img05.png, m8book1_p221_img06.png, m8book1_p222_img01.png, m8book1_p222_img02.png, m8book1_p222_img03.png

---
<!-- section m8book1-239-solid-tools | Solid Tools | m8book1 pages 219-222 -->

<!-- page 219 -->

# CHAPTER 21
# Solid Tools
### Learn to create closed “watertight” surfaces known as Solids and how to join or cut
### them using the Boolean operations.
selected Boolean operation. However, any of the
three Boolean operations found in the Solid menu
will have the same results as their counterparts in
the builder:
Modeling with Solids
Solids are a closed surface. Recall that a solid is a
closed surface; or, a surface with no open edges
(edges that aren’t joined to other edges) that
encloses a volume. Solids may have more than
one side, making them polysurfaces, like the box,
pipe, cylinder, and cone tools. Or, a solid may be a
single surface, like a sphere, torus, or tube. All of
these solid objects may be drawn using the tools
in the Solid menu. Also found in this menu are
tools that control solids, like Booleans, explained
below.
Intersecting Solids (Left)
Boolean Union
(Right)
Boolean Difference (Above)
The box on the left is a polysurface. Where as the torus
is created from a single surface. This affects the types of
commands that can be used on them.
Boolean Intersection:
About Booleans
There are three Boolean operations available in
the Solid menu: Boolean Union, which joins two
solids; Boolean Difference, which removes a
section of a solid with another solid that intersects
it; and Boolean Intersection, which makes a new
solid at the point where two solids intersect. It
may be more convenient to perform these same
operations with Boolean Builder, found in the
Tools fly-out menu, which lets the user select the
objects on which the Boolean operation will be
performed and place them into two separate
object boxes. The builder then performs the

<!-- page 220 -->

When in doubt about whether or not the surfaces
are coplanar, use the following trick. Select both
surfaces and click on Intersect in the Curve menu.
If the intersection command cannot locate the
place where the two objects intersect, Booleans
won’t be able to, either.
Notice that, unlike Split or Trim, Booleans are the
only operations that, when used with two solid
objects (closed surfaces or polysurfaces), produce
a solid following the operation. This can
sometimes make Booleans trickier to use then
other tools like Split and Trim; however, since solid,
watertight models are often required for certain
rapid prototyping machines, Booleans are often
inevitable.
Sink the cutter into the surface of the ring. Run the
Intersect command again and a resulting curve
will be drawn and “Found One Intersection” will
appear in Feedback line. Now try Booleans.
Sphere split with a curve results as an open surface
(Left)  Sphere cut using Boolean Difference results
as a solid object (closed polysurface). (Right)
If it is absolutely necessary that both objects
are located in space as close as possible to their
present positions, use the Boolean Builder in
the Tools menu and set the AutoMove and/
or AutoScale options to move (or scale) one of
the objects ever so slightly. This causes them to
intersect with one another on a smaller scale than
above, and the builder moves them for the user so
that a trial and error process is not necessary.
Tips and Tricks for Successful Booleans:
1: Make sure that objects are not coplanar.
Booleans will always fail when objects are co-
planar, or, when they have edges that share the
same grid coordinates in space. In the following
example, Booleans will fail because the cutter and
the ring are co-planar (on the left, below), meaning
that the edge of the cutter is inhabiting the same
grid space as the edge of the ring. Booleans
become confused over which edge to keep
when two edges are in the same exact location in
space. Therefore, the cutter must be plunged into
the surface of the ring, not merely touching the
surface of the ring, for Booleans to work properly
(right-hand illustration).
2: Use Object Checker, Show Edges, Join
Edge: Booleans work best on solids (closed
polysurfaces or closed Nurbs surfaces). Although
they can be used with open polysurfaces, this is
NOT recommended. Running the Object Checker
in the Tools menu to see whether the object
passes both checks will help the user find out
whether or not the object will pass Booleans.
If the object fails either of the checks in Object
Checker, Booleans will be unsuccessful. However,
passing both of these checks does not ensure a
successful Boolean (read on to find out why).
If the objects do not pass Object Checker, it is a
good idea to select the object and use the Show
Edges function in the Utilities menu to highlight
any “naked edges”, or, open surfaces. When you
use the Boolean Builder from the tools menu the
builder automatically checks that the objects are
valid and will yield the expected results.
Naked Edges can occur at any point in the
modeling process and while it is always best to
avoid Naked Edges. Rather than starting over
from scratch, click on Join Edge from the Utilities
menu and select the errant edges. The Edge
Joining dialog box will appear. Click on Yes to
reconnect the edges. When the Naked Edge has
Objects are coplanar and Boolean will fail (Left).
Object sunk into the surface of the band Boolean will
succeed. Note: If the Feedback line reads “Found no
intersections”, objects are either coplanar or they
don’t touch at all.

<!-- page 221 -->

been healed with Edge Joining, it will not be
highlighted when the Object Checker is run the
next time.
This one is pretty scary: the Edge Analysis thinks that
the original curve is the edge of the surface, even
though the surface extends outside the curve!
(2) A surface with a twist:
To fix this model, rebuild it with Seam Point Arrows
adjusted correctly.
If larger gaps appear in you model between
Naked Edges a different approach may be
required. Loft, Sweep 2, Edge Curves and Patch
are all tools that can be used to repair a hole in a
surface. These each create a separate surface that
will need to be joined to the original to create a
water tight solid.
(3) A surface with a “kink” in which the
model folds in on itself.
Highlight the model to see intersecting surfaces.
Fix this by placing two rails instead of one and
using Sweep 2.
3: Make sure to create valid surfaces
during modeling.
Booleans will fail when objects are invalid. An
invalid surface is one that intersects itself or twists
through itself. Sometimes, invalid surfaces even
pass the Object Checker, which can make them
tricky to find until Booleans fail. By then, it may
take a lot of work to go back and fix an invalid
surface. Therefore, be very careful when modeling
not to use curves that intersect themselves,
surfaces that twist, or surfaces with “kinks”. These
break models and make them invalid so that
Booleans will not know what to do with them.
Booleans will fail with the following invalid
surfaces:
(1) A surface drawn from a curve that
intersects itself:
Original Curve:
Resulting surface:

<!-- page 222 -->

### Skills & Commands in this Chapter
Booleans
Boolean Union, Boolean Difference, Boolean
Intersection, Boolean Two Objects
4: Check Surface Normals using Directions:
The reason that Booleans are only recommended
for solid objects is because Booleans look for the
surface normals, or directions, on objects to be
pointing outward. On a solid, the normals are
always pointing outward. Yet on an open surface,
its normals may be pointing outward or inward, so
the results with Booleans may be unexpected.
Primitives
Pipe, Box, Sphere, Ellipsoid, Torus, Cylinder,
Tube, Pyramid, Cone, Truncated Cone
Other Solid Creation Tools:
Text,Extrude Curve, Solid Extrude All, Boss, Rib,
Slab
Surface normals “out” (Left)  Surface normals “in”
(Right)
Editing Surfaces:
Cap Planar, Extract Surface, Fillet, Array Hole,
Array Hole Polar, Move Hole, Move Face, Shell
For successful Booleans, the surface normals
should point outward. If the results of a Boolean
turn out opposite from what was expected (i.e.
Boolean Difference is selected but Boolean
Union is performed, or vice versa), first reverse the
direction of the surface normals using Direction
in the Utilities menu. To do so, select the surfaces
and run the Direction command so the direction
arrows are visible, click once on the surface to flip
the direction of the arrows. Press Enter to end the
Directions command. Try the Boolean operation
again with the surface normals on both objects
pointing out. When working with open surfaces
(surfaces with Naked Edges), Booleans are NOT
recommended. Try Join or Split instead.
