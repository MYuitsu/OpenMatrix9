# OM9-L3-04-01-032 — Transform · Stretch — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-032`
- Authored script: `! _Stretch`
- Menu: Transform ▸ Stretch (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`

## What the row does

Stretch is the one-direction scale published by gumball::scaling_along: the picked base point and direction define the axis and the factor scales along it only, so the other two directions keep their size.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Base point for stretch: ` | point |
| 2 | `Stretch direction: ` | point |
| 3 | `Stretch factor: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`, and the same rule is re-applied when the run commits.

## What it refuses

- a factor of 0 or below, or a direction with no length -> refused at its own prompt and the run continues on the same question
- A locked or hidden object never reaches the command: the selection refuses it first, and `begin_transform` repeats the check as defence in depth.

## Limits of this row

- the factor is uniform along the picked axis; the soft falloff of the original command is not available because no falloff deformer exists in the kernel

## Milestone ownership

early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_032_stretch_scales_along_the_picked_direction`

The golden case: box 0..2, base (0,0,0), direction (1,0,0), factor 2 -> the far corner moves to (4,2,2) and the centre to (2,1,1), one Undo entry.
