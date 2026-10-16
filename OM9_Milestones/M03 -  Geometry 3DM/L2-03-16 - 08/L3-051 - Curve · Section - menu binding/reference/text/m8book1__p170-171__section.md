# Section

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 170-171
- Section title: Section
- Section id: m8book1-192-section
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p171_img01.png, m8book1_p171_img02.png, m8book1_p171_img03.png

---
<!-- section m8book1-192-section | Section | m8book1 pages 170-171 -->

<!-- page 170 -->

## Curve to Lines
Tolerance: The tolerance at segment midpoints,
overriding the system tolerance setting.
‘Convert’
Changes the structure of a curve to a polyline or
arc segments.
MinLength: The minimum segment length.
Specify 0 (zero) for no minimum limit.
MaxLength: The maximum segment length.
Specify 0 (zero) for no maximum limit.
Where is this Command:
Access this command from Curve>Curve to
Lines
### Output Layer Options:
Steps:
OutputLayer: Specifies the layer for the results
of the command.
• Select Curve to Lines from the Curve menu.
• Select curves to convert.
• Select the options.
Current: Places the results on the current layer.
### Command Line Options:
Input: Places the results on the same layer as the
input curve.
## Section
Output:
Arcs: Converts the curve to arc segments.
Sections of curve that are nearly straight are
converted to straight-line segments.
Lines: Converts the curve to polyline segments.
‘Section’
Creates section curves on a surface, polysurface,
or mesh.
Where is this Command:
Access this command from Curve>Section
Steps:
SimplifyInput:Yes/No
Yes: Simplifying ensures that NURBS curves
that consist of arc and line segments are split
into proper arc and line segments making the
conversion to arcs and lines more accurate.
• Select Section from the Curve menu.
• Select objects
• Pick the start of the section plane.
• Pick the end of the section.
No: Simplifying can be too aggressive, converting
normal NURBS curves into arcs or lines, especially
if the curves are very small in relation to the
absolute tolerance. In that case turning off
simplifying may give a better and more accurate
answer.
Section curves and points are created by
intersecting the selected objects with the section
plane, which is perpendicular to the construction
plane.
• Press enter when you finish creating sections
DeleteInput: Deletes the original geometry from
the file.
Note: The curve will not cross seams in the surface.
Object snaps End, Cen, Mid, Near, Knot, and Int can
be used.
AngleTolerance: The maximum angle between
segments at endpoints. Specify 0 (zero) to allow
non-tangent arcs. The output will have kinks, but
since it consists of arc segments, there will be
fewer segments than if the Lines option was used.
This is useful for approximating a curve with the
smallest number of segments.

<!-- page 171 -->

No: Section curves will not be created for objects
that are not cut by the drawn section plane.
Yes: Section curves will be created for all selected
objects as though the drawn section plane
extended through them.
## Match
The green curves are the resulting section curve
across the surface.
‘Match’
Use this command to Match the end points of two
curves so that the two curves start and end in
the same location. This command is useful when
two curves are very close but do not touch, and
so Join cannot be run on them. Match the curves
first, and then run Join and they will join up.
### Command Line Options:
AssignLayersBy: Specifies the layer for the
contour curves and points.
Where is this Command:
Access this command from Curve>Match
Steps:
CurrentLayer: The output will be on the current
layer.
InputObject: Contour curves and points will be
on the same layer as the input objects.
• Select Match from the Curve menu.
• Select the end of an open curve.
• Select the curve to match.
• Choose the options for the Match.
JoinCurves: Specifies how contour curves
created from polysurfaces will be joined.
ByPolysurface: Curves in the same contour
plane created from a polysurface will be joined.
ByContourPlane: All curves in the same
contour plane will be joined.
Two curves before match command on the left. Right
is a preview of the curves after the match command
has run.
None: No curve joining.
### Command Line Options:
GroupObjectsByContourPlane: Specifies
how contour curves will be grouped.
SurfaceEdge: Matches the curve perpendicular
to the curve and uses the surface to determine
continuity.
No: No grouping.
### Options:
Yes: Curves on the same contour plane will be
grouped.
Sets the continuity of the match.
ExtendSection: Specifies whether the section
curves will automatically apply to all selected
objects if the section plane does not intersect
them.
Position: Ensures the points of the line match up
perfectly – meaning that the end point of the first
line is located at the same coordinates on the grid
as the start point of the next line.
