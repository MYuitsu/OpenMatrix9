# OM9-L3-04-01-012 — Transform · SmartFlowRigid — menu binding

- Owner: `OM9-L3-04-05-003` (Smartflow Rigid Object Placement), M08
- Command id: `OM9-TRANSFORM-012`
- Menu: Transform ▸ Smart Flow family
- Body: `crates/om9-app/src/session/transform/flow.rs`
- Kernel owner: `crates/om9-core/src/deform.rs`

## What the row does

rigid transport of each selected object: its origin keeps its (u, v) on the base surface, lands on the same parameters of the target surface, and the frame turns from the base normal onto the target normal, so the geometry is never rewritten.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on the base surface: ` | point |

## What it refuses

- the same frame rule as Smart Flow: two surfaces and at least one object
- A locked or hidden object never reaches the command.

## Limits of this row

- rigid means the object frame moves: no geometry is deformed, so a large surface curvature difference is not compensated
- only the object origin is projected onto the base surface, not its whole bounding box

## Milestone ownership

Early M03 bridge of the M08 owner `OM9-L3-04-05-003`: the flow kernel and the `om9.flow`
extension slot carry the behaviour; M08 keeps its acceptance and the row keeps its manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_012_smart_flow_rigid_moves_the_frame`
- `cargo test -p om9-core --lib deform`

The golden case: solid at placement (2,2,0) -> origin lands on (2,2,2), the geometry is unchanged and the link records rigid = true.
