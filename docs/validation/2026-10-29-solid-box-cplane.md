# Box construction-plane frame — 2026-10-29

Box previously used world axes for base, height and placement. Default/Diagonal and Center now capture the input CPlane at the first point, use its frame for base and height previews, project the height ray against that frame, and place the committed BRep in that frame. CMD point coordinates use the shared coordinate parser, including `0` as the CPlane origin. Undo of the first point clears the captured frame. The frame is also passed to the existing other-mode construction code, but their full interaction is not established by these fixtures.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 28 passed, including signed point heights, Center dimensions, preview and measured volume on a rotated CPlane, CMD origin/local coordinates and a projected height ray.
- `cargo test -p om9-app --lib solid_input_regressions`: 56 passed.
- `cargo test -p om9-app --test solid_difference_ui viewport_box_negative_height_stays_below_base`: passed as a world-XY viewport regression.

The rotated test uses one orthogonal plane; changing viewport/CPlane during a command, rotated viewport picking, full unit conversion, Vertical/3Point/Cube previews and real desktop/GPU remain open. No full Box or Solid completion is claimed.
