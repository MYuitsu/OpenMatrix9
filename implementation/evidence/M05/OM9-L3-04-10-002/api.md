# Gem On Curve Advanced (`OM9-L3-04-10-002`)

## Command

`BuilderGemOnCrvAdvanced / gvGemOnCrvAdvanced` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::taper_specs(start, end, count)` | monotonic interpolation between the first and last spec; every intermediate stone is validated, never clamped |
| `gem_placement::place_on_curve_variable` | one spec per station; a length mismatch is `Error::Invalid("gem spec list")` |
| `GemPlacementTool::OnCurveAdvanced` | `Size=` seeds the start, `EndSize=` seeds the end, `Spacing=` sets the station count |

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

- Positive: monotonic variable-diameter row between the two requested sizes
- Negative: bad sizes and mismatched spec lists are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
