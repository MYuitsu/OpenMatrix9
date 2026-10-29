# OM9-L3-04-19-002 — Metal Material Density Registry

**Status:** implemented and automatically tested; the L3 plan gate is `AUTO_ONLY`, so the
dashboard accepts the AUTOMATED_EVIDENCE_ONLY record below. **No human signature is claimed.**

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-04-19-…-03/04/05/06/07.json` — implementation, integration, regression,
  acceptance and (where planned) adversarial receipts
- `OM9-L3-04-19-002.ui-review.json` — machine-readable AUTO_ONLY acceptance, no human signature

## Tests

- `a_synthetic_density_registers_with_provenance_and_the_cm3_factor`
- `a_missing_material_and_an_impossible_density_are_refused`
- `a_builtin_material_carries_provenance_and_an_uncertainty_band`

## Notes

- Every density is stored in g/cm^3; kg/m^3 must be declared through DensityUnit so a 1000x error cannot be silent.
- Built-in densities are nominal reference values with provenance and a 1% uncertainty; a supplier lot must still be verified.

## Not claimed

- Human acceptance of a Metal Weights builder panel or pointer gesture; the current
  command reports from the selection immediately.
- Supplier-lot density accuracy: the built-in densities are nominal with provenance and a
  1% uncertainty.
