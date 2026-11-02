# OM9-L3-04-19-004 — Multi Material Model Mass Breakdown

## Goal

sum per-body weights and keep instances from double-counting

## Contract

- Positive: body A 100 mm^3 at 10 g/cm^3 = 1.0 g; body B 200 mm^3 at 8 g/cm^3 = 1.6 g; total 2.6 g; an explicit instance count multiplies one row
- Negative: the same body id twice without an explicit count is refused; an assignment that omits a selected body is refused
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

- `multi_material_breakdown_sums_per_body_and_avoids_double_counting`
- `a_duplicate_body_id_without_an_explicit_count_is_refused`
- `a_breakdown_refuses_unknown_materials_and_bad_counts`
- `a_multi_material_selection_reports_each_body_and_the_total`
- `an_assignment_that_omits_a_selected_body_is_refused`
