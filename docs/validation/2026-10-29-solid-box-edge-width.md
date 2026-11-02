# Box 3Point and Vertical width/height — 2026-10-29

The approved Box contract specifies a signed width chosen by point or number and height perpendicular to the chosen base. Previously these modes required numeric width, discarded negative width, and used the CPlane normal even when Vertical's height axis is different. The session now projects the first edge into the captured CPlane, derives the width and height axes, accepts signed width by point or text, and uses those axes for the height preview and commit. The Vertical width stage can intersect a ray with its vertical base plane. Zero projected edge or width is rejected without committing.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_box_`: 30 passed, including all four width/height sign combinations for each mode on one rotated CPlane, preview points, measured volume, undo and typed width/point routes.
- `cargo test -p om9-app --test solid solid_sphere_`: 38 passed as neighboring command regression.
- `cargo test -p om9-app --test solid_difference_ui viewport_box_vertical_picks_width_and_height_on_distinct_axes`: passed using simulated viewport picks and undo.

Preview while choosing the first edge, all edge-on and parallel camera cases, unit conversion, all coordinate systems and real desktop/GPU behavior remain open; these results do not establish full Box or Solid completion.
