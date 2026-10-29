# OM9-L3-03-13-004 — Matrix Clear Single Object History

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

- `clear_history_detaches_only_the_selected_object_and_refuses_a_lock`

## Notes

- Detach is one HistoryUpdate transaction (state Detached), so Undo restores the association.
- The published result is intentionally kept as a plain snapshot; there is no result-removal mutation in the transaction model.
- A frozen (lock) recipe is refused so Clear never bypasses the lock.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
