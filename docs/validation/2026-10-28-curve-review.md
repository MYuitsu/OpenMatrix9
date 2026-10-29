# Curve review — 2026-10-28

Scope: standalone Rust, M03 `OM9-L2-03-16`, 58 documented Curve menu rows. This is a partial behavioral review, not M03-UI acceptance or proof that every option matches Matrix. The full row inventory is in the milestone L2 and `crates/om9-app/src/curve_menu.rs`.

## Findings

- All 58 rows are marked `Implemented` in the current catalog. The five `curve_menu` tests pass, covering catalog/icon lookup, alias resolution and dispatch entry. The dispatch test accepts errors other than `Unsupported`; it does not operate each command or prove geometry, option behavior, mouse clicks, preview, cancellation or undo. Manual M03-UI acceptance remains unsigned.
- Extend Curve (`OM9-L3-03-16-027`): `extend_curve.rs` samples the source and commits a separate straight line, using a sampled endpoint direction. It offers Length, Start, End and Cancel; the milestone lists other variants including By Arc and Curve on Surface. Two existing tests pass for the separate line behavior, not full variant parity. The catalog's missing-option metadata is a planning signal, not proof that every listed sibling is absent.
- Curve Boolean (`OM9-L3-03-16-037`): `curve_boolean.rs` attempts union of two closed curves and otherwise joins connected curves; inside testing projects to world XY. It commits each resulting piece separately, so multiple pieces can produce multiple Undo steps and partial publication if a later commit fails. Four existing tests pass for connected curves and pairs of circles; other planes, region choices, atomicity and wider input sets are not established by those tests.
- `python tools/audit_m03_l2_03_16.py` reports 116 stale hashes: 58 receipts each cite older `icons.rs` and `ui/chrome.rs` hashes. This is an evidence freshness failure, not itself proof of a functional bug.
- Root README's 2026-10-23 paragraph claims only Line and Point are functional and 56 rows disabled; the current catalog and 2026-10-26 module record say 58 rows are wired. Neither statement establishes full behavior. Preserve historical claims but do not use the old count as current status.
- The dashboard records 60 missing option catalog entries across 51 L4 parameter records. Some entries recur across sibling rows and some now have implementations (e.g. Circle and Ellipse modes exercised by current tests); these are not 60 independently verified product defects.

## Run

Windows x64, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

`cargo test -p om9-app -j 2 --test curve_menu --test polyline --test polyline_shortcuts --test circle --test arc --test ellipse --test extend_curve --test curve_boolean --test tween_curves --test uv_curves --test curve_soft_edit --test angle_isocurve`

Result: 85 passed, 0 failed, 0 ignored. This test selection does not cover all 58 commands. No application UI run, geometry parity matrix, or human M03-UI signoff was performed in this review. No source behavior was modified.

## Next

For each row, inspect its exact milestone contract and current option handler, run menu/mouse and typed CMD with representative geometry, preview/commit/cancel and Undo, then document deviations and target-specific evidence. Start with Extend and Curve Boolean; include non-XY planes and one-transaction failure cases. Refresh the stale receipts only after revalidating the current sources; do not merely replace their hashes.
