# Slab development point input and preview — 2026-10-29

The hidden development session for a preselected open line/polyline now accepts a point for signed offset and a point for signed height. A wire preview uses the same strip geometry and extrusion vector as commit. A viewport ray intersects a plane containing the source normal during height input. BothSides is honored in height preview. Source stamps are checked; invalid point input remains in session. The user-facing Slab command remains unavailable.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 6 passed. The new fixture compares preview bounds with measured BRep bounds and volume for both signs and BothSides, and tests Undo.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_picks_side_and_height`: 1 passed using hidden session initialization and simulated pointer events in Looking Down and Side views; measured volume is checked.
- `cargo check -j 2 -p om9-app --no-default-features`: passed with 12 warnings.
- `python tools/public_source_audit.py`: 14734 source files, 594 icon bindings, 593 SVGs, no errors.

The point-side calculation currently uses the first polyline segment, not a nearest-segment or general offset solver. Real source selection and Menu/CMD, explicit Distance-side workflow, ThroughPoint, InCPlane, general curves, rotated views, desktop/GPU and original Matrix behavior remain open. Preview and commit share the geometry foundation for the stated fixtures only.
