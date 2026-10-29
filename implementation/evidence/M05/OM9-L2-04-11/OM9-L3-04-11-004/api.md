# Tapered Baguettes Between Two Curves (`OM9-L3-04-11-004`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderTaperBagBetweenTwoCurves)` / `gvTaperBagBetweenTwoCurves` | opens the tapered builder |
| `ChannelParams` (taper, point_a, point_b) | drives the single-stone span and the two end widths |
| `tapered_baguette` | returns the stone, gap and station for the recorded feature |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).

## Verification

- Positive: one stone is measured between Point A and Point B on the first wall; the two widths are the local wall gaps, so the stone is tapered and bent
- Negative: a zero-length span is refused
- Gate: `OPEN` (M05-UI manual review open).
