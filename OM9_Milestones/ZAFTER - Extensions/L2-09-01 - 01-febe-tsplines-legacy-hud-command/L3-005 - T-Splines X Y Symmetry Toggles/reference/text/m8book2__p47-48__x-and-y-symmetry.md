# X and Y Symmetry ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 47-48
- Section title: X and Y Symmetry ￼
- Section id: m8book2-051-x-and-y-symmetry
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-051-x-and-y-symmetry | X and Y Symmetry ￼ | m8book2 pages 47-48 -->

<!-- page 47 -->

### (3) Select Face Edge Vert:
3Point: Allows you to determine the Symmetry
axis by selecting 3 points.
Steps:
### Radial Symmetry:
Creates symmetry around a circle by placing a
center of rotation.
Steps:
In case the “root face” method gets mixed up,
this provides more information on where the
symmetry line should be. Select root face. If the
two sides of your model have different shapes,
then click any face on the side of the model you
want to keep as the “root face.” Select symmetric
face. Pick the face you want to be symmetric to
the root face on the other side of the model.
Steps:
• Select the T-Spline object to add symmetry to.
• Select Add in the Command Line.
• Select the type of symmetry to apply (Radial).
• Select the center of rotation.
• Click to set a point for the angle or first reference.
• Click to set a point for the second reference.
### Command Line Options:
• Select root edge. Select any edge on the root face.
• Select symmetric edge. Select the corresponding
edge on the symmetric face.
• Select root vert. Select a vertex on the root edge.
• Select symmetric vert. Select the corresponding
vertex on the symmetric edge.
Segments: Determines the number of radial
portions of the symmetry.
### Adding Symmetry to complete a partial
### model:
Steps:
Weld: Determines whether to weld the vertices
on the borders of the model or not. Weld=No will
combine the parts into one object but not weld
the vertices.
Tolerance: If Weld=Yes, all vertices on the
symmetry border will be welded if they fall within
this tolerance.
• Select the Symmetry type: Axial (two symmetric
halves), or Radial (multiple symmetric parts
around a circle).
• Apply Symmetry. Axial and Radial symmetry each
have their own procedures for doing this. See
below.
## X and Y Symmetry
### Axial Symmetry (along X, Y or Z axis):
Steps:
‘gvtsMirrorXY’
This Gemvision command is a Symmetry short cut
which adds Symmetry to a model across both the
X and Y axis.
• Start of Symmetry Plane: click a point on the start
of the plane. It is helpful to have the Point O-Snap
turned on for this step.
• End of Symmetry Plane: click a point on the end of
the plane.
Where is this Command:
Access this command from T-Spline>TSplines
HUD> X and Y Symmetry
### Command Line Options:
Weld: Determines whether to weld the vertices
on the borders of the model. Weld=No will
combine the parts into one object but not weld
the vertices.
Steps:
• Center the T-Spline object at F4.
• Delete the faces from 3 of the 4 quadrants (X,Y)
(X,-Y) (-X,Y) (-X,-Y).
• Select X and Y Symmetry from the HUD menu.
• Select the T-Splines object.
• Press Enter when done.
Tolerance: If Weld=Yes, all vertices on the
symmetry border will be welded if they fall within
this tolerance.

<!-- page 48 -->

## X Symmetry
‘gvtsMirrorX’
A symmetry short cut which adds symmetry to a
model across the X axis.
Note: The object in one quadrant will mirror in X and
Y axis and become Symmetric. This command only
gives Symmetric results. Also, if the object’s verts
do not fall along the axis in which you are mirroring,
you may end up with odd overlaps and a model that
is not solid or has holes in it. The Set Points tool is
great for ensuring the item will line up perfectly prior
to mirroring. Refer to T-Spline>Set Points to better
understand how to use this tool
Where is this Command:
Access this command from T-Spline>TSplines
HUD> X Symmetry
Steps:
• Center the T-Spline object at F4.
• Delete the faces from the X or -X quadrant.
• Click on X Symmetry in the HUD menu.
• Select a T-Spline object.
• Press Enter when done.
Object is now Symmetric across the X axis.
Object with X and Y Symmetry. Use Append to add
faces to close both sides.
## Y Symmetry
Object is
Symmetric
across the X
axis. (X+ is
mirrored to X-)
‘gvtsMirrorY’
A symmetry short cut which adds Symmetry
to a model across the Y axis. This creates a
symmetrical relationship between the top and
bottom of the object in the Looking Down
viewport.
## Box/Smooth Toggle
‘tsSmoothToggle’
Toggles the T-Splines surface between Smooth
Mode and Box Mode.
Where is this Command:
Access this command from T-Spline>TSplines
HUD> Y Symmetry
Steps:
Where is this Command:
Access this command from TSpline >TSplines
HUD>Box/Smooth Toggle
Hot Key: TAB
Steps:
Start with a complete model that doesn’t have
Symmetry.
• Center the T-Spline object at F4
• Delete the faces from the Y or -Y quadrants.
• Select Y Symmetry from the HUD menu.
• Select the T-Splines object.
• Press Enter when done.
• Select a T-Spline object.
• Click on the Box/Smooth Toggle.
Object is now Symmetric across the Y axis.
Object is
Symmetric
across the
Y axis. (Y+
is mirrored
to Y-)
Sphere in Box Mode (left) and Smooth Mode (right)


---

## Images

- ![m8book2_p48_img01.png](../images/m8book2_p48_img01.png) page 48 (shared page)
- ![m8book2_p48_img02.png](../images/m8book2_p48_img02.png) page 48 (shared page)
- ![m8book2_p48_img03.png](../images/m8book2_p48_img03.png) page 48 (shared page)
- ![m8book2_p48_img04.png](../images/m8book2_p48_img04.png) page 48 (shared page)
