# OM9-L3-03-07-004 — Menu Scrolling And Icon Grid

## Contract

- API: `Main Menu paged grid (10 columns x 2 rows) + fixed page column`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

ten icons per row, two rows per page, page column anchored right at the 300 px dock

## Negative oracle

page index is clamped; icons never overflow the rail and page buttons cannot detach from the right edge

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
