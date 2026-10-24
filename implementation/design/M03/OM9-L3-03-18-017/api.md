# OM9-L3-03-18-017 — Tube

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Tube 5 3 10`
- **Feature:** `OM9-SOLID-017`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-017/OM9-L4-03-18-017-03.focused.log` (`c590050b4eedc02725d7ca2d04bb014d1446d76f9cfc4953fc2509dc5fe20167`)
- `implementation/evidence/M03/OM9-L3-03-18-017/OM9-L4-03-18-017-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-017/OM9-L4-03-18-017-03.ui-review.json`
