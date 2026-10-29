# OM9-L3-04-14-003 — Head Builder Prong Rail Level Edit

## Command

- feature id: `BuilderHead`
- aliases: `HeadBuilder`, `Head Builder`, `gvHeadBuilder`
- option row: `Level=`, `Prong=`, `Rail=`, `Size=`, `RailDrop=`, `RailSize=`, `RailOffset=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::HeadLevel` | the manual's four levels: overall, grouped, ends, individual |
| `settings::head::HeadEdit` | one field plus the edited element |
| `settings::head::apply_edit` | rebuilds only the permitted element set; the rest keeps ids, frames and vertices |

## Errors

- `Grouped level with an overall field` → `invalid head edit level field`
- `Prong index 0` → `invalid head prong index`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
