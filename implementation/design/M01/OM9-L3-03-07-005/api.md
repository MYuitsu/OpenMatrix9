# OM9-L3-03-07-005 — Recent Command Icon History

## Contract

- API: `recent-command list + nine history slots`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

completed commands enter history once, newest first, capped at nine

## Negative oracle

empty/failed commands do not enter history; duplicate replay does not create extra slots

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
