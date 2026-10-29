# OM9-L3-04-14-009 — Head Bezel Geometry Validity And Update

## Command

- feature id: no separate icon: options on the head/bezel commands
- aliases: `head payload`, `rebuild of a committed head`
- option row: `Level=`, `Enter`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::HeadFeature` | feature id, gem id, typed params and revision |
| `settings::head::check` | measured closed/positive-volume/deterministic report with the gem dependency |
| `settings::head::rebuild` | identical inputs reproduce identical bodies |
| `settings::head::update` | same gem preserves ids, a new gem replaces them |
| `session::head_commands::HeadPayload` | the committed bodies carry the feature state for the next edit |

## Errors

- `Rebuild a feature on a different gem id` → `invalid head gem dependency`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
