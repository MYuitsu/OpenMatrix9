# API contract — Award Ring Builder

L3: `OM9-L3-04-03-004`

| API | Behaviour |
|---|---|
| `ring_builder::build(Award)` | signet construction with the wider 10x12 mm top and a 2.5 mm shoulder |
| `BuilderKind::AwardRing` | same field set as the signet, different defaults |
| Handles | same pad grips as the signet, painted on the wider 10x12 mm top |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
