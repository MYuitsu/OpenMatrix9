# OM9-L3-04-15-008 — Pull Object To Rail Setting

## Command

- feature id: `BuilderPullToRail`
- aliases: `PullToRail`, `Pull To Rail`
- option row: `Mode=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::rail::Rail` | circle rail: centre, axis, radius, range |
| `settings::rail::pull_to_rail` | rigid | deform -> origin on the rail, displacement, radial gap |
| `settings::rail::pull_mesh` | mesh copy moved with the same mode + MeshPullReport |

## Errors

- `selection without a circle` → `pull needs a selected circle rail`
- `Mode=Spring` → `invalid pull mode option`
- `only the rail selected` → `missing pull selection`

## Evidence

- `cargo check -p om9-core -j 2` exit 0 (2.08 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/check-core.log`
- `cargo check -p om9-app -j 2` exit 0 (0.36 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/check-app.log`
- `cargo test -p om9-core --lib -j 2 settings::` exit 0 (2.93 s, 90 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/core-settings.log`
- `cargo test -p om9-core --test settings_family -j 2` exit 0 (4.41 s, 14 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/core-family.log`
- `cargo test -p om9-app --test settings_commands -j 2` exit 0 (40.86 s, 24 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/app-settings.log`
- `cargo test -p om9-core -j 2` exit 0 (56.63 s, 781 tests) → `implementation/evidence/M06/OM9-L3-04-15-001/logs/core-all.log`

Every committed prong body carries the `om9.settings.prong` payload (feature id, prong id,
site and parameters); the editor and the settings mass read-out use it instead of guessing,
and a selection without it is refused. SettingsMass has no toolbar icon; it is a text
command, and the row end modes are options on the prong commands (`Ends=`, `Spacing=`).
