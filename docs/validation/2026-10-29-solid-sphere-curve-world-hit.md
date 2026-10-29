# Sphere AroundCurve standalone curve world hit — 2026-10-29

After a standalone curve source is chosen, viewport center selection now accepts a revalidated hit on that specific curve and passes its world point to the existing curve projection. The former generic point route intersected the viewport CPlane before projecting, which could select a different location for an elevated curve. Other geometry under the pointer cannot replace the captured source. Source stamps and the existing stale guard remain active.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_sphere_aroundcurve`: 5 passed, including an elevated straight curve in a simulated Perspective viewport and four existing source/edge workflows.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 51 passed.
- `python tools/public_source_audit.py`: 14725 source files, 594 icon bindings, 593 SVGs, no errors.

Curve hit geometry and bounded nearest-point search remain approximations; this does not prove arbitrary curves, snap behavior, instances, desktop/GPU or original Matrix behavior.
