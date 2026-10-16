# Sweep Multi ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 212-213
- Section title: Sweep Multi ￼
- Section id: m8book1-230-sweep-multi
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p212_img01.png, m8book1_p212_img02.png, m8book1_p212_img03.png, m8book1_p213_img01.png, m8book1_p213_img02.png, m8book1_p213_img03.png

---
<!-- section m8book1-230-sweep-multi | Sweep Multi ￼ | m8book1 pages 212-213 -->

<!-- page 212 -->

Note:
AutoDetectMaxDepth: Yes/No: Yes; stops
the draped surface at what is automatically
determined to be the farthest visible point within
the rectangle. No; controls the custom depth
setting.
MaxDepth: Sets the maximum depth for the
draped surface when Auto Detect is set to OFF.
This can be farther away from (1.0) and/or closer
to the camera (0.0), providing complete or partial
coverage of an object.
## Sweep Multi
• At least two and up to four rails can be used in
Sweep Multi.
• At least two profiles are needed.
• This tool places additional invisible “rails” in the
model to sweep in a more controlled way from
one profile to the next using the Sweep Edit
Points of each profile.
• Edit the Sweep Edit Points to control how the
Sweep surfaces are made. Click and drag one
point and the mirrored partner will move too.
See Tools menu>Profile Placer>Edit Sweep Edit
Points for complete instructions.
• Edit a rail or profile and see History update the
created Sweep Multi surfaces.
‘gvfourSweep’
History-enabled Sweep Multi controls the shape
of the surface using the Sweep Edit Points of the
profiles. Use this in place of Sweep 1 or 2 History
when extra control is needed.
Where is this Command:
Access this command from Surface>Sweep
Multi
Steps:
Sweep Edit Points located on the Profile curves are
used as guides in Sweep Multi. Adjust these points to
change the way the Sweep surface is created.
Note: See Tools menu>Profile Placer>Edit Sweep Edit
Points for complete instructions.
• Select Sweep Multi from the Surface menu.
• Select the rails curves (2, 3 or 4 rails are needed).
• Press Enter when done.
• Select profiles (2 or more).
• Press Enter when done.
## Heightfield from Image
‘Heightfield’
Creates a NURBS surface or mesh based on
grayscale values of the colors in an image file.
Where is this Command:
Access this command from Surface>Heightfield
Steps:
The Profile’s Sweep Edit points are moved inwards.
The Sweep flows from these points in Sweep Multi.
• Select Heightfield from Image from the Surface
menu.
• Browse to an image file and Open it.
• Pick the first corner location in the viewport. The
base of the surface will be parallel to the current
Construction plane.
• Pick the second corner or type a length. (Hold
Shift key) The shape of the pick rectangle will
match the aspect ratio of the bitmap file and be
the size of the resulting surface.
• Specify options in the Heightfield dialog box.
Sweep Multi surfaces made with four rail curves and
two profiles. (Sweep Edit Points are in the original
locations.)

<!-- page 213 -->

Number of Sample Points ‘U’ x ‘V’: The
number of points indicates the density of the
surface in the U and V directions. Enter a higher
point count for more detail, and lower for less
detail.
Height ‘X’ mm: Sets the height of the surface in
the Z direction relative to the X and Y directions
selected when placing the image in the viewport.
Set Image as Texture: Uses the image as a
render texture for the created object.
Create Vertex Colors (Mesh only): Evaluates
the color of the texture at each texture coordinate
(U and V) and sets the vertex color to match.
Use a high-quality image for best results.
Hint:
• Use a high-quality image for the best results. There
are no adjustments for its quality the way there
are in Matrix Art.
• The number of sample points (U and V counts)
controls the detail outcome of the resulting
surface (see below).
• The created surface matches the contours of the
bitmap or image file. Dark colors are low, and light
colors are high.
### Heightfield Options:
Resulting surfaces using 50 U and 50 V counts.
‘Surface with control points at sample locations’
(left) and ‘Interpolate surface through samples’
(right). Experiment between the options
for the best results.
### Create Object By:
Mesh with vertexes at sample locations:
Creates a mesh with vertex points at each of the U
and V locations.
Surface with control points at sample
locations: Creates a surface with control points
at each of the U and V locations.
Interpolate surface through samples:
Creates a surface that passes through each
sample location’s height.
Hint: Try adjusting these various options to change
the outcome of the tool.
