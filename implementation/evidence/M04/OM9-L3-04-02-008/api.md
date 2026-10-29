# OM9-L3-04-02-008 — Rail Profile Sweep Associative Output

## Goal

re-evaluate the swept band after a ring size or profile edit

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: a RingRail -> RailProfile -> RailSweep chain rebuilds once when the ring diameter changes from 18 to 19 mm; the dependent sweep gets a new geometry revision and the plan is clean
- Negative: an invalid profile width makes the downstream rebuild fail explicitly and leaves the committed sweep geometry revision unchanged

## Tested entry points

- `a_ring_size_edit_rebuilds_the_profile_and_one_dirty_sweep`
- `a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one`
