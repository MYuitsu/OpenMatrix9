# API contract — Builder Typed Input Boxes

L3: `OM9-L3-03-11-001`

| API | Behaviour |
|---|---|
| `BuilderKind::input_roles` | declared slot order for each mode |
| `BuilderSession::new` | count and geometry-kind check before any field |
| `BuilderSession::inputs` | the stored (role, ObjectId) pairs |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
