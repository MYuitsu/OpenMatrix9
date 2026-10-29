# Gems Between Two Curves (`OM9-L3-04-11-005`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderGemBetweenTwoCurves)` / `gvGemsBetween2Crvs` | opens the between-curves builder |
| `gems_between_curves` | reuses the channel plan; the stone width follows the wall gap |
| `ChannelParams` | Mode=Spacing/Shape is `auto_spacing`; flip and Gem Top keep their channel meaning |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).

## Verification

- Positive: round section stones take the wall gap as diameter and touch both walls; the same channel fit/auto-spacing engine is reused
- Negative: a non-round section with a gap smaller than the stone is refused
- Gate: `OPEN` (M05-UI manual review open).
