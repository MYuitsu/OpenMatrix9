# Gem Match Attributes (`OM9-L3-04-13-003`)

## Command

`OthersMatchAttributes / gvMatchAttributes` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_edits::MatchMask` | `cut/shape/dimensions/finish` groups; `ALL`/`GEOMETRY`/`FINISH` presets and a comma list parser; an empty mask is refused |
| `gem_edits::match_attributes(source, targets, mask)` | builds one `GemEdit` from the source, applies it to every target and keeps each target `GemId` |
| `OthersMatchAttributes` session | the first selected stone is the source and the remaining ordered selection is the target list; fewer than two stones is a typed refusal |

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

- Positive: masked copy from the first selected stone with target identity kept
- Negative: an empty or unknown mask and a single selection are refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
