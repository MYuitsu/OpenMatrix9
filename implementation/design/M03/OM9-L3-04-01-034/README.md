# OM9-L3-04-01-034 — Transform · Shear — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-034`
- Authored script: `! _Shear`
- Menu: Transform ▸ Shear (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

an affine shear: the component of a point measured along the measure direction moves it by `factor * that component` along the shear direction, so planes along the shear direction slide while the base stays fixed and curves keep their type.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for shear: ` | point |
| 2 | `Point along the measure direction: ` | point |
| 3 | `Point along the shear direction: ` | point |
| 4 | `Shear factor: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- two parallel directions -> Error::Invalid("shear directions are parallel"); a non-finite factor or a missing point -> refused at its own prompt
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the shear is affine, so a line stays a line and a placement-only shape could have been used; the row still writes geometry because the measure and shear directions are arbitrary
- a solid is refused: a sheared B-Rep needs a kernel deformer

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_034_shear_slides_the_section_along_the_picked_direction`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: path (0,1,0)->(10,1,0), base (0,0,0), measure (1,0,0), shear (0,1,0), factor 0.5 -> the far end sits at (10,6,0) and the curve is still a line, one Undo entry.
