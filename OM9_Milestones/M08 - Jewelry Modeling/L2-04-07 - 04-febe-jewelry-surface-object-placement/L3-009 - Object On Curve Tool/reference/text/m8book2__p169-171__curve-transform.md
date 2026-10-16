# Curve Transform ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 169-171
- Section title: Curve Transform ￼
- Section id: m8book2-178-curve-transform

---

<!-- section m8book2-178-curve-transform | Curve Transform ￼ | m8book2 pages 169-171 -->

<!-- page 169 -->

mesh. If it is shrunk with Mesh Mapper, the height
will be shrunk: if the mesh is grown with Mesh
Mapper, the height will be grown. To leave the Z
Height of the mesh alone during the Transform,
turn this ON.
Curve direction is top to bottom
Transform: Maps the mesh between the curves
in the input boxes.
Select: Selects the mesh that was mapped
between the curve so it can be further modified.
Undo: Reverts the mapping of the mesh.
## Curve Transform
This tool allows you to bend a curve the way
you can bend a wire. Assign a “hold” point
and a “bend” point for any curve in space, and
manipulate this curve. Profiles adhere to the curve
when modeling.
Where is this Command:
Access this command from Tools>Curve
Transform
Steps:
Example of Mesh Mapper with letters and football
### Options:
• Select Curve Transform from the
Tools menu.
• Input the curve into the in-box in the
builder.
• Adjust the curve using the sliders in
the Curve Transform builder.
Top and Bottom Lines/Left And Right
Lines: Controls whether the input curves for
Mesh Mapper are on
the top and bottom of
where the mesh will
be mapped or on the
left and right.
Hold point (left) and Bend point (right)
Maintain Z: Turn ON the Maintain
Z option when you wish to
maintain the current height of the
mesh in the World Z direction (the
height you see in the Through
Finger or Side View viewports).
With this option OFF, the height
will be scaled up or down relative to the rest of the

<!-- page 170 -->

Bend Point: This establishes a point along the
curve where the curve will bend from.
Adjust in the builder by selecting the
Bend Point icon at the left, running the
slider below-or typing a percentage
between 1 and 100 in the box.
After Bend in Looking Down viewport
Bend: Controls the bending that takes place
between the Bend Point and Hold
Point. Adjust in the builder by selecting
the Bend icon at the left, running the
slider below-or typing a percentage
between -180 and 180 in the box.
Scale: Increases the distance between the Hold
Point and Bend Point stretching this
part of the curve. Adjust in the builder
by selecting the Scale icon at the left,
running the slider below-or typing a
value between -100 and 100 in the box.
Straighten: Straightens the part of the curve
After Bend in Perspective viewport
between the Hold Point and bend
point. Adjust in the builder by selecting
the Straighten icon at the left, running
the slider below-or typing a value
between 0 and 100 in the box.
Blend: Effects the curvature of the curve
between the Hold Point and Bend Point.
The curve will not be affected beyond
that point . Adjust in the builder by
selecting the Blend icon at the left,
running the slider below-or typing a percentage
between 0 and 100 in the box.
### Options:
Complete with Profiles and a Sweep
Mirror: When you have mirror toggled on the
curve will be transformed on both
### Editing Dimension:
sides. When mirror is on there are
two additional options Reflect
and Rotate.
Hold Point: This establishes a point along a
Reflect/Rotate (with Mirror
ON): Reflect causes the curve to mend on both
sides as if it had been mirrored and Rotate as if it
had been rotated with Copy.
curve that serves like a bending break.
The curve will not be affected beyond
that point. Adjust in the builder by
selecting the Hold Point icon at the left,
running the slider below-or typing a percentage
between 1 and 100 in the box.
Undo: Undoes the mapping of the mesh.

<!-- page 171 -->

## Boolean Builder
## Center Line Builder
This calculates the center line when given a
surface with two curves on it.
Where is this Command:
Access this command from Tools>Center Line
Builder
Steps:
This builder allows you to easily input an “Object”
to be cut and “Cutter” to do the cutting; or two
objects to be Joined or Unjoined. Or, it allows you
to create a new object from where two objects
Intersect. The Boolean Builder checks that all
objects input into the builder are valid and will
perform as expected. Additionally, the builder
has tools to help you overcome typical obstacles
when performing booleans.
Where is this Command:
Access this command from Tools>Boolean
Builder
Or Cutters>Boolean Builder
• Select the Center Line Builder from the
Tools menu.
• Select the surface and input it into the
Surface box.
• Select the curve on the surface and
input into the Curve 1 box.
• Select the second curve
on the surface and
input in the Curve 2
box.
• Click Start.
Or, choose from the F6 menu when a cutter
is selected. This will perform a boolean
difference. You will be prompted by the
Command Line to select the surface to cut
from. This won’t give you access to all the
options in the builder but is a good short cut
for an often repeated action.
### Boolean Types:
Before
The Boolean Builder allows you to perform 3
common types of booleans: Boolean Difference,
Boolean Union, and Boolean Intersection. Objects
used in the builder must be watertight solids.
We will discuss each of these boolean types
individually.
Boolean Difference:
After
This is the most common type of
boolean for the kind of modeling we do.
A Boolean Difference uses one solid
(Cutter) to remove material from
another solid (Object). We call each of these the
cutter and the object. Cutters are used to create a
piercing or void in another object and are then
### Options:
Reset: Clears the surface and curves from the
builder.
discarded-for
instance when
cutting
channels.
