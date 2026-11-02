# OM9-L3-04-01-033 — Transform · Taper — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-033`
- Authored script: `! _Taper`
- Menu: Transform ▸ Taper (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

the cross-section perpendicular to the picked axis is scaled by a factor that grows linearly from 1 at the base to the typed factor at the far end of the span, so a tapered object keeps its length and changes its section.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for taper: ` | point |
| 2 | `Taper axis end point: ` | point |
| 3 | `Taper factor at the far end: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a factor of 0 or below, or an axis with no length -> refused at its own prompt; a solid -> Error::Unsupported("deformation of a solid") before any mutation
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the factor is linear over the span; there is no editable taper curve and no falloff
- a solid is refused for the same reason as Bend: no kernel deformer exists

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_033_taper_scales_the_section_over_the_span`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: path (0,1,0)->(10,1,0), base (0,0,0), axis (1,0,0), factor 3 -> the base keeps its 1 mm offset and the far end sits at (10,3,0), one Undo entry.
