# API contract — Builder Style Mode And Shape Filter

L3: `OM9-L3-03-12-005`

| API | Behaviour |
|---|---|
| `StyleFamily::matches` | band/signet/setting fit ring builders; gem does not |
| `BuilderKind::tag` / `from_tag` / `is_ring_builder` | stable tags for the style file and the filter |
| `BuilderStyle::compatible` / `targets` | family and optional builder tags decide whether a row is enabled |
| `BuilderStyle::apply` | a foreign style is `Invalid("style mode")`, never a silent no-op |
| `StyleEntry.enabled` | the panel lists the incompatible style disabled instead of hiding it |

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
