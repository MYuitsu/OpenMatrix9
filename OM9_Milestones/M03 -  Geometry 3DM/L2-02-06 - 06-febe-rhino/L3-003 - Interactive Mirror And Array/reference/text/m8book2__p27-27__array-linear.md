# Array Linear ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 27-27
- Section title: Array Linear ￼
- Section id: m8book2-025-array-linear
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book2_p27_img01.png, m8book2_p27_img02.png, m8book2_p27_img03.png

---
<!-- section m8book2-025-array-linear | Array Linear ￼ | m8book2 pages 27-27 -->

<!-- page 27 -->

### Command Line Options:
Number: The number of total objects resulting
from the array
Objects arrayed with No Rotation.
Roadlike: Objects follow the curve but
maintain a consistent “up” direction relative to
the Construction plane. Select a C-Plane for the
objects to orient when prompted.
Preview:
Yes: Displays a dynamic preview. The options can
be changed and the preview will update.
No: No preview will display. There is no
opportunity to change the options.
## Array Along Surface
‘ArraySrf’
Copies objects in rows and columns on a surface.
The surface normal direction determines the
orientation of the objects.
Where is this Command:
Access this command from Transform>Array
Along Surface
Steps:
Objects arrayed Roadlike with the Looking Down
C-Plane selected.
## Array Linear
• Select Array Along Surface from the Transform
menu.
• Select the Object(s) to Array.
• Press Enter when done.
‘ArrayLinear’
Copies and spaces objects in a single direction.
The array directions are the X,Y, and Z directions of
the active Construction plane.
Where is this Command:
Access this command from Transform>Array
Linear
Steps:
• Select Array Linear from the Transform menu.
• Select the Object(s) to Array.
• Press Enter when done.
• Select the Number of items for the Array.
• Pick the first Reference point.
• Pick the second Reference point.
• Pick a Base point relative to the objects that will be
arrayed on the surface.
• Pick a Normal direction for the objects, or press
Enter if the normal direction for the object is the
Construction plane Z-axis. This direction will be
normal to the surface.
• Select the Target surface.
• Type the Number of elements in the U-direction.
• Type the Number of elements in the V-direction.
Note: The direction of the array and the distance
between the objects is determined by the second
Reference point.
The objects are arrayed over the untrimmed
surface evenly, spaced along the U and V
directions of the surface. If objects are placed
upside down, change the U or V direction (V
Reverse was used in the example below) of the
surface by using the ‘Dir’ command.
Array Linear with five as the number of items.
