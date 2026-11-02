# Gem On Curve Multi (`OM9-L3-04-10-005`)

## Command

`BuilderGemOnCrvMulti / gvGemOnCrvMulti` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::place_on_curves` | `MultiPlacement { groups, .. }` — one `CurvePlacement` per input curve |
| `GemPlacementTool::OnCurveMulti` | every selected curve is claimed; fewer than two curves is a typed refusal |
| `Controller::gem_placement_apply` | all groups commit in one transaction, so Undo removes every run together |

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

- Positive: one atomic transaction for a run on every selected curve
- Negative: missing curves and unplaceable curves are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
