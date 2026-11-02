# Periodic double-seam exact extrusion — 2026-10-29

The rational cubic repeated-knot seam fixture previously formed a valid planar face but its one-edge prism failed OCCT face tessellation at deflections 0.001, 0.01 and 0.1. Simply splitting the original B-spline into two edge parameter intervals made tessellation succeed but produced grossly incorrect mass, so that approach was discarded.

The capability-scoped OCCT extrusion adapter now splits a single closed non-clamped B-spline with repeated endpoint knots into two exact, independently trimmed B-spline segments using GeomConvert. It builds the wire in reversed order when needed. The existing Rust feature/session, document and history logic remains in Rust.

Using the pinned OCCT 8.0.1 SDK/runtime, the double-seam cubic fixture now yields a closed single solid with volume matching independent 4096-station polygon area within 0.0001 mm³ in both directions. A hidden Slab fixture computes an independent sampled offset area, checks BRep volume within 0.02 mm³ and Undo. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion` passed 3 tests; `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 32; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 15.

The double-seam Slab fixture uses controller input rather than viewport/desktop. Other knot arrangements, precision scales, periodic representations and global offset topology remain open; public Slab remains disabled.
