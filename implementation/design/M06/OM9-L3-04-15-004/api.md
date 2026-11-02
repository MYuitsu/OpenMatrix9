# OM9-L3-04-15-004 — Metal Piece From Gems

## Command

- feature id: `BuilderMetalPiece`
- aliases: `MetalPiece`, `Metal Piece`
- option row: `Mode=`, `Wall=`, `Height=`, `Depth=`, `Offset=`, `Border=`, `Enter`, `Cancel`

## API

| function | contract |
| --- | --- |
| `settings::metal_piece::MetalPieceMode` | outside, inside, curve, top, round corners, pinched ends, ... closed variants |
| `settings::metal_piece::MetalPieceParams` | wall 0.8, height 1.0, depth 1.2, floor 0.6, offsets, use_border |
| `settings::metal_piece::build` | shared row -> MetalPiece {bodies, volume_mm3, is_cutter, closed_row, on_border, on_curve} |
| `settings::metal_piece::build_with_border` | committed channel border -> walls whose inner face lies on the border curve |
| `settings::metal_piece::build_along_curve` | picked curve -> walls whose inner face lies on that curve |

## Errors

- `Mode=Curve without a picked curve` → `missing metal piece curve`
- `Border=On before any channel border is committed` → `missing metal piece channel border`
- `no gem selected` → `missing gem row selection`
- `Wall=-1` → `invalid metal piece wall`

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
