# Slab NURBS knot-aware side pick — 2026-10-29

The hidden session previously used a generic closest-point search seeded only by 65 uniform samples, so a short knot span could be missed when determining offset magnitude and side. The Slab session now chooses the nearest of its knot-aware sampled stations, refines within its neighboring fractions and compares the refined result to the original station. Preview and side acceptance use the same selection.

The planar degree-one spike fixture has a short span around parameter 0.002 missed by 65 uniform samples. A pick near that span produces the expected approximately 0.1 offset in the preview; document and Undo history remain unchanged. Existing forward and reversed rational quarter-circle session and simulated viewport cases also pass. This is still a bounded sampled search: a closer minimum between other stations, multiple minima, and ambiguous corners are not globally resolved or certified. The fixture validates preview side calculation, not successful publication for that spiked curve. Slab remains disabled publicly.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 17 tests; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 10 tests.
