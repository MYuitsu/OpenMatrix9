# Flow Along Curve ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 29-30
- Section title: Flow Along Curve ￼
- Section id: m8book2-029-flow-along-curve
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p29_img01.png, m8book2_p29_img02.png, m8book2_p29_img03.png, m8book2_p30_img01.png

---
<!-- section m8book2-029-flow-along-curve | Flow Along Curve ￼ | m8book2 pages 29-30 -->

<!-- page 29 -->

Steps:
• Select Flow Along Curve from the Transform
menu.  Turn ON Rhino History Recording and
History Update before starting the command.
• Select the Object(s) to Flow along a curve.
• Press Enter when done.
• Select the Base curve near one end.
• Select the Target curve near the matching end.
Preview of the Array. Select the corners of the
rectangle for X and Y spacing.
Measure the Target curve length in advance and
make the Base curve the same length.
History: After running this command, the objects
on the Base curve can be scaled or transformed
for the desired outcome. Objects flowed along the
curve will update with Rhino History enabled.
Array is complete
### Command Line Options:
Number: XNumber/YNumber/ZNumber
Changes the number of objects in the X, Y, and Z
directions.
Preview=Yes: Displays a dynamic preview. You
can change the options at this point and the
preview will update.
Preview=No: No preview will display. There is no
opportunity to change the options.
Base curve (green) and Target curve (yellow) are
used with Flow Along Curve.
Spacing: XSpacing/YSpacing/ZSpacing:
Changes the spacing in the X, Y and Z directions.
### Command Line Options:
## Flow Along Curve
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is on.
‘Flow’
The Flow Along Curve command orients objects
from a Base curve to a Target curve.
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed as
they flow along the curve (default).
Where is this Command:
Access this command from Transform>Flow
Along Curve

<!-- page 30 -->

Line: Draw a line to be used as the Base curve.
Local:
Yes: Specify circles to define a ‘Tube’ of influence
around the Base curve. Everything inside the tube
is flowed. Everything outside the tube is fixed.
Inside the tube wall the flow dies off. This enables
part of the objects flowed to be deformed and
part is left alone.
No: All of the space is deformed (default).
Stretch:
No: The length of the objects flowed does not
change to match the Target curve. (Use when
Base curve and Target curves are different
lengths. The objects flowed remain the same
length.)
Yes: Objects are stretched or compressed so that
the relationship between the Target curve and the
Base curve is the same.
Objects placed on a Base surface are flowed to a
Target Surface.
## Flow Along Surface
### Command Line Options:
This command flows objects from a Base
surface to a Target surface.
Copy=Yes/No: Specifies whether or not the
objects are copied. A plus sign appears at the
cursor when copy mode is ON.
Where is this Command:
Access this command from Transform>Flow
Along Surface
Steps:
Rigid:
Yes: Individual objects are not deformed as they
are transformed.
No: Individual selected objects are deformed.
Plane: Draw a plane instead of using an existing
surface Base surface. See the Rectangle
command for option descriptions.
• Select Flow Along Surface from the Transform
menu.
• Select the Object(s) to Flow along a surface.
• Press Enter when done.
• Select the Base surface near a corner.
• Select the Target surface near a matching corner.
## Splop
‘Splop’
Copies, rotates, scales, and wraps objects on a
surface using a reference sphere
Where is this Command:
Access this command from Transform>Splop
Steps:
• Select Splop from the Transform menu.
• Select object(s) to Splop.
Creating a Base Surface: In advance, select
the Target surface and use “ShrinkTrimmedSrf’ (if
the surface has been previously trimmed) and use
‘Create UV Curves” to create curves for making
the Base. Then, use Sweep or ‘Planar Curves’ to
build a Base surface.
History: Flow Along Surface is also a Rhino
History enabled command. So, have History
recording and Update turned ON prior to running
the command. Then, after objects on flowed onto
the Target surface they can be scaled, moved or
transformed and will update accordingly.
