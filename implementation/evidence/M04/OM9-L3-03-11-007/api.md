# API contract — Builder Profile Selector Window

L3: `OM9-L3-03-11-007`

| API | Behaviour |
|---|---|
| `ProfilePreset::ALL` | rectangle, rounded_rectangle, circle |
| `Controller::builder_presets` | library rows for the selector window |
| `Controller::builder_set_preset` | preview-only selection |
| `Controller::builder_remove_preset` | drop a preset from this session library |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
