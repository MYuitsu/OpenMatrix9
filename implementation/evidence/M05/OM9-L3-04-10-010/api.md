# Gem Splop On Surface (`OM9-L3-04-10-010`)

## Command

`BuilderGVDGemSplop / gvGemSplop` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::splop_station` | closest point on the surface, surface normal frame, `x_hint` for the spin |
| `SplopParams` | `z_offset_mm`, `rotation_deg`, `x_hint`; a degenerate normal is refused |
| `Controller` splop tool | `ZOffset=` and `Rotate=` move the projection along the normal / spin the stone |

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

- Positive: sphere-style projection onto the surface normal with a UV link
- Negative: degenerate normals and bad parameters are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
