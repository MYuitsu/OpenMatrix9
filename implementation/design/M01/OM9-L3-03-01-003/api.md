# OM9-L3-03-01-003 — Active Viewport Indicator

## Contract

- API: `Workspace.active + view title/tab hit rects`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

view_tab_restores_cmd_focus_and_preserves_numeric_draft; view_menu_restores_cmd_focus_and_preserves_numeric_draft

## Negative oracle

inactive pane cannot change active view without a valid tab/menu target

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
