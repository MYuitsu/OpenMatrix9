# OM9-L3-03-07-002 — Matrix Main Menu Category Tabs

## Contract

- API: `category_palette + main category selection + top menu selection`
- Owner: `OM9-L2-03-07`
- Input: menu/dock/main-menu state only.
- Output: visible UI state and command invocation decisions; no geometry mutation.
- Errors: unknown names, stale targets and unavailable actions are refused explicitly.

## Positive oracle

all 16 reference menus expose their full authored icon list; category cards select the matching palette

## Negative oracle

unknown category/menu names are refused and cannot produce an empty titled palette

## Verification

- chrome_actions: 13 passed; left_stack_layers: 8 passed
- panel_ops: 33 passed; shell_docking: 10 passed
- Manual M01-UI review remains pending.
