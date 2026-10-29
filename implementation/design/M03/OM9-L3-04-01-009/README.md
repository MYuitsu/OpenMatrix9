# OM9-L3-04-01-009 — Transform · Array Polar — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-009`
- Authored script: `! _ArrayPolar`
- Menu: Transform ▸ Array Polar (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

`gumball::polar_array` with `include_source = false`, committed as `Mutation::Insert` copies.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Center of array: ` | point |
| 2 | `Number of items: ` | number |
| 3 | `Total angle in degrees: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("array count")`; an array needs at least two items

## Limits of this row

- Copies are new objects; there is no history-linked instance form.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_009`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box at (1,0,0), center 0,0,0, count 4, total angle 360 → three copies a quarter turn apart; the source does not move; one Undo entry for the whole array.
