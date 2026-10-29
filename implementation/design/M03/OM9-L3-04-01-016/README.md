# OM9-L3-04-01-016 — Transform · Orient Perpendicular to Curve — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-016`
- Authored script: `! _OrientOnCrv _Pause _Pause _Pause _Copy=_Yes _Perpendicular`
- Menu: Transform ▸ Orient Perpendicular to Curve (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

The picked point snaps to the nearest station on the path curve (bisection on the
signed tangent projection, so the station is not limited by a sampling grid), and
`transform_session::align_transform` rotates each object so its construction-plane normal
follows the curve tangent while its centre lands on that station. Every target is solved
against its own centre, so several objects stand on the curve together.

The path curve is not transformed: it is the reference of the run.

## Selection rule

The path curve comes from the selection: exactly one selected curve is the path and every
other selected object is transformed along it. The row is refused before the first prompt
when that split is not possible.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on curve to orient at: ` | point |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- No curve in the selection → `Error::Missing("path curve selection")`.
- More than one curve in the selection → `Error::Invalid("one path curve")`.
- Only curves in the selection → `Error::Missing("select the objects and the path curve together")`; nothing would be oriented.
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- The path curve comes from the selection: this shell has no mid-command object pick, so the objects and the path are selected together.
- The typed point is snapped to the path; there is no editable snap distance and no preview.
- The rotation is the minimal one that maps the plane normal to the tangent, so the object's roll around the curve is not user-controlled.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_016_orient_perpendicular_to_curve_stands_the_object_on_the_path`
- `cargo test -p om9-app --lib -j 2` (unit tests for the station helpers and scalar rules)

The golden case: box centred at (1,1,0) on a path from (0,0,0) to (10,0,0), point (5,2,0) → the object stands at (5,0,0) with its normal along +X, the path does not move, one Undo entry.
