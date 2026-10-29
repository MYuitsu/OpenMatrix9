# Box Cube cross-viewport corner — 2026-10-29

Cube requires a second 3D corner. Previously the mouse ray used the previously active viewport's CPlane before the clicked viewport became active; choosing the second corner in an orthogonal view could leave the ray parallel to that plane and fail to commit. While Cube waits for its second corner, OM9 now intersects the ray with the hovered viewport's CPlane, clears stale hover/snap on a failed projection, and still uses the frame captured at the first point to orient the cube.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid_difference_ui viewport_box_cube_uses_second_view_for_3d_corner`: passed two simulated fixtures, Looking Down to Through Finger and Looking Down to Side View; measured volume, opposite-corner position and Undo checked. The fixture failed before the change.
- `cargo test -p om9-app --test solid_difference_ui viewport_box_`: 3 Box viewport tests passed before the expanded second-view fixture; both new view cases subsequently passed.
- `cargo test -p om9-app --test solid solid_box_`: 36 passed.

Snap candidates, custom or perspective CPlanes, all camera orientations and actual desktop/GPU interaction remain open; this is not full Box or Solid completion.
