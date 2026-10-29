# Multiple Stone Interactive Boolean (`OM9-L3-04-17-006`)

Feature: `BuilderChannelBuilder` · commands: `BuilderChannelBuilder`, `BuilderMicroProngCutter`, `BuilderBrightCutChannel`

## Owner

- `crates/om9-core/src/cutters/boolean.rs`
- `crates/om9-app/src/session/channel_cutters.rs`

## Symbols

- `cutters::boolean::apply`
- `session::channel_cutters::RowCutterSession::recompute`
- `session::Controller::row_cutter_commit`
- `model::CUTTER_CUT_EXTENSION_SLOT`

## Command options

- `Boolean=off|on|one`
- `Preserve=preserve|remove`

## Input contract

The builder reads the gem owner's `om9.gem` record and the document placement of each selected stone, in selection order; that order is the row. Every selected closed mesh body that is not a gem and not already an `om9.cutter` is an Interactive Boolean target. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `channel_cutter` | `micro_prong_cutter` | `bright_cut_channel` | `bright_cutter`) carrying `row` (the stone labels in row order), the volume, the Boolean/Preserve answers and the typed parameters; for a cut, one `om9.cutter.cut` record on each replaced metal body with its removed volume and the same row.

## Refusals

- `Boolean=On with gems only` → Missing("cutter boolean surface")
- `Preserve=Remove` → Invalid("cutter preserve needs a boolean")
- `Boolean=sometimes` → Invalid("cutter boolean")
- `a block far from the metal` → Invalid("cutter boolean removed nothing")

## Tests

- `cargo test -p om9-app --test channel_cutters the_interactive_boolean_cuts_every_selected_metal_body`
- `cargo test -p om9-app --test channel_cutters a_boolean_without_a_metal_body_is_refused_and_keeps_the_preview`
- `cargo test -p om9-core --test cutters the_interactive_boolean_keeps_or_removes_the_cutter_bodies`

## Limitations

- The Boolean reuses the one mesh CSG service (mesh_csg::mesh_boolean); it never runs OCCT inside the adapter.
- Every target must be a closed mesh body; a surface body is refused instead of silently meshed.
- The cut is applied in world space, so a row cut through several bodies is one transaction but not one CSG union.
