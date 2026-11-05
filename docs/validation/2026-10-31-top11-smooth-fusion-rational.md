# Top11 smooth control fusion: rational and budget regressions — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. This extends the earlier smooth-control-fuse record, not a claim of complete Top11 coverage.

Output pole, weight and knot buffers now have checked size arithmetic, point/byte admission against limits and fallible exact reservation before assembly. The temporary converted/elevated inputs are not yet charged together with the output, so this is not a complete aggregate peak-memory proof. A fixture sets a two-point budget and confirms that a fused three-pole result is refused without document mutation.

Two controller fixtures compare sampled fused geometry against the input pieces: a rational quadratic multi-span curve joined to a line under nonuniform scale and shear, including a reversed source record; and a rational degree-one multi-span curve elevated to degree two before joining a rational quadratic. They sample 101 fractions, exercise Undo/Redo, and the first saves and reopens `.om9`. These tests do not prove all degree elevation or all placements.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: core curve_ops/top11 43 passed; app Top11/options/BRep/snapshot 145 passed; no-default app 77 passed. Near-but-not-identical endpoints, periodic or trimmed curves, general geometry and aggregate budgets remain open. Control point selection and deletion, 3DM, physical pointer UI and M03-UI human acceptance remain open. No openNURBS measurement batch was run.

Source SHA-256: `crates/om9-core/src/curve_ops.rs` a55f840b5dd6f65de7ddd8c4abb0bfcb4920f2b65e36ea825c712742a35c832d; `crates/om9-core/tests/top11.rs` f6a162eea588aa1059f690a8f32733c39dff7203697ddcb98c59c1b986096383; `crates/om9-app/tests/top11.rs` b28a76f749cd343d632f700e5077d5ec4aec346de4e99828f57775ca72b118a3.
