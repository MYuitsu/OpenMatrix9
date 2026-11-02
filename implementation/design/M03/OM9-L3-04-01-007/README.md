# OM9-L3-04-01-007 — Transform · Rotate3D — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-008`
- Authored script: `! _Rotate3D`
- Menu: Transform ▸ Rotate3D (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Rotation about the CPlane normal through the typed center, through `gumball::rotation_about`.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Center of rotation: ` | point |
| 2 | `Rotation angle in degrees: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("rotation angle")`; a zero rotation is refused rather than opening an empty Undo entry

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_007`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x0x0 at the world origin, center 0,0,0, angle 90 → the body point (10,0,0) moves onto (0,10,0).
