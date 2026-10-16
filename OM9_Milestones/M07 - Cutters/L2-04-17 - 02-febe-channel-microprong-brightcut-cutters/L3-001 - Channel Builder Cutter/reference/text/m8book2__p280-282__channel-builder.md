---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 280-282
section_title: Channel Builder
section_id: m8book2-234-channel-builder
---

# Channel Builder

Source: Matrix 8 Instruction Manual Book 2 (`matrix_8_book_2.pdf`), pages 280-282

<!-- section m8book2-234-channel-builder | Channel Builder | m8book2 pages 280-282 -->

<!-- page 280 -->

turn Draw All on to generate the other cutters.
## Channel Builder
by selecting the Height icon at the left in the
builder and running the slider below-or typing a
percentage in the box. You can also use the
Yellow horizontal Viewport Control Handle on the
base of the cutter.
Z Offset: Z Offset adjusts the height of the
‘gvChannelBuilder’
Channel Builder generates a channel cutter for a
line of gems.
Where is this Command:
Access this command from Cutters>Channel
Builder
or from the F6 menu when a line of gems is
selected
channel cutter-measured in terms of
mm. The default is 0 mm. Positive values
will move the cutter up while a negative
value will move the cutter down. Adjust
by selecting the Z Offset icon at the left in the
builder and running the slider below-or typing a
percentage in the box. You can also use the
Yellow Cube Viewport Control Handle on the side
of the cutter.
Steps:
Extension: Extension Adjusts the distance past
• Select the Channel Builder from the Cutters
Menu or from the F6 menu with a line of stones
selected.
• Put the selected stones into the Gem
Cutter input box (not necessary if you
used F6).
• Matrix will generate a preview of the
Channel Cutter.
the ends of the gems that the channel
extends. Adjust in the builder by
selecting the Extension icon at the left
and running the slider below-or typing
a length in mm in the box. You can also use the
Yellow Vertical Viewport Control Handle on the
side of the cutter. The Extension Value does not
include the cap length.
Cap Length: Cap Length along with Cap Scale
When you are satisfied with your adjustments
press enter to create the channel cutter. Then
preform a Boolean Difference. This step is not
necessary if you used Interactive Boolean.
### Editing Dimensions:
Dimension for gem cutters are all based on
percentage-because the cutter is built relative to
the size and shape of the stone.
Width: Width adjusts the width of the channel
affect how the channel tapers into the
rest of the band. Cap length changes
how far from the extension the cap
extends into the band. Adjust in the
builder by selecting the Cap Length icon at the
left and running the slider below-or typing a
length in mm in the box. You can also use the
Orange Vertical Viewport Control Handle on the
end of the cutter.
Cap Scale: Cap Scale changes how much taper
cutter-measured in terms of a
percentage. The default is 90% so the
channel extends 5% on each side of the
stone. Adjust by selecting the Width
icon at the left in the builder and running the slider
below-or typing a percentage in the box. You can
also use the Yellow horizontal Viewport Control
Handle on the side of the cutter.
the cap has by adjusting the scale at
the end of the cap. Adjust in the builder
by selecting the Cap Scale icon at the
left and running the slider below-or
typing a percentage in the box. You can also use
the Orange Vertical Viewport Control Handle on
the base of the cutter.
Height: Height adjusts the height of the channel
cutter-measured in terms of a
percentage. The default is 100%. Adjust

<!-- page 281 -->

### Options:
the sliders or the control handles. Press enter
when complete.
Styles: Styles allow you quick access to standard
designs within the builder. You can also add your
own styles so that you can access them quickly
through the builder. Learn more about using and
creating styles in the Styles section of the manual.
Reset: Will return the builder to its default
settings.
Mirror: Mirror
allows both ends
of the channel
to be controlled
together. To
edit each side
separately turn off
the option. Click
on ‘Mirror=On’ in
the Command
Line to toggle to
off. When mirror is off a second set of sliders and
control handles will appear.
### Command Line:
These options are only available as options in the
Command Line.
Cap A/Cap B: Use this option to set the end
cap for your channel. Cap B will only appear when
mirror is shut off.
Extension Type: ExtTypeA/ExtTypeB - toggle
between these three options: line, arc or smooth.
### Cap Types:
Round: The end of the channel
terminates with a half round cap.
Square: The end of the channel
terminates with a square cap.
Sharp: The end of the channel
terminates at a point creating a triangle
cap.
Line: Line is the default mode for extension - it
extends in a straight line from the end of the
gems.
Arc: This option will follow an arc as the channel
extends beyond the gems.
Smooth: The channel continues to follow the
curvature of the gems as it extends beyond the
stones.
Flat: No cap - the channel ends at the
end of the 	 channel extension.
Corner Angle: Type a value between 0-1
(0=Smooth, 1=Sharp) change the control how
smooth or sharp the corner angle is. For example,
0.3 would be smooth but less smooth than 0.
Flip AB: If mirror is off Flip AB changes the A end
of the channel to B and vice versa.
Surface: Use the surface button to tell Matrix
which surface to subtract. Click on the ‘Surface’
button in the builder then select your surface.
Tweak Corners: When you have a sharp corner
in your channel with “Tweak Corners = On” the
channel will get slightly larger at the corner to
accommodate for pinching.
Closed: Toggle closed to ON to create a channel
around a closed group of gems. This should only
be necessary if the group of gems isn’t actually
closed-but is close. Otherwise, the builder will
detect that it is closed and turn the option on for
you.
Boolean: Select the Boolean option in the
builder or in the Command Line to perform an
interactive boolean. First turn on boolean in the
builder. Then, turn on “surface” and select the
surface to subtract from. This will generate a
preview of your object with the cutter already
removed. Make adjustments to the boolean using

<!-- page 282 -->

to accommodate the other cutters nearby-
preventing overlapping cutters.
Middle Profile: This Command adds additional
profiles to the channel that can sometimes give
you a more uniform surface.
Middle: Creates cutters between
gems as opposed to centered on
them. Middle has a different set of
controls than Standard.
Surface to Attach: This allows you to select
a surface that you wish for the channel to push
up to or pull back from. It places the channel’s
imaginary rail (a curve created through the center
points of the gems) on the selected surface.
Basic: Has the same controls and layout as
Standard but doesn’t taper the end of the cutters
to prevent overlap.
## Micro Prong Cutter
Standard/Basic Mode
### Editing Dimensions:
‘gvMicroProngCutter’
Create a micro prong setting for a line of gems.
Drop: Drop adjusts the Z offset of the micro
Where is this Command:
Access this command from Cutters>Micro
Prong Cutter
or from the F6 menu when a line of gems is
selected
prong cutter-measured in terms of mm
from the stones center. Adjust in the
builder by selecting the Drop icon at
the left, running the slider below-or
typing an amount in mm in the box. You can also
use the Yellow Cube Viewport Control Handle on
the side of the cutter.
Steps:
Middle Width: Middle Width adjusts how
• Select the Micro Prong Cutter from the Cutters
Menu or from the F6 menu with a line of stones
selected.
• Put the selected stones into the Micro Prong
Cutter input box (not necessary if you used F6).
• Matrix will generate a preview
of the Micro Prong Cutter.
wide the cutters center is measured in
terms of mm. Use to create a taper
cutter and a scalloped effect. Adjust in
the builder by selecting the Middle
Width icon at the left in the builder and running
the slider below-or typing the dimension in mm in
the box. You can also use the Orange horizontal
Viewport Control Handle on the middle of the
cutter.
Width Adjustment: Width Adjustment adjusts
When you are satisfied with
your adjustments press enter
to create the Micro Prong
Cutter. Then, perform a
Boolean Difference. This step
is not necessary if you used
Interactive Boolean.
### Micro Prong Cutter Modes:
the width of the prongs left behind after
the boolean-not the width of the cutter.
So in this instance a small value will
generate a larger cutter. Adjust in the
builder by selecting the  Width Adjustment icon at
the left in the builder and running the slider
below-or typing the dimension in mm in the box.
You can also use the Yellow horizontal Viewport
Control Handle on the middle of the cutter.
The Micro Prong Cutter has 3 modes: Standard,
Middle and Basic. Toggle between the modes by
clicking on the mode box at the top of the builder.
We will cover each mode individually.
Height: Height adjusts the height of the micro
prong cutter-measured in terms of mm.
Adjust in the builder by selecting the
Standard: The Micro Prong Cutter’s default
mode creates cutters that are centered on
the Gem and tapers the ends of the cutters
