# Gem Facet Outline Girdle Reference API (`OM9-L3-04-09-008`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `GemReferenceGeometry` | `name`, `points`, `closed`, `normal`; no triangle index is exposed |
| `gem_reference(spec, name)` | `girdle`, `table`, `culet`, `profile`, `crown_facet:<n>`, `pavilion_facet:<n>` |
| `GemGeometryReference` | `gem: GemId` + the named geometric reference for the current mesh |
| `girdle_outline` / `table_outline` / `profile_points` / `crown_facet` / `pavilion_facet` | finite points and a stable outward normal |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: named girdle/table/facet references with finite stable outward normals
- Negative: unknown names/indices and degenerate facets are typed refusals
- Gate: `AUTO_ONLY` (automated evidence only).
