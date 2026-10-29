# OM9-L3-04-02-009 — Rail And Profile Viewport Handles

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

- `the_rail_profile_handles_edit_width_and_commit_exactly_once`
- `a_rail_profile_builder_reads_the_selected_rail_and_exposes_two_handles`

## Notes

- Handle drag deltas are projected millimetres times the handle scale; the position handle scales by 0.01 so one projected millimetre is 0.01 of the rail.
- Apply is the only insert path: Cancel clears the session without a mutation.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
