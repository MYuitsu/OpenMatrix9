# Through Points

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 157-158
- Section title: Through Points
- Section id: m8book1-167-through-points
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p157_img01.png, m8book1_p157_img02.png, m8book1_p157_img03.png

---
<!-- section m8book1-167-through-points | Through Points | m8book1 pages 157-158 -->

<!-- page 157 -->

Where is this Command:
Access this command from Curve>Apply UV
Curves
Line: The arc is extended with a line segment,
which is joined to the arc converting it to a
polycurve.
Steps:
## Create UV Curves
• Select planar curves on world XY plane to apply to
a surface.
• Select a surface to apply the planar curves to.
• The curves are mapped to the surface.
Corresponding 3-D curves are created on the
surface.
‘CreateUVCrv’
Projects a surface’s untrimmed boundary and
trim curves onto the world x-y plane. The results
can be used in conjunction with Apply UV Curve
or Flow Along Surface. Use the results as a 2D
pattern on which to layout design details and then
return onto the 3D surface.
Note: Use the CreateUVCrv command to generate
planar versions of the u and v curves of a surface
so you can use them as a guide to orient your text
or pattern. Then Apply UV curves to the original
surface.
Where is this Command:
Access this command from Curve>Create UV
Curves
Steps:
• Select Create UV Curves from the Curve menu.
• Select a surface.
Note: The control polygon determines the sizes of
the uv curves.
Create UV Curves works with single untrimmed
surfaces.
A pattern is created and placed on the UV curve.
Then the Apply UV Curve command is used to place
the pattern onto the rings surface.
## Through Points
‘CurveThroughPt’
Fits a Curve through selected point objects.
Where is this Command:
Access this command from Curve>Through
Points
The UV curves are created from the ring.
Steps
• Select Through Points from the Curve menu.
• Select points.
## Apply UV Curves
‘ApplyCrv’
Wrap planar curves onto a surface. Use the
Create UV Curves prior to this command to create
template on which to design then use the Apply
UV Curves command to transfer the planar curves
onto the 3D surface.

<!-- page 158 -->

### Command Line Options:
Closed: Choose to create a closed curve or an
open curve. Toggle between Yes or No.
## Extend Curve
Degree: Specifies the degree of the curve (or
surface). When drawing a high-degree curve, the
output curve will not be the degree you request
unless there is at least one more control point
than the degree.
### CurveType
‘Extend’
Lengthens a given curve. Extend a curve to
touch another curve (boundary object) or its own
endpoint to create a closed curve.
Specify the type of curve created a control point
curve or a interpolated curve.
Where is this Command:
Access this command from Curve>Extend
Curve
CurveType=Interpolated: The curve passes
through the vertices, points, or control points.
Steps:
CurveType=ControlPoint: The curve’s control
points pass through the vertices, points, or control
points.
• Select Extend from the Curve menu.
• Select a boundary object, type a length, or press
Enter to extend without boundary edges.
• Select a curve or an surface edge near the end.
• The extension will now join to the original curve.
To extend to multiple objects, pick the end of
the curve again to extend the curve beyond
the first boundary to the next.
Knots: Determines how an interpolated curve is
parameterized. When the spacing between the
picked points is equal for the whole curve, all three
parameterizations generate the same curve.
Notes: Use any combination of curves, surfaces, and
solids as boundary edges. If you want to extend two
curves to meet each other simultaneously, use the
Connect command.
When you draw an interpolated curve, the points
you pick are converted into knot values on the
curve. The parameterization means how the
intervals between knots are chosen.
### Command Line Options:
Type=Natural: Lines, polylines, and polycurves
with a line at the end to extend are extended by
line. Arcs and polycurves with arc at the end to
extend are extended by same radius arc. Other
curves are extended smooth.
Type=Arc: Makes an arc extension tangent to
the original curve.
Uniform: The knot spacing is always 1 and is
not based on the physical spacing of the points.
Uniform parameterization can be used if the
point spacing is roughly equal and it is desirable
to draw several curves that have the same
parameterization. This is possible only with uniform
curves (unless you rebuild the curves). Uniform
guarantees that every control point affects the
surface exactly the same way, no matter how
much the surface is edited.
Arc Options:
Center: Extends a curve with an arc by picking
the arc’s center and endpoint.
Steps:
Chord: The spacing between the picked points is
used for the knot spacing. This makes curves that
have widely varying point spacing behave better
than uniform curves.
• Pick the arc’s center point.
• Pick the arc’s end point.
SqrtChrd: The square root of the spacing
between picked points is used for the knot
spacing.
