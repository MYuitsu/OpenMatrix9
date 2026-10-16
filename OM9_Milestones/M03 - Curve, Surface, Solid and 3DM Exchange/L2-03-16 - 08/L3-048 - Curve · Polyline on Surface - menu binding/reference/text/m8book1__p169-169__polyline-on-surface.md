# Polyline on Surface

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 169-169
- Section title: Polyline on Surface
- Section id: m8book1-189-polyline-on-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p169_img01.png, m8book1_p169_img02.png

---
<!-- section m8book1-189-polyline-on-surface | Polyline on Surface | m8book1 pages 169-169 -->

<!-- page 169 -->

## Sketch on Mesh
Pick points along the
selected surface. The polyline
will be constrained to the
surface.
‘Sketch N’
Draws a curve on a mesh by dragging the mouse
while depressing the left mouse button.
Where is this Command:
Access this command from Curve>Sketch on
Mesh
Steps:
## Interp Curve on Surface
• Select Sketch on Mesh from the Curve menu.
• Select the mesh to Sketch on.
• Click and hold the left mouse button and drag the
mouse to sketch.
• Release the mouse button to end the curve.
Note: Drawing a single curve in multiple viewports is
allowed using this method.
‘InterpCrvOnSrf’
Draws a curve through selected locations on a
selected surface.
### Command Line Options:
Closed=No: Creates an open curve
Where is this Command:
Access this command from Curve>Interp Curve
on Surface
Steps:
Closed=Yes: Creates a closed curve. Select the
option before dragging points with the mouse.
The curve closes when you stop dragging points.
## Polyline on Surface
• Select Interp Curve on Surface from the Curve
menu.
• Select the surface.
• Pick the start of the curve.
• Pick the subsequent points for the curve to pass
through.
• Press Enter when done.
‘gvPolylineOnSurface’
Draws a polyline on a surface by picking points on
the selected surface.
Note: The curve will not cross seams in the surface.
Object snaps End, Center, Mid, Near, Knot, and
Intersect can be used.
Where is this Command:
Access this command from Curve>Polyline on
Surface
Steps:
• Select Polyline on Surface from the Curve menu.
• Select the surface.
• Pick the points on the surface to create the
polyline. Press Enter when done.
Pick points along the
selected surface. The interp
curve will be constrained to
the surface.
Note: Drawing a single curve in multiple viewports is
allowed using this method.
### Command Line Options:
Undo: Reverses the last action
