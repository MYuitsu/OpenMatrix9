# Baguette Channel Layout (`OM9-L3-04-11-003`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderTaperBagChannel)` / `gvTaperBagChannel` | opens the channel builder on two selected curves |
| `ChannelParams` | section, spacing, banking, flip/middle/straight, taper, Point A/B, Gem Top, auto spacing |
| `baguette_channel` | typed plan; width follows the wall gap, taper builds the profile, frame banks/flips |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).

## Verification

- Positive: a straight channel spans the wall gap and lays stones at the requested pitch; taper < 1 builds a tapered trapezoid profile while keeping Baguette/Gem identity
- Negative: a zero or negative wall gap is refused
- Gate: `OPEN` (M05-UI manual review open).
