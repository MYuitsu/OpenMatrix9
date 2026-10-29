# Microprong Cutter (`OM9-L3-04-17-002`)

Feature: `BuilderMicroProngCutter` · commands: `BuilderMicroProngCutter`, `gvMicroProngCutter`, `MicroProng`

## Owner

- `crates/om9-core/src/cutters/microprong.rs`
- `crates/om9-app/src/session/channel_cutters.rs`

## Symbols

- `cutters::microprong::MicroProngParams`
- `cutters::microprong::build_row`
- `cutters::microprong::MicroProngCutter`

## Command options

- `Radius=<mm>`
- `Depth=<mm>`
- `Height=<mm>`
- `Count=<n> (0 = one per gap)`
- `Spacing=<mm>`
- `Angle=<degrees>`
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
- `Radius=0` → Invalid("microprong radius")
- `Angle=70` → Invalid("microprong angle")
- `build_row(&[])` → Missing("microprong gems")

## Tests

- `cargo test -p om9-core --test channel_cutters microprongs_land_in_every_gap_and_stay_closed`
- `cargo test -p om9-app --test channel_cutters micro_prong_builds_one_scallop_per_gap_and_records_the_count`

## Limitations

- A scallop is a rounded cap (5 rings, 16 segments by default) lofted into one closed body.
- Count=0 follows the gaps; a positive Count distributes the scallops along the row, so the spacing stops following the stones.
- Micro prongs are cutter bodies only: the bead itself belongs to the M06 prong/channel owner.
