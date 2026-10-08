# Four-view grid zoom investigation

User report: grid remains stationary while zooming. Local workspace specification remains authoritative; no PDF reread.

Current `OM9ConstructionGrid` is native world-space geometry on each view's C-plane, with fixed 1 mm spacing and +/-20 mm extent. It should scale through the same native camera as the model. This investigation has not reproduced a stationary grid in the tested builds, so no speculative grid implementation change was made.

`tests/core_grid_camera_probe.FCMacro` exercises all four views, checking independent camera projection and exported native image pixels. On the patched host with `build/snaps-runtime/OpenMatrix9Gui.pyd`, all four views passed pan, manual camera zoom, wheel zoom, Ctrl-left drag, menu Zoom_Dynamic and CMD Zoom_Dynamic. Process exited 0. Evidence: `build/grid-camera-isolated-1/0e2e0ea40f8a409ea39e25bcfa921f18/results.json`.

Orthographic spacing was 8 -> 16 pixels with manual 2x zoom, then 19.542 pixels with one positive wheel tick. Perspective spacing was 3.789 -> 7.891 -> 9.554 pixels. Wheel zoom changed 29,122 central image pixels in each orthographic view and 32,708 in Perspective. Ctrl/menu/CMD zoom each changed projected spacing in every view.

200% DPI passed the same four-view checks and exited 0: `build/grid-camera-isolated-2/bcef2ed3d9674b0e8e455fee738d43bf/results.json`. Drag endpoints are limited to the target viewport height so the fixture does not leave small high-DPI quadrants.

An initial probe sent wheel events directly to the GL viewport and observed no camera change. Quarter installs its Coin input translation event filter on the native `Gui::View3DInventorViewer`; sending the test event to that viewer exercises the actual translator. This was a fixture delivery error, not evidence of a product wheel bug.

Comparison using the previous preview module also changed grid spacing with wheel zoom. That older host failed during process teardown with access violation; it is not the validated runnable host. No conclusion that the user's grid issue is caused by an old build is established.

Limitations: synthetic Qt input, fixed grid extent, no adaptive ruler labels/spacing. Exact user zoom gesture/build remains to be confirmed. The full core inventory remains in progress.
