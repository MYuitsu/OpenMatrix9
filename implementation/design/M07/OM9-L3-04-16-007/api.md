# Cutter Body Preserve Remove Option (`OM9-L3-04-16-007`)

Feature: `BuilderGemCutter` · commands: `BuilderGemCutter`, `BuilderGemCutterLibrary`

## Owner

- `crates/om9-app/src/session/gem_cutters.rs`
- `crates/om9-core/src/cutters/boolean.rs`

## Symbols

- `cutters::boolean::CutterKeep`
- `cutters::boolean::CutterCut::keep_cutters`
- `session::gem_cutters::GemCutterSession::keep`
- `Controller::gem_cutter_commit`

## Command options

- `Preserve=preserve`
- `Preserve=remove`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `Preserve=Remove with Boolean=Off` → Invalid("cutter preserve needs a boolean") and the preview survives
- `Preserve=maybe` → Invalid("cutter preserve option")

## Tests

- `cargo test -p om9-app --test gem_cutters the_interactive_boolean_cuts_the_metal_and_the_preserve_answer_removes_the_bodies`
- `cargo test -p om9-app --test gem_cutters a_removed_body_without_a_boolean_is_refused_and_the_preview_survives`
- `cargo test -p om9-core --lib cutters::boolean`

## Limitations

- The answer applies to the cut transaction only; a cutter committed with the boolean off always stays as its own body.
