# OM9-L3-04-02-001 — Ring Rail Jewelry Size Standard

## Goal

an explicit millimetre diameter or a US ring size drives the same rail builder

## Controller contract

- The Builder/Tools feature IDs and the typed command names dispatch to the same
  `Controller` methods.
- Positive: US size 8 resolves through the frozen table to 18.13 mm inside diameter (radius 9.065); US size 9 resolves to 18.95 mm; an explicit 18 mm rail keeps radius 9.0 and circumference 56.55 mm
- Negative: an unverified locale such as UK is refused by the choice domain; a size outside 1..15 is refused by the count domain

## Tested entry points

- `a_us_ring_size_resolves_through_the_frozen_table_and_an_unknown_locale_is_refused`
- `the_ring_size_command_switches_the_builder_to_the_us_table`
