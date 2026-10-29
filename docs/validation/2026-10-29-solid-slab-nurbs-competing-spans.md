# Slab NURBS competing knot spans — 2026-10-29

A regression with two nearby horizontal spans shows why refining only the closest sampled station is insufficient: the former code returned offset 0.6 although the nearest short span gives 0.4. The hidden Slab session now refines each nonempty active knot span as well as the sampled station neighborhood, choosing the best result. Reversed parameter mapping is supported. An estimated operation budget rejects excessively costly searches.

The piecewise-linear NURBS fixture checks both curve directions in the controller preview, verifies the corrected 0.4 magnitude and leaves the document and Undo history unchanged. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 18 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 10 tests. The fixture does not establish successful publication for a cornered curve.

Golden-section refinement assumes local unimodality and is not a global closest-point proof for arbitrary higher-degree spans; equal-distance corner ambiguity, resource behavior on large curves, global offset/topology, and public Slab remain open.
