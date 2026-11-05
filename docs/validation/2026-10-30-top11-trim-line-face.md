# Top11 Trim: planar BRep face and line

With OCCT enabled, selecting one BRep face and one line starts a shared Trim point-pick state through CMD or the toolbar adapter. The face and line are transformed to world space for splitting and picking. The retained exact BRep is transformed back into the target's local space and committed as one geometry mutation; the cutter is retained. The tested selection is a single planar face and a coplanar straight line.

The application fixture uses independently translated face and cutter, verifies 140 mm² retained from a 200 mm² face, and checks the original placement, cutter, ambiguous boundary pick, Undo and Redo. A second fixture verifies cancellation and a missed cutter leave the document and Undo journal unchanged. The CMD fixture selects objects after invoking the command; the toolbar fixture preselects both. These are controller-level tests, not physical viewport or desktop validation.

With `CARGO_TARGET_DIR=L:\om9-target`, `cargo test -j 2 -p om9-app --test top11 --test top11_options --test top11_brep` passed 82 tests; after changing the cutter placement, `cargo test -j 2 -p om9-app --test top11_brep trim_` passed both relevant tests. The no-default Top11 and options suites passed 65 tests. `python tools/public_source_audit.py` found zero errors.

Source SHA-256: `session/top11.rs` 246805c8e4319b49e8eb63191b6745bbbd92bb826fb2a3f93d794b800af7725c; `tests/top11_brep.rs` 9b2831d02be264a1a0af0c94304249cc63826ab571b9710beb9c6382d8f8a760.

This does not implement general mutual Trim of surfaces or BReps: only two selected objects, one line cutter, and a single face are admitted in this branch. No multiple cuts or repeated face-region edits, arbitrary curves, general surface/BRep intersections, affine placement fixtures, actual viewport clicks, or desktop acceptance are proved. The curve is retained and is not offered as a trim target in this coplanar case.
