# Gem Attribute History And ID Preservation (`OM9-L3-04-13-009`)

## Command

`GemAttributeHistory / gvGemHistory` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `GemEdit::record` identity contract | the edited record is cloned from the stored one, so `gem_id` is never regenerated |
| `Controller::gem_edits_apply` extension merge | only `om9.gem` is replaced; the `om9.gem.placement` link and every other slot are carried over unchanged |
| `GemAttributeHistory` report | one line per stone: entity id, `GemId`, shape/cut, girdle and the live placement link (source tag, object, revision) |

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

- Positive: ids and the placement link survive an edit and the history names them
- Negative: a missing link is reported honestly, never fabricated
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
