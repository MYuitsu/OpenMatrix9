# Gem Cutter Multiple Stones (`OM9-L3-04-16-002`)

Feature: `BuilderGemCutter` · commands: `BuilderGemCutter`

## Owner

- `crates/om9-core/src/cutters/gem.rs`
- `crates/om9-app/src/session/gem_cutters.rs`

## Symbols

- `cutters::gem::build_for_stones`
- `session::gem_cutters::GemCutterSession::open`
- `Controller::gem_cutter_commit`

## Command options

- `the same option contract as the single-stone builder, applied to every selected stone`

## Input contract

The builder reads the gem owner's `om9.gem` record for each selected stone and the document placement of that entity; a metal mesh selected with the stones becomes the Interactive Boolean surface. Nothing else is read and no state is written before Enter.

## Output contract

One `om9.cutter` record per committed cutter body (`kind` = `gem_cutter` | `gem_cutter_library`, `seat` = mode or preset, volume, clearance, boolean/preserve answers, the typed parameters) and, for a cut, one `om9.cutter.cut` record on the metal body with the removed volume. Enter commits exactly one host transaction, so Undo restores the whole command.

## Refusals

- `build_for_stones(&[])` → Missing("cutter gems")
- `a mesh entity in the selection` → it is not a stone; it becomes the Boolean surface candidate instead

## Tests

- `cargo test -p om9-core --test cutters multiple_stones_get_one_cutter_each`
- `cargo test -p om9-core --lib cutters::gem::tests::multiple_stones_get_one_cutter_each_and_an_empty_group_is_refused`

## Limitations

- Every stone gets the same parameter set; per-stone overrides are not part of this owner.
- A mixed selection (gems plus one metal body) uses the metal body as the Interactive Boolean surface.
