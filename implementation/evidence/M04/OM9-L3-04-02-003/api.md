# OM9-L3-04-02-003 — Profile Placer On Rail

## Goal

place a typed profile on the selected rail with a stable tangent/normal frame

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: rail radius 9 with a closed 2 x 1 profile: position 0.00 sits at the rail start, 0.25 moves to the quarter station, and a 90 degree roll moves the section corner while the profile centre stays on the rail
- Negative: a missing rail input is refused; a zero-length or degenerate tangent is refused before a preview is built

## Tested entry points

- `a_profile_placer_moves_the_section_along_the_rail_and_rolls_it_without_flipping`
- `a_profile_placer_refuses_a_missing_rail_selection`
- `the_profile_placer_and_outside_rail_icons_open_the_existing_builder_modes`
