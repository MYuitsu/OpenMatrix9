# OM9-L3-04-14-006 — Scallop Bezel Builder

## Command

- feature id: `BuilderBezelCutter`
- aliases: `ScallopBezel`, `Scallop Bezel`, `BezelCutter`, `gvBezelCutter`
- option row: `Profile=`, `ProfileSave=`, `Count=`, `Angle=`, `Offset=`, `Depth=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::bezel::ScallopParams` | profile, count, pattern angle, offset and radial depth |
| `settings::bezel::build_scallop` | one closed cutter body per position, extruded along the outline tangent |
| `settings::bezel::ScallopSetting::is_cutter` | cutters never enter the settings metal mass |
| `session::settings_commands::SettingsMassReport` | publishes the cutter volume and the net metal |

## Errors

- `Count 200` → `invalid scallop count`
- `Unknown edge profile` → `invalid edge profile`
- `Depth below 0.1 mm` → `invalid scallop depth`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
