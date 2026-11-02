# Gem Shape Cut Dimension Schema (`OM9-L3-04-09-001`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `om9_core::gem::GemSpec` | `shape`, `cut`, `length_mm`, `width_mm`, `girdle_mm`, `crown_height_mm`, `pavilion_depth_mm`, `crown_angle_deg`, `facet_count`, `pavilion_rows`, `bulge`, `corner_mm`, `profile` |
| `GemSpec::validate` | typed refusals: shape-for-cut, non-finite, range, round diameter, width>length, depth, zero pavilion, cabochon dome, crown angle, bulge, facet count, corner, table ratio; never a clamped gem |
| `GemSpec::{to_json,from_json}` | `om9.gem_spec.v1` / version 1 round-trip, `deny_unknown_fields` |
| `GemSize` / `from_outer_dimensions` | the loader's length/width/depth entry point with the depth split |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: every cut x shape validates and JSON round-trips; dimensions are exact
- Negative: every malformed spec is a typed refusal, never a clamp
- Gate: `AUTO_ONLY` (automated evidence only).
