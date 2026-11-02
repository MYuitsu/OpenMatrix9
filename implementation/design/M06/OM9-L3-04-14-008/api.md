# OM9-L3-04-14-008 — Head Bezel Girdle Seat Attachment

## Command

- feature id: no separate icon: options on the head/bezel commands
- aliases: `Seats=`, `seat frames of the head`
- option row: `Seats=`, `SeatAngle=`, `SeatDepth=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::head::SeatParams` | count, seat angle, depth and offset |
| `settings::head::seat_frames` | one stable frame per prong on the girdle plane |
| `settings::head::seat_cutters` | one closed wedge cutter per seat, angled by the seat angle |
| `settings::head::cut_seats` | subtracts the seats from the stone and returns the seated mesh and removed volume |
| `settings::head::SeatFrame` | id, gem id, angle, origin, unit normal, tangent, angle and depth |

## Errors

- `Seat count 2` → `invalid seat count`
- `Seat angle 90` → `invalid seat angle`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
