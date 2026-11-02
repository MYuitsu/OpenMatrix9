# Sphere AroundCurve trimmed circular edge — 2026-10-29

A cylinder intersected with a box supplies an open circular BRep edge. Its recovered curve endpoints match independently queried native oriented vertices, and its midpoint matches the circle radius and endpoint-bisector oracle. The edge's parent receives a rotation and translation. Sphere AroundCurve evaluates fractions 0, 0.5 and 1, and projects an off-arc named-path pick; all produce the expected placed center and Undo restores the input.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_sphere_aroundcurve_trimmed_circular_edge_preserves_orientation_and_placement`: passed, four sphere fixtures on one open circular edge. The first test oracle incorrectly assumed a half-circle; OCCT split the edge at its seam. The corrected independent endpoint-bisector oracle passed.
- `cargo test -j 2 -p om9-app --test solid solid_sphere_`: 52 passed.
- `python tools/public_source_audit.py`: 14726 source files, 594 icon bindings, 593 SVGs, no errors.

One trimmed edge and one rigid placement are proven; seam-crossing arcs, both edge orientations, nonuniform placement, arbitrary curved edges, desktop/GPU and Matrix-original behavior remain open.
