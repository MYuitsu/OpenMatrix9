# OM9-L3-03-13-003 — Matrix History Broken Link Report

## Goal

show missing/changed history inputs and warnings

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: the History window lists each recorded feature, its recipe id, state and a detail naming the missing source id; a missing rail input is reported with its role, with no silent repair and an explicit Edit/Clear remediation
- Negative: the phantom stale reference survives save/reopen and is still reported instead of being dropped

## Tested entry points

- `the_report_names_a_missing_input_and_survives_a_reopen`
