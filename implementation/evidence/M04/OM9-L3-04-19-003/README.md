# OM9-L3-04-19-003 — Metal Estimated Mass Calculation

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `OM9-L3-04-19-003.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

- `a_kg_m3_density_is_converted_before_it_is_used`
- `mass_is_volume_over_one_thousand_times_density_in_grams_and_dwt`
- `the_command_line_matches_the_icon_and_accepts_a_material`

## Notes

- DWT uses 1 dwt = 1.55517384 g; both grams and dwt are carried in the report and command line.
- The material option is parsed by the same command path as the menu icon; an unknown option is a typed refusal.

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
