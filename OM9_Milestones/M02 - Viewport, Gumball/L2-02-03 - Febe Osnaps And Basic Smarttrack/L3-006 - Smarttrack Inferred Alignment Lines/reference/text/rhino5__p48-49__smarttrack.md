# SmartTrack™

- Source PDF: Rhinoceros 5 User's Guide (h:\OpenMatrix9\manuals\rhino5.pdf)
- Page range: 48-49
- Section title: SmartTrack™
- Section id: rhino5-035-smarttrack
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.

---

<!-- section rhino5-035-smarttrack | SmartTrack™ | rhino5 pages 48-49 -->

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

<!-- page 49 -->

Accurate Modeling
You can snap to intersections of the tracking lines, perpendiculars, and directly to smart points as well as
intersections of tracking lines and real curves. The tracking lines and smart points are displayed for the
duration of a command.
### Coordinate systems
Rhino uses two coordinate systems: construction plane coordinates and world coordinates. World coordinates
are fixed in space. Construction plane coordinates are defined for each viewport.
### Cartesian coordinates
When Rhino prompts you for a point, if you type x and y Cartesian coordinates, the point will lie on the
construction plane of the current viewport. For more information about coordinate systems and numeric
constraints, see www.mathopenref.com/coordinates.
### Right-hand rule
Rhino follows what is called the right-hand rule. The right-hand rule can help you determine the direction of
the z-axis. Form a right angle with the thumb and forefinger of your right hand. When your thumb points in
the positive x-direction, your forefinger points in the positive y-direction, and the palm of your hand faces in
the positive z-direction.
### World coordinates
Rhino contains one world coordinate system. The world coordinate system cannot be changed. When Rhino
prompts you for a point, you can type coordinates in the world coordinate system.
The arrow icon in the lower left corner of each viewport displays the direction of the world x-, y-, and z-axes.
The arrows move to show the orientation of the world axes when you rotate a view.
### Construction plane coordinates
Each viewport has a construction plane. A construction plane is like a tabletop that the cursor moves on unless
you use coordinate input, elevator mode, or object snaps or a few other instances where input is constrained.
The construction plane has an origin, x- and y-axes, and a grid. The construction plane can be set to any
orientation. By default, each viewport’s construction plane is independent of those in other viewports.
