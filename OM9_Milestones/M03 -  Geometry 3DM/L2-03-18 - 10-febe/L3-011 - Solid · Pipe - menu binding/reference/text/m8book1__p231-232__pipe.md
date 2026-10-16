# Pipe ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 231-232
- Section title: Pipe ￼
- Section id: m8book1-250-pipe
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p231_img01.png, m8book1_p231_img02.png, m8book1_p232_img01.png, m8book1_p232_img02.png, m8book1_p232_img03.png

---
<!-- section m8book1-250-pipe | Pipe ￼ | m8book1 pages 231-232 -->

<!-- page 231 -->

### Chain Edge Options:
SubCurve= Yes/No: Select a start and end
point along the Path Curve, to extrude the shape
along only that part of the path curve.
AutoChain Yes/No: Selecting a curve or a
surface edge automatically selects all curve
segments connected with the level of continuity
set by the Chain Continuity option.
Chain Edges Steps:
• Select Chain Edges option in the Command Line.
• Select first chain segment. (If Auto Chain is
enabled the chain of edges will auto select.)
• Select next chain segment. (If Auto Chain is set to
‘No’.)
• Press Enter when done.
Extrude a closed curve along a Path Curve using
Extrude All - Along Curve.
## Pipe
‘Pipe’
This command creates an open or closed tube-
like surface around a curve as though a round
profile curve, with a radius selected by the user,
has been swept along the chosen curve. This is a
useful tool when round “wire” is a required as part
of a design.
ChainContinuity: Controls the level of continuity
(Position, Tangency or Curvature) required
between segments to be selected with the Auto
Chain option. With Position the curves are position
continuous (G0) and touch at the ends. Tangency;
the curves touch at the ends and go the same
direction a the point where they touch (G1).
Curvature continuity measures position, direction
and radius of curvature at the ends. They go the
same direction and have the same radius (G2).
Where is this Command:
Access this command from Solid>Pipe
or from the F6 menu with a curve selected.
Direction: ‘Forward’ selects curves in the positive
curve direction. ‘Backwards’ selects curves in the
negative curve direction. ‘Both’ selects curves in
both the positive and negative curve direction.
Right-Click Command: Pipe with Chain Edges
Steps:
GapTolerance: If the gap between two edges or
curves is less than this value, the chain selection
will ignore the gap and will select the next
segment.
• Select Pipe from the Solid menu.
• Select curve to create Pipe around.
• Set the start Radius at the beginning of the Pipe.
Choose the Command Line options.
• Set the Radius at the end of the Pipe.
• Set a point for the next radius, or press Enter for
none.
AngleTolerance: When Continuity is set to
Tangency, if the angle between two edges or
curves is less than this value, the chain selection
will consider the criteria for continuity met and will
select the next segment.
Undo: Undo last segment selection.
Next: Select next segment.
All: Select all segments.
Pipe is
created
around a
single curve.

<!-- page 232 -->

### Command Line Option:
Multiple: Allows selecting more than one curve.
Multiple curves are
able to be selected
with the ‘Multiple’
option.
Shape Blending of Pipe is Local (left) or Global
(right).
## Box
### Pipe Options:
‘Box’
Draws a solid Box.
Diameter/Radius: Toggle between using a
value for Radius or Diameter when creating Pipe.
Where is this Command:
Access this command from Solid>Box
Right-Click Command: Box - 3 Points
Thick=No/Yes: When set to ‘No’ the pipe has
only one wall. When set to ‘Yes’ the pipe has two
walls creating a tube that follows the selected
curve.
Steps:
Pipe Thick (Yes) Steps:
• Pick the first start Radius.
• Pick the second start Radius.
• Pick the first end Radius.
• Pick the second end Radius.
The first and second Radius set the inside and
outside diameter of the tube.
• Select Box from the Solid menu.
• Draw the base rectangle of the box by selecting
the first corner of the base.
• Select the other corner of the base or length.
• Select the Width. Press Enter to use length.
• Pick the Height of the box. Press Enter to use
Width.
Draw a Box using the
‘Box’ command.
Cap = None/ Flat/ Round: Specifies the type
of the end of the Pipe. None has no cap. The pipe
is an open surface. Flat caps the end of the pipe
with a planar surface. Round caps the end of the
pipe with a hemispherical surface
### Command Line Options:
FitRail No/Yes: If the curve is a polycurve of
lines and arcs, the curve is fit and a single surface
is created Otherwise, the result is a polysurface
with joined surfaces created from the polycurve
segments.
Diagonal: Draws the base rectangle from two
diagonal corners. No option for side length is
offered.
Diagonal steps
ShapeBlending (Single Curve Only):
Local: The Pipe radius stays constant at the ends
and changes more rapidly in the middle.
Global: The Radius is linearly blended from one
end to the other, creating Pipes that taper from
one radius to the other.
• Pick two opposite corners - first and second
corners.
• Or, select the Cube option. Cube allows you to
select diagonal corners, one at the base and the
other at the top for height.
• Next, pick a height that also define the cubes
orientation.
