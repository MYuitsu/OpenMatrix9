---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 236-238
section_title: Gem Flow
section_id: m8book2-212-gem-flow
---

# Gem Flow

Source: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf), pages 236-238.

<!-- section m8book2-212-gem-flow | Gem Flow | m8book2 pages 236-238 -->

<!-- page 236 -->

“Styles” option in the Builder Menu or Command
Line.
Now, you can access the saved Style using Load
Style (in a particular builder or from the Gems
Menu) and navigating to the location on your
computer where you saved it. (You can only click
the number 1 - 8 if you saved it “1.mss” to “8.mss”.)
You can Load any named or numbered file when
using Load Style in either the builders Style sheet
or in the Gems menu. Learn more about Styles in
the Style Chapter of the manual.
Note: The file extension is .mss and this must be
present with the file, so its full name is “1.mss”, to
work correctly.)
Steps:
## Load Styles
Load Styles for most setting and cutter
tools from your save style sheets (.mss file) to
use in your current project.
Where is this command:
Access this command from Gems>Load Styles
### About Loading Styles
• Select a Style from the menu of Gemvision-
created styles for that builder by clicking it. For
each style you click, you will see it previewed
on the gem (or curve) that you are using in the
builder.
• Apply the style and close the style sheet by
clicking the green “Go” arrow, placing the style on
the gem. This also sets the Builder Menu values
to those established for the selected style, but
leaves you the handles and builder menu controls
so you can continue editing the style.
### To Load a style from the Gems menu.
Steps:
Much like Match Attributes, any gem with a
setting or cutter applied to it retains a “memory” of
the builder values used to create the related items.
The contents of this “memory” - or, the builder
values used to create the item - is known as a
Style in the program. Match Attributes transfers
these styles from one gem to another, while
Save & Load Styles creates a file (.mss) on your
computer containing this information, which you
can access and apply to another gem (“Load”) in
the future.
Note: You should only use Match Attributes and Save
/ Load Styles to match styles to gems of the same
SHAPE. Gems of a different shape may not be able
to accept the styles; just the way you would never
order a head style designed for a marquise for a
round or square stone. Some shapes will be able to
translate styles well; others will not.
• Go to the Load Styles command in the Gems
menu.
• Select the item on which you would like to place a
Saved Style.
•  Navigate to the location where you saved the
style. (My Documents > Matrix > Styles > Builder
you created it in > Mode). Select and Load it.
Remember that handles will NOT appear. If you
need to edit the style further, you should instead
complete this process within the builder’s Style
Sheet. Open this by clicking on the “Styles” option
in the builder. Learn more about Styles in the
Style Chapter of the manual.
### Load a Style
## Gem Flow
The easiest introduction to styles is to open a
builder in which they are available and select the
Gem Flow incorporates both Flow Along
Curve and Flow Along Surface for gems.

<!-- page 237 -->

Where is this command:
Access this command from Gems>Gem Flow
editing the original objects (Move, Scale, Rotate,
and other Transform tools may be used to edit the
original objects- but not Trim functions such as
Booleans, Split, or Join).
### Using Flow
### Using Flow: Base & Target Objects.
When using one of these commands, a “Target”
Surface or Curve (the surface or curve onto which
you wish to orient the objects) must be present. A
“Base” curve or surface may also be drawn: either
before or during one of these commands (Line
option in Flow Along Curve; Plane option in Flow
Along Surface). For best results, though, here’s
how we recommend you create the Base Curve
for Flow Along Curve:
Preparation for Flow Along Curve Steps:
Create Base Object: When flowing objects
along a curve, measure curve length and create
a straight line of equal length on which to place
objects. For Flow Along Surface, create a planar
surface for objects by sweeping UV Curves.
Select a Base Object: Select a “Base Curve” in
Flow Along Curve; Select a “Base Surface” in Flow
Along Surface. Or, using the Line option (Curve)
or Plane option (Surface), create the base object
with the relationship you want the flowed objects
to have to the target curve or surface.
Select the Target: Object Select the target
curve or surface to flow the objects onto. If History
was enabled before you started the command,
you can now edit (but not Trim)  original objects
and the results will appear in the flowed objects.
• Measure the curve you’re flowing to (target curve)
using the Measure Menu command Length.
• Start up the Line command and type this value
into the Command Line.
• Hold down Shift to draw a straight line, this length,
underneath the objects you want to flow.
Similarly, to create a base surface, we recommend
you select the Target Surface and run Create UV
Curves..
Preparation for Flow Along Surface Steps:
Gems Flowed from a surface along with their prongs.
• Explode the UV Curves box.
• Run the Sweep 2 command, using the two long
edges as rails and the two short edges as profiles.
### About Each Tool
Use this surface as the Base Surface in order to
ensure that your objects will not skew too badly
when they are flowed up onto the target surface
(because the base surface was created directly
from the target surface).
### To Flow Gems
If you wish to include gems in the objects to be
flowed, use the command Gem Flow from the
Gems menu and follow the same directions.
The Flow Along Curve command orients objects
along a curve. Objects usually begin in
a straight line, since this is an easy configuration
to create. Flow Along Surface similarly allows you
to copy and orient objects up onto a surface
when, previously, they were all located along the
same flat plane in space. If Rhino History enabled
prior to running either of these 2 commands
(Record = Yes and Update = Yes), it is possible
for you to run either command with Copy turned
‘ON’ and then, subsequently, edit the original
object(s) to see the changes carry through to the
new objects arranged on the surface or curve. In
this way, editing can take place: position objects
on a difficult surface; then edit them there by

<!-- page 238 -->

### Command Line options:
for orienting gems on a surface. Gem Splop is
best used for more free from designs. For more
precision use Gem on Surface.
Where is this Command:
Access this command from Gems>Gem Splop
Copy = Y/N:  Keep “Copy” turned on in order to
add copies of the objects you are flowing, and not
delete the originals. This is a good idea when you
plan to use History, since you can then edit the
original items (but don’t Trim or Join them) and
see the same changes in the flowed items.
or from the F6 menu when a curve is selected.
Steps:
Rigid = Y / N: This option keeps the command
from deforming the item(s) in order to fit it to
the target object. With “Rigid = Y”, only the
configuration of the objects being flowed will be
conformed to the curvature of the target object.
With “Rigid = N”, the objects being flowed will be
curved to fit the target object.
### Along Curve Options:
• Select a Gem Splop from the gem menu.
• Select a gem to Splop on a surface.
• When prompted create a reference sphere
matching the size of the stone.
• Pick the Splop point on the surface. (Make sure
that Rigid and Copy options are both set to yes in
the Command Line.
• Press Enter when done.
The sploped stones will be replaced on the
surface with block instances (gems).
Line: The Line command will be started. This
allows you to draw your base line (instead of
selecting it). Instead, we recommend measuring
the target curve with the Length command and
creating a base line ahead of time.
Local: Select this command to draw two circles
defining a “tube” within which the objects are
deformed and outside of which they will not be
deformed. This means part of an object may be
stretched to touch the curve and part can be left
alone.
Stretch: With “Stretch” set to Yes, the objects
will be stretched to accommodate the length of
the curve when the Base line is shorter than the
Target curve.
Splopping gems on a ring surface
### Along Surface Option:
## Pavé Sphere
Plane: Starts up the Plane command during the
Flow Along Surface command, so you can draw
the base surface instead of selecting one. Instead,
we recommend you follow the instructions we
have set forth earlier in this section to create your
own base surface.
‘gvPaveSphere’
Create a pavé’ sphere with no surface on-screen,
or using a spherical surface. Adjust gem count,
gem size, and symmetry. Preview results before
you commit.
## Gem Splop
You can access this command from Gems>Pavé
Sphere
‘gvGemSplop’
Gem Splop is another gem tool that you can use
