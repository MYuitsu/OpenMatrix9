# OM9-L3-03-18-021 — Shell

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Shell 0 1`
- **Feature:** `OM9-SOLID-029`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-021/OM9-L4-03-18-021-03.focused.log` (`08b381d0bb800b1f9f7254e33f286ecd69c072432c612f120f54e21b1c306d81`)
- `implementation/evidence/M03/OM9-L3-03-18-021/OM9-L4-03-18-021-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-021/OM9-L4-03-18-021-03.ui-review.json`
