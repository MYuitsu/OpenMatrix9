# OM9-L3-04-01-031 — Transform · Scale By Plane — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-031`
- Authored script: `! _ScaleByPlane`
- Menu: Transform ▸ Scale By Plane (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Single-axis scaling along the CPlane normal through `gumball::scaling_along`.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for scale by plane: ` | point |
| 2 | `Scale factor: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("scale factor")`; the run stays open

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_031`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x10x10 at the world origin, base point 0,0,0, factor 2 → the far corner moves from (10,10,10) to (10,10,20): only the normal axis scales.
