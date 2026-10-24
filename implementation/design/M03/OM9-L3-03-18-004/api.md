# OM9-L3-03-18-004 — Boolean Two Objects

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Boolean2Objects U`
- **Feature:** `OM9-SOLID-004`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-004/OM9-L4-03-18-004-03.focused.log` (`60ed67e06fc979eb94cefe34f1d3abc58dbfc45cfd76d8954c5603a6de634e13`)
- `implementation/evidence/M03/OM9-L3-03-18-004/OM9-L4-03-18-004-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-004/OM9-L4-03-18-004-03.ui-review.json`
