---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_label: Matrix 8 Instruction Manual Book 1
page_range: 43-44
section_title: Rhino History
section_id: m8book1-023-rhino-history
shared_start_page: True
shared_end_page: True
---

# Rhino History

Source: matrix8_book_1.pdf, pages 43-44

<!-- section m8book1-023-rhino-history | Rhino History | m8book1 pages 43-44 -->

<!-- page 43 -->

Metal pricing is listed in raw materials only. Gem
pricing is per group. (There are no setting fees
associated with the Design Report. Pricing is for
raw materials only.)
Changing Selections: The report will quote any
item that is currently available at Stuller. If an item
appears as “not available” you can go back and
change the quality or check the item’s availability
at Stuller.com. If a stone quality is changed a
“Reset” icon will appear along with a round, yellow
warning icon.
Save: Save in a .xps format for future viewing.
Reset:  Reset will reset that item back to the
default upon launch of the report. Viewing an
Item on Stuller.com: After the quote is completed
a “preview” link will appear under each listed item.
Click the link to view that item’s Stuller web page.
Viewing the Design Report Summary along with
the Print Preview.
Add to Cart: Click on “Add to Cart” to display
your shopping cart at Stuller.com. You can
complete your purchase here.
Print / View: After clicking on “Print” a Design
Report Summary will appear as the first page
along with the main Design Report as page two.
## Rhino History
Design Report Summary: The summary
includes a four-view capture of the design along
with a stone map (see explanation in Gems>Gem
Map) and the listing of stones, metals and parts.
The bottom of the page includes a Customer
Approval signature space, date, and a pricing box.
‘History’
History stores the connection between a
command’s input geometry and the result so
that when the input geometry changes, the result
updates accordingly. For example, with History
Recording and Update turned on, a lofted surface
can be changed by editing the input curves.
Increase or Decrease the Preview: Increase
the print preview size by clicking on the “+”
microscope symbol or decrease the size by
clicking on the “-.”
Where is this Command:
Access this command from Info and
Settings>Rhino History
Steps:
• Click on the Rhino History button to toggle History
Recording and Updating “On” or” Off.”
Preview Layout Styles: Choose a viewable
layout by selecting any one of the layout style
icons shown at the top of the Preview box (one
page, two pages, etc).
### Command Line Options:
Print: Print to your attached printer or save as a
document (if computer software is installed that
allows print to file).
These options are only available when you run the
history command in the Command Line. If you use
the Rhino History button it simply toggles the
recording and updating of History “On” or” Off.”

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
