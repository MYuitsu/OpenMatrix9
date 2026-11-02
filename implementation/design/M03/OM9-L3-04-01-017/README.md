# OM9-L3-04-01-017 — Transform · Orient on Curve — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-017`
- Authored script: `! _OrientOnCrv`
- Menu: Transform ▸ Orient on Curve (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

the picked point snaps to the nearest station on the selected path curve (curve_surface_ops::closest_point_on_curve) and each object's local X axis is aligned with the curve tangent at that station by transform_session::align_transform, so the object lies along the path; the perpendicular variant is row 016.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on curve to orient at: ` | point |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a selection without a curve -> Error::Missing("path curve selection") before the first prompt; two curves -> Error::Invalid("one path curve")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the path curve comes from the selection and is not transformed
- the rotation is the minimal one that maps X to the tangent, so the roll around the path is not user-controlled

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_017_orient_on_curve_follows_the_tangent`
- `cargo test -p om9-app --test transform_menu m03_04_01_reference_rows_refuse_a_selection_without_their_reference`

The golden case: path (0,0,0)->(10,0,0), box 0..2, point (5,2,0) -> the reference point snaps to (5,0,0) and the local X axis is (1,0,0), one Undo entry.
