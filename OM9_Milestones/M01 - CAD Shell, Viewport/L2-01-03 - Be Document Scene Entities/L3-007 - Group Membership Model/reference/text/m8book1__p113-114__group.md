---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 113-114
section_title: Group ￼
section_id: m8book1-112-group
---
# Group ￼

Source: matrix8_book_1.pdf, pages 113-114

<!-- section m8book1-112-group | Group ￼ | m8book1 pages 113-114 -->

<!-- page 113 -->

## Center Object
## Group
‘gvCenterObjects’
Center Object returns a dialog window with
options allowing you to center an object, move to
a 0 coordinate on one or more axis or put a point
at center.
Where is this Command:
Access this command from Utilities>Center
Object
‘Group’
Use the Group command to cause separate
objects to act as one group. It is most commonly
used when several objects that are either (1) next
to each other in space, or (2) of the same type
should be treated as one object.
Where is this Command:
Access this command from Utilities>Group
The default keyboard shortcut to Center an
object along the X,Y, and Z axis is Ctrl+Alt+C.
The Default Keyboard shortcut for Group is
Crtl + Q
Steps:
Steps:
• Select Group from the Utilities menu.
• Select the objects you wish to Group together.
• Select the object(s) in the viewport to be centered.
• Select Center Objects from the Utilities menu.
• Select the desired centering options.
Menu Options
Plane:
Note: The objects are NOT joined. They are only
acting as a group for the purposes of commands
such as Move, Duplicate, Mirror, etc. Heads in Matrix
are a common example of a group of objects.
When objects are grouped all objects in the group
are selected when you click on them. You can select
an object within the group by using subselect
(Crtl+Shift+Left Click)
X/Y/Z: Allows you to
optionally toggle along which
axis the selected object will
be centered.
Related Commands:
‘AddToGroup’
Move to 0: Moves the
selected objects center to 0.
Where is this Command:
Access this command from the top drop-down
menu. Edit>Groups>Add to Group
Move at Center: Starts the
Move command from the
center point of the object.
Steps:
Put Point at Center:
Places a point at the center
of the selected object(s).
Coordinate Space:
• Select Add To Group from the Edit drop-down
menu.
• Select the objects you want to add to the group.
• Press Enter.
• Select the group you want to add to.
‘RemoveFromGroup’
C-Plane: Centers the object based on
construction plane coordinates. This is helpful
when you are in Surface View or Gem View.
Where is this Command:
Access this command from the top drop-down
menu. Edit>Groups>Remove from Group
World: Centers the object on world coordinates
(0,0,0).

<!-- page 114 -->

Steps:
## Extract Bad Surface
• Select Remove From Group from the Edit drop-
down menu.
• Select the objects you want to remove from the
group.
• Press Enter.
‘ExtractBadSrf’
Extract Bad Surface separates surfaces with
errors from a polysurface. The polysurface in
this instance would be invalid, which you might
discover using the Object Checker, This command
will remove the surface or surfaces of a model that
are causing it to fail. Then rebuild these surfaces to
complete the model.
‘SetGroupName’
Names a selected group.
Where is this Command:
Access this command from the top drop-down
menu. Edit>Groups>Set Group Name
Where is this Command:
Access this command from Utilities>Extract
Bad Surface
Steps:
Steps:
• Select Set Group Name from the Edit drop-down
menu.
• Select the group of objects to name.
• Press Enter.
• Type the new group name into the Command Line
and press Enter.
## UnGroup
• If a polysurface does not pass the Check
command, use the Extract Bad Surface
command to extract the bad surfaces from the
original polysurface.
• Fix the bad surfaces and use the Join command to
reattach them.
## Show Z Buffer
‘UnGroup’
Returns grouped items to their ungrouped phase
so that they may be selected and used separately.
Where is this Command:
Access this command from Utilities>UnGroup
The Default Keyboard shortcut for UnGroup is
Crtl + W
‘ShowZBuffer’
The Show Z Buffer command shows the depth
and contour of an object in grayscale. It is used
by Matrix Art and other programs. Each pixel is
given a grayscale value that relates directly to the
distance between the camera and that pixel (in
the scene). The closest pixel becomes white and
the furthest pixel becomes black.
Steps:
Where is this Command:
Access this command from Utilities>Show Z
Buffer
• Select Ungroup from the Utilities menu.
• Select the group you wish to return as separate
objects.
Steps:
• Select Show Z Buffer from
the Utilities menu.
• Click again to toggle back
to the normal view.
Note: If gems are part of a group, DO NOT click
UnGroup twice! Gem objects are grouped so that
the program can recognize them as gems. In order
to keep them separate from other items that may be
grouped, the Matrix developers grouped gems twice
so that, in order to UnGroup them, someone would
have to run UnGroup two times. It is very dangerous
to UnGroup a gem object because the program will
cease to recognize it as a gem. If the user thinks a
gem object may have been inadvertently UnGrouped,
DELETE what is left of the former gem and simply
replace it with a new one.
