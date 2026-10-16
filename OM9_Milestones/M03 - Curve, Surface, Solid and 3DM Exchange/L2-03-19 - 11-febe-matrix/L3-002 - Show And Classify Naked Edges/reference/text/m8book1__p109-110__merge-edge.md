# Merge Edge ￼

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 109-110
- Section title: Merge Edge ￼
- Section id: m8book1-106-merge-edge
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p106_img01.png, m8book1_p109_img01.png, m8book1_p110_img01.png

---
<!-- section m8book1-106-merge-edge | Merge Edge ￼ | m8book1 pages 109-110 -->

<!-- page 109 -->

### Menu Options:
AllEdges: Displays all surface and polysurface
edges
Previous: Moves to the previous naked or non-
manifold edge on the model.
Mark: Adds a point representing each naked or
non-manifold edge.
Naked Edges: Displays any open edges (naked)
in surfaces or polysurfaces.
## Split Edge
‘SplitEdge’
Split Edge divides a surface edge at designated
locations.
Where is this Command:
Access this command from Utilities>Split Edge
Steps:
• Select Split Edge from the Utilities menu.
• Select a surface edge.
• Pick locations along the
edge to split.
Non-Manifold Edges: Displays edges of
polysurfaces, meshes that have more than one
face joined to a single edge.
Note: Use the
Show Edges
command to
display the
surface edge
and its end
points.
Zoom: Magnifies the view to either a Naked or
Non-Manifold edge. See options below.
Edge Color: Set the display color for the edges.
## Merge Edge
Add Objects: Add objects to the edge display.
Remove Objects: Remove objects from the
edge display.
‘MergeEdge’
The Merge Edge command will join adjacent
edges of the same simple surface into one edge.
The edges must be naked (open), must belong
to the same surface, must share an endpoint, and
must meet smoothly at the shared endpoint.
### Command Line Options:
Where is this Command:
Access this command from Utilities>Merge
Edge
Steps:
All: Repositions the active viewport so that all
naked or non-manifold edges are visible in the
viewport.
Current: Repositions the active viewport to the
first in the list of naked or non-manifold edge on
the model.
• Select Merge Edge from the Utilities menu.
• Select a naked edge on a surface or a polysurface.
• If possible, the edge will merge to an adjacent
Next: Moves to the next naked or non-manifold
edge on the model.

<!-- page 110 -->

each other (within tolerance). If not, you may have
some problems later on with the model.
edge of the same simple surface into one edge.
Otherwise “The edge is already merged” appears
at the Command Line, and the prompt repeats.
• Select an adjacent edge.
Notes: Use Show Edges > Naked Edges to display
edges and their ends. Use Merge Edge when
complex polysurfaces have been unjoined and Join
will not work properly.
If you cannot join surfaces using this command,
you should either make the surfaces a little more
accurate or your tolerance higher. If the edges are
too far out of line, no join will occur and “Unable
to find overlapping intervals” will appear at the
Command Line.
## Join 2 Naked Edges
## Unjoin Edge
‘Join Edges’
Joins two naked edges that are out of tolerance
to join.
‘UnjoinEdge’
Unjoin Edge will unjoin selected polysurface
edges.
Where is this Command:
Access this command from Utilities>Join 2
Naked Edges
Where is this Command:
Access this command from Utilities>UnJoin
Edge
Steps:
Steps:
• Select Join 2 Naked Edges from the Utilities menu.
• Select two naked surface edges that are close
together. (Use Show Edges command to help
identify naked edges)
• Select the Unjoin Edge command from the Utilities
menu.
• Select polysurface edges to unjoin. Press enter
when done.
Note: Seams in closed surfaces will not unjoin.
## Make 2D Drawing
Note: If the edges overlap (run somewhat parallel)
along at least part of their length (an interval),
but are not coincident, the Edge Joining dialog
box reports, “Joining these edges requires a join
tolerance of <distance>. Do you want to join these
edges?”. The surfaces will extend to join along the
intervals.
‘Make2D’
The Make 2D Drawing command takes the
selected objects in the viewports and makes
a 2-dimensional drawing of them. The most
common use of this button is to create a classic
3-view drawing with an included isometric view.
In the Make 2D Drawing menu, this can be done
using the 4 view USA option. This command
creates curves from the silhouette of the model.
### Warnings
Where is this Command:
Access this command from Utilities>Make 2D
Drawing
Steps:
The Join 2 Naked Edges command is a tolerance
override. It will “join” edges no matter how far apart
they are. Joining has to do with topology (what is
listed as being connected) rather than geometry
(where the parts are in relation to each other). It
works fine if the surface edges are pretty close to
• Select Make 2D Drawing from the Utilities menu.
• Select the object(s) in the viewport to be drawn in
2D.
