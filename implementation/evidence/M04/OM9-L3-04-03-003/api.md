# API contract — Raised Band Builder

L3: `OM9-L3-04-03-003`

| API | Behaviour |
|---|---|
| `ring_builder::build(Raised)` | band with an outward ridge over `raised_width_mm` |
| `BuilderKind::RaisedBand` | fields `raised_height_mm` (height 3, width 4 in the fixture) |
| `RingParams::validate` | a ridge deeper than `thickness_mm - MIN_WALL_MM` leaves zero inner wall: `Invalid(\"ring raised height\")` |
| Handle `raised` | the ridge grip sits on the ridge crest and cannot grow into the wall |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
