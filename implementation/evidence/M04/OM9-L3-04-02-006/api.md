# OM9-L3-04-02-006 — Profile Edit Merge And Endcap

## Goal

merge compatible profile sections and close a profile for a valid cap

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: two half-sections of one circle merge into a closed polycurve; a closed rectangular profile produces a planar cap surface; a closed profile path is checked before the cap
- Negative: one section is refused; reversed winding is refused with profile section winding; an open curve has no cap boundary and the document is unchanged

## Tested entry points

- `profile_sections_merge_when_they_are_connected_and_consistently_wound`
- `the_profile_merge_icon_joins_compatible_sections_and_refuses_one_section`
- `the_profile_end_cap_icon_builds_a_planar_cap_from_closed_sections`
- `the_profile_end_cap_refuses_an_open_curve_without_changing_the_document`
