# OM9-L3-03-18-005 — Cap Planar

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Cap`
- **Feature:** `OM9-SOLID-005`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-005/OM9-L4-03-18-005-03.focused.log` (`1233fa01e4c300ebf4a761cdb3dfd699fe3672dad1cbc992a9f9b680c138e4c4`)
- `implementation/evidence/M03/OM9-L3-03-18-005/OM9-L4-03-18-005-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-005/OM9-L4-03-18-005-03.ui-review.json`
