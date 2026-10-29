# API contract — Eternity Band Builder

L3: `OM9-L3-04-03-001`

| API | Behaviour |
|---|---|
| `ring_builder::build(Eternity)` | one watertight band solid from `diameter_mm`, `width_mm`, `thickness_mm` |
| `ring_builder::stone_frames` | `stone_01..stone_NN` placement frames on the outer surface; no gem solid |
| `BuilderKind::EternityBand` | fields `diameter_mm`, `width_mm`, `thickness_mm`, `stone_count`, `stone_diameter_mm` |
| `RingParams::validate` | zero stones -> `Invalid(\"ring stone count\")`, crowded ladder -> `Invalid(\"ring stone spacing\")` |
| Handles `diameter`/`width`/`wall`/`stone` | `BuilderHandle` grips on the band silhouette; a drag calls `adjust_field` on the same field the panel edits |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
