# OM9-L3-03-18-013 — Sphere

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Sphere 5`
- **Feature:** `OM9-SOLID-014`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-013/OM9-L4-03-18-013-03.focused.log` (`549add22e1aa9a684ac0a37951f0521167269cd43d3fa0f99ba48600e03ee301`)
- `implementation/evidence/M03/OM9-L3-03-18-013/OM9-L4-03-18-013-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-013/OM9-L4-03-18-013-03.ui-review.json`
