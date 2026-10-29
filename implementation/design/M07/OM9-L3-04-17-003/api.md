# Bright Cut Channel Builder (`OM9-L3-04-17-003`)

Feature: `BuilderBrightCutChannel` · commands: `BuilderBrightCutChannel`, `gvBrightCutChannel`, `BrightChannel`

## Owner

- `crates/om9-core/src/cutters/brightcut.rs`
- `crates/om9-app/src/session/channel_cutters.rs`

## Symbols

- `cutters::brightcut::BrightParams`
- `cutters::brightcut::build_channel`
- `cutters::brightcut::BrightMode::Channel`

## Command options

- `Height=<mm>`
- `Length=<mm>`
- `Scale2D=<percent>`
- `Angle=<degrees>`
- `Depth=<mm>`
- `Mirror=on|off`
- `Boolean=off|on|one`
- `Preserve=preserve|remove`
- `Reset`
- `Enter`
- `Cancel`

## Input contract

The builder reads the gem owner's `om9.gem` record and the document placement of each selected stone, in selection order; that order is the row. Every selected closed mesh body that is not a gem and not already an `om9.cutter` is an Interactive Boolean target. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `channel_cutter` | `micro_prong_cutter` | `bright_cut_channel` | `bright_cutter`) carrying `row` (the stone labels in row order), the volume, the Boolean/Preserve answers and the typed parameters; for a cut, one `om9.cutter.cut` record on each replaced metal body with its removed volume and the same row.

## Refusals

- `a metal mesh only` → Missing("cutter gems")
- `Scale2D=5` → Invalid("bright cutter 2d scale")
- `Angle=88` → Invalid("bright cutter angle")
- `build_channel(&[])` → Missing("bright cutter gems")

## Tests

- `cargo test -p om9-core --test channel_cutters the_bright_channel_mirrors_both_sides_of_the_row`
- `cargo test -p om9-app --test channel_cutters the_bright_cutter_rings_one_stone_and_the_channel_mirrors_the_row`

## Limitations

- Each facet is one closed 8-corner block with a sloped top face; the channel variant places the same block along the row.
- Mirror=on builds the far side by mirroring the same frame; it does not re-derive a second profile.
