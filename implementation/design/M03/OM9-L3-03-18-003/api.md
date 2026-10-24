# OM9-L3-03-18-003 — Boolean Intersection

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Command:** `BooleanIntersection`
- **Feature:** `OM9-SOLID-002`
- **Owner:** OCCT `om9-kernel`; app adapter `session/solid.rs`

## Behavior
Menu and command line resolve to the same feature id. The adapter validates the selection/arguments, calls the OCCT owner once, and commits one transaction with Undo.

## Evidence
- `implementation/evidence/M03/OM9-L3-03-18-003/OM9-L4-03-18-003-03.focused.log` (`5a9c2ce95cbc59b3e7d3291beeb483776ab4f96e94a7a9a3d606b324510b7674`)
- `implementation/evidence/M03/OM9-L3-03-18-003/OM9-L4-03-18-003-03.acceptance.md`
- `implementation/evidence/M03/OM9-L3-03-18-003/OM9-L4-03-18-003-03.ui-review.json`
