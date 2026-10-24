# OM9-L2-03-17 — 09-febe-matrix-surface-menu-workflows

- **Owner:** `OM9-L2-03-17` — 38 L3 / 114 L4
- **Date:** 2026-10-23
- **Status:** Sweep 1/2, Sweep 1 History, Sweep 2 History, Sweep 4, Profile Sweep, Sweep Multi, Merge, Patch, Planar, Variable Offset, Variable Blend, Shrink, Untrim, Chamfer, Fillet, Variable Fillet, Variable Chamfer, Edge Curves, Cutting Plane, Split, Drape, Tangent, Extrude All, Rail Revolve, Match, Seam, Soft Edit, Tween, Extend, Rebuild, Blend, Loft, Network, Unroll, Offset, Extrude Normal and Plane from Corners are functional with scene preview/commit; all 38 rows are functional with Sweep History Closed/Flip/Simple/Add Slash options, Four Rail Profile command and FourRailProfile metadata. Acceptance records are READY_FOR_REVIEW (M03-UI).

Rebuild, Unroll, Offset, Extrude Normal and Plane from Corners run through `surface_edit` and
`om9_core::surface_ops`; the scene now tessellates surfaces for display/picking without mutating
document geometry. The other 3 rows stay disabled: every row resolves to one stable command id,
names its owner candidate and reports why it is disabled. Fake success is explicitly rejected.
History rows share the base authored icon but keep distinct command ids.

## Evidence layout

- `implementation/design/M03/OM9-L3-03-17-00X/{dispatch_contract.json,wire_ui.json,source_crosswalk.json,api.md}`
- `implementation/fixtures/M03/OM9-L3-03-17-00X/{positive.json,negative.json}`
- `implementation/evidence/M03/OM9-L3-03-17-00X/` — one receipt per planned L4 role.

## Verification

```
cargo test -p om9-app --test surface_menu
cargo check -p om9-app -j 2
cargo clippy -p om9-app --test surface_menu -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/surface_menu.rs crates/om9-app/src/lib.rs crates/om9-app/src/ui/chrome.rs crates/om9-app/tests/surface_menu.rs
python tools/public_source_audit.py
```

## Limits

- 38 commands have no interactive command session yet; they stay disabled with the owner candidate and reason.
- The M03-UI manual review of click paths/previews/warnings is unsigned.
- Existing surface geometry services are not duplicated in the UI and do not enable a row by themselves.
