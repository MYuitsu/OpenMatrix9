# API contract — Builder Accept Cancel Reset And Undo

L3: `OM9-L3-03-11-008`

| API | Behaviour |
|---|---|
| `Controller::builder_apply` | one Undo transaction per Apply |
| `Controller::builder_cancel` | bytewise no change |
| `Controller::builder_reset` | fields back to defaults, no journal entry |
| `Undo/Redo` | restores the applied builder feature |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
