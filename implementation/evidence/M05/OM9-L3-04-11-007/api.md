# Halo Cluster Edit Parameters (`OM9-L3-04-11-007`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `ClusterFeature` / `CLUSTER_EXTENSION_SLOT` | schema v1 persisted on the host entity (`om9.gem.cluster`) |
| `ClusterKind` | Halo/Cluster/Channel parameters are stored with the ordered child GemIds |
| `Controller::cluster_feature` / `gem_cluster_apply` | read and rewrite the plan; match children by index, delete trailing, insert new |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.cluster` (`ClusterFeature`, schema version 1) on the center gem or first wall curve.

## Verification

- Positive: the feature record stores typed params, host and ordered child GemIds under om9.gem.cluster; reopening a builder edits the plan and keeps the child ids that still exist
- Negative: a feature with a nil, duplicate or over-budget child list is refused
- Gate: `OPEN` (M05-UI manual review open).
