# Gem List On Curve (`OM9-L3-04-10-004`)

## Command

`BuilderGemListOnCrv / gvGemListOnCrv` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::place_on_curve_variable` | heterogeneous `&[GemSpec]` placed in order; a one-element list broadcasts |
| `GemPlacementTool::ListOnCurve` | `Sizes=` parses a comma-separated diameter list into validated round specs |
| `CurveParams::with_range` | the list is distributed between `Start=` and `End=` with a computed pitch |

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

- Positive: heterogeneous diameters placed in order with per-stone specs
- Negative: empty or invalid stone lists are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
