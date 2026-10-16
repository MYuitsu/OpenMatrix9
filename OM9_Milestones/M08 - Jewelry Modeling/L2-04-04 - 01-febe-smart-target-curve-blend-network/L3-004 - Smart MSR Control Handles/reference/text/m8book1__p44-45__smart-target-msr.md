# Smart Target MSR

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
- Page range: 44-45
- Section title: Smart Target MSR
- Section id: m8book1-027-smart-target-msr

---

<!-- section m8book1-027-smart-target-msr | Smart Target MSR | m8book1 pages 44-45 -->

<!-- page 44 -->

Steps:
• .Click on the Matrix History Update button to
toggle History Updating “On” or” Off.”
Record=Yes/No: When set to ‘Yes,” Rhino
will, by default, record History for all commands.
When set to “No,” Rhino will, by default, not
record History. These defaults can be explicitly
overridden when running a command if necessary.
## Matrix History Record
Matrix Update History and Matrix History Record
must both be on in order to see the changes
History is making while you work. There may be
some instances when you want to record History,
but prefer not to have it update until you have
finished making the changes. For instance, if you
are working on a complex design and the changes
may take a while to redraw.
Update=Yes/No: When Update is set to ‘Yes,”
the output objects will update when the input
changes. So, if you modify a profile that is part
of a Swept surface the surface will update. Any
children (copies,mirrors etc.) of an object will also
update when the parent object (the original) is
adjusted. When the option is set to no children,
the output objects will not update when parent or
input objects are moved.
Where is this Command:
Access this command from Info and
Settings>Matrix History Record
Lock=Yes/No: “Yes” locks child objects created
with History to discourage direct editing, which
would break the History link to the parent object.
“No” will not lock the children objects.
Steps:
• Click on the Matrix History Record button to toggle
History Recording “On” or” Off.”
Note: History locked objects are selectable to
use as input and edit properties, but not change
geometry. If full locking is needed, children can
be selected and locked with Lock command.
## Matrix Clear Object History
Clears Matrix History information from a selected
object.
Broken History Warning: Displays a warning
dialog when an action is taken that breaks History.
You can restore history by clicking on the “Undo”
button.
Where is this Command:
Access this command from Info and
Settings>Matrix Clear Object History
## Matrix History Update
Steps:
• Select the object(s) you wish to purge Matrix
History information from.
• Click on the Matrix Clear Object History.
## Smart Target MSR
Matrix Update History and Matrix History Record
must both be “On” in order to see the changes
History is making while you work. There may be
some instance when you want to record History
but prefer not to have it update until you have
fished making the changes. For instance if you are
working on a complex design and the changes
may take a while to redraw. Matrix will update the
design when you turn Matrix History Update back
“On.”
Where is this Command:
Access this command from Info and
Settings>Matrix History Update
‘gvSmartMSR’
This tool acts as a selection filter for Smart
Targets making them easier to select and
manipulate without breaking History. Smart MSR
also exposes additional functionality making
it possible to modify part of a Smart Target as
opposed to the whole target.

<!-- page 45 -->

Steps:
• Click on the Gumball toggle in Info and Settings.
Where is this Command:
Access this command from >Info and
Settings>Smart MSR
Z
Or, Tools>Smart MSR
Y
Smart MSR Selection Filter Steps:
X
• Turn on Smart MSR by clicking on Smart Targets
MSR in Info and Settings menu.
• Select Smart Targets using any selection method.
All other objects will be ignored.
### The Gumball
Note: It is easy to forget that you have left Smart
MSR on. When Smart MSR is active only Smart
Targets can be selected. Turn off when it isn’t
needed.
### Move:
Moves the selected object along the specified
axis. Click and drag the X, Y or Z arrow.
Move Z: Only moves the selected object on the
Z axis. Click and drag the Yellow arrow.
Rotating a single blend handle in a Smart
Target:
When a Smart Target is rotated using the Gumball,
or other methods, all of the Blend Handles rotate
together. With Smart MSR the handles can rotate
independent of one another.
Move Y: Only moves the selected object on the
Y axis. Click and drag the Blue arrow.
Note: A Smart Target must already have more than
one set of Blend Handles in order to rotate them
independently.
Steps:
Move X: Only moves the selected object on the
X axis. Click and drag the Red arrow.
4=Axis Plane Indicator: Move the selected
object on two axises. The two axises are indicated
by their color.
• Select a Smart Target.
• With the Smart Target selected, click on the Blend
Handle to rotate.
• Use Smart MSR controls to manipulate the Blend
Handle.
5=Free Move Origin: Move the selected object
freely in any direction.
## Gumball
Z
4
Y
5
X
‘Gumball’
The Gumball is a set of Display Widgets and
Viewport Control Handles that when ‘On” is
displayed on any selected object. The
Gumball makes it so you can Move, Scale and
Rotate any selected object.
Where is this Command:
Access this command from Info and
Settings > Gumball
