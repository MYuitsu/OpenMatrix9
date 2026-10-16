# Headphone - Sweep, Loft, and Extrude

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 143-172
- Section title: Headphone - Sweep, Loft, and Extrude
- Section id: rhino5-080-headphone-sweep-loft-and-extrude
- Kind: tutorial
- Images (../images/): rhino5_p143_img01.png, rhino5_p144_img01.png, rhino5_p144_img02.png, rhino5_p145_img01.png, rhino5_p145_img02.png, rhino5_p146_img01.png, rhino5_p146_img02.png, rhino5_p146_img03.png, rhino5_p147_img01.png, rhino5_p147_img02.png, rhino5_p148_img01.png, rhino5_p148_img02.png, rhino5_p149_img01.png, rhino5_p149_img02.png, rhino5_p150_img01.png, rhino5_p150_img02.png, rhino5_p151_img01.png, rhino5_p151_img02.png, rhino5_p152_img01.png, rhino5_p152_img02.png, rhino5_p152_img03.png, rhino5_p153_img01.png, rhino5_p153_img02.png, rhino5_p154_img01.png, rhino5_p154_img02.png, rhino5_p155_img01.png, rhino5_p155_img02.png, rhino5_p156_img01.png, rhino5_p156_img02.png, rhino5_p157_img01.png, rhino5_p157_img02.png, rhino5_p158_img01.png, rhino5_p158_img02.png, rhino5_p159_img01.png, rhino5_p159_img02.png, rhino5_p160_img01.png, rhino5_p160_img02.png, rhino5_p160_img03.png, rhino5_p161_img01.png, rhino5_p162_img01.png, rhino5_p162_img02.png, rhino5_p162_img03.png, rhino5_p163_img01.png, rhino5_p163_img02.png, rhino5_p164_img01.png, rhino5_p165_img01.png, rhino5_p165_img02.png, rhino5_p166_img01.png, rhino5_p166_img02.png, rhino5_p167_img01.png, rhino5_p168_img01.png, rhino5_p168_img02.png, rhino5_p168_img03.png, rhino5_p169_img01.png, rhino5_p169_img02.png, rhino5_p170_img01.png, rhino5_p170_img02.png, rhino5_p171_img01.png

---
<!-- section rhino5-080-headphone-sweep-loft-and-extrude | Headphone - Sweep, Loft, and Extrude | rhino5 pages 143-172 -->

<!-- page 143 -->

## Headphone - Sweep, Loft, and Extrude
This tutorial demonstrates creating surfaces from profile curves using lofts, sweeps, and extrudes.
You will learn how to:
●
Use sub-object selection to pre-select objects for a command.
●
Create a surface from a planar curve.
●
Loft, revolve, sweep, and extrude surfaces.
●
Cap planar holes to create a solid.
●
Draw a helix around a curve.
●
Match curve ends.
●
Create solid pipes.
●
Mirror objects.
●
Use layers.
●
Use object snaps.
To open the headphone model.
4
Open the tutorial model file Headphone.3dm.
Download the tutorial models.

<!-- page 144 -->

Headphone - Sweep, Loft, and Extrude
### Create the speaker shell
The speaker shell is created using a lofted surface, a one-rail sweep, a solid extrusion of a planar curve, and a
surface fillet. The resulting geometry is joined into one solid.
Loft curves to create a surface
One way to create a surface is to use existing curves as a guide. When lofting through curves, the curves
are used as a guide for creating a smooth surface.
1. Turn on Shaded mode in the Perspective viewport.
2. Select the three circular curves, with a crossing selection as illustrated.
3. On the Surface menu, click Loft.
4. At the Drag seam point to adjust… prompt, note the display of the curve direction arrows at the
seam points, and press Enter.
In this model, they are nicely lined up for you, so you do not need to adjust them.

<!-- page 145 -->

Headphone - Sweep, Loft, and Extrude
5. In the Loft Options dialog box, click OK to create the loft.
Extrude the surface edge
Extrude the lofted surface edge in the center to make a magnet housing.
1. Hold the Ctrl and Shift keys to select the surface edge at the center of the lofted surface.
Tip:
Selecting objects with the Ctrl and Shift keys is called sub-object selection.
Hold the Ctrl and Shift keys, and click to select polysurface faces; surface and
polysurface edge curves; control points; mesh vertices, faces, boundaries, and edges;
and objects within a group.
2. On the Solid menu, click Extrude Planar Curve > Straight.
3. At the Extrusion Distance… prompt, type -2 (notice the negative number) and press Enter.
This makes a solid cylinder for the magnet housing that is two units thick and extends in the negative
direction from the original surface edge.
Extract the bottom surface
The cylinder you just created is an extrusion object (solid). To remove the bottom, extract the face.

<!-- page 146 -->

Headphone - Sweep, Loft, and Extrude
1. Hold the Ctrl and Shift keys, and click to select the bottom face.
2.
On the Solid menu, click Extract Surface.
3. At the Select surfaces to extract… prompt, select the surface as illustrated and press Enter.
4. Press the Delete key.

<!-- page 147 -->

Headphone - Sweep, Loft, and Extrude
Fillet the edge of the cylinder surface
1. On the Solid menu, click Fillet Edge > Fillet Edge.
The current radius setting should be 1.
2. At the Select edges to fillet… prompt, select the edge at the top of the cylinder press Enter.
3. At the Select fillet handle to edit prompt, press Enter.

<!-- page 148 -->

Headphone - Sweep, Loft, and Extrude
Join the surfaces
Surfaces that share an edge can be joined into a polysurface. You will join all the surfaces. Since the faces
are sometimes hard to see, use two viewports to select them all.
1. Select the surface and the polysurface.
2. On the Edit menu, click Join.
To join surfaces, you must select surfaces that are adjacent to each other and the edges must match.
### Create the padding and cover
Sweep a curve around the edge of the speaker cone to create the padding around the edge of the speaker.
Reset the view
1. On the View menu, click Zoom > Zoom Extents All.
2. Hide or Delete all of curves used for the loft.
Sweep a curve along one rail
1. Hold the Ctrl and Shift keys, and click to select the loft surface outer edge.
2. Hold theShift key and select the rail curve at the top of the speaker as shown.

<!-- page 149 -->

Headphone - Sweep, Loft, and Extrude
3. On the Surface menu, click Sweep 1 Rail.
4. In the Sweep 1 Rail Options dialog box, click OK.

<!-- page 150 -->

Headphone - Sweep, Loft, and Extrude
Make a surface from planar curves
Fill the area at the base of the padding with a planar surface created from the edge of the sweep.
1. Hold the Ctrl and Shift keys, and click to select the surface edge of the speaker cone as illustrated.
2. On the Surface menu, click Planar Curves.
A planar surface is created at the base of the padding.
### Create the mounting bracket
The next part is the bracket that holds the speaker to the headband. Since the speaker unit is complete, you
can turn its layer off and make the Bracket layer current.
Reset the layers
1. On the status bar, click the Layer pane.
2. Make Bracket the current layer and turn on Bracket Shape Curves.
Turn all other layers off.
Reset the view
4
On the View menu, click Zoom > Zoom Extents All to zoom in on the bracket shape curves in all
viewports.
Extrude a curve into a solid
Use a planar curve to create a solid shape.

<!-- page 151 -->

Headphone - Sweep, Loft, and Extrude
1. Select the closed curve.
2. On the Solid menu, click Extrude Planar Curve > Straight.
3. At the Extrusion distance… prompt, type -1 and press Enter.
Fillet the edges
Round the sharp edges with a fillet.
1. On the Solid menu, click Fillet Edge > Fillet Edge.
2. At the Select edges to fillet… prompt, type .2 and press Enter.
3. At the Select edges to fillet… prompt, click ChainEdges and select the front edge of the bracket.
The entire edge of the solid should highlight.

<!-- page 152 -->

Headphone - Sweep, Loft, and Extrude
4. Press Enter to close that edge selection.
5. At the Select edges to fillet… prompt, click ChainEdges and select the back edge of the bracket.
6. Press Enter to close that edge selection.
7. Press Enter to finish the edge selection.
8. At the Select fillet handle to edit prompt, press Enter.

<!-- page 153 -->

Headphone - Sweep, Loft, and Extrude
Create a tubular surface from the shape curves
1. Select the curve at the top of the bracket.
2. On the Solid menu, click Pipe.
3. At the Starting radius… prompt, type .3 and press Enter.
Before typing the radius, set the command-line options to Cap=Flat and Thick=No.
4. At the End radius… prompt, press Enter.
5. At the Point for next radius prompt, press Enter.
For the second tube
1. Select the curve at the bottom of the bracket.
2. On the Solid menu, click Pipe.
3. At the Starting radius… prompt, type .2, and press Enter.
4. At the End radius… prompt, press Enter.

<!-- page 154 -->

Headphone - Sweep, Loft, and Extrude
5. At the Point for next radius prompt, press Enter.
### Create the headband
The headband consists of a series of ellipses swept along a path.
Reset the layers
1. On the status bar, click the Layer pane.
2. Make Headband the current layer and turn on Headband Shape Curves.
Turn all other layers off.
Reset the view
4
On the View menu, click Zoom > Zoom Extents All to zoom in on the headband shape curves in all
viewports.
Create an ellipse perpendicular to a curve
1. Turn Ortho on.
2. On the Curve menu, click Ellipse > From Center.

<!-- page 155 -->

Headphone - Sweep, Loft, and Extrude
3. At the Ellipse center… prompt, click AroundCurve.
4. At the Ellipse center prompt, snap to an endpoint of the headband curve.
Use the End object snap.
5. At the End of first axis prompt, type 0.5, and press Enter.
6. At the End of first axis prompt, drag the cursor in the x-direction and click.
7. At the End of second axis prompt, type 2, and press Enter.

<!-- page 156 -->

Headphone - Sweep, Loft, and Extrude
8. At the End of second axis prompt, drag the cursor in the y-direction and click.
Array a curve along a path
1. Select the ellipse.
2. On the Transform menu, click Array > Along Curve.
3. At the Select path curve prompt, select headband curve.
4. In the Array Along Curve Options dialog box, under Method, set the Number of items to 3.

<!-- page 157 -->

Headphone - Sweep, Loft, and Extrude
5. Under Orientation, click Freeform, and click OK.
Scale the ellipse
Scale the center ellipse to make it larger.
1. Select the center ellipse.
2. On the Transform menu, click Scale > Scale 1-D.
Scale1D stretches an object in one direction.
3. At the Origin point… prompt, in the Perspective viewport, snap to the center of selected ellipse.
4. At the Scale factor or first reference point… prompt, type 2, and press Enter.

<!-- page 158 -->

Headphone - Sweep, Loft, and Extrude
5. At the Scale direction… prompt, drag the cursor in the y-direction and click.

<!-- page 159 -->

Headphone - Sweep, Loft, and Extrude
Sweep along one rail
1. Select the curves.
2. On the Surface menu, click Sweep 1 Rail.
3. At the Drag seam point to adjust… prompt, examine the direction and seam points of the curves
to make sure they are not twisted, and press Enter.
4. In the Sweep 1 Rail Options dialog box, click OK.
### Round the headband ends
Use the same ellipse that formed the first cross-section curve for the headband to create a rounded end for the
headband. Start by splitting the ellipse in half.
Reset the view
1. On the View menu, click Zoom > Window.
2. In the Perspective viewport, zoom in on the left end of the headband you just created.
Split the ellipse in half
1. Select the ellipse.
2. On the Edit menu, click Split.

<!-- page 160 -->

Headphone - Sweep, Loft, and Extrude
3. At the Select cutting objects… prompt, click the Point option.
4. Turn on Quadrant object snap.
5. At the Point to split curve prompts, snap to the two quadrants at the narrow axis of the ellipse.
6. At the Point to split curve prompt, press Enter.
The ellipse is split into two halves.

<!-- page 161 -->

Headphone - Sweep, Loft, and Extrude
Create a surface of revolution
1. Select left half of the ellipse.
2. On the Surface menu, click Revolve.

<!-- page 162 -->

Headphone - Sweep, Loft, and Extrude
3. At the Start of revolve axis prompt, snap to the end of the ellipse half.
4. At the End of revolve axis prompt, snap to the other end of the ellipse half.
5. At the Start angle... prompt, type 0, press Enter.
6. At the Revolution angle... prompt, type 180, press Enter.
A rounded surface is created at the end of the headband.
7. Repeat these steps for the other side of the headband.

<!-- page 163 -->

Headphone - Sweep, Loft, and Extrude
Mirror the rounded end
1. Select the rounded end.
2. On the Transform menu, click Mirror.
3. At the Start of the mirror plane prompt, type 0.
4. At the End of the mirror plane prompt, drag the mirror line in the y-direction as illustrated.

<!-- page 164 -->

Headphone - Sweep, Loft, and Extrude
Join the surfaces
1. Select the surfaces.
2. On the Edit menu, click Join.
Three surfaces join into one polysurface.

<!-- page 165 -->

Headphone - Sweep, Loft, and Extrude
### Create the speaker wire
Use a separate layer to create the speaker wire.
Reset the layers
1. On the status bar, click the Layer pane.
2. Make Wire Shape Curves the current layer and turn on Wire.
Turn all other layers off.
Reset the view
4
On the View menu, click Zoom > Zoom Extents All.
Make the helix
1. On the Curve menu, click Helix.
2. At the Start of axis… prompt, click AroundCurve.
3. At the Select curve prompt, select the long free-form curve.
4. At the Radius and start point… prompt, type 1 and press Enter.
This sets the radius for the helix.
5. At the Radius and start point… prompt, set Turns=30 and NumPointsPerTurn=8.

<!-- page 166 -->

Headphone - Sweep, Loft, and Extrude
6. At the Radius and start point… prompt, in the Right viewport drag the cursor to the left and click.
Reset the view
1. On the View menu, click Zoom > Window.
2. In the Perspective viewport, zoom in on the left end of the helix you just created.
Match and join the helix to the end curves
1. On the Curve menu, click Curve Edit Tools > Match.
2. At the Select open curve to change - pick near end prompt, select near the left end of the helix.

<!-- page 167 -->

Headphone - Sweep, Loft, and Extrude
3. At the Select open curve to match - pick near end… prompt, select near the lower end of the
vertical curve.
4. In the Match Curve dialog box, under Continuity, click Tangency, under Preserve other end,
click Position, and click Join.

<!-- page 168 -->

Headphone - Sweep, Loft, and Extrude
5. Repeat steps 3 through 6 for the other end of the helix.
Create the speaker wire
1. Select the extended helical curve.
2. On the Solid menu, click Pipe.
3. At the Starting radius… prompt, type .2 and press Enter.

<!-- page 169 -->

Headphone - Sweep, Loft, and Extrude
4. At the End radius… prompt, press Enter.
5. At the Point for next radius prompt, press Enter.
Create the second wire
1. Select the curve at the top left.
2. On the Solid menu, click Pipe.
3. At the Starting radius… prompt, type 0.1 and press Enter.
4. At the End radius… prompt, press Enter.
5. At the Point for next radius prompt, press Enter.
### Mirror the headphone parts
Mirror the parts to create the parts for the other side of the headphones.
Reset the layers
1. On the status bar, click the Layer pane.
2. Turn on all layers.

<!-- page 170 -->

Headphone - Sweep, Loft, and Extrude
Reset the view
4
On the View menu, click Zoom > Zoom Extents All.
Delete all the shape curves
1. Press Esc to deselect everything.
2. On the Edit menu, click Select Objects > Curves.
3. Press the Delete key.
Mirror the left half of the headphones
1. In the Front viewport, window select the objects as illustrated.
(Select the speaker, bracket, small wire, and rotated ellipse.)
2. On the Transform menu, click Mirror.
The Mirror command depends on which viewport is active. It uses the construction plane in the
active viewport to define the mirror plane. The mirror plane is perpendicular to the construction
plane. Two points define the line in this plane about which the selected objects are mirrored.

<!-- page 171 -->

Headphone - Sweep, Loft, and Extrude
3. At the Start of mirror plane prompt, type 0,0.
This is the first point of the mirror line.
4. At the End of mirror plane prompt, turn on Ortho, and drag the mirror line straight up and pick.
### Learn more
For a video tutorial showing a more sophisticated modeling method for a headphone set using Rhino's Gumball
feature, see: Modeling stereo headphones.

<!-- page 172 -->
