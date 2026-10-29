# M04 L2-03-12 builder styles and presets

This directory records the builder style/preset batch for `OM9-L2-03-12`
(03-febe-matrix-builder-styles-and-presets). It reuses the builder session and the producer
preset vocabulary from `OM9-L2-03-11`; no second builder or profile library was added.

## What is implemented

- `om9_core::builder_style` — the `om9.builder_style.v1` file, typed `StyleValue`
  (`length_mm`/`number`/`choice`/`boolean`), `BuilderStyle::snapshot`/`defaults_of`/`validate`,
  `StyleLibrary` (built-ins + user list), `StyleThumbnail`, `match_attributes`.
- Browser — `Controller::builder_styles` returns one row per style for the open mode; an
  incompatible style is listed but disabled, and a missing/corrupt thumbnail falls back to the
  placeholder instead of dropping the row.
- Apply — `BuilderStyle::apply` loads the parameters into the session (the preview source) and
  reports the applied/ignored names; the document is untouched until the builder's Apply. A
  style without the mandatory `profile_ref` is refused with `Missing("style profile")`.
- Save — `StyleLibrary::save_user` validates the name (no separators, traversal, control text or
  duplicates) and never rewrites the built-ins; `save_to`/`load_from` move the user list through
  `OM9_STYLE_LIBRARY`, and a refused write is reported.
- Filter — `StyleFamily::matches`/`BuilderKind::tag` gate the list; a gem style is refused by a
  ring builder with `Invalid("style mode")`.
- Match attributes — declared `om9.rail`/`om9.head` members (`width_mm`, `height_mm`,
  `material`) are copied with their types; the source entity keeps its id and slots.
- UI — the builder panel's `Styles…` button opens the Builder Styles window (rows, selected
  marker, Save field, Match action, user-file line); the command line accepts `Style=<name>`,
  `StyleSave=<name>` and `StyleMatch`.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
| `OM9-L3-03-12-001` | Builder Style Param Snapshot Schema | automated_verified_manual_gate_open |
| `OM9-L3-03-12-002` | Builder Style Browser And Thumbnail | automated_verified_manual_gate_open |
| `OM9-L3-03-12-003` | Builder Style Apply And Preview | automated_verified_manual_gate_open |
| `OM9-L3-03-12-004` | Builder Style Save New Name | automated_verified_manual_gate_open |
| `OM9-L3-03-12-005` | Builder Style Mode And Shape Filter | automated_verified_manual_gate_open |
| `OM9-L3-03-12-006` | Builder Style Match Attributes | automated_verified_manual_gate_open |

## L4 packages

Each L3 carries its own L4 artefacts under `implementation/evidence/M04/<L3>/`:
`source_crosswalk.json` (SOURCE_AUDIT), `api.md` + `positive.json` + `negative.json`
(CONTRACT_TEST), and one `OM9-L4-...-0X.json` receipt per IMPLEMENT / INTEGRATION /
REGRESSION / ACCEPTANCE / ADVERSARIAL role — **27 receipts, 39 planned L4**.
A receipt names the command that ran, its exit code and the hash of its output, and keeps the
M04-UI manual gate open.

## Verification

See `observed.json` and the logs here. Focused commands:

- `cargo test -p om9-core --test builder_style`
- `cargo test -p om9-core --test builder_controls --test builder_session --test history_mutation`
- `cargo test -p om9-app --test builder_styles --test builder_panel --test builder_controls --test builder_slot --test top11`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo clippy -p om9-core --lib --no-deps -- -D warnings`
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: no human has reviewed the style browser, the apply/save gestures or
  the match action on screen, and no screenshot/pointer frame records them.
- A default user-style directory: the path comes from `OM9_STYLE_LIBRARY`, otherwise the user
  list is session-only.
- Raster thumbnails: the browser uses authored icon keys with a placeholder fallback.
- macOS runtime for any of this: the evidence is Windows x64 only.
