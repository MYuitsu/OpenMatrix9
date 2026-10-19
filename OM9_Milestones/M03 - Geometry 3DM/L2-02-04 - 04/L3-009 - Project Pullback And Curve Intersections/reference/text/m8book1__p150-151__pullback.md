# Pullback

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 150-151
- Section title: Pullback
- Section id: m8book1-156-pullback
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p150_img01.png, m8book1_p150_img02.png, m8book1_p150_img03.png, m8book1_p150_img04.png, m8book1_p150_img05.png, m8book1_p150_img06.png, m8book1_p151_img01.png, m8book1_p151_img02.png

---
<!-- section m8book1-156-pullback | Pullback | m8book1 pages 150-151 -->

<!-- page 150 -->

Where is this Command:
Access this command from Curve>Curve From
2 View History
Steps:
• Select Curve from 2 Views with History from the
Curve menu.
• Select The first curve then the second curve.
• Full=True or False in the Command Line and press
Enter.
### Command Line Options:
Full=True/False: When set to true Matrix will
draw a curve both on the top and bottom of the
ring rail. When set to false Matrix will only draw a
curve on the top of the finger rail (from 9 o’clock
to 3 o’clock as seen in the Through Finger View.
Full=True Matrix creates a curve on both side of the
finger rail.
The red curve have matches the shape of both the
brown curves and green curve when viewed in there
respective planar views.
### Command Line Options:
Direction: Two points establish the direction
angle.
Steps
Full=False leaves the bottom of the ring shank
unfinished. Complete the Shank by sweeping the ring
manually.
• Pick a base point.
• Pick a second point that establishes the direction
angle.
## Pullback
## Curve 2 Views with History
‘Pull’
Creates curves and points on a given surface by
pulling a curve or points onto a single surface or
mesh object.
Where is this Command:
Access this command from Curve>Pull
Creates a third curve from two curves that are
planar in different construction planes. The new
curve looks the same as each of the original
curves when viewed in each plane. This command
behaves the same as Curve from 2 views but has
an additional option.

<!-- page 151 -->

Steps:
TargetObject: Places the results on the same
layer as the target surface.
• Select Pullback from the Curve menu.
• Select curves.
• Select surface to pull the curves back to.
## Project
Notes: Pullback can be used in any viewport. For
best results place the curves to be pulled as close
to the surface as possible to minimize the amount
of distortion that may occur as the curve is pulled.
The curve pulls towards a point at the center of the
surface which is pulling. When exiting the command,
the newly created curves will be always be selected.
When drawing the curves, use the fewest control
points possible. This guarantees the smoothest
possible curve pullback to the target surface.
‘Project’
Creates curves and points on a given surface, by
finding the intersection of a surface and curve
projected towards the construction plane. This
command is viewport specific. For best results,
run the Project in the viewport where the curve
appears in front of the surface. This command
works on surfaces, polysurfaces and meshes.
Where is this Command:
Access this command from Curve>Project
Steps:
• Select Project from the Curve menu.
• Select curves.
• Select surface to project the curves on to.
The input curve get pulled back to the selected
surface.
### Command Line Options:
Loose: Pulls edit points back to the surface. If any
edit point misses the surface, the curve will not be
created.
Unlike Pullback project finds the intersection of all
surfaces
DeleteInput: Deletes the original curve from the
file.
OutputLayer: Specifies the layer for the results
of the command.
Notes: When exiting the command, the newly
created curves will be always be selected.
When drawing the curves, use the fewest control
points possible. This guarantees the smoothest
possible curve pullback to the target surface.
Input: Places the results on the same layer as the
input curve.
Current: Places the results on the current layer.
