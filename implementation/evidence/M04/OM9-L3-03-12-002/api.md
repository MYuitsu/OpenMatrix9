# API contract — Builder Style Browser And Thumbnail

L3: `OM9-L3-03-12-002`

| API | Behaviour |
|---|---|
| `StyleLibrary::styles_for(kind)` | rows for the open mode: compatible first, incompatible listed but `enabled=false` |
| `StyleEntry` | style, `enabled`, `thumbnail`, `builtin` |
| `StyleThumbnail::for_style` | authored icon key, else the placeholder with `fallback=true`; a corrupt key never drops the row |
| `Controller::builder_styles` | owned rows for the panel (name, enabled, icon, fallback, builtin, applied) |

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
