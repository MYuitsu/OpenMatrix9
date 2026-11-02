# Gem Follow Underlying Curve (`OM9-L3-04-10-008`)

## Command

`BuilderGemFollow / gvGemFollow` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::station_at_arc` | rebuilds one station at the recorded arc length from the current curve |
| `PlacementLink::refresh(revision, arc, station)` | revives an ACTIVE link; a revision older than the recorded one is `Error::Stale` |
| `PlacementLink::mark_conflict` | a lost source station keeps the last transform and turns the link CONFLICT |

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

- Positive: arc-length follow reuses the link and keeps gem identity
- Negative: lost sources become visible conflicts; stale writes are refused
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
