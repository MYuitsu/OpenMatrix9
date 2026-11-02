# OM9-L3-04-01-029 — Transform · Splop — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-029`
- Authored script: `! _Splop`
- Menu: Transform ▸ Splop (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

every control point is moved to the nearest point of the selected surface with curve_surface_ops::closest_point_on_surface, so geometry that does not touch the surface is wrapped onto it.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| — | (no prompt: the frames come from the selection) | — |

Every point is resolved through the CPlane captured at start.

## What it refuses

- no surface, or more than one -> Error::Missing("reference surface selection") / Error::Invalid("one reference surface") before the row runs; a solid object -> Error::Unsupported("deformation of a solid")
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- the attachment is the nearest point of the surface: there is no along-normal projection and no distance limit, so a point far from the surface still wraps
- the surface is the reference and is never deformed
- a solid is refused

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_029_splop_wraps_the_selection_onto_the_surface`
- `cargo test -p om9-core --lib deform`

The golden case: 10 x 10 plane at z=0, curve (1,1,2)->(3,1,2) -> the ends land on (1,1,0) and (3,1,0), one Undo entry.
