# API contract — Builder Style Save New Name

L3: `OM9-L3-03-12-004`

| API | Behaviour |
|---|---|
| `StyleLibrary::save_user` | user list only, built-ins untouched; duplicate name is `Invalid("style exists")` |
| `validate_style_name` | trimmed, bounded, no separators, traversal, leading dot or control text |
| `StyleLibrary::save_to` / `load_from` | `om9.builder_style.v1` user file; a refused write is `Permission("style library")`, a corrupt file `Invalid("style schema")` |
| `Controller::builder_save_style` / `StyleSave=` | snapshot the open session under a new name |
| `OM9_STYLE_LIBRARY` / `Controller::builder_set_style_path` | where the user file lives; without it the list is session-only |

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
