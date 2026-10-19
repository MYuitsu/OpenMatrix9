---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 140-141
section_title: Custom Rail ￼
section_id: m8book2-143-custom-rail
shared_start_page: True
shared_end_page: True
---

# Custom Rail ￼

Source: matrix_8_book_2.pdf, pages 140-141

<!-- section m8book2-143-custom-rail | Custom Rail ￼ | m8book2 pages 140-141 -->

<!-- page 140 -->

### Options:
Reset: This returns all of the Outside Rail Profile
settings to their defaults.
Rail Profile: Select the rail profile shape from the
Gemvision profile library, edit or create your own.
Open the profile library by clicking on the Image
Note: Make changes to the dimensions or shape of
an existing Outside Ring Rail by selecting it in the
viewport, clicking F6 and choosing Edit Outside.
### Editing the Outside Ring Rail Profile
See Profile Placer Editor for detailed instructions
how to edit profiles. Follow these
in the Rail Profile box in the menu
on the left or selecting the Profile
Library icon on the right side of the
rail in the viewport. Like with most
profiles libraries you can modify or add to this
profile library.
### Editing Dimensions:
same instructions to edit the
Outside Ring Rail. The returned
rail must be one closed curve.
## Custom Rail
Custom Rail creates a custom rail curve from a
Ring Rail and a planar curve drawn in the Looking
Down viewport. This is helpful for making bypass
rings and other advanced ring shapes.
Top Thickness: Controls the distance between
the Ring Rail and Outside Ring Rail at
12 o’clock as seen in the Through
Finger viewport. Adjust in the builder
by selecting the Top Thickness icon at
the left, running the slider below-or typing a
length in mm the box. You can also use the Yellow
Vertical Viewport Control Handle on the profile.
Where is this Command:
Access this command from Tools>Custom Rail
Steps:
• Select Custom Rail from the Tools menu.
• Click on the ‘+Ring Rail’ button to
Side Thickness: Controls the distance between
the Ring Rail and Outside Ring Rail at 3
& 9 o’clock as seen in the Through
Finger Viewport. Adjust in the builder by
selecting the Side Thickness icon at the
left, running the slider below-or typing a length in
mm the box. You can also use the Yellow
Horizontal Viewport Control Handle on the profile.
open Ring Rail and select the desired size.
• Draw a planar curve in the Looking Down viewport.
Interp Curve and Curve command
buttons are in the builder to make
it convenient.
• Select the curve you drew and select ‘Make Rail’ .
Bottom Thickness: Controls the distance
between the Ring Rail and Outside
Ring Rail at 6 o’clock as seen in the
Through Finger Viewport. Adjust in the
builder by selecting the Bottom
Thickness icon at the left in the builder and
running the slider below-or typing a length in mm
the box.  You can also use the Yellow Vertical
Viewport Control Handle on the top of the profile.
Lock: With the lock option ON this limits the
amount of influence each of the
Planar Curve drawn              Make Rail complete
Note: This command results in two unjoined curves.
control handles has on the rest of the Outside Rail
Profile. For example, when the Lock option is ON,
Top Thickness will have zero effect on the lower
half of the rail curve. When OFF, it will have some
effect. This will skew the rail as you adjust the
curve.

<!-- page 141 -->

Steps:
### Options:
Ring Size Reference: This draws
two vertical guidelines in
the Looking Down
viewport that mark the finger width.
Ring Width Reference: This
draws two horizontal guidelines
in the looking down viewport
that marks the width
in mm that you set in the builder. Enter
an overall ring width you wish to set.
Add (+) Ring Rail: This opens Ring Rail so a rail
can be added to the viewport. You must start with
a Ring Rail first.
Delete Curves: When this option
is ON, the input curves are deleted, both the top
of the ring rail and the curve that you drew.
• Select Four Rail Profile from the Tools menu.
• Input four curves into the curve input
box which appears in the menu below.
• Follow the Command Line prompt for
Start and End of line. This line defines
the location of the profile by passing
through all four curves. Select the start of the line
by clicking in the Through Finger Viewport (for
a ring this is typically F4 - somewhere below the
bottom rails)
• Select the end of the line by clicking in the
Through Finger Viewport. This is typically selected
a little ways above the first point, either directly,
to the left or to the right - above the top rails. This
also represents the angle the profile will orient to
the rails.
• Select the desired profile from the profile library.
• Add additional profiles-position either by defining
the plane where the profile intersects the four rails
or by using the Viewport Control Handles.
• Press enter when done. (This adds a profile.)
Note: If necessary, input the rail curves in the Curve
in-box to start the command again to add more
profiles.
Make Rail: After a curve has been drawn in the
viewport, select the curve and choose the green
arrow for “Make Rail.” The original ring rail along
with the drawn curve will be deleted (if Delete
Curve is ON) and a new custom rail will appear.
Undo: This will undo the ‘Make Rail’ command
Interp Curve / Curve: Use these drawing
commands to draw a planar curve in the Looking
Down Viewport that represents the rail shape
wanted for your custom rail.
## Four Rail Profile
### Editing Dimensions:
Add profiles to four rail curves to perform a Four
Rail Sweep. The profiles are attached to the rails
based on a bounding box. This bounding box can
be seen when the profiles are active and editable.
The bounding box connects to the four rails. But,
the profiles will not necessarily touch all four rails,
depending on their shape.
Move: Adjust the position of the
profile by using the Yellow Cube
Move Viewport Control Handle. As
you move the profile along the rails
it does not maintain it rotation.
Where is this Command:
Access this command from Tools>Four Rail
Profile
Rotate Z: Rotate the profile along the Z axis
using the white Rotate Z Viewport Control Handle.
This works similar to Tilt in Profile Placer.
