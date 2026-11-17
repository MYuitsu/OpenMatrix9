# Object snap when the view is parallel to the input plane

Previously the viewport attempted object snapping only after the pointer ray
intersected the active input plane. In a side view parallel to that plane,
even a valid surface or endpoint under the pointer produced no snap. The
viewport now queries object snaps while a point command is active even when
there is no input-plane intersection. Without a valid snap, it does not use a
previous hover point; normal input-plane and SmartTrack behavior remains on
the existing path when an intersection exists.

A headless egui fixture opens a saved trimmed cylinder, starts Line, and uses
Surface Snap through a front trim hole. The selected start lies on the rear
surface; a preceding pointer miss does not modify the drawing, and the new
line has one working Undo/Redo. Another side-view fixture verifies Endpoint
Snap on a source line. The four related batches passed: Gumball viewport 7,
B-rep snap 1, snap UI 25 and Rectangle regressions 5. The debug app build and
public-source audit passed. These fixtures do not establish behavior for every
command, constraint or camera, physical desktop input, or all trim geometry.
