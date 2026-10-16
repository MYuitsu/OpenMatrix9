# Metal Weights ￼

- Source PDF: matrix_8_book_2.pdf (Matrix 8 Instruction Manual Book 2)
- Page range: 142-143
- Section title: Metal Weights ￼
- Section id: m8book2-145-metal-weights
- Note: boundary page(s) are shared with the neighbouring section (two-column layout); first/last page may contain text of the adjacent section.

---


<!-- page 142 -->

## Metal Weights
Rotate Y: Rotate the profile along the Y axis
using the white Rotate Y Viewport Control Handle.
This works similar to Roll in Profile Placer except
the profile’s bounding box will never leave the rails.
Calculate the weight of your design using the
Object’s assigned render materials or display all
the metal types available.
### Options:
Profile: Open the profile library by
Where is this Command:
Access this command from Tools>Metal
Weights
Steps:
clicking on the Image in the Profile
box or clicking on Profile in the
Command Line. Like with most
profiles libraries you can modify or
add to this profile library
• Select Metal Weights from the Tools Menu.
• Select the solid objects you want to weigh and
select “Calculate” in the builder.
Edit Profile: Click on the Edit button below
the Profile window to bring up the Profile Editor.
See Profile Placer Editor to learn how to use this
function.
Metal Weights will generate a report of the
estimate weights in Grams and Pennyweights
(DWT).
Set Plane: Use this option to
define the plane through which the profile
intersects the rails. Click once below the bottom
rails and click again above the top rails in the
Through Finger viewport. The second click
represents the direction you wish the profile to be
angled.
Add Profile: Adds an additional
Note: Objects must be watertight solids to perform
metal weight calculations. The resulting calculations
are representative of the volume of each individual
part.
profile to the rails. This is added on top of the
existing profile and can be positioned by using the
Viewport Control Handles or the Set Plane option.
(Press Enter on the keyboard to complete the
command and to add your final profile.)
### Options:
GV: With this toggle on all of the
Sweep 4: This runs the 4 Rail Profile Sweep
command. This command is also found in the
Surface menu. You will be prompted to select only
the Profiles connected to the four rails.
default Gemvision Materials will
be displayed in Metal Weights
User: With this toggle on Materials added as User
Materials in VRay Styles to be displayed in Metal
Weights.
Use Object Materials: With this option toggled
Metal Weights will only present the weights for
VRay materials that have been applied to the
selected object. Apply materials using VRay Styles
or via the Material Mode
in F6.

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
