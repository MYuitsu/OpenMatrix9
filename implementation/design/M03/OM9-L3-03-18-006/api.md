# OM9-L3-03-18-006 — Extract Surface

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `ExtractSrf 0`
- **Feature:** `OM9-SOLID-006`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-006/OM9-L4-03-18-006-03.focused.log` (`c9895f40842a20c4ac6584332fb9e80ad1fe870223dd6fe6e7a82ad5b840a012`)
- `implementation/evidence/M03/OM9-L3-03-18-006/OM9-L4-03-18-006-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-006/OM9-L4-03-18-006-03.ui-review.json`
