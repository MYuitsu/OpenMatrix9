# Boolean Difference ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 223-223
- Section title: Boolean Difference ￼
- Section id: m8book1-241-boolean-difference
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p223_img01.png, m8book1_p223_img02.png

---
<!-- section m8book1-241-boolean-difference | Boolean Difference ￼ | m8book1 pages 223-223 -->

<!-- page 223 -->

## Boolean Union
## Boolean Difference
‘BooleanDifference’
This command cuts one solid object out of
another, leaving space in the shape of the cutter.
‘BooleanUnion’
This command combines two solid objects
together. It removes the shared area between
selected objects (where objects overlap) and
creates a single polysurface from the unshared
surfaces.
Where is this Command:
Access this command from Solid>Boolean
Difference
Steps:
Where is this Command:
Access this command from Solid>Boolean
Union
Steps:
• Select Boolean Union from the Solid menu.
• Select surfaces or polysurfaces to union.
• Press Enter when done.
• The objects combine into one polysurface.
• Select Boolean Difference from the Solid menu.
• Select surfaces or polysurfaces to subtract from
(the object being cut into).
• Press Enter when done.
• Select surfaces or polysurface to subtract with (the
cutter).
• Press Enter when done.
Note: See Rules for Successful Booleans listed under
Boolean Union.
Note: Use the Boolean Builder in the Tools or Cutters
menu to boolean meshes. The Boolean Builder
requires both objects to be either meshes or both to
be NURBs objects.
Two surfaces are
joined together
with Boolean
Union. Objects
before (top) and
after (bottom).
Surfaces before (blue, red) and after (green) using
Boolean Difference with the red sphere as the cutter.
### Command Line Option:
### Rules for Successful Booleans:
DeleteInput: Deletes the surface or polysurfaces
being used to subtract with (the cutter). Set
DeleteInput to ‘No’ to keep the cutters.
Note: Use the Boolean Builder in the Tools or Cutters
menu to boolean meshes. The Boolean Builder
requires both objects to be either meshes or both to
be NURBs objects.
## Boolean Intersection
• Objects must be closed (solid) and valid
polysurfaces. Surfaces can also be used.
• Open Surfaces may yield opposite results ( a
Union would cause a Difference).
• Objects must not be co-planar. Co-planar objects
have corners or whole edges that exist on the
same plane in space. (Move one slightly away
from the other one.)
• Objects with lots of sharp corners, or objects that
pass into and then out of one another one or
more times can also have problems booleaning.
• An object that is fully inside another object cannot
be booleaned from it.
‘BooleanIntersection’
This command creates a solid shape only where
two objects intersect. It removes the unshared
ares of the selected polysurfaces or surfaces.
