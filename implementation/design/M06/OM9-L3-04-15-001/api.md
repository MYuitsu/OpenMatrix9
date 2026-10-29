# OM9-L3-04-15-001 — Prong Builder Gem Line

## Command

- feature id: `BuilderProngAdder`
- aliases: `ProngBuilder`, `Prong Builder`, `ProngAdder`
- option row: `Mode=`, `Diameter=`, `Height=`, `Nudge=`, `Ends=`, `Spacing=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::gems::GemRow / straight_row` | shared input row, never re-implemented |
| `settings::prong::place_prongs` | sites only: gem, end and junction prongs in row order |
| `settings::prong::place_prongs_with_report` | sites plus the count of facing prongs merged at one shared point |
| `settings::prong::build_prong_set` | sites + one closed body per prong + volume + merged_prongs |
| `settings::prong::build_prong` | one closed prong body from a site |

## Errors

- `no gem selected` → `missing gem row selection`
- `Diameter=-2` → `invalid settings numeric option`
- `Mode=Not A Style` → `invalid prong mode option`
- `Nudge=0.6` → `invalid prong nudge`

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
