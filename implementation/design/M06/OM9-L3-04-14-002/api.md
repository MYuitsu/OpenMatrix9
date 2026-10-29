# OM9-L3-04-14-002 — Head Builder Half Bezel Mode

## Command

- feature id: `BuilderHead`
- aliases: `HeadBuilder`, `Head Builder`, `gvHeadBuilder`
- option row: `Mode=HalfBezel`, `Span=`, `Position=`, `Thickness=`, `BezelHeight=`, `SeatAngle=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::HeadMode` | Basket or Half Bezel, the manual's shape toggle |
| `settings::head::HalfBezelParams` | position, span, thickness, height and seat angle of the band |
| `settings::head::build_half_bezel` | one closed band following the girdle arc; the prong profile is ignored |

## Errors

- `Half bezel span 0` → `invalid half bezel span`
- `Half bezel thickness below 0.05 mm` → `invalid half bezel thickness`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
