# Slab closed NURBS preview guard — 2026-10-29

For closed NURBS, hidden-session preview now runs the same 65-point polygonal topology guard used before offset fitting at acceptance. An ellipse with inward Distance=4 mm is rejected by that guard: no preview is drawn, no document/Undo mutation occurs, and changing Distance to 0.5 mm restores preview. The guard runs in Rust without fitting or OCCT.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 29 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 13 tests. This guard only catches failures visible to the sampled approximation; fitting, global topology and OCCT can still reject a displayed preview. Its desktop responsiveness and general curves have not been benchmarked; public Slab remains disabled.
