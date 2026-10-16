# Snap to existing objects

- Source PDF: Rhinoceros 5 User's Guide (h:\OpenMatrix9\manuals\rhino5.pdf)
- Page range: 46-47
- Section title: Snap to existing objects
- Section id: rhino5-032-snap-to-existing-objects
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section rhino5-032-snap-to-existing-objects | Snap to existing objects | rhino5 pages 46-47 -->

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

<!-- page 47 -->

Accurate Modeling
To turn on one object snap and turn all others off with one click
4
In the Osnap control, right-click the object snap you want to turn on.
### Special case object snaps
Complex object snaps that allow selecting multiple reference points or add other advanced controls. See the
Rhino help topic Object snaps for more information.
These are also available from the Osnap control.
To turn on complex object snaps
4
Hold the Ctrl key and hover over the Osnap control.
An alternate set of controls appears.
### Cursor constraints
When entering points, you can constrain the marker to a distance or angle from the previous point. Once you
have set the distance, drag the line around to any angle. You can also use further snaps to point the line in a
specific direction.
### Distance constraint
During any command that requires two points, such as the Line command, place the first point. Then at the
next prompt, type a distance and press Enter.
The marker will be constrained to the specified distance from the previous point. Drag the cursor around the
first point and then pick a point.
### Angle constraint
Angle constraint is similar to ortho, but you can set any angle and it is a one-time setting.
The < symbol is used because it is similar to the symbol used in geometry to indicate an angle.
The marker will be constrained to lines radiating from the previous point separated by the specified angle,
where the first line is the specified number of degrees counterclockwise from the x-axis. If you enter a negative
number, the angle will be clockwise from the x-axis.
