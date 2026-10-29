# Sphere AroundCurve fraction and preview — 2026-10-29

`OnCurve` now evaluates the source curve at the requested parameter fraction and applies its placement, rather than interpolating two sampled points. After a center is set, AroundCurve displays the same three-circle radius preview as Center; committing a picked radius uses the same center and distance. Existing source-stamp checks also suppress stale previews.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_aroundcurve_fraction_and_preview_match_committed_geometry`: passed on a translated full circle at fraction 0.125; checks center precision, 192 preview segments, volume and Undo.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 48 passed.
- `python tools/public_source_audit.py`: 14719 source files, 594 icon bindings, 593 SVGs, no errors.

This is controller evidence, not an actual viewport/GPU or Matrix-original verification. BRep-edge input, arbitrary curve cases, desktop and complete Sphere/Solid coverage remain open.
