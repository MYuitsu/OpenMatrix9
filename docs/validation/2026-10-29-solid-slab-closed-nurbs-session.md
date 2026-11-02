# Slab closed NURBS development session — 2026-10-29

The hidden session captures closed clamped nonperiodic NURBS, computes the picked offset using the existing knot-aware approximate nearest-point search, and previews two separate sampled rings rather than connecting them across the seam. Side acceptance runs the sample-checked closed NURBS validation; height acceptance publishes through the source-stamped adapter. Preview is sampled and is not the exact fitted commit geometry.

Controller fixtures use a rational ellipse, inward/outward offsets, BothSides and both traversal directions. They check that preview rings are not falsely joined, compare measured volume to an independent ellipse-perimeter formula and Undo. A simulated viewport fixture selects the ellipse after session start, sets numeric distance, picks side and height, checks measured volume within 3 mm³ pixel-pick tolerance and Undo. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 28 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 13 tests.

Fit, topology and nearest-point checks are sampled/approximate, and preview can display an offset rejected on acceptance. Periodic curves, other shapes, performance, real desktop and original Matrix behavior remain open; Slab stays disabled publicly.
