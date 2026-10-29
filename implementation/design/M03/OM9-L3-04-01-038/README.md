# OM9-L3-04-01-038 — Transform · Remap to C-Plane — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-038`
- Authored script: `! _RemapCPlane`
- Menu: Transform ▸ Remap to C-Plane (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Maps the source plane's X axis onto the CPlane X axis, then translates the source origin onto the CPlane origin.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Source plane origin: ` | point |
| 2 | `Source plane X point: ` | point |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid` from the align solve; the object keeps its placement

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_038`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box at the world origin, source origin 0,0,0, source X point 1,0,0 → the source plane already is the CPlane, so the placement is unchanged.
