# Top11 Split Isocurve Shrink for NURBS — 2026-10-31

The Split reference describes Shrink as reducing the underlying surface near the resulting trim boundaries. Previously Top11 Isocurve did not expose it. For a nonperiodic typed NURBS surface, the point stage now offers Shrink=No/Yes through the shared command-option interface; No is the provisional OM9 default. Yes restricts each result exactly by rational knot insertion along U and V, keeping its parameterization and shape. Original attributes and placement persist, and all resulting surfaces are one Undo transaction. Periodic typed surfaces and BRep faces do not offer Shrink yet; ordinary object-cutter Split also does not yet expose it.

The Rust operation validates the domain and input, refuses periodic surfaces, and accounts for the input, retained rows and output arrays against limits before allocating result arrays. It reuses the existing exact NURBS curve splitter for each control-net row. It does not invoke the pre-existing sampled/refitted shrink function.

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- Two core fixtures verify a rational quadratic surface and a surface with an existing internal knot; values sampled on the boundaries and interior match the original within 1e-9. One restricted point budget is refused. `cargo test -j 2 -p om9-core --test surface_ops`: 20 passed.
- The new Shrink option test initially failed because the option was absent. It now checks option activation and invalid text, restricted knot extents, unchanged rational evaluations, `.om9` save/open and Undo/Redo. A simulated viewport pick with Shrink=Yes verifies the resulting domain and knots.
- `cargo test -j 2 -p om9-app --test top11 --test top11_options --test top11_brep --test command_snapshot`: 124 passed. The no-default Top11/options/snapshot suites passed 68 tests.

Exploration: `ShapeUpgrade_FaceDivide` with surface-segment mode left a planar BRep's underlying surface bounds unchanged in a local fixture; its area alone remained correct. The experimental adapter was removed and Shrink is not offered for BRep faces until the underlying extent and trimming curves can be proven correct. This was an OCCT experiment, not an openNURBS measurement batch.

Limits: periodic/reversed knot edge cases, nonsmooth or near-degenerate surfaces, comprehensive aggregate transient-memory/performance, Shrink for BReps and object cutters, viewport ray accuracy, 3DM, SDK plugin parity and physical M03-UI acceptance remain open. Matrix's default Shrink value is not established by the local reference.

SHA-256: `surface_ops.rs` 64afaf1ba616f7d6aa635cb7311e0a4c18f7d0a283d1be7fb07e3ecf104760da; `tests/surface_ops.rs` 2d24ef136ff87886e6e89512435db5f4cb29ceb1078e7da979a584aa96047e9b; `session/top11.rs` 8d2313bdc6c17c77700fb6a2578a0b619d5f029ea1b5677550467043b8a35140; `session/command_options.rs` aab3169991ad0420c046bea4c9e2fc5904a787cada60752e30c30f89fe8ac063; `ui/chrome.rs` 62d6c216a7f9b6b127a9b039c941a54b8c63744d1314efb774d58814c6f98a38; `tests/top11_brep.rs` a32f06474816563761466d9a217e92bc0a8bfe3f45b42b90fb1a56adb7ae5c9c.
