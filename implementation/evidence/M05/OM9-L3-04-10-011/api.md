# Gem To Curve Surface History Links (`OM9-L3-04-10-011`)

## Command

`GemPlacementLinks / gvGemPlacementLinks` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::PlacementLink` | `schema` (`om9.gem_placement.v1`), `version`, gem id, source, arc, station, uv, source revision, state and spacing |
| `PlacementLink::{to_extension,from_extension,to_json,from_json}` | the `om9.gem.placement` entity extension, version 1, survives document save/reopen |
| `PlacementLink::{refresh,mark_broken,mark_conflict}` | ACTIVE/BROKEN/CONFLICT lifecycle with a stale-revision guard |
| `Controller::gem_placement_links_report` | the command-line report lists the source tag, object, arc, station, revision and UV |

## Effects

- A preview is pure placement data plus a computed station list; it never inserts a document
  object and never mutates the source curve/surface.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the
  gem object identity and its `GemId`.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); a station is never silently dropped or moved to a guessed place.
- Each placed gem carries `om9.gem` (`GemRecord`) and `om9.gem.placement` (`PlacementLink`,
  schema `om9.gem_placement.v1`, version 1).

## Verification

- Positive: persisted `om9.gem.placement` link with a lifecycle and a report
- Negative: unreadable links are reported and stale writes/records are refused
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
