# OM9-L3-04-01-013 — Transform · SmartFlowBreak — menu binding

- Owner: `OM9-L3-04-05-005` (Smartflow Break And Reconnect), M08
- Command id: `OM9-TRANSFORM-014`
- Menu: Transform ▸ Smart Flow family
- Body: `crates/om9-app/src/session/transform/flow.rs`
- Kernel owner: `crates/om9-core/src/deform.rs`

## What the row does

marks the flow link of every selected object BROKEN and bumps its revision, keeping the geometry exactly as the flow left it.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| — | (no prompt: the link is document state) | — |

## What it refuses

- a selection with no link, or with links that are already broken -> Error::Invalid("selection has no live Smart Flow link")
- A locked or hidden object never reaches the command.

## Limits of this row

- reconnect is the flow row itself: running Smart Flow again re-activates the link at the next revision
- a broken link keeps its recorded base and target, so breaking is reversible without re-picking

## Milestone ownership

Early M03 bridge of the M08 owner `OM9-L3-04-05-005`: the flow kernel and the `om9.flow`
extension slot carry the behaviour; M08 keeps its acceptance and the row keeps its manual gate.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_014_smart_flow_break_marks_the_link_broken`
- `cargo test -p om9-core --lib deform`

The golden case: curve flowed, then broken -> state BROKEN at revision 2 and the ends are unchanged.
