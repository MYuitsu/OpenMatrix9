# OM9-L3-04-01-036 — Transform · Maelstrom — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-035`
- Authored script: `! _Maelstrom`
- Menu: Transform ▸ Maelstrom (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

a twist about the picked axis combined with a radial pull: the cross-section turns by the angle that grows over the span and shrinks by `1 - pull` at the far end, which is the spiral the command is named after.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for maelstrom: ` | point |
| 2 | `Maelstrom axis end point: ` | point |
| 3 | `Maelstrom angle in degrees: ` | number |
| 4 | `Pull between 0 and 1: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a pull outside 0..1 -> Error::Invalid("pull between 0 and 1") at its own prompt; a solid -> Error::Unsupported("deformation of a solid")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the pull is linear over the span and applies to the whole section; there is no editable falloff curve and no cap on the inner radius
- a solid is refused: no kernel deformer exists

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_036_maelstrom_twists_and_pulls_the_section_in`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: path (0,1,0)->(10,1,0), base (0,0,0), axis (1,0,0), 90 degrees, pull 0.5 -> the far end lands at (10,0,0.5), half the turned offset, one Undo entry.
