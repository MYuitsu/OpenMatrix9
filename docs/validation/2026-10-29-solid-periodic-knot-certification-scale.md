# Periodic NURBS knot certification scale — 2026-10-29

The repeated-periodic certificate previously compared knot-period differences with an absolute tolerance derived from max(period, 1). For a period around 4e-9 this admitted a deliberately nonperiodic 1e-13 knot discrepancy. The check now scales its tolerance to the period and floating-point roundoff at the knot magnitude; it refuses certification if numerical resolution is too coarse relative to the period. This is conservative and may reject some valid but poorly resolved records.

A fixture verifies a valid 1e-9-scaled periodic basis is admitted, a 1e-13 mismatch is refused, and an excessively narrow domain around 1e6 is refused for insufficient precision. Existing OCCT tests still pass at a 1e-9 scale and a 1e6 origin with 1e-4 scale. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: `cargo test -j 2 -p om9-core --test curve_oracles` passed 9, `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion` passed 3, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 32, and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 16.

This is a numerical admission rule for the tested repeated-pole family, not proof of all periodic representations or seam-parameter semantics. Desktop and public Slab remain open.
