# Sphere Tangent numeric radius units — 2026-10-29

Existing Sphere Tangent branches passed numeric radius directly to solvers whose geometry is in millimeters. The one-, two- and three-curve, curve-plus-point and two-point branches now convert a typed positive Radius from document units once, accept case-insensitive `Radius=`, and retain the session on conversion failure. Branches that derive radius from picked geometry are unchanged.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_sphere_`: 45 passed, including five centimeter Tangent line/point fixtures, invalid input retry, measured volumes, constraint distances and Undo.
- `cargo test -p om9-app --test command_options_interface sphere_tangent_radius_centimeters_uses_shared_option_routes`: one test, three simulated keyboard/pointer/typed routes, passed.

These tests cover selected line/point cases only. General tangent solver correctness, transformed curves, other units and real desktop/GPU remain open. No full Sphere or Solid completion is claimed.
