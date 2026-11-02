# Gem Profile Curve And Guides (`OM9-L3-04-10-006`)

## Command

`BuilderGemProfile / BuilderGemGuides / gvGemProfile / gvGemGuides` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::profile_and_guides` | `PlacementGuides { profile, girdle, guides, table_points }` from the placed stones |
| `GemStation::table_point` | the profile runs through every table point, the girdle through every station point |
| `GemPlacementTool::{Profile, Guides}` | inserts the profile/girdle/guide curves as normal document curves |

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

- Positive: profile, girdle and per-stone guide curves built from the placement
- Negative: too few stones and degenerate guides are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
