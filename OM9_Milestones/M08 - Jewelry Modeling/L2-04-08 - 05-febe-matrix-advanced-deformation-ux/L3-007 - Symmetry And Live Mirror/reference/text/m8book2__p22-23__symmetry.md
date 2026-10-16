# Symmetry ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 22-23
- Section title: Symmetry ￼
- Section id: m8book2-020-symmetry

---

<!-- section m8book2-020-symmetry | Symmetry ￼ | m8book2 pages 22-23 -->

<!-- page 22 -->

### Options:
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when Copy mode is turned ON.
XFlip/YFlip: Reverses the orientation relative to
the original orientation to the Construction plane.
Rotate: Pick an angle or two Reference points to
rotate the oriented object.
## Orient on Curve
‘OrientOnCrv’
This command orients an object along a selected
curve at a chosen Base point.
Select object to Orient in the Looking Down
Viewport.
### Command Line Options:
Where is this Command:
Access this command from Transform>Orient
on Curve
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is turned ON.
Steps:
Perpendicular: Orients the object perpendicular
to the curve. The perpendicular plane is
determined by the current Construction plane
when the base point is picked.
## Symmetry
• Select Orient on Curve from the Transform menu.
• Select the Object(s) to Orient.
• Press Enter when done.
• Pick a Base point (where the object will touch the
curve) on the object to be oriented.
• Select the Orientation curve.
• Pick a new Base point on the curve.
‘Symmetry’
This command mirrors curves and surfaces, makes
the mirrored half tangent to the original. As the
original object is edited, the mirrored half updates
to match the original.
Where is this Command:
Access this command from
Transform>Symmetry
Object is oriented in the Through Finger Viewport.
Note: Rhino History recording must be enabled when
the command is run.
Steps:
• Select Symmetry from the Transform menu.
• Select curve end or surface edge.
• Pick the Start of the Symmetry plane (this is similar
to the Mirror command.)
• Pick the End of the Symmetry plane.

<!-- page 23 -->

### Command Line Option:
### Continuity:
Determines how the continuity between the two
selected objects is handled.
None: No constraint. The end of curve or surface
edge does not change shape.
Select curve top end
Position: The mirrored curve or surface edge
will connect to the original, blending without a
tangency (G0 Continuity),
Smooth: The mirrored curve or surface will
connect to the original with a tangent blend (G2
Continuity). This is the default.
## Solid Points ON
Start of line of Symmetry is F4. End of line is above
the curve.
‘SolidPtOn’
Turn on control points at the ends of surface and
joined polysurface edges, including closed solids.
Where is this Command:
Access this command from Transform>Solid
Points ON
Steps:
• Select Solid Point On from the Transform Menu.
• Select an object to turn edit points ON.
• Press ESC to turn OFF points.
Symmetry Continuity is Smooth.
Solid Points ON             Select a point to move
Object after points were moved
With Control Points turned ON, adjust the points to
reshape the curve. Symmetry updates the changes to
the other side (mirrored).
