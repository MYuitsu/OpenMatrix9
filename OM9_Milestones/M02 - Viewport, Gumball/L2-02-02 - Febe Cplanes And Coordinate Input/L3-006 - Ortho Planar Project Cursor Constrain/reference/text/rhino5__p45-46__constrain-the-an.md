# Constrain the angle of movement

- Source PDF: Rhinoceros 5 User's Guide (h:\OpenMatrix9\manuals\rhino5.pdf)
- Page range: 45-46
- Section title: Constrain the angle of movement
- Section id: rhino5-031-constrain-the-angle-of-movement
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section rhino5-031-constrain-the-angle-of-movement | Constrain the angle of movement | rhino5 pages 45-46 -->

<!-- page 45 -->

## Accurate Modeling
The cursor can always move freely in space, but chances are, you will want to relate your modeling elements to
the construction plane grid, existing objects, or coordinates in space. You can restrict the cursor’s movement to
the grid, enter specific distances and angles from a point, snap to specific locations on existing objects, and
enter Cartesian coordinates to locate points in 2-D or 3-D space.
### The Rhino cursor
There are two parts of the cursor: the cursor (1) and the marker (2). The cursor always follow the mouse
movement.
The marker sometimes leaves the center of the cursor because of some constraint on it such as grid snap or
ortho. The marker is a dynamic preview of the point that will be picked when the left mouse button is clicked.
When the marker is constrained, in elevator mode, for example, a tracking line (3) also displays.
Constraints move your marker to a specific point in space or make its movement track according to the
constraint so you can model accurately.
### Snap to the construction plane grid
Grid snap constrains the marker to an imaginary grid that extends infinitely. You can set the snap spacing to
any value.
Click the Grid Snap pane on the status bar to turn grid snap on and off.
Tip: To read more about a command, click the red, underlined command name link.
### Constrain the angle of movement
Ortho mode constrains the marker movement or object dragging to a specific set of angles. By default, this is
parallel to the grid lines, but you can change this. Ortho is similar to the axis lock function found in drawing or
animation programs.
Click the Ortho pane on the status bar to turn ortho on and off. Press and hold the Shift key to temporarily
toggle the ortho mode.
Another common use for ortho is to constrain object dragging to a specific axis.

<!-- page 46 -->

Accurate Modeling
Ortho is active after the first point for a command. For example, after picking the first point for a line, the
second point is constrained to the ortho angle.
Ortho off (left); Ortho on (right).
If you only need a different angle for a single operation, angle constraint is faster to use. Enter a specific angle
for one operation instead of changing the ortho angle and then changing it back.
### Snap to existing objects
Object snaps constrain the marker to specific points on an object. When Rhino asks you to specify a point,
you can constrain the marker to specific parts of existing geometry. When an object snap is active, moving the
cursor near a specified point on an object causes the marker to jump to that point.
Object snaps can persist from pick to pick, or can be activated for one pick only. Multiple persistent object snaps
can be set from the status bar. All object snaps behave similarly, but snap to different parts of existing
geometry. In addition, there are special object snaps that work for one pick only.
### Persistent object snaps
Use persistent objects snaps to maintain an object snap through choosing several points. Since persistent
object snaps are easy to turn on and off, you can set them and leave them on until they get in your way. You
can then set a different one or just disable them.
Sometimes object snaps interfere with each other and with grid snap or ortho. Object snaps normally take
precedence over grid snap or other constraints.
There are other situations where object snaps work in conjunction with other constraints. You will see
examples of this in this chapter. For more information including video demonstrations, see the Rhino help topic
Object snaps.
The Osnap control
The Osnap control is usually docked at the bottom of the screen.
4
If the Osnap control is not open, click the Osnap pane in the status bar.
4
Click a check box to turn on an object snap.
4
Right click a check box to turn an object snap on and turn off all other object snaps.
4
When an object snap is active, moving the cursor near an eligible point on an object causes the
marker to jump to that point and a tooltip to appear.
The check boxes in the Osnap control allow single-use overrides for the persistent object snaps.
To suspend all persistent object snaps
4
In the Osnap control, click the Disable button.
To clear all persistent object snaps
4
In the Osnap control, click Disable with the right mouse button.
