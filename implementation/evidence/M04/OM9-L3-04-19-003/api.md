# OM9-L3-04-19-003 — Metal Estimated Mass Calculation

## Goal

estimate grams and pennyweights from volume and density

## Contract

- Positive: 100 mm^3 at 10.0 g/cm^3 gives 1.0 g (100/1000*10) and 0.643 dwt; a 10,000 kg/m^3 material is converted to 10 g/cm^3 before the same calculation
- Negative: a kg/m^3 number read as g/cm^3 is refused by the density guard; a missing material and non-positive volume are refused
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

- `a_kg_m3_density_is_converted_before_it_is_used`
- `mass_is_volume_over_one_thousand_times_density_in_grams_and_dwt`
- `the_command_line_matches_the_icon_and_accepts_a_material`
