# OM9-L3-03-18-019 — Cone

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Cone 3 6`
- **Feature:** `OM9-SOLID-019`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-019/OM9-L4-03-18-019-03.focused.log` (`38a8a86906d779fac53b8e3184a74c3dceee01a9f7186b424c6c523d947db3d9`)
- `implementation/evidence/M03/OM9-L3-03-18-019/OM9-L4-03-18-019-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-019/OM9-L4-03-18-019-03.ui-review.json`
