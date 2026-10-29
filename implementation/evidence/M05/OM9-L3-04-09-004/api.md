# Gem Custom Shape From Profile (`OM9-L3-04-09-004`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderCustomGem)` / `gvCustomGem` | opens the custom builder |
| `GemSession::selected_profile` | samples the single selected curve, requires planar and closed |
| `GemSpec::from_profile(profile, cut, params)` | centres the profile, rotates a tall profile into the X-long frame, stores it |
| `CustomGemParams` | facet count, crown angle, crown height, girdle, pavilion depth, pavilion rows, bulge |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: closed planar profile -> centred custom gem with the profile stored
- Negative: open, non-planar, self-intersecting and zero-area profiles are refused
- Gate: `OPEN` (M05-UI manual review open).
