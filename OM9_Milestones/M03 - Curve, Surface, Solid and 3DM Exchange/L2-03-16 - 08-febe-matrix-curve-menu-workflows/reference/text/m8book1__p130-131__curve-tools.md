# Curve Tools

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 130-131
- Section title: Curve Tools
- Section id: m8book1-141-curve-tools
- Kind: chapter_intro
- Images (../images/): m8book1_p130_img01.png, m8book1_p130_img02.png, m8book1_p131_img01.png

---
<!-- section m8book1-141-curve-tools | Curve Tools | m8book1 pages 130-131 -->

<!-- page 130 -->

# CHAPTER 19
# Curve Tools
### Learn to Create Closed Curves, Open Curves, and Curves on
### Surfaces for Basic Designs.
Split a closed curve
with two cutting
curves to change it.
Open vs.Closed Curves
Some curve tools create closed curves (Circle,
Rectangle, Ellipse, and Polygon). Other curve tools
create open curves that you can “close” if you
wish (Polyline, Interp Curve, Curve). Still others
create only open curves (Line, Arc, Arc Direction).
Usually, you’ll use these shapes together to create
closed and open curves. Then, with tools from the
Top 11 Buttons and other basic curve tools (Fillet,
Chamfer, and Offset), you’ll be able to create new,
closed curve shapes. The O-Snaps and Grid
Snaps Menus contain tools that will help you
create closed curves at the shape, location, and
size you want. Later in this manual, you’ll study
other tools from the Curve menu that play vital
roles in more advanced 3D modeling.
Why Start with Curves?
An understanding of curves is essential to 3D
modeling because curves
form the foundation of all
surfaces and solid objects in
Matrix. You’ll recall from your
high school geometry class
that, by themselves, curves
have no 3D mass: they are
1-dimensional lines that are
only the thickness of a single
point object. But in Matrix,
when curves are used together with tools found in
the Surface and Solid menus, it quickly becomes
apparent that curves form the “skeleton” of almost
all 3D models.
Types of Curves
There are two different kinds of curves: open
and closed. In most modeling you’ll be doing,
you will need to draw closed curves to ensure
that the surfaces and solids created from the
curves are also closed,
“watertight” models.
You can think of a
3D model as a ship:
it shouldn’t have any
“holes” or leaks in it
where water could
seep in and sink it!
Closed curves result
in closed, watertight
models. As for
open curves... well...
they can “sink” your
modeling efforts
pretty quickly!
Avoiding Problem Curves
Beware of creating curves that “loop” or intersect
themselves. These will create invalid surfaces
when they are used in modeling. Also, make sure
you use O-Snaps to achieve closed curves. In
order to be closed, each segment in a polycurve
- or, the Start and
End point of a Nurbs
curve - must have
Position Continuity
- which means they
must occupy the
same point in space.
The End O-Snap will
help you ensure that
the end of one curve
snaps to the end
of another curve,
so they will have
Position Continuity
and can be joined.

<!-- page 131 -->

### Skills & Commands in this Chapter
Closed Curve Tools:
Circle, Ellipse, Rectangle, Polygon
Open Curve Tools:
Line, Arc, Arc Direction
Open or Closed Curve Tools:
Polyline, Interp Curve, Curve
Modifying Curves:
Offset, Offset on Surface, Offset Normal to
Surface, Blend, Blend Curve Handles, Fillet, Fillet
Corners, Chamfer, Curve Boolean
Fixing Problem Curves:
Rebuild, Fit, Mark Curve Start & End, Extend,
Match
Curves on Surfaces:
Extract Isocurve, GV Extract Isocurve, Pullback,
Project, Polyline & Interp Curve on Surface,
Sketch, Sketch on Surface & Mesh, Curve to
Lines, Helix, Spiral
