# OM9-L3-04-19-004 — Multi Material Model Mass Breakdown

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `OM9-L3-04-19-004.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

- `multi_material_breakdown_sums_per_body_and_avoids_double_counting`
- `a_duplicate_body_id_without_an_explicit_count_is_refused`
- `a_breakdown_refuses_unknown_materials_and_bad_counts`
- `a_multi_material_selection_reports_each_body_and_the_total`
- `an_assignment_that_omits_a_selected_body_is_refused`

## Notes

- A duplicate key is refused rather than silently merged; the one-row `instances` field is the only way to state a count.
- The app assignment list must cover the selection exactly once, so a report cannot omit a selected body.

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
