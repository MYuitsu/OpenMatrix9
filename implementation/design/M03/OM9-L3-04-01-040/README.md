# OM9-L3-04-01-040 — Transform · Soft Move — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-040`
- Authored script: `! _SoftMove`
- Menu: Transform ▸ Soft Move (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

points inside the falloff radius move by the picked displacement with a smoothstep weight `(1 - (d/r)^2)^2`: the base moves by the whole displacement, the weight and its slope reach zero at the radius, and geometry outside the radius is untouched.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for soft move: ` | point |
| 2 | `Displacement point: ` | point |
| 3 | `Falloff radius: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a displacement of zero or a radius of 0 or below -> refused at its own prompt; a solid -> Error::Unsupported("deformation of a solid")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the falloff is the fixed smoothstep of the row; the radius and the displacement are the only controls
- the geometry kind must carry control points (point, curve, surface, mesh, cloud); a solid is refused

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_040_soft_move_moves_only_inside_the_falloff`
- `cargo test -p om9-app --test transform_menu m03_04_01_deform_rows_refuse_a_solid`
- `cargo test -p om9-core --lib deform`

The golden case: 2 mm mesh cube at the origin, base (0,0,0), displacement point (0,0,5), radius 1.5 -> the corner at the base moves to (0,0,5) while every corner outside the radius keeps its coordinates, one Undo entry.
