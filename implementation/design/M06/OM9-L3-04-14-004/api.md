# OM9-L3-04-14-004 — Bezel Builder Full

## Command

- feature id: `BuilderBezel`
- aliases: `BezelBuilder`, `Bezel Builder`, `gvBezelBuilder`
- option row: `Mode=`, `Profile=`, `ProfileSave=`, `Height=`, `Top=`, `Base=`, `Dome=`, `Angle=`, `Seat=`, `Placement=`, `Scale=`, `ChamferX=`, `ChamferZ=`, `Shear=`, `Boolean=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::bezel::BezelParams` | typed edge profile, height, top/base, dome, angles, placement, scale, chamfer, shear, boolean and the finger rail |
| `settings::bezel::section` | closed (radial, axial) section, sheared, upright or with the Boolean groove cut out |
| `settings::bezel::finger_rail` | the rail ring the Boolean subtracts; committed as its own Finger Rail body |
| `settings::bezel::build` | one closed ring swept along the girdle outline, plus the removed volume of the Boolean |
| `settings::bezel::BezelMode` | Bezel, Under Bezel and Gallery Rail |

## Errors

- `Finger rail size 0` → `invalid finger rail size`
- `Height below 0.1 mm` → `invalid bezel height`
- `Seat length above half the base thickness` → `invalid bezel seat length`

## Evidence

- `cargo test -p om9-core --lib settings::head -j 2` exit 0 (0.34 s, 12 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_head.log`
- `cargo test -p om9-core --lib settings::bezel -j 2` exit 0 (0.21 s, 10 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_bezel.log`
- `cargo test -p om9-app --test head_commands -j 2` exit 0 (1.16 s, 9 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_head.log`
- `cargo test -p om9-core --lib -j 2` exit 0 (0.37 s, 171 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/core_lib.log`
- `cargo check -p om9-app -j 2` exit 0 (1.85 s, 0 tests) → `implementation/evidence/M06/OM9-L3-04-14-001/logs/app_check.log`

Automated evidence only. The manual M06-UI review is not signed.
