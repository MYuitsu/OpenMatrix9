# OM9-L3-03-07-008 — Disabled Menu Action Visibility

## Contract

- API: `disabled authored actions + visible-but-non-invoking card`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

unsupported authored actions stay visible with disabled styling and do not invoke

## Negative oracle

disabled action cannot enable a backend command, mutate the document, or enter Undo

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
