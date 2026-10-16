# Text ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 227-228
- Section title: Text ￼
- Section id: m8book1-247-text
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p227_img01.png, m8book1_p228_img01.png, m8book1_p228_img02.png, m8book1_p228_img03.png

---
<!-- section m8book1-247-text | Text ￼ | m8book1 pages 227-228 -->

<!-- page 227 -->

Copy Handle Steps:
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces.
## Text
• Select Fillet handle to copy.
• Select new handle location or an existing handle.
Drag your cursor to the desired location. Left-
click to set the location. Repeat as many times as
desired.
• Press Enter when done.
‘TextObject’
Draws text-shaped curves, surfaces or
polysurfaces based on True Type fonts.
Where is this Command:
Access this command from Solid>Text
RemoveHandle: Visible only when at least one
handle has been added. Select the Fillet handle
to remove. Click on the handle(s) to remove and
press enter when done.
Steps:
SetAll: Sets the distance or radius for all handles.
LinkHandles Yes/No: Editing a single handle
updates all handles.
• Select Text from the Solid menu.
• Specify the options in the text object window.
Enter the text to create, select the font, select if
curve, surfaces or solids, etc.
• Pick a location in the viewport to place the Text.
Note:
• Only added handles can be removed.
• The default handles at the ends of each open
edge segment cannot be moved or deleted. This
is the minimum information the command needs
in order to work.
•  The handle at the end of a single closed edge
can be moved but not deleted.
### RailType options
DistFromEdge: The distance from the edge
curves determines the intersection,
Rolling Ball: The radius of a rolling ball
determines the intersection (default).
DistBetweenRails: The distance between the
edge rails determines the intersection.
### Handle Options Continued:
SelectEdges: Allows selecting more edges after
the selection has been closed.
### Options:
Text to Create: Type the desired text  in this
window. Right click in the edit area to cut, copy
and paste text.
Preview Yes/No: ‘Yes’ displays a dynamic
preview. You can change the options at this
point and the preview will update. ‘No’ means no
preview will display. There is no opportunity to
change the options.
Font: Selects the font and style.

<!-- page 228 -->

Solid Thickness: Sets the thickness of text
created from solids (Z direction)
Note: Matrix uses the fonts in Windows on your
computer. Add fonts in Windows to have them
available in ‘Text’.
Name: Use the drop down menu to select from
the installed fonts on your computer.
Lower case as small caps: This option creates
lower case letters as small caps. Set the relative
text size as a percentage of the normal text.
Add spacing: Adds spacing between letters.
Note: Not all fonts will create valid surfaces or
solids without requiring reworking.
Bold: Sets the font to Bold.
## Extrude Curve Straight
Italic: Sets the font to Italic.
‘ExtrudeCrv’
Creates solid polysurfaces from closed, planar
curves. This is the same command as Extrude
Curve All but it defaults to the Straight option.
Comic Sans font used to create curves with ‘Text’.
Where is this Command:
Access this command from Solid>Extrude
Curve Straight
### Create
Steps:
Specifies the object type.
Curves: Creates text using outline curves.
Surfaces: Creates text using planar surfaces.
Solids: Creates text using solids.
• Select Extrude Curve Straight from the Solid
menu.
• Select Curves to Extrude.
• Press Enter when done.
• Select Command Line options.
• Specify an extrusion distance by picking a point or
typing in a value.
Note: Not every font is suitable to create as valid
solids. They may first need to be fixed at the curve
level. Then, extrude the curves as solids.
Solid polysurfaces are created from closed curves.
Solid polysurface text created with ‘Text’.
Group Objects: Groups the resulting text
objects.
Allow single-stroke fonts: Allows fonts that
do not have closed loop boundaries. Engraving-
style fonts for machine applications are the most
common. If unchecked, single-stroke engraving
fonts display closed loop boundaries
Text Size: Set text height and thickness.
Notes:
• Curves must be planar, or, appear as a single line
when seen in another viewport.
• Nonplanar curves will not extrude properly.
• Curves must be closed and joined, or have position
continuity to create valid, solid objects with no
naked (open) edges.
• When two or more curves are selected, they
should not intersect. If they do, trim them together
first.
• In order to make a hole through the solid object,
one curve should be fully inside the other, and on
the same plane in space.
• When curves on separate planes are selected
together, two solid objects will result.
Height: Sets the height in model units (mm as
default in the Y direction).
