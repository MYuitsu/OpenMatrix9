# Pave Azure Builder (`OM9-L3-04-12-003`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderPaveAzureBuilder)` / `gvPaveAzureBuilder` | opens the Azure Builder on a surface + gems |
| `AzureParams` | hexagonal, angle, azure depth/size, through hole, seat angle, scale |
| `pave_azures` | typed 48-point circles or 6-point hex profiles per selected stone |
| `Controller::gem_pave_apply` | inserts one closed curve per profile; metal cutting belongs downstream |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: round azures emit top/through/bottom/seat profiles as closed curves; hexagonal azures emit six-point profiles; Angle rotates them
- Negative: azure placement without any selected gem is refused
- Gate: `OPEN` (M05-UI manual review open).
