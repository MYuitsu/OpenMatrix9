# Edge evaluation

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 96-96
- Section title: Edge evaluation
- Section id: rhino5-060-edge-evaluation
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p96_img01.png

---
<!-- section rhino5-060-edge-evaluation | Edge evaluation | rhino5 pages 96-96 -->

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
