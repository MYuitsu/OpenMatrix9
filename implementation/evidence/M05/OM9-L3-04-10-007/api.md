# Gem On Surface (`OM9-L3-04-10-007`)

## Command

`BuilderGemOnSurface / gvGemOnSurface` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_placement::place_on_surface` | evaluates the surface at each UV, builds an orthonormal frame from the U/V tangents and the normal |
| `SurfaceParams` | `uv`, `z_offset_mm`, `rotation_deg`; a UV outside the domain is refused |
| `GemPlacementTool::OnSurface` | `U=`, `V=` and `Grid=` place one stone or a UV grid; `ZOffset=`/`Rotate=` apply |

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

- Positive: surface normal frame, UV grid placement and a UV-bearing link
- Negative: out-of-domain UVs and degenerate normals are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
