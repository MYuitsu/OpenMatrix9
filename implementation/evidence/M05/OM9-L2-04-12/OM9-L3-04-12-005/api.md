# Pave Sphere Layout (`OM9-L3-04-12-005`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderPaveSphere)` / `gvPaveSphere` | opens Pavé Sphere with no mandatory surface |
| `SphereParams` | count or X/T, removal, gem/min size, radius, relax amount/strength |
| `pave_sphere` | Fibonacci sphere plan; removal patterns and projection onto the radius |
| `Controller::gem_pave_handles` / `gem_pave_adjust_handle` | viewport Radius and Count handles use the same typed fields |
| `Controller::gem_pave_apply` | inserts the planned stones in one transaction |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: Count or X/T produce a Fibonacci sphere layout on the typed radius; Removal None/A/B are typed and change the realized count
- Negative: a radius smaller than the stone is refused
- Gate: `OPEN` (M05-UI manual review open).
