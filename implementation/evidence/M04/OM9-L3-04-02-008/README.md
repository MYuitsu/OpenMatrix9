# OM9-L3-04-02-008 — Rail Profile Sweep Associative Output

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

- `a_ring_size_edit_rebuilds_the_profile_and_one_dirty_sweep`
- `a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one`

## Notes

- This build uses the OCCT registry, so the committed sweep is a B-Rep; the core reference backend test covers the analytic surface variant.
- The chain is A -> B -> C: a ring edit re-stamps the profile against the new rail revision, then the sweep against the new profile revision.
- A failed profile rebuild is reported as RecomputeOutcome::Failed and the sweep keeps its last committed geometry revision.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
