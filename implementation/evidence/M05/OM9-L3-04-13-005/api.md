# Gem Style Load (`OM9-L3-04-13-005`)

## Command

`OthersLoadStyle / gvLoadStyle` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `GemStyle::apply(record)` | applies the saved attribute edit to a stone and keeps the target `GemId` |
| `GemStyleStore::get(name)` | case-insensitive lookup; an absent name is `Missing("gem style name")` |
| `OthersLoadStyle` session | `Name=` chooses the style, `File=` the store; every selected stone gets the same style in ONE transaction |

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

- Positive: named style applied to every selected stone with identity kept
- Negative: an absent name or a malformed store is a typed refusal
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
