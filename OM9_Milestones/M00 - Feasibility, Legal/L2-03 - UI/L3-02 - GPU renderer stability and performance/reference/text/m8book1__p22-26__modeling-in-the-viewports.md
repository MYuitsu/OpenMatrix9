# Modeling in the Viewports

- Source PDF: matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 22-26
- Section: Modeling in the Viewports (id m8book1-005-modeling-in-the-viewports)
- Mapping confidence: low
- Rationale: Viewport pan/zoom/orbit proof; the manual chapter is the behavioural reference for the spike

---

<!-- section m8book1-005-modeling-in-the-viewports | Modeling in the Viewports | m8book1 pages 22-26 -->

<!-- page 22 -->

# CHAPTER 4
# Modeling in the
# Viewports
### Learn the important features of the main workspace in the product: the
### Viewports. See and work on one model from four views at a time.
About the Viewports
Picture the viewports as four different views of
the same thing. In this case, the model is more
than likely a ring, so they are set up to present
four views of the ring: “Looking Down” on the ring
as though the wearer were gazing at her finger;
the “Through Finger” view of the ring as though
you were holding it up to look through the finger
opening; the “Side View” of the ring, as though
you were looking at it from one side of the
wearer’s finger; and finally, the “Perspective”
viewport, which allows the user to freely rotate the
view to any angle necessary.
Each viewport contains the visible grid: a 40 X
40 mm grid or Construction Plane (shortened to
“C-Plane”) which is centered on the ring in each
of the aforementioned views (as long as the ring
is not moved off center of where the Ring Rail is
placed when using this tool). The center of the
grid is marked with a red (horizontal) and blue
(vertical) line so you can find the center visually
very easily. These grids, grid lines, and center
lines represent the familiar Cartesian planes you
learned in high school geometry, where X is the
horizontal line, Y is the vertical line, and Z is the
direction coming “toward” the user - or, up and
down.
The 40x40mm grid is broken into
1 x 1 mm squares indicated by
the light grid lines and 5 X 5 mm
squares indicated by the LIGHTER
grid lines.
Looking Down
Perspective
Side View
Through Finger
Four Views of one model. Looking Down, Side View
and Through Finger are planar viewports. These are
mathematically accurate but give you a limited view of the
model. The Perspective viewport gives you a good overall
view of your model.
Global vs. Local X, Y, & Z
You will notice that each viewport has its own X,
Y and Z (termed “Local” in Rhino), represented by
the colors of the lines in the center of the grid, but
this is not always the program X, Y, and Z (termed
“Global”or “World”). Global X, Y, and Z are only
represented in the Looking Down and Perspective
viewports, and can be tracked to each of the four
viewports by looking at the small Cartesian “key” in
the lower left-hand corner of the viewports, which
shows the Global coordinates relative to each
view.
For example, Looking Down shows you X and Y;

<!-- page 23 -->

Through Finger shows you X and Z, and Side View
shows you Y and Z.
Why this is so beneficial?
The program is set up this way because there are
often times when it will be easier to work on one
part of the ring in one viewport and one part of
the ring in another viewport: the view in which you
can see and access that part of the design best.
a mention anyway just to understand a bit more
about the viewports. Each light gray “box” on
the grid represents 1 mm. Each brighter gray box
represents 5 mm square. The grid is therefore
40 X 40 mm square. But because Rhino is
a sophisticated CAD modeler living in this
Cartesian world, you can actually type values
into the Command Line to start a design at a
certain Cartesian coordinate on the grid. Type in
the format “1, 2, 3” to place a point at the world
coordinate 1, 2, 3. Or, leave off the last number
(program reads it as “0”) and the point will be
placed flush against the Looking Down grid.
When the Command Line is asking for a value,
remember these are always in Millimeters (mm),
because the grid lines each represent 1 mm
square. You will learn later on in this section how
to change to other values if you wish.
F4: The Center of the Universe
From this moment on, the new center of your
universe is “F4”, which is the keystroke on your
keyboard representing 0,0,0 in the Cartesian
system in Matrix: or, the center of the grid axes.
Since the finger size for the ring you will build is
centered on this point (0,0,0) in space, it is very
likely you will need to refer to it when performing
modeling functions that - for example - make the
ring symmetrical (Mirror), make a bypass shape
that is the same in two corners in the Looking
Down view of the ring (Rotate), or for any number
of tools you will learn in the program. Whether or
not you are building a ring, it is a good idea to
center your model on F4, as it is very easy to
Mirror designs just by pressing this key as the
center of the mirror plane around which you are
mirroring an item. Without it, you never know what
you are centered on, and your design can easily
come out lopsided and give you bigger problems
down the road when you are producing it on a mill
or growing machine.
Activating & Maximizing Viewports
For many commands, it matters what viewport
you are working in. These commands are called
viewport dependent. That is because by default,
objects and commands run from a viewport’s
Construction plane. The Perspective and Looking
Down viewport share a Construction plane that is
at the zero on the Z axis. The Through Finger
viewport’s Construction plane (C-Plane) is at zero
on  the Y axis and Side view is at Zero on the X
axis. This is the same location as the grid in each
viewport, so it maybe helpful to think of  the
C-Plane and the viewport grid as nearly
synonymous.
The Perspective and Looking Down viewports share a
Construction plane, indicated by the orange surface.
Mirroring from F4
Cartesian Coordinates & MM Values
Although it is possible to model this way in
Matrix, It doesn’t mean you should; but it’s worth

<!-- page 24 -->

To work in any viewport, move the mouse into that
viewport, left-click on the grid, and the viewport
will be activated - i.e., that is the one the object will
be drawn in. If you don’t need to click on the grid
but wish to activate the viewport, click on the
Right-click and drag in a planar viewport to pan.
Hold Shift and Right-Click to pan in the Perspective
viewport.
Zooming the View
In any of the four viewports, move the mouse over
the portion of the grid that you would like to focus
in on and scroll the mouse wheel up to zoom in
and down to zoom out.
Position your mouse cursor over the area of interest.
Roll you mouse wheel away from you to Zoom.
viewport name, turning it from a dim blue to a
brighter blue color, and it will be active. Suppose
the viewport is too small to see comfortably while
you are working?
Double-click on the viewport name and the
viewport will take up the entire four-viewport view
of the product so you can really see what you are
doing. Obviously, this is now the active viewport.
To minimize the viewport again, returning to the
four-viewport view, double-click the name of the
viewport again and it will be reduced. Or, click the
Reset button, which restores the default view of
the viewports that the program originally has at
startup.
When one of your views is maximized you can
quickly change between viewports by clicking on
the viewport tabs at the top of the window.
Resetting the View Again
To restore either the Pan or Zoom back to the
default at program start-up, use the Reset button
in the Matrix Main Menu. Pressing the F5 key will
recenter the camera at F4 but not change the
zoom amount or Shade mode.
Panning the View
In any of the three planar viewports, (planar
means flat) Looking Down, Side View, and
Through Finger, position your mouse anywhere in
the grid and hold down your right mouse button
while moving the mouse left, right, up or down
(right-click and drag). This “pans” the view - or,
moves it in the direction your mouse moves - to
give you a better picture of everything you can
see in that viewport.
Rotating the View In the Perspective
Viewport ONLY
Holding down the right mouse button while
moving the mouse left, right up or down (right-
click and drag) rotates the view in the direction of
the mouse. The center of this rotation might not
be on the model: it is at the center of the current
view of the model. So, if this feels a little weird,
click “Reset” and re-try. To “Pan” the Perspective

<!-- page 25 -->

viewport, hold down the Shift key while using
“right-click and drag” in the direction you wish to
pan.
object, it will flag you with a little Selection Box
that lists all the possible objects in the general
vicinity of your cursor that you could be trying to
select. To help you out, it displays their Layer color
beside what they are (Curve, Surface, Gem, etc.)
to assist you in the selection process. If you see
what you want in that box, move the mouse over
its name and click it. It will be selected, and the
box will disappear.
Surrounding Objects to Select
Another way to select multiple objects at the
same time (the selection box won’t appear) is to
The Perspective viewport gives you the best overall
sense of the look of your design; though it can be
easier to be precise in the planar views.
The Selection window appears when there are multiple
objects where you clicked.
Selecting an Object
There are lots of instances in the program when
you’ll need to select an object in the viewports.
The best way is to left-click on the object, and
it will turn to the pink highlighted color that
indicates to you it is selected. If you need to add
objects to your selection, hold down the “Shift”
key on the keyboard and click on more objects. If
you need to remove objects from your selection,
hold down the “Ctrl” key and click on selected
objects to remove from the selection.
Selection Box
If the program can’t figure out what you are
clicking on when you are trying to select an
“region-select” them. Position the mouse at the
upper left-hand corner PAST the objects to select,
hold down the left mouse-button, and “click and
drag” a selection rectangle (solid line) down and
to the right, past the other side of the objects to
select and fully surrounding EVERYTHING you
wish to select. Everything in that rectangle will be
selected.
Crossing Objects to Select
Another way to select multiple objects is to click
Click and drag, FROM LEFT TO RIGHT, a box to
surround all objects you want selected. Objects will
only be selected if they are completely surrounded.
Left-click on an object to select it. Hold down Shift
and left-click to select multiple objects. Ctrl and left-
click will remove an object from the selection.

<!-- page 26 -->

to the RIGHT of all the objects to select, hold
down the left mouse-button, and click and drag a
selection rectangle down and to the left (dotted
line). In this situation, you only need to “cross” - or,
touch with the selection box - everything you
wish to select, and it will be selected.
Note: To deselect anything, click on another object
to select; or, click on the black grid of the viewport,
selecting nothing.
Click and drag, FROM RIGHT TO LEFT, a box to
CROSS all objects you want selected.
Visual Cues
Obviously, the pink highlight color indicates
what is selected. Other visual cues include: the
cursor turns into a flat “hand” while panning; a
“magnifying glass” while zooming, and a little
pointer finger while selecting Command Line
options. A simple text cursor appears when you
position the cursor to type in the Command Line.
While drawing, you will see cross-hairs, indicating
the next point will be placed, and you may see a
marker with a Tracking line in functions.
Keyboard commands
Keyboard Shortcuts for the Viewports
Press F7 if you don’t wish to see the grid lines,
F4 to indicate 0,0,0 during a Command, and
F5 recenter the viewport camera at  F4. “Enter”
ends a command, “space bar” and “right mouse-
click” start up the same command again, Ctrl+
C, V, X, Z, Y, and A are the same as they are in
Windows: respectively, Copy, Paste, Cut, Undo,
Redo and Select All (objects
in the viewports). Other
keyboard shortcuts will
be discussed with the
commands they relate
to. You can also view
additional keyboard short
cuts in the Rhino Options
(Info & Settings menu)
under “Keyboard”-you can
even add your own.
