# OM9-L3-03-13-004 — Matrix Clear Single Object History

## Goal

detach a specific object while leaving other associations

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: clearing the profile detaches only its recipe; the profile geometry stays, the rail recipe stays Clean and the report lists the remaining feature
- Negative: locked recipe -> Invalid(history recipe locked); no owner -> Missing(history recipe)

## Tested entry points

- `clear_history_detaches_only_the_selected_object_and_refuses_a_lock`
