# OM9-L3-04-01-024 — Transform · Array Along Surface — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-023`
- Authored script: `! _ArraySrf`
- Menu: Transform ▸ Array Along Surface (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

the evaluated grid of the selected surface (surface_kernel::evaluate over its own domain) gives the stations, the source stays at the first corner and every other row/column pair becomes one copy; the optional switch also aligns each copy with surface_kernel::normal at its station.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Number of rows: ` | number |
| 2 | `Number of columns: ` | number |
| 3 | `Orient copies to the surface normal (0 = no, 1 = yes): ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- rows x columns above MAX_CURVE_ITEMS (512) -> Error::Invalid("array count"); a count below 2 is refused at its prompt
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the grid is the surface's own parameter rectangle, not an evenly spaced arc-length grid
- the surface and the objects are selected together

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_024_array_along_surface_places_copies_on_the_surface_grid`
- `cargo test -p om9-app --test transform_menu m03_04_01_reference_rows_refuse_a_selection_without_their_reference`

The golden case: 10 x 10 reference plane, box 0..2, 2 rows x 2 columns, switch 0 -> four objects with centres (1,1,1), (1,11,1), (11,1,1) and (11,11,1), one Undo entry.
