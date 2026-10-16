# Fillet

Key **F**.

![The radius grows: the fillet eats material while the overall size of the part stays the same.](img/part-fillet/)

## How to do it

1. Select edges — a click on an edge; a click on a face takes all its edges. Edges selected before the command are
   taken at once; you can also press **Fillet** first and click the edges.
2. Type the **Radius** into the field at the geometry. The preview shows the surface of the fillet on the part before
   it is applied.
3. **Enter** applies, **Esc** cancels.

**A different radius at the ends**: with the tool in hand, click a corner (a vertex) on a picked edge — a radius field
of its own appears at the corner. The radius varies along the edge from corner to corner.

A fillet holds on to the edge, not to its number: change the extrusion height or move a wall — the fillet stays on
the same edge.

## If it did not work

- **The radius is refused before Enter** — the program tries the fillet in advance and writes at the field why it
  fails: the radius is more than the geometry allows. The analysis goes edge by edge: which one takes a radius no
  larger than so much, which one takes none. Make the radius smaller or drop that edge.
- **The node is yellow, some edges stayed sharp** — those edges could not be taken, the rest are rounded; the warning
  says how many. Open the node with a double-click and pick other edges or another radius.
- Two neighbouring fillets cross — such edges are taken one at a time only. Round them in separate operations.

## Order matters

A fillet before a cut and the same fillet after it are different shapes. Fillets usually go last, once the main shape
is ready; draft goes before fillets.

## See also

- [Chamfer](part/06-chamfer) — the same edge, but a flat cut.
- [The timeline and rollback](general/03-timeline) — why fillets go at the end.
