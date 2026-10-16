# Solid Extrude All ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 229-231
- Section title: Solid Extrude All ￼
- Section id: m8book1-249-solid-extrude-all
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p229_img01.png, m8book1_p229_img02.png, m8book1_p229_img03.png, m8book1_p230_img01.png, m8book1_p230_img02.png, m8book1_p230_img03.png, m8book1_p231_img01.png, m8book1_p231_img02.png

---
<!-- section m8book1-249-solid-extrude-all | Solid Extrude All ￼ | m8book1 pages 229-231 -->

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

<!-- page 230 -->

curves. The output will be a polysurface. When
UseExtrusions is on, this setting has no effect.
Straight: Extrudes objects at a tangency.
Polysurfaces are made with straight sides.
SetBasePoint: Specify a location that serves as
the first point when picking two points that set
the extrusion distance.
Tapered: Extrudes objects at an angle.
Polysurfaces are made with angled sides with one
end smaller in two dimensions than the other end.
### Command Line Options (Tapered):
ToPoint: Creates angled polysurfaces that meet
at a single designated point.
See instructions above (Command Line Options
- Straight) for shared options: Direction, Solid,
DeleteInput, ToBoundary and Set BasePoint.
AlongCurve: Select a ‘Path Curve’ for the
original object to create the polysurface along.
### Command Line Options (Straight):
Extrude closed curves Tapered
Draft Angle: Enter a value for the angle of the
taper.
Extrude closed curves Straight
Direction: Pick Two points to establish the
direction angle.
Steps:
Corners=Sharp/ Round/ Smooth: Sharp
produces straight-edged corners. Round makes
Arc curve-shaped corners. And, Smooth creates
Blend curve-shaped corners.
• Pick a base point.
• Pick a second point that establishes the direction
angle.
FlipAngle: This option reverses the Draft Angle
from a positive to a negative amount or vice versa.
### Command Line Options (To Point):
BothSides: Draws the object on both sides of
the start point, creating the extrusion twice as
high as the distance value.
See instructions above (Command Line Options
- Straight) for shared options: Solid, DeleteInput,
ToBoundary and SplitAtTangents.
Solid: If the profile curve is closed and planar,
both ends of the extruded object are filled with
planar surfaces and joined to make a closed
polysurface.
DeleteInput: Deletes the original object used for
the extrusion.
ToBoundary: Extrudes the object to a boundary
surface.
### Command Line Options (Along Curve):
See instructions above (Command Line Options
- Straight) for shared options: Solid, DeleteInput,
ToBoundary and SplitAtTangents.
SplitAtTangents: Specifies whether resulting
surfaces will be one surface or a polysurface
when the input curves are joined tangent curves.
Surfaces in the resulting polysurface object
correspond to the tangent sub-curves in the input

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
