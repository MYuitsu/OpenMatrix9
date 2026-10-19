# Extract Bad Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 114-114
- Section title: Extract Bad Surface ￼
- Section id: m8book1-114-extract-bad-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p114_img01.png

---
<!-- section m8book1-114-extract-bad-surface | Extract Bad Surface ￼ | m8book1 pages 114-114 -->

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
