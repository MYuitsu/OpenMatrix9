# Sphere Center and 2Point document units — 2026-10-29

Sphere Center and 2Point numeric radius/diameter previously treated the entered number as millimeters; typed center coordinates were also treated as world coordinates. Center point input now uses the shared document-unit/CPlane coordinate parser; Center Radius/Diameter, numeric 2Point Diameter and the direct one-number Sphere path convert to millimeters once. 2Point captures the CPlane at its first point and uses its X axis for a typed diameter. World-space picked points are not converted again.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_sphere_`: 43 passed, including 30 Center/2Point unit-and-input fixtures across six document units, rotated CPlane typed center and typed diameter.
- `cargo test -p om9-app --test command_options_interface sphere_radius_non_mm_uses_shared_option_routes`: one test with three simulated inch Radius option routes passed.
- `cargo test -p om9-app --lib solid_input_regressions`: 56 passed.

The initial large-radius unit fixture was interrupted because it drove excessive tessellation; the completed matrix uses the same 5 mm physical radius in each unit. 3Point/Tangent numeric radius, other solver semantics, custom view changes and actual desktop/GPU remain open; this does not establish full Sphere or Solid completion.
