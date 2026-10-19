# Profile Sweep ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 187-188
- Section title: Profile Sweep ￼
- Section id: m8book1-205-profile-sweep
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p187_img01.png, m8book1_p187_img02.png, m8book1_p188_img01.png, m8book1_p188_img02.png, m8book1_p188_img03.png, m8book1_p188_img04.png, m8book1_p188_img05.png, m8book1_p188_img06.png, m8book1_p188_img07.png

---
<!-- section m8book1-205-profile-sweep | Profile Sweep ￼ | m8book1 pages 187-188 -->

<!-- page 187 -->

Rules:
## Profile Sweep
‘gvProfileSweep’
This History-enabled command creates a surface
between two Profiles or any two, planar, closed
curves without requiring a rail curve.
• Select multiple open Profile curves (cross-section)
near the same ends.
• Select multiple closed Profile curves (cross-section)
in the order the surface will pass through them.
• Select either all open or all closed cross-section
curves, but not a combination of both.
### Command Line Options:
Where is this Command:
Access this command from Surface>Profile
Sweep
Or, from the F6 menu when two Profiles or two
planar, closed curves are selected
Steps:
Closed=Yes (default): Creates a closed surface,
continuing the surface past the last profile curve
around to the first profile curve. This option only
works when there are two or more profiles as part
of a Sweep.
Closed=No: Creates an open surface. Leaving
a gap between the first and last profile selected.
This option only works when there are two or
more profiles as part of a Sweep.
• Select Profile Sweep from the Surface menu.
• Choose from any Command Line options
• Select two Profiles (two, planar, closed curves).
• Edit a profile and press Enter to complete the edit.
See the surface update automatically after each
edit.
Note:
Flip=Yes/No: When Flip equals ‘Yes’ it reverses
the direction of the Sweep and therefore the
direction of the created surface.
Maintain Height=Yes/No: A relationship
between the height and the width is maintained;
where the rails are wider apart, the surface is
taller; where they are closer together, the surface
is shorter. Height is kept consistent regardless
of width when Maintain Height is ON. This is the
default setting.
• With Profile Sweep, Matrix creates a blended
surface using the shortest path between the two
Profiles.
• Edit the scale or position of the Profiles and the
surface will automatically update with Matrix
History turned ON.
• Use Profiles from Profile Placer, if possible. The
Profile Sweep command utilizes the profile’s origin
point.
• Convert a closed, planar curve into a Profile curve.
Two closed, planar curves are needed to run this
command. (See Tools>Profile for making a Profile)
• Be sure Matrix History is turned on before running
this command. Using Trim, Boolean, Split or Join
on the curves or surface will break History. If
History breaks, the surface will not update with
edits. (See Info & Settings menu> Matrix History.)
Lay out two sets of
mirrored Profiles on a
Ring Rail. Use Profile
Sweep between the
two mirrored Profiles
on each side.
Maintain Height is ON (left) and turned Off (right).

<!-- page 188 -->

Use Profile Sweep
between the top set
of mirrored Profiles.
Then, edit the Profile
with MSR, rotating
it into the desired
position.
Blend Amount is set to 0.5 along with using the
Ring Rail option in Profile Sweep. The blend avoids
the space inside the Ring Rail regardless of the
Blend Amount set.
## Rebuild
Profile Sweep surface now follows the Bezel after the
Profile is rotated and edited.
‘Rebuild’
Use Rebuild to assign a new number of U and V
control points to unjoined surfaces.
### Command Line Options:
Where is this Command:
Access this command from Surface>Rebuild
Or, from the F6 menu when a surface is
selected
Blend Amount: A Blend Amount can be set
starting at 0.1 up to 1.0. The default of ‘1.0’ gives
the best results the majority of the time. However,
experiment with this setting to get the results
desired.
Steps:
Blend amount
of 1.0 (left)
and 0.5
(right).
• Select Rebuild from the Surface menu.
• Select a surface or multiple unjoined surfaces to
Rebuild.
• Press Enter when done.
• Set the Point count U and V values in the Rebuild
Surface Options box.
• Click on “OK’ when done.
The existing
surface UV point
counts (shown in
image = 2) are in
parenthesis near
each value box.
Start Blend: Values can be set between 0.1 and
1.0 to adjust the amount of the blend at the Start
of the input curve. The default value of ‘1.0’ is best
for most blends.
End Blend: Values can be set between 0.1 and
1.0 to adjust the amount of the blend at the End
of the input curve. The default value of ‘1.0’ is best
for most blends.
A simple surface with point counts of U=2 and V=2.
Ring Rail: This options allows selection of a Ring
Rail that the output surface avoids regardless of
the Blend Amount set.
