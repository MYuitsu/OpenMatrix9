# OM9-L3-04-01-018 — Transform · Symmetry — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-018`
- Authored script: `! _Symmetry`
- Menu: Transform ▸ Symmetry (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

the two points define the mirror plane; the row commits one affine reflection x -> x - 2 (n . (x - origin)) n as a placement, so every geometry kind is mirrored without a geometry-specific mirror routine, and the copy switch decides whether the source survives.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Mirror plane origin: ` | point |
| 2 | `Mirror plane normal point: ` | point |
| 3 | `Copy mirrored objects (0 = no, 1 = yes): ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a switch other than 0 or 1 -> Error::Invalid("0 or 1 switch") at its prompt; a degenerate plane (both points equal) -> Error::Invalid("mirror plane normal")
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the mirror is a placement transform, so a copy shares the source geometry instead of baking a new body
- the plane is defined by two typed or picked points; there is no plane pick and no preview

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_018_symmetry_mirrors_across_the_picked_plane`

The golden case: box 0..2, plane through (0,0,0) with normal +X -> the box centre moves to (-1,1,1) with 0 (one object) and to (-1,1,1) plus (1,1,1) with 1 (two objects), one Undo entry.
