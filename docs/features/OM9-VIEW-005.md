# OM9-VIEW-005 — Viewport to File

Original command `ViewCaptureToFile` is implemented from the local 01-core spec. Menu/sidebar and CMD invoke the same CoreViewControls capture handler. CMD recognizes capture before pending Curve input, so taking a screenshot does not consume or cancel the pending Curve command.

The handler opens a save-file dialog with PNG, BMP and JPEG filters. PNG is the OpenMatrix9 initial choice; choosing another filter updates the default suffix. Explicit supported filename extensions choose the encoder. Unsupported extensions report an error without writing. QFileDialog provides overwrite confirmation; QSaveFile writes atomically. Unicode filenames are supported. Cancel does not create a file or record successful execution.

Capture targets the native view active when invoked. After the modal dialog, a QPointer and native document closing check prevent dereferencing a destroyed view. Native savePicture uses the viewport's physical pixel dimensions, current camera, background and LiveInteractive render intent. The output includes native scene decorations such as the grid and navigation cube, but does not include the surrounding sidebar/CMD docks. PNG/BMP/JPEG support and initial format are OM9 choices where the local spec does not enumerate exact formats.

This is a read-only command: it does not change model geometry, selection, cameras, Undo or pending Curve state. It remains available when a task forbids model/view changes. Output is a standalone image; document save/reload is unaffected.

Native tests cover dialog cancellation, real encodings/signatures and decoding, physical pixel dimensions, rendered scene pixels, filename extension selection, overwrite rejection, invalid-format errors, inactive-camera invariance, changing the active viewport, restricted tasks, pending Curve input and clean process exit. The fixture uses Qt mouse events on Save and confirmation buttons, with native Windows dialogs disabled in the test process for automation. Physical desktop clicks and the Windows-native dialog backend remain unverified.

Evidence: `tests/core_capture_smoke.FCMacro`, 13 checks at100% in `build/capture-isolated-1/241d8a1019fc45bc9250981cac1f5ea0/results.json`; 16 checks at150% in `build/capture-isolated-1.5/3c6ffd4441b24a73897429c78c4ce335/results.json`, both process exit zero. The generated PNG from the initial successful run was visually inspected and contains the active Perspective box, grid and navigation cube.

The same expanded16 checks also pass at200% DPI with process exit zero: `build/capture-isolated-2/e04a26a7b148463593faff4e210dd8fa/results.json`.
