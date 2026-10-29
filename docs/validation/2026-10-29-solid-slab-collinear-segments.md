# Slab disjoint collinear segments — 2026-10-29

The open-polyline Slab boundary intersection predicate previously treated separated collinear finite segments as touching because orientation signs alone were insufficient. The predicate now also checks projected finite segment bounds, dropping the dominant normal axis; overlapping or touching segments remain rejected.

A zigzag with two separated collinear source segments was rejected before the change. Horizontal and vertical-plane fixtures now create closed single BReps with measured volume 24 mm³. An overlapping collinear fixture and the existing self-crossing case remain rejected. The fixture tests the core geometry and OCCT extrusion, not interactive source selection or desktop behavior.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 21 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 11 simulated viewport tests. Closed polygon sources, general topology robustness and the public Slab command remain open.
