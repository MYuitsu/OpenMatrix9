---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 288-290
section_title: Cut to Finger Rail
section_id: m8book2-240-cut-to-finger-rail
---

# Cut to Finger Rail

Source: Matrix 8 Instruction Manual Book 2 (`matrix_8_book_2.pdf`), pages 288-290

<!-- section m8book2-240-cut-to-finger-rail | Cut to Finger Rail | m8book2 pages 288-290 -->

<!-- page 288 -->

Number of Cutters: Number of Cutters
Where is this Command:
Access this command from Cutters>Cut To
Finger Rail
or from the F6 menu when a bezel or head are
selected-Cut to Finger Rail.
changes the count of cutters around
the gem. Adjust in the builder by
selecting the Number of Cutters icon
at the left and running the slider below.
Or, type the count in the box.
### Options:
Note: This command will work with any object that
you can perform a boolean on - select the object
and then run the command from the Cutters Menu.
You can only access the command from the F6 menu
when a head or bezel is selected.
Position (Even/Odd): Odd will position the
metal so that is located at the top and bottom of
the stone. Even positions the metal so that is
offset from the top and bottom of
Steps:
the stone.
• Select the object you want to
cut.
• With the object selected
choose Cut to Finger Rail
from the Cutters Menu (or you may use F6 if the
object is a head or bezel)
The object is then cut to match
the finger rail.
Boolean: Select the Boolean
option in the builder or in the
Command Line to perform an
interactive boolean. First, turn on
boolean in the builder; when prompted, select
the surface to subtract from. This will generate
a preview of your object with the cutter already
removed. Make adjustments to the boolean using
the sliders or the control handles. Press enter
when complete.
### How Cut to Finger Rail works:
Behind the scenes we Extrude the Ring Rail to
make a cylinder. We then perform a Boolean
Difference on the object you selected to cut.
Surface: When Boolean preview option is set to
‘ON’ button is blue. Use the surface button to tell
Matrix which surface to subtract from. Click on the
‘Surface’ button in the builder ,then select your
surface.
## Plane & Cube Cutters
Find some helpful shapes for preforming Splits
and Booleans on objects, all oriented around F4.
Styles: Styles allow you quick access to standard
designs within the builder. You can also add your
own styles so that you can access them quickly
through the builder. Learn more about using and
creating styles in the Styles section of the manual.
Where is this Command:
Access this command from Cutters>Plane and
Cube Cutters
Reset: Reset will return the builder to its default
settings.
Steps:
## Cut to Finger Rail
• Select the Plane and Cube Cutter from the Cutters
Menu.
• Select the desired Plane or Cube for the job.
‘gvCutRingRail’
Remove excess material that intersects the ring
rail with this command.
Use to preform a Split (Cutting Plane) or Boolean
(Cutting Cube)

<!-- page 289 -->

-X/+X Half Cube Cutter:
-X creates a Cutting Cube on the left
hand side of a model when viewed in
the Through Finger viewport.
+X produces a Cutting Cube on the right
hand side of the model.
-Y/+Y Half Cube Cutter:
-Y creates a Cutting Cube on the left
hand side of a model when viewed in
the Side View viewport.
+Y produces a Cutting Cube on the right
hand side of the model.
-Z/+Z Half Cube Cutter:
-Z creates a Cutting Cube on the
bottom of a model when viewed in the
Through Finger viewport.
Z produces a Cutting Cube on the top
Cutting Planes: Use Cutting planes to spit
objects in half (if built around F4). A Split solid will
result in an open surface. If you mirror this surface
to the opposite side of your object and join, you
should again have a closed water tight solid.
of the model.
X Plane Cutter: This brings a Cutting Plane out
X Quarter Cutter: There are 4 - X Quarter
into the viewport that is centered on the
X axis. The plane runs through the
center of your ring, running up and
down on the X axis.
Y Plane Cutter: This brings a Cutting Plane out
Cutters in the through finger view.
Quad 1 creates a cutter in the upper
right hand corner. Quad 2 creates one
in the upper left hand corner. Quad 3
creates one in the lower left hand
corner and Quad 4 in the lower right hand corner.
Toggle between the cutters by clicking on the
quad toggle.
into the viewport that is centered on the
Y axis. The plane runs through the
center of your ring running up and
down on the Y Axis.
Y Quarter Cutter: There are 4 - Y Quarter
Z Plane Cutter: This brings a Cutting Plane out
into the viewport that is centered on
the Z axis. The plane runs through the
center of your ring laying flat on the Z
axis.
Cutters in the through finger view; Quad
1 creates a cutter in the upper right
hand corner, Quad 2 in the upper left
hand corner, Quad 3 in the lower left
hand corner and Quad 4 in the lower
right hand corner. Toggle between the cutters by
clicking on the quad toggle.
Z Quarter Cutter: There are 4 - Z Quarter
Cutting Cubes: Cutting Cubes are solid cubes
that you can use to perform a Boolean on your
model.
Cutters in the through finger view;
Quad 1 creates a cutter in the upper
right hand corner, Quad 2 in the upper

<!-- page 290 -->

left hand corner, Quad 3 in the lower left hand
corner and Quad 4 in the lower right hand corner.
Toggle between the cutters by clicking on the
quad toggle.
Intersect. The Boolean Builder checks that all
objects input into the builder are valid and will
perform as expected. Additionally, the builder
has tools to help you overcome typical obstacles
when performing booleans.
## Quad Flip Routines
Where is this Command:
Access this command from Cutters>Boolean
Builder
When a design overlaps F4 and the four
quadrants should be alike, this tool places the
planes and does the splits and mirrors for you.
Or Tools>Boolean Builder
Where is this Command:
Access this command from Cutters>Quad Flip
Routines
Steps:
Or, choose from the F6 menu when a cutter
is selected to perform a boolean difference.
You will be prompted by the Command Line
to select the surface to cut from. This doesn’t
provide access to all the options in the boolean
builder but it’s a good short cut for an often
repeated action.
### Boolean Types:
• Select the Quad Flip Routines from the Cutters
Menu.
• Select the object that you want to have quad
flipped.
• Refer to the Looking down viewport and in the
builder select the quadrant you want to mirror into
all quadrants.
• Click Apply.
Once the quad flip routines have completed, join
your model to create a watertight solid.
The Boolean Builder allows you to perform 3
common types of booleans: Boolean Difference,
Boolean Union, and Boolean Intersection. Objects
used in the builder must be watertight solids.
We will discuss each of these boolean types
individually.
Boolean Difference: This is the most common
type of boolean for the kind of
modeling we do. A boolean difference
uses one solid (Cutter) to remove
material from another solid (Object).
We call each of these the cutter and the object.
Cutters are used to create a piercing or void
in another object and are then discarded-for
instance when cutting channels.
### Options:
Steps:
Select: Selects the original object and the quad
flipped objects.
Undo: This reverts the Quad Flip Routines.
## Boolean Builder
This builder allows you to easily input an “Object”
to be cut and “Cutter” to do the cutting; or two
objects to be join or Unjoined. Or, it allows you
to create a new object from where two objects
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Boolean Difference Icon. (The menu will
change to reflect scissors icon.)
• Select the object you wish to remove material
from, and input into the Object box. Notice the
green validation check appears if your object is a
watertight solid.
• Select the object you will use as the cutter and
input into the Cutter box.
• Click on ‘Do Boolean.’
