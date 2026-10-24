# OM9-L3-03-18-012 — Box

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `Box 10 20 30`
- **Feature:** `OM9-SOLID-012`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-012/OM9-L4-03-18-012-03.focused.log` (`19005d5e31bd18b27f312ae900b0a7ded13f642726c60b2bab9d3fd0d8b59630`)
- `implementation/evidence/M03/OM9-L3-03-18-012/OM9-L4-03-18-012-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-012/OM9-L4-03-18-012-03.ui-review.json`
