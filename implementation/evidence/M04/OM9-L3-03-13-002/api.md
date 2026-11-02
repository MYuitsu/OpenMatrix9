# OM9-L3-03-13-002 — Matrix History Update Toggle

## Goal

enable/disable associative recompute display

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: with update off a parameter edit leaves the rail geometry and marks the recipe Dirty; re-enabling update rebuilds the edited feature and its downstream profile once
- Negative: re-enable with a broken dependency -> Invalid(history dependency broken) and the switch stays off

## Tested entry points

- `update_off_marks_dirty_and_rebuilds_only_after_update_is_reenabled`
- `reenabling_update_refuses_a_broken_dependency`
