# OM9-L3-04-19-002 — Metal Material Density Registry

## Goal

resolve a jewelry alloy density with its unit and provenance

## Contract

- Positive: synthetic_material (10.0 g/cm^3, test-only) resolves with provenance and the 1/1000 mm^3-to-cm^3 factor; built-in 18k yellow gold carries a nominal density and a 1% uncertainty
- Negative: a missing id, blank id, negative density, a kg/m^3 value entered as g/cm^3 (15500) and a blank provenance are refused
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

- `a_synthetic_density_registers_with_provenance_and_the_cm3_factor`
- `a_missing_material_and_an_impossible_density_are_refused`
- `a_builtin_material_carries_provenance_and_an_uncertainty_band`
