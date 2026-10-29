# Gem Cutter Azure For Single Stone (`OM9-L3-04-16-001`)

Feature: `BuilderGemCutter` · commands: `BuilderGemCutter`, `gvAzureCutter`, `AzureCutter`, `GemCutter`

## Owner

- `crates/om9-core/src/cutters/gem.rs`
- `crates/om9-app/src/session/gem_cutters.rs`

## Symbols

- `cutters::gem::CutterMode`
- `cutters::gem::GemCutterParams`
- `cutters::gem::build`
- `session::gem_cutters::GemCutterSession`
- `Controller::dispatch_gem_cutter`

## Command options

- `Mode=Gem|Azure|Advanced`
- `GirdleScale=<percent>`
- `GirdleWidth=<percent>`
- `LowerSeatSize=<percent>`
- `LowerSeatDepth=<percent>`
- `LowerDepth=<percent>`
- `LowerTaper=<percent>`
- `UpperSeatSize=<percent>`
- `UpperSeatHeight=<percent>`
- `UpperHeight=<percent>`
- `UpperTaper=<percent>`
- `ZOffset=<percent>`
- `Rotation1..5=<degrees>`
- `YScale1..5=<percent>`
- `EdgeProfile=Round|Square|Point|Heart|Brite|Grooves`
- `BottomDir=down|up`
- `Clearance=<mm>`
- `Segments=<count>`
- `Boolean=off|on|one`
- `Preserve=preserve|remove`
- `Reset`
- `Enter`
- `Cancel`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `a metal mesh only` → Missing("cutter gems")
- `GirdleScale=60 with Clearance=0` → Invalid("cutter pavilion clearance")
- `Mode=titan` → Invalid("cutter mode")
- `GirdleScale=110 with no session` → the command line falls through to the option parser, never a silent no-op

## Tests

- `cargo test -p om9-core --test cutters the_three_builder_modes_build_closed_cutters_that_clear_the_stone`
- `cargo test -p om9-core --test cutters a_cutter_that_would_cut_into_the_stone_is_refused`
- `cargo test -p om9-app --test gem_cutters the_gem_cutter_icon_stages_a_preview_and_enter_commits_cutter_bodies`
- `cargo test -p om9-app --test gem_cutters the_builder_needs_a_selected_gem`

## Limitations

- The percentage controls are relative to the measured gem: radius percentages to the girdle half-extent, depth percentages to the pavilion depth, height percentages to the crown height, ZOffset to the pavilion depth.
- The cutter is a lofted ring stack (48 segments by default); it is a cutter body, not a B-Rep solid.
- Edge profiles only shape the azure hole ring; the seat rings stay round so the bearing always clears the stone.
