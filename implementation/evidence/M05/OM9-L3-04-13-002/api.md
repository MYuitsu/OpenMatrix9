# Gem Control And Size Edits (`OM9-L3-04-13-002`)

## Command

`BuilderGemControl / gvGemControl` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `om9_core::gem_edits::GemEdit` | partial attribute edit (`cut/shape/length/width/depth/girdle/corner/material/color`); a missing field means `leave alone`, never `reset to default` |
| `GemEdit::{spec,record,validate,changes_geometry}` | applies the edit through `GemSpec::set_girdle_mm/set_depth_mm/...`; a cut/shape mismatch or an out-of-range size is a typed `Invalid` error |
| `Controller::dispatch_gem_edits` + `gem_edits_text` | one session over the ordered selection; `Size=`/`Length=`/`Width=`/`Depth=`/`Girdle=`/`Corner=`/`Material=`/`Color=`/`Label=` recompute every plan |
| `Controller::gem_edits_apply` | all selected stones commit in ONE transaction (geometry + record + optional label), so Undo restores every stone together |

## Effects

- A preview is pure session data: planned records and, for a report, computed rows. Nothing is
  written to the document before Apply.
- Apply is exactly one document transaction for every mutating command; a style save writes the
  versioned store atomically. Undo restores the previous records, geometry and label.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); an attribute, stone or row is never silently dropped or replaced.
- Every edit keeps the `GemId`, the object id and the `om9.gem.placement` link; only the
  `om9.gem` record and the regenerated geometry change.

## Verification

- Positive: one-transaction multi-stone edit with stable identities and Undo/Redo
- Negative: wrong selection and unused options are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
