# Gem Positioner (`OM9-L3-04-13-001`)

## Command

`BuilderGemPositioner / gvGemPositioner` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `om9_app::session::gem` positioner (`BuilderGemPositioner`) | opens on the selected gem and edits `origin`/`rotation_deg`/`scale` without touching the stored record |
| `Controller::gem_apply` positioner branch | one `Mutation::Placement` per Apply, so Undo restores the previous handle frame |
| `gem_record` on the target | a non-gem or a shared definition instance is refused before the tool opens |

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

- Positive: handle move/rotate/scale on one instance, identity preserved
- Negative: a non-gem or a shared definition is a typed refusal
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
