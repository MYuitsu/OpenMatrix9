# Picture Frame

- Source PDF: Matrix 8 Instruction Manual Book 1 (h:\OpenMatrix9\manuals\matrix8_book_1.pdf)
- Page range: 100-100
- Section title: Picture Frame
- Section id: m8book1-087-picture-frame

---

<!-- section m8book1-087-picture-frame | Picture Frame | m8book1 pages 100-100 -->

<!-- page 100 -->

## Picture Frame
SelfIllumination: The image mapped to the
Picture Frame plane always displays at full
intensity and is not affected by light or shadow.
‘PictureFrame’
Place a scalable and movable image in the
viewports using two reference points (similar to
the Rectangle command in the Curve menu)  to
use as a guide in modeling.
Where is this Command:
Access this command from View>Picture
Frame
Steps:
Left=Self Illumination ‘No’. Right=Self Illumination
‘Yes’.
• Select Picture Frame command from the View
menu.
• Select an image from the file browser.
• Pick a corner of the plane.
• Pick or type the length in the Command Line. The
plane retains the same aspect ratio as the source
image. (Hold ‘Shift’ key for Ortho)
EmbedBitmap: Matrix stores the image in the
.3dm file. This increases the file size, but ensures
that the image is always available to the Picture
Frame surface. Otherwise, when this option is
‘OFF’, if you move or delete the image from the file
system, it will disappear from Matrix.
Note: Edit Picture Frame properties such as
transparency using the Material Properties.
### Options:
Autoname: Automatically assigns the image files
name to be the Name property for the Picture
Frame surface.
Vertical: Draws the picture frame vertical to the
current construction plane.
The Autoname option allows you to select the picture
frame by its file name using the SelName command.
‘Vertical’ draws the Picture Frame vertical to the
active Construction plane.
AlphaTrasnparency: If the image has alpha
channel transparency, the transparent areas
can either show the object color through the
transparency or make the object transparent.
