# OM9-L2-03-16 — 08-febe-matrix-curve-menu-workflows

- **Owner:** `OM9-L2-03-16` — 58 L3 / 174 L4
- **Date:** 2026-10-23
- **Status:** 55 functional rows, 3 verified disabled rows with visible reasons (Curve alias, Sketch on Mesh, Curve 2 Views with History); all acceptance records READY_FOR_REVIEW (M03-UI).

Every documented Curve Tools icon now resolves through one catalog row: stable command ID, owner candidate,
disabled reason and one transaction policy. The menu never reports fake success: a row without an
interactive command session is disabled and shows the missing-session reason. Line and Point share the
existing command session and are covered by `crates/om9-app/tests/curve_menu.rs`.

## Evidence layout

- `implementation/design/M03/OM9-L3-03-16-00X/{dispatch_contract.json,wire_ui.json}`
- `implementation/evidence/M03/OM9-L3-03-16-00X/` — one receipt per planned L4 role.

## Verification

```
cargo test -p om9-app --test curve_menu
cargo check -p om9-app -j 2
cargo clippy -p om9-app --test curve_menu -j 2 -- -D warnings
rustfmt --edition 2024 --check crates/om9-app/src/curve_menu.rs crates/om9-app/src/lib.rs crates/om9-app/src/ui/chrome.rs crates/om9-app/tests/curve_menu.rs
python tools/public_source_audit.py
```

## Limits

- `Curve` top menu maps all 58 rows; 55 enabled rows dispatch through their existing session and 3 remain disabled with a visible owner/session reason.
- The M03-UI manual review of click paths, previews and warnings is unsigned.
- Existing geometry services are not duplicated in the UI; an unverified owner candidate does not enable a row.

