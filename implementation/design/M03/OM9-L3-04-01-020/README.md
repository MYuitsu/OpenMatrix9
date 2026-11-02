# OM9-L3-04-01-020 — Transform · Cage Edit — menu binding

- Owner: `OM9-L3-04-08-005` (M08), dispatched by `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-020`
- Authored script: `! _CageEdit`
- Menu: Transform ▸ Cage Editing ▸ Cage Edit
- Body: `crates/om9-app/src/session/transform/cage.rs`
- Kernel owner: `crates/om9-core/src/cage.rs`

## What the row does

one corner of the selected cage moves by the typed offset and every attached object carries its points with the cage's own trilinear field: each point keeps its parameter in the cage as it was before the edit (cage::Cage::invert) and lands on the same parameter of the cage after it (cage::displacement).

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Cage corner to move (1-8): ` | number |
| 2 | `Corner offset: ` | point |

Every point is resolved through the CPlane captured at start.

## What it refuses

- no cage in the selection and no object attached to one -> Error::Missing before the first question; two cages, or a selection spanning two cages -> Error::Invalid; a corner outside 1..8 -> Error::Invalid on the typed number
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- one corner per command: the row asks for one corner index and one offset, so a multi-corner move is several commands, each its own Undo entry
- only box cages (degree one, two poles per axis) are solvable in the portable kernel; anything else is refused by name
- points outside the cage follow its nearest boundary because the parameter is clamped into the unit domain

## Milestone ownership

Early implementation inside the M03 bridge of an owner that the milestone tree assigns to M08:
the cage kernel (`cage::box_cage`, `Cage::invert`, `cage::displacement`) and the `om9.cage`
extension slot carry the behaviour; M08 still owns its acceptance and the row keeps its own
manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu {row["test"]}`
- `cargo test -p om9-core --lib cage`

The golden case: {row["golden"]}.
