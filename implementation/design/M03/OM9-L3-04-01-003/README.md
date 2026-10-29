# OM9-L3-04-01-003 — Transform · Scale1D — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-004`
- Authored script: `! _Scale1D`
- Menu: Transform ▸ Scale1D (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Single-axis scale along the CPlane X axis.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for scale1D: ` | point |
| 2 | `Scale factor: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Missing("transform selection")`; the command never asks for a base point

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_003`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x10x10 at the world origin, base point 0,0,0, factor 3 → the far corner moves from (10,10,10) to (30,10,10).
