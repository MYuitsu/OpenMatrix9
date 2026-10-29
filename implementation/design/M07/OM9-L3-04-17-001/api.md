# Channel Builder Cutter (`OM9-L3-04-17-001`)

Feature: `BuilderChannelBuilder` · commands: `BuilderChannelBuilder`, `gvChannelBuilder`, `ChannelCutter`

## Owner

- `crates/om9-core/src/cutters/channel.rs`
- `crates/om9-core/src/cutters/mod.rs`
- `crates/om9-app/src/session/channel_cutters.rs`

## Symbols

- `cutters::channel::ChannelParams`
- `cutters::channel::build_row`
- `cutters::channel::ChannelCutter`
- `cutters::sweep`
- `session::channel_cutters::RowCutterSession`

## Command options

- `Width=<mm>`
- `Depth=<mm>`
- `Height=<mm>`
- `Wall=<mm>`
- `Spacing=<mm>`
- `Start=Square|Round|Taper`
- `End=Square|Round|Taper`
- `StartDepth=<mm>`
- `EndDepth=<mm>`
- `Mirror=on|off`
- `Flip=ab|ba`
- `Middle=None|Wide|Deep`
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
- `Width=2.0 with a 6.5 mm stone` → Invalid("channel width")
- `Depth=9.0` → Invalid("channel depth")
- `two centres 0.001 mm apart` → Invalid("channel row")
- `Width=wide` → Invalid("channel width")

## Tests

- `cargo test -p om9-core --test channel_cutters the_channel_builder_sweeps_one_closed_trench_through_the_row`
- `cargo test -p om9-app --test channel_cutters the_channel_icon_stages_a_preview_and_enter_commits_one_transaction`
- `cargo test -p om9-app --test channel_cutters every_row_command_resolves_from_the_typed_line_and_needs_a_gem`

## Limitations

- The channel is a swept 4-corner prism (one station per stone plus two end caps), not a B-Rep solid.
- A channel narrower than its widest stone is refused instead of clipped: the stones sit in the channel it cuts.
- The end profiles shape the two end caps only; the middle section (Middle=Wide|Deep) is a third profile.
