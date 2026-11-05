# Top11 projected snap preview

When Project was enabled, the viewport sent a snapped 3D point directly to transform preview but projected it onto the active CPlane before accepting the click. A snap above the plane could therefore show the Move preview at a different height from the committed geometry. The UI now uses the same constraint, snap and projection routine for the transform preview and the point accepted by the click; this also applies to the other commands using that preview owner.

An egui fixture first failed at the projected preview position. After the change it passes with Project on and off in four signed XY directions: a synthetic pointer hovers an actual scene point at Z=5, the fixture verifies that point was snapped, checks the preview dot at the independently projected world position in the Through Finger view, clicks the Looking Down view, and checks the committed Move position. This exercises egui event routing and scene snap rather than a physical mouse, OS focus or GPU presentation. It uses the active Looking Down CPlane; interactions when clicking an inactive view with a different CPlane need separate verification.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target-top11-nested`, cargo `-j 2`, runtime DLL path `L:\om9-target\debug\deps`: this focused egui test passed with default and without default features; 144 Top11 and two UI replay integration tests passed with default features. Public source audit passed. No openNURBS measurement batch was performed.

SHA-256: `ui.rs` `f986c3650e20f8ac51b8162cbdaf786d3b9a1c098dae67d18db85f50a060b218`; `ui/chrome.rs` `59f132a5ed2f194153d9d632a89786bf5e05ff5e0ede8cc69462da3ef182a60d`.
