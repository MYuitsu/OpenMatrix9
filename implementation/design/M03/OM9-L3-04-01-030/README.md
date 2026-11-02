# OM9-L3-04-01-030 — Transform · Set Points — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-030`
- Authored script: `! _SetPt`
- Menu: Transform ▸ Set Points (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Sets the object placement origin; control-point editing of a mesh or B-Rep is out of scope for this row.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `New object position: ` | point |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid` from the coordinate parser; the object keeps its placement

## Limits of this row

- Sets the object placement origin; per-control-point editing is out of scope.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_030`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 1x1x1 at the world origin, position 5,6,7 → the object origin lands on (5,6,7).
