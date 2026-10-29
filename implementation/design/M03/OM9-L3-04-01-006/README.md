# OM9-L3-04-01-006 — Transform · Orient 3 Points — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-006`
- Authored script: `! _Orient3Pt`
- Menu: Transform ▸ Orient 3 Points (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Rotation mapping the reference direction onto the target direction, about the base point.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point: ` | point |
| 2 | `Target point: ` | point |
| 3 | `Reference direction point: ` | point |
| 4 | `Target direction point: ` | point |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid` from the align solve; the selection keeps its placement

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_006`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 1x1x1 at the world origin, base 0,0,0, target 0,0,0, reference 1,0,0, target direction 0,1,0 → the body point (1,0,0) moves onto (0,1,0): a quarter turn about the CPlane normal.
