# OM9-L3-03-18-002 — Boolean Difference

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `BooleanDifference`
- **Feature:** `OM9-SOLID-001`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-002/OM9-L4-03-18-002-03.focused.log` (`3aa8f8b203bd6fa9412cc528b32d9492132cba57acebe21cfe189bbfa1a6dcc7`)
- `implementation/evidence/M03/OM9-L3-03-18-002/OM9-L4-03-18-002-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-002/OM9-L4-03-18-002-03.ui-review.json`
