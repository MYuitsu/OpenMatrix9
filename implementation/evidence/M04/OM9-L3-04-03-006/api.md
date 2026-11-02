# API contract — Ring Builder History Updates

L3: `OM9-L3-04-03-006`

| API | Behaviour |
|---|---|
| `BuilderSession::adopt_history` | reopen the recorded feature instead of creating a second ring |
| `BuilderSession::plan` | the recipe carries the edited parameters |
| `Mutation::HistoryUpdate` | the same object id and recipe are updated in one transaction |
| Undo/Redo | the associative resize rolls back and forward |

## Invariants

- A builder preview never writes the document; only Apply opens one transaction.
- The family emits one closed mesh solid per ring; stone placement stays a frame list.
- Every refusal is a typed error: no silent clamp and no guessed default.
- The same producer serves the builder and the history rebuild (core and OCCT registries).
