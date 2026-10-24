# OM9-L3-03-18-016 — Cylinder

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Cylinder 2 10`
- **Feature:** `OM9-SOLID-016`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-016/OM9-L4-03-18-016-03.focused.log` (`dad7db684873ac164947edf476c4d32040bb35cee42bb83612b9c9bfde2eec8b`)
- `implementation/evidence/M03/OM9-L3-03-18-016/OM9-L4-03-18-016-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-016/OM9-L4-03-18-016-03.ui-review.json`
