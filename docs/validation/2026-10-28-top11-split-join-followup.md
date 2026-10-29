# Top 11 Split, Join and selection follow-up — 2026-10-28

Standalone Rust on Windows x64. This advances `OM9-L2-03-08`; it is not completion of the module or its human M03-UI gate.

- Duplicate exposes InPlace at the base-point prompt and uses the existing duplicate owner. A test verifies the point geometry and one Undo/Redo.
- The controller can remove a picked selection or a Split/Trim cutter; the viewport routes Ctrl-click to this operation. One controller test removes and reselects a target and cutter, splits a line, and undoes it. Actual Ctrl-click UI was not tested.
- Split Point supports one preselected straight line. World-space projection is analytic, requires the document's explicit absolute tolerance, and rejects points outside the interior or tolerance. Tests check placement, typed world point, invalid retry, geometry lengths, Undo/Redo, alias/Cancel and non-line refusal. General curves, closed curves, multiple points, post-invocation selection for the alternate and right-click binding remain open.
- Join can accept three or more selected BReps. The Rust session transforms inputs to world space, tries bounded pairwise OCCT sewing in a connected order using document tolerance, and publishes only after all members join. Tests cover three box faces, Undo/Redo and failure with a disconnected member. The face/shell/naked-edge checks are not proof for every topology; complex compounds, mixed input types, non-rigid placements, large input stress and actual UI remain open. The existing two-input path is also tested.

The bounded recovered `modTopMenu` listing associates Split with `! _Split` and the alternate with `! _Split _P`; it does not establish the mathematical implementation. `creation::measure`, `editing::sew`, and the existing curve splitter remain the owners; no native bridge was changed.

## Validation

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`: `cargo test -p om9-app -j 2 --test top11 --test top11_options --test top11_brep` passed 26 + 10 + 14 = 50 tests. The batch was rerun after formatting the final source. Scoped rustfmt check and `python tools/public_source_audit.py` passed; artifact pruning removed zero runs. A subsequent filtered app library test is waiting for a build lock and is not counted here.

Source SHA256 at validation: `session/top11.rs` b8b06d43258fc877cae365182f73f1c012f9eb0a22533af6c80529c6577e74d7; `session.rs` 6a306f3b4b57f0633fdbb0837875b4b173d304118ee18a10e4f3d7fd9ce2cc94; `ui.rs` 748a0d086cea8e743baac9d7dec3ddb91067b8deb4f5d4b43bd7a41fd406d772; `tests/top11.rs` 2bf274f7c0413bbb65c573ad26157b6823368fe414d9720143220ced7575028f; `tests/top11_brep.rs` 7e460a68cc7ae802ab842ef1359871d6863c9dc6d0dd96f3323a9883c3723faf; `tests/top11_options.rs` c319330bde32b1f55f83a3eb6f13e2913e819f1c14558957442f6b2f7d9dbcde.

Previously documented WIP states for Duplicate and line Split Point are superseded only for the tested cases above. Multiple target Split/Trim, surface/polysurface targets, general curve Split Point and broader Explode/Join types remain incomplete. Full geometry, UI, Undo and Redo coverage for all Top 11 commands is not established.
