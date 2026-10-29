# API contract — Outside Ring Rail Generator

L3: `OM9-L3-04-02-002`

| API | Behaviour |
|---|---|
| `OutsideRailProducer::produce` | concentric outer circle at radius + offset_mm |
| `BuilderKind::OutsideRail` | offset_mm 0.1..50, one rail input |
| `BuilderHandle offset` | drag changes offset_mm |

## Commands

| Mode | Command | Inputs |
|---|---|---|
| Builder tab | panel row / `switch_builder` | selection that fits the mode |
| Command line | `RingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`, `Ring Resizer` | Enter applies, `Reset`, `Cancel` |

## Invariants

- Preview never writes the document; only Apply opens one transaction.
- A typed error leaves the committed document unchanged.
- Stable object ids and the recipe/result owner stay in the document.
