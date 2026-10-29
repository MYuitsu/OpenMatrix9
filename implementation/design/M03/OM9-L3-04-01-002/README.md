# OM9-L3-04-01-002 — Transform · Scale2D — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-002`
- Authored script: `! _Scale2D`
- Menu: Transform ▸ Scale2D (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Two-axis scale in the CPlane frame, so the normal coordinate is untouched.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for scale2D: ` | point |
| 2 | `Scale factor: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("scale factor")`; a negative factor is refused instead of flipping the object

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_002`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x10x10 at the world origin, base point 0,0,0, factor 2 → the far corner moves from (10,10,10) to (20,20,10): the CPlane X and Y axes scale, the normal does not.
