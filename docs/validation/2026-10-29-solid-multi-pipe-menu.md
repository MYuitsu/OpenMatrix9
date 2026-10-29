# Solid Multi Pipe menu and macro — 2026-10-29

The recovered Matrix menu crosswalk identifies `SolidGVDMultiPipe` with the macro `_Pipe _Multiple`. OM9 now exposes an authored purple SVG next to Pipe and routes that icon and the macro to the existing Pipe Multiple session. The existing Pipe icon's right-click remains ChainEdges. The session selects multiple curves, then Enter advances to radius entry; successful creation is one undoable transaction.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -p om9-app --test solid_difference_ui viewport_multi_pipe_menu_and_original_macro_share_geometry`: passed, testing menu dispatcher and exact macro with two selected line curves, measured volume and undo.
- `cargo test -p om9-app --test chrome_actions`: 26 passed, including a simulated click on the new icon and existing Solid icons across both menu pages.
- Solid icon audit: 33 authored SVG entries, no errors or shared geometry; public-source audit: no errors. The new asset was rendered for visual inspection.

The tests do not establish complete Multiple Pipe geometry, original Rhino/Matrix internal behavior, or real desktop/GPU behavior. Those remain open; see existing Pipe evidence for supported geometry and limitations.
