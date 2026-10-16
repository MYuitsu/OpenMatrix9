# Smart MSR ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 148-149
- Section title: Smart MSR ￼
- Section id: m8book2-153-smart-msr

---

<!-- section m8book2-153-smart-msr | Smart MSR ￼ | m8book2 pages 148-149 -->

<!-- page 148 -->

Note: Smart MSR handles appear on selected Smart
Targets.
Note: It is easy to forget that you have Smart MSR
turned ON. When Smart MSR is active only Smart
Targets can be selected. Turn OFF by selecting Smart
MSR from the Tools menu when it isn’t needed.
Moving the Smart Target with Smart MSR:
Steps:
Note: A Smart Blend (green) curve lies between two
Blend Points.
• Select a Smart Target
• Select a directional handle (Light Pink, Dark Pink
or White arrows) to move the Smart Target in X, Y
or Z.
• Click and drag the arrow into the desired position.
## Smart MSR
‘gvSmartMSR’
Smart MSR acts as a selection filter for Smart
Targets making them easier to select and
manipulate without breaking history. Smart MSR
also exposes additional functionality making it
possible to modify part of a Smart Target as
opposed to the whole target.
Where is this Command:
Access this command from Tools>Smart MSR
Note: MSR handles (arrows) to move.
Or from Info and Settings>Smart Targets MSR
Smart MSR Selection Filter Steps:
Rotating a single handle in a Smart Target:
When you rotate a Smart Target using the
Gumball or other methods all of the Blend Target
Handles rotate together. With Smart MSR you can
rotate the handles independent of one another.
• Select Smart MSR from the Tools menu.
• Select Smart Targets using any selection method.
(All other objects will be ignored.)
• MSR (Gumball) handles will appear on the Smart
Target.
Note: A Smart Target must already have more than
one set of Smart Target Handles in order to rotate
them independently.
Steps:
• Select a Smart Target.
• With the Smart Target selected click on the Smart
Target Handle you wish to rotate.
• Use the Smart MSR controls to manipulate the
Smart Target Handle. (Represented by the circle
segments in Light Pink, Dark Pink and White).
• Click and drag into the desired rotated position.

<!-- page 149 -->

## Image Trace
‘gvImageTrace’
Create curves from an image to use for modeling
in Matrix.
Where is this Command:
Access this command from Tools>Image Trace
Steps:
Note: Smart MSR rotation handles.
## Join History
• Select Image Trace from the Tools
menu.
• Click on the Image input box in the
Image Trace menu.
• Navigate to the desired file on you
computer from the file browser window.
• Adjust the Image Trace settings as necessary.
• Press Enter to end command.
‘gvJoinHistory’
This command creates a child curve that would
be the result of two or more curves if they were
joined. The parent curves remain unjoined and can
be modified. The joined child curve will update
along with any dependent geometry.
Where is this Command:
Access this command from Tools>Join History
Getting Started:
The image file should have a good contrast of
colors - not just one or similar tones. The better
the contrast along with the fewest amount of
colors will produce the best results. Choose image
files in the following formats: PNG, JPG GIF, JPEG,
BMP, and TIFF.
### Options:
Steps:
Threshold: Threshold helps define which pixels
• Run the Join History command.
• Select the curves to Join History.
• A Join History curve appears on top of the original
curve(s). (See picture below.)
Note: Using the traditional Join command breaks
history.  Use the Join History to create a single
history enabled curve.
are light and which are dark. It converts
grayscale images into high-contrast,
black and white images adding
definition to the traced image. For light
images increase the Threshold value.
For darker images lower the Threshold. Adjust in
the builder by clicking on the Threshold icon at
the left and running the slider below.
Despeckle: Despeckle finds the small curves
under a specific value and deletes
them. Increase the value to remove
larger curves. Adjust in the builder by
clicking on the Despeckle icon at the
left and running the slider below.
Corner Soften: This will fillet or blend sharp
corners. Use this or Blur to find the best
solution for softening the corners.
