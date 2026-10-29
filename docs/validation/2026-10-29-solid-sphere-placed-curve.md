# Sphere curve placement and stale source — 2026-10-29

Sphere AroundCurve and the existing curve-based Tangent modes previously sampled the source curve in local coordinates. They now sample the curve and apply the entity placement before passing the points to the existing solver. A source stamp prevents point or text completion after a captured source changes or disappears; cancel and undo remain available. Captured sphere data is counted in the pending-session budget.

Standalone Rust with OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 46 passed before the rotation assertion was added.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_curve_modes_use_placed_sources_and_guard_stale`: passed after adding rotation. Exercises translated selected and named-path AroundCurve, translated one- and two-line Tangent, a rotated Tangent line, Undo, and deletion of the captured source before completion.
- `cargo test -j 2 -p om9-app --lib solid_input_regressions`: 56 passed.
- `python tools/public_source_audit.py`: passed; 14717 source files, 594 icon bindings, 593 SVGs.

The curves are approximated by samples. This evidence does not establish the general tangent solver, arbitrary non-uniform transforms, projection of a typed Path pick onto the curve, actual desktop/viewport operation, or original Matrix behavior. Full Sphere and Solid completion remain open.
