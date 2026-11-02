# OM9-L3-03-13-002 — Matrix History Update Toggle

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

- `update_off_marks_dirty_and_rebuilds_only_after_update_is_reenabled`
- `reenabling_update_refuses_a_broken_dependency`

## Notes

- recompute_history walks plan_recompute in execution order after the edited recipe; refresh_inputs re-stamps a Dirty consumer against the source as it is now, because the old stamp names the previous geometry revision.
- Re-enabling update validates execution_order (cycles) and broken_links (missing input/output) before the flag flips.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
