# OM9-L3-04-02-004 — Custom Rail And Advanced Rail

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

- `a_custom_rail_must_be_closed_planar_and_non_self_intersecting`
- `a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one`
- `the_custom_rail_icon_records_a_validated_closed_rail_and_refuses_an_open_one`

## Notes

- The self-intersection test includes endpoint-touching crossings: a strict orientation-only test misses a bow-tie whose crossing lands exactly on a sample.
- A clamped NURBS seam can refuse D1; path_frames and tangent_at fall back to a symmetric secant so the frame stays defined without changing the curve.

## Not claimed

- Human acceptance of the panel, pointer gestures or the visual result.
- A signature: record only the automated receipts; a human owns the remaining gate.
