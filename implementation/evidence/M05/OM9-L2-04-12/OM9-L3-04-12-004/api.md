# Pave Prong Layout (`OM9-L3-04-12-004`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderPaveProngBuilder)` / `gvPaveProngBuilder` | opens the Prong Builder on gems, surface optional |
| `ProngParams` | fillet, height, drop, taper, scale, shared/unshared/center min/max, touch fraction |
| `pave_prongs` | contact-graph plan with Shared/Center/Unshared kinds and clamped sizes |
| `Controller::gem_pave_apply` | one curve marker per prong; metal geometry belongs to M06/M07 |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: shared prongs sit between touching stones, center prongs between three; unshared stones receive boundary prongs and an isolated stone still gets one
- Negative: a prong layout without any selected gem is refused
- Gate: `OPEN` (M05-UI manual review open).
