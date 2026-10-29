# OM9-L3-04-01-025 — Transform · Array Curve on Surface — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-026`
- Authored script: `! _ArrayCrvOnSrf`
- Menu: Transform ▸ Array Curve on Surface (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

the selected path curve is sampled at equal arc-length stations (curve_ops::arc_length plus curve_ops::parameter_at_length), each station is projected onto the selected surface by curve_surface_ops::closest_point_on_surface, and every projected point becomes one copy; the optional switch also aligns each copy with surface_kernel::normal.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Number of items: ` | number |
| 2 | `Orient copies to the surface normal (0 = no, 1 = yes): ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- no surface (or no curve) in the selection -> Error::Missing("reference surface selection") / Error::Missing("path curve selection") before the first prompt; a path of zero length -> Error::Invalid("array path length")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the projection is a nearest-point attachment: there is no attachment tolerance and no trimming of the copies to the surface
- the curve, the surface and the objects are selected together

## Milestone ownership

the row was blocked on a missing closest-point service; this bridge uses the closest-point/pullback services that exist in om9-core today.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_025_array_curve_on_surface_projects_the_stations_onto_the_surface`
- `cargo test -p om9-app --test transform_menu m03_04_01_reference_rows_refuse_a_selection_without_their_reference`

The golden case: 10 x 10 reference plane, path (0,0,0)->(10,0,0), box 0..2, count 3, switch 0 -> three objects with centres (1,1,1), (6,1,1) and (11,1,1), one Undo entry.
