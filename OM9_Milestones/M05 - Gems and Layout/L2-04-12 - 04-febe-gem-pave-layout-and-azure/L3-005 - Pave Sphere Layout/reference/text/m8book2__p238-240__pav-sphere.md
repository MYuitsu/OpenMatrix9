---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 238-240
section_title: Pavé Sphere
section_id: m8book2-214-pav-sphere
---

# Pavé Sphere

Source: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf), pages 238-240.

<!-- section m8book2-214-pav-sphere | Pavé Sphere | m8book2 pages 238-240 -->

<!-- page 238 -->

### Command Line options:
for orienting gems on a surface. Gem Splop is
best used for more free from designs. For more
precision use Gem on Surface.
Where is this Command:
Access this command from Gems>Gem Splop
Copy = Y/N:  Keep “Copy” turned on in order to
add copies of the objects you are flowing, and not
delete the originals. This is a good idea when you
plan to use History, since you can then edit the
original items (but don’t Trim or Join them) and
see the same changes in the flowed items.
or from the F6 menu when a curve is selected.
Steps:
Rigid = Y / N: This option keeps the command
from deforming the item(s) in order to fit it to
the target object. With “Rigid = Y”, only the
configuration of the objects being flowed will be
conformed to the curvature of the target object.
With “Rigid = N”, the objects being flowed will be
curved to fit the target object.
### Along Curve Options:
• Select a Gem Splop from the gem menu.
• Select a gem to Splop on a surface.
• When prompted create a reference sphere
matching the size of the stone.
• Pick the Splop point on the surface. (Make sure
that Rigid and Copy options are both set to yes in
the Command Line.
• Press Enter when done.
The sploped stones will be replaced on the
surface with block instances (gems).
Line: The Line command will be started. This
allows you to draw your base line (instead of
selecting it). Instead, we recommend measuring
the target curve with the Length command and
creating a base line ahead of time.
Local: Select this command to draw two circles
defining a “tube” within which the objects are
deformed and outside of which they will not be
deformed. This means part of an object may be
stretched to touch the curve and part can be left
alone.
Stretch: With “Stretch” set to Yes, the objects
will be stretched to accommodate the length of
the curve when the Base line is shorter than the
Target curve.
Splopping gems on a ring surface
### Along Surface Option:
## Pavé Sphere
Plane: Starts up the Plane command during the
Flow Along Surface command, so you can draw
the base surface instead of selecting one. Instead,
we recommend you follow the instructions we
have set forth earlier in this section to create your
own base surface.
‘gvPaveSphere’
Create a pavé’ sphere with no surface on-screen,
or using a spherical surface. Adjust gem count,
gem size, and symmetry. Preview results before
you commit.
## Gem Splop
You can access this command from Gems>Pavé
Sphere
‘gvGemSplop’
Gem Splop is another gem tool that you can use

<!-- page 239 -->

Steps:
values, for X and T yield different results, but each
affect the Gem Count, so you must choose one
way to define the shape: Basic (Gem Count) or
Advanced (X and T).
• Select Pave Sphere from the Gems menu.
• In the Pavé sphere dialog click on ‘Select Srf’ and
choose a surface or simply begin manipulating
the sliders and settings in the menu.
• Click on preview to see the results.
• When you achieve the desired layout click okay.
By defining the X or T values Matrix will then
determine the best number of gems to use in the
layout (when you define count Matrix controls the
X and T functions).
### Layout Options
### Removal:
Select a gem removal pattern by running the
slider. Choose from None, Pattern A or Pattern B.
None: Removes no stones from the pavé layout.
A/B Pattern: Pattern A or B will yield different
results depending on your surface and other
settings. Experiment and see what you like.
Count: Lets you control the number of
gemstones in the pavé layout. Adjust in the Pavé
Sphere dialog by running the slider.
Removal Pattern A left, Removal Pattern B right.
X/T: Basically, X and T define the number of
gems in a lattice or grid based on each triangle in
a dodecahedron shape. From the dodecahedron,
the points (in the number you define using X and
T to represent gems) are projected onto a sphere;
their distances are averaged; and gems are placed
accordingly. Defining different values, or the same

<!-- page 240 -->

### Sizing:
## Gem Positioner (MSR)
Gem Size: Defines the target (ideal) size for all
gems. Set the value by typing it in mm in the box.
‘gvGemPositioner’
Move, Scale, Rotate and Add gems using the
Viewport Control Handles. This tool can be used
on a single gem or a gem configuration.
Gem Min Size: Sets the minimum stone in the
pavé layout.
Where is this command:
Access this command from Gems>Gem
Positioner
Sphere Radius: If you don’t input a surface, you
may change the radius of the sphere that this tool
creates by typing in a new mm dimension for this
value.
or Select a gem or multiple gems and choose
F6, MSR
### Relax Controls:
Steps:
These controls determine how distance between
stones are averaged.
Relax Amount: Can be raised to allow the
builder to spend more time trying to get your
layout “just right”. With this value higher, it may
spend more time trying to get the look right, but
gems will be more evenly spaced and uniform.
• Select Gem Positioner from the Gems menu. (Or
press F6 and choose MSR.)
• Move, scale or rotate (MSR) using the Viewport
Control Handles (VCH)
Hold Shift while drag the handle to confine the
movement 0.10 mm increments (or 5 degree in
the case of rotation)
### Position the Gems
Relax Strength: Raise this value and the layout
will be more random; lower it and the layout will be
more uniform. So, for a good, uniform layout, try
adjusting the Relax Amount higher and the Relax
Strength lower. For more randomness; raise Relax
Strength; for faster results, lower Relax Amount
The three arrows move the gem in its relative X, Y,
and Z axes. Mouse over them to see them light up
in red, then click and drag them to move the gem
planar (flat) along each relative axis.
## Gem Update
‘gvGemUpdater’
Every once in a while you will find a model in
which a gem has lost its ability to be a gem. You
will know a gem is broken if when you try to use
it in a builder you get a No Valid Object error. Use
the Gem Update button found within the
Gem Menu to repair broken Gems.
Where is this command:
Access this command from Gems>Gem
Update
Steps:
• With Nothing selected click on Gem Update in the
Gems menu..
Position gems using the yellow, light blue or dark
blue handle to move the gem along on axis at a time.
