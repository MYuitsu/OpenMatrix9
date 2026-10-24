# OM9-L3-03-07-007 — Default Menu Layout Reset

## Contract

- API: `DockLayout reset/default JSON + Reset Menus`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

reset recreates the 300 px factory baseline after any placement kind

## Negative oracle

malformed schema/version or impossible size falls back atomically to the baseline

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
