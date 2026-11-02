# Gem Count On Curve (`OM9-L3-04-10-003`)

## Command

`BuilderGemCountOnCrv / gvGemCountOnCrv` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::count_on_curve` | `CurveFit { count, spacing_mm, first_center_mm, last_center_mm, fits }` from the diameter, gap and end allowance |
| `CurveFit` | an explicit `fits == false` for a curve shorter than one stone; the caller must refuse rather than place a partial row |
| `GemPlacementTool::CountOnCurve` | `Gap=` and `Allowance=` drive the fit; Apply reuses the fitted first/last centres |

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

- Positive: fit count derives from arc length, stone diameter and the requested gap
- Negative: a non-fitting row is refused instead of placing a partial row
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
