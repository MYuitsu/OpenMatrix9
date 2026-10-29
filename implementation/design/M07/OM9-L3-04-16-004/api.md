# Bezel Cutter Builder (`OM9-L3-04-16-004`)

Feature: `BuilderBezelCutter` · commands: `BuilderBezelCutter`, `gvBezelCutter`

## Owner

- `crates/om9-core/src/cutters/bezel.rs`
- `crates/om9-core/src/settings/bezel.rs`
- `crates/om9-app/src/session/head_commands.rs`

## Symbols

- `cutters::bezel::BezelCutterDims`
- `cutters::bezel::CutterPosition`
- `cutters::bezel::build`
- `settings::bezel::ScallopParams::dims`
- `settings::bezel::build_scallop`

## Command options

- `Dims=on|off`
- `Height=<mm>`
- `Length=<mm>`
- `Scale2D=<percent>`
- `CutterLength=<mm>`
- `XInside=<mm>`
- `XOutside=<mm>`
- `YInside=<mm>`
- `YOutside=<mm>`
- `CutterAngle=<degrees>`
- `XOffset=<mm>`
- `YOffset=<mm>`
- `ZOffset=<mm>`
- `Count=<n>`
- `Position=odd|even`
- `Profile=/Count=/Angle=/Offset=/Depth= (the swept profile contract, still the default)`
- `Enter`
- `Cancel`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `Length=1 CutterLength=2` → Invalid("bezel cutter length")
- `Count=0` → Invalid("bezel cutter count")
- `CutterLength=0` → Invalid("bezel cutter cutter length")
- `CutterAngle=120` → Invalid("bezel cutter angle")
- `Position=Even` → Invalid("bezel cutter position needs Dims=on")

## Tests

- `cargo test -p om9-core --lib cutters::bezel`
- `cargo test -p om9-core --test cutters the_dimensional_bezel_builder_makes_a_closed_block_per_cutter`
- `cargo test -p om9-core --test cutters the_bezier_profile_still_wins_when_no_dimensional_block_is_asked_for`
- `cargo test -p om9-app --test gem_cutters the_dimensional_bezel_cutter_options_reach_the_shared_scallop_command`
- `cargo test -p om9-app --test head_commands`

## Limitations

- Ownership stays with the one gvBezelCutter command of M06 (OM9-L2-04-14): this owner publishes the dimensional parameter contract and the block geometry that the shared scallop command dispatches to, so no second command kernel exists.
- The block is an eight-corner tapered prism; Edge Profiles/Styles for the dimensional mode are not in this owner (the swept contract keeps the M06 profile library).
- The manual's X/Y Inside/Outside sizes are tangential widths and axial heights of the two ends; Cutter Angle lifts the outer end.
