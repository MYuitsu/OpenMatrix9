# OM9-L3-04-02-005 — Four Rail Profile Tool

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

- `the_four_profile_icon_reaches_the_existing_four_rail_metadata_command`
- `the_four_profile_icon_refuses_an_inconsistent_selection`
- `four_rail_profile_command_tags_profiles_then_sweep4_uses_them`

## Notes

- OM9-TOOLS-032 is an alias of the existing OM9-TOOLS-004 metadata command; both record the same stable command and label, so a recipe is not split by the icon used.
- Five or more selected curves on OM9-TOOLS-004 take the four-rail path; a smaller selection is treated as a custom rail.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
