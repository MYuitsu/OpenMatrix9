# API contract — Builder Live Preview Lifecycle

L3: `OM9-L3-03-11-004`

| API | Behaviour |
|---|---|
| `BuilderSession::refresh` | recompute the preview from fields only |
| `Controller::builder_cancel` | drop the preview, keep the committed document |
| `escape/Enter` | Escape cancels the preview, Enter commits |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
