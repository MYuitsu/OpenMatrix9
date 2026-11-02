# Sphere AroundCurve world pick — 2026-10-29

Previously `Path=...@x,y,z` and `Curve=...@x,y,z` used the raw point as sphere center even when it was off the curve, while interactive point input chose the closest of 128 discrete samples. Both paths now use a bounded world-space closest-point search evaluated on the curve; entity placement is applied before distance comparison. This is an OM9 interpretation of the center-on-curve contract, not verified Matrix original behavior.

With OCCT, `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_around_curve_projects_world_pick_for_all_routes`: one test with five fixtures passed: interactive input, Path and Curve aliases, an outside endpoint and a circle; the line and circle are translated, and each commit is undone.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 47 passed.
- `python tools/public_source_audit.py`: 14718 source files, 594 icon bindings, 593 SVGs, no errors.

The search uses 129 coarse points and local refinement: it is not a proof of the global nearest point for arbitrary geometry. Real viewport picking, original Matrix behavior, arbitrary transforms and general Sphere solvers remain unverified. Test route `accept_world_point` is controller-level simulation, not a desktop click.
