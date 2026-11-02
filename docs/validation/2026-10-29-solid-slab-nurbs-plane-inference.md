# Slab NURBS source-plane inference — 2026-10-29

The hidden Slab session formerly inferred the source plane and checked InCPlane compatibility from 65 sampled points. A narrow knot span could be missed. The session now infers the normal from the entire validated, world-transformed NURBS control hull and checks that hull for InCPlane compatibility. This remains conservative for trimmed or nearly planar curves; source-normal fitting uses the existing scale-relative plane threshold. The sampled wire preview and independently fitted offset remain approximations.

A degree-one NURBS has a spike absent from all 65 samples, but lies in the XZ plane: the session now derives its Y normal and rejects InCPlane=Yes for the XY construction plane. A spatial variant with a second out-of-plane pole is rejected when starting the session. Both preserve the document and create no Undo transaction. The spike is planar in XZ, so this evidence does not assert that it is globally nonplanar.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`: `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 17 tests; `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 10 simulated viewport tests. Public Slab, periodic/closed NURBS, global fit and topology, and original Matrix behavior remain open.
