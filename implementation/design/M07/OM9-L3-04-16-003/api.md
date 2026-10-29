# Gem Cutter Library Profiles (`OM9-L3-04-16-003`)

Feature: `BuilderGemCutterLibrary` · commands: `BuilderGemCutterLibrary`, `gvGemCutterLibrary`, `GemCutterLibrary`

## Owner

- `crates/om9-core/src/cutters/library.rs`
- `crates/om9-app/src/session/gem_cutters.rs`

## Symbols

- `cutters::library::CutterPreset`
- `cutters::library::LibraryParams`
- `cutters::library::build`
- `Controller::dispatch_gem_cutter`

## Command options

- `Preset=SettingAzure|HeartAzure|Setting|Straight|Brite|Grooves|ChannelBur|Hart|TruncatedCone|Cone`
- `ScaleX=<percent>`
- `ScaleY=<percent>`
- `ZScale=<percent>`
- `ZOffset=<mm>`
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

- `Preset=Titan` → Invalid("cutter preset")
- `Preset=` → Missing("cutter preset")
- `ScaleX=5` → Invalid("cutter library 2d scale")
- `a scaled-down heart azure` → Invalid("cutter library clearance")

## Tests

- `cargo test -p om9-core --test cutters every_library_preset_builds_a_closed_cutter`
- `cargo test -p om9-core --test cutters the_library_2d_scale_is_a_typed_control`
- `cargo test -p om9-app --test gem_cutters the_library_command_builds_each_preset_and_records_the_seat`

## Limitations

- The ten presets are the manual's own table (Z in pavilion-depth units, radius in girdle-radius units); a user preset library is not part of this owner.
- The library writes the same om9.cutter slot as the Gem Cutter with kind=gem_cutter_library, so the display filter and the jewelry report read one record shape.
