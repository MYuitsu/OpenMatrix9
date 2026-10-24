# OM9-L3-03-18-020 — Truncated Cone

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `TCone 3 1 6`
- **Feature:** `OM9-SOLID-020`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-020/OM9-L4-03-18-020-03.focused.log` (`9a652eae39803dc15022f2113b65989114a6ef44c2751b024979a48ff4541f5f`)
- `implementation/evidence/M03/OM9-L3-03-18-020/OM9-L4-03-18-020-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-020/OM9-L4-03-18-020-03.ui-review.json`
