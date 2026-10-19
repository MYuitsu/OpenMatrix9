# Cross Section Profiles

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 167-168
- Section title: Cross Section Profiles
- Section id: m8book1-185-cross-section-profiles
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p167_img01.png, m8book1_p167_img02.png, m8book1_p168_img01.png, m8book1_p168_img02.png, m8book1_p168_img03.png

---
<!-- section m8book1-185-cross-section-profiles | Cross Section Profiles | m8book1 pages 167-168 -->

<!-- page 167 -->

Steps:
Note: The cursor is constrained to the surface
selected at the beginning of the command. Use
O-Snaps to precisely locate the extracted curve.
• Select Continue Control Point Curve from the
Curve menu.
• Select an open curve near an end.
• Pick the next points.
• Press Enter to end the curve.
Extract a curve on a surface
at a specified.
### Command Line Options:
### Command Line Options:
Angle: Set the angle of the curve that will be
extracted on the surface.
PersistentClose:  Closes the curve after
pressing Enter. The curve updates the shape when
closed.
## Cross Section Profiles
Close: Closes the curve smoothly, creating a
periodic curve.
Sharp: Closes the curve with a kink, creating a
non-periodic curve.
‘CSec’
Create cross-section curves through several
different (parallel) curves known as “Profiles” in this
command.
Undo: Reverses the last action.
Where is this Command:
Access this command from Curve>Cross
Section Profiles
## Extract Angle Isocurve
Steps:
‘gvExtractIsocurve’
Extracts a curve on a surface at a specified angle.
Where is this Command:
Access this command from Curve>Extract
Angle Isocurve
Steps:
• Select Extract Angle Isocurve from the Curve
menu.
• Select a surface.
• Specify an angle in the Command Line. (Use
hotkeys A & D to modify the angle visually).
• Pick a place to extract curve on the selected
surface.
• Using any curve creation command, draw “profile
curves” that roughly define the shape of your
object.
• Select the “profiles” in either clockwise or
counterclockwise order..
• Pick the start of a line that is perpendicular to the
current “profile curves”.
• Pick the end of the line (perpendicular to the
current “profile curves”.
• In the plane defined at the cross-section line, a
curve will be created that passes through the
profile curves, and press Enter. Once you have
made the cross-section curves, you can use the
Loft or other command to create a surface
through them.

<!-- page 168 -->

Planar: Creates a planar curve.
OnSurface: Sketches a curve on a surface.
OnMesh: Sketches a curve on a polygon mesh.
## Sketch on Surface
The yellow curves indicate the profiles and the green
curves are the cross sections.
### Command Line Options:
‘Sketch O’
Draws a curve on a surface by dragging the
mouse while depressing the left mouse button.
Closed:  Yes, creates closed curves. No, the
curves start at the first profile line and end at the
last profile line.
Where is this Command:
Access this command from Curve>Sketch on
Surface
Steps:
Notes: The cross section lines must cross all of the
profile curves. For best results, space the cross
section curves relatively evenly.
## Sketch
• Select Sketch from the Curve menu.
• Select the surface to Sketch on.
• Click and hold the left mouse button and drag the
mouse to sketch.
• Release the mouse button to end the curve.
‘Sketch’
Draws a curve by dragging the mouse while
depressing the left mouse button.
Note: Drawing a single curve in multiple viewports is
allowed using this method.
Where is this Command:
Access this command from Curve>Sketch
Steps:
• Select Sketch from the Curve menu.
• Click and hold the left mouse button and drag the
mouse to sketch.
• Release the mouse button to end the curve.
Note: Drawing a single curve in multiple viewports is
allowed using this method.
### Command Line Options:
Closed=No: Creates an open curve
### Command Line Options:
Closed=Yes: Creates a closed curve. Select the
option before dragging points with the mouse.
The curve closes when you stop dragging points.
Closed: Creates a closed curve. Select the
option before dragging points with the mouse.
The curve closes when you stop dragging points.
