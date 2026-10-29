# OM9-L3-04-01-004 — Transform · Scale Non-Uniform — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-003`
- Authored script: `! _ScaleNU`
- Menu: Transform ▸ Scale Non-Uniform (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Three independent factors in the CPlane frame.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for scaleNU: ` | point |
| 2 | `X scale factor: ` | number |
| 3 | `Y scale factor: ` | number |
| 4 | `Z scale factor: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("scale factor")` at the X prompt, so the remaining factors are never collected

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_004`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x10x10 at the world origin, base point 0,0,0, factors 2 / 3 / 4 → the far corner moves from (10,10,10) to (20,30,40).
