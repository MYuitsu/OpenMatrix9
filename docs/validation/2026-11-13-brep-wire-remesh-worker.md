# B-rep interior wires on the remesh worker

2026-11-13. Scoped I1 progress; the overall source/display plan remains open.

The explicitly requested, topology-attested root remesh path now derives default face isocurves from the captured immutable OCCT source on the existing worker. The worker samples the curves using the scene's NURBS display policy and interior knots, prepares render-only lines alongside the candidate mesh, and publishes only after the existing source, scene and proof freshness checks. Candidate source, curves, sampled lines, old/new buffer overlap and evaluation scratch have distinct reservation accounting. Callback checkpoints support cancellation between native clipping calls; they cannot interrupt one OCCT call already executing. Native temporary peak is not measured or proven bounded by managed counters.

The new controller test requests a remesh for an attested Box through the real request/poll path, confirms additional wire vertices and edges, checks sampled points against independently specified Box bounds/faces, refuses edge picks at the interior, compares wire points under two deflections, and checks document plus Undo/Redo. A second test forces insufficient display budget and verifies old scene publication, document and stacks remain. These are CPU scene/controller tests, not physical desktop or GPU-present evidence; point samples are not a global geometric error certificate.

- `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`, default features, OCCT SDK runtime on PATH: `cargo test -p om9-app -j 2 --lib remesh -- --nocapture`: **64 passed, 1 ignored**, 781 filtered; includes the new tests and existing stale/cancel/lifetime/remesh regressions.
- `cargo test -p om9-scene -j 2 --lib`: **47 passed**. Public source audit: 15,983 source files, 594 bindings, 593 SVGs, no errors. `git diff --check` passed with CRLF warnings.
- SHA-256: `topology_remesh.rs` `6a41a9bbe3210da8db0a1ecc218ae0ff3f6f953354e25fe9e692cdec01c8cfd3`; `session.rs` `25b04fb22ab3d23c53ba61a2b5309c944350408d583bf7b7e5126d5bdead7f3a`; `scene.rs` `200d13d335181447f516e29adee9319db758046c820130c707d551e755893e11`.

This applies only to the worker path with an explicit display reservation. Ordinary scene construction and the alternative no-display-reservation remesh path still omit these lines; ExtractWireframe is not connected to them. Imported NativeBrep, blocks, arbitrary trims/periodicity and density, actual GPU/desktop, worker peak and physical OOM remain unverified. No claim is made that C0, I1, canonical ownership, cache eviction, cold restore, v5, downstream migration or M09 is complete.

Next: integrate the same source-bound representation into normal scene construction and ExtractWireframe, including native/instance paths, while preserving cancellation, allocation lifetime and topology-pick exclusion. Validate trim and placement through product paths, then continue C1–C5.
