# Slab closed-polyline foundation — 2026-10-29

The Rust geometry foundation builds mitered nested rings for a closed planar polyline. It checks planarity, degenerate or reversed offset edges, self-intersections, boundary intersections and containment; offset direction and input traversal determine which ring is outer. BothSides shifts the base and doubles extrusion. This is not yet connected to the document or interactive session.

The test extrudes both rings and subtracts them with OCCT, measuring closed single-solid results for a rectangle outward, inward and BothSides, plus a concave L polygon outward, each in both traversal directions. It verifies vertical-plane ring orientation and rejects over-collapsed inward and self-crossing fixtures. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 22 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 11 tests.

This does not establish robust behavior for every concave offset, tolerance, nested or multiple-loop case, nor reference Matrix behavior. Source capture, publication/Undo, viewport interaction, desktop and public Slab remain open.
