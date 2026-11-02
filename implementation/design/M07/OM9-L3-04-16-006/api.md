# Interactive Boolean On Metal (`OM9-L3-04-16-006`)

Feature: `BuilderGemCutter` · commands: `BuilderGemCutter`, `BuilderGemCutterLibrary`

## Owner

- `crates/om9-core/src/cutters/boolean.rs`
- `crates/om9-app/src/session/gem_cutters.rs`

## Symbols

- `cutters::boolean::BooleanMode`
- `cutters::boolean::apply`
- `cutters::boolean::CutterCut`
- `mesh_csg::mesh_boolean`
- `model::CUTTER_CUT_EXTENSION_SLOT`

## Command options

- `Boolean=off|on|one`
- `Preserve=preserve|remove (applies to the committed cut)`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `Boolean=On with gems only` → Missing("cutter boolean surface")
- `apply(..., BooleanMode::Off, ...)` → Invalid("cutter boolean mode")
- `apply(target, &[], ...)` → Missing("cutter bodies")
- `a block 50 mm away` → Invalid("cutter boolean removed nothing")
- `a single triangle` → the CSG owner refuses it ("CSG requires closed consistently oriented mesh")

## Tests

- `cargo test -p om9-core --test cutters the_interactive_boolean_keeps_or_removes_the_cutter_bodies`
- `cargo test -p om9-core --lib cutters::boolean`
- `cargo test -p om9-app --test gem_cutters the_interactive_boolean_cuts_the_metal_and_the_preserve_answer_removes_the_bodies`

## Limitations

- The boolean uses the one shared mesh CSG service (mesh_csg::mesh_boolean); it never runs OCCT inside the adapter.
- The cut target must be a closed mesh body; a B-Rep surface body is refused rather than silently meshed.
