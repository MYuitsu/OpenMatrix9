---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 32-34
section_title: Using Info & Settings Menu
section_id: m8book1-008-using-info-settings-menu
---
# Using Info & Settings Menu

Source: matrix8_book_1.pdf, pages 32-34

<!-- section m8book1-008-using-info-settings-menu | Using Info & Settings Menu | m8book1 pages 32-34 -->

<!-- page 32 -->

# CHAPTER 7
# Using Info &
# Settings Menu
### See basic or complex information about models, keep project notes, select certain
### types of objects, and control History.
Object info
Rhino Options - Changing the Viewport
Properties
Rhino Options to change viewport properties
(color, dimensions, etc.) are found by clicking the
button for Rhino Options in the Info & Settings
menu (or by typing “Options” into the Command
Line and pressing Enter). It is NOT recommended
that you change things in here if you are not
aware of what they do, as these are the program
settings and there is no easy way to revert to
the defaults. However, a quick exploration of the
“Appearance” menu (find menu on left-hand side
of this screen) will show you where to change
the grid colors, while the “Grid” menu under
“Document Properties” will help you change the
Grid Extents (Size) and grid line properties. “Units”,
three menus above that, lets you switch from
mm to other dimensions, such as feet (yes, more
Matrix customers than you would guess have
modeled their new store layout in the software.).
Finally, the “Keyboard” option shows you where
the keyboard shortcuts are found.
Object Properties / Object Information /
All Object Information
The program stores information about all objects
on-screen. To see this information in quick, easy-to
read format or longer, more complex format, use
one of these tools. Object Info displays a quick
“pop up” label beside a
selected object in the
viewports. This display
is the layer color of the
object it is referring to.
It displays the type of
object and important
information about
it: surface or polysurface
(made up of more than once surface); closed or
open (can’t be used in growing/milling), valid or
invalid (can’t be used
in Boolean and further
modeling operations),
and point count (for
curves), or surface count
(for polysurfaces).
All Object Info
Appearance Menu
As you learn more
about the program, you
will come to appreciate all this
information more. To use Object Information,
select the object or objects for which you wish
to see information displayed, and click this tool.
Their information boxes will appear. Press Enter to
hide them again. To view All Object Information,
or Object Properties select the object(s) to view
and click the button. All the same information is

<!-- page 33 -->

Project Notes
Start this up and type any project notes
displayed about all objects on-screen; however,
there is more stuff to wade through that we’re not
going to explain here. If an advanced user needs
this information; it’s still here.
you wish to keep in the Saved file into
the window that appears. Click “Done”
to keep them, and open them again in the future.
Check the Notes by clicking here again. Just
remember to do a File > Save As, or place the file
into a Master Job Bag, in order to save the Notes
with the file.
Super Select
This tool determines what types of objects you
The Gumball
The Gumball is a set of translation handles similar
to the Gem Positioner (MSR) or the T-Spline
translation handles and in some instances they
can be used interchangeably. When Gumball is on
the Gumball will appear centered on whatever
object or objects are selected. With Gumball you
can Move, Scale, and Rotate the object using the
handles.
At the left the Gumball appears
on a selected cube. Bellow are
the 3 Gumball commands that
appear in the Info & Settings
menu.
Smart Targets MSR
The Smart Targets MSR looks similar to the
Gumball but has some additional functionality.
have on-screen and will display them in the
menu that appears It is a quick way to
check if you have any open curves (won’t
properly build solids), open surfaces (won’t
mill or grow correctly), invalid surfaces (won’t
Boolean or function properly in modeling/
growing), or duplicate objects (especially a pain:
located right on top of one another and therefore
indiscernible). If these problematic objects exist, or
if you simply wish to, for instance, select all curves
and hide/ job bag/ delete them, etc., you can do
so here. To use this tool, simply make the selection
from the list that appears and it will be highlighted
in the viewports for further modeling operations.
When a Smart Target with Smart
MSR is selected, the whole
thing highlights pink. Click on
one of the handles to edit each,
individually.
Rhino History On / Off
This History stores the connection between a
command’s input geometry and the result so
that when the input geometry changes, the result
updates accordingly. For example, with History
recording and Update turned “On,” a lofted
surface can be changed by editing the input
curves.
It applies a selection filter for Smart Targets
making it so no other objects can be selected.
This makes it easier to move the Smart Target
(input geometry/Parent) without accidentally
moving the blended curves (output geometry/
child). The Smart Target MSR also makes it
possible to manipulate a single handle of the
Smart Target as opposed to the whole target.
Matrix History Record & Update On / Off
Turn Matrix History Record and Update “Off”
when they are not needed; otherwise, we
recommend you leave them on: all History-related
functionality will work as expected when they’re
both on.
Command History
The Command History button displays the
commands that have been passed by Matrix to
Rhino. Scroll through to see the list of processes
completed.
Matrix Clear History
Use Clear History on any object to let Matrix
forget about updating a single part. This function
is particularly useful while dragging objects from
one location to another on-screen and you want
the program to forget the history only on certain

<!-- page 34 -->

items. That way, Matrix does not have to work so
hard at remembering everything.
Design Summary, a gem map along with a four-
view render of your design, to organize your jobs
more efficiently.
### Skills & Commands in this Chapter
Viewport Tab Toggle
Matrix has added viewport tabs which make it
easier to switch between viewports when one
view is maximized. The Viewport Tabs Toggle will
hide these tabs if you so choose.
Rhino Options:
Rhino Options, Object Properties, Viewport Tabs
Toggle, and Display Menu
Display Menu
This controls the appearance of objects in your
viewport. Control things like whether curves are
shown or if objects cast a shadow.
Object Info, Command History & Notes
Object Info, All Object Info, Command  History,
Notes
Box Edit Menu
This tool edits objects according to their
bounding box size, orientation, and position. New
bounding boxes are calculated each time the
settings are applied.
Other:
Super Select,Box Edit, Library, Selection Filter
Menu, Smart Track, Design Report
Library Menu
Materials, textures and environment files are
contained in Rhino’s Libraries and displayed here.
History:
Rhino History, Matrix History Update, Matrix
History Record, Matrix Clear Object History.
Selection Filter
This feature allows optional selection of the types
of objects that you want to be able to select. It is
like locking by an object class as opposed to by
an individual object or the layer they are on. Left-
click on a check mark to ‘lock’ that class of object.
Right-click on a check box to deselect all objects
but that one.
Translation Handles:
Smart Targets MSR, Gumball Alignment,
Relocate Gumball, Gumball
The Selection Filter opens in a window. Dock it by dragging
it to the edges of the viewport.
Smart Track
Use this modeling aid to reference tracking lines
and smart points while drawing items in the
viewports.
Design Report
The Design Report accesses the model’s render
materials while joining reports for gem and metal
weights, and providing real-time Stuller pricing
quotes. Use this, combined with the Print option,
