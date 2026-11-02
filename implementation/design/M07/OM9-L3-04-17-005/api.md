# Channel Cutter Profile And End Controls (`OM9-L3-04-17-005`)

Feature: `BuilderChannelBuilder` · commands: `BuilderChannelBuilder`

## Owner

- `crates/om9-core/src/cutters/channel.rs`
- `crates/om9-app/src/session/channel_cutters.rs`
- `crates/om9-app/src/ui/channel_cutters.rs`

## Symbols

- `cutters::channel::ChannelEnd`
- `cutters::channel::ChannelMiddle`
- `cutters::channel::ChannelParams::ends`
- `cutters::channel::ChannelParams::validate`

## Command options

- `Start=Square|Round|Taper`
- `End=Square|Round|Taper`
- `StartDepth=<mm>`
- `EndDepth=<mm>`
- `Mirror=on|off`
- `Flip=ab|ba`
- `Middle=None|Wide|Deep`
- `Spacing=<mm>`

## Input contract

The builder reads the gem owner's `om9.gem` record and the document placement of each selected stone, in selection order; that order is the row. Every selected closed mesh body that is not a gem and not already an `om9.cutter` is an Interactive Boolean target. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `channel_cutter` | `micro_prong_cutter` | `bright_cut_channel` | `bright_cutter`) carrying `row` (the stone labels in row order), the volume, the Boolean/Preserve answers and the typed parameters; for a cut, one `om9.cutter.cut` record on each replaced metal body with its removed volume and the same row.

## Refusals

- `Start=Bevel` → Invalid("channel end profile")
- `Middle=Slant` → Invalid("channel middle profile")
- `Spacing=-1` → Invalid("channel height")
- `StartDepth=0` → Invalid("channel end depth")

## Tests

- `cargo test -p om9-core --test channel_cutters the_channel_end_profiles_shape_the_caps`
- `cargo test -p om9-core --test channel_cutters the_channel_mirror_and_flip_swap_the_ends`
- `cargo test -p om9-core --test channel_cutters the_channel_middle_profile_adds_a_station`
- `cargo test -p om9-app --test channel_cutters the_channel_options_reach_the_cutter_and_a_bad_width_is_refused`

## Limitations

- Mirror picks the wider of the two end profiles for both ends and the deeper of the two caps; it does not invent a third profile.
- Flip swaps A and B; the A/B ends are the first and last station of the row as selected, so the selection order is the orientation.
- The manual's profile IDs are mapped to three typed end profiles (Square, Round, Taper); a user profile library is not part of this owner.
