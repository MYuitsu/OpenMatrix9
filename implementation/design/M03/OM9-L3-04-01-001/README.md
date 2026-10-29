# OM9-L3-04-01-001 — Transform · Scale 3D — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-001`
- Authored script: `! _Scale`
- Menu: Transform ▸ Scale 3D (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Uniform scale about the base point along the three CPlane axes, composed from `gumball::scaling_along`.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for scale: ` | point |
| 2 | `Scale factor: ` | number |


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Invalid("scale factor")`; the run stays open on "Scale factor: " and the document is untouched

## Limits of this row

- No additional limit beyond the shared module note.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_001`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: box 10x10x10 at the world origin, base point 0,0,0, factor 2 → the far corner moves from (10,10,10) to (20,20,20); exactly one Undo entry.
