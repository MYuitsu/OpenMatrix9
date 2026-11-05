# Top11 Split: single-face targets — 2026-10-31

The M03 Top11 Split reference calls for separate sets of targets and cutters, including curves, surfaces and polysurfaces. Before this change, Split rejected surface targets. The shared Split session now accepts typed surfaces and single-face open BReps as targets, including mixed curve/surface selections; curve, surface and BRep cutting objects are accepted. The existing OCCT surface splitter returns a world-space partition, whose faces are extracted and transformed back to the target frame. New entities inherit source attributes and placement. All target replacements are committed in one transaction; cutters and uncut targets remain intact. The Point option is not advertised for a surface target.

On Windows x64 with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- The new placed-face, two-cutter regression failed before implementation because Split rejected the target, and passed afterwards through both CMD selection and the toolbar controller route; it checks areas 60, 60 and 80 mm², placement, cutters, Undo and Redo.
- A mixed typed-surface and curve fixture checks both are split, an unhit surface is preserved, Cancel is non-mutating and Undo restores the document. A missed cutter is rejected without a transaction and leaves the session active. Unsupported solid targets are disabled/rejected.
- A BRep surface cutter produces two faces of 100 mm² each and Undo restores the source.
- `cargo test -j 2 -p om9-app --test top11 --test top11_options --test top11_brep --test command_snapshot` passed 107 tests (55 + 10 + 39 + 3); the additional BRep cutter test passed separately. `--no-default-features` on the first, second and fourth suites passed 68 tests before the final OCCT-only availability change. A simulated egui viewport click on a cutter passed separately.

Scope limits: Split of multiface shells/solids is not implemented: exploding the partition into all faces would destroy polysurface connectivity. Isocurve/Direction/Shrink options, general curved/periodic geometry, geometric tolerances, affine placements, 3DM, physical desktop interaction and M03-UI human acceptance remain unverified. Availability currently measures selected BReps to identify single-face targets; performance on large selections has not been assessed. Surface cutters in this batch cover a planar BRep fixture, not arbitrary cutters. OCCT is native and not certified memory-safe by its Rust caller.

Source SHA-256: `session/top11.rs` 29e26b30dcaf2df3287e689bdd1eba04d11c921a71dcf8d2c7225371cfaf3534; `ui/chrome.rs` 20d44a7a9ca81a5e02a93bb1e834d8fe225802a08826d94e11badf955800f21f; `tests/top11_brep.rs` 0ff853c1c2d9961016296c5089cbbdc397d5c09cc5e9acb63f651698b31bed76.
