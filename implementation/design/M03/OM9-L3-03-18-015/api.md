# OM9-L3-03-18-015 — Torus

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Torus 5 1`
- **Feature:** `OM9-SOLID-015`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-015/OM9-L4-03-18-015-03.focused.log` (`5343744a987156a0905a196758e9610e6f3ea36380047896b1da95f514b8c4b2`)
- `implementation/evidence/M03/OM9-L3-03-18-015/OM9-L4-03-18-015-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-015/OM9-L4-03-18-015-03.ui-review.json`
