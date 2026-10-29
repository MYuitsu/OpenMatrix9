# Slab NURBS control-hull planarity check — 2026-10-29

The hidden NURBS Slab previously checked planarity at 33 parameter samples. A degree-one NURBS with a narrow out-of-plane knot span can evade those samples and the session's 65-point preview sample. Before fitting an offset, the core now checks all control points against the requested plane and tolerance. Validated positive weights ensure the curve lies in the control hull, so an accepted hull bounds the full curve's plane deviation. This sufficient condition is conservative and can reject a curve whose control hull exceeds tolerance even if the relevant curve itself stays within it, especially for trimmed domains.

A regression uses a spike near parameter 0.002 that is absent from all 65 samples. It verifies the specific rejection, unchanged document, and no Undo transaction. Existing planar forward/reversed NURBS and viewport cases continue to pass.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 17 passed.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 10 passed.

The session's initial plane inference and preview are still sampled, and offset fit/topology are not globally certified. Periodic/closed NURBS and the public Slab command remain open. This is an OM9 development safeguard, not a claim about original Matrix behavior.
