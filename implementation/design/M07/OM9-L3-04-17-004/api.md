# Bright Cutter Single Gem (`OM9-L3-04-17-004`)

Feature: `BuilderBrightCutter` · commands: `BuilderBrightCutter`, `gvBrightCutter`, `BrightCutter`

## Owner

- `crates/om9-core/src/cutters/brightcut.rs`
- `crates/om9-app/src/session/channel_cutters.rs`
- `crates/om9-app/src/ui/channel_cutters.rs`

## Symbols

- `cutters::brightcut::BrightParams::for_gem`
- `cutters::brightcut::build_single`
- `cutters::brightcut::BrightMode::Single`

## Command options

- `Height=<mm>`
- `Length=<mm>`
- `Scale2D=<percent>`
- `Count=<n>`
- `Angle=<degrees>`
- `Depth=<mm>`
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
- `Count=2` → Invalid("bright cutter count")
- `a 0 mm stone` → Invalid("bright cutter gem")

## Tests

- `cargo test -p om9-core --test channel_cutters the_bright_cutter_builds_one_closed_facet_per_count`
- `cargo test -p om9-core --test channel_cutters the_bright_2d_scale_and_angle_are_typed_controls`

## Limitations

- The facet is a block whose inner edge sits on the girdle and whose outer edge is Length away; the top face carries the bright angle.
- A facet that would sink below its own bottom is deepened instead of refused: the cutter has to contain its facet.
