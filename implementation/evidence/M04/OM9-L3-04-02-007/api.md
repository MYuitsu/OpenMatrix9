# API contract — Ring Resizer Builder

L3: `OM9-L3-04-02-007`

| API | Behaviour |
|---|---|
| `RingResizerProducer::produce` | new independent ring at target_diameter_mm |
| `BuilderKind::RingResizer` | target 1..60 mm, thickness 0.1..10 mm |
| `Controller::builder_apply` | one transaction: entity + recipe + result |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
