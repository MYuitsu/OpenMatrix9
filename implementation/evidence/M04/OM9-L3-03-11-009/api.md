# API contract — Builder F6 And Commandline Input Binding

L3: `OM9-L3-03-11-009`

| API | Behaviour |
|---|---|
| `Controller::submit` | command-line builder names and field tokens |
| `Controller::begin_builder` | the same service the F6 prefill calls |
| `F6 prefill` | selects a builder from the selection |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
