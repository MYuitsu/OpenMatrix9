# Periodic double-seam extrusion parameter scale — 2026-10-29

A certified rational cubic double-seam fixture with knot scale 1e-9 initially produced a BRep whose OCCT mass integration failed with `surface integration subdivision failed`. The capability-scoped extrusion adapter now affinely normalizes the knot parameter domain before exact GeomConvert splitting; the source document and geometry are not changed.

The pinned OCCT 8.0.1 SDK/runtime fixture verifies closed-solid volume against independent sampled area at parameter scale 1e-9, at base 1e6 with scale 1e-4, and at the previously covered 0.01 and 1000 scales. It also checks translated geometry, both traversal directions and signed heights. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion` passed 3 tests; `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 32; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 16.

These are finite fixtures, not proof for arbitrary knot spacing, very large offsets, precision exhaustion or all periodic representations. The Slab fitting/topology checks remain sampled, actual desktop and public Slab remain open.
