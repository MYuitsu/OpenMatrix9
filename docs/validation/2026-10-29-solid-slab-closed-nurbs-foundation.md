# Slab closed NURBS foundation — 2026-10-29

The Rust foundation accepts a closed clamped nonperiodic planar NURBS of degree at least two, fits an offset, assigns inner and outer by traversal and signed area, and provides extrusion and BothSides base shift. Positive-weight control-hull planarity is checked. A 65-point polygonal source check rejects sampled crossings and collapsed offsets; 33 expected offset points are checked against the fitted curve with 0.0001 mm tolerance. These are sampled, one-sided checks, not global deviation or topology proofs.

A rational ellipse fixture (semi-axes 5 and 3 mm) builds the two BReps and subtracts them using OCCT. Outward and inward/BothSides offsets in both traversal directions agree within 0.02 mm³ with an independent numerical ellipse-perimeter formula. Invalid plane and excessive inward offset are rejected. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 26 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 12 tests.

This foundation is not connected to document publication or interaction. Periodic NURBS, general shape robustness, source/offset topology, actual desktop behavior and public Slab remain open.
