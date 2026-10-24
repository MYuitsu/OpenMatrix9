# OM9-L3-03-07-001 — Matrix Left Sidebar Stack And Order

## Contract

- API: `DockLayout section order + sidebar section stack`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

left sidebar sections remain ordered and docked across 1280x720 and 1600x1000

## Negative oracle

a section cannot disappear, duplicate, or leave the left column after reset/reload

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
