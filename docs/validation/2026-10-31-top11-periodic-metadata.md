# Top11 Join: periodic surface metadata — 2026-10-31

Status: partial automated evidence; Top11 and M03-UI are not fully accepted.

For validated repeated-pole/weight and shifted-knot NURBS bases, the Rust adapter still constructs the extended nonperiodic basis, then the selected OCCT surface adapter converts the certified axes to OCCT periodic axes before making a face. The native exception is limited to OCCT surface representation and bounded BRep snapshot; Rust validates the basis and owns the model. This supersedes the earlier periodic-surface record's observation that the resulting OCCT surface was nonperiodic.

Native tests directly check U, V and both periodic axes, and sample agreement with Rust evaluation (three points on each untrimmed fixture, five for one U-trimmed interval). The two-axis fixture tests conversion and sampled geometry, not Join. Two U-periodic rational surfaces were joined through CMD and toolbar: two faces, one shell, five edges, two naked edges, preserved area, Undo/Redo and .om9 roundtrip. Both extracted faces retain U periodicity before and after the archive roundtrip. The explicit periodicity query requires exactly one face.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 83 selected OCCT kernel tests, 80 core curve/surface/semantics/Top11 tests, 188 app Top11/options/BRep/snapshot tests and 107 no-default Top11/options tests passed. No-default compilation has existing unrelated warnings. OCCT adapter is the project's pinned SDK; this is not an openNURBS measurement attempt.

This evidence does not establish arbitrary periodic encodings, every trimming/sewing configuration, 3DM export interoperability, native peak allocation, or human physical viewport acceptance. Desktop pointer input previously failed with GetCursorPos access denied; M03-UI remains unsigned.

Source SHA-256 at validation:

- `crates/om9-core/src/geometry.rs`: `6b5d061abedb05d6828e8c76f382c4c0f6c21d87bf40eb8092cdd8322487fa38`
- `crates/om9-kernel/src/measurement.rs`: `ac3fc280a06e65b087307120d02e8c2e29418f6208f885c588cbf3cbaecccc9f`
- `crates/om9-kernel/src/measurement_bridge.cpp`: `8b5e9fc47f39c32d5b325b1601b0cdfd028ab495c1a84c450c75360ad1d82b5c`
- `crates/om9-kernel/tests/l2_02_05.rs`: `5c72bca2c177654f69591719ad8fff0a8cb1c46033fd0a6634fee8ef268712fb`
- `crates/om9-app/tests/top11_brep.rs`: `5262848c3b207aaec9f020c3869820149443a77149dddd37d6120d17289a1d13`
