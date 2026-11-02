# OM9-L3-04-19-005 — Metal Weight Error And Open B-Rep Policy

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `OM9-L3-04-19-005.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

- `an_open_shell_or_an_empty_solid_never_gets_a_volume`
- `non_finite_and_zero_volumes_are_refused`
- `a_duplicate_body_id_without_an_explicit_count_is_refused`
- `an_open_shell_is_refused_and_the_previous_report_is_preserved`
- `a_missing_density_is_a_typed_refusal_not_a_zero_mass`

## Notes

- The report is computed into a local value and stored only after every body succeeds.
- A kernel measurement failure is mapped to Invalid("metal weight geometry") rather than an I/O string, so the caller sees a typed refusal.

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
