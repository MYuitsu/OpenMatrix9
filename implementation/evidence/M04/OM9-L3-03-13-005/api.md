# OM9-L3-03-13-005 — Edit Existing Builder By History

## Goal

reopen builder with parameters from selected object

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: the ring builder reopens at the recorded 22 mm diameter; Apply at 24 mm updates the same object and recipe in one transaction
- Negative: missing builder kind/schema mismatch or an object without a recipe -> typed refusal, no partial edit

## Tested entry points

- `edit_from_history_reopens_the_builder_at_the_recorded_parameters`
