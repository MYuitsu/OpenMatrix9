# Top11 Trim: curve piercing surface or BRep

When the selected surface is not itself divided by the selected curves, or the selected BRep is a closed solid, Top11 Trim can use that surface/BRep to cut selected curves. The existing mutual-curve state recomputes intersections after each transaction; a surface or BRep remains unchanged in the covered cases. The old surface-region state is still used when the surface is divided and does not yet expose the selected curves as targets.

OCCT curve/BRep intersection can now return endpoint hits to Trim while the existing Split-facing API filters the outer endpoints. The Rust curve/surface solver also admits isolated endpoints; a wholly sampled potential surface overlap is refused rather than interpreted as isolated intersections. Sampling is not a proof of absence of partial overlaps or a globally complete intersection solver.

Controller fixtures cover a line piercing a typed NURBS surface and a planar BRep, two consecutive removals and two Undos, plus removing the interior of a line crossing a closed BRep box. An OCCT endpoint fixture checks the two API filters; a Rust fixture checks both isolated endpoints and overlap refusal. CMD selection is exercised through the controller, not physical viewport/desktop input.

Windows MSVC with pinned OCCT 8.0.1 SDK/runtime and `CARGO_TARGET_DIR=L:\om9-target`: core curve/surface/Top11 suites passed 46 tests; Top11/options/BRep passed 88 tests before the final box regression, then all 24 BRep tests passed including it; no-default Top11/options passed 65 tests; kernel editing/transform passed 77 tests; public source audit reported zero errors. Native code computes only bounded OCCT intersections; Rust owns session, selection and transactions.

SHA-256: `session/top11.rs` f2436358e95f5756c4bc037a8cf4bfbdc62bb600008082ccbe2fca1c2f1a73ae; `tests/top11_brep.rs` 0f8f0d34d33eb6721553b3b09d493308ff2771d350340f557ccf7f433bdb66e9; `curve_surface_ops.rs` 8f15d6f28a12d163e07f0fe6df63ae1d056095c2e82c25ef64604870fd1bdf42; `tests/curve_surface_ops.rs` 163f6adb569a718a958400fc0056c77b9a8fdc8f453b020cde445169fb5fa207; `intersect.rs` 5351239a36c215ba0f58bd5d144d56427dabd26a0c83a9832d624a780bc31fb1; `intersect_bridge.cpp` a22e13eecf4517208f7e3ce09f15a2bb1e4545a13581eea032948dd074190af5.

General mutual editing when both surface and curve can be cut, multiple surface/BRep targets, surface-region recomputation after cutter edits, general numerical overlap and tangent handling, .om9/3DM, performance and physical UI acceptance remain open.
