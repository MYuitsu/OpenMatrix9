# Surface Tools

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 176-179
- Section title: Surface Tools
- Section id: m8book1-200-surface-tools
- Kind: chapter_intro
- Images (../images/): m8book1_p176_img01.png, m8book1_p176_img02.png, m8book1_p176_img03.png, m8book1_p176_img04.png, m8book1_p176_img05.png, m8book1_p177_img01.png, m8book1_p177_img02.png, m8book1_p177_img03.png, m8book1_p177_img04.png, m8book1_p178_img01.png, m8book1_p178_img02.png, m8book1_p178_img03.png, m8book1_p178_img04.png, m8book1_p178_img05.png, m8book1_p178_img06.png, m8book1_p179_img01.png, m8book1_p179_img02.png

---
<!-- section m8book1-200-surface-tools | Surface Tools | m8book1 pages 176-179 -->

<!-- page 176 -->

# CHAPTER 20
# Surface Tools
### Learn how to create surfaces. Which are fundamental building blocks that create
### solid objects. Understand the difference between a Nurbs surface and a polysurface.
Closed surfaces are also known as Solids. Closed
single surfaces are objects like a Sphere, Torus, or
Ellipsoid (found in the Solid menu). They are single
surfaces because they have no edges, no sharp
angles, and may therefore be created using one
surface. They cannot be exploded into
component surfaces. A solid object encloses a
volume, and different commands can be run on
single-surface solids and closed polysurface
solids. Refer to the Solid menu to learn more
about these.
Types of Surfaces:
A single, open surface in Matrix is an object with
length and width but no depth (thickness). An
open surface may be flat, it may curve in space, or
it may be tubelike, as if a straight, flat surface in
space had been rolled up so that two of its edges
meet each other, like a rolled-up newspaper (the
other two edges are still showing, making it an
open, single surface. A polysurface is comprised
of more than one surface that has been Joined.
Therefore, when a polysurface is Exploded, more
than one surface remains. An open surface
encloses no volume, meaning that it is only one
point object thick.
Example of three Closed NURBS Surfaces found in
the Solid menu. Ellipsoid, Torus and Sphere.
The tube on the left and plane on the right are both
examples of a single open surface. In fact the plane
was created by unrolling the tube.
Examples of Closed PolySurfaces: Box, Truncated
Cone, Cylinder.
Polysurfaces are made up multiple joined surfaces.
When exploded they become unjoined individual
surfaces. Some commands behave differently
on polysurfaces than single surfaces and some
commands won’t work at all.
An important distinction is made between single-
surface solids (NURBS objects) and polysurface
solids. When solids cannot be exploded into
component surfaces because they are made
with only one surface, they are known as Nurbs
surfaces. A polysurface is made up of several
Nurbs surfaces put together (Joined),

<!-- page 177 -->

The above polysurfaces have been exploded into
their individual surfaces indicated by different layer
colors.  Each polysurface is several surfaces joined
together.
Parts of Surfaces:
Nurbs surfaces have rectangular construction.
Observe how there are lines running through each
surface that are perpendicular to one another,
The command Direction in the Utilities menu will
display the U V and Surface Normal directions
on a selected surface. You can also reverse and
swap the U and V directions in this command. This
is used when you need to orient a base surface to
the flow surface when using Flow along Surface.
Matrix’s Smart Flow command has the ability to
orient the base surface to the flow surface within the
command. Smart Flow calls U,V and Surface Normal
X,Y and Z but they are for all intents and purposes
the same.
Single open surfaces have border curves that
mark each edge. Single surfaces that have been
made into polysurfaces also have curves at each
edge where they meet. These are called edge
curves. Edge Curves can be used in the same
way as Curves for some commands. For instance
you can use an Edge Curve as a profile or rail in a
Sweep
command.
When you Loft
from one Edge
Curve to another
you get the
additional option
to have the loft
match tangency
with the existing
edge.
meaning that they run in two directions: up-and-
down, and left-to-right. These curves are known
as Isocurves. The program ascribes two names
to these directions: “U” and “V”. “U” always runs
one way (for example, left-to-right) and “V” always
runs the other way (for example, up-and-down);
however, due to the complex composition of
surfaces, the program will sometimes assign these
letters in what seems like an arbitrary fashion. Just
be assured that there will always be two directions
in any surface: “U” going one way and “V” going
at right angles to it. U and V become important
when performing complex commands such as
Flow along Surface.
In a very flat
surface, there
are fewer isocurves needed, with a minimum of 2
needed for a totally flat, rectangular surface. For a
more curvy surface, there are more isocurves. At
each place where the isocurves intersect, Control
Points may be found on the surface. Like in a
curve, the Control Points may be reorganized and

<!-- page 178 -->

This produces a surface that has two openings in
it, like a tube.
moved around to adjust the shape of a surface.
If more control points are needed in any given
surface, a surface, like a curve, may be rebuilt
using the Rebuild command found in the Surface
menu.
Creating Surfaces:
To create surfaces with many of the tools from the
Surface menu, users are actually drawing the
shapes of the U and/or V isocurves first, then
allowing the program to create the surface
between them. When creating a ring, for example,
the user first places the “rail curve” in the
viewports, and then the “profile curve”. It is very
important that the profile curve runs
perpendicular to the rail curve for the most
predictable surface. This is because the rail curve
and the profile curve will become the “U” and “V”
directions in the new surface.
Although the rail curve is a closed circle in the
above examples, it is also possible to sweep
a rail that is not closed or a circle. Place any
rails in space, along with a profile curve that is
perpendicular to it, and sweep them. If an open
rail curve is swept with an open profile curve, a
single, open surface will result.
If the profile curve is a closed polycurve, the
resulting surface will be an open polysurface.
Original Curves (Left) Swept Surface (Right)
Notice that the profile curve is used to make the
cross-section shape of the ring. Put another way, if
the ring were to be sliced in half, the shape of the
cross-section would
be the same as the
shape of the profile.
That is why the
profile curve is
referred to as the
“cross-section curve”
during a Sweep
operation.
The ring pictured above is called a “Closed Sweep”
because it does not have any openings. An “Open
Sweep” can also be run by placing two profile
curves and only sweeping the rail between them.
If, however, the profile curve is first rebuilt, the
resulting surface will be an open Nurbs surface,
or, a single surface (not a polysurface). This is
important to consider when creating surfaces
with Sweep 1 or Sweep 2 (a sweep that uses

<!-- page 179 -->

### Skills & Commands in this Chapter
two rail curves instead of just one), since the two
different kinds of surfaces work differently with
the functions of builders and other operations in
Matrix.
Create Surfaces:
Sweep 1, Sweep 1 History, Sweep 2, Sweep 2
History, Profile Sweep, Blend Surface, Variable
Blend Surface, Loft, Curve Network, Patch,
Plannar Curves, Plane From Corners, Surface
Extrude All, Revolve, Rail Revolve, 4 Rail Profile
Sweep, Offset Surface, Variable Offset Surface,
Tween Surfaces, Sweep Multi, Heightfield from
Image,
There is one more “direction” on a surface in
addition to the U and V directions: Surface
Normals. The surface normals tell the user “which
end is up”. To check the direction of a surface,
select the surface and click on the Direction
command in the Utilities menu.
Up: To reverse the directions on a surface, click on
it once with the mouse and the arrows will flip.
Edit Surfaces:
Rebuild, Shrink Trimmed Surface, Untrim, Fillet
Surfaces, Variable Fillet Surface, Chamfer
Surfaces, Variable Chamfer Surfaces,  Merge,
Match, drape, Split at Isocurve, Extend Surface,
Extrude Normal to Surface, Unroll Surface, Soft
Edit, Cutting Plane, Adjust ClosedCurve Seam,
Set Surface Tangent
Down:
On a closed surface that is valid (correct), the
direction is always “out”. It is important to consider
surface normals when using builders and other
functions in Matrix.
