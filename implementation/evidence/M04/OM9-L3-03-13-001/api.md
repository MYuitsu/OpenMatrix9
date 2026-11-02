# OM9-L3-03-13-001 — Matrix History Record Toggle

## Goal

turn history recording on/off for a selected command sequence

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: record off inserts the rail geometry without a recipe; record on stores the typed recipe; a toggle is refused while the builder transaction is open
- Negative: toggle while an edit transaction is open -> Invalid(history record during edit transaction), policy unchanged

## Tested entry points

- `record_off_commits_geometry_without_a_recipe_and_toggles_stamp_new_records`
