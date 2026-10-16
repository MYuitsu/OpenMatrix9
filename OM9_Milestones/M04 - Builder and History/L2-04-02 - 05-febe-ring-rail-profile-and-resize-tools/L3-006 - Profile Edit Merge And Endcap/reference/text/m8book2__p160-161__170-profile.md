---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 160-161
section_title: Profile ￼
section_id: m8book2-170-profile
shared_start_page: True
shared_end_page: False
---

# Profile ￼

Source: matrix_8_book_2.pdf, pages 160-161

<!-- section m8book2-170-profile | Profile ￼ | m8book2 pages 160-161 -->

<!-- page 160 -->

### Command Line Options:
Where is this Command:
Access this command from Tools>3D Printing >
Supports
Mirror X=Off/On: When option is ON, support
mirrors across the X axis.
Steps:
Mirror Y=Off/On: When option is ON, support
mirrors across the Y axis.
SharedStartPoint=Off/On: Shared Start
Point is ON with mirrored supports ON, shares
the start point with every set of supports placed.
(AutoProject must be turned OFF to activate
Shared Start.)
StartDiameter: Set the diameter of the support
start point.
• Select 3D Printing from the Tools menu.
• Select Supports from the 3D Printing menu, (along
the left-hand side below Projects).
• Select a Start point (places the support tip on the
object).
• Choose your settings..
• Select an End point.
• Continue selecting start and ends points, placing
multiple supports if needed.
• Press Enter to complete the command.
Note: By default the supports automatically project
to the Base support surface. To share a start point
turn AutoProject off.
EndDiameter: Set the diameter of the support
end point. (Default End Diameter is the same as
the Start Diameter.)
AutoProject: Projects the support end to the
support surface. (AutoProject must be turned
OFF to activate Shared Start Point.)
## Profile
Support Shared Start Point Mirrored X
‘gvProfile’
Use Curve tools to create your own Profile and
instantly make it a Profile so that it has Viewport
Control Handles and can be used with compatible
profile commands.
### Supports Modes:
Where is this Command:
Access this command from Tools>Profile
Support: This places a single, straight
support in the viewport.
### Profile Options:
Support Shared Start Point: This adds
straight supports while keeping a common
start location. (AutoProject must be turned OFF
to activate Shared Start.)
There are two options to create Profiles using this
tool. Draw a closed, planar curve for Option 1. Or,
use a existing library Profile and edit the profile
using Curve tools in Option 2.
### Option 1:
Support Shared Start Point Mirrored
X: This adds straight, mirrored supports
(across X) while keeping a common start location.
Hand-Drawn Curve Steps:
• Select the Profile tool from the Tools menu.
• Select a closed, planar curve and input into the
Curve in-box by clicking on the white triangle.
Support Shared Start Point Quad
Mirror: This adds straight, mirrored
supports (across both X and Y) while keeping a
common start location.

<!-- page 161 -->

The curve is now a Profile, displaying Viewport
Control Handles and is fully editable.
• Press Enter to complete editing or select Move (in
the Command Line or at the left in the builder) to
continue editing.
Note: Curve must be closed and planar to make a
profile.
### Edit Origin Point:
Hand-drawn profiles most likely do not have the
correct Origin Point. This causes the handles to
orient in an unexpected way. Follow the steps
below to edit the Origin Point.
Before Edit Origin
After Edit Origin
### Move and Select Rail:
Viewport Control Handles before Edit Origin Point
Steps - Move and Select Rail Curve:
• With the Viewport Control Handles visible, choose
“SelectRailCurve” in the Command Line or choose
“Select Rail” at the left in the builder (or select
Move in the Command Line if in Edit mode).
• Select a Ring Rail or other rail curve. (The profile
temporarily disappears while your selection is
made.) The Profile is now oriented to the selected
rail and can be edited.
Note: Follow the steps outlined in Profile Placer
at the beginning of the Tools Chapter to edit your
profile.
Viewport Control Handles after Edit Origin Point
Steps to Edit Origin Point:
Profile before Select Rail
• Select Profile from the Tools menu.
• Select a closed, planar curve and input into the
Curve in-box by clicking on the white triangle.
• Select Edit in the Command Line, in the builder (or
choose the “E” Viewport Control Handle).
• Select “EditOrigin” in the Command Line.
• A white origin point will appear on the profile.
Select (on the cross point of the profile curve)
left-click and hold while moving it to the bottom,
middle of the profile. (Do not grab by the white
offset circle.)
• Release to place the new Origin Point.
