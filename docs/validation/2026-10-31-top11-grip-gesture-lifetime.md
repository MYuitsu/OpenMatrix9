# Top11 grip gesture lifetime

A grip drag previously retained only a handle and plane. Changing the object selection during the drag then releasing the pointer could still modify the previously selected object. The regression first failed with a new geometry revision on that object.

The drag now captures its source stamp, coordinator epoch, selection revision, mode, viewport, camera, viewport rectangle and construction plane. It is canceled if those owners or the application focus change; only the originating viewport may update or commit the gesture. A viewport layout or displayed-camera change also cancels it. Escape follows the existing cancel path. The cancellation reports status and does not create a transaction.

A simulated egui pointer fixture checks selection replacement, an intervening edit to the same object, camera zoom, active-view switch, CPlane change, maximize, resize, grip-mode change, focus loss and Escape. It verifies the document and Undo depth remain as they were immediately before release. Three viewport-plane/lifetime tests and two existing pointer-drag tests passed with default features; the ten-scenario lifetime test passed without default features, with existing unrelated warnings. The 133 app Top11 controller tests passed. This is simulated input and does not prove OS focus, GPU presentation, all asynchronous failures or human desktop acceptance.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`. SHA-256: `ui.rs` `d781042e584b415f8cf2f3553fb8c6c56b075d1318f7e997707bc47298f02261`; `ui/top11.rs` `cfc1e943352ab2d3c821ca36bd4ca0198f9f7abe17a6c702647476f4d1957725`; `ui/chrome.rs` `f6d8de56de5e381c1891646a96d576cdd70c02dfaefaa6fe8210826a7c20d59d`. No native adapter changed and no openNURBS measurement batch was performed.
