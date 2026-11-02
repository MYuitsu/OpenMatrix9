# OM9-L3-04-01-014 — Transform · SmartFlowPullDownCurves — menu binding

- Owner: `OM9-L3-04-05-006` (Smartflow Pulldown Reference Curves), M08
- Command id: `OM9-TRANSFORM-015`
- Menu: Transform ▸ Smart Flow family
- Body: `crates/om9-app/src/session/transform/flow.rs`
- Kernel owner: `crates/om9-core/src/deform.rs`

## What the row does

pulls the selected curves from the target surface down onto the base surface at the same proportional parameters (deform::pulldown_from_target), the inverse of the surface flow.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on the base surface: ` | point |

## What it refuses

- the same frame rule as Smart Flow: two surfaces and at least one curve
- A locked or hidden object never reaches the command.

## Limits of this row

- the curve is rebuilt from its mapped points, so a curved segment becomes a polyline of the mapped samples
- the pulldown is the exact inverse of the flow only at the parameters the projection recovers

## Milestone ownership

Early M03 bridge of the M08 owner `OM9-L3-04-05-006`: the flow kernel and the `om9.flow`
extension slot carry the behaviour; M08 keeps its acceptance and the row keeps its manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_015_smart_flow_pulldown_brings_curves_to_the_base`
- `cargo test -p om9-core --lib deform`

The golden case: curve (1,1,1)->(3,1,3) on the tilted plane, base plane picked -> the ends land on (1,1,0) and (3,1,0).
