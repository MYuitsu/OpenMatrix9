# Halo Builder (`OM9-L3-04-11-001`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderHalo)` / `gvHaloBuilder` | opens the Halo Builder on one selected center gem |
| `HaloParams` | accent, spacing/from-center/angle/rotation/height/placement, channel/prong parameters, count, fix/even/match |
| `halo_layout` | typed ring plan; equal-arc placement, closed-ring spacing check, 12:00/3:00 fix |
| `Controller::gem_cluster_apply` | one transaction for every accent insert/update/delete plus the host feature |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1).

## Verification

- Positive: auto halo closes the ring and every gap equals the realized pitch; Fix Position pins the first stone at 12:00; off starts at 3:00
- Negative: a requested spacing that cannot fit is refused (no squeeze)
- Gate: `OPEN` (M05-UI manual review open).
