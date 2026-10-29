# Orient To Gem (`OM9-L3-04-11-006`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderOrientToGem)` / `gvOrientGems` | opens the orient tool on a gem + object selection |
| `OrientOptions` | scale_to_gem (guide gem display at 10 mm; the object keeps its scale) and the gem-frame offset |
| `orient_to_gem` | returns the exact gem basis/origin placement; the app commits Mutation::Placement |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).

## Verification

- Positive: the object placement reuses the gem X/Y/Z basis and origin exactly; offset along the gem axes moves the object origin in the gem frame
- Negative: a zero or non-finite object scale is refused
- Gate: `OPEN` (M05-UI manual review open).
