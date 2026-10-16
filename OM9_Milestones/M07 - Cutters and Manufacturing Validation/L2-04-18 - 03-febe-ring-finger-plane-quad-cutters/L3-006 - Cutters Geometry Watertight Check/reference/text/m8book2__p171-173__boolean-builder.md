---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 171-173
section_title: Boolean Builder
section_id: m8book2-180-boolean-builder
---

# Boolean Builder

Source: Matrix 8 Instruction Manual Book 2 (`matrix_8_book_2.pdf`), pages 171-173

<!-- section m8book2-180-boolean-builder | Boolean Builder | m8book2 pages 171-173 -->

<!-- page 171 -->

## Boolean Builder
## Center Line Builder
This calculates the center line when given a
surface with two curves on it.
Where is this Command:
Access this command from Tools>Center Line
Builder
Steps:
This builder allows you to easily input an “Object”
to be cut and “Cutter” to do the cutting; or two
objects to be Joined or Unjoined. Or, it allows you
to create a new object from where two objects
Intersect. The Boolean Builder checks that all
objects input into the builder are valid and will
perform as expected. Additionally, the builder
has tools to help you overcome typical obstacles
when performing booleans.
Where is this Command:
Access this command from Tools>Boolean
Builder
Or Cutters>Boolean Builder
• Select the Center Line Builder from the
Tools menu.
• Select the surface and input it into the
Surface box.
• Select the curve on the surface and
input into the Curve 1 box.
• Select the second curve
on the surface and
input in the Curve 2
box.
• Click Start.
Or, choose from the F6 menu when a cutter
is selected. This will perform a boolean
difference. You will be prompted by the
Command Line to select the surface to cut
from. This won’t give you access to all the
options in the builder but is a good short cut
for an often repeated action.
### Boolean Types:
Before
The Boolean Builder allows you to perform 3
common types of booleans: Boolean Difference,
Boolean Union, and Boolean Intersection. Objects
used in the builder must be watertight solids.
We will discuss each of these boolean types
individually.
Boolean Difference:
After
This is the most common type of
boolean for the kind of modeling we do.
A Boolean Difference uses one solid
(Cutter) to remove material from
another solid (Object). We call each of these the
cutter and the object. Cutters are used to create a
piercing or void in another object and are then
### Options:
Reset: Clears the surface and curves from the
builder.
discarded-for
instance when
cutting
channels.

<!-- page 172 -->

Steps:
• Click on ‘Do Boolean’
Both objects will be joined together into a new
single object.
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Boolean Difference icon (the menu will
change to reflect scissors).
• Select the object you wish to remove material
from, and input into the Object box. Notice the
green validation check appears if your object is a
watertight solid.
• Select the object you will use as the cutter and
input into the Cutter box.
• Click on ‘Do Boolean.’
Boolean Intersection:
Boolean Intersection creates an new
object from where two solids intersect.
Steps:
The object will
remain and if
‘Keep Cutter’ is
toggled to the
ON position. If
this option is
OFF the cutter will disappear. The object will have
material removed where the two objects
overlapped creating a new shape.
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Boolean Intersection icon (the menu will
change to reflect a traffic intersection).
• Select the objects you wish to find the intersection
for and input them into Object 1 and Object 2
respectively,
• Click on ‘Do Boolean’ in the builder.
Note: Booleans break both Rhino and Matrix history.
Job Bag your project before performing booleans so
you have the option to revert to a flexible history
enabled state.
Boolean Union: This is sometimes called
This will leave you with an new object where the
previous solids intersected.
boolean join. This command will take
two objects that intersect and join
them together into one new object.
### Options:
Steps:
• Select the Boolean Builder from the Tools Menu or
Cutters Menu.
• Select the Boolean Union icon (the menu will
change to reflect a glue bottle).
• Select the first object and input into the Object 1
box.
• Select the second object and input into the Object
2 box.
Keep Cutter: When performing
a Boolean Difference, the ‘Keep
Cutter’ option will not delete the
object placed into the Cutter input
after performing the boolean.
This option is helpful when you
are using a head or other part of
a finished design to perform the boolean. When
cutting channels or azures the cutters in this
instance are not part of the design, and therefore
don’t need to be saved. Having keep cutter shut
off saves you the time of having to delete them.
Note: Each input box can be given multiple objects.
Simply select the objects, by using shift+click, region
or cross over select or any other selection method
and put into the desired input box.

<!-- page 173 -->

grid. Here you can type in a distance to move an
object to try to remedy an issue with coplanar
surfaces.
One by One: When you input multiple objects
into the cutter box, if ‘One by One’ is selected,
Matrix will automatically perform each boolean
separately; this can often be faster than doing
them all once. This has an additional benefit that
if any of the booleans fail, only the cutters that
couldn’t boolean will remain, making it easier to
address where you maybe having an issue.
Note: Be mindful of the objects that you move when
trying to fix a coplanar surface. When working with
Matrix we often reference the ‘center of the world’
or F4 when performing certain commands. If you
move an object it will no longer be centered on F4.
Use AutoScale and Auto Move on either Object 1 or
Object 2, not both.
Force Boolean: This option is only available
when ‘One by One’ is on. If any boolean fails Matrix
will attempt to force it by moving and scaling the
cutter slightly.
Do Boolean: Performs the selected boolean
type with the objects in the input box.
Undo Boolean: This will ‘Undo’ the last boolean
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
