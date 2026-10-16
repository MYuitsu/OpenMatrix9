# Plane ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 73-74
- Section title: Plane ￼
- Section id: m8book2-090-plane
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-090-plane | Plane ￼ | m8book2 pages 73-74 -->

<!-- page 73 -->

### T-Spline Options:
2Point: Draws the Circle from the two ends of its
diameter.
Steps:
OutputType:
Smooth: This is the default mode for T-Splines
primitives.
• Pick the start and end Diameter.
• Pick the second Radius for the height.
Box: Setting the output to box will allow for faster
editing of the model.
3Point: Draws a circle through three points on
the circumference.
Steps:
VerticalFaces: Set the amount of control points
in the vertical direction of the Torus.
• Pick three points that define the circumference.
• Pick the second Radius for the height.
AroundFaces: Set the amount of control points
in the horizontal direction of the Torus.
3Point Options:
Radius: Specifies the radius and orientation for
the base circle after two points have been picked.
Steps:
Symmetry: When set to ‘Yes’ options for Axial
Symmetry and Radial Symmetry appear in the
Command Line.
• Pick a location or type a radius value.
• Pick a direction to orient the circle.
Tangent: Draws a circle tangent to curves.
Steps:
AxialSymmetry: Adds Symmetry across an
axis, when set to ‘Yes’ you can add Symmetry
in the X, Y, or Z axes. XSymmetry: Option is only
available when AxialSymmetry is set to ‘Yes’. Adds
Symmetry along the X axis.
• Pick the first tangent location on the first curve.
• Pick the second tangent curve or Radius.
• Pick the third tangent curve.
• Pick the second Radius for the height.
YSymmetry: Option is only available when
AxialSymmetry is set to ‘Yes’. Adds Symmetry
along the Y axis.
AroundCurve: Draws a circle perpendicular to a
curve.
Steps:
ZSymmetry: Option is only available when
AxialSymmetry is set to ‘Yes’. Adds Symmetry
along the Z axis.
Radial Symmetry: Adds Radial Symmetry when
set to ‘Yes’
• Select a curve and pick the center of the circle on
the curve.
• Pick a Radius for the Torus.
• Pick the second Radius for the height.
FitPoints: Draws a circle by fitting to selected
point objects, curve and surface control points,
and mesh vertices.
SymmetrySegments: When Radial Symmetry
is set to ‘Yes’ this option becomes available. Select
how many Symmetry segments or ‘pie pieces’ the
Torus is broken into.
Steps:
• Select point objects or control points.
• A minimum of three points or control points is
required.
FacesPerSegment: When Radial Symmetry is
set to ‘Yes’ this option becomes available. Select
how many faces appear in each Symmetry.
## Plane
### Torus Options:
Vertical: Draws the Torus perpendicular to the
Construction plane.
‘tsPlane’
Creates a T-Spline rectangular Plane. There are
three modes for creating a plane: 2Point, 3Point,
Vertical and Center. The default is 2Point.

<!-- page 74 -->

### Options:
Where is this Command:
Access this command from T-Spline>Plane
Steps:
OutputType=Smooth/Box: Controls whether
the resulting frame is output in Smooth mode or
in Box mode.
XFaces: Defines the number of faces along the X
axis
• Select Plane from the T-Spline menu.
• Pick the first corner.
• Pick the Second corner
### Command Line Options:
YFaces: Defines the number of faces along the Y
axis
Axial Symmetry=Yes/No: If yes resulting
T-Spline will have Symmetry applied as specified.
3Point: The “3Point” option draws the plane on
an angle, as opposed to aligning the Plane along
the viewport grid.
Steps:
• Pick the first corner.
• Pick the second corner.
• Pick the third corner.
XSymmetry=Yes/No: This option is available
when Axial Symmetry is set to ‘Yes’. This will
add Symmetry along the X axis in the resulting
T-Spline.
YSymmetry=Yes/No: This option is available
when Axial Symmetry is set to ‘Yes’. This adds
Symmetry along the Y axis in the resulting
T-Spline.
Vertical: The “Vertical” Command Line option
draws the Plane, beginning from one point
in space. The next click draws a line, then a
rectangle. This option allows you to draw a Plane
on an angle and determine its exact orientation in
the viewport.
## T-Splines Tube from Curves
Steps:
• Pick the first corner.
• Pick the second corner.
• Pick the third corner.
‘gvtsTubeFromCrvs’
This useful builder was created to quickly build
ring shanks and control the topology of the shank
with its Command Line options.
Center: The “Center” Command Line option
begins drawing the plane from it’s center out, as
opposed to drawing the plane from one of its
corners.
Where is this Command:
Access this command from T-Spline> T-Splines
Tube From Curves
Steps:
Steps:
• Pick a point to set the center of the Plane.
• Pick a second point to set the length and width of
the Planes edges.
• Start with two existing curves.
• Select the Tube from Curves command in the
T-Spline menu.
• Select the first curve.
• Select the second curve.
• Adjust the Tube using the Viewport Control
Handles and the options in the Command Line.
Selecting a second corner for the Plane (left) and
Plane complete (right).


---

## Images

- ![m8book2_p74_img01.png](../images/m8book2_p74_img01.png) page 74 (shared page)
