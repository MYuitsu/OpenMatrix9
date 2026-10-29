# OM9-L3-04-01-011 — Transform · SmartFlow — menu binding

- Owner: `OM9-L3-04-05-002` (Smartflow Mapping Base To Target), M08
- Command id: `OM9-TRANSFORM-011`
- Menu: Transform ▸ Smart Flow family
- Body: `crates/om9-app/src/session/transform/flow.rs`
- Kernel owner: `crates/om9-core/src/deform.rs`

## What the row does

maps the selection from the base surface to the target surface (same proportional (u, v), turned from the base normal onto the target normal) and records the flow link that makes the relation live: base, target, rigid flag, revision and state.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Point on the base surface: ` | point |

## What it refuses

- fewer than two surfaces, or no object besides them -> Error::Missing before the single prompt
- A locked or hidden object never reaches the command.

## Limits of this row

- the base is named by a picked point because the selection is a set with no pick order
- the link is document state, not a live history node: a later change of the target surface does not recompute the flowed geometry until the row runs again

## Milestone ownership

Early M03 bridge of the M08 owner `OM9-L3-04-05-002`: the flow kernel and the `om9.flow`
extension slot carry the behaviour; M08 keeps its acceptance and the row keeps its manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_011_smart_flow_maps_from_the_base_surface`
- `cargo test -p om9-core --lib deform`

The golden case: 10 x 10 plane (base), plane (10u,10v,10u) (target), curve (1,1,0)->(3,1,0), base pick (9,9,0) -> the ends land on (1,1,1) and (3,1,3) and the object carries an ACTIVE om9.flow link at revision 1.
