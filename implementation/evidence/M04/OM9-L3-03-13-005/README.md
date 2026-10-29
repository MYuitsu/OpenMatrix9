# OM9-L3-03-13-005 — Edit Existing Builder By History

**Status:** implemented and automatically tested; **manual M04-UI gate is open** (no
reviewer signature is claimed). The dashboard keeps this L3 at 🟡 until a human signs the
gate.

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-…-03/04/05/06.json` (+ `-07` where the plan adds an adversarial task) —
  implementation, integration, regression, acceptance and adversarial receipts

## Tests

- `edit_from_history_reopens_the_builder_at_the_recorded_parameters`

## Notes

- The plan adopts the selected object's ObjectId and recipe id, so Apply writes Geometry + HistoryUpdate instead of Insert + History.
- Only the five builder commands are re-openable from history; another command or a schema_version other than 1 is a typed refusal.
- The published geometry_revision is advanced to the revision the Geometry mutation will produce before the transaction is prepared.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
