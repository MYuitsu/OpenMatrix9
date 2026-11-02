# OM9-L3-03-13-001 — Matrix History Record Toggle

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

- `record_off_commits_geometry_without_a_recipe_and_toggles_stamp_new_records`

## Notes

- Recording off omits the History mutation entirely: the entity is committed in the same one-transaction journal, so Undo still removes it.
- The policy is stamped on new recipes (record/update) rather than rewriting recipes that already exist.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
