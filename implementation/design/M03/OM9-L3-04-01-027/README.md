# OM9-L3-04-01-027 — Transform · Flow Along Curve — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-027`
- Authored script: `! _Flow`
- Menu: Transform ▸ Flow Along Curve (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Deformer owner: `crates/om9-core/src/deform.rs`

## What the row does

every point keeps its station along the source path and rides the target path at the same share of the arc length, moving rigidly with the tangent there: the source station comes from curve_surface_ops::closest_point_on_curve, the share from curve_ops::length_between, and the frame turn from transform_session::align_rotation.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on the source curve: ` | point |

Every point is resolved through the CPlane captured at start.

## What it refuses

- fewer than two curves, or no object besides the two frames -> Error::Missing before the single prompt; a solid object -> Error::Unsupported("deformation of a solid") with the document and the Undo journal untouched
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- the source frame is named by the picked point, because the selection is a set: the curve nearest that point is the source and the other is the target
- the flow carries sections rigidly: it does not stretch geometry along the path and it has no twist control around the tangent
- a solid is refused, as with every deformer row

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_027_flow_along_curve_carries_geometry_onto_the_target_path`
- `cargo test -p om9-core --lib deform`

The golden case: source (0,0,0)->(10,0,0), target (0,0,0)->(0,10,0), 1 x 2 x 2 mesh at (5,1,0), source pick (9,0,0) -> the corners at stations 5 and 6 land on (-1,5,0) and (-1,6,0), one Undo entry.
