# Top11 Control Points: connected polycurve joints — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. Previously a control grip changed only its own polycurve segment and could open a shared joint; the reversed polycurve path also changed traversal direction.

The Rust control-grip operation now follows moved endpoints of a segment to coincident adjacent endpoints. Where the neighbor has an endpoint control pole, it moves that pole; otherwise it uses the neighbor's edit endpoint, which can convert a circular arc into NURBS. This handles an arc center or radius change affecting a joint as well as a directly moved line/NURBS endpoint. The controller publishes the result atomically; invalid neighboring geometry fails without changing the document. The original polycurve traversal and reversed flag are retained.

Fixtures cover either duplicate grip of a two-line joint, forward/reversed polycurves, a closed seam, line/arc connection including moving the arc center, rational NURBS endpoint and weights, controller rollback and Undo/Redo. Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: core curve_ops/top11 41 passed; app Top11/options/BRep/snapshot 140 passed; no-default app 72 passed.

Joining uses 1e-9 object-space coincidence, not document world-space tolerance. General placed, nested, periodic or trimmed segments and tangent continuity are unverified. Matrix references describe fusing smooth joined curves after control editing; this implementation preserves separate polycurve segments and does not establish that behavior. Physical pointer, .om9/3DM and human M03-UI acceptance remain open; full Top11 is incomplete.

Source SHA-256: `crates/om9-core/src/curve_ops.rs` 84d3c0f488990bdc6c703d70225e3461cbace6b1211e5e08453a0cb75cb5ab64; `crates/om9-core/tests/curve_ops.rs` fe7ca58e4b7da0fd78e883e9c6d05ac9dc4ad3775fd5e2114da9806bc4c3f580; `crates/om9-app/tests/top11.rs` 7f8b33adaed9239d8ac754482658eaa40c08acfcffaabd2668a5cde880f9aed2.
