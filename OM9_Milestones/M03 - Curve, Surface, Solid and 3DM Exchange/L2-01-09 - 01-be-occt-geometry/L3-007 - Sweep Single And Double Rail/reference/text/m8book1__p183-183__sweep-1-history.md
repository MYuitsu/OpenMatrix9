# Sweep 1 History ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 183-183
- Section title: Sweep 1 History ￼
- Section id: m8book1-202-sweep-1-history
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p183_img01.png, m8book1_p183_img02.png, m8book1_p183_img03.png, m8book1_p183_img04.png

---
<!-- section m8book1-202-sweep-1-history | Sweep 1 History ￼ | m8book1 pages 183-183 -->

<!-- page 183 -->

Refit Rail: This refits the rail curve, taking the
curve’s control points and rearranging them to
smooth the curve. See ‘Refit to Tolerance” (FitCrv)
in the Curve menu.
Note: The fitting tolerance for the rail curve is
controlled by Rhino Options > Document
Properties > Units > Absolute Tolerance.
Preview: Click to display a preview of the output.
After a setting change, click the Preview button
again to refresh the display.
Edit a Profile curve. Press Enter to complete the edit
and Sweep 1 History will automatically update the
surface.
Rules:
## Sweep 1 History
• Select multiple open cross-section curves near the
same ends.
• Select multiple closed cross-section curves in the
order the surface will pass through them.
• Select either all open or all closed cross-section
curves, but not a combination of both.
‘gvSweepHistory’
This History-enabled command creates a surface
from a series of cross-section (Profile) curves and
a single rail curve (often a Ring Rail). The created
surface automatically updates after a Profile or the
rail is edited.
### Command Line Options:
Where is this Command:
Access this command from Surface>Sweep 1
History
Closed=Yes (default): Creates a closed surface,
continuing the surface past the last curve around
to the first curve. This option only appears with a
rail that is a closed curve and two or more Profiles.
Or, from the F6 menu when a Ring Rail is
selected
Steps:
Closed=No: Creates an open surface, leaving
a gap between the first and last profile selected.
This option only works when there are two or
more profiles. This option only appears with a rail
that is a closed curve and two or more Profiles.
Flip=Yes/No: When Flip equals ‘Yes’, it reverses
the direction of the Sweep (as shown by the
preview curves) and therefore the direction of the
surface created.
• Select Sweep 1 History from the Surface menu.
• Select a rail curve.
• Select Profile curves (cross-section curve), one-by-
one in the order that the surface will pass through
them following the direction of the arrows.
• Press Enter when done.
• Edit a profile, or the rail and press Enter to
complete the edit. See the surface update
automatically after each edit.
Sweep 1
History
displays
preview
curves to
indicate the
Sweep surface
direction and
the order in
which to pick
the Profile
curves (pick
order in red).
Using Flip with Sweep 1 History, before (left) and
after (right).
