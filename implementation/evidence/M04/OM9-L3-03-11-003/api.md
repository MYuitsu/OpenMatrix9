# API contract — Builder Parameter Controls And Sliders

L3: `OM9-L3-03-11-003`

| API | Behaviour |
|---|---|
| `BuilderField::step` | quantisation in millimetres |
| `BuilderSession::adjust_field` | typed/step delta |
| `BuilderSession::set_slider` | ratio 0..1 mapped onto min..max then quantised |
| `Controller::builder_set_slider` | panel slider adapter |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
