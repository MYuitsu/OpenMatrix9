# API contract — Ring Builder Style Presets

L3: `OM9-L3-04-03-005`

| API | Behaviour |
|---|---|
| `BuilderStyle::snapshot` | records every typed field of the ring session |
| `StyleLibrary::save_user` / `save_to` / `load_from` | `om9.builder_style.v1` user file; unknown schema -> `Invalid(\"style schema\")` |
| `BuilderStyle::apply` | loads the preset into a fresh ring session; the preview and the body match |
| `PARAM_ALIASES` | ring parameters (`stone_count`, `top_width_mm`, `raised_*`, `shoulder_mm`) are style-addressable |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
