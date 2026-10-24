# OM9-L3-03-07-006 — Main Menu Collapse Reorder And Float

## Contract

- API: `SectionState collapse/order/dock/float + panel ops`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

sections collapse, reorder, float and dock right while preserving owner state

## Negative oracle

invalid/stale float or dock target is repaired to a valid layout; no geometry/document mutation

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
