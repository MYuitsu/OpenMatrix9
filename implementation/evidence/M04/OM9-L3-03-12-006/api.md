# API contract — Builder Style Match Attributes

L3: `OM9-L3-03-12-006`

| API | Behaviour |
|---|---|
| `BuilderStyle::match_attributes(entity)` | copies the declared `om9.rail`/`om9.head` members with their types |
| `STYLE_ATTRIBUTES` | `width_mm`, `height_mm` (length) and `material` (choice) |
| `Controller::builder_match_attributes` | reads the selected entity, sets the matching session fields, leaves ids alone |
| `StyleMatch` | command-line route to the same service |

## Builders the styles drive

| Mode | Tag | Inputs |
|---|---|---|
| Ring Rail | `ring_rail` | none |
| Rail Profile | `rail_profile` | rail curve |
| Rail Sweep | `rail_sweep` | rail + profile curve |
| Outside Rail | `outside_rail` | rail curve |
| Ring Resizer | `ring_resizer` | ring curve |

## Invariants

- A style is a declarative record: applying it never writes the document, and the builder's own
  Apply stays the one transaction.
- Built-in styles are read-only; the user list is a separate file/list.
- An incompatible mode/family or a missing mandatory input is a typed error, never a silent no-op.
- Matching attributes keeps the source entity identity and its extension slots.
- A corrupt style file or thumbnail degrades explicitly (typed error, placeholder), never silently.
