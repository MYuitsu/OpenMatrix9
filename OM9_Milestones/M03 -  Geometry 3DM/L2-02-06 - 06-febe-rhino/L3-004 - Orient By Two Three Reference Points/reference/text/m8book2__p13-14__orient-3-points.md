# Orient 3 Points ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 13-14
- Section title: Orient 3 Points ￼
- Section id: m8book2-008-orient-3-points
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p13_img01.png, m8book2_p13_img02.png, m8book2_p13_img03.png, m8book2_p14_img01.png, m8book2_p14_img02.png, m8book2_p14_img03.png, m8book2_p14_img04.png

---
<!-- section m8book2-008-orient-3-points | Orient 3 Points ￼ | m8book2 pages 13-14 -->

<!-- page 13 -->

## Orient 2 Points
‘Orient’
Moves or copies, rotates, and scales objects using
two Reference and two Target points.
Where is this Command:
Access this command from Transform>Orient 2
Points
Right Click Command: Orient without Copy
### Command Line Options:
Steps:
Copy=Yes/No: This specifies whether or not
the objects are copied. A plus sign appears at the
cursor when Copy mode is on.
Scale=NO: The second Target Point defines the
direction of the transformation, but objects remain
the same size.
Scale=1D: Scale the object only along the axis
between the Target Points.
• Select Orient 2 Points from the Transform menu.
• Select the Object(s) to Orient.
• Press Enter.
• Pick two Reference Points on the object. Markers
are drawn at the Reference Points.
• Pick two Target Points. This is at the location where
the object is copied.
• Continue picking additional Target Points to place
multiple copies of the Object(s) or press ESC to
end the command.
Scale=3D: The second Target Point defines the
direction of the transformation and the Scale
Factor for the objects. This is the default.
The first and second Reference Points will match
the Target Point locations. The objects are moved,
scaled and rotated so the Reference Points
transform to the Target Points.
## Orient 3 Points
‘Orient3Pt’
Moves or copies and rotates objects using three
Reference and three Target Points. This command
works the same as Orient 2 Points, but locks the
scale of the object.
Example: Draw two profiles, one
higher than the other. With
Orient 2 Points, choose Scale=
1D (keeping the height) to place
the profiles between the two
rails. The profile curve ends are
Reference points. The targets
points on the rails are found with
Near and Perpendicular O-Snaps
turned ON.
Where is this Command:
Access this command from Transform>Orient 3
Points
Right Click Command: Orient3Pt without Copy
Steps:
• Select Orient 3 Points from the Transform menu.
• Select the Object(s) to Orient.
• Press Enter.
• Pick the first Reference Point.
• Pick the second Reference Point. (This defines a
base direction from the first Reference Point.)
• Pick the third Reference Point.

<!-- page 14 -->

• Pick the first Target Point. (This corresponds to
the first Reference Point.)
• Pick the second Target Point. (This corresponds
to the second Reference Point.)
• Pick the third Target Point. (This corresponds to
the third Reference Point.)
Note: Scaling is automatically determined by the
location of the three Target Points relative to the
three Reference Points.
### Command Line Options:
Example 2: Select Start and End Rotation Axis points
as the Middle of the Profile bottom and Middle of the
Profile top.
Copy=Yes/No: This specifies whether or not
the objects are copied. A plus sign appears at the
cursor when Copy mode is on.
## Rotate3D
‘Rotate3D’
Rotates Objects around a specified axis in 3-D
space.
Example 2: Rotate 3D with Copy ON - complete.
Where is this Command:
Access this command from Transform>Rotate
3D
### Command Line Options:
Right Click Command: Rotate 3D with Copy
Copy=Yes/No: This specifies whether or not
the objects are copied. A plus sign appears at the
cursor when Copy mode is on.
Steps:
## Bend
‘Bend’
Deforms objects by bending along a spine arc.
• Select Rotate 3D from the Transform menu.
• Select Object(s) to rotate.
• Press Enter.
• Pick the Start of the Rotation Axis.
• Pick the End of the Rotation Axis. (The axis points
act as a hinge on which the object will rotate).
• Type the Angle, or pick two Reference Points.
Where is this Command:
Access this command from Transform>Bend
Right Click Command: Bend Symmetric
Steps:
Example 1: Select Start and End Rotation Axis points
as the Middle and End of the Profile bottom.
• Select Bend from the Transform menu.
• Select Object(s) to Bend.
• Press Enter.
• Pick the Start of a Spine Line that represents the
original orientation of the object.
• Pick the End of the Spine Line.
• Pick a point to Bend through (move your mouse).
This point defines the Bend direction and
magnitude or amount.
