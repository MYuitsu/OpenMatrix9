# OM9-L3-04-14-007 — Head Bezel Profile Browser

## Command

- feature id: no separate icon: options on the head/bezel commands
- aliases: `Profile=`, `RailProfile=`, `Edge profile library`
- option row: `Profile=`, `ProfileSave=`, `RailProfile=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::ProfileKind` | Prong, Rail and Edge libraries |
| `settings::head::Profile` | closed unit section with width/height validation |
| `settings::head::Profile::from_section` | a user section normalised to the unit profile frame |
| `settings::head::Profile::from_bezier` | closed cubic Bezier sampled into a typed user profile |
| `settings::head::ProfileLibrary::builtin` | read-only built-in entries; user records use the same typed list |
| `settings::head::ProfileLibrary::find` | name lookup with a typed error per library |

## Errors

- `Profile=Unobtainium on the head` → `invalid prong profile`
- `Profile=Nonexistent on the bezel` → `invalid edge profile`
- `Bezier with 2 control points` → `invalid profile bezier control`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
