# Join History

- Source PDF: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
- Page range: 149-149
- Section title: Join History
- Section id: m8book2-154-join-history

---

<!-- section m8book2-154-join-history | Join History | m8book2 pages 149-149 -->

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
