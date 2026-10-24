# OM9-L3-03-18-011 — Pipe

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Pipe 1`
- **Feature:** `OM9-SOLID-011`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-011/OM9-L4-03-18-011-03.focused.log` (`9c240ce82e4cc9928e0dcd37144839ccf649665c86d4e166826a6aaacfdda1cb`)
- `implementation/evidence/M03/OM9-L3-03-18-011/OM9-L4-03-18-011-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-011/OM9-L4-03-18-011-03.ui-review.json`
