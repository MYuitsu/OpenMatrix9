# OM9-L3-04-02-006 — Profile Edit Merge And Endcap

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

- `profile_sections_merge_when_they_are_connected_and_consistently_wound`
- `the_profile_merge_icon_joins_compatible_sections_and_refuses_one_section`
- `the_profile_end_cap_icon_builds_a_planar_cap_from_closed_sections`
- `the_profile_end_cap_refuses_an_open_curve_without_changing_the_document`

## Notes

- Winding is measured in the first section's fixed plane frame; a per-section basis would flip twice for a reversed loop and hide the change.
- A curve selection gets a planar cap surface. An open B-Rep selection uses the kernel planar cap under the occt-brep feature and otherwise reports Unsupported.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
