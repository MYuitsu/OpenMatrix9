# Cutter Pavilion Clearance And Depth Controls (`OM9-L3-04-16-005`)

Feature: `BuilderGemCutter` · commands: `BuilderGemCutter`, `BuilderGemCutterLibrary`

## Owner

- `crates/om9-core/src/cutters/mod.rs`
- `crates/om9-core/src/cutters/gem.rs`
- `crates/om9-core/src/cutters/library.rs`

## Symbols

- `cutters::GemCutterReference`
- `cutters::gem::GemCutterParams::validate`
- `cutters::gem::build`
- `cutters::library::build`

## Command options

- `Clearance=<mm>`
- `LowerDepth=<percent>`
- `LowerSeatDepth=<percent>`
- `LowerSeatSize=<percent>`
- `ZOffset=<percent>`
- `GirdleScale=<percent>`
- `GirdleWidth=<percent>`
- `Preset= + ScaleX/ScaleY/ZScale/ZOffset (library)`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `Clearance=0` → Invalid("cutter clearance") (never silently clamped)
- `LowerSeatDepth > LowerDepth` → Invalid("cutter lower seat depth")
- `a hand-built ring list out of order` → Invalid("cutter ring order")
- `GirdleScale below the stone profile` → Invalid("cutter pavilion clearance")

## Tests

- `cargo test -p om9-core --test cutters the_cutter_reference_is_measured_from_the_stone_profile`
- `cargo test -p om9-core --lib cutters::gem`
- `cargo test -p om9-core --test cutters a_cutter_that_would_cut_into_the_stone_is_refused`
- `cargo test -p om9-core --test cutters every_edge_profile_stays_a_closed_cutter_and_changes_the_hole`

## Limitations

- The clearance gate checks the ring radii against the linearly interpolated measured profile; it is a conservative radial test, not a full CSG interference test.
- Percentage semantics are documented approximations of the manual's unnamed reference lengths.
