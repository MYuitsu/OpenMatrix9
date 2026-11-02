# OM9-L3-04-01-028 — Transform · Flow Along Surface — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-028`
- Authored script: `! _FlowAlongSrf`
- Menu: Transform ▸ Flow Along Surface (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

a point keeps its `(u, v)` on the source surface and lands on the target surface at the matching parameters, turned from the source normal onto the target normal: the parameters come from curve_surface_ops::closest_point_on_surface, the landing from surface_kernel::evaluate and the turn from surface_kernel::normal plus transform_session::align_rotation.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on the source surface: ` | point |

Every point is resolved through the CPlane captured at start.

## What it refuses

- fewer than two surfaces, or no object besides the two frames -> Error::Missing before the single prompt; a solid object -> Error::Unsupported("deformation of a solid")
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- the source frame is named by the picked point, as with the curve flow
- the parameters are matched proportionally between the two domains, so a source and a target with different aspect ratios flow with that stretch
- a solid is refused

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_028_flow_along_surface_maps_the_source_parameters_onto_the_target`
- `cargo test -p om9-core --lib deform`

The golden case: source plane over 10 x 10 z=0, target plane (10u, 10v, 10u), curve (1,1,0)->(3,1,0), source pick (9,9,0) -> the ends land on (1,1,1) and (3,1,3), one Undo entry.
