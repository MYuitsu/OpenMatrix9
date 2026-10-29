# OM9-L3-03-13-006 — History Commandline And Preview Status

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

- `the_command_line_drives_the_switches_report_and_clear`
- `closing_the_window_is_refused_while_a_recipe_is_pending`

## Notes

- The command line and the sidebar/menu IDs dispatch to the same Controller methods; the status line consumes one history notice per command.
- The rebuild is synchronous, so an interactive 'updating' frame is not observable; the terminal status distinguishes complete (with count) from failed (with node id and reason).
- A recipe in Pending state is the queued rebuild the lifecycle guard refuses to close over.

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
