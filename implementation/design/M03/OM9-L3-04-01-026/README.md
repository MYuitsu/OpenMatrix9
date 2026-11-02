# OM9-L3-04-01-026 — Transform · Array — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-025`
- Authored script: `! _Array`
- Menu: Transform ▸ Array (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Rectangular grid in the CPlane frame; the source is item (0,0).

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Number of columns: ` | number |
| 2 | `Number of rows: ` | number |
| 3 | `Column spacing in mm: ` | number |
| 4 | `Row spacing in mm: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("array count")`; a one-cell grid is not an array

## Limits of this row

- Copies are new objects; there is no history-linked instance form.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_026`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 1x1x1 at the world origin, 2 columns, 3 rows, spacing 10 / 20 → six objects in total (source plus five copies).
