# Surface Extrude All ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 199-201
- Section title: Surface Extrude All ￼
- Section id: m8book1-217-surface-extrude-all
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p199_img01.png, m8book1_p199_img02.png, m8book1_p199_img03.png, m8book1_p200_img01.png, m8book1_p200_img02.png, m8book1_p200_img03.png, m8book1_p201_img01.png, m8book1_p201_img02.png, m8book1_p201_img03.png, m8book1_p201_img04.png

---
<!-- section m8book1-217-surface-extrude-all | Surface Extrude All ￼ | m8book1 pages 199-201 -->

<!-- page 199 -->

the trimming objects are deleted. This also deletes
any geometry attached to the edges of the
boundary.
A Deformable
plane is created
with 6- U and V
degrees with 10-
U and V points.
AllSimilar: Removes all trimming curves on
the edge of a trimmed surface, or if a hole edge
is selected, all holes on the same face will be
deleted.
UDegree/VDegree: Sets the Degree of the
surface in the U and V directions.
## Surface Extrude All
Use planar or non-planar, curves or surfaces to
extrude into capped or uncapped polysurfaces, to
a point, straight, tapered or along a curve.
Degree: Specifies the degree of the curve (or
surface). When drawing a high-degree curve, the
output curve will not be the degree you request
unless there is at least one more control point
than the degree.
UPointCount/VPointCount: The number of
control points in the U and V directions.
Where is this Command:
Access this command from Surface>Surface
Extrude All
Steps:
## Untrim
‘Untrim’
Removes a trimming boundary from a surface,
returning it to its untrimmed state.
• Select Surface Extrude All from the Surface menu.
• Select or get objects to Extrude.
• Select the Extrusion Type.
• Press Enter when done.
• Pick an Extrude Option in the Command Line.
Note:
Where is this Command:
Access this command from Surface>Untrim
Steps:
• Non-planar curves will not cap into a solid
polysurface.
• Planar curves will cap into a solid polysurface.
• Surfaces and curves can be extruded separately
but not together.
• Non-planar curves do not always give the best
results when choosing the tapered option.
• Select Untrim from the Surface menu.
• Select the edge to Untrim.
• Select another edge to Untrim or press Enter to
end the command.
### Straight Extrude Options (Default):
Curves or surfaces
are extruded straight,
perpendicular to the input
objects in Surface
Extrude All - Straight
(Solid=Yes).
Untrim is used to return the trimmed surfaces (left)
to their original state (right).
Direction: Two points establish the direction
angle.
### Command Line Options:
Steps
KeepTrimObjects Yes/No: Yes; the original
trimming objects are separated and retained. No;
• Pick a base point.
• Pick a second point that establishes the direction
angle.

<!-- page 200 -->

BothSides: Draws the object
Corners Round: The corners of the tapered
surfaces will be filled with filleted segments with
tangent (G1) continuity.
on both sides of the start
point, creating the object
twice as long as you indicate.
Corners Smooth: The corners of the tapered
surfaces will be filled with blend segments with
curvature (G2) continuity.
Solid Yes/No: Places caps
and joins all to make a closed
polysurface.
See Surface menu > Blend Surface > Continuity
Options for a full explanation of Continuity types.
DeleteInput Yes/No: Deletes the original
geometry from the file.
DeleteInput: Deletes the original geometry from
the file.
ToBoundary: Extrudes the object to a boundary
surface.
FlipAngle: Toggles the draft angle direction.
ToBoundary: Extrudes the object to a boundary
surface.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined tangent curves.
SetBasePoint: Specify a location that serves as
the first point when picking two points that set
the extrusion distance.
SetBasePoint: Specify a location that serves as
the first point when picking two points that set
the extrusion distance.
### Tapered Extrude Options:
### To Point Extrude Options:
Curves or surfaces are extruded using a draft angle,
and tapered (Solid=Yes).
Direction: Two points establish the direction
angle.
Steps
Non-planar or planar curves or surfaces are extruded
to a specified point (Solid=No)
• Pick a base point.
• Pick a second point that establishes the direction
angle.
Solid Yes/No: Places caps and joins all to make
a closed polysurface.
DraftAngle: Specify the draft angle for the taper.
DeleteInput Yes/No: Deletes the original
geometry from the file.
Solid Yes/No: Places caps and joins all to make
a closed polysurface.
ToBoundary: Extrudes the object to a boundary
surface.
Corners Sharp (Default): The corners of the
tapered surfaces will extend to meet at sharp
corners with position (G0) continuity.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined tangent curves.

<!-- page 201 -->

Steps
### Along Curve Extrude Options:
• Select Revolve from the Surface menu.
• Select curves to revolve.
• Press Enter when done.
• Pick the start of the revolve axis.
• Pick the end of the revolve axis. (Press Enter to use
C-Plane Z axis direction.)
• Select the start angle location. Enter a degree
amount (from 0° to 360°). Specify a Command
Line option. Press Enter.
• Select a revolution angle (0° to 360°). Press Enter
when done.
With Surface Extrude All - Along Curve, Select a
path curve for the curve or surface extrusion near
the start of the extrusion. Non-planar surface and
curve (left) and planar curve and surface (right).
Solid Yes/No: Places caps and joins all to make
a closed polysurface.
DeleteInput Yes/No: Deletes the original
geometry from the file.
Picking the start and end
of the revolve axis (left).
Selecting a revolution angle
(above).
SubCurve: Extrudes a curve the distance
specified by picking two points along a curve.
Note: The extruded surface starts from the beginning
of the curve, not the first picked point. Picking the
points only establishes the extrusion distance.
Steps:
• Select the path curve.
• Pick a start along the path curve.
• Pick an end along the path curve.
ToBoundary: Extrudes the object to a boundary
surface.
Revolve a profile curve 360° to create a
custom-shaped object.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface when
the input curves are joined tangent curves.
### Command Line Options:
## Revolve
DeleteInput: Deletes the original geometry from
the file.
‘Revolve’
Creates a surface or polysurface by revolving
a profile curve that defines the surface shape
around an axis.
Where is this Command:
Access this command from Surface>Revolve
Deformable Yes/No: No: the resulting revolved
surface is an exact revolve: a rational surface
with fully-multiple knots at the quadrants. This
kind of surface is not easy to deform smoothly
by point editing. Yes: the surface is rebuilt on
the ‘around’ direction to a degree 3 non-rational
