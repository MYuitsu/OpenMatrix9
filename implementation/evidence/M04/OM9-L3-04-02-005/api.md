# OM9-L3-04-02-005 — Four Rail Profile Tool

## Goal

tag two cross-sections from four consistent rails and sweep them

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: four rails with consistent start/direction/span tag both profiles with om9.four-rail-profile metadata, and Sweep4 consumes them as one four-boundary surface
- Negative: a selection that is not four rails plus two profiles is refused; the command does not insert metadata or a partial surface

## Tested entry points

- `the_four_profile_icon_reaches_the_existing_four_rail_metadata_command`
- `the_four_profile_icon_refuses_an_inconsistent_selection`
- `four_rail_profile_command_tags_profiles_then_sweep4_uses_them`
