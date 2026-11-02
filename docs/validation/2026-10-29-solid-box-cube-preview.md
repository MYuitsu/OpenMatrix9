# Box Cube and initial edge preview — 2026-10-29

The approved Box contract calls for a captured frame, two opposite 3D corners and the minimum rotation that aligns the body diagonal. Cube preview previously used world axes, and the exact half-turn fallback did not normalize its rotation axis. It now uses the same captured-frame rotation as commit, including the antiparallel case. A valid second corner commits immediately; a duplicate corner leaves the session open. 3Point and Vertical first-edge previews project onto the captured CPlane. After a first point, the input `3` is interpreted as a dimension rather than an attempt to switch mode.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 33 passed, including rotated-plane previews against all eight committed cube vertices, exact and near antiparallel diagonals, direct point and typed second-corner commit, invalid retry, edge preview and undo.
- `cargo test -p om9-app --test chrome_actions`: 27 passed as simulated menu regression.
- `cargo test -p om9-app --lib solid_input_regressions`: 56 passed.

Actual 3D corner acquisition through all viewport and snap modes, edge-on ray behavior, document units and real desktop/GPU interaction remain open. This does not establish full Box or Solid completion.
