# Slab NURBS knot-aware preview — 2026-10-29

The hidden Slab session used 65 uniform parameter samples for the NURBS preview, omitting a short knot span in the regression fixture. It now includes uniform samples and the start and midpoint of every nonempty active knot span, with domain and reversed-direction mapping. Each preview tangent uses its own actual fraction. Sample allocations are checked against limits.

The regression uses a degree-one planar curve with a narrow spike around parameter 0.002: all 65 uniform points miss it, while the source preview includes it. Controller and simulated viewport suites pass, including forward and reversed rational quarter-circle cases. This does not establish an error bound between samples, verify picking near narrow spans, or prove offset topology. The current closest-point search remains separately sampled, and Slab is disabled in Menu/CMD.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 17 tests; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 10 tests.
