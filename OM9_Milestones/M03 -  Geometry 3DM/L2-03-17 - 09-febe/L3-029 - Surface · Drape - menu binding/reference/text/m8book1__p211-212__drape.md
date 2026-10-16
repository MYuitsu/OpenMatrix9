# Drape ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 211-212
- Section title: Drape ￼
- Section id: m8book1-229-drape
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p211_img01.png, m8book1_p211_img02.png, m8book1_p212_img01.png, m8book1_p212_img02.png, m8book1_p212_img03.png

---
<!-- section m8book1-229-drape | Drape ￼ | m8book1 pages 211-212 -->

<!-- page 211 -->

Where is this Command:
Access this command from Surface>Drape
Steps:
Match Edges by Closest Points: Aligns the
surface being changed to the target edge in two
ways: Stretches or compresses the surface to
match the entire edge end to end, or pulls each
point object on the edge to the closest point
object on the other edge.
• Select Drape from the Surface menu.
• Drag a window (rectangle) over the area to Drape.
A surface is created that drapes over the objects.
Refine Match: Determines if the match results
should be tested for accuracy and refined so the
faces match within tolerance.
Note:
• Drape works over meshes, surfaces, and solids.
• Drape does not create a UV surface and cannot be
used with Create and Apply UV curves.
Distance ‘X’ units: Position matching in model
units.
Tangency ‘X’ degrees: Tangency matching.
Curvature ‘X’  percent: Curvature matching, in
percent of the radius of curvature.
### Isocurve Direction Adjustment:
Specifies the way the parameterization of the
matched surfaces is determined.
Mesh object before using Drape.
(Display=Shiny Plastic)
Automatic: If the target edge is not trimmed,
it works like the Match target isocurve direction
option. If the target edge is trimmed, it works like
the Make perpendicular to the target edge option.
Preserve Isocurve Direction: Does not
change the existing isoparametric curves
directions.
Match Target Isocurve Direction: The
isoparametric curves of the surface will be parallel
to those of the target surface.
The NURBs object created with Drape over a mesh
object. Point Spacing Density was set at ‘1’.
(Display= Tech White)
Make Perpendicular to Target Edge: The
isoparametric curves of the surface will be
perpendicular to the target surface edge.
### Command Line Options:
## Drape
AutoSpacing: Yes/No: Yes; all points within the
draped surface are evenly-spaced at a distance
set by the Spacing option.
‘Drape’
Places a NURBs surface over any objects: meshes,
surfaces and solids. The surface follows the shape
of the objects that literally ‘drapes’ over these
objects.
Spacing: Sets the control points spacing value.
Controls custom spacing for the points. The lower
the number, the more dense the surface will be.

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
