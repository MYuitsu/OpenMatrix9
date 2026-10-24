# OM9-L3-03-18-008 — Text

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Text A 5`
- **Feature:** `OM9-SOLID-008`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-008/OM9-L4-03-18-008-03.focused.log` (`895ef550ca0c11d429b5e22cac705d721dbf221df1c4c29b407026114c172d74`)
- `implementation/evidence/M03/OM9-L3-03-18-008/OM9-L4-03-18-008-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-008/OM9-L4-03-18-008-03.ui-review.json`
