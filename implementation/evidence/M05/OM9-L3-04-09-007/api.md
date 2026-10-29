# Gem Asset Properties And Layer Defaults (`OM9-L3-04-09-007`)

## Public surface

| Symbol | Contract |
| --- | --- |
| F6 `GemProperties` action | opens `GemPropertiesEditor` for one selected gem |
| `GemPropertiesEditor` | label, material, colour, layer, length/width/depth/girdle/corner, shared count |
| `gem_properties_apply` | one transaction: Label + Attributes + Geometry + Extensions as needed |
| `Controller::gem_layer` | current layer when it is in the Gem group, else the `Gem 01` preset, else current |
| `ensure_layer_writable` | locks/hidden Gem layers refuse the command: "gem layer locked"/"gem layer hidden" |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: one property transaction, Gem layer routing, no scene-wide recolour
- Negative: shared definitions and locked layers refuse edits
- Gate: `OPEN` (M05-UI manual review open).
