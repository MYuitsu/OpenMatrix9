# OM9-L3-04-19-001 — Metal Volume From Closed B-Rep

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `OM9-L3-04-19-001.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

- `a_closed_box_volume_is_admitted_and_project_units_convert_to_mm`
- `an_open_shell_or_an_empty_solid_never_gets_a_volume`
- `non_finite_and_zero_volumes_are_refused`
- `the_icon_reports_a_closed_box_in_grams_and_pennyweights_without_editing`

## Notes

- The kernel measures canonical source, never the display mesh or a bounding box.
- Volume conversion is cubic: a volume in cm^3 is multiplied by 1000, not passed through a length conversion.

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
