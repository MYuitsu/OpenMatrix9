---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 290-292
section_title: Boolean Builder
section_id: m8book2-242-boolean-builder
---

# Boolean Builder

Source: Matrix 8 Instruction Manual Book 2 (`matrix_8_book_2.pdf`), pages 290-292

<!-- section m8book2-242-boolean-builder | Boolean Builder | m8book2 pages 290-292 -->

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

<!-- page 291 -->

Boolean Intersection: Boolean Intersection
creates an new object from where two
solids intersect.
Steps:
The object will
remain and if
the ‘Keep
Cutter’ option is
off the cutter
will disappear.
The object will have any material remove where
the two objects overlapped creating a new shape.
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Intersection icon. (The menu will change
to reflect a traffic intersection icon.)
• Select the objects you wish to find the intersection
for and input them into Object 1 and Object 2
respectively,
• Click on ‘Do Boolean’ in the builder.
Note: Booleans break both Rhino and Matrix history.
Job Bag your project before performing booleans so
you have the option to revert to a flexible history
enabled state.
Boolean Union: This is sometimes called
boolean join. This command will take
two objects that intersect and join
them together into one new object.
Steps:
This will leave you with an new object where the
previous solids intersected.
### Options:
Keep Cutter: When performing a Boolean
Difference Keep Cutter will not delete the object
placed into the Cutter input after performing the
boolean. This option is helpful when you are using
a head or other part of a finished
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Boolean Union icon. (The menu will
change to reflect a glue bottle icon.)
• Select the first object and input into the Object 1
box.
• Select the second object and input into the Object
2 box (cutter).
• Click on ‘Do Boolean’
Note: Each input box can be given multiple objects.
Simply select the objects, by using shift+click, region
or cross over select or any other selection method
and input into the desired input box.
design to perform the boolean.
When cutting channels or azures
the cutters in this instance are
not part of the design, and
therefore don’t need to be
saved. Having keep cutter shut
off saves you the time of having to delete them.
Both objects will be joined together into a new
single object.
One by One: When you input multiple objects
into the cutter box, if One by One is selected,
Matrix will automatically perform each boolean
separately; this can often be faster than doing
them all once. This has an additional benefit that
if any of the booleans fail, only the cutters that
couldn’t boolean will remain, making it easier to
address where you maybe having an issue.

<!-- page 292 -->

Force Boolean: This option is only available
when One by One is on. If any boolean fails Matrix
will attempt to force it by moving and scaling the
cutter slightly.
Do Boolean: Performs the selected boolean
type with the objects in the input box.
Undo Boolean: This will undo the last boolean
performed.
Reset: Resets the Boolean Builder.
### Trouble Shooting Booleans:
Sometimes when running a boolean command
a boolean will fail. The builder has some tools
that make it easier to overcome one of the most
common obstacles when working with booleans,
coplanar surfaces.
Object 1 signifies the first object in the object box.
Object 2 signifies the “cutter” object in the second
input box.
Auto Scale Object 1 or 2: When either or both
of these options are selected a scale dialog will
appear in the builder. Type in a scale factor into
the box. By default, the scale factor is 1.001 or
scaling up by 1000th percent.
Auto Move Object 1 or 2: When either of these
options are selected a series dialog boxes appear
in the builder related to the coordinates on the
grid. Here you can type in a distance to move an
object to try to remedy an issue with coplanar
surfaces.
Note: Be mindful of the objects that you move when
trying to fix a coplanar surface. When working with
Matrix we often reference the ‘center of the world’ or
F4 when performing certain commands. If you move
an object it will no longer be centered on F4.
