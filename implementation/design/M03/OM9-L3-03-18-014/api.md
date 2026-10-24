# OM9-L3-03-18-014 — Ellipsoid

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Ellipsoid 2 3 4`
- **Feature:** `OM9-SOLID-013`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-014/OM9-L4-03-18-014-03.focused.log` (`2c43a7fd1c00a2abaf66ce95316410555cfd4ab93b2f28c6da086668551c0083`)
- `implementation/evidence/M03/OM9-L3-03-18-014/OM9-L4-03-18-014-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-014/OM9-L4-03-18-014-03.ui-review.json`
