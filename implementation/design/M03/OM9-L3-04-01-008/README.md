# OM9-L3-04-01-008 — Transform · Bend — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-007`
- Authored script: `! _Bend`
- Menu: Transform ▸ Bend (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

the span along the picked axis is bent into an arc of the typed angle in the plane of the axis and the bend direction: a point is written in the (along, height, section) frame of that plane and laid on the arc of radius `span / theta`, so the section keeps its size and the far end arrives at the commanded angle.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for bend: ` | point |
| 2 | `Bend axis end point: ` | point |
| 3 | `Point in the bend direction: ` | point |
| 4 | `Bend angle in degrees: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a bend direction parallel to the axis -> Error::Invalid("bend direction is the axis"); a zero or non-finite angle -> refused at its own prompt and the run continues on the same question
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- a solid is refused: deforming a B-Rep needs a kernel deformer, and the row will not report a display-mesh deformation as a solid deformation
- a straight segment becomes a polycurve of 17 mapped samples because a bend no longer contains a straight line; the sample count is fixed so a record and a test name the same geometry
- the bend lies in one plane; there is no two-direction bend and no editable bend radius

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_008_bend_turns_the_curve_into_an_arc`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: path (0,0,0)->(10,0,0), base (0,0,0), axis (1,0,0), bend direction (0,1,0), 90 degrees -> the far end lands on (10/(pi/2), 10/(pi/2), 0) = (6.3662, 6.3662, 0) as a polycurve of the mapped samples, one Undo entry.
