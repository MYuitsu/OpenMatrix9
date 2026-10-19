# Hole

Key **O**.

![A through hole in the top face: the command remembers the face, not coordinates.](img/part-hole.png)

## How to do it

1. Press **Hole** and click a face — the centre of the hole goes to the point of the click. A second click at another
   point of the same face moves the centre; a click at the same place lets the face go.
2. At the geometry — **Diameter** and **Depth**, and **Shift 1** and **Shift 2 from the face centre** if the position
   is to be given in numbers.
3. **Kind** in the bar above: **Simple**, **Counterbore** or **Countersink**; a counterbore and a countersink add
   **Recess Ø** and **Recess depth**. The preview shows the walls of the hole before it is applied.
4. **Enter** applies, **Esc** cancels.

**Placement -> By a sketch**: click a sketch on the face with isolated points — a hole goes into each. Their positions
are then edited by the sketch's dimensions.

## Why not a cut with a circle

A hole keeps **a reference to the face**: edit the part higher up the timeline, and the hole stays on its face rather
than where the coordinates happened to land.

A through hole is a depth well past the thickness. A thread in a finished hole is cut by [Thread](part/10-thread).

## If it did not work

- The diameter is not taken — the hole is wider than the face; the reason is written at the field.
- **By a sketch** places nothing — the sketch has no isolated points: add them with the **Point** tool.

## See also

- [Thread](part/10-thread) — how to cut it in the hole.
- [Linear pattern](part/17-linear-array) — a row of holes as one operation.
