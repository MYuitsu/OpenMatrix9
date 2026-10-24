# OM9-L3-03-18-009 — Extrude Curve Straight

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `ExtrudeCrv 5`
- **Feature:** `OM9-SOLID-009`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-009/OM9-L4-03-18-009-03.focused.log` (`4b4e06bfae9a4aaa6fab6ba18665e1d4553b4009a630d556a2b38d771996d48c`)
- `implementation/evidence/M03/OM9-L3-03-18-009/OM9-L4-03-18-009-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-009/OM9-L4-03-18-009-03.ui-review.json`
