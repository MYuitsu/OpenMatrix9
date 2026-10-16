# Sweep 2 History ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 186-186
- Section title: Sweep 2 History ￼
- Section id: m8book1-204-sweep-2-history
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p186_img01.png, m8book1_p186_img02.png, m8book1_p186_img03.png

---
<!-- section m8book1-204-sweep-2-history | Sweep 2 History ￼ | m8book1 pages 186-186 -->

<!-- page 186 -->

continuous (G2). In other words, the curves not
only go the same direction when they meet, but
also have the same radius at that point.
Preview: Click to display a preview of the output.
After a setting change, click the Preview button
again to refresh the display.
## Sweep 2 History
‘gvSweep2History’
This History-enabled command creates a surface
from a series of cross-section (Profile) curves and
two rail curves (often Ring Rails). The created
surface automatically updates after profiles or rails
are edited.
Where is this Command:
Access this command from Surface>Sweep 2
History
Or, from the F6 menu when two curves are
selected
Closed Sweep: Creates a closed surface,
continuing the surface past the last Profile
curve around to the first curve. This option is
only available after two cross-section curves are
selected.
Steps:
Note: With closed rail curves, the first cross-section
curve selected is added to the end of the list if you
choose to create a closed surface.
Simple Sweep: Simple Sweep works in cases
where the shape curves intersect the rail at rail
edit points. This option generates simpler surfaces
in cases when the curves are perfectly set up.
• Select Sweep 2 History from the Surface menu.
• Select two rail curves.
• Select profile curves (cross-section) in the order
that the surface will pass through them.
• Press Enter when done.
• Edit a profile, or a rail and press Enter to complete
the edit. See the surface update automatically
after each edit.
Add Slash: Adds additional cross-section
alignments to control how the surface is created
between sections. Select first point on one rail.
Then, select a point on the other rail. Press Enter
when done.
Sweep 2 History displays preview curves to indicate
the Sweep surface direction and the order in which
to pick the Profile curves (pick order in red). Change
Closed = Yes to Closed = No in the Command Line
to create a gap between the first and last selected
profiles.
Add Slash - Additional alignment is added as if more
profiles existed. Select any point on one rail. Use
Perpendicular O-Snaps to set the second point on
the second rail.
