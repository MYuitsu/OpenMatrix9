# OM9-L3-04-19-001 — Metal Volume From Closed B-Rep

## Goal

compute a solid's volume in mm^3 and refuse an open shell

## Contract

- Positive: a closed 10 x 5 x 2 mm box measures 100 mm^3; 100 cm^3 converts to 100,000 mm^3 before aggregation
- Negative: an open shell, zero solids, NaN, infinite and zero/negative volumes are typed refusals; no mass is ever reported
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

- `a_closed_box_volume_is_admitted_and_project_units_convert_to_mm`
- `an_open_shell_or_an_empty_solid_never_gets_a_volume`
- `non_finite_and_zero_volumes_are_refused`
- `the_icon_reports_a_closed_box_in_grams_and_pennyweights_without_editing`
