# OM9-L3-04-01-015 — Transform · Orient on Surface — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-013`
- Authored script: `! _OrientOnSrf`
- Menu: Transform ▸ Orient on Surface (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

the picked point is projected onto the selected surface by curve_surface_ops::closest_point_on_surface, the surface normal comes from surface_kernel::normal at the projected parameters, and transform_session::align_transform rotates each object so its CPlane normal follows that normal while its reference point lands on the projected point.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on surface to orient at: ` | point |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- no surface in the selection -> Error::Missing("reference surface selection") before the first prompt; the document and the Undo journal are untouched
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the surface comes from the selection: the shell has no mid-command object pick, so the objects and the surface are selected together
- the normal is the geometric surface normal; there is no flip option and no editable snap distance

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_015_orient_on_surface_stands_the_object_on_the_surface`
- `cargo test -p om9-app --test transform_menu m03_04_01_reference_rows_refuse_a_selection_without_their_reference`

The golden case: tilted reference surface (10u, 10v, 10u), box 0..2, point (5,5,5) -> the object's reference point lands on (5,5,5) and its Z axis is the surface normal (-1,0,1)/sqrt 2, one Undo entry.
