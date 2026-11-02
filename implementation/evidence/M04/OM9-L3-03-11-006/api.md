# API contract — Builder Handle And Panel Bidirectional Sync

L3: `OM9-L3-03-11-006`

| API | Behaviour |
|---|---|
| `Controller::builder_revision` | edit counter captured when a widget paints |
| `Controller::builder_set_field_at` | apply only if the revision is still current |
| `Controller::builder` | one parameter authority read by panel and handles |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
