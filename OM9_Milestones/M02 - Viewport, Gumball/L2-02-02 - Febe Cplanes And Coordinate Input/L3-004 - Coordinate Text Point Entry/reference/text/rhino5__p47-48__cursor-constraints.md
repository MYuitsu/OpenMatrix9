# Cursor constraints

- Source PDF: Rhinoceros 5 User's Guide (h:\OpenMatrix9\manuals\rhino5.pdf)
- Page range: 47-48
- Section title: Cursor constraints
- Section id: rhino5-033-cursor-constraints
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section rhino5-033-cursor-constraints | Cursor constraints | rhino5 pages 47-48 -->

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

<!-- page 48 -->

Accurate Modeling
### Distance and angle together
Distance and angle constraints can be used at the same time.
1. At the command prompt, type the distance, and press Enter.
2. At the next command prompt, type the < character and the angle value, and press Enter.
### Elevator mode
Pick a second point to specify the z-coordinate of the desired point. It is easiest to see this in a different
viewport or use the Perspective viewport. Drag the mouse cursor around to see the marker move vertically
from the base point along the tracking line.
Pick the point with the mouse or type the height above the construction plane. Positive numbers are above the
construction plane; negative numbers are below it. You can use further constraints like coordinates, object
snaps or grid snap for the first point, and you can use object snaps for the height.
To move the marker in the construction plane z-direction, hold the Ctrl key and click a point on the
construction plane, and then drag vertically from the construction plane and click to pick a point.
This constraint is called elevator mode. Using elevator mode to move your pick point vertically from the
construction plane lets you work more in the Perspective viewport.
### SmartTrack™
SmartTrack is a system of temporary reference lines and points that is drawn in the Rhino viewport using
implicit relationships among various 3-D points, other geometry in space, and the coordinate axes directions.
Temporary infinite lines (tracking lines) and points (smart points) are available to object snaps very much like
real lines and points.
