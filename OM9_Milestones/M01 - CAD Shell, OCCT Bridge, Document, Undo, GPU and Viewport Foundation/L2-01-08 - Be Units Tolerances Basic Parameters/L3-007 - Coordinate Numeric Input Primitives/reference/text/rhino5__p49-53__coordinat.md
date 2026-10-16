---
source_pdf: h:\OpenMatrix9\manuals\rhino5.pdf
source_key: rhino5
page_range: 49-53
section_title: Coordinate systems
section_id: rhino5-036-coordinate-systems
---
# Coordinate systems

Source: rhino5.pdf, pages 49-53

<!-- section rhino5-036-coordinate-systems | Coordinate systems | rhino5 pages 49-53 -->

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

<!-- page 50 -->

Accurate Modeling
The construction plane represents the local coordinate system for the viewport and can be different from the
world coordinate system.
Rhino’s standard viewports come with construction planes that correspond to the viewport. The default
Perspective viewport, however, uses the world Top construction plane, which is the same construction plane
that is used in the Top viewport.
The grid lies on the construction plane. The dark red line represents the construction plane x-axis. The dark
green line represents the construction plane y-axis. The red and green lines meet at the construction plane
origin.
To change the direction and origin of a construction plane, use the CPlane command. Preset construction
planes (World Top, Right, and Front) give you quick access to common construction planes. In addition, you
can save and restore named construction planes and import named construction planes from another Rhino
file.
2-D construction plane coordinates
4
At a command prompt, type the coordinates in the format x,y where x is the x-coordinate and y is
the y-coordinate of the point.
3-D construction plane coordinates
4
At a command prompt, type the coordinates in the format x,y,z where x is the x-coordinate, y is the
y-coordinate, and z is the z-coordinate of the point.
4
To place a point 3 units in the x-direction, 4 units in the y-direction, and 10 units in the z-direction
from the construction plane origin, type 3,4,10 at the prompt.
Tip: If you enter only x- and y-coordinates, the point will lie on the construction
plane.
### Relative coordinates
Rhino remembers the last point used, so you can enter the next point relative to it. Relative coordinates are
useful for entering a list of points where the relative locations instead of absolute locations of the points are
known. Use relative coordinates to locate points according to their relationship to the previous active point.
To use relative coordinates
4
At a command prompt, type the coordinates in the format rx,y where r signifies that the coordinate
is relative to the previous point.
For example
1. Start the Line command.
2. At the Start of line… prompt, click to place the first end of the line.

<!-- page 51 -->

Accurate Modeling
3. At the End of line… prompt, type r2,3, and press Enter.
The line is drawn to a point 2 units in the x-direction and 3 units in the y-direction from the last point.

<!-- page 52 -->

<!-- page 53 -->

## Create Surfaces from Curves
A common way of working in 3-D is to draw curves that represent edges, profiles, cross-sections, or other
surface features and then to use surfacing commands to create surfaces from those curves.
### Edge curves
You can create a surface from three or four curves that form the sides of the surface.
Create a surface from edge curves
1. Open the tutorial model EdgeSrf.3dm.
Download the tutorial models.
2. On the Surface menu, click Edge Curves.
Tip: Open the Help panel to review the help topic for the EdgeSrf command.
3. Select the four curves.
Objects change to yellow when you select them.
A surface is created from the curves that form its edges.
