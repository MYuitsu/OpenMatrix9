# API contract — Builder Mode Tabs And Context

L3: `OM9-L3-03-11-002`

| API | Behaviour |
|---|---|
| `BuilderKind::ALL` | five modes in tab order |
| `Controller::switch_builder` | move the active tab and keep each session's fields |
| `Controller::builder_tabs` | tab row with the active flag |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
