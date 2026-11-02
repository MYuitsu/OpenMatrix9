# OM9-L3-04-02-003 — Profile Placer On Rail

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

- `a_profile_placer_moves_the_section_along_the_rail_and_rolls_it_without_flipping`
- `a_profile_placer_refuses_a_missing_rail_selection`
- `the_profile_placer_and_outside_rail_icons_open_the_existing_builder_modes`

## Notes

- Position is a Ratio in 0..1 and roll is an Angle in degrees; both are optional so older recipes replay at position 0 / roll 0.
- The third builder handle carries scale 0.01 because the value is a ratio, not millimetres.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
