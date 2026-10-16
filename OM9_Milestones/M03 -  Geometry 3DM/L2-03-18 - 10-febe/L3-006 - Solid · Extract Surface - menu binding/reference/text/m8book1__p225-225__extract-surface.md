# Extract Surface ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 225-225
- Section title: Extract Surface ￼
- Section id: m8book1-245-extract-surface
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p225_img01.png, m8book1_p225_img02.png, m8book1_p225_img03.png, m8book1_p225_img04.png

---
<!-- section m8book1-245-extract-surface | Extract Surface ￼ | m8book1 pages 225-225 -->

<!-- page 225 -->

## Extract Surface
Outputlayer = Current: Current places the
results (the extracted surface) on the current layer.
‘ExtractSrf’
Use this command to separate a component
surface from a polysurface, creating a single
surface and an open polysurface.
With the
“OutputLayer=Current”
option, the extracted surface
will appear on the current
Layer color (Metal 01).
Where is this Command:
Access this command from Solid>Extract
Surface
Outputlayer = Input: Places the results (the
extracted surface) on the same layer as the input
curve.
Or, right-click the command to activate the
Copy=Yes option.
Steps:
## Fillet Edge
‘FilletEdge’
Use this tool to fillet (add tangent curvature to)
edges in a polysurface.
• Select Extract Surface from the Solid menu.
• Select surfaces or polysurfaces to extract.
• Press Enter when done.
• The extracted surface remains in the same place
until it is deleted or moved by you.
Where is this Command:
Access this command from Solid>Fillet Edge
Right-Click Command: Fillet Edge - Previous
Edge
Steps:
Using Extract Surface to extract the surface on the
shank end.
• Select Fillet Edge from the Solid menu.
• Select edges to Fillet. (Choose a ‘NextRadius’
amount before selecting the edge or use the
default amount shown.)
• Press Enter when done.
• Select Fillet handle to edit.
• Press Enter when done.
Note: Moving a handle at the end of the edge will
cause the fillet to extend beyond the surface. This
will have to be trimmed by other means.
Note: The polysurface may be open or closed, but
component surfaces must be joined.
Note:
• The surfaces are separated from the polysurfaces.
The remainder of each polysurface will stay joined.
• The extracted surface is also removed from any
groups to which the original surface may belong.
• To remove a surface from the selection, press the
‘Ctrl’ key while clicking.
• When extracting a surface from a polysurface this
command can save time. It only lets you extract
the selected surfaces from the polysurface
instead of using Explode and having to join
everything together again.
### Command Line Options:
Fillet before (left) and after (right).
Copy=Yes/No: Specifies whether or not the
objects (the surfaces being extracted) are copied.
A plus sign appears at the cursor when copy
mode is on.
