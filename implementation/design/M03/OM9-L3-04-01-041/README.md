# OM9-L3-04-01-041 — Transform · Create Cage — menu binding

- Owner: `OM9-L3-04-08-005` (M08), dispatched by `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-041`
- Authored script: `! _Cage`
- Menu: Transform ▸ Cage Editing ▸ Create Cage
- Body: `crates/om9-app/src/session/transform/cage.rs`
- Kernel owner: `crates/om9-core/src/cage.rs`

## What the row does

inserts one cage object (a mesh box whose eight vertices are the cage corners) around the world bounds of the selection and writes a back pointer on every caged object; the corners the cage was created with are stored in its own extension slot.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| — | (no prompt: the selection already says what to do) | — |

Every point is resolved through the CPlane captured at start.

## What it refuses

- an object already attached to a cage -> Error::Invalid("object is already attached to a cage"); a selection whose bounds have no volume -> Error::Invalid("cage needs a box with volume")
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- the cage hugs the world bounds of the selection: there is no margin and no user-drawn cage
- the cage is an ordinary mesh object, so it renders, selects, moves and deletes like any other object; moving it does not re-capture the relation

## Milestone ownership

Early implementation inside the M03 bridge of an owner that the milestone tree assigns to M08:
the cage kernel (`cage::box_cage`, `Cage::invert`, `cage::displacement`) and the `om9.cage`
extension slot carry the behaviour; M08 still owns its acceptance and the row keeps its own
manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu {row["test"]}`
- `cargo test -p om9-core --lib cage`

The golden case: {row["golden"]}.
