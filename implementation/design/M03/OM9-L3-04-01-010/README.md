# OM9-L3-04-01-010 — Transform · Project To C-Plane — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-010`
- Authored script: `! _ProjectToCPlane`
- Menu: Transform ▸ Project To C-Plane (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

Per-object translation along the plane normal; no typed input, so the command commits immediately.

## Input contract

No typed input: the row commits as soon as it starts.


Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- A locked or hidden object never reaches the command: the selection refuses it first, and
  `begin_transform` repeats the check as defence in depth.
- `Error::Missing("transform selection")`; the command is refused before any edit

## Limits of this row

- Whole objects move by their placement-origin height; individual vertices are not projected.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_010`
- `cargo test -p om9-app --lib -j 2` (unit tests for the field table and scalar rules)

The golden case: one box on the CPlane and one raised 10 mm, both selected → each object moves along the normal by its own signed height, so both end on the plane, in one Undo entry.
