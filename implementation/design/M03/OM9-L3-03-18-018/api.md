# OM9-L3-03-18-018 — Pyramid

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Pyramid 4 5 10`
- **Feature:** `OM9-SOLID-018`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-018/OM9-L4-03-18-018-03.focused.log` (`04dbc3c071a6961f32e7987e1e713f00c1b465999b037736062f33bafb56e03d`)
- `implementation/evidence/M03/OM9-L3-03-18-018/OM9-L4-03-18-018-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-018/OM9-L4-03-18-018-03.ui-review.json`
