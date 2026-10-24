# OM9-L3-03-01-007 — Viewport Tab Switch When Maximized

## Contract

- API: `activate_view + Workspace.maximized`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

view_tab_switches_active_and_maximized_view

## Negative oracle

switching to a non-visible view must not keep a stale maximized pane

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
