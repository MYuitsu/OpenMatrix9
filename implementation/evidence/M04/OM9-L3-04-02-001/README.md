# OM9-L3-04-02-001 — Ring Rail Jewelry Size Standard

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

- `a_us_ring_size_resolves_through_the_frozen_table_and_an_unknown_locale_is_refused`
- `the_ring_size_command_switches_the_builder_to_the_us_table`

## Notes

- The table is frozen in om9-core with provenance text; the producer reads the standard choice and the size count instead of a hidden locale lookup.
- The panel sends Value::Quantity with the field's Dimension so a count or ratio is not mis-typed as a length.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
