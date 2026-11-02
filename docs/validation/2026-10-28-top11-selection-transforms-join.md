# Top 11 selection, transforms and BRep Join — 2026-10-28

Standalone Rust, Windows x64. This batch advances `OM9-L2-03-08`; it does not complete the module or its M03-UI human gate.

- When no objects are selected, the ten object-dependent Top 11 actions can start an object-selection stage. Viewport picking uses the existing Top 11 object-pick route; Enter continues into the action after selection and its ordinary availability validation. Tests open and cancel the selection stage for all ten actions; Move is exercised through commit and Undo, Mirror through Cancel. Actual mouse/UI testing, deselection modifiers and full post-selection workflow tests for every action remain open.
- Rotate has `Copy=Yes/No`, accepts a numeric angle or reference points, and accepts CMD points through the shared coordinate parser. Tests cover direct and clickable Copy, signed reference rotation, Cancel and a one-transaction copy Undo/Redo. The decompiled menu text identifies the original `Copy=No` and `Copy=Yes` scripts; alternate pointer/F4 binding is unverified.
- Mirror exposes 3Point, XAxis and YAxis plus Copy. X/Y reflect about the corresponding axis of the active CPlane. Tests cover translated/rotated CPlane, both axes, Copy=No with Undo/Redo, and default Copy with Undo/Cancel. Actual pointer clicks, BRep axes and all modes remain open.
- Join accepts exactly two BReps; it transforms them to world coordinates, uses the document's stated absolute tolerance for sewing, verifies face count, a single shell and a reduction in naked edges, and transforms the result back to the first source's local frame. Adjacent box faces, distinct placements, Undo/Redo, disjoint faces and missing tolerance have tests. The topology guard and native adapter have not been proven for all geometry, overlaps or non-rigid transformations. More than two BReps, raw surfaces and mixed selection remain unsupported.

## Validation

`CARGO_TARGET_DIR=L:\om9-target`; all Cargo commands used `-j 2`:

- `cargo test -p om9-app -j 2 --test top11 --test top11_options --test top11_brep`: 43 passed, 0 failed.
- `cargo test -p om9-core -j 2 --test top11`: 17 passed.
- `cargo test -p om9-app -j 2 --lib top11`: 4 passed; 193 filtered.
- Scoped rustfmt check and `python tools/public_source_audit.py` passed. Pruning removed zero runs.

Source SHA256: `session/top11.rs` 19cef87290c9c5623e5aab498eb9f7ab6191063f6b5aa96dc256e3199a223dea; `session.rs` 1819c4bb8d17689e10b031a8390205b649715732aaf2650824d9cb920c76f8af; `session/command_options.rs` 21180b92d32af754f7d24222b40c0154f5a5e980a0119bd10366c5f968fa16b7; `tests/top11.rs` e87c929add5ff313084b64338460d2a8283def2b292b429c84f2e54650989fbb; `tests/top11_brep.rs` 6ba1b475d1c1ee61a099afefe7c6ebd376b87e947efbb457c5e709391ca2652f; `tests/top11_options.rs` c882db794bc599a41eb85418cb9fa3f8cb9fa09b824407423c748c82f4ede605.

Still open: surface/polysurface and multiple-target Split/Trim, broader Join and Explode types, documented selection/deselection and all options, UI preview and click acceptance, and comprehensive Undo/Redo for all commands. Prior WIP reports record historical untested states; this report supersedes them only for the exact tested scope above.
