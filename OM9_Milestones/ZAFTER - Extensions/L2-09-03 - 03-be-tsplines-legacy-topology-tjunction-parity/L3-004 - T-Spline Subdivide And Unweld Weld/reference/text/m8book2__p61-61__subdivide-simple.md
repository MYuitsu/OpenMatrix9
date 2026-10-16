# Subdivide Simple ￼

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 61-61
- Section title: Subdivide Simple ￼
- Section id: m8book2-075-subdivide-simple
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-075-subdivide-simple | Subdivide Simple ￼ | m8book2 pages 61-61 -->

<!-- page 61 -->

### History:
tool can be very useful to add detail on flat faces,
by quickly subdividing the face and getting more
control points.
Where is this Command:
Access this command from T-Splines>
Subdivide Simple
Steps:
This command works with Rhino’s History. To
use history, turn on Record History, and Lock
Children. Then, after you create the T-Spline, you
can move the curves, or the control points on the
curves, and the surface will update. If you change
the angle of the curve intersection, then the joints
will update to something that better fits that
angle. Once you Unlock Children and move the
T-Splines control points, then History is broken.
## Fill Hole
• Select Subdivide Simple from the T-Spline menu.
• When prompted select the T-Spline face or faces
to be subdivided.
• Press Enter when done.
Each selected T-Spline Face is divided into 4
T-Spline faces.
‘tsFillHole’
The Fill Hole command is a quick and easy way to
fill any open gap in a T-Spline model. Use this or
Append Face to create a face where a hole exists.
Where is this Command:
Access this command from T-Spline>Fill Hole
Steps:
• Select Fill Hole from the T-Spline menu.
• Select an edge along a hole on a T-Spline object.
Or, region select an entire object with multiple
holes to fill.
T-Spline Face before and after Subdivide Simple.
Note: Fill Hole can create undesirable surfaces at
times. Use Append on oddly shaped holes if needed.
### Command Line Options:
Insertion Type = Exact/ Simple: Exact:
Subdividing a face in exact mode will maintain
the surface shape. Simple: As the faces are
subdivided and multiplied, the T-Spline surface
changes to accommodate the newly-added
edges and verts.
Fill Hole before and after using Region Select to
select the T-Splines object to fill multiple holes at
once.
RetopoSnap=(Yes/No):  Turns on and off
RetopoSnap.
## Subdivide Simple
RetopoOffset: When Retopo Snap is active
this option appears. The offset distance from the
mesh that each point will snap to. If 0, the control
points will snap right on the mesh.
‘tsSubdivideFaceSimple’
Create more control over the object by adding
more Faces to a T-Spline surface. Subdividing
a face in simple mode will change the surface
making it flatter and sharper (more isocurves). This


---

## Images

- ![m8book2_p61_img01.png](../images/m8book2_p61_img01.png) page 61 (shared page)
- ![m8book2_p61_img02.png](../images/m8book2_p61_img02.png) page 61 (shared page)
