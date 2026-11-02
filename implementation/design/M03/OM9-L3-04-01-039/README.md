# OM9-L3-04-01-039 — Transform · MoveUVN — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-039`
- Authored script: `! _MoveUVN`
- Menu: Transform ▸ MoveUVN (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform/rows.rs`
- Control-net owner: `crates/om9-core/src/deform.rs::move_uvn`

## What the row does

Every pole of every selected surface moves along the surface's own frame at that pole's
Greville parameter: the U tangent `S_u`, the V tangent `S_v` and the normal `S_u x S_v`, all
from `surface_kernel::derivatives`, with the parameters from
`surface_kernel::greville_parameter`. The frame is taken from the surface as it is before the
edit, so the three offsets are independent and one command is one step.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Move in the surface normal direction: ` | number (may be 0) |
| 2 | `Move along the U direction: ` | number (may be 0) |
| 3 | `Move along the V direction: ` | number (may be 0) |

A zero is meaningful because the three offsets are independent; the row refuses only the step
where all three are zero (`Error::Invalid("MoveUVN needs one non-zero offset")`).

## What it refuses

- An empty selection — `Error::Missing("select at least one surface to move")`.
- Any selected object that is not a surface, including a solid — `Error::Invalid("MoveUVN
  edits surfaces; select surfaces only")`. The row never silently skips part of the selection.
- A degenerate frame at a pole: a zero-length tangent or a surface whose two tangents are
  parallel is refused by `surface_kernel` instead of producing a zero direction.
- A locked or hidden object never reaches the command: the selection refuses it first.

## Limits of this row

- The offset is measured at each pole's **Greville parameter**, so on a curved surface the
  directions differ from pole to pole; the row is not a rigid translation of the control net.
- The frame is frozen before the edit: the row does not re-solve directions after the poles
  move, which is what a second application is for.
- The row edits surfaces only. A solid or a curve keeps its own commands.

## Milestone ownership

Early implementation inside the M03 bridge: the row keeps its M08 owner and its own manual
gate, and M08 remains responsible for its acceptance.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_039`
- `cargo test -p om9-core --lib deform`

The golden cases: the flat 10 x 10 plane lifts every pole to `z = 1` with a normal offset of 1,
and a U offset of 2 slides every pole two along `+X` while Y and Z stay put; a solid is refused
before the first question.
