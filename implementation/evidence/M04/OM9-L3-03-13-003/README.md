# OM9-L3-03-13-003 — Matrix History Broken Link Report

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

- `the_report_names_a_missing_input_and_survives_a_reopen`

## Notes

- broken_links reports the recipe id, command, state, missing input roles, changed input roles and the matching source object ids; a Clean recipe with intact bindings never appears.
- The History window shows the recipe id, the detail (`missing input <role> (source <id>)` or `stale: <reason>`) and the Edit/Clear remediation; nothing is repaired silently.
- Dirty/Unresolved/Failed rows are warnings too, so a parameter edit with update off (or a source revision change) is visible without being called missing.
- The row is derived from the graph, not cached UI state, so a reopen re-reports the phantom reference.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
