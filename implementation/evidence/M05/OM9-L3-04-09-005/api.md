# Gem Emerald And Baguette Builders (`OM9-L3-04-09-005`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(OM9-GEM-013)` / `gvEmerald` | emerald builder |
| `Controller::dispatch(OM9-GEM-014)` / `gvBaguette` | baguette builder |
| builder fields | `Length=`, `Width=`, `Corner=`, `Depth=`, `Girdle=` |
| `GemSpec::{emerald,baguette}` / `set_girdle_mm` / `set_depth_mm` | explicit girdle thickness, exact total depth, corner <= half the short axis |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: emerald and baguette commit the requested length/width/corner/depth/girdle
- Negative: a degenerate girdle and an oversized corner are refused
- Gate: `OPEN` (M05-UI manual review open).
