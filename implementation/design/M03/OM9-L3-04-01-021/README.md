# OM9-L3-04-01-021 — Transform · ReleaseFromCage — menu binding

- Owner: `OM9-L3-04-08-005` (M08), dispatched by `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-022`
- Authored script: `! _ReleaseFromCage`
- Menu: Transform ▸ Cage Editing ▸ ReleaseFromCage
- Body: `crates/om9-app/src/session/transform/cage.rs`
- Kernel owner: `crates/om9-core/src/cage.rs`

## What the row does

drops the cage back pointer from the selected objects, or from every object attached to a selected cage, and leaves the geometry exactly where the cage put it.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| — | (no prompt: the selection already says what to do) | — |

Every point is resolved through the CPlane captured at start.

## What it refuses

- a selection with no cage relation -> Error::Invalid("selection is not attached to a cage") and no question is asked
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- the cage object itself is kept: releasing is a relation change, not a delete, so the user decides when the cage box goes away
- released geometry is a plain object again: a later cage edit cannot reach it

## Milestone ownership

Early implementation inside the M03 bridge of an owner that the milestone tree assigns to M08:
the cage kernel (`cage::box_cage`, `Cage::invert`, `cage::displacement`) and the `om9.cage`
extension slot carry the behaviour; M08 still owns its acceptance and the row keeps its own
manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu {row["test"]}`
- `cargo test -p om9-core --lib cage`

The golden case: {row["golden"]}.
