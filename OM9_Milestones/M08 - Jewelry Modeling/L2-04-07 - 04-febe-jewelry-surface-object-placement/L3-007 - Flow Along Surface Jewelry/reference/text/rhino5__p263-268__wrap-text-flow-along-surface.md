# Wrap Text - Flow along surface

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf
- Page range: 263-268
- Section title: Wrap Text - Flow along surface
- Section id: rhino5-084-wrap-text-flow-along-surface

---

<!-- section rhino5-084-wrap-text-flow-along-surface | Wrap Text - Flow along surface | rhino5 pages 263-268 -->

<!-- page 263 -->

## Wrap Text - Flow along surface
This tutorial demonstrates wrapping text solids and other objects on a cylinder. These objects could be used to
trim holes in the cylinder.
You will learn how to:
●
Create text as solid objects.
●
Wrap the objects to a surface.
### Make a surface
For this example, create a simple cylinder. Once you have learned the basic technique, you will be able to use
other types of surfaces. Remember that trimmed surfaces maintain their basic rectangular shape. This
underlying shape will affect the placement of the text.
Create a cylinder
4
In the Perspective viewport, use the Cylinder command with the Vertical option to create a solid
cylinder.

<!-- page 264 -->

Wrap Text - Flow along surface
### Create the objects to wrap
These solid objects will be wrapped on the cylinder surface.
Create the text
1. Use the TextObject command to create your text using Solids.
Choose a font that is fairly large and blocky rather than one with many holes and details.
Set the Height at about 1.5 units.
Set the Solid thickness to .1 units.
2. Place the text on the construction plane near the cylinder. The location is not important.
### Control the object placement
The CreateUVCrv command generates the planar border curves of a surface that can be used as a guide to
orient your text. Use the border rectangle to lay your text out before re-applying it to the cylinder. The
rectangle then is used as a reference to guide the placement of the other objects.

<!-- page 265 -->

Wrap Text - Flow along surface
Create the UV curve
1. Use the CreateUVCrv command on the cylinder side to create curves that represent the border of
the surface on the construction plane.
2. Select the side of the cylinder.
A rectangle is created starting at 0,0 on the Top construction plane.
Position the text objects
4
Use Move, Rotate, and Scale or other transforms to arrange the text objects inside the rectangle
just the way you want them to appear on the cylinder.
Add any other decoration curves you want to use.
Create a reference surface
4
Use the PlanarSrf command to make the rectangle into a surface.
You will use this surface later as a reference object for the FlowAlongSrf command.
Extrude the decoration curves
1. If you have created other curves, select these.
2. Use the ExtrudeCrv command to thicken the decorations to match the letters.

<!-- page 266 -->

Wrap Text - Flow along surface
3. At the Extrusion Distance… prompt, set Solid=Yes.
4. At the Extrusion distance… prompt, type .1.
Wrap the lettering on the cylinder
1. Select the lettering and the decoration.
2. Start the FlowAlongSrf command.
3. At the Base surface… prompt, set Rigid=No.

<!-- page 267 -->

Wrap Text - Flow along surface
4. Click the rectangular plane near the "lower-left" corner as illustrated.
5. At the Target surface… prompt, click the cylinder near the lower edge of the seam as illustrated.
The text and decoration solids wrap around the cylinder.
Now you can use the letters to cut the cylinder or Boolean the objects together.

<!-- page 268 -->
