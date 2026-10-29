# API contract — Builder Style Param Snapshot Schema

L3: `OM9-L3-03-12-001`

| API | Behaviour |
|---|---|
| `StyleFile` / `STYLE_SCHEMA` | `om9.builder_style.v1` file with `schema` and `styles`; an unknown tag is `Invalid("style schema")` |
| `StyleValue` | `length_mm` / `number` / `choice` / `boolean`; unit and type survive the roundtrip |
| `BuilderStyle::snapshot` / `defaults_of` | record every typed field of a session explicitly, not implicitly |
| `BuilderStyle::validate` | name rules, at least one family and parameter, typed finite values |
| `profile_choice` | `profile_ref=basic_round` -> builder preset `circle`; an unknown reference is `Invalid("style profile")` |

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
