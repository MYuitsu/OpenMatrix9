# API contract — Ring Builder Final B-Rep Validation

L3: `OM9-L3-04-03-007`

| API | Behaviour |
|---|---|
| `ring_builder::inspect_measures` | the one gate both representations run: closed and volume > 0 |
| `ring_builder::inspect_geometry` | mesh -> `inspect_solid`; B-Rep -> binding check then the same gate |
| `om9_kernel::creation::sewed_solid` | one planar triangle per quad corner, sewn into exactly one closed shell, free edges and volume refused |
| Open model | `Invalid(\"ring solid open\")` — a deliberately open face model is not production geometry |
| Thin model | `Invalid(\"ring wall\")` before a degenerate solid is built |
| `BuilderKind` wiring | the family is registered as builder modes and as style targets |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
