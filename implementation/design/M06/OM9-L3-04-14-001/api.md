# OM9-L3-04-14-001 — Head Builder Basket

## Command

- feature id: `BuilderHead`
- aliases: `HeadBuilder`, `Head Builder`, `gvHeadBuilder`
- option row: `Mode=`, `Prongs=`, `Profile=`, `ProfileSave=`, `Rails=`, `Height=`, `Drop=`, `SeatDepth=`, `Level=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::GemGirdle::from_spec` | the gem owner's spec and placement become one girdle spine |
| `settings::head::build` | basket head: N closed prongs + gallery rails on the gem frame |
| `settings::head::HeadParams` | typed level-1 controls, validated before any geometry |
| `settings::head::ProfileLibrary` | prong/rail/edge profile library entry point |
| `settings::head::validate_clearance` | no metal vertex inside the gem surface |

## Errors

- `Head drop below the culet` → `invalid head prong below pavilion`
- `Metal body inside the pavilion` → `invalid head metal inside pavilion`
- `Selection without a gem` → `missing head gem selection`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
