# OM9-L3-04-19-005 — Metal Weight Error And Open B-Rep Policy

## Goal

reject unsupported geometry and preserve the previous valid report

## Contract

- Positive: open shell, invalid B-Rep and missing density are typed errors; the document, Undo journal and previous report stay unchanged
- Negative: NaN/infinite/zero volume, zero instances, duplicate bodies and a malformed material option are refused
- Read-only: the document, selection and Undo journal are unchanged; a refusal keeps the
  previous valid report.

## Tested entry points

- `an_open_shell_or_an_empty_solid_never_gets_a_volume`
- `non_finite_and_zero_volumes_are_refused`
- `a_duplicate_body_id_without_an_explicit_count_is_refused`
- `an_open_shell_is_refused_and_the_previous_report_is_preserved`
- `a_missing_density_is_a_typed_refusal_not_a_zero_mass`
