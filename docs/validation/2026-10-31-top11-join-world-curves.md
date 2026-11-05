# Top11 Join: curve placement and document tolerance — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. The retained-gap limitation below was superseded for supported curves by [midpoint continuity](2026-10-31-top11-join-midpoint.md). Before this change, curve Join rejected different placements and used a fixed 1e-7 local-coordinate tolerance. The new regression initially failed with `Unsupported("join across different placements")`.

The Rust Join path now bakes each curve into world coordinates, orders and joins the curves using the validated document absolute tolerance, and publishes the result with identity placement and the chosen source's attributes. Nonuniformly transformed circular arcs use the existing exact rational conversion. The result preserves the tested world geometry; baking changes the representation and may change line or arc parameterization. As before, endpoints accepted within tolerance are not made exactly coincident, so downstream operations must not assume a perfectly closed seam.

CMD and toolbar fixtures cover differently translated/scaled lines, a gap within 0.001 mm, a gap above tolerance rejected without mutation, missing tolerance, Undo/Redo and `.om9` round trip. A nonuniformly placed arc is checked at 17 points against its expected ellipse. Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: core curve_ops/top11 44 passed; app Top11/options/BRep/snapshot 159 passed; no-default app 91 passed.

General curve geometry, branched/ambiguous ordering, exact seam healing, arbitrary numerical extremes, 3DM and physical M03-UI acceptance remain open. The native OCCT BRep Join path was not changed. No openNURBS measurement batch was performed. Full Top11 is not complete.

Source SHA-256: `crates/om9-core/src/top11.rs` d1116c12cfb5ca30f4363c72120f235bbf7fac9196052e959d11b4907a44808d; `crates/om9-app/src/session/top11.rs` 0e258244e538b07f781c4afa25a4da69bf1b719c0206bf5e6ec21e3a61300056; `crates/om9-app/tests/top11.rs` 1edfa0d27a0d5176becc94f0ac775c623cc6d8fdf919d628a584de3238ffebce.
