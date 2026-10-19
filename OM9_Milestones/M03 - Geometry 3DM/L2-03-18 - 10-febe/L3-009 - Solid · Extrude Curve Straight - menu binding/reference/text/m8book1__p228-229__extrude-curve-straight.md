# Extrude Curve Straight ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 228-229
- Section title: Extrude Curve Straight ￼
- Section id: m8book1-248-extrude-curve-straight
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p228_img01.png, m8book1_p228_img02.png, m8book1_p228_img03.png, m8book1_p229_img01.png, m8book1_p229_img02.png, m8book1_p229_img03.png

---
<!-- section m8book1-248-extrude-curve-straight | Extrude Curve Straight ￼ | m8book1 pages 228-229 -->

<!-- page 228 -->

Solid Thickness: Sets the thickness of text
created from solids (Z direction)
Note: Matrix uses the fonts in Windows on your
computer. Add fonts in Windows to have them
available in ‘Text’.
Name: Use the drop down menu to select from
the installed fonts on your computer.
Lower case as small caps: This option creates
lower case letters as small caps. Set the relative
text size as a percentage of the normal text.
Add spacing: Adds spacing between letters.
Note: Not all fonts will create valid surfaces or
solids without requiring reworking.
Bold: Sets the font to Bold.
## Extrude Curve Straight
Italic: Sets the font to Italic.
‘ExtrudeCrv’
Creates solid polysurfaces from closed, planar
curves. This is the same command as Extrude
Curve All but it defaults to the Straight option.
Comic Sans font used to create curves with ‘Text’.
Where is this Command:
Access this command from Solid>Extrude
Curve Straight
### Create
Steps:
Specifies the object type.
Curves: Creates text using outline curves.
Surfaces: Creates text using planar surfaces.
Solids: Creates text using solids.
• Select Extrude Curve Straight from the Solid
menu.
• Select Curves to Extrude.
• Press Enter when done.
• Select Command Line options.
• Specify an extrusion distance by picking a point or
typing in a value.
Note: Not every font is suitable to create as valid
solids. They may first need to be fixed at the curve
level. Then, extrude the curves as solids.
Solid polysurfaces are created from closed curves.
Solid polysurface text created with ‘Text’.
Group Objects: Groups the resulting text
objects.
Allow single-stroke fonts: Allows fonts that
do not have closed loop boundaries. Engraving-
style fonts for machine applications are the most
common. If unchecked, single-stroke engraving
fonts display closed loop boundaries
Text Size: Set text height and thickness.
Notes:
• Curves must be planar, or, appear as a single line
when seen in another viewport.
• Nonplanar curves will not extrude properly.
• Curves must be closed and joined, or have position
continuity to create valid, solid objects with no
naked (open) edges.
• When two or more curves are selected, they
should not intersect. If they do, trim them together
first.
• In order to make a hole through the solid object,
one curve should be fully inside the other, and on
the same plane in space.
• When curves on separate planes are selected
together, two solid objects will result.
Height: Sets the height in model units (mm as
default in the Y direction).

<!-- page 229 -->

correspond to the tangent sub-curves in the input
curves. The output will be a polysurface. When
UseExtrusions is on, this setting has no effect.
SetBasePoint: Specify a location that serves as
the first point when picking two points that set
the extrusion distance.
## Solid Extrude All
Nonplanar curve shown in Through Finger viewport
produces surface walls but does not create closed,
solid, polysurface.
### Command Line Options:
‘ExtrudeCrv’
Extrudes open curves, closed curves, or surfaces
(either planar or non-planar) and creates
polysurfaces that are straight-edged, come to a
single point, tapered or follow a path curve.
Direction: Pick Two points to establish the
direction angle.
Steps:
Where is this Command:
Access this command from Solid>Solid Extrude
All
Steps:
• Pick a base point.
• Pick a second point that establishes the direction
angle.
• Select Solid Extrude All from the Solid menu.
• Select objects to Extrude.
• Pick an Extrude option.
• Specify a distance by picking a point or typing in a
value.
Hints:
Pick a  point in the Through Finger viewport to set
the direction (if Curves are positioned planar in the
Looking Down viewport).
• Curves and surfaces cannot be selected together
in one step. Either select all curves or all surfaces.
• Both planar and non-planar objects can be
selected in one step.
BothSides: Draws the object on both sides of
the start point, creating the extrusion twice as
high as the distance value.
Solid: If the profile curve is closed and planar,
both ends of the extruded object are filled with
planar surfaces and joined to make a closed
polysurface.
ToBoundary: Extrudes the object to a boundary
surface.
Solid Extrude All
DeleteInput: Deletes the original curve.
### Extrude Options:
There are four extrude options; Straight, Tapered,
ToPoint or AlongCurve. See each expanded
Command Line options further below.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface
when the input curves are joined tangent curves.
Surfaces in the resulting polysurface object
