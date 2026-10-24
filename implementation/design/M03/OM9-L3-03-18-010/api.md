# OM9-L3-03-18-010 — Solid Extrude All

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `ExtrudeAll 5`
- **Feature:** `OM9-SOLID-010`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-010/OM9-L4-03-18-010-03.focused.log` (`8ee7188ff11fe31bc6f1315f5485974c6a5bacaa20a096a9f4f2ea2dcaf1ba9c`)
- `implementation/evidence/M03/OM9-L3-03-18-010/OM9-L4-03-18-010-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-010/OM9-L4-03-18-010-03.ui-review.json`
