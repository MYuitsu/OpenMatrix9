# Periodic double-seam Slab simulated viewport — 2026-10-29

A simulated egui viewport selects the certified rational cubic with double seam, sets Distance=0.25, picks the outside and height, measures a closed single BRep against an independently sampled source/offset area with 3 mm³ pixel-pick tolerance, and undoes to the original document. Both traversal directions pass.

The OCCT extrusion fixture additionally checks both directions at positive and negative heights and affine knot-parameter scales 0.01 and 1000; measured volumes agree with independently sampled area within 0.0001 mm³. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion` passed 3 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 16.

This is simulated input and one source family; arbitrary periodic representations, extreme parameter ranges, sampled offset topology, actual desktop and public Slab remain open.
