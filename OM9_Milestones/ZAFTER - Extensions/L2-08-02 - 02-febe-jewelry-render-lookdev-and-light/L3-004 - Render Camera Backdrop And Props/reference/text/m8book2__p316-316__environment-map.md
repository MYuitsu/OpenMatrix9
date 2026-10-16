# Environment Map

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 316-316
- Section title: Environment Map
- Section id: m8book2-260-environment-map
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-260-environment-map | Environment Map | m8book2 pages 316-316 -->

<!-- page 316 -->

## Environment Map
for fewer or more polygons. Or, use the detailed
mesh options.
Add or Remove Objects: Select additional
objects or select objects to remove for
environment map analysis.
‘EMap’
This command visually evaluates surface
smoothness using an image bitmap reflected in
the surface.
## Apply Edge Softening
Where is this tool:
Access this tool from Render>Environment
Map
Steps:
‘ApplyEdgeSoftening’
Constructs an edge-softening display mesh for
surfaces, polysurfaces, or meshes.
• Select Environment Map from the Render menu.
• Select objects for environment map analysis:
• See the Command Line prompt. After the objects
are selected an options dialog box will appear.
Note: After running this command, Edge Softening
appears as a property for the surface.
Edge softening applies a mesh radius or chamfer to
edges in the model.
Steps
• Select Apply Edge Softening from the Render
menu.
• Select objects to apply edge softening to.
• Specify Command Line options.
• Box object (left) and box with edge softening
applied (right).
### Command-line options:
RemoveAll: After edge softening is applied,
removes edge softening from all objects.
Edge Softening Properties: Manages surface
edge softening properties for the selected
objects.
Selection box or Browse: Choose from a
preset list (click on the drop down arrow) of
environment map images or choose “Browse” to
select your own.
On: Enables edges softening.
### Settings:
Blend with Object: This blends the bitmap with
the render color of the object, which
lets you simulate different materials with the
environment map.
Softening: Specifies a softening amount. The
visual radius approximation depends on the
density of the underlying mesh.
Show Isocurve: This displays the object’s
isocurves even while shaded.
Chamfer: The edge will be cut straight across
with a hard edged chamfer. No attempt is made to
soften this into a rounded edge.
Adjust Mesh: When using Environment Map,
an invisible mesh will be created based on the
settings in the Mesh Options dialog box if the
object does not have a surface analysis mesh. Use
this dialog box Simple controls to move the slider
Faceted: The object, including the edge
softening effect will be shaded as faceted as in
flat shade.


---

## Images

- ![m8book2_p316_img01.png](../images/m8book2_p316_img01.png) page 316 (shared page)
