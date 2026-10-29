# OM9-L3-04-01-035 — Transform · Twist — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-036`
- Authored script: `! _Twist`
- Menu: Transform ▸ Twist (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

the cross-section rotates about the picked axis by an angle that grows linearly from 0 at the base to the typed angle at the far end; the axis direction is invariant and the rotation follows the right-hand rule.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for twist: ` | point |
| 2 | `Twist axis end point: ` | point |
| 3 | `Twist angle in degrees: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- an axis with no length or a zero angle -> refused at its own prompt; a solid -> Error::Unsupported("deformation of a solid")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the twist is linear in the distance along the axis; there is no per-station twist curve
- a solid is refused: no kernel deformer exists for a twisted B-Rep

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_035_twist_turns_the_section_over_the_span`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: path (0,1,0)->(10,1,0), base (0,0,0), axis (1,0,0), 90 degrees -> the 1 mm offset turns onto +Z, so the far end sits at (10,0,1), one Undo entry.
