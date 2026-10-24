# OM9-L3-03-18-001 — Boolean Union

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `BooleanUnion`
- **Feature:** `OM9-SOLID-003`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-001/OM9-L4-03-18-001-03.focused.log` (`6f1668da65e911b12896fbd8f921151febce4c11ef1655b5526666b08ef2e6d9`)
- `implementation/evidence/M03/OM9-L3-03-18-001/OM9-L4-03-18-001-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-001/OM9-L4-03-18-001-03.ui-review.json`
