# OM9-L3-03-11-006 — Builder Handle And Panel Bidirectional Sync

**Status:** implemented and automatically tested; **manual M04-UI gate is open** (no
reviewer signature is claimed). The dashboard keeps this L3 at 🟡 with all L4 automated
artefacts present (6/6, or 7/7 where the plan adds an adversarial task).

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the API contract the tests exercise
- `positive.json` / `negative.json` — fixtures and receipts
- `OM9-L4-…-03/04/05/06.json` (+ `-07` where the plan adds an adversarial task) —
  implementation, integration, regression, acceptance and adversarial receipts (command
  argv, exit code, stdout/stderr SHA-256, fixture manifest SHA-256)

## Tests

- `a_stale_panel_event_cannot_overwrite_a_newer_handle_drag`
- `a_handle_drag_and_a_typed_value_share_the_one_field`

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
