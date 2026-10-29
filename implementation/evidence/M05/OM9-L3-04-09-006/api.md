# Gem Instance Create Rotate And Position (`OM9-L3-04-09-006`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(OM9-GEM-033)` / `gvGemPositioner` | positioner |
| `GemSession` positioner fields | `target`, `moved`, `rotation_deg`, `scale`, `duplicate` |
| `GemInstance::transform` | Rz*Ry*Rx, re-orthonormalized basis, validated placement |
| `apply_gem_positioner` | Placement mutation for move/rotate; DefinitionInsert + Geometry(Instance) for a duplicate so copies share one definition |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: move/rotate keeps identity; a duplicate shares one definition and undoes as one step
- Negative: a non-gem and a definition-owned instance cannot be edited in place
- Gate: `OPEN` (M05-UI manual review open).
