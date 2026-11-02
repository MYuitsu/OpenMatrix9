# Sphere menu variants and radius entry — 2026-10-29

The recovered Matrix crosswalk has six Sphere variant IDs and `_Sphere` mode macros. OM9 now exposes authored icons and dispatches the recovered macros to the existing Sphere session for 2Point, 3Point, 4Point, AroundCurve, Tangent and FitPoints. The numeric shortcuts 2/3/4 now remain numeric radius input after center selection, rather than changing construction mode.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -p om9-app --test solid solid_sphere_`: 40 passed, including recovered macros and IDs, radius 2/3/4 measured geometries, and existing mode fixtures.
- `cargo test -p om9-app --test chrome_actions`: 28 passed, including simulated clicks on six variant icons and Solid menu pagination regression.
- Solid icon audit: 43 authored SVG entries, no errors or shared geometry; 24/32/64px dark review sheet visually inspected. Public-source audit: no errors.

The menu tests establish mode start, not all possible outputs. Full tangent/fit solvers, document-unit behavior, original Matrix interaction and real desktop/GPU remain open. No full Sphere or Solid completion is claimed.
