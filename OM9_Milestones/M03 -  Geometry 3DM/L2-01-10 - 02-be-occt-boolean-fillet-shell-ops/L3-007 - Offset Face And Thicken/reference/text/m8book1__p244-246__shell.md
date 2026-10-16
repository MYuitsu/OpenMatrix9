# Shell ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 244-246
- Section title: Shell ￼
- Section id: m8book1-268-shell
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p244_img01.png, m8book1_p244_img02.png, m8book1_p244_img03.png, m8book1_p246_img01.png

---
<!-- section m8book1-268-shell | Shell ￼ | m8book1 pages 244-246 -->

<!-- page 244 -->

### Command Line Option:
Copy: Specifies whether or not the objects
(holes) are copied.
To Boundary
object is used in
Move Face.
## Move Face
‘MoveFace
Moves a polysurface face.
Where is this Command:
Access this command from Solid> Move Face
Delete Boundary Yes/ No: This determines
if the designated boundary is removed from the
selection or not.
Steps:
## Shell
• Select Move Face from the Solid menu.
• Select a face or faces to move.
• Press Enter when done.
• Pick a point to move from.
• Pick a point to move to.
‘Shell’
Creates a hollowed-out shell from a solid, closed
polysurface.
Note:
Where is this Command:
Access this command from Solid> Shell
Steps:
• Select Shell from the Solid menu.
• Select faces to remove from closed polysurface.
Leave at least one face unselected.
• Press Enter when done.
• The surrounding joined surfaces are adjusted
to accommodate the new face shape and
orientation.
• All adjusted faces must be either planar or easy to
stretch.
• Holes in the adjusted faces generally do not move
or stretch.
Note: Shell only operates on simple, solid, manifold
(only one face joined to one edge) polysurfaces.
These surfaces are removed and the remainder is
offset inward, using the outer parts of the removed
surfaces to join the inner and outer parts.
Faces (in
green) were
selected to
move with
Move Face.
Before (left)
and after
(right).
### Command Line Options:
Direction Constraint=None (default): Faces
can be moved in any direction.
Shell before (left) and after (right).
Direction Constraint=Normal: Face can only
be moved in the positive or negative normal
direction (facing toward the interior of the object).
### Command Line Option:
Thickness: Specifies the thickness amount for
the Shell.
ToBoundary: The face is moved until the side
surfaces intersect the boundary object and the
face itself is replaced by a trimmed section of the
boundary object.

<!-- page 245 -->

<!-- page 246 -->

### Instruction Manual, Book 1
Visit www.gemvisionforum.com to connect
online with thousands of other users and Gemvision
to share information and resources. You’ll also have
access to tutorials, tips, news, and announcements
direct from Gemvision.
Matrix Technical Support:
TOLL FREE 888-357-6272
EMAIL matrix@gemvision.com
© 2015 GEMVISION CORPORATION
