# Curve to Lines

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 170-170
- Section title: Curve to Lines
- Section id: m8book1-191-curve-to-lines
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---
<!-- section m8book1-191-curve-to-lines | Curve to Lines | m8book1 pages 170-170 -->

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
