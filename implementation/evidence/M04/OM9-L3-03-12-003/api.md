# API contract — Builder Style Apply And Preview

L3: `OM9-L3-03-12-003`

| API | Behaviour |
|---|---|
| `BuilderStyle::apply(session)` | loads the parameters into the session (preview source) and reports `applied`/`ignored`; the document is untouched |
| `StyleApply::summary` | one-line report for the panel and the command line |
| `Controller::builder_apply_style` | preview refresh through `builder_refresh`, applied-style marker, document byte-identical |
| `Style=<name>` | command-line route to the same service |

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
