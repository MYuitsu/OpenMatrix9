# Component patterns and mirrors

A row of identical parts is placed by a pattern, not by copying.

![Four posts at a pitch of 22 mm — one timeline row, not four insertions by hand.](img/assembly-array.png)

## Linear pattern of components

1. Select a part and press **Linear pattern of components** — or press the button and click the part.
2. In the bar above: **Copies** and **Direction** — X, Y or Z. For a grid tick **2nd dir.** (and on top of it
   **3rd dir.**) — each with its own count of copies and axis.
3. The **Pitch** of each direction goes into the fields at the geometry. **Enter** applies, **Esc** cancels.

## Circular pattern of components

1. Select a part and press **Circular pattern of components**.
2. In the bar: **Copies** and **Axis**. By default the axis is the assembly's Z; with the axis button pick a datum
   axis, a straight edge, a cylindrical face or two points in turn. An axis taken from a part follows it on an edit.
3. **Full circle** spreads the copies round the whole circle; without the tick the angle of a sector appears at the
   geometry. **Enter** applies.

## One row instead of a handful of copies

A pattern is **one row** of the assembly timeline: the count and the pitch are edited in one place (a double-click on
the row), and a delete removes all the copies at once. Deleting one copy removes the whole pattern: the pattern leads
its place, it does not live on its own. Editing the source part changes all the copies.

## Mirrored copy

Press **Mirrored copy**, pick a part or a subassembly and point at a plane — XY, XZ, YZ, a datum plane or a face. A
mirrored copy appears: a left hand out of a right one. It can be moved by its gizmo; its shape follows the source —
editing the original carries over to the reflection.
