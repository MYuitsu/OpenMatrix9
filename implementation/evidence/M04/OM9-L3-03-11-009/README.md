# OM9-L3-03-11-009 — Builder F6 And Commandline Input Binding

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

- `the_f6_prefill_and_the_command_line_reach_the_same_builder_service`
- `f6_uses_the_selected_class_and_empty_general_context`

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
