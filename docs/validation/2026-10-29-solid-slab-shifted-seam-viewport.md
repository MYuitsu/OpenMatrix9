# Slab shifted-seam NURBS simulated viewport — 2026-10-29

A simulated egui viewport selects a nonperiodic full-domain rational ellipse whose seam is shifted to 0.17, then sets Distance=0.5, picks side and height, measures the result and undoes it. Both forward and reversed source directions pass. The resulting volume is compared to an independently integrated perimeter formula with 3 mm³ tolerance for pixel height picks; Undo restores the source including its seam.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 14 tests. This is simulated input, not desktop or public Menu/CMD validation. Genuine periodic NURBS remains unsupported by the relevant evaluator/extrusion path; sampled fit/topology and public Slab remain open.
