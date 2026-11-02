# Sphere 3Point Radius and direction — 2026-10-29

The approved Sphere contract requires radius followed by a direction after two chord points. Previously Radius immediately committed a sphere with a world-Y offset, ignored the document unit and gave no direction choice. OM9 now stages a document-unit radius, rejects a zero chord or radius below half the chord, and derives the center from the component of the picked direction perpendicular to the chord. Preview and commit use the same calculation. Undo removes the staged radius, invalid direction retains the session and no history is created before commit.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_sphere_`: 44 passed, including two opposite directions, a rotated CPlane, centimeter input, invalid retry, preview, geometry and Undo.
- `cargo test -p om9-app --test command_options_interface sphere_three_point_radius_routes_wait_for_direction`: passed three simulated keyboard, pointer and typed option routes.
- `cargo test -p om9-app --test solid_difference_ui viewport_sphere_three_point_radius_selects_direction`: passed simulated viewport direction and Undo.

Other Sphere construction solvers, degenerate/numerically extreme cases beyond these fixtures, Matrix original interaction and actual desktop/GPU remain open. This does not establish full Sphere or Solid completion.
