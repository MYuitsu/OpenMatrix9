# Offset Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 207-207
- Section title: Offset Surface ￼
- Section id: m8book1-224-offset-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p207_img01.png

---
<!-- section m8book1-224-offset-surface | Offset Surface ￼ | m8book1 pages 207-207 -->

<!-- page 207 -->

### Handle Options:
AddHandle: Adds a handle along the edges.
Drag the cursor and click to set the location. Press
Enter when done.
CopyHandle: Adds a new handle using the
distance from the selected handle. Drag to move
the position, click to set it, and press Enter when
done.
A split surface is offset with Offset Surface to create
a custom channel bottom.
### Command Line Options:
RemoveHandle: Visible only when at least one
handle has been added. Click on a handle to
remove it.
Distance: Enter a distance amount for the offset.
Type a new value and press Enter when done.
SetAll: Sets the distance or radius for all handles
and all handles will be updated.
Corner = Round/ Sharp: Round creates a fillet
at sharp corners on the original surface. Sharp
maintains the sharp corner when the original
surface has a sharp corner.
Note: Only added handles can be removed. The
default handles at the ends of each open edge
segment cannot be moved or deleted. This is the
minimum information the command needs in order
to work. The handle at the end of a single closed
edge can be moved but not deleted.
Solid = Yes/ No: Makes a closed solid from the
input and offset surfaces by lofting a ruled surface
between all of the matching edges.
TrimAndJoin: Trims and joins the resulting
surface to the input surfaces when set to ‘Yes’.
Loose = Yes/ No: For surfaces only, the
resulting surface point structure is identical to the
original surface when set to ‘Yes’.
Preview Yes/No: Yes; displays a dynamic
preview. When set to ‘No’, no preview will display.
There is no opportunity to change the options.
## Offset Surface
Tolerance: Sets the tolerance for the offset
surface. Type ‘0’ to use the default tolerance.
Both Sides: Creates an offset on both sides of
the original.
‘OffsetSrf’
Copies a surface or polysurface so that locations
on the copied surface are the same specified
distance from the original surface.
FlipAll: Changes the offset direction on all the
input objects.
Where is this Command:
Access this command from Surface>Offset
Surface
## Variable Offset Surface
Steps:
‘VariableOffsetSrf’
Use this tool to deform the surface while it offsets.
Offset different parts of a surface to different
varying distances away from the original surface
Where is this Command:
Access this command from Surface> Variable
Offset Surface
• Select Offset Surface from the Surface menu.
• Select a surfaces or polysurfaces to offset.
• Press Enter when done.
• Select object to flip direction (if needed). Preview
arrows will display the direction of the offset. Click
on the object to change the offset (flip) direction.
Enter an offset distance amount in the Command
Line when selecting ‘Distance’.
• Press Enter when done.
