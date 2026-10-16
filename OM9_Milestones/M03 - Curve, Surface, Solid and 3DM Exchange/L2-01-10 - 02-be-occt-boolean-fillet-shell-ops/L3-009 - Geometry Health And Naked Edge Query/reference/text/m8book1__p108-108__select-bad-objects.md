# Select Bad Objects ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 108-108
- Section title: Select Bad Objects ￼
- Section id: m8book1-102-select-bad-objects
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p108_img01.png, m8book1_p108_img02.png

---
<!-- section m8book1-102-select-bad-objects | Select Bad Objects ￼ | m8book1 pages 108-108 -->

<!-- page 108 -->

## Check
‘Check’
Check reports errors in the selected object’s data
structure. Unfortunately, there is no reason we
can give in this help as to why a model develops
an error. Hopefully, most users will never see
the window appear with the occurrence of bad
geometry.
During advanced modeling techniques such as Smart
Flow and curve network controlling the direction of
surface normals as well as the orientation of U and V
direction is important. For surfaces, a user may need
to alter normals when applying a bitmap texture map
or during a Boolean operation for a surface that is
not closed. If gems placed with Gem Layout tools
appear upside-down, it is usually necessary to adjust
Directions on a surface. Sometimes, if a polysurface
is open and it doesn’t shade properly, it is necessary
to check its directions and make sure they are
consistent (all facing out).
Where is this Command:
Access this command from Utilities>Check
## Select Bad Objects
Steps:
‘SelBadObjects’
“Bad” objects are ones that violate certain NURBS
rules or have structural issues that cause invalid
geometry. The Select Bad Objects command
scans all objects in the viewport. If a model is
“bad” or invalid, an error message appears and the
object highlights.
• Select Check
from the
Utilities menu.
• Select objects
to check.
Note: If there are no problems with the model, a
message stating so appears in the feedback window.
Where is this Command:
Access this command from Utilities>Select Bad
Objects
Steps:
If there are errors in the model, a message
displays. Click the Done button to close the
window. The best fix for a failed model is to delete
and rebuild it.
## Show Edges
• Click the Select Bad Objects command from the
Utilities menu.
• If bad objects are present the ‘Model Errors’ dialog
appears.
• Click Done in the ‘Model Errors’ dialog. The bad
objects will be selected.
Note: The best way to fix bad models is to delete or
rebuild the object.
‘ShowEdges’
The Show Edges command shows the edges of
a surface or polysurface if it has open or naked
edges, or if the object is closed. This tool shows
the start/stop point and sections of a surface
edge as shown in the Split Edge section. It is
also useful to preview where an object would be
separated if the Explode command is used.
If there are no errors in any objects in the viewport,
a message in the
feedback window
appears indicating
there are no bad
objects.
Where is this Command:
Access this command from Utilities>Show
Edges
Steps:
It is a good idea to check models right after
Boolean operations, which can cause invalid
models. If it is early in the design process and a
model has failed, it is best to remake the model.
• Select a polysurface for edge display.
• Select Show Edges from the Utilities menu.
This opens the ‘Edge Analysis’ menu.
