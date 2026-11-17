# Ordinary scene B-rep display ownership

Ordinary `SceneObject`s previously cloned the document B-rep's display Arc
for picking and snap. A detached clone of one object could therefore retain
the display after its document and controller were dropped, even though its
render-buffer lease accounted only for the buffer. Ordinary objects no longer
own that Arc. Picking and surface snap first resolve the current entity through
the scene's existing source/leaf/display receipt, then borrow its document
display for the duration of the operation. A published remesh still owns its
separately admitted display override and uses that display's topology order.

The new production-Sphere fixture failed before the change: after dropping the
controller and document, a detached ordinary scene object kept the document
display alive. Afterward the document and display were released while the
object's render buffer remained usable and retained its existing CPU-cache
lease until the clone was dropped. Scene tests passed 34 unit, 13 snap, 4
metadata, 6 shared-instance and 5 surface-broad-phase cases. App tests passed
20 B-rep precision, 2 Sphere/Gumball including unattested-v4 fallback, 48
topology-owner with 2 ignored, 13 scene-memory and 1 GPU offscreen case.
Builds used `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.

This removes a redundant scene-object owner; it does not evict the display
from a live document or its Undo history. A complete scene snapshot still owns
its document, and remeshed overrides keep their independent lease. The tests do
not establish native/process peak, all concurrent owner interleavings, desktop
behavior, or M0/M1 completion.
