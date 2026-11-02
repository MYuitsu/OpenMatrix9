# OM9-L3-04-02-004 — Custom Rail And Advanced Rail

## Goal

accept a user-authored closed smooth rail for the profile sweep

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: a closed rational-quadratic ellipse rail (9 x 6) validates as closed/planar/non-self-intersecting and a builder sweep over it produces a surface with continuous frames
- Negative: an open line is refused with ring rail not closed; a bow-tie polyline is refused with ring rail self-intersection; the advanced icon refuses an open rail without inserting geometry

## Tested entry points

- `a_custom_rail_must_be_closed_planar_and_non_self_intersecting`
- `a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one`
- `the_custom_rail_icon_records_a_validated_closed_rail_and_refuses_an_open_one`
