# OM9-L3-04-01-005 — Transform · Orient 2 Points — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-005`
- Authored script: `! _Orient`
- Menu: Transform ▸ Orient 2 Points (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Two-point orient is a pure translation solved by `transform_session::align_transform`.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point: ` | point |
| 2 | `Target point: ` | point |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid` from the coordinate parser; the run stays open and the document is untouched

## Limits of this row

- Translation only; the dialog's scale-between-points mode is not exposed.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_005`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 1x1x1 at the world origin, base point 0,0,0, target 5,5,5 → the object origin lands on (5,5,5) through one translation.
