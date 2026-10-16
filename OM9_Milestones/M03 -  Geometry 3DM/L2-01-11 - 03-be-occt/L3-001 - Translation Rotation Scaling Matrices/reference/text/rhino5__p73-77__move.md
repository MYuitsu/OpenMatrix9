# Move

- Source PDF: h:\OpenMatrix9\manuals\rhino5.pdf (Rhinoceros 5 User's Guide)
- Page range: 73-77
- Section title: Move
- Section id: rhino5-049-move
- Kind: topic
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): rhino5_p73_img01.png, rhino5_p74_img01.png, rhino5_p74_img02.png, rhino5_p74_img03.png, rhino5_p75_img01.png, rhino5_p75_img02.png, rhino5_p75_img03.png, rhino5_p76_img01.png, rhino5_p76_img02.png, rhino5_p76_img03.png

---
<!-- section rhino5-049-move | Move | rhino5 pages 73-77 -->

<!-- page 73 -->

## Transforms - Move, Copy, Rotate, Scale
Transforms change the location, rotation, number and shape of whole objects by moving, mirroring, arraying,
rotating, scaling, shearing, twisting, bending, and tapering. The transform commands do not break the objects
into pieces or cut holes in them.
### Note: For all of the following exercises, the images were cap-
### tured using Shaded mode display.
### Move
Use the Move command when you want to move an object a certain distance or if you want to use object
snaps to place an object accurately.
### Move objects using distance values
The Move command requires a from and to location.
You can pick these locations on the screen or type coordinates at the command prompt.
Practice moving objects
The object of this exercise is to move an object from a specific location on the object to a location in the
coordinate system.
1. Start a new model using any template.
2. Draw a Sphere of any size anywhere on the screen.
3. Select the sphere.
4. Start the Move command.

<!-- page 74 -->

Transforms - Move, Copy, Rotate, Scale
5. At the Point to move from prompt, with the Center object snap on, move the mouse around the
edge of the sphere until the Cen tooltip displays and click.
6. At the Point to move to prompt, type 0,0,0.
The sphere moves to the 0,0,0 coordinate point.
Tip: Simply typing 0 is a shortcut for the coordinates 0,0,0.
### Move objects by dragging
The quickest way is to click the object and drag it. Rhino provides tools for making dragging objects accurate.
You can drag objects in any viewport. Object snaps will help align objects to each other.
Practice dragging objects
1. Open the tutorial model Drag Objects.3dm.
Download the tutorial models.
2. In the Osnap control, turn on the Center object snap.

<!-- page 75 -->

Transforms - Move, Copy, Rotate, Scale
3. In the Perspective viewport, click the cone at the bottom edge and pause until the Center object
snap tooltip displays.
4. Drag the cone until the center of the cone's base lines up with the top surface of the cylinder and the
Center object snap for the cylinder's top face displays.
5. Release the mouse button to place the cone.
6. In the Front viewport, drag the cone to the top of the cylinder.
Watch what happens in the Perspective viewport.
There are many times when you have to watch what is happening in other viewports to accurately
place your objects.
### Elevator mode
You can press the Ctrl key to move objects in the z-direction. This is called elevator mode.
Elevator mode is like Ortho, except the movement is vertical to the active construction plane.
To practice using the Ctrl key to move vertically, you are going to move the box to a location 5 units above the
center of the sphere.
Using elevator mode to move objects vertically lets you work more in the Perspective viewport.

<!-- page 76 -->

Transforms - Move, Copy, Rotate, Scale
Move the box vertically
Note:
For the following images, Shade-highlight selected surfaces and
polysurfaces has been turned on.
(Options > View >Display Modes > Shaded > Objects > Selection)
1. Turn Ortho off.
2. On the Transform menu, click Move.
3. In the Perspective viewport, rotate the view so the sphere is toward the front, and select the box.
4. At the Point to move from... prompt, turn on the End object snap and click a corner of the box.
5. At the Point to move to prompt, turn on the Center object snap, hold down the Ctrl key, and click
the center of the sphere.
6. Release the mouse button and the Ctrl key and start to drag the box.
7. At the command prompt, type 5.

<!-- page 77 -->

Transforms - Move, Copy, Rotate, Scale
### Copy
The Copy command makes copies of objects.
Some transform commands like Rotate, Rotate3D, and Scale have a Copy option. This lets you create a copy
of the object as you rotate or scale it.
