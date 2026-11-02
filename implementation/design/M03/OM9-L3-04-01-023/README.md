# OM9-L3-04-01-023 — Transform · Array Linear — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-024`
- Authored script: `! _ArrayLinear`
- Menu: Transform ▸ Array Linear (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

`gumball::rectangular_array` with one column count, spacing along the CPlane X axis.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Number of items: ` | number |
| 2 | `Spacing in mm: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("array spacing")`; copies are never stacked on the source

## Limits of this row

- Copies are new objects; there is no history-linked instance form.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_023`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 1x1x1 at the world origin, count 3, spacing 10 → two copies at x = 10 and x = 20, the source keeps its placement, one Undo entry.
