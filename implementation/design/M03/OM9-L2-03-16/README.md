# OM9-L2-03-16 — 08-febe-matrix-curve-menu-workflows

- **Owner:** `OM9-L2-03-16` — 58 L3 / 174 L4
- **Date:** 2026-10-26
- **Status:** 58 implemented rows, 0 disabled placeholders; all acceptance records
  `READY_FOR_REVIEW` behind the unsigned `M03-UI` manual gate.

Every documented Curve Tools icon resolves through one catalog row: stable command ID,
owner candidate, authored icon, documented Matrix command alias and one transaction
policy. The Curve menu row, the palette icon and the typed command (menu label or the
documented Matrix spelling) dispatch the same Rust session; the command line can never
reach a different implementation than the menu.

## What the 2026-10-26 audit changed

- Three rows pointed at placeholder ids that no catalog contains (`OM9-CURVE-114`,
  `-141`, `-147`); they now use `OM9-CURVE-014`, `-042` and `-049`.
- Eleven icons in `crates/om9-app/src/icons.rs` mapped a row to a different command
  (for example `Curve` ran Point, `Sketch on Mesh` ran Polygon, `Continue Interpolated
  Curve` ran Interp Curve); every palette icon now maps to its own row.
- `CrvStart`/`CrvEnd` were swapped against the authored catalog (032/033); the ids now
  follow `features.json`, so menu and command line mark the same end.
- `Curve 2 Views with History` (`OM9-CURVE-014`) was a disabled placeholder; it now runs
  the two-view construction and records one history recipe over both view curves that
  `replay_history` rebuilds.
- Documented Matrix scripts (`_Fillet`, `_Chamfer`, `_CSec`, `_ExtractWireframe`,
  `_SoftEditCrv`, `_OffsetNormal`, `_gvExtractIsocurve`, `_Sketch _OnSurface`,
  `_Sketch _OnMesh`, `_Polyline _OnSurface`, `_Arc _StartPoint`) are accepted command
  aliases, and the visible menu label is accepted too.

## Evidence layout

- `implementation/design/M03/OM9-L3-03-16-00X/{dispatch_contract.json,wire_ui.json,
  source_crosswalk.json,api.md,dispatch_contract.md}`
- `implementation/evidence/M03/OM9-L3-03-16-00X/` — one receipt per planned L4 role.
- `implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/` — test logs,
  `receipt.json` and `audit.json`.

## Verification

```
cargo test -p om9-app -j 2 --test curve_menu --test crv2view --test crv_ends
cargo check -p om9-app -j 2
cargo clippy -p om9-app --test curve_menu -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/curve_menu.rs
python tools/sync_m03_l2_03_16_records.py
python tools/audit_m03_l2_03_16.py
python tools/public_source_audit.py
```

## Limits

- The M03-UI manual review of click paths, previews and warnings is unsigned; no row is
  promoted to full L3 acceptance by the automated evidence.
- Existing geometry services are not duplicated in the UI; the menu only dispatches.
- Per-command option coverage comes from the existing command test binaries, not from
  this row-level audit.
