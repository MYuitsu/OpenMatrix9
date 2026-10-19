# Cage Edit ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 24-26
- Section title: Cage Edit ￼
- Section id: m8book2-022-cage-edit
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p24_img01.png, m8book2_p24_img02.png, m8book2_p26_img01.png, m8book2_p26_img02.png, m8book2_p26_img03.png

---
<!-- section m8book2-022-cage-edit | Cage Edit ￼ | m8book2 pages 24-26 -->

<!-- page 24 -->

## Cage Edit
‘CageEdit’
Deforms captive objects smoothly using one, two
and three-dimensional cages and control point
editing.
Where is this Command:
Access this command from Transform>Cage
Edit
Right Click Command: Transform>Cage Edit
using Bounding Box
Steps:
• Select Cage Edit from the Transform menu.
• Select the captive objects (object to edit).
• Press Enter when done.
• Select a control object. (Bounding Box, Line, etc.)
• Define the region to edit by selecting and
manipulating the control points on the cage.
Notes:
1. Cage editing allows smooth deformation of
surfaces with dense control points.
2. Polysurfaces are not broken apart at the seams
by CageEdit deformation.
3. The control points can be arranged in one, two,
or three directions depending upon the control
object chosen.
4. CageEdit allows both overall deformation and
partial deformation of an object.
5. The control object can be made with the Cage
command or can be an existing surface or curve.
6. Box, plane, and line control objects can be
created within the command.
7. Preexisting objects and the object itself can be
also used.
8.The volume of space to be deformed can be
limited to specified spherical, cylindrical, and box
volumes.
9. The falloff distance between the deformed
volume and the rest of the objects can be
specified.
10. History is built in regardless of the History
setting.
### Options for Control Object:
BoundingBox: Uses the captive object’s
Bounding Box as the control object.
### CoordinateSystem:
Sets the coordinate system for the Bounding Box.
Bounding Box with World Coordinates placed around
object with 4 Points in X and Y and 3 Points in Z.
CPlane: Uses Construction plane coordinates.
World: Uses World coordinates.
3Point: Pick three points to establish a
coordinate system.
### Line:
Draw a line to be used as the control object.
Steps:
• Select Start of Line.
• Select End of Line.
• Select NURBS Parameters. (Degree and Point
Points selected and moved to edit the object

<!-- page 25 -->

Count)
• Press Enter.
• Select a Region to Edit. (Global, Local or Other)
• Press Enter when done.
• Select the Cage points.
• Press Enter when done.
• Select a Region to Edit. (Global, Local or Other)
• Press Enter when done.
### Deformation:
Nurbs Parameter: Specify the parameters for
the line.
Accurate: Accurate is slower to update and may
result in denser surfaces when deformed objects
are refit.
Degree: Specifies the degree of the curve (or
surface).
Fast: Creates surfaces that have fewer control
points and therefore are less accurate.
Note: When drawing a high-degree curve, the
output curve will not be the degree you request
unless there is at least one more control point
than the degree.
### PreserveStructure:
PointCount: Specifies the number of control
points.
### Rectangle:
Yes: Preserves the control point structure of the
surface. Deformation may be less accurate if there
are too few control points in the object.
No: Objects are refit as needed with more control
points to allow accurate deformation.
Draw a plane to be used as a control object.
### Region Options:
Steps:
• Select first corner of Rectangle.
• Select other corner or length (3Point, Vertical, etc.)
• Select NURBS Parameters (UDegree, VDegree,
UPointCount, VPointCount)
• Press Enter when done.
• Select a Region to Edit. (Global, Local or Other)
• Press Enter when done.
Global: Objects are deformed throughout 3-D
space. The influence of the control object on
the captives is not limited to the region inside
cage objects or adjacent to control curves or
surfaces. Objects that are only partly contained in
cage objects are still deformed throughout. The
influence of control objects is greatly magnified
the farther captives are outside them.
3Point: Select start of edge, end of edge and
width.
Local: Specify a falloff distance from the control
object to surrounding space. Captives or parts of
captive objects that fall outside the falloff distance
are not deformed.
Vertical: Select start of edge, end of edge and
height.
Center: Select center of rectangle and other
length or corner.
Other: Define a box, sphere or cylinder that
limits the influence of the control object over
the captives in space. Specify a falloff distance.
A large cage can be used to deform an object
smaller than the cage.
### Box:
Draw a box to be used as a control object.
Steps:
• Select first corner of base.
• Select other corner of base or length.
• Select the height.

<!-- page 26 -->

### Cage points:
### Command Line Options:
Basepoint: Use this option when the object to
be arrayed is not on the curve. It establishes a
reference location of the Array object to the path
curve.
### Method:
Once the box shape is defined, define the number
of control points and degree of the cage in each
of three directions.
XPointCount/YPointCount/ZPointCount:
Number of control points in the X, Y, and Z
directions
Distance Between Items: Specify the
distance between items. The number of items is
determined by the length of the curve.
XDegree/YDegree/ZDegree:  Number of
Degrees in the X, Y, and Z directions.
## ReleaseFromCage
‘ReleaseFromCage’
Use the command ‘ReleaseFromCage’ to remove
selected objects from the influence of a control
object.
## Array Along Curve
‘ArrayCrv’
Use Array Along Curve to copy an object and
array it an indicated number of times along a
curve.
Number of items: Type the number of items to
array along the curve.
Where is this Command:
Access this command from Transform>Array
Along Curve
Steps:
Orientation:
Freeform: Objects are rotated in three
dimensions as they are arrayed along the path
curve.
• Select Array Along Curve from the Transform
menu.
• Select the Object(s) to Array.
• Press Enter when done.
• Select a path curve near the end where you want
the array to start. Or, use the Basepoint option.
Objects arrayed in a Freeform Orientation.
The object should be touching the path curve and
it should be placed so that it is first in the line of
objects to be arrayed and need not be repositioned
prior to the array.
No Rotation: The objects do not rotate as they
are arrayed, but remain in their original orientation.
• Specify the Number of elements to array or the
Spacing Distance along the curve.
• Type a value of 1 or more for the Number of
elements.
