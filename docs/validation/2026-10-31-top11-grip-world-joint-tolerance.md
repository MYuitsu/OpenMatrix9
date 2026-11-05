# Top11 polycurve grips: world-space joint tolerance — 2026-10-31

Status: PARTIAL_CHECKS_PASSED. Earlier polycurve grip edits used a fixed 1e-9 local-coordinate coincidence threshold, so a joint accepted at document tolerance could be left behind, and nonuniform placement made that test inappropriate in world space.

The document/controller path now passes the validated absolute document tolerance and object placement into both edit- and control-grip joint operations. They compare the transformed endpoints in world coordinates, including the closing seam, and bring endpoints within tolerance together when a neighboring endpoint moves. Gaps outside tolerance are not treated as joints. A polycurve grip edit without document tolerance is rejected before publication. The lower-level geometry APIs retain their explicit identity/1e-9 behavior for existing standalone callers.

Core fixtures exercise both modes on scaled [10, 0.1, 1] objects: local gaps becoming inside or outside the 0.001 mm world tolerance, and missing tolerance. Controller fixtures cover an approximately closed seam, both modes, forward/reversed traversal and Undo/Redo. Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: core curve_ops/top11 42 passed; app Top11/options/BRep/snapshot 141 passed; no-default app 73 passed.

Complex/nested and near-singular placement, general trimmed/periodic segments, seam tangent continuity and smooth-curve fusion remain open. Physical pointer drag, 3DM and human M03-UI acceptance remain open. This evidence does not close all Top11 requirements.

Source SHA-256: `crates/om9-core/src/curve_ops.rs` ea7fbda8d87dcaeca5685f305039df8e063fffd8a43604fb8d2708acf4df255e; `crates/om9-core/src/top11.rs` 77ff667a92cf4ac5426a470a8cd268e16ac6dd3a0e82407f40c3004f546c6e0d; `crates/om9-core/tests/top11.rs` 9de8a9d0ca3a0a503f293cb2c30589a0cfdf6946c78c95a07b6dcc24c86ab0ec; `crates/om9-app/tests/top11.rs` 826e0f8dd1d09165bd65fb0b42076868456aef8c60d56f7572b53d98e852479c.
