# Rebuild ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 188-189
- Section title: Rebuild ￼
- Section id: m8book1-206-rebuild
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p188_img01.png, m8book1_p188_img02.png, m8book1_p188_img03.png, m8book1_p188_img04.png, m8book1_p188_img05.png, m8book1_p188_img06.png, m8book1_p188_img07.png, m8book1_p189_img01.png, m8book1_p189_img02.png, m8book1_p189_img03.png, m8book1_p189_img04.png

---
<!-- section m8book1-206-rebuild | Rebuild ￼ | m8book1 pages 188-189 -->

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

<!-- page 189 -->

Calculate: The calculation tests how far away the
new surface is at knot line intersections and half-
way between knot lines. Conducts tests at knot
line intersections and halfway between knot lines.
The display color indicates how far away the new
surface is from the original. Points are; green if the
surface is within absolute tolerance; yellow if it is
between tolerance and 10 times tolerance, and;
red if it is farther away than that. The lines are 10
times longer than the measured deviation in the
direction of the deviation.
Surface after Rebuild with point counts of U=6 and
V=6.
Note: Rebuild can also be used for curves, extrusions
(a type of surface), along with unjoined surfaces.
However, a combination of these cannot be selected
at the same time. Run the command individually for
each object type.
### Rebuild Surface Options:
Rebuild Surface preview.
Point Count: A surface has a U and V direction
(perpendicular). The input surface UV point
counts are shown in parenthesis next to the value
boxes. Enter a new U and V point count for the
rebuilt surface in the respective value boxes.
Degree: A surface degree is always set to 3.0.
So, in the case of rebuilding surfaces leave this
value set to 3.0. Curves have a different degree
value. Straight curves are 1.0 degree.
Preview: Click to display a preview of the
output. If you change the settings, click the
Preview button again to refresh the display. The
highlighted lines are the original surface. The
green lines are the Preview of the new  surface.
### Options:
## Blend Surface
‘BlendSrf’
Creates a surface that connects or blends
together two, open or closed, surface edges.
Delete Input:
Deletes the
original geometry
from the file.
Where is this Command:
Access this command from Surface>Blend
Surface
Current Layer: Creates the new surfaces on the
current layer. Clear this check box to place the
new surface on the layer of the original surface.
Steps:
Retrim: Trims the rebuilt surface with the original
trimming curves.
Maximum deviation: Reports the maximum
deviation from the
original surface.
• Select Blend Surface from the Surface menu.
• Select a segment for the first edge.
• Select the next adjacent segment for the first
edge. Continue selecting segments to complete
the first edge and press Enter when done.
• Select a segment for the second edge. Continue
selecting segments to complete the second edge
and press Enter when done.
