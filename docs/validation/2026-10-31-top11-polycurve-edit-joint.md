# Top11 Edit Points: connected polycurve joints — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. Previously moving one edit grip at a polycurve joint edited one segment only, leaving a gap. The reversed polycurve path also reversed the output traversal after editing.

The Rust edit operation now updates both segments at a coincident joint in one geometry result, including a closing joint, and preserves the original traversal and reversed flag. Each side is computed before the controller commits; a degenerate neighboring segment fails without publishing a partial change. A line-to-arc fixture exercises conversion of the arc to NURBS. No-op retains the original curve.

Fixtures cover either duplicate grip at a two-line joint, forward and reversed polycurves, a closed three-line seam, a line/arc joint, invalid edit rollback and controller Undo/Redo. Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: 38 core curve_ops/top11 tests and 139 app Top11/options/BRep/snapshot tests passed; the strengthened rollback test was rerun afterward; 71 no-default app tests passed.

The coincidence threshold is currently 1e-9 in object coordinates, not the document's world-space join tolerance. Larger allowable join gaps, nonuniform placements, complex or nested segments, tangent continuity, and Control Points at polycurve joints remain open. These fixtures do not prove Matrix's smooth-curve fusing behavior. Periodic NURBS, physical pointer drag, .om9/3DM and human M03-UI acceptance remain open; full Top11 is not complete.

Source SHA-256: `crates/om9-core/src/curve_ops.rs` d4358896174e7c9c2de292f3b552c1d484b2ac818cc4e19b240d70b7f7495a78; `crates/om9-core/tests/curve_ops.rs` 1208d9447637756931f4bb3c58f21cb9d35303ce01295dd3c340e5ff98c74ff1; `crates/om9-app/tests/top11.rs` b05d9e609c293ed73b2e455e80e6d262a8956222b8d93297251e5709ec4e17f7.
