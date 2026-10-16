# Flow Along Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 30-30
- Section title: Flow Along Surface ￼
- Section id: m8book2-030-flow-along-surface

---

<!-- section m8book2-030-flow-along-surface | Flow Along Surface ￼ | m8book2 pages 30-30 -->

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
