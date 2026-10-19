# Diagnostics

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 96-97
- Section title: Diagnostics
- Section id: rhino5-061-diagnostics
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p96_img01.png

---
<!-- section rhino5-061-diagnostics | Diagnostics | rhino5 pages 96-97 -->

<!-- page 96 -->

Curve and Surface Analysis
### Edge evaluation
Geometry problems such as Boolean or join failures can be caused by edges on surfaces that have become
broken or edges between surfaces that have been moved through point editing so they create holes. An edge is
a separate object that is part of the surface’s boundary representation.
The ShowEdges command highlights all the edges of the surface.
A polysurface may look closed, but the Properties command may tell you that it is open. Some operations and
export features require closed polysurfaces, and a model using closed polysurfaces is generally higher quality
than one with small cracks and slivers.
Rhino provides a tool for finding the unjoined or “naked” edges. When a surface is not joined to another
surface, it has naked edges. Use Properties command to examine the object details. A polysurface that has
naked edges lists as an open polysurface. Use the ShowEdges command to display the unjoined edges.
Other edge tools let you split an edge, merge edges that meet end-to-end, or force surfaces with naked edges
to join. You can rebuild edges based on internal tolerances. Other edge tools include:
●
SplitEdge splits an edge at a point.
●
MergeEdge merges edges that meet end to end.
●
JoinEdge forces unjoined (naked) edges to join nearby surfaces.
●
RebuildEdges redistributes edge control points based on internal tolerances.
### Diagnostics
Diagnostic tools report on an object’s internal data structure and select objects that may need repair. The
output from the List, Check, SelBadObjects, and Audit3dmFile commands is normally most useful to a
Rhino programmer to diagnose problems with surfaces that are causing errors.

<!-- page 97 -->

## Organization and Annotation
Rhino offers aids to organizing your work:
●
Layers
●
Groups
●
Blocks
●
Worksessions
Each method offers a different approach to model organization. Using layers lets you assign a layer designation
to objects. Groups associate objects so they can be selected as one. Blocks let you store and update an
association of objects. Worksessions let you work on a part of a project while using other models in the project
as references.
Rhino also provides the ability to add notation to your model. These appear as objects in the model.
●
Dimensions
●
Leaders
●
Text blocks
A different form of notation always displays facing towards the view plane.
●
Annotation dots
●
Arrowheads
In addition, you can add Notes to the model. Notes do not appear in the model, but display in a separate
window.
### Layers
Layers are a way of grouping objects and applying certain characteristics to all objects that have that layer
assignment. There are two “mental models” you can use when you think of layers—they can be thought of
either as a “storage location” for the objects or as a way to assign a set of characteristics or properties to
objects.
Layer states include a layer name, the color used to display the objects, and the on/off and locked/unlocked
status of all the objects on a layer. Objects on layers that are off are not visible in the model. Objects on locked
layers cannot be selected but can be snapped to. Objects are always created on the current layer. This layer
assignment can be changed later.
To accomplish the most common tasks related to layers, click the Layer pane in the status bar to display the
popup layer list. You can set the current layer; change the on/off, locked/unlocked state; and the layer color. In
addition, right-click the layer name to create a new layer, rename a layer, delete the selected layer, select
objects on the selected layer, change objects to the selected layer, and copy objects to the selected layer.
Accomplish more detailed layer management with the Layers panel. Right-click the Layer pane to open the
Layers panel. The Layers panel sets the current layer, locks and unlocks layers, turns layers on and off,
changes the layer color and sets the layer render material. You can create new layers, delete layers, move
layers up or down in the layer list, filter the layer list, set the current layer to match an object in the model,
change objects to a selected layer, select all layers, and invert the selection.
The SelLayer command selects all objects on a layer.
