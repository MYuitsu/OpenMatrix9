---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_label: Matrix 8 Instruction Manual Book 1
page_range: 44-44
section_title: Matrix History Record
section_id: m8book1-025-matrix-history-record
shared_start_page: True
shared_end_page: True
---

# Matrix History Record

Source: matrix8_book_1.pdf, pages 44-44

<!-- section m8book1-025-matrix-history-record | Matrix History Record | m8book1 pages 44-44 -->

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
