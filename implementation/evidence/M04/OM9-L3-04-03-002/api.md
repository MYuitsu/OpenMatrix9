# API contract — Signet Ring Builder

L3: `OM9-L3-04-03-002`

| API | Behaviour |
|---|---|
| `ring_builder::build(Signet)` | band plus a smooth top pad; the shoulders taper into the band |
| `BuilderKind::SignetRing` | fields `top_width_mm`, `top_length_mm`, `shoulder_mm` on top of the band fields |
| `RingParams::validate` | `Invalid(\"ring wall\")`, `Invalid(\"ring shoulder\")`, `Invalid(\"ring signet top\")` |
| Top contour rule | `top_length_mm` must fit half the circumference, so the shoulders cannot fold |
| Handles `shoulder`/`top_width`/`top_length` | pad grips; each maps to its own field through `adjust_field`, so a drag and a slider cannot disagree |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
