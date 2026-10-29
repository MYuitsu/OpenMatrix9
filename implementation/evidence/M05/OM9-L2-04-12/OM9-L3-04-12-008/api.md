# Pave History Geometry Refresh (`OM9-L3-04-12-008`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `PaveFeature` / `PAVE_EXTENSION_SLOT` | schema v1 persisted on the host surface (om9.gem.pave) |
| `refresh_pave_layout` | re-evaluates stored UVs on the changed surface and reports conflicts |
| `Controller::gem_pave_refresh` | reloads the refreshed stations into the open builder |
| `Controller::gem_pave_apply` | matches children by station GemId or index; one transaction |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: the feature stores the host surface, typed params and ordered child GemIds under om9.gem.pave; refreshing a feature after reopening preserves every child id
- Negative: a feature whose UV left the new surface domain reports conflicts instead of silently dropping ids
- Gate: `OPEN` (M05-UI manual review open).
