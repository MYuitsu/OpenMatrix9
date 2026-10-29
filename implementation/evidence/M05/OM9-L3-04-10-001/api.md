# Gem On Curve Basic (`OM9-L3-04-10-001`)

## Command

`BuilderGemOnCrv / gvGemOnCrv` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `om9_core::gem_placement::place_on_curve` | arc-length stations between `start_mm` and `end_mm`, `spacing_mm`, closed-seam dedupe, overlap refusal unless requested |
| `CurveParams` | `spacing/with_range/with_z_offset/with_rotation/with_orientation/with_overlap/validate` — typed refusals for non-finite or non-positive spacing |
| `GemStation` | index, fraction, arc length, point, tangent, table axis, `Transform`, spec, z offset, spin and an orthonormal right-handed frame |
| `Controller::dispatch_gem_placement` + `gem_placement_text` | palette icon and typed command reach one session; `Start/End/Spacing/Count` recompute the row, an option the tool does not consume is a typed refusal, `Enter` applies, `Esc` cancels |

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

- Positive: even arc-length row, orthonormal frames, one-transaction apply and Undo
- Negative: overlap, seam duplication and wrong selections are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
