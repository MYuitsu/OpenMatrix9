# OM9-L3-03-07-003 — Command Icon Tooltip And Rightclick Slot

## Contract

- API: `feature_icon tooltip + disabled/interaction slot`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

hover tooltip names the command and right-click slot remains inert when unregistered

## Negative oracle

unregistered right-click does not invoke the left-click command or mutate the document

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
