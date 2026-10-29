# OM9-L3-04-02-009 — Rail And Profile Viewport Handles

## Goal

drag the profile width handle and commit exactly one profile

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: the RailProfile builder exposes width, height and position handles; adjusting width by +1 mm reports 3.00 x 1.50 in the panel and one Apply inserts one entity and one recipe
- Negative: Cancel after a drag inserts nothing; an invalid or locked drag is refused before the model changes

## Tested entry points

- `the_rail_profile_handles_edit_width_and_commit_exactly_once`
- `a_rail_profile_builder_reads_the_selected_rail_and_exposes_two_handles`
