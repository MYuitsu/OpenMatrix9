# Gem Update And Regenerate (`OM9-L3-04-13-006`)

## Command

`BuilderGemUpdate / gvGemUpdate` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_edits::recompute` update branch | rebuilds the procedural mesh from `GemRecord::spec` before promising the update; an open mesh is refused |
| `Controller::gem_edits_apply` | writes `Mutation::Geometry` plus the unchanged record as ONE transaction |
| `build_mesh` reuse | the regenerated body is the same closed mesh the loader produces, never a separately authored approximation |

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

- Positive: geometry regenerated from the stored record, identity and record kept
- Negative: a spec that cannot rebuild a closed mesh is refused
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
