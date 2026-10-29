# API contract — Builder Viewport Control Handles

L3: `OM9-L3-03-11-005`

| API | Behaviour |
|---|---|
| `BuilderSession::handles` | dimension handles with anchor, axis and field |
| `Controller::builder_adjust_field` | handle drag target |
| `BuilderSession::editable` | locked layer/object hides handles |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
