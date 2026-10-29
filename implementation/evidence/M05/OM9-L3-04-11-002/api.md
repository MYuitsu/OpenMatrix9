# Cluster Builder (`OM9-L3-04-11-002`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderClusterBuilderOld)` / `gvClusterBuilder` | opens the Cluster Builder on one center gem |
| `ClusterParams` | accent, XY offset, Z offset (positive down), degree, count/maximum, start point, rings, ring spacing |
| `cluster_layout` | typed ring plan with start angle, degree rotation and two-ring offsets |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1) on the center gem or first wall curve.

## Verification

- Positive: Start Bottom/Top/Left/Right maps to 6/12/9/3 o'clock; positive Z offset moves the ring down (manual direction)
- Negative: a second ring without ring spacing is refused
- Gate: `OPEN` (M05-UI manual review open).
