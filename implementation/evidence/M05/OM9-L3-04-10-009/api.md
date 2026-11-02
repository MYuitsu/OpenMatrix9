# Gem Flow Placement (`OM9-L3-04-10-009`)

## Command

`BuilderGVDGemFlow / gvGemFlow` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::flow_station` | maps a station from the base curve to the target curve at the same curve fraction |
| `gem_placement::flow_placement` / `flow_surface_placement` | batch flow for curves and for surfaces; the stone proportions are preserved |
| `gem_placement::rigid_flow` | the deform owner `deform::rigid_flow` moves the frame, not the stone shape |
| `GemPlacementTool::Flow` | one target may be a curve or a surface; a curve-linked row flows to a curve, a surface-linked row flows to a surface, and the link names the new source |

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

- Positive: rigid flow from a base curve/surface to a curve/surface target with a retargeted link
- Negative: missing targets and unmappable stations are typed refusals/conflicts
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
