# Top11 Control Points: synthetic egui pointer and Delete — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. This is an input-routing regression on the actual `MatrixApp::show` viewport path, not native desktop UI acceptance.

A typed 4x4 NURBS surface fixture invokes `PointsOn` through CMD, then drives four projected control grips using egui press/release events, including Shift modifier changes. It confirms each grip is selected, invokes the same raw Delete handler used by the app, verifies the net changes to 3x4, and checks Undo/Redo. A second fixture drives a NURBS pole drag via press, movement and release, checks its new world position within 1e-4, confirms the object remains selected instead of becoming a selection band, and checks Undo.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: both default-feature targeted UI tests passed; the click/Shift/Delete test also passed with `--no-default-features`. No GPU output, native pointer/keyboard focus, physical desktop, multiple DPI, 3DM or human M03-UI acceptance was tested. This does not establish all control-grip behaviors or completion of other Top11 commands. No production code was changed in this validation step.

Source SHA-256: `crates/om9-app/src/ui/chrome.rs` b8496afcc05566e048024c6c62b39c73d37ab1910b77c485e239555bff1108ba; `crates/om9-app/src/ui/top11.rs` d269ef01a1fdf76ce3509fc268a076aec37d3e30129889d77a25447a243a6571; `crates/om9-app/src/ui.rs` 5c2c187dfd569b78ee14343c982ca6265ea852ddd923f4aab6c018e405fc3538.
