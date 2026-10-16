---
source_pdf: h:\OpenMatrix9\manuals\matrix_8_book_2.pdf
source_label: Matrix 8 Instruction Manual Book 2
page_range: 143-144
section_title: Ring Resizer ￼
section_id: m8book2-146-ring-resizer
shared_start_page: True
shared_end_page: True
---

# Ring Resizer ￼

Source: matrix_8_book_2.pdf, pages 143-144

<!-- section m8book2-146-ring-resizer | Ring Resizer ￼ | m8book2 pages 143-144 -->

<!-- page 143 -->

Note: Use Object Materials will only display the
weight representing the applied VRay material on
that specific object.
## Ring Resizer
Once your design is complete, use Ring Resizer to
create various sizes of the same design. This is not
history-related; it simply resizes the shank.
Where is this Command:
Access this command from Tools>Ring Resizer
• Choose output sizes. Hold down the
‘Control’ key on the keyboard to
select multiple sizes.
• Set remaining options and click on
“Resize.”
Note: Save your ring prior to using Ring Resizer as it
produces a mesh output. Resizer requires a valid ring
rail of the original size. Exclude any gems in the input
box. Also, make sure your object is valid and closed
for the best mesh outcome.
### Options:
Steps:
Symmetrical: Mirrors the Left
angle curve and uses the same
angle for the Right angle. Toggle
this option Off if your ring is not
symmetrical. This is ON by default.
Add Rings: Adds mesh copies
of your resized rings to the viewport so you can
inspect or save them. This is ON by default.
Save to File: Saves each ring as its own .3dm file
in the file’s location, using the file name specified
in the file name field.
STL Options: Allows you to change the mesh
output settings of the .STL generated for each
ring. For instructions on Mesh options see Mesh
From NURBS in the Utilities menu.
• Select Ring Resizer from the Tools menu.
• Select the ring model (just the metal
objects and ring rail - minus any
gems) and input the model into the
model box in the Ring Resizer. The
ring rail size will appear
in the “Original Ring
Size” box.
• Set the primary radius of the ring-use the circle in
the builder and in the Through Finger view draw
a circle that intersects where the girdle of the
primary or center gemstone would lay.
• Set the Left/Right angle of the ring. Click on the
Line in the builder, this will set your start point of
your curve at F4. Then in Through Finger View set
the end point of the line so that the line extends
just past the head detail of the ring (center and
side gems).

<!-- page 144 -->

the input curve.. Adjust in the builder by selecting
the Extrude Distance icon at the left, running the
slider below-or typing a length in mm the box.
Original Ring with three mesh outputs.
Left: Curve on Surface       Right: Insert complete
## Surface Pullback
Create inset areas in jewelry designs, such as the
side shield on a signet ring, by inputting a closed
curve on a surface into this tool.
Where is this Command:
Access this command from Tools>Surface
Pullback
Option with Cap
Note: This command requires a closed curve on the
surface of an object. Use your curve commands such
as Pullback, Project or Extract IsoCurve to get the
curve on the surface of the model.
### Options:
Steps:
• Select a surface and input
Bevel Edge: Adds an additional Extrude
Distance slider to the builder. Allowing you to
create a tapered inset.
Insert Only: Only create the insert
in the selected surface.
into the surface box.
• Select a curve and input
into the curve box.
• Adjust the settings in the
builder.
• Click Apply.
Cap: Creates an insert and a
corresponding cap.
### Editing Dimensions:
Drop Depth: Controls the overall depth of the
Apply: Creates the inset as specified in the
builder.
Select: Selects the new objects that are the
results of the command.
inset. Adjust in the builder by selecting
the Drop Depth icon at the left, running
the slider below-or typing a length in
mm the box.
Extrude Distance: This option only appears
Undo: Reverts the object to the state before the
surface pullback command was run.
when Bevel Edge is selected. This
affects the taper of the inset by making
the inset cutter extrude to a point-this
controls the distance of the point from
